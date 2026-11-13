//! Le cadeau qu'un agent laisse dans la cour (user, 13-11 : « pour l'instant c'est un cube composé de 9 petits
//! cubes, on peut faire mieux »), dessiné à la main en lettres : la bourse de cuir et ses pièces d'or, le lembas
//! sur sa feuille de mallorn, le minerai veiné de mithril, le mot scellé à la cire. Au pixel d'art (`pet`) il
//! remplace le tampon de 3 × 3 cellules d'atelier.rs, à la même place, posé dans l'herbe.

use super::atelier::{SH, SW};
use super::monde::{Cadeau, Don};
use super::paysage::GRAIN;
use super::pet::Eclairage;

const CADEAU_PIECES: [&str; 14] = ["....kkk.......", "...kmLmk......", "....kRk.......", "...kRrRk......", "..kmLLLLk.....", ".kmLLLLLlk....", "kmLLLLLLLlk...", "kLLLLLLLllk...", "kLLLLLLlllk...", ".kLLLlllk.kkk.", "..kkkkkkkkYYyk", "kkkkkkkk.kYyyk", "kYYkkYyk..kkk.", ".kk..kk......."];
const CADEAU_LEMBAS: [&str; 13] = ["......kkkk....", "....kkGGvGk...", "..kkGGvGGGGk..", ".kGGkkkkkkGGk.", "kGGkWWWWWWkGk.", "kGvkWwWWwWkGGk", "kGGkwwwwwwkgGk", "kGgkWwWWwWkggk", ".kgkWwWWwWkgk.", ".kgkbbbbbbkk..", "..kkkkkkkkk...", "..kgk.........", "...kk........."];
const CADEAU_MINERAI: [&str; 13] = [".....kkkk.....", "....kSSSSk....", "..kkSSMSssk...", ".kSSSMNSssskk.", ".kSSMNSSsMssk.", "kSSSNSSsMNssdk", "kSSSSSsMNsssdk", "kSMSSssNssssdk", "kSNMssssssMsdk", ".kSNssssssNddk", ".kssssMssdddk.", "..kddsNddddk..", "...kkkkkkkk..."];
const CADEAU_MOT: [&str; 10] = ["..kkkkkkkkkk..", ".kWWWWWWWWWWk.", "kwWWWWkWWWWWwk", "kbwWWkrkWWWwbk", "kbwWkrRrkWWwbk", "kbwWWkrkWWWwbk", "kwWWWWkRkWWWwk", ".kWWWWWkRkWWk.", "..kkkkkkkRkkk.", "..........kk.."];


fn image(d: &Don) -> &'static [&'static str] {
    match d.kind {
        Cadeau::Coin => &CADEAU_PIECES,
        Cadeau::Cookie => &CADEAU_LEMBAS,
        Cadeau::Ore => &CADEAU_MINERAI,
        Cadeau::Note => &CADEAU_MOT,
    }
}

fn couleur(l: u8) -> Option<(u32, bool)> {
    Some((
        match l {
            b'k' => 0x1b1924,
            b'L' => 0x9a5a30,
            b'l' => 0x6a3a1e,
            b'm' => 0xc07a44,
            b'R' => 0xc82828,
            b'r' => 0x7a1418,
            b'Y' => 0xf4d070,
            b'y' => 0xc89038,
            b'z' => 0x8a5c1c,
            b'G' => 0x6aa040,
            b'g' => 0x3e6a2a,
            b'v' => 0x9ccc5a,
            b'W' => 0xf2e6c6,
            b'w' => 0xc8b48a,
            b'b' => 0xa08860,
            b'S' => 0x8a8a98,
            b's' => 0x5a5a6a,
            b'd' => 0x3a3a48,
            b'M' => 0xe8f4ff,
            b'N' => 0x9cc8f0,
            _ => return None,
        },
        false,
    ))
}

/// Où il se tient, en pixels de forge (la place de l'ancien tampon).
pub fn emprise(_d: &Don) -> (i32, i32, i32, i32) {
    (SW - 4, SH - 4, 5, 4)
}

/// Le peint au pixel d'art de la cour, le bas dans l'herbe, sous la lumière du dehors ; un éclat passe dessus de
/// temps en temps. Rend ce qu'il a couvert.
#[allow(clippy::too_many_arguments)]
pub fn peindre(d: &Don, e: Option<&Eclairage>, pixels: &mut [u32], w: usize, h: usize, x0: i32, cw: i32, ch: i32) -> Option<(i32, i32, i32, i32)> {
    let image = image(d);
    let k = GRAIN as i32;
    let demi = ch as f32 / 2.0;
    let (iw, ih) = (image[0].len() as i32, image.len() as i32);
    // trois colonnes plus loin que l'ancien tampon, dans l'herbe plutôt que devant le muret
    let cx = ((SW - 6 - x0) as f32 * cw as f32 + 4.5 * cw as f32).round() as i32;
    let bas = ((SH as f32 - 0.4) * demi / k as f32).round() as i32 * k;
    let (gauche, haut) = (cx - iw * k / 2, bas - ih * k);
    let mut couvert = super::chouette::peindre_image(image, &couleur, false, gauche, haut, k, e, pixels, w, h, x0, cw, ch);
    // l'éclat, en haut à droite de ce qui brille, trois tics sur seize
    if d.age.rem_euclid(16) < 3 {
        let (sx, sy) = (gauche + (iw - 3) * k, haut + k);
        for (dx, dy) in [(0, -1), (-1, 0), (0, 0), (1, 0), (0, 1)] {
            let (x, y) = (sx + dx * k, sy + dy * k);
            for yy in y.max(0)..(y + k).min(h as i32) {
                for xx in x.max(0)..(x + k).min(w as i32) {
                    pixels[yy as usize * w + xx as usize] = 0xFFFF_FFF0;
                }
            }
        }
        couvert = couvert.map(|z| (z.0, z.1.min(sy - k), z.2, z.3));
    }
    couvert
}
