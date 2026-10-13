//! La gerbe d'étincelles d'un coup de marteau, au vrai pixel de l'écran (d'après la mire
//! etincelles.rs du commit 980af2f) : elles partent en arcs, traînent derrière elles leur
//! mouvement, refroidissent du blanc au rouge et rebondissent au sol. Tout se tient en unités de
//! la hauteur du bandeau, et le temps avance par petits pas, quelle que soit la cadence.

const PAS: f32 = 1.0 / 120.0;
/// positions gardées par étincelle : à peu près ce qu'elle parcourt entre deux images
const TRAINE: usize = 16;
const PAS_TRAINE: usize = 3;

struct Etincelle {
    p: [f32; 2],
    v: [f32; 2],
    age: f32,
    vie: f32,
    rebonds: u8,
    traine: [[f32; 2]; TRAINE],
    n: usize,
}

#[derive(Default)]
pub struct Gerbe {
    graine: u64,
    reste: f32,
    etincelles: Vec<Etincelle>,
}

impl Gerbe {
    fn hasard(&mut self) -> f32 {
        // xorshift
        let mut x = self.graine | 1;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.graine = x;
        (x >> 40) as f32 / (1u64 << 24) as f32
    }

    fn entre(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.hasard()
    }

    /// Un coup en (x, y) : `n` étincelles, en demi-cercle vers le haut, plus nourri sur les côtés.
    pub fn frapper(&mut self, x: f32, y: f32, h: f32, n: usize) {
        for _ in 0..n {
            let angle = self.entre(-std::f32::consts::PI + 0.12, -0.12);
            let vitesse = self.entre(0.35, 1.6) * h * (0.6 + 0.4 * angle.cos().abs());
            let p = [x + self.entre(-0.02, 0.02) * h, y];
            let vie = self.entre(0.4, 1.2);
            self.etincelles.push(Etincelle { p, v: [angle.cos() * vitesse, angle.sin() * vitesse], age: 0.0, vie, rebonds: 0, traine: [p; TRAINE], n: 1 });
        }
    }

    /// Avance de `dt` secondes : la pesanteur, l'air, le sol en `sol`.
    pub fn avancer(&mut self, dt: f32, h: f32, sol: f32) {
        self.reste += dt.min(0.5);
        let (g, air) = (2.9 * h, (-PAS * 0.9f32).exp());
        while self.reste >= PAS {
            self.reste -= PAS;
            for e in &mut self.etincelles {
                e.age += PAS;
                e.v[1] += g * PAS;
                e.v[0] *= air;
                e.v[1] *= air;
                e.p[0] += e.v[0] * PAS;
                e.p[1] += e.v[1] * PAS;
                if e.p[1] > sol && e.v[1] > 0.0 {
                    e.p[1] = sol;
                    e.v[1] *= -0.32;
                    e.v[0] *= 0.55;
                    e.rebonds += 1;
                    if e.rebonds > 1 {
                        e.vie = e.vie.min(e.age + 0.12);
                    }
                }
                e.traine.copy_within(0..TRAINE - 1, 1);
                e.traine[0] = e.p;
                e.n = (e.n + 1).min(TRAINE);
            }
            self.etincelles.retain(|e| e.age < e.vie);
        }
    }

    /// Les étincelles ajoutées à la lumière de `pixels` (0xAARRGGBB prémultiplié, `w` × `h`).
    /// Rend le rectangle peint.
    pub fn peindre(&self, pixels: &mut [u32], w: usize, h: usize) -> Option<(i32, i32, i32, i32)> {
        let mut zone: Option<(i32, i32, i32, i32)> = None;
        let mut ajouter = |x: i32, y: i32, c: [f32; 3], a: f32| {
            if x < 0 || y < 0 || x as usize >= w || y as usize >= h || a <= 0.01 {
                return;
            }
            let d = &mut pixels[y as usize * w + x as usize];
            let canal = |n: u32, v: f32| (((*d >> n) & 255) as f32 + v * a * 255.0).min(255.0) as u32;
            let (r, g, b) = (canal(16, c[0]), canal(8, c[1]), canal(0, c[2]));
            *d = ((*d >> 24).max(r.max(g).max(b)) << 24) | (r << 16) | (g << 8) | b;
            zone = Some(zone.map_or((x, y, x + 1, y + 1), |z| (z.0.min(x), z.1.min(y), z.2.max(x + 1), z.3.max(y + 1))));
        };
        for e in &self.etincelles {
            let k = e.age / e.vie; // 0 : blanche, 1 : éteinte
            let c = if k < 0.25 {
                [1.0, 1.0 - 0.3 * k / 0.25, 0.85 - 0.6 * k / 0.25]
            } else {
                let m = (k - 0.25) / 0.75;
                [1.0 - 0.45 * m, 0.7 - 0.6 * m, 0.25 - 0.23 * m]
            };
            let a = (1.0 - k).powf(1.4);
            // la traîne, de plus en plus pâle vers sa queue, un point par pixel parcouru
            let mut i = 0;
            while i + PAS_TRAINE < e.n {
                let queue = 1.0 - (i + PAS_TRAINE) as f32 / TRAINE as f32;
                let ([x0, y0], [x1, y1]) = (e.traine[i], e.traine[i + PAS_TRAINE]);
                let pas = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0) as i32;
                for s in 0..pas {
                    let t = s as f32 / pas as f32;
                    ajouter((x0 + (x1 - x0) * t) as i32, (y0 + (y1 - y0) * t) as i32, c, 0.75 * a * queue);
                }
                i += PAS_TRAINE;
            }
            // la tête, plus chaude
            let (x, y) = (e.p[0] as i32, e.p[1] as i32);
            ajouter(x, y, [1.0, 0.95, 0.8], a);
            ajouter(x + 1, y, c, 0.5 * a);
            ajouter(x, y + 1, c, 0.5 * a);
        }
        zone
    }
}
