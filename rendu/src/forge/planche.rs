//! La planche : chaque moment et le décor dans ses situations, comme tools/apercu.mts les
//! montre, mais peints par l'appli (glyphes compris), dans un PNG.
//!
//!   rendu.exe --planche <sortie.png> [--large <colonnes>]

use super::atelier::{anchor_of, GUEST_SPOT, ROWS};
use super::cellules::{peindre, Cible};
use super::monde::*;
use super::Forge;

const CW: i32 = 9;
const CH: i32 = 19;
const ECART: i32 = 8;

fn petit(seed: i64, age: i64, colour: u8) -> Petit {
    Petit { seed, colour: Lettre(colour), age, leaving: None }
}

/// Les scènes de la planche : (activité, tic, monde).
pub fn scenes() -> Vec<(Activite, i64, Monde)> {
    let calme = Monde::default;
    let avec = |f: &dyn Fn(&mut Monde)| {
        let mut w = calme();
        f(&mut w);
        w
    };
    let invite = Invite {
        x: GUEST_SPOT,
        walking: false,
        facing: -1,
        cape: [Lettre(b'N'), Lettre(b'L')],
        accessory: Accessoire::Lunettes,
        tick: 3,
        waving: true,
    };
    let mut s: Vec<(Activite, i64, Monde)> = vec![
        (Activite::Idle, 0, calme()),
        (Activite::Thinking, 4, calme()),
        (Activite::Running, 3, calme()),
        (Activite::Running, 5, avec(&|w| { w.flavor = Saveur::Build; w.effort = 2 })),
        (Activite::Review, 2, avec(&|w| w.flavor = Saveur::Web)),
        (Activite::Waiting, 1, calme()),
        (Activite::Failed, 2, calme()),
        (Activite::Jumping, 3, calme()),
        (Activite::Waving, 2, calme()),
        (Activite::Quench, 12, avec(&|w| { w.outcome = Some(Issue::Pass); w.flavor = Saveur::Test })),
        (Activite::Quench, 12, avec(&|w| { w.outcome = Some(Issue::Fail); w.flavor = Saveur::Test })),
        (Activite::Ship, 5, avec(&|w| { w.ship = Some(Envoi::Commit); w.flavor = Saveur::Git })),
        (Activite::Ship, 10, avec(&|w| { w.ship = Some(Envoi::Push); w.flavor = Saveur::Git })),
        (Activite::Sweep, 3, avec(&|w| w.ctx = 90.0)),
        (Activite::Sleep, 4, avec(&|w| { w.dream = Some(Reve::Fusee); w.hour = 23.0 })),
        (Activite::Waking, 2, calme()),
        (Activite::Idle, 1, avec(&|w| { w.x = -5; w.walking = true; w.facing = -1 })),
        (Activite::Idle, 2, avec(&|w| { w.x = 3; w.walking = true; w.facing = 1 })),
        (Activite::Idle, 13, avec(&|w| w.hour = 9.0)),
        (Activite::Idle, 5, avec(&|w| w.speaking = true)),
        (Activite::Idle, 0, avec(&|w| w.apprentices = vec![petit(3, 20, b'P'), petit(7, 33, b'b'), petit(11, 90, b'G')])),
        (Activite::Idle, 0, {
            let invite = invite.clone();
            let mut w = calme();
            w.guests = vec![invite];
            w.cat = Some(Chat { mode: ModeChat::Roof, x: 0, age: 3, dir: None, step: None });
            w
        }),
        (Activite::Idle, 0, avec(&|w| w.cat = Some(Chat { mode: ModeChat::Walk, x: 30, age: 3, dir: Some(-1), step: Some(1) }))),
        (Activite::Idle, 0, avec(&|w| {
            w.hour = 20.0;
            w.lantern_lit = true;
            w.rack = vec![Arme::Sword, Arme::Axe, Arme::Dagger, Arme::Mace];
            w.masterpiece = true;
            w.ctx = 80.0;
            w.fuel5h = Some(40.0);
            w.fuel7d = Some(70.0);
            w.usd = Some(6.0);
        })),
        (Activite::Idle, 0, avec(&|w| { w.weather = Some(Temps::Storm); w.pressure = 92.0 })),
        (Activite::Idle, 0, avec(&|w| {
            w.banquet = true;
            w.messenger = Some(Messager { kind: Nouvelle::Waiting, age: 12 });
            w.gift = Some(Don { kind: Cadeau::Coin, age: 1 });
        })),
        (Activite::Idle, 0, avec(&|w| { w.month = 1; w.hour = 11.0 })),
    ];
    let mut trois = calme();
    trois.hour = 19.0;
    trois.month = 10;
    trois.lantern_lit = true;
    trois.guests = vec![
        invite.clone(),
        Invite { x: GUEST_SPOT - 14, cape: [Lettre(b'C'), Lettre(b'D')], accessory: Accessoire::Bandeau, tick: 40, waving: false, ..invite.clone() },
        Invite { x: GUEST_SPOT - 14, cape: [Lettre(b'o'), Lettre(b'O')], accessory: Accessoire::Tablier, tick: 90, waving: false, ..invite },
    ];
    s.push((Activite::Idle, 0, trois));
    for (a, t, w) in &mut s {
        if let Some(x) = anchor_of(*a, w) {
            if !w.walking {
                w.x = x;
            }
        }
        w.clock = *t * 7 + 1;
    }
    s
}

pub fn planche(sortie: &str, extra: i32) -> std::io::Result<()> {
    let mut forge = Forge::default();
    let scenes = scenes();
    let cols = (super::atelier::SW + extra.clamp(0, super::atelier::GROW)) as usize;
    let (fw, fh) = (cols as i32 * CW, ROWS * CH);
    let par_ligne = 2;
    let lignes = scenes.len().div_ceil(par_ligne) as i32;
    let (w, h) = (ECART + (fw + ECART) * par_ligne as i32, ECART + (fh + ECART) * lignes);
    let mut pixels = vec![0xFF16_1616u32; (w * h) as usize];
    let cases = super::pet::cases(CH as u32);
    let mut scene = vec![0u32; (fw * fh) as usize];
    for (n, (a, t, monde)) in scenes.iter().enumerate() {
        // chaque scène peinte à part, Scorpheus par-dessus comme dans l'appli, puis posée
        let (cells, acteur, _) = forge.image(Recadrage::Full, *a, *t, monde, true, extra, cases.is_some());
        scene.fill(0);
        peindre(&mut Cible { pixels: &mut scene, w: fw as usize, h: fh as usize }, &cells, None, cols, 0, 0, CW, CH);
        if let (Some(cases), Some(acteur)) = (&cases, &acteur) {
            super::pet::peindre(cases, acteur, &mut scene, fw as usize, fh as usize, 0, CW, CH);
        }
        let (ox, oy) = (ECART + (n % par_ligne) as i32 * (fw + ECART), ECART + (n / par_ligne) as i32 * (fh + ECART));
        for y in 0..fh {
            let d = ((oy + y) * w + ox) as usize;
            pixels[d..d + fw as usize].copy_from_slice(&scene[(y * fw) as usize..((y + 1) * fw) as usize]);
        }
    }
    // le fond du terminal, là où la scène est transparente
    for p in pixels.iter_mut() {
        if *p >> 24 == 0 {
            *p = 0xFF0C_0C0C;
        }
    }
    std::fs::write(sortie, png(&pixels, w as u32, h as u32))
}

/// Un PNG sans compression : des blocs « stockés » de deflate, assez pour regarder une planche.
fn png(pixels: &[u32], w: u32, h: u32) -> Vec<u8> {
    let mut brut = Vec::with_capacity(((w * 3 + 1) * h) as usize);
    for y in 0..h {
        brut.push(0);
        for x in 0..w {
            let p = pixels[(y * w + x) as usize];
            brut.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, p as u8]);
        }
    }
    let mut z = vec![0x78, 0x01];
    let blocs: Vec<&[u8]> = brut.chunks(65535).collect();
    for (i, b) in blocs.iter().enumerate() {
        z.push(if i + 1 == blocs.len() { 1 } else { 0 });
        let n = b.len() as u16;
        z.extend_from_slice(&n.to_le_bytes());
        z.extend_from_slice(&(!n).to_le_bytes());
        z.extend_from_slice(b);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &o in &brut {
        a = (a + o as u32) % 65521;
        b = (b + a) % 65521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    for (t, d) in [(b"IHDR", &ihdr), (b"IDAT", &z), (b"IEND", &Vec::new())] {
        out.extend_from_slice(&(d.len() as u32).to_be_bytes());
        let mut corps = t.to_vec();
        corps.extend_from_slice(d);
        out.extend_from_slice(&corps);
        out.extend_from_slice(&crc32(&corps).to_be_bytes());
    }
    out
}

fn crc32(d: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &o in d {
        c ^= o as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
    }
    !c
}

#[cfg(test)]
mod film {
    use super::super::monde::*;
    use super::super::{Etat, SceneForge};
    use crate::scene::{Image, Scene};

    /// Des images qui se suivent, comme l'appli les peint (le pet, la gerbe), côte à côte :
    /// FILM=<sortie.png> cargo test --release film -- --ignored
    #[test]
    #[ignore]
    fn film() {
        let (cw, ch, cols) = (9i32, 19i32, 144i32);
        let (w, h) = (cols * cw, 16 * ch);
        let mut s = SceneForge::new();
        s.recevoir(Etat { recadrage: Recadrage::Full, extra: 72, lit: true, activite: Activite::Running, tick: 0, monde: Monde { clock: 1, ..Monde::default() } });
        let mut px = vec![0u32; (w * h) as usize];
        let (vue, n) = (300i32, 12i32); // le coin de l'enclume et de Scorpheus, douze images
        let mut out = vec![0xFF16_1616u32; ((vue + 4) * n * h) as usize];
        for k in 0..16 + n {
            s.peindre(&mut Image { pixels: &mut px, w: w as u32, h: h as u32, cellule: (cw as u32, ch as u32) }, 0.15, k == 0);
            let i = k - 16;
            if i < 0 {
                continue;
            }
            for y in 0..h {
                for x in 0..vue {
                    let p = px[(y * w + 60 + x) as usize];
                    out[(y * (vue + 4) * n + i * (vue + 4) + x) as usize] = if p >> 24 == 0 { 0xFF0C_0C0C } else { p };
                }
            }
        }
        std::fs::write(std::env::var("FILM").unwrap(), super::png(&out, ((vue + 4) * n) as u32, h as u32)).unwrap();
    }
}

#[cfg(test)]
mod chrono {
    use super::super::cellules::{peindre, Cible};
    use super::super::monde::*;
    use super::super::Forge;
    use std::time::Instant;

    /// Où passe le temps d'une image : cargo test --release chrono -- --ignored --nocapture
    #[test]
    #[ignore]
    fn chrono() {
        let mut w = Monde { clock: 1, ..Monde::default() };
        w.apprentices = (0..3).map(|i| Petit { seed: 3 + 4 * i, colour: Lettre(b'P'), age: 20 + 30 * i, leaving: None }).collect();
        let mut f = Forge::default();
        let n = 300;
        for (nom, lit) in [("sans lumière", false), ("avec lumière", true)] {
            let t = Instant::now();
            for k in 0..n {
                let w = w.plus_tard(k);
                std::hint::black_box(f.frame(Recadrage::Full, Activite::Running, k, &w, lit, 72));
            }
            println!("frame {nom} : {:.3} ms", t.elapsed().as_secs_f64() * 1000.0 / n as f64);
        }
        let mut cour = super::super::cour::Cour::default();
        let t = Instant::now();
        for k in 0..n {
            std::hint::black_box(cour.outdoors(&w.plus_tard(k), true, 54, 144));
        }
        println!("dont la cour : {:?} pixels mobiles", cour.mobiles());
        println!("dont la cour : {:.3} ms", t.elapsed().as_secs_f64() * 1000.0 / n as f64);
        let (cw, ch, cols) = (9, 19, 144usize);
        let mut px = vec![0u32; cols * 9 * 16 * 19];
        let mut avant: Option<Vec<u32>> = None;
        let (mut tp, mut aire, mut nz) = (0.0, 0i64, 0usize);
        for k in 0..n {
            let cells = f.frame(Recadrage::Full, Activite::Running, k, &w.plus_tard(k), true, 72);
            let t = Instant::now();
            let mut cible = Cible { pixels: &mut px, w: cols * 9, h: 16 * 19 };
            if let Some((a, b, c, d)) = peindre(&mut cible, &cells, avant.as_deref(), cols, 0, 0, cw, ch) {
                aire += ((c - a) * (d - b)) as i64;
                nz += 1;
            }
            tp += t.elapsed().as_secs_f64();
            avant = Some(cells);
        }
        // les cellules qui changent vraiment, par activité, et leurs colonnes extrêmes
        for a in [Activite::Idle, Activite::Running] {
            let (mut av, mut ch, mut tot) = (None::<Vec<u32>>, 0usize, 0usize);
            let mut parcol = vec![0usize; cols];
            for k in 0..n {
                let c = f.frame(Recadrage::Full, a, k, &w.plus_tard(k), true, 72);
                if let Some(av) = &av {
                    for (i, (x, y)) in c.chunks(3).zip(av.chunks(3)).enumerate() {
                        tot += 1;
                        if x != y {
                            ch += 1;
                            parcol[i % cols] += 1;
                        }
                    }
                }
                av = Some(c);
            }
            let bandes: Vec<String> = parcol.chunks(8).map(|b| format!("{}", b.iter().sum::<usize>() * 100 / (n as usize * 16 * 8))).collect();
            println!("{a:?} : {:.0} % des cellules changent ; par tranche de 8 colonnes (%) : {}", 100.0 * ch as f64 / tot as f64, bandes.join(" "));
        }
        println!("peindre : {:.3} ms, {:.1} zones, {:.0} % de l'image", tp * 1000.0 / n as f64, nz as f64 / n as f64, 100.0 * aire as f64 / n as f64 / (cols * 9 * 16 * 19) as f64);
        // Scorpheus d'après son pet : l'image avec lui, puis lui seul peint
        let cases = super::super::pet::cases(19).unwrap();
        let (mut ti, mut tpet) = (0.0, 0.0);
        for k in 0..n {
            let t = Instant::now();
            let (_, a, _) = f.image(Recadrage::Full, Activite::Running, k, &w.plus_tard(k), true, 72, true);
            ti += t.elapsed().as_secs_f64();
            let t = Instant::now();
            super::super::pet::peindre(&cases, a.as_ref().unwrap(), &mut px, cols * 9, 16 * 19, 0, cw, ch);
            tpet += t.elapsed().as_secs_f64();
        }
        println!("image avec le pet : {:.3} ms ; le pet peint : {:.3} ms", ti * 1000.0 / n as f64, tpet * 1000.0 / n as f64);
    }
}
