//! La cour, dehors, derrière qui s'y tient (hooks/themes/forge-yard.ts) : le ciel de l'heure,
//! des nuages qui passent, deux chaînes de montagnes qui pâlissent au loin et des cascades sur
//! leurs flancs, une ligne de sapins ; et quand la fenêtre s'élargit, un arbre qui suit les
//! saisons, un puits, une roue de moulin. Peinte en vraies couleurs plutôt qu'en lettres.
//! La porte (montants de chêne, avant-toit, seuil) la sépare de la pièce.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::f64::consts::PI;

use super::monde::{Monde, PhaseRhum, Saison, Temps};
use super::pixels::{arrondi, hasard, Grille};

pub type Rgb = [f64; 3];

const fn hex(n: u32) -> Rgb {
    [((n >> 16) & 255) as f64, ((n >> 8) & 255) as f64, (n & 255) as f64]
}
fn mix(a: Rgb, b: Rgb, k: f64) -> Rgb {
    [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, a[2] + (b[2] - a[2]) * k]
}
fn mul(a: Rgb, m: Rgb) -> Rgb {
    [a[0] * m[0], a[1] * m[1], a[2] * m[2]]
}
fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}
fn smooth(a: f64, b: f64, v: f64) -> f64 {
    let t = clamp01((v - a) / (b - a));
    t * t * (3.0 - 2.0 * t)
}
/// Un bruit qui roule plutôt qu'il ne saute, le long d'un axe.
fn roll(x: f64, seed: i64) -> f64 {
    let i = x.floor();
    let f = x - i;
    let u = f * f * (3.0 - 2.0 * f);
    let i = i as i64;
    hasard(i * 7 + seed) * (1.0 - u) + hasard((i + 1) * 7 + seed) * u
}
fn byte(v: f64) -> u32 {
    arrondi(v).clamp(0.0, 255.0) as u32
}
pub fn pack(c: Rgb) -> u32 {
    (byte(c[0]) << 16) | (byte(c[1]) << 8) | byte(c[2])
}

const HEIGHT: i32 = 32;
const GROUND: i32 = 28; // le sol de la cour, à partir de cette rangée
const HORIZON: f64 = 25.0;

/// Les crêtes des deux chaînes à la colonne x, et où y commencent les sapins : la terre ne
/// bouge pas, elles se calculent une fois pour toutes les colonnes qu'une cour peut avoir.
fn terre(x: i32) -> (f64, f64, f64) {
    const DE: i32 = -16;
    static TABLE: std::sync::OnceLock<Vec<(f64, f64, f64)>> = std::sync::OnceLock::new();
    let calcul = |x: i32| {
        let xf = x as f64;
        let far = 8.0 + roll(xf / 9.0, 1) * 6.0 + roll(xf / 4.0, 2) * 2.0;
        let near = 15.0 + roll(xf / 6.0, 3) * 5.0;
        // un sapin toutes les cinq colonnes
        let fir = arrondi(xf / 5.0) as i32 * 5;
        let top = (27 - (5 + (hasard(fir as i64) * 4.0).floor() as i32)) as f64 + ((x - fir).abs() * 2) as f64;
        (far, near, top)
    };
    let table = TABLE.get_or_init(|| (DE..DE + 512).map(calcul).collect());
    table.get((x - DE) as usize).copied().unwrap_or_else(|| calcul(x))
}
fn far_ridge(x: i32) -> f64 {
    terre(x).0
}
fn near_ridge(x: i32) -> f64 {
    terre(x).1
}
fn fir_top(x: i32) -> f64 {
    terre(x).2
}

/// Les pentes des branches, du tronc vers l'extérieur.
const BOUGHS: [f64; 3] = [0.45, 1.0, 1.8];

/// L'heure et le temps dehors : les deux bouts du ciel, la lumière, la lueur de la forge à la porte.
#[derive(Clone, Copy)]
struct Ciel {
    top: Rgb,
    low: Rgb,
    amb: Rgb,
    spill: f64,
    night: bool,
    dusk: bool,
    storm: bool,
    flash: bool,
}

fn sky_of(w: &Monde) -> Ciel {
    let night = w.nuit();
    let dusk = !night && (w.hour < 8.0 || w.hour >= 18.0);
    let storm = w.weather == Some(Temps::Storm);
    // l'éclair, au rythme de la fenêtre
    let flash = storm && w.clock % 23 < 2;
    let (top, low, amb, spill) = if storm {
        if night {
            (hex(0x0d0f16), hex(0x262a36), [0.26, 0.29, 0.38], 0.5)
        } else {
            (hex(0x3a3f4c), hex(0x7a808c), [0.58, 0.61, 0.68], 0.25)
        }
    } else if night {
        (hex(0x070b22), hex(0x1d2c58), [0.3, 0.36, 0.6], 0.55)
    } else if dusk {
        (hex(0x34407e), hex(0xf3a35e), [0.92, 0.72, 0.62], 0.25)
    } else {
        (hex(0x4f9fe6), hex(0xc4e6f6), [1.0, 1.0, 0.97], 0.0)
    };
    if flash {
        return Ciel {
            top: mix(top, hex(0xc8ccdc), 0.5),
            low: mix(low, hex(0xe0e4f0), 0.5),
            amb: mix(amb, [1.0, 1.0, 1.1], 0.5),
            spill,
            night,
            dusk,
            storm,
            flash,
        };
    }
    Ciel { top, low, amb, spill, night, dusk, storm, flash }
}

/// Où se tiennent les choses dans une cour de la porte (`from`) au bord (`to`).
#[derive(Clone)]
pub struct Plan {
    pub wheel: Option<i32>,
    /// l'arbre : son tronc, le rayon de sa couronne
    pub tree: (i32, f64),
    pub well: Option<i32>,
    falls: Vec<(i32, bool)>,
}

fn place_all(from: i32, to: i32) -> Plan {
    let room = to - from;
    let wheel = if room >= 62 { Some(to - 11) } else { None };
    // une cour étroite n'a que le bord d'une couronne qui dépasse, haut à droite
    let tree = if room < 30 { (to + 1, 7.0) } else { (if wheel.is_some() { to - 30 } else { to - 9 }, 8.5) };
    let well = if room >= 44 { Some(tree.0 - 21) } else { None };
    // les cascades : les lointaines jaillissent des cols de la chaîne lointaine, ou d'un flanc,
    // là où elle domine bien la proche ; la proche descend la chaîne proche où elle montre le
    // plus de face au-dessus des sapins. Toutes loin du tronc et de la roue.
    let hidden = |x: i32| (x - tree.0).abs() <= 3 || wheel.is_some_and(|wh| x >= wh - 10);
    let around = |ridge: fn(i32) -> f64, x: i32, test: &dyn Fn(f64, f64) -> bool| {
        (1..=3).all(|k| test(ridge(x), ridge(x - k)) && test(ridge(x), ridge(x + k)))
    };
    let pick = |score: &dyn Fn(i32) -> Option<f64>, n: usize, apart: i32| {
        let mut spots: Vec<(i32, f64)> = Vec::new();
        for x in from + 7..to - 3 {
            if !hidden(x) {
                if let Some(v) = score(x) {
                    spots.push((x, v));
                }
            }
        }
        spots.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut chosen: Vec<i32> = Vec::new();
        for (x, _) in spots {
            if chosen.len() < n && chosen.iter().all(|c| (c - x).abs() >= apart) {
                chosen.push(x);
            }
        }
        chosen
    };
    let flank = |ridge: fn(i32) -> f64, x: i32| !around(ridge, x, &|r, k| r <= k + 0.6);
    let far = pick(
        &|x| {
            let gap = near_ridge(x) - far_ridge(x);
            if gap < 3.5 || !flank(far_ridge, x) {
                None
            } else {
                Some(gap + if around(far_ridge, x, &|r, k| r >= k) { 4.0 } else { 0.0 })
            }
        },
        (arrondi(room as f64 / 26.0) as usize).max(1),
        10,
    );
    let near = pick(
        &|x| {
            let face = fir_top(x) - near_ridge(x);
            if face < 3.0 || !flank(near_ridge, x) || far.iter().any(|&f| (f - x).abs() < 4 && f != x) {
                None
            } else {
                Some(face)
            }
        },
        1,
        0,
    );
    let falls = far.iter().map(|&x| (x, false)).chain(near.iter().map(|&x| (x, true))).collect();
    Plan { wheel, tree, well, falls }
}

/// Un pixel de la cour que la baie de la Route du Rhum peint dessous : les cellules l'y laissent transparent.
const BAIE: u32 = u32::MAX;

/// La cour immobile pour un ciel, une saison et une largeur : la couleur de chaque pixel, et
/// ceux qui lisent l'horloge, à repeindre à chaque image.
struct Immobile {
    colours: Vec<u32>,
    moving: Vec<u32>,
}

type Cle = (i32, i32, u8, bool, bool, bool, bool, u8);
const STILLS: usize = 12; // les derniers ciels et largeurs gardés ; l'heure et la fenêtre changent lentement

/// Ce que la cour garde d'une image à l'autre : ce qui ne bouge pas n'est jamais peint deux fois.
#[derive(Default)]
pub struct Cour {
    plans: HashMap<(i32, i32), Plan>,
    stills: HashMap<Cle, Immobile>,
    ordre: Vec<Cle>,
}

/// La cour d'une image : la couleur derrière les acteurs, et la lumière du dehors sur eux.
pub struct Dehors {
    from: i32,
    to: i32,
    colours: Vec<u32>,
    sky: Ciel,
}

impl Dehors {
    pub fn back(&self, x: i32, y: i32) -> Option<u32> {
        if x >= self.from && x < self.to && (0..HEIGHT).contains(&y) {
            self.colours.get((y * (self.to - self.from) + x - self.from) as usize).copied().filter(|&c| c != BAIE)
        } else {
            None
        }
    }

    pub fn light(&self, c: Rgb, x: i32, y: i32) -> Rgb {
        spill(&self.sky, self.from, x, y, mul(c, self.sky.amb))
    }
}

// la lueur de la forge par la porte, le soir et la nuit : une flaque chaude au sol
fn spill(sky: &Ciel, from: i32, x: i32, y: i32, c: Rgb) -> Rgb {
    if sky.spill <= 0.0 || y < 22 {
        return c;
    }
    let d = ((x - from) as f64 / 1.6).hypot((y - 31) as f64 * 1.4);
    let f = (1.0 - d / 14.0).max(0.0).powi(2) * sky.spill;
    [c[0] + 230.0 * f, c[1] + 135.0 * f, c[2] + 63.0 * f]
}

impl Cour {
    /// Les pixels repeints à chaque image, par cour gardée (pour le chrono).
    #[cfg(test)]
    pub fn mobiles(&self) -> Vec<(usize, usize)> {
        self.stills.values().map(|s| (s.moving.len(), s.colours.len())).collect()
    }

    pub fn layout(&mut self, from: i32, to: i32) -> &Plan {
        self.plans.entry((from, to)).or_insert_with(|| place_all(from, to))
    }

    /// La cour pour une image. `working` : la roue tourne plus vite.
    pub fn outdoors(&mut self, w: &Monde, working: bool, from: i32, to: i32) -> Dehors {
        let sky = sky_of(w);
        let se = w.saison();
        let plan = self.layout(from, to).clone();
        let width = (to - from).max(0);
        let p = Peintre {
            sky,
            se,
            rhum: w.rhum,
            winter: se == Saison::Hiver,
            clock: w.clock,
            working,
            from,
            to,
            plan,
            moved: Cell::new(false),
            broad: RefCell::new(vec![f64::NAN; (HEIGHT * width) as usize]),
            fine: RefCell::new(vec![f64::NAN; (HEIGHT * width) as usize]),
            pouring: RefCell::new(None),
        };
        // peinte entière une fois par ciel, saison et largeur ; puis, image après image, ce qui bouge
        let key: Cle = (from, to, se as u8, sky.night, sky.dusk, sky.storm, sky.flash, w.rhum.map_or(0, |r| r as u8 + 1));
        let mut colours = vec![0u32; (width * HEIGHT) as usize];
        match self.stills.get(&key) {
            None => {
                let mut moving = Vec::new();
                let mut i = 0usize;
                for y in 0..HEIGHT {
                    for x in from..to {
                        p.moved.set(false);
                        colours[i] = p.paint(x, y);
                        if p.moved.get() {
                            moving.push(i as u32);
                        }
                        i += 1;
                    }
                }
                if self.stills.len() >= STILLS && !self.ordre.is_empty() {
                    let vieux = self.ordre.remove(0);
                    self.stills.remove(&vieux);
                }
                self.stills.insert(key, Immobile { colours: colours.clone(), moving });
                self.ordre.push(key);
            }
            Some(still) => {
                colours.copy_from_slice(&still.colours);
                for &i in &still.moving {
                    let i = i as i32;
                    colours[i as usize] = p.paint(from + i % width, i / width);
                }
            }
        }
        Dehors { from, to, colours, sky }
    }
}

/// Ce qui peint un pixel de la cour, pour une image.
struct Peintre {
    sky: Ciel,
    se: Saison,
    rhum: Option<PhaseRhum>,
    winter: bool,
    clock: i64,
    working: bool,
    from: i32,
    to: i32,
    plan: Plan,
    /// le pixel a lu l'horloge : il bouge
    moved: Cell<bool>,
    broad: RefCell<Vec<f64>>,
    fine: RefCell<Vec<f64>>,
    pouring: RefCell<Option<Vec<(i32, f64, f64)>>>,
}

const CY: f64 = 19.0;
const R: f64 = 7.0;
const STEP: f64 = 2.0 * PI / 10.0;

/// Jusqu'où descend un angle sur la roue : 0 à la couronne, π au pied ; hors de [0, π] de l'autre côté.
fn down(th: f64) -> f64 {
    if th < 0.0 {
        -PI / 2.0 - th
    } else {
        1.5 * PI - th
    }
}
/// Combien les godets sont pleins : remplis à la couronne, pleins le long du flanc, vidés passé le bas.
fn fill(q: f64) -> f64 {
    smooth(0.05, 0.45, q) * (1.0 - smooth(1.5, 2.3, q))
}

impl Peintre {
    /// L'horloge, lue seulement par ici : un pixel qui ne l'appelle jamais reste immobile.
    fn now(&self) -> f64 {
        self.moved.set(true);
        self.clock as f64
    }

    /// Une couleur sous la lumière de cette image, pâlie de `far` vers le bas du ciel.
    fn lit(&self, c: Rgb, far: f64) -> Rgb {
        mix(mul(c, self.sky.amb), self.sky.low, far)
    }

    /// De l'eau qui tombe en (x, y), vue à travers `alpha` : des traînées d'écume qui descendent
    /// de `speed` pixels par tic ; de la glace immobile l'hiver.
    #[allow(clippy::too_many_arguments)]
    fn falling(&self, c: Rgb, x: i32, y: i32, speed: f64, alpha: f64, far: f64, white: bool, drips: bool) -> Rgb {
        if self.winter {
            let k = hasard((x * 3 + y.div_euclid(2) * 7) as i64);
            return mix(c, self.lit(if k > 0.5 { hex(0xe4f2fa) } else { hex(0xbfe0f0) }, far), alpha);
        }
        let k = hasard(x as i64 * 3 + ((y as f64 - self.now() * speed) / 2.0).floor() as i64 * 7);
        if drips && k < 0.45 {
            return c; // des filets qui se brisent en gouttes
        }
        let teinte = if k > if white { 0.35 } else { 0.65 } {
            hex(0xf2fbff)
        } else if white {
            hex(0xb4e2f6)
        } else {
            hex(0x4fb0f0)
        };
        mix(c, self.lit(teinte, far), alpha)
    }

    fn jog(&self, y: i32, seed: i64) -> i32 {
        arrondi((roll(y as f64 / 3.0, seed) - 0.5) * 3.2) as i32
    }

    /// Les cascades : des filets qui jaillissent des cols lointains, pâlis par la distance, et
    /// une plus large sur la chaîne proche. Aucune ne tombe tout droit : elle fait un pas de côté
    /// à ses corniches, une fente d'ombre à sa gauche.
    fn falls_at(&self, x: i32, y: i32, mut c: Rgb, near: bool) -> Rgb {
        for &(fx, fnear) in &self.plan.falls {
            if near != fnear || x < fx - 3 || x > fx + 4 {
                continue;
            }
            let top = if near { near_ridge(fx) } else { far_ridge(fx) } + 1.5;
            if (y as f64) < top - 1.0 || y > 26 {
                continue;
            }
            let at = fx + self.jog(y, fx as i64 * 7 + if near { 3 } else { 0 });
            let far = if near { 0.12 } else { 0.3 };
            let speed = if near { 1.2 } else { 0.7 };
            if (y as f64) < top {
                if x == at {
                    c = mul(c, [0.75, 0.75, 0.8]); // la fente d'où elle jaillit
                }
            } else if x == at - 1 {
                c = mul(c, [0.82, 0.82, 0.86]);
            } else if x == at {
                c = self.falling(c, x, y + fx, speed, 1.0, far, true, false);
            } else if x == at + 1 {
                c = self.falling(c, x, y + fx + 3, speed, if near { 0.9 } else { 0.55 }, far, true, false);
            } else if near && x == at + 2 {
                c = self.falling(c, x, y + fx + 5, 1.2, 0.4, far, true, false);
            }
        }
        c
    }

    /// Et l'embrun où tombe la proche, parmi les pointes des sapins.
    fn mist(&self, x: i32, y: i32, c: Rgb) -> Rgb {
        let Some(&(sx, _)) = self.plan.falls.iter().find(|f| f.1) else { return c };
        let base = fir_top(sx);
        if self.winter || (y as f64 - base).abs() >= 1.3 {
            return c;
        }
        let at = (sx + self.jog(base as i32, sx as i64 * 7 + 3)) as f64 + 0.5;
        let d = ((x as f64 - at) / 2.6).hypot((y as f64 - base) / 1.3);
        if d < 1.0 {
            mix(c, self.lit(hex(0xf4f9fc), 0.2), (1.0 - d) * 0.6 * (0.7 + 0.3 * (self.now() / 3.0 + x as f64).sin()))
        } else {
            c
        }
    }

    /// Le bruit des nuages, qui roule sur un plan : gardé une fois par image, par rangée du plan.
    fn rolled(&self, known: &RefCell<Vec<f64>>, x: i32, along: f64, down: f64, seed: i64) -> f64 {
        let w = self.to - self.from;
        let j = down.floor();
        let f = down - j;
        let u = f * f * (3.0 - 2.0 * f);
        let j = j as i64;
        let i = (j * w as i64 + (x - self.from) as i64) as usize;
        let mut k = known.borrow_mut();
        let mut lire = |i: usize, s: i64| match k.get(i) {
            Some(v) if !v.is_nan() => *v,
            Some(_) => {
                let v = roll(along, s);
                k[i] = v;
                v
            }
            None => roll(along, s),
        };
        let a = lire(i, seed + j * 131);
        let b = lire(i + w as usize, seed + (j + 1) * 131);
        a * (1.0 - u) + b * u
    }

    fn sky_at(&self, x: i32, y: i32) -> Rgb {
        let sky = &self.sky;
        let yf = y as f64;
        let mut c = mix(sky.top, sky.low, clamp01(yf / HORIZON));
        if sky.night && !sky.storm && y < 18 && hasard((x * 97 + y * 13) as i64) > 0.975 {
            c = if hasard((x + y) as i64) > 0.6 { [250.0, 245.0, 220.0] } else { [170.0, 180.0, 220.0] };
        }
        // les nuages, de droite à gauche ; épais et rapides dans l'orage
        let speed = if sky.storm { 0.04 } else { 0.012 };
        let band = 1.0 - (yf - 6.0).abs() / 11.0;
        let thin = if sky.storm { 0.22 } else { 0.5 };
        if band <= thin {
            return c;
        }
        let t = self.now();
        let xf = x as f64;
        let d = (self.rolled(&self.broad, x, xf / 10.0 + t * speed, yf / 3.0, 11) * 0.65
            + self.rolled(&self.fine, x, xf / 5.0 + t * speed * 1.4, yf / 2.0, 23) * 0.35)
            * band;
        let a = smooth(thin, if sky.storm { 0.4 } else { 0.66 }, d);
        if a <= 0.0 {
            return c;
        }
        let under = clamp01((yf - 2.0) / 9.0); // un nuage est plus sombre dessous
        let cloud = if sky.storm {
            mix(hex(0x4e5260), hex(0x2c303c), under)
        } else if sky.night {
            mix(hex(0x2c3554), hex(0x1a2240), under)
        } else if sky.dusk {
            mix(hex(0xf2b8a6), hex(0x8a5a7a), under)
        } else {
            mix(hex(0xf6f8fb), hex(0xbcc6d4), under)
        };
        mix(c, if sky.flash { mix(cloud, [230.0, 232.0, 245.0], 0.4) } else { cloud }, a * if sky.night { 0.8 } else { 1.0 })
    }

    /// Deux chaînes et les sapins sur le ciel : chaque crête adoucie où elle traverse un pixel.
    fn hills(&self, x: i32, y: i32) -> Rgb {
        let yf = y as f64;
        if yf >= fir_top(x) || y >= 26 {
            return self.mist(x, y, self.lit(if self.winter && (x + y) % 3 == 0 { hex(0xdfe8f0) } else { hex(0x23402c) }, 0.05));
        }
        let near = near_ridge(x);
        let near_colour = || self.lit(if self.winter && yf < near + 1.0 { hex(0xc8d4e0) } else { hex(0x3e5560) }, 0.38);
        let mut c;
        if yf >= near {
            c = near_colour();
        } else {
            let far = far_ridge(x);
            // les hauts sommets gardent leur neige
            let far_colour =
                || self.lit(if yf < far + if self.winter { 4.0 } else { 1.5 } || y < 11 { hex(0xe8eef4) } else { hex(0x5d6e86) }, 0.6);
            c = if yf >= far {
                far_colour()
            } else if yf + 1.0 > far {
                mix(self.sky_at(x, y), far_colour(), yf + 1.0 - far)
            } else {
                self.sky_at(x, y)
            };
            c = self.falls_at(x, y, c, false);
            if yf + 1.0 > near {
                c = mix(c, near_colour(), yf + 1.0 - near);
            }
        }
        c = self.falls_at(x, y, c, true);
        self.mist(x, y, c)
    }

    fn ground(&self, x: i32, y: i32) -> Rgb {
        let path = x < self.from + 14; // des pavés depuis la porte, puis l'herbe
        if self.winter && !(path && x < self.from + 6) {
            return self.lit(if (x * 3 + y * 5) % 11 == 0 { hex(0xc9d6e6) } else { hex(0xeaf0f7) }, 0.0);
        }
        if path {
            let row = y % 2;
            let stone = (x + row * 2).div_euclid(4);
            let edge = (x + row * 2) % 4 == 0 || y == GROUND;
            let c = if edge {
                hex(0x4a4048)
            } else if hasard((stone * 13 + y) as i64) > 0.5 {
                hex(0x7a7078)
            } else {
                hex(0x6c6470)
            };
            return self.lit(c, 0.0);
        }
        if self.se == Saison::Automne && hasard((x * 17 + y * 3) as i64) > 0.9 {
            return self.lit(if hasard(x as i64) > 0.5 { hex(0xd06a22) } else { hex(0xe3a63c) }, 0.0);
        }
        let tuft = hasard((x * 31 + y * 7) as i64) > 0.8;
        let c = match (self.se, tuft) {
            (Saison::Automne, true) => 0x8a6a2a,
            (Saison::Automne, false) => 0x6f6f30,
            (Saison::Printemps, true) => 0x6fb04a,
            (Saison::Printemps, false) => 0x4f8a36,
            (_, true) => 0x3c6a2a,
            (_, false) => 0x4c7a34,
        };
        self.lit(hex(c), 0.0)
    }

    /// L'arbre : sa couronne se balance au vent (plus fort dans l'orage) et suit la saison.
    fn crown(&self, x: i32, y: i32) -> Option<Rgb> {
        let (tx, r) = self.plan.tree;
        let (xf, yf, txf) = (x as f64, y as f64, tx as f64);
        let ty = r + 0.5;
        let gust = if self.sky.storm { 1.4 } else { 0.6 };
        if (xf - txf).abs() >= (r + 2.5) / 0.85 + gust {
            return None;
        }
        // le balancement, lu seulement où il peut porter une feuille ou une branche sur ce pixel
        let give = clamp01((ty + r - yf) / (2.0 * r));
        let least = (xf - txf).abs() - gust * give - 1e-6;
        let still = give <= 0.0 || (least > r - 1.0 + 0.55 && (least * 0.85).hypot(yf - ty) >= r + 2.5);
        let sx = if still { xf } else { xf - (self.now() / if self.sky.storm { 3.0 } else { 7.0 } + yf / 3.0).sin() * gust * give };
        let leafy = yf < ty + r * 0.75 && ((sx - txf) * 0.85).hypot(yf - ty) < r + roll((yf - ty).atan2(sx - txf) * 3.0 + 5.0, 5) * 2.5;
        if leafy && !self.winter {
            let k = hasard(arrondi(sx) as i64 * 13 + y as i64 * 29);
            let leaf = match self.se {
                Saison::Automne => {
                    if k > 0.66 {
                        0xe07a22
                    } else if k > 0.33 {
                        0xc04a1a
                    } else {
                        0xe3a63c
                    }
                }
                Saison::Printemps => {
                    if k > 0.85 {
                        0xf6b8d0
                    } else if k > 0.4 {
                        0x6fb04a
                    } else {
                        0x4f8a36
                    }
                }
                _ => {
                    if k > 0.6 {
                        0x3c7a30
                    } else {
                        0x2f6228
                    }
                }
            };
            // à l'ombre loin de la lumière, qui vient de gauche comme celle de la pièce
            let shade = if sx > txf + 2.0 || yf > ty + 4.0 { 0.8 } else { 1.05 };
            return Some(self.lit(mul(hex(leaf), [shade; 3]), 0.0));
        }
        let trunk = (x - tx).abs() <= 1 && yf >= ty + 3.0 && y < GROUND + 2;
        // les branches : trois paires en éventail depuis le tronc, vues entières une fois les feuilles tombées
        let up = ty + 3.0 - yf;
        let off = (sx - txf).abs();
        let mut bough = false;
        if up >= 0.0 && yf > ty - r * 0.8 && off <= r - 1.0 {
            for (i, b) in BOUGHS.iter().enumerate() {
                if (off - up * b).abs() < 0.55 && up > i as f64 * 1.5 {
                    bough = true;
                    break;
                }
            }
        }
        if trunk || bough {
            return Some(self.lit(if x == tx - 1 { hex(0x6a4228) } else { hex(0x4a2c16) }, 0.0));
        }
        if self.winter && leafy && y % 3 == 0 && hasard((x * 5 + y) as i64) > 0.55 {
            return Some(self.lit(hex(0xeef2f8), 0.0)); // de la neige sur les brindilles nues
        }
        None
    }

    fn well_at(&self, x: i32, y: i32, c: Rgb) -> Rgb {
        let Some(well) = self.plan.well else { return c };
        if x < well || x >= well + 9 || y < 17 || y >= GROUND + 2 {
            return c;
        }
        let dx = x - well;
        let lit = |c| self.lit(c, 0.0);
        if y >= 24 {
            return lit(if (dx + (y % 2) * 2) % 3 == 0 { hex(0x4f4a55) } else { hex(0x8a8494) });
        }
        if y == 17 || (y == 18 && dx > 0 && dx < 8) {
            return lit(if self.winter { hex(0xeef2f8) } else { hex(0x8a3a24) });
        }
        if dx == 1 || dx == 7 {
            return lit(hex(0x5a3a1e));
        }
        if y == 20 && dx > 1 && dx < 7 {
            return lit(hex(0x7a4a24)); // le treuil
        }
        if dx == 4 && y > 20 && y < 23 {
            return lit(hex(0x8a90a0)); // la corde du seau
        }
        if (3..=5).contains(&dx) && y == 23 {
            return lit(hex(0x6a4228)); // le seau
        }
        c
    }

    // la roue : l'eau du bief à droite tombe dans les godets à sa couronne et descend avec eux
    // le flanc proche, débordant en passant le bas ; elle tourne plus vite pendant qu'il
    // travaille (elle mène le soufflet), et reste gelée l'hiver
    fn turn(&self) -> f64 {
        if self.winter {
            0.4
        } else {
            self.now() * if self.working { 0.22 } else { 0.09 }
        }
    }

    /// Les godets qui débordent maintenant : où est la lèvre de chacun, et combien il verse.
    fn spills(&self) -> Vec<(i32, f64, f64)> {
        let Some(wheel) = self.plan.wheel else { return Vec::new() };
        if self.winter {
            return Vec::new();
        }
        let turned = self.turn();
        self.pouring
            .borrow_mut()
            .get_or_insert_with(|| {
                (0..10)
                    .filter_map(|k| {
                        let a = k as f64 * STEP - turned;
                        let th = a.sin().atan2(a.cos());
                        let q = down(th);
                        let f = smooth(1.3, 1.6, q) * (1.0 - smooth(2.1, 2.4, q));
                        (f > 0.05).then(|| (arrondi(wheel as f64 + (R + 1.6) * th.cos()) as i32, CY + (R + 1.6) * th.sin(), f))
                    })
                    .collect()
            })
            .clone()
    }

    /// L'étang sous la roue, et le ruisseau qu'il laisse filer vers nous, qui s'élargit en venant.
    fn wet(&self, x: i32, y: i32) -> bool {
        let Some(wheel) = self.plan.wheel else { return false };
        let dx = x - wheel;
        match y - GROUND {
            -1 => dx >= -8,
            0 => dx >= -10,
            1 => (-13..=-8).contains(&dx),
            2 => (-15..=-10).contains(&dx),
            3 => (-17..=-11).contains(&dx),
            _ => false,
        }
    }

    /// Ses rives (herbe mouillée, un galet ici et là), et ses roseaux qui plient au vent.
    fn shore(&self, x: i32, y: i32, c: Rgb) -> Option<Rgb> {
        let wheel = self.plan.wheel?;
        let dx = x - wheel;
        for (rx, h) in [(-11, 4), (-13, 3), (-7, 2)] {
            let base = if rx == -7 { GROUND + 1 } else { GROUND };
            if dx == rx && y <= base && y > base - h {
                return Some(self.lit(if self.winter { hex(0xb8a57c) } else { hex(0x3e6a2a) }, 0.0));
            }
            // une tête se balance d'un pixel au plus de chaque côté : le vent n'est lu que si près
            if h < 3 || (dx - rx).abs() > 1 || (y != base - h && y != base - h - 1) {
                continue;
            }
            let sway = if self.sky.storm {
                arrondi((self.now() / 2.0 + rx as f64).sin()) as i32
            } else {
                arrondi((self.now() / 6.0 + rx as f64).sin() * 0.6) as i32
            };
            if dx == rx + sway {
                return Some(self.lit(if self.winter && y == base - h - 1 { hex(0xeef2f8) } else { hex(0x6a4022) }, 0.0));
            }
        }
        if y < GROUND || self.wet(x, y) {
            return None;
        }
        if self.wet(x - 1, y) || self.wet(x + 1, y) || self.wet(x, y - 1) || self.wet(x, y + 1) {
            if self.winter {
                return Some(self.lit(hex(0xdde6f0), 0.0));
            }
            return Some(if hasard((x * 13 + y * 7) as i64) > 0.8 { self.lit(hex(0x8a8494), 0.0) } else { mul(c, [0.72, 0.74, 0.7]) });
        }
        None
    }

    fn wheel_at(&self, x: i32, y: i32, mut c: Rgb) -> Rgb {
        let Some(cx) = self.plan.wheel else { return c };
        let (dxf, dyf) = ((x - cx) as f64, y as f64 - CY);
        let d = dxf.hypot(dyf);
        if d > R + 2.0 {
            return c;
        }
        if d < 1.5 {
            return self.lit(hex(0x4a4e5c), 0.0);
        }
        // l'eau arrive du bief par la droite sur son haut : le haut va à gauche
        let th = dyf.atan2(dxf);
        let ang = th + self.turn();
        let near = |n: f64| {
            let s = 2.0 * PI / n;
            let a = ang.rem_euclid(s);
            a.min(s - a) * d
        };
        // dessinée aux bords doux, pour qu'elle tourne en douceur plutôt que par pixels entiers
        let paddle = if d > R && d < R + 2.0 { clamp01(1.5 - near(10.0) / 0.7) } else { 0.0 };
        let rim = clamp01(1.6 - (d - R).abs() / 0.6);
        let inner = clamp01(1.2 - (d - R + 2.5).abs() / 0.5);
        let spoke = if d < R { clamp01(1.4 - near(6.0) / 0.6) } else { 0.0 };
        c = mix(c, self.lit(hex(0x3a2214), 0.0), spoke.max(inner * 0.8));
        // l'eau dans les godets, posée sur la face haute de chaque palette, qui tourne avec elle
        if !self.winter && d > R + 0.3 {
            let u = ang.rem_euclid(STEP);
            let depth = fill(down(th)) * STEP * 0.6;
            if u > 0.05 && u < 0.05 + depth {
                let k = hasard(arrondi(u * 20.0) as i64 + arrondi(d) as i64 * 3 + self.now() as i64);
                c = mix(c, self.lit(if k > 0.85 { hex(0xdff6ff) } else { hex(0x4fb0f0) }, 0.0), 0.9);
            }
        }
        c = mix(c, self.lit(hex(0x2a1a10), 0.0), paddle);
        mix(c, self.lit(hex(0x8a5a2e), 0.0), rim)
    }

    fn mill_at(&self, x: i32, y: i32, mut c: Rgb) -> Rgb {
        let Some(cx) = self.plan.wheel else { return c };
        if x < cx - 19 {
            return c;
        }
        let lit = |c| self.lit(c, 0.0);
        // le bief sur son chevalet, depuis l'au-delà du bord jusqu'au-dessus de la roue
        if x == self.to - 2 && y > 11 && y < GROUND {
            c = lit(hex(0x5a3a1e));
        }
        if y == 11 && x >= cx - 1 {
            return lit(hex(0x7a4a24));
        }
        if y == 10 && x >= cx {
            if self.winter {
                return lit(hex(0xbfe0f0));
            }
            let k = hasard(((x as f64 + self.now() * 0.6).floor() as i64) * 5);
            return lit(if k > 0.75 { hex(0xdff6ff) } else { hex(0x7fd4ff) });
        }
        if self.winter && y == 12 && x >= cx && x % 2 == 0 {
            return lit(hex(0xcfe8f6)); // des glaçons
        }
        let under = || self.spills().into_iter().find(|s| (s.0 - x).abs() <= 1);
        if let Some(b) = self.shore(x, y, c) {
            return b;
        }
        // l'étang à son pied, blanc où tombe le débord
        if self.wet(x, y) {
            if self.winter {
                return lit(hex(0xd8ecf6));
            }
            let u = if y == GROUND - 1 { under() } else { None };
            if let Some(u) = u {
                if hasard(x as i64 * 7 + (self.now() * 13.0) as i64) < 0.3 + u.2 * 0.6 {
                    return lit(hex(0xf2fbff));
                }
            }
            if y > GROUND {
                // le ruisseau qui en sort vers nous, un peu plus vite au milieu
                let u = (((y - x) as f64) / 2.0 - self.now() * 0.45).floor() as i64;
                return if hasard(u * 7 + ((x + y) % 3) as i64) > 0.82 {
                    lit(hex(0xdff6ff))
                } else {
                    mul(lit(hex(0x5fb4e0)), [0.86, 0.92, 1.0])
                };
            }
            let k = hasard(((x as f64 + self.now() * 0.3).floor() as i64) * 11 + y as i64);
            return if k > 0.85 { lit(hex(0xdff6ff)) } else { mul(lit(hex(0x5fb4e0)), [0.9, 0.95, 1.0]) };
        }
        // les éclaboussures qui en sautent, d'un pixel de haut
        if y == GROUND - 2 {
            if let Some(u) = under() {
                if hasard(x as i64 * 5 + (self.now() * 17.0) as i64) > 1.0 - u.2 * 0.4 {
                    return mix(c, lit(hex(0xeef8ff)), 0.8);
                }
            }
        }
        c = self.wheel_at(x, y, c);
        if self.winter {
            return c;
        }
        // du bec du bief dans le godet qui passe dessous
        if (y == 10 && x == cx - 1) || (x == cx - 2 && (y == 11 || y == 12)) {
            return self.falling(c, x, y, 1.6, 0.9, 0.0, false, false);
        }
        // ce que lâchent les godets qui débordent, tout droit vers l'étang en filets
        if ((x - cx) as f64).abs() <= R + 2.0 && (y as f64) > CY - R - 2.0 && y < GROUND - 1 {
            for s in self.spills() {
                if x == s.0 && y as f64 > s.1 {
                    return self.falling(c, x, y + x * 2, 1.6, 0.9 * s.2, 0.0, false, true);
                }
            }
        }
        c
    }

    /// La porte : deux montants de chêne, l'avant-toit sur la cour, le seuil de pierre. Sous
    /// l'orage la pluie ruisselle et goutte du bout de l'avant-toit ; l'hiver, des glaçons dessous.
    fn door(&self, x: i32, y: i32) -> Option<Rgb> {
        let from = self.from;
        let eave_end = from + 12;
        let eave = (x - from).div_euclid(3);
        let lit = |c| Some(self.lit(c, 0.0));
        if x <= eave_end {
            if y >= eave && y <= eave + 2 {
                return lit(if y == eave {
                    if self.winter {
                        hex(0xeef2f8)
                    } else {
                        hex(0x8a8494)
                    }
                } else if y == eave + 1 {
                    hex(0x5d5866)
                } else {
                    hex(0x3a2418)
                });
            }
            if self.winter && y == eave + 3 && x > from && x % 2 == 0 {
                return lit(hex(0xcfe8f6));
            }
            // l'eau sur les ardoises, qui descend la pente
            if self.sky.storm && y == eave - 1 && x > from && (x - (self.now() / 2.0).floor() as i32).rem_euclid(3) == 0 {
                return lit(hex(0x7fd4ff));
            }
        }
        if self.sky.storm && x == eave_end {
            // et du bout, goutte après goutte, jusqu'au sol
            let lip = (eave_end - from).div_euclid(3) + 3;
            for k in 0..2 {
                if y == lip + (((self.now() * 1.5).floor() as i32 + k * 9) % (GROUND - lip)) {
                    return lit(hex(0x9fdcff));
                }
            }
        }
        if x <= from + 1 && y >= 2 {
            return lit(if x == from { hex(0x4a2c16) } else { hex(0x6a4228) });
        }
        if y >= 30 && x <= from + 2 {
            return lit(if y == 30 { hex(0x8a8494) } else { hex(0x5d5866) });
        }
        None
    }

    fn paint(&self, x: i32, y: i32) -> u32 {
        let c = match self.door(x, y) {
            Some(c) => c,
            // la Route du Rhum : la baie se peint dessous, au vrai pixel (baie.rs)
            None if self.rhum.is_some() => return BAIE,
            None => {
                // sous les feuilles, ni ciel ni collines ne se peignent
                let c = self.crown(x, y).unwrap_or_else(|| if y >= GROUND { self.ground(x, y) } else { self.hills(x, y) });
                self.mill_at(x, y, self.well_at(x, y, c))
            }
        };
        pack(spill(&self.sky, self.from, x, y, c))
    }
}

/// Ce qui tombe sur la cour, en glyphes plus fins qu'un pixel et sur ses cellules claires
/// seulement : la pluie en quarts, la neige, les feuilles et les pétales en braille.
pub fn draw_yard_weather(cour: &mut Cour, g: &mut Grille, w: &Monde, from: i32, to: i32) {
    let t = w.clock as f64;
    let room = to - from - 2;
    let se = w.saison();
    if room <= 0 {
        return;
    }
    let (fromf, roomf) = (from as f64, room as f64);
    let clair: &[u8] = b".";
    if w.weather == Some(Temps::Storm) {
        for i in 0..arrondi(roomf * 0.45) as i64 {
            // un trait d'un demi-pixel de large, deux de long, penché par le vent
            let qy = (t * 1.5 + (i * 5) as f64) % 30.0;
            let qx = 2.0 * (fromf + 2.0 + hasard(i * 13) * roomf) - qy * 0.3;
            g.quad(qx, qy, b'c', Some(clair));
            g.quad(qx, qy - 1.0, b'c', Some(clair));
        }
    }
    if se == Saison::Hiver {
        for i in 0..arrondi(roomf * 0.2) as i64 {
            let fx = fromf + 2.0 + hasard(i * 13) * roomf + ((t + (i * 7) as f64) / 3.0).sin() * 0.7;
            let fy = (t * (0.35 + 0.3 * hasard(i * 3)) + (i * 5) as f64) % 30.0;
            g.dot(2.0 * fx, 2.0 * fy, b'w', Some(clair));
        }
    }
    if (se == Saison::Automne || se == Saison::Printemps) && w.rhum.is_none() {
        // au bord de la mer, pas d'arbre ; ailleurs, de la couronne de l'arbre jusqu'à l'herbe, en tournoyant, le vent les prenant à gauche
        let (tx, tr) = cour.layout(from, to).tree;
        let colours = if se == Saison::Automne { b"FyF" } else { b"MMi" };
        for i in 0..3i64 {
            let vitesse = if se == Saison::Automne { 0.3 } else { 0.25 };
            let fall = (t * vitesse + (i * 9) as f64) % 24.0;
            let graine = i * 13 + ((t * 0.3 + (i * 9) as f64) / 24.0).floor() as i64;
            let fx = 2.0 * (tx as f64 - tr + hasard(graine) * tr * 2.0 - fall * 0.5 + ((t + (i * 7) as f64) / 3.0).sin());
            let fy = 2.0 * (6.0 + fall);
            let forme: [(f64, f64); 3] = if ((t / 3.0).floor() as i64 + i) % 2 != 0 {
                [(0.0, 0.0), (1.0, 1.0), (2.0, 1.0)]
            } else {
                [(0.0, 1.0), (1.0, 0.0), (2.0, 0.0)]
            };
            for (dx, dy) in forme {
                g.dot(fx + dx, fy + dy, colours[i as usize], Some(clair));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pour_la_route_du_rhum_la_cour_laisse_la_place_a_la_baie() {
        let (from, to) = (54, 144);
        let mut w = Monde { month: 11, day: 2, hour: 14.0, ..Monde::default() };
        let mut cour = Cour::default();
        let montagnes = cour.outdoors(&w, false, from, to);
        w.rhum = Some(PhaseRhum::Course);
        let baie = cour.outdoors(&w, false, from, to);
        assert!(montagnes.back(from + 40, 20).is_some());
        assert_eq!(baie.back(from + 40, 20), None, "la baie se peint dessous");
        assert_eq!(baie.back(from, 20), montagnes.back(from, 20), "la porte reste");
    }
}
