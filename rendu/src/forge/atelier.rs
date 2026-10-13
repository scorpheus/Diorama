//! L'atelier et son forgeron (hooks/themes/forge.ts) : la pièce, Scorpheus, les effets du
//! moment, les invités, les louveteaux dans la cour.

use super::accessoires::{
    corbeau, draw_dream, draw_sky, draw_sparks, draw_zzz, etincelles, Vitres, BUBBLE, CLAWS, PUFF, SAYING, THOUGHT,
};
use super::cour::{draw_yard_weather, Cour};
use super::monde::{
    Accessoire, Activite, Arme, Banniere, Cadeau, Chat, Don, Fete, Humeur, Marteau, Messager, ModeChat, Monde, Nouvelle,
    Petit, Saison, Saveur, Temps,
};
use super::pixels::{arr, hasard, Grille};
use super::poses::{pose_of, Caisse, Droite, Gauche, Lame, Lingot, Pose, Yeux};

pub const SW: i32 = 72; // la largeur de la scène : la forge, puis la cour des apprentis
pub const YARD: i32 = 54; // où commence la cour, passé la bannière
pub const GROW: i32 = 72; // ce que la cour s'élargit au plus quand la fenêtre a de la place
pub const SH: i32 = 32; // la hauteur de la scène, deux pixels par rangée
pub const ROWS: i32 = SH / 2;

pub const GUEST_SPOT: i32 = 26;
const GUEST_STEP: i32 = 19;

/// Ce que compose fixe pour un dessin et que les fonctions de dessin lisent : le bord droit
/// de la scène, la tête du marteau, la cape de la session.
pub struct Contexte {
    pub edge: i32,
    /// Scorpheus est peint d'après son pet (pet.rs) : la scène ne dessine que ce qui l'entoure
    pub pet: bool,
    tete: &'static [&'static str],
    cape: [u8; 2],
}

impl Default for Contexte {
    fn default() -> Self {
        Contexte { edge: SW, pet: false, tete: &HAMMER_NORMAL, cape: *b"oO" }
    }
}

// --- la distribution ------------------------------------------------------------

// Le crâne sans ses traits : ils se dessinent dessus, glissés de côté quand il tourne la tête.
const SKULL: [&str; 14] = [
    ".k............k.",
    "kwk.....k....kwk",
    "kgwk..nkbk..kwgk",
    "kgdwknBbbBnkkdgk",
    "kgdlnBbBbBBnldgk",
    ".kglnBbBBbnBlgk.",
    ".klllnBnBnBllldk",
    "kglllllnlnllllgk",
    "kgllllllllllllgk",
    "kllllllllllllllk",
    "kllllllllllllllk",
    "kgllllllllllllgk",
    ".kgllllllllllgk.",
    ".kgllllllllllgkB",
];

// Le museau, la truffe, la lune d'or et les écailles, de face.
const MARKS: [(i32, i32, u8); 25] = [
    (5, 11, b'w'),
    (6, 11, b'w'),
    (7, 11, b'm'),
    (8, 11, b'm'),
    (9, 11, b'w'),
    (10, 11, b'w'),
    (5, 12, b'w'),
    (6, 12, b'w'),
    (7, 12, b'w'),
    (8, 12, b'w'),
    (9, 12, b'w'),
    (10, 12, b'w'),
    (6, 13, b'w'),
    (7, 13, b'w'),
    (8, 13, b'w'),
    (9, 13, b'w'),
    (14, 8, b'y'),
    (14, 9, b'y'),
    (14, 10, b'y'),
    (11, 11, b'y'),
    (12, 11, b'y'),
    (13, 11, b'y'),
    (1, 10, b't'),
    (2, 11, b't'),
    (12, 12, b't'),
];

const BODY: [&str; 12] = [
    ".oOkkllwwllkkOoB", // le col de la capuche
    "..koOrwwwwrOok..",
    "..koOOrwwrOOok..",
    "..koOOOrrOOOok..",
    "..krrrryyrrrrk..", // la ceinture, la boucle d'or
    "..kOrrkOOkrrOk..", // les bourses
    "..koOOkOOkOOok..",
    "...kwwwkkwwwk...", // la fourrure
    "...kwlwkkwlwk...",
    "...kglgkkglgk...",
    "..kaaaakkaaaak..", // les bottes
    "..kkkkk..kkkkk..",
];

// Assis sur son tabouret : la cape jusqu'aux bourses, puis les jambes repliées.
const BODY_SEATED: [&str; 8] = [
    ".oOkkllwwllkkOoB",
    "..koOrwwwwrOok..",
    "..koOOrwwrOOok..",
    "..koOOOrrOOOok..",
    "..krrrryyrrrrk..",
    "..kOrrkOOkrrOk..",
    "..koOOkOOkOOok..",
    "..kwwwwkkwwwwk..",
];

// La queue, blanche à la bande bleue et à la rayure d'or ; un balancement en quatre temps.
const TAIL: [[&str; 11]; 4] = [
    ["....kk.", "...kwwk", "..kwbwk", "..kwBbk", ".kwwBbk", ".kwyBwk", "kwwyBwk", "kwwBbwk", ".kwbwk.", ".kwwk..", "..kk..."],
    [".....kk", "....kwk", "...kwbk", "..kwBbk", ".kwwBbk", ".kwyBwk", "kwwyBwk", "kwwBbwk", ".kwbwk.", ".kwwk..", "..kk..."],
    ["....kk.", "...kwwk", "..kwbwk", "..kwBbk", ".kwwBbk", ".kwyBwk", "kwwyBwk", "kwwBbwk", ".kwbwk.", ".kwwk..", "..kk..."],
    ["..kk...", ".kwwk..", ".kwbwk.", "..kwBbk", ".kwwBbk", ".kwyBwk", "kwwyBwk", "kwwBbwk", ".kwbwk.", ".kwwk..", "..kk..."],
];

const FURNACE: [&str; 23] = [
    "....kkkkk......",
    "....kQqqk......",
    "....kqQqk......",
    "....kqqQk......",
    "....kQqqk......",
    "..kkkkkkkkkk...",
    ".kQqqQqqqQqqk..",
    "kqqzqqqzqqqzqk.",
    "kqQqqqQqqqQqqk.",
    "kzqqqzqqqzqqzk.",
    "kqqkkkkkkkkqqk.",
    "kqkzzzzzzzzkqk.",
    "kqkzzzzzzzzkqk.",
    "kqkzzzzzzzzkqk.",
    "kqkzzzzzzzzkqk.",
    "kqkzzzzzzzzkqk.",
    "kqkzzzzzzzzkqk.",
    "kqkzzzzzzzzkqk.",
    "kqkzzzzzzzzkqk.",
    "kqkzzzzzzzzkqk.", // le foyer ; sa rangée du bas tient les bûches
    "kqqkkkkkkkkqqk.",
    "kQqqzqqqzqqqQk.",
    "kkkkkkkkkkkkkk.",
];
const FURNACE_AT: (i32, i32) = (0, 8);
pub(super) const MOUTH: (i32, i32, i32, i32) = (3, 18, 8, 8); // la chambre du feu : x, y, w, h
const LOGS_Y: i32 = 26; // le sol du foyer : une bûche par 12,5 % de la fenêtre de 5 h qui reste
// Le tas de bois sur le toit, du bas : une bûche par 10 % de la semaine qui reste.
const WOODPILE: [(i32, i32); 10] = [(0, 12), (1, 12), (2, 12), (3, 12), (0, 11), (1, 11), (2, 11), (3, 11), (1, 10), (2, 10)];
// Fixé au mur droit du four, sa buse dans le foyer, son levier en haut à droite.
const BELLOWS: (i32, i32) = (13, 20);
const BELLOWS_SPOT: i32 = -8; // où se tient Scorpheus : à bout de bras, le soufflet en vue

const ANVIL: [&str; 7] =
    ["kkkkkkkkkkkkk", "kAAAAAAAAAAAk", ".kkaaaaaaakk.", "...kaaaaak...", "...kaaaaak...", "..kaaaaaaak..", ".kkkkkkkkkkk."];
const ANVIL_AT: (i32, i32) = (16, 25);
const INGOT: (i32, i32, i32) = (21, 24, 4);
pub(super) const IMPACT: (i32, i32) = (22, 23);

pub(super) const C: (i32, i32) = (30, 6); // la tête de Scorpheus, en haut à gauche

const SCROLL: [&str; 5] = ["kWWWWWWk", "kWVVVVWk", "kWWWWWWk", "kWVVVWWk", ".kkkkkk."];
const BUCKET: [&str; 5] = ["kcccccck", "kAAAAAAk", "krRrRrRk", "krRrRrRk", ".kkkkkk."];
const BUCKET_AT: (i32, i32) = (18, 27);
// La caisse du jour sur l'enclume, en planches claires aux coins d'acier, son couvercle dessus.
const CRATE_BOX: [&str; 3] = ["kArrRrrrAk", "kVrrrrRrVk", "kkkkkkkkkk"];
const LID_OPEN: [&str; 8] = ["kk", "kV", "kV", "kV", "kV", "kV", "kV", "kV"];
const LID_CLOSING: [&str; 3] = ["....kkkkkk", "kkkkVVVVVk", "kVVVkkkkkk"];
const LID_SHUT: [&str; 2] = ["kkkkkkkkkk", "kVVVVVVVVk"];
const CRATE_AT: (i32, i32) = (17, 19); // le bord, en haut à gauche ; la caisse pose sur l'enclume
const SEAL: (i32, i32) = (CRATE_AT.0 + 3, CRATE_AT.1 + 1); // la cire d'or sur la jointure du couvercle
const HOURGLASS_X: [i32; 3] = [17, 21, 25]; // sur l'enclume, de gauche à droite
const BELL_AT: (i32, i32) = (25, 0);

// Les débris qui s'entassent au sol à mesure que le contexte se remplit, dans l'ordre où ils tombent.
const SCRAPS: [(i32, i32, u8); 12] = [
    (15, 31, b'a'),
    (29, 31, b'A'),
    (16, 31, b'r'),
    (14, 30, b'E'),
    (28, 31, b'a'),
    (15, 30, b'R'),
    (17, 30, b'A'),
    (30, 31, b'r'),
    (27, 30, b'E'),
    (14, 31, b'a'),
    (16, 30, b'r'),
    (28, 30, b'a'),
];

/// L'or de la session, du bas, en quarts de bloc (deux par pixel) : le tas monte d'une marche
/// par marche du logarithme de la dépense, onze en tout.
const COINS: [(i32, i32); 30] = {
    let mut c = [(0, 0); 30];
    let mut n = 0;
    let rangs: [(i32, i32, i32); 5] = [(0, 9, 31), (1, 8, 30), (2, 7, 29), (3, 6, 28), (4, 5, 27)];
    let mut r = 0;
    while r < rangs.len() {
        let (de, a, y) = rangs[r];
        let mut i = de;
        while i <= a {
            c[n] = (98 + i, y);
            n += 1;
            i += 1;
        }
        r += 1;
    }
    c
};
const COIN_STEPS: i32 = 11;

// La fenêtre de l'atelier sur le monde, entre la cloche et le four.
pub(super) const WINDOW: (i32, i32, i32, i32) = (13, 0, 12, 10); // cadre compris : x, y, w, h
pub(super) const LANTERN: (i32, i32) = (10, 0);
const HAT_PARTY: [&str; 5] = ["..Y..", "..M..", ".MyM.", ".yMy.", "MMyMM"];
const HAT_SANTA: [&str; 5] = ["...w.", "..FF.", ".FFF.", "FFFFE", "wwwww"];
const HAMMER_SMALL: [&str; 2] = ["AA", "aa"];
const HAMMER_NORMAL: [&str; 2] = ["AAA", "aaa"];
const HAMMER_SLEDGE: [&str; 3] = ["AAAA", "aaaa", "aaaa"];

// Le râtelier de la semaine sous la fenêtre : une barre et sept chevilles, une arme par jour travaillé.
const RACK: (i32, i32) = (13, 11);
const RACK_SLOTS: [i32; 7] = [15, 17, 19, 21, 23, 25, 27];

fn arme(a: Arme) -> (&'static str, &'static [(i32, i32, u8)]) {
    match a {
        Arme::Sword => ("wAAAyr", &[(-1, 4, b'y'), (1, 4, b'y')]),
        Arme::Axe => ("AArrrr", &[(-1, 0, b'A'), (-1, 1, b'a')]),
        Arme::Dagger => ("..wAyr", &[]),
        Arme::Mace => ("Aarrrr", &[(-1, 0, b'a'), (1, 0, b'a')]),
    }
}

// La bannière du projet, à droite de Scorpheus.
const BANNER: (i32, i32) = (47, 8);
const DUCK: [&str; 4] = [".yy..", "Fky..", ".yyyy", "..yyy"];
const DUCK_AT: (i32, i32) = (25, 21);
const SLATE_AT: (i32, i32) = (18, 17);

// Un louveteau de profil, tourné à droite : pelage gris, ventre clair, long museau, la houppe
// bleue de son père, une écharpe à la couleur de sa sorte ('C' ici).
const CUB_BODY: [&str; 6] = ["........d.d.", ".......dgBd.", "dd....dggpgd", "dgdd.CCglwwm", ".dddCCgggww.", "..gwwwwwgl.."];
// aux pas de passage du trot, la queue plonge
const CUB_TAIL_LOW: [&str; 2] = ["d.....dggpgd", "dddd.CCglwwm"];
// un trot en quatre pas (pattes proches claires, lointaines dans l'ombre), puis assis au travail
const CUB_LEGS: [&str; 5] = ["..d.g..d.g..", "...gd...gd..", "..g.d..g.d..", "...dg...dg..", "..dddd.d.d.."];
const CUB_SIT: i64 = 4;
const CUB_W: i32 = 12;
const VISIT_EVERY: i64 = 3200; // tics entre deux visiteurs : huit minutes
const VISIT_FOR: i64 = 56;

// --- les morceaux --------------------------------------------------------------

/// arr[..n] comme slice(0, n) de JavaScript : n négatif compte depuis la fin.
fn premiers<T>(list: &[T], n: i64) -> &[T] {
    let len = list.len() as i64;
    let fin = if n < 0 { (len + n).max(0) } else { n.min(len) };
    &list[..fin as usize]
}

const EYES_OPEN: [&str; 3] = ["kk", "eh", "ee"];
const EYES_LOOK: [&str; 3] = ["kk", "he", "ee"];
const EYES_BLINK: [&str; 3] = ["ll", "ll", "kk"];
const EYES_HAPPY: [&str; 3] = ["ll", "kk", "ll"];

fn draw_fire(g: &mut Grille, level: i32, tick: i64) {
    let (mx, my, mw, mh) = MOUTH;
    for c in 0..mw {
        let centre = 1.0 - (c as f64 - (mw - 1) as f64 / 2.0).abs() / (mw as f64 / 2.0);
        let flicker = hasard(tick * 31 + c as i64 * 7) * 2.0 - 0.6;
        let h = if level == 0 {
            if hasard(tick * 13 + c as i64) > 0.7 { 1 } else { 0 }
        } else {
            arr(level as f64 * 2.0 * centre + level as f64 + flicker).min(mh).max(1)
        };
        for k in 0..h {
            let f = k as f64 / (h - 1).max(1) as f64;
            let ch = if level == 0 {
                b'E'
            } else if f < 0.25 {
                b'Y'
            } else if f < 0.55 {
                b'f'
            } else if f < 0.85 {
                b'F'
            } else {
                b'E'
            };
            g.put(mx + c, my + mh - 1 - k, ch);
        }
    }
}

fn draw_smoke(g: &mut Grille, every: i64, tick: i64) {
    if every <= 0 {
        return;
    }
    let top = FURNACE_AT.1 - 1;
    for born in tick - 10..=tick {
        if born < 0 || born % every != 0 {
            continue;
        }
        let age = tick - born;
        let drift = arr(hasard(born) * 2.0 - 1.0 + age as f64 * 0.3);
        let (x, y) = (FURNACE_AT.0 + 6 + drift, top - age as i32);
        let ch = if age < 3 { b'X' } else { b'x' };
        g.put(x, y, ch);
        g.put(x + 1, y, ch);
        if age > 1 && age < 7 {
            g.put(x, y - 1, b'x');
            g.put(x - 1, y, b'x');
        }
        if age > 3 && age < 8 {
            g.put(x + 2, y - 1, b'x');
        }
    }
}

fn draw_embers(g: &mut Grille, tick: i64) {
    for born in tick - 7..=tick {
        if born < 0 || hasard(born * 5) < 0.4 {
            continue;
        }
        let age = tick - born;
        let x = FURNACE_AT.0 + 5 + arr(hasard(born * 3) * 3.0 + ((age + born) as f64).sin() * 0.8);
        g.put(x, FURNACE_AT.1 - 1 - age as i32, if age < 3 { b'Y' } else if age < 5 { b'S' } else { b'E' });
    }
}

fn draw_steam(g: &mut Grille, age: i64, ox: i32, oy: i32) {
    for i in 0..6i64 {
        let a = age as f64 - i as f64 * 0.7;
        if !(0.0..=8.0).contains(&a) {
            continue;
        }
        let x = ox + arr((hasard(i * 13) * 2.0 - 1.0) * 3.0 + (a + i as f64).sin() * 0.7);
        let y = oy - arr(a * 1.2);
        g.put(x, y, if a < 3.0 { b'w' } else if a < 6.0 { b'X' } else { b'x' });
        if a < 5.0 {
            g.put(x + 1, y, b'X');
        }
    }
}

impl Contexte {
    fn arm(&self, g: &mut Grille, sx: i32, sy: i32, px: i32, py: i32) {
        if self.pet {
            return; // le pet a les siennes
        }
        g.line(sx, sy, px, py, self.cape[1]);
        g.line(sx, sy + 1, px, py + 1, self.cape[0]);
        g.put(px, py, b'l');
        g.put(px, py + 1, b'g');
    }

    /// Un dessin aux couleurs de la cape de la session : 'o' puis 'O' remplacés, dans cet ordre.
    fn dress(&self, rows: &[&str]) -> Vec<String> {
        rows.iter()
            .map(|r| {
                let a: Vec<u8> = r.bytes().map(|b| if b == b'o' { self.cape[0] } else { b }).collect();
                String::from_utf8(a.into_iter().map(|b| if b == b'O' { self.cape[1] } else { b }).collect()).unwrap_or_default()
            })
            .collect()
    }

    fn hammer(&self, g: &mut Grille, px: i32, py: i32, hx: i32, hy: i32) {
        if self.pet {
            return;
        }
        g.line(px, py, hx + 1, hy + 1, b'r');
        let w = self.tete.first().map_or(3, |r| r.len() as i32);
        g.stamp(hx - (w - 3).div_euclid(2), hy - (self.tete.len() as i32 - 2), self.tete);
    }
}

/// La fenêtre : le ciel selon l'heure, la saison, le temps.
fn draw_window(g: &mut Grille, w: &Monde, tick: i64) {
    let (x, y, ww, h) = WINDOW;
    draw_sky(g, w, tick, &Vitres { x: x + 1, y: y + 1, w: ww - 2, h: h - 2 }, true);
    // le cadre, une croix de meneaux, l'appui
    for i in 0..ww {
        g.put(x + i, y, b'q');
        g.put(x + i, y + h - 1, b'Q');
    }
    for j in 0..h {
        g.put(x, y + j, b'q');
        g.put(x + ww - 1, y + j, b'q');
    }
    for j in 1..h - 1 {
        g.put(x + ww / 2, y + j, b'q');
    }
    for i in 1..ww - 1 {
        g.put(x + i, y + h / 2, b'q');
    }
    if w.fete() == Some(Fete::Halloween) {
        g.stamp(x + ww - 4, y + h - 3, &[".r.", "FfF", "fYf"]); // la citrouille sur l'appui
    }
}

fn draw_lantern(g: &mut Grille, lit: bool, tick: i64) {
    let (lx, ly) = LANTERN;
    g.put(lx + 1, ly, b'a');
    g.put(lx + 1, ly + 1, b'a');
    let flame = if lit {
        if hasard(tick) > 0.5 { b'Y' } else { b'f' }
    } else {
        b'a'
    };
    let corps = [b'k', if lit { b'F' } else { b'a' }, b'k'];
    g.stamp(lx, ly + 2, &["kak"]);
    for (i, &ch) in [b'k', flame, b'k'].iter().enumerate() {
        g.put(lx + i as i32, ly + 3, ch);
    }
    for (i, &ch) in corps.iter().enumerate() {
        g.put(lx + i as i32, ly + 4, ch);
    }
    g.stamp(lx, ly + 5, &["kkk"]);
    if lit && tick % 4 < 2 {
        g.put(lx - 1, ly + 3, b'E');
        g.put(lx + 3, ly + 3, b'E');
    }
}

fn draw_rack(g: &mut Grille, w: &Monde) {
    if w.rack.is_empty() && !w.masterpiece {
        return;
    }
    let (rx, ry) = RACK;
    for i in rx..=rx + 15 {
        g.put(i, ry, b'R');
    }
    if w.masterpiece {
        // la place d'honneur : une grande épée d'or, qui accroche la lumière
        let x = rx;
        for (j, &ch) in b"YyyYyr".iter().enumerate() {
            g.put(x, ry + 1 + j as i32, ch);
        }
        g.put(x - 1, ry + 5, b'y');
        g.put(x + 1, ry + 5, b'y');
        if w.clock % 20 < 3 {
            g.put(x, ry + 1 + (w.clock % 4) as i32, b'w');
        }
        fine_weapon(g, Arme::Sword, x, b'Y', if w.clock % 20 < 3 { Some((w.clock % 4).div_euclid(2) as i32) } else { None });
    }
    let derniers = &w.rack[w.rack.len().saturating_sub(RACK_SLOTS.len())..];
    for (i, &weapon) in derniers.iter().enumerate() {
        let x = RACK_SLOTS[i];
        let hot = w.forging && i == derniers.len() - 1;
        let (shaft, side) = arme(weapon);
        for (j, ch) in shaft.bytes().enumerate() {
            g.put(x, ry + 1 + j as i32, if hot && (ch == b'A' || ch == b'w') { if j % 2 != 0 { b'H' } else { b'Y' } } else { ch });
        }
        for &(dx, dy, ch) in side {
            g.put(x + dx, ry + 1 + dy, if hot && ch == b'A' { b'H' } else { ch });
        }
        fine_weapon(g, weapon, x, if hot { b'H' } else { b'A' }, None);
    }
}

// Les lames du râtelier en traits de boîte sur leurs pixels : une pointe fine, la lame, une
// garde en travers de la poignée ; haches et masses gardent leur tête, sur un manche fin.
fn fine_weapon(g: &mut Grille, weapon: Arme, x: i32, steel: u8, glint: Option<i32>) {
    let cy = (RACK.1 + 1) / 2;
    let hilt = b'y';
    match weapon {
        Arme::Sword => {
            g.glyph(x, cy, '╽', if glint == Some(0) { b'w' } else { steel });
            g.glyph(x, cy + 1, '┃', if glint == Some(1) { b'w' } else if steel == b'Y' { b'y' } else { steel });
            g.glyph(x, cy + 2, '╁', hilt);
            g.glyph(x - 1, cy + 2, '╶', hilt);
            g.glyph(x + 1, cy + 2, '╴', hilt);
        }
        Arme::Dagger => {
            g.glyph(x, cy + 1, '╽', steel);
            g.glyph(x, cy + 2, '╁', hilt);
        }
        _ => {
            g.glyph(x, cy + 1, '│', b'r');
            g.glyph(x, cy + 2, '│', b'r');
        }
    }
}

fn draw_banner(g: &mut Grille, b: &Banniere, tick: i64) {
    let (x, y) = BANNER;
    let cloth = b.cloth.0;
    for i in -1..=6 {
        g.put(x + i, y, b'r'); // la hampe
    }
    for j in 1..=7 {
        for i in 0..6 {
            g.put(x + i, y + j, cloth);
        }
    }
    // l'emblème : dix bits, en miroir dans un signe de 3 × 5
    for j in 0..5 {
        for i in 0..2 {
            if (b.bits >> (j * 2 + i)) & 1 != 0 {
                g.put(x + 1 + i as i32, y + 2 + j as i32, b.emblem.0);
                g.put(x + 4 - i as i32, y + 2 + j as i32, b.emblem.0);
            }
        }
    }
    let flap = tick.div_euclid(8) % 2; // la queue d'aronde bouge dans le courant d'air de la forge
    let motif: [u8; 6] = if flap != 0 { [cloth, b'.', cloth, cloth, b'.', cloth] } else { [cloth, cloth, b'.', b'.', cloth, cloth] };
    for (i, &ch) in motif.iter().enumerate() {
        g.put(x + i as i32, y + 8, ch);
    }
}

fn draw_slate(g: &mut Grille, tick: i64) {
    let (x, y) = SLATE_AT;
    g.stamp(x, y, &["rrrrrrr", "rzzzzzr", "rzwwwzr", "rzwzwzr", "rzwwwzr", "rzzzzzr", "rrrrrrr"]);
    // l'aiguille de l'horloge à la craie tourne : il attend le charbon
    let hands = [(3, 2), (4, 3), (3, 4), (2, 3)];
    let (hx, hy) = hands[(tick.div_euclid(6) % 4) as usize];
    g.put(x + 3, y + 3, b'w');
    g.put(x + hx, y + hy, b'Y');
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Visiteur {
    Corbeau,
    Phalene,
}

/// Qui passe à ce tic, s'il passe quelqu'un : un corbeau le jour, une phalène la nuit.
pub fn visitor_of(w: &Monde, clock: i64) -> Option<Visiteur> {
    if clock < VISIT_EVERY || clock % VISIT_EVERY >= VISIT_FOR {
        return None;
    }
    Some(if w.nuit() { Visiteur::Phalene } else { Visiteur::Corbeau })
}

fn draw_visitor(ctx: &Contexte, g: &mut Grille, w: &Monde, clock: i64) {
    let Some(kind) = visitor_of(w, clock) else { return };
    let age = clock % VISIT_EVERY;
    if kind == Visiteur::Corbeau {
        let (x, y) = (ctx.edge - arr(age as f64 * 1.3), 1 + arr((age as f64 / 3.0).sin()));
        g.stamp(x, y, corbeau(age));
    } else {
        // une phalène autour de la lanterne
        let a = age as f64 * 0.6;
        g.put(LANTERN.0 + 1 + arr(a.cos() * 3.0), LANTERN.1 + 4 + arr(a.sin() * 2.0), if age % 2 != 0 { b'W' } else { b'X' });
    }
}

/// Jusqu'où Scorpheus peut aller : quelques pas vers le four, quelques-uns vers la bannière.
pub const ROAM: (i32, i32) = (-12, 5);

/// Le poste où un moment tient Scorpheus ; rien : il est libre d'aller.
pub fn anchor_of(activity: Activite, w: &Monde) -> Option<i32> {
    match activity {
        Activite::Running => Some(if w.flavor == Saveur::Build { BELLOWS_SPOT } else { 0 }),
        Activite::Quench | Activite::Ship | Activite::Failed => Some(0),
        Activite::Idle => {
            if w.routine == Some(super::monde::Routine::Lantern) {
                Some(ROAM.0)
            } else if w.routine == Some(super::monde::Routine::Lunch) || w.hourglasses > 0 || w.exhausted || w.tea || w.duck {
                Some(0)
            } else {
                None
            }
        }
        _ => None,
    }
}

// Le balai à sa droite, vers la cour : le manche sur l'épaule, la tête de paille au sol.
const BROOM_X: i32 = 20; // le pied du manche, depuis son bord gauche, au début d'un coup
const BROOM_HEAD: [&str; 4] = ["..RRR..", ".yRRRy.", "yyVyVyy", "yVyVyVy"];

impl Contexte {
    #[allow(clippy::too_many_arguments)]
    fn draw_broom(&self, g: &mut Grille, p: &Pose, x: i32, y: i32, ls: (i32, i32), rs: (i32, i32), back: bool) {
        // en revenant pour un autre passage il le porte, la tête bien levée du sol
        let reach = if back { 0 } else { p.sweep.unwrap_or(0) };
        let lift = if back {
            2
        } else if p.broom_up {
            1
        } else {
            0
        };
        let (bx, by) = (x + BROOM_X + reach, C.1 + 22 - lift); // le pied du manche, sur la ligature
        let (tx, ty) = (x + 14 + arr(reach as f64 / 2.0), y + 11); // son haut, passé l'épaule
        g.line(tx, ty, bx, by, b'r');
        let mut head: Vec<String> = BROOM_HEAD.iter().map(|r| r.to_string()).collect();
        if lift == 0 && p.dust.is_some() {
            let r = BROOM_HEAD[3];
            head[3] = format!("{}{}", &r[1..], &r[..1]); // poussée, la paille traîne
        }
        g.stamp_s(bx - 3, by, &head);
        // les pattes sur le manche, celle du haut sa gauche en travers de la poitrine
        let along = |f: f64| (arr(tx as f64 + f * (bx - tx) as f64), arr(ty as f64 + f * (by - ty) as f64));
        let ((ux, uy), (lx, ly)) = (along(0.2), along(0.5));
        self.arm(g, ls.0, ls.1, ux, uy);
        self.arm(g, rs.0, rs.1, lx, ly);
    }
}

/// La poussière qu'un coup lève devant la paille, et les débris qu'il envoie glisser vers la cour.
fn draw_dust(g: &mut Grille, fx: i32, age: i64) {
    let floor = C.1 + 25;
    const CLOUD: [(i32, i32); 14] =
        [(0, 0), (1, 0), (0, -1), (1, -1), (2, -1), (2, 0), (1, -2), (3, -1), (2, -2), (3, -2), (4, -1), (2, -3), (3, -3), (4, -2)];
    let drift = age.div_euclid(2) as i32;
    for (i, &(dx, dy)) in premiers(&CLOUD, (4 + age * 3).min(CLOUD.len() as i64)).iter().enumerate() {
        g.put(fx + dx + drift, floor + dy - drift, if age > 3 { b'x' } else if i < 6 { b'X' } else { b'x' });
    }
    // les débris balayés glissent devant, au ras du sol
    for (i, &ch) in b"arE".iter().enumerate() {
        g.put(fx + 1 + age as i32 * (i as i32 + 1) + i as i32, floor, ch);
    }
}

/// Une lame tenue par sa poignée en (px, py), pointe en haut (dir −1) ou en bas (dir 1).
fn blade(g: &mut Grille, px: i32, py: i32, dir: i32, state: Lame, tick: i64) {
    g.put(px - 1, py, b'r');
    g.put(px + 1, py, b'r');
    for i in 1..=5 {
        if state == Lame::Cracked && i == 5 {
            continue; // la pointe s'est cassée
        }
        let ch = match state {
            Lame::Hot => {
                if i > 3 {
                    b'Y'
                } else {
                    b'H'
                }
            }
            Lame::Cracked if i == 3 => b'E',
            _ => {
                if i as i64 == ((tick >> 1) % 5) + 1 {
                    b'w'
                } else {
                    b'A'
                }
            }
        };
        g.put(px, py + dir * i, ch);
    }
    if state == Lame::Cracked {
        g.put(px + 1, py + dir * 4, b'a'); // l'éclat
    }
}

// La cour des apprentis : le sol, pattes sur la dernière rangée.
const LANE: (i32, i32, i32) = (YARD - 8, SW - CUB_W, SH - 7); // de, à, y
const CUB_CYCLE: i64 = 40; // tics : 28 à marcher, puis 12 au marteau
const CUB_WALK: i64 = 28;
// Trois places dans la cour, en profondeur décalée ; un louveteau fait quelques pas autour de la sienne.
const SLOTS: [(i32, i32); 3] = [(0, 2), (7, 0), (14, 1)];
const ROAM_STEPS: i32 = 3;

struct Place {
    x: i32,
    y: i32,
    dir: i32,
    working: bool,
    travel: f64,
}

/// Où est un louveteau à un âge : il va et vient autour de sa place à son pas, s'arrêtant pour travailler.
fn cub_place(c: &Petit, slot: i64) -> Place {
    let place = if slot % 3 < 0 { (0, 0) } else { SLOTS[(slot % 3) as usize] };
    let pace = 0.3 + hasard(c.seed) * 0.3;
    let cycles = c.age.div_euclid(CUB_CYCLE);
    let walked = (cycles * CUB_WALK + (c.age % CUB_CYCLE).min(CUB_WALK)) as f64;
    let span = (ROAM_STEPS * 2) as f64;
    let d = (walked * pace + hasard(c.seed * 3) * span * 2.0) % (span * 2.0);
    let out = d < span;
    Place {
        x: LANE.0 + place.0 + arr((if out { d } else { span * 2.0 - d }) - ROAM_STEPS as f64),
        y: LANE.2 - place.1,
        dir: if out { 1 } else { -1 },
        working: c.age % CUB_CYCLE >= CUB_WALK,
        travel: walked * pace, // les pixels marchés : les pattes suivent le sol
    }
}

// Le chat de la machine : une panthère tachetée, or aux rosettes sombres, un œil vert.
const PANTHER_WALK: [[&str; 7]; 4] = [
    [".11...........", "1111.........1", "14111.......1.", "3311121121111.", ".111211121111.", ".1.5.....1.5..", "1...5...1...5."],
    [".11...........", "1111..........", "14111.......11", "3311121121111.", ".111211121111.", "..15......15..", "..51......51.."],
    [".11...........", "1111.........1", "14111.......1.", "3311121121111.", ".111211121111.", ".5.1.....5.1..", "5...1...5...1."],
    [".11...........", "1111..........", "14111.......11", "3311121121111.", ".111211121111.", "..51......51..", "..15......15.."],
];
const PANTHER_STAND: [&str; 7] =
    [".11...........", "1111.........1", "14111.......1.", "3311121121111.", ".111211121111.", ".15.......15..", ".15.......15.."];
const PANTHER_ASLEEP: [&str; 5] = ["..1.1....", ".1111..1.", "12111.1..", "311121211", ".1121211."];
pub const CAT_SILL: (i32, i32) = (15, 4); // sur l'appui de la fenêtre, devant la vitre
pub const CAT_FLOOR_Y: i32 = SH - 7;

fn draw_cat(g: &mut Grille, cat: &Chat) {
    if cat.mode == ModeChat::Roof {
        // roulé en boule sur l'appui, un z de temps en temps
        g.stamp(CAT_SILL.0, CAT_SILL.1, &PANTHER_ASLEEP);
        if cat.age % 40 < 12 {
            g.put(CAT_SILL.0 + 9, CAT_SILL.1 - (cat.age % 40).div_euclid(4) as i32, b'u');
        }
        return;
    }
    // ses pattes bougent quand il bouge : arrivé près de lui il se tient debout
    let frame: &[&str] = match cat.step {
        None => &PANTHER_STAND,
        Some(s) if s >= 0 => &PANTHER_WALK[(s % 4) as usize],
        Some(_) => &PANTHER_STAND,
    };
    let right = cat.dir.unwrap_or(if cat.mode == ModeChat::Flee { 1 } else { -1 }) > 0;
    if right {
        g.stamp_s(cat.x, CAT_FLOOR_Y, &super::pixels::mirror(frame));
    } else {
        g.stamp(cat.x, CAT_FLOOR_Y, frame);
    }
}

fn draw_gift(g: &mut Grille, gift: &Don) {
    let dessin: [&str; 3] = match gift.kind {
        Cadeau::Coin => [".y.", "yYy", ".y."],
        Cadeau::Cookie => ["rVr", "VrV", "rVr"],
        Cadeau::Ore => [".A.", "AbA", "aAa"],
    };
    g.stamp(SW - 6, SH - 3, &dessin);
    if gift.age % 16 < 3 {
        g.put(SW - 5, SH - 3, b'w'); // un éclat, sur le cadeau lui-même
    }
}

fn draw_messenger(ctx: &Contexte, g: &mut Grille, m: &Messager) {
    // de l'autre forge, de droite à gauche au-dessus de la pièce, la nouvelle dans ses serres
    let (x, y) = (ctx.edge - arr(m.age as f64 * 1.6), arr((m.age as f64 / 3.0).sin()));
    g.stamp(x, y, corbeau(m.age));
    g.stamp(x + CLAWS.0, y + CLAWS.1, if m.kind == Nouvelle::Waiting { &["WFW", ".F."][..] } else { &["WWV"][..] });
}

/// La cour selon le temps : une flaque après l'orage, un bonhomme de neige l'hiver, des feuilles l'automne.
fn draw_yard(g: &mut Grille, w: &Monde) {
    if w.weather == Some(Temps::Storm) {
        g.stamp(YARD + 2, SH - 1, &[".cbbbc."]);
    }
    if w.saison() == Saison::Hiver {
        g.stamp(SW - 5, SH - 8, &[".kFk", ".ww.", "wwww", ".ww.", "wwww", "wwww", ".ww."]);
    }
    if w.saison() == Saison::Automne {
        for (x, ch) in [(YARD + 1, b'F'), (YARD + 4, b'y'), (YARD + 9, b'f'), (YARD + 12, b'F'), (SW - 3, b'y')] {
            g.put(x, SH - 1, ch);
        }
    }
}

fn draw_banquet(g: &mut Grille, clock: i64) {
    // une longue table dans la cour : un rôti au milieu, des chopes, la vapeur du festin
    let (x, y) = (YARD - 6, SH - 6);
    g.stamp(x, y, &["......SEHHES......", ".y..ESHHHHSE...y..", "rrrrrrrrrrrrrrrrrr", ".R..............R.", ".R..............R."]);
    if clock % 6 < 3 {
        g.put(x + 8, y - 1, b'X');
        g.put(x + 9, y - 2, b'x');
    }
}

/// Un louveteau de profil en (x, y), tourné vers `dir`, les pattes en `step` (0..3 au trot, CUB_SIT au travail).
fn draw_wolf(g: &mut Grille, x: i32, y: i32, dir: i32, step: i64, scarf: u8) {
    let mut rows: Vec<&str> = if step % 2 == 1 && step != CUB_SIT {
        [&CUB_BODY[..2], &CUB_TAIL_LOW[..], &CUB_BODY[4..]].concat()
    } else {
        CUB_BODY.to_vec()
    };
    if (0..CUB_LEGS.len() as i64).contains(&step) {
        rows.push(CUB_LEGS[step as usize]);
    }
    let rows: Vec<String> = rows
        .iter()
        .map(|r| {
            let row: Vec<u8> = r.bytes().map(|b| if b == b'C' { scarf } else { b }).collect();
            let row = if dir > 0 { row } else { row.into_iter().rev().collect() };
            String::from_utf8(row).unwrap_or_default()
        })
        .collect();
    g.stamp_s(x, y, &rows);
}

// Dans une cour large chaque louveteau a sa corvée, à sa place : l'un tire l'eau du puits et la
// porte à la porte, l'un va chercher du minerai au fond de la cour, l'un lit son rouleau à
// l'ombre de l'arbre, les autres martèlent près de la porte comme dans une cour étroite.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Corvee {
    Hammer,
    Water,
    Ore,
    Read,
}
const WIDE_YARD: i32 = 40; // passé autant de pixels depuis la porte, les louveteaux partent à leurs corvées

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fait {
    Hammer,
    Crank,
    Pour,
    Pick,
    Drop,
    Read,
}

#[derive(Clone)]
struct PetitIci {
    x: i32,
    y: i32,
    dir: i32,
    travel: f64,
    doing: Option<Fait>,
    since: Option<f64>,
    carry: Option<&'static [&'static str]>,
    trips: Option<i64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Etape {
    Going,
    B,
    Back,
    A,
}

struct Navette {
    x: f64,
    dir: i32,
    travel: f64,
    at: Etape,
    since: f64,
    trips: i64,
}

/// L'aller-retour entre a et b au pas du louveteau, posé `at_b` tics en b et `at_a` en a.
fn shuttle(c: &Petit, a: i32, b: i32, at_b: f64, at_a: f64) -> Navette {
    let pace = 0.4 + hasard(c.seed) * 0.3;
    let span = (b - a).abs() as f64;
    let leg = span / pace;
    let round = 2.0 * leg + at_b + at_a;
    let age = c.age as f64;
    let trips = (age / round).floor() as i64;
    let p = age % round;
    let dir = if b >= a { 1 } else { -1 };
    let done = trips as f64 * 2.0 * span;
    let (a, b) = (a as f64, b as f64);
    if p < leg {
        return Navette { x: a + dir as f64 * p * pace, dir, travel: done + p * pace, at: Etape::Going, since: p, trips };
    }
    if p < leg + at_b {
        return Navette { x: b, dir, travel: done + span, at: Etape::B, since: p - leg, trips };
    }
    if p < 2.0 * leg + at_b {
        let q = p - leg - at_b;
        return Navette { x: b - dir as f64 * q * pace, dir: -dir, travel: done + span + q * pace, at: Etape::Back, since: q, trips };
    }
    Navette { x: a, dir: -dir, travel: done + 2.0 * span, at: Etape::A, since: p - 2.0 * leg - at_b, trips: trips + 1 }
}

impl Contexte {
    /// Les corvées qu'offre cette cour, dans l'ordre où les louveteaux les prennent ; aucune dans une cour étroite.
    fn chores_of(&self, cour: &mut Cour) -> Vec<Corvee> {
        if self.edge - YARD < WIDE_YARD {
            return Vec::new();
        }
        let mut c = vec![Corvee::Hammer];
        if cour.layout(YARD, self.edge).well.is_some() {
            c.push(Corvee::Water);
        }
        c.extend([Corvee::Ore, Corvee::Read, Corvee::Hammer]);
        c
    }

    fn chore_at(&self, cour: &mut Cour, c: &Petit, chore: Corvee, hammer_slot: i64) -> PetitIci {
        let l = cour.layout(YARD, self.edge);
        let (well, tree) = (l.well, l.tree);
        let y = LANE.2;
        if let (Corvee::Water, Some(well)) = (chore, well) {
            let s = shuttle(c, YARD + 3, well - CUB_W, 24.0, 8.0);
            return PetitIci {
                x: arr(s.x),
                y: y - 1,
                dir: s.dir,
                travel: s.travel,
                since: Some(s.since),
                trips: Some(s.trips),
                doing: match s.at {
                    Etape::B => Some(Fait::Crank),
                    Etape::A => Some(Fait::Pour),
                    _ => None,
                },
                carry: if s.at == Etape::Back { Some(&["cc", "aa"]) } else { None },
            };
        }
        if chore == Corvee::Ore {
            let s = shuttle(c, YARD + 8, self.edge - CUB_W - 1, 10.0, 6.0);
            return PetitIci {
                x: arr(s.x),
                y,
                dir: s.dir,
                travel: s.travel,
                since: Some(s.since),
                trips: Some(s.trips),
                doing: match s.at {
                    Etape::B => Some(Fait::Pick),
                    Etape::A => Some(Fait::Drop),
                    _ => None,
                },
                // du minerai un voyage, une bûche le suivant
                carry: if s.at == Etape::Back { Some(if s.trips % 2 != 0 { &["RV"] } else { &["qA"] }) } else { None },
            };
        }
        if chore == Corvee::Read {
            // à l'ombre, à gauche du tronc : il lit longtemps, se dégourdit les pattes, relit
            let s = shuttle(c, tree.0 - CUB_W - 7, tree.0 - CUB_W - 1, 70.0, 70.0);
            return PetitIci {
                x: arr(s.x),
                y: y - 2,
                dir: s.dir,
                travel: s.travel,
                since: Some(s.since),
                trips: None,
                doing: if s.at == Etape::A || s.at == Etape::B { Some(Fait::Read) } else { None },
                carry: None,
            };
        }
        let at = cub_place(c, hammer_slot);
        PetitIci {
            x: at.x,
            y: at.y,
            dir: at.dir,
            travel: at.travel,
            doing: if at.working { Some(Fait::Hammer) } else { None },
            since: None,
            carry: None,
            trips: None,
        }
    }

    fn draw_cubs(&self, cour: &mut Cour, g: &mut Grille, cubs: &[Petit], puddle: bool) {
        // trois au plus dans une cour étroite, chacun à sa place ; dans une large, un par corvée
        let chores = self.chores_of(cour);
        let live: Vec<&Petit> = cubs.iter().filter(|c| c.leaving.is_none()).take(SLOTS.len().max(chores.len())).collect();
        let going: Vec<&Petit> = cubs.iter().filter(|c| c.leaving.is_some()).take(2).collect();
        let mut hammers = 0i64;
        // (le louveteau, s'il est des vivants son rang, où il est)
        let mut placed: Vec<(&Petit, Option<usize>, PetitIci)> = Vec::new();
        for (i, c) in live.iter().enumerate() {
            let chore = chores.get(i).copied().unwrap_or(Corvee::Hammer);
            let slot = if chore == Corvee::Hammer {
                hammers += 1;
                hammers - 1
            } else {
                i as i64
            };
            placed.push((c, Some(i), self.chore_at(cour, c, chore, slot)));
        }
        for c in &going {
            let n = chores.len().max(1) as i64;
            let k = c.seed % n;
            let chore = if k >= 0 { chores.get(k as usize).copied().unwrap_or(Corvee::Hammer) } else { Corvee::Hammer };
            let reste = Petit { leaving: None, ..(*c).clone() };
            let from = self.chore_at(cour, &reste, chore, c.seed % SLOTS.len() as i64);
            // son travail est fait : il trotte vers la droite, le rouleau dans la gueule, passé le bord à temps
            let pace = ((self.edge + 1 - from.x) as f64 / 36.0).max(1.3);
            let gone = c.leaving.unwrap_or(0) as f64;
            placed.push((
                c,
                None,
                PetitIci {
                    x: from.x + arr(gone * pace),
                    y: from.y,
                    dir: 1,
                    travel: gone * pace,
                    doing: None,
                    since: None,
                    carry: None,
                    trips: None,
                },
            ));
        }
        // les plus au fond dessinés d'abord
        placed.sort_by_key(|p| p.2.y);
        for (c, _, at) in &placed {
            let (x, dir) = (at.x, at.dir);
            let sitting = at.doing.is_some();
            let y = at.y - if puddle && !sitting && c.age % 6 < 2 { 1 } else { 0 }; // il patauge dans la flaque
            // un pas par pixel marché : une patte posée reste où elle est
            let step = if sitting { CUB_SIT } else { (at.travel.floor() as i64) % 4 };
            draw_wolf(g, x, y, dir, step, c.colour.0);
            let nose = if dir > 0 { x + CUB_W - 1 } else { x };
            let mouth = if dir > 0 { nose + 1 } else { nose - 2 };
            let since = at.since.unwrap_or(0.0);
            match at.doing {
                Some(Fait::Hammer) => {
                    // assis, son petit marteau monte et descend, des étincelles à sa couleur
                    let up = c.age % 4 < 2;
                    let hx = if dir > 0 { x + CUB_W } else { x - 1 };
                    g.put(hx, y + if up { 1 } else { 3 }, b'A');
                    g.put(hx, y + if up { 2 } else { 4 }, b'r');
                    if !up {
                        g.put(hx + dir, y + 4, c.colour.0);
                        g.put(hx + 2 * dir, y + 3, b'Y');
                    }
                }
                Some(Fait::Crank) => {
                    // une patte sur le treuil : la manivelle tourne, le seau monte, plein
                    if let Some(well) = cour.layout(YARD, self.edge).well {
                        let a = since % 8.0 < 4.0;
                        g.put(well + if a { 7 } else { 8 }, if a { 19 } else { 20 }, b'R');
                        g.stamp(well + 3, 23 - (since / 8.0).floor().min(2.0) as i32, &["aca"]);
                    }
                }
                Some(Fait::Pour) => {
                    // à la porte, le seau renversé : l'eau éclabousse
                    g.stamp(mouth, y + 3, &["ac"]);
                    for k in 0..3 {
                        // une rangée à virgule ne tombe sur aucun pixel (comme en JavaScript)
                        let r = (since + k as f64) % 3.0;
                        if r.fract() == 0.0 {
                            g.put(mouth - dir * k, y + 4 + r as i32, b'c');
                        }
                    }
                }
                Some(Fait::Pick) | Some(Fait::Drop) => {
                    // le nez au tas : du minerai, ou une bûche
                    let tas: &[&str] = if at.doing == Some(Fait::Pick) {
                        &["qAq"]
                    } else if at.trips.unwrap_or(0) % 2 != 0 {
                        &["qA"]
                    } else {
                        &["RV"]
                    };
                    g.stamp(mouth, y + 4, tas);
                }
                Some(Fait::Read) => {
                    // le rouleau ouvert devant lui, une page tournée de temps en temps
                    let page: &[&str] = if (since / 24.0).floor() as i64 % 2 != 0 { &["WVW"] } else { &["WWV"] };
                    g.stamp(if dir > 0 { mouth } else { mouth - 1 }, y + 3, page);
                }
                None => {}
            }
            if let Some(carry) = at.carry {
                g.stamp(mouth, y + 3, carry);
            }
            if c.leaving.is_some() {
                g.stamp(mouth, y + 3, &["WV"]); // le rouleau
            }
            if c.age < 8 && c.leaving.is_none() {
                g.stamp(x + 3, y + 1, PUFF[(c.age >> 2).min(2) as usize]);
            }
        }
        // ce que le louveteau du minerai a rapporté, en tas près de la porte
        let ore = chores.iter().take(live.len()).position(|&c| c == Corvee::Ore);
        let brought = ore.and_then(|i| placed.iter().find(|p| p.1 == Some(i))).and_then(|p| p.2.trips).unwrap_or(0).min(3);
        if brought > 0 {
            let tas: &[&str] = if brought > 2 {
                &[".qA.", "qAVR"]
            } else if brought > 1 {
                &["qAVR"]
            } else {
                &["qA"]
            };
            g.stamp(YARD + 3, SH - 1 - if brought > 2 { 1 } else { 0 }, tas);
        }
    }
}

/// Un coin qui dépasse perd son quart extérieur : une marche d'un demi-pixel au lieu d'un entier.
fn chamfer(g: &mut Grille, cx: i32, cy: i32) {
    let (top, bottom) = (g.at(cx, 2 * cy), g.at(cx, 2 * cy + 1));
    let clear = |g: &Grille, x: i32| g.at(x, 2 * cy) == b'.' && g.at(x, 2 * cy + 1) == b'.';
    if top == b'.' && bottom != b'.' {
        if clear(g, cx - 1) && g.at(cx + 1, 2 * cy) != b'.' {
            g.glyph(cx, cy, '▗', bottom);
        } else if clear(g, cx + 1) && g.at(cx - 1, 2 * cy) != b'.' {
            g.glyph(cx, cy, '▖', bottom);
        }
    } else if top != b'.' && bottom == b'.' {
        if clear(g, cx - 1) && g.at(cx + 1, 2 * cy + 1) != b'.' {
            g.glyph(cx, cy, '▝', top);
        } else if clear(g, cx + 1) && g.at(cx - 1, 2 * cy + 1) != b'.' {
            g.glyph(cx, cy, '▘', top);
        }
    }
}

const SAND: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

fn draw_hourglasses(g: &mut Grille, n: i32, clock: i64) {
    for i in 0..n.min(HOURGLASS_X.len() as i32) {
        // un couvercle fin, le verre en traits de boîte, le sable en huitièmes de cellule, le pied
        // en pixels sur l'enclume : les rangées 18..23 sont les cellules 9..11, le pied la rangée 24
        let x = HOURGLASS_X[i as usize];
        let phase = (clock + i as i64 * 13) % 40; // le sable coule 4,5 s, repose, puis il le retourne
        let left = arr((1.0 - (phase as f64 / 30.0).min(1.0)) * 8.0);
        for k in 0..3 {
            g.glyph(x + k, 9, '▁', b'A');
            g.put(x + k, 24, b'a');
        }
        g.glyph(x, 10, '╲', b'A');
        g.glyph(x + 2, 10, '╱', b'A');
        g.glyph(x, 11, '╱', b'A');
        g.glyph(x + 2, 11, '╲', b'A');
        g.glyph(x + 1, 10, SAND.get(left as usize).copied().unwrap_or(' '), b's');
        g.glyph(x + 1, 11, SAND.get((8 - left) as usize).copied().unwrap_or(' '), b's');
    }
}

fn draw_bell(g: &mut Grille, age: i64) {
    let swing = if age < 18 { [0, 1, 0, -1][age.rem_euclid(4) as usize] } else { 0 };
    let (bx, by) = BELL_AT;
    g.put(bx + 2, by, b'r');
    g.put(bx + 2, by + 1, b'r');
    g.stamp(bx + swing, by + 2, &[".yyy.", "yyyyy", "yyyyy", "kkkkk"]);
    g.put(bx + 2 - swing, by + 6, b'k'); // le battant
    if age < 18 && age % 4 < 2 {
        g.put(bx - 2, by + 3, b's');
        g.put(bx - 3, by + 2, b's');
        g.put(bx + 6, by + 3, b's');
        g.put(bx + 7, by + 2, b's');
    }
}

fn draw_crate(g: &mut Grille, state: Caisse, seal_age: i64) {
    let (x, y) = CRATE_AT;
    g.stamp(x, y + 2, &CRATE_BOX);
    if state == Caisse::Open {
        g.stamp(x - 1, y - 6, &LID_OPEN); // debout sur ses gonds, à gauche
        g.stamp(x, y, &["kkkkkkkkkk", "kzzzzzzzzk"]); // le bord, le dedans sombre
    } else {
        if state == Caisse::Closing {
            g.stamp(x, y + 1, &["kzzzzzzzzk"]);
        }
        // il retombe, puis il est fermé
        g.stamp(x, y - if state == Caisse::Closing { 1 } else { 0 }, if state == Caisse::Closing { &LID_CLOSING[..] } else { &LID_SHUT[..] });
    }
    if state == Caisse::Sealed {
        // la cire sur la jointure, un éclat qui en fait le tour de temps en temps
        g.stamp(SEAL.0, SEAL.1, &["yYy", "yEy"]);
        if seal_age < 2 {
            g.stamp(SEAL.0 - 1, SEAL.1, &["y...y"]); // la cire s'étale encore
        } else if seal_age % 6 == 0 {
            g.put(SEAL.0 + 2, SEAL.1 + 1, b'Y');
        }
    }
}

/// Où se tient un invité dans la cour de ce dessin : le moteur les promène entre leur place et
/// la coulisse ; une cour plus large les range côte à côte, autant qu'il en tient.
fn guest_x(edge: i32, x: i32, i: i32, fit: i32) -> i32 {
    if i >= fit {
        return x;
    }
    let (spot, at, off) = (GUEST_SPOT - if i > 0 { 14 } else { 0 }, GUEST_SPOT + i * GUEST_STEP, edge - C.0 + 2);
    arr(at as f64 + ((x - spot) as f64 * (off - at) as f64) / (off - spot) as f64)
}

/// La scène d'un dessin. `actor_only` dessine Scorpheus seul, sans la pièce ni les effets du
/// moment : ainsi se dessine un invité dans une autre forge.
#[allow(clippy::too_many_arguments)]
pub fn compose(ctx: &mut Contexte, cour: &mut Cour, p: &Pose, world: &Monde, tick: i64, actor_only: bool, width: i32) -> Grille {
    let pal = etincelles(world.flavor);
    let mut g = Grille::new(width, SH);
    if !actor_only {
        ctx.edge = width;
    }
    let clock = world.clock;
    ctx.tete = match world.hammer {
        Marteau::Small => &HAMMER_SMALL,
        Marteau::Normal => &HAMMER_NORMAL,
        Marteau::Sledge => &HAMMER_SLEDGE,
    };
    ctx.cape = [world.cape[0].0, world.cape[1].0];
    let night = world.nuit();
    let mut p = *p;

    if !actor_only {
        // la pièce
        draw_window(&mut g, world, clock);
        draw_lantern(&mut g, world.lantern_lit, clock);
        draw_rack(&mut g, world);
        if let Some(b) = &world.banner {
            draw_banner(&mut g, b, clock);
        }

        // la forge
        g.stamp(FURNACE_AT.0, FURNACE_AT.1, &FURNACE);
        if world.saison() == Saison::Hiver {
            for i in 2..=11 {
                g.put(FURNACE_AT.0 + i, FURNACE_AT.1 + 4, b'w');
            }
        }
        let logs = match world.fuel5h {
            None => 8,
            Some(f) => arr(8.0 * (1.0 - f.min(100.0) / 100.0)),
        };
        for i in 0..logs {
            g.put(MOUTH.0 + i, LOGS_Y, if i % 3 == 1 { b'r' } else { b'R' });
        }
        let pile = match world.fuel7d {
            None => WOODPILE.len() as i64,
            Some(f) => arr(WOODPILE.len() as f64 * (1.0 - f.min(100.0) / 100.0)) as i64,
        };
        for (i, &(wx, wy)) in premiers(&WOODPILE, pile).iter().enumerate() {
            g.put(wx, wy, if i % 2 != 0 { b'V' } else { b'r' });
        }
        for cy in 5..=6 {
            for cx in 0..=3 {
                chamfer(&mut g, cx, cy); // la cheminée se lit ronde, pas en marches
            }
        }
        draw_fire(&mut g, if world.exhausted { 0 } else { p.fire }, tick);
        draw_smoke(&mut g, if world.exhausted { 0 } else { p.smoke_every }, tick);
        if p.embers {
            draw_embers(&mut g, tick);
        }
        if night && p.fire > 0 {
            // la nuit le foyer éclaire les pierres autour de lui
            let mut j = MOUTH.1 - 1;
            while j <= LOGS_Y {
                if hasard(tick * 3 + j as i64) > 0.5 {
                    g.put(MOUTH.0 - 2, j, b'E');
                    g.put(MOUTH.0 + MOUTH.2 + 1, j, b'E');
                }
                j += 2;
            }
        }
        // le soufflet : pressé, il souffle une bouffée dans le foyer
        g.stamp(
            BELLOWS.0,
            BELLOWS.1,
            if p.bellows {
                &["......", "......", ".....r", ".kRRr.", "kRRRRk", "kkkkkk"]
            } else {
                &[".....r", "....r.", ".kRRr.", "kRrrRk", "kRrrRk", "kkkkkk"]
            },
        );
        g.put(BELLOWS.0 - 1, BELLOWS.1 + 3, b'a'); // la buse
        if p.bellows {
            g.put(MOUTH.0 + MOUTH.2 - 1, BELLOWS.1 + 3, b'X');
            g.put(MOUTH.0 + MOUTH.2 - 2, BELLOWS.1 + 2, b'x');
        }
        g.stamp(ANVIL_AT.0, ANVIL_AT.1, &ANVIL);
        if p.ingot != Lingot::None {
            let ingot = match p.ingot {
                Lingot::Flash => b'Y',
                Lingot::Hot => b'H',
                _ => b'E',
            };
            for i in 0..INGOT.2 {
                g.put(INGOT.0 + i, INGOT.1, if i == 0 || i == INGOT.2 - 1 { if ingot == b'Y' { b'H' } else { b'E' } } else { ingot });
            }
        }
        let scraps = if p.sweep.is_some() { 0 } else { arr((world.ctx - 50.0).max(0.0) / 50.0 * SCRAPS.len() as f64) as i64 };
        for &(sx, sy, ch) in premiers(&SCRAPS, scraps) {
            g.put(sx, sy, ch);
        }
        let coins = match world.usd {
            Some(usd) if usd > 0.0 => COIN_STEPS.min(1 + ((1.0 + usd).log2() * 2.0).floor() as i32),
            _ => 0,
        };
        let n = arr(coins as f64 / COIN_STEPS as f64 * COINS.len() as f64) as i64;
        for (i, &(qx, qy)) in premiers(&COINS, n).iter().enumerate() {
            let c = if (i as i64 + clock.div_euclid(10)) % 13 == 0 {
                b'Y'
            } else if (qx + qy) % 3 != 0 {
                b'y'
            } else {
                b's'
            };
            g.quad(qx as f64, qy as f64, c, None);
        }
        if world.hourglasses > 0 {
            draw_hourglasses(&mut g, world.hourglasses, clock);
        }
        if world.exhausted && world.hourglasses == 0 {
            draw_slate(&mut g, clock);
        }
        if world.duck {
            g.stamp(DUCK_AT.0, DUCK_AT.1, &DUCK);
        }
        if let Some(state) = p.crate_ {
            if state != Caisse::Gone {
                draw_crate(&mut g, state, p.seal_age.unwrap_or(-1));
            }
        }
    }

    // Scorpheus, sauf s'il est parti en visite
    if !world.absent {
        let walking = world.walking;
        // son pas suit le sol, pas l'horloge : un pas par pixel marché, si bien qu'une botte posée
        // reste en place pendant qu'il avance ; la tête hoche quand chaque botte se pose
        let stride = (world.x * if world.facing < 0 { -1 } else { 1 }).rem_euclid(4);
        if walking {
            p.look = world.facing;
            p.head_dy = if stride % 2 == 0 { 1 } else { 0 };
            p.tail = if stride < 2 { 0 } else { 1 };
        }
        let x = C.0 + world.x;
        let y = C.1 + p.dy + if p.seated { 2 } else { 0 };
        if p.stool {
            g.stamp(x + 1, C.1 + 23, &["RRRRRRRRRRRRRR", ".R..........R.", ".R..........R."]);
        }
        let hy = y + p.head_dy;
        if !ctx.pet {
        if let Some(tail) = TAIL.get(p.tail) {
            g.stamp(x + 13, y + 13, tail);
        }
        if p.seated {
            g.stamp_s(x, y + 14, &ctx.dress(&BODY_SEATED));
            g.stamp(x + 2, C.1 + 24, &["kaaaak"]);
            g.stamp(x + 8, C.1 + 24 - if p.tap { 1 } else { 0 }, &["kaaaak"]);
            g.stamp(x + 2, C.1 + 25, &["kkkkk"]);
            g.stamp(x + 9, C.1 + 25 - if p.tap { 1 } else { 0 }, &["kkkkk"]);
        } else {
            g.stamp_s(x, y + 14, &ctx.dress(&BODY));
            if world.accessory == Accessoire::Tablier {
                // le tablier de cuir sur sa poitrine
                g.stamp(x + 4, y + 15, &[".rrrrrr.", ".rRrrRr.", ".rrrrrr.", "..rrrr.."]);
            }
            if walking {
                // un pas de côté en quatre : écartés, la botte de derrière se lève et rejoint,
                // ensemble, celle de devant se lève et s'écarte
                for i in 1..=14 {
                    g.erase(x + i, y + 24);
                    g.erase(x + i, y + 25);
                }
                let left_lifted = stride == if world.facing < 0 { 3 } else { 1 };
                let right_lifted = stride == if world.facing < 0 { 1 } else { 3 };
                g.stamp(x + [1, 2, 3, 2][stride as usize], y + 24 - if left_lifted { 1 } else { 0 }, &["kaaaak", "kkkkk"]);
                g.stamp(x + [9, 8, 7, 8][stride as usize], y + 24 - if right_lifted { 1 } else { 0 }, &["kaaaak", ".kkkkk"]);
            }
            if p.tap {
                for i in 8..=13 {
                    g.erase(x + i, y + 24);
                    g.erase(x + i, y + 25);
                }
                g.stamp(x + 8, y + 23, &["kaaaak", ".kkkkk"]);
            }
        }
        let mut head: Vec<Vec<u8>> = SKULL.iter().map(|r| r.as_bytes().to_vec()).collect();
        // une face de 16 px tourne d'un pixel au plus : au-delà, un œil bute sur le contour
        let dx = p.look.clamp(-1, 1);
        // un trait ne se pose que sur la fourrure, jamais sur le contour
        let feature = |head: &mut Vec<Vec<u8>>, fx: i32, fy: i32, ch: u8| {
            if fy < 0 || ch == b'.' {
                return;
            }
            let Some(row) = head.get_mut(fy as usize) else { return };
            let i = fx + dx;
            if i < 0 || i as usize >= row.len() {
                return;
            }
            let under = row[i as usize];
            if under != b'k' && under != b'.' {
                row[i as usize] = ch;
            }
        };
        let paste = |head: &mut Vec<Vec<u8>>, fx: i32, fy: i32, rows: &[&str]| {
            for (j, row) in rows.iter().enumerate() {
                for (i, ch) in row.bytes().enumerate() {
                    feature(head, fx + i as i32, fy + j as i32, ch);
                }
            }
        };
        for &(mx, my, ch) in &MARKS {
            feature(&mut head, mx, my, ch);
        }
        if p.worried {
            paste(&mut head, 2, 5, &["..k", "kkl"]);
            paste(&mut head, 10, 5, &["k..", "lkk"]);
        } else if p.focused {
            // les sourcils tirés vers le museau : la concentration, les yeux laissés entiers
            paste(&mut head, 2, 5, &["k..", "lkk"]);
            paste(&mut head, 10, 5, &["..k", "kkl"]);
        } else {
            paste(&mut head, 2, 6, &["kkk"]);
            paste(&mut head, 10, 6, &["kkk"]);
        }
        let (gauche, droit): (&[&str], &[&str]) = match p.eyes {
            Yeux::Sad => (&["ll", "kk", "cl"], &["ll", "kk", "lc"]),
            Yeux::Open => (&EYES_OPEN, &EYES_OPEN),
            Yeux::Look => (&EYES_LOOK, &EYES_LOOK),
            Yeux::Blink => (&EYES_BLINK, &EYES_BLINK),
            Yeux::Happy => (&EYES_HAPPY, &EYES_HAPPY),
        };
        paste(&mut head, 2, 8, gauche);
        paste(&mut head, 10, 8, droit);
        let bouche: [&str; 2] = match p.mouth {
            super::poses::Bouche::None => ["wwww", "wwww"],
            super::poses::Bouche::Smile => ["wwww", "wmmw"],
            super::poses::Bouche::Open => ["wmmw", "wrrw"],
            super::poses::Bouche::Frown => ["wmmw", "wwww"],
            super::poses::Bouche::Grit => ["wwww", "wmmw"],
        };
        paste(&mut head, 6, 12, &bouche);
        if p.ear {
            head[0][1] = b'.';
            head[1][0] = b'.';
            head[1][1] = b'k';
            head[1][2] = b'k';
            head[2][1] = b'k';
            head[2][2] = b'w';
        }
        if p.ear_r {
            head[0][14] = b'.';
            head[1][15] = b'.';
            head[1][14] = b'k';
            head[1][13] = b'k';
            head[2][14] = b'k';
            head[2][13] = b'w';
        }
        for (j, row) in head.iter().enumerate() {
            for (i, &ch) in row.iter().enumerate() {
                g.put(x + i as i32, hy + j as i32, ch);
            }
        }
        let fe = world.fete();
        if world.accessory == Accessoire::Lunettes {
            // les lunettes relevées sur le front : un verre de chaque côté, la sangle entre
            g.stamp(x + 1, hy + 6, &["RAcA"]);
            g.stamp(x + 11, hy + 6, &["AcAR"]);
            for i in 5..=10 {
                g.put(x + i, hy + 6, b'R');
            }
        }
        if world.accessory == Accessoire::Bandeau {
            for i in 1..=14 {
                g.put(x + i, hy + 6, ctx.cape[1]);
            }
            g.put(x + 15, hy + 7, ctx.cape[1]);
            g.put(x + 15, hy + 8, ctx.cape[0]); // les pans de son nœud
        }
        if fe == Some(Fete::Anniversaire) {
            g.stamp(x + 5, hy - 4, &HAT_PARTY);
        }
        if fe == Some(Fete::Noel) {
            g.stamp(x + 5, hy - 4, &HAT_SANTA);
        }
        }

        // son bras gauche (à gauche de l'écran) : marteau, levier, lame, sceau, balai ; d'après
        // son pet, le pet peint la patte et le marteau, la forge ce qu'elle tient d'autre
        let ls = (x + 1, y + 15);
        // en marchant, ses pattes pendent au repos ; le balai y reste, il balaie en allant
        let left = if walking && p.sweep.is_none() { Gauche::Rest } else { p.left };
        let right = if walking && p.sweep.is_none() { Droite::Rest } else { p.right };
        match left {
            Gauche::Rest => {
                ctx.arm(&mut g, ls.0, ls.1, x, y + 18);
                ctx.hammer(&mut g, x, y + 19, x - 1, y + 21);
            }
            Gauche::Raise => {
                ctx.arm(&mut g, ls.0, ls.1, x - 1, y + 11);
                ctx.hammer(&mut g, x - 1, y + 10, x - 3, y + 6);
            }
            Gauche::Strike => {
                ctx.arm(&mut g, ls.0, ls.1, x - 3, y + 15);
                ctx.hammer(&mut g, x - 4, y + 16, IMPACT.0 - 1, IMPACT.1 - 1 + p.dy);
            }
            Gauche::Drop => {
                ctx.arm(&mut g, ls.0, ls.1, x, y + 19);
                if !ctx.pet {
                    g.stamp(x - 5, C.1 + 24, &["r...", ".rAA", "..aa"]); // tombé à ses pieds
                }
            }
            Gauche::Scroll => ctx.arm(&mut g, ls.0, ls.1, x + 4, y + 18),
            Gauche::Pull => {
                // le levier du soufflet : sa patte l'accompagne en bas, le soufflet se serre
                let (lx, ly) = (BELLOWS.0 + 5, BELLOWS.1 + if p.bellows { 2 } else { 0 }); // le bout du levier
                ctx.arm(&mut g, ls.0, ls.1, lx + 1, ly);
                g.stamp(IMPACT.0 - 1, IMPACT.1, &[".r.AA", "rr.aa"]); // le marteau posé sur l'enclume
            }
            Gauche::Blade => {
                ctx.arm(&mut g, ls.0, ls.1, x - 2, y + 11);
                blade(&mut g, x - 2, y + 10, -1, p.blade.unwrap_or(Lame::Hot), tick);
            }
            Gauche::Dip => {
                ctx.arm(&mut g, ls.0, ls.1, BUCKET_AT.0 + 4, BUCKET_AT.1 - 6);
                blade(&mut g, BUCKET_AT.0 + 4, BUCKET_AT.1 - 6, 1, p.blade.unwrap_or(Lame::Hot), tick);
            }
            Gauche::Stow => {
                // la lame du jour, à plat dans sa patte, descendue dans la caisse
                let by = CRATE_AT.1 - p.stow.unwrap_or(0);
                ctx.arm(&mut g, ls.0, ls.1, CRATE_AT.0 + 9, by);
                if by < CRATE_AT.1 {
                    g.stamp(CRATE_AT.0 + 2, by, &["wAAAAyr"]);
                }
            }
            Gauche::Lid => ctx.arm(&mut g, ls.0, ls.1, CRATE_AT.0 + 8, CRATE_AT.1 - 1), // sa patte rabat le couvercle
            Gauche::StampUp | Gauche::StampDown => {
                // le sceau : un manche de bois, un pied d'or
                let fy = if left == Gauche::StampUp { SEAL.1 - 6 } else { SEAL.1 - 1 };
                g.stamp(SEAL.0, fy - 3, &[".R.", ".r.", ".r.", "yyy"]);
                ctx.arm(&mut g, ls.0, ls.1, SEAL.0 + 2, fy - 3);
            }
            Gauche::Broom => {} // dessiné avec le bras droit : les deux pattes sur un manche
            Gauche::Lap => ctx.arm(&mut g, ls.0, ls.1, x + 4, y + 19),
        }

        // son bras droit
        let rs = (x + 14, y + 15);
        if !ctx.pet {
        match right {
            Droite::Rest => ctx.arm(&mut g, rs.0, rs.1, x + 15, y + 18),
            Droite::Wave1 => ctx.arm(&mut g, rs.0, rs.1, x + 17, y + 9),
            Droite::Wave2 => ctx.arm(&mut g, rs.0, rs.1, x + 18, y + 11),
            Droite::Up => ctx.arm(&mut g, rs.0, rs.1, x + 17, y + 8),
            Droite::Scroll => ctx.arm(&mut g, rs.0, rs.1, x + 11, y + 18),
            Droite::Scratch => ctx.arm(&mut g, rs.0, rs.1, x + 15, y + 6 + (tick % 2) as i32),
            Droite::Wipe => ctx.arm(&mut g, rs.0, rs.1, x + 12 - (tick % 2) as i32, hy + 7),
            Droite::Drink => {
                ctx.arm(&mut g, rs.0, rs.1, x + 11, hy + 13);
                g.stamp(x + 10, hy + 10, &["A", "a", "a"]); // la gourde, penchée au museau
            }
            Droite::Broom => ctx.draw_broom(&mut g, &p, x, y, ls, rs, walking && world.facing < 0),
            Droite::Lap => ctx.arm(&mut g, rs.0, rs.1, x + 11, y + 19),
            Droite::Chin => ctx.arm(&mut g, rs.0, rs.1, x + 10, hy + 14),
            Droite::Bread => {
                ctx.arm(&mut g, rs.0, rs.1, x + 11, hy + 13);
                g.stamp(x + 9, hy + 12, &["VWV", "rVr"]); // son pain, au museau
            }
            Droite::Tea => {
                ctx.arm(&mut g, rs.0, rs.1, x + 12, y + 17);
                g.stamp(x + 11, y + 15, &["WW", "WV"]); // la tasse
                let a = tick % 6;
                g.put(x + 11 + (a % 2) as i32, y + 14 - a.div_euclid(2) as i32, if a < 3 { b'X' } else { b'x' }); // sa vapeur
            }
        }
        if p.scroll {
            g.stamp(x + 4, y + 16, &SCROLL);
        }
        }
        if ctx.pet && right == Droite::Broom {
            ctx.draw_broom(&mut g, &p, x, y, ls, rs, walking && world.facing < 0); // ses pattes : le pet
        }
        if p.thought {
            g.put(x + 15, hy + 3, b'w');
            g.put(x + 16, hy + 1, b'w');
            g.stamp(x + 17, (hy - 6).max(0), &THOUGHT[(tick.div_euclid(3) % 2) as usize]);
        }
        if p.glints {
            for i in 0..3i64 {
                let a = (tick + i * 5) % 9;
                let gx = x + 3 + arr(hasard(i * 7 + (tick + i * 5).div_euclid(9)) * 9.0);
                g.put(gx, y + 15 - a.div_euclid(2) as i32, if a < 4 { pal[0] } else { pal[1] });
            }
        }

        if let Some(bubble) = p.bubble {
            g.stamp(x + 17, (y - 6 + bubble).max(0), &BUBBLE);
        } else if world.speaking && !p.thought {
            g.stamp(x + 17, (hy - 5).max(0), &SAYING);
        }

        // les effets du moment
        if !actor_only {
            if p.bucket {
                g.stamp(BUCKET_AT.0, BUCKET_AT.1, &BUCKET);
            }
            if let Some(steam) = p.steam {
                draw_steam(&mut g, steam, BUCKET_AT.0 + 4, BUCKET_AT.1 - 1);
            }
            if let Some(a) = p.raven.filter(|&a| a < 16) {
                // il tombe d'en haut (âge négatif), loin de sa tête, prend la caisse, et repart en haut à gauche
                let af = a as f64;
                let rx = CRATE_AT.0 + 1 - arr(af * if a < 0 { 0.7 } else { 1.4 });
                let ry = CRATE_AT.1 - 6 - arr(af.abs() * if a < 0 { 3.0 } else { 1.3 });
                g.stamp(rx, ry, super::accessoires::RAVEN[a.rem_euclid(4) as usize]);
                if a >= 0 {
                    g.stamp(rx + CLAWS.0 - 2, ry + CLAWS.1, &["kVVVVk", "kryyrk", "kkkkkk"]); // la caisse scellée dans ses serres
                }
            }
            if let Some(s) = p.seal_age.filter(|&s| (0..6).contains(&s)) {
                draw_sparks(&mut g, s, 7, 10, SEAL.0 + 1, SEAL.1, etincelles(Saveur::Git));
            }
            if p.lid_puff {
                g.stamp(CRATE_AT.0 - 2, CRATE_AT.1 - 1, &["x.", "Xx"]);
                g.stamp(CRATE_AT.0 + 10, CRATE_AT.1 - 1, &[".x", "xX"]);
            }
            if let Some(dust) = p.dust {
                if !(walking && world.facing < 0) {
                    draw_dust(&mut g, x + BROOM_X + p.sweep.unwrap_or(0) + 4, dust);
                }
            }
            // au-dessus de sa tête, loin des z à sa droite : la scène la plus étroite (jusqu'à x 54) la tient entière
            if p.zzz {
                if let Some(dream) = world.dream {
                    draw_dream(&mut g, x + 2, (hy - 9).max(0), dream);
                }
                draw_zzz(&mut g, x + 15, hy + 1, tick);
            }
            // d'après son pet, la gerbe se peint au vrai pixel (gerbe.rs)
            if let Some(age) = p.sparks_age.filter(|&a| a < 6 && !ctx.pet) {
                draw_sparks(&mut g, age, tick.div_euclid(8), p.spark_count.unwrap_or(12), IMPACT.0 + 1, IMPACT.1, pal);
            }
            if let Some(f) = p.fireworks {
                draw_sparks(&mut g, f % 7, f.div_euclid(7) + 99, 16, IMPACT.0 + 1, IMPACT.1 - 2, etincelles(Saveur::Forge));
            }
            if let Some(puff) = p.puff {
                let k = (puff >> 1).min(2);
                if k >= 0 {
                    g.stamp(IMPACT.0 - 2, IMPACT.1 - 4 - k as i32, PUFF[k as usize]);
                }
            }
            if p.stars {
                for i in 0..6i64 {
                    let sx = (hasard(tick.div_euclid(2) * 11 + i) * ctx.edge as f64).floor() as i32;
                    let sy = (hasard(tick.div_euclid(2) * 23 + i) * 12.0).floor() as i32;
                    g.put(sx, sy, if i % 2 != 0 { b's' } else { b'Y' });
                    if i % 3 == 0 {
                        g.put(sx - 1, sy, b's');
                        g.put(sx + 1, sy, b's');
                        g.put(sx, sy - 1, b's');
                        g.put(sx, sy + 1, b's');
                    }
                }
            }
            if p.sweat {
                let d = if tick % 6 > 2 { 1 } else { 0 };
                g.put(x + 16, hy + 3 + d, b'c');
                g.put(x + 16, hy + 4 + d, b'c');
            }
        }
    }
    if actor_only {
        return g;
    }

    draw_yard(&mut g, world);
    if let Some(cat) = &world.cat {
        draw_cat(&mut g, cat);
    }
    if let Some(gift) = &world.gift {
        draw_gift(&mut g, gift);
    }

    // les invités d'autres sessions : le même Scorpheus, chacun à son allure, dans la cour
    let edge = ctx.edge;
    let fit = ((edge - (C.0 + GUEST_SPOT) - 17).div_euclid(GUEST_STEP) + 1).max(1);
    for (i, v) in world.guests.iter().take(fit.max(2) as usize).enumerate() {
        let gx = guest_x(edge, v.x, i as i32, fit);
        // dans une cour large il se presse, plus d'un pixel par tic : ses pattes suivent alors le
        // temps (un pas par tic), pas le sol, sinon elles sauteraient des pas ; dessiné où ce pas
        // tombe, puis déplacé
        let sign = if v.facing < 0 { -1 } else { 1 };
        let shift = if v.walking && edge > SW { sign * ((gx * sign) as i64 - v.tick).rem_euclid(4) as i32 } else { 0 };
        let at = Monde {
            x: gx - shift,
            walking: v.walking,
            facing: v.facing,
            cape: v.cape,
            accessory: v.accessory,
            absent: false,
            guests: Vec::new(),
            speaking: false,
            hammer: Marteau::Normal,
            routine: None,
            dream: None,
            hourglasses: 0,
            exhausted: false,
            tea: false,
            duck: false,
            pressure: 0.0,
            personality: Humeur::Calme,
            ..world.clone()
        };
        let pose = pose_of(if v.waving { Activite::Waving } else { Activite::Idle }, v.tick, &at);
        let pet = std::mem::replace(&mut ctx.pet, false);
        let guest = compose(ctx, cour, &pose, &at, v.tick, true, edge);
        ctx.pet = pet;
        for gy in 0..guest.h {
            for gxx in 0..guest.w {
                let ch = guest.at(gxx, gy);
                if ch != b'.' {
                    g.put(gxx + shift, gy, ch);
                }
            }
        }
    }
    if world.banquet {
        draw_banquet(&mut g, clock);
    }
    if let Some(m) = &world.messenger {
        draw_messenger(ctx, &mut g, m);
    }

    // ce qui passe au-dessus de tout ce qu'il fait
    draw_visitor(ctx, &mut g, world, clock);
    if let Some(age) = world.bell_age.filter(|&a| a < 40) {
        draw_bell(&mut g, age);
    }
    ctx.draw_cubs(cour, &mut g, &world.apprentices, world.weather == Some(Temps::Storm));
    draw_yard_weather(cour, &mut g, world, YARD, ctx.edge);
    g
}
