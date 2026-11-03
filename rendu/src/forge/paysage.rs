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
use super::monde::{Monde, Saison};
use super::pixels::hasard;
use super::trace::{fois, hex, mix, mul, pack, palier, paliers, smooth, unpack, Rgb, Toile};

/// Le pixel d'art de la cour fine, en pixels d'écran, quand rien d'autre n'est dit (user, 03-11 : « pas du pixel
/// d'écran »… « sur mon écran c'est vraiment tout petit ») ; et les paliers de ses bords.
pub const GRAIN: usize = 4;
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
    3.5 + roule(u / 13.0, 41) * 7.0 + roule(u / 4.5, 42) * 1.8
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
            oiseaux(&mut toile, &c, &sky, t);
            arbre(&mut toile, &c, &sky, plan, se, t);
            puits(&mut toile, &c, &sky, plan, se, t);
            moulin(&mut toile, &c, &sky, plan, se, t, travail);
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
            px[y * wd + x] = pack(mix(sky.haut, sky.bas, trame(k, 7.0, x, y)));
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
            t.disque(c.x(u), c.y(v), 7.0 * c.demi, halo, 0.32, 1.0);
            t.disque(c.x(u), c.y(v), 1.5 * c.demi, disque, 1.0, 0.15);
        } else if sky.nuit && !sky.orage {
            // la lune, un croissant
            let (u, v) = (c.from + room * 0.72, 4.5);
            t.disque(c.x(u), c.y(v), 4.0 * c.demi, hex(0x8090c0), 0.18, 1.0);
            t.disque(c.x(u), c.y(v), 1.4 * c.demi, hex(0xeef0f6), 1.0, 0.05);
            let fond_lune = mix(sky.haut, sky.bas, (v / HORIZON).powf(1.15));
            t.disque(c.x(u + 0.55), c.y(v - 0.35), 1.2 * c.demi, fond_lune, 1.0, 0.05);
        }
    }
    let dir = sky.cote;
    let cretes = |f: fn(f32) -> f32| (0..wd).map(|x| f(u_de(x))).collect::<Vec<f32>>();
    let chaines = [
        Chaine { cretes: cretes(lointaine), lisses: cretes(lointaine_lisse), roche: [hex(0x6c7c9e), hex(0x7d8fb0), hex(0x98a8c6)], neige: 1.4, loin: 0.6, boisee: false },
        Chaine { cretes: cretes(haute), lisses: cretes(haute_lisse), roche: [hex(0x48566e), hex(0x5d6e86), hex(0x7d8ea6)], neige: 1.1, loin: 0.42, boisee: false },
        Chaine { cretes: cretes(proche), lisses: cretes(proche_lisse), roche: [hex(0x2c404c), hex(0x3e5560), hex(0x587480)], neige: 0.0, loin: 0.28, boisee: true },
    ];
    let pas = (0.8 * c.cw).max(1.0) as isize;
    for x in 0..wd {
        let u = u_de(x);
        for y in 0..hd {
            let v = (y as f32 + 0.5) / c.demi;
            if v >= SOL {
                break;
            }
            let i = y * wd + x;
            let mut col = unpack(px[i]);
            let mut ciel = 1.0f32;
            for (n, ch) in chaines.iter().enumerate() {
                let r = ch.cretes[x];
                let cov = palier(((v - r) * c.demi + 0.5).clamp(0.0, 1.0));
                if cov <= 0.0 {
                    continue;
                }
                let d = (v - r).max(0.0);
                // le versant : la pente de la crête au-dessus, cherchée en remontant vers la lumière (des pans en biais)
                let xs = (x as isize - (dir * d * 0.6 * c.cw) as isize).clamp(pas, wd as isize - 1 - pas) as usize;
                let pente = (ch.lisses[xs + pas as usize] - ch.lisses[xs - pas as usize]) / (2.0 * pas as f32 / c.cw);
                let face = trame(smooth(-0.3, 0.3, pente * dir), 2.0, x, y);
                let mut roche = if face < 0.5 { mix(ch.roche[0], ch.roche[1], face * 2.0) } else { mix(ch.roche[1], ch.roche[2], face * 2.0 - 1.0) };
                if ch.boisee && d > 2.0 {
                    // la proche se couvre de forêt en descendant
                    roche = mix(roche, mix(hex(0x24402e), hex(0x335a3c), face), trame(smooth(2.0, 4.5, d), 3.0, x, y) * 0.85);
                }
                // la neige des sommets, plus bas l'hiver et à l'ombre, aux bords déchiquetés
                let neige = if hiver { ch.neige + 2.6 + if n == 2 { 1.4 } else { 0.0 } } else { ch.neige } + roule(u * 1.3, 50 + n as i64) * 1.2 - 0.5 + (1.0 - face) * 0.6;
                // les plus hauts sommets la gardent plus bas
                let neige = neige + if n == 1 { (11.5 - r).max(0.0) * 0.5 } else { 0.0 };
                if (ch.neige > 0.0 || hiver) && d < neige {
                    let blanc = mix(hex(0xb4c4dc), hex(0xf4f7fb), face);
                    roche = mix(roche, blanc, trame(smooth(neige, neige - 0.3, d), 2.0, x, y));
                }
                // la brume du fond des vallées
                let brume = trame(smooth(r + 2.5, r + 9.0, v), 3.0, x, y) * 0.22;
                col = mix(col, sky.lit(roche, (ch.loin + brume).min(0.9)), cov);
                ciel *= 1.0 - cov;
            }
            px[i] = pack(col);
            voile[i] = (ciel * 255.0) as u8;
        }
    }
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
                    col = mix(col, sky.lit(teinte_sapin(u - cx, d, haut, [hex(0x1c3426), hex(0x264430), hex(0x365c3e)]), 0.1), cov);
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
                    col = mix(col, sky.lit(teinte_sapin(u - cx, d, haut, [hex(0x182e20), hex(0x23402c), hex(0x34583a)]), 0.03), cov);
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
            // le chemin, aux bords irréguliers ; l'hiver, la neige n'en laisse que le seuil
            let long = if hiver { 6.0 } else { 14.0 - roule(v * 1.3, 61) * 3.0 };
            if !gala && u - c.from < long {
                let rang = ((v - SOL) / 0.95).floor();
                let decale = (rang as i64 % 2) as f32;
                let (fu, fv) = (((u - c.from + decale) / 2.0).fract(), ((v - SOL) / 0.95).fract());
                let pave = ((u - c.from + decale) / 2.0).floor() as i64;
                let joint = fu < 0.07 || fu > 0.93 || fv < 0.1 || fv > 0.9 || ((fu - 0.5).abs() > 0.4 && (fv - 0.5).abs() > 0.35);
                col = if joint {
                    hex(0x4a4048)
                } else {
                    let base = [hex(0x7a7078), hex(0x6c6470), hex(0x847a84)][(h(pave * 13 + rang as i64 * 5) * 3.0) as usize % 3];
                    if fv < 0.3 {
                        mix(base, hex(0xa49aa4), 0.35)
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
    // le tronc de l'arbre, évasé au pied, l'écorce éclairée du côté du jour
    let (tx, r) = (plan.tree.0 as f32 + 0.5, plan.tree.1 as f32);
    let ty = r + 0.5;
    {
        let (haut, bas) = (ty + 3.0, SOL + 1.6);
        let pts = [(tx - 0.55, haut), (tx + 0.55, haut), (tx + 0.8, bas - 1.2), (tx + 1.6, bas), (tx - 1.6, bas), (tx - 0.8, bas - 1.2)];
        let pts: Vec<(f32, f32)> = pts.iter().map(|&(u, v)| (c.x(u), c.y(v))).collect();
        t.poly(&pts, sky.lit(hex(0x4a2c16), 0.0), 1.0);
        let clair = if dir > 0.0 { 0.25 } else { -0.45 };
        t.trait_((c.x(tx + clair), c.y(haut)), (c.x(tx + clair * 1.3), c.y(bas - 0.6)), 0.35 * c.cw, sky.lit(hex(0x6a4228), 0.0), 0.9);
        for k in 0..4 {
            let du = (h(k * 9 + 4) - 0.5) * 0.8;
            let (v0, v1) = (haut + 1.0 + h(k * 9 + 5) * 6.0, haut + 2.0 + h(k * 9 + 6) * 9.0);
            t.trait_((c.x(tx + du), c.y(v0)), (c.x(tx + du * 1.1), c.y(v1.min(bas - 0.5))), 1.0, sky.lit(hex(0x2e1a0c), 0.0), 0.8);
        }
    }
    // le puits : sa margelle de pierres, ses poteaux, son toit de tuiles et son treuil (le seau est animé)
    if let Some(p) = plan.well {
        let p = p as f32;
        let lit = |c: u32, ombre: bool| sky.lit(fois(hex(c), if ombre { 0.8 } else { 1.04 }), 0.0);
        // les poteaux
        for (a, b) in [(p + 1.0, p + 1.8), (p + 7.2, p + 8.0)] {
            t.poly(&[(c.x(a), c.y(18.0)), (c.x(b), c.y(18.0)), (c.x(b), c.y(24.2)), (c.x(a), c.y(24.2))], lit(0x5a3a1e, a > p + 5.0), 1.0);
            let bord = if dir > 0.0 { b - 0.2 } else { a + 0.2 };
            t.trait_((c.x(bord), c.y(18.2)), (c.x(bord), c.y(24.0)), 1.5, lit(0x7a5230, false), 0.8);
        }
        // le treuil
        t.poly(&[(c.x(p + 1.8), c.y(19.5)), (c.x(p + 7.2), c.y(19.5)), (c.x(p + 7.2), c.y(20.5)), (c.x(p + 1.8), c.y(20.5))], lit(0x7a4a24, false), 1.0);
        t.trait_((c.x(p + 1.8), c.y(19.7)), (c.x(p + 7.2), c.y(19.7)), 1.2, lit(0x9a6a3a, false), 0.8);
        // le toit, ses rangs de tuiles, la neige l'hiver
        let toit = [(c.x(p - 0.5), c.y(18.5)), (c.x(p + 9.5), c.y(18.5)), (c.x(p + 8.3), c.y(16.4)), (c.x(p + 0.7), c.y(16.4))];
        t.poly(&toit, lit(0x8a3a24, false), 1.0);
        t.trait_((c.x(p - 0.5), c.y(18.45)), (c.x(p + 9.5), c.y(18.45)), 0.25 * c.demi, lit(0x4a1c10, false), 1.0);
        t.trait_((c.x(p + 0.1), c.y(17.45)), (c.x(p + 8.9), c.y(17.45)), 1.2, lit(0x6a2a18, false), 0.8);
        for k in 0..12 {
            let a = p + 0.2 + k as f32 * 0.8;
            t.trait_((c.x(a + 0.25), c.y(16.5)), (c.x(a), c.y(18.3)), 1.0, lit(0x6a2a18, false), 0.6);
        }
        if hiver {
            t.poly(&[(c.x(p + 0.4), c.y(16.9)), (c.x(p + 8.6), c.y(16.9)), (c.x(p + 8.3), c.y(16.3)), (c.x(p + 0.7), c.y(16.3))], lit(0xeef2f8, false), 1.0);
        }
        // la margelle : des rangs de pierres décalés, la lèvre claire, l'ombre du côté loin du jour
        let (gauche, droite) = (p + 0.3, p + 8.7);
        t.poly(&[(c.x(gauche), c.y(24.0)), (c.x(droite), c.y(24.0)), (c.x(droite), c.y(30.0)), (c.x(gauche), c.y(30.0))], lit(0x4f4a55, false), 1.0);
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
                    let base = [0x8a8494, 0x7c7686, 0x948ea0][(h(rang * 17 + k * 5 + p as i64) * 3.0) as usize % 3];
                    t.poly(&[(c.x(a2), c.y(v0 + 0.1)), (c.x(b), c.y(v0 + 0.1)), (c.x(b), c.y(v0 + 0.92)), (c.x(a2), c.y(v0 + 0.92))], lit(base, ombre), 1.0);
                    t.trait_((c.x(a2 + 0.1), c.y(v0 + 0.22)), (c.x(b - 0.1), c.y(v0 + 0.22)), 1.2, lit(0xb0aabc, ombre), 0.6);
                }
                a += 1.8;
                k += 1;
            }
        }
        t.poly(&[(c.x(gauche - 0.25), c.y(23.6)), (c.x(droite + 0.25), c.y(23.6)), (c.x(droite + 0.25), c.y(24.3)), (c.x(gauche - 0.25), c.y(24.3))], lit(if hiver { 0xeef2f8 } else { 0xa49eb0 }, false), 1.0);
    }
    // la roue : son étang, sa rive, le ruisseau qui vient vers nous ; le chevalet du bief
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
        // le chevalet : un poteau et sa contre-fiche
        let poteau = c.to - 1.6;
        t.poly(&[(c.x(poteau - 0.4), c.y(11.0)), (c.x(poteau + 0.4), c.y(11.0)), (c.x(poteau + 0.4), c.y(SOL)), (c.x(poteau - 0.4), c.y(SOL))], sky.lit(hex(0x5a3a1e), 0.0), 1.0);
        t.trait_((c.x(poteau), c.y(15.0)), (c.x(poteau - 2.5), c.y(11.2)), 0.35 * c.cw, sky.lit(hex(0x4a2c16), 0.0), 1.0);
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

/// Les nuages qui passent derrière les monts, de droite à gauche, ronds, éclairés dessus et plus sombres dessous ;
/// plus vite, plus nombreux et plus sombres dans l'orage.
fn nuages(t: &mut Toile, voile: &[u8], c: &Cadre, sky: &Ciel, temps: f32) {
    let room = c.to - c.from;
    let n = ((room / 22.0).ceil() as i64).max(1) + if sky.orage { 3 } else { 0 };
    let vitesse = if sky.orage { 0.05 } else { 0.012 };
    let tour = room + 34.0;
    let (clair, ombre) = if sky.orage {
        (hex(0x5e6270), hex(0x34384a))
    } else if sky.nuit {
        (hex(0x34406a), hex(0x1c2444))
    } else if sky.soir {
        (hex(0xf6c0aa), hex(0x8a5a7a))
    } else {
        (hex(0xfbfcfe), hex(0xbac6d6))
    };
    for i in 0..n {
        let depart = h(i * 41 + 3) * tour;
        let u = c.to + 14.0 - (depart + temps * vitesse * (0.7 + 0.6 * h(i * 3))).rem_euclid(tour);
        let taille = 1.5 + h(i * 11) * 1.5 + if sky.orage { 1.0 } else { 0.0 };
        let base = 3.5 + h(i * 7 + 1) * 6.0 + taille;
        let a = if sky.nuit { 0.75 } else { 0.96 };
        let bosses = 7;
        for passe in 0..2 {
            for k in 0..bosses {
                let f = k as f32 / (bosses - 1) as f32;
                let r = taille * (0.55 + 0.45 * (f * PI).sin()) * (0.8 + 0.4 * h(i * 97 + k));
                let x = c.x(u + (f - 0.5) * taille * 3.2);
                let y = c.y(base - r * 0.8 - (f * PI).sin() * taille * 0.5);
                if passe == 0 {
                    disque_ciel(t, voile, x, y + 0.25 * r * c.demi, r * c.demi, ombre, a, 0.12);
                } else {
                    disque_ciel(t, voile, x - 0.15 * r * c.cw, y - 0.14 * r * c.demi, r * c.demi * 0.8, clair, a, 0.22);
                }
            }
        }
    }
}

/// Les cascades des cols : des filets d'eau qui descendent, avec un pas de côté à chaque corniche, une fente d'ombre à
/// leur gauche ; l'embrun au pied de la proche. Gelées l'hiver.
fn cascades(t: &mut Toile, bois: &[u8], c: &Cadre, sky: &Ciel, plan: &Plan, se: Saison, temps: f32) {
    let hiver = se == Saison::Hiver;
    for &(fx, pres) in plan.falls() {
        let fx = fx as f32 + 0.5;
        let haut = if pres { proche(fx) } else { haute(fx) } + 1.4;
        let fin = if pres { 23.6 } else { proche(fx) };
        if fin <= haut {
            continue;
        }
        let graine = fx as i64 * 7 + if pres { 3 } else { 0 };
        let loin = if pres { 0.08 } else { 0.3 };
        let large = if pres { 1.6 } else { 1.05 };
        // en pixels d'art par tic : la proche court plus vite
        let vitesse = if pres { 1.6 } else { 0.9 };
        let (y0, y1) = ((haut * c.demi) as i32, (fin * c.demi) as i32);
        let place = |v: f32| fx + ((roule(v.floor() / 3.0, graine) - 0.5) * 3.2).round() * 0.3;
        for y in y0..y1 {
            let v = (y as f32 + 0.5) / c.demi;
            // un pas de côté par corniche (une rangée de forge), et l'écume là où elle retombe
            let a = place(v);
            let corniche = (place(v - 1.0) - a).abs() > 0.01 && v.fract() < 0.55;
            // des bords qui tremblent d'un pixel, rangée par rangée : de l'eau, pas une colonne
            let (ga, gb) = ((h(y as i64 * 7 + graine) > 0.65) as i32, (h(y as i64 * 11 + graine) > 0.6) as i32);
            let (xa, xb) = (c.x(a - 0.35).floor() as i32, c.x(a + large).ceil() as i32 + gb);
            let xd = c.x(a).round() as i32 - ga;
            for x in xa..xb {
                let (xc, yc) = ((x.max(0) as usize).min(t.w - 1), (y.max(0) as usize).min(t.h - 1));
                let devant = 1.0 - bois[yc * t.w + xc] as f32 / 255.0;
                if devant <= 0.0 {
                    continue;
                }
                if x < xd {
                    if pres && !corniche {
                        t.poser(x, y, hex(0x101420), 0.25 * devant);
                    } // la fente d'ombre à sa gauche
                    continue;
                }
                let i = (x - xd).max(0);
                let teinte = if hiver {
                    if h((x / 2) as i64 * 3 + (y / 3) as i64 * 7) > 0.5 { hex(0xe4f2fa) } else { hex(0xbfe0f0) }
                } else if corniche {
                    if h(x as i64 * 5 + y as i64) > 0.3 { hex(0xf8fdff) } else { hex(0xcdeefc) }
                } else {
                    let phase = h(i as i64 * 13 + graine) * 9.0;
                    let s = (y as f32 - temps * vitesse * (0.8 + h(i as i64 * 7 + graine) * 0.4) + phase).rem_euclid(9.0);
                    if x == xd {
                        hex(0xdff6ff)
                    } else if x >= xb - 1 {
                        hex(0x5aa8dc)
                    } else if s < 2.5 {
                        hex(0xf8fdff)
                    } else if s < 4.5 {
                        hex(0xbfe6fa)
                    } else {
                        hex(0x7cc4ec)
                    }
                };
                t.poser(x, y, sky.lit(teinte, loin), if pres { 0.92 } else { 0.72 } * devant);
            }
        }
        // l'embrun au pied de la proche, parmi les pointes des sapins
        if pres && !hiver {
            let respire = 0.5 + 0.2 * (temps / 3.0).sin();
            t.disque(c.x(fx + 0.4), c.y(fin - 0.4), 1.6 * c.demi, sky.lit(hex(0xf4f9fc), 0.15), respire * 0.45, 1.0);
        }
    }
}

/// Des oiseaux qui passent de temps en temps, le jour sans orage : un petit vol en V, les ailes qui battent.
fn oiseaux(t: &mut Toile, c: &Cadre, sky: &Ciel, temps: f32) {
    if sky.nuit || sky.orage {
        return;
    }
    let room = c.to - c.from;
    let periode = 700.0;
    let tour = (temps / periode).floor() as i64;
    let f = (temps / periode).fract() * 2.6;
    if f > 1.0 {
        return; // ils sont passés ; d'autres viendront
    }
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

/// Les couleurs d'une grappe de feuilles, de l'ombre à la lumière, selon la saison.
fn feuillage(se: Saison) -> Option<[Rgb; 4]> {
    Some(match se {
        Saison::Automne => [hex(0x8a3412), hex(0xc85a1c), hex(0xe88a2a), hex(0xf2c24a)],
        Saison::Ete => [hex(0x1f4626), hex(0x2c6230), hex(0x3e7e36), hex(0x5e9a44)],
        Saison::Printemps => [hex(0x33702c), hex(0x4f9038), hex(0x6eb04a), hex(0x96cc62)],
        Saison::Hiver => return None,
    })
}

/// L'arbre : ses branches en éventail, sa couronne en grappes qui se balancent au vent (plus fort dans l'orage),
/// éclairées du côté du jour ; des feuilles qui tombent en tournoyant l'automne, des pétales au printemps ; nu l'hiver,
/// la neige sur ses branches.
fn arbre(t: &mut Toile, c: &Cadre, sky: &Ciel, plan: &Plan, se: Saison, temps: f32) {
    let (tx, r) = (plan.tree.0 as f32 + 0.5, plan.tree.1 as f32);
    let ty = r + 0.5;
    let dir = sky.cote;
    let vent = if sky.orage { 1.4 } else { 0.6 };
    let rythme = if sky.orage { 3.0 } else { 7.0 };
    let balance = |v: f32| (temps / rythme + v / 3.0).sin() * vent * ((ty + r - v) / (2.0 * r)).clamp(0.0, 1.0);
    let bois = sky.lit(hex(0x4a2c16), 0.0);
    // les branches : trois paires depuis le tronc
    for (i, b) in [0.45f32, 1.0, 1.8].iter().enumerate() {
        for cote in [-1.0f32, 1.0] {
            let up0 = i as f32 * 1.5 + 0.2;
            let up1 = ((r - 1.0) / b).min(3.0 + r * 0.8);
            let (v0, v1) = (ty + 3.0 - up0, ty + 3.0 - up1);
            let (u0, u1) = (tx + cote * up0 * b, tx + cote * up1 * b + balance(v1));
            // la branche se courbe un peu vers le haut, puis se fourche en rameaux
            let (um, vm) = ((u0 + u1) * 0.5 - cote * 0.3, (v0 + v1) * 0.5 - 0.4);
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
    let Some(couleurs) = feuillage(se) else { return };
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
    let contour = sky.lit(if se == Saison::Automne { hex(0x4a1c0a) } else { hex(0x10261a) }, 0.0);
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
            let teinte = if se == Saison::Printemps { hex(0xf8c4d8) } else { [hex(0xe07a22), hex(0xc04a1a), hex(0xe3a63c)][i as usize % 3] };
            let (x, y) = (c.x(u), c.y(v));
            let tourne = (temps * 0.5 + i as f32).sin();
            t.trait_((x - 2.0 * tourne, y - 1.0), (x + 2.0 * tourne, y + 1.0), 2.0, sky.lit(teinte, 0.0), 1.0);
        }
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

/// Le seau du puits : il pend sous le treuil, et de temps en temps il descend dans le puits et remonte en gouttant,
/// la manivelle tournant.
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
    t.trait_((c.x(corde), c.y(20.5)), (c.x(corde), c.y(haut.min(23.9))), 1.4, lit(0x9a94a4), 1.0);
    // la manivelle, à droite du poteau
    let angle = if bouge { temps * 0.8 } else { 0.6 };
    let (mx, my) = (c.x(p + 8.4), c.y(20.0));
    let (bx, by) = (mx + angle.cos() * 0.9 * c.cw, my + angle.sin() * 0.9 * c.demi);
    t.trait_((mx, my), (bx, by), 2.0, lit(0x5a3a1e), 1.0);
    t.disque(bx, by, 0.25 * c.demi, lit(0x7a4a24), 1.0, 0.0);
    // le seau, caché par la margelle quand il descend
    if haut < 23.9 {
        let bas = (haut + 1.3).min(23.9);
        t.poly(&[(c.x(p + 3.5), c.y(haut)), (c.x(p + 5.5), c.y(haut)), (c.x(p + 5.3), c.y(bas)), (c.x(p + 3.7), c.y(bas))], lit(0x6a4228), 1.0);
        for v in [haut + 0.3, haut + 1.0] {
            if v < bas {
                t.trait_((c.x(p + 3.55), c.y(v)), (c.x(p + 5.45), c.y(v)), 1.3, lit(0x3a3a44), 1.0);
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

/// La roue du moulin, à la manière de cour.rs (son tour, ses godets qui se remplissent à la couronne et versent passé
/// le bas), au vrai pixel : la jante, les rayons, les palettes, le moyeu ; le bief et son eau qui court, le filet qui
/// tombe du bec, les débords, les éclaboussures, l'étang qui ondule et le ruisseau qui file, les roseaux qui plient.
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
    let net = |d: f32| palier((d / e + 0.5).clamp(0.0, 1.0));
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
                // les godets, entre les deux anneaux : le fond dans l'ombre, les cloisons de planche penchées, l'eau
                // dedans du côté où ils descendent. La roue tourne vers les angles décroissants (`tour` croît) : le bout
                // de chaque cloison se rabat vers l'arrière, vers l'eau qu'elle porte, en coupe (user, 03-11 : « est-ce
                // que l'intérieur est à l'envers ? » : elles s'ouvraient vers l'avant, l'eau aurait coulé)
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
                // le bout des cloisons, qui dépasse : la silhouette dentée qui montre que ça tourne
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
    // le bief sur son chevalet, de l'au-delà du bord jusqu'au-dessus de la roue, et son eau qui court
    let (b0, b1) = (cx - 1.6, c.to + 1.0);
    t.poly(&[(c.x(b0), c.y(11.0)), (c.x(b1), c.y(11.0)), (c.x(b1), c.y(11.8)), (c.x(b0), c.y(11.8))], lit(0x7a4a24), 1.0);
    t.trait_((c.x(b0), c.y(11.75)), (c.x(b1), c.y(11.75)), 1.2, lit(0x4a2c16), 1.0);
    let (ya, yb) = (c.y(10.1) as i32, c.y(11.0) as i32);
    for y in ya..yb {
        for x in c.x(b0) as i32..(c.x(b1) as i32).min(t.w as i32) {
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
        // des glaçons sous le bief
        for k in 0..((b1 - b0) as i64) {
            if k % 2 == 0 {
                let u = b0 + 0.4 + k as f32;
                t.poly(&[(c.x(u - 0.2), c.y(11.8)), (c.x(u + 0.2), c.y(11.8)), (c.x(u), c.y(12.6 + h(k) * 0.6))], lit(0xcfe8f6), 1.0);
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
                // et les éclaboussures où il tombe
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
                let f = ((temps * 0.012 + h(k * 5) ).fract()) as f32;
                (cx - 9.0 - f * 6.0 + (h(k * 3) - 0.5) * 2.0, 29.2 + f * 3.0)
            } else {
                (cx - 7.5 + h(k * 7) * (c.to - cx + 7.0) - (temps * 0.02 + h(k)).fract() * 0.0, 27.5 + h(k * 11) * 1.3)
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

/// L'avant-toit de la porte (user, 03-11 : « elle pourrait être un mélange des deux… l'intérieur de la forge et
/// l'extérieur ») : sur la pente des cellules de cour.rs (une rangée pour trois colonnes), des rangs d'ardoises plus
/// grosses que les pixels de la cour, jointes de sombre, la lèvre claire en haut ; la poutre de chêne dessous et son
/// bout coupé. L'hiver la neige dessus et des glaçons ; dans l'orage l'eau qui descend les ardoises et goutte du bout.
fn avant_toit(t: &mut Toile, c: &Cadre, sky: &Ciel, se: Saison, temps: f32) {
    let fin = c.from + 13.0;
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
                // les ardoises : des rangs parallèles à la pente, décalés d'un rang à l'autre
                let rang = (n / 0.7).floor();
                let long = (u - c.from + rang * 0.7) / 1.4;
                let (fu, fn_) = (long.fract(), (n / 0.7).fract());
                let ardoise = long.floor() as i64 * 7 + rang as i64 * 3;
                if fu < 0.1 || fn_ > 0.82 {
                    lit(0x2e2a36)
                } else if se == Saison::Hiver && rang == 0.0 {
                    lit(if fn_ < 0.4 { 0xf6f9fc } else { 0xdfe6f0 })
                } else {
                    let base = [0x6e6878, 0x5d5866, 0x7a7486][(h(ardoise) * 3.0) as usize % 3];
                    if fn_ < 0.22 {
                        lit(0x9a94a8)
                    } else {
                        lit(base)
                    }
                }
            } else if u > fin - 0.5 {
                lit(0x6a4228) // le bout de la poutre, coupé
            } else if n < 2.3 {
                lit(0x5a3a1e)
            } else {
                lit(0x3a2418)
            };
            t.poser(x, y, col, 1.0);
        }
    }
    // la jambe de force, du poteau à la poutre, et son liseré clair
    let (p0, p1) = ((c.x(c.from + 0.9), c.y(8.2)), (c.x(c.from + 6.0), c.y(2.0 + 2.9)));
    t.trait_(p0, p1, 0.55 * c.cw, lit(0x3a2418), 1.0);
    t.trait_((p0.0, p0.1 - 1.0), (p1.0, p1.1 - 1.0), 1.0, lit(0x6a4228), 1.0);
    if se == Saison::Hiver {
        for k in 0..6 {
            let u = c.from + 1.2 + k as f32 * 2.0 + h(k) * 0.6;
            let v = (u - c.from) / 3.0 + 3.0;
            let l = 0.5 + h(k * 3 + 1) * 0.9;
            t.poly(&[(c.x(u - 0.25), c.y(v)), (c.x(u + 0.25), c.y(v)), (c.x(u), c.y(v + l))], lit(0xcfe8f6), 1.0);
        }
    }
    if sky.orage {
        // l'eau qui descend les ardoises, et qui goutte du bout jusqu'au sol
        for k in 0..5 {
            let f = ((temps * 0.08 + k as f32 * 0.2).fract()) * 13.0;
            let u = c.from + f;
            t.trait_((c.x(u), c.y(u / 3.0 - c.from / 3.0 - 0.1)), (c.x(u + 0.6), c.y((u + 0.6 - c.from) / 3.0 - 0.1)), 1.0, lit(0x9fdcff), 1.0);
        }
        for k in 0..3 {
            let f = (temps * 0.12 + k as f32 * 0.33).fract();
            let v = fin / 3.0 - c.from / 3.0 + 3.0 + f * (SOL - 7.0);
            t.trait_((c.x(fin - 0.2), c.y(v)), (c.x(fin - 0.2), c.y(v + 0.5)), 1.0, lit(0x9fdcff), 1.0);
        }
    }
}

/// Des lucioles, les nuits d'été, qui clignent au ras de l'herbe.
fn lucioles(t: &mut Toile, c: &Cadre, sky: &Ciel, se: Saison, temps: f32) {
    if !sky.nuit || sky.orage || se != Saison::Ete {
        return;
    }
    let room = c.to - c.from;
    for i in 0..(room / 6.0) as i64 {
        let u = c.from + 8.0 + h(i * 7) * (room - 8.0) + (temps / 20.0 + i as f32).sin() * 1.5;
        let v = 25.0 + h(i * 9) * 5.0 + (temps / 13.0 + i as f32 * 2.0).cos() * 0.8;
        let eclat = ((temps * 0.25 + i as f32 * 1.3).sin() * 0.5 + 0.5).powi(3);
        t.disque(c.x(u), c.y(v), 1.6, hex(0xd8ff7a), eclat, 0.5);
        t.disque(c.x(u), c.y(v), 5.0, hex(0xb8f060), eclat * 0.25, 1.0);
    }
}
