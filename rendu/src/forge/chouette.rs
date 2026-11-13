//! La chouette mécanique messagère (user, 12-11 : la touche steampunk ; 13-11 : « la chouette messagère mécanique,
//! oui »), dessinée à la main en lettres : de laiton et de cuivre, ses lentilles cyan qui luisent, un engrenage au
//! flanc, le rouleau de parchemin scellé dans les serres. Elle remplace au pixel d'art (`pet`) la chouette blanche en
//! cellules d'atelier.rs, à la même place (9 × 5 pixels de forge, le coin haut gauche en `x`, `y`) ; la cour la fait
//! aussi passer dans le ciel du soir (paysage.rs).

use super::pet::Eclairage;

// Tournée vers la gauche : ailes hautes, ailes basses.
pub const CHOUETTE_HAUTES: [&str; 18] = ["......k.k.k.......", ".....kYkYkYk......", ".....kYyYyYyk.....", "......kYyyyyzk....", ".......kYyyzzk....", "..k..k..kYyzk.....", ".kYkkYk..kyzk.....", "kYYYYYYk.kzk......", "kYEEEYYYkkkkkkk...", "kEeeeEyCCCCCCCCkk.", "zEeeeEyCCGgGCCcckk", "kzEEEyyCCgkgCcccck", ".kyyyyyCCGgGCcckk.", "..kyyyycCCCCcck...", "...kkkyyyyyykk....", ".....kzkkzk.......", "....kWWWRWWk......", ".....kkkkkk......."];
pub const CHOUETTE_BASSES: [&str; 18] = ["..................", "..................", "..................", "..................", "..................", "..k..k............", ".kYkkYk...........", "kYYYYYYk..........", "kYEEEYYYkkkkkkk...", "kEeeeEyCCCCCCCCkk.", "zEeeeEyCCGgGCCcckk", "kzEEEyyCCgkgCcccck", ".kyyyyyCCGgGCcckk.", "..kyyyycYYYYyzk...", "...kkkyyYyyyyzk...", ".....kzkYyYyYyk...", "....kWWkYkYkYk....", ".....kkkk.k.k....."];


/// Une chouette en vol telle qu'atelier.rs l'a placée.
#[derive(Clone, Debug)]
pub struct Vol {
    /// le coin haut gauche de la chouette en cellules (9 × 5), en pixels de forge
    pub x: i32,
    pub y: i32,
    pub age: i64,
    /// elle porte son rouleau
    pub porte: bool,
    /// −1 vers la gauche (son dessin), 1 vers la droite
    pub sens: i32,
}

/// Où elle se tient, en pixels de forge, de quoi couvrir ses ailes levées et son rouleau.
pub fn emprise(v: &Vol) -> (i32, i32, i32, i32) {
    (v.x - 2, v.y - 4, 13, 12)
}

/// La couleur d'une lettre de la chouette, et si elle luit d'elle-même (les lentilles). `porte` faux : sans le rouleau.
pub fn couleur(l: u8, porte: bool) -> Option<(u32, bool)> {
    Some(match l {
        b'k' => (0x1b1924, false),
        b'Y' => (0xf4d070, false),
        b'y' => (0xc89038, false),
        b'z' => (0x7a5420, false),
        b'C' => (0xd88050, false),
        b'c' => (0x8a4428, false),
        b'G' => (0xb8c0d0, false),
        b'g' => (0x5a6070, false),
        b'E' => (0x2a3040, false),
        b'e' => (0x7af0ff, true),
        b'W' if porte => (0xf2e6c6, false),
        b'w' if porte => (0xc0ac88, false),
        b'R' if porte => (0xc82828, false),
        _ => return None,
    })
}

/// L'image du battement : ailes hautes deux tics, basses deux tics.
pub fn image(age: i64) -> &'static [&'static str] {
    if age.rem_euclid(4) < 2 {
        &CHOUETTE_HAUTES
    } else {
        &CHOUETTE_BASSES
    }
}

/// La peint au pixel d'art, chaque lettre un carré de pixels d'écran entier, le rouleau en bas, sous la lumière de
/// la forge (ses lentilles luisent d'elles-mêmes). Rend ce qu'elle a couvert.
#[allow(clippy::too_many_arguments)]
pub fn peindre(v: &Vol, e: Option<&Eclairage>, pixels: &mut [u32], w: usize, h: usize, x0: i32, cw: i32, ch: i32) -> Option<(i32, i32, i32, i32)> {
    let image = image(v.age);
    let (cwf, demi) = (cw as f32, ch as f32 / 2.0);
    // le grain des louveteaux ; le rouleau en bas, là où les serres le tenaient en cellules (deux sous l'oiseau)
    let k = ((demi * 0.34).round() as i32).max(1);
    let (iw, ih) = (image[0].len() as i32, image.len() as i32);
    let (cx, bas) = ((((v.x - x0) as f32 + 4.5) * cwf).round() as i32, ((v.y as f32 + 6.5) * demi).round() as i32);
    let porte = v.porte;
    peindre_image(image, &|l| couleur(l, porte), v.sens > 0, cx - iw * k / 2, bas - ih * k, k, e, pixels, w, h, x0, cw, ch)
}

/// Une image en lettres au pixel d'art, son coin haut gauche en (`gauche`, `haut`) pixels d'écran, chaque lettre un
/// carré de `k` pixels, retournée si `miroir` ; chaque lettre prend la lumière de la forge (`e`) au pixel de forge
/// où elle tombe, sauf celles qui luisent d'elles-mêmes. Rend ce qu'elle a couvert.
#[allow(clippy::too_many_arguments)]
pub fn peindre_image(
    image: &[&str],
    couleur: &dyn Fn(u8) -> Option<(u32, bool)>,
    miroir: bool,
    gauche: i32,
    haut: i32,
    k: i32,
    e: Option<&Eclairage>,
    pixels: &mut [u32],
    w: usize,
    h: usize,
    x0: i32,
    cw: i32,
    ch: i32,
) -> Option<(i32, i32, i32, i32)> {
    let (cwf, demi) = (cw as f32, ch as f32 / 2.0);
    let iw = image[0].len();
    let mut couvert: Option<(i32, i32, i32, i32)> = None;
    for (j, ligne) in image.iter().enumerate() {
        let ligne = ligne.as_bytes();
        for i in 0..iw {
            let lettre = if miroir { ligne[iw - 1 - i] } else { ligne[i] };
            let Some((hex, luit)) = couleur(lettre) else { continue };
            let col = [((hex >> 16) & 255) as f32, ((hex >> 8) & 255) as f32, (hex & 255) as f32];
            let (sx, sy) = (gauche + i as i32 * k, haut + j as i32 * k);
            let l = match e.filter(|_| !luit) {
                Some(e) => {
                    let fx = ((sx + k / 2) as f32 / cwf).floor() as i32 + x0 - e.x0;
                    let fy = ((sy + k / 2) as f32 / demi).floor() as i32 - e.y0;
                    e.k[(fy.clamp(0, e.h - 1) * e.w + fx.clamp(0, e.w - 1)) as usize]
                }
                None => [1.0; 3],
            };
            let p = 0xFF00_0000 | ((col[0] * l[0]).clamp(0.0, 255.0) as u32) << 16 | ((col[1] * l[1]).clamp(0.0, 255.0) as u32) << 8 | (col[2] * l[2]).clamp(0.0, 255.0) as u32;
            let (xa, xb, ya, yb) = (sx.max(0), (sx + k).min(w as i32), sy.max(0), (sy + k).min(h as i32));
            for y in ya..yb {
                for x in xa..xb {
                    pixels[y as usize * w + x as usize] = p;
                }
            }
            if xa < xb && ya < yb {
                couvert = Some(couvert.map_or((xa, ya, xb, yb), |z| (z.0.min(xa), z.1.min(ya), z.2.max(xb), z.3.max(yb))));
            }
        }
    }
    couvert
}
