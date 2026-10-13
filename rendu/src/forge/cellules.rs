//! Les cellules en vrais pixels : ce que le terminal faisait de chaque caractère (demi-blocs,
//! quarts, points braille, traits de boîte, huitièmes de bloc), peint par l'appli dans une
//! cellule de `cw` × `ch` pixels.

use super::pixels::DEFAUT;

/// Une couleur de cellule en pixel : opaque, ou transparente pour le fond du terminal.
#[inline]
fn px(c: u32) -> u32 {
    if c == DEFAUT {
        0
    } else {
        0xFF00_0000 | (c & 0x00FF_FFFF)
    }
}

/// Où peindre : des pixels 0xAARRGGBB, ligne après ligne, `w` de large.
pub struct Cible<'a> {
    pub pixels: &'a mut [u32],
    pub w: usize,
    pub h: usize,
}

impl Cible<'_> {
    #[inline]
    fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: u32) {
        let (x0, x1) = (x0.max(0) as usize, (x1.max(0) as usize).min(self.w));
        let (y0, y1) = (y0.max(0) as usize, (y1.max(0) as usize).min(self.h));
        if x0 >= x1 {
            return;
        }
        for y in y0..y1 {
            self.pixels[y * self.w + x0..y * self.w + x1].fill(c);
        }
    }
}

/// Peint la cellule `(ch, encre, fond)` dans le rectangle (x, y, cw, hh) de la cible.
#[allow(clippy::too_many_arguments)]
pub fn cellule(cible: &mut Cible, x: i32, y: i32, cw: i32, hh: i32, car: u32, encre: u32, fond: u32) {
    let (fg, bg) = (px(encre), px(fond));
    let mid = hh / 2;
    let (xm, x1, y1) = (x + cw / 2, x + cw, y + hh);
    cible.rect(x, y, x1, y1, bg);
    match car {
        0x20 => {}
        0x2580 => cible.rect(x, y, x1, y + mid, fg), // ▀
        0x2584 => cible.rect(x, y + mid, x1, y1, fg), // ▄
        0x2588 => cible.rect(x, y, x1, y1, fg),       // █
        0x258C => cible.rect(x, y, xm, y1, fg),       // ▌
        0x2590 => cible.rect(xm, y, x1, y1, fg),      // ▐
        // quarts de bloc : ▖▗▘▙▚▛▜▝▞▟
        0x2596..=0x259F => {
            let bits: u32 = match car {
                0x2596 => 4,
                0x2597 => 8,
                0x2598 => 1,
                0x2599 => 1 | 4 | 8,
                0x259A => 1 | 8,
                0x259B => 1 | 2 | 4,
                0x259C => 1 | 2 | 8,
                0x259D => 2,
                0x259E => 2 | 4,
                _ => 2 | 4 | 8, // ▟
            };
            for q in 0..4 {
                if bits & (1 << q) != 0 {
                    let (qx0, qx1) = if q & 1 == 0 { (x, xm) } else { (xm, x1) };
                    let (qy0, qy1) = if q & 2 == 0 { (y, y + mid) } else { (y + mid, y1) };
                    cible.rect(qx0, qy0, qx1, qy1, fg);
                }
            }
        }
        // huitièmes de bloc par le bas : ▁▂▃▄▅▆▇
        0x2581..=0x2587 => {
            let n = (car - 0x2580) as i32;
            cible.rect(x, y1 - (hh * n + 4) / 8, x1, y1, fg);
        }
        // points braille : deux colonnes de quatre
        0x2800..=0x28FF => {
            const POINTS: [(i32, i32); 8] = [(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2), (0, 3), (1, 3)];
            let bits = car - 0x2800;
            let (dw, dh) = ((cw / 4).max(1), (hh / 8).max(1));
            for (k, &(i, j)) in POINTS.iter().enumerate() {
                if bits & (1 << k) != 0 {
                    let cx = x + (cw * (2 * i + 1)) / 4;
                    let cy = y + (hh * (2 * j + 1)) / 8;
                    cible.rect(cx - dw / 2, cy - dh / 2, cx - dw / 2 + dw, cy - dh / 2 + dh, fg);
                }
            }
        }
        _ => {
            let fin = (cw / 9).max(1); // un trait fin
            let gras = (cw / 4).max(2); // un trait gras
            let vertical = |cible: &mut Cible, ya: i32, yb: i32, e: i32| cible.rect(xm - e / 2, ya, xm - e / 2 + e, yb, fg);
            let horizontal = |cible: &mut Cible, xa: i32, xb: i32, e: i32| cible.rect(xa, y + mid - e / 2, xb, y + mid - e / 2 + e, fg);
            match car {
                0x2502 => vertical(cible, y, y1, fin),  // │
                0x2503 => vertical(cible, y, y1, gras), // ┃
                0x257D => {
                    // ╽ : fin en haut, gras en bas
                    vertical(cible, y, y + mid, fin);
                    vertical(cible, y + mid, y1, gras);
                }
                0x2541 => {
                    // ╁ : gras en bas, fin en haut et en travers
                    vertical(cible, y, y + mid, fin);
                    vertical(cible, y + mid, y1, gras);
                    horizontal(cible, x, x1, fin);
                }
                0x2574 => horizontal(cible, x, xm, fin),  // ╴
                0x2576 => horizontal(cible, xm, x1, fin), // ╶
                0x2571 | 0x2572 => {
                    // ╱ ╲ : une diagonale d'un coin à l'autre
                    for r in 0..hh {
                        let t = (2 * r + 1) as f32 / (2 * hh) as f32;
                        let f = if car == 0x2571 { 1.0 - t } else { t };
                        let cx = x + (f * cw as f32) as i32;
                        cible.rect(cx - fin, y + r, cx + fin, y + r + 1, fg);
                    }
                }
                0x5A | 0x7A => {
                    // Z, z : la barre du haut, la diagonale, la barre du bas
                    let haut = if car == 0x5A { y + hh / 5 } else { y + hh * 2 / 5 };
                    let bas = y + hh * 4 / 5;
                    let (g, d) = (x + cw / 5, x1 - cw / 5);
                    cible.rect(g, haut, d, haut + fin.max(2), fg);
                    cible.rect(g, bas - fin.max(2), d, bas, fg);
                    for r in haut..bas {
                        let f = (r - haut) as f32 / (bas - haut).max(1) as f32;
                        let cx = d - (f * (d - g) as f32) as i32;
                        cible.rect(cx - 1, r, cx + 1, r + 1, fg);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Peint les cellules `cells` (trois mots chacune, `cols` par rangée) à partir de (x, y) ;
/// seulement celles qui diffèrent de `avant` s'il est donné. Rend la zone peinte, en pixels
/// (x0, y0, x1, y1), ou rien si rien n'a changé.
///
/// Une seule zone, même quand ce qui change est semé sur tout le bandeau (le feu à gauche, la
/// roue à droite, la lueur du foyer partout) : au banc, un appel de plus au compositeur coûte
/// 0,08 ms, autant que 96 000 pixels remis (l'image entière d'un bandeau de 144 × 16 cellules
/// en vaut 0,32 ms) ; découpée en tuiles, la forge coûtait 0,72 % d'un cœur au lieu de 0,44.
pub fn peindre(
    cible: &mut Cible,
    cells: &[u32],
    avant: Option<&[u32]>,
    cols: usize,
    x: i32,
    y: i32,
    cw: i32,
    ch: i32,
) -> Option<(i32, i32, i32, i32)> {
    let mut zone: Option<(i32, i32, i32, i32)> = None;
    for (n, c) in cells.chunks_exact(3).enumerate() {
        if avant.is_some_and(|a| a.get(3 * n..3 * n + 3) == Some(c)) {
            continue;
        }
        let (cx, cy) = (x + (n % cols) as i32 * cw, y + (n / cols) as i32 * ch);
        cellule(cible, cx, cy, cw, ch, c[0], c[1], c[2]);
        zone = Some(match zone {
            None => (cx, cy, cx + cw, cy + ch),
            Some((a, b, c2, d)) => (a.min(cx), b.min(cy), c2.max(cx + cw), d.max(cy + ch)),
        });
    }
    zone
}
