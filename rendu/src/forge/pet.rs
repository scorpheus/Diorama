//! Scorpheus d'après son pet de Codex (rendu/pet/scorpheus.webp : 8 × 11 cases de 192 × 208,
//! une rangée par état, cf. pet.json) : peint au vrai pixel de l'écran par-dessus la forge, à la
//! place du personnage en lettres, de la taille de celui-ci, et sous les mêmes lampes.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use super::monde::{Activite, Monde};

const PLANCHE: &[u8] = include_bytes!("../../pet/scorpheus.webp");
const CASE: (usize, usize) = (192, 208);
const COLONNES: usize = 8;
/// Le point de la case qui se pose au sol : sous ses pattes, au milieu.
const PIED: (f32, f32) = (96.0, 203.0);
/// Sa hauteur dans la case, de la crête aux pattes ; et dans la forge, en pixels de forge :
/// celle du personnage en lettres.
const HAUTEUR_CASE: f32 = 198.0;
pub const HAUTEUR: f32 = 26.0;

/// Les rangées de la planche, et combien d'images chacune tient.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Rangee {
    Repos = 0,
    Droite = 1,
    Gauche = 2,
    Salut = 3,
    Saut = 4,
    Echec = 5,
    Attente = 6,
    Travail = 7,
    Relecture = 8,
}
const IMAGES: [usize; 9] = [6, 8, 8, 4, 5, 8, 6, 6, 6];

/// L'image du moment : sa rangée et sa case.
pub fn moment(a: Activite, tick: i64, w: &Monde) -> (Rangee, usize) {
    use Rangee::*;
    let boucle = |r: Rangee, tics: i64| (r, (tick.div_euclid(tics) as usize) % IMAGES[r as usize]);
    if w.walking {
        // un pas d'image par pixel marché : ses pattes suivent le sol
        let r = if w.facing < 0 { Gauche } else { Droite };
        return (r, (w.x * w.facing.signum()).rem_euclid(8) as usize);
    }
    match a {
        Activite::Running | Activite::Quench | Activite::Sweep => boucle(Travail, 2),
        Activite::Thinking => (Relecture, [1, 1, 2, 2, 3, 3][(tick.div_euclid(3) % 6) as usize]),
        Activite::Review => boucle(Relecture, 3),
        Activite::Waiting => boucle(Attente, 3),
        Activite::Failed => (Echec, [0, 1, 2, 2, 7, 6][(tick.div_euclid(3) % 6) as usize]),
        Activite::Jumping => boucle(Saut, 2),
        Activite::Waving | Activite::Ship => boucle(Salut, 2),
        // couché, endormi ; puis il se relève
        Activite::Sleep => (Echec, 4),
        Activite::Waking => (Echec, (5 + tick.div_euclid(3).max(0) as usize).min(7)),
        Activite::Idle | Activite::Away => boucle(Repos, 3),
    }
}

/// Une case mise à l'échelle de l'écran, rognée à ce qu'elle montre.
pub struct Case {
    /// son coin haut gauche, depuis le point qui se pose au sol, en pixels d'écran
    pub dx: i32,
    pub dy: i32,
    pub w: usize,
    pub h: usize,
    /// 0xAARRGGBB, alpha prémultiplié ; le relief déjà posé, divisé par RELIEF
    pub px: Vec<u32>,
}

/// Le relief éclaircit au plus de 1 + 0,24 × 1,25 : les couleurs le portent divisées d'autant,
/// la lumière le leur rend.
const RELIEF: f32 = 1.3;

/// Le relief d'un pixel (sa part `t` de la rangée, de gauche à droite ; `haut` : rien au-dessus) :
/// le côté vers le foyer (à gauche) chaud, l'autre froid, un liseré de lumière en haut.
fn relief(t: f32, haut: bool) -> [f32; 3] {
    let m = 0.16 - 0.32 * t.clamp(0.0, 1.0) + if haut { 0.08 } else { 0.0 };
    if m > 0.0 {
        [1.0 + m * 1.25, 1.0 + m, 1.0 + m * 0.55]
    } else {
        [1.0 + m * 1.2, 1.0 + m, 1.0 + m * 0.3]
    }
}

/// Toutes les cases, à une hauteur de cellule donnée.
pub struct Cases(HashMap<(Rangee, usize), Case>);

impl Cases {
    pub fn case(&self, r: Rangee, i: usize) -> Option<&Case> {
        self.0.get(&(r, i))
    }
}

/// Les cases, à la hauteur de cellule `ch` : faites une fois pour toutes les bandes de cette
/// taille, la planche décodée seulement le temps de les faire.
pub fn cases(ch: u32) -> Option<Arc<Cases>> {
    static FAITES: OnceLock<Mutex<HashMap<u32, Arc<Cases>>>> = OnceLock::new();
    let mut faites = FAITES.get_or_init(Default::default).lock().ok()?;
    if let Some(c) = faites.get(&ch) {
        return Some(c.clone());
    }
    let (w, rgba) = decoder()?;
    let echelle = HAUTEUR * ch as f32 / 2.0 / HAUTEUR_CASE;
    let mut toutes = HashMap::new();
    for (r, rangee) in [
        Rangee::Repos,
        Rangee::Droite,
        Rangee::Gauche,
        Rangee::Salut,
        Rangee::Saut,
        Rangee::Echec,
        Rangee::Attente,
        Rangee::Travail,
        Rangee::Relecture,
    ]
    .into_iter()
    .enumerate()
    {
        for i in 0..IMAGES[r].min(COLONNES) {
            toutes.insert((rangee, i), case(&rgba, w, i * CASE.0, r * CASE.1, echelle));
        }
    }
    let c = Arc::new(Cases(toutes));
    faites.insert(ch, c.clone());
    Some(c)
}

/// La planche en RGBA (alpha prémultiplié, en flottants) et sa largeur.
fn decoder() -> Option<(usize, Vec<[f32; 4]>)> {
    let mut d = image_webp::WebPDecoder::new(std::io::Cursor::new(PLANCHE)).ok()?;
    let (w, h) = d.dimensions();
    let mut brut = vec![0u8; d.output_buffer_size()?];
    d.read_image(&mut brut).ok()?;
    let n = if d.has_alpha() { 4 } else { 3 };
    let rgba = brut
        .chunks_exact(n)
        .map(|p| {
            let a = if n == 4 { p[3] as f32 / 255.0 } else { 1.0 };
            [p[0] as f32 * a, p[1] as f32 * a, p[2] as f32 * a, a * 255.0]
        })
        .collect::<Vec<_>>();
    (rgba.len() == (w * h) as usize).then_some((w as usize, rgba))
}

/// Lanczos à deux lobes : net sans trop d'auréole sur les contours d'un pixel art.
fn lanczos(x: f32) -> f32 {
    let x = x.abs();
    if x < 1e-5 {
        1.0
    } else if x >= 2.0 {
        0.0
    } else {
        let p = std::f32::consts::PI * x;
        2.0 * p.sin() * (p / 2.0).sin() / (p * p)
    }
}

/// Pour chaque pixel de sortie, le premier pixel source et ses poids (somme 1).
fn poids(n: usize, sortie: usize, echelle: f32) -> Vec<(usize, Vec<f32>)> {
    let large = (1.0 / echelle).max(1.0); // en réduisant, le noyau s'élargit
    (0..sortie)
        .map(|o| {
            let centre = (o as f32 + 0.5) / echelle - 0.5;
            let (a, b) = ((centre - 2.0 * large).floor().max(0.0) as usize, ((centre + 2.0 * large).ceil() as usize).min(n - 1));
            let mut p: Vec<f32> = (a..=b).map(|s| lanczos((s as f32 - centre) / large)).collect();
            let somme: f32 = p.iter().sum();
            p.iter_mut().for_each(|v| *v /= somme);
            (a, p)
        })
        .collect()
}

fn case(rgba: &[[f32; 4]], largeur: usize, x0: usize, y0: usize, echelle: f32) -> Case {
    let (sw, sh) = ((CASE.0 as f32 * echelle).round() as usize, (CASE.1 as f32 * echelle).round() as usize);
    let (px_, py_) = (poids(CASE.0, sw, echelle), poids(CASE.1, sh, echelle));
    // en largeur d'abord, puis en hauteur
    let mut large = vec![[0f32; 4]; sw * CASE.1];
    for y in 0..CASE.1 {
        let ligne = &rgba[(y0 + y) * largeur + x0..(y0 + y) * largeur + x0 + CASE.0];
        for (o, (a, p)) in px_.iter().enumerate() {
            let mut s = [0f32; 4];
            for (k, w) in p.iter().enumerate() {
                let v = ligne[a + k];
                for c in 0..4 {
                    s[c] += v[c] * w;
                }
            }
            large[y * sw + o] = s;
        }
    }
    let mut out = vec![0u32; sw * sh];
    for (o, (a, p)) in py_.iter().enumerate() {
        for x in 0..sw {
            let mut s = [0f32; 4];
            for (k, w) in p.iter().enumerate() {
                let v = large[(a + k) * sw + x];
                for c in 0..4 {
                    s[c] += v[c] * w;
                }
            }
            let al = s[3].clamp(0.0, 255.0);
            // presque rien : rien, que le contour reste net sur le décor
            if al < 8.0 {
                continue;
            }
            let c = |v: f32| v.clamp(0.0, al).round() as u32;
            out[o * sw + x] = ((al.round() as u32) << 24) | (c(s[0]) << 16) | (c(s[1]) << 8) | c(s[2]);
        }
    }
    // rognée à ce qui se voit
    let (mut gx, mut gy, mut dx, mut dy) = (sw, sh, 0, 0);
    for y in 0..sh {
        for x in 0..sw {
            if out[y * sw + x] != 0 {
                (gx, gy, dx, dy) = (gx.min(x), gy.min(y), dx.max(x + 1), dy.max(y + 1));
            }
        }
    }
    if gx >= dx {
        return Case { dx: 0, dy: 0, w: 0, h: 0, px: Vec::new() };
    }
    let (w, h) = (dx - gx, dy - gy);
    let mut px = Vec::with_capacity(w * h);
    let plein = |p: u32| p >> 24 >= 128;
    for y in gy..dy {
        let ligne = &out[y * sw + gx..y * sw + dx];
        let (a, b) = (ligne.iter().position(|&p| plein(p)).unwrap_or(0), ligne.iter().rposition(|&p| plein(p)).unwrap_or(0));
        for (i, &p) in ligne.iter().enumerate() {
            let haut = y == 0 || !plein(out[(y - 1) * sw + gx + i]);
            let k = relief(if b > a { (i as f32 - a as f32) / (b - a) as f32 } else { 0.5 }, haut);
            let canal = |s: u32, k: f32| ((((p >> s) & 255) as f32 * k / RELIEF).round() as u32).min(p >> 24) << s;
            px.push((p & 0xFF00_0000) | canal(16, k[0]) | canal(8, k[1]) | canal(0, k[2]));
        }
    }
    Case {
        dx: gx as i32 - (PIED.0 * echelle).round() as i32,
        dy: gy as i32 - (PIED.1 * echelle).round() as i32,
        w,
        h,
        px,
    }
}

/// Ce que les lampes posent autour de lui : un facteur par canal pour chaque pixel de forge
/// d'un rectangle, lu entre les centres des pixels pour ne pas faire de paliers.
pub struct Eclairage {
    pub x0: i32,
    pub y0: i32,
    pub w: i32,
    pub h: i32,
    pub k: Vec<[f32; 3]>,
}

/// Entre quels deux pixels de forge tombe `f` (en pixels de forge, depuis le premier de `n`), et à
/// quelle part du second, en 256es.
fn entre(f: f32, n: i32) -> (usize, i32) {
    let i = (f.floor() as i32).clamp(0, (n - 2).max(0));
    (i as usize, ((f - i as f32).clamp(0.0, 1.0) * 256.0) as i32)
}

/// Où et comment il se tient : la case du moment, son point au sol en pixels d'écran, et la
/// lumière autour (aucune : ses couleurs telles quelles).
pub struct Acteur {
    pub rangee: Rangee,
    pub image: usize,
    /// le point au sol, en pixels de forge
    pub x: f32,
    pub y: f32,
    pub eclairage: Option<Eclairage>,
}

/// Peint `a` dans `pixels` (0xAARRGGBB prémultiplié, `w` × `h`) ; `x0` : la première colonne
/// de forge montrée ; (cw, ch) : la cellule. Rend le rectangle peint.
#[allow(clippy::too_many_arguments)]
pub fn peindre(cases: &Cases, a: &Acteur, pixels: &mut [u32], w: usize, h: usize, x0: i32, cw: i32, ch: i32) -> Option<(i32, i32, i32, i32)> {
    let c = cases.case(a.rangee, a.image)?;
    if c.w == 0 {
        return None;
    }
    let demi = ch as f32 / 2.0; // un pixel de forge en hauteur
    let ox = ((a.x - x0 as f32) * cw as f32).round() as i32 + c.dx;
    let oy = (a.y * demi).round() as i32 + c.dy;
    // les colonnes et les rangées de la case qui tombent dans l'image
    let (i0, i1) = ((-ox).max(0) as usize, (w as i32 - ox).clamp(0, c.w as i32) as usize);
    let (j0, j1) = ((-oy).max(0) as usize, (h as i32 - oy).clamp(0, c.h as i32) as usize);
    if i0 >= i1 || j0 >= j1 {
        return None;
    }
    // la lumière lue entre les centres des pixels de forge : la part horizontale est la même à
    // chaque rangée, la verticale la même sur toute une rangée
    // sans lumière, la sienne : ses couleurs telles quelles, relief compris
    let (ew, eh, ex0, ey0) = a.eclairage.as_ref().map_or((2, 2, 0, 0), |e| (e.w, e.h, e.x0, e.y0));
    let fixe = |k: f32| (k * RELIEF * 256.0).round() as i32;
    let k: Vec<[i32; 3]> = match &a.eclairage {
        Some(e) => e.k.iter().map(|v| [fixe(v[0]), fixe(v[1]), fixe(v[2])]).collect(),
        None => vec![[fixe(1.0); 3]; 4],
    };
    let colonnes: Vec<(usize, i32)> = (0..c.w).map(|i| entre((ox + i as i32) as f32 / cw as f32 + x0 as f32 - 0.5 - ex0 as f32, ew)).collect();
    let mut rangee: Vec<[i32; 3]> = vec![[0; 3]; ew as usize + 1];
    for j in j0..j1 {
        let y = oy + j as i32;
        let (iy, ty) = entre(y as f32 / demi - 0.5 - ey0 as f32, eh);
        let (haut, bas) = (&k[iy * ew as usize..][..ew as usize], &k[(iy + 1).min(eh as usize - 1) * ew as usize..][..ew as usize]);
        for (r, (u, v)) in rangee.iter_mut().zip(haut.iter().zip(bas)) {
            *r = [u[0] + (((v[0] - u[0]) * ty) >> 8), u[1] + (((v[1] - u[1]) * ty) >> 8), u[2] + (((v[2] - u[2]) * ty) >> 8)];
        }
        rangee[ew as usize] = rangee[ew as usize - 1];
        let ligne = &c.px[j * c.w..(j + 1) * c.w];
        let sortie = &mut pixels[y as usize * w..(y as usize + 1) * w];
        for i in i0..i1 {
            let s = ligne[i];
            let al = s >> 24;
            if al == 0 {
                continue;
            }
            let (ix, tx) = colonnes[i];
            let (u, v) = (rangee[ix], rangee[ix + 1]);
            let canal = |n: u32, c: usize| {
                let k = u[c] + (((v[c] - u[c]) * tx) >> 8);
                ((((s >> n) & 255) as i32 * k) >> 8).clamp(0, al as i32) as u32
            };
            let (r, g, b) = (canal(16, 0), canal(8, 1), canal(0, 2));
            let d = &mut sortie[(ox + i as i32) as usize];
            if al == 255 {
                *d = 0xFF00_0000 | (r << 16) | (g << 8) | b;
            } else {
                // par-dessus, en prémultiplié
                let inv = 255 - al;
                let mel = |s: u32, d: u32| s + (d * inv + 127) / 255;
                let da = *d;
                *d = (mel(al, da >> 24) << 24) | (mel(r, (da >> 16) & 255) << 16) | (mel(g, (da >> 8) & 255) << 8) | mel(b, da & 255);
            }
        }
    }
    Some((ox + i0 as i32, oy + j0 as i32, ox + i1 as i32, oy + j1 as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_planche_se_decoupe_en_cases_a_la_taille_du_personnage() {
        let c = cases(19).expect("la planche se décode");
        let repos = c.case(Rangee::Repos, 0).unwrap();
        // 26 pixels de forge de 9,5 px : ~247 px de haut, posés au sol
        assert!((235..=260).contains(&repos.h), "hauteur {}", repos.h);
        assert!(repos.dy + repos.h as i32 <= 2 && repos.dy + repos.h as i32 >= -4, "pieds à {}", repos.dy + repos.h as i32);
        for (r, n) in [(Rangee::Travail, 6), (Rangee::Droite, 8), (Rangee::Echec, 8)] {
            for i in 0..n {
                assert!(c.case(r, i).is_some_and(|k| k.w > 0), "{r:?} {i}");
            }
        }
    }
}
