//! La lumière de la forge (lightOf de hooks/themes/forge.ts). La pièce est sombre le jour,
//! bleue la nuit ; le foyer éclaire devant et en bas (sa hotte arrête ce qui monterait), la
//! lanterne son coin, un coup son éclair, la fenêtre le sol sous elle à la couleur de l'heure.
//! Là où Scorpheus fait face au feu il se réchauffe, là où il s'en détourne il se refroidit ;
//! et le mur du fond se montre où une lumière tombe.

use std::collections::HashMap;

use super::atelier::{C, IMPACT, LANTERN, MOUTH, SH, SW, WINDOW, YARD};
use super::cour::{pack, Dehors};
use super::monde::{Monde, Temps};
use super::pixels::{arrondi, hasard, Grille, Lumiere};
use super::poses::Pose;

/// Ce qui luit de soi-même : flammes, braises, étincelles, métal chaud.
const GLOWS: &[u8] = b"YfFEsSH";
/// Dehors aucune flamme ne brûle : seules les étincelles et la chaleur blanche luisent.
const GLOWS_OUT: &[u8] = b"YsS";
/// Son contour, sa crête bleue (son emblème) gardent leur couleur.
const FLAT: &[u8] = b"knBb";

// Le mur derrière la scène, de pierre sombre : il ne se voit que là où une lumière tombe.
const WALL: [f64; 3] = [74.0, 62.0, 70.0];

/// Un bruit qui dérive plutôt qu'il ne saute : une flamme qui vacille, pas un stroboscope.
fn drift(t: i64, every: i64, seed: i64) -> f64 {
    let i = t.div_euclid(every);
    let f = (t % every) as f64 / every as f64;
    hasard(i * 7 + seed) * (1.0 - f) + hasard((i + 1) * 7 + seed) * f
}

#[derive(Clone, Copy)]
struct Lampe {
    x: f64,
    y: f64,
    reach: f64,
    up: f64,
    down: f64,
    rgb: [f64; 3],
    power: f64,
}

/// Comment la lumière d'une lampe tombe sur la pièce, avant sa puissance : calculé une fois
/// par lampe, puisqu'aucune ne bouge ; seul son éclat change d'une image à l'autre.
#[derive(Default)]
pub struct Chutes(HashMap<[u64; 5], Vec<f64>>);

impl Chutes {
    fn de(&mut self, l: &Lampe) -> &[f64] {
        let key = [l.x.to_bits(), l.y.to_bits(), l.reach.to_bits(), l.up.to_bits(), l.down.to_bits()];
        self.0.entry(key).or_insert_with(|| {
            let mut map = vec![0.0; (YARD * SH) as usize];
            for y in 0..SH {
                for x in 0..YARD {
                    let (yf, xf) = (y as f64, x as f64);
                    let d = (xf - l.x).hypot((yf - l.y) * if yf < l.y { l.up } else { l.down });
                    map[(y * YARD + x) as usize] = (1.0 - d / l.reach).max(0.0).powi(2);
                }
            }
            map
        })
    }
}

/// Scorpheus en relief : le long de chaque rangée de ses pixels, le côté vers le foyer (à
/// gauche) prend la lumière, l'autre l'ombre, et ce qui regarde en haut un liseré de lumière.
/// Son contour, ses bulles, sa crête et ses bottes (une botte posée ne doit pas changer quand
/// l'autre passe) gardent leur couleur.
fn model_of(actor: &Grille, w: &Monde, p: &Pose) -> Vec<f32> {
    let sx = C.0 + w.x;
    let sy = C.1 + p.dy + if p.seated { 2 } else { 0 };
    let boots = C.1 + p.dy + 24;
    let mut k = vec![1f32; (SW * SH) as usize];
    for y in 0..SH {
        let mut x0 = 0;
        while x0 < SW {
            if actor.at(x0, y) == b'.' {
                x0 += 1;
                continue;
            }
            let mut x1 = x0;
            while actor.at(x1 + 1, y) != b'.' {
                x1 += 1;
            }
            for x in x0..=x1 {
                let (ch, up) = (actor.at(x, y), actor.at(x, y - 1));
                if FLAT.contains(&ch) || y >= boots || (x >= sx + 17 && y < sy + 12) || x >= SW {
                    continue;
                }
                k[(y * SW + x) as usize] =
                    (1.16 - (0.32 * (x - x0) as f64) / (x1 - x0).max(1) as f64 + if up == b'.' || up == b'k' { 0.08 } else { 0.0 }) as f32;
            }
            x0 = x1 + 1;
        }
    }
    k
}

pub struct LumiereForge<'a> {
    lamps: Vec<(Lampe, &'a [f64])>,
    amb: [f64; 3],
    actor: &'a Grille,
    shade: Vec<f32>,
    dehors: &'a Dehors,
}

impl<'a> LumiereForge<'a> {
    pub fn new(chutes: &'a mut Chutes, p: &Pose, w: &Monde, tick: i64, actor: &'a Grille, dehors: &'a Dehors) -> Self {
        let night = w.nuit();
        let dusk = !night && (w.hour < 8.0 || w.hour >= 18.0);
        let closed = w.hour >= 23.0 || w.hour < 6.0;
        let amb = if night {
            [0.36, 0.4, 0.56]
        } else if dusk {
            [0.74, 0.68, 0.7]
        } else {
            [0.8, 0.79, 0.77]
        };
        let fire = if w.exhausted { 0 } else { p.fire };
        let mut lamps = vec![Lampe {
            x: MOUTH.0 as f64 + MOUTH.2 as f64 / 2.0,
            y: MOUTH.1 as f64 + MOUTH.3 as f64 / 2.0,
            reach: 34.0,
            up: 2.2,
            down: 1.0,
            rgb: [1.0, 0.55, 0.25],
            power: 0.22 * fire as f64 * (0.85 + 0.15 * drift(tick, 3, 3)),
        }];
        if night && w.lantern_lit {
            lamps.push(Lampe {
                x: (LANTERN.0 + 1) as f64,
                y: (LANTERN.1 + 4) as f64,
                reach: 13.0,
                up: 1.0,
                down: 1.0,
                rgb: [1.0, 0.75, 0.4],
                power: 0.55 * (0.92 + 0.08 * drift(tick, 4, 9)),
            });
        }
        if let Some(age) = p.sparks_age.filter(|&a| a < 3) {
            lamps.push(Lampe {
                x: IMPACT.0 as f64,
                y: IMPACT.1 as f64,
                reach: 16.0,
                up: 1.6,
                down: 1.0,
                rgb: [1.0, 0.95, 0.7],
                power: 0.9 * (1.0 - age as f64 / 3.0),
            });
        }
        let storm = w.weather == Some(Temps::Storm);
        let sky = if storm {
            [0.55, 0.6, 0.7]
        } else if night {
            [0.45, 0.55, 0.95]
        } else if dusk {
            [1.0, 0.7, 0.45]
        } else {
            [1.0, 0.97, 0.88]
        };
        let daylight = if closed {
            0.0
        } else if storm {
            0.25
        } else if night {
            0.3
        } else if dusk {
            0.45
        } else {
            0.55
        };
        if daylight > 0.0 {
            lamps.push(Lampe {
                x: WINDOW.0 as f64 + WINDOW.2 as f64 / 2.0,
                y: (WINDOW.1 + WINDOW.3 - 1) as f64,
                reach: 26.0,
                up: 3.0,
                down: 0.6,
                rgb: sky,
                power: daylight,
            });
        }
        // les cartes de chute d'abord (elles empruntent le cache), puis la liste
        for l in &lamps {
            chutes.de(l);
        }
        let chutes: &'a Chutes = chutes;
        let lamps = lamps
            .into_iter()
            .map(|l| {
                let key = [l.x.to_bits(), l.y.to_bits(), l.reach.to_bits(), l.up.to_bits(), l.down.to_bits()];
                (l, chutes.0.get(&key).map_or(&[][..], |v| &v[..]))
            })
            .collect();
        LumiereForge { lamps, amb, actor, shade: model_of(actor, w, p), dehors }
    }

    /// Ce que toutes les lampes posent sur un pixel de la pièce.
    fn gather(&self, x: i32, y: i32) -> [f64; 3] {
        let i = (y * YARD + x) as usize;
        let mut k = [0.0; 3];
        for (l, map) in &self.lamps {
            let f = map.get(i).copied().unwrap_or(0.0) * l.power;
            k[0] += f * l.rgb[0];
            k[1] += f * l.rgb[1];
            k[2] += f * l.rgb[2];
        }
        k
    }
}

fn byte(v: f64) -> u32 {
    arrondi(v).clamp(0.0, 255.0) as u32
}

impl Lumiere for LumiereForge<'_> {
    fn couleur(&mut self, x: i32, y: i32, lettre: u8, c: Option<u32>) -> Option<u32> {
        if x >= YARD {
            // dehors : le ciel derrière, la lumière de l'heure sur qui s'y tient, celle de la forge à la porte
            let Some(c) = c else { return self.dehors.back(x, y) };
            if GLOWS_OUT.contains(&lettre) {
                return Some(c);
            }
            let rgb = [(c >> 16) as f64, ((c >> 8) & 255) as f64, (c & 255) as f64];
            return Some(pack(self.dehors.light(rgb, x, y)));
        }
        let Some(c) = c else {
            let k = self.gather(x, y);
            let (r, g, b) = (byte(WALL[0] * k[0]), byte(WALL[1] * k[1]), byte(WALL[2] * k[2]));
            return if r.max(g).max(b) < 12 { None } else { Some((r << 16) | (g << 8) | b) };
        };
        if GLOWS.contains(&lettre) || (x >= WINDOW.0 && x < WINDOW.0 + WINDOW.2 && y < WINDOW.1 + WINDOW.3) {
            return Some(c);
        }
        let [kr, kg, kb] = self.gather(x, y);
        // ses propres pixels : chauds vers le feu, froids à l'écart
        let s = if self.actor.at(x, y) == lettre && (0..SW).contains(&x) && (0..SH).contains(&y) {
            self.shade[(y * SW + x) as usize] as f64 - 1.0
        } else {
            0.0
        };
        let (mr, mg, mb) = if s > 0.0 { (1.0 + s * 1.25, 1.0 + s, 1.0 + s * 0.55) } else { (1.0 + s * 1.2, 1.0 + s, 1.0 + s * 0.3) };
        let [ar, ag, ab] = self.amb;
        Some(
            (byte((c >> 16) as f64 * (ar + kr) * mr) << 16)
                | (byte(((c >> 8) & 255) as f64 * (ag + kg) * mg) << 8)
                | byte((c & 255) as f64 * (ab + kb) * mb),
        )
    }
}
