//! La lumière de la forge (lightOf de hooks/themes/forge.ts). La pièce est sombre le jour,
//! bleue la nuit ; le foyer éclaire devant et en bas (sa hotte arrête ce qui monterait), la
//! lanterne son coin, un coup son éclair, la fenêtre le sol sous elle à la couleur de l'heure.
//! Là où Scorpheus fait face au feu il se réchauffe, là où il s'en détourne il se refroidit ;
//! et le mur du fond se montre où une lumière tombe.

use std::collections::HashMap;

use super::atelier::{ANVIL_AT, BUCKET_AT, C, CRATE_AT, IMPACT, LANTERN, MOUTH, SH, SW, WINDOW, YARD};
use super::cour::{pack, Dehors};
use super::monde::{Monde, Temps};
use super::pet::Eclairage;
use super::pixels::{arrondi, hasard, Grille, Lumiere};
use super::poses::Pose;

/// Ce qui luit de soi-même : flammes, braises, étincelles, métal chaud.
const GLOWS: &[u8] = b"YfFEsSH";
/// Dehors aucune flamme ne brûle : seules les étincelles et la chaleur blanche luisent.
const GLOWS_OUT: &[u8] = b"YsS";
/// Son contour, sa crête bleue (son emblème) gardent leur couleur.
const FLAT: &[u8] = b"knBb";

// Le mur derrière la scène, de pierre sombre : à peine visible à l'ambiance (MUR), il se montre
// où une lumière tombe.
const WALL: [f64; 3] = [74.0, 62.0, 70.0];
/// La part de l'ambiance que reçoit le mur : la pièce est plus sombre que ce qui s'y tient.
const MUR: f64 = 0.42;

/// L'appareil du mur : des pierres de 3 × 2 pixels, en quinconce d'une rangée à l'autre, chacune
/// d'un ton à elle (de 0,82 à 1,1), pour que la lumière qui passe y montre un grain.
fn pierre(x: i32, y: i32) -> f64 {
    let rang = y.div_euclid(2) as i64;
    let col = (x + if rang % 2 == 1 { 1 } else { 0 }).div_euclid(3) as i64;
    0.82 + 0.28 * hasard(rang * 131 + col * 17 + 5)
}

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
    /// son halo au vrai pixel : sa portée en pixels de forge (0 : elle ne luit pas d'elle-même), et
    /// son éclat par unité de puissance
    halo: (f64, f64),
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
    /// ce qui arrête la lumière, par pixel de la pièce (`ombres`)
    ombre: Vec<bool>,
    /// le modelé de ce qui arrête la lumière (`relief`)
    relief: Vec<f32>,
    /// le jour qui entre par la fenêtre : sa couleur et sa force (`rai`)
    jour: Option<([f64; 3], f64)>,
}

/// La part de sa lumière qu'une lampe garde derrière ce qui l'arrête : l'ombre n'est jamais noire, la pièce
/// renvoie un peu de lumière.
const OMBRE: f64 = 0.15;

/// La part du jour à cette heure du ciel (`Monde::heure_ciel`) : 0 la nuit (avant 6 h, après 21 h), 0,6 à 8 h et
/// à 18 h (le plein jour commence et finit), 1 à 13 h (le midi solaire), en ligne droite entre deux.
pub fn jour(h: f64) -> f64 {
    match h {
        h if !(6.0..21.0).contains(&h) => 0.0,
        h if h < 8.0 => 0.6 * (h - 6.0) / 2.0,
        h if h < 13.0 => 0.6 + 0.4 * (h - 8.0) / 5.0,
        h if h < 18.0 => 1.0 - 0.4 * (h - 13.0) / 5.0,
        h => 0.6 * (21.0 - h) / 3.0,
    }
}

/// Le modelé de ce qui arrête la lumière, comme celui de Scorpheus (`model_of`) : le long de chaque rangée, le
/// côté vers le foyer (à gauche) prend la lumière, l'autre l'ombre ; ce qui regarde en haut, un liseré.
fn relief(ombre: &[bool]) -> Vec<f32> {
    let mut k = vec![1f32; ombre.len()];
    let a = |x: i32, y: i32| (0..YARD).contains(&x) && (0..SH).contains(&y) && ombre[(y * YARD + x) as usize];
    for y in 0..SH {
        let mut x0 = 0;
        while x0 < YARD {
            if !a(x0, y) {
                x0 += 1;
                continue;
            }
            let mut x1 = x0;
            while a(x1 + 1, y) {
                x1 += 1;
            }
            for x in x0..=x1 {
                k[(y * YARD + x) as usize] = (1.16 - (0.36 * (x - x0) as f64) / (x1 - x0).max(1) as f64 + if a(x, y - 1) { 0.0 } else { 0.1 }) as f32;
            }
            x0 = x1 + 1;
        }
    }
    k
}

/// Ce qui arrête la lumière dans la pièce et porte une ombre, par pixel (YARD × SH) : l'enclume, la caisse qui
/// pose dessus, le seau ; Scorpheus en lettres (`actor`) ; et chacun de ceux que pet.rs peint au vrai pixel
/// (`corps` : le milieu de leurs pieds, en pixels de forge), par une ellipse de sa taille. Les étincelles et le
/// métal chaud luisent : ils n'arrêtent rien.
pub fn ombres(scene: &Grille, actor: &Grille, corps: &[f32]) -> Vec<bool> {
    let mut m = vec![false; (YARD * SH) as usize];
    let mut poser = |x: i32, y: i32| {
        if (0..YARD).contains(&x) && (0..SH).contains(&y) {
            m[(y * YARD + x) as usize] = true;
        }
    };
    for (bx, by, bw, bh) in [(ANVIL_AT.0, ANVIL_AT.1, 13, 7), (BUCKET_AT.0, BUCKET_AT.1, 8, 5), (CRATE_AT.0, CRATE_AT.1 - 7, 10, 10)] {
        for y in by..by + bh {
            for x in bx..bx + bw {
                let c = scene.at(x, y);
                if c != b'.' && !GLOWS.contains(&c) {
                    poser(x, y);
                }
            }
        }
    }
    for y in 0..SH {
        for x in 0..YARD.min(SW) {
            if actor.at(x, y) != b'.' {
                poser(x, y);
            }
        }
    }
    for &cx in corps {
        for y in 0..SH {
            for x in 0..YARD {
                let (dx, dy) = ((x as f32 + 0.5 - cx) / 6.0, (y as f32 + 0.5 - (SH as f32 - 12.0)) / 12.0);
                if dx * dx + dy * dy <= 1.0 {
                    poser(x, y);
                }
            }
        }
    }
    m
}

impl<'a> LumiereForge<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(chutes: &'a mut Chutes, p: &Pose, w: &Monde, tick: i64, actor: &'a Grille, dehors: &'a Dehors, ombre: Vec<bool>) -> Self {
        let night = w.nuit();
        let ciel = w.heure_ciel();
        // les volets fermés pour la nuit (la routine de la maison, à la pendule) : plus de jour ni de lune
        let closed = w.hour >= 23.0 || w.hour < 6.0;
        // le jour monte et descend sans paliers (user, 02-11 : « la luminosité qui change en fonction de
        // l'heure ») : 0 la nuit, 0,6 à la fin du crépuscule du matin et au début de celui du soir, 1 au midi solaire
        let j = jour(ciel);
        let par = |nuit: [f64; 3], soir: [f64; 3], midi: [f64; 3]| -> [f64; 3] {
            let (a, b, t) = if j < 0.6 { (nuit, soir, j / 0.6) } else { (soir, midi, (j - 0.6) / 0.4) };
            [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
        };
        let amb = par([0.36, 0.4, 0.56], [0.74, 0.68, 0.7], [0.8, 0.79, 0.77]);
        let fire = if w.exhausted { 0 } else { p.fire };
        let mut lamps = vec![Lampe {
            x: MOUTH.0 as f64 + MOUTH.2 as f64 / 2.0,
            y: MOUTH.1 as f64 + MOUTH.3 as f64 / 2.0,
            // jusqu'au fond de la pièce : ce qui se tient devant le feu y porte son ombre (02-11)
            reach: 48.0,
            up: 2.2,
            down: 1.0,
            rgb: [1.0, 0.55, 0.25],
            // un feu, même bas, éclaire la pièce : 0,22 par niveau seulement suffisait sur le fond
            // noir du mod, pas sur une pièce peinte (02-11)
            power: if fire > 0 { 0.3 + 0.22 * fire as f64 } else { 0.0 } * (0.85 + 0.15 * drift(tick, 3, 3)),
            halo: (16.0, 1.5),
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
                halo: (7.0, 0.45),
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
                halo: (9.0, 0.45),
            });
        }
        let storm = w.weather == Some(Temps::Storm);
        let sky = if storm { [0.55, 0.6, 0.7] } else { par([0.45, 0.55, 0.95], [1.0, 0.7, 0.45], [1.0, 0.97, 0.88]) };
        let daylight = if closed {
            0.0
        } else if storm {
            0.25
        } else {
            // la lune 0,3 ; le jour de 0,46 au bord du crépuscule à 0,58 au midi solaire
            0.3 + 0.28 * j
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
                halo: (0.0, 0.0),
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
        let jour = (daylight > 0.0).then_some((sky, daylight));
        let relief = relief(&ombre);
        LumiereForge { lamps, amb, actor, shade: model_of(actor, w, p), dehors, ombre, relief, jour }
    }

    /// L'ambiance et les lampes sur chaque pixel de forge d'un rectangle (borné à la pièce) :
    /// la lumière que pet.rs pose sur Scorpheus.
    pub fn eclairage(&self, x0: i32, y0: i32, w: i32, h: i32) -> Eclairage {
        let mut k = Vec::with_capacity((w * h).max(0) as usize);
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                let g = self.gather(x.clamp(0, YARD - 1), y.clamp(0, SH - 1));
                k.push([(self.amb[0] + g[0]) as f32, (self.amb[1] + g[1]) as f32, (self.amb[2] + g[2]) as f32]);
            }
        }
        Eclairage { x0, y0, w, h, k }
    }

    /// Les halos des lampes qui luisent d'elles-mêmes (le foyer, la lanterne, le coup), à poser au
    /// vrai pixel par-dessus la scène (`halos`).
    pub fn lueurs(&self) -> Vec<Lueur> {
        self.lamps
            .iter()
            .filter(|(l, _)| l.halo.0 > 0.0 && l.power > 0.0)
            .map(|(l, _)| Lueur { x: l.x as f32, y: l.y as f32, portee: l.halo.0 as f32, haut: l.up as f32, rgb: l.rgb.map(|c| c as f32), eclat: (l.power * l.halo.1).min(ECLAT_MAX) as f32 })
            .collect()
    }

    /// Le jour qui entre par la fenêtre, s'il en entre : sa couleur et sa force.
    pub fn jour(&self) -> Option<([f32; 3], f32)> {
        self.jour.map(|(c, f)| (c.map(|v| v as f32), f as f32))
    }

    /// Ce qui se tient entre la lampe et ce pixel : on va de lui vers elle, pixel par pixel.
    fn cache(&self, l: &Lampe, x: i32, y: i32) -> bool {
        let (dx, dy) = (l.x - x as f64, l.y - y as f64);
        let n = dx.abs().max(dy.abs()).ceil() as i32;
        (1..n).any(|s| {
            let t = s as f64 / n as f64;
            let (qx, qy) = ((x as f64 + dx * t).round() as i32, (y as f64 + dy * t).round() as i32);
            (0..YARD).contains(&qx) && (0..SH).contains(&qy) && self.ombre[(qy * YARD + qx) as usize]
        })
    }

    /// Ce que les lampes posent sur ce pixel, chacune gardée à OMBRE derrière ce qui l'arrête.
    fn gather_ombre(&self, x: i32, y: i32) -> [f64; 3] {
        let i = (y * YARD + x) as usize;
        let mut k = [0.0; 3];
        for (l, map) in &self.lamps {
            let mut f = map.get(i).copied().unwrap_or(0.0) * l.power;
            if f <= 0.0 {
                continue;
            }
            if self.cache(l, x, y) {
                f *= OMBRE;
            }
            k[0] += f * l.rgb[0];
            k[1] += f * l.rgb[1];
            k[2] += f * l.rgb[2];
        }
        k
    }

    /// Ce pixel porte-t-il une ombre (il arrête la lumière, il ne la reçoit pas d'ombre) ?
    fn arrete(&self, x: i32, y: i32) -> bool {
        (0..YARD).contains(&x) && (0..SH).contains(&y) && self.ombre.get((y * YARD + x) as usize).copied().unwrap_or(false)
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
            // le mur du fond, partout dans la pièce : sombre à l'ambiance, montré par chaque lampe.
            // Peint là seulement où une lumière tombait, il laissait voir ailleurs le fond du
            // terminal ; gris dans ClaudeTerminal, ce fond rendait la lueur du feu plus sombre que
            // la pièce (user, 02-11 : « là où il y a de la lumière, c'est noir »)
            let k = self.gather_ombre(x, y);
            let p = pierre(x, y);
            let a = self.amb;
            return Some(
                (byte(WALL[0] * p * (MUR * a[0] + k[0])) << 16) | (byte(WALL[1] * p * (MUR * a[1] + k[1])) << 8) | byte(WALL[2] * p * (MUR * a[2] + k[2])),
            );
        };
        if GLOWS.contains(&lettre) || (x >= WINDOW.0 && x < WINDOW.0 + WINDOW.2 && y < WINDOW.1 + WINDOW.3) {
            return Some(c);
        }
        // ce qui porte une ombre reçoit toute la lumière ; le reste, ce qu'on ne lui cache pas
        let [kr, kg, kb] = if self.arrete(x, y) { self.gather(x, y) } else { self.gather_ombre(x, y) };
        // ses propres pixels : chauds vers le feu, froids à l'écart
        let s = if self.actor.at(x, y) == lettre && (0..SW).contains(&x) && (0..SH).contains(&y) {
            self.shade[(y * SW + x) as usize] as f64 - 1.0
        } else if self.arrete(x, y) && !FLAT.contains(&lettre) {
            // l'enclume, le seau, la caisse : modelés comme lui (user, 02-11 : « pas d'ombre sur l'enclume »)
            self.relief[(y * YARD + x) as usize] as f64 - 1.0
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

/// Ce qu'un halo ajoute au plus en son centre : au-delà, le feu brûlerait l'image en blanc.
const ECLAT_MAX: f64 = 0.6;

/// Une lueur au vrai pixel : son centre et sa portée en pixels de forge, ce qu'elle s'écrase vers le
/// haut (la hotte du foyer), sa couleur et son éclat (la part de blanc qu'elle ajoute en son centre).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lueur {
    pub x: f32,
    pub y: f32,
    pub portee: f32,
    pub haut: f32,
    pub rgb: [f32; 3],
    pub eclat: f32,
}

/// La lumière dans l'air, comme les jeux en sprites 2D la simulent : autour de chaque lueur, un
/// halo doux ajouté aux pixels déjà peints (scène, personnages, étincelles), sans marches, borné à
/// la pièce (`bord`, en pixels d'image). Rend les rectangles touchés : leurs cellules se repeignent
/// à l'image suivante, sans quoi le halo s'y ajouterait encore.
#[allow(clippy::too_many_arguments)]
pub fn halos(px: &mut [u32], w: usize, h: usize, lueurs: &[Lueur], x0: i32, cw: i32, ch: i32, bord: i32) -> Vec<(i32, i32, i32, i32)> {
    let mut zones = Vec::new();
    let droite_max = bord.min(w as i32);
    for l in lueurs {
        if l.eclat < 0.004 {
            continue;
        }
        let cx = (l.x - x0 as f32 + 0.5) * cw as f32;
        let cy = (l.y + 0.5) * ch as f32 / 2.0;
        let r = l.portee * cw as f32;
        let (gauche, haut) = ((cx - r).floor().max(0.0) as i32, (cy - r / l.haut.max(1.0)).floor().max(0.0) as i32);
        let (droite, bas) = (((cx + r).ceil() as i32).min(droite_max), ((cy + r).ceil() as i32).min(h as i32));
        if gauche >= droite || haut >= bas {
            continue;
        }
        let r2 = r * r;
        for y in haut..bas {
            let dy = (y as f32 + 0.5 - cy) * if (y as f32 + 0.5) < cy { l.haut } else { 1.0 };
            for x in gauche..droite {
                let dx = x as f32 + 0.5 - cx;
                let t = (dx * dx + dy * dy) / r2;
                if t >= 1.0 {
                    continue;
                }
                let i = y as usize * w + x as usize;
                let p = px[i];
                // un pixel transparent n'a rien sur quoi la lumière tombe
                if p >> 24 == 0 {
                    continue;
                }
                let f = (1.0 - t) * (1.0 - t) * (1.0 - t) * l.eclat * 255.0;
                let plus = |d: u32, k: f32| ((((p >> d) & 255) as f32 + k * f).min(255.0) as u32) << d;
                px[i] = (p & 0xff00_0000) | plus(16, l.rgb[0]) | plus(8, l.rgb[1]) | plus(0, l.rgb[2]);
            }
        }
        zones.push((gauche, haut, droite, bas));
    }
    zones
}

/// Le rai de jour : de la fenêtre au sol, une bande de lumière à la couleur du ciel, penchée selon l'heure (le
/// soleil du matin la couche vers la droite, celui du soir vers la gauche), et sa poussière qui flotte. Au vrai
/// pixel, ajouté par-dessus tout ce qui est peint dans la pièce (borné à `bord`), comme `halos` ; rend le
/// rectangle touché.
#[allow(clippy::too_many_arguments)]
pub fn rai(px: &mut [u32], w: usize, h: usize, jour: ([f32; 3], f32), heure: f64, horloge: i64, x0: i32, cw: i32, ch: i32, bord: i32) -> Option<(i32, i32, i32, i32)> {
    let (rgb, force) = jour;
    if force <= 0.0 {
        return None;
    }
    // l'ouverture, au dedans du cadre ; la pente, en pixels de forge vers la droite par pixel vers le bas
    let (gauche, droite) = ((WINDOW.0 + 1) as f32, (WINDOW.0 + WINDOW.2 - 1) as f32);
    let (haut, sol) = ((WINDOW.1 + WINDOW.3) as f32, SH as f32);
    let pente = (((13.0 - heure) / 6.0).clamp(-1.0, 1.0) * 0.75) as f32;
    let (sx, sy) = (cw as f32, ch as f32 / 2.0);
    let (ex0, ex1) = (gauche.min(gauche + pente * (sol - haut)), droite.max(droite + pente * (sol - haut)));
    let l = (((ex0 - x0 as f32) * sx).floor() as i32 - cw).max(0);
    let r = (((ex1 - x0 as f32) * sx).ceil() as i32 + cw).min(bord.min(w as i32));
    let (t, b) = ((haut * sy) as i32, ((sol * sy) as i32).min(h as i32));
    if l >= r || t >= b {
        return None;
    }
    let plume = cw as f32 * 1.5;
    for y in t..b {
        // la profondeur dans le rai, de 0 à la fenêtre à 1 au sol : il pâlit en descendant
        let v = (y as f32 + 0.5) / sy - haut;
        let p = v / (sol - haut);
        let (a0, a1) = ((gauche + pente * v - x0 as f32) * sx, (droite + pente * v - x0 as f32) * sx);
        let k = force * 0.3 * (1.0 - 0.55 * p) * 255.0;
        for x in l..r {
            let xf = x as f32 + 0.5;
            let bordure = ((xf - a0) / plume).min((a1 - xf) / plume).clamp(0.0, 1.0);
            if bordure <= 0.0 {
                continue;
            }
            let i = y as usize * w + x as usize;
            let q = px[i];
            if q >> 24 == 0 {
                continue;
            }
            let f = k * bordure * bordure * (3.0 - 2.0 * bordure);
            let plus = |d: u32, c: f32| ((((q >> d) & 255) as f32 + c * f).min(255.0) as u32) << d;
            px[i] = (q & 0xff00_0000) | plus(16, rgb[0]) | plus(8, rgb[1]) | plus(0, rgb[2]);
        }
    }
    // la poussière : des grains qui descendent lentement en dérivant, chacun à sa place dans le rai
    let grain = (cw / 5).max(1);
    for g in 0..9i64 {
        let periode = 260 + (hasard(g * 31 + 7) * 200.0) as i64;
        let age = (horloge + (hasard(g * 13 + 3) * periode as f64) as i64).rem_euclid(periode) as f32 / periode as f32;
        let v = age * (sol - haut - 1.0);
        let u = hasard(g * 17 + 11) as f32 + 0.08 * (age * 6.283 * 2.0 + g as f32).sin();
        let gx = ((gauche + 1.0 + u * (droite - gauche - 2.0) + pente * v - x0 as f32) * sx) as i32;
        let gy = ((haut + v) * sy) as i32;
        // il paraît et s'efface au bout de son chemin
        let eclat = (age * 5.0).min((1.0 - age) * 5.0).min(1.0) * force * 0.9 * 255.0;
        for y in gy..gy + grain {
            for x in gx..gx + grain {
                if x < l || x >= r || y < t || y >= b {
                    continue;
                }
                let i = y as usize * w + x as usize;
                let q = px[i];
                if q >> 24 == 0 {
                    continue;
                }
                let plus = |d: u32, c: f32| ((((q >> d) & 255) as f32 + c * eclat).min(255.0) as u32) << d;
                px[i] = (q & 0xff00_0000) | plus(16, rgb[0]) | plus(8, rgb[1]) | plus(0, rgb[2]);
            }
        }
    }
    Some((l, t, r, b))
}
