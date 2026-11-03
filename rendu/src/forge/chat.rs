//! La panthère de la forge au pixel d'art de la cour (user, 03-11 : « le chat panthère aurait bien besoin d'être à
//! jour », puis « pixel art fin »). Elle remplace le dessin en lettres d'atelier.rs quand les personnages se
//! peignent au pixel (`pet`) : à la même place, de la même taille (14 × 7 pixels de forge debout, 9 × 5 roulée
//! sur l'appui), mais tracée en formes au pixel d'art de 4 pixels d'écran (`paysage::GRAIN`) : la tête et ses
//! oreilles, le museau crème, le poitrail, le corps, la croupe, quatre pattes qui marchent l'amble du chat (la
//! patte arrière, puis l'avant du même côté), la longue queue qui pend et remonte du bout. Puis les couleurs :
//! l'or éclairé sur le dos, ombré dessous, le ventre crème, les rosettes, l'œil vert ; et le contour sombre du
//! pixel art. Enfin la lumière de la forge, comme sur les personnages (`pet::Eclairage`).

use super::atelier::{CAT_FLOOR_Y, CAT_SILL};
use super::monde::{Chat, ModeChat};
use super::paysage::GRAIN;
use super::pet::Eclairage;
use std::f32::consts::PI;

/// Où elle se tient, en pixels de forge : la gauche, le haut, la largeur, la hauteur ; le bas est le sol (ou
/// l'appui, roulée).
pub fn emprise(cat: &Chat) -> (i32, i32, i32, i32) {
    if cat.mode == ModeChat::Roof {
        // l'appui sous l'ancien dessin (cinq rangées dès CAT_SILL.1) ; trois de plus au-dessus pour le z
        (CAT_SILL.0, CAT_SILL.1 - 3, 11, 8)
    } else {
        (cat.x, CAT_FLOOR_Y, 14, 7)
    }
}

const VIDE: u8 = 0;
const POIL: u8 = 1;
const LOIN: u8 = 2; // les pattes de l'autre côté, plus sombres
const CREME: u8 = 3;
const QUEUE: u8 = 4;
const PATTE: u8 = 5; // les pattes de ce côté : de l'or, pas le crème du ventre
const TETE: u8 = 6;

/// De l'avant à l'arrière : la partie de derrière prend le trait sombre là où celle de devant la touche (le pixel
/// art cerne chaque partie, pas seulement la silhouette). Roulée, la queue passe devant tout.
fn profondeur(f: u8, dort: bool) -> u8 {
    match (f, dort) {
        (QUEUE, true) => 0,
        (PATTE, _) => 1,
        (TETE | CREME, _) => 2,
        (POIL, _) => 3,
        (QUEUE, false) => 4,
        _ => 5,
    }
}

fn hex(c: u32) -> [f32; 3] {
    [((c >> 16) & 255) as f32, ((c >> 8) & 255) as f32, (c & 255) as f32]
}

fn hache(n: i64) -> f32 {
    let x = (n.wrapping_mul(0x9E37_79B9) ^ (n >> 7)).wrapping_mul(0x85EB_CA6B) as u64;
    ((x >> 11) & 0xFFFF) as f32 / 65535.0
}

fn ell(u: f32, v: f32, cx: f32, cy: f32, rx: f32, ry: f32) -> bool {
    let (a, b) = ((u - cx) / rx, (v - cy) / ry);
    a * a + b * b <= 1.0
}

fn capsule(u: f32, v: f32, a: (f32, f32), b: (f32, f32), r: f32) -> bool {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l = dx * dx + dy * dy;
    let t = if l > 0.0 { (((u - a.0) * dx + (v - a.1) * dy) / l).clamp(0.0, 1.0) } else { 0.0 };
    let (px, py) = (a.0 + dx * t - u, a.1 + dy * t - v);
    px * px + py * py <= r * r
}

fn tri(u: f32, v: f32, a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> bool {
    let s = |p: (f32, f32), q: (f32, f32)| (q.0 - p.0) * (v - p.1) - (q.1 - p.1) * (u - p.0);
    let (d1, d2, d3) = (s(a, b), s(b, c), s(c, a));
    !((d1 < 0.0 || d2 < 0.0 || d3 < 0.0) && (d1 > 0.0 || d2 > 0.0 || d3 > 0.0))
}

fn bezier(p: [(f32, f32); 4], t: f32) -> (f32, f32) {
    let m = 1.0 - t;
    let (a, b, c, d) = (m * m * m, 3.0 * m * m * t, 3.0 * m * t * t, t * t * t);
    (a * p[0].0 + b * p[1].0 + c * p[2].0 + d * p[3].0, a * p[0].1 + b * p[1].1 + c * p[2].1 + d * p[3].1)
}

/// Une queue : des capsules le long d'une courbe, de plus en plus fines.
fn le_long(u: f32, v: f32, p: [(f32, f32); 4], r0: f32, r1: f32) -> bool {
    let n = 14;
    let mut avant = p[0];
    for i in 1..=n {
        let t = i as f32 / n as f32;
        let q = bezier(p, t);
        if capsule(u, v, avant, q, r0 + (r1 - r0) * t) {
            return true;
        }
        avant = q;
    }
    false
}

/// Ce qu'il y a en (u, v), en pixels de forge dans son emprise, tournée vers la gauche : debout ou en marche.
/// `phase` : le pas, en tours ; `None`, arrêtée. `age` : les tics, pour la queue qui ondule.
fn debout(u: f32, v: f32, phase: Option<f32>, age: f32) -> u8 {
    let sol = 7.0;
    let marche = phase.is_some();
    let ph = phase.unwrap_or(0.0);
    let hoche = if marche { (ph * 4.0 * PI).sin() * 0.08 } else { 0.0 };
    // les pattes : (hanche, décalage dans le pas, avant ?, de l'autre côté ?)
    let pattes = [((5.2, 4.1), 0.75, true, true), ((10.9, 3.9), 0.5, false, true), ((4.5, 4.2), 0.25, true, false), ((10.2, 3.9), 0.0, false, false)];
    // 1 : l'épaule ou la cuisse, fondue dans le corps ; 2 : la patte qui en descend
    let patte = |loin: bool| -> u8 {
        for &(hanche, dec, avant, cote) in &pattes {
            if cote != loin {
                continue;
            }
            let a = (ph + dec) * 2.0 * PI;
            let (balance, leve) = if marche { (a.sin() * 0.34, a.cos().max(0.0) * 0.5) } else { (0.0, 0.0) };
            let l = sol - hanche.1;
            let pied = (hanche.0 - balance.sin() * l, sol - 0.25 - leve);
            let (haut, bas) = if avant {
                (ell(u, v, hanche.0, hanche.1 - 0.15, 0.6, 0.8), capsule(u, v, hanche, pied, 0.34))
            } else {
                // la cuisse, le jarret en arrière, puis le canon jusqu'au pied
                let jarret = ((hanche.0 + pied.0) * 0.5 + 0.55, hanche.1 + l * 0.55 - leve * 0.5);
                (ell(u, v, hanche.0 - 0.1, hanche.1, 0.95, 1.1), capsule(u, v, hanche, jarret, 0.55) || capsule(u, v, jarret, pied, 0.36))
            };
            // dans le corps, la patte n'est que du corps : elle ne se cerne qu'en dessous
            if (bas && v > 4.75) || ell(u, v, pied.0 - 0.15, pied.1, 0.48, 0.25) {
                return 2;
            }
            if haut || bas {
                return 1;
            }
        }
        0
    };
    match patte(false) {
        2 => return PATTE,
        1 => return POIL,
        _ => {}
    }
    // la tête, ses oreilles, le museau
    let (tv, tu) = (hoche, 0.0);
    if ell(u, v, 1.15 + tu, 3.05 + tv, 0.95, 0.7) || ell(u, v, 1.6 + tu, 3.45 + tv, 0.8, 0.45) {
        return CREME;
    }
    if ell(u, v, 2.4 + tu, 2.5 + tv, 1.45, 1.3)
        || tri(u, v, (1.7, 1.7 + tv), (2.0, 0.45 + tv), (2.75, 1.4 + tv))
        || tri(u, v, (2.8, 1.4 + tv), (3.35, 0.5 + tv), (3.75, 1.75 + tv))
    {
        return TETE;
    }
    // le cou, le poitrail, le corps, la croupe
    if capsule(u, v, (2.9, 2.9 + tv), (4.4, 3.3), 1.1) || ell(u, v, 4.4, 3.7, 1.7, 1.45) || ell(u, v, 7.5, 3.55, 4.2, 1.4) || ell(u, v, 10.6, 3.5, 1.55, 1.55) {
        return POIL;
    }
    // la queue : elle pend, et son bout remonte ; elle ondule
    let ondule = (age * if marche { 0.5 } else { 0.18 }).sin();
    let q = [(11.7, 3.0), (13.3, 3.1), (13.7 + ondule * 0.15, 6.2), (12.9 + ondule * 0.4, 5.3 - ondule * 0.2)];
    if le_long(u, v, q, 0.42, 0.3) {
        return QUEUE;
    }
    // de l'autre côté, seule la patte dépasse sous le corps : sa cuisse n'ajouterait que des traits
    if patte(true) == 2 {
        return LOIN;
    }
    VIDE
}

/// Roulée en boule sur l'appui (l'emprise de 11 × 8, l'appui en bas) : elle respire.
fn roulee(u: f32, v: f32, age: f32) -> u8 {
    let souffle = 0.1 * (age * 0.35).sin();
    // la tête posée sur les pattes de devant, le museau crème, deux oreilles
    if ell(u, v, 1.0, 6.55, 0.7, 0.5) {
        return CREME;
    }
    if ell(u, v, 2.0, 6.0, 1.35, 1.15) || tri(u, v, (1.1, 5.3), (1.35, 4.05), (2.15, 5.0)) || tri(u, v, (2.2, 4.95), (2.95, 4.0), (3.25, 5.3)) {
        return TETE;
    }
    // la queue ramenée devant, jusque sous le menton
    if le_long(u, v, [(8.9, 7.0), (10.3, 8.1), (5.5, 8.35), (2.6, 7.8)], 0.42, 0.3) {
        return QUEUE;
    }
    // la patte de devant, repliée sous la tête
    if ell(u, v, 3.0, 7.55, 1.1, 0.42) {
        return PATTE;
    }
    // le dos rond, qui respire ; la croupe
    if ell(u, v, 5.6, 6.5 - souffle, 3.3, 1.55 + souffle) || ell(u, v, 8.0, 6.6, 1.6, 1.45) {
        return POIL;
    }
    VIDE
}

/// La peint, au pixel d'art, en pixels d'écran ; `x0` la première colonne de forge montrée. Rend ce qu'elle a
/// couvert.
#[allow(clippy::too_many_arguments)]
pub fn peindre(cat: &Chat, e: Option<&Eclairage>, pixels: &mut [u32], w: usize, h: usize, x0: i32, cw: i32, ch: i32) -> Option<(i32, i32, i32, i32)> {
    let (ex, ey, ew, eh) = emprise(cat);
    let g = GRAIN as f32;
    let (cwf, demi) = (cw as f32, ch as f32 / 2.0);
    let (nx, ny) = ((ew as f32 * cwf / g).ceil() as usize, (eh as f32 * demi / g).ceil() as usize);
    let dort = cat.mode == ModeChat::Roof;
    let droite = !dort && cat.dir.unwrap_or(if cat.mode == ModeChat::Flee { 1 } else { -1 }) > 0;
    let phase = match cat.step {
        Some(s) if s >= 0 => Some(s as f32 / 4.0),
        _ => None,
    };
    let age = cat.age as f32;
    // la silhouette, pixel d'art par pixel d'art (le bas aligné sur le sol)
    let haut = eh as f32 - ny as f32 * g / demi;
    let mut forme = vec![VIDE; nx * ny];
    for ay in 0..ny {
        let v = haut + (ay as f32 + 0.5) * g / demi;
        for ax in 0..nx {
            let mut u = (ax as f32 + 0.5) * g / cwf;
            if droite {
                u = ew as f32 - u;
            }
            forme[ay * nx + ax] = if dort { roulee(u, v, age) } else { debout(u, v, phase, age) };
        }
    }
    let a = |x: i32, y: i32| if x < 0 || y < 0 || x >= nx as i32 || y >= ny as i32 { VIDE } else { forme[y as usize * nx + x as usize] };
    // les couleurs, avant la lumière
    let (or, clair, ombre, loin) = (hex(0xc99a3c), hex(0xe8c272), hex(0x9a6e24), hex(0x9a7428));
    let (creme, creme_ombre) = (hex(0xf2dea6), hex(0xcfb27a));
    let (pale, pale_ombre) = (hex(0xe4c47e), hex(0xb89450)); // le ventre, d'un or pâle
    let (tache, coeur, cerne) = (hex(0x3a2410), hex(0xa87428), hex(0x24140a));
    let mut col: Vec<Option<[f32; 3]>> = vec![None; nx * ny];
    for ay in 0..ny as i32 {
        let v = haut + (ay as f32 + 0.5) * g / demi;
        for ax in 0..nx as i32 {
            let f = a(ax, ay);
            let i = ay as usize * nx + ax as usize;
            if f == VIDE {
                // le contour du pixel art, d'un pixel autour
                if [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| a(ax + dx, ay + dy) != VIDE) {
                    col[i] = Some(cerne);
                }
                continue;
            }
            // les rosettes suivent le corps (en coordonnées de dessin, tournées avec lui)
            let sx = if droite { nx as i32 - 1 - ax } else { ax };
            let (dessus, dessous) = (a(ax, ay - 1) == VIDE, a(ax, ay + 1) == VIDE);
            let ventre = !dort && v > 4.3 && f == POIL && {
                let u = (sx as f32 + 0.5) * g / cwf;
                u > 3.0 && u < 11.5
            };
            let mut c = match f {
                CREME => {
                    if dessous {
                        creme_ombre
                    } else {
                        creme
                    }
                }
                LOIN => loin,
                PATTE if dessous => ombre,
                PATTE => or,
                _ if ventre => {
                    if dessous {
                        pale_ombre
                    } else {
                        pale
                    }
                }
                _ if dessus => clair,
                _ if dessous => ombre,
                _ => or,
            };
            if f != CREME && !ventre && !dessus {
                // des rosettes semées : une par case de 4 × 3 (une sur cinq manque), placée au hasard dans sa case ;
                // sur le corps un anneau ouvert au cœur plus foncé, ailleurs (tête, pattes, queue) un point
                let petite = f == LOIN || f == QUEUE || f == PATTE || (!dort && (sx as f32 + 0.5) * g / cwf < 3.8);
                let (cx, cy) = (sx.div_euclid(4), ay.div_euclid(3));
                for (kx, ky) in [(cx, cy), (cx - 1, cy), (cx, cy - 1), (cx - 1, cy - 1)] {
                    let n = (kx * 7919 + ky * 104729) as i64;
                    if hache(n) < 0.2 {
                        continue;
                    }
                    let (ox, oy) = (kx * 4 + (hache(n + 1) * 4.0) as i32, ky * 3 + (hache(n + 2) * 3.0) as i32);
                    let (dx, dy) = (sx - ox, ay - oy);
                    if petite {
                        if dx == 0 && dy == 0 {
                            c = tache;
                        }
                    } else {
                        // l'anneau : quatre à six pixels autour du cœur, un ou deux qui manquent
                        let trou = (hache(n + 3) * 6.0) as i32;
                        let anneau = [(0, -1), (1, 0), (0, 1), (-1, 0), (1, -1), (-1, 1)];
                        let large = hache(n + 4) > 0.5;
                        for (j, &(ax2, ay2)) in anneau.iter().enumerate().take(if large { 6 } else { 4 }) {
                            if j as i32 != trou && dx == ax2 && dy == ay2 {
                                c = tache;
                            }
                        }
                        if dx == 0 && dy == 0 {
                            c = coeur;
                        }
                    }
                }
            }
            // le trait entre les parties : celle de derrière le prend
            let p = profondeur(f, dort);
            if [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
                let q = a(ax + dx, ay + dy);
                q != VIDE && profondeur(q, dort) < p && !(q == CREME && f == TETE) && !(q == TETE && f == CREME)
            }) {
                c = cerne;
            }
            col[i] = Some(c);
        }
    }
    // l'œil vert et la pupille, le nez, le dedans des oreilles ; roulée, l'œil fermé et le z qui monte
    let pixel = |u: f32, v: f32| -> (i32, i32) {
        let u = if droite { ew as f32 - u } else { u };
        ((u * cwf / g).floor() as i32, ((v - haut) * demi / g).floor() as i32)
    };
    let mut pose = |p: (i32, i32), c: [f32; 3]| {
        if p.0 >= 0 && p.1 >= 0 && (p.0 as usize) < nx && (p.1 as usize) < ny {
            col[p.1 as usize * nx + p.0 as usize] = Some(c);
        }
    };
    let devant = if droite { 1 } else { -1 };
    if dort {
        let o = pixel(1.75, 5.85);
        pose(o, cerne);
        pose((o.0 - devant, o.1), cerne);
        pose(pixel(0.35, 6.45), hex(0x3a1c12));
        if cat.age % 40 < 12 {
            let k = (cat.age % 40) as f32 / 12.0;
            let z = pixel(8.8 + k * 0.6, 3.6 - k * 2.2);
            for (dx, dy) in [(0, 0), (1, 0), (2, 0), (1, 1), (0, 2), (1, 2), (2, 2)] {
                pose((z.0 + dx, z.1 + dy), hex(0xe8eef8));
            }
        }
    } else {
        let hoche = phase.map_or(0.0, |p| (p * 4.0 * PI).sin() * 0.08);
        let o = pixel(1.85, 2.35 + hoche);
        pose(o, hex(0x8ae04a));
        pose((o.0 - devant, o.1), hex(0x101010));
        pose((o.0, o.1 - 1), cerne);
        pose((o.0 - devant, o.1 - 1), cerne);
        pose(pixel(0.3, 2.9 + hoche), hex(0x3a1c12));
        pose(pixel(2.15, 1.25 + hoche), hex(0xd08a6a));
        pose(pixel(3.3, 1.3 + hoche), hex(0xd08a6a));
    }
    // posée en pixels d'écran sous la lumière de la forge, lue au pixel de forge
    let gauche = (ex - x0) * cw;
    let dessus = (ey as f32 * demi + haut * demi).round() as i32;
    let g = GRAIN as i32;
    let mut couvert: Option<(i32, i32, i32, i32)> = None;
    for ay in 0..ny {
        for ax in 0..nx {
            let Some(c) = col[ay * nx + ax] else { continue };
            let (sx, sy) = (gauche + ax as i32 * g, dessus + ay as i32 * g);
            let k = e.map_or([1.0; 3], |e| {
                let fx = (ex as f32 + (ax as f32 + 0.5) * g as f32 / cwf).floor() as i32 - e.x0;
                let fy = (ey as f32 + haut + (ay as f32 + 0.5) * g as f32 / demi).floor() as i32 - e.y0;
                e.k[(fy.clamp(0, e.h - 1) * e.w + fx.clamp(0, e.w - 1)) as usize]
            });
            let p = 0xFF00_0000 | ((c[0] * k[0]).clamp(0.0, 255.0) as u32) << 16 | ((c[1] * k[1]).clamp(0.0, 255.0) as u32) << 8 | (c[2] * k[2]).clamp(0.0, 255.0) as u32;
            let (x1, y1) = ((sx + g).min(w as i32), (sy + g).min(h as i32));
            for y in sy.max(0)..y1 {
                for x in sx.max(0)..x1 {
                    pixels[y as usize * w + x as usize] = p;
                }
            }
            if sx.max(0) < x1 && sy.max(0) < y1 {
                couvert = Some(couvert.map_or((sx.max(0), sy.max(0), x1, y1), |z| (z.0.min(sx.max(0)), z.1.min(sy.max(0)), z.2.max(x1), z.3.max(y1))));
            }
        }
    }
    couvert
}
