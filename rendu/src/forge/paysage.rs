//! La cour au pixel fin (user, 03-11 : « pour le décor, oui, on peut faire avec des pixels plus fins. Mais toujours
//! garder quand même du stylisé… comme tu as fait avec la route du Rhum… on peut se permettre de donner de la vie, de
//! la lumière, de l'animation… plus d'animation, plus de profondeur »). Les mêmes choses que cour.rs, aux mêmes places
//! (son `Plan`), peintes au vrai pixel sous les cellules que la cour laisse transparentes, comme la baie de la Route
//! du Rhum : trois chaînes de montagnes de plus en plus bleues au loin, leurs versants au soleil ou à l'ombre selon
//! l'heure ; des cascades qui coulent ; deux rangées de sapins devant la forêt ; l'herbe, ses touffes, le chemin pavé ;
//! l'arbre et ses grappes de feuilles qui se balancent et tombent ; le puits et son seau qui descend de temps en temps ;
//! la roue qui tourne sous son bief, son étang, ses roseaux ; des oiseaux qui passent, des étoiles qui scintillent,
//! des lucioles l'été. La pluie et la neige restent celles des cellules, par-dessus.

use std::f32::consts::PI;

use super::baie::Cadre;
use super::cour::{self, Plan};
use super::monde::{Monde, Rarete, Saison};
use super::pixels::hasard;
use super::trace::{fois, hex, mix, mul, pack, palier, paliers, smooth, unpack, Rgb, Toile};

/// Le pixel d'art de la cour fine, en pixels d'écran : 2 × 2 (1 pixel d'art = 4 vrais pixels), sans tramage.
pub const GRAIN: usize = 2;
const PALIERS: u8 = 3;

const SOL: f32 = 28.0;
const HORIZON: f32 = 25.0;

/// `v` (de 0 à 1) arrondi à `n` paliers : les dégradés du pixel art en aplats, sans trame (user, 03-11 : « j'aime pas
/// du tout l'effet tramé… le dithering » ; la trame de Bayer est retirée).
fn trame(v: f32, n: f32, _x: usize, _y: usize) -> f32 {
    ((v.clamp(0.0, 1.0) * n).round() / n).clamp(0.0, 1.0)
}

fn h(n: i64) -> f32 {
    hasard(n) as f32
}

fn roule(x: f32, graine: i64) -> f32 {
    cour::roll(x as f64, graine) as f32
}

/// Le ciel de l'image (celui de cour.rs, que la lumière des acteurs suit aussi), en f32.
#[derive(Clone, Copy)]
struct Ciel {
    haut: Rgb,
    bas: Rgb,
    amb: Rgb,
    spill: f32,
    nuit: bool,
    soir: bool,
    orage: bool,
    arc: bool,
    /// d'où vient le soleil : −1 de gauche (le matin), 1 de droite (l'après-midi)
    cote: f32,
}

impl Ciel {
    fn de(w: &Monde) -> Ciel {
        let c = cour::ciel(w);
        let f = |a: [f64; 3]| [a[0] as f32, a[1] as f32, a[2] as f32];
        let heure = w.heure_ciel() as f32;
        Ciel { haut: f(c.top), bas: f(c.low), amb: f(c.amb), spill: c.spill as f32, nuit: c.night, soir: c.dusk, orage: c.storm, arc: c.arc, cote: ((heure - 12.5) / 3.5).clamp(-1.0, 1.0) }
    }

    /// Une couleur sous la lumière de l'heure, pâlie de `loin` vers le bas du ciel.
    fn lit(&self, c: Rgb, loin: f32) -> Rgb {
        mix(mul(c, self.amb), self.bas, loin)
    }
}

// les trois chaînes, en rangées de forge : la lointaine (nouvelle), la haute et la proche de cour.rs, et leur grain
fn lointaine_lisse(u: f32) -> f32 {
    // des pics isolés plutôt qu'un mur : la hauteur suit le bruit à la puissance 1,8, les cols descendent bas et
    // laissent passer le ciel (user, 13-11 : « ça fait toujours assez massif »)
    11.5 - 6.8 * roule(u / 11.0, 41).powf(1.8) - roule(u / 4.5, 42) * 1.3
}
fn haute_lisse(u: f32) -> f32 {
    8.0 + roule(u / 9.0, 1) * 6.0 + roule(u / 4.0, 2) * 2.0
}
fn proche_lisse(u: f32) -> f32 {
    15.0 + roule(u / 6.0, 3) * 5.0
}
fn lointaine(u: f32) -> f32 {
    lointaine_lisse(u) + roule(u / 1.4, 43) * 0.5
}
fn haute(u: f32) -> f32 {
    haute_lisse(u) + roule(u / 1.2, 44) * 0.5
}
fn proche(u: f32) -> f32 {
    proche_lisse(u) + roule(u / 1.7, 45) * 0.45
}

/// La demi-largeur d'un sapin à `d` sous sa pointe, `haut` de haut : trois étages, chacun s'évasant.
fn sapin(d: f32, haut: f32) -> f32 {
    if d < 0.0 {
        return -1.0;
    }
    let l = haut / 3.0;
    let t = (d / l).min(2.999);
    (t.floor() * 0.5 + t.fract() * 0.9 + 0.08) * l * 0.45
}

/// Une chaîne : sa crête par colonne de pixels, ses couleurs, sa distance.
struct Chaine {
    cretes: Vec<f32>,
    /// la même sans son grain : ses versants
    lisses: Vec<f32>,
    roche: [Rgb; 3],
    neige: f32,
    loin: f32,
    boisee: bool,
}

/// Ce que la cour fine garde d'une image à l'autre : le fond immobile, ce qu'il laisse voir du ciel, et la clé qui dit
/// quand le refaire.
#[derive(Default)]
pub struct Paysage {
    cle: Option<Vec<i64>>,
    fond: Vec<u32>,
    /// la part de ciel de chaque pixel du fond (255 : ciel seul) : les nuages et les étoiles passent derrière les monts
    ciel: Vec<u8>,
    /// la part de forêt de chaque pixel du fond : les cascades passent derrière les sapins
    bois: Vec<u8>,
    buf: Vec<u32>,
    /// le pixel d'art, en pixels d'écran (0 : `GRAIN`)
    pub grain: usize,
}

impl Paysage {
    /// Peint la cour fine sous les cellules transparentes du rectangle `c` de `pixels` ; rend ce rectangle.
    #[allow(clippy::too_many_arguments)]
    pub fn peindre(&mut self, pixels: &mut [u32], pw: usize, ph: usize, c: Cadre, w: &Monde, plan: &Plan, travail: bool) -> Option<(i32, i32, i32, i32)> {
        if c.w == 0 || c.h == 0 {
            return None;
        }
        // peinte en petit, un pixel d'art pour `n` × `n` pixels d'écran, puis agrandie sans lissage
        let n = if self.grain == 0 { GRAIN } else { self.grain };
        let ecran = c;
        let c = Cadre { left: 0, w: c.w.div_ceil(n), h: c.h.div_ceil(n), cw: c.cw / n as f32, demi: c.demi / n as f32, from: c.from, to: c.to };
        paliers(PALIERS);
        let sky = Ciel::de(w);
        let se = w.saison();
        let gala = w.galadriel();
        let cle = vec![
            c.w as i64,
            c.h as i64,
            (c.cw * 100.0) as i64,
            (c.demi * 100.0) as i64,
            c.from as i64,
            c.to as i64,
            se as i64,
            sky.nuit as i64,
            sky.soir as i64,
            sky.orage as i64,
            sky.arc as i64,
            (w.heure_ciel() * 10.0) as i64,
            gala as i64,
            (sky.amb[0] * 100.0) as i64,
            n as i64,
        ];
        if self.cle.as_ref() != Some(&cle) {
            self.fond = vec![0; c.w * c.h];
            self.ciel = vec![0; c.w * c.h];
            self.bois = vec![0; c.w * c.h];
            fond(&mut self.fond, &mut self.ciel, &mut self.bois, &c, &sky, w, se, plan, gala);
            self.cle = Some(cle);
        }
        let t = w.clock as f32;
        let mut buf = std::mem::take(&mut self.buf);
        buf.clear();
        buf.extend_from_slice(&self.fond);
        {
            let mut toile = Toile { px: &mut buf, w: c.w, h: c.h };
            etoiles(&mut toile, &self.ciel, &c, &sky, t);
            nuages(&mut toile, &self.ciel, &c, &sky, t);
            cascades(&mut toile, &self.bois, &c, &sky, plan, se, t);
            oiseaux(&mut toile, &c, &sky, w, t);
            arbre(&mut toile, &c, &sky, plan, se, t);
            puits(&mut toile, &c, &sky, plan, se, t);
            moulin(&mut toile, &c, &sky, plan, se, t, travail);
            meteo_et_menhir(&mut toile, &c, &sky, se, gala, t);
            if w.banquet && !gala {
                banquet(&mut toile, &c, &sky, t);
            }
            if !gala {
                petites_betes(&mut toile, &c, &sky, plan, se, t);
            }
            lucioles(&mut toile, &c, &sky, se, t);
            if !gala {
                avant_toit(&mut toile, &c, &sky, se, t);
            }
        }
        paliers(0);
        // agrandie, sous les cellules transparentes seulement : la porte, la pluie, la neige et ce qui se tient dans la
        // cour restent devant
        let (x0, y1) = (ecran.left.max(0) as usize, ecran.h.min(ph));
        let large = ecran.w.min(pw.saturating_sub(x0));
        for y in 0..y1 {
            let source = &buf[(y / n) * c.w..(y / n + 1) * c.w];
            let ligne = &mut pixels[y * pw + x0..y * pw + x0 + large];
            for (x, d) in ligne.iter_mut().enumerate() {
                if *d >> 24 == 0 {
                    *d = source[x / n];
                }
            }
        }
        self.buf = buf;
        Some((ecran.left, 0, ecran.left + ecran.w as i32, ecran.h as i32))
    }
}

/// Le fond immobile : le ciel et son astre, l'arc-en-ciel, les trois chaînes et leur neige, la forêt et ses sapins,
/// l'herbe et le chemin, le tronc de l'arbre, le puits sans son seau, le chevalet du bief et l'étang.
#[allow(clippy::too_many_arguments)]
fn fond(px: &mut [u32], voile: &mut [u8], bois: &mut [u8], c: &Cadre, sky: &Ciel, w: &Monde, se: Saison, plan: &Plan, gala: bool) {
    let (wd, hd) = (c.w, c.h);
    let hiver = se == Saison::Hiver;
    let room = c.to - c.from;
    let u_de = |x: usize| c.from + (x as f32 + 0.5) / c.cw;
    // le ciel, rangée après rangée
    for y in 0..hd {
        let v = (y as f32 + 0.5) / c.demi;
        let k = (v / HORIZON).clamp(0.0, 1.0).powf(1.15);
        for x in 0..wd {
            // un dégradé continu : arrondi à sept paliers, il se lisait en bandes (user : pas de « lumière en paliers »)
            px[y * wd + x] = pack(mix(sky.haut, sky.bas, k));
        }
    }
    // l'arc-en-ciel, après une série de réussites
    if sky.arc {
        let (cx, cy) = (c.from + room * 0.5, HORIZON + room * 0.22);
        let (rayon, ep) = (cy - 3.0, 4.5);
        let bandes = [hex(0xff6a5a), hex(0xffae4a), hex(0xfff07a), hex(0x7ee08a), hex(0x6aa8ff), hex(0xa682e8)];
        for y in 0..hd {
            let v = (y as f32 + 0.5) / c.demi;
            for x in 0..wd {
                let d = (u_de(x) - cx).hypot(v - cy);
                let k = (rayon - d) / ep;
                if !(0.0..1.0).contains(&k) {
                    continue;
                }
                let f = k * 5.0;
                let teinte = mix(bandes[f as usize], bandes[(f as usize + 1).min(5)], f.fract());
                let bord = (k * (1.0 - k) * 4.0).sqrt();
                let pied = (1.0 - (v - 4.0) / (HORIZON - 2.0)).clamp(0.0, 1.0).sqrt();
                let i = y * wd + x;
                px[i] = pack(mix(unpack(px[i]), teinte, 0.5 * bord * (0.35 + 0.65 * pied)));
            }
        }
    }
    {
        let mut t = Toile { px, w: wd, h: hd };
        let heure = w.heure_ciel() as f32;
        if !sky.nuit && !sky.orage {
            // le soleil, qui monte de la gauche et descend à droite, rougi le soir, derrière les monts quand il est bas
            let f = ((heure - 6.0) / 15.0).clamp(0.0, 1.0);
            let (u, v) = (c.from + room * (0.1 + 0.8 * f), 3.0 + 15.0 * (1.0 - (PI * f).sin()));
            let (halo, disque) = if sky.soir { (hex(0xffa860), hex(0xffd08a)) } else { (hex(0xfff0c0), hex(0xfffbe8)) };
            // au pixel d'art net : deux anneaux d'une seule teinte chacun plutôt qu'un halo dégradé (l'alpha en
            // paliers y traçait des cernes), puis le disque plein
            let (x, y, r) = (c.x(u).round() + 0.5, c.y(v).round() + 0.5, (1.5 * c.demi).round());
            t.disque(x, y, r + 5.0, halo, 0.16, 0.0);
            t.disque(x, y, r + 2.0, halo, 0.3, 0.0);
            t.disque(x, y, r, disque, 1.0, 0.0);
        } else if sky.nuit && !sky.orage {
            // la lune, un croissant
            let (u, v) = (c.from + room * 0.72, 4.5);
            let (x, y, r) = (c.x(u).round() + 0.5, c.y(v).round() + 0.5, (1.4 * c.demi).round());
            t.disque(x, y, r + 4.0, hex(0x8090c0), 0.12, 0.0);
            t.disque(x, y, r + 2.0, hex(0x8090c0), 0.14, 0.0);
            t.disque(x, y, r, hex(0xeef0f6), 1.0, 0.0);
            let fond_lune = mix(sky.haut, sky.bas, (v / HORIZON).powf(1.15));
            t.disque(x + (0.55 * c.cw).round(), y - (0.35 * c.demi).round(), r - 1.0, fond_lune, 1.0, 0.0);
        }
    }
    let dir = sky.cote;
    let cretes = |f: fn(f32) -> f32| (0..wd).map(|x| f(u_de(x))).collect::<Vec<f32>>();
    let chaines = [
        Chaine { cretes: cretes(lointaine), lisses: cretes(lointaine_lisse), roche: [hex(0x6c7c9e), hex(0x7d8fb0), hex(0x98a8c6)], neige: 0.95, loin: 0.7, boisee: false },
        Chaine { cretes: cretes(haute), lisses: cretes(haute_lisse), roche: [hex(0x48566e), hex(0x5d6e86), hex(0x7d8ea6)], neige: 1.1, loin: 0.42, boisee: false },
        Chaine { cretes: cretes(proche), lisses: cretes(proche_lisse), roche: [hex(0x2c404c), hex(0x3e5560), hex(0x587480)], neige: 0.0, loin: 0.28, boisee: true },
    ];
    // la pente des versants, lue sur une fenêtre large, là où aucun sommet ne domine (près des cols)
    let pas = (2.2 * c.cw).max(1.0) as isize;
    // Les sommets de chaque chaîne (les points hauts de sa crête lisse) : de chacun descendent deux faces, l'une au
    // soleil, l'autre à l'ombre, séparées par une arête qui serpente ; la neige coiffe le sommet et coule le long de
    // l'arête. Lus en colonnes droites, les versants faisaient un mur de glace (user, 12-11) ; c'est ce partage en
    // pyramides qui fait la haute montagne.
    let sommets: Vec<Vec<(f32, f32)>> = chaines
        .iter()
        .map(|ch| {
            let fen = (2.5 * c.cw).max(2.0) as usize;
            let mut s: Vec<(f32, f32)> = Vec::new();
            for x in 0..wd {
                let r = ch.lisses[x];
                let (lo, hi) = (x.saturating_sub(fen), (x + fen).min(wd - 1));
                if (lo..=hi).all(|j| ch.lisses[j] >= r) && s.last().is_none_or(|&(us, _)| u_de(x) - us > 1.5) {
                    s.push((u_de(x), r));
                }
            }
            s
        })
        .collect();
    // le soleil d'un côté ou de l'autre ; à midi, un côté reste un peu plus clair que l'autre
    let (sens, force) = (if dir >= 0.0 { 1.0 } else { -1.0 }, 0.45 + 0.55 * dir.abs());
    let peindre_chaines = |px: &mut [u32], voile: &mut [u8], indices: &[usize]| {
        for x in 0..wd {
            let u = u_de(x);
            for y in 0..hd {
                let v = (y as f32 + 0.5) / c.demi;
                if v >= SOL {
                    break;
                }
                let i = y * wd + x;
                let mut col = unpack(px[i]);
                let mut ciel = voile[i] as f32 / 255.0;
                for &n in indices {
                    let ch = &chaines[n];
                    let r = ch.cretes[x];
                    let cov = palier(((v - r) * c.demi + 0.5).clamp(0.0, 1.0));
                    if cov <= 0.0 {
                        continue;
                    }
                    let d = (v - r).max(0.0);
                    // le sommet qui tient ce pixel : le plus proche de ceux dont le cône (une pente de 35°) le contient
                    let mut tenu: Option<(f32, f32, f32)> = None;
                    for &(us, ts) in &sommets[n] {
                        let dv = v - ts;
                        if dv <= 0.0 {
                            continue;
                        }
                        let serpente = roule(dv / 1.6 + us * 0.37, 70 + n as i64) * 0.8 * (dv * 0.35).min(1.0);
                        let du = u - us - serpente;
                        if du.abs() <= dv * 1.4 && tenu.is_none_or(|t| du.abs() < t.0.abs()) {
                            tenu = Some((du, dv, du.abs() / (dv * 1.4)));
                        }
                    }
                    let face = match tenu {
                        Some((du, _, ecart)) => {
                            // la face au soleil s'éclaire vers l'arête, celle à l'ombre s'y assombrit
                            if du * sens > 0.0 {
                                0.5 + 0.5 * force * (1.0 - ecart * 0.35)
                            } else {
                                0.5 - 0.5 * force * (1.0 - ecart * 0.25)
                            }
                        }
                        None => {
                            let xs = x.clamp(pas as usize, wd - 1 - pas as usize);
                            let pente = (ch.lisses[xs + pas as usize] - ch.lisses[xs - pas as usize]) / (2.0 * pas as f32 / c.cw);
                            trame(smooth(-0.3, 0.3, pente * dir), 2.0, x, y)
                        }
                    };
                    let mut roche = if face < 0.5 { mix(ch.roche[0], ch.roche[1], face * 2.0) } else { mix(ch.roche[1], ch.roche[2], face * 2.0 - 1.0) };
                    if ch.boisee && d > 2.0 {
                        roche = mix(roche, mix(hex(0x24402e), hex(0x335a3c), face), trame(smooth(2.0, 4.5, d), 3.0, x, y) * 0.85);
                    }
                    let neige = if hiver { ch.neige + 2.6 + if n == 2 { 1.4 } else { 0.0 } } else { ch.neige } + roule(u * 1.3, 50 + n as i64) * 1.2 - 0.5 + (1.0 - face) * 0.6;
                    let neige = neige + if n == 1 { (11.5 - r).max(0.0) * 0.5 } else { 0.0 };
                    // sous un sommet, la neige se mesure depuis lui : elle descend loin le long de l'arête, en coulées
                    // dans les couloirs, et se retire vers le bord des faces
                    let (d, neige) = match tenu {
                        Some((du, dv, ecart)) => {
                            let arete = (1.0 - ecart).max(0.0);
                            let coulee = (roule(du * 2.2 + n as f32 * 3.1, 80 + n as i64) - 0.35).max(0.0) * 1.6;
                            (dv, neige * (0.55 + 1.5 * arete * arete) + coulee * arete)
                        }
                        None => (d, neige),
                    };
                    if (ch.neige > 0.0 || hiver) && d < neige {
                        let blanc = mix(hex(0xb4c4dc), hex(0xf4f7fb), face);
                        roche = mix(roche, blanc, trame(smooth(neige, neige - 0.3, d), 2.0, x, y));
                    }
                    let brume = smooth(r + 2.5, r + 9.0, v) * 0.22;
                    col = mix(col, sky.lit(roche, (ch.loin + brume).min(0.9)), cov);
                    ciel *= 1.0 - cov;
                }
                px[i] = pack(col);
                voile[i] = (ciel * 255.0) as u8;
            }
        }
    };
    for v in voile.iter_mut() {
        *v = 255;
    }
    peindre_chaines(px, voile, &[0, 1]);
    // entre la chaîne haute et la proche : silhouette d'un château aux tours pointues (façon Poudlard) et son viaduc
    chateau_et_viaduc(px, voile, c, sky, se);
    peindre_chaines(px, voile, &[2]);
    // la forêt : la rangée de derrière, la masse sombre, puis les sapins de devant (aux places de cour.rs) et ceux d'entre eux
    let teinte_sapin = |du: f32, d: f32, haut: f32, base: [Rgb; 3]| {
        let mut c = if du * dir > 0.05 { base[2] } else if du * dir < -0.05 { base[0] } else { base[1] };
        if hiver && (d / (haut / 3.0)).fract() < 0.3 {
            c = mix(c, if du * dir > 0.0 { hex(0xf2f6fa) } else { hex(0xc4d2e4) }, 0.9);
        }
        c
    };
    for x in 0..wd {
        let u = u_de(x);
        let masse = 25.4 + roule(u / 2.0, 60) * 0.6;
        for y in 0..hd {
            let v = (y as f32 + 0.5) / c.demi;
            if v < 16.0 {
                continue;
            }
            if v >= SOL {
                break;
            }
            let i = y * wd + x;
            let mut col = unpack(px[i]);
            let mut couvert = 0.0f32;
            // derrière : petits, serrés, bleuis
            let k = (u / 2.6).floor() as i64;
            for j in k - 1..=k + 1 {
                let cx = (j as f32 + 0.5 + (h(j * 3 + 1) - 0.5) * 0.9) * 2.6;
                let haut = 2.6 + h(j * 5 + 2) * 3.0;
                let d = v - (26.0 + (h(j * 5 + 9) - 0.5) * 0.8 - haut);
                let cov = palier(((sapin(d, haut) * (0.8 + h(j * 5 + 10) * 0.4) - (u - cx).abs()) * c.cw + 0.5).clamp(0.0, 1.0) * (d * c.demi + 0.5).clamp(0.0, 1.0));
                if cov > 0.0 {
                    col = mix(col, sky.lit(teinte_sapin(u - cx, d, haut, [hex(0x203c30), hex(0x2a4a3a), hex(0x3a5e48)]), 0.2), cov);
                    couvert = couvert.max(cov);
                }
            }
            let cov = palier(((v - masse) * c.demi + 0.5).clamp(0.0, 1.0));
            col = mix(col, sky.lit(hex(0x1c3626), 0.08), cov);
            couvert = couvert.max(cov);
            // entre ceux de devant, un peu plus petits
            let k = ((u - 2.5) / 5.0).round() as i64;
            for j in k - 1..=k + 1 {
                if h(j * 7 + 14) < 0.25 {
                    continue; // une clairière
                }
                let cx = j as f32 * 5.0 + 2.5 + (h(j * 7 + 11) - 0.5) * 2.2;
                let haut = 3.4 + h(j * 7 + 3) * 2.6;
                let d = v - (27.0 + (h(j * 7 + 12) - 0.5) * 0.8 - haut);
                let cov = palier(((sapin(d, haut) * (0.8 + h(j * 7 + 13) * 0.4) - (u - cx).abs()) * c.cw + 0.5).clamp(0.0, 1.0) * (d * c.demi + 0.5).clamp(0.0, 1.0));
                if cov > 0.0 {
                    let ton = 0.88 + h(j * 7 + 19) * 0.24;
                    let teinte = fois(teinte_sapin(u - cx, d, haut, [hex(0x1c3426), hex(0x264430), hex(0x365c3e)]), ton);
                    col = mix(col, sky.lit(teinte, 0.1), cov);
                    couvert = couvert.max(cov);
                }
            }
            let k = (u / 5.0).round() as i64;
            for j in k - 1..=k + 1 {
                if h(j * 5 + 14) < 0.1 {
                    continue;
                }
                let cx = j as f32 * 5.0 + (h(j * 5 + 11) - 0.5) * 2.4;
                let pointe = 22.0 - (h(j * 5) * 4.0).floor() - if h(j * 5 + 12) > 0.8 { 1.5 } else { 0.0 };
                let haut = 27.9 + (h(j * 5 + 13) - 0.5) * 0.6 - pointe;
                let d = v - pointe;
                let cov = palier(((sapin(d, haut) * (0.82 + h(j * 5 + 15) * 0.36) - (u - cx).abs()) * c.cw + 0.5).clamp(0.0, 1.0) * (d * c.demi + 0.5).clamp(0.0, 1.0));
                if cov > 0.0 {
                    // chaque sapin de devant de son ton, un peu plus clair ou plus sombre : une rangée de triangles
                    // tous pareils faisait un peigne
                    let ton = 0.86 + h(j * 5 + 17) * 0.28;
                    let teinte = fois(teinte_sapin(u - cx, d, haut, [hex(0x182e20), hex(0x23402c), hex(0x34583a)]), ton);
                    col = mix(col, sky.lit(teinte, 0.03), cov);
                    couvert = couvert.max(cov);
                }
            }
            px[i] = pack(col);
            bois[i] = (couvert * 255.0) as u8;
        }
    }
    // le sol : l'herbe de la saison en petites touches, le chemin pavé depuis la porte
    let (herbe, touffe, sombre) = match se {
        Saison::Automne => (hex(0x6f6f30), hex(0x8a7a34), hex(0x5a5a28)),
        Saison::Printemps => (hex(0x4f8a36), hex(0x6fb04a), hex(0x3e7028)),
        Saison::Ete => (hex(0x4c7a34), hex(0x62903e), hex(0x3c6a2a)),
        Saison::Hiver => (hex(0xeaf0f7), hex(0xf6f9fc), hex(0xc9d6e6)),
    };
    let y_sol = (SOL * c.demi) as usize;
    for y in y_sol.min(hd)..hd {
        let v = (y as f32 + 0.5) / c.demi;
        for x in 0..wd {
            let u = u_de(x);
            // de grandes plaques d'herbe plus claire ou plus sombre, et la lumière qui baisse vers nous
            let plaque = roule(u / 4.0, 63) * 0.6 + roule(u / 1.5 + v * 0.7, 64) * 0.4;
            let plaque = trame(plaque, 3.0, x, y);
            let mut col = if plaque > 0.55 { mix(herbe, touffe, (plaque - 0.55) * 1.6) } else { mix(sombre, herbe, plaque / 0.55 * 0.7 + 0.3) };
            col = fois(col, 1.0 - (v - SOL) * 0.03);
            if hiver {
                col = mix(col, hex(0xc9d6e6), roule(u / 3.0 + v, 62) * 0.35);
            }
            // le chemin dallé irrégulier aux joints moussus ; l'hiver, la neige n'en laisse que le seuil
            let long = if hiver { 6.0 } else { 14.2 - roule(v * 1.3, 61) * 2.8 };
            if !gala && u - c.from < long {
                let rang = ((v - SOL) / 0.95).floor();
                let decale = (rang as i64 % 2) as f32 * 0.85 + (h(rang as i64 * 19 + 3) - 0.5) * 0.35;
                let (fu, fv) = (((u - c.from + decale) / 2.1).fract(), ((v - SOL) / 0.95).fract());
                let pave = ((u - c.from + decale) / 2.1).floor() as i64;
                let joint = fu < 0.08 || fu > 0.92 || fv < 0.11 || fv > 0.89 || ((fu - 0.5).abs() > 0.39 && (fv - 0.5).abs() > 0.34);
                col = if joint {
                    if !hiver && h(pave * 31 + rang as i64 * 17 + x as i64) > 0.45 {
                        hex(0x3b5c2e)
                    } else {
                        hex(0x463d46)
                    }
                } else {
                    let base = [hex(0x7c737c), hex(0x6b6370), hex(0x887e88), hex(0x746c72)][(h(pave * 13 + rang as i64 * 5) * 4.0) as usize % 4];
                    if fv < 0.28 {
                        mix(base, hex(0xa89ea8), 0.38)
                    } else if fv > 0.76 {
                        fois(base, 0.86)
                    } else {
                        base
                    }
                };
            }
            // au fond de la cour, l'herbe se fond un peu dans la brume
            let loin = (1.0 - (v - SOL) / 2.5).clamp(0.0, 1.0) * 0.12;
            px[y * wd + x] = pack(sky.lit(col, loin));
        }
    }
    let mut t = Toile { px, w: wd, h: hd };
    // des brins d'herbe ; l'automne des feuilles tombées, au printemps des fleurs, l'été quelques boutons d'or
    if !hiver {
        for i in 0..(room * 9.0) as i64 {
            let u = c.from + h(i * 7 + 3) * room;
            if !gala && u - c.from < 13.0 {
                continue;
            }
            let v = SOL + 0.3 + h(i * 7 + 4) * 3.7;
            let haut = (0.3 + h(i * 7 + 5) * 0.35) * (0.7 + (v - SOL) * 0.12);
            let teinte = if h(i * 7 + 6) > 0.5 { touffe } else { sombre };
            let penche = (h(i * 7 + 8) - 0.5) * 0.3;
            t.trait_((c.x(u), c.y(v)), (c.x(u + penche), c.y(v - haut)), 1.3, sky.lit(teinte, 0.0), 0.9);
        }
        let (n, couleurs): (f32, &[u32]) = match se {
            Saison::Automne => (2.0, &[0xd06a22, 0xe3a63c, 0xb84818]),
            Saison::Printemps => (1.2, &[0xf6b8d0, 0xfff6e0, 0xffd75e]),
            _ => (0.4, &[0xfff0a0]),
        };
        for i in 0..(room * n) as i64 {
            let u = c.from + h(i * 11 + 1) * room;
            if !gala && u - c.from < 13.0 {
                continue;
            }
            let v = SOL + 0.4 + h(i * 11 + 2) * 3.5;
            let k = couleurs[(h(i * 11 + 3) * couleurs.len() as f32) as usize % couleurs.len()];
            let (x, y) = (c.x(u), c.y(v));
            if se == Saison::Automne {
                let a = h(i * 11 + 4) * PI;
                t.trait_((x - a.cos() * 2.0, y - a.sin() * 1.0), (x + a.cos() * 2.0, y + a.sin() * 1.0), 2.0, sky.lit(hex(k), 0.0), 1.0);
            } else {
                t.disque(x, y, 1.3, sky.lit(hex(k), 0.0), 1.0, 0.0);
            }
        }
    }
    // ce qui suit se tient devant les cascades (user, 03-11 : « des cascades qui passent par-dessus le toit du puits ») :
    // ce que ça peint entre dans le masque `bois`
    let avant = t.px.to_vec();
    // menhir runique ancien au bord du chemin dallé (ses runes s'éveillent d'une lueur pâle la nuit dans `meteo_et_menhir`)
    if !gala && room >= 20.0 {
        let um = c.from + 15.2;
        let lit_m = |k: u32| sky.lit(hex(k), 0.0);
        // ombre au pied et mousse
        t.poly(
            &[(c.x(um - 1.5), c.y(SOL + 0.9)), (c.x(um + 1.8), c.y(SOL + 0.9)), (c.x(um + 1.4), c.y(SOL + 1.35)), (c.x(um - 1.2), c.y(SOL + 1.35))],
            lit_m(0x283224),
            0.7,
        );
        // fût de granit taillé en pointe asymétrique
        let pierre_m = [
            (c.x(um - 1.15), c.y(SOL + 1.0)),
            (c.x(um - 0.95), c.y(SOL - 3.3)),
            (c.x(um - 0.25), c.y(SOL - 4.7)),
            (c.x(um + 0.55), c.y(SOL - 4.2)),
            (c.x(um + 1.05), c.y(SOL - 2.4)),
            (c.x(um + 1.2), c.y(SOL + 1.0)),
        ];
        t.poly(&pierre_m, lit_m(0x646070), 1.0);
        let cote_clair = if dir > 0.0 { um + 0.72 } else { um - 0.78 };
        t.trait_((c.x(cote_clair), c.y(SOL - 4.1)), (c.x(cote_clair), c.y(SOL + 0.7)), 1.5, lit_m(0x9490a4), 0.85);
        t.trait_((c.x(um - 0.2), c.y(SOL - 4.6)), (c.x(um + 0.5), c.y(SOL - 4.1)), 1.3, lit_m(0xa8a4b8), 0.85);
        // entaille runique gravée dans la pierre (sombre le jour)
        for k in 0..4 {
            let rv = SOL - 3.4 + k as f32 * 0.95;
            t.trait_((c.x(um - 0.05), c.y(rv)), (c.x(um - 0.05), c.y(rv + 0.65)), 1.1, lit_m(0x363242), 0.9);
            let br = if k % 2 == 0 { 0.35 } else { -0.35 };
            t.trait_((c.x(um - 0.05), c.y(rv + 0.2)), (c.x(um - 0.05 + br), c.y(rv + 0.5)), 1.0, lit_m(0x363242), 0.9);
        }
        if hiver {
            t.poly(&[(c.x(um - 0.8), c.y(SOL - 3.6)), (c.x(um - 0.25), c.y(SOL - 4.75)), (c.x(um + 0.6), c.y(SOL - 4.15)), (c.x(um + 0.1), c.y(SOL - 3.7))], lit_m(0xf2f6fc), 1.0);
        } else {
            for (du, dv, r_m) in [(-0.85, 0.75, 0.45), (-0.35, 0.88, 0.38), (0.65, 0.82, 0.42), (-0.7, -0.9, 0.28)] {
                t.disque(c.x(um + du), c.y(SOL + dv), r_m * c.demi, lit_m(0x3d6c32), 0.95, 0.1);
            }
        }
    }
    // grand Mallorn / chêne féerique : racines noueuses moussues, écorce argent-doré, creux elfique et champignons au pied
    let (tx, r) = (plan.tree.0 as f32 + 0.5, plan.tree.1 as f32);
    let ty = r + 0.5;
    {
        let (haut, bas) = (ty + 3.0, SOL + 1.6);
        let ecorce = if se == Saison::Automne { 0x62503c } else { 0x564230 };
        let ecorce_claire = if se == Saison::Automne { 0x968268 } else { 0x7e644a };
        let pts = [
            (tx - 0.65, haut),
            (tx + 0.65, haut),
            (tx + 0.95, bas - 2.2),
            (tx + 1.55, bas - 0.9),
            (tx + 2.75, bas + 0.15),
            (tx + 1.15, bas + 0.1),
            (tx + 0.35, bas - 0.25),
            (tx - 0.45, bas + 0.15),
            (tx - 1.35, bas + 0.05),
            (tx - 2.65, bas + 0.15),
            (tx - 1.5, bas - 0.95),
            (tx - 0.95, bas - 2.2),
        ];
        let pts: Vec<(f32, f32)> = pts.iter().map(|&(u, v)| (c.x(u), c.y(v))).collect();
        t.poly(&pts, sky.lit(hex(ecorce), 0.0), 1.0);
        let clair = if dir > 0.0 { 0.32 } else { -0.48 };
        t.trait_((c.x(tx + clair), c.y(haut)), (c.x(tx + clair * 1.45), c.y(bas - 0.5)), 0.42 * c.cw, sky.lit(hex(ecorce_claire), 0.0), 0.9);
        // nervures des racines noueuses et mousse
        for (u0, v0, u1, v1) in [
            (tx - 0.7, bas - 1.8, tx - 2.45, bas + 0.08),
            (tx - 0.15, bas - 1.4, tx - 0.55, bas + 0.12),
            (tx + 0.65, bas - 1.8, tx + 2.55, bas + 0.08),
        ] {
            t.trait_((c.x(u0), c.y(v0)), (c.x(u1), c.y(v1)), 0.28 * c.cw, sky.lit(hex(ecorce_claire), 0.0), 0.75);
            if !hiver {
                t.trait_((c.x((u0 + u1) * 0.5), c.y((v0 + v1) * 0.5 + 0.15)), (c.x(u1), c.y(v1)), 1.3, sky.lit(hex(0x3d6e32), 0.0), 0.75);
            }
        }
        // petit creux arqué au cœur du tronc
        t.poly(
            &[(c.x(tx - 0.28), c.y(bas - 2.2)), (c.x(tx), c.y(bas - 2.85)), (c.x(tx + 0.28), c.y(bas - 2.2)), (c.x(tx + 0.24), c.y(bas - 1.3)), (c.x(tx - 0.24), c.y(bas - 1.3))],
            sky.lit(hex(0x22140c), 0.0),
            0.95,
        );
        for k in 0..4 {
            let du = (h(k * 9 + 4) - 0.5) * 0.9;
            let (v0, v1) = (haut + 1.0 + h(k * 9 + 5) * 5.5, haut + 2.0 + h(k * 9 + 6) * 8.0);
            t.trait_((c.x(tx + du), c.y(v0)), (c.x(tx + du * 1.1), c.y(v1.min(bas - 2.9))), 1.0, sky.lit(hex(0x2e1c10), 0.0), 0.8);
        }
        // champignons féeriques au pied des racines (luminescents au crépuscule et la nuit)
        if !hiver {
            for (du, dv, or_champi) in [(-2.1, 0.05, false), (-1.65, 0.22, true), (1.55, 0.18, false), (2.05, 0.05, true), (2.4, 0.26, false)] {
                let (mu, mv) = (tx + du, bas + dv);
                t.trait_((c.x(mu), c.y(mv)), (c.x(mu), c.y(mv - 0.42)), 1.2, sky.lit(hex(0xe8dec8), 0.0), 1.0);
                let chapeau = if sky.nuit || sky.soir {
                    if or_champi { hex(0xffd86b) } else { hex(0x66f0ff) }
                } else if or_champi {
                    hex(0xdca044)
                } else {
                    hex(0x6aa8b8)
                };
                if sky.nuit || sky.soir {
                    t.disque(c.x(mu), c.y(mv - 0.48), 0.9 * c.demi, chapeau, 0.26, 1.0);
                    t.poly(&[(c.x(mu - 0.35), c.y(mv - 0.38)), (c.x(mu), c.y(mv - 0.72)), (c.x(mu + 0.35), c.y(mv - 0.38))], chapeau, 1.0);
                } else {
                    t.poly(&[(c.x(mu - 0.35), c.y(mv - 0.38)), (c.x(mu), c.y(mv - 0.72)), (c.x(mu + 0.35), c.y(mv - 0.38))], sky.lit(chapeau, 0.0), 1.0);
                }
            }
        }
    }
    // le puits médiéval : margelle de pierre moussue, montants à ferrures, toit pointu d'ardoise à épi de fer forgé
    if let Some(p) = plan.well {
        let p = p as f32;
        let lit = |c: u32, ombre: bool| sky.lit(fois(hex(c), if ombre { 0.8 } else { 1.04 }), 0.0);
        // les montants de chêne et leurs consoles
        for (a, b) in [(p + 1.0, p + 1.85), (p + 7.15, p + 8.0)] {
            t.poly(&[(c.x(a), c.y(17.8)), (c.x(b), c.y(17.8)), (c.x(b), c.y(24.2)), (c.x(a), c.y(24.2))], lit(0x4e321a, a > p + 5.0), 1.0);
            let bord = if dir > 0.0 { b - 0.2 } else { a + 0.2 };
            t.trait_((c.x(bord), c.y(18.0)), (c.x(bord), c.y(24.0)), 1.5, lit(0x78502e, false), 0.85);
            let sens = if a < p + 4.0 { 1.0 } else { -1.0 };
            t.trait_((c.x((a + b) * 0.5), c.y(19.2)), (c.x((a + b) * 0.5 + sens * 1.1), c.y(18.1)), 1.4, lit(0x3e2614, false), 1.0);
        }
        // le treuil bagué de fer
        t.poly(&[(c.x(p + 1.8), c.y(19.5)), (c.x(p + 7.2), c.y(19.5)), (c.x(p + 7.2), c.y(20.5)), (c.x(p + 1.8), c.y(20.5))], lit(0x744622, false), 1.0);
        t.trait_((c.x(p + 1.8), c.y(19.7)), (c.x(p + 7.2), c.y(19.7)), 1.2, lit(0x9e6e3c, false), 0.85);
        for u_fer in [p + 2.2, p + 6.8] {
            t.trait_((c.x(u_fer), c.y(19.45)), (c.x(u_fer), c.y(20.55)), 1.4, lit(0x4a4e5c, false), 1.0);
        }
        // le toit pointu d'ardoise (chapeau de sorcière / tourelle médiévale) et son épi en fer forgé
        let sommet = (p + 4.5, 13.5);
        let gauche_ombre = dir > 0.0;
        t.poly(
            &[(c.x(p - 0.8), c.y(18.55)), (c.x(p + 4.5), c.y(18.55)), (c.x(sommet.0), c.y(sommet.1)), (c.x(p + 1.2), c.y(16.3))],
            lit(0x4d5268, gauche_ombre),
            1.0,
        );
        t.poly(
            &[(c.x(p + 4.5), c.y(18.55)), (c.x(p + 9.8), c.y(18.55)), (c.x(p + 7.8), c.y(16.3)), (c.x(sommet.0), c.y(sommet.1))],
            lit(0x5c627a, !gauche_ombre),
            1.0,
        );
        // rangs d'ardoises écaillées et sablière de bois
        for v_ard in [15.1, 16.3, 17.4] {
            let k = (v_ard - sommet.1) / (18.55 - sommet.1);
            let demi = 0.4 + k * 4.8;
            t.trait_((c.x(p + 4.5 - demi), c.y(v_ard)), (c.x(p + 4.5 + demi), c.y(v_ard)), 1.1, lit(0x323646, false), 0.85);
        }
        t.trait_((c.x(p - 0.85), c.y(18.55)), (c.x(p + 9.85), c.y(18.55)), 0.28 * c.demi, lit(0x3a2416, false), 1.0);
        // épi de faîtage en fer forgé et étoile d'or terni
        t.trait_((c.x(sommet.0), c.y(sommet.1 + 0.1)), (c.x(sommet.0), c.y(sommet.1 - 1.15)), 1.4, lit(0x3a3e4c, false), 1.0);
        t.trait_((c.x(sommet.0 - 0.45), c.y(sommet.1 - 0.65)), (c.x(sommet.0 + 0.45), c.y(sommet.1 - 0.65)), 1.1, lit(0xc8a24e, false), 1.0);
        if hiver {
            t.poly(
                &[(c.x(p - 0.5), c.y(18.35)), (c.x(p + 2.4), c.y(18.1)), (c.x(sommet.0), c.y(sommet.1)), (c.x(p + 6.8), c.y(18.1)), (c.x(p + 9.5), c.y(18.35)), (c.x(p + 7.5), c.y(16.1)), (c.x(sommet.0), c.y(sommet.1 - 0.1)), (c.x(p + 1.5), c.y(16.1))],
                lit(0xeef4fa, false),
                0.92,
            );
        } else {
            for du in [-0.2, 1.6, 6.9, 8.6] {
                t.disque(c.x(p + du), c.y(18.25), 0.32 * c.demi, lit(0x467436, false), 0.9, 0.1);
            }
        }
        // la margelle de pierre taillée et ses touches de mousse
        let (gauche, droite) = (p + 0.3, p + 8.7);
        t.poly(&[(c.x(gauche), c.y(24.0)), (c.x(droite), c.y(24.0)), (c.x(droite), c.y(30.0)), (c.x(gauche), c.y(30.0))], lit(0x464250, false), 1.0);
        for rang in 0..6 {
            let v0 = 24.0 + rang as f32;
            let decale = if rang % 2 == 0 { 0.0 } else { 0.9 };
            let mut a = gauche + 0.08 - decale;
            let mut k = 0;
            while a < droite {
                let b = (a + 1.8).min(droite - 0.08);
                let a2 = a.max(gauche + 0.08);
                if b > a2 + 0.2 {
                    let ombre = (a2 + b) * 0.5 - p > 5.0 && dir <= 0.0 || (a2 + b) * 0.5 - p < 4.0 && dir > 0.0;
                    let base = [0x888294, 0x767082, 0x948ea2][(h(rang * 17 + k * 5 + p as i64) * 3.0) as usize % 3];
                    t.poly(&[(c.x(a2), c.y(v0 + 0.1)), (c.x(b), c.y(v0 + 0.1)), (c.x(b), c.y(v0 + 0.92)), (c.x(a2), c.y(v0 + 0.92))], lit(base, ombre), 1.0);
                    t.trait_((c.x(a2 + 0.1), c.y(v0 + 0.22)), (c.x(b - 0.1), c.y(v0 + 0.22)), 1.2, lit(0xb4aec2, ombre), 0.6);
                    if !hiver && h(rang * 31 + k * 13) > 0.72 {
                        t.trait_((c.x(a2 + 0.15), c.y(v0 + 0.88)), (c.x((a2 + b) * 0.5), c.y(v0 + 0.88)), 1.2, lit(0x3f6e34, ombre), 0.85);
                    }
                }
                a += 1.8;
                k += 1;
            }
        }
        t.poly(&[(c.x(gauche - 0.3), c.y(23.55)), (c.x(droite + 0.3), c.y(23.55)), (c.x(droite + 0.3), c.y(24.3)), (c.x(gauche - 0.3), c.y(24.3))], lit(if hiver { 0xeef2f8 } else { 0xa8a2b6 }, false), 1.0);
    }
    // la roue : son étang, sa rive, le ruisseau qui vient vers nous ; et l'aqueduc de pierre moussue à arches et lierre
    if let Some(cx) = plan.wheel {
        let cx = cx as f32 + 0.5;
        let etang = etang(cx, c.to);
        let rive: Vec<(f32, f32)> = etang.iter().map(|&(u, v)| (c.x(u + if u < cx - 4.0 { -0.6 } else { 0.0 }), c.y(v - 0.35))).collect();
        t.poly(&rive, sky.lit(if hiver { hex(0xdde6f0) } else { hex(0x3c5a2a) }, 0.0), 1.0);
        let pts: Vec<(f32, f32)> = etang.iter().map(|&(u, v)| (c.x(u), c.y(v))).collect();
        t.poly(&pts, sky.lit(if hiver { hex(0xd8ecf6) } else { hex(0x4f9ccc) }, 0.0), 1.0);
        for k in 0..7 {
            let (u, v) = (cx - 15.0 + h(k * 5 + 1) * 8.0, 29.0 + h(k * 5 + 2) * 3.0);
            t.disque(c.x(u), c.y(v), 0.3 * c.demi, sky.lit(hex(0x8a8494), 0.0), 1.0, 0.1);
        }
        // piles et arches de l'aqueduc de pierre sous le bief (de derrière la roue jusqu'au bord droit)
        let lit_aq = |k: u32| sky.lit(hex(k), 0.0);
        let p0 = cx + 4.2;
        let pas_arche = 5.4;
        let mut up = p0;
        let mut idx = 0i64;
        while up < c.to + 3.0 {
            // pile de pierre taillée et sa base élargie dans l'étang
            if up > cx + 2.5 {
                t.poly(&[(c.x(up - 0.75), c.y(11.8)), (c.x(up + 0.75), c.y(11.8)), (c.x(up + 0.9), c.y(27.6)), (c.x(up - 0.9), c.y(27.6))], lit_aq(0x6c6678), 1.0);
                let bord = if dir > 0.0 { up + 0.55 } else { up - 0.55 };
                t.trait_((c.x(bord), c.y(12.0)), (c.x(bord), c.y(27.3)), 1.4, lit_aq(0x9690a4), 0.8);
                t.poly(&[(c.x(up - 1.15), c.y(26.4)), (c.x(up + 1.15), c.y(26.4)), (c.x(up + 1.25), c.y(27.8)), (c.x(up - 1.25), c.y(27.8))], lit_aq(0x585264), 1.0);
            }
            // écoinçon d'arche entre cette pile et la suivante
            let u_suiv = up + pas_arche;
            let n_seg = 10;
            for s in 0..n_seg {
                let f0 = s as f32 / n_seg as f32;
                let f1 = (s + 1) as f32 / n_seg as f32;
                let ua = up + 0.65 + f0 * (pas_arche - 1.3);
                let ub = up + 0.65 + f1 * (pas_arche - 1.3);
                let va = 15.6 - (f0 * PI).sin() * 2.7;
                let vb = 15.6 - (f1 * PI).sin() * 2.7;
                t.poly(&[(c.x(ua), c.y(11.8)), (c.x(ub), c.y(11.8)), (c.x(ub), c.y(vb)), (c.x(ua), c.y(va))], lit_aq(0x625c6e), 1.0);
                t.trait_((c.x(ua), c.y(va)), (c.x(ub), c.y(vb)), 1.3, lit_aq(0x8e889c), 0.85);
            }
            // lierre grimpant et pendant sous l'arche
            if !hiver {
                for l in 0..4 {
                    let ul = up + 0.4 + h(idx * 17 + l * 5) * (pas_arche - 0.8);
                    let long_l = 1.6 + h(idx * 23 + l * 7) * 3.2;
                    t.trait_((c.x(ul), c.y(12.2)), (c.x(ul + (h(l * 11) - 0.5) * 0.5), c.y(12.2 + long_l)), 1.5, lit_aq(0x2f5e2c), 0.9);
                    t.disque(c.x(ul), c.y(12.4 + long_l * 0.5), 0.28 * c.demi, lit_aq(0x48823c), 0.9, 0.0);
                }
            }
            up = u_suiv;
            idx += 1;
        }
    }
    for (i, (p, a)) in t.px.iter().zip(&avant).enumerate() {
        if p != a {
            bois[i] = 255;
        }
    }
    // la lueur de la forge par la porte, le soir et la nuit
    if sky.spill > 0.0 && !gala {
        for y in (22.0 * c.demi) as usize..hd {
            let v = (y as f32 + 0.5) / c.demi;
            for x in 0..wd {
                let u = u_de(x);
                let d = ((u - c.from) / 1.6).hypot((v - 31.0) * 1.4);
                let f = (1.0 - d / 14.0).max(0.0).powi(2) * sky.spill;
                if f > 0.0 {
                    let i = y * wd + x;
                    let p = unpack(t.px[i]);
                    t.px[i] = pack([p[0] + 230.0 * f, p[1] + 135.0 * f, p[2] + 63.0 * f]);
                }
            }
        }
    }
}

/// L'étang sous la roue et le ruisseau qui en sort vers nous (les rangées mouillées de cour.rs, adoucies).
fn etang(cx: f32, to: f32) -> Vec<(f32, f32)> {
    vec![
        (cx - 8.3, 27.15),
        (to + 1.0, 27.15),
        (to + 1.0, 29.0),
        (cx - 7.4, 29.0),
        (cx - 9.2, 30.0),
        (cx - 10.4, 31.0),
        (cx - 10.8, 32.6),
        (cx - 17.6, 32.6),
        (cx - 15.6, 31.0),
        (cx - 13.6, 30.0),
        (cx - 10.8, 29.0),
        (cx - 10.4, 28.0),
    ]
}

/// Silhouette lointaine d'un château aux tours pointues (façon Poudlard) juché sur un éperon de la chaîne haute,
/// prolongé d'un viaduc de pierre à arches au-dessus de la vallée brumeuse ; fenêtres dorées le soir et la nuit.
fn chateau_et_viaduc(px: &mut [u32], voile: &mut [u8], c: &Cadre, sky: &Ciel, se: Saison) {
    let room = c.to - c.from;
    if room < 16.0 {
        return;
    }
    let hiver = se == Saison::Hiver;
    let u_min = c.from + 7.5;
    let u_max = c.from + (room * 0.42).clamp(11.0, 40.0);
    let mut uc = (u_min + u_max) * 0.5;
    let mut vc = haute_lisse(uc);
    let mut u_scan = u_min;
    while u_scan <= u_max {
        let v_cand = haute_lisse(u_scan);
        if v_cand < vc {
            vc = v_cand;
            uc = u_scan;
        }
        u_scan += 0.5;
    }
    let vc = vc.clamp(7.8, 13.2);
    let avant = px.to_vec();
    let loin_ch = 0.34;
    let dir = sky.cote;
    let lit = |k: u32| sky.lit(hex(k), loin_ch);
    let pierre_ombre = lit(0x3b3d54);
    let pierre_base = lit(0x525672);
    let pierre_clair = lit(0x727898);
    let toit_ombre = lit(if hiver { 0x7e8ca8 } else { 0x26293c });
    let toit_clair = lit(if hiver { 0xd8e2f2 } else { 0x3e4460 });

    let mut t = Toile { px, w: c.w, h: c.h };

    // viaduc à arches reliant l'éperon vers la droite au-dessus de la brume, jusqu'à un pilier de roche coiffé d'une
    // tour de garde
    let mut bout_viaduc: Option<(f32, f32)> = None;
    if room >= 25.0 {
        let v_tab = vc + 0.55;
        let u0 = uc + 2.7;
        let u1 = (uc + 13.2).min(c.to - 1.5);
        if u1 > u0 + 3.5 {
            bout_viaduc = Some((u1, v_tab));
            t.poly(
                &[(c.x(u0), c.y(v_tab)), (c.x(u1), c.y(v_tab)), (c.x(u1), c.y(v_tab + 0.65)), (c.x(u0), c.y(v_tab + 0.65))],
                pierre_base,
                1.0,
            );
            t.trait_((c.x(u0), c.y(v_tab)), (c.x(u1), c.y(v_tab)), 1.2, if hiver { lit(0xe8eef8) } else { pierre_clair }, 0.95);
            let pas = 2.1;
            let mut up = u0 + 0.35;
            while up < u1 - 0.5 {
                // les piles descendent jusque derrière la chaîne proche, peinte après : jamais coupées dans le vide
                let h_pile = 14.0;
                t.poly(
                    &[
                        (c.x(up - 0.28), c.y(v_tab + 0.5)),
                        (c.x(up + 0.28), c.y(v_tab + 0.5)),
                        (c.x(up + 0.36), c.y(v_tab + h_pile)),
                        (c.x(up - 0.36), c.y(v_tab + h_pile)),
                    ],
                    pierre_ombre,
                    1.0,
                );
                let bord = if dir > 0.0 { up + 0.2 } else { up - 0.2 };
                t.trait_((c.x(bord), c.y(v_tab + 0.6)), (c.x(bord), c.y(v_tab + h_pile - 0.2)), 1.0, pierre_clair, 0.75);
                if up + pas < u1 + 0.2 {
                    for s in 0..6 {
                        let f0 = s as f32 / 6.0;
                        let f1 = (s + 1) as f32 / 6.0;
                        let ua = up + 0.24 + f0 * (pas - 0.48);
                        let ub = up + 0.24 + f1 * (pas - 0.48);
                        let va = v_tab + 1.55 - (f0 * PI).sin() * 0.85;
                        let vb = v_tab + 1.55 - (f1 * PI).sin() * 0.85;
                        t.poly(
                            &[(c.x(ua), c.y(v_tab + 0.55)), (c.x(ub), c.y(v_tab + 0.55)), (c.x(ub), c.y(vb)), (c.x(ua), c.y(va))],
                            pierre_base,
                            1.0,
                        );
                    }
                }
                up += pas;
            }
        }
    }

    // l'éperon rocheux qui porte le château (et le pilier où finit le viaduc) : un rocher qui sort de la chaîne haute et
    // s'élargit vers le bas jusque derrière la chaîne proche, éclairé du côté du soleil, cerné, des névés sur ses
    // corniches ; sans lui, le château flottait sur une dalle
    let eperon = |t: &mut Toile, g0: f32, d0: f32, v_haut: f32, graine: i64| {
        let roche = [lit(0x343e52), lit(0x46526a), lit(0x5c6a84), lit(0x74849e)];
        let neige = lit(if hiver { 0xf2f6fa } else { 0xc8d2e2 });
        let (y_a, y_b) = (c.y(v_haut).floor() as i32, c.y(v_haut + 14.0).ceil() as i32);
        // des assises de roche : à chaque corniche (tous les ~1,3 rang), la paroi ressort ou rentre d'un coup, de
        // chaque côté ; la masse s'élargit vers le bas
        let bords = |y: i32| -> (f32, f32) {
            let v = (y as f32 + 0.5) / c.demi - v_haut;
            let assise = (v / 1.3).floor();
            let sg = (h(assise as i64 * 17 + graine) - 0.5) * 1.1;
            let sd = (h(assise as i64 * 23 + graine + 3) - 0.5) * 1.1;
            (c.x(g0 - v * 0.45 + sg), c.x(d0 + v * 0.4 + sd))
        };
        for y in y_a.max(0)..y_b.min(t.h as i32) {
            let v = (y as f32 + 0.5) / c.demi - v_haut;
            let (g, d) = bords(y);
            let (gh, dh) = bords(y - 1);
            let (xa, xb) = (g.round() as i32, d.round() as i32);
            let assise = (v / 1.3).floor() as i64;
            for x in xa..xb {
                let f = (x - xa) as f32 / (xb - xa).max(1) as f32;
                let cote = if dir >= 0.0 { f } else { 1.0 - f };
                // des facettes de roche : la lumière selon le côté, troublée d'un bruit lisse qui penche (pas de pans
                // réguliers, qui faisaient de la maçonnerie)
                let pan = roule(x as f32 / (1.6 * c.cw) + v * 0.7, graine + assise);
                let k = (cote * 2.4 + pan * 1.6 - 0.5).clamp(0.0, 3.0) as usize;
                let mut col = roche[k.min(3)];
                // le dessus de chaque corniche, là où la paroi rentre, prend la neige (ou le névé l'été)
                let corniche = (x as f32) < gh.round() || (x as f32) >= dh.round();
                if corniche && y > y_a {
                    col = neige;
                }
                if y == y_a {
                    col = roche[3];
                }
                // le flanc à l'ombre s'assombrit au bord ; pas de cerne noir autour
                let ombre = if dir >= 0.0 { x == xa } else { x == xb - 1 };
                if ombre {
                    col = mix(col, hex(0x1c2030), 0.4);
                }
                t.poser(x, y, col, 1.0);
            }
        }
    };
    eperon(&mut t, uc - 3.6, uc + 3.3, vc + 0.6, 71);

    // une tour cylindrique coiffée d'une flèche pointue
    let tour_pointue = |t: &mut Toile, ut: f32, larg: f32, v_base: f32, v_sommet: f32, h_fleche: f32| {
        let g = ut - larg * 0.5;
        let dr = ut + larg * 0.5;
        t.poly(&[(c.x(g), c.y(v_sommet)), (c.x(dr), c.y(v_sommet)), (c.x(dr), c.y(v_base)), (c.x(g), c.y(v_base))], pierre_base, 1.0);
        let (c0, c1) = if dir > 0.0 { (ut, dr) } else { (g, ut) };
        t.poly(&[(c.x(c0), c.y(v_sommet)), (c.x(c1), c.y(v_sommet)), (c.x(c1), c.y(v_base)), (c.x(c0), c.y(v_base))], pierre_clair, 0.55);
        // encorbellement sous la flèche
        t.trait_((c.x(g - 0.18), c.y(v_sommet)), (c.x(dr + 0.18), c.y(v_sommet)), 1.4, pierre_ombre, 1.0);
        // flèche conique élancée à deux versants
        let pointe = (c.x(ut), c.y(v_sommet - h_fleche));
        t.poly(&[(c.x(g - 0.25), c.y(v_sommet)), (c.x(ut), c.y(v_sommet)), pointe], if dir < 0.0 { toit_clair } else { toit_ombre }, 1.0);
        t.poly(&[(c.x(ut), c.y(v_sommet)), (c.x(dr + 0.25), c.y(v_sommet)), pointe], if dir >= 0.0 { toit_clair } else { toit_ombre }, 1.0);
        // épi de faîtage
        t.trait_(pointe, (pointe.0, pointe.1 - 1.5), 1.0, pierre_clair, 0.9);
    };

    if let Some((u1, v_tab)) = bout_viaduc {
        eperon(&mut t, u1 - 0.7, u1 + 0.9, v_tab + 0.65, 83);
        tour_pointue(&mut t, u1 + 0.1, 0.9, v_tab + 0.7, v_tab - 1.6, 1.6);
    }

    // Grande Salle à nef haute et pignon gothique (à gauche du donjon)
    let (gs0, gs1) = (uc - 2.4, uc + 0.25);
    t.poly(&[(c.x(gs0), c.y(vc - 1.3)), (c.x(gs1), c.y(vc - 1.3)), (c.x(gs1), c.y(vc + 1.0)), (c.x(gs0), c.y(vc + 1.0))], pierre_base, 1.0);
    t.poly(
        &[(c.x(gs0 - 0.15), c.y(vc - 1.3)), (c.x(gs1 + 0.1), c.y(vc - 1.3)), (c.x(gs1 - 0.1), c.y(vc - 2.65)), (c.x(gs0 + 0.25), c.y(vc - 2.65))],
        if dir >= 0.0 { toit_clair } else { toit_ombre },
        1.0,
    );
    // pignon pointu ouest et clocheton
    t.poly(&[(c.x(gs0 - 0.15), c.y(vc - 1.3)), (c.x(gs0 + 0.55), c.y(vc - 1.3)), (c.x(gs0 + 0.2), c.y(vc - 2.95))], pierre_clair, 0.85);

    // tour ouest de garde
    tour_pointue(&mut t, uc - 2.95, 1.05, vc + 1.0, vc - 3.1, 2.2);
    // grand donjon d'astronomie central et ses deux échauguettes
    tour_pointue(&mut t, uc + 0.85, 1.45, vc + 1.1, vc - 4.7, 3.1);
    tour_pointue(&mut t, uc + 0.05, 0.62, vc - 1.8, vc - 3.8, 1.5);
    tour_pointue(&mut t, uc + 1.62, 0.62, vc - 2.1, vc - 4.1, 1.5);
    // galerie couverte et tour de la volière à l'est
    t.poly(&[(c.x(uc + 1.5), c.y(vc - 0.6)), (c.x(uc + 2.4), c.y(vc - 0.6)), (c.x(uc + 2.4), c.y(vc + 0.3)), (c.x(uc + 1.5), c.y(vc + 0.3))], pierre_ombre, 1.0);
    tour_pointue(&mut t, uc + 2.55, 0.95, vc + 0.9, vc - 2.7, 2.0);

    // le jour, des baies sombres : le château a des fenêtres même éteint
    if !(sky.nuit || sky.soir) {
        let baie = lit(0x23263a);
        for &(uw, vw) in &[(uc - 2.95, vc - 2.1), (uc + 0.85, vc - 3.7), (uc + 0.72, vc - 2.3), (uc + 2.55, vc - 1.7), (uc - 1.6, vc - 0.65), (uc - 0.6, vc - 0.65)] {
            let (wx, wy) = (c.x(uw).round() as i32, c.y(vw).round() as i32);
            t.poser(wx, wy, baie, 0.9);
            t.poser(wx, wy + 1, baie, 0.9);
        }
    }
    // fenêtres dorées allumées au crépuscule et dans la nuit
    if sky.nuit || sky.soir {
        let or_vif = if sky.nuit { hex(0xffd468) } else { hex(0xffbe52) };
        let or_doux = hex(0xe89232);
        // lancettes de la Grande Salle
        for k in 0..4 {
            let uw = gs0 + 0.45 + k as f32 * 0.55;
            let (wx, wy) = (c.x(uw).round() as i32, c.y(vc - 0.65).round() as i32);
            t.poser(wx, wy, or_vif, 0.95);
            t.poser(wx, wy + 1, or_doux, 0.85);
        }
        // fenêtres des tours
        for &(uw, vw) in &[
            (uc - 2.95, vc - 2.1),
            (uc - 2.95, vc - 0.8),
            (uc + 0.85, vc - 3.7),
            (uc + 0.72, vc - 2.3),
            (uc + 1.05, vc - 1.1),
            (uc + 2.55, vc - 1.7),
        ] {
            t.poser(c.x(uw).round() as i32, c.y(vw).round() as i32, or_vif, 0.92);
        }
    }

    for (i, (&p, &a)) in t.px.iter().zip(&avant).enumerate() {
        if p != a {
            voile[i] = 0;
        }
    }
}

/// Un disque vu à travers la part de ciel du fond : derrière les monts, rien.
#[allow(clippy::too_many_arguments)]
fn disque_ciel(t: &mut Toile, voile: &[u8], cx: f32, cy: f32, r: f32, c: Rgb, a: f32, doux: f32) {
    if r <= 0.0 || a <= 0.0 {
        return;
    }
    let (xa, xb) = ((cx - r - 1.0).floor().max(0.0) as i32, ((cx + r + 1.0).ceil() as i32).min(t.w as i32));
    let (ya, yb) = ((cy - r - 1.0).floor().max(0.0) as i32, ((cy + r + 1.0).ceil() as i32).min(t.h as i32));
    for y in ya..yb {
        let dy = y as f32 + 0.5 - cy;
        for x in xa..xb {
            let ciel = voile[y as usize * t.w + x as usize];
            if ciel == 0 {
                continue;
            }
            let dx = x as f32 + 0.5 - cx;
            let d = (dx * dx + dy * dy).sqrt();
            if d >= r + 0.5 {
                continue;
            }
            let net = (r - d + 0.5).min(1.0);
            let f = 1.0 - doux * (d / r).min(1.0);
            t.poser(x, y, c, a * net * f * f * ciel as f32 / 255.0);
        }
    }
}

/// Les étoiles qui scintillent, la nuit claire.
fn etoiles(t: &mut Toile, voile: &[u8], c: &Cadre, sky: &Ciel, temps: f32) {
    if !sky.nuit || sky.orage {
        return;
    }
    let room = c.to - c.from;
    for i in 0..(room * 1.4) as i64 {
        let (u, v) = (c.from + h(i * 7 + 1) * room, h(i * 13 + 5) * 16.0);
        let eclat = 0.55 + 0.45 * (temps * (0.15 + h(i * 3) * 0.3) + i as f32).sin();
        let grosse = h(i * 11 + 2) > 0.85;
        let teinte = if h(i + 9) > 0.6 { hex(0xfaf5dc) } else { hex(0xaab4dc) };
        disque_ciel(t, voile, c.x(u), c.y(v), if grosse { 1.3 } else { 0.7 }, teinte, eclat, 0.3);
        if grosse && eclat > 0.85 {
            for (dx, dy) in [(2.0, 0.0), (-2.0, 0.0), (0.0, 2.0), (0.0, -2.0)] {
                disque_ciel(t, voile, c.x(u) + dx, c.y(v) + dy, 0.6, teinte, 0.5, 0.3);
            }
        }
    }
}

/// Les nuages qui passent derrière les monts, de droite à gauche, au pixel d'art net (user, 13-11 : « le ciel peut
/// être mieux, comme les nuages » ; l'étalon du propre, les engrenages) : des bosses rondes sur un ventre plat, un
/// liseré de lumière en haut de chaque bosse, l'ombre dans les jointures et sous le ventre, en aplats francs, sans
/// flou ; plus vite, plus nombreux et plus sombres dans l'orage.
fn nuages(t: &mut Toile, voile: &[u8], c: &Cadre, sky: &Ciel, temps: f32) {
    let room = c.to - c.from;
    let n = ((room / 22.0).ceil() as i64).max(1) + if sky.orage { 3 } else { 0 };
    let vitesse = if sky.orage { 0.05 } else { 0.012 };
    let tour = room + 34.0;
    // le liseré, le corps, la jointure, l'ombre, le ventre
    let teintes: [u32; 5] = if sky.orage {
        [0x7a7e8c, 0x5e6270, 0x50546a, 0x44485a, 0x34384a]
    } else if sky.nuit {
        [0x4a5680, 0x34406a, 0x2e3962, 0x283258, 0x1c2444]
    } else if sky.soir {
        [0xffe2cc, 0xf6c0aa, 0xdaa2a2, 0xb47e92, 0x8a5a7a]
    } else {
        [0xfbfdff, 0xe8eff8, 0xd0dcea, 0xb8c8dc, 0x9cb0c8]
    };
    let a = if sky.nuit { 0.75 } else { 1.0 };
    for i in 0..n {
        let depart = h(i * 41 + 3) * tour;
        let u = c.to + 14.0 - (depart + temps * vitesse * (0.7 + 0.6 * h(i * 3))).rem_euclid(tour);
        let image: &[&str] = match ((h(i * 11) * 6.0) as i64, sky.orage) {
            (0 | 1, _) | (2 | 3, true) => &NUAGE_GRAND,
            (2 | 3, false) => &NUAGE_MOYEN,
            (4, _) => &NUAGE_LONG,
            _ => &NUAGE_PETIT,
        };
        let (iw, ih) = (image[0].len() as i32, image.len() as i32);
        let (gauche, haut) = (c.x(u).round() as i32 - iw / 2, c.y(3.6 + h(i * 7 + 1) * 4.2).round() as i32 - ih);
        for (j, ligne) in image.iter().enumerate() {
            for (k, l) in ligne.bytes().enumerate() {
                let teinte = match l {
                    b'W' => teintes[0],
                    b'B' => teintes[1],
                    b'M' => teintes[2],
                    b'S' => teintes[3],
                    b'D' => teintes[4],
                    _ => continue,
                };
                let (x, y) = (gauche + k as i32, haut + j as i32);
                if x < 0 || y < 0 || x >= t.w as i32 || y >= t.h as i32 {
                    continue;
                }
                let ciel = voile[y as usize * t.w + x as usize];
                if ciel > 0 {
                    t.poser(x, y, hex(teinte), a * ciel as f32 / 255.0);
                }
            }
        }
    }
}

const NUAGE_GRAND: [&str; 18] = [
    ".....................WWWWWWWWWW...................",
    "....................WBBBBBBBBBBW..................",
    ".............WWWWWWWBBBBBBBBBBBBW.................",
    "............WBBBBBBWMBBBBBBBBBBBB.................",
    "...........WBBBBBBBBWMBBBBBBBBBMMWWWW.............",
    "..........WBBBBBBBBBBWMBBBBBBBMWWBBBBWW...........",
    ".........WBBBBBBBBBMMMWBBBBBBMWMMBBBBBBW..........",
    "......WWWWMMBBBBBMMWWWWMMBBMMWWWWMMBBBMMWWWW......",
    "....WWBBBBWWBBBBBWWBBBBWWBBWWBBBBWWBBBWWBBBBWW....",
    "....BBBBBBBBMBBBMBBBBBBBBMMBBBBBBBBMBMBBBBBBBB....",
    "...WBBBBBBBBWBBBWBBBBBBBBWWBBBBBBBBWBWBBBBBBBBW...",
    "...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB...",
    "...BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB...",
    "...SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSS...",
    "....SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSS....",
    "....DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD....",
    "..................................................",
    "..................................................",
];
const NUAGE_MOYEN: [&str; 14] = [
    ".................WWWWWWWW...........",
    "................WBBBBBBBBW..........",
    "...........WWWW.MBBBBBBBBB..........",
    ".........WWBBBBWWBBBBBBBBBW.........",
    ".........BBBBBBMMMMBBBBBBMWWWW......",
    ".....WWWWMBBBBMWWWWMBBBBMWBBBBW.....",
    "....WBBBBWMBBMWBBBBWMBBBWBBBBBBW....",
    "...WBBBBBBWBBWBBBBBBWBBBBBBBBBBB....",
    "...BBBBBBBBBBBBBBBBBBBBBBBBBBBBB....",
    "...SSSSSSSSSSSSSSSSSSSSSSSSSSSSS....",
    "...SSSSSSSSSSSSSSSSSSSSSSSSSSSS.....",
    "....DDDDDDDDDDDDDDDDDDDDDDDDDD......",
    "....................................",
    "....................................",
];
const NUAGE_PETIT: [&str; 9] = [
    "........WWWW..........",
    ".......WBBBBW.........",
    "......WBBBBBBW........",
    "...WWWWMBBBBBMWWWW....",
    "..WBBBBWBBBBBWBBBBW...",
    "..SSSSSSSSSSSSSSSSS...",
    "..SSSSSSSSSSSSSSSSS...",
    "..DDDDDDDDDDDDDDDDD...",
    "......................",
];
const NUAGE_LONG: [&str; 10] = [
    "........................WWWWWWWW..........................",
    "..................WWWW..BBBBBBBB..........................",
    "..........WWWW...WBBBBWWBBBBBBBBW..WWWW...................",
    ".........WBBBBW.WBBBBBBWBBBBBBBBB.WBBBBW...WWWW...........",
    "....WWWWWBBBBBBWBBBBBBBBBBBBBBBBBWBBBBBBW.WBBBBW.WWWW.....",
    "...WBBBBWBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBWBBBBW....",
    "...SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSS....",
    "...SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSS....",
    "...DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD....",
    "..........................................................",
];


//// Les cascades des cols : de minces filets qui sortent d'une entaille de la roche (un col en V, jamais un sommet),
/// sautent de corniche en corniche (un pas de côté et un bouillon d'écume à chaque saut), dans une coulée de roche
/// mouillée qui les tient au flanc ; la proche dévale une paroi de rochers qui perce la forêt et finit en embrun parmi
/// les sapins. Gelées l'hiver.
fn cascades(t: &mut Toile, bois: &[u8], c: &Cadre, sky: &Ciel, plan: &Plan, se: Saison, temps: f32) {
    let hiver = se == Saison::Hiver;
    for &(fx, pres) in plan.falls() {
        let fx = fx as f32 + 0.5;
        let crete = if pres { proche(fx) } else { haute(fx) };
        let haut = crete + if pres { 1.6 } else { 0.9 };
        let fin = if pres { 23.6 } else { proche(fx) };
        if fin <= haut {
            continue;
        }
        let graine = fx as i64 * 7 + if pres { 3 } else { 0 };
        let loin = if pres { 0.08 } else { 0.3 };
        // en u : un filet mince au loin, plus large de près
        let large = if pres { 1.1 } else { 0.5 };
        let vitesse = if pres { 1.6 } else { 0.9 };
        let (y_ent, y0, y1) = (((crete + 0.1) * c.demi) as i32, (haut * c.demi) as i32, (fin * c.demi) as i32);
        // une corniche tous les ~1,6 rang : le filet y fait un pas de côté
        let saut = 1.6;
        let place = |v: f32| {
            let k = ((v - haut) / saut).floor();
            fx + ((h(k as i64 * 29 + graine) - 0.5) * 2.2).round() * 0.32
        };
        let (tw, th) = (t.w, t.h);
        let devant_de = |x: i32, y: i32| {
            let (xc, yc) = ((x.max(0) as usize).min(tw - 1), (y.max(0) as usize).min(th - 1));
            1.0 - bois[yc * tw + xc] as f32 / 255.0
        };
        // la proche : une paroi de rochers en assises irrégulières, qui sort de la forêt
        if pres {
            // les teintes de la chaîne proche elle-même : la paroi en fait partie (plus sombre, elle faisait un bloc)
            let roche = [hex(0x34485a), hex(0x466070), hex(0x5c7888), hex(0x7894a2)];
            for y in (y0 - (0.8 * c.demi) as i32)..y1 {
                let v = (y as f32 + 0.5) / c.demi;
                let assise = ((v - haut) / 1.1).floor() as i64;
                let evase = ((v - haut) / (fin - haut)).clamp(0.0, 1.0) * 1.2;
                let (mg, md) = (0.3 + evase + h(assise * 11 + graine) * 0.6, 0.3 + evase + h(assise * 17 + graine) * 0.6);
                let a = place(v.max(haut));
                let (xa, xb) = (c.x(a - mg).round() as i32, c.x(a + large + md).round() as i32);
                let haut_assise = ((v - haut) / 1.1).rem_euclid(1.0) < 0.2;
                for x in xa..xb {
                    let devant = devant_de(x, y);
                    if devant <= 0.0 {
                        continue;
                    }
                    let f = (x - xa) as f32 / (xb - xa).max(1) as f32;
                    let cote = if sky.cote >= 0.0 { f } else { 1.0 - f };
                    let pan = roule(x as f32 / (1.4 * c.cw) + v * 0.8, graine + assise);
                    let k = (cote * 2.2 + pan * 1.6 - 0.4).clamp(0.0, 3.0) as usize;
                    let col = if haut_assise && hiver { hex(0xeef3f8) } else { roche[k.min(3)] };
                    t.poser(x, y, sky.lit(col, 0.28), devant);
                }
            }
        }
        // la coulée de roche mouillée, un pixel de part et d'autre du filet, qui naît en pointe sous la crête
        let mouille = sky.lit(if pres { hex(0x1e2826) } else { hex(0x34404e) }, loin);
        for y in y_ent..y1 {
            let v = (y as f32 + 0.5) / c.demi;
            let a = place(v.max(haut));
            let ouvre = ((v - crete) / (haut - crete + 0.6)).clamp(0.0, 1.0);
            let (xa, xb) = (c.x(a).floor() as i32 - 1, c.x(a + large * ouvre.max(0.4)).ceil() as i32 + 1);
            for x in xa..xb {
                let devant = devant_de(x, y);
                if devant > 0.0 {
                    let bord = x == xa || x == xb - 1;
                    t.poser(x, y, mouille, (if bord { 0.35 } else { 0.6 }) * devant * ouvre.max(0.3));
                }
            }
        }
        // l'eau
        for y in y0..y1 {
            let v = (y as f32 + 0.5) / c.demi;
            let a = place(v);
            // l'écume où il retombe, juste sous chaque corniche
            let bouillon = ((v - haut) / saut).rem_euclid(1.0) < 0.12 && v > haut + 0.5;
            let lg = large * (0.5 + 0.5 * ((v - haut) / 2.0).clamp(0.0, 1.0));
            let ga = (h(y as i64 * 7 + graine) > 0.7) as i32;
            let (xa, xb) = (c.x(a).floor() as i32 - ga, c.x(a + lg).ceil() as i32);
            for x in xa..xb {
                let devant = devant_de(x, y);
                if devant <= 0.0 {
                    continue;
                }
                let i = (x - xa).max(0);
                let teinte = if hiver {
                    if h((x / 2) as i64 * 3 + (y / 3) as i64 * 7) > 0.5 { hex(0xe4f2fa) } else { hex(0xbfe0f0) }
                } else if bouillon {
                    if h(x as i64 * 5 + y as i64 + (temps as i64 / 2)) > 0.3 { hex(0xffffff) } else { hex(0xd6f0fc) }
                } else {
                    let phase = h(i as i64 * 13 + graine) * 7.0;
                    let s = (y as f32 - temps * vitesse * (0.8 + h(i as i64 * 7 + graine) * 0.4) + phase).rem_euclid(7.0);
                    if x == xb - 1 && xb - xa > 2 {
                        hex(0x6ab4e2)
                    } else if s < 2.5 {
                        hex(0xf8fdff)
                    } else if s < 4.5 {
                        hex(0xc8eafa)
                    } else {
                        hex(0x8ccaee)
                    }
                };
                t.poser(x, y, sky.lit(teinte, loin), devant);
            }
        }
        // l'embrun au pied : parmi les pointes des sapins pour la proche, sur la crête qui cache le pied des lointaines
        if !hiver {
            let respire = 0.5 + 0.2 * (temps / 3.0).sin();
            let (r, a) = if pres { (1.6, 0.45) } else { (0.8, 0.3) };
            t.disque(c.x(place(fin) + large * 0.5), c.y(fin - 0.4), r * c.demi, sky.lit(hex(0xf4f9fc), 0.15), respire * a, 1.0);
        }
    }
}

// Vie du ciel de la cour : vol d'oiseaux en V le jour, silhouette rare d'un dragon (ou grand aigle) au-dessus des cimes,
/// et chouette mécanique messagère glissant au crépuscule et la nuit.
fn oiseaux(t: &mut Toile, c: &Cadre, sky: &Ciel, w: &Monde, temps: f32) {
    if sky.orage {
        return;
    }
    let room = c.to - c.from;
    if !sky.nuit {
        // vol d'oiseaux en V le jour
        let periode = 700.0;
        let tour = (temps / periode).floor() as i64;
        let f = (temps / periode).fract() * 2.6;
        if f <= 1.0 {
            let n = 3 + (h(tour * 5) * 3.0) as i64;
            let v0 = 4.0 + h(tour * 7) * 5.0;
            let u0 = c.to + 4.0 - f * (room + 10.0);
            for i in 0..n {
                let (u, v) = (u0 + (i as f32) * 1.6, v0 + (i as f32 * 0.9).sin().abs() * 1.4 + (i as f32) * 0.5);
                let bat = ((temps * 0.9 + i as f32 * 1.7).sin() * 0.5 + 0.5) * 0.8;
                let (x, y) = (c.x(u), c.y(v));
                let l = 0.7 * c.cw;
                let col = sky.lit(hex(0x2a3040), 0.25);
                t.trait_((x - l, y - bat * l), (x, y), 1.4, col, 0.9);
                t.trait_((x, y), (x + l, y - bat * l), 1.4, col, 0.9);
            }
        }
    }

    // la chouette mécanique messagère au crépuscule et dans la nuit claire, en lettres (chouette.rs) : laiton sous le
    // ciel, la lentille qui luit d'elle-même ; elle ne porte rien, elle rentre
    if sky.nuit || sky.soir {
        let periode = 460.0;
        let tour = (temps / periode).floor() as i64;
        let f = (temps / periode).fract() * 2.3;
        if f <= 1.0 {
            let vers_droite = tour % 2 == 0;
            let u = if vers_droite { c.from - 3.0 + f * (room + 6.0) } else { c.to + 3.0 - f * (room + 6.0) };
            let v = 6.2 + h(tour * 11 + 3) * 4.5 + (temps * 0.14).sin() * 0.45;
            let image = super::chouette::image(temps as i64);
            let (x, y) = (c.x(u).round() as i32 - 9, c.y(v).round() as i32 - 9);
            lettres(t, x, y, image, vers_droite, &|l| {
                let (k, luit) = super::chouette::couleur(l, false)?;
                Some(if luit { hex(k) } else { sky.lit(hex(k), 0.0) })
            });
        }
    }

    // le dragon : de loin en loin il passe au-dessus des monts (à chaque événement rare, il ne fait que ça)
    let rare_dragon = w.rare.as_ref().is_some_and(|r| r.kind == Rarete::Dragon);
    let periode_dr = 1350.0;
    let tour = (temps / periode_dr).floor() as i64;
    let f_dr = if rare_dragon { ((temps * 0.012).fract()) * 0.95 } else { (temps / periode_dr).fract() * 4.2 };
    if f_dr <= 1.0 && room >= 18.0 {
        let sens = if tour % 2 == 0 { 1.0 } else { -1.0 };
        let u = if sens > 0.0 { c.from - 6.0 + f_dr * (room + 12.0) } else { c.to + 6.0 - f_dr * (room + 12.0) };
        let v = 4.4 + h(tour * 13) * 1.6 + (temps * 0.08).sin() * 0.55;
        dragon(t, c, sky, u, v, sens, temps, f_dr);
    }
}

/// Un point dans un polygone (pair-impair).
fn dans(p: f32, q: f32, pts: &[(f32, f32)]) -> bool {
    let mut dedans = false;
    let mut j = pts.len() - 1;
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[j]);
        if (a.1 > q) != (b.1 > q) && p < (b.0 - a.0) * (q - a.1) / (b.1 - a.1) + a.0 {
            dedans = !dedans;
        }
        j = i;
    }
    dedans
}

/// Ce qu'est le dragon en (p, q), en colonnes de forge depuis le milieu de son corps, tourné vers la droite (q vers
/// le bas) : 0 rien, 1 le corps, 2 l'aile de derrière, 3 l'aile de devant, 4 le ventre, 5 les cornes et la crête.
fn dragon_ici(p: f32, q: f32, haut: f32, onde: f32) -> u8 {
    // une aile de chauve-souris : l'épaule, le poignet, trois doigts, la membrane festonnée entre eux, la hanche
    let aile = |dp: f32, dq: f32, k: f32| -> bool {
        let up = haut * k;
        let e = (0.5 + dp, -0.3 + dq);
        let poignet = (-0.1 + dp, -0.35 - 1.6 * up + dq);
        let d1 = (-1.3 + dp, -0.55 - 2.4 * up + dq);
        let d2 = (-2.5 + dp, -0.25 - 1.5 * up + dq);
        let d3 = (-2.5 + dp, 0.15 - 0.45 * up + dq);
        let hanche = (-1.15 + dp, 0.05 + dq);
        let creux = |a: (f32, f32), b: (f32, f32)| {
            let m = ((a.0 + b.0) * 0.5, (a.1 + b.1) * 0.5);
            (m.0 + (poignet.0 - m.0) * 0.32, m.1 + (poignet.1 - m.1) * 0.32)
        };
        let pts = [e, poignet, d1, creux(d1, d2), d2, creux(d2, d3), d3, creux(d3, hanche), hanche];
        // les os des doigts, plus épais que la membrane
        dans(p, q, &pts)
    };
    if aile(0.0, 0.0, 1.0) {
        return 3;
    }
    // la tête cornue au bout du cou qui ondule
    let cou = [(1.1, -0.15), (1.9, -0.25 + onde * 0.1), (2.4, -0.75), (2.95, -0.85)];
    let mut avant = cou[0];
    for i in 1..=10 {
        let t = i as f32 / 10.0;
        let m = 1.0 - t;
        let b = (
            m * m * m * cou[0].0 + 3.0 * m * m * t * cou[1].0 + 3.0 * m * t * t * cou[2].0 + t * t * t * cou[3].0,
            m * m * m * cou[0].1 + 3.0 * m * m * t * cou[1].1 + 3.0 * m * t * t * cou[2].1 + t * t * t * cou[3].1,
        );
        if seg(p, q, avant, b) < 0.34 - 0.1 * t {
            return 1;
        }
        avant = b;
    }
    let (a, b) = ((p - 3.15) / 0.55, (q + 0.95) / 0.33);
    if a * a + b * b <= 1.0 || dans(p, q, &[(3.4, -1.12), (4.15, -0.88), (3.4, -0.72)]) {
        return 1;
    }
    if dans(p, q, &[(2.95, -1.18), (2.35, -1.75), (3.2, -1.22)]) || dans(p, q, &[(3.2, -1.2), (2.85, -1.8), (3.45, -1.2)]) {
        return 5;
    }
    // le corps, le ventre clair dessous ; la crête du dos
    let (a, b) = (p / 1.65, (q - 0.02) / 0.55);
    if a * a + b * b <= 1.0 {
        return if q > 0.22 && p > -1.1 { 4 } else { 1 };
    }
    for k in 0..4 {
        let x = -1.0 + k as f32 * 0.6;
        if dans(p, q, &[(x - 0.2, -0.45), (x + 0.2, -0.48), (x - 0.05, -0.8)]) {
            return 5;
        }
    }
    // les pattes repliées sous le ventre
    if dans(p, q, &[(0.6, 0.4), (1.0, 0.45), (0.9, 0.85)]) || dans(p, q, &[(-0.9, 0.35), (-0.4, 0.45), (-0.7, 0.9)]) {
        return 1;
    }
    // la queue, longue et sinueuse, jusqu'à la pointe en fer de lance
    let queue = [(-1.4, 0.1), (-2.8, 0.45 + onde * 0.4), (-4.0, -0.35 - onde * 0.4), (-5.2, 0.1 + onde * 0.2)];
    let mut avant = queue[0];
    for i in 1..=14 {
        let t = i as f32 / 14.0;
        let m = 1.0 - t;
        let b = (
            m * m * m * queue[0].0 + 3.0 * m * m * t * queue[1].0 + 3.0 * m * t * t * queue[2].0 + t * t * t * queue[3].0,
            m * m * m * queue[0].1 + 3.0 * m * m * t * queue[1].1 + 3.0 * m * t * t * queue[2].1 + t * t * t * queue[3].1,
        );
        if seg(p, q, avant, b) < 0.34 - 0.26 * t {
            return 1;
        }
        avant = b;
    }
    let bout = queue[3];
    if dans(p, q, &[(bout.0 + 0.1, bout.1), (bout.0 - 0.45, bout.1 - 0.3), (bout.0 - 0.7, bout.1), (bout.0 - 0.45, bout.1 + 0.3)]) {
        return 5;
    }
    if aile(0.45, -0.2, 0.85) {
        return 2;
    }
    0
}

/// La distance de (p, q) au segment ab.
fn seg(p: f32, q: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l = dx * dx + dy * dy;
    let t = if l > 0.0 { (((p - a.0) * dx + (q - a.1) * dy) / l).clamp(0.0, 1.0) } else { 0.0 };
    (a.0 + dx * t - p).hypot(a.1 + dy * t - q)
}

/// Le dragon qui passe, au pixel d'art : de jour un grand dragon rouge sombre au ventre d'or, les ailes battantes,
/// pâli par la distance ; la nuit une silhouette noire où rougeoie la braise du ventre, l'œil d'or, et qui crache
/// parfois une gerbe de feu.
#[allow(clippy::too_many_arguments)]
fn dragon(t: &mut Toile, c: &Cadre, sky: &Ciel, u: f32, v: f32, sens: f32, temps: f32, f: f32) {
    let (x0, y0) = (c.x(u), c.y(v));
    let k = c.cw; // une colonne de forge en pixels de la toile (q aussi, pour garder les proportions)
    let haut = (temps * 0.38).sin();
    let onde = (temps * 0.21).sin();
    // le corps monte quand les ailes s'abaissent
    let y0 = y0 + haut * 0.25 * k;
    let nuit = sky.nuit;
    let braise = 0.75 + 0.25 * (temps * 0.9).sin();
    let (corps, ventre, aile_av, aile_ar, corne) = if nuit {
        (hex(0x161626), mix(hex(0x3a1a12), hex(0xff7a2a), 0.55 * braise), hex(0x221c34), hex(0x141420), hex(0x2a2a3e))
    } else {
        (sky.lit(hex(0x6a2a20), 0.3), sky.lit(hex(0xc8963a), 0.3), sky.lit(hex(0x9a4030), 0.3), sky.lit(hex(0x4e1e18), 0.3), sky.lit(hex(0xd8c8a0), 0.3))
    };
    let (pa, pb, qa, qb) = (-5.8f32, 4.4f32, -3.4f32, 2.4f32);
    let (xa, xb) = ((x0 + pa.min(-pb) * k).floor() as i32, (x0 + pb.max(-pa) * k).ceil() as i32);
    let (ya, yb) = ((y0 + qa * k).floor() as i32, (y0 + qb * k).ceil() as i32);
    for y in ya.max(0)..yb.min(t.h as i32) {
        for x in xa.max(0)..xb.min(t.w as i32) {
            let p = (x as f32 + 0.5 - x0) / k * sens;
            let q = (y as f32 + 0.5 - y0) / k;
            let n = dragon_ici(p, q, haut, onde);
            if n == 0 {
                continue;
            }
            // le dessus de chaque partie prend la lumière
            let dessus = dragon_ici(p, q - 1.0 / k, haut, onde) != n;
            let mut col = match n {
                2 => aile_ar,
                3 => aile_av,
                4 => ventre,
                5 => corne,
                _ => corps,
            };
            if dessus && !nuit {
                col = mix(col, hex(0xffffff), 0.18);
            }
            t.poser(x, y, col, if n == 2 || n == 3 { 0.94 } else { 1.0 });
        }
    }
    // l'œil
    let (ex, ey) = (x0 + 3.3 * k * sens, y0 - 1.0 * k);
    t.poser(ex.round() as i32, ey.round() as i32, if nuit { hex(0xffd040) } else { hex(0xffe08a) }, 1.0);
    // la nuit, au milieu de sa traversée, une gerbe de feu devant le museau
    if nuit && (0.35..0.6).contains(&f) {
        let souffle = ((f - 0.35) / 0.25 * PI).sin();
        for i in 0..9 {
            let d = 0.4 + i as f32 * 0.42;
            let r = (0.25 + i as f32 * 0.09) * k * souffle;
            let px = x0 + (4.2 + d) * k * sens;
            let py = y0 - 0.9 * k + d * 0.22 * k + (temps * 1.7 + i as f32).sin() * 0.12 * k;
            let col = if i < 3 { hex(0xfff0a0) } else if i < 6 { hex(0xffa03a) } else { hex(0xe0482a) };
            t.disque(px, py, r, col, (1.0 - i as f32 / 10.0) * souffle, 0.6);
        }
    }
}

/// Les couleurs d'une grappe de feuilles, de l'ombre à la lumière, selon la saison (or lumineux de Mallorn à l'automne).
fn feuillage(se: Saison) -> Option<[Rgb; 4]> {
    Some(match se {
        Saison::Automne => [hex(0x8e4614), hex(0xc9781c), hex(0xeab434), hex(0xfce276)],
        Saison::Ete => [hex(0x1f4626), hex(0x2c6230), hex(0x3e7e36), hex(0x5e9a44)],
        Saison::Printemps => [hex(0x33702c), hex(0x4f9038), hex(0x6eb04a), hex(0x96cc62)],
        Saison::Hiver => return None,
    })
}

/// Le grand Mallorn / chêne féerique : ses branches en éventail, deux lanternes de fer forgé suspendues aux branches basses,
/// sa couronne en grappes qui se balancent au vent, et ses feuilles d'or ou pétales qui tombent en tournoyant.
fn arbre(t: &mut Toile, c: &Cadre, sky: &Ciel, plan: &Plan, se: Saison, temps: f32) {
    let (tx, r) = (plan.tree.0 as f32 + 0.5, plan.tree.1 as f32);
    let ty = r + 0.5;
    let dir = sky.cote;
    let vent = if sky.orage { 1.4 } else { 0.6 };
    let rythme = if sky.orage { 3.0 } else { 7.0 };
    let balance = |v: f32| (temps / rythme + v / 3.0).sin() * vent * ((ty + r - v) / (2.0 * r)).clamp(0.0, 1.0);
    let bois = sky.lit(hex(0x564232), 0.0);
    let mut attaches_lanternes = [(0.0f32, 0.0f32); 2];
    // les branches : trois paires depuis le tronc
    for (i, b) in [0.45f32, 1.0, 1.8].iter().enumerate() {
        for (idx_c, cote) in [-1.0f32, 1.0].iter().copied().enumerate() {
            let up0 = i as f32 * 1.5 + 0.2;
            let up1 = ((r - 1.0) / b).min(3.0 + r * 0.8);
            let (v0, v1) = (ty + 3.0 - up0, ty + 3.0 - up1);
            let (u0, u1) = (tx + cote * up0 * b, tx + cote * up1 * b + balance(v1));
            // la branche se courbe un peu vers le haut, puis se fourche en rameaux
            let (um, vm) = ((u0 + u1) * 0.5 - cote * 0.3, (v0 + v1) * 0.5 - 0.4);
            if i == 0 {
                attaches_lanternes[idx_c] = (um, vm);
            }
            let ep = (0.55 - i as f32 * 0.1) * c.cw;
            t.trait_((c.x(u0), c.y(v0)), (c.x(um), c.y(vm)), ep, bois, 1.0);
            t.trait_((c.x(um), c.y(vm)), (c.x(u1), c.y(v1)), ep * 0.7, bois, 1.0);
            let longueur = ((u1 - um).powi(2) + (v1 - vm).powi(2)).sqrt();
            for (j, (du, dv)) in [(0.55f32, -0.75f32), (0.9, -0.2), (0.1, -1.0)].iter().enumerate() {
                let l = longueur * (0.45 + h(i as i64 * 13 + j as i64 + if cote > 0.0 { 7 } else { 0 }) * 0.3);
                let (ur, vr) = (u1 + cote * du * l, v1 + dv * l);
                t.trait_((c.x(u1), c.y(v1)), (c.x(ur), c.y(vr)), ep * 0.35, bois, 1.0);
                if se == Saison::Hiver {
                    t.trait_((c.x(u1), c.y(v1) - 1.5), (c.x(ur), c.y(vr) - 1.5), 1.2, sky.lit(hex(0xeef2f8), 0.0), 0.8);
                }
                // et du milieu de la branche, un rameau de plus
                let (us, vs) = (um + cote * du * l * 0.6, vm + dv * l * 0.7);
                t.trait_((c.x(um), c.y(vm)), (c.x(us), c.y(vs)), ep * 0.3, bois, 1.0);
            }
            if se == Saison::Hiver {
                t.trait_((c.x(u0), c.y(v0) - 0.3 * c.demi), (c.x(um), c.y(vm) - 0.3 * c.demi), 1.6, sky.lit(hex(0xeef2f8), 0.0), 0.9);
                t.trait_((c.x(um), c.y(vm) - 0.25 * c.demi), (c.x(u1), c.y(v1) - 0.25 * c.demi), 1.4, sky.lit(hex(0xeef2f8), 0.0), 0.9);
            }
        }
    }
    if let Some(couleurs) = feuillage(se) {
        let couleurs = couleurs.map(|k| sky.lit(k, 0.0));
        // les grappes, réparties dans la couronne
        let n = 18;
        let grappes: Vec<(f32, f32, f32)> = (0..n)
            .map(|i| {
                let a = h(i * 7 + 1) * 2.0 * PI;
                let rr = h(i * 7 + 2).sqrt() * 0.8;
                let v = (ty - 0.5 + a.sin() * rr * (r + 1.0) * 0.8).min(ty + 0.55 * r);
                let u = tx + a.cos() * rr * (r + 1.5) / 0.85;
                (u + balance(v), v, r * (0.34 + h(i * 7 + 3) * 0.16))
            })
            .collect();
        let lumiere = |u: f32, v: f32| ((u - tx) * dir * 0.6 - (v - ty) * 0.8) / r;
        let contour = sky.lit(if se == Saison::Automne { hex(0x4a220a) } else { hex(0x10261a) }, 0.0);
        for (i, &(u, v, g)) in grappes.iter().enumerate() {
            grappe(t, c.x(u), c.y(v + 0.15 * g), g * c.demi + 1.0, contour, i as i64 * 5);
        }
        for (i, &(u, v, g)) in grappes.iter().enumerate() {
            grappe(t, c.x(u), c.y(v + 0.15 * g), g * c.demi, couleurs[0], i as i64 * 5);
        }
        for (i, &(u, v, g)) in grappes.iter().enumerate() {
            grappe(t, c.x(u - dir * 0.08 * g), c.y(v - 0.1 * g), g * 0.85 * c.demi, couleurs[1], i as i64 * 5 + 1);
        }
        for (i, &(u, v, g)) in grappes.iter().enumerate() {
            if lumiere(u, v) > -0.15 {
                grappe(t, c.x(u + dir * 0.25 * g), c.y(v - 0.32 * g), g * 0.55 * c.demi, couleurs[2], i as i64 * 5 + 2);
            }
        }
        // des feuilles en touches, pour le grain
        for (i, &(u, v, g)) in grappes.iter().enumerate() {
            for k in 0..7 {
                let (du, dv) = ((h(i as i64 * 31 + k * 3) - 0.5) * 1.6 * g, (h(i as i64 * 31 + k * 3 + 1) - 0.5) * 1.6 * g);
                if k % 2 == 1 && lumiere(u + du, v + dv) < 0.2 {
                    continue;
                }
                let teinte = if lumiere(u + du, v + dv) > 0.2 { couleurs[3] } else { couleurs[2] };
                let fleur = se == Saison::Printemps && k == 0;
                let (x, y) = (c.x(u + du) as i32, c.y(v + dv) as i32);
                let teinte = if fleur { sky.lit(hex(0xf8c4d8), 0.0) } else { teinte };
                t.poser(x, y, teinte, 1.0);
                if k % 3 == 0 {
                    t.poser(x + 1, y, teinte, 1.0);
                }
            }
        }
        // celles qui tombent, en tournoyant, le vent les prenant à gauche
        if matches!(se, Saison::Automne | Saison::Printemps) {
            for i in 0..5i64 {
                let periode = 150.0 + h(i * 3) * 90.0;
                let age = temps + i as f32 * 53.0;
                let tour = (age / periode).floor() as i64;
                let f = (age / periode).fract();
                let (v0, v1) = (ty - 1.0, SOL + 1.6);
                let v = v0 + f * (v1 - v0);
                let u = tx - r * 0.8 + h(i * 11 + tour * 17) * r * 1.6 - f * 4.0 + (temps / 9.0 + i as f32).sin() * 1.2;
                let teinte = if se == Saison::Printemps { hex(0xf8c4d8) } else { [hex(0xe8a832), hex(0xd4741e), hex(0xf8dc68)][i as usize % 3] };
                let (x, y) = (c.x(u), c.y(v));
                let tourne = (temps * 0.5 + i as f32).sin();
                t.trait_((x - 2.0 * tourne, y - 1.0), (x + 2.0 * tourne, y + 1.0), 2.0, sky.lit(teinte, 0.0), 1.0);
            }
        }
    }

    // deux lanternes féeriques de fer forgé, pendues SOUS la couronne (au bord bas du feuillage, ou aux branches basses
    // quand l'arbre est nu) pour se découper sur le fond : dedans, elles se perdaient comme des pinceaux
    let allume = sky.nuit || sky.soir || sky.orage;
    let fer = sky.lit(hex(0x1e2028), 0.0);
    let feuillu = feuillage(se).is_some();
    for (idx, &(ub, vb)) in attaches_lanternes.iter().enumerate() {
        if ub <= 0.0 {
            continue;
        }
        let cote = if idx == 0 { -1.0 } else { 1.0 };
        let (ua, va) = if feuillu { (tx + cote * r * 0.7, ty + r * 0.62) } else { (ub, vb) };
        let osc = (temps / (rythme * 0.85) + idx as f32 * 2.1).sin() * (0.22 + vent * 0.15);
        let (ul, vl) = (ua + osc, va + 1.3);
        let (lw, lh) = (0.5, 1.5);
        if allume {
            let pulse = 0.88 + 0.12 * (temps * 0.35 + idx as f32 * 1.9).sin();
            t.disque(c.x(ul), c.y(vl + lh * 0.5), 3.0 * c.demi, hex(0xffa434), 0.22 * pulse, 1.0);
            t.disque(c.x(ul), c.y(vl + lh * 0.5), 1.4 * c.demi, hex(0xffd264), 0.38 * pulse, 0.8);
        }
        // la chaînette
        t.trait_((c.x(ua), c.y(va)), (c.x(ul), c.y(vl - 0.3)), 1.0, fer, 0.95);
        // l'anneau, le chapeau pointu qui déborde, la cage et sa vitre, le fond et sa pointe
        t.poser(c.x(ul).round() as i32, c.y(vl - 0.45).round() as i32, fer, 1.0);
        t.poly(&[(c.x(ul - lw - 0.2), c.y(vl + 0.15)), (c.x(ul + lw + 0.2), c.y(vl + 0.15)), (c.x(ul), c.y(vl - 0.4))], fer, 1.0);
        t.poly(&[(c.x(ul - lw), c.y(vl + 0.1)), (c.x(ul + lw), c.y(vl + 0.1)), (c.x(ul + lw), c.y(vl + lh)), (c.x(ul - lw), c.y(vl + lh))], fer, 1.0);
        let verre = if allume { hex(0xffd870) } else { sky.lit(hex(0x8aa0b4), 0.0) };
        let reflet = if allume { hex(0xfff6d0) } else { sky.lit(hex(0xc4d4e2), 0.0) };
        let (gx0, gx1) = (c.x(ul - lw).round() as i32 + 1, c.x(ul + lw).round() as i32 - 1);
        let (gy0, gy1) = (c.y(vl + 0.1).round() as i32 + 1, c.y(vl + lh).round() as i32 - 1);
        for y in gy0..gy1 {
            for x in gx0..gx1 {
                t.poser(x, y, if x == gx0 && y < gy0 + 2 { reflet } else { verre }, 1.0);
            }
        }
        if allume {
            t.poser((gx0 + gx1) / 2, (gy0 + gy1) / 2, hex(0xffffff), 1.0);
        }
        t.poser(c.x(ul).round() as i32, c.y(vl + lh + 0.3).round() as i32, fer, 1.0);
    }
}

/// Une grappe de feuilles : un rond bosselé (son rayon varie avec l'angle), à bord net.
fn grappe(t: &mut Toile, cx: f32, cy: f32, r: f32, c: Rgb, graine: i64) {
    let (xa, xb) = ((cx - r * 1.2).floor().max(0.0) as i32, ((cx + r * 1.2).ceil() as i32).min(t.w as i32));
    let (ya, yb) = ((cy - r * 1.2).floor().max(0.0) as i32, ((cy + r * 1.2).ceil() as i32).min(t.h as i32));
    for y in ya..yb {
        for x in xa..xb {
            let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            let a = dy.atan2(dx) / (2.0 * PI) + 0.5;
            // cinq à sept bosses autour, le raccord en a = 0 fondu
            let bosse = roule(a * 6.0, graine) * (1.0 - a) + roule((a - 1.0) * 6.0, graine) * a;
            if dx.hypot(dy) < r * (0.82 + 0.3 * bosse) {
                t.poser(x, y, c, 1.0);
            }
        }
    }
}

/// Le seau du puits : cerclé de cuivre poli, il pend sous le treuil, descend de temps en temps dans le puits et remonte
/// en gouttant, tandis qu'un reflet étoilé danse au bord de l'eau la nuit.
fn puits(t: &mut Toile, c: &Cadre, sky: &Ciel, plan: &Plan, se: Saison, temps: f32) {
    let Some(p) = plan.well else { return };
    let p = p as f32;
    let cycle = (temps + p * 37.0).rem_euclid(480.0);
    // où est le haut du seau, et s'il bouge
    let (haut, bouge) = match cycle {
        a if a < 400.0 => (22.4, false),
        a if a < 425.0 => (22.4 + (a - 400.0) / 25.0 * 4.0, true),
        a if a < 440.0 => (26.4, false),
        a if a < 465.0 => (26.4 - (a - 440.0) / 25.0 * 4.0, true),
        _ => (22.4, false),
    };
    let lit = |k: u32| sky.lit(hex(k), 0.0);
    let corde = p + 4.5;
    t.trait_((c.x(corde), c.y(20.5)), (c.x(corde), c.y(haut.min(23.9))), 1.4, lit(0xa89e88), 1.0);
    // la manivelle de fer forgé et poignée de bois, à droite du poteau
    let angle = if bouge { temps * 0.8 } else { 0.6 };
    let (mx, my) = (c.x(p + 8.4), c.y(20.0));
    let (bx, by) = (mx + angle.cos() * 0.9 * c.cw, my + angle.sin() * 0.9 * c.demi);
    t.trait_((mx, my), (bx, by), 2.0, lit(0x464858), 1.0);
    t.disque(bx, by, 0.28 * c.demi, lit(0x9a6432), 1.0, 0.0);
    // léger scintillement argenté à l'intérieur de la margelle la nuit
    if sky.nuit && se != Saison::Hiver {
        let scint = 0.45 + 0.45 * (temps * 0.22 + p).sin();
        let u_eau = p + 3.2 + (temps * 0.09).sin() * 1.1;
        t.trait_((c.x(u_eau), c.y(23.75)), (c.x(u_eau + 1.1), c.y(23.75)), 1.0, hex(0x8cd8ff), scint * 0.75);
    }
    // le seau de chêne cerclé de cuivre, caché par la margelle quand il descend
    if haut < 23.9 {
        let bas = (haut + 1.3).min(23.9);
        t.poly(&[(c.x(p + 3.5), c.y(haut)), (c.x(p + 5.5), c.y(haut)), (c.x(p + 5.3), c.y(bas)), (c.x(p + 3.7), c.y(bas))], lit(0x6e4426), 1.0);
        t.trait_((c.x(p + 3.85), c.y(haut)), (c.x(p + 3.95), c.y(bas)), 1.1, lit(0x946238), 0.85);
        for v in [haut + 0.3, haut + 1.0] {
            if v < bas {
                t.trait_((c.x(p + 3.55), c.y(v)), (c.x(p + 5.45), c.y(v)), 1.4, lit(0xb87634), 1.0);
                t.poser(c.x(p + 4.1).round() as i32, c.y(v).round() as i32, lit(0xebba6a), 0.9);
            }
        }
        // il remonte plein : des gouttes
        if (440.0..480.0).contains(&cycle) && se != Saison::Hiver {
            for k in 0..3 {
                let f = ((temps * 0.35 + k as f32 * 0.33).fract()) * 2.0;
                t.disque(c.x(p + 4.0 + k as f32 * 0.5), c.y(bas + f), 0.12 * c.demi, lit(0x9fdcff), 1.0 - f * 0.4, 0.0);
            }
        }
    }
}

/// La roue du moulin et le canal de pierre de l'aqueduc : la jante, les rayons, les palettes, le moyeu ; le bief de
/// pierre moussue et son eau qui court, le filet qui tombe du bec, les débords, les éclaboussures, l'étang et les roseaux.
#[allow(clippy::too_many_arguments)]
fn moulin(t: &mut Toile, c: &Cadre, sky: &Ciel, plan: &Plan, se: Saison, temps: f32, travail: bool) {
    let Some(cx) = plan.wheel else { return };
    let cx = cx as f32 + 0.5;
    let hiver = se == Saison::Hiver;
    let (cy, r) = (cour::CY as f32 + 0.5, cour::R as f32);
    let tour = if hiver { 0.4 } else { temps * if travail { 0.22 } else { 0.09 } };
    let pas = cour::STEP as f32;
    let lit = |k: u32| sky.lit(hex(k), 0.0);
    // la roue, pixel par pixel dans son carré
    let (xa, xb) = (c.x(cx - r - 2.3).floor().max(0.0) as i32, (c.x(cx + r + 2.3).ceil() as i32).min(t.w as i32));
    let (ya, yb) = (c.y(cy - r - 2.3).floor().max(0.0) as i32, (c.y(cy + r + 2.3).ceil() as i32).min(t.h as i32));
    let e = 1.0 / c.demi;
    let _net = |d: f32| palier((d / e + 0.5).clamp(0.0, 1.0));
    let dir_lum = sky.cote;
    for y in ya..yb {
        let dv = (y as f32 + 0.5) / c.demi - cy;
        for x in xa..xb {
            let du = c.from + (x as f32 + 0.5) / c.cw - cx;
            let d = du.hypot(dv);
            if d > r + 2.2 {
                continue;
            }
            let th = dv.atan2(du);
            let ang = th + tour;
            let pres = |n: f32| {
                let s = 2.0 * PI / n;
                let a = ang.rem_euclid(s);
                a.min(s - a) * d
            };
            let i = y as usize * t.w + x as usize;
            let mut col = unpack(t.px[i]);
            let eclaire = 0.85 + 0.25 * (-(th + PI * 0.75)).cos() * 0.5;
            // une bande de bois ou de fer : son bord tourné vers le haut éclairé, l'autre dans l'ombre, d'un pixel
            let bande = |d0: f32, d1: f32, base: u32, clair: u32, sombre: u32| {
                let (bas, haut) = if dv < 0.0 { (d0, d1) } else { (d1, d0) };
                if (d - haut).abs() < e {
                    fois(lit(clair), eclaire)
                } else if (d - bas).abs() < e {
                    fois(lit(sombre), eclaire)
                } else {
                    fois(lit(base), eclaire)
                }
            };
            let cerne = lit(0x1e1008);
            let (r_int, r_god, r_jan, r_fer, r_pal) = (r - 2.7, r - 1.9, r - 0.1, r + 0.6, r + 0.95);
            if d < 1.45 {
                // le moyeu de fer et son collier de bois, six boulons, l'axe
                col = if d > 1.45 - e {
                    cerne
                } else if d > 1.0 {
                    bande(1.0, 1.45, 0x6a4228, 0x9a6a38, 0x3a2214)
                } else if d < 0.34 {
                    lit(0x9aa0b0)
                } else if (d - 0.68).abs() < 0.2 && pres(6.0) < 0.2 {
                    lit(0xc8ccd8)
                } else {
                    fois(lit(0x4a4e5c), eclaire)
                };
            } else if d < r_int {
                // huit rais, un liseré clair du côté du jour, cernés
                let sr = 2.0 * PI / 8.0;
                let a8 = ang.rem_euclid(sr);
                let q = if a8 < sr * 0.5 { a8 } else { a8 - sr } * d;
                if q.abs() < 0.42 {
                    col = if q * dir_lum.signum() > 0.18 { fois(lit(0x9a6a38), eclaire) } else if q * dir_lum.signum() < -0.18 { fois(lit(0x4a2c16), eclaire) } else { fois(lit(0x7a4a24), eclaire) };
                } else if q.abs() < 0.42 + e {
                    col = mix(col, cerne, 0.8);
                }
            } else if d < r_god {
                // l'anneau intérieur, planches jointes
                col = if d < r_int + e { cerne } else { bande(r_int, r_god, 0x7a4a24, 0xa8743e, 0x4a2c16) };
                if pres(12.0) < 0.12 {
                    col = cerne;
                }
            } else if d < r_jan {
                // les godets, entre les deux anneaux
                let penche = ang - (d - r_god) * 0.22;
                let ag = penche.rem_euclid(pas) * d;
                let creux = cour::fill(cour::down(th as f64)) as f32;
                col = fois(lit(0x3a2214), eclaire);
                if !hiver && creux > 0.05 && ag > 0.4 && ag < 0.4 + creux * pas * d * 0.75 && d > r_jan - 0.4 - creux * 1.4 {
                    col = if ag > 0.4 + creux * pas * d * 0.75 - e { lit(0xdff6ff) } else { lit(0x4fa8e6) };
                }
                if ag < 0.4 {
                    col = if ag < 0.4 - e { fois(lit(0x8a5a2e), eclaire) } else { fois(lit(0xb07a40), eclaire) };
                }
            } else if d < r_fer {
                // la jante extérieure, ses joints décalés d'un demi-godet ; la neige sur le haut, l'hiver
                col = bande(r_jan, r_fer, 0x8a5a2e, 0xb8844a, 0x4a2c16);
                if (ang + pas * 0.5).rem_euclid(pas) * d < 0.12 {
                    col = cerne;
                }
            } else if d < r_pal {
                // le cercle de fer, ses rivets
                col = if pres(16.0) < 0.2 { lit(0xc8ccd8) } else { fois(lit(0x5a5e6a), eclaire) };
                if hiver && dv < -r * 0.55 {
                    col = lit(0xf2f6fa);
                }
            } else if pres(10.0) < 0.36 && d < r + 1.7 {
                // le bout des cloisons, qui dépasse
                col = if d > r + 1.7 - e { cerne } else { fois(lit(0x6a4228), eclaire) };
            } else if pres(10.0) < 0.36 + e && d < r + 1.7 {
                col = mix(col, cerne, 0.7);
            }
            t.px[i] = pack(col);
        }
    }
    // le bas de la roue dans l'étang
    let surface = 27.15;
    for y in (c.y(surface) as i32).max(ya)..yb {
        for x in xa..xb {
            let du = c.from + (x as f32 + 0.5) / c.cw - cx;
            let dv = (y as f32 + 0.5) / c.demi - cy;
            if du.hypot(dv) < r + 2.2 && du > -8.3 {
                t.poser(x, y, lit(if hiver { 0xd8ecf6 } else { 0x4f9ccc }), if hiver { 0.9 } else { 0.62 });
            }
        }
    }
    // le canal de pierre taillée de l'aqueduc, son parapet moussu, son bec sculpté et l'eau qui court
    let (b0, b1) = (cx - 1.7, c.to + 1.0);
    t.poly(&[(c.x(b0), c.y(10.95)), (c.x(b1), c.y(10.95)), (c.x(b1), c.y(12.0)), (c.x(b0), c.y(12.0))], lit(0x746e82), 1.0);
    t.trait_((c.x(b0), c.y(11.0)), (c.x(b1), c.y(11.0)), 1.2, lit(if hiver { 0xeef2f8 } else { 0xa49eb4 }), 1.0);
    t.trait_((c.x(b0), c.y(11.95)), (c.x(b1), c.y(11.95)), 1.3, lit(0x464252), 1.0);
    // joints de blocs et mousse sur le parapet
    let mut ub = b0 + 1.4;
    let mut kb = 0i64;
    while ub < b1 {
        t.trait_((c.x(ub), c.y(11.05)), (c.x(ub), c.y(11.9)), 1.0, lit(0x4e485c), 0.85);
        if !hiver && h(kb * 19 + 3) > 0.55 {
            t.trait_((c.x(ub - 0.5), c.y(11.82)), (c.x(ub + 0.35), c.y(11.82)), 1.2, lit(0x3a6c32), 0.9);
        }
        ub += 2.2;
        kb += 1;
    }
    // bec gargouille de pierre à l'extrémité de l'aqueduc
    t.poly(&[(c.x(b0 - 0.65), c.y(10.85)), (c.x(b0 + 0.2), c.y(10.85)), (c.x(b0 + 0.2), c.y(11.75)), (c.x(b0 - 0.45), c.y(11.6))], lit(0x888298), 1.0);
    let (ya, yb) = (c.y(10.1) as i32, c.y(11.0) as i32);
    for y in ya..yb {
        for x in c.x(b0 - 0.4) as i32..(c.x(b1) as i32).min(t.w as i32) {
            let u = c.from + (x as f32 + 0.5) / c.cw;
            let teinte = if hiver {
                lit(0xbfe0f0)
            } else if h(((u + temps * 0.6) * 1.5) as i64 * 5 + (y - ya) as i64 / 3) > 0.78 {
                lit(0xdff6ff)
            } else {
                lit(0x6cc4f4)
            };
            t.poser(x, y, teinte, 1.0);
        }
    }
    if hiver {
        // des glaçons sous l'aqueduc
        for k in 0..((b1 - b0) as i64) {
            if k % 2 == 0 {
                let u = b0 + 0.4 + k as f32;
                t.poly(&[(c.x(u - 0.2), c.y(12.0)), (c.x(u + 0.2), c.y(12.0)), (c.x(u), c.y(12.8 + h(k) * 0.6))], lit(0xcfe8f6), 1.0);
            }
        }
    } else {
        // le filet qui tombe du bec dans le godet qui passe dessous
        chute(t, c, sky, cx - 2.2, 10.6, 13.4, 1.1, temps, 1.0);
        // ce que lâchent les godets qui débordent, tout droit vers l'étang
        for k in 0..10 {
            let a = k as f32 * pas - tour;
            let th = a.sin().atan2(a.cos());
            let q = cour::down(th as f64) as f32;
            let f = smooth(1.3, 1.6, q) * (1.0 - smooth(2.1, 2.4, q));
            if f > 0.05 {
                let (u, v) = (cx + (r + 1.6) * th.cos(), cy + (r + 1.6) * th.sin());
                chute(t, c, sky, u - 0.4, v, surface, 0.8, temps + k as f32 * 3.0, 0.95 * f);
                for j in 0..3 {
                    let g = ((temps * 0.3 + j as f32 * 0.37 + k as f32 * 0.11).fract()) as f32;
                    let saut = 4.0 * g * (1.0 - g) * 1.3;
                    let du = (j as f32 - 1.0) * 0.6 * g * 2.0;
                    t.disque(c.x(u + du), c.y(surface - saut), 0.16 * c.demi, lit(0xeef8ff), f * (1.0 - g), 0.2);
                }
            }
        }
        // l'étang qui ondule, le ruisseau qui file vers nous
        for k in 0..14i64 {
            let ruisseau = k >= 8;
            let (u, v) = if ruisseau {
                let f = ((temps * 0.012 + h(k * 5)).fract()) as f32;
                (cx - 9.0 - f * 6.0 + (h(k * 3) - 0.5) * 2.0, 29.2 + f * 3.0)
            } else {
                (cx - 7.5 + h(k * 7) * (c.to - cx + 7.0), 27.5 + h(k * 11) * 1.3)
            };
            let l = 0.6 + h(k * 13) * 0.8;
            let eclat = 0.5 + 0.5 * (temps * 0.2 + k as f32 * 1.7).sin();
            if eclat > 0.3 {
                t.trait_((c.x(u - l), c.y(v)), (c.x(u + l), c.y(v)), 1.0, lit(0xdff6ff), 1.0);
            }
        }
    }
    // les roseaux, qui plient au vent ; leurs têtes brunes
    for (rx, haut) in [(-11.0f32, 4.0f32), (-13.0, 3.0), (-7.0, 2.2), (-12.2, 2.6), (-6.2, 1.6)] {
        let base = if rx > -8.0 { SOL + 0.9 } else { SOL };
        let plie = if sky.orage { (temps / 2.0 + rx).sin() } else { (temps / 6.0 + rx).sin() * 0.5 };
        let (u0, v0) = (cx + rx, base);
        let (u1, v1) = (u0 + plie * 0.6, base - haut);
        let tige = if hiver { lit(0xb8a57c) } else { lit(0x3e6a2a) };
        t.trait_((c.x(u0), c.y(v0)), (c.x((u0 + u1) * 0.5 + plie * 0.1), c.y((v0 + v1) * 0.5)), 2.0, tige, 1.0);
        t.trait_((c.x((u0 + u1) * 0.5 + plie * 0.1), c.y((v0 + v1) * 0.5)), (c.x(u1), c.y(v1)), 1.6, tige, 1.0);
        if haut >= 2.5 {
            t.trait_((c.x(u1), c.y(v1 + 0.2)), (c.x(u1 + plie * 0.15), c.y(v1 - 0.8)), 0.4 * c.cw, if hiver { lit(0xeef2f8) } else { lit(0x6a4022) }, 1.0);
        }
    }
}

/// Un filet d'eau qui tombe de `v0` à `v1` en `u`, `large` de large : des traînées d'écume qui descendent.
#[allow(clippy::too_many_arguments)]
fn chute(t: &mut Toile, c: &Cadre, sky: &Ciel, u: f32, v0: f32, v1: f32, large: f32, temps: f32, a: f32) {
    if v1 <= v0 {
        return;
    }
    let (xa, xb) = (c.x(u) as i32, c.x(u + large).ceil() as i32);
    for y in c.y(v0) as i32..c.y(v1) as i32 {
        let v = (y as f32 + 0.5) / c.demi;
        for x in xa..xb {
            let k = h((x - xa) as i64 * 3 + ((v * 1.5 - temps * 0.9) as i64) * 7);
            if k < 0.15 && x != xa {
                continue;
            }
            let teinte = if k > 0.6 { hex(0xf8fdff) } else if k > 0.3 { hex(0x8fd0f4) } else { hex(0x4f9ccc) };
            t.poser(x, y, sky.lit(teinte, 0.0), a);
        }
    }
}

/// Runes nocturnes du menhir et bonhomme de neige à chapeau de sorcier l'hiver.
fn meteo_et_menhir(t: &mut Toile, c: &Cadre, sky: &Ciel, se: Saison, gala: bool, temps: f32) {
    if gala {
        return;
    }
    let room = c.to - c.from;
    let lit = |k: u32| sky.lit(hex(k), 0.0);

    // 1. Lueur runique pâle sur le menhir au crépuscule ou la nuit
    if room >= 20.0 && (sky.nuit || sky.soir) {
        let um = c.from + 15.2;
        let respire = 0.55 + 0.45 * (temps * 0.09).sin();
        t.disque(c.x(um), c.y(SOL - 2.0), 2.3 * c.demi, hex(0x48c8f8), 0.14 * respire, 1.0);
        for k in 0..4 {
            let rv = SOL - 3.4 + k as f32 * 0.95;
            let onde = 0.65 + 0.35 * (temps * 0.15 + k as f32 * 0.9).sin();
            let c_rune = mix(hex(0x64dcff), hex(0xdafaff), onde);
            t.trait_((c.x(um - 0.05), c.y(rv)), (c.x(um - 0.05), c.y(rv + 0.65)), 1.15, c_rune, 0.92 * respire);
            let br = if k % 2 == 0 { 0.35 } else { -0.35 };
            t.trait_((c.x(um - 0.05), c.y(rv + 0.2)), (c.x(um - 0.05 + br), c.y(rv + 0.5)), 1.0, c_rune, 0.88 * respire);
        }
    }

    // 3. Bonhomme de neige coiffé d'un chapeau pointu de sorcier étoilé, l'hiver
    if se == Saison::Hiver && room >= 21.0 {
        let us = c.from + 18.6;
        let vs = SOL + 1.55;
        let neige_o = lit(0xc8d6ea);
        let neige_c = lit(0xf6f9fe);
        // ombre au sol
        t.poly(&[(c.x(us - 1.5), c.y(vs + 0.3)), (c.x(us + 1.6), c.y(vs + 0.3)), (c.x(us + 1.3), c.y(vs + 0.7)), (c.x(us - 1.2), c.y(vs + 0.7))], lit(0x98a8c4), 0.55);
        // trois boules de neige modelées (ombre + lumière)
        for &(dv, r_b) in &[(-0.45f32, 1.08f32), (-2.05, 0.82), (-3.35, 0.62)] {
            t.disque(c.x(us), c.y(vs + dv), r_b * c.demi, neige_o, 1.0, 0.0);
            t.disque(c.x(us - 0.14), c.y(vs + dv - 0.12), r_b * 0.82 * c.demi, neige_c, 1.0, 0.0);
        }
        // bras en rameaux noueux + petit bâton magique
        t.trait_((c.x(us - 0.75), c.y(vs - 2.1)), (c.x(us - 1.65), c.y(vs - 2.75)), 1.2, lit(0x5a3a20), 1.0);
        t.trait_((c.x(us + 0.75), c.y(vs - 2.1)), (c.x(us + 1.65), c.y(vs - 2.85)), 1.2, lit(0x5a3a20), 1.0);
        t.poser(c.x(us + 1.72).round() as i32, c.y(vs - 2.95).round() as i32, hex(0x8ee8ff), 0.95);
        // écharpe cramoisie et or (façon Poudlard)
        t.trait_((c.x(us - 0.68), c.y(vs - 2.72)), (c.x(us + 0.68), c.y(vs - 2.72)), 1.7, lit(0xa82228), 1.0);
        t.poser(c.x(us - 0.2).round() as i32, c.y(vs - 2.72).round() as i32, lit(0xebba44), 1.0);
        t.trait_((c.x(us + 0.35), c.y(vs - 2.7)), (c.x(us + 0.52), c.y(vs - 1.75)), 1.5, lit(0xa82228), 1.0);
        // yeux de charbon et nez carotte
        t.poser(c.x(us - 0.22).round() as i32, c.y(vs - 3.45).round() as i32, lit(0x1e1e28), 1.0);
        t.poser(c.x(us + 0.18).round() as i32, c.y(vs - 3.45).round() as i32, lit(0x1e1e28), 1.0);
        t.trait_((c.x(us + 0.02), c.y(vs - 3.25)), (c.x(us + 0.55), c.y(vs - 3.2)), 1.2, lit(0xe87420), 1.0);
        // chapeau pointu bleu nuit étoilé à ruban d'or
        t.trait_((c.x(us - 1.05), c.y(vs - 3.85)), (c.x(us + 1.05), c.y(vs - 3.85)), 1.6, lit(0x222c64), 1.0);
        t.poly(
            &[(c.x(us - 0.68), c.y(vs - 3.85)), (c.x(us + 0.68), c.y(vs - 3.85)), (c.x(us + 0.22), c.y(vs - 5.55)), (c.x(us - 0.05), c.y(vs - 5.35))],
            lit(0x29387a),
            1.0,
        );
        t.trait_((c.x(us - 0.62), c.y(vs - 3.98)), (c.x(us + 0.62), c.y(vs - 3.98)), 1.1, lit(0xe8be48), 1.0);
        t.poser(c.x(us + 0.05).round() as i32, c.y(vs - 4.65).round() as i32, hex(0xffe478), 0.95);
    }
}

/// Grande table de banquet médiéval-fantastique dans la cour (`w.banquet`) : chêne massif, chemin de table cramoisi et or,
/// candélabres dorés à trois branches, coupes d'or, plats de rôtis et fumets chauds.
fn banquet(t: &mut Toile, c: &Cadre, sky: &Ciel, temps: f32) {
    let room = c.to - c.from;
    if room < 14.0 {
        return;
    }
    let u0 = c.from + 5.8;
    let u1 = (c.from + 20.6).min(c.to - 1.5);
    if u1 <= u0 + 5.0 {
        return;
    }
    let vt = SOL + 0.55;
    let lit = |k: u32| sky.lit(hex(k), 0.0);

    // halo chaleureux des candélabres sur la table et l'herbe le soir / la nuit
    let c1 = u0 + (u1 - u0) * 0.26;
    let c2 = u0 + (u1 - u0) * 0.74;
    if sky.nuit || sky.soir {
        for &uc in &[c1, c2] {
            let p = 0.88 + 0.12 * (temps * 0.4 + uc).sin();
            t.disque(c.x(uc), c.y(vt - 1.8), 3.4 * c.demi, hex(0xff9e32), 0.22 * p, 1.0);
        }
    }

    // ombre sous la table et tréteaux de chêne sculpté
    t.poly(&[(c.x(u0 - 0.4), c.y(vt + 1.95)), (c.x(u1 + 0.4), c.y(vt + 1.95)), (c.x(u1 + 0.2), c.y(vt + 2.45)), (c.x(u0 - 0.2), c.y(vt + 2.45))], lit(0x1e2418), 0.55);
    for &ul in &[u0 + 1.2, (u0 + u1) * 0.5, u1 - 1.2] {
        t.poly(&[(c.x(ul - 0.45), c.y(vt + 0.4)), (c.x(ul + 0.45), c.y(vt + 0.4)), (c.x(ul + 0.55), c.y(vt + 2.2)), (c.x(ul - 0.55), c.y(vt + 2.2))], lit(0x422614), 1.0);
    }
    t.trait_((c.x(u0 + 1.2), c.y(vt + 1.35)), (c.x(u1 - 1.2), c.y(vt + 1.35)), 1.4, lit(0x361e0e), 1.0);

    // plateau de chêne épais
    t.poly(&[(c.x(u0), c.y(vt - 0.45)), (c.x(u1), c.y(vt - 0.45)), (c.x(u1 + 0.25), c.y(vt + 0.55)), (c.x(u0 - 0.25), c.y(vt + 0.55))], lit(0x7c4c28), 1.0);
    t.trait_((c.x(u0), c.y(vt - 0.45)), (c.x(u1), c.y(vt - 0.45)), 1.2, lit(0xa46c3c), 0.9);
    t.trait_((c.x(u0 - 0.25), c.y(vt + 0.55)), (c.x(u1 + 0.25), c.y(vt + 0.55)), 1.3, lit(0x4a2a14), 1.0);

    // chemin de table cramoisi brodé d'or et fanions pendants
    t.trait_((c.x(u0 + 0.4), c.y(vt + 0.05)), (c.x(u1 - 0.4), c.y(vt + 0.05)), 1.8, lit(0x9e222a), 1.0);
    for &uf in &[u0 + 2.8, u1 - 2.8] {
        t.poly(&[(c.x(uf - 0.75), c.y(vt + 0.15)), (c.x(uf + 0.75), c.y(vt + 0.15)), (c.x(uf + 0.65), c.y(vt + 1.45)), (c.x(uf), c.y(vt + 1.85)), (c.x(uf - 0.65), c.y(vt + 1.45))], lit(0x9e222a), 1.0);
        t.trait_((c.x(uf - 0.65), c.y(vt + 1.42)), (c.x(uf), c.y(vt + 1.82)), 1.0, lit(0xe6ba4c), 1.0);
        t.trait_((c.x(uf), c.y(vt + 1.82)), (c.x(uf + 0.65), c.y(vt + 1.42)), 1.0, lit(0xe6ba4c), 1.0);
    }

    // plats d'argent garnis de rôtis dorés, miches, fruits et coupes d'or
    let um = (u0 + u1) * 0.5;
    for &(up, r_roti) in &[(um - 2.1, true), (um + 2.1, false)] {
        if up > u0 + 1.2 && up < u1 - 1.2 {
            t.trait_((c.x(up - 1.15), c.y(vt - 0.35)), (c.x(up + 1.15), c.y(vt - 0.35)), 1.4, lit(0xc4cad8), 1.0);
            if r_roti {
                t.poly(&[(c.x(up - 0.85), c.y(vt - 0.4)), (c.x(up - 0.35), c.y(vt - 1.25)), (c.x(up + 0.55), c.y(vt - 1.2)), (c.x(up + 0.9), c.y(vt - 0.4))], lit(0xc67428), 1.0);
                t.poser(c.x(up - 0.15).round() as i32, c.y(vt - 1.05).round() as i32, lit(0xeda44c), 1.0);
                // fumet chaud qui s'élève
                let fv = (temps * 0.05).fract();
                let su = up + (temps * 0.12).sin() * 0.3;
                t.trait_((c.x(su), c.y(vt - 1.4 - fv * 1.4)), (c.x(su + 0.2), c.y(vt - 1.9 - fv * 1.4)), 1.0, hex(0xeef2fa), (1.0 - fv) * 0.55);
            } else {
                // corbeille de fruits et tourte dorée
                t.disque(c.x(up - 0.3), c.y(vt - 0.78), 0.42 * c.demi, lit(0xd89638), 1.0, 0.0);
                t.disque(c.x(up + 0.42), c.y(vt - 0.72), 0.32 * c.demi, lit(0xc42830), 1.0, 0.0);
                t.disque(c.x(up + 0.1), c.y(vt - 0.68), 0.28 * c.demi, lit(0x6e3888), 1.0, 0.0);
            }
        }
    }
    // coupes d'or ciselé
    for &ug in &[u0 + 1.6, um - 0.3, u1 - 1.6] {
        if ug > u0 + 0.8 && ug < u1 - 0.8 {
            t.trait_((c.x(ug), c.y(vt - 0.35)), (c.x(ug), c.y(vt - 0.95)), 1.2, lit(0xe4b442), 1.0);
            t.trait_((c.x(ug - 0.28), c.y(vt - 0.98)), (c.x(ug + 0.28), c.y(vt - 0.98)), 1.4, lit(0xf6d468), 1.0);
        }
    }

    // deux candélabres d'or à trois branches et flammes vives
    for (idx, &uc) in [c1, c2].iter().enumerate() {
        let or_c = lit(0xdca83a);
        t.trait_((c.x(uc - 0.35), c.y(vt - 0.35)), (c.x(uc + 0.35), c.y(vt - 0.35)), 1.2, or_c, 1.0);
        t.trait_((c.x(uc), c.y(vt - 0.35)), (c.x(uc), c.y(vt - 1.85)), 1.3, or_c, 1.0);
        t.trait_((c.x(uc - 0.65), c.y(vt - 1.35)), (c.x(uc + 0.65), c.y(vt - 1.35)), 1.1, or_c, 1.0);
        for (b, du) in [(-0.65f32, 0.2f32), (0.0, 0.0), (0.65, 0.2)].iter().copied() {
            let vb = vt - 1.85 + du;
            t.trait_((c.x(uc + b), c.y(vb)), (c.x(uc + b), c.y(vb - 0.45)), 1.2, hex(0xf6f0dc), 1.0);
            let vacille = ((temps * 0.45 + idx as f32 * 1.7 + b * 3.0).sin()) * 0.12;
            t.poser(c.x(uc + b + vacille).round() as i32, c.y(vb - 0.68).round() as i32, hex(0xff9e2c), 1.0);
            t.poser(c.x(uc + b).round() as i32, c.y(vb - 0.56).round() as i32, hex(0xfff4a8), 1.0);
        }
    }
}

/// L'avant-toit d'ardoises moussues de la porte : gargouille gothique sculptée au bout (crachant l'eau dans l'orage),
/// enseigne d'enclume en fer forgé qui oscille sous la poutre et lanterne chaude de porche.
fn avant_toit(t: &mut Toile, c: &Cadre, sky: &Ciel, se: Saison, temps: f32) {
    let fin = c.from + 13.0;
    let hiver = se == Saison::Hiver;
    let lit = |k: u32| sky.lit(hex(k), 0.0);
    let (xa, xb) = (0, (c.x(fin + 0.6).ceil() as i32).min(t.w as i32));
    let yb = (c.y(8.5).ceil() as i32).min(t.h as i32);
    for y in 0..yb {
        let v = (y as f32 + 0.5) / c.demi;
        for x in xa..xb {
            let u = c.from + (x as f32 + 0.5) / c.cw;
            let haut = (u - c.from) / 3.0;
            let n = v - haut;
            if !(0.0..3.0).contains(&n) || u > fin + 0.3 - n * 0.1 {
                continue;
            }
            let col = if n < 2.1 {
                // rangs d'ardoises en écailles légèrement arrondies, ponctuées de mousse hors hiver
                let rang = (n / 0.7).floor();
                let long = (u - c.from + rang * 0.7) / 1.4;
                let (fu, fn_) = (long.fract(), (n / 0.7).fract());
                let ardoise = long.floor() as i64 * 7 + rang as i64 * 3;
                let ecaille = fn_ + (fu - 0.5).powi(2) * 0.35;
                if fu < 0.09 || ecaille > 0.86 {
                    lit(0x282432)
                } else if hiver && rang == 0.0 {
                    lit(if fn_ < 0.4 { 0xf6f9fc } else { 0xdfe6f0 })
                } else if !hiver && fn_ > 0.64 && h(ardoise * 13 + 5) > 0.66 {
                    lit(0x3d6e34)
                } else {
                    let base = [0x666074, 0x565064, 0x746e84][(h(ardoise) * 3.0) as usize % 3];
                    if fn_ < 0.22 {
                        lit(0x9892aa)
                    } else {
                        lit(base)
                    }
                }
            } else if u > fin - 0.5 {
                lit(0x6a4228)
            } else if n < 2.3 {
                lit(0x5a3a1e)
            } else {
                lit(0x3a2418)
            };
            t.poser(x, y, col, 1.0);
        }
    }
    // la jambe de force, du poteau à la poutre, et ses ferrures
    let (p0, p1) = ((c.x(c.from + 0.9), c.y(8.2)), (c.x(c.from + 6.0), c.y(4.9)));
    t.trait_(p0, p1, 0.55 * c.cw, lit(0x3a2418), 1.0);
    t.trait_((p0.0, p0.1 - 1.0), (p1.0, p1.1 - 1.0), 1.0, lit(0x6a4228), 1.0);

    // gargouille gothique en pierre sculptée à la pointe de l'avant-toit
    let vg = (fin - c.from) / 3.0 + 1.35;
    t.poly(
        &[
            (c.x(fin - 0.35), c.y(vg - 0.55)),
            (c.x(fin + 1.15), c.y(vg - 0.35)),
            (c.x(fin + 1.65), c.y(vg + 0.05)),
            (c.x(fin + 0.85), c.y(vg + 0.42)),
            (c.x(fin - 0.35), c.y(vg + 0.55)),
        ],
        lit(0x706a7e),
        1.0,
    );
    // corne, aile repliée et relief clair sur le museau de la gargouille
    t.poly(&[(c.x(fin + 0.15), c.y(vg - 0.45)), (c.x(fin + 0.65), c.y(vg - 1.05)), (c.x(fin + 0.85), c.y(vg - 0.38))], lit(0x8e889e), 1.0);
    t.trait_((c.x(fin - 0.1), c.y(vg - 0.35)), (c.x(fin + 1.35), c.y(vg - 0.18)), 1.1, lit(0x9e98b0), 0.9);
    t.poser(c.x(fin + 1.1).round() as i32, c.y(vg - 0.18).round() as i32, lit(0x262230), 1.0);

    // potence en fer forgé et enseigne d'enclume oscillant sous l'avant-toit
    let ue = c.from + 8.3;
    let ve = (ue - c.from) / 3.0 + 2.85;
    let fer = lit(0x242632);
    let osc = (temps / (if sky.orage { 2.8 } else { 8.5 })).sin() * (if sky.orage { 0.42 } else { 0.14 });
    t.trait_((c.x(ue - 0.9), c.y(ve)), (c.x(ue + 0.9), c.y(ve)), 1.2, fer, 1.0);
    let ue_bas = ue + osc;
    let ve_bas = ve + 1.35;
    t.trait_((c.x(ue - 0.55), c.y(ve)), (c.x(ue_bas - 0.55), c.y(ve_bas)), 1.0, fer, 0.95);
    t.trait_((c.x(ue + 0.55), c.y(ve)), (c.x(ue_bas + 0.55), c.y(ve_bas)), 1.0, fer, 0.95);
    // silhouette d'enclume dorée/bronzée sur écusson sombre
    t.poly(
        &[
            (c.x(ue_bas - 0.95), c.y(ve_bas)),
            (c.x(ue_bas + 0.95), c.y(ve_bas)),
            (c.x(ue_bas + 0.85), c.y(ve_bas + 1.35)),
            (c.x(ue_bas), c.y(ve_bas + 1.75)),
            (c.x(ue_bas - 0.85), c.y(ve_bas + 1.35)),
        ],
        lit(0x2c2230),
        1.0,
    );
    t.trait_((c.x(ue_bas - 0.65), c.y(ve_bas + 0.55)), (c.x(ue_bas + 0.72), c.y(ve_bas + 0.55)), 1.4, lit(0xd4a442), 1.0);
    t.trait_((c.x(ue_bas - 0.42), c.y(ve_bas + 1.1)), (c.x(ue_bas + 0.42), c.y(ve_bas + 1.1)), 1.3, lit(0xb8862e), 1.0);
    t.poser(c.x(ue_bas).round() as i32, c.y(ve_bas + 0.82).round() as i32, lit(0xd4a442), 1.0);

    // lanterne chaude de porche près du poteau
    let (ul, vl) = (c.from + 2.35, 5.15);
    let allume = sky.nuit || sky.soir || sky.orage;
    if allume {
        let p = 0.9 + 0.1 * (temps * 0.35).sin();
        t.disque(c.x(ul), c.y(vl + 0.55), 2.8 * c.demi, hex(0xff9e30), 0.25 * p, 1.0);
    }
    t.trait_((c.x(ul), c.y(vl - 0.7)), (c.x(ul), c.y(vl)), 1.1, fer, 1.0);
    t.poly(&[(c.x(ul - 0.48), c.y(vl + 0.2)), (c.x(ul + 0.48), c.y(vl + 0.2)), (c.x(ul), c.y(vl - 0.2))], fer, 1.0);
    t.poly(
        &[(c.x(ul - 0.34), c.y(vl + 0.2)), (c.x(ul + 0.34), c.y(vl + 0.2)), (c.x(ul + 0.26), c.y(vl + 1.05)), (c.x(ul - 0.26), c.y(vl + 1.05))],
        if allume { hex(0xffe88a) } else { lit(0x9ab0c4) },
        if allume { 0.98 } else { 0.75 },
    );
    t.trait_((c.x(ul - 0.36), c.y(vl + 1.08)), (c.x(ul + 0.36), c.y(vl + 1.08)), 1.3, fer, 1.0);

    if hiver {
        for k in 0..6 {
            let u = c.from + 1.2 + k as f32 * 2.0 + h(k) * 0.6;
            let v = (u - c.from) / 3.0 + 3.0;
            let l = 0.5 + h(k * 3 + 1) * 0.9;
            t.poly(&[(c.x(u - 0.25), c.y(v)), (c.x(u + 0.25), c.y(v)), (c.x(u), c.y(v + l))], lit(0xcfe8f6), 1.0);
        }
    }
}

// Les petites bêtes de la cour, dessinées à la main en lettres (un pixel d'art par lettre), vues de côté, tournées vers
// la droite : le lapin assis et en plein bond, l'écureuil qui grimpe, le poisson qui saute, le papillon ailes ouvertes et
// fermées.
const LAPIN_ASSIS: [&str; 10] = ["...kk....", "..kwWk...", "..kwWk...", "...kwWk..", "..kkWWWk.", ".kWWWWmWk", "kcWWWWWWn", "kcWWWWWk.", ".kwWkkWk.", "..kk..kk."];
const LAPIN_BOND: [&str; 10] = [".....kk...", "....kwWk..", ".....kwWk.", "......kWk.", ".kkkkWWWWk", "kcWWWWWmWn", "kcWWWWWWk.", ".kwkkkwk..", "kwk....kk.", "kk........"];
const ECUREUIL: [&str; 12] = [".....kk..", "....kRRk.", "....kRmRk", "...kkRRRn", ".kk.kqRk.", "kRRkkqRRk", "kRrRkRRk.", "kRRrRRRk.", "kqRRRrRk.", ".kqRRRk..", "..kkkRkk.", ".....kk.."];
const ECUREUIL_2: [&str; 12] = [".....kk..", "....kRRk.", "....kRmRk", "...kkRRRn", "kk..kqRkk", "kRk.kqRRk", "kRRkRRRk.", "kRrRRRk..", "kqRRrRRk.", ".kqRRRRk.", "..kkkRRk.", "....kk..."];
const POISSON_HAUT: [&str; 5] = ["..kkk..", ".kFFFk.", "kFeFFfk", ".kfffkFk", "..kkk.kk"];
const POISSON_BAS: [&str; 5] = ["kk.kkk..", "kFkfffk.", ".kffFeFk", "..kFFFk.", "...kkk.."];
const PAPILLON_OUVERT: [&str; 3] = ["Ak.kA", "AAkAA", ".aka."];
const PAPILLON_FERME: [&str; 3] = [".AkA.", ".aka.", "..k.."];


/// Une image en lettres au pixel d'art, son coin haut gauche en (`x`, `y`), retournée si `miroir`.
fn lettres(t: &mut Toile, x: i32, y: i32, image: &[&str], miroir: bool, couleur: &dyn Fn(u8) -> Option<Rgb>) {
    let large = image.iter().map(|l| l.len()).max().unwrap_or(0) as i32;
    for (j, ligne) in image.iter().enumerate() {
        for (i, l) in ligne.bytes().enumerate() {
            if let Some(col) = couleur(l) {
                let i = if miroir { large - 1 - i as i32 } else { i as i32 };
                t.poser(x + i, y + j as i32, col, 1.0);
            }
        }
    }
}

/// La vie de la cour, de temps en temps (user, 13-11 : « plus de vie dans la cour ») : un lapin sort du bois, fait
/// quelques bonds dans le pré, s'assied, regarde alentour et rentre ; un écureuil grimpe au tronc jusqu'à la couronne ;
/// un poisson saute dans le ruisseau du moulin ; des papillons dansent sur le pré au printemps et l'été.
fn petites_betes(t: &mut Toile, c: &Cadre, sky: &Ciel, plan: &Plan, se: Saison, temps: f32) {
    if sky.orage {
        return;
    }
    let room = c.to - c.from;
    let tic = temps as i64;
    let hiver = se == Saison::Hiver;
    let pal = |l: u8| -> Option<Rgb> {
        let k = match l {
            b'k' => 0x2a2224,
            b'W' => if hiver { 0xf0f0f6 } else { 0x9c8068 },
            b'w' => if hiver { 0xbcc2d0 } else { 0x6c5646 },
            b'c' => 0xf2ead8,
            b'm' => 0x141018,
            b'n' => 0xe89aa0,
            b'R' => 0xc8642c,
            b'r' => 0x8a3e1c,
            b'q' => 0xf0c890,
            b'F' => 0xc8d4e4,
            b'f' => 0x7890b0,
            b'e' => 0x101418,
            b'A' => 0xf4a83a,
            b'a' => 0x8a4a1a,
            b'B' => 0x6ab8f0,
            b'b' => 0x2a5a9a,
            _ => return None,
        };
        Some(sky.lit(hex(k), 0.0))
    };

    // 1. Le lapin, le jour : quatre bonds vers le pré, une pause à regarder, quatre bonds pour rentrer
    let droite = plan.well.map_or(plan.tree.0 as f32 - 6.0, |p| p as f32 - 3.0);
    if !sky.nuit && droite - (c.from + 46.0) > 10.0 {
        let (periode, tour) = (1100, tic.div_euclid(1100));
        let a = tic.rem_euclid(periode);
        if a < 220 {
            let u0 = c.from + 46.0 + h(tour * 13 + 1) * (droite - c.from - 56.0);
            let pas = 1.2f32;
            let (bonds, dans_bond, sens) = if a < 60 {
                ((a / 15) as f32 + (a % 15).min(6) as f32 / 6.0, a % 15 < 6, 1)
            } else if a < 140 {
                (4.0, false, if a < 100 { 1 } else { -1 })
            } else if a < 200 {
                let b = a - 140;
                (4.0 - (b / 15) as f32 - (b % 15).min(6) as f32 / 6.0, b % 15 < 6, -1)
            } else {
                (0.0, false, -1)
            };
            let saut = if dans_bond { ((a % 15) as f32 / 6.0 * std::f32::consts::PI).sin() * 3.0 } else { 0.0 };
            let image: &[&str] = if dans_bond { &LAPIN_BOND } else { &LAPIN_ASSIS };
            let (x, y) = (c.x(u0 + bonds * pas).round() as i32, (c.y(29.7) - saut).round() as i32 - image.len() as i32);
            lettres(t, x, y, image, sens < 0, &pal);
        }
    }

    // 2. L'écureuil, hors l'hiver et la nuit : il grimpe le long du tronc, de l'herbe jusque sous la couronne
    if !sky.nuit && !hiver && room >= 30.0 {
        let (tx, r) = (plan.tree.0 as f32 + 0.5, plan.tree.1 as f32);
        let a = tic.rem_euclid(800);
        if a < 90 {
            let (v_bas, v_haut) = (26.5f32, r + 0.5 + r * 0.75);
            let f = (a as f32 / 70.0).min(1.0);
            let v = v_bas + (v_haut - v_bas) * f;
            let image: &[&str] = if a < 70 && (a / 2) % 2 == 0 { &ECUREUIL } else { &ECUREUIL_2 };
            let (x, y) = (c.x(tx).round() as i32 - 4, c.y(v).round() as i32 - image.len() as i32);
            lettres(t, x, y, image, false, &pal);
        }
    }

    // 3. Le poisson qui saute dans le ruisseau du moulin, hors l'hiver : un arc d'argent, une gerbe en partant et en
    //    retombant
    if let Some(cx) = plan.wheel.filter(|_| !hiver) {
        let a = tic.rem_euclid(520);
        let u = cx as f32 + 0.5 - 13.6 + h(tic.div_euclid(520) * 7 + 3) * 1.5;
        let (x0, y0) = (c.x(u).round() as i32, c.y(31.2).round() as i32);
        let gerbe = |t: &mut Toile, x: i32| {
            let eau = sky.lit(hex(0xdcf2ff), 0.0);
            for (dx, dy) in [(-2, 0), (2, 0), (-1, -1), (1, -1)] {
                t.poser(x + dx, y0 + dy, eau, 1.0);
            }
        };
        if a < 16 {
            let f = a as f32 / 16.0;
            let x = x0 + (f * 12.0).round() as i32;
            let y = y0 - ((f * std::f32::consts::PI).sin() * 13.0).round() as i32 - 4;
            lettres(t, x - 4, y, if f < 0.5 { &POISSON_HAUT } else { &POISSON_BAS }, true, &pal);
            if a < 4 {
                gerbe(t, x0);
            }
        } else if a < 21 {
            gerbe(t, x0 + 12);
        }
    }

    // 4. Les papillons, le jour au printemps et l'été : trois qui dansent sur le pré, l'un monarque, l'autre bleu
    if !sky.nuit && matches!(se, Saison::Printemps | Saison::Ete) && room >= 30.0 {
        for i in 0..3i64 {
            let fi = i as f32;
            let base = c.from + 12.0 + h(i * 31 + 7) * (room - 24.0);
            let u = base + (temps / 41.0 + fi * 2.1).sin() * 4.5 + (temps / 13.0 + fi).sin() * 0.8;
            let v = 27.0 + (temps / 9.0 + fi * 1.7).sin() * 0.9 + (temps / 23.0 + fi).cos() * 0.6;
            let ouvert = (tic + i) % 3 != 0;
            let bleu = i % 2 == 1;
            let couleur = |l: u8| pal(match (l, bleu) {
                (b'A', true) => b'B',
                (b'a', true) => b'b',
                (l, _) => l,
            });
            let image: &[&str] = if ouvert { &PAPILLON_OUVERT } else { &PAPILLON_FERME };
            lettres(t, c.x(u).round() as i32 - 2, c.y(v).round() as i32 - 1, image, false, &couleur);
        }
    }
}

/// Lucioles dorées les nuits d'été, et feux follets bleutés qui dérivent doucement au-dessus de l'herbe et de l'eau la nuit.
fn lucioles(t: &mut Toile, c: &Cadre, sky: &Ciel, se: Saison, temps: f32) {
    if !sky.nuit || sky.orage {
        return;
    }
    let room = c.to - c.from;
    if se == Saison::Ete {
        for i in 0..(room / 6.0) as i64 {
            let u = c.from + 8.0 + h(i * 7) * (room - 8.0) + (temps / 20.0 + i as f32).sin() * 1.5;
            let v = 25.0 + h(i * 9) * 5.0 + (temps / 13.0 + i as f32 * 2.0).cos() * 0.8;
            let eclat = ((temps * 0.25 + i as f32 * 1.3).sin() * 0.5 + 0.5).powi(3);
            t.disque(c.x(u), c.y(v), 1.6, hex(0xd8ff7a), eclat, 0.5);
            t.disque(c.x(u), c.y(v), 5.0, hex(0xb8f060), eclat * 0.25, 1.0);
        }
    }
    // feux follets féeriques cyan/argent flottant lentement près des bois et du ruisseau
    if room >= 18.0 {
        let n_ff = ((room / 18.0).ceil() as i64).clamp(1, 3);
        for i in 0..n_ff {
            let base_u = c.from + 16.0 + h(i * 19 + 5) * (room - 17.5).max(1.0);
            let u = base_u + (temps / 26.0 + i as f32 * 2.3).sin() * 2.1;
            let v = 24.4 + h(i * 23 + 3) * 4.2 + (temps / 17.0 + i as f32 * 1.7).cos() * 1.1;
            let respire = 0.45 + 0.55 * ((temps * 0.14 + i as f32 * 2.1).sin() * 0.5 + 0.5);
            t.disque(c.x(u), c.y(v), 5.2, hex(0x38a4ee), respire * 0.24, 1.0);
            t.disque(c.x(u), c.y(v), 1.8, hex(0xa8f4ff), respire * 0.9, 0.3);
        }
    }
}
