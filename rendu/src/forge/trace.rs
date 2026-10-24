//! Le dessin au vrai pixel : des polygones, des traits et des disques lissés (la couverture de
//! chaque pixel comptée sur quatre sous-rangées, au bord exact en largeur), posés sur une toile
//! de pixels 0xFFRRGGBB.

pub type Rgb = [f32; 3];

pub const fn hex(n: u32) -> Rgb {
    [((n >> 16) & 255) as f32, ((n >> 8) & 255) as f32, (n & 255) as f32]
}

pub fn mix(a: Rgb, b: Rgb, k: f32) -> Rgb {
    [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, a[2] + (b[2] - a[2]) * k]
}

pub fn mul(a: Rgb, m: Rgb) -> Rgb {
    [a[0] * m[0], a[1] * m[1], a[2] * m[2]]
}

pub fn fois(a: Rgb, k: f32) -> Rgb {
    [a[0] * k, a[1] * k, a[2] * k]
}

#[inline]
pub fn pack(c: Rgb) -> u32 {
    let b = |v: f32| (v + 0.5).clamp(0.0, 255.0) as u32;
    0xFF00_0000 | (b(c[0]) << 16) | (b(c[1]) << 8) | b(c[2])
}

#[inline]
pub fn unpack(p: u32) -> Rgb {
    [((p >> 16) & 255) as f32, ((p >> 8) & 255) as f32, (p & 255) as f32]
}

pub fn smooth(a: f32, b: f32, v: f32) -> f32 {
    let t = ((v - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Une toile : `w` × `h` pixels, ligne après ligne.
pub struct Toile<'a> {
    pub px: &'a mut [u32],
    pub w: usize,
    pub h: usize,
}

impl Toile<'_> {
    /// Pose `c` sur le pixel (x, y), vu à travers `a`.
    #[inline]
    pub fn poser(&mut self, x: i32, y: i32, c: Rgb, a: f32) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h || a <= 0.0 {
            return;
        }
        let i = y as usize * self.w + x as usize;
        self.px[i] = if a >= 1.0 { pack(c) } else { pack(mix(unpack(self.px[i]), c, a)) };
    }

    /// Un polygone plein (pair-impair), bords lissés.
    pub fn poly(&mut self, pts: &[(f32, f32)], c: Rgb, a: f32) {
        let n = pts.len();
        if n < 3 || a <= 0.0 {
            return;
        }
        let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for &(x, y) in pts {
            x0 = x0.min(x);
            x1 = x1.max(x);
            y0 = y0.min(y);
            y1 = y1.max(y);
        }
        let (xa, xb) = ((x0.floor() as i32).max(0), (x1.ceil() as i32).min(self.w as i32));
        let (ya, yb) = ((y0.floor() as i32).max(0), (y1.ceil() as i32).min(self.h as i32));
        if xa >= xb || ya >= yb {
            return;
        }
        // les côtés, une fois : du haut vers le bas, et leur pente
        let mut cotes: Vec<(f32, f32, f32, f32)> = Vec::with_capacity(n);
        for k in 0..n {
            let (a, b) = (pts[k], pts[if k + 1 == n { 0 } else { k + 1 }]);
            if a.1 == b.1 {
                continue;
            }
            let (h, l) = if a.1 < b.1 { (a, b) } else { (b, a) };
            cotes.push((h.1, l.1, h.0, (l.0 - h.0) / (l.1 - h.1)));
        }
        let wd = (xb - xa) as usize;
        let mut cov = vec![0f32; wd];
        let mut xs: Vec<f32> = Vec::with_capacity(8);
        for py in ya..yb {
            let mut rien = true;
            let (mut lo, mut hi) = (wd, 0usize);
            for s in 0..4 {
                let sy = py as f32 + (s as f32 + 0.5) / 4.0;
                xs.clear();
                for &(ha, ba, hx, pente) in &cotes {
                    if ha <= sy && sy < ba {
                        xs.push(hx + (sy - ha) * pente);
                    }
                }
                if xs.len() < 2 {
                    continue;
                }
                xs.sort_by(|a, b| a.total_cmp(b));
                for p in xs.chunks_exact(2) {
                    let (l, r) = (p[0] - xa as f32, p[1] - xa as f32);
                    if r <= 0.0 || l >= wd as f32 {
                        continue;
                    }
                    rien = false;
                    lo = lo.min(l.max(0.0) as usize);
                    hi = hi.max((r.ceil() as usize).min(wd));
                    couvrir(&mut cov, l, r, 0.25);
                }
            }
            if rien {
                continue;
            }
            // l'intérieur, couvert en entier : la couleur telle quelle ; les bords, mêlés
            let plein = pack(c);
            let ligne = &mut self.px[py as usize * self.w + xa as usize..][..wd];
            for (k, p) in cov[lo..hi].iter_mut().zip(&mut ligne[lo..hi]) {
                if *k > 0.0 {
                    let al = a * k.min(1.0);
                    *p = if al >= 0.999 { plein } else { pack(mix(unpack(*p), c, al)) };
                    *k = 0.0;
                }
            }
        }
    }

    /// Un trait de `e` pixels d'épaisseur, de a à b.
    pub fn trait_(&mut self, a: (f32, f32), b: (f32, f32), e: f32, c: Rgb, al: f32) {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l = dx.hypot(dy).max(1e-3);
        let (nx, ny) = (-dy / l * e / 2.0, dx / l * e / 2.0);
        self.poly(&[(a.0 + nx, a.1 + ny), (b.0 + nx, b.1 + ny), (b.0 - nx, b.1 - ny), (a.0 - nx, a.1 - ny)], c, al);
    }

    /// Une ligne brisée.
    pub fn ligne(&mut self, pts: &[(f32, f32)], e: f32, c: Rgb, al: f32) {
        for p in pts.windows(2) {
            self.trait_(p[0], p[1], e, c, al);
        }
    }

    /// Un disque, au bord net (`doux` 0) ou qui s'estompe vers son bord (`doux` 1).
    pub fn disque(&mut self, cx: f32, cy: f32, r: f32, c: Rgb, a: f32, doux: f32) {
        if r <= 0.0 || a <= 0.0 {
            return;
        }
        let (xa, xb) = ((cx - r - 1.0).floor().max(0.0) as i32, ((cx + r + 1.0).ceil() as i32).min(self.w as i32));
        let (ya, yb) = ((cy - r - 1.0).floor().max(0.0) as i32, ((cy + r + 1.0).ceil() as i32).min(self.h as i32));
        let loin = (r + 0.5) * (r + 0.5);
        for y in ya..yb {
            let dy = y as f32 + 0.5 - cy;
            for x in xa..xb {
                let dx = x as f32 + 0.5 - cx;
                let d2 = dx * dx + dy * dy;
                if d2 >= loin {
                    continue;
                }
                let d = d2.sqrt();
                let net = (r - d + 0.5).min(1.0);
                let k = if doux > 0.0 {
                    let f = 1.0 - doux * (d / r).min(1.0);
                    net * f * f
                } else {
                    net
                };
                if k > 0.0 {
                    self.poser(x, y, c, a * k);
                }
            }
        }
    }
}

/// Ajoute `poids` aux pixels que couvre [l, r), au bord près.
fn couvrir(cov: &mut [f32], l: f32, r: f32, poids: f32) {
    let wd = cov.len() as f32;
    let (l, r) = (l.max(0.0), r.min(wd));
    if r <= l {
        return;
    }
    let (il, ir) = (l.floor() as usize, r.floor() as usize);
    if il == ir {
        cov[il] += (r - l) * poids;
        return;
    }
    cov[il] += (il as f32 + 1.0 - l) * poids;
    for v in &mut cov[il + 1..ir] {
        *v += poids;
    }
    if ir < cov.len() {
        cov[ir] += (r - ir as f32) * poids;
    }
}

/// Le morceau d'un polygone entre deux hauteurs (y de `haut` à `bas`).
pub fn bande(pts: &[(f32, f32)], haut: f32, bas: f32) -> Vec<(f32, f32)> {
    let couper = |pts: &[(f32, f32)], garde: &dyn Fn(f32) -> bool, y0: f32| {
        let mut out = Vec::with_capacity(pts.len() + 2);
        for k in 0..pts.len() {
            let a = pts[k];
            let b = pts[(k + 1) % pts.len()];
            let (ia, ib) = (garde(a.1), garde(b.1));
            if ia {
                out.push(a);
            }
            if ia != ib {
                let t = (y0 - a.1) / (b.1 - a.1);
                out.push((a.0 + (b.0 - a.0) * t, y0));
            }
        }
        out
    };
    let p = couper(pts, &|y| y >= haut, haut);
    couper(&p, &|y| y <= bas, bas)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_carre_couvre_ses_pixels_et_lisse_ses_bords() {
        let mut px = vec![0xFF00_0000u32; 100];
        let mut t = Toile { px: &mut px, w: 10, h: 10 };
        t.poly(&[(2.0, 2.0), (6.0, 2.0), (6.0, 6.0), (2.0, 6.0)], [255.0, 255.0, 255.0], 1.0);
        assert_eq!(px[3 * 10 + 3], 0xFFFF_FFFF);
        assert_eq!(px[7 * 10 + 7], 0xFF00_0000);
        let mut px = vec![0xFF00_0000u32; 100];
        let mut t = Toile { px: &mut px, w: 10, h: 10 };
        t.poly(&[(2.5, 2.0), (6.0, 2.0), (6.0, 6.0), (2.5, 6.0)], [255.0, 255.0, 255.0], 1.0);
        assert_eq!(px[3 * 10 + 2] & 0xFF, 128, "un demi-pixel couvert à moitié");
    }

    #[test]
    fn une_bande_coupe_un_triangle() {
        let b = bande(&[(0.0, 0.0), (10.0, 10.0), (0.0, 10.0)], 4.0, 6.0);
        assert!(b.iter().all(|p| (4.0..=6.0).contains(&p.1)));
        assert_eq!(b.len(), 4);
    }
}
