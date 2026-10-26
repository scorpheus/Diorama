//! Les jauges au sol : quand le panneau est discret, le contexte, la fenêtre de 5 h et celle de
//! 7 jours se peignent sur la dernière rangée de la scène, côte à côte et calées à droite, en
//! petites lettres de pixels cernées d'ombre, pour se lire sur le quai, l'herbe ou le pont.

use super::monde::Monde;
use super::trace::{hex, smooth, unpack, Rgb, Toile};

/// Une lettre, trois pixels sur cinq : une rangée par entier, le bit fort à gauche.
fn lettre(c: char) -> [u8; 5] {
    match c {
        '0' => [7, 5, 5, 5, 7],
        '1' => [2, 6, 2, 2, 7],
        '2' => [7, 1, 7, 4, 7],
        '3' => [7, 1, 7, 1, 7],
        '4' => [5, 5, 7, 1, 1],
        '5' => [7, 4, 7, 1, 7],
        '6' => [7, 4, 7, 5, 7],
        '7' => [7, 1, 1, 2, 2],
        '8' => [7, 5, 7, 5, 7],
        '9' => [7, 5, 7, 1, 7],
        '%' => [5, 1, 2, 4, 5],
        'c' => [0, 3, 4, 4, 3],
        't' => [2, 7, 2, 2, 1],
        'x' => [0, 5, 2, 2, 5],
        'h' => [4, 4, 7, 5, 5],
        'j' => [1, 0, 1, 5, 2],
        _ => [0; 5],
    }
}

/// Un pixel de lettre, en pixels d'écran ; une lettre avance de quatre.
const P: i32 = 2;
const AVANCE: i32 = 4 * P;
/// La barre de niveau, entre le nom et le pourcentage.
const BARRE: i32 = 26;
const ESPACE: i32 = 4;
/// Entre deux jauges.
const ENTRE: i32 = 18;

const ENCRE: Rgb = hex(0xf2e8d0);
const OMBRE: Rgb = hex(0x120c08);

/// La teinte d'un niveau : celles du panneau (vert, jaune passé 60, rouge passé 85).
fn teinte(p: f32) -> Rgb {
    if p >= 85.0 {
        hex(0xef6b6b)
    } else if p >= 60.0 {
        hex(0xf2c14e)
    } else {
        hex(0x6fcf7f)
    }
}

/// Des rectangles à peindre : (x, y, largeur, hauteur, couleur).
type Rects = Vec<(i32, i32, i32, i32, Rgb)>;

fn ecrire(r: &mut Rects, texte: &str, x: i32, y: i32, c: Rgb) {
    for (i, ch) in texte.chars().enumerate() {
        for (rangee, bits) in lettre(ch).iter().enumerate() {
            for col in 0..3 {
                if bits >> (2 - col) & 1 == 1 {
                    r.push((x + i as i32 * AVANCE + col * P, y + rangee as i32 * P, P, P, c));
                }
            }
        }
    }
}

const fn largeur(texte: &str) -> i32 {
    texte.len() as i32 * AVANCE - P
}

/// Les trois jauges sur la rangée du bas d'une scène de `w` × `h` pixels ; ce qu'elles couvrent.
pub fn peindre(px: &mut [u32], w: usize, h: usize, cw: i32, ch: i32, monde: &Monde) -> Option<(i32, i32, i32, i32)> {
    if monde.gauges != Some(true) {
        return None;
    }
    let jauges: Vec<(&str, f32, String)> = [("ctx", Some(monde.ctx)), ("5h", monde.fuel5h), ("7j", monde.fuel7d)]
        .into_iter()
        .filter_map(|(nom, p)| p.map(|p| (nom, p.clamp(0.0, 100.0) as f32, format!("{}%", p.round() as i32))))
        .collect();
    if jauges.is_empty() {
        return None;
    }
    let une = |nom: &str, pc: &str| largeur(nom) + ESPACE + BARRE + ESPACE + largeur(pc);
    let total: i32 = jauges.iter().map(|(n, _, pc)| une(n, pc)).sum::<i32>() + ENTRE * (jauges.len() as i32 - 1);
    // calées à droite, une cellule du bord ; les lettres au milieu de la rangée du bas
    let y = h as i32 - ch + (ch - 5 * P) / 2;
    let mut x = (w as i32 - cw - total).max(0);
    let gauche = x;
    let (mut fond, mut dessus): (Rects, Rects) = (Vec::new(), Vec::new());
    for (nom, p, pc) in &jauges {
        ecrire(&mut dessus, nom, x, y, ENCRE);
        x += largeur(nom) + ESPACE;
        // la barre : un creux sombre, rempli à la teinte du niveau
        let (by, bh) = (y + P, 3 * P);
        fond.push((x, by, BARRE, bh, OMBRE));
        let plein = ((p / 100.0) * BARRE as f32).round() as i32;
        if plein > 0 {
            dessus.push((x, by, plein, bh, teinte(*p)));
        }
        x += BARRE + ESPACE;
        // le pourcentage, collé à sa barre
        ecrire(&mut dessus, pc, x, y, teinte(*p));
        x += largeur(pc) + ENTRE;
    }
    // la clarté du sol dessous : sur la neige ou le sable, l'ombre se fonce et une plaque passe dessous
    let (y0, y1, x1) = (y - 3, y + 5 * P + 3, (gauche + total + 3).min(w as i32));
    let (x0, y0, y1) = ((gauche - 3).max(0), y0.max(0), y1.min(h as i32));
    let mut somme = 0.0;
    for yy in y0..y1 {
        for xx in x0..x1 {
            let c = unpack(px[yy as usize * w + xx as usize]);
            somme += 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
        }
    }
    let clair = somme / ((y1 - y0) * (x1 - x0)).max(1) as f32;
    let mut t = Toile { px, w, h };
    let plaque = 0.5 * smooth(120.0, 200.0, clair);
    if plaque > 0.0 {
        for yy in y0..y1 {
            for xx in x0..x1 {
                // les coins arrondis d'un pixel
                let coin = (yy == y0 || yy == y1 - 1) && (xx == x0 || xx == x1 - 1);
                t.poser(xx, yy, OMBRE, if coin { plaque * 0.4 } else { plaque });
            }
        }
    }
    // l'ombre d'abord, un pixel autour de tout, puis le creux des barres, puis les lettres
    let ombre = 0.55 + 0.35 * smooth(100.0, 180.0, clair);
    for &(rx, ry, rw, rh, _) in fond.iter().chain(&dessus) {
        for yy in ry - 1..ry + rh + 1 {
            for xx in rx - 1..rx + rw + 1 {
                t.poser(xx, yy, OMBRE, ombre);
            }
        }
    }
    for &(rx, ry, rw, rh, c) in fond.iter().chain(&dessus) {
        for yy in ry..ry + rh {
            for xx in rx..rx + rw {
                t.poser(xx, yy, c, 1.0);
            }
        }
    }
    Some((x0, y0, x1, y1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_jauges_se_peignent_au_sol_et_nulle_part_ailleurs() {
        let (cw, ch, cols) = (9, 19, 80);
        let (w, h) = ((cols * cw) as usize, (16 * ch) as usize);
        let mut monde = Monde { ctx: 64.0, fuel5h: Some(27.0), fuel7d: Some(91.0), ..Monde::default() };
        let mut px = vec![0xFF20_3040u32; w * h];
        assert!(peindre(&mut px, w, h, cw, ch, &monde).is_none(), "sans le drapeau, rien");
        monde.gauges = Some(true);
        let (l, t, r, b) = peindre(&mut px, w, h, cw, ch, &monde).expect("les jauges se peignent");
        // sur la rangée du bas, à droite, avant la marge d'une cellule (la plaque y déborde de trois pixels)
        assert!(t >= h as i32 - ch && b <= h as i32 && r <= w as i32 - cw + 3 && l > 0, "{l} {t} {r} {b}");
        let change = |y0: usize, y1: usize| (y0..y1).flat_map(|y| (0..w).map(move |x| (x, y))).filter(|&(x, y)| px[y * w + x] != 0xFF20_3040).count();
        assert_eq!(change(0, h - ch as usize), 0, "rien au-dessus du sol");
        // contrat de sortie : des lettres et des barres, pas trois points
        assert!(change(h - ch as usize, h) > 600, "{} pixels", change(h - ch as usize, h));
        // la teinte suit le niveau : vert (27 %), jaune (64 %), rouge (91 %)
        let a = |c: u32| px.iter().filter(|&&p| p & 0xffffff == c).count();
        assert!(a(0x6fcf7f) > 0 && a(0xf2c14e) > 0 && a(0xef6b6b) > 0);
    }
}
