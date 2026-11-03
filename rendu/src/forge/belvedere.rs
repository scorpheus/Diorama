//! Le belvédère de Galadriel, peint au vrai pixel de l'écran à la place de la forge quand elle
//! tient la scène : un grand oculus festonné sur le ciel de l'heure (la lune à sa phase, les
//! étoiles, la forêt d'argent et sa brume), le bassin de vision qui luit, les lanternes de ses
//! aides, un mur indigo semé d'étoiles d'or sur un lambris de pierre de lune, et le pilier où une
//! branche d'argent passe sur la cour.
//!
//! Ce qui ne bouge pas se peint une fois : le fond, gardé tant que l'heure (au quart), la saison,
//! le temps et les lanternes ne changent pas. À chaque image, seuls les pixels animés se
//! repeignent sur lui (l'eau, les éclats, les étoiles qui scintillent, les flammes, ce que les
//! moments font naître), et ce qu'un acteur a couvert se reprend du fond.
//!
//! Le dessin est pensé pour une cellule de 9 × 19 (un pixel de dessin, un pixel d'écran) ; une
//! autre cellule l'étire.

use std::f32::consts::PI;

use super::atelier::{SH, YARD};
use super::monde::{Activite, Envoi, Issue, ModeChat, Monde, Nouvelle, Saison, Saveur, Temps};
use super::pet::Eclairage;
use super::trace::{hex, mix, Rgb};

/// Les colonnes de forge que le belvédère couvre : la pièce, puis le pilier passé la porte.
pub const FIN: i32 = YARD + 2;
const DW: f32 = 9.0;
const DH: f32 = 19.0;
const LARGE: f32 = FIN as f32 * DW;
const SOL: f32 = 268.0;
const LAMBRIS: f32 = 198.0;
/// L'oculus : son centre, le bord de la vitre, le bord du cadre.
const OCX: f32 = 342.0;
const OCY: f32 = 114.0;
const RIN: f32 = 100.0;
const RFR: f32 = 111.0;
/// Le bassin : son centre, la hauteur de son bord, sa demi-largeur, et son pied, posé sur le sol devant le
/// mur (user, 02-11 : collé au pied du mur, « on dirait que c'est collé contre le mur »).
const BX: f32 = 116.0;
const BORD: f32 = 206.0;
const BW: f32 = 56.0;
const PIED: f32 = SOL + 22.0;
const PILIER: f32 = 478.0;
/// Les lanternes : où pend chacune (le haut de sa cage).
const LANTERNES: [(f32, f32); 4] = [(26.0, 58.0), (74.0, 42.0), (200.0, 70.0), (468.0, 52.0)];
/// Où dort le chat : sur le rebord du lambris, près du bassin.
const CHAT_DORT: (f32, f32) = (184.0, LAMBRIS - 24.0);

const PIERRE: Rgb = hex(0x9aa5cc);
const OR: Rgb = hex(0xe8c878);
const OR_TERNI: Rgb = hex(0xb08a3e);
const ENDUIT: Rgb = hex(0x20224a);
const VIGNE: Rgb = hex(0xb9c3e3);
const FEUILLES: [Rgb; 4] = [hex(0xf6dc8a), hex(0xe2b95a), hex(0xc9963c), hex(0xfff0b0)];
const CYAN: Rgb = hex(0x4fe6ff);

fn fois(c: Rgb, k: f32) -> Rgb {
    [c[0] * k, c[1] * k, c[2] * k]
}

fn plus(a: Rgb, b: Rgb) -> Rgb {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Un hasard fixe pour deux entiers, de 0 à 1.
fn h2(a: i32, b: i32) -> f32 {
    let mut x = (a as u32).wrapping_mul(0x9E37_79B1) ^ (b as u32).wrapping_mul(0x85EB_CA77).rotate_left(13);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^= x >> 15;
    (x & 0xFFFF) as f32 / 65535.0
}

#[inline]
fn pack(c: Rgb) -> u32 {
    let b = |v: f32| (v + 0.5).clamp(0.0, 255.0) as u32;
    0xFF00_0000 | (b(c[0]) << 16) | (b(c[1]) << 8) | b(c[2])
}

#[inline]
fn unpack(p: u32) -> Rgb {
    [((p >> 16) & 255) as f32, ((p >> 8) & 255) as f32, (p & 255) as f32]
}

/// Le ciel de l'heure, vu par l'oculus, et ce qu'il fait à la pièce.
#[derive(Clone, Copy)]
struct Ciel {
    haut: Rgb,
    bas: Rgb,
    /// 1 la nuit, 0 le jour
    nuit: f32,
    crepuscule: f32,
    /// la lumière qui baigne la pièce, en facteur par canal
    ambiance: Rgb,
    /// la lune la nuit, le soleil le jour : où, et sa couleur
    astre: (f32, f32),
    lumiere_astre: Rgb,
    /// l'astre est la lune
    lune: bool,
    /// la force du rai que l'astre jette par l'oculus (0 à 1), et sa direction (vers le bas)
    rai: f32,
    dir: (f32, f32),
    /// le ciel qui lave le mur autour de l'oculus
    lavis: Rgb,
    /// où en sont les nuages, en pixels de dessin
    derive: f32,
    orage: bool,
}

fn ciel(heure: f32, orage: bool, phase: f32) -> Ciel {
    let h = heure.rem_euclid(24.0);
    let nuit = if h < 5.0 || h >= 21.5 {
        1.0
    } else if h < 7.0 {
        1.0 - (h - 5.0) / 2.0
    } else if h >= 18.5 {
        (h - 18.5) / 3.0
    } else {
        0.0
    };
    // le doré commence quand le soleil baisse (17 h du ciel), culmine au coucher, s'éteint dans la nuit
    let crepuscule = ((1.0 - (h - 19.4).abs() / 2.4).max(0.0) + (1.0 - (h - 6.8).abs() / 1.8).max(0.0)).min(1.0);
    let jour_haut = hex(0x5d8fd6);
    let jour_bas = hex(0xbfd8f0);
    let soir_haut = hex(0x3b3a7a);
    let soir_bas = hex(0xf0a070);
    let nuit_haut = hex(0x060a24);
    let nuit_bas = hex(0x273a7a);
    let mut haut = mix(mix(jour_haut, soir_haut, crepuscule), nuit_haut, nuit);
    let mut bas = mix(mix(jour_bas, soir_bas, crepuscule), nuit_bas, nuit);
    // la lumière du ciel dans la pièce, avant le rai : bleutée le jour, rose au crépuscule, d'encre
    // la nuit ; l'intérieur reste plus sombre que dehors, c'est le rai qui fait le jour
    let mut ambiance = mix(mix([1.45, 1.42, 1.48], [1.05, 0.76, 0.78], crepuscule), [0.15, 0.165, 0.33], nuit);
    if orage {
        haut = mix(haut, hex(0x2a2e3a), 0.6);
        bas = mix(bas, hex(0x4a5060), 0.6);
        ambiance = fois(ambiance, 0.75);
    }
    // le soleil court de gauche à droite au-dessus de sa capuche, du lever (7 h du ciel) au coucher
    // (19 h 30) ; la lune, de droite à gauche, du coucher au lever
    let (p, lune) = if (7.0..19.5).contains(&h) { ((h - 7.0) / 12.5, false) } else { (((h - 19.5).rem_euclid(24.0) / 11.5).clamp(0.0, 1.0), true) };
    let astre = if lune { (OCX + 64.0 - 128.0 * p, 64.0 - 26.0 * (PI * p).sin()) } else { (OCX - 64.0 + 128.0 * p, 62.0 - 24.0 * (PI * p).sin()) };
    let lumiere_astre = if lune { hex(0xa8bcff) } else { mix(hex(0xfff0d0), hex(0xffa860), crepuscule) };
    // le rai : plein au midi, doré et bas vers le coucher ; la lune le donne à sa phase
    let haut_astre = (PI * p).sin().max(0.0);
    let eclairee = 0.5 - 0.5 * (2.0 * PI * phase).cos();
    let mut rai = if lune { 0.7 * eclairee * haut_astre.powf(0.5) * nuit } else { haut_astre.powf(0.6) * (1.0 - 0.7 * nuit) };
    if orage {
        rai *= 0.12;
    }
    let dx = -(astre.0 - OCX) / 64.0 * 0.62;
    let n = dx.hypot(1.0);
    let lavis = fois(mix(bas, haut, 0.3), 1.0 - 0.5 * nuit);
    Ciel { haut, bas, nuit, crepuscule, ambiance, astre, lumiere_astre, lune, rai, dir: (dx / n, 1.0 / n), lavis, derive: h * 9.0, orage }
}

/// Le rai de l'astre par l'oculus : ce que le point (u, v) en reçoit, de 0 à 1, ses stries (les
/// festons de la vitre découpent la lumière) et son déclin compris ; rien au-dessus de l'oculus.
fn rai(u: f32, v: f32, c: &Ciel) -> f32 {
    if c.rai <= 0.0 {
        return 0.0;
    }
    let (dx, dy) = c.dir;
    let (wx, wy) = (u - OCX, v - OCY);
    // le long du rai, depuis le centre de l'oculus ; et de travers, signé
    let s = wx * dx + wy * dy;
    if s <= 0.0 {
        return 0.0;
    }
    let e = wx * dy - wy * dx;
    let bord = RIN * 0.86 * (1.0 + s / 260.0) - e.abs();
    let k = ((bord + 10.0) / 22.0).clamp(0.0, 1.0);
    if k <= 0.0 {
        return 0.0;
    }
    let k = k * k * (3.0 - 2.0 * k);
    let stries = 0.62 + 0.38 * (0.5 + 0.5 * (e * 0.14 + 0.7).sin()) * (0.55 + 0.45 * (e * 0.053 - 1.1).cos());
    // il naît au bas de la vitre, et s'use en descendant
    let naitre = ((s - 40.0) / 60.0).clamp(0.0, 1.0);
    c.rai * k * stries * (-s / 520.0).exp() * (0.25 + 0.75 * naitre)
}

/// La phase de la lune, de 0 (nouvelle) à 1, pour un jour de 2026 : la nouvelle lune du
/// 18 janvier, et le mois synodique.
fn phase_lune(mois: i32, jour: i32) -> f32 {
    const AVANT: [i32; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let j = AVANT[(mois.clamp(1, 12) - 1) as usize] + jour;
    ((j as f32 - 18.8) / 29.53).rem_euclid(1.0)
}

/// Les mailles du festonnage : seize lobes au bord de la vitre.
#[inline]
fn bord_vitre(u: f32, v: f32) -> f32 {
    let a = (v - OCY).atan2(u - OCX);
    RIN - 5.0 * (8.0 * a).cos().abs()
}

#[inline]
fn dans_vitre(u: f32, v: f32) -> bool {
    let r = (u - OCX).hypot(v - OCY);
    r < RIN && r < bord_vitre(u, v)
}

/// Les lanternes d'une image : allumée (et sa couleur), ou éteinte.
type Lanternes = [Option<Rgb>; 4];

#[derive(Clone, Copy, PartialEq)]
struct Cle {
    w: usize,
    h: usize,
    sx: u32,
    sy: u32,
    quart: i32,
    saison: u8,
    temps: u8,
    lanternes: [u32; 4],
    sommeil: bool,
}

/// Ce que le belvédère garde d'une image à l'autre.
#[derive(Default)]
pub struct Belvedere {
    cle: Option<Cle>,
    /// le fond, à la taille de la région (FIN colonnes × la hauteur de la scène)
    fond: Vec<u32>,
    rw: usize,
    rh: usize,
    /// la lumière posée sur les acteurs, par pixel de forge (FIN × SH)
    grille: Vec<[f32; 3]>,
    ciel: Option<Ciel>,
    /// les pixels de la région peints par-dessus le fond à l'image d'avant
    traces: Vec<u32>,
    /// les rectangles (en pixels d'écran) qu'un acteur a couverts, à reprendre du fond
    sales: Vec<(i32, i32, i32, i32)>,
    /// où la région tombe dans l'image peinte, pour la passe devant elle
    cadre: Option<Cadre>,
}

/// Où la région tombe dans l'image : gardé de `peindre` pour `devant`.
#[derive(Clone, Copy)]
struct Cadre {
    pw: usize,
    ph: usize,
    ox: i32,
    visible: (i32, i32),
    fin_image: i32,
    sx: f32,
    sy: f32,
}

/// Combien de secondes dure une vision : elle monte de l'eau, flotte, se défait.
pub const VISION_S: f32 = 9.0;

/// Une vision dans la vasque : l'habitant d'une session qui écrit à Galadriel, ou à qui elle écrit, monte de
/// l'eau, pâle et cyan, flotte au-dessus d'elle quelques secondes, puis se défait (user, 02-11 : « je prends
/// le 3, la vision dans la vasque »). L'appli la pose ; le monde du mod n'en sait rien. Posée quand le
/// belvédère n'est pas à l'écran, elle attend qu'il y soit (`VISION_ATTENTE` au plus).
pub struct Vision {
    /// ses images, `w` × `h` en ARGB (alpha nul : transparent), qui tournent toutes les 150 ms
    pub images: Vec<Vec<u32>>,
    pub w: u32,
    pub h: u32,
    /// quand l'appli l'a posée
    pub posee: std::time::Instant,
    /// quand elle a commencé à monter : à la première image peinte après sa pose
    pub nee: Option<std::time::Instant>,
}

/// Combien de temps une vision attend que le belvédère soit à l'écran : au-delà, elle est oubliée.
pub const VISION_ATTENTE: std::time::Duration = std::time::Duration::from_secs(600);

impl Vision {
    /// Son âge en secondes depuis qu'elle monte ; `None` : pas encore montée.
    pub fn age(&self) -> Option<f32> {
        self.nee.map(|n| n.elapsed().as_secs_f32())
    }
}

/// Où en est Lara, la voix de Galadriel, sur la carte graphique (l'appli le dit : `voix::EtatLara`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Lara {
    /// pas de Lara pour cette appli : pas de dragon
    #[default]
    Aucune,
    /// libérée : la carte est rendue, le dragon est parti
    Liberee,
    /// elle se charge : le dragon est là, éveillé
    SeCharge,
    /// chargée, elle tient la carte : le dragon dort sur son or
    Prete,
}

/// Combien de secondes dure un vol du dragon, quand il part ou qu'il revient.
pub const DRAGON_VOL_S: f32 = 3.5;

/// Le dragon de Lara (user, 03-11 : « le dragon de la Lara, oui ») : quand Lara occupe la carte graphique, un petit
/// dragon dort sur un tas d'or dans le coin du belvédère ; quand on la libère, il s'envole, et l'or reste seul ;
/// quand on la reprend, il revient. L'appli dit où en est Lara (`suivre`) ; le monde du mod n'en sait rien.
#[derive(Clone, Copy, Debug)]
pub struct Dragon {
    pub lara: Lara,
    /// où elle en était avant le dernier changement
    pub avant: Lara,
    /// quand elle a changé
    pub depuis: std::time::Instant,
}

impl Default for Dragon {
    fn default() -> Self {
        Dragon { lara: Lara::Aucune, avant: Lara::Aucune, depuis: std::time::Instant::now() }
    }
}

/// Ce que le dragon fait à un instant.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Geste {
    Absent,
    Dort,
    Eveille,
    /// il part, ou il revient (`part` faux), à `f` de son vol (de 0 à 1)
    Vole { part: bool, f: f32 },
    /// parti : l'or reste seul
    Parti,
}

impl Dragon {
    /// Lara a peut-être changé.
    pub fn suivre(&mut self, lara: Lara) {
        if lara != self.lara {
            self.avant = self.lara;
            self.lara = lara;
            self.depuis = std::time::Instant::now();
        }
    }

    /// Ce que le dragon fait, `age` secondes après le dernier changement de Lara.
    pub fn geste(&self, age: f32) -> Geste {
        let la = |l: Lara| matches!(l, Lara::SeCharge | Lara::Prete);
        let f = age / DRAGON_VOL_S;
        match self.lara {
            Lara::Aucune => Geste::Absent,
            Lara::Liberee if la(self.avant) && f < 1.0 => Geste::Vole { part: true, f },
            Lara::Liberee => Geste::Parti,
            _ if !la(self.avant) && f < 1.0 => Geste::Vole { part: false, f },
            Lara::SeCharge => Geste::Eveille,
            Lara::Prete => Geste::Dort,
        }
    }
}

/// Ce qu'on demande au belvédère pour une image.
pub struct Vue<'a> {
    pub monde: &'a Monde,
    pub activite: Activite,
    pub tick: i64,
    /// la première colonne de forge montrée, et combien
    pub x0: i32,
    pub cols: i32,
    pub cw: i32,
    pub ch: i32,
    /// où se tient Galadriel, en pixels de forge (le milieu de sa place)
    pub elle: Option<f32>,
    /// la vision qui monte de la vasque, s'il y en a une
    pub vision: Option<&'a Vision>,
    /// le dragon de Lara
    pub dragon: Option<&'a Dragon>,
}

impl Belvedere {
    /// Ce qu'un acteur a couvert : repris du fond à l'image suivante.
    pub fn salir(&mut self, r: (i32, i32, i32, i32)) {
        self.sales.push(r);
    }

    /// La lumière sur un acteur, à la façon de lumiere.rs : `w` pixels de forge depuis `x0`.
    pub fn eclairage(&self, x0: i32, w: i32) -> Option<Eclairage> {
        if self.grille.is_empty() {
            return None;
        }
        let mut k = Vec::with_capacity((w * SH) as usize);
        for y in 0..SH {
            for x in x0..x0 + w {
                k.push(self.grille[(y * FIN + x.clamp(0, FIN - 1)) as usize]);
            }
        }
        Some(Eclairage { x0, y0: 0, w, h: SH, k })
    }

    /// Peint le belvédère dans `pixels` (`pw` × `ph`) : en entier si `neuve` ou que le fond
    /// change, sinon ce qui bouge. Rend la zone peinte, et ce qui a débordé hors de la région
    /// (sur la cour : ses cellules sont à repeindre).
    pub fn peindre(&mut self, pixels: &mut [u32], pw: usize, ph: usize, v: &Vue, neuve: bool) -> (Option<(i32, i32, i32, i32)>, Option<(i32, i32, i32, i32)>) {
        let (sx, sy) = (v.cw as f32 / DW, v.ch as f32 / DH);
        let (rw, rh) = ((FIN * v.cw) as usize, (v.ch * SH / 2) as usize);
        let w = v.monde;
        let sommeil = v.activite == Activite::Sleep;
        let lanternes = lanternes(w, sommeil);
        let code = |l: &Option<Rgb>| l.map_or(0, |c| pack(c) & 0xFF_FFFF | 1 << 24);
        let cle = Cle {
            w: rw,
            h: rh,
            sx: sx.to_bits(),
            sy: sy.to_bits(),
            quart: (w.heure_ciel() * 4.0).floor() as i32,
            saison: w.saison() as u8,
            temps: w.weather.map_or(0, |t| t as u8 + 1),
            lanternes: [code(&lanternes[0]), code(&lanternes[1]), code(&lanternes[2]), code(&lanternes[3])],
            sommeil,
        };
        let ox = -v.x0 * v.cw;
        let mut entier = neuve;
        if self.cle != Some(cle) {
            let ciel = ciel(w.heure_ciel() as f32, w.weather == Some(Temps::Storm), phase_lune(w.month, w.day));
            self.fond = fond(rw, rh, sx, sy, &ciel, w, &lanternes, sommeil);
            self.grille = grille(&ciel, &lanternes, sommeil);
            self.ciel = Some(ciel);
            self.rw = rw;
            self.rh = rh;
            self.cle = Some(cle);
            entier = true;
        }
        let ciel = self.ciel.unwrap_or_else(|| ciel(w.heure_ciel() as f32, false, phase_lune(w.month, w.day)));
        // la part visible de la région
        let (xa, xb) = (ox.max(0), (ox + rw as i32).min(pw as i32).min(v.cols * v.cw));
        let yb = rh.min(ph) as i32;
        let mut zone: Option<(i32, i32, i32, i32)> = None;
        let etendre = |z: &mut Option<(i32, i32, i32, i32)>, r: (i32, i32, i32, i32)| {
            *z = Some(z.map_or(r, |a| (a.0.min(r.0), a.1.min(r.1), a.2.max(r.2), a.3.max(r.3))));
        };
        if xa < xb && yb > 0 {
            if entier {
                for y in 0..yb as usize {
                    let s = &self.fond[y * rw + (xa - ox) as usize..y * rw + (xb - ox) as usize];
                    pixels[y * pw + xa as usize..y * pw + xb as usize].copy_from_slice(s);
                }
                etendre(&mut zone, (xa, 0, xb, yb));
            } else {
                // ce qui a bougé à l'image d'avant, et ce que les acteurs ont couvert
                for &i in &self.traces {
                    let (rx, ry) = ((i as usize % rw) as i32, (i as usize / rw) as i32);
                    let x = rx + ox;
                    if x >= xa && x < xb && ry < yb {
                        pixels[ry as usize * pw + x as usize] = self.fond[i as usize];
                        etendre(&mut zone, (x, ry, x + 1, ry + 1));
                    }
                }
                for &(l, t, r, b) in &self.sales {
                    let (l, r) = (l.max(xa), r.min(xb));
                    let (t, b) = (t.max(0), b.min(yb));
                    if l >= r || t >= b {
                        continue;
                    }
                    for y in t as usize..b as usize {
                        let s = &self.fond[y * rw + (l - ox) as usize..y * rw + (r - ox) as usize];
                        pixels[y * pw + l as usize..y * pw + r as usize].copy_from_slice(s);
                    }
                    etendre(&mut zone, (l, t, r, b));
                }
            }
        }
        self.sales.clear();
        self.traces.clear();
        let cadre = Cadre { pw, ph: yb.max(0) as usize, ox, visible: (xa, xb), fin_image: (v.cols * v.cw).min(pw as i32), sx, sy };
        self.cadre = Some(cadre);
        let (z, dehors) = self.passe(pixels, cadre, |p| anime_derriere(p, v, &ciel, &lanternes));
        if let Some(z) = z {
            etendre(&mut zone, z);
        }
        (zone, dehors)
    }

    /// Ce qui passe devant elle, peint après les acteurs : la zone peinte et ce qui a débordé
    /// sur la cour. Repris du fond à l'image suivante, comme le reste.
    pub fn devant(&mut self, pixels: &mut [u32], v: &Vue) -> (Option<(i32, i32, i32, i32)>, Option<(i32, i32, i32, i32)>) {
        match self.cadre {
            Some(cadre) => self.passe(pixels, cadre, |p| anime_devant(p, v)),
            None => (None, None),
        }
    }

    /// Une passe de pinceau sur l'image, qui ajoute ses pixels aux traces.
    fn passe(&mut self, pixels: &mut [u32], c: Cadre, f: impl FnOnce(&mut Pinceau)) -> (Option<(i32, i32, i32, i32)>, Option<(i32, i32, i32, i32)>) {
        let mut traces = std::mem::take(&mut self.traces);
        let mut p = Pinceau {
            px: pixels,
            pw: c.pw,
            ph: c.ph,
            ox: c.ox,
            rw: self.rw,
            visible: c.visible,
            fin_image: c.fin_image,
            traces: &mut traces,
            zone: None,
            dehors: None,
            sx: c.sx,
            sy: c.sy,
            grille: &self.grille,
            muet: false,
        };
        f(&mut p);
        let r = (p.zone, p.dehors);
        self.traces = traces;
        r
    }
}

/// Les lanternes d'un monde : une par aide, à sa couleur ; la première, d'un or chaud, quand le
/// soir tombe ou que la lanterne est allumée. Assoupie, elle les laisse à demi.
fn lanternes(w: &Monde, sommeil: bool) -> Lanternes {
    let mut l: Lanternes = [None; 4];
    let chaude = hex(0xffb85c);
    if w.lantern_lit || !(7.0..18.5).contains(&w.heure_ciel()) {
        l[0] = Some(chaude);
    }
    let libres: Vec<usize> = [2usize, 1, 3, 0].into_iter().filter(|&i| l[i].is_none()).collect();
    let mut libre = libres.into_iter();
    for a in w.apprentices.iter().filter(|a| a.leaving.is_none()).take(4) {
        let c = super::accessoires::PALETTE.get(a.colour.0 as usize).copied().flatten().map_or(chaude, |c| mix(hex(c), hex(0xfff2d0), 0.35));
        if let Some(i) = libre.next() {
            l[i] = Some(c);
        }
    }
    if sommeil {
        for c in l.iter_mut().flatten() {
            *c = fois(*c, 0.55);
        }
    }
    l
}

/// La lumière d'un point de la pièce, en facteur par canal.
fn lumiere(u: f32, v: f32, ciel: &Ciel, lanternes: &Lanternes, sommeil: bool) -> Rgb {
    let point = |l: &mut Rgb, x: f32, y: f32, c: Rgb, p: f32, r: f32| {
        let d2 = ((u - x) * (u - x) + (v - y) * (v - y)) / (r * r);
        let k = p / (1.0 + d2) / 255.0;
        l[0] += c[0] * k;
        l[1] += c[1] * k;
        l[2] += c[2] * k;
    };
    // le jour du ciel : plein autour de l'oculus, plus sombre aux coins, sous le plafond, au pied
    // du mur et sous le bassin
    let loin = ((u - OCX) / 320.0).powi(2) + ((v - OCY - 40.0) / 240.0).powi(2);
    let mut creux = (0.6 + 0.4 * (-loin).exp()) * (0.74 + 0.26 * ((v - 4.0) / 46.0).clamp(0.0, 1.0));
    if v < SOL {
        creux *= 1.0 - 0.3 * (1.0 - (SOL - v) / 12.0).clamp(0.0, 1.0);
        // derrière la coupe, le mur est à l'ombre de la vasque, un peu décalée : elle est devant lui
        if v > BORD - 2.0 {
            let e = ((u - BX - 6.0) / (BW + 6.0)).powi(2) + ((v - BORD - 30.0) / 40.0).powi(2);
            creux *= 1.0 - 0.32 * (1.0 - e).clamp(0.0, 1.0);
        }
    } else {
        creux *= 1.0 - 0.6 * (-((u - BX - 4.0) / 40.0).powi(2) - ((v - PIED + 1.0) / 5.0).powi(2)).exp();
    }
    let mut l = fois(ciel.ambiance, creux);
    // le ciel par l'oculus lave le mur autour
    let r = (u - OCX).hypot(v - OCY);
    let lavis = (-((r - RIN).max(0.0) / 30.0).powi(2)).exp() * 0.5;
    l = plus(l, fois(ciel.lavis, lavis / 255.0));
    // le rai de l'astre : il effleure le mur, il s'étale au sol
    let k = rai(u, v, ciel) * if v >= SOL { 1.5 } else { 0.5 };
    l = plus(l, fois(ciel.lumiere_astre, k / 255.0));
    point(&mut l, BX, BORD - 6.0, CYAN, if sommeil { 0.6 } else { 1.0 }, 62.0);
    if v >= SOL {
        let k = (-((u - BX) / 70.0).powi(2) - ((v - PIED - 4.0) / 12.0).powi(2)).exp() * if sommeil { 0.18 } else { 0.3 };
        l = plus(l, fois(CYAN, k / 255.0));
    }
    for (i, c) in lanternes.iter().enumerate() {
        if let Some(c) = c {
            let (x, y) = LANTERNES[i];
            point(&mut l, x, y + 12.0, *c, 0.9, 34.0);
        }
    }
    l
}

/// La lumière sur les acteurs, par pixel de forge : un peu de la sienne, beaucoup de la pièce.
fn grille(ciel: &Ciel, lanternes: &Lanternes, sommeil: bool) -> Vec<[f32; 3]> {
    let mut g = Vec::with_capacity((FIN * SH) as usize);
    for y in 0..SH {
        for x in 0..FIN {
            let l = lumiere((x as f32 + 0.5) * DW, (y as f32 + 0.5) * DH / 2.0, ciel, lanternes, sommeil);
            g.push([(0.6 + 0.55 * l[0]).min(1.25), (0.6 + 0.55 * l[1]).min(1.25), (0.62 + 0.55 * l[2]).min(1.25)]);
        }
    }
    g
}

/// Le fond : tout ce qui ne bouge pas, éclairé.
#[allow(clippy::too_many_arguments)]
fn fond(rw: usize, rh: usize, sx: f32, sy: f32, ciel: &Ciel, w: &Monde, lanternes: &Lanternes, sommeil: bool) -> Vec<u32> {
    let n = rw * rh;
    // la couleur sous pleine lumière, et ce qui luit de soi-même
    let mut alb = vec![[0f32; 3]; n];
    let mut emi = vec![[0f32; 3]; n];
    let saison = w.saison();
    let phase = phase_lune(w.month, w.day);
    for y in 0..rh {
        let v = (y as f32 + 0.5) / sy;
        for x in 0..rw {
            let u = (x as f32 + 0.5) / sx;
            let i = y * rw + x;
            let (a, e) = pixel_fond(u, v, ciel, saison, phase, w.weather == Some(Temps::Rainbow));
            alb[i] = a;
            emi[i] = e;
        }
    }
    {
        let mut t = Trace { alb: &mut alb, emi: &mut emi, rw, rh, sx, sy };
        decor(&mut t, lanternes);
    }
    let mut col = vec![[0f32; 3]; n];
    for y in 0..rh {
        let v = (y as f32 + 0.5) / sy;
        for x in 0..rw {
            let u = (x as f32 + 0.5) / sx;
            let i = y * rw + x;
            let a = alb[i];
            let mut c = if a == [0.0; 3] {
                emi[i]
            } else {
                let l = lumiere(u, v, ciel, lanternes, sommeil);
                plus([a[0] * l[0], a[1] * l[1], a[2] * l[2]], emi[i])
            };
            if !dans_vitre(u, v) {
                // l'air que le rai traverse, poudré de sa lumière
                let k = rai(u, v, ciel);
                if k > 0.0 {
                    c = plus(c, fois(ciel.lumiere_astre, 0.21 * k));
                }
                // le halo d'une lanterne allumée
                for (j, l) in lanternes.iter().enumerate() {
                    if let Some(l) = l {
                        let (lx, ly) = LANTERNES[j];
                        let d = (u - lx).hypot(v - ly - 11.0) / 30.0;
                        if d < 1.0 {
                            c = plus(c, fois(*l, 0.2 * (1.0 - d) * (1.0 - d)));
                        }
                    }
                }
            }
            col[i] = c;
        }
    }
    // le sol de pierre de lune, poli : il rend un peu du lambris, du bassin et du rai
    let sol = (SOL * sy).round() as usize;
    let profond = 34.0 * sy;
    for y in sol..rh {
        let miroir = 2 * sol as i64 - 1 - y as i64;
        if miroir < 0 {
            continue;
        }
        let f = 0.2 * (1.0 - (y - sol) as f32 / profond).max(0.0);
        for x in 0..rw {
            let u = (x as f32 + 0.5) / sx;
            if u >= PILIER || bassin(u, (y as f32 + 0.5) / sy).is_some() {
                continue;
            }
            let m = col[miroir as usize * rw + x];
            let c = &mut col[y * rw + x];
            *c = plus(*c, fois(m, f));
        }
    }
    col.into_iter().map(|c| pack(doux(c))).collect()
}

/// Un genou doux au-dessus de 200 : la lumière forte blanchit sans se figer d'un coup.
fn doux(c: Rgb) -> Rgb {
    let k = |x: f32| if x <= 200.0 { x } else { 200.0 + 55.0 * (1.0 - (-(x - 200.0) / 55.0).exp()) };
    [k(c[0]), k(c[1]), k(c[2])]
}

/// Un pixel du fond avant les objets posés : le mur, le lambris, le sol, l'oculus et son ciel,
/// le bassin, le pilier.
fn pixel_fond(u: f32, v: f32, ciel: &Ciel, saison: Saison, phase: f32, arc: bool) -> (Rgb, Rgb) {
    let rien = [0.0; 3];
    // le pilier, à la porte
    if u >= PILIER {
        let t = (u - PILIER) / (LARGE - PILIER);
        let mut c = fois(PIERRE, 0.5 + 0.6 * (PI * (0.15 + 0.8 * t)).sin());
        if (22.0..28.0).contains(&v) || v >= SOL - 8.0 && v < SOL {
            c = fois(PIERRE, 1.05);
        }
        if v >= SOL {
            c = fois(PIERRE, 0.62);
        }
        return (c, rien);
    }
    // l'oculus
    let r = (u - OCX).hypot(v - OCY);
    if r < RFR {
        let bord = bord_vitre(u, v);
        if r < bord {
            return (rien, ciel_px(u, v, ciel, saison, phase, arc));
        }
        // le cadre : pierre biseautée, filet d'or au feston, ombre au bord
        let a = (v - OCY).atan2(u - OCX);
        let mut c = fois(PIERRE, if a.sin() < -0.2 { 0.8 } else { 0.72 });
        if r < bord + 1.5 {
            c = OR;
        } else if r >= RFR - 1.5 {
            c = fois(PIERRE, 0.45);
        }
        return (c, rien);
    }
    // le bassin
    if let Some(p) = bassin(u, v) {
        return p;
    }
    if v < 14.0 {
        // la frise : linteau de pierre de lune et volute d'or
        let yv = 7.0 + 3.2 * (u / 9.0).sin();
        let c = if (v - yv).abs() < 0.6 {
            OR
        } else if v < 2.0 {
            fois(PIERRE, 0.95)
        } else if v >= 12.0 {
            fois(PIERRE, 0.45)
        } else {
            fois(PIERRE, 0.75)
        };
        return (c, rien);
    }
    if v < LAMBRIS {
        // l'enduit indigo, son grain, et le pochoir : un treillis d'étoiles d'or terni
        let t = (v - 14.0) / (LAMBRIS - 14.0);
        let grain = h2((u / 3.0) as i32, (v / 3.0) as i32);
        let mut c = fois(ENDUIT, (0.92 + 0.12 * t) * (0.95 + 0.08 * grain));
        let j = ((v - 26.0) / 16.0).round();
        let yj = 26.0 + 16.0 * j;
        if j >= 0.0 && yj < LAMBRIS - 6.0 {
            let decal = if (j as i32) % 2 == 1 { 14.0 } else { 0.0 };
            let xk = ((u - decal) / 28.0).round() * 28.0 + decal;
            let (dx, dy) = ((u - xk).abs(), (v - yj).abs());
            if dx < 0.5 && dy < 0.5 {
                c = fois(OR_TERNI, 0.95);
            } else if dx + dy < 1.5 {
                c = mix(c, OR_TERNI, 0.45);
            }
        }
        return (c, rien);
    }
    if v < SOL {
        // le lambris : la cimaise, des panneaux cernés d'un filet d'or, un croissant au milieu
        let dv = v - LAMBRIS;
        if dv < 4.0 {
            let k = if dv < 1.0 { 1.2 } else if dv >= 3.0 { 0.4 } else { 0.95 };
            return (fois(PIERRE, k), rien);
        }
        let pu = (u - 4.0).rem_euclid(54.0);
        let (xa, xb, ya, yb) = (4.0, 48.0, LAMBRIS + 10.0, SOL - 7.0);
        if (u - 4.0) / 54.0 < 8.0 && pu >= xa && pu < xb && v >= ya && v < yb {
            let (ix, iy) = (pu - xa, v - ya);
            let (lw, lh) = (xb - xa, yb - ya);
            let mut c = fois(PIERRE, 0.62);
            let mut e = rien;
            if iy < 1.0 || ix < 1.0 {
                c = fois(PIERRE, 0.35);
            } else if iy >= lh - 1.0 || ix >= lw - 1.0 {
                c = fois(PIERRE, 0.85);
            } else if ((iy - 4.0).abs() < 0.5 || (iy - (lh - 5.0)).abs() < 0.5) && ix >= 4.0 && ix < lw - 4.0
                || ((ix - 4.0).abs() < 0.5 || (ix - (lw - 5.0)).abs() < 0.5) && iy >= 4.0 && iy < lh - 4.0
            {
                c = OR_TERNI;
                e = fois(OR_TERNI, 0.35);
            } else {
                let (cx, cy) = (lw / 2.0, lh / 2.0);
                if (ix - cx).hypot(iy - cy) <= 4.5 && (ix - cx - 2.0).hypot(iy - cy + 1.0) > 3.6 {
                    c = OR;
                    e = fois(OR, 0.55);
                }
            }
            return (c, e);
        }
        return (fois(PIERRE, 0.55), rien);
    }
    // le sol : trois rangs de dalles, plus larges vers nous
    let (rang, y0, haut) = if v < SOL + 11.0 { (0, SOL, 11.0) } else if v < SOL + 23.0 { (1, SOL + 11.0, 12.0) } else { (2, SOL + 23.0, 13.0) };
    let large = 30.0 + (y0 - SOL);
    let decal = h2(rang, 7) * large;
    let k = ((u + decal) / large).floor() as i32;
    let mut c = fois(PIERRE, 0.62 + 0.10 * h2(k, rang));
    if v - y0 < 1.0 {
        c = fois(c, 1.18);
    }
    if (u + decal).rem_euclid(large) < 1.0 {
        c = fois(c, 0.6);
    }
    if v < SOL + 1.0 {
        c = fois(PIERRE, 1.1);
    }
    let _ = haut;
    (c, rien)
}

/// Le bassin de vision : le fût cannelé, la coupe, son bord d'or, l'eau qui luit.
fn bassin(u: f32, v: f32) -> Option<(Rgb, Rgb)> {
    let rien = [0.0; 3];
    if !(BX - BW - 1.0..=BX + BW + 1.0).contains(&u) || !(BORD - 6.0..PIED).contains(&v) {
        return None;
    }
    // l'eau, et le bord
    let e = (u - BX) / BW;
    if e.abs() <= 1.0 {
        let dy = 5.0 * (1.0 - e * e).max(0.0).sqrt();
        if (v - (BORD - dy)).abs() < 0.5 {
            return Some((OR, rien));
        }
        if (v - (BORD + dy)).abs() < 0.5 {
            return Some((OR_TERNI, rien));
        }
        if (v - (BORD + dy + 1.0)).abs() < 0.5 {
            return Some((fois(PIERRE, 0.5), rien));
        }
        let ee = ((u - BX) / (BW - 4.0)).powi(2) + ((v - BORD) / 4.2).powi(2);
        if ee < 1.0 {
            let anneau = 0.5 + 0.5 * (ee.sqrt() * 18.0).cos();
            return Some((rien, mix(hex(0x0f7fa0), hex(0x6ff4ff), 0.35 + 0.45 * (1.0 - ee) + 0.2 * anneau)));
        }
    }
    // la coupe : une demi-ellipse qui se resserre
    if (BORD..BORD + 17.0).contains(&v) {
        let t = (v - BORD) / 16.0;
        let hw = BW * (1.0 - t * t).max(0.0).sqrt() * 0.9 + 12.0 * t;
        if (u - BX).abs() <= hw {
            let q = (u - (BX - hw)) / (2.0 * hw).max(1.0);
            let k = if q < 0.02 { 0.4 } else { (0.45 + 0.55 * (PI * q).sin()) * (1.0 - 0.25 * t) };
            return Some((fois(PIERRE, k), rien));
        }
        return None;
    }
    // le fût, cannelé, et son socle
    if v >= BORD + 16.0 {
        // le fût, puis le socle à deux marches, son arête claire
        let hw = if v < PIED - 12.0 { 13.0 } else if v < PIED - 5.0 { 22.0 } else { 27.0 };
        if (u - BX).abs() <= hw {
            if (v - (PIED - 12.0)).abs() < 0.5 || (v - (PIED - 5.0)).abs() < 0.5 {
                return Some((fois(PIERRE, 1.25), rien));
            }
            if v >= PIED - 1.0 {
                return Some((fois(PIERRE, 0.4), rien));
            }
            let t = (u - (BX - hw)) / (2.0 * hw);
            let mut c = fois(PIERRE, 0.55 + 0.6 * (PI * (t * 0.85 + 0.1)).sin());
            if hw == 13.0 && ((u - BX).round() as i32).rem_euclid(5) == 0 {
                c = fois(c, 0.72);
            }
            return Some((c, rien));
        }
    }
    None
}

/// Le ciel par l'oculus : son dégradé, l'astre et son halo, les étoiles, la forêt d'argent au
/// loin et sa brume ; l'arc-en-ciel après l'orage.
fn ciel_px(u: f32, v: f32, ciel: &Ciel, saison: Saison, phase: f32, arc: bool) -> Rgb {
    let apex = OCY - RIN;
    let t = ((v - apex) / (2.0 * RIN)).clamp(0.0, 1.0);
    let mut c = mix(ciel.haut, ciel.bas, t.powf(1.6));
    let (mx, my) = ciel.astre;
    let d = (u - mx).hypot(v - my);
    let lune = ciel.lune;
    let rayon = if lune { 14.5 } else { 11.0 };
    // le halo
    let halo = (-((d - rayon).max(0.0) / if lune { 30.0 } else { 40.0 }).powf(1.1)).exp() * if lune { 0.6 } else { 0.8 };
    c = plus(c, fois(ciel.lumiere_astre, halo * if lune { 1.0 } else { 0.9 }));
    if arc && ciel.nuit < 0.5 {
        let ra = (u - OCX).hypot(v - (OCY + 120.0));
        let k = (ra - 150.0) / 14.0;
        if (0.0..1.0).contains(&k) {
            let bandes = [hex(0xff6060), hex(0xffb060), hex(0xfff070), hex(0x70e070), hex(0x60a0ff), hex(0xa070e0)];
            c = mix(c, bandes[((k * 6.0) as usize).min(5)], 0.35);
        }
    }
    // les étoiles, la nuit : fines, quelques-unes en croix
    if ciel.nuit > 0.05 && !ciel.orage {
        let (cu, cv) = (u.floor() as i32, v.floor() as i32);
        let s = h2(cu, cv);
        if s > 0.993 && d > rayon + 8.0 {
            let b = 0.35 + 0.65 * h2(cv, cu);
            let teinte = if h2(cu + 3, cv) < 0.3 { hex(0xfff4d6) } else { hex(0xcfe0ff) };
            c = mix(c, teinte, b * ciel.nuit);
        }
    }
    // l'astre : la lune à sa phase, le soleil
    if d <= rayon {
        if lune {
            let dx = u - mx;
            let dy = v - my;
            let demi = (rayon * rayon - dy * dy).max(0.01).sqrt();
            let xn = dx / demi;
            let k = (2.0 * PI * phase).cos();
            let eclairee = if phase < 0.5 { xn > k } else { xn < -k };
            let mut l = fois(hex(0xf6f3e6), 1.0 - 0.2 * (d / rayon).powi(3));
            for (ex, ey, er) in [(-5.0, -3.0, 5.0), (4.0, 3.0, 4.0), (-1.0, 7.0, 3.0), (6.0, -6.0, 2.5)] {
                if (dx - ex).hypot(dy - ey) < er {
                    l = fois(l, 0.85);
                }
            }
            c = if eclairee { l } else { mix(c, hex(0x3a4670), 0.6) };
        } else {
            c = mix(hex(0xfff8e0), hex(0xffd090), ciel.crepuscule);
        }
    }
    // des nuages qui passent, éclairés par-dessous au couchant, d'encre et ourlés de lune la nuit
    if !ciel.orage {
        let mut nuage = 0f32;
        let mut ombre = 0f32;
        for k in 0..4 {
            let cx = OCX - 130.0 + (h2(k, 201) * 260.0 + ciel.derive + 37.0 * k as f32).rem_euclid(300.0);
            let cy = 34.0 + 70.0 * h2(k, 203);
            for j in 0..5 {
                let (ox, oy, r) = ((j as f32 - 2.0) * 7.0 + 4.0 * h2(k, j + 210), -4.0 * (1.0 - ((j as f32 - 2.0) / 2.5).powi(2)) + 2.0 * h2(k, j + 220), 6.0 + 4.0 * h2(k, j + 230));
                let d = (u - cx - ox).hypot((v - cy - oy) * 1.6) / r;
                if d < 1.0 {
                    let a = (1.0 - d * d).min(1.0);
                    if a > nuage {
                        nuage = a;
                        ombre = ((v - cy + 2.0) / 9.0).clamp(0.0, 1.0);
                    }
                }
            }
        }
        if nuage > 0.0 {
            let clair = mix(mix(hex(0xfafcff), hex(0xffd6b8), ciel.crepuscule), hex(0x38406a), ciel.nuit);
            let sombre = mix(mix(hex(0xc4d0e6), hex(0x8a6a98), ciel.crepuscule), hex(0x141a38), ciel.nuit);
            let teinte = mix(clair, sombre, ombre);
            c = mix(c, teinte, (nuage * 2.2).min(1.0) * (0.85 - 0.25 * ciel.nuit));
        }
    }
    // la forêt d'argent au loin, puis la brume
    let cime = 176.0 + 6.0 * (u / 6.0).sin() + 4.0 * (u / 2.7 + 1.0).sin() + 6.0 * (u / 19.0).sin();
    if v >= cime {
        let foret = match saison {
            Saison::Automne => mix(hex(0x3a2a20), hex(0x6a4a2a), 1.0 - ciel.nuit),
            Saison::Hiver => mix(hex(0x1a2440), hex(0x6a7a90), 1.0 - ciel.nuit),
            _ => mix(hex(0x151d3e), hex(0x4a6a5a), 1.0 - ciel.nuit),
        };
        let crete = match saison {
            Saison::Automne => hex(0xe2b95a),
            Saison::Hiver => hex(0xf0f6ff),
            _ => mix(hex(0x8a9cd6), hex(0xc0d8c8), 1.0 - ciel.nuit),
        };
        c = if v - cime < 2.0 { mix(foret, crete, 0.8) } else { foret };
    }
    let brume = (-((v - 196.0) / 9.0).powi(2)).exp() * 0.38;
    mix(c, mix(hex(0x9aaee8), hex(0xe8eef8), 1.0 - ciel.nuit), brume)
}

/// Une toile du fond, avant la lumière.
struct Trace<'a> {
    alb: &'a mut [Rgb],
    emi: &'a mut [Rgb],
    rw: usize,
    rh: usize,
    sx: f32,
    sy: f32,
}

impl Trace<'_> {
    /// Remplit le rectangle de dessin [u, u + du) × [v, v + dv).
    fn tache(&mut self, u: f32, v: f32, du: f32, dv: f32, c: Rgb, luit: bool) {
        let (x0, x1) = ((u * self.sx).round() as i32, (((u + du) * self.sx).round() as i32).max((u * self.sx).round() as i32 + 1));
        let (y0, y1) = ((v * self.sy).round() as i32, (((v + dv) * self.sy).round() as i32).max((v * self.sy).round() as i32 + 1));
        for y in y0.max(0)..y1.min(self.rh as i32) {
            for x in x0.max(0)..x1.min(self.rw as i32) {
                let i = y as usize * self.rw + x as usize;
                if luit {
                    self.alb[i] = [0.0; 3];
                    self.emi[i] = c;
                } else {
                    self.alb[i] = c;
                }
            }
        }
    }

    fn vitre(&self, u: f32, v: f32) -> bool {
        dans_vitre(u, v)
    }
}

/// Ce qui se pose sur le fond : la guirlande et ses feuilles d'or, la vigne du pilier, les
/// lanternes et leurs chaînes.
fn decor(t: &mut Trace, lanternes: &Lanternes) {
    let feuille = |t: &mut Trace, u: f32, v: f32, k: i32, grande: bool| {
        if t.vitre(u, v) {
            return;
        }
        let c = FEUILLES[(h2(k, 11) * 4.0) as usize % 4];
        if grande {
            t.tache(u, v, 2.0, 2.0, c, false);
            t.tache(u, v + 2.0, 1.0, 1.0, fois(c, 0.8), false);
        } else {
            t.tache(u, v, 1.0, 2.0, c, false);
            t.tache(u + 1.0, v + 1.0, 1.0, 1.0, c, false);
        }
    };
    // la guirlande : festons d'une attache à l'autre, grappes pendantes
    let attaches = [0.0, 70.0, 150.0, 222.0, 300.0, 360.0, 430.0, PILIER + 4.0];
    for (n, p) in attaches.windows(2).enumerate() {
        let (a, b) = (p[0], p[1]);
        let mut u = a;
        while u < b {
            let q = (u - a) / (b - a);
            let v = (16.0 + 9.0 * (PI * q).sin()).floor();
            if !t.vitre(u, v) {
                t.tache(u, v, 1.0, 1.0, fois(VIGNE, 0.85), false);
                t.tache(u, v + 1.0, 1.0, 1.0, fois(VIGNE, 0.45), false);
            }
            if (u as i32) % 9 == 4 {
                feuille(t, u, v + 2.0, u as i32, true);
            }
            u += 1.0;
        }
        let grappe = 5 + (h2(n as i32, 3) * 6.0) as i32;
        for k in 0..grappe {
            feuille(t, a + (2.0 * (k as f32).sin()).round(), 17.0 + 3.0 * k as f32, n as i32 * 31 + k, k % 2 == 0);
        }
    }
    // la vigne qui monte autour du pilier
    let mut v = SOL - 4.0;
    while v > 10.0 {
        let s = (v / 11.0).sin();
        let u = (PILIER + 11.0 + 8.0 * s).floor();
        t.tache(u, v, 1.0, 1.0, VIGNE, false);
        t.tache(u + 1.0, v, 1.0, 1.0, fois(VIGNE, 0.7), false);
        if (v as i32) % 7 == 0 {
            feuille(t, u + if s > 0.0 { 3.0 } else { -4.0 }, v, v as i32, false);
        }
        v -= 1.0;
    }
    // la branche, sa part au-dessus du pilier
    branche(0.0, LARGE, &mut |u, v, c| t.tache(u, v, 1.0, 1.0, c, false));
    // les lanternes et leurs chaînes d'or, maillon après maillon ; le verre luit de sa couleur quand l'aide
    // est là, éteint il garde un reflet
    for (i, &(lx, ly)) in LANTERNES.iter().enumerate() {
        let mut y = 20.0;
        while y < ly - 3.0 {
            if !t.vitre(lx, y) {
                match (y as i32).rem_euclid(4) {
                    0 | 2 => t.tache(lx, y, 1.0, 1.0, OR_TERNI, false),
                    1 => {
                        t.tache(lx - 1.0, y, 1.0, 1.0, OR_TERNI, false);
                        t.tache(lx + 1.0, y, 1.0, 1.0, fois(OR_TERNI, 0.7), false);
                    }
                    _ => {}
                }
            }
            y += 1.0;
        }
        for (j, rang) in LANTERNE.iter().enumerate() {
            for (k, c) in rang.bytes().enumerate() {
                let (u, v) = (lx - 6.0 + k as f32, ly - 3.0 + j as f32);
                // l'or et l'argent prennent la lumière par la gauche du haut
                let biais = 1.0 + 0.18 * (6.0 - k as f32) / 6.0;
                match c {
                    b'r' => t.tache(u, v, 1.0, 1.0, OR_TERNI, false),
                    b'g' => t.tache(u, v, 1.0, 1.0, fois(OR_TERNI, 0.85 * biais), false),
                    b'G' => t.tache(u, v, 1.0, 1.0, fois(OR, biais), false),
                    b'd' => t.tache(u, v, 1.0, 1.0, if k < 6 { OR } else { OR_TERNI }, false),
                    b's' => t.tache(u, v, 1.0, 1.0, if k < 6 { hex(0xdde3f6) } else { hex(0x9aa3c4) }, false),
                    b'w' | b'W' | b'Y' | b'F' => match lanternes[i] {
                        Some(l) => {
                            let verre = match c {
                                b'w' => mix(fois(l, 0.7), hex(0xfff8e6), 0.15),
                                b'W' => mix(l, hex(0xfff8e6), 0.35),
                                b'Y' => mix(fois(l, 1.2), hex(0xfffbf0), 0.7),
                                _ => hex(0xfffef6),
                            };
                            t.tache(u, v, 1.0, 1.0, verre, true);
                        }
                        None => {
                            // le verre éteint, sombre, un reflet sur sa gauche
                            let reflet = (k == 4 && (10..=13).contains(&j)) || (k == 8 && (15..=16).contains(&j));
                            let c = if reflet { hex(0x8a94c8) } else if c == b'w' { hex(0x23284c) } else { hex(0x2c3260) };
                            t.tache(u, v, 1.0, 1.0, c, false);
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}

/// La branche d'argent qui passe le pilier et s'avance sur la cour, ses feuilles d'or pendantes :
/// chaque pixel de dessin de `de` à `a`, et sa couleur.
fn branche(de: f32, a: f32, poser: &mut dyn FnMut(f32, f32, Rgb)) {
    let mut u = (PILIER - 6.0).max(de.floor());
    while u < a.min(LARGE + 210.0) {
        let q = (u - (PILIER - 6.0)) / 236.0;
        let y = (30.0 - 22.0 * q + 9.0 * q * q + 2.0 * (u / 9.0).sin()).floor();
        let e = if q < 0.3 { 3 } else if q < 0.7 { 2 } else { 1 };
        for k in 0..e {
            poser(u, y + k as f32, if k == 0 { hex(0xb8c0dc) } else { hex(0x8e96b4) });
        }
        if (u as i32) % 7 == 0 {
            let n = 1 + (h2(u as i32, 19) * if q < 0.8 { 3.0 } else { 1.0 }) as i32;
            for j in 0..n {
                let c = fois(FEUILLES[(h2(u as i32, j) * 4.0) as usize % 4], 0.85);
                let (fu, fy) = (u + (h2(u as i32, j + 7) * 3.0).floor() - 1.0, y + e as f32 + 3.0 * j as f32);
                poser(fu, fy, c);
                poser(fu + 1.0, fy, c);
                poser(fu, fy + 1.0, c);
                poser(fu + 1.0, fy + 1.0, fois(c, 0.8));
            }
        }
        u += 1.0;
    }
}

/// Une lanterne d'elfe, un pixel de dessin par lettre, de treize de large : l'anneau (r), le dôme d'or (g, G),
/// ses bords (d), la cage d'argent (s), le verre (w, W, Y) et la flamme (F) ; le haut de la cage à la quatrième
/// rangée, la flamme à la onzième sous lui.
const LANTERNE: [&str; 27] = [
    "......r......",
    ".....r.r.....",
    "......r......",
    "......g......",
    ".....gGg.....",
    "....gGGGg....",
    "...gGGGGGg...",
    "..gGGGGGGGg..",
    ".ddddddddddd.",
    "..swwwwwwws..",
    "..swwWWWwws..",
    "..swWWWWWws..",
    "..swWWYWWws..",
    "..swWYFYWws..",
    "..swWYFYWws..",
    "..swWYYYWws..",
    "..swWWYWWws..",
    "..swWWWWWws..",
    "..swwWWWwws..",
    "..swwwwwwws..",
    ".ddddddddddd.",
    "..gGGGGGGGg..",
    "...gGGGGGg...",
    "....gGGGg....",
    ".....gGg.....",
    "......g......",
    "......G......",
];

// Le chat de la machine : une panthère tachetée, or aux rosettes sombres, l'œil vert ; tournée
// vers la gauche, trois pixels de dessin par pixel.
const PANTHERE: [&str; 8] = [
    ".k.k......................",
    "kOkOk...................k.",
    "kOOOOk.................kOk",
    "kgOOOOkkkkkkkkkkkkkk...kOk",
    "kOOwOOOOdOOOdOOOOdOOk.kOk.",
    ".kpwOOdOOOOOOOdOOOOOOkOk..",
    "..kwwOOOOdOOOdOOOOdOOOk...",
    "...kwwwwwwwwwwwwwwwOOk....",
];
const PATTES: [[&str; 3]; 4] = [
    ["...kOk.kOk.....kOk.kOk....", "...kOk.kOk.....kOk.kOk....", "...kwk.kwk.....kwk.kwk...."],
    ["...kOkkOk......kOkkOk.....", "..kOk..kOk....kOk..kOk....", ".kwk....kwk..kwk....kwk..."],
    ["....kOkOk.......kOkOk.....", "....kOkOk.......kOkOk.....", "....kwkwk.......kwkwk....."],
    ["....kOkkOk......kOkkOk....", "...kOk..kOk....kOk..kOk...", "..kwk....kwk..kwk....kwk.."],
];
const PANTHERE_DORT: [&str; 8] = [
    "....k.k...........",
    "...kOkOk..........",
    "..kOOOOOkkkkkk....",
    ".kOkkOOOdOOOdOk...",
    "kOOOOOdOOOOOOOdk..",
    "kdOOOOOOOdOOOOOOk.",
    "kOwwwwwwwwwwwOOOOk",
    ".kkkkkkkkkkkkkkkk.",
];

// Le dragon de Lara : sarcelle, ventre d'or pâle, ailes violettes, cornes d'ivoire ; roulé sur son or, ou en vol.
const DRAGON_DORT: [&str; 13] = [
    ".h......h.................",
    ".hk....kh.....k...........",
    "..kkTTkk.....kWk..k.......",
    ".kTTTTTTk...kWwWkkWk......",
    "kTTtTTtTTk.kWwwwWWwWk.....",
    "kTkkTTkkTTkTWWwwwwwWk.....",
    "kTTTTTTTTTTTTWWWWWWTTk....",
    ".kTnbbnTTTTTTTTTTTTTTTk...",
    ".kTbbbbTkTTTtTTTTDTTTTTk..",
    "..kkkkkkbbbbbbbbbTDDTTTTk.",
    "....kTTkkkkkkkkkkkkkkTTTTk",
    "...kTTTTTTTTTTTTTTTTTTTTk.",
    "....kkkkkkkkkkkkkkkkkkkk..",
];
const DRAGON_EVEIL: [&str; 13] = [
    ".h......h.................",
    ".hk....kh.....k...........",
    "..kkTTkk.....kWk..k.......",
    ".kTTTTTTk...kWwWkkWk......",
    "kTTtTTtTTk.kWwwwWWwWk.....",
    "kTkeTTkeTTkTWWwwwwwWk.....",
    "kTTTTTTTTTTTTWWWWWWTTk....",
    ".kTnbbnTTTTTTTTTTTTTTTk...",
    ".kTbbbbTkTTTtTTTTDTTTTTk..",
    "..kkkkkkbbbbbbbbbTDDTTTTk.",
    "....kTTkkkkkkkkkkkkkkTTTTk",
    "...kTTTTTTTTTTTTTTTTTTTTk.",
    "....kkkkkkkkkkkkkkkkkkkk..",
];
const TAS_OR: [&str; 5] = [
    "......kkkOOYOOOOOkkk......",
    "....kkOOOOoOOOYOOoOOkk....",
    "..kkOOoOOYOOOOoOOOOOOOkk..",
    ".kOOYOOOoOOOOYOOoOOOYOOOk.",
    "kkkkkkkkkkkkkkkkkkkkkkkkkk",
];
const DRAGON_VOL_HAUT: [&str; 17] = [
    "kk....................kk",
    "kWk......h....h......kWk",
    "kwWk.....hk..kh.....kWwk",
    ".kwWk...kTTTTTTk...kWwk.",
    "..kwWk.kTtTTTTtTk.kWwk..",
    "...kwWkTkeTTTTekTkWwk...",
    "....kwWkTTnbbnTTkWwk....",
    ".....kWTkTbbbbTkTWk.....",
    "......kTTTTTTTTTTk......",
    "......kTTbbbbbbTTk......",
    ".......kTbbbbbbTk.......",
    "........kTTkkTTk........",
    ".......kTk....kTk.......",
    "........k..kTTk.k.......",
    "..........kTk...........",
    "...........kTTk.........",
    "............kk..........",
];
const DRAGON_VOL_BAS: [&str; 17] = [
    "........................",
    ".........h....h.........",
    ".........hk..kh.........",
    "........kTTTTTTk........",
    ".......kTtTTTTtTk.......",
    ".......TkeTTTTekT.......",
    "......kkTTnbbnTTkk......",
    "...kkkWTkTbbbbTkTWkkk...",
    "..kWwwwWTTTTTTTTWwwwWk..",
    ".kWwwwWkTTbbbbTTkWwwwWk.",
    "kWwwWk.kTbbbbbbTk.kWwwWk",
    "kwwk....kTTkkTTk....kwwk",
    "kwk....kTk....kTk....kwk",
    "kk......k..kTTk.k.....kk",
    "..........kTk...........",
    "...........kTTk.........",
    "............kk..........",
];

fn couleur_dragon(c: u8) -> Option<Rgb> {
    Some(match c {
        b'k' => hex(0x14202e),
        b'D' => hex(0x1d5a66),
        b'T' => hex(0x2f8f9c),
        b't' => hex(0x6fd0d0),
        b'b' => hex(0xf0d890),
        b'h' => hex(0xeadfb8),
        b'W' => hex(0x4b3f8f),
        b'w' => hex(0x7f70d0),
        b'e' => hex(0xffd84a),
        b'n' => hex(0x0c1418),
        b'O' => hex(0xe8c878),
        b'o' => hex(0xb08a3e),
        b'Y' => hex(0xfff3b0),
        _ => return None,
    })
}

/// Où le dragon dort : le coin gauche du belvédère, devant le lambris, loin d'elle et de la vasque.
const DRAGON_U: f32 = 4.0;
/// Un pixel de dragon fait PAS_DRAGON × PAS_DRAGON pixels de dessin.
const PAS_DRAGON: f32 = 3.0;

/// Le haut du tas d'or, et celui du dragon posé dessus, en pixels de dessin.
fn dragon_haut() -> (f32, f32) {
    let tas = SOL + 30.0 - TAS_OR.len() as f32 * PAS_DRAGON;
    (tas, tas - (DRAGON_DORT.len() as f32 - 2.0) * PAS_DRAGON)
}

/// Le tas d'or, et le dragon qui y dort ou y veille, derrière elle.
fn dragon_au_nid(p: &mut Pinceau, d: &Dragon, t: f32) {
    let geste = d.geste(d.depuis.elapsed().as_secs_f32());
    if geste == Geste::Absent {
        return;
    }
    let (tas, haut) = dragon_haut();
    let k = p.lumiere(DRAGON_U + 26.0, tas);
    p.dessin(DRAGON_U, tas, &TAS_OR, couleur_dragon, PAS_DRAGON, false, k);
    // un éclat sur l'or, de temps en temps
    let n = (t / 7.0) as i32;
    if (t as i32) % 7 < 2 {
        let u = DRAGON_U + (4.0 + h2(n, 3) * 64.0).floor();
        p.luire(u, tas + 2.0 + (h2(n, 5) * 8.0).floor(), hex(0xfff3b0), 0.8);
    }
    match geste {
        Geste::Dort => {
            p.dessin(DRAGON_U, haut, &DRAGON_DORT, couleur_dragon, PAS_DRAGON, false, k);
            // un petit z qui monte, de temps en temps
            let age = t as i32 % 36;
            if age < 14 {
                let f = age as f32 / 14.0;
                for (j, r) in ["zzz", ".z.", "zzz"].iter().enumerate() {
                    for (i, c) in r.bytes().enumerate() {
                        if c == b'z' {
                            p.poser(DRAGON_U + 26.0 + 6.0 * f + i as f32, haut - 4.0 - 10.0 * f + j as f32, hex(0xdfe8ff), 1.0 - f);
                        }
                    }
                }
            }
        }
        // éveillé, il cligne de temps en temps
        Geste::Eveille => {
            let cligne = (t as i32) % 30 == 0;
            p.dessin(DRAGON_U, haut, if cligne { &DRAGON_DORT } else { &DRAGON_EVEIL }, couleur_dragon, PAS_DRAGON, false, k);
        }
        _ => {}
    }
}

/// Le dragon en vol, devant elle : il quitte son or vers le haut du belvédère, ou il en revient et s'y pose.
fn dragon_en_vol(p: &mut Pinceau, d: &Dragon, t: f32) {
    let Geste::Vole { part, f } = d.geste(d.depuis.elapsed().as_secs_f32()) else { return };
    let (_, haut) = dragon_haut();
    // où il en est entre son or (0) et le haut (1) : il part en prenant de la vitesse, il se pose en freinant
    let g = if part { f * f } else { (1.0 - f) * (1.0 - f) };
    let (u0, v0) = (DRAGON_U + 1.0, haut - 6.0);
    let (u1, v1) = (DRAGON_U + 90.0, -70.0);
    let u = u0 + (u1 - u0) * g + (g * PI * 2.0).sin() * 6.0;
    let v = v0 + (v1 - v0) * g;
    let image = if (t as i64) % 2 == 0 { &DRAGON_VOL_HAUT } else { &DRAGON_VOL_BAS };
    let k = p.lumiere(u + 24.0, v.max(0.0) + 17.0);
    p.dessin(u, v, image, couleur_dragon, PAS_DRAGON, false, k);
}

fn couleur_panthere(c: u8) -> Option<Rgb> {
    Some(match c {
        b'k' => hex(0x2a1a10),
        b'O' => hex(0xd9a441),
        b'o' => hex(0xb07a28),
        b'd' => hex(0x5a3a1a),
        b'w' => hex(0xf5e2b0),
        b'g' => hex(0x6fe08a),
        b'p' => hex(0xe08a8a),
        _ => return None,
    })
}

// La chouette messagère, d'argent aux yeux d'or : ailes basses, puis hautes.
const CHOUETTE: [[&str; 9]; 2] = [
    [
        "...k.....k...",
        "..kWk...kWk..",
        ".kWyykWkyyWk.",
        ".kWykkWkkyWk.",
        "kWWWWWbWWWWWk",
        "kSWWWWWWWWWSk",
        "kSSWWwWwWWSSk",
        ".kSSWWWWWSSk.",
        "...kk...kk...",
    ],
    [
        "Sk.k.....k.kS",
        "SSkWk...kWkSS",
        ".kWyykWkyyWk.",
        ".kWykkWkkyWk.",
        "..kWWWbWWWk..",
        "..kWWWWWWWk..",
        "..kWwWwWwWk..",
        "...kWWWWWk...",
        "...kk...kk...",
    ],
];

fn couleur_chouette(c: u8) -> Option<Rgb> {
    Some(match c {
        b'k' => hex(0x3a3448),
        b'W' => hex(0xeef0f8),
        b'w' => hex(0xc8cce0),
        b'S' => hex(0xa8b0cc),
        b'y' => hex(0xf2c14e),
        b'b' => hex(0xd0a040),
        _ => return None,
    })
}

// Le papillon de lune qui porte un envoi : ailes hautes, puis basses.
const PAPILLON: [[&str; 5]; 2] = [["S.......S", ".SS.c.SS.", "..SScSS..", "...ScS...", "....c...."], [".........", "SSS.c.SSS", ".SSScSSS.", "...ScS...", "....c...."]];

/// Peint par-dessus le fond, en notant chaque pixel touché.
struct Pinceau<'a> {
    px: &'a mut [u32],
    pw: usize,
    ph: usize,
    ox: i32,
    rw: usize,
    /// la part de l'image où la région se voit
    visible: (i32, i32),
    /// le bord droit de la scène dans l'image
    fin_image: i32,
    traces: &'a mut Vec<u32>,
    zone: Option<(i32, i32, i32, i32)>,
    dehors: Option<(i32, i32, i32, i32)>,
    sx: f32,
    sy: f32,
    grille: &'a [[f32; 3]],
    /// hors de la région, peindre sans le dire
    muet: bool,
}

impl Pinceau<'_> {
    /// Change le pixel d'écran (x, y) par `f`, qui reçoit sa couleur.
    #[inline]
    fn pixel(&mut self, x: i32, y: i32, f: impl Fn(Rgb) -> Rgb) {
        if x < 0 || y < 0 || y as usize >= self.ph || x >= self.fin_image || x as usize >= self.pw {
            return;
        }
        let i = y as usize * self.pw + x as usize;
        self.px[i] = pack(f(unpack(self.px[i])));
        let r = (x, y, x + 1, y + 1);
        if x >= self.visible.0 && x < self.visible.1 {
            self.traces.push((y as usize * self.rw + (x - self.ox) as usize) as u32);
            self.zone = Some(self.zone.map_or(r, |a| (a.0.min(x), a.1.min(y), a.2.max(x + 1), a.3.max(y + 1))));
        } else if !self.muet {
            self.dehors = Some(self.dehors.map_or(r, |a| (a.0.min(x), a.1.min(y), a.2.max(x + 1), a.3.max(y + 1))));
        }
    }

    /// Les pixels d'écran du rectangle de dessin [u, u + du) × [v, v + dv).
    fn ecran(&self, u: f32, v: f32, du: f32, dv: f32) -> (i32, i32, i32, i32) {
        let x0 = (u * self.sx).round() as i32 + self.ox;
        let y0 = (v * self.sy).round() as i32;
        let x1 = ((u + du) * self.sx).round() as i32 + self.ox;
        let y1 = ((v + dv) * self.sy).round() as i32;
        (x0, y0, x1.max(x0 + 1), y1.max(y0 + 1))
    }

    /// Pose `c` sur le point de dessin (u, v), vu à travers `a`.
    fn poser(&mut self, u: f32, v: f32, c: Rgb, a: f32) {
        if a <= 0.0 {
            return;
        }
        let (x0, y0, x1, y1) = self.ecran(u, v, 1.0, 1.0);
        let a = a.min(1.0);
        for y in y0..y1 {
            for x in x0..x1 {
                self.pixel(x, y, |p| mix(p, c, a));
            }
        }
    }

    /// Ajoute de la lumière au point de dessin (u, v).
    fn luire(&mut self, u: f32, v: f32, c: Rgb, a: f32) {
        if a <= 0.0 {
            return;
        }
        let (x0, y0, x1, y1) = self.ecran(u, v, 1.0, 1.0);
        for y in y0..y1 {
            for x in x0..x1 {
                self.pixel(x, y, |p| [p[0] + c[0] * a, p[1] + c[1] * a, p[2] + c[2] * a]);
            }
        }
    }

    /// Un éclat : un point, et une croix qui s'estompe si `croix` > 0.
    fn eclat(&mut self, u: f32, v: f32, c: Rgb, a: f32, croix: f32) {
        let (u, v) = (u.floor(), v.floor());
        self.luire(u, v, c, a);
        if croix > 0.0 {
            for k in 1..=(croix.ceil() as i32) {
                let f = a * (1.0 - (k as f32 - 1.0) / croix.max(1.0)) * 0.55;
                let k = k as f32;
                self.luire(u + k, v, c, f);
                self.luire(u - k, v, c, f);
                self.luire(u, v + k, c, f);
                self.luire(u, v - k, c, f);
            }
        }
    }

    /// Une lueur douce : un disque qui s'estompe vers son bord.
    fn lueur(&mut self, u: f32, v: f32, r: f32, c: Rgb, a: f32) {
        let mut y = (v - r).floor();
        while y <= v + r {
            let mut x = (u - r).floor();
            while x <= u + r {
                let d = (x + 0.5 - u).hypot(y + 0.5 - v) / r;
                if d < 1.0 {
                    let f = (1.0 - d) * (1.0 - d);
                    self.luire(x, y, c, a * f);
                }
                x += 1.0;
            }
            y += 1.0;
        }
    }

    /// Un dessin en lettres, `p` pixels de dessin par lettre, éclairé par `k`.
    fn dessin(&mut self, u: f32, v: f32, rangs: &[&str], couleur: fn(u8) -> Option<Rgb>, p: f32, miroir: bool, k: Rgb) {
        let large = rangs.iter().map(|r| r.len()).max().unwrap_or(0);
        for (j, rang) in rangs.iter().enumerate() {
            for (i, c) in rang.bytes().enumerate() {
                let Some(c) = couleur(c) else { continue };
                let i = if miroir { large - 1 - i } else { i };
                let c = [c[0] * k[0], c[1] * k[1], c[2] * k[2]];
                let (x0, y0, x1, y1) = self.ecran(u + i as f32 * p, v + j as f32 * p, p, p);
                for y in y0..y1 {
                    for x in x0..x1 {
                        self.pixel(x, y, |_| c);
                    }
                }
            }
        }
    }

    /// La lumière de la pièce au point de dessin (u, v), pour ce qui s'y pose.
    fn lumiere(&self, u: f32, v: f32) -> Rgb {
        if self.grille.is_empty() {
            return [1.0; 3];
        }
        let fx = ((u / DW) as i32).clamp(0, FIN - 1);
        let fy = ((v / (DH / 2.0)) as i32).clamp(0, SH - 1);
        let g = self.grille[(fy * FIN + fx) as usize];
        [g[0] * 0.9, g[1] * 0.9, g[2] * 0.9]
    }
}

/// La couleur d'un travail : celle de la vision qui monte du bassin.
fn teinte(s: Saveur) -> Rgb {
    hex(match s {
        Saveur::Forge => 0x8ff6ff,
        Saveur::Build => 0xffd36a,
        Saveur::Test => 0x8dffb0,
        Saveur::Shell => 0xa8b8ff,
        Saveur::Git => 0xffb070,
        Saveur::Web => 0x7fd4ff,
        Saveur::Agent => 0xd0a0ff,
        Saveur::Mcp => 0xff9fd0,
    })
}

/// Un point d'une courbe de Bézier de degré deux.
fn bezier(a: (f32, f32), b: (f32, f32), c: (f32, f32), t: f32) -> (f32, f32) {
    let s = 1.0 - t;
    (s * s * a.0 + 2.0 * s * t * b.0 + t * t * c.0, s * s * a.1 + 2.0 * s * t * b.1 + t * t * c.1)
}

/// Les constellations qu'elle trace, dans le haut de l'oculus que sa capuche laisse voir.
const CONSTELLATIONS: [&[(f32, f32)]; 3] = [
    &[(262.0, 92.0), (270.0, 70.0), (288.0, 58.0), (300.0, 38.0), (322.0, 30.0), (318.0, 22.0)],
    &[(366.0, 24.0), (384.0, 34.0), (402.0, 30.0), (418.0, 46.0), (432.0, 70.0), (424.0, 92.0)],
    &[(276.0, 44.0), (296.0, 28.0), (330.0, 22.0), (360.0, 26.0), (392.0, 22.0), (410.0, 40.0)],
];

/// Où elle est, et l'état des choses, pour les deux passes d'une image.
struct Scene<'a> {
    w: &'a Monde,
    a: Activite,
    tick: i64,
    t: f32,
    /// le milieu de sa place, en pixels de dessin
    ex: f32,
    /// la vision du travail, qui flotte au-dessus du bassin
    orbe: (f32, f32),
    echec: bool,
    sommeil: bool,
}

impl<'a> Scene<'a> {
    fn de(v: &'a Vue) -> Self {
        let w = v.monde;
        let ex = v.elle.map_or(OCX, |x| x * DW);
        let a = v.activite;
        Scene {
            w,
            a,
            tick: v.tick,
            t: w.clock as f32,
            ex,
            orbe: (BX, 112.0),
            echec: a == Activite::Failed || a == Activite::Quench && w.outcome == Some(Issue::Fail),
            sommeil: a == Activite::Sleep,
        }
    }
}

/// Un éclat de lumière qui se voit à l'écran : une étoile à quatre branches, son cœur blanc, un
/// petit halo de sa couleur.
fn etincelle(p: &mut Pinceau, u: f32, v: f32, c: Rgb, a: f32, grand: bool) {
    if a <= 0.02 {
        return;
    }
    let (u, v) = (u.floor(), v.floor());
    let l = if grand { 5 } else { 3 };
    p.lueur(u + 0.5, v + 0.5, l as f32 + 3.0, c, 0.3 * a);
    let blanc = mix(c, hex(0xffffff), 0.6);
    p.poser(u, v, hex(0xffffff), a);
    for k in 1..=l {
        let f = a * (1.0 - (k - 1) as f32 / l as f32);
        let col = if k == 1 { blanc } else { c };
        let k = k as f32;
        for (du, dv) in [(k, 0.0), (-k, 0.0), (0.0, k), (0.0, -k)] {
            p.poser(u + du, v + dv, col, f);
        }
    }
    for (du, dv) in [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)] {
        p.luire(u + du, v + dv, c, 0.35 * a);
    }
}

/// Ce qui bouge derrière elle : l'eau, les éclats qui montent du bassin, les étoiles, les
/// flammes, le temps qu'il fait, ce que le ciel montre d'un moment, le chat, la branche.
fn anime_derriere(p: &mut Pinceau, v: &Vue, ciel: &Ciel, lanternes: &Lanternes) {
    let s = Scene::de(v);
    let (w, t, a, tick) = (s.w, s.t, s.a, s.tick);
    let travail = a == Activite::Running;

    // l'eau du bassin : des rides qui s'éloignent du centre, plus vives au travail
    let vif = if travail { 1.0 } else if s.sommeil { 0.35 } else { 0.65 };
    let mut u = BX - BW + 4.0;
    while u < BX + BW - 3.0 {
        let mut y = BORD - 4.0;
        while y <= BORD + 4.0 {
            let ee = ((u - BX) / (BW - 4.0)).powi(2) + ((y - BORD) / 4.2).powi(2);
            if ee < 0.9 {
                let r = ee.sqrt();
                let onde = (r * 22.0 - t * 0.9).sin();
                if onde > 0.6 {
                    let c = if s.echec { hex(0xc070ff) } else { hex(0xb8fbff) };
                    p.luire(u, y, c, (onde - 0.6) * 1.4 * vif * (1.0 - r * 0.7));
                }
            }
            y += 1.0;
        }
        u += 1.0;
    }
    // les reflets de l'eau qui dansent sur le mur, au-dessus du bassin : un filet de lumière qui
    // ondule, plus vif au travail
    let force = vif * (0.55 + 0.45 * ciel.nuit) * if s.echec { 0.6 } else { 1.0 };
    let reflet = if s.echec { hex(0xc8a0ff) } else { hex(0xa8f8ff) };
    let mut y = BORD - 70.0;
    while y < BORD - 7.0 {
        let haut = ((BORD - 7.0 - y) / 63.0).clamp(0.0, 1.0);
        let large = (BW + 8.0) * (1.0 - 0.35 * haut);
        let mut u = BX - large;
        while u < BX + large {
            let n = (u * 0.19 + t * 0.11 + (y * 0.13 + t * 0.05).sin() * 1.7).sin() + (y * 0.31 - t * 0.09 + (u * 0.08 - t * 0.06).sin() * 1.9).sin();
            let k = (1.0 - n.abs() / 0.32).max(0.0);
            if k > 0.0 {
                let bord = 1.0 - ((u - BX) / large).powi(2);
                p.luire(u, y, reflet, 0.2 * k * k * bord * (1.0 - haut).powf(0.7) * force);
            }
            u += 1.0;
        }
        y += 1.0;
    }
    // la poussière qui flotte dans le rai
    if ciel.rai > 0.12 {
        let (dx, dy) = ciel.dir;
        for i in 0..22 {
            let along = 110.0 + 150.0 * (h2(i, 301) + t * (0.0012 + 0.001 * h2(i, 303))).fract();
            // sur toute la largeur du rai, qui s'ouvre en descendant : les flancs se voient de part et d'autre d'elle
            let travers = (h2(i, 305) - 0.5) * 2.0 * RIN * 0.86 * (1.0 + along / 260.0) * 0.95 + 5.0 * (t * 0.02 + 6.0 * h2(i, 307)).sin();
            let (u, y) = (OCX + along * dx + travers * dy, OCY + along * dy - travers * dx);
            if y >= SOL + 6.0 || dans_vitre(u, y) {
                continue;
            }
            let on = (0.5 + 0.5 * (t * (0.05 + 0.05 * h2(i, 309)) + 6.0 * h2(i, 311)).sin()).powi(2);
            let k = rai(u, y, ciel);
            p.luire(u.floor(), y.floor(), mix(ciel.lumiere_astre, hex(0xffffff), 0.5), 0.9 * on * k);
        }
    }
    // son ombre au sol, à l'opposé du rai
    {
        let cx = s.ex - ciel.dir.0 * 26.0 * ciel.rai.min(1.0);
        let (rx, ry, cy) = (118.0, 8.0, SOL + 21.0);
        let mut y = (cy - ry).floor();
        while y <= cy + ry {
            let mut u = (cx - rx).floor();
            while u <= cx + rx {
                let d = ((u + 0.5 - cx) / rx).powi(2) + ((y + 0.5 - cy) / ry).powi(2);
                if d < 1.0 {
                    p.poser(u, y, hex(0x080a18), 0.55 * (1.0 - d).powf(0.7) * (0.6 + 0.4 * ciel.rai.min(1.0)));
                }
                u += 1.0;
            }
            y += 1.0;
        }
    }

    // les lueurs des aides, qui dérivent sur l'eau
    for (i, ap) in w.apprentices.iter().filter(|a| a.leaving.is_none()).take(6).enumerate() {
        let c = super::accessoires::PALETTE.get(ap.colour.0 as usize).copied().flatten().map_or(hex(0xffd36a), hex);
        let ph = ap.seed as f32 * 1.7 + i as f32;
        let (lu, lv) = (BX + (BW - 12.0) * (t * 0.03 + ph).sin(), BORD + 2.5 * (t * 0.05 + ph * 2.0).sin());
        p.lueur(lu, lv, 4.0, c, 0.8);
        p.eclat(lu, lv, hex(0xffffff), 0.8, 1.0);
    }

    // les éclats du bassin, hors du travail (au travail ils montent en vision, devant)
    if !travail {
        let n = match a {
            Activite::Thinking | Activite::Review => 16,
            Activite::Sleep => 5,
            Activite::Failed => 12,
            _ => 10,
        };
        for i in 0..n {
            let vitesse = 0.012 + 0.02 * h2(i, 5);
            let f = (t * vitesse * if s.sommeil { 0.4 } else { 1.0 } + h2(i, 91)).fract();
            let (u, y, al) = if s.echec {
                // ils retombent, ternis
                (BX + (h2(i, 3) - 0.5) * 90.0, BORD - 140.0 + f * 136.0, 0.7 * (1.0 - f))
            } else {
                // ils montent et se balancent
                let haut = f.powf(0.8);
                let u = BX + (h2(i, 3) - 0.5) * (28.0 + 36.0 * haut) + 6.0 * ((BORD - haut * 150.0) / 9.0).sin();
                (u, BORD - 6.0 - haut * 150.0, (1.0 - haut * 0.85) * (f * 8.0).min(1.0))
            };
            let c = if s.echec { hex(0x9a7ad8) } else if h2(i, 17) < 0.3 { hex(0xffe39a) } else { hex(0x8ff6ff) };
            etincelle(p, u.floor(), y.floor(), c, al, h2(i, 47) < 0.2);
        }
    }

    // les étoiles qui scintillent, la nuit
    if ciel.nuit > 0.3 && !ciel.orage {
        for i in 0..16 {
            let (u, y) = (OCX - 92.0 + 184.0 * h2(i, 61), OCY - 94.0 + 64.0 * h2(i, 67));
            if dans_vitre(u, y) && (u - ciel.astre.0).hypot(y - ciel.astre.1) > 24.0 {
                let sc = (t * (0.15 + 0.2 * h2(i, 71)) + 6.0 * h2(i, 73)).sin();
                if sc > 0.2 {
                    p.eclat(u, y, hex(0xfff0c8), (sc - 0.2) * 1.2 * ciel.nuit, if sc > 0.75 { 2.0 } else { 1.0 });
                }
            }
        }
    }

    // les étoiles d'or du pochoir qui scintillent, une à une, lentement
    for i in 0..9 {
        let ph = 6.28 * h2(i, 501);
        let g = (t * (0.045 + 0.03 * h2(i, 503)) + ph).sin();
        if g < 0.86 {
            continue;
        }
        // une nouvelle étoile à chaque éclat
        let n = ((t * (0.045 + 0.03 * h2(i, 503)) + ph) / 6.283).floor() as i32;
        let j = (h2(i * 31 + n, 505) * 10.0) as i32;
        let decal = if j % 2 == 1 { 14.0 } else { 0.0 };
        let (u, v) = (decal + 28.0 * (h2(i * 17 + n, 507) * 17.0).floor(), 26.0 + 16.0 * j as f32);
        if v < LAMBRIS - 6.0 && u < PILIER - 6.0 && (u - OCX).hypot(v - OCY) > RFR + 4.0 {
            let a = (g - 0.86) / 0.14;
            p.eclat(u, v, hex(0xfff0b8), 0.9 * a, 1.0 + 2.0 * a);
        }
    }

    // une étoile filante, de loin en loin, dans le haut de l'oculus
    if ciel.nuit > 0.6 && !ciel.orage {
        let n = w.clock.div_euclid(260);
        let age = w.clock.rem_euclid(260);
        if age < 7 {
            let (u0, v0) = (OCX - 80.0 + 110.0 * h2(n as i32, 401), 18.0 + 30.0 * h2(n as i32, 403));
            let (du, dv) = (13.0, 5.0 + 3.0 * h2(n as i32, 405));
            let f = age as f32;
            for k in 0..14 {
                let q = f - k as f32 * 0.18;
                if q < 0.0 {
                    break;
                }
                let (u, y) = ((u0 + du * q).floor(), (v0 + dv * q).floor());
                if dans_vitre(u, y) {
                    p.luire(u, y, hex(0xfff6e0), (1.0 - k as f32 / 14.0) * (1.0 - f / 7.0) * 0.95);
                }
            }
        }
    }

    // les flammes des lanternes, qui vacillent
    for (i, l) in lanternes.iter().enumerate() {
        if let Some(c) = l {
            let (lx, ly) = LANTERNES[i];
            let f = 0.5 + 0.5 * (t * 0.7 + i as f32 * 2.1).sin() * (t * 0.23 + i as f32).cos();
            p.lueur(lx, ly + 11.0, 9.0, *c, 0.12 + 0.12 * f);
            p.luire(lx, ly + 10.0 + (f * 2.0).floor(), mix(*c, hex(0xffffff), 0.5), 0.3 + 0.3 * f);
            p.luire(lx, ly + 11.0, hex(0xfff6e0), 0.25 * f);
        }
    }

    // une luciole ou deux, la nuit, quand rien ne presse
    if ciel.nuit > 0.5 && matches!(a, Activite::Idle | Activite::Waiting | Activite::Sleep) {
        for i in 0..3 {
            let ph = i as f32 * 3.7;
            let (u, y) = (30.0 + 190.0 * (0.5 + 0.5 * (t * 0.011 + ph).sin()), 80.0 + 80.0 * (0.5 + 0.5 * (t * 0.017 + ph * 1.3).cos()));
            let on = (0.5 + 0.5 * (t * 0.35 + ph).sin()).powi(2);
            p.lueur(u, y, 5.0, hex(0xd8ff8a), 0.5 * on);
            p.eclat(u, y, hex(0xf8ffd0), on, 0.0);
        }
    }

    // la pluie sur l'oculus sous l'orage, et l'éclair ; la neige l'hiver
    if ciel.orage {
        for i in 0..50 {
            let u = OCX - RIN + 2.0 * RIN * h2(i, 101);
            let y = (OCY - RIN) + ((t * 9.0 + 400.0 * h2(i, 103)) % (2.0 * RIN));
            for k in 0..5 {
                let (uu, yy) = ((u - k as f32 * 0.5).floor(), y - k as f32);
                if dans_vitre(uu, yy) {
                    p.poser(uu, yy, hex(0xb8d0f0), 0.45 - 0.07 * k as f32);
                }
            }
        }
        if w.clock % 23 < 2 {
            let mut y = OCY - RIN;
            while y < OCY + RIN {
                let mut u = OCX - RIN;
                while u < OCX + RIN {
                    if dans_vitre(u, y) {
                        p.luire(u, y, hex(0xd0d8ff), 0.45);
                    }
                    u += 1.0;
                }
                y += 1.0;
            }
        }
    } else if w.saison() == Saison::Hiver {
        for i in 0..30 {
            let u = OCX - RIN + 2.0 * RIN * h2(i, 107) + 4.0 * (t * 0.1 + i as f32).sin();
            let y = (OCY - RIN) + ((t * 1.2 + 400.0 * h2(i, 109)) % (2.0 * RIN));
            if dans_vitre(u, y) {
                p.poser(u.floor(), y.floor(), hex(0xf4f8ff), 0.9);
                if i % 3 == 0 {
                    p.poser(u.floor() + 1.0, y.floor(), hex(0xf4f8ff), 0.5);
                }
            }
        }
    }
    // à Halloween, des chauves-souris passent devant la lune
    if w.fete() == Some(super::monde::Fete::Halloween) && ciel.nuit > 0.5 {
        for i in 0..2 {
            let f = ((t * 0.006 + i as f32 * 0.5) % 1.0) * 2.4 - 0.7;
            let (u, y) = ((OCX - RIN + 2.0 * RIN * f).floor(), (24.0 + 16.0 * i as f32 + 6.0 * (t * 0.5 + i as f32).sin()).floor());
            let ailes: &[&str] = if (w.clock + i) % 4 < 2 { &["k.....k", "kk.k.kk", ".kkkkk."] } else { &[".......", ".kkkkk.", "kk.k.kk"] };
            for (j, r) in ailes.iter().enumerate() {
                for (k, c) in r.bytes().enumerate() {
                    let (uu, yy) = (u + 2.0 * k as f32, y + 2.0 * j as f32);
                    if c == b'k' && dans_vitre(uu, yy) {
                        for (du, dv) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
                            p.poser(uu + du, yy + dv, hex(0x0a0814), 1.0);
                        }
                    }
                }
            }
        }
    }

    // ce que les moments montrent derrière elle
    match a {
        Activite::Thinking | Activite::Review => {
            // elle trace une constellation, étoile après étoile, d'un fil d'or
            let c = CONSTELLATIONS[(w.turn_ticks.max(w.clock) / 300).rem_euclid(3) as usize];
            let f = if a == Activite::Review { 1.0 } else { (tick.rem_euclid(48) as f32 / 34.0).min(1.0) };
            let segs = (c.len() - 1) as f32;
            for (k, sg) in c.windows(2).enumerate() {
                let part = (f * segs - k as f32).clamp(0.0, 1.0);
                if part <= 0.0 {
                    continue;
                }
                let (a0, b0) = (sg[0], sg[1]);
                let n = (b0.0 - a0.0).abs().max((b0.1 - a0.1).abs()).ceil() as i32;
                for j in 0..=(n as f32 * part) as i32 {
                    let q = j as f32 / n.max(1) as f32;
                    let (u, y) = ((a0.0 + (b0.0 - a0.0) * q).floor(), (a0.1 + (b0.1 - a0.1) * q).floor());
                    if dans_vitre(u, y) {
                        p.luire(u, y, OR, 0.75);
                    }
                }
                // la pointe du fil, qui avance
                if part < 1.0 {
                    let (u, y) = (a0.0 + (b0.0 - a0.0) * part, a0.1 + (b0.1 - a0.1) * part);
                    etincelle(p, u.floor(), y.floor(), hex(0xfff0c0), 1.0, false);
                }
            }
            for (k, &(u, y)) in c.iter().enumerate() {
                if f * segs >= k as f32 - 0.01 && dans_vitre(u, y) {
                    let vive = (f * segs - k as f32).clamp(0.0, 1.0);
                    p.lueur(u, y, 5.0, OR, 0.35 + 0.4 * (1.0 - vive));
                    p.eclat(u, y, hex(0xfff6dc), 1.0, 1.0 + 2.0 * (1.0 - vive));
                }
            }
        }
        Activite::Quench => {
            // les tests : les lanternes s'allument l'une après l'autre d'un éclat vert d'or, ou
            // l'une crachote rouge et s'éteint
            let pass = w.outcome != Some(Issue::Fail);
            for (i, &(lx, ly)) in LANTERNES.iter().enumerate() {
                let age = tick - (2 + 3 * i as i64);
                if pass && (0..10).contains(&age) {
                    let f = age as f32 / 10.0;
                    p.lueur(lx, ly + 11.0, 22.0 * (0.4 + f), hex(0xb8ffb0), 0.6 * (1.0 - f));
                    etincelle(p, lx, ly - 5.0, hex(0xe8ffd0), 1.0 - f, true);
                }
            }
            if !pass {
                let (lx, ly) = LANTERNES[2];
                if tick < 12 && tick % 3 != 2 {
                    p.lueur(lx, ly + 11.0, 18.0, hex(0xff5040), 0.6);
                }
                for i in 0..4 {
                    let f = ((tick as f32 + i as f32 * 2.5) / 10.0).fract();
                    p.eclat(lx - 4.0 + 8.0 * h2(i, 3), ly + 26.0 + 20.0 * f, hex(0xff8060), 1.0 - f, 0.0);
                }
            }
        }
        Activite::Ship if w.ship != Some(Envoi::Push) => {
            // l'œuvre consignée : une étoile monte du bassin et se fixe dans l'oculus
            let n = (w.clock / 500) as i32;
            let cible = (OCX - 70.0 + 140.0 * h2(n, 3), 26.0 + 12.0 * h2(n, 7));
            let (a0, c0) = ((BX, BORD - 4.0), (BX + 60.0, cible.1 - 30.0));
            let f = (tick as f32 / 9.0).clamp(0.0, 1.0);
            let (u, y) = bezier(a0, c0, cible, f);
            if f < 1.0 {
                for k in 1..8 {
                    let (tu, ty) = bezier(a0, c0, cible, (f - k as f32 * 0.025).max(0.0));
                    p.eclat(tu.floor(), ty.floor(), OR, 0.9 - k as f32 * 0.11, 0.0);
                }
                etincelle(p, u.floor(), y.floor(), hex(0xfff6dc), 1.0, true);
            } else {
                let g = ((tick - 9) as f32 / 6.0).clamp(0.0, 1.0);
                let taille = 2.0 + 5.0 * (1.0 - (2.0 * g - 1.0).abs());
                p.lueur(u, y, 10.0, OR, 0.5 * (1.0 - g) + 0.2);
                p.eclat(u.floor(), y.floor(), hex(0xfff6dc), 1.0, taille);
            }
        }
        _ => {}
    }
    if s.echec {
        // l'eau se trouble d'un reflet violet
        p.lueur(BX, BORD, 44.0, hex(0x8040c0), 0.2);
    }

    // le dragon de Lara, sur son or
    if let Some(d) = v.dragon {
        dragon_au_nid(p, d, t);
    }

    // le chat de la machine
    if let Some(chat) = &w.cat {
        if chat.mode == ModeChat::Roof {
            let (u, y) = CHAT_DORT;
            let k = p.lumiere(u + 27.0, y + 12.0);
            p.dessin(u, y, &PANTHERE_DORT, couleur_panthere, 3.0, false, k);
            // la queue qui bat, de temps en temps ; un petit z
            if chat.age % 40 < 16 {
                let f = (chat.age % 40) as f32 / 16.0;
                for (j, r) in ["zzz", ".z.", "zzz"].iter().enumerate() {
                    for (i, c) in r.bytes().enumerate() {
                        if c == b'z' {
                            p.poser(u + 8.0 + 6.0 * f + i as f32, y - 6.0 - 10.0 * f + j as f32, hex(0xdfe8ff), 1.0 - f);
                        }
                    }
                }
            }
        } else {
            let pattes = match chat.step {
                Some(s) if s >= 0 => &PATTES[(s % 4) as usize],
                _ => &PATTES[0],
            };
            let droite = chat.dir.unwrap_or(if chat.mode == ModeChat::Flee { 1 } else { -1 }) > 0;
            let u = chat.x as f32 * DW;
            let y = SOL + 30.0 - 33.0;
            let k = p.lumiere(u + 39.0, y + 15.0);
            let mut corps: Vec<&str> = PANTHERE.to_vec();
            corps.extend_from_slice(pattes);
            p.dessin(u, y, &corps, couleur_panthere, 3.0, droite, k);
        }
    }

    // la part de la branche sur la cour : repeinte à chaque image par-dessus ce que la cour y a
    // repeint, pleine et toujours la même, sans rien demander (les cellules dessous savent
    // quand elles changent)
    p.muet = true;
    branche(LARGE, LARGE + 210.0, &mut |u, v, c| p.poser(u, v, c, 1.0));
    p.muet = false;
}

/// Ce qui passe devant elle : la vision qui monte du bassin, les éclats autour d'elle, les
/// z du sommeil, le papillon d'un envoi, la chouette messagère.
fn anime_devant(p: &mut Pinceau, v: &Vue) {
    let age = v.vision.and_then(Vision::age).filter(|a| *a < VISION_S);
    anime_activite(p, v, age.is_some());
    if let (Some(vi), Some(age)) = (v.vision, age) {
        vision(p, vi, age, v.monde.clock as f32);
    }
    if let Some(d) = v.dragon {
        dragon_en_vol(p, d, v.monde.clock as f32);
    }
}

/// La vision de la vasque (`Vision`), `age` secondes après sa venue, au tic `t` : une colonne de lumière
/// monte de l'eau, l'habitant s'y forme de bas en haut, pâle et cyan, ondule et flotte, puis se défait.
fn vision(p: &mut Pinceau, vi: &Vision, age: f32, t: f32) {
    if age >= VISION_S || vi.images.is_empty() || vi.w == 0 || vi.h == 0 {
        return;
    }
    // un pixel d'habitant fait PAS × PAS pixels de dessin
    const PAS: f32 = 4.0;
    let monte = (age / 1.4).min(1.0);
    let monte = monte * monte * (3.0 - 2.0 * monte);
    let part = ((VISION_S - age) / 1.6).clamp(0.0, 1.0);
    let a = monte.min(part);
    let (lw, lh) = (vi.w as f32 * PAS, vi.h as f32 * PAS);
    // ses pieds flottent au-dessus de l'eau ; il monte de plus bas
    let bas = BORD - 22.0 - 2.5 * (t * 0.07).sin() + 30.0 * (1.0 - monte);
    let (u0, v0) = (BX - lw / 2.0, bas - lh);
    let pale = hex(0xb4f8ff);
    // la colonne de lumière, de l'eau à ses pieds, qui ondule en montant
    let mut y = (bas - 10.0).floor();
    while y < BORD - 3.0 {
        let onde = 0.7 + 0.3 * (y * 0.35 - t * 0.5).sin();
        let mut du = -18.0;
        while du <= 18.0 {
            let k = 1.0 - (du / 18.0f32).powi(2);
            p.luire((BX + du).floor(), y, CYAN, 0.11 * a * k * onde);
            du += 1.0;
        }
        y += 1.0;
    }
    // le halo, qui respire
    p.lueur(BX, bas - lh * 0.5, lw * 0.8, CYAN, (0.2 + 0.06 * (t * 0.3).sin()) * a);
    let img = &vi.images[(age * 1000.0 / 150.0) as usize % vi.images.len()];
    let (w, h) = (vi.w as usize, vi.h as usize);
    let plein = |i: i32, j: i32| i >= 0 && j >= 0 && (i as usize) < w && (j as usize) < h && img[j as usize * w + i as usize] >> 24 != 0;
    for j in 0..h {
        let rang = (h - 1 - j) as f32;
        // il se forme de bas en haut en montant, et se défait de haut en bas, rendu à l'eau, en partant
        let forme = (monte * (h as f32 + 3.0) - rang).clamp(0.0, 1.0) * (part * (h as f32 + 3.0) - rang).clamp(0.0, 1.0);
        if forme <= 0.0 {
            continue;
        }
        // chaque rangée ondule un peu, comme vue à travers l'eau
        let dx = (t * 0.33 + j as f32 * 0.8).sin() * 1.3;
        let ligne = 0.84 + 0.16 * (t * 0.5 + j as f32 * 1.3).sin();
        for i in 0..w {
            let (u, v) = ((u0 + i as f32 * PAS + dx).floor(), (v0 + j as f32 * PAS).floor());
            let c = img[j * w + i];
            if c >> 24 == 0 {
                // son contour luit
                let (ii, jj) = (i as i32, j as i32);
                if plein(ii - 1, jj) || plein(ii + 1, jj) || plein(ii, jj - 1) || plein(ii, jj + 1) {
                    for dy in 0..PAS as i32 {
                        for dxx in 0..PAS as i32 {
                            p.luire(u + dxx as f32, v + dy as f32, CYAN, 0.16 * a * forme);
                        }
                    }
                }
                continue;
            }
            let fantome = mix(unpack(c & 0xFF_FFFF), pale, 0.5);
            let al = 0.8 * a * forme * ligne;
            for dy in 0..PAS as i32 {
                // une raie plus pâle au bas de chaque rangée : une image faite de lumière
                let k = if dy == PAS as i32 - 1 { 0.65 } else { 1.0 };
                for dxx in 0..PAS as i32 {
                    p.poser(u + dxx as f32, v + dy as f32, fantome, al * k);
                }
            }
        }
    }
    // des éclats montent de l'eau le long de la colonne
    for i in 0..10 {
        let f = (t * (0.025 + 0.015 * h2(i, 71)) + h2(i, 73)).fract();
        let (u, y) = (BX + (h2(i, 75) - 0.5) * 34.0 + 3.0 * (t * 0.1 + i as f32).sin(), BORD - 4.0 - f * (BORD - 4.0 - v0));
        p.eclat(u.floor(), y.floor(), if i % 3 == 0 { hex(0xffffff) } else { pale }, a * (1.0 - f) * 0.9, 0.0);
    }
}

/// Ce que fait Galadriel, devant elle ; `en_vision` : une vision tient la place de la sphère du travail.
fn anime_activite(p: &mut Pinceau, v: &Vue, en_vision: bool) {
    let s = Scene::de(v);
    let (w, t, a, tick, ex) = (s.w, s.t, s.a, s.tick, s.ex);
    match a {
        Activite::Running => {
            // le travail : la vision monte du bassin en spirale, à la couleur de l'ouvrage, et se
            // noue en une sphère qui flotte au-dessus de l'eau. Rien ne va sur elle : un flot qui
            // finissait à ses mains se lisait sur son ventre.
            let base = teinte(w.flavor);
            let (ou, ov) = s.orbe;
            for i in 0..28 {
                let f = (t * (0.03 + 0.02 * h2(i, 5)) + h2(i, 91)).fract();
                let c = if h2(i, 17) < 0.3 { hex(0xffe39a) } else { base };
                for k in 0..3 {
                    let q = (f - k as f32 * 0.03).max(0.0);
                    // la spirale se resserre en montant, de la largeur de l'eau à celle de la sphère
                    let ang = h2(i, 23) * 2.0 * PI + q * 3.0 * PI + t * 0.08;
                    let rayon = (BW - 14.0) * (1.0 - q) + 8.0 * q;
                    let (u, y) = (BX + rayon * ang.cos(), BORD - 3.0 - (BORD - 3.0 - ov - 10.0) * q + 3.0 * ang.sin());
                    let al = (1.0 - k as f32 * 0.35) * (q * 8.0).min(1.0) * (1.0 - (q - 0.85).max(0.0) * 6.0);
                    if k == 0 {
                        etincelle(p, u.floor(), y.floor(), c, al, h2(i, 47) < 0.2);
                    } else {
                        p.eclat(u.floor(), y.floor(), c, al * 0.6, 0.0);
                    }
                }
            }
            // la sphère : un halo qui respire, un cœur blanc, un anneau d'éclats qui tourne ; une vision
            // tient sa place tant qu'elle est là
            if !en_vision {
                let souffle = 0.5 + 0.5 * (t * 0.4).sin();
                p.lueur(ou, ov, 26.0, base, 0.35 + 0.15 * souffle);
                // un globe de verre plein de lumière : clair au cœur, le reflet en haut à gauche
                for dy in -8..=8 {
                    for dx in -8..=8 {
                        let d = ((dx * dx + dy * dy) as f32).sqrt() / 8.5;
                        if d <= 1.0 {
                            let reflet = ((dx as f32 + 3.0).powi(2) + (dy as f32 + 3.0).powi(2)).sqrt() < 2.2;
                            let c = if reflet { hex(0xffffff) } else { mix(mix(base, hex(0xffffff), 0.75), fois(base, 0.75), d) };
                            p.poser(ou + dx as f32, ov + dy as f32, c, if d > 0.85 { 0.7 } else { 0.92 });
                        }
                    }
                }
                for i in 0..8 {
                    let ang = t * 0.12 + i as f32 * 2.0 * PI / 8.0;
                    // l'anneau penché : les éclats de derrière plus pâles
                    let (u, y) = (ou + 26.0 * ang.cos(), ov + 8.0 * ang.sin() - 5.0 * ang.cos());
                    let c = if i % 3 == 0 { OR } else { base };
                    etincelle(p, u.floor(), y.floor(), c, if ang.sin() < 0.0 { 0.45 } else { 1.0 }, false);
                }
            }
        }
        Activite::Waiting => {
            // trois éclats tournent autour de sa tête : elle attend un mot
            for i in 0..3 {
                let ang = t * 0.25 + i as f32 * 2.0 * PI / 3.0;
                let (u, y) = (ex + 66.0 * ang.cos(), 122.0 + 18.0 * ang.sin());
                if ang.sin() > -0.2 || (u - ex).abs() > 50.0 {
                    etincelle(p, u.floor(), y.floor(), hex(0x8ff6ff), 1.0, true);
                }
            }
        }
        Activite::Jumping | Activite::Waking => {
            // une gerbe d'étoiles d'or et de cyan autour d'elle
            let f = (tick.rem_euclid(8)) as f32 / 8.0;
            // elle les lance loin d'elle : sur sa robe semée d'étoiles, elles se perdraient
            for i in 0..16 {
                let ang = i as f32 / 16.0 * 2.0 * PI + 0.4 * h2(i, 5);
                let r = (80.0 + 40.0 * h2(i, 9)) + 120.0 * f;
                let (u, y) = (ex + r * ang.cos(), 150.0 + r * 0.6 * ang.sin() + 30.0 * f * f);
                let c = [OR, hex(0x8ff6ff), hex(0xffffff), hex(0xd0a0ff)][i as usize % 4];
                etincelle(p, u.floor(), y.floor(), c, 1.0 - f * f, i % 2 == 0);
            }
        }
        Activite::Waving => {
            // sa main levée laisse une traîne d'éclats
            for i in 0..7 {
                let f = (t * 0.08 + i as f32 / 7.0).fract();
                etincelle(p, (ex - 86.0 - 10.0 * (f * 6.0).sin()).floor(), (104.0 - f * 60.0).floor(), OR, 1.0 - f, i % 3 == 0);
            }
        }
        Activite::Sweep => {
            // un souffle emporte les feuilles d'or de la guirlande à travers la pièce
            for i in 0..26 {
                let u = ((t * 7.0 + 37.0 * i as f32) % (LARGE + 60.0)) - 30.0;
                let y = 30.0 + 220.0 * h2(i, 13) + 14.0 * (t * 0.4 + i as f32).sin();
                let c = FEUILLES[i as usize % 4];
                let k = p.lumiere(u, y);
                let c = [c[0] * k[0] * 1.1, c[1] * k[1] * 1.1, c[2] * k[2] * 1.1];
                let (u, y) = (u.floor(), y.floor());
                let forme: &[(f32, f32)] = match (w.clock + i as i64) % 4 {
                    0 => &[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0), (2.0, 1.0)],
                    1 => &[(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 2.0)],
                    2 => &[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (1.0, 1.0)],
                    _ => &[(1.0, 0.0), (0.0, 1.0), (1.0, 1.0), (0.0, 2.0)],
                };
                for &(du, dv) in forme {
                    for (a, b) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
                        p.poser(u + 2.0 * du + a, y + 2.0 * dv + b, if dv == 0.0 && b == 0.0 { mix(c, hex(0xffffff), 0.3) } else { c }, 1.0);
                    }
                }
            }
        }
        Activite::Sleep => {
            // des z qui montent et rapetissent
            for i in 0..3 {
                let f = (t * 0.05 + i as f32 / 3.0).fract();
                let (u, y) = ((ex + 50.0 + 30.0 * f).floor(), (96.0 - 60.0 * f).floor());
                let al = (1.0 - f) * (f * 5.0).min(1.0);
                let z: &[&str] = if f < 0.33 { &["zzzzzz", "....z.", "...z..", "..z...", ".z....", "zzzzzz"] } else if f < 0.66 { &["zzzz", "..z.", ".z..", "zzzz"] } else { &["zzz", ".z.", "zzz"] };
                for (j, r) in z.iter().enumerate() {
                    for (k, c) in r.bytes().enumerate() {
                        if c == b'z' {
                            for (du, dv) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
                                p.poser(u + 2.0 * k as f32 + du, y + 2.0 * j as f32 + dv, hex(0xdfe8ff), al);
                            }
                        }
                    }
                }
            }
        }
        Activite::Ship if w.ship == Some(Envoi::Push) => {
            // un papillon de lune s'envole du bassin, passe au-dessus d'elle et file vers le large
            let f = (tick as f32 / 26.0).clamp(0.0, 1.0);
            let (ou, ov) = s.orbe;
            let route = |f: f32| (ou + (LARGE + 300.0 - ou) * f, ov - 10.0 - 70.0 * (f * PI).sin() * (1.0 - f * 0.5) - 14.0 * (f * PI * 3.0).sin());
            if f < 1.0 {
                for k in 1..10 {
                    let (u, y) = route((f - k as f32 * 0.012).max(0.0));
                    p.eclat(u.floor() + 6.0, y.floor() + 6.0, hex(0x8ff6ff), 0.8 - k as f32 * 0.08, 0.0);
                }
                let (u, y) = route(f);
                p.lueur(u + 13.0, y + 7.0, 14.0, hex(0x8ff6ff), 0.45);
                p.dessin(u, y, &PAPILLON[(w.clock % 2) as usize], |c| match c {
                    b'S' => Some(hex(0xc8f8ff)),
                    b's' => Some(hex(0x6fd8f0)),
                    b'c' => Some(hex(0xffffff)),
                    _ => None,
                }, 3.0, false, [1.0; 3]);
            }
        }
        _ => {}
    }

    // la chouette messagère, de la cour vers l'autre bout de la pièce, la nouvelle dans ses serres
    if let Some(m) = &w.messenger {
        let f = m.age as f32 / 50.0;
        let (u, y) = ((LARGE + 180.0 - (LARGE + 260.0) * f).floor(), (30.0 + 10.0 * (m.age as f32 / 3.0).sin()).floor());
        p.dessin(u, y, &CHOUETTE[(m.age / 2 % 2) as usize], couleur_chouette, 3.0, false, [1.0; 3]);
        let sceau = if m.kind == Nouvelle::Waiting { hex(0xd04030) } else { hex(0xf2c14e) };
        for (du, dv, c) in [(15.0, 27.0, hex(0xf2e8d0)), (18.0, 27.0, hex(0xf2e8d0)), (21.0, 27.0, sceau), (15.0, 30.0, hex(0xd8ccb0)), (18.0, 30.0, hex(0xd8ccb0)), (21.0, 30.0, hex(0xd8ccb0))] {
            for (a, b) in [(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (0.0, 1.0), (1.0, 1.0), (2.0, 1.0), (0.0, 2.0), (1.0, 2.0), (2.0, 2.0)] {
                p.poser(u + du + a, y + dv + b, c, 1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_lune_suit_ses_phases() {
        // pleine vers le 1er février 2026, nouvelle vers le 17 février
        assert!((phase_lune(2, 1) - 0.5).abs() < 0.08, "{}", phase_lune(2, 1));
        assert!(phase_lune(2, 17) < 0.05 || phase_lune(2, 17) > 0.95, "{}", phase_lune(2, 17));
    }

    /// La tache du soleil au sol suit le soleil : le matin (il est à gauche dans l'oculus) elle part à droite
    /// d'elle, l'après-midi à gauche ; la nuit sans lune, il n'y en a pas.
    #[test]
    fn la_tache_du_soleil_au_sol_suit_le_soleil() {
        let w = Monde { perso: Some(super::super::monde::Perso::Galadriel), ..Monde::default() };
        let (rw, rh) = ((FIN * 9) as usize, (19 * SH / 2) as usize);
        let sol = |heure: f32, phase: f32, u: f32| {
            let c = ciel(heure, false, phase);
            let f = fond(rw, rh, 1.0, 1.0, &c, &w, &[None; 4], false);
            let px = unpack(f[(SOL + 10.0) as usize * rw + u as usize]);
            px[0] + px[1] + px[2]
        };
        let (gauche, droite) = (OCX - 150.0, OCX + 120.0);
        let (matin_d, matin_g) = (sol(9.2, 0.5, droite), sol(9.2, 0.5, gauche));
        let (soir_d, soir_g) = (sol(16.8, 0.5, droite), sol(16.8, 0.5, gauche));
        println!("matin : gauche {matin_g}, droite {matin_d} ; après-midi : gauche {soir_g}, droite {soir_d}");
        assert!(matin_d > soir_d * 1.15, "le matin, le sol à sa droite est au soleil : {matin_d} contre {soir_d}");
        assert!(soir_g > matin_g * 1.15, "l'après-midi, le sol à sa gauche est au soleil : {soir_g} contre {matin_g}");
        let (sans_lune, pleine) = (sol(0.5, 0.0, OCX + 100.0), sol(0.5, 0.5, OCX + 100.0));
        assert!(pleine > sans_lune * 1.1, "la pleine lune jette son rai, la nouvelle non : {pleine} contre {sans_lune}");
    }

    /// La vasque se tient sur le sol, devant le mur : son pied descend dans les dalles, sa coupe passe devant la
    /// cimaise du lambris (user, 02-11 : « on dirait que c'est collé contre le mur »).
    #[test]
    fn la_vasque_se_tient_sur_le_sol() {
        assert!(bassin(BX, SOL + 12.0).is_some(), "le pied dans les dalles");
        assert!(bassin(BX, PIED + 1.0).is_none(), "et pas plus bas");
        assert!(BORD > LAMBRIS + 4.0 && bassin(BX + BW - 6.0, BORD + 3.0).is_some(), "la coupe devant le lambris, pas sur le mur");
    }

    /// La vision : l'habitant d'une session monte au-dessus de la vasque, à sa place, quelques secondes, puis
    /// s'en va ; au travail, elle prend la place de la sphère.
    #[test]
    fn la_vision_monte_de_la_vasque() {
        let (cw, ch) = (9, 19);
        let (pw, ph) = ((72 * cw) as usize, (16 * ch) as usize);
        // un petit habitant plein, rouge vif : ce qu'on cherche au-dessus de l'eau
        let vi = |age: f32| Vision {
            images: vec![vec![0xffff2020; 16 * 16]],
            w: 16,
            h: 16,
            posee: std::time::Instant::now(),
            nee: std::time::Instant::now().checked_sub(std::time::Duration::from_secs_f32(age)),
        };
        // le rouge sur la boîte de la vision (u autour de BX, v au-dessus du bord), moins le fond
        let rouge = |activite: Activite, v: Option<&Vision>| {
            let w = Monde { perso: Some(super::super::monde::Perso::Galadriel), hour: 23.0, clock: 40, ..Monde::default() };
            let mut b = Belvedere::default();
            let mut px = vec![0u32; pw * ph];
            let vue = Vue { monde: &w, activite, tick: 40, x0: 0, cols: 72, cw, ch, elle: Some(38.0), vision: v, dragon: None };
            b.peindre(&mut px, pw, ph, &vue, true);
            b.devant(&mut px, &vue);
            let mut r = 0.0;
            for y in (BORD - 90.0) as usize..(BORD - 20.0) as usize {
                for x in (BX - 30.0) as usize..(BX + 30.0) as usize {
                    let c = unpack(px[y * pw + x]);
                    r += (c[0] - c[1].max(c[2])).max(0.0);
                }
            }
            r
        };
        let sans = rouge(Activite::Idle, None);
        let (montee, pleine, finie) = (rouge(Activite::Idle, Some(&vi(0.3))), rouge(Activite::Idle, Some(&vi(4.0))), rouge(Activite::Idle, Some(&vi(VISION_S + 0.5))));
        let au_travail = rouge(Activite::Running, Some(&vi(4.0)));
        println!("rouge : sans {sans:.0}, à 0,3 s {montee:.0}, à 4 s {pleine:.0}, finie {finie:.0}, au travail {au_travail:.0}");
        assert!(pleine > sans + 20_000.0, "la vision se voit au-dessus de l'eau : {pleine} contre {sans}");
        assert!(montee < pleine * 0.7, "elle monte, d'abord à peine là : {montee} contre {pleine}");
        assert!((finie - sans).abs() < 1.0, "partie, elle ne laisse rien : {finie} contre {sans}");
        assert!(au_travail > pleine * 0.8, "au travail, elle prend la place de la sphère : {au_travail} contre {pleine}");
    }

    #[test]
    fn le_fond_se_peint_et_se_garde() {
        let mut b = Belvedere::default();
        let w = Monde { perso: Some(super::super::monde::Perso::Galadriel), hour: 23.0, ..Monde::default() };
        let (cw, ch) = (9, 19);
        let (pw, ph) = ((72 * cw) as usize, (16 * ch) as usize);
        let mut px = vec![0u32; pw * ph];
        let vue = Vue { monde: &w, activite: Activite::Idle, tick: 0, x0: 0, cols: 72, cw, ch, elle: Some(38.0), vision: None, dragon: None };
        let (z, _) = b.peindre(&mut px, pw, ph, &vue, true);
        assert_eq!(z.map(|z| (z.0, z.2)), Some((0, FIN * cw)));
        // le fond est opaque partout dans la région
        assert!((0..ph).all(|y| (0..(FIN * cw) as usize).all(|x| px[y * pw + x] >> 24 == 255)));
        // la fois d'après, seuls les pixels animés se repeignent
        let avant = b.fond.clone();
        let (z, _) = b.peindre(&mut px, pw, ph, &vue, false);
        assert_eq!(b.fond, avant, "le fond gardé");
        let z = z.expect("l'eau bouge toujours");
        assert!((z.2 - z.0) * (z.3 - z.1) < (FIN * cw) * ph as i32, "une image partielle");
    }
}
