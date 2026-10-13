//! Ce que tout dessin peut prendre (hooks/props.ts) : la palette, les rêves, les bulles, les
//! étincelles, le corbeau, le ciel par une fenêtre.

use super::monde::{Fete, Monde, Rarete, Reve, Saison, Saveur, Temps};
use super::pixels::{arr, hasard, Grille, Palette};

const fn palette() -> Palette {
    let mut p: Palette = [None; 128];
    let couleurs: [(u8, u32); 59] = [
        (b'k', 0x1b1a24), // contour
        (b'd', 0x7c808c), // fourrure, ombre
        (b'g', 0xb4b7bf), // fourrure grise
        (b'l', 0xdedbd3), // fourrure claire
        (b'w', 0xfaf9f4), // fourrure blanche
        (b'n', 0x13294f), // crête marine
        (b'B', 0x1f4f9e), // crête bleue
        (b'b', 0x3f80dc), // crête claire
        (b'e', 0x2b8ff2), // iris
        (b'p', 0x060b18), // pupille
        (b'h', 0xffffff), // reflet de l'œil
        (b'y', 0xe3a63c), // or
        (b't', 0x34b5ae), // écailles sarcelle
        (b'o', 0x3f5a24), // cape sombre
        (b'O', 0x62802f), // cape
        (b'r', 0x7a4a24), // cuir, bois
        (b'R', 0x4a2c16), // bois sombre, bûches
        (b'm', 0x141018), // truffe, bouche
        (b'a', 0x4a4e5c), // acier
        (b'A', 0x8a90a0), // acier clair
        (b'q', 0x4f4a55), // pierre
        (b'Q', 0x6e6876), // pierre claire
        (b'z', 0x1f1a20), // mortier, gueule du four
        (b'E', 0xa3301b), // braise
        (b'F', 0xff5a1f), // flamme
        (b'f', 0xffb02e), // flamme vive
        (b'Y', 0xfff3b0), // chaleur blanche
        (b'H', 0xff8c2a), // métal chaud
        (b's', 0xffd75e), // étincelle
        (b'S', 0xff8a2a), // étincelle chaude
        (b'x', 0x5e5966), // fumée
        (b'X', 0x8d8896), // fumée claire
        (b'c', 0x7fd4ff), // eau, sueur, larmes
        (b'W', 0xefe2c0), // parchemin
        (b'V', 0xb8a57c), // encre, bouts de bûches
        (b'G', 0x74e07a), // étincelle des tests
        (b'j', 0xc4f5a0), // étincelle des tests, claire
        (b'P', 0xb07cff), // apprenti
        (b'i', 0xe2c8ff), // apprenti, clair
        (b'M', 0xff7ad9), // étincelle de rune
        (b'u', 0xbff3ff), // étincelle du shell, z du sommeil
        (b'C', 0x6e2618), // cape braise, sombre
        (b'D', 0xb5452a), // cape braise
        (b'N', 0x1e2a55), // cape nuit, sombre
        (b'L', 0x3a5a9a), // cape nuit
        (b'U', 0x45285a), // cape prune, sombre
        (b'I', 0x7d4f9e), // cape prune
        (b'T', 0x5e4212), // cape ocre, sombre
        (b'J', 0xa8792a), // cape ocre
        (b'Z', 0x2e343a), // cape ardoise, sombre
        (b'v', 0x5d6873), // cape ardoise
        (b'1', 0xc99a3c), // l'or de la panthère
        (b'2', 0x2e2010), // ses rosettes
        (b'3', 0xead29a), // son museau
        (b'4', 0xd6e64a), // son œil vert
        (b'5', 0x8a6424), // ses pattes du fond, dans l'ombre
        (0, 0),
        (0, 0),
        (0, 0),
    ];
    let mut i = 0;
    while i < couleurs.len() {
        let (l, c) = couleurs[i];
        if l != 0 {
            p[l as usize] = Some(c);
        }
        i += 1;
    }
    p
}

/// La palette : lettre → 0xRRGGBB.
pub const PALETTE: Palette = palette();

/// Les couleurs des étincelles par sorte de travail : [éclair, traînée, étincelle, étincelle].
pub const fn etincelles(s: Saveur) -> [u8; 4] {
    match s {
        Saveur::Forge => *b"YSsS",
        Saveur::Build => *b"YFfS",
        Saveur::Test => *b"jGjG",
        Saveur::Shell => *b"ubce",
        Saveur::Git => *b"wAwg",
        Saveur::Web => *b"utct",
        Saveur::Agent => *b"iPiP",
        Saveur::Mcp => *b"iMMP",
    }
}

/// Ce dont il rêve, 9 × 5 chacun, sur la nuit de la bulle.
fn reve(r: Reve) -> [&'static str; 5] {
    match r {
        Reve::Voiture => ["..FFFF...", ".FwbbwF..", "FFFFFFFFY", "FkakFFkak", ".kak..kak"],
        Reve::Fusee => ["..F......", "..wwwwF..", "fFwwbwwwF", "..wwwwF..", "..F......"],
        Reve::Bug => ["k..k.k..k", ".k.GGG.k.", "..GjGjG..", ".kGGGGGk.", "k..G.G..k"],
        Reve::Circuit => [".A.A.A.A.", "AkkkkkkkA", ".kkGjGkk.", "AkkkkkkkA", ".A.A.A.A."],
        Reve::Livre => [".WWWkWWW.", "WVVWkWVVW", "WWWWkWWWW", "WVVWkWVVW", ".rrrkrrr."],
        Reve::Montagne => ["....w....", "...www...", "..gwgdg..", ".gggdddd.", "ggggddddd"],
        Reve::Poisson => ["...bbb..b", ".bbbbbbbb", "bwbbbbbb.", ".bbbbbbbb", "...bbb..b"],
        Reve::Etoile => ["....Y....", "...YYY...", "YYYYYYYYY", "..YYYYY..", ".YY...YY."],
        Reve::Gateau => ["..F.F.F..", "..y.y.y..", ".MMMMMMM.", ".WWWWWWW.", ".MMMMMMM."],
        Reve::Dragon => ["......GG.", "G....GjGG", ".GGGGGG..", "..GG.GG..", "..G...G.."],
        Reve::Drapeau => ["kwkwkwk..", "wkwkwkw..", "kwkwkwk..", "A........", "A........"],
        Reve::Carburant => ["..AA.....", ".FFFFF...", ".FyFyF...", ".FFyFF...", ".FyFyF..."],
        Reve::Cle => [".........", "A.A...AAA", "AAAAAAA.A", ".A....AAA", "........."],
        Reve::Ecran => ["AAAAAAAAA", "AGG.yy..A", "A.bbb.G.A", "AAAAAAAAA", "...AAA..."],
        Reve::Musique => ["...YYYYY.", "...Y...Y.", "...Y...Y.", ".YYY.YYY.", ".YYY.YYY."],
        Reve::Base => [".cwwwwwc.", ".bcccccb.", ".bbbbbbb.", ".bcccccb.", ".bbbbbbb."],
        Reve::Reseau => ["cc.....cc", "cc.w.w.cc", "...ccc...", "cc.w.w.cc", "cc.....cc"],
        Reve::Manette => ["..aaaaa..", ".aAAAAAa.", "aAwAAAFAa", "aAAAAAGAa", ".aa...aa."],
        Reve::Graphique => [".......G.", "....y..G.", ".F..y..G.", ".F..y..G.", "AAAAAAAAA"],
        Reve::Photo => ["..aaa....", "AAAAAAAAA", "AAAkkkAyA", "AAAkckAAA", "AAAAAAAAA"],
        Reve::Carte => ["...FFF...", "..FFwFF..", "..FFFFF..", "...FFF...", "....F...."],
        Reve::Maison => ["...FFF...", "..FFFFF..", ".FFFFFFF.", "..WbWrW..", "..WWWrW.."],
        Reve::Horloge => ["..AAAAA..", ".AWWkWWA.", ".AWWkkkA.", ".AWWWWWA.", "..AAAAA.."],
        Reve::Cadenas => ["...aaa...", "..a...a..", ".yyyyyyy.", ".yyykyyy.", ".yyyyyyy."],
        Reve::Nuage => ["...www...", ".wwwwww..", "wwwwwwwww", ".wwwwwww.", ".c.c.c.c."],
        Reve::Ampoule => ["...YYY...", "..YYYYY..", "..YYwYY..", "...YYY...", "...aaa..."],
        Reve::Engrenage => ["...A.A...", "..AAAAA..", ".AAA.AAA.", "..AAAAA..", "...A.A..."],
        Reve::Robot => ["....F....", ".AAAAAAA.", ".AcAAAcA.", ".AAkkkAA.", ".AAAAAAA."],
        Reve::Plante => ["..G...G..", ".GGG.GGG.", "...GjG...", "..rrrrr..", "...rrr..."],
        Reve::Cafe => ["...w.w...", "..w.w....", ".WWWWW.W.", ".WrrrWW..", "..WWW...."],
    }
}

const BULLE_DU_REVE: [&str; 8] =
    [".wwwwwwwwwww.", "ww.........ww", "w...........w", "w...........w", "w...........w", "w...........w", "ww.........ww", ".wwwwwwwwwww."];

/// La bulle du rêve, 13 × 8, en (x, y).
pub fn draw_dream(g: &mut Grille, x: i32, y: i32, r: Reve) {
    g.stamp(x, y, &BULLE_DU_REVE);
    // la nuit du rêve dans le bord, puis le motif au milieu
    for (j, row) in BULLE_DU_REVE.iter().enumerate() {
        let (first, last) = (row.find('w').map_or(-1, |i| i as i32), row.rfind('w').map_or(-1, |i| i as i32));
        for i in first + 1..last {
            if row.as_bytes()[i as usize] == b'.' && j > 0 && j < BULLE_DU_REVE.len() - 1 {
                g.put(x + i, y + j as i32, b'n');
            }
        }
    }
    g.stamp(x + 2, y + 2, &reve(r));
}

/// La bulle « ! » : on a besoin de la personne.
pub const BUBBLE: [&str; 8] = [".kkkkk.", "kwwFwwk", "kwwFwwk", "kwwFwwk", "kwwwwwk", "kwwFwwk", ".kkkkk.", "kk....."];
/// Une parole dite.
pub const SAYING: [&str; 6] = [".kkkkk.", "kwwwwwk", "kwkwkwk", "kwwwwwk", ".kkkkk.", "kk....."];
/// Un engrenage qui tourne dans un nuage : il réfléchit.
pub const THOUGHT: [[&str; 7]; 2] = [
    [".wwwww.", "wwwwwww", "ww.a.ww", "wwaAaww", "ww.a.ww", "wwwwwww", ".wwwww."],
    [".wwwww.", "wwwwwww", "wwa.aww", "ww.A.ww", "wwa.aww", "wwwwwww", ".wwwww."],
];
const Z_GRAND: [&str; 4] = ["uuuu", "..u.", ".u..", "uuuu"];

/// Les z du sommeil, deux à la fois, qui montent de (x, y) en rapetissant : un Z en pixels,
/// puis les lettres Z et z, puis un z en braille (⠵), sur des cellules claires seulement.
pub fn draw_zzz(g: &mut Grille, x: i32, y: i32, tick: i64) {
    for i in 0..2 {
        let a = (tick + i * 10) % 20;
        let (zx, zy) = (x + a.div_euclid(4) as i32, y - a.div_euclid(3) as i32);
        if a < 6 {
            g.stamp(zx, zy - 1, &Z_GRAND);
            continue;
        }
        let (cx, cy) = (zx + 1, (zy + 1).div_euclid(2));
        if g.at(cx, 2 * cy) == b'.' && g.at(cx, 2 * cy + 1) == b'.' && 2 * cy >= 0 && 2 * cy + 1 < g.h && cx >= 0 && cx < g.w {
            g.glyph(cx, cy, if a < 11 { 'Z' } else if a < 15 { 'z' } else { '⠵' }, b'u');
        }
    }
}

/// Un nuage de poussière, en trois âges : un échec, une arrivée.
pub const PUFF: [&[&str]; 3] = [
    &["..xx..", ".xXXx.", "..xx.."],
    &[".xXXx.", "xXXXXx", "xXxXXx", ".xXXx."],
    &["x.XX.x", ".X..X.", "X....X", ".x..x."],
];
/// Un plumage gris : un oiseau noir disparaît sur un terminal sombre. Il regarde à gauche,
/// où il vole ; quatre battements, le corps aux rangées 2..4, ses serres sous la rangée 4.
pub const RAVEN: [&[&str]; 4] = [
    &[".....XX..", "......Xx.", ".xw....x.", "yxxxxxxx.", "..xxxxxXX"],
    &[".........", ".........", ".xw.XXXX.", "yxxxxxxxX", "..xxxxxX."],
    &[".........", ".........", ".xw......", "yxxxxxxx.", "..xxxXxXX", "....XX...", "...X....."],
    &[".........", ".........", ".xw......", "yxxxXXXxX", "..xxxxxX."],
];
/// Le battement d'aile d'un âge, rien pour un âge négatif (comme RAVEN[age % 4] en JavaScript).
pub fn corbeau(age: i64) -> &'static [&'static str] {
    if age < 0 {
        &[]
    } else {
        RAVEN[(age % 4) as usize]
    }
}
/// Où pend ce que porte un corbeau, depuis son coin haut gauche.
pub const CLAWS: (i32, i32) = (3, 5);

/// Des étincelles qui partent de (ox, oy), `age` tics après le coup.
pub fn draw_sparks(g: &mut Grille, age: i64, seed: i64, count: i64, ox: i32, oy: i32, pal: [u8; 4]) {
    let at = |vx: f64, vy: f64, a: f64| (arr(ox as f64 + vx * a), arr(oy as f64 - vy * a + 0.4 * a * a));
    if age == 0 {
        // l'éclair du coup
        for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (-2, -1), (2, -1)] {
            g.put(ox + dx, oy + dy, if dx != 0 || dy != 0 { pal[2] } else { pal[0] });
        }
    }
    for i in 0..count {
        let vx = (hasard(seed * 17 + i) * 2.0 - 1.0) * 2.4;
        let vy = 1.0 + hasard(seed * 29 + i) * 1.8;
        if hasard(seed * 41 + i) * 6.0 < (age - 1) as f64 {
            continue; // l'étincelle s'éteint
        }
        let (x, y) = at(vx, vy, age as f64);
        let (tx, ty) = at(vx, vy, (age as f64 - 0.5).max(0.0));
        g.put(tx, ty, pal[1]);
        g.put(x, y, if age < 2 { pal[0] } else if i % 3 == 0 { pal[3] } else { pal[2] });
    }
}

// Un dragon au loin, volant vers la droite : un battement d'aile en quatre.
const DRAGON: [&[&str]; 4] = [
    &["...B......", "..BBn...n.", "..BBnn..nn", "n..nnnnnnB", ".nnn.n.n.."],
    &["..........", "........n.", ".BBBBn..nn", "n..nnnnnnB", ".nnn.n.n.."],
    &["..........", "........n.", "........nn", "n..nnnnnnB", ".nnnBBnn..", "...BBB....", "...B......"],
    &["..........", "........n.", "........nn", "n..nnnnnnB", ".nnBBBBn.."],
];
const DRAGON_W: i32 = 10;

/// Des vitres : le dedans d'une fenêtre, où le ciel se voit.
pub struct Vitres {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

const CIELS: &[u8] = b"cnxXBbfF";

/// Le ciel par des vitres : selon l'heure, la saison, le temps, les raretés ; avec `volets`,
/// fermés pour la nuit de 23 h à 6 h.
pub fn draw_sky(g: &mut Grille, w: &Monde, tick: i64, v: &Vitres, volets: bool) {
    let (ix, iy, iw, ih) = (v.x, v.y, v.w, v.h);
    let night = w.nuit();
    let twilight = !night && (w.hour < 8.0 || w.hour >= 18.0);
    let storm = w.weather == Some(Temps::Storm);
    for j in 0..ih {
        for i in 0..iw {
            let sky = if storm {
                if j < 3 { b'x' } else { b'X' }
            } else if night {
                b'n'
            } else if twilight {
                if j < 2 { b'B' } else if j < 4 { b'b' } else if j < 6 { b'f' } else { b'F' }
            } else {
                b'c'
            };
            g.put(ix + i, iy + j, sky);
        }
    }
    // le soleil ou la lune, le long d'un arc à travers le jour ou la nuit
    let along = if night { ((w.hour + 3.0) % 24.0) / 9.0 } else { (w.hour - 6.0) / 15.0 };
    let bx = ix + arr(along.clamp(0.0, 1.0) * (iw - 2) as f64);
    let by = iy + arr((along - 0.5).abs() * 2.0 * (ih - 3) as f64);
    if !storm {
        if night {
            g.stamp(bx, by, &["WW", "Wn"]);
            for i in 0..4 {
                if hasard(i * 31 + tick.div_euclid(6)) > 0.35 {
                    g.put(ix + ((i as i32 * 5 + 2) % iw), iy + ((i as i32 * 3 + 1) % (ih - 2)), b'w');
                }
            }
        } else {
            g.stamp(bx, by, &["YY", "Yy"]);
        }
    }
    // Ce qui tombe est plus fin qu'un pixel, posé sur le ciel seul : la pluie en quarts de
    // bloc, la neige et les feuilles en braille. Chaque flocon garde sa couleur.
    let closed = volets && (w.hour >= 23.0 || w.hour < 6.0);
    let scale = (iw * ih) as f64 / 80.0;
    let t = tick as f64;
    if storm && !closed {
        // la pluie : un trait d'un demi-pixel de large, deux de long
        for i in 0..arr(9.0 * scale) as i64 {
            let qx = 2.0 * (ix as f64 + hasard(i * 13) * iw as f64);
            let qy = iy as f64 + ((t * 1.5 + (i * 5) as f64) % ih as f64);
            g.quad(qx, qy, b'c', Some(CIELS));
            g.quad(qx, qy - 1.0, b'c', Some(CIELS));
        }
    }
    if storm {
        if tick % 23 < 2 {
            for j in 0..ih {
                for i in 0..iw {
                    g.put(ix + i, iy + j, b'X');
                }
            }
            g.line(ix + 6, iy, ix + 4, iy + 3, b'Y');
            g.line(ix + 4, iy + 3, ix + 6, iy + 6, b'Y');
        }
    } else if w.weather == Some(Temps::Rainbow) {
        for (k, &ch) in b"FfyGbP".iter().enumerate() {
            for i in 0..iw {
                let h = (iw - 1) as f64 / 2.0;
                let t = (i as f64 - h) / h;
                let ry = iy + 1 + k as i32 + arr(t * t * 3.0);
                if ry < iy + ih {
                    g.put(ix + i, ry, ch);
                }
            }
        }
    }
    let sea = w.saison();
    if sea == Saison::Hiver && !closed {
        // la neige en points seuls, chaque flocon à son pas
        for i in 0..arr(14.0 * scale) as i64 {
            let fx = ix as f64 + hasard(i * 13) * iw as f64 + ((t + (i * 7) as f64) / 3.0).sin() * 0.7;
            let fy = iy as f64 + ((t * (0.35 + 0.3 * hasard(i * 3)) + (i * 5) as f64) % ih as f64);
            g.dot(2.0 * fx, 2.0 * fy, b'w', Some(CIELS));
        }
    }
    if (sea == Saison::Automne || sea == Saison::Printemps) && !closed {
        // des feuilles (rousse, or, ambre) ou des pétales : trois points qui basculent en tombant
        let colours = if sea == Saison::Automne { b"FyF" } else { b"MMi" };
        for i in 0..3i64 {
            let fx = 2.0 * (ix as f64 + hasard(i * 13) * iw as f64 + ((t + (i * 7) as f64) / 3.0).sin());
            let fy = 2.0 * (iy as f64 + ((t * if sea == Saison::Automne { 0.35 } else { 0.3 } + (i * 5) as f64) % ih as f64));
            let forme: [(f64, f64); 3] =
                if (tick.div_euclid(3) + i) % 2 != 0 { [(0.0, 0.0), (1.0, 1.0), (2.0, 1.0)] } else { [(0.0, 1.0), (1.0, 0.0), (2.0, 0.0)] };
            for (dx, dy) in forme {
                g.dot(fx + dx, fy + dy, colours[i as usize], Some(CIELS));
            }
        }
    }
    if w.fete() == Some(Fete::NouvelAn) && night {
        draw_sparks(g, tick % 6, tick.div_euclid(6), 6, ix + 3 + (tick.div_euclid(6) % 5) as i32, iy + 3, *b"YMsb");
    }
    if let Some(rare) = &w.rare {
        let a = rare.age;
        match rare.kind {
            Rarete::Star => {
                // une étoile filante traverse les vitres, sa traînée derrière elle
                for k in 0..4 {
                    g.put(ix + iw - 1 - a as i32 + k, iy + (a - k as i64).div_euclid(2) as i32, if k == 0 { b'Y' } else { b'w' });
                }
            }
            Rarete::Dragon => {
                // au loin, un dragon bat des ailes à travers le ciel, vu par les vitres seulement
                let dx = ix - DRAGON_W + a.div_euclid(3) as i32;
                let rows = if a < 0 { &[][..] } else { DRAGON[(a.div_euclid(2) % 4) as usize] };
                for (j, row) in rows.iter().enumerate() {
                    for (i, ch) in row.bytes().enumerate() {
                        let (px, py) = (dx + i as i32, iy - 1 + j as i32);
                        if px >= ix && px < ix + iw && py >= iy && py < iy + ih {
                            g.put(px, py, ch);
                        }
                    }
                }
            }
        }
    }
    if closed {
        // les volets fermés pour la nuit, un fil de lune entre eux
        for j in 0..ih {
            for i in 0..iw {
                g.put(ix + i, iy + j, if i == iw.div_euclid(2) - 1 { b'n' } else if i % 3 == 0 { b'R' } else { b'r' });
            }
        }
    }
}
