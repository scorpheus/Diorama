//! Dessiner sur une grille de lettres, une par pixel ('.' transparent), et la changer en
//! cellules de terminal : des demi-blocs, deux pixels l'un sur l'autre par cellule.
//!
//! Le pendant exact de hooks/pixels.ts : les mêmes cellules, au mot près.

use std::collections::HashMap;

/// Math.round de JavaScript : une moitié va vers +∞ (−2,5 → −2), pas loin de zéro.
pub fn arrondi(v: f64) -> f64 {
    let f = v.floor();
    if v - f >= 0.5 {
        f + 1.0
    } else {
        f
    }
}

pub fn arr(v: f64) -> i32 {
    arrondi(v) as i32
}

/// Un hasard fixe dans [0, 1) pour une graine entière (rand de pixels.ts).
pub fn hasard(n: i64) -> f64 {
    let m = ((n as i32) ^ 0x5bd1e995u32 as i32).wrapping_mul(0x27d4eb2d) as u32;
    (m % 10007) as f64 / 10007.0
}

/// Ce qui se pose sur une cellule, plus fin que le demi-bloc.
#[derive(Clone, Copy, PartialEq)]
pub enum Sorte {
    Points,
    Quarts,
    Glyphe,
}

/// Un glyphe posé sur une cellule : points braille, quarts de bloc, ou un glyphe à lui. Il ne
/// se montre que tant que la cellule garde les deux pixels sur lesquels il a été posé (`sous`).
#[derive(Clone, Copy)]
pub struct Sur {
    pub sorte: Sorte,
    pub bits: u32,
    pub c: u8,
    pub fond: u8,
    pub sous: [u8; 2],
}

pub struct Grille {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
    pub sur: HashMap<i32, Sur>,
}

const fn cellule(cx: i32, cy: i32) -> i32 {
    cy * 4096 + cx
}

impl Grille {
    pub fn new(w: i32, h: i32) -> Self {
        Grille { w, h, px: vec![b'.'; (w.max(0) * h.max(0)) as usize], sur: HashMap::new() }
    }

    /// La lettre en (x, y), '.' hors de la grille.
    #[inline]
    pub fn at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            b'.'
        } else {
            self.px[(y * self.w + x) as usize]
        }
    }

    #[inline]
    pub fn put(&mut self, x: i32, y: i32, ch: u8) {
        if ch != b'.' && x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = ch;
        }
    }

    #[inline]
    pub fn erase(&mut self, x: i32, y: i32) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = b'.';
        }
    }

    pub fn stamp(&mut self, x: i32, y: i32, rows: &[&str]) {
        for (j, row) in rows.iter().enumerate() {
            for (i, ch) in row.bytes().enumerate() {
                self.put(x + i as i32, y + j as i32, ch);
            }
        }
    }

    /// Comme `stamp`, pour des lignes faites à la volée.
    pub fn stamp_s(&mut self, x: i32, y: i32, rows: &[String]) {
        for (j, row) in rows.iter().enumerate() {
            for (i, ch) in row.bytes().enumerate() {
                self.put(x + i as i32, y + j as i32, ch);
            }
        }
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, ch: u8) {
        self.line_f(x0, y0, x1, y1, |_| ch)
    }

    pub fn line_f(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, ch: impl Fn(i32) -> u8) {
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let (mut err, mut x, mut y) = (dx + dy, x0, y0);
        let mut i = 0;
        loop {
            self.put(x, y, ch(i));
            if x == x1 && y == y1 {
                return;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
            i += 1;
        }
    }

    fn sous(&self, cx: i32, cy: i32) -> [u8; 2] {
        [self.at(cx, 2 * cy), self.at(cx, 2 * cy + 1)]
    }

    fn lay(&mut self, cx: i32, cy: i32, sorte: Sorte, bits: u32, c: u8, sur: Option<&[u8]>) {
        if cx < 0 || cy < 0 || 2 * cy + 1 >= self.h || cx >= self.w {
            return;
        }
        let sous = self.sous(cx, cy);
        let uni = sous[0] == sous[1];
        // des points ne se posent que sur un seul fond
        if sorte != Sorte::Glyphe && (!uni || sur.is_some_and(|s| !s.contains(&sous[0]))) {
            return;
        }
        let k = cellule(cx, cy);
        let fusion = match self.sur.get(&k) {
            Some(était) if sorte != Sorte::Glyphe && était.sorte == sorte && était.sous == sous => était.bits | bits,
            _ => bits,
        };
        self.sur.insert(k, Sur { sorte, bits: fusion, c, fond: if uni { sous[0] } else { b'.' }, sous });
    }

    /// Un point braille en (fx, fy) : deux par pixel en largeur, deux en hauteur ; seulement
    /// sur une cellule d'une couleur, parmi `sur` si donné.
    pub fn dot(&mut self, fx: f64, fy: f64, c: u8, sur: Option<&[u8]>) {
        let (x, y) = (arr(fx), arr(fy));
        if x < 0 || y < 0 {
            return;
        }
        self.lay(x >> 1, y >> 2, Sorte::Points, BRAILLE[(x & 1) as usize][(y & 3) as usize], c, sur);
    }

    /// Un quart de bloc en (qx, qy) : deux par pixel en largeur, un en hauteur.
    pub fn quad(&mut self, qx: f64, qy: f64, c: u8, sur: Option<&[u8]>) {
        let (x, y) = (arr(qx), arr(qy));
        if x < 0 || y < 0 {
            return;
        }
        self.lay(x >> 1, y >> 1, Sorte::Quarts, 1 << ((y & 1) * 2 + (x & 1)), c, sur);
    }

    /// Un glyphe sur la cellule (cx, cy), à la place des pixels qu'elle tient.
    pub fn glyph(&mut self, cx: i32, cy: i32, ch: char, c: u8) {
        self.lay(cx, cy, Sorte::Glyphe, ch as u32, c, None);
    }
}

const BRAILLE: [[u32; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];

/// Une rangée lue de droite à gauche : un dessin tourné de l'autre côté.
pub fn mirror(rows: &[&str]) -> Vec<String> {
    rows.iter().map(|r| r.chars().rev().collect()).collect()
}

pub const DEFAUT: u32 = 0x01000000;
const HAUT: u32 = 0x2580; // ▀
const BAS: u32 = 0x2584; // ▄
const QUARTS: [char; 16] = [' ', '▘', '▝', '▀', '▖', '▌', '▞', '▛', '▗', '▚', '▐', '▜', '▄', '▙', '▟', '█'];

/// Lettre → 0xRRGGBB.
pub type Palette = [Option<u32>; 128];

/// Une lumière sur la scène : la couleur que prend un pixel de `lettre` en (x, y), d'après sa
/// couleur de palette ; pour un pixel clair (sans couleur), ce que la lumière montre derrière.
pub trait Lumiere {
    fn couleur(&mut self, x: i32, y: i32, lettre: u8, c: Option<u32>) -> Option<u32>;
}

/// Les cellules des colonnes x0..x1 de la grille : deux pixels par cellule, puis ce qui se
/// pose dessus, sous `lumiere` si donnée. Trois mots par cellule : le caractère, l'encre, le fond.
pub fn to_cells(g: &Grille, palette: &Palette, x0: i32, x1: i32, mut lumiere: Option<&mut dyn Lumiere>) -> Vec<u32> {
    let w = (x1 - x0).max(0);
    let rows = g.h / 2;
    let mut words = vec![0u32; (w * rows * 3) as usize];
    let mut colour = |x: i32, y: i32, lettre: u8| {
        let c = palette.get(lettre as usize).copied().flatten();
        match lumiere.as_deref_mut() {
            Some(l) => l.couleur(x, y, lettre, c),
            None => c,
        }
    };
    let mut i = 0usize;
    for r in 0..rows {
        for x in x0..x1 {
            let top = colour(x, 2 * r, g.at(x, 2 * r));
            let bot = colour(x, 2 * r + 1, g.at(x, 2 * r + 1));
            match top {
                None => {
                    words[i] = if bot.is_none() { 0x20 } else { BAS };
                    words[i + 1] = bot.unwrap_or(DEFAUT);
                    words[i + 2] = DEFAUT;
                }
                Some(t) => {
                    words[i] = HAUT;
                    words[i + 1] = t;
                    words[i + 2] = bot.unwrap_or(DEFAUT);
                }
            }
            if let Some(o) = g.sur.get(&cellule(x, r)) {
                if o.sous == g.sous(x, r) {
                    words[i] = match o.sorte {
                        Sorte::Points => 0x2800 + o.bits,
                        Sorte::Quarts => QUARTS.get(o.bits as usize).map_or(0x20, |&c| c as u32),
                        Sorte::Glyphe => o.bits,
                    };
                    words[i + 1] = colour(x, 2 * r, o.c).unwrap_or(DEFAUT);
                    words[i + 2] = colour(x, 2 * r, o.fond).unwrap_or(DEFAUT);
                }
            }
            i += 3;
        }
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrondi_comme_javascript() {
        assert_eq!(arrondi(2.5), 3.0);
        assert_eq!(arrondi(-2.5), -2.0);
        assert_eq!(arrondi(-0.4), 0.0);
        assert_eq!(arrondi(0.49999999999999994), 0.0);
    }
}
