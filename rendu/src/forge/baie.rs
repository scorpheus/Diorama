//! La Route du Rhum, peinte au vrai pixel de l'écran dans la cour de la session qui la montre :
//! le ciel de l'heure et ses cumulus, la mer et sa houle, la flotte (Ultim, Ocean Fifty, IMOCA,
//! Class40, et de petites voiles à l'horizon) qui tire des bords avant le départ et file vers
//! l'Atlantique après, sillages et embruns, et ce qu'on croise en route. Le décor suit la course :
//! le quai de granit de Saint-Malo, puis le pont de teck d'un bateau au large (les Açores au
//! passage, des dauphins, une baleine, des poissons volants et les sargasses des tropiques), puis
//! la plage de Guadeloupe, ses palmiers et la Soufrière.
//!
//! Les cellules de la cour y laissent leur fond transparent (cour.rs) : la baie se peint dessous,
//! et ce qui s'y tient (les petits, les invités, le chat, la pluie) reste par-dessus.

use std::f32::consts::PI;

use super::monde::{FeteRhum, Monde, PhaseRhum, Temps};
use super::pixels::hasard;
use super::trace::{bande, fois, hex, mix, mul, pack, smooth, unpack, Rgb, Toile};

const SEA: f32 = 17.0; // l'horizon, en pixels de forge
const GROUND: f32 = 28.0; // le sol, où se tiennent les acteurs

/// Où en est le voyage : le décor de la cour.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Etape {
    Bretagne,
    Large,
    Acores,
    Tropiques,
    Guadeloupe,
}

/// Le spectacle du lancement d'une session : au tic COUP, le canon tonne (ou le premier coupe la
/// ligne d'arrivée) ; à SORTIE après lui, les plus lents ont quitté la baie ; il dure FETE tics.
pub const COUP: f32 = 24.0;
const SORTIE: f32 = 110.0;
pub const FETE: i64 = 170;
/// La graine du bateau qui coupe la ligne d'arrivée : il ne fait pas partie de la flotte.
const HEROS: i64 = -7;

/// Le spectacle en cours, et son âge.
fn fete(w: &Monde) -> Option<(FeteRhum, f32)> {
    w.rhum_fete.as_ref().filter(|f| w.rhum.is_some() && (0..FETE).contains(&f.age)).map(|f| (f.kind, f.age as f32))
}

/// L'étape d'un monde : celle du spectacle s'il s'en joue un, sinon celle de la course.
pub fn etape(w: &Monde) -> Etape {
    match fete(w) {
        Some((FeteRhum::Depart, _)) => Etape::Bretagne,
        Some((FeteRhum::Arrivee, _)) => Etape::Guadeloupe,
        None => etape_de_la_course(w),
    }
}

/// Saint-Malo jusqu'au départ et dans les premières heures ; puis le large, les Açores au
/// passage, les tropiques, la Guadeloupe aux arrivées.
fn etape_de_la_course(w: &Monde) -> Etape {
    if w.rhum != Some(PhaseRhum::Course) {
        return Etape::Bretagne;
    }
    match w.rhum_progress.unwrap_or(0.0) {
        p if p < 0.05 => Etape::Bretagne,
        p if p < 0.24 => Etape::Large,
        p if p < 0.40 => Etape::Acores,
        p if p < 0.62 => Etape::Large,
        p if p < 0.9 => Etape::Tropiques,
        _ => Etape::Guadeloupe,
    }
}

/// Où se peint la baie : son rectangle à l'écran, la taille d'un pixel de forge, et les colonnes
/// de la scène qu'il couvre (`from` la porte, `to` le bord).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Cadre {
    pub left: i32,
    pub w: usize,
    pub h: usize,
    pub cw: f32,
    pub demi: f32,
    pub from: f32,
    pub to: f32,
}

impl Cadre {
    #[inline]
    fn x(&self, u: f32) -> f32 {
        (u - self.from) * self.cw
    }
    #[inline]
    fn y(&self, v: f32) -> f32 {
        v * self.demi
    }
    fn room(&self) -> f32 {
        self.to - self.from
    }
}

/// Le ciel d'une heure : ses deux bouts, la lumière qu'il donne, la nuit (0 à 1), le soleil bas.
#[derive(Clone, Copy)]
struct Ciel {
    haut: Rgb,
    bas: Rgb,
    amb: Rgb,
    nuit: f32,
    /// le soleil près de l'horizon : où (part de la largeur), à quelle hauteur (pixels de forge)
    soleil: Option<(f32, f32)>,
    orage: bool,
}

fn ciel(hour: f32, orage: bool, eclair: bool) -> Ciel {
    // (heure, haut, bas, lumière, nuit)
    const CLES: [(f32, u32, u32, [f32; 3], f32); 8] = [
        (0.0, 0x070b22, 0x1d2c58, [0.30, 0.36, 0.60], 1.0),
        (5.6, 0x070b22, 0x1d2c58, [0.30, 0.36, 0.60], 1.0),
        (6.8, 0x4a5a9e, 0xf6b58a, [0.95, 0.80, 0.75], 0.25),
        (8.5, 0x3d8fe0, 0xbfe3f7, [1.00, 1.00, 0.97], 0.0),
        (17.0, 0x3d8fe0, 0xbfe3f7, [1.00, 1.00, 0.97], 0.0),
        (18.6, 0x34407e, 0xf3a35e, [0.92, 0.72, 0.62], 0.3),
        (20.2, 0x1d2550, 0xa8506a, [0.55, 0.48, 0.62], 0.7),
        (21.4, 0x070b22, 0x1d2c58, [0.30, 0.36, 0.60], 1.0),
    ];
    let h = hour.rem_euclid(24.0);
    let i = CLES.iter().rposition(|k| k.0 <= h).unwrap_or(0);
    let (a, b) = (CLES[i], CLES.get(i + 1).copied().unwrap_or((24.0, CLES[0].1, CLES[0].2, CLES[0].3, CLES[0].4)));
    let k = smooth(0.0, 1.0, (h - a.0) / (b.0 - a.0));
    let mut c = Ciel {
        haut: mix(hex(a.1), hex(b.1), k),
        bas: mix(hex(a.2), hex(b.2), k),
        amb: mix(a.3, b.3, k),
        nuit: a.4 + (b.4 - a.4) * k,
        soleil: None,
        orage,
    };
    // le soleil se lève à gauche, se couche à droite, dans la mer
    if (6.0..8.6).contains(&h) {
        c.soleil = Some((0.22, SEA + 0.6 - (h - 6.0) * 4.0));
    } else if (16.6..19.3).contains(&h) {
        c.soleil = Some((0.78, SEA - 8.0 + (h - 16.6) * 3.2));
    }
    if orage {
        let gris = if c.nuit > 0.5 { (hex(0x0d0f16), hex(0x262a36)) } else { (hex(0x3a3f4c), hex(0x7a808c)) };
        c.haut = mix(c.haut, gris.0, 0.85);
        c.bas = mix(c.bas, gris.1, 0.85);
        c.amb = mix(c.amb, [0.58, 0.61, 0.68], 0.7);
        c.soleil = None;
    }
    if eclair {
        c.haut = mix(c.haut, hex(0xc8ccdc), 0.5);
        c.bas = mix(c.bas, hex(0xe0e4f0), 0.5);
        c.amb = mix(c.amb, [1.0, 1.0, 1.1], 0.5);
    }
    c
}

/// Les sortes de bateaux de la course.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sorte {
    Ultim,
    OceanFifty,
    Imoca,
    Class40,
    Rhum,
}

/// Un bateau de la flotte peinte : sa sorte, ses couleurs, la rangée de sa ligne de flottaison,
/// son allure (pixels de forge par tic), sa graine.
#[derive(Clone, Debug)]
pub struct Bateau {
    pub sorte: Sorte,
    pub coque: Rgb,
    pub voile: Rgb,
    pub bande: Rgb,
    pub lane: f32,
    pub vitesse: f32,
    pub graine: i64,
}

/// La classe de course d'une sorte de bateau, comme le pointage les numérote.
fn classe(s: Sorte) -> u32 {
    match s {
        Sorte::Ultim => 0,
        Sorte::OceanFifty => 1,
        Sorte::Imoca => 2,
        Sorte::Class40 => 3,
        Sorte::Rhum => 4,
    }
}

/// La flotte qu'une baie de cette largeur montre, de l'horizon vers nous ; le plus proche de
/// chaque classe aux couleurs de son premier, quand le pointage les donne.
pub fn flotte(room: f32, tetes: Option<&[[u32; 3]]>) -> Vec<Bateau> {
    let mut f = flotte_type(room);
    for t in tetes.unwrap_or(&[]) {
        let classe_t = if t[0] == 5 { 4 } else { t[0] };
        if let Some(b) = f.iter_mut().rev().find(|b| classe(b.sorte) == classe_t) {
            (b.coque, b.bande) = (hex(t[1]), hex(t[2]));
        }
    }
    f
}

fn flotte_type(room: f32) -> Vec<Bateau> {
    let b = |sorte, coque: u32, voile: u32, bande: u32, lane, vitesse, graine| Bateau { sorte, coque: hex(coque), voile: hex(voile), bande: hex(bande), lane, vitesse, graine };
    let mut f = vec![
        b(Sorte::Rhum, 0xf0f0f0, 0xf4f2ea, 0xc0392b, 17.7, 0.035, 3),
        b(Sorte::Class40, 0x2a3a6a, 0xf4f2ea, 0x2a3a6a, 18.1, 0.045, 7),
        b(Sorte::Rhum, 0xd04a2a, 0xf4f2ea, 0xf4f2ea, 18.6, 0.04, 11),
        b(Sorte::Class40, 0xf2c14e, 0xf6f6f2, 0x3f80dc, 20.4, 0.12, 29),
        b(Sorte::Imoca, 0x24242c, 0xe8e8ea, 0xd04a2a, 23.4, 0.2, 41),
    ];
    if room >= 26.0 {
        f.insert(3, b(Sorte::Class40, 0x3c8a3c, 0xf6f6f2, 0x3c8a3c, 19.3, 0.08, 17));
        f.insert(5, b(Sorte::OceanFifty, 0xf4f4f4, 0xf6f6f2, 0x7a3ab0, 21.9, 0.17, 37));
    }
    if room >= 30.0 {
        f.push(b(Sorte::Ultim, 0x1d3f8a, 0xf8f8f6, 0x1d3f8a, 26.2, 0.33, 59));
    }
    if room >= 64.0 {
        f.insert(f.len() - 1, b(Sorte::Ultim, 0xc0392b, 0xf8f8f6, 0xc0392b, 25.3, 0.3, 83));
    }
    f
}

/// Combien un bateau est loin : son échelle, de l'horizon (petit) à nous (grand).
fn echelle(lane: f32) -> f32 {
    0.14 + 0.86 * ((lane - SEA) / (GROUND - SEA)).clamp(0.0, 1.0)
}

/// Sa longueur et la hauteur de son mât, en pixels de forge, tout près.
fn gabarit(s: Sorte) -> (f32, f32) {
    match s {
        Sorte::Ultim => (22.0, 1.0),
        Sorte::OceanFifty => (13.0, 1.15),
        Sorte::Imoca => (13.0, 1.55),
        Sorte::Class40 => (11.0, 1.5),
        Sorte::Rhum => (10.0, 1.35),
    }
}

/// Où est un bateau à `t` (son milieu, pixels de forge) et dans quel sens il va. En course, il
/// traverse la baie vers l'Atlantique et repasse plus tard ; avant, il tire des bords.
pub fn cap(b: &Bateau, t: f32, phase: PhaseRhum, from: f32, to: f32, taille: f32) -> (f32, f32) {
    let l = gabarit(b.sorte).0 * echelle(b.lane) * taille;
    let room = to - from;
    if phase == PhaseRhum::Course {
        let tour = room + l + 8.0 + (hasard(b.graine) as f32) * 20.0;
        return (from - l / 2.0 + (t * b.vitesse + hasard(b.graine * 7) as f32 * tour).rem_euclid(tour), 1.0);
    }
    let portee = ((room - l) / 3.0).clamp(1.0, 14.0);
    let centre = from + 2.0 + l / 2.0 + hasard(b.graine * 3) as f32 * (room - l - 4.0).max(0.0);
    let a = t * b.vitesse * 0.45 / portee + hasard(b.graine * 11) as f32 * 6.28;
    ((centre + a.sin() * portee).clamp(from + 2.0 + l / 2.0, to - l / 2.0), if a.cos() >= 0.0 { 1.0 } else { -1.0 })
}

/// Le départ rejoué : alignés sous la ligne jusqu'au coup de canon, puis lâchés vers le large,
/// les plus vifs devant. Quand le vrai départ n'est pas encore donné, ils reviennent ensuite de
/// la droite tirer leurs bords (`retour` : où leur cap ordinaire les met).
fn au_depart(b: &Bateau, a: f32, from: f32, to: f32, taille: f32, retour: Option<(f32, f32)>) -> (f32, f32, PhaseRhum) {
    let l = gabarit(b.sorte).0 * echelle(b.lane) * taille;
    // étalés le long de la ligne, du bateau-comité à la bouée du large
    let room = to - from;
    let ligne = (from + room * (0.22 + 0.55 * hasard(b.graine * 5) as f32)).clamp(from + l / 2.0, to - l / 2.0);
    if a < COUP {
        return (ligne + (a * 0.09 + b.graine as f32).sin() * 0.6, 1.0, PhaseRhum::Avant);
    }
    let d = a - COUP;
    let duree = SORTIE * (1.0 - 0.5 * (b.vitesse / 0.33).clamp(0.1, 1.0));
    let loin = to + l + 4.0;
    if d < duree {
        let f = d / duree;
        return (ligne + (loin - ligne) * f * f, 1.0, PhaseRhum::Course);
    }
    match retour {
        Some((u, dir)) if d >= SORTIE => {
            let f = ((d - SORTIE) / (FETE as f32 - COUP - SORTIE)).clamp(0.0, 1.0);
            let e = f * f * (3.0 - 2.0 * f);
            (loin + (u - loin) * e, if f < 1.0 { -1.0 } else { dir }, PhaseRhum::Avant)
        }
        _ => (loin + 50.0, 1.0, PhaseRhum::Course),
    }
}

/// Où la ligne d'arrivée coupe la baie, en colonnes depuis la porte.
fn ligne_d_arrivee(c: &Cadre) -> f32 {
    c.room() * 0.55
}

/// Ce que la baie garde d'une image à l'autre : le fond immobile, et la table des sinus.
#[derive(Default)]
pub struct Baie {
    cle: Option<(usize, usize, i32, i32, i32, i32, Etape, i32, bool, bool)>,
    fond: Vec<u32>,
    /// le fond et ses nuages, repeints quand ils ont glissé d'un pixel
    ciel: Vec<u32>,
    ciel_pas: Option<i64>,
    gardees: Vignettes,
    /// par rangée de mer, les sinus et cosinus des deux houles en chaque pixel
    ondes: std::cell::RefCell<(usize, Vec<[f32; 4]>)>,
    houles: std::cell::RefCell<Vec<f32>>,
    buf: Vec<u32>,
    sinus: Vec<f32>,
}

const N_SIN: usize = 1024;

impl Baie {
    #[inline]
    fn sin(&self, a: f32) -> f32 {
        self.sinus[((a * (N_SIN as f32 / (2.0 * PI))) as i32 & (N_SIN as i32 - 1)) as usize]
    }

    /// Peint la baie sous les cellules transparentes du rectangle `c` de `pixels` ; rend ce rectangle.
    pub fn peindre(&mut self, pixels: &mut [u32], pw: usize, ph: usize, c: Cadre, w: &Monde) -> Option<(i32, i32, i32, i32)> {
        if c.w == 0 || c.h == 0 || w.rhum.is_none() {
            return None;
        }
        if self.sinus.is_empty() {
            self.sinus = (0..N_SIN).map(|i| (i as f32 * 2.0 * PI / N_SIN as f32).sin()).collect();
        }
        let et = etape(w);
        let orage = w.weather == Some(Temps::Storm);
        let eclair = orage && w.clock % 23 < 2;
        let sky = ciel(w.hour as f32, orage, eclair);
        let cle = (c.w, c.h, (c.cw * 100.0) as i32, (c.demi * 100.0) as i32, c.from as i32, c.to as i32, et, (w.hour * 10.0) as i32, orage, eclair);
        if self.cle != Some(cle) {
            self.fond = vec![0; c.w * c.h];
            fond(&mut Toile { px: &mut self.fond, w: c.w, h: c.h }, &c, &sky, et);
            self.cle = Some(cle);
            self.ciel_pas = None;
            self.gardees.clear();
            self.ondes.borrow_mut().1.clear();
            *self.houles.borrow_mut() = vec![0.0; c.w];
        }
        // les nuages glissent lentement : repeints sur le fond quand le plus vif a fait un pixel
        let t = w.clock as f32;
        let pas = (t * vitesse_nuages(&sky) * 1.3 * c.cw).floor() as i64;
        if self.ciel_pas != Some(pas) {
            self.ciel.clear();
            self.ciel.extend_from_slice(&self.fond);
            let tq = pas as f32 / (vitesse_nuages(&sky) * 1.3 * c.cw);
            nuages(&mut Toile { px: &mut self.ciel, w: c.w, h: c.h }, &c, &sky, et, tq);
            self.ciel_pas = Some(pas);
        }
        let mut buf = std::mem::take(&mut self.buf);
        buf.clear();
        buf.extend_from_slice(&self.ciel);
        {
            let mut toile = Toile { px: &mut buf, w: c.w, h: c.h };
            self.mer(&mut toile, &c, &sky, et, t);
            decor_anime(&mut toile, &mut self.gardees, &c, &sky, et, w, t);
            let taille = (c.room() / 40.0).clamp(0.55, 1.0);
            let phase = w.phase();
            let spectacle = fete(w);
            // de l'horizon vers nous : les bateaux et les bêtes, chacun à sa profondeur
            let la_flotte = flotte(c.room(), w.rhum_tetes.as_deref());
            let mut tous: Vec<(f32, Option<Bateau>, Option<Box<dyn Fn(&mut Toile)>>)> = la_flotte.iter().cloned().map(|b| (b.lane, Some(b), None)).collect();
            // à l'arrivée, le premier Ultim, à ses couleurs, coupe la ligne au plus près
            // (à sa place : on ne le verrait pas deux fois)
            if let (Some((FeteRhum::Arrivee, _)), Some(premier)) = (spectacle, la_flotte.iter().rev().find(|b| b.sorte == Sorte::Ultim).or(la_flotte.last())) {
                tous.retain(|(_, b, _)| b.as_ref().is_none_or(|b| b.graine != premier.graine));
                let heros = Bateau { lane: 27.2, graine: HEROS, ..premier.clone() };
                tous.push((heros.lane, Some(heros), None));
            }
            tous.extend(faune(&c, &sky, et, t).into_iter().map(|(lane, f)| (lane, None, Some(f))));
            tous.sort_by(|a, b| a.0.total_cmp(&b.0));
            if self.gardees.len() > 400 {
                self.gardees.clear();
            }
            for (_, b, f) in &tous {
                if let Some(b) = b {
                    let ordinaire = || cap(b, t, phase, c.from, c.to, taille);
                    let (u, dir, phase) = match spectacle {
                        Some((FeteRhum::Depart, a)) => {
                            let retour = (phase == PhaseRhum::Avant && etape_de_la_course(w) == Etape::Bretagne).then(ordinaire);
                            au_depart(b, a, c.from, c.to, taille, retour)
                        }
                        Some((FeteRhum::Arrivee, a)) if b.graine == HEROS => {
                            let l = gabarit(b.sorte).0 * echelle(b.lane) * taille;
                            let (debut, ligne) = (c.from - l, c.from + ligne_d_arrivee(&c));
                            (debut + (ligne - debut) * a / COUP, 1.0, PhaseRhum::Course)
                        }
                        _ => {
                            let (u, dir) = ordinaire();
                            (u, dir, phase)
                        }
                    };
                    // la nuit au large, la mer s'allume dans les sillages
                    let phosphore = sky.nuit > 0.6 && matches!(et, Etape::Large | Etape::Acores | Etape::Tropiques);
                    bateau(&mut toile, &mut self.gardees, &c, &sky, b, (u, dir), t, phase, taille, phosphore);
                }
                if let Some(f) = f {
                    f(&mut toile);
                }
            }
            premier_plan(&mut toile, &mut self.gardees, &c, &sky, et, w, t);
        }
        // sous les cellules transparentes seulement : ce qui se tient dans la cour reste devant
        let (x0, y1) = (c.left.max(0) as usize, c.h.min(ph));
        let large = c.w.min(pw.saturating_sub(x0));
        for y in 0..y1 {
            let ligne = &mut pixels[y * pw + x0..y * pw + x0 + large];
            for (d, &s) in ligne.iter_mut().zip(&buf[y * c.w..y * c.w + large]) {
                *d = if *d >> 24 == 0 { s } else { *d };
            }
        }
        self.buf = buf;
        Some((c.left, 0, c.left + c.w as i32, c.h as i32))
    }

    /// Les sinus et cosinus des deux houles d'une rangée de mer, en chaque pixel : calculés une fois.
    fn ondes(&self, y: usize, c: &Cadre) -> std::cell::Ref<'_, [[f32; 4]]> {
        {
            let mut o = self.ondes.borrow_mut();
            if o.1.is_empty() {
                o.0 = (c.y(SEA).ceil() as usize).min(c.h);
                let rangees = (c.y(GROUND).ceil() as usize).min(c.h).saturating_sub(o.0);
                o.1 = vec![[0.0; 4]; rangees * c.w];
                for j in 0..rangees {
                    let v = (o.0 + j) as f32 / c.demi;
                    let d = ((v - SEA) / (GROUND - SEA)).clamp(0.0, 1.0);
                    let k = 1.0 / (0.25 + 1.5 * d); // au loin, des vagues serrées
                    for x in 0..c.w {
                        let u = c.from + x as f32 / c.cw;
                        let (x1, x2) = (u * 0.9 * k, u * 2.3 * k);
                        o.1[j * c.w + x] = [x1.sin(), x1.cos(), x2.sin(), x2.cos()];
                    }
                }
            }
        }
        std::cell::Ref::map(self.ondes.borrow(), |o| &o.1[(y - o.0) * c.w..(y - o.0 + 1) * c.w])
    }

    /// La mer : sa couleur au fond immobile, la houle et ses crêtes par-dessus, image après image ;
    /// le reflet du soleil ou de la lune, et le bord qui écume (le quai, la plage).
    fn mer(&self, t_: &mut Toile, c: &Cadre, sky: &Ciel, et: Etape, t: f32) {
        let (ya, yb) = (c.y(SEA).ceil() as usize, (c.y(GROUND).ceil() as usize).min(c.h));
        let houle = if sky.orage { 1.8 } else if et == Etape::Large || et == Etape::Acores { 1.25 } else { 1.0 };
        let blanc = mul(hex(0xf2fbff), sky.amb);
        let (reflet, ru) = match sky.soleil {
            Some((f, v)) if v < SEA + 0.5 => (Some(mix(hex(0xffd27a), hex(0xff8a4a), smooth(SEA - 8.0, SEA, v))), c.from + f * c.room()),
            _ if sky.nuit > 0.6 && !sky.orage => (Some(hex(0xdfe6ff)), c.from + 0.8 * c.room()),
            _ => (None, 0.0),
        };
        for y in ya..yb {
            let v = y as f32 / c.demi;
            let d = ((v - SEA) / (GROUND - SEA)).clamp(0.0, 1.0);
            let ph = v * 7.3;
            let amp = (0.05 + 0.07 * d) * houle;
            // deux houles : sin(f·x + φ) = sin(f·x)·cos φ + cos(f·x)·sin φ, les sinus de la
            // rangée gardés ; seul le temps les fait tourner
            let ondes = self.ondes(y, c);
            let (p1, p2) = (t * 0.07 + ph, -t * 0.11 + ph * 1.7);
            let (a1, b1, a2, b2) = (0.6 * p1.cos(), 0.6 * p1.sin(), 0.4 * p2.cos(), 0.4 * p2.sin());
            let mut hb = self.houles.borrow_mut();
            let houles = &mut hb[..c.w];
            for (h, o) in houles.iter_mut().zip(ondes.iter()) {
                *h = o[0] * a1 + o[1] * b1 + o[2] * a2 + o[3] * b2;
            }
            // la couleur de la rangée selon la houle, en table : la mer à plat de la rangée,
            // assombrie au creux, éclaircie et blanchie à la crête
            let fond_px = t_.px[y * c.w + c.w - 1];
            let colorer = |base: Rgb, s: f32| {
                let mut col = fois(base, 1.0 + s * amp);
                let crete = s - (0.72 - 0.12 * (houle - 1.0));
                if crete > 0.0 {
                    col = mix(col, blanc, (crete * 3.0).min(1.0) * (0.25 + 0.4 * d));
                }
                col
            };
            let table: [u32; 64] = std::array::from_fn(|q| pack(colorer(unpack(fond_px), q as f32 / 31.5 - 1.0)));
            let ligne = &mut t_.px[y * c.w..(y + 1) * c.w];
            for (p, &s) in ligne.iter_mut().zip(houles.iter()) {
                if *p == fond_px {
                    *p = table[((s + 1.0) * 31.5) as usize & 63];
                } else {
                    *p = pack(colorer(unpack(*p), s)); // la côte, l'îlot, le halo du soleil
                }
            }
            // le reflet du soleil ou de la lune, une colonne qui scintille
            if let Some(r) = reflet {
                let large = 0.6 + d * 4.5;
                let (xr0, xr1) = (((ru - large - c.from) * c.cw).max(0.0) as usize, (((ru + large - c.from) * c.cw).max(0.0) as usize).min(c.w));
                for x in xr0..xr1 {
                    let s = houles[x];
                    if s > 0.15 {
                        let du = (c.from + x as f32 / c.cw - ru).abs();
                        let a = ((1.0 - du / large) * (s - 0.15) * 1.4).clamp(0.0, 0.85);
                        ligne[x] = pack(mix(unpack(ligne[x]), r, a));
                    }
                }
            }
        }
        // des éclats ici et là, qui scintillent
        let n = c.room() as i64;
        let tic = (t / 2.0).floor() as i64;
        for k in 0..n {
            let (x, y) = (hasard(k * 7919 + tic * 31) as f32 * c.w as f32, ya as f32 + hasard(k * 104_729 + tic * 17) as f32 * (yb - ya) as f32);
            t_.disque(x, y, 0.9, blanc, 0.75, 0.3);
        }
        // le bord : l'écume au pied du quai, la vague qui monte sur le sable
        match et {
            Etape::Bretagne => {
                // une frange d'écume qui va et vient le long du pied du quai
                let yw = c.y(GROUND - 0.35);
                let (ya, yb) = ((yw - 4.0).max(0.0) as usize, ((yw + 3.0) as usize).min(t_.h));
                for x in 0..c.w {
                    let u = c.from + x as f32 / c.cw;
                    let a = 0.5 + 0.5 * self.sin(u * 2.7 + t * 0.21) * self.sin(u * 0.9 - t * 0.13);
                    if a <= 0.35 {
                        continue;
                    }
                    let haut = yw + self.sin(u * 3.0 + t * 0.3) * 1.5 - (1.0 + a * 1.5);
                    for y in ya..yb {
                        let k = (y as f32 + 1.0 - haut).clamp(0.0, 1.0) * a * 0.75;
                        if k > 0.0 {
                            let i = y * c.w + x;
                            t_.px[i] = pack(mix(unpack(t_.px[i]), blanc, k));
                        }
                    }
                }
            }
            Etape::Guadeloupe => {
                // la vague monte, s'étale sur le sable mouillé, se retire
                let montee = 0.5 + 0.5 * self.sin(t * 0.09);
                let yv = c.y(GROUND - 0.6 + montee * 0.9);
                let mut bord = Vec::new();
                for x in (0..=c.w).step_by(3) {
                    let u = c.from + x as f32 / c.cw;
                    bord.push((x as f32, yv + self.sin(u * 1.3 + t * 0.05) * 2.0 + self.sin(u * 3.7) * 1.0));
                }
                let mut nappe = bord.clone();
                nappe.push((c.w as f32, c.y(GROUND - 0.6)));
                nappe.push((0.0, c.y(GROUND - 0.6)));
                t_.poly(&nappe, mul(hex(0x7fe0d8), sky.amb), 0.55);
                t_.ligne(&bord, 2.2, blanc, 0.85);
            }
            _ => {}
        }
    }
}

impl Monde {
    /// La phase de la course telle qu'elle se peint : avant le départ, ou en course.
    pub fn phase(&self) -> PhaseRhum {
        self.rhum.unwrap_or(PhaseRhum::Course)
    }
}

/// Le fond immobile : le ciel, le soleil ou la lune, les étoiles, les terres à l'horizon, la mer
/// à plat, le sol (quai, pont ou plage), la lueur de la forge à la porte le soir.
fn fond(t: &mut Toile, c: &Cadre, sky: &Ciel, et: Etape) {
    let room = c.room();
    let (mer_loin, mer_pres) = match et {
        _ if sky.orage => (hex(0x3a4a56), hex(0x46565f)),
        Etape::Bretagne => (hex(0x3f7590), hex(0x2a6774)),
        Etape::Large | Etape::Acores => (hex(0x2c5f9e), hex(0x1d4a86)),
        Etape::Tropiques => (hex(0x2b86c4), hex(0x1f6fb0)),
        Etape::Guadeloupe => (hex(0x3cbfd0), hex(0x2fcfc6)),
    };
    for y in 0..t.h {
        let v = (y as f32 + 0.5) / c.demi;
        let ligne = if v < SEA {
            let k = (v / SEA).clamp(0.0, 1.0);
            mix(sky.haut, sky.bas, k.powf(1.4))
        } else {
            let d = ((v - SEA) / (GROUND - SEA)).clamp(0.0, 1.0);
            // au loin, la mer se fond dans le bas du ciel
            mix(mix(mul(mer_loin, sky.amb), sky.bas, 0.35 * (1.0 - d).powi(2)), mul(mer_pres, sky.amb), d)
        };
        let p = pack(ligne);
        t.px[y * t.w..(y + 1) * t.w].fill(p);
    }
    // les étoiles
    if sky.nuit > 0.4 && !sky.orage {
        for i in 0..(room * 1.2) as i64 {
            let (u, v) = (hasard(i * 7 + 1) as f32 * room, hasard(i * 13 + 5) as f32 * (SEA - 4.0));
            let k = hasard(i * 3 + 2) as f32;
            t.disque(u * c.cw, c.y(v), 0.6 + k * 0.7, mix(hex(0xaab4e0), hex(0xfff6dc), k), (sky.nuit - 0.4) * 1.6 * (0.4 + k * 0.6), 0.5);
        }
    }
    // le soleil bas, son halo ; la lune
    if let Some((f, v)) = sky.soleil {
        let (x, y) = (f * room * c.cw, c.y(v));
        t.disque(x, y, 6.0 * c.demi, hex(0xffc070), 0.35, 1.0);
        t.disque(x, y, 1.6 * c.demi, mix(hex(0xfff2c0), hex(0xff9a50), smooth(SEA - 8.0, SEA, v)), 1.0, 0.0);
    } else if sky.nuit > 0.6 && !sky.orage {
        let (x, y) = (0.8 * room * c.cw, c.y(4.5));
        t.disque(x, y, 4.0 * c.demi, hex(0x8090c0), 0.18, 1.0);
        t.disque(x, y, 1.3 * c.demi, hex(0xf2f0e2), 1.0, 0.0);
        t.disque(x + 0.5 * c.demi, y - 0.3 * c.demi, 1.1 * c.demi, mix(sky.haut, hex(0xf2f0e2), 0.15), 1.0, 0.0); // en croissant
    }
    // ce qu'on voit à l'horizon
    let loin = |col: u32, k: f32| mix(mul(hex(col), sky.amb), sky.bas, k);
    let h = SEA;
    match et {
        Etape::Bretagne => {
            // la côte d'en face, basse, de Dinard à la pointe ; l'îlot et son phare au large à droite
            let mut cote = vec![(0.0, c.y(h + 0.2))];
            for i in 0..=40 {
                let u = i as f32 / 40.0 * room * 0.62;
                let haut = 1.0 + 0.7 * (u * 0.31).sin() + 0.4 * (u * 0.9).sin() - u / room * 1.2;
                cote.push((u * c.cw, c.y(h - haut.max(0.2))));
            }
            cote.push((room * 0.62 * c.cw, c.y(h + 0.2)));
            t.poly(&cote, loin(0x4a5a6a, 0.45), 1.0);
            if room >= 40.0 {
                let ux = room - 10.0;
                let ile: Vec<(f32, f32)> = (0..=16)
                    .map(|i| {
                        let a = i as f32 / 16.0 * PI;
                        ((ux + 6.5 * a.cos()) * c.cw, c.y(h + 0.3 - 2.4 * a.sin().powf(0.7)))
                    })
                    .collect();
                t.poly(&ile, loin(0x5d5866, 0.3), 1.0);
                // le phare : la tour blanche, ses bandes rouges, la lanterne (allumée plus loin)
                let (x, base, sommet) = (ux * c.cw, c.y(h - 1.8), c.y(h - 10.2));
                t.poly(&[(x - 0.7 * c.cw, base), (x + 0.7 * c.cw, base), (x + 0.5 * c.cw, sommet), (x - 0.5 * c.cw, sommet)], loin(0xeef0f4, 0.2), 1.0);
                for k in [0.15, 0.55] {
                    let (ya, yb) = (base + (sommet - base) * k, base + (sommet - base) * (k + 0.18));
                    t.poly(&[(x - 0.66 * c.cw, ya), (x + 0.66 * c.cw, ya), (x + 0.62 * c.cw, yb), (x - 0.62 * c.cw, yb)], loin(0xc0392b, 0.2), 1.0);
                }
                t.poly(&[(x - 0.75 * c.cw, sommet), (x + 0.75 * c.cw, sommet), (x + 0.75 * c.cw, sommet + 1.5), (x - 0.75 * c.cw, sommet + 1.5)], loin(0x2a2a30, 0.2), 1.0);
                t.poly(&[(x - 0.45 * c.cw, sommet - 1.2 * c.demi), (x + 0.45 * c.cw, sommet - 1.2 * c.demi), (x + 0.45 * c.cw, sommet), (x - 0.45 * c.cw, sommet)], loin(0x9ab0c0, 0.2), 1.0);
                t.poly(&[(x - 0.6 * c.cw, sommet - 1.2 * c.demi), (x + 0.6 * c.cw, sommet - 1.2 * c.demi), (x, sommet - 2.0 * c.demi)], loin(0x2a2a30, 0.2), 1.0);
            }
        }
        Etape::Acores => {
            // Pico, le volcan, et une île basse : bleus de lointain
            let ux = room * 0.68;
            let pico: Vec<(f32, f32)> = (0..=30)
                .map(|i| {
                    let a = i as f32 / 30.0 * 2.0 - 1.0;
                    ((ux + a * 9.0) * c.cw, c.y(h + 0.2 - 8.5 * (1.0 - a.abs()).powf(1.5)))
                })
                .collect();
            t.poly(&pico, loin(0x3a4a5e, 0.55), 1.0);
            let ile: Vec<(f32, f32)> = (0..=20).map(|i| {
                let a = i as f32 / 20.0;
                ((room * 0.12 + a * 14.0) * c.cw, c.y(h + 0.2 - 1.6 * (a * PI).sin()))
            }).collect();
            t.poly(&ile, loin(0x3a4a5e, 0.65), 1.0);
        }
        Etape::Tropiques => {
            // les cumulus des alizés, en tours sur l'horizon
            for i in 0..((room / 30.0) as i64 + 1) {
                let u = (0.1 + 0.8 * hasard(i * 17 + 3) as f32) * room;
                cumulus(t, c, sky, u, SEA - 0.3, 1.6 + 1.0 * hasard(i * 5) as f32, i * 31 + 7, 0.7);
            }
        }
        Etape::Guadeloupe => {
            // Basse-Terre : ses mornes, la Soufrière coiffée de son nuage ; la côte, plus près
            let mut monts = vec![(room * 0.18 * c.cw, c.y(h + 0.2))];
            for i in 0..=60 {
                let a = i as f32 / 60.0;
                let u = room * (0.18 + 0.82 * a);
                let dome = (-(((a - 0.55) / 0.13).powi(2))).exp() * 4.5;
                let haut = 1.5 + 2.0 * (a * PI).sin() + dome + 0.4 * (u * 0.7).sin();
                monts.push((u * c.cw, c.y(h - haut)));
            }
            monts.push((room * c.cw, c.y(h + 0.2)));
            t.poly(&monts, loin(0x2f5a4a, 0.5), 1.0);
            let mut cote = vec![(room * 0.3 * c.cw, c.y(h + 0.3))];
            for i in 0..=40 {
                let a = i as f32 / 40.0;
                let u = room * (0.3 + 0.7 * a);
                cote.push((u * c.cw, c.y(h - 1.1 - 0.5 * (u * 1.1).sin())));
            }
            cote.push((room * c.cw, c.y(h + 0.3)));
            t.poly(&cote, loin(0x2e6a3a, 0.25), 1.0);
            cumulus(t, c, sky, room * 0.73, h - 7.6, 2.4, 91, 0.75);
            // le soir, les lumières de la côte
            if sky.nuit > 0.3 {
                for i in 0..(room * 0.5) as i64 {
                    let u = room * (0.32 + 0.66 * hasard(i * 3 + 1) as f32);
                    t.disque(u * c.cw, c.y(h - 0.6), 0.9, hex(0xffd890), sky.nuit, 0.4);
                }
            }
        }
        Etape::Large => {}
    }
    // le sol : le quai de Saint-Malo, le pont du bateau au large, la plage à l'arrivée
    let (y0, y1) = (c.y(GROUND).floor() as usize, t.h);
    for y in y0..y1 {
        for x in 0..t.w {
            let (u, v) = (c.from + x as f32 / c.cw, y as f32 / c.demi);
            let col = match et {
                Etape::Bretagne => granit(u, v, c),
                Etape::Guadeloupe => sable(u, v),
                _ => teck(u, v, c),
            };
            t.px[y * t.w + x] = pack(mul(col, sky.amb));
        }
    }
    match et {
        Etape::Bretagne if room >= 26.0 => {
            // une bitte d'amarrage
            let x = 19.0 * c.cw;
            let (yb, yt) = (c.y(GROUND + 0.5), c.y(GROUND - 2.2));
            t.poly(&[(x - 0.8 * c.cw, yb), (x + 0.8 * c.cw, yb), (x + 0.6 * c.cw, yt), (x - 0.6 * c.cw, yt)], mul(hex(0x24242c), sky.amb), 1.0);
            t.disque(x, yt, 1.1 * c.cw, mul(hex(0x3a3a44), sky.amb), 1.0, 0.0);
        }
        Etape::Large | Etape::Acores | Etape::Tropiques => {
            // la filière du bateau : les chandeliers, les deux filières d'inox
            let haut = c.y(GROUND - 2.6);
            let inox = mul(hex(0xd8dde4), sky.amb);
            let mut u = 3.0;
            while u < room {
                t.trait_((u * c.cw, c.y(GROUND)), (u * c.cw, haut), 0.22 * c.cw, inox, 1.0);
                u += 7.0;
            }
            t.trait_((0.0, haut), (room * c.cw, haut), 0.16 * c.cw, inox, 0.95);
            t.trait_((0.0, c.y(GROUND - 1.3)), (room * c.cw, c.y(GROUND - 1.3)), 0.12 * c.cw, inox, 0.85);
        }
        _ => {}
    }
    // la corde du pavois (ses fanions claquent au vent, plus loin)
    if et == Etape::Bretagne && room - 13.0 >= 4.0 {
        let (x0, x1) = (12.0, room - 1.0);
        let mut fil = Vec::new();
        let mut u = x0;
        while u <= x1 {
            fil.push((u * c.cw, c.y(2.0 + 2.5 * (PI * (u - x0) / (x1 - x0)).sin())));
            u += 0.5;
        }
        t.ligne(&fil, 1.1, mul(hex(0x3a3a44), sky.amb), 1.0);
    }
    // la lueur de la forge par la porte, le soir : une flaque chaude au sol
    let lueur = sky.nuit.min(1.0) * 0.55;
    if lueur > 0.0 {
        for y in c.y(22.0) as usize..t.h {
            for x in 0..t.w.min((15.0 * c.cw) as usize) {
                let (u, v) = (x as f32 / c.cw, y as f32 / c.demi);
                let d = (u / 1.6).hypot((v - 31.0) * 1.4);
                let f = (1.0 - d / 14.0).max(0.0).powi(2) * lueur;
                if f > 0.0 {
                    let i = y * t.w + x;
                    let p = unpack(t.px[i]);
                    t.px[i] = pack([p[0] + 230.0 * f, p[1] + 135.0 * f, p[2] + 63.0 * f]);
                }
            }
        }
    }
}

/// Le granit du quai : des pierres de taille, leurs joints, le couronnement le long du bord.
fn granit(u: f32, v: f32, c: &Cadre) -> Rgb {
    let rang = ((v - GROUND) * 1.2).floor() as i64;
    let decale = if rang % 2 == 0 { 0.0 } else { 1.7 };
    let pierre = ((u + decale) / 3.4).floor() as i64;
    let (fu, fv) = (((u + decale) / 3.4).fract(), ((v - GROUND) * 1.2).fract());
    let joint = fu * 3.4 * c.cw < 1.2 || fv * c.demi / 1.2 < 1.0;
    if v < GROUND + 0.45 {
        return if (u / 5.0).fract() < 0.03 { hex(0x5d5866) } else { mix(hex(0xb4b0ba), hex(0x9a96a2), (v - GROUND) / 0.45) };
    }
    if joint {
        return hex(0x4a4450);
    }
    let k = hasard(pierre * 131 + rang * 17) as f32;
    let grain = hasard((u * 4.0) as i64 * 7 + (v * 6.0) as i64 * 13) as f32;
    fois(mix(hex(0x8a8592), hex(0x726c7a), k), 0.92 + grain * 0.12)
}

/// Le pont : des lattes de teck, leurs joints noirs, en fuyant vers nous.
fn teck(u: f32, v: f32, c: &Cadre) -> Rgb {
    let latte = ((v - GROUND) * 1.6).floor() as i64;
    let f = ((v - GROUND) * 1.6).fract();
    if f * c.demi / 1.6 < 1.0 {
        return hex(0x2a2018);
    }
    let bout = ((u + latte as f32 * 5.3) / 11.0).fract() < 0.012;
    if bout {
        return hex(0x3a2c20);
    }
    let k = hasard(latte * 31 + ((u + latte as f32 * 5.3) / 11.0).floor() as i64 * 7) as f32;
    let veine = 0.5 + 0.5 * (u * 2.1 + (v * 9.0).sin() * 0.8 + k * 6.0).sin();
    mix(mix(hex(0xb07a48), hex(0x9a6436), k), hex(0xc08a58), veine * 0.25)
}

/// Le sable de la plage, son grain, mouillé près de l'eau.
fn sable(u: f32, v: f32) -> Rgb {
    let grain = hasard((u * 7.0) as i64 * 31 + (v * 9.0) as i64 * 17) as f32;
    let mouille = smooth(GROUND + 1.2, GROUND, v);
    mix(fois(hex(0xf0dca8), 0.94 + grain * 0.1), hex(0xc8b080), mouille * 0.6)
}

/// Un cumulus, `u` compté depuis la porte : des boules qui se tassent, ombrées dessous, éclairées dessus.
#[allow(clippy::too_many_arguments)]
fn cumulus(t: &mut Toile, c: &Cadre, sky: &Ciel, u: f32, base: f32, taille: f32, graine: i64, a: f32) {
    let (clair, ombre) = if sky.orage {
        (hex(0x6a6e7a), hex(0x3a3e4a))
    } else if sky.nuit > 0.6 {
        (hex(0x3a4466), hex(0x1a2240))
    } else {
        (mix(hex(0xffffff), sky.bas, 0.25 + sky.nuit * 0.4), mix(hex(0xb8c4d4), sky.haut, 0.2 + sky.nuit * 0.3))
    };
    let n = 7;
    for pass in 0..2 {
        for i in 0..n {
            let k = i as f32 / (n - 1) as f32;
            let r = taille * (0.55 + 0.45 * (k * PI).sin()) * (0.8 + 0.4 * hasard(graine + i) as f32);
            let x = (u + (k - 0.5) * taille * 3.2) * c.cw;
            let y = c.y(base - r * 0.8 - (k * PI).sin() * taille * 0.5);
            if pass == 0 {
                t.disque(x, y + 0.25 * r * c.demi, r * c.demi, ombre, a, 0.15);
            } else {
                t.disque(x - 0.15 * r * c.cw, y - 0.12 * r * c.demi, r * c.demi * 0.82, clair, a, 0.25);
            }
        }
    }
}

/// Les nuages qui passent, de droite à gauche, plus vite et plus sombres dans l'orage.
fn vitesse_nuages(sky: &Ciel) -> f32 {
    if sky.orage {
        0.05
    } else {
        0.012
    }
}

fn nuages(t: &mut Toile, c: &Cadre, sky: &Ciel, et: Etape, t_: f32) {
    let room = c.room();
    let n = ((room / 26.0).ceil() as i64).max(1) + if sky.orage { 2 } else { 0 };
    let vitesse = vitesse_nuages(sky);
    let tour = room + 30.0;
    for i in 0..n {
        let base = hasard(i * 41 + 3) as f32 * tour;
        let u = room + 12.0 - (base + t_ * vitesse * (0.7 + 0.6 * hasard(i * 3) as f32)).rem_euclid(tour);
        let haut = 3.5 + hasard(i * 7 + 1) as f32 * 5.0;
        let taille = if et == Etape::Tropiques { 2.2 } else { 1.6 } + hasard(i * 11) as f32 * 1.4;
        cumulus(t, c, sky, u, haut + taille, taille, i * 97 + 13, if sky.nuit > 0.6 { 0.7 } else { 0.95 });
    }
}

/// Ce qui bouge sans être un bateau ni une bête : le phare qui tourne, les bouées du départ.
fn decor_anime(t: &mut Toile, gardees: &mut Vignettes, c: &Cadre, sky: &Ciel, et: Etape, w: &Monde, t_: f32) {
    let room = c.room();
    let course = w.phase() == PhaseRhum::Course;
    let spectacle = fete(w);
    let depart = matches!(spectacle, Some((FeteRhum::Depart, _)));
    // la semaine du départ et sa première heure, et le départ rejoué : le bateau-comité, son
    // canon (au coup du spectacle, sinon à intervalles)
    if et == Etape::Bretagne && room >= 20.0 && (!course || w.rhum_progress.unwrap_or(0.0) < 0.005 || depart) {
        let (coup, fort) = match spectacle {
            Some((FeteRhum::Depart, a)) => (if a < COUP { 1000.0 } else { a - COUP }, 1.8),
            _ => (t_.rem_euclid(90.0), 1.0),
        };
        comite(t, gardees, c, sky, t_, coup, fort);
    }
    if et == Etape::Tropiques {
        grain(t, gardees, c, sky, t_);
    }
    match spectacle {
        Some((FeteRhum::Arrivee, a)) => {
            ligne_arrivee(t, c, sky, t_);
            // le feu d'artifice part quand la ligne est coupée, de jour comme de nuit, nourri
            if a >= COUP {
                feu_d_artifice(t, c, a - COUP, 6, 9.0, 1.5, sky.nuit < 0.5);
            }
        }
        _ if et == Etape::Guadeloupe && sky.nuit > 0.5 && !sky.orage => feu_d_artifice(t, c, t_, 3, 19.0, 1.0, false),
        _ => {}
    }
    if et == Etape::Bretagne && room >= 40.0 && (sky.nuit > 0.2 || sky.orage) {
        // la lanterne s'allume, le faisceau balaie la baie
        let (x, y) = ((room - 10.0) * c.cw, c.y(SEA - 10.8));
        let a = (t_ * 0.06).rem_euclid(2.0 * PI);
        let face = a.cos(); // le faisceau vers nous : il éblouit ; de dos, on ne le voit pas
        let force = sky.nuit.max(0.5);
        t.disque(x, y, 2.5 * c.demi * (0.6 + 0.4 * face.max(0.0)), hex(0xfff2a0), force * (0.35 + 0.5 * face.max(0.0)), 1.0);
        t.disque(x, y, 0.45 * c.demi, hex(0xfffbe0), force, 0.0);
        let long = room * 0.7 * c.cw;
        let dx = a.sin() * long;
        if a.sin().abs() > 0.15 {
            t.poly(&[(x, y - 2.0), (x + dx, y - 2.0 - 0.07 * long), (x + dx, y + 2.0 + 0.07 * long), (x, y + 2.0)], hex(0xfff2a0), 0.13 * force * a.sin().abs());
        }
    }
    if (w.phase() == PhaseRhum::Avant || depart) && room >= 20.0 {
        // les bouées de la ligne de départ, qui dansent
        for (k, f) in [0.3, 0.72].into_iter().enumerate() {
            let u = room * f;
            let v = 22.6 + (t_ * 0.25 + k as f32 * 2.0).sin() * 0.25;
            let (x, y) = (u * c.cw, c.y(v));
            let jaune = mul(hex(0xf2c14e), sky.amb);
            t.poly(&[(x - 0.9 * c.cw, y), (x + 0.9 * c.cw, y), (x + 0.5 * c.cw, y - 1.4 * c.demi), (x - 0.5 * c.cw, y - 1.4 * c.demi)], jaune, 1.0);
            t.trait_((x, y - 1.4 * c.demi), (x, y - 2.6 * c.demi), 1.2, mul(hex(0x2a2a30), sky.amb), 1.0);
            t.poly(&[(x, y - 2.6 * c.demi), (x + 1.2 * c.cw, y - 2.3 * c.demi), (x, y - 2.0 * c.demi)], mul(hex(0xd04a2a), sky.amb), 1.0);
        }
    }
}

/// Le bateau-comité au mouillage, au bout de la ligne de départ : sa coque, sa passerelle, son
/// grand pavois ; et son canon (un éclair, une bouffée qui s'étire), `coup` tics après qu'il a tonné.
fn comite(t: &mut Toile, gardees: &mut Vignettes, c: &Cadre, sky: &Ciel, t_: f32, coup: f32, fort: f32) {
    let lane = 22.6;
    let s = echelle(lane);
    let l = 10.0 * s;
    let (u, v) = (c.room() * 0.12 + l / 2.0 + 1.0, lane + 0.12 * (t_ * 0.15).sin());
    let (x, y) = (u * c.cw, c.y(v));
    let cle = (-300, (l * 100.0) as i32, (sky.amb[0] * 100.0) as i32, (sky.bas[2] * 10.0) as i32, 0);
    let v_ = gardees.entry(cle).or_insert_with(|| {
        let (w, h) = ((1.3 * l * c.cw) as usize + 8, (1.1 * l * c.demi) as usize + 8);
        let (ox, oy) = (0.6 * l * c.cw + 4.0, 0.95 * l * c.demi + 4.0);
        let (c, sky) = (*c, *sky);
        Vignette::faire(w, h, ox, oy, &move |t: &mut Toile, ox: f32, oy: f32| {
            let p = |a: f32, b: f32| (ox + a * l * c.cw, oy + b * l * c.demi);
            let pts = |liste: &[(f32, f32)]| -> Vec<(f32, f32)> { liste.iter().map(|&(a, b)| p(a, b)).collect() };
            let teinte = |col: u32| mix(mul(hex(col), sky.amb), sky.bas, 0.15);
            t.poly(&pts(&[(-0.5, -0.15), (0.42, -0.17), (0.52, -0.22), (0.45, 0.0), (-0.47, 0.0)]), teinte(0x1d2a4a), 1.0);
            t.poly(&pts(&[(-0.5, -0.15), (0.42, -0.17), (0.45, -0.15), (-0.5, -0.13)]), teinte(0xf4f4f0), 1.0);
            t.poly(&pts(&[(-0.32, -0.15), (0.16, -0.16), (0.12, -0.33), (-0.28, -0.33)]), teinte(0xf0f0ec), 1.0);
            t.poly(&pts(&[(-0.26, -0.27), (0.11, -0.27), (0.1, -0.3), (-0.25, -0.3)]), teinte(0x2a3a5a), 1.0);
            t.trait_(p(-0.06, -0.33), p(-0.06, -0.85), 1.4, teinte(0x2a2a30), 1.0);
            // le grand pavois : de l'étrave à la tête du mât, et jusqu'à la poupe
            const FANIONS: [u32; 6] = [0xd04a2a, 0xf2c14e, 0x1d5a9a, 0xf4f4f0, 0x3c8a3c, 0x111111];
            for (a, b) in [((0.5, -0.2), (-0.06, -0.85)), ((-0.06, -0.85), (-0.5, -0.16))] {
                t.trait_(p(a.0, a.1), p(b.0, b.1), 0.8, teinte(0x3a3a44), 1.0);
                for k in 1..8 {
                    let f = k as f32 / 8.0;
                    let (fx, fy) = (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f);
                    let (x, y) = p(fx, fy);
                    t.poly(&[(x - 1.6, y), (x + 1.6, y), (x + 1.6, y + 3.5), (x - 1.6, y + 3.5)], teinte(FANIONS[k % FANIONS.len()]), 1.0);
                }
            }
        })
    });
    v_.poser(t, x, y);
    // le coup de canon
    let a = coup;
    let (bx, by) = (x + 0.48 * l * c.cw, y - 0.2 * l * c.demi);
    if a < 2.0 * fort {
        t.disque(bx, by, 0.6 * l * c.cw * 0.25 * fort, hex(0xffd070), 0.9, 0.6);
        t.disque(bx, by, 1.5 * l * c.cw * 0.25 * fort, hex(0xfff0c0), 0.35, 1.0);
    }
    let duree = 40.0 * fort;
    if a < duree {
        let f = a / duree;
        for k in 0..6 {
            let g = k as f32 / 5.0;
            let r = (1.0 + 3.0 * f.sqrt() * (0.6 + g * 0.4)) * s * c.demi * fort;
            t.disque(bx + (g * 2.5 + f * 4.0) * s * c.cw, by - (g * 0.8 + f * 2.5) * s * c.demi, r, mul(hex(0xe8e8ec), sky.amb), (1.0 - f) * 0.55, 0.8);
        }
    }
}

/// Un grain des tropiques : un gros nuage sombre qui traverse lentement, son rideau de pluie jusqu'à la mer.
fn grain(t: &mut Toile, gardees: &mut Vignettes, c: &Cadre, sky: &Ciel, t_: f32) {
    let room = c.room();
    let tour = room + 50.0;
    let u = room + 20.0 - (t_ * 0.02 + 30.0).rem_euclid(tour);
    let taille = 3.2;
    let base = SEA - 6.0;
    // la pluie : des traits penchés par le vent, qui tombent
    let gris = mix(mul(hex(0x6a7480), sky.amb), sky.bas, 0.3);
    for k in 0..36 {
        let f = hasard(k * 7 + 3) as f32;
        let x = (u + (f - 0.5) * taille * 3.0) * c.cw;
        let chute = ((t_ * 0.9 + k as f32 * 13.0) % 30.0) / 30.0;
        let (ya, yb) = (c.y(base + 1.0 + chute * 6.0), c.y(base + 3.0 + chute * 6.0 + hasard(k) as f32 * 4.0));
        t.trait_((x, ya), (x - (yb - ya) * 0.25, yb), 1.0, gris, 0.35);
    }
    let rideau: Vec<(f32, f32)> = vec![
        ((u - taille * 1.4) * c.cw, c.y(base + 0.5)),
        ((u + taille * 1.4) * c.cw, c.y(base + 0.5)),
        ((u + taille * 0.9) * c.cw, c.y(SEA + 2.5)),
        ((u - taille * 1.9) * c.cw, c.y(SEA + 2.5)),
    ];
    t.poly(&rideau, gris, 0.18);
    let cle = (-400, (c.cw * 100.0) as i32, (sky.amb[0] * 100.0) as i32, (sky.bas[2] * 10.0) as i32, 0);
    let nuage = gardees.entry(cle).or_insert_with(|| {
        let (w, h) = ((taille * 5.0 * c.cw) as usize, (taille * 3.0 * c.demi) as usize);
        let (ox, oy) = (w as f32 / 2.0, h as f32 * 0.8);
        let (c, mut sombre) = (*c, *sky);
        sombre.orage = true;
        sombre.amb = mul(sombre.amb, [0.9, 0.9, 0.95]);
        Vignette::faire(w, h, ox, oy, &move |t: &mut Toile, ox: f32, oy: f32| cumulus(t, &c, &sombre, ox / c.cw, oy / c.demi, taille, 404, 1.0))
    });
    nuage.poser(t, u * c.cw, c.y(base));
}

/// La ligne d'arrivée de Pointe-à-Pitre : deux bouées jaunes, leur damier, qui dansent.
fn ligne_arrivee(t: &mut Toile, c: &Cadre, sky: &Ciel, t_: f32) {
    let u = ligne_d_arrivee(c);
    for (k, v) in [21.0f32, 25.6].into_iter().enumerate() {
        let s = echelle(v);
        let v = v + (t_ * 0.25 + k as f32 * 2.0).sin() * 0.2;
        let (x, y) = (c.x(c.from + u), c.y(v));
        let (bw, bh) = (1.4 * s * c.cw, 2.6 * s * c.demi);
        t.poly(&[(x - bw, y), (x + bw, y), (x + 0.6 * bw, y - bh), (x - 0.6 * bw, y - bh)], mul(hex(0xf2c14e), sky.amb), 1.0);
        let haut = y - 2.4 * bh;
        t.trait_((x, y - bh), (x, haut), 1.2, mul(hex(0x2a2a30), sky.amb), 1.0);
        // le damier, qui claque au vent
        let (cw, ch) = (0.7 * s * c.cw, 0.5 * s * c.demi);
        for i in 0..4 {
            for j in 0..3 {
                let ondule = ((t_ * 0.4 + i as f32) * 0.9).sin() * 0.3 * ch;
                let (x0, y0) = (x + i as f32 * cw, haut + j as f32 * ch + ondule);
                let col = if (i + j) % 2 == 0 { hex(0x1a1a20) } else { hex(0xf4f4f0) };
                t.poly(&[(x0, y0), (x0 + cw, y0), (x0 + cw, y0 + ch), (x0, y0 + ch)], mul(col, sky.amb), 1.0);
            }
        }
    }
}

/// Le feu d'artifice de l'arrivée, sur Pointe-à-Pitre : des fusées qui montent et éclatent en
/// gerbes, `n_fusees` à la fois, à `ecart` tics l'une de l'autre, `fort` fois plus grandes. De
/// jour, des teintes franches et des rayons : les pastels de la nuit s'y perdraient dans le bleu.
fn feu_d_artifice(t: &mut Toile, c: &Cadre, t_: f32, n_fusees: i64, ecart: f32, fort: f32, jour: bool) {
    let room = c.room();
    const TEINTES_NUIT: [u32; 5] = [0xffd27a, 0xff6a5a, 0x7fd4ff, 0xb98cff, 0x8af08a];
    const TEINTES_JOUR: [u32; 5] = [0xffb000, 0xe8281c, 0x1c4ee8, 0xc01cb0, 0x10a040];
    let teintes = if jour { TEINTES_JOUR } else { TEINTES_NUIT };
    for k in 0..n_fusees {
        let cycle = 56.0;
        let a = (t_ + k as f32 * ecart).rem_euclid(cycle);
        let n = ((t_ + k as f32 * ecart) / cycle).floor() as i64;
        // à gauche de la carte, qui tient le haut à droite
        let (u, v) = (room * (0.08 + 0.52 * hasard(n * 7 + k) as f32), 2.5 + 6.0 * hasard(n * 13 + k) as f32);
        let col = hex(teintes[(hasard(n * 3 + k) * 5.0) as usize % 5]);
        if a < 7.0 {
            // la fusée qui monte, sa traînée
            let f = a / 7.0;
            let (x, y) = (u * c.cw, c.y(SEA - 1.0 + (v - SEA + 1.0) * f));
            t.trait_((x, y), (x, y + 2.0 * c.demi), 1.2, hex(0xffe0a0), 0.6);
            t.disque(x, y, 1.4, hex(0xfff4d0), 1.0, 0.3);
            continue;
        }
        let f = (a - 7.0) / (cycle - 7.0);
        if f > 0.75 {
            continue;
        }
        let r = f.sqrt() * 5.5 * fort;
        let reste = 1.0 - f / 0.75;
        for i in 0..28 {
            let ang = i as f32 / 28.0 * 2.0 * PI + n as f32;
            let (x, y) = ((u + ang.cos() * r) * c.cw, c.y(v + ang.sin() * r * 0.9 + f * f * 4.0));
            if jour {
                let (xi, yi) = ((u + ang.cos() * r * 0.55) * c.cw, c.y(v + ang.sin() * r * 0.5 + f * f * 4.0));
                t.trait_((xi, yi), (x, y), 1.6, col, reste * 0.8);
            }
            t.disque(x, y, 1.7 * fort, col, reste * 0.95, 0.3);
            t.disque(x, y, 4.5 * fort, col, reste * if jour { 0.15 } else { 0.25 }, 1.0);
        }
    }
}

/// Une vignette : un dessin gardé, ses pixels en couleur prémultipliée et leur opacité
/// (0xAARRGGBB), et le point qui se pose à sa place.
pub struct Vignette {
    w: usize,
    h: usize,
    ox: f32,
    oy: f32,
    px: Vec<u32>,
    /// par rangée, les colonnes où il y a quelque chose
    rangs: Vec<(usize, usize)>,
}

impl Vignette {
    /// Dessine une fois, sur du noir puis sur du blanc : la différence donne l'opacité.
    fn faire(w: usize, h: usize, ox: f32, oy: f32, dessin: &dyn Fn(&mut Toile, f32, f32)) -> Vignette {
        let mut noir = vec![0xFF00_0000u32; w * h];
        let mut blanc = vec![0xFFFF_FFFFu32; w * h];
        dessin(&mut Toile { px: &mut noir, w, h }, ox, oy);
        dessin(&mut Toile { px: &mut blanc, w, h }, ox, oy);
        let px = noir
            .iter()
            .zip(&blanc)
            .map(|(&n, &b)| {
                let (cn, cb) = (unpack(n), unpack(b));
                let a = 255.0 - ((cb[0] - cn[0]) + (cb[1] - cn[1]) + (cb[2] - cn[2])) / 3.0;
                let a = a.clamp(0.0, 255.0) as u32;
                (a << 24) | (n & 0x00FF_FFFF)
            })
            .collect::<Vec<u32>>();
        let rangs = px
            .chunks(w)
            .map(|r| match (r.iter().position(|p| p >> 24 != 0), r.iter().rposition(|p| p >> 24 != 0)) {
                (Some(a), Some(b)) => (a, b + 1),
                _ => (0, 0),
            })
            .collect();
        Vignette { w, h, ox, oy, px, rangs }
    }

    /// La pose sur la toile, son point en (x, y).
    fn poser(&self, t: &mut Toile, x: f32, y: f32) {
        let (x0, y0) = ((x - self.ox).round() as i32, (y - self.oy).round() as i32);
        for j in 0..self.h {
            let ty = y0 + j as i32;
            if ty < 0 || ty as usize >= t.h {
                continue;
            }
            let (de, a) = self.rangs[j];
            let ligne = &self.px[j * self.w + de..j * self.w + a];
            for (i, &s) in ligne.iter().enumerate() {
                let i = i + de;
                let a = s >> 24;
                let tx = x0 + i as i32;
                if a == 0 || tx < 0 || tx as usize >= t.w {
                    continue;
                }
                let k = ty as usize * t.w + tx as usize;
                if a == 255 {
                    t.px[k] = 0xFF00_0000 | s;
                    continue;
                }
                // prémultipliée : la vignette, plus ce qu'elle laisse voir de dessous
                let (d, r) = (t.px[k], 256 - a);
                let canal = |sh: u32| (((s >> sh) & 255) + ((((d >> sh) & 255) * r) >> 8)).min(255) << sh;
                t.px[k] = 0xFF00_0000 | canal(16) | canal(8) | canal(0);
            }
        }
    }
}

/// Les vignettes gardées : les bateaux à chaque pas de tangage, les palmiers à chaque pas de vent.
pub type Vignettes = std::collections::HashMap<(i64, i32, i32, i32, i32), Vignette>;

/// Un bateau à sa place `u` (son milieu), allant dans le sens `dir` : son sillage et ses embruns,
/// lui (une vignette par pas de tangage), ses feux la nuit.
#[allow(clippy::too_many_arguments)]
fn bateau(t: &mut Toile, gardees: &mut Vignettes, c: &Cadre, sky: &Ciel, b: &Bateau, (u, dir): (f32, f32), t_: f32, phase: PhaseRhum, taille: f32, phosphore: bool) {
    let (lg, mat) = gabarit(b.sorte);
    let s = echelle(b.lane);
    let l = lg * s * taille;
    if u + l < c.from - 2.0 || u - l > c.to + 2.0 {
        return;
    }
    let g = b.graine as f32;
    let course = phase == PhaseRhum::Course;
    // il tangue (par pas : une vignette chacun) et pilonne dans la houle
    let pas = ((t_ * 0.21 + g).sin() * 2.0).round();
    let tangage = pas * 0.0125 * if course { 1.4 } else { 0.7 };
    let pilon = 0.12 * s * (t_ * 0.17 + g * 1.3).sin();
    // l'Ultim vole, par moments : le flotteur au vent se lève
    let vol = if course && matches!(b.sorte, Sorte::Ultim | Sorte::OceanFifty) { (smooth(0.2, 0.9, (t_ * 0.013 + g).sin()) * 3.0).round() } else { 0.0 };
    let (cs, sn) = (tangage.cos(), tangage.sin());
    let v0 = b.lane + pilon;
    let (x0, y0) = (c.x(u), c.y(v0));
    let p = |x: f32, y: f32| -> (f32, f32) {
        let (x, y) = (x * l, y * l);
        let (x, y) = (x * cs - y * sn, x * sn + y * cs);
        (x0 + dir * x * c.cw, y0 + y * c.demi)
    };
    let loin = (1.0 - s).powi(2) * 0.55; // ce qui est loin pâlit dans la brume
    let blanc = mix(mul(hex(0xf6fbff), sky.amb), sky.bas, loin);

    // le sillage : une traînée d'écume qui s'élargit et s'efface derrière, longue en course
    let long = if course { 1.8 } else { 0.6 };
    let n = if s > 0.6 { 8 } else { 4 };
    for k in 0..n {
        let (f0, f1) = (k as f32 / n as f32, (k + 1) as f32 / n as f32);
        let (xa, xb) = (-0.48 - f0 * long, -0.48 - f1 * long);
        let (wa, wb) = (0.008 + 0.05 * f0, 0.008 + 0.05 * f1);
        let pts = [p(xa, 0.01 - wa * 0.4), p(xb, 0.01 - wb * 0.4), p(xb, 0.01 + wb), p(xa, 0.01 + wa)];
        t.poly(&pts, blanc, (1.0 - f0).powf(1.4) * 0.5);
    }
    let bulles = (8.0 * s + 3.0) as i64;
    for k in 0..bulles {
        let f = (k as f32 + (t_ * 0.3).fract()) / bulles as f32;
        let (px, py) = p(-0.48 - f * long, 0.01 + (hasard(k * 7 + b.graine) as f32 - 0.3) * 0.05 * f);
        if phosphore {
            t.disque(px, py, 1.5 + s * 2.5 * (1.0 - f), hex(0x6ff0d8), (1.0 - f) * 0.35, 1.0);
            t.disque(px, py, 0.6 + s * (1.0 - f), hex(0xb8fff0), (1.0 - f) * 0.9, 0.3);
        } else {
            t.disque(px, py, 0.7 + s * 1.2 * (1.0 - f), blanc, (1.0 - f) * 0.7, 0.4);
        }
    }

    // lui, d'une vignette : dessinée une fois par pas de tangage, de vol et de sens
    let cle = (b.graine, dir as i32, pas as i32 + 10 * vol as i32, (l * 100.0) as i32, course as i32);
    let v = gardees.entry(cle).or_insert_with(|| {
        let (w, h) = ((1.6 * l * c.cw) as usize + 8, ((1.5 * mat + 0.15) * l * c.demi) as usize + 8);
        let (ox, oy) = (if dir > 0.0 { 0.6 * l * c.cw + 4.0 } else { w as f32 - 0.6 * l * c.cw - 4.0 }, (mat + 0.06) * l * c.demi + 4.0);
        let sky = *sky;
        let b = b.clone();
        Vignette::faire(w, h, ox, oy, &move |toile: &mut Toile, ox: f32, oy: f32| {
            let p = |x: f32, y: f32| -> (f32, f32) {
                let (x, y) = (x * l, y * l);
                let (x, y) = (x * cs - y * sn, x * sn + y * cs);
                (ox + dir * x * c.cw, oy + y * c.demi)
            };
            gree(toile, &p, c, &sky, &b, l, s, mat, course, vol / 3.0);
        })
    });
    v.poser(t, x0, y0);

    // les embruns de l'étrave, en course
    if course {
        for k in 0..6 {
            let f = (t_ * 0.09 + k as f32 / 6.0 + g).fract();
            let (px, py) = p(0.5 + f * 0.12, -f * (1.0 - f) * 0.35);
            t.disque(px, py, (0.5 + s * 1.6) * (1.0 - f * 0.5), blanc, (1.0 - f) * 0.75, 0.4);
        }
    }
    // la nuit, ses feux : en tête de mât, à l'étrave (vert ou rouge selon le bord vu)
    if sky.nuit > 0.3 {
        let a = (sky.nuit - 0.3) * 1.4;
        let tete = p(0.02, -mat);
        t.disque(tete.0, tete.1, 3.0 + s * 4.0, hex(0xfff6d0), a * 0.35, 1.0);
        t.disque(tete.0, tete.1, 0.8 + s, hex(0xfffbe8), a, 0.0);
        let (bx, by) = p(0.45, -0.07);
        let feu = if dir > 0.0 { hex(0x6fe07a) } else { hex(0xff5a4a) };
        t.disque(bx, by, 3.0 + s * 4.0, feu, a * 0.35, 1.0);
        t.disque(bx, by, 0.8 + s, feu, a, 0.0);
    }
}

/// Le bateau lui-même : son reflet, ses coques, son mât et ses voiles. `p` place un point du
/// bateau (en longueurs, l'étrave vers +x, le haut vers -y) ; `vol` de 0 à 1, l'Ultim qui vole.
#[allow(clippy::too_many_arguments)]
fn gree(t: &mut Toile, p: &dyn Fn(f32, f32) -> (f32, f32), c: &Cadre, sky: &Ciel, b: &Bateau, l: f32, s: f32, mat: f32, course: bool, vol: f32) {
    let pts = |liste: &[(f32, f32)]| -> Vec<(f32, f32)> { liste.iter().map(|&(x, y)| p(x, y)).collect() };
    let loin = (1.0 - s).powi(2) * 0.55;
    let teinte = |col: Rgb| mix(mul(col, sky.amb), sky.bas, loin);
    let ombre = |col: Rgb, k: f32| teinte(fois(col, k));
    let blanc = teinte(hex(0xf6fbff));
    let noir = ombre(hex(0x2a2a30), 1.0);
    let voile_pts: Vec<(f32, f32)> = match b.sorte {
        Sorte::Ultim | Sorte::OceanFifty => vec![(0.03, -0.98), (0.03, -0.09), (-0.44, -0.10), (-0.42, -0.3), (-0.33, -0.6), (-0.18, -0.85), (-0.06, -0.96)],
        Sorte::Imoca => vec![(0.04, -1.5), (0.05, -0.12), (-0.42, -0.13), (-0.41, -0.3), (-0.36, -0.9), (-0.22, -1.33), (-0.08, -1.5)],
        _ => vec![(0.02, -1.42), (0.03, -0.12), (-0.40, -0.13), (-0.34, -0.7), (-0.16, -1.2)],
    };
    let haut_mat = -mat;
    // la têtière de la grand-voile juste sous la tête du mât
    let k = mat * 0.98 / voile_pts[0].1.abs();
    let grand: Vec<(f32, f32)> = voile_pts.iter().map(|&(x, y)| (x, y * k)).collect();
    // le reflet, sous lui
    if s > 0.4 {
        let reflet: Vec<(f32, f32)> = grand.iter().map(|&(x, y)| p(x, -y * 0.35 + 0.03)).collect();
        t.poly(&reflet, teinte(b.voile), 0.12);
    }
    // les coques
    match b.sorte {
        Sorte::Ultim | Sorte::OceanFifty => {
            // il vole : la coque centrale hors de l'eau, plus haut par moments ; seul le flotteur
            // sous le vent (devant nous) touche l'eau, porté par son foil ; les bras entre les deux
            let h = if b.sorte == Sorte::Ultim { -0.05 - 0.035 * vol } else { -0.03 - 0.02 * vol };
            for bx in [-0.17, 0.16] {
                t.ligne(&pts(&[(bx - 0.02, -0.012), (bx - 0.01, -0.04 + h * 0.4), (bx + 0.01, -0.05 + h)]), (0.028 * l * c.cw).max(1.4), ombre(b.coque, 0.55), 1.0);
            }
            t.poly(&pts(&[(-0.5, -0.115 + h), (0.36, -0.11 + h), (0.54, -0.07 + h), (0.5, -0.045 + h), (-0.47, -0.04 + h)]), teinte(b.coque), 1.0);
            t.poly(&pts(&[(-0.5, -0.115 + h), (0.36, -0.11 + h), (0.43, -0.094 + h), (-0.5, -0.1 + h)]), teinte(b.bande), 0.9);
            t.poly(&pts(&[(-0.47, -0.05 + h), (0.5, -0.05 + h), (0.5, -0.045 + h), (-0.47, -0.04 + h)]), ombre(b.coque, 0.6), 1.0);
            // le safran de la coque centrale, qui file dans l'air puis l'eau
            t.trait_(p(-0.47, -0.045 + h), p(-0.49, 0.03), (0.012 * l * c.cw).max(1.0), noir, 0.9);
            // le flotteur, ses foils : le grand en L sous lui, la dérive
            t.ligne(&pts(&[(0.05, 0.0), (0.07, 0.07), (0.17, 0.08)]), (0.016 * l * c.cw).max(1.0), noir, 0.85);
            t.trait_(p(-0.3, 0.0), p(-0.31, 0.05), (0.012 * l * c.cw).max(1.0), noir, 0.7);
            t.poly(&pts(&[(-0.34, -0.03), (0.34, -0.03), (0.44, -0.008), (0.39, 0.012), (-0.32, 0.01)]), ombre(b.coque, 0.92), 1.0);
            t.poly(&pts(&[(-0.34, -0.03), (0.34, -0.03), (0.38, -0.022), (-0.34, -0.022)]), teinte(b.bande), 0.85);
        }
        Sorte::Imoca => {
            t.poly(&pts(&[(-0.5, -0.11), (0.46, -0.10), (0.505, -0.06), (0.49, -0.015), (0.42, 0.0), (-0.48, 0.0)]), teinte(b.coque), 1.0);
            t.poly(&pts(&[(-0.5, -0.11), (0.46, -0.10), (0.48, -0.085), (-0.5, -0.093)]), teinte(b.bande), 0.95);
            // le roof, et son foil : un grand croc sous le vent qui sort de l'eau
            t.poly(&pts(&[(-0.3, -0.11), (0.0, -0.11), (-0.04, -0.145), (-0.26, -0.145)]), ombre(b.coque, 0.75), 1.0);
            t.ligne(&pts(&[(0.06, -0.04), (0.11, 0.03), (0.2, 0.065), (0.29, 0.05)]), (0.022 * l * c.cw).max(1.0), ombre(hex(0x24242c), 1.0), 0.95);
        }
        _ => {
            // un scow, l'étrave ronde
            t.poly(&pts(&[(-0.5, -0.11), (0.40, -0.11), (0.5, -0.075), (0.505, -0.03), (0.46, 0.0), (-0.48, 0.0)]), teinte(b.coque), 1.0);
            t.poly(&pts(&[(-0.5, -0.11), (0.40, -0.11), (0.45, -0.095), (-0.5, -0.1)]), teinte(b.bande), 0.9);
        }
    }
    // la ligne de flottaison qui écume
    let multi = matches!(b.sorte, Sorte::Ultim | Sorte::OceanFifty);
    if multi {
        t.trait_(p(-0.34, 0.008), p(0.42, 0.008), (0.7 + s * 1.2).min(2.2), blanc, 0.55);
    } else {
        t.trait_(p(-0.5, 0.004), p(0.48, 0.004), (0.9 + s * 1.4).min(2.6), blanc, 0.55);
    }
    // sur le multicoque qui vole, le gréement monte avec la coque
    let leve = if b.sorte == Sorte::Ultim { -0.05 - 0.035 * vol - 0.03 } else if multi { -0.03 - 0.02 * vol - 0.02 } else { 0.0 };
    let p = |x: f32, y: f32| p(x, y + leve);
    let pts = |liste: &[(f32, f32)]| -> Vec<(f32, f32)> { liste.iter().map(|&(x, y)| p(x, y)).collect() };
    // le mât, la bôme, les voiles
    let e = (0.011 * l * c.cw).max(1.0);
    let voile = teinte(b.voile);
    t.poly(&pts(&grand), voile, 1.0);
    // le creux de la voile, plus sombre vers la chute ; ses lattes
    let creux: Vec<(f32, f32)> = grand.iter().map(|&(x, y)| (x * 0.55 - 0.18 * (1.0 - (y / haut_mat).abs()), y)).collect();
    t.poly(&pts(&creux), fois(voile, 0.9), 0.35);
    if s > 0.35 {
        for f in [0.25, 0.45, 0.65, 0.82] {
            let y = haut_mat * f;
            // la chute : le côté de la voile, vers l'arrière, que croise cette hauteur
            let bord = grand.iter().zip(grand.iter().skip(1)).find(|(a, b)| (a.1 >= y) != (b.1 >= y) && (a.0 < 0.0 || b.0 < 0.0));
            if let Some((a, b2)) = bord {
                let x = a.0 + (y - a.1) / (b2.1 - a.1) * (b2.0 - a.0);
                t.trait_(p(x + 0.01, y), p(x * 0.45, y + 0.004), 1.0, fois(voile, 0.78), 0.55);
            }
        }
    }
    let marque = bande(&grand, haut_mat * 0.58, haut_mat * 0.44);
    t.poly(&pts(&marque), teinte(b.bande), 0.95);
    t.trait_(p(0.04, -0.1), p(-0.43, -0.11), e * 0.9, noir, 1.0);
    // le mât : une aile, épaisse, sur l'Ultim
    let epais = if matches!(b.sorte, Sorte::Ultim | Sorte::OceanFifty) { 2.0 } else { 1.3 };
    t.trait_(p(0.04, -0.09), p(0.02, haut_mat), e * epais, noir, 1.0);
    let foc_haut = haut_mat * if b.sorte == Sorte::Ultim { 0.82 } else { 0.74 };
    let gen = course && matches!(b.sorte, Sorte::Class40 | Sorte::Rhum | Sorte::Imoca);
    if gen {
        // portant vers les Antilles : le gennaker, gonflé, devant
        let couleur = teinte(mix(b.bande, hex(0xffffff), 0.15));
        t.poly(&pts(&[(0.04, foc_haut * 1.08), (0.82, -0.11), (0.72, foc_haut * 0.35), (0.5, foc_haut * 0.75)]), couleur, 0.97);
        t.ligne(&pts(&[(0.04, foc_haut * 1.08), (0.5, foc_haut * 0.75), (0.72, foc_haut * 0.35), (0.82, -0.11)]), 1.0, fois(couleur, 0.8), 0.6);
    } else {
        t.poly(&pts(&[(0.05, foc_haut), (0.5, -0.085), (0.08, -0.095)]), fois(voile, 0.93), 1.0);
    }
}

/// Les bêtes de l'étape, chacune à sa profondeur : des mouettes en Bretagne, des dauphins et une
/// baleine au large, des puffins, des poissons volants et des sargasses dans les tropiques, des
/// frégates en Guadeloupe.
fn faune(c: &Cadre, sky: &Ciel, et: Etape, t: f32) -> Vec<(f32, Box<dyn Fn(&mut Toile)>)> {
    let (c, sky) = (*c, *sky);
    let room = c.room();
    let mut out: Vec<(f32, Box<dyn Fn(&mut Toile)>)> = Vec::new();
    let oiseaux = match et {
        Etape::Bretagne => 3,
        Etape::Guadeloupe => 2,
        _ => 2,
    };
    if sky.nuit < 0.6 {
        for i in 0..oiseaux {
            let fi = i as f32;
            let u = c.from + room * (0.5 + 0.45 * (t / (110.0 + fi * 41.0) + fi * 2.1).sin());
            let v = 5.0 + fi * 1.8 + 2.0 * (t / (27.0 + fi * 9.0) + fi).sin();
            let sens = (t / (110.0 + fi * 41.0) + fi * 2.1).cos().signum();
            out.push((
                SEA - 1.0,
                Box::new(move |toile: &mut Toile| oiseau(toile, &c, &sky, et, u, v, t + fi * 13.0, sens)),
            ));
        }
    }
    match et {
        Etape::Large | Etape::Acores | Etape::Tropiques => {
            // un banc de dauphins qui sautent, de temps en temps
            let cycle = 160.0;
            let n = (t / cycle).floor();
            let f = (t - n * cycle) / 34.0;
            if f < 1.0 {
                let u0 = c.from + room * (0.15 + 0.6 * hasard(n as i64 * 7 + 3) as f32);
                for j in 0..3 {
                    let fj = f * 1.0 - j as f32 * 0.12;
                    if !(0.0..1.0).contains(&fj) {
                        continue;
                    }
                    let lane = 24.0 + j as f32 * 0.6;
                    out.push((lane, Box::new(move |toile: &mut Toile| dauphin(toile, &c, &sky, u0 + j as f32 * 1.6, lane, fj))));
                }
            }
        }
        _ => {}
    }
    if matches!(et, Etape::Large | Etape::Acores) {
        // la baleine, au loin : son dos qui roule, son souffle
        let cycle = 260.0;
        let n = (t / cycle).floor();
        let f = (t - n * cycle) / 70.0;
        if f < 1.0 {
            let u = c.from + room * (0.25 + 0.5 * hasard(n as i64 * 13 + 1) as f32);
            out.push((19.5, Box::new(move |toile: &mut Toile| baleine(toile, &c, &sky, u, 19.5, f))));
        }
    }
    if et == Etape::Tropiques {
        // des sargasses qui dérivent ; des poissons volants qui filent au ras de l'eau
        for i in 0..((room / 14.0) as i64 + 1) {
            let lane = 19.5 + hasard(i * 5 + 2) as f32 * 6.5;
            let u = c.from + (hasard(i * 3 + 1) as f32 * room + t * 0.01).rem_euclid(room + 6.0) - 3.0;
            out.push((lane - 0.01, Box::new(move |toile: &mut Toile| sargasse(toile, &c, &sky, u, lane, i))));
        }
        for i in 0..4 {
            let cycle = 90.0 + i as f32 * 17.0;
            let f = ((t + i as f32 * 40.0) / cycle).fract() * 2.5;
            if f < 1.0 {
                let n = ((t + i as f32 * 40.0) / cycle).floor() as i64;
                let lane = 22.0 + hasard(n * 7 + i) as f32 * 4.0;
                let u = c.from + room * (0.1 + 0.6 * hasard(n * 11 + i) as f32) + f * 9.0;
                out.push((lane, Box::new(move |toile: &mut Toile| poisson_volant(toile, &c, &sky, u, lane, f))));
            }
        }
    }
    out
}

/// Un oiseau qui plane et bat des ailes : une mouette, un puffin, une frégate.
#[allow(clippy::too_many_arguments)]
fn oiseau(t: &mut Toile, c: &Cadre, sky: &Ciel, et: Etape, u: f32, v: f32, t_: f32, sens: f32) {
    let (x, y) = (c.x(u), c.y(v));
    let bat = (t_ * 0.9).sin();
    let plane = ((t_ / 7.0).floor() as i64 % 3 != 0) as i32 as f32; // il plane, puis bat des ailes
    let k = if plane > 0.5 { 0.25 } else { bat };
    let (env, col, bout) = match et {
        Etape::Guadeloupe | Etape::Tropiques => (1.5 * c.cw, hex(0x1a1a22), hex(0x1a1a22)),
        Etape::Bretagne => (1.1 * c.cw, hex(0xf4f4f0), hex(0x5a5e66)),
        _ => (0.8 * c.cw, hex(0x3a3e48), hex(0x2a2e38)),
    };
    let col = mul(col, sky.amb);
    let aile = |s: f32| vec![(x, y), (x + s * env * 0.5, y - env * 0.28 * k - env * 0.12), (x + s * env, y - env * 0.1 * k + env * 0.08)];
    t.ligne(&aile(-1.0), (0.13 * env).max(1.2), col, 1.0);
    t.ligne(&aile(1.0), (0.13 * env).max(1.2), col, 1.0);
    t.disque(x + sens * env * 0.08, y, 0.12 * env + 0.6, col, 1.0, 0.0);
    t.disque(x - env, y - env * 0.1 * k + env * 0.08, 0.08 * env + 0.4, mul(bout, sky.amb), 0.9, 0.0);
    t.disque(x + env, y - env * 0.1 * k + env * 0.08, 0.08 * env + 0.4, mul(bout, sky.amb), 0.9, 0.0);
    if matches!(et, Etape::Guadeloupe | Etape::Tropiques) {
        // la queue en ciseaux de la frégate
        t.ligne(&[(x - sens * env * 0.1, y), (x - sens * env * 0.45, y + env * 0.12)], 1.0, col, 0.9);
        t.ligne(&[(x - sens * env * 0.1, y), (x - sens * env * 0.42, y - env * 0.05)], 1.0, col, 0.9);
    }
}

/// Un dauphin en plein saut, `f` de 0 (il sort) à 1 (il rentre) : son arc, ses gerbes.
fn dauphin(t: &mut Toile, c: &Cadre, sky: &Ciel, u: f32, lane: f32, f: f32) {
    let s = echelle(lane);
    let long = 2.2 * s;
    let x = u + (f - 0.5) * 5.0 * s;
    let v = lane - (f * PI).sin() * 2.4 * s;
    let pente = -(f * PI).cos() * 0.9; // le nez monte, puis plonge
    let (cs, sn) = (pente.atan().cos(), pente.atan().sin());
    let p = |a: f32, b: f32| -> (f32, f32) {
        let (a, b) = (a * long, b * long);
        (c.x(x + a * cs - b * sn), c.y(v + a * sn + b * cs))
    };
    let dos = mul(hex(0x4a5a6e), sky.amb);
    let corps: Vec<(f32, f32)> = [(0.55, 0.0), (0.4, -0.08), (0.1, -0.13), (-0.25, -0.1), (-0.45, -0.04), (-0.55, -0.1), (-0.6, 0.06), (-0.45, 0.03), (-0.2, 0.08), (0.2, 0.08), (0.48, 0.03)]
        .iter()
        .map(|&(a, b)| p(a, b))
        .collect();
    t.poly(&corps, dos, 1.0);
    t.poly(&[p(0.0, -0.12), p(-0.12, -0.26), p(-0.16, -0.1)], dos, 1.0);
    t.poly(&[p(0.35, 0.03), p(0.0, 0.07), p(-0.3, 0.05), p(0.0, 0.03)], mul(hex(0xc8d0d8), sky.amb), 0.8);
    // les gerbes en sortant et en rentrant
    for (debut, gx) in [(0.0, u - 2.5 * s), (0.85, u + 2.5 * s)] {
        let g = (f - debut) / 0.25;
        if (0.0..1.0).contains(&g) {
            for k in 0..5 {
                let a = k as f32 / 4.0 - 0.5;
                t.disque(c.x(gx + a * 0.8 * s), c.y(lane - g * (1.0 - g) * 2.4 * s * (1.0 - a.abs())), 0.8 + s, mul(hex(0xf2fbff), sky.amb), (1.0 - g) * 0.8, 0.4);
            }
        }
    }
}

/// La baleine au loin : son dos qui roule hors de l'eau, son souffle qui monte et se défait.
fn baleine(t: &mut Toile, c: &Cadre, sky: &Ciel, u: f32, lane: f32, f: f32) {
    let s = echelle(lane);
    let dos = (f * PI).sin();
    let (x, y) = (c.x(u), c.y(lane));
    let l = 5.0 * s * c.cw;
    let h = 0.9 * s * c.demi * dos;
    let pts: Vec<(f32, f32)> = (0..=16).map(|i| {
        let a = i as f32 / 16.0;
        (x - l / 2.0 + a * l, y - h * (a * PI).sin().powf(0.6))
    }).collect();
    t.poly(&pts, mul(hex(0x2a3440), sky.amb), 0.95);
    let souffle = (f - 0.15) / 0.5;
    if (0.0..1.0).contains(&souffle) {
        for k in 0..8 {
            let a = k as f32 / 7.0;
            let r = (1.0 + a * 2.5 * souffle) * s * c.demi * 0.5;
            t.disque(x + l * 0.2 + (a - 0.5) * souffle * 1.2 * c.cw * s, y - h - a * 3.5 * s * c.demi * souffle.sqrt(), r, mul(hex(0xf0f6fa), sky.amb), (1.0 - souffle) * 0.45, 0.8);
        }
    }
}

/// Une touffe de sargasses dorées, à plat sur la mer.
fn sargasse(t: &mut Toile, c: &Cadre, sky: &Ciel, u: f32, lane: f32, i: i64) {
    let s = echelle(lane);
    for k in 0..7 {
        let a = hasard(i * 31 + k) as f32;
        let b = hasard(i * 17 + k * 3) as f32;
        t.disque(c.x(u + (a - 0.5) * 3.0 * s), c.y(lane + (b - 0.5) * 0.5 * s), (0.6 + b) * s * c.demi * 0.45, mul(mix(hex(0xb08a2a), hex(0xd8b048), a), sky.amb), 0.85, 0.3);
    }
}

/// Un poisson volant qui file au ras de l'eau, ses ailes qui brillent.
fn poisson_volant(t: &mut Toile, c: &Cadre, sky: &Ciel, u: f32, lane: f32, f: f32) {
    let s = echelle(lane);
    let v = lane - (f * PI).sin() * 0.9 * s - 0.3;
    let (x, y) = (c.x(u), c.y(v));
    let l = 1.1 * s * c.cw;
    t.trait_((x - l, y + 0.5), (x + l, y), (0.25 * l).max(1.0), mul(hex(0x9ab8d0), sky.amb), 1.0);
    let ailes = 0.6 * l;
    t.poly(&[(x, y), (x - ailes, y - ailes * 0.8), (x - ailes * 0.3, y)], mul(hex(0xe0f0ff), sky.amb), 0.9);
    t.disque(x - l * 2.0 * f, y + 2.0, 0.8, mul(hex(0xf2fbff), sky.amb), 0.5, 0.4);
}

/// Ce qui est devant tout : le pavois de Saint-Malo qui claque au vent, les palmiers de
/// Guadeloupe qui se balancent.
fn premier_plan(t: &mut Toile, gardees: &mut Vignettes, c: &Cadre, sky: &Ciel, et: Etape, w: &Monde, t_: f32) {
    let room = c.room();
    match et {
        Etape::Bretagne => {
            let couleurs = [0xd04a2a, 0xf2c14e, 0x1d5a9a, 0xf4f4f0, 0x3c8a3c];
            let (x0, x1) = (12.0, room - 1.0);
            let corde = |u: f32| 2.0 + 2.5 * (PI * (u - x0) / (x1 - x0)).sin();
            let mut k = 0;
            let mut u = x0 + 0.6;
            while x1 - x0 >= 4.0 && u < x1 - 1.2 {
                let a = corde(u);
                let b = corde(u + 1.2);
                let vent = (t_ * 0.35 + u * 0.7).sin() * 0.35;
                let col = mul(hex(couleurs[k % couleurs.len()]), sky.amb);
                t.poly(&[(u * c.cw, c.y(a)), ((u + 1.2) * c.cw, c.y(b)), ((u + 0.6 + vent) * c.cw, c.y((a + b) / 2.0 + 1.6))], col, 1.0);
                k += 1;
                u += 1.8;
            }
        }
        Etape::Guadeloupe => {
            let palmiers: Vec<f32> = if room >= 40.0 { vec![room - 6.0, room - 15.0] } else { vec![room - 4.0] };
            let orage = w.weather == Some(Temps::Storm);
            for (i, &u) in palmiers.iter().enumerate() {
                // le vent par pas : une vignette chacun
                let graine = i as f32 * 2.3;
                let pas = ((t_ * 0.05 + graine + i as f32 * 1.5).sin() * 4.0).round();
                let vent = pas / 4.0 * if orage { 1.0 } else { 0.35 };
                let hauteur = 15.0 + graine;
                let cle = (-1 - i as i64, pas as i32, orage as i32, 0, 0);
                let v = gardees.entry(cle).or_insert_with(|| {
                    let (w, h) = ((22.0 * c.cw) as usize, ((hauteur + 9.0) * c.demi) as usize);
                    let (ox, oy) = (14.0 * c.cw, (hauteur + 6.0) * c.demi);
                    let (c, sky) = (*c, *sky);
                    Vignette::faire(w, h, ox, oy, &move |toile: &mut Toile, ox: f32, oy: f32| palmier(toile, &c, &sky, ox, oy, hauteur, vent))
                });
                v.poser(t, c.x(c.from + u), c.y(GROUND + 2.5));
            }
        }
        _ => {}
    }
    // la carte, par-dessus tout, tout le temps ; elle laisse la place au spectacle, le temps qu'il se joue
    if room >= 40.0 && fete(w).is_none() {
        carte(t, gardees, c, sky, w, t_);
    }
}

// --- la carte de la course : la flotte au dernier pointage --------------------------

/// Ce que montre la carte : de la Manche aux Antilles, en degrés.
const CARTE: (f32, f32, f32, f32) = (-66.0, 0.0, 12.0, 52.0);
const SAINT_MALO: (f32, f32) = (-2.02, 48.65);
const POINTE_A_PITRE: (f32, f32) = (-61.53, 16.24);

/// Les terres, à grands traits : (longitude, latitude).
const EUROPE_AFRIQUE: [(f32, f32); 42] = [
    (2.0, 53.0), (2.0, 51.0), (1.6, 50.1), (0.2, 49.7), (-1.3, 49.6), (-1.9, 49.7), (-1.6, 48.7), (-3.0, 48.8),
    (-4.8, 48.5), (-4.3, 47.8), (-2.5, 47.3), (-1.2, 46.2), (-1.3, 44.5), (-1.8, 43.4), (-3.8, 43.5), (-8.0, 43.7),
    (-9.3, 43.0), (-8.9, 41.9), (-9.5, 39.4), (-9.0, 38.5), (-8.9, 37.0), (-6.3, 36.8), (-5.6, 36.0), (-6.0, 35.8),
    (-6.9, 34.0), (-8.5, 33.3), (-9.8, 30.4), (-11.5, 28.0), (-13.0, 27.5), (-15.0, 24.5), (-16.1, 22.5), (-17.0, 21.0),
    (-16.5, 19.5), (-16.1, 18.0), (-16.8, 15.0), (-17.5, 14.7), (-16.7, 13.0), (-16.8, 12.0), (-16.8, 11.0), (2.0, 11.0),
    (2.0, 30.0), (2.0, 45.0),
];
const ANGLETERRE: [(f32, f32); 11] = [
    (-5.7, 50.0), (-3.0, 50.6), (1.4, 51.2), (1.7, 52.6), (0.0, 53.5), (-3.0, 53.4), (-4.6, 52.8), (-5.3, 51.7),
    (-4.2, 51.5), (-3.2, 51.4), (-5.0, 50.5),
];
const IRLANDE: [(f32, f32); 7] = [(-10.2, 51.6), (-8.0, 51.6), (-6.3, 52.2), (-6.0, 53.5), (-7.0, 54.0), (-10.0, 54.0), (-9.8, 53.0)];
/// Les îles, en points : Açores, Madère, Canaries, Cap-Vert, et l'arc des Antilles.
const ILES: [(f32, f32, f32); 17] = [
    (-25.6, 37.8, 0.5), (-28.1, 38.5, 0.5), (-27.2, 38.7, 0.4), (-31.1, 39.4, 0.3), (-16.9, 32.7, 0.4),
    (-15.5, 28.1, 0.4), (-13.7, 28.6, 0.4), (-16.6, 28.3, 0.4), (-17.9, 27.8, 0.3), (-23.6, 15.1, 0.4), (-24.4, 16.6, 0.4),
    (-61.6, 16.2, 0.6), (-61.35, 15.4, 0.4), (-61.0, 14.6, 0.5), (-61.8, 17.1, 0.35), (-66.4, 18.2, 0.9), (-62.8, 17.9, 0.3),
];
/// Les classes sur la carte : Ultim, Ocean Fifty, IMOCA, Class40, Rhum Multi, Rhum Mono.
const TEINTES: [u32; 6] = [0xd8402e, 0x8a4ad0, 0x1a1a24, 0xf0a020, 0x2e9a4a, 0x2a8ad8];

/// Un point de la route de Saint-Malo à Pointe-à-Pitre, sur l'arc de grand cercle : `f` de 0 à 1 ;
/// (longitude, latitude).
fn sur_la_route(f: f32) -> (f32, f32) {
    let vers = |p: (f32, f32)| {
        let (lo, la) = (p.0.to_radians(), p.1.to_radians());
        [la.cos() * lo.cos(), la.cos() * lo.sin(), la.sin()]
    };
    let (va, vb) = (vers(SAINT_MALO), vers(POINTE_A_PITRE));
    let om = (va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2]).acos();
    let (ka, kb) = (((1.0 - f) * om).sin() / om.sin(), (f * om).sin() / om.sin());
    let v = [ka * va[0] + kb * vb[0], ka * va[1] + kb * vb[1], ka * va[2] + kb * vb[2]];
    (v[1].atan2(v[0]).to_degrees(), v[2].atan2(v[0].hypot(v[1])).to_degrees())
}

/// La carte de la course, épinglée en haut à droite de la cour : la côte, les îles, la route de
/// Saint-Malo à Pointe-à-Pitre, la trace des premiers, et chaque bateau à sa place au dernier
/// pointage, le premier de chaque classe cerclé, ceux qui ont abandonné en gris. Sans pointage,
/// la flotte à l'estime : au port avant le départ, puis sur la route. Le papier se dessine une
/// fois ; le reste, à chaque image.
fn carte(t: &mut Toile, gardees: &mut Vignettes, c: &Cadre, sky: &Ciel, monde: &Monde, t_: f32) {
    let flotte: &[[f32; 4]] = monde.rhum_flotte.as_deref().unwrap_or(&[]);
    let room = c.room();
    let w = (room * c.cw * 0.34).min(300.0);
    let h = w * 0.66;
    let (x0, y0) = (room * c.cw - w - 1.2 * c.cw, 1.0 * c.demi);
    let (gx, gy) = (w * 0.04, h * 0.06); // la marge du papier
    let proj = move |lon: f32, lat: f32| -> (f32, f32) {
        let (l0, l1, b0, b1) = CARTE;
        (gx + (lon - l0) / (l1 - l0) * (w - 2.0 * gx), gy + (b1 - lat) / (b1 - b0) * (h - 2.0 * gy))
    };
    let cle = (-100, w as i32, h as i32, 0, 0);
    let papier = gardees.entry(cle).or_insert_with(|| {
        Vignette::faire(w as usize + 6, h as usize + 6, 0.0, 0.0, &move |t: &mut Toile, _, _| {
            // l'ombre, le papier, la mer, le cadre
            t.poly(&[(4.0, 4.0), (w + 4.0, 4.0), (w + 4.0, h + 4.0), (4.0, h + 4.0)], [0.0, 0.0, 0.0], 0.3);
            t.poly(&[(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)], hex(0xe9dcb8), 1.0);
            t.poly(&[(gx, gy), (w - gx, gy), (w - gx, h - gy), (gx, h - gy)], hex(0xbcd8e0), 1.0);
            let terre = hex(0xd2be8c);
            let trait_cote = hex(0x8a7a58);
            for cote in [&EUROPE_AFRIQUE[..], &ANGLETERRE[..], &IRLANDE[..]] {
                let pts: Vec<(f32, f32)> = cote.iter().map(|&(lo, la)| proj(lo.min(CARTE.1), la.clamp(CARTE.2, CARTE.3))).collect();
                t.poly(&pts, terre, 1.0);
                let mut ferme = pts.clone();
                ferme.push(pts[0]);
                t.ligne(&ferme, 0.8, trait_cote, 0.7);
            }
            for &(lo, la, r) in &ILES {
                let (x, y) = proj(lo, la);
                t.disque(x, y, (r * w / 120.0).max(1.0), terre, 1.0, 0.0);
                t.disque(x, y, (r * w / 120.0).max(1.0) + 0.5, trait_cote, 0.4, 0.8);
            }
            // la rose des vents, au large, dans le coin vide du sud-est
            let (rx, ry) = proj(-28.0, 20.0);
            let rr = w * 0.07;
            let encre = hex(0x6a5a3a);
            for k in 0..8 {
                let a = k as f32 * PI / 4.0;
                let long = if k % 2 == 0 { rr } else { rr * 0.55 };
                let (dx, dy) = (a.sin(), -a.cos());
                let cote = (dy * 0.18 * rr, -dx * 0.18 * rr);
                let pointe = (rx + dx * long, ry + dy * long);
                t.poly(&[(rx + cote.0, ry + cote.1), pointe, (rx - cote.0, ry - cote.1)], if k == 0 { hex(0xb03020) } else { encre }, 0.75);
            }
            t.disque(rx, ry, rr * 0.16, hex(0xe9dcb8), 1.0, 0.0);
            // la route, en tirets, sur l'arc de grand cercle
            let (a, b) = (SAINT_MALO, POINTE_A_PITRE);
            let point = |f: f32| {
                let (lo, la) = sur_la_route(f);
                proj(lo, la)
            };
            for k in 0..30 {
                if k % 2 == 0 {
                    t.trait_(point(k as f32 / 30.0), point((k + 1) as f32 / 30.0), 1.0, hex(0xb03020), 0.75);
                }
            }
            for (p, col) in [(a, 0x1d3f8a), (b, 0xb03020)] {
                let (x, y) = proj(p.0, p.1);
                t.disque(x, y, 2.6, hex(col), 1.0, 0.0);
                t.disque(x, y, 1.2, hex(0xffffff), 1.0, 0.0);
            }
            t.ligne(&[(0.5, 0.5), (w - 0.5, 0.5), (w - 0.5, h - 0.5), (0.5, h - 0.5), (0.5, 0.5)], 1.0, hex(0x6a5a3a), 1.0);
            // la punaise qui la tient
            t.disque(w * 0.5 + 1.0, 4.5, 3.4, [0.0, 0.0, 0.0], 0.3, 0.6);
            t.disque(w * 0.5, 3.5, 3.0, hex(0xc0392b), 1.0, 0.0);
            t.disque(w * 0.5 - 0.9, 2.6, 1.0, hex(0xffb0a0), 0.9, 0.3);
        })
    });
    papier.poser(t, x0, y0);
    // la trace des premiers, de leurs pointages passés à leur place
    for trace in monde.rhum_traces.as_deref().unwrap_or(&[]) {
        let Some((&classe, pts)) = trace.split_first() else { continue };
        let pts: Vec<(f32, f32)> = pts.chunks_exact(2).map(|p| {
            let (x, y) = proj(p[1], p[0]);
            (x0 + x, y0 + y)
        }).collect();
        if pts.len() >= 2 {
            t.ligne(&pts, 1.3, hex(TEINTES[(classe as usize).min(5)]), 0.75);
        }
    }
    // sans pointage, la flotte à l'estime : au port, puis là où le rythme d'un Ultim la met
    if flotte.is_empty() {
        let f = if monde.phase() == PhaseRhum::Course { monde.rhum_progress.unwrap_or(0.0) as f32 } else { 0.0 };
        let (lo, la) = sur_la_route(f.clamp(0.0, 1.0));
        let (x, y) = proj(lo, la);
        let palpite = 0.5 + 0.5 * (t_ * 0.25).sin();
        t.disque(x0 + x, y0 + y, 3.0 + 3.0 * palpite, hex(0xd8402e), 0.45 * (1.0 - palpite) + 0.1, 0.8);
        t.disque(x0 + x, y0 + y, 2.6, hex(0xffffff), 1.0, 0.0);
        t.disque(x0 + x, y0 + y, 1.7, hex(0xd8402e), 1.0, 0.0);
    }
    // la flotte : les petits d'abord, les Ultim par-dessus ; le premier de chaque classe cerclé,
    // et qui palpite doucement
    let r = (w / 110.0).clamp(1.2, 2.4);
    let mut ordre: Vec<&[f32; 4]> = flotte.iter().collect();
    ordre.sort_by(|a, b| (a[3] != 0.0).cmp(&(b[3] != 0.0)).then(b[0].total_cmp(&a[0])));
    for b in ordre {
        let (x, y) = proj(b[2], b[1]);
        // hors course : un point gris, sans éclat
        if b[3] == 0.0 {
            t.disque(x0 + x, y0 + y, r * 0.85, hex(0x8a8478), 0.7, 0.0);
            continue;
        }
        let col = hex(TEINTES[(b[0] as usize).min(5)]);
        if b[3] == 1.0 {
            t.disque(x0 + x, y0 + y, r * (2.0 + 0.4 * (t_ * 0.3).sin()), col, 0.35, 0.6);
            t.disque(x0 + x, y0 + y, r + 1.2, hex(0xffffff), 1.0, 0.0);
        }
        t.disque(x0 + x, y0 + y, r, col, 1.0, 0.0);
    }
    // la nuit, le papier n'a que la lueur du ciel
    if sky.nuit > 0.05 {
        t.poly(&[(x0, y0), (x0 + w, y0), (x0 + w, y0 + h), (x0, y0 + h)], hex(0x0a1430), sky.nuit.min(1.0) * 0.5);
    }
}

/// Un palmier sur le sable, son pied en (ox, oy) : son tronc courbe et bagué, ses palmes que le vent couche.
fn palmier(t: &mut Toile, c: &Cadre, sky: &Ciel, ox: f32, oy: f32, hauteur: f32, vent: f32) {
    let tronc: Vec<(f32, f32)> = (0..=10)
        .map(|i| {
            let a = i as f32 / 10.0;
            (ox + (-a * a * 3.0 - a * vent * 0.6) * c.cw, oy - a * hauteur * c.demi)
        })
        .collect();
    let brun = mul(hex(0x7a5a3a), sky.amb);
    for (k, s) in tronc.windows(2).enumerate() {
        let e = (1.0 - k as f32 / 12.0) * 0.9 * c.cw;
        t.trait_(s[0], s[1], e, if k % 2 == 0 { brun } else { fois(brun, 0.82) }, 1.0);
    }
    let (hx, hy) = tronc[10];
    let vert = mul(hex(0x2e8a3a), sky.amb);
    for j in 0..7 {
        let a = -PI + j as f32 / 6.0 * PI + vent * 0.25 + (j as f32 * 2.7).sin() * 0.06;
        let long = (5.5 + (j as f32 * 1.7).sin()) * c.cw;
        let mut nervure = Vec::new();
        for k in 0..=8 {
            let f = k as f32 / 8.0;
            let ang = a + f * 0.9 * if a.cos() > 0.0 { 1.0 } else { -1.0 } * 0.6;
            nervure.push((hx + ang.cos() * long * f, hy + ang.sin() * long * f * 0.7 + f * f * long * 0.45));
        }
        for (k, s) in nervure.windows(2).enumerate() {
            let e = (1.0 - k as f32 / 9.0) * 1.1 * c.cw;
            t.trait_(s[0], s[1], e, if j % 2 == 0 { vert } else { fois(vert, 0.8) }, 1.0);
        }
    }
    for k in 0..3 {
        t.disque(hx + (k as f32 - 1.0) * 0.5 * c.cw, hy + 0.6 * c.demi, 0.45 * c.cw, mul(hex(0x5a3a1a), sky.amb), 1.0, 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cadre(room: f32) -> Cadre {
        Cadre { left: 0, w: (room * 9.0) as usize, h: 304, cw: 9.0, demi: 9.5, from: 54.0, to: 54.0 + room }
    }

    fn monde(rhum: PhaseRhum, progres: f64, hour: f64) -> Monde {
        Monde { rhum: Some(rhum), rhum_progress: Some(progres), hour, month: 11, day: 3, clock: 1234, ..Monde::default() }
    }

    #[test]
    fn le_voyage_va_de_saint_malo_a_la_guadeloupe() {
        assert_eq!(etape(&monde(PhaseRhum::Avant, 0.5, 12.0)), Etape::Bretagne);
        let etapes: Vec<Etape> = [0.0, 0.1, 0.3, 0.5, 0.7, 0.95].iter().map(|&p| etape(&monde(PhaseRhum::Course, p, 12.0))).collect();
        assert_eq!(etapes, [Etape::Bretagne, Etape::Large, Etape::Acores, Etape::Large, Etape::Tropiques, Etape::Guadeloupe]);
    }

    /// Le spectacle du lancement : le départ se joue à Saint-Malo et l'arrivée en Guadeloupe, où
    /// qu'en soit la course ; au départ, la flotte s'aligne, file au coup de canon, et revient tirer
    /// ses bords quand le vrai départ n'est pas encore donné.
    #[test]
    fn le_spectacle_rejoue_le_depart_ou_l_arrivee() {
        use super::super::monde::SpectacleRhum;
        let mut w = monde(PhaseRhum::Course, 0.5, 12.0);
        w.rhum_fete = Some(SpectacleRhum { kind: FeteRhum::Depart, age: 10 });
        assert_eq!(etape(&w), Etape::Bretagne);
        w.rhum_fete = Some(SpectacleRhum { kind: FeteRhum::Arrivee, age: 10 });
        assert_eq!(etape(&w), Etape::Guadeloupe);
        w.rhum_fete = Some(SpectacleRhum { kind: FeteRhum::Arrivee, age: FETE });
        assert_eq!(etape(&w), Etape::Large, "fini, la course reprend sa place");
        let (from, to) = (54.0, 144.0);
        for b in flotte(to - from, None) {
            let ordinaire = cap(&b, 99.0, PhaseRhum::Avant, from, to, 1.0);
            let (u, _, phase) = au_depart(&b, COUP - 1.0, from, to, 1.0, Some(ordinaire));
            assert!((from..to).contains(&u) && phase == PhaseRhum::Avant, "{:?} aligné dans la baie : {u}", b.sorte);
            let (u, _, _) = au_depart(&b, COUP + SORTIE - 1.0, from, to, 1.0, Some(ordinaire));
            assert!(u > to, "{:?} parti au large : {u}", b.sorte);
            let (u, dir, _) = au_depart(&b, FETE as f32, from, to, 1.0, Some(ordinaire));
            assert_eq!((u, dir), ordinaire, "{:?} revenu à son bord", b.sorte);
        }
    }

    #[test]
    fn la_baie_se_peint_sous_les_cellules_transparentes_seulement() {
        let c = cadre(72.0);
        let mut px = vec![0u32; c.w * c.h];
        px[5] = 0xFF12_3456; // une cellule qui porte quelqu'un
        let mut baie = Baie::default();
        let zone = baie.peindre(&mut px, c.w, c.h, c, &monde(PhaseRhum::Course, 0.3, 14.0));
        assert_eq!(zone, Some((0, 0, c.w as i32, c.h as i32)));
        assert_eq!(px[5], 0xFF12_3456);
        assert!(px.iter().all(|p| p >> 24 == 0xFF), "tout le reste est peint");
        // la mer, bleue, sous l'horizon (en moyenne : des voiles passent)
        let y = c.y(21.0) as usize;
        let bleu = px[y * c.w..(y + 1) * c.w].iter().filter(|&&p| (p & 0xFF) > (p >> 16) & 0xFF).count();
        assert!(bleu > c.w * 2 / 3, "{bleu} pixels bleus sur {}", c.w);
    }

    #[test]
    fn la_flotte_avance_et_chaque_etape_se_peint() {
        let c = cadre(90.0);
        let mut baie = Baie::default();
        for (p, h) in [(0.0, 12.0), (0.1, 20.5), (0.3, 7.0), (0.7, 17.8), (0.95, 23.0)] {
            let w = monde(PhaseRhum::Course, p, h);
            let mut a = vec![0u32; c.w * c.h];
            baie.peindre(&mut a, c.w, c.h, c, &w);
            let mut b = vec![0u32; c.w * c.h];
            baie.peindre(&mut b, c.w, c.h, c, &w.plus_tard(20));
            let bouge = a.iter().zip(&b).filter(|(x, y)| x != y).count();
            assert!(bouge > 2000, "étape {:?} : {bouge} pixels ont bougé", etape(&w));
        }
    }

    #[test]
    fn la_carte_place_la_flotte_et_les_couleurs_des_premiers_passent_aux_bateaux() {
        let c = cadre(90.0);
        let mut w = monde(PhaseRhum::Course, 0.5, 12.0);
        let mut sans = vec![0u32; c.w * c.h];
        Baie::default().peindre(&mut sans, c.w, c.h, c, &w);
        // sans pointage, la carte est là, en haut à droite : son papier
        let haut_droite = || (0..c.h / 2).flat_map(|y| (c.w / 2..c.w).map(move |x| (x, y)));
        let papier = haut_droite().filter(|&(x, y)| sans[y * c.w + x] & 0xffffff == 0xe9dcb8).count();
        assert!(papier > 300, "le papier de la carte se voit : {papier} pixels");
        // un bateau au milieu de l'Atlantique : la carte le montre, et seule la flotte y change
        w.rhum_flotte = Some(vec![[0.0, 30.0, -35.0, 1.0]]);
        let mut avec = vec![0u32; c.w * c.h];
        Baie::default().peindre(&mut avec, c.w, c.h, c, &w);
        let change = haut_droite().filter(|&(x, y)| sans[y * c.w + x] != avec[y * c.w + x]).count();
        assert!(change > 20 && change < 2000, "le bateau se place : {change} pixels");
        // pendant le spectacle, la carte laisse la place
        w.rhum_fete = Some(super::super::monde::SpectacleRhum { kind: FeteRhum::Depart, age: 50 });
        let mut fete = vec![0u32; c.w * c.h];
        Baie::default().peindre(&mut fete, c.w, c.h, c, &w);
        assert_eq!(haut_droite().filter(|&(x, y)| fete[y * c.w + x] & 0xffffff == 0xe9dcb8).count(), 0);
        // le premier Ultim prête ses couleurs à l'Ultim de la baie
        let f = flotte(90.0, Some(&[[0, 0x123456, 0x654321]]));
        let ultim = f.iter().rev().find(|b| b.sorte == Sorte::Ultim).unwrap();
        assert_eq!((ultim.coque, ultim.bande), (hex(0x123456), hex(0x654321)));
    }

    #[test]
    fn avant_le_depart_les_bateaux_tirent_des_bords_dans_la_baie() {
        let (from, to) = (54.0, 126.0);
        for b in flotte(to - from, None) {
            let sens: Vec<f32> = (0..4000).step_by(50).map(|t| cap(&b, t as f32, PhaseRhum::Avant, from, to, 1.0).1).collect();
            assert!(sens.contains(&1.0) && sens.contains(&-1.0), "il vire de bord");
            for t in (0..4000).step_by(37) {
                let (u, _) = cap(&b, t as f32, PhaseRhum::Avant, from, to, 1.0);
                assert!(u >= from && u <= to, "il reste dans la baie");
            }
        }
    }
}
