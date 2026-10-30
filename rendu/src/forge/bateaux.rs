//! Les bateaux, au vrai pixel : leurs sortes (les classes de course, les vieux gréements, une
//! vedette), leur gréement, leur sillage, leurs feux, chacun gardé en vignette par pas de
//! tangage. La baie de la Route du Rhum (baie.rs) en fait une flotte ; toute autre scène de mer
//! peut s'en servir : il suffit d'un cadre (`Cadre`), d'un ciel (`Ciel`) et d'une profondeur.

use super::baie::{echelle, Cadre, Ciel};
use super::monde::PhaseRhum;
use super::pixels::hasard;
use super::trace::{bande, fois, hex, mix, mul, smooth, unpack, Rgb, Toile};

/// Les sortes de bateaux : les classes de la course, puis ceux qui la regardent passer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sorte {
    Ultim,
    OceanFifty,
    Imoca,
    Class40,
    /// un vieux trimaran de course : trois coques basses, ni foils ni vol
    VintageMulti,
    /// un sloop classique : coque vernie, voiles crème, longs élancements
    VintageMono,
    /// un vieux gréement à deux mâts, voiles tannées à corne, beaupré
    Goelette,
    /// une vedette de spectateurs : une cabine, pas de voile
    Vedette,
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


/// Sa longueur et la hauteur de son mât, en pixels de forge, tout près.
pub fn gabarit(s: Sorte) -> (f32, f32) {
    match s {
        Sorte::Ultim => (22.0, 1.0),
        Sorte::OceanFifty => (13.0, 1.15),
        Sorte::Imoca => (13.0, 1.55),
        Sorte::Class40 => (11.0, 1.5),
        Sorte::VintageMulti => (12.0, 1.15),
        Sorte::VintageMono => (10.5, 1.4),
        Sorte::Goelette => (18.0, 1.05),
        Sorte::Vedette => (7.5, 0.0),
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
    pub fn faire(w: usize, h: usize, ox: f32, oy: f32, dessin: &dyn Fn(&mut Toile, f32, f32)) -> Vignette {
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
    pub fn poser(&self, t: &mut Toile, x: f32, y: f32) {
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
pub fn bateau(t: &mut Toile, gardees: &mut Vignettes, c: &Cadre, sky: &Ciel, b: &Bateau, (u, dir): (f32, f32), t_: f32, phase: PhaseRhum, taille: f32, phosphore: bool) {
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
    let long = if course { 1.8 } else if b.sorte == Sorte::Vedette { 1.2 } else { 0.6 };
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
    match b.sorte {
        Sorte::Vedette => return vedette(t, &pts, &teinte, b, l, s, c),
        Sorte::Goelette => return goelette(t, p, &pts, &teinte, b, l, s, mat, c),
        _ => {}
    }
    let voile_pts: Vec<(f32, f32)> = match b.sorte {
        Sorte::Ultim | Sorte::OceanFifty => vec![(0.03, -0.98), (0.03, -0.09), (-0.44, -0.10), (-0.42, -0.3), (-0.33, -0.6), (-0.18, -0.85), (-0.06, -0.96)],
        Sorte::VintageMulti => vec![(0.03, -0.98), (0.03, -0.09), (-0.40, -0.10), (-0.34, -0.5), (-0.18, -0.82), (-0.05, -0.96)],
        // la grand-voile triangulaire, sa bôme longue qui déborde l'arrière
        Sorte::VintageMono => vec![(0.06, -1.4), (0.07, -0.16), (-0.52, -0.17), (-0.2, -0.85)],
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
        Sorte::VintageMulti => {
            // trois coques basses dans l'eau, les bras par-dessus : un trimaran d'avant les foils
            let h = -0.035;
            for bx in [-0.17, 0.16] {
                t.ligne(&pts(&[(bx - 0.02, -0.012), (bx - 0.01, -0.03 + h * 0.4), (bx + 0.01, -0.045 + h)]), (0.026 * l * c.cw).max(1.3), ombre(b.coque, 0.55), 1.0);
            }
            t.poly(&pts(&[(-0.48, -0.105 + h), (0.34, -0.1 + h), (0.5, -0.06 + h), (0.46, -0.04 + h), (-0.45, -0.035 + h)]), teinte(b.coque), 1.0);
            t.poly(&pts(&[(-0.48, -0.105 + h), (0.34, -0.1 + h), (0.41, -0.088 + h), (-0.48, -0.092 + h)]), teinte(b.bande), 0.9);
            t.poly(&pts(&[(-0.33, -0.032), (0.33, -0.032), (0.43, -0.01), (0.38, 0.008), (-0.31, 0.006)]), ombre(b.coque, 0.9), 1.0);
        }
        Sorte::VintageMono => {
            // l'étrave élancée, la voûte qui déborde, le liston verni, le rouf de bois
            t.poly(&pts(&[(-0.56, -0.115), (0.52, -0.125), (0.42, -0.045), (0.3, 0.0), (-0.36, 0.0), (-0.47, -0.05)]), teinte(b.coque), 1.0);
            t.poly(&pts(&[(-0.56, -0.115), (0.52, -0.125), (0.5, -0.108), (-0.55, -0.1)]), teinte(b.bande), 0.95);
            t.poly(&pts(&[(-0.24, -0.118), (0.06, -0.12), (0.03, -0.16), (-0.21, -0.158)]), ombre(hex(0x9a6a3a), 1.0), 1.0);
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
    let multi = matches!(b.sorte, Sorte::Ultim | Sorte::OceanFifty | Sorte::VintageMulti);
    if multi {
        t.trait_(p(-0.34, 0.008), p(0.42, 0.008), (0.7 + s * 1.2).min(2.2), blanc, 0.55);
    } else {
        t.trait_(p(-0.5, 0.004), p(0.48, 0.004), (0.9 + s * 1.4).min(2.6), blanc, 0.55);
    }
    // sur le multicoque qui vole, le gréement monte avec la coque
    let leve = if b.sorte == Sorte::Ultim { -0.05 - 0.035 * vol - 0.03 } else if b.sorte == Sorte::VintageMulti { -0.07 } else if multi { -0.03 - 0.02 * vol - 0.02 } else { 0.0 };
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
    let gen = course && matches!(b.sorte, Sorte::Class40 | Sorte::VintageMono | Sorte::VintageMulti | Sorte::Imoca);
    if gen {
        // portant vers les Antilles : le gennaker, gonflé, devant
        let couleur = teinte(mix(b.bande, hex(0xffffff), 0.15));
        t.poly(&pts(&[(0.04, foc_haut * 1.08), (0.82, -0.11), (0.72, foc_haut * 0.35), (0.5, foc_haut * 0.75)]), couleur, 0.97);
        t.ligne(&pts(&[(0.04, foc_haut * 1.08), (0.5, foc_haut * 0.75), (0.72, foc_haut * 0.35), (0.82, -0.11)]), 1.0, fois(couleur, 0.8), 0.6);
    } else {
        t.poly(&pts(&[(0.05, foc_haut), (0.5, -0.085), (0.08, -0.095)]), fois(voile, 0.93), 1.0);
    }
}

/// La vedette des spectateurs : une coque blanche à liston, sa cabine (de la couleur de « voile »),
/// ses hublots, une antenne.
fn vedette(t: &mut Toile, pts: &dyn Fn(&[(f32, f32)]) -> Vec<(f32, f32)>, teinte: &dyn Fn(Rgb) -> Rgb, b: &Bateau, l: f32, s: f32, c: &Cadre) {
    t.poly(&pts(&[(-0.5, -0.17), (0.38, -0.17), (0.52, -0.12), (0.44, -0.03), (0.3, 0.0), (-0.48, 0.0)]), teinte(b.coque), 1.0);
    t.poly(&pts(&[(-0.5, -0.07), (0.44, -0.07), (0.41, -0.04), (-0.49, -0.04)]), teinte(b.bande), 1.0);
    t.poly(&pts(&[(-0.34, -0.17), (0.12, -0.17), (0.02, -0.31), (-0.3, -0.31)]), teinte(b.voile), 1.0);
    if s > 0.3 {
        for k in 0..3 {
            let x = -0.27 + k as f32 * 0.1;
            t.poly(&pts(&[(x, -0.28), (x + 0.065, -0.28), (x + 0.065, -0.22), (x, -0.22)]), teinte(hex(0x22303c)), 0.9);
        }
    }
    let haut = pts(&[(-0.2, -0.31), (-0.21, -0.48)]);
    t.trait_(haut[0], haut[1], (0.01 * l * c.cw).max(1.0), teinte(hex(0x2a2a30)), 0.9);
}

/// La goélette : coque sombre à liston blanc, beaupré, deux mâts de bois, deux voiles à corne
/// tannées, deux focs ; son reflet sous elle.
#[allow(clippy::too_many_arguments)]
fn goelette(t: &mut Toile, p: &dyn Fn(f32, f32) -> (f32, f32), pts: &dyn Fn(&[(f32, f32)]) -> Vec<(f32, f32)>, teinte: &dyn Fn(Rgb) -> Rgb, b: &Bateau, l: f32, s: f32, mat: f32, c: &Cadre) {
    let voile = teinte(b.voile);
    let bois = teinte(hex(0x6a4a2a));
    let e = (0.008 * l * c.cw).max(1.0);
    // misaine (devant), grand-voile (derrière) : de la bôme à la corne
    let misaine = [(0.16, -0.14), (-0.12, -0.14), (-0.08, -0.68 * mat), (0.17, -0.82 * mat)];
    let grande = [(-0.14, -0.14), (-0.48, -0.15), (-0.42, -0.74 * mat), (-0.13, -0.92 * mat)];
    if s > 0.4 {
        for v in [&misaine, &grande] {
            let reflet: Vec<(f32, f32)> = v.iter().map(|&(x, y)| p(x, -y * 0.3 + 0.03)).collect();
            t.poly(&reflet, voile, 0.1);
        }
    }
    t.poly(&pts(&[(-0.5, -0.12), (0.46, -0.13), (0.5, -0.1), (0.4, -0.02), (0.28, 0.0), (-0.4, 0.0), (-0.48, -0.06)]), teinte(b.coque), 1.0);
    t.poly(&pts(&[(-0.5, -0.12), (0.46, -0.13), (0.47, -0.113), (-0.5, -0.104)]), teinte(b.bande), 1.0);
    let beaupre = pts(&[(0.45, -0.125), (0.7, -0.17)]);
    t.trait_(beaupre[0], beaupre[1], e, bois, 1.0);
    for (x, h) in [(0.17, 0.9 * mat), (-0.13, mat)] {
        let m = pts(&[(x, -0.12), (x, -h)]);
        t.trait_(m[0], m[1], e * 1.2, bois, 1.0);
    }
    t.poly(&pts(&misaine), voile, 1.0);
    t.poly(&pts(&grande), voile, 1.0);
    // les coutures des voiles tannées, un ton plus sombre
    for v in [&misaine, &grande] {
        for f in [0.33, 0.66] {
            let (a, b2) = ((v[1].0 + (v[2].0 - v[1].0) * f, v[1].1 + (v[2].1 - v[1].1) * f), (v[0].0 + (v[3].0 - v[0].0) * f, v[0].1 + (v[3].1 - v[0].1) * f));
            t.trait_(p(a.0, a.1), p(b2.0, b2.1), 1.0, fois(voile, 0.82), 0.6);
        }
    }
    t.poly(&pts(&[(0.19, -0.84 * mat), (0.68, -0.17), (0.22, -0.16)]), mix(voile, hex(0xf2ead8), 0.4), 1.0);
    t.poly(&pts(&[(0.19, -0.6 * mat), (0.5, -0.15), (0.2, -0.15)]), mix(voile, hex(0xf2ead8), 0.25), 1.0);
}
