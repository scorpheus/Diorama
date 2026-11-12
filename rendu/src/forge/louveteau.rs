//! Les louveteaux (les aides, un par sous-agent), dessinés à la main en lettres à la manière des habitants
//! (Dwarf Fortress : de face, trapus, contour sombre, peu de couleurs) : le louveteau à lunettes de l'agent Fable,
//! choisi par l'user le 12-11 parmi trois variantes, après le refus des formes calculées (« uncanny valley », « on
//! dirait des petits chats »). Ils remplacent le dessin en lettres d'atelier.rs quand les personnages se peignent
//! au pixel (`pet`) : seize lignes de seize lettres par image, le foulard aux couleurs du sous-agent ; ils marchent,
//! clignent, s'assoient, et ceux du marteau frappent l'enclume de leur coin de cour (`peindre_coin` : l'enclume sur
//! son billot, le brasero).

use super::paysage::GRAIN;
use super::pet::Eclairage;

/// Un louveteau tel qu'atelier.rs l'a placé.
#[derive(Clone, Debug)]
pub struct Vu {
    /// le coin haut gauche du dessin en lettres (12 × 7), en pixels de forge
    pub x: i32,
    pub y: i32,
    pub dir: i32,
    /// le trot, en tours (continu) ; `None` arrêté
    pub phase: Option<f32>,
    pub assis: bool,
    pub marteau: bool,
    pub age: i64,
    pub seed: i64,
    /// sa lettre de couleur (accessoires::PALETTE)
    pub couleur: u8,
}

/// Où il se tient, en pixels de forge : deux de marge de chaque côté (la queue, l'enclume), trois au-dessus
/// (les oreilles, le marteau levé).
pub fn emprise(c: &Vu) -> (i32, i32, i32, i32) {
    (c.x - 2, c.y - 3, 16, 10)
}

fn hex(c: u32) -> [f32; 3] {
    [((c >> 16) & 255) as f32, ((c >> 8) & 255) as f32, (c & 255) as f32]
}

fn fois(a: [f32; 3], k: f32) -> [f32; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
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

/// Pose une image de pixels d'art en pixels d'écran sous la lumière de la forge (lue au pixel de forge) ; ce qui est
/// dans `propres` luit de lui-même. Rend ce qui a été couvert.
#[allow(clippy::too_many_arguments)]
fn poser(col: &[Option<[f32; 3]>], nx: usize, ny: usize, emprise: (i32, i32, i32, i32), haut: f32, propres: &[(i32, i32)], e: Option<&Eclairage>, pixels: &mut [u32], w: usize, h: usize, x0: i32, cw: i32, ch: i32) -> Option<(i32, i32, i32, i32)> {
    let (ex, ey, _, _) = emprise;
    let g = GRAIN as f32;
    let (cwf, demi) = (cw as f32, ch as f32 / 2.0);
    let gauche = (ex - x0) * cw;
    let dessus = (ey as f32 * demi + haut * demi).round() as i32;
    let gi = GRAIN as i32;
    let mut couvert: Option<(i32, i32, i32, i32)> = None;
    for ay in 0..ny {
        for ax in 0..nx {
            let Some(k0) = col[ay * nx + ax] else { continue };
            let (sx, sy) = (gauche + ax as i32 * gi, dessus + ay as i32 * gi);
            let k = if propres.contains(&(ax as i32, ay as i32)) {
                [1.0; 3]
            } else {
                e.map_or([1.0; 3], |e| {
                    let fx = (ex as f32 + (ax as f32 + 0.5) * g / cwf).floor() as i32 - e.x0;
                    let fy = (ey as f32 + haut + (ay as f32 + 0.5) * g / demi).floor() as i32 - e.y0;
                    e.k[(fy.clamp(0, e.h - 1) * e.w + fx.clamp(0, e.w - 1)) as usize]
                })
            };
            let p = 0xFF00_0000 | ((k0[0] * k[0]).clamp(0.0, 255.0) as u32) << 16 | ((k0[1] * k[1]).clamp(0.0, 255.0) as u32) << 8 | (k0[2] * k[2]).clamp(0.0, 255.0) as u32;
            let (x1, y1) = ((sx + gi).min(w as i32), (sy + gi).min(h as i32));
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

/// Le peint, au pixel d'art, en pixels d'écran ; `x0` la première colonne de forge montrée. Rend ce qu'il a
/// couvert.
#[allow(clippy::too_many_arguments)]
pub fn peindre(c: &Vu, e: Option<&Eclairage>, pixels: &mut [u32], w: usize, h: usize, x0: i32, cw: i32, ch: i32) -> Option<(i32, i32, i32, i32)> {
    peindre_lettres(c, DESSIN, e, pixels, w, h, x0, cw, ch)
}

/// Un louveteau dessiné à la main en lettres, à la manière des habitants (Dwarf Fortress : de face, trapu, contour
/// sombre, peu de couleurs) : seize lignes de seize lettres par image, la queue à gauche, le marteau à droite.
pub struct Dessin {
    /// (lettre, couleur) ; `O` et `o`, le foulard, prennent la couleur du sous-agent
    pub palette: &'static [(u8, u32)],
    pub debout: [&'static str; 16],
    pub cligne: [&'static str; 16],
    pub marche: [[&'static str; 16]; 2],
    pub assis: [&'static str; 16],
    pub marteau: [[&'static str; 16]; 2],
}

/// Le louveteau à lunettes (« le fils de son père ») dessiné par l'agent Fable : blanc, crête bleue, lunettes de laiton
/// relevées sur le front, foulard aux couleurs du sous-agent.
pub const LOUVETEAU_LUNETTES: Dessin = Dessin {
    palette: &[(b'k', 0x1b1a24), (b'W', 0xf4f2fb), (b'l', 0xc5c0e2), (b'v', 0x9a94c4), (b'V', 0x5d5890), (b'b', 0x3f74e0), (b'B', 0x24408f), (b'g', 0x4c4858), (b'm', 0x141018), (b'y', 0xe0b040), (b'Y', 0x9a6a1c), (b'e', 0x58c8ff), (b't', 0x7a4a24), (b'A', 0x9aa0b0), (b'H', 0xd8dce6)],
    debout: [
        "................",
        "....k......k....",
        "...kWk.k..kWk...",
        "..kWvWkbkkWvWk..",
        "..kWvWbbbbWvWk..",
        "..kYyeeyyeeyYk..",
        ".kllWmWWWWmWllk.",
        "kllllWWWWWWllllk",
        ".kkllWWWWWWllkk.",
        "kbbkkWWWWWWkk...",
        "kbbkOkWmmWkOk...",
        "kbWWkOOOOOOOk...",
        "kWWWkOOOOOOgk...",
        "klWWkWOooOWgk...",
        "klWkWWk..kWWk...",
        ".kkgggk..kgggk..",
    ],
    cligne: [
        "................",
        "....k......k....",
        "...kWk.k..kWk...",
        "..kWvWkbkkWvWk..",
        "..kWvWbbbbWvWk..",
        "..kYyeeyyeeyYk..",
        ".kllvvWWWWvvllk.",
        "kllllWWWWWWllllk",
        ".kkllWWWWWWllkk.",
        "kbbkkWWWWWWkk...",
        "kbbkOkWmmWkOk...",
        "kbWWkOOOOOOOk...",
        "kWWWkOOOOOOgk...",
        "klWWkWOooOWgk...",
        "klWkWWk..kWWk...",
        ".kkgggk..kgggk..",
    ],
    marche: [[
            "....k......k....",
            "...kWk.k..kWk...",
            "..kWvWkbkkWvWk..",
            "..kWvWbbbbWvWk..",
            "..kYyeeyyeeyYk..",
            ".kllWmWWWWmWllk.",
            "kllllWWWWWWllllk",
            ".kkllWWWWWWllkk.",
            "kbbkkWWWWWWkk...",
            "kbbkOkWmmWkOk...",
            "kbWWkOOOOOOOk...",
            "kWWWkOOOOOOgk...",
            "klWWkWOooOWgk...",
            "klWkWWk..kWWk...",
            ".kkgggk..kWWk...",
            ".........kgggk..",
        ], [
            "................",
            "....k......k....",
            "...kWk.k..kWk...",
            "..kWvWkbkkWvWk..",
            "..kWvWbbbbWvWk..",
            "..kYyeeyyeeyYk..",
            ".kllWmWWWWmWllk.",
            "kllllWWWWWWllllk",
            ".kkllWWWWWWllkk.",
            "...kkWWWWWWkk...",
            ".kkkOkWmmWkOk...",
            "kWWkOOOOOOOOk...",
            "kWWWkOOOOOOgk...",
            "klWWkWOooOWgk...",
            "kbbkWWk..kgggk..",
            ".kkgggk.........",
        ]],
    assis: [
        "................",
        "................",
        "....k......k....",
        "...kWk.k..kWk...",
        "..kWvWkbkkWvWk..",
        "..kWvWbbbbWvWk..",
        "..kYyeeyyeeyYk..",
        ".kllWmWWWWmWllk.",
        "kllllWWWWWWllllk",
        ".kkllWWWWWWllkk.",
        "kbbkkWWWWWWkk...",
        "kbbWkkWmmWkOk...",
        "kWWWkOOOOOOgk...",
        "klWWkWOOOOWgk...",
        ".kkgggWooWgggk..",
        "..kgggkkkkgggk..",
    ],
    marteau: [[
            "................",
            "................",
            "....k......k.AHA",
            "...kWk.k..kWkAAA",
            "..kWvWkbkkWvWkt.",
            "..kWvWbbbbWvWkt.",
            "..kYyeeyyeeyYkt.",
            ".kllWmWWWWmWllgk",
            "kllllWWWWWWlllgk",
            ".kkllWWWWWWllkgk",
            "kbbkkWWWWWWkkgk.",
            "kbbWkkWmmWkOgk..",
            "kWWWkOOOOOOOk...",
            "klWWkWOOOOWWk...",
            ".kkgggWooWgggk..",
            "..kgggkkkkgggk..",
        ], [
            "................",
            "................",
            "................",
            "....k......k....",
            "...kWk.k..kWk...",
            "..kWvWkbkkWvWk..",
            "..kWvWbbbbWvWk..",
            "..kYyeeyyeeyYk..",
            ".kllWmWWWWmWllk.",
            "kllllWWWWWWllllk",
            "kkkllWWWWWWllkk.",
            "kbbkkWWWWWWkk...",
            "kWWWkkWmmWkggk..",
            "klWWkOOOOOOWkgt.",
            ".kkgggOooOgggkAH",
            "..kgggkkkkgggkAA",
        ]],
};

/// Le dessin de la forge.
const DESSIN: &Dessin = &LOUVETEAU_LUNETTES;

/// La hauteur d'un louveteau en lettres, en demi-rangées de forge.
const HAUT_LETTRES: f32 = 5.0;

/// Le peint d'après son dessin en lettres, chaque lettre un carré de pixels d'écran entier, retourné s'il va à gauche,
/// sous la lumière de la forge.
#[allow(clippy::too_many_arguments)]
fn peindre_lettres(c: &Vu, d: &Dessin, e: Option<&Eclairage>, pixels: &mut [u32], w: usize, h: usize, x0: i32, cw: i32, ch: i32) -> Option<(i32, i32, i32, i32)> {
    let image: &[&str; 16] = if c.assis && c.marteau {
        &d.marteau[usize::from(c.age.rem_euclid(4) == 2)]
    } else if c.assis {
        &d.assis
    } else if let Some(p) = c.phase {
        &d.marche[((p * 2.0).floor() as i64).rem_euclid(2) as usize]
    } else if (c.age + c.seed * 7).rem_euclid(41) < 2 {
        &d.cligne
    } else {
        &d.debout
    };
    let (cwf, demi) = (cw as f32, ch as f32 / 2.0);
    let k = ((HAUT_LETTRES * demi / 16.0).round() as i32).max(1);
    let (pied, sol) = ((((c.x - x0) as f32 + 6.0) * cwf).round() as i32, ((c.y as f32 + 7.0) * demi).round() as i32);
    let (gauche, haut) = (pied - 8 * k, sol - 16 * k);
    let couleur = super::accessoires::PALETTE.get(c.couleur as usize).copied().flatten().map_or(hex(0xc04040), hex);
    let mut couvert: Option<(i32, i32, i32, i32)> = None;
    for (j, ligne) in image.iter().enumerate() {
        let ligne = ligne.as_bytes();
        for i in 0..16usize {
            let lettre = if c.dir < 0 { ligne.get(15 - i) } else { ligne.get(i) }.copied().unwrap_or(b'.');
            let col = match lettre {
                b'O' => couleur,
                b'o' => fois(couleur, 0.62),
                _ => match d.palette.iter().find(|(l, _)| *l == lettre) {
                    Some(&(_, c)) => hex(c),
                    None => continue,
                },
            };
            let (sx, sy) = (gauche + i as i32 * k, haut + j as i32 * k);
            let l = e.map_or([1.0; 3], |e| {
                let fx = ((sx + k / 2) as f32 / cwf).floor() as i32 + x0 - e.x0;
                let fy = ((sy + k / 2) as f32 / demi).floor() as i32 - e.y0;
                e.k[(fy.clamp(0, e.h - 1) * e.w + fx.clamp(0, e.w - 1)) as usize]
            });
            let p = 0xFF00_0000 | ((col[0] * l[0]).clamp(0.0, 255.0) as u32) << 16 | ((col[1] * l[1]).clamp(0.0, 255.0) as u32) << 8 | (col[2] * l[2]).clamp(0.0, 255.0) as u32;
            let (xa, xb, ya, yb) = (sx.max(0), (sx + k).min(w as i32), sy.max(0), (sy + k).min(h as i32));
            for y in ya..yb {
                for x in xa..xb {
                    pixels[y as usize * w + x as usize] = p;
                }
            }
            if xa < xb && ya < yb {
                couvert = Some(couvert.map_or((xa, ya, xb, yb), |z| (z.0.min(xa), z.1.min(ya), z.2.max(xb), z.3.max(yb))));
            }
        }
    }
    couvert
}

/// Le coin des louveteaux dans la cour : l'enclume sur son billot, que deux louveteaux frappent face à face, et le
/// brasero sur son trépied qui rougeoie à côté (ses braises, ses flammes, des étincelles qui montent). `x` : le
/// milieu de l'enclume, en pixels de forge ; le sol est le bas de la scène.
pub fn emprise_coin(x: i32, sol: i32) -> (i32, i32, i32, i32) {
    (x - 3, sol - 6, 19, 6)
}

#[allow(clippy::too_many_arguments)]
pub fn peindre_coin(x: i32, sol: i32, temps: i64, e: Option<&Eclairage>, pixels: &mut [u32], w: usize, h: usize, x0: i32, cw: i32, ch: i32) -> Option<(i32, i32, i32, i32)> {
    let em = emprise_coin(x, sol);
    let g = GRAIN as f32;
    let (cwf, demi) = (cw as f32, ch as f32 / 2.0);
    let (nx, ny) = ((em.2 as f32 * cwf / g).ceil() as usize, (em.3 as f32 * demi / g).ceil() as usize);
    let haut = em.3 as f32 - ny as f32 * g / demi;
    let t = temps as f32;
    // (u, v) en pixels de forge depuis le coin haut gauche de l'emprise ; l'enclume au milieu de u = 3
    const A: f32 = 3.0;
    const B: f32 = 15.0; // le brasero
    let quoi = |u: f32, v: f32| -> u8 {
        // l'enclume : la table, la bigorne, la taille, le pied, sur un billot de chêne
        let (tu, tv) = (u - A, v - 6.0 + 1.0);
        if tu > -1.4 && tu < 0.4 && tv > -0.15 && tv < 0.3 || tri(tu, tv, (0.4, -0.15), (1.5, -0.1), (0.4, 0.25)) {
            return 1;
        }
        if tu > -0.95 && tu < -0.05 && tv >= 0.3 && tv < 0.55 || tu > -1.2 && tu < 0.2 && tv >= 0.55 && tv < 0.75 {
            return 2;
        }
        if tu > -1.5 && tu < 0.5 && tv >= 0.75 && tv < 1.0 {
            return 3; // le billot
        }
        // le brasero : trois pieds, la vasque de fer, les braises, les flammes
        let (bu, bv) = (u - B, v - 6.0);
        if capsule(bu, bv, (-0.6, -1.4), (-1.1, 0.0), 0.16) || capsule(bu, bv, (0.6, -1.4), (1.1, 0.0), 0.16) || capsule(bu, bv, (0.0, -1.4), (0.05, -0.05), 0.14) {
            return 4;
        }
        if ell(bu, bv, 0.0, -1.75, 1.25, 0.5) && bv > -1.95 {
            return 5;
        }
        if ell(bu, bv, 0.0, -2.0, 1.05, 0.28) {
            return 6;
        }
        // les flammes : trois langues qui dansent
        for k in 0..3 {
            let kx = (k as f32 - 1.0) * 0.55;
            let haute = 1.1 + 0.5 * ((t * 0.9 + k as f32 * 2.1).sin() * 0.5 + 0.5) - (k as f32 - 1.0).abs() * 0.35;
            let pointe = (kx + 0.2 * (t * 1.3 + k as f32).sin(), -2.0 - haute);
            if tri(bu, bv, (kx - 0.42, -2.0), (kx + 0.42, -2.0), pointe) {
                let f = (-(bv + 2.0) / haute).clamp(0.0, 1.0);
                return if f < 0.35 { 7 } else if f < 0.7 { 8 } else { 9 };
            }
        }
        0
    };
    let mut forme = vec![0u8; nx * ny];
    for ay in 0..ny {
        for ax in 0..nx {
            let (u, v) = ((ax as f32 + 0.5) * g / cwf, haut + (ay as f32 + 0.5) * g / demi);
            forme[ay * nx + ax] = quoi(u, v);
        }
    }
    let a = |x: i32, y: i32| if x < 0 || y < 0 || x >= nx as i32 || y >= ny as i32 { 0 } else { forme[y as usize * nx + x as usize] };
    let cerne = hex(0x1c1c2a);
    let mut col: Vec<Option<[f32; 3]>> = vec![None; nx * ny];
    let mut propres: Vec<(i32, i32)> = Vec::new();
    for ay in 0..ny as i32 {
        for ax in 0..nx as i32 {
            let f = a(ax, ay);
            let i = ay as usize * nx + ax as usize;
            let lueur = f >= 6;
            if f == 0 {
                // le cerne, sauf autour des flammes
                if [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| matches!(a(ax + dx, ay + dy), 1..=5)) {
                    col[i] = Some(cerne);
                }
                continue;
            }
            let dessus = a(ax, ay - 1) != f;
            let k = match f {
                1 => {
                    if dessus {
                        hex(0xc8d0e0)
                    } else {
                        hex(0x6a7286)
                    }
                }
                2 => hex(0x4a5062),
                3 => {
                    if dessus {
                        hex(0xb08050)
                    } else if (ax / 2) % 3 == 0 {
                        hex(0x5a3a20)
                    } else {
                        hex(0x7a5030)
                    }
                }
                4 => hex(0x3a3a46),
                5 => {
                    if dessus {
                        hex(0x6a6a78)
                    } else {
                        hex(0x34343e)
                    }
                }
                6 => {
                    if hache(ax as i64 * 13 + ay as i64 * 7 + temps / 3) > 0.5 {
                        hex(0xffb040)
                    } else {
                        hex(0xd85020)
                    }
                }
                7 => hex(0xff7a2a),
                8 => hex(0xffb84a),
                _ => hex(0xfff0a0),
            };
            col[i] = Some(k);
            if lueur {
                propres.push((ax, ay));
            }
        }
    }
    // des étincelles qui montent du brasero
    for j in 0..4i64 {
        let f = ((t * 0.07 + hache(j * 17) ) % 1.0 + 1.0) % 1.0;
        let u = B + (hache(j * 29 + (t * 0.07 + hache(j * 17)).floor() as i64) - 0.5) * 1.6 + (f * 6.0 + j as f32).sin() * 0.3;
        let v = 6.0 - 3.2 - f * 2.6;
        if v < haut {
            continue;
        }
        let p = ((u * cwf / g) as i32, ((v - haut) * demi / g) as i32);
        if p.0 >= 0 && p.1 >= 0 && (p.0 as usize) < nx && (p.1 as usize) < ny {
            col[p.1 as usize * nx + p.0 as usize] = Some(if f < 0.5 { hex(0xffd070) } else { hex(0xff8a3a) });
            propres.push(p);
        }
    }
    let z = poser(&col, nx, ny, em, haut, &propres, e, pixels, w, h, x0, cw, ch);
    // la lueur du brasero sur ce qui l'entoure : un halo chaud qui respire
    let (bx, by) = (((em.0 - x0) as f32 + B) * cwf, (em.1 as f32 + 3.6) * demi);
    let r = 3.2 * cwf;
    let souffle = 0.8 + 0.2 * (t * 0.5).sin();
    let (xa, xb) = (((bx - r) as i32).max(0), ((bx + r) as i32).min(w as i32));
    let (ya, yb) = (((by - r) as i32).max(0), ((by + r) as i32).min(h as i32));
    for y in ya..yb {
        for x in xa..xb {
            let d = ((x as f32 - bx).hypot(y as f32 - by) / r).min(1.0);
            let k = (1.0 - d) * (1.0 - d) * 0.22 * souffle;
            if k <= 0.0 {
                continue;
            }
            let i = y as usize * w + x as usize;
            let p = pixels[i];
            let c = [((p >> 16) & 255) as f32, ((p >> 8) & 255) as f32, (p & 255) as f32];
            let n = [c[0] + (255.0 - c[0]) * k, c[1] + (170.0 - c[1]).max(0.0) * k * 0.8, c[2] * (1.0 - k * 0.3)];
            pixels[i] = 0xFF00_0000 | (n[0].clamp(0.0, 255.0) as u32) << 16 | (n[1].clamp(0.0, 255.0) as u32) << 8 | n[2].clamp(0.0, 255.0) as u32;
        }
    }
    let halo = (xa, ya, xb, yb);
    Some(z.map_or(halo, |z| (z.0.min(halo.0), z.1.min(halo.1), z.2.max(halo.2), z.3.max(halo.3))))
}
