//! Scorpheus d'après son pet de Codex (rendu/pet/scorpheus.webp : 8 × 11 cases de 192 × 208,
//! une rangée par état, cf. pet.json), et ce que la forge lui demande en plus, dessiné au pixel
//! dans son style (rendu/pet/forge.webp, mêmes cases) : peint au vrai pixel de l'écran par-dessus
//! la forge, à la place du personnage en lettres, de la taille de celui-ci, et sous les mêmes lampes.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use super::monde::{Accessoire, Activite, Monde, Perso};

const PLANCHES: [&[u8]; 2] = [include_bytes!("../../pet/scorpheus.webp"), include_bytes!("../../pet/forge.webp")];
/// Galadriel d'après son pet de Codex : neuf rangées pleines de huit cases, détourée sur du vert.
const GALADRIEL: &[u8] = include_bytes!("../../pet/galadriel.webp");
/// Les accessoires, chacun un calque posé sur les cases (tools/pet/chatgpt.py --tenue) : les
/// rangées 0 à 8 de la première planche, puis les six de la seconde. Une image d'un pixel :
/// pas encore dessiné, le pet sans.
const CALQUES: [(Accessoire, &[u8]); 3] = [
    (Accessoire::Lunettes, include_bytes!("../../pet/lunettes.webp")),
    (Accessoire::Tablier, include_bytes!("../../pet/tablier.webp")),
    (Accessoire::Bandeau, include_bytes!("../../pet/bandeau.webp")),
];
/// Les rangées de la première planche que le calque couvre.
const RANGEES_PET: usize = 9;
const CASE: (usize, usize) = (192, 208);
const COLONNES: usize = 8;
/// Le point de la case qui se pose au sol : sous ses pattes, au milieu.
const PIED: (f32, f32) = (96.0, 203.0);
/// Sa hauteur dans la case, de la crête aux pattes ; et dans la forge, en pixels de forge :
/// celle du personnage en lettres.
const HAUTEUR_CASE: f32 = 198.0;
pub const HAUTEUR: f32 = 26.0;

/// Les rangées des planches.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Rangee {
    Repos,
    Droite,
    Gauche,
    Salut,
    Saut,
    Echec,
    Attente,
    Travail,
    Relecture,
    /// le coup de marteau sur l'enclume : au repos, levé, abattu, la frappe
    Frappe,
    /// le levier du soufflet : haut, pressé
    Soufflet,
    /// la trempe : la lame levée, puis dans le seau
    Trempe,
    /// l'envoi : la lame rangée dans la caisse en quatre temps, le couvercle, le sceau levé, frappé
    Envoi,
    /// le balai que la forge dessine, à chaque allonge du coup (0 à 4)
    Balai,
    /// assis sur son tabouret, les yeux fermés ; la tête qui dodeline
    Sommeil,
}

const RANGEES: [Rangee; 15] = [
    Rangee::Repos,
    Rangee::Droite,
    Rangee::Gauche,
    Rangee::Salut,
    Rangee::Saut,
    Rangee::Echec,
    Rangee::Attente,
    Rangee::Travail,
    Rangee::Relecture,
    Rangee::Frappe,
    Rangee::Soufflet,
    Rangee::Trempe,
    Rangee::Envoi,
    Rangee::Balai,
    Rangee::Sommeil,
];

impl Rangee {
    /// Sa planche, sa rangée dans la planche, et combien d'images elle tient, chez `perso` ;
    /// Galadriel n'a que les neuf rangées de son pet, de huit images chacune.
    const fn ou_chez(self, perso: Perso) -> Option<(usize, usize, usize)> {
        match perso {
            Perso::Scorpheus => Some(self.ou()),
            Perso::Galadriel => {
                let (p, r, _) = self.ou();
                if p == 0 {
                    Some((0, r, 8))
                } else {
                    None
                }
            }
        }
    }

    /// Sa planche, sa rangée dans la planche, et combien d'images elle tient.
    const fn ou(self) -> (usize, usize, usize) {
        match self {
            Rangee::Repos => (0, 0, 6),
            Rangee::Droite => (0, 1, 8),
            Rangee::Gauche => (0, 2, 8),
            Rangee::Salut => (0, 3, 4),
            Rangee::Saut => (0, 4, 5),
            Rangee::Echec => (0, 5, 8),
            Rangee::Attente => (0, 6, 6),
            Rangee::Travail => (0, 7, 6),
            Rangee::Relecture => (0, 8, 6),
            Rangee::Frappe => (1, 0, 4),
            Rangee::Soufflet => (1, 1, 2),
            Rangee::Trempe => (1, 2, 2),
            Rangee::Envoi => (1, 3, 7),
            Rangee::Balai => (1, 4, 5),
            Rangee::Sommeil => (1, 5, 2),
        }
    }
}

/// De combien de pixels de forge il se décale de sa place : à l'enclume (le marteau, la trempe,
/// l'envoi), il s'en approche assez pour que sa patte y arrive, la tête du marteau sur le lingot.
/// Tout le moment durant, pour ne pas sauter d'une image à l'autre.
pub fn decalage(a: Activite, w: &Monde) -> f32 {
    match a {
        _ if w.walking => 0.0,
        Activite::Running if w.flavor != super::monde::Saveur::Build => -4.0,
        Activite::Quench | Activite::Ship => -4.0,
        // vers la cour : la tête du balai reste dans sa case
        Activite::Sweep => 2.0,
        _ => 0.0,
    }
}

/// L'image du moment : sa rangée et sa case.
pub fn moment(a: Activite, tick: i64, w: &Monde) -> (Rangee, usize) {
    use Rangee::*;
    let boucle = |r: Rangee, tics: i64| (r, (tick.div_euclid(tics) as usize) % r.ou().2);
    if w.walking {
        // un pas d'image par pixel marché : ses pattes suivent le sol
        let r = if w.facing < 0 { Gauche } else { Droite };
        return (r, (w.x * w.facing.signum()).rem_euclid(8) as usize);
    }
    match a {
        // le coup suit la pose de la forge (poses.rs, running) : levé, abattu, la frappe au
        // quatrième tic (la gerbe part), puis au repos et de nouveau abattu pour remonter
        Activite::Running if w.flavor != super::monde::Saveur::Build => (Frappe, [1, 1, 2, 3, 3, 0, 0, 2][tick.rem_euclid(8) as usize]),
        // la patte suit le levier, pressé trois tics sur six (poses.rs, running)
        Activite::Running => (Soufflet, (tick.rem_euclid(6) < 3) as usize),
        // la lame levée, au seau du quatrième au neuvième tic, puis levée de nouveau (poses.rs, quench)
        Activite::Quench => (Trempe, (3..9).contains(&tick) as usize),
        // rangée, le couvercle, le sceau levé puis frappé ; fier ensuite (poses.rs, ship)
        Activite::Ship if tick < 10 => (Envoi, [0, 1, 2, 3, 4, 4, 5, 5, 6, 6][tick.max(0) as usize]),
        // ses pattes suivent le manche, poussé puis ramené (poses.rs, sweep)
        Activite::Sweep => (Balai, [0, 1, 2, 3, 4, 3, 2, 1][tick.rem_euclid(8) as usize]),
        Activite::Thinking => (Relecture, [1, 1, 2, 2, 3, 3][(tick.div_euclid(3) % 6) as usize]),
        Activite::Review => boucle(Relecture, 3),
        Activite::Waiting => boucle(Attente, 3),
        Activite::Failed => (Echec, [0, 1, 2, 2, 7, 6][(tick.div_euclid(3) % 6) as usize]),
        Activite::Jumping => boucle(Saut, 2),
        Activite::Waving | Activite::Ship => boucle(Salut, 2),
        // il somnole sur son tabouret, la tête tombe douze tics sur vingt-quatre (poses.rs, sleep) ;
        // un message le fait bondir
        Activite::Sleep => (Sommeil, (tick.rem_euclid(24) < 12) as usize),
        Activite::Waking => (Saut, tick.clamp(0, 4) as usize),
        Activite::Idle | Activite::Away => boucle(Repos, 3),
    }
}

/// L'image du moment pour Galadriel : son pet seul, sans outil ; ce qu'elle fait se voit dans
/// le belvédère (belvedere.rs). Elle glisse plus qu'elle ne marche : une image tous les deux pixels.
pub fn moment_galadriel(a: Activite, tick: i64, w: &Monde) -> (Rangee, usize) {
    use Rangee::*;
    let boucle = |r: Rangee, tics: i64, n: usize| (r, (tick.div_euclid(tics) as usize) % n);
    if w.walking {
        let r = if w.facing < 0 { Gauche } else { Droite };
        return (r, (w.x * w.facing.signum()).rem_euclid(8) as usize);
    }
    match a {
        // au travail : ses mains sur le flot du bassin (les deux dernières cases filent, gardées pour la marche)
        Activite::Running => boucle(Travail, 3, 6),
        Activite::Thinking => boucle(Relecture, 3, 8),
        Activite::Review | Activite::Quench => boucle(Relecture, 2, 8),
        Activite::Waiting => boucle(Attente, 3, 8),
        Activite::Failed => boucle(Echec, 3, 8),
        Activite::Jumping | Activite::Waking => boucle(Saut, 2, 8),
        Activite::Waving | Activite::Ship => boucle(Salut, 2, 8),
        Activite::Sweep => boucle(Saut, 3, 8),
        // assoupie : les yeux clos de sa rangée d'attente
        Activite::Sleep => (Attente, 3),
        Activite::Idle | Activite::Away => boucle(Repos, 3, 8),
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
fn relief(perso: Perso, t: f32, haut: bool) -> [f32; 3] {
    let m = 0.16 - 0.32 * t.clamp(0.0, 1.0) + if haut { 0.08 } else { 0.0 };
    if perso == Perso::Galadriel {
        // le bassin à sa gauche : un côté cyan ; la lune en haut, l'autre côté bleu nuit
        return if m > 0.0 { [1.0 + m * 0.45, 1.0 + m * 1.1, 1.0 + m * 1.25] } else { [1.0 + m * 1.1, 1.0 + m, 1.0 + m * 0.6] };
    }
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

/// Les cases, à la hauteur de cellule `ch`, à la cape `cape` (sa lettre claire, 'O' : la verte
/// du pet), avec l'accessoire `acc` : faites une fois pour toutes les bandes de cette taille et
/// de cette tenue, la planche décodée seulement le temps de les faire.
pub fn cases(perso: Perso, ch: u32, cape: u8, acc: Accessoire) -> Option<Arc<Cases>> {
    static FAITES: OnceLock<Mutex<HashMap<(u8, u32, u8, u8), Arc<Cases>>>> = OnceLock::new();
    // Galadriel n'a ni cape de session ni accessoire : une seule tenue
    let (cape, acc) = if perso == Perso::Galadriel { (b'O', Accessoire::Aucun) } else { (cape, acc) };
    let cle = (perso as u8, ch, cape, acc as u8);
    let mut faites = FAITES.get_or_init(Default::default).lock().ok()?;
    if let Some(c) = faites.get(&cle) {
        return Some(c.clone());
    }
    let echelle = HAUTEUR * ch as f32 / 2.0 / HAUTEUR_CASE;
    if perso == Perso::Galadriel {
        let (w, mut rgba) = decoder(GALADRIEL)?;
        reparer(w, &mut rgba, Fond::Vert);
        let mut toutes = HashMap::new();
        for rangee in RANGEES {
            let Some((_, r, n)) = rangee.ou_chez(perso) else { continue };
            for i in 0..n.min(COLONNES) {
                toutes.insert((rangee, i), case(&rgba, w, i * CASE.0, r * CASE.1, echelle, perso));
            }
        }
        let c = Arc::new(Cases(toutes));
        faites.insert(cle, c.clone());
        return Some(c);
    }
    let mut planches = PLANCHES.iter().map(|p| decoder(p)).collect::<Option<Vec<_>>>()?;
    for (w, rgba) in &mut planches {
        reparer(*w, rgba, Fond::Magenta);
    }
    // l'accessoire avant la cape : le bandeau, de son vert, en prend la teinte
    if let Some((_, calque)) = CALQUES.iter().find(|(a, _)| *a == acc) {
        if let Some(calque) = decoder(calque) {
            habiller(&mut planches, &calque);
        }
    }
    for (_, rgba) in &mut planches {
        teindre(rgba, cape);
    }
    let mut toutes = HashMap::new();
    for rangee in RANGEES {
        let (p, r, n) = rangee.ou();
        let (w, rgba) = &planches[p];
        for i in 0..n.min(COLONNES) {
            toutes.insert((rangee, i), case(rgba, *w, i * CASE.0, r * CASE.1, echelle, perso));
        }
    }
    let c = Arc::new(Cases(toutes));
    faites.insert(cle, c.clone());
    Some(c)
}

/// Le calque d'un accessoire posé sur les planches, par-dessus (en prémultiplié) ; un calque
/// d'une autre taille que (8 cases, 9 + 6 rangées) n'est pas encore dessiné.
fn habiller(planches: &mut [(usize, Vec<[f32; 4]>)], (w, calque): &(usize, Vec<[f32; 4]>)) {
    let largeur = COLONNES * CASE.0;
    if *w != largeur || calque.len() != largeur * CASE.1 * (RANGEES_PET + 6) {
        return;
    }
    for (k, (pw, rgba)) in planches.iter_mut().enumerate() {
        if *pw != largeur {
            continue;
        }
        // ses rangées dans le calque : les neuf de la première planche, puis la seconde
        let (debut, n) = if k == 0 { (0, RANGEES_PET) } else { (RANGEES_PET, 6) };
        let n = (n * CASE.1).min(rgba.len() / largeur);
        for y in 0..n {
            for x in 0..largeur {
                let c = calque[(debut * CASE.1 + y) * largeur + x];
                if c[3] <= 0.0 {
                    continue;
                }
                let d = &mut rgba[y * largeur + x];
                let inv = 1.0 - c[3] / 255.0;
                for i in 0..4 {
                    d[i] = c[i] + d[i] * inv;
                }
            }
        }
    }
}

/// La couleur claire de la cape forêt (accessoires::PALETTE, 'O') : celle du pet, à peu près.
const VERT: u32 = 0x62802f;

/// Teinte, saturation, valeur (de 0 à 1) d'une couleur de 0 à 1.
fn tsv([r, g, b]: [f32; 3]) -> [f32; 3] {
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let d = mx - mn;
    let h = if d <= 0.0 {
        0.0
    } else if mx == r {
        ((g - b) / d).rem_euclid(6.0) / 6.0
    } else if mx == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    [h, if mx > 0.0 { d / mx } else { 0.0 }, mx]
}

fn rvb([h, s, v]: [f32; 3]) -> [f32; 3] {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
    match (i as i32).rem_euclid(6) {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

/// La tenue de la session sur le pet : ce qui a la teinte de sa cape verte (la capuche, la
/// tunique, la gourde) prend celle de la cape `cape`, chaque nuance à sa place. Les valeurs
/// suivent de loin (racine du rapport) : les capes en lettres sont plus vives que la sienne.
fn teindre(rgba: &mut [[f32; 4]], cape: u8) {
    let hexa = |c: u32| [(c >> 16 & 255) as f32 / 255.0, (c >> 8 & 255) as f32 / 255.0, (c & 255) as f32 / 255.0];
    let Some(cible) = super::accessoires::PALETTE.get(cape as usize).copied().flatten().filter(|_| cape != b'O') else {
        return;
    };
    let (r, t) = (tsv(hexa(VERT)), tsv(hexa(cible)));
    let (ks, kv) = (t[1] / r[1], (t[2] / r[2]).sqrt());
    for p in rgba {
        if p[3] < 1.0 {
            continue;
        }
        let a = p[3] / 255.0;
        let [h, s, v] = tsv([p[0] / a / 255.0, p[1] / a / 255.0, p[2] / a / 255.0]);
        if !(50.0 / 360.0..=110.0 / 360.0).contains(&h) || s <= 0.25 || v <= 0.12 {
            continue;
        }
        let n = rvb([(t[0] + h - r[0]).rem_euclid(1.0), (s * ks).min(1.0), (v * kv).min(1.0)]);
        for c in 0..3 {
            p[c] = n[c] * 255.0 * a;
        }
    }
}

/// Une planche en RGBA (alpha prémultiplié, en flottants) et sa largeur.
fn decoder(planche: &[u8]) -> Option<(usize, Vec<[f32; 4]>)> {
    let mut d = image_webp::WebPDecoder::new(std::io::Cursor::new(planche)).ok()?;
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

/// Le fond sur lequel un pet a été détouré.
#[derive(Clone, Copy, PartialEq)]
enum Fond {
    Magenta,
    Vert,
}

/// La somme des couleurs de pixels pleins, et leur moyenne opaque.
fn moyenne(rgba: &[[f32; 4]], pixels: &[usize]) -> [f32; 4] {
    let n = pixels.len().max(1) as f32;
    let s = pixels.iter().fold([0.0f32; 3], |s, &v| [s[0] + rgba[v][0], s[1] + rgba[v][1], s[2] + rgba[v][2]]);
    [s[0] / n, s[1] / n, s[2] / n, 255.0]
}

/// Ce que le détourage a laissé sur une planche. Le fond mêlé aux couleurs en est ôté : le
/// magenta (plus de rouge et de bleu que de vert, ce que ni la fourrure, ni la cape, ni la crête
/// n'ont), ou le vert (plus de vert que de rouge et de bleu : ni la robe ni les runes cyan de
/// Galadriel). Le bord devient net, plein ou vide : ni voile ni liseré à demi transparent. Les
/// points perdus autour s'en vont, les piqûres du bord se comblent, les éclats sombres d'un bord
/// clair prennent la couleur de leurs voisins, et un bord clair que le détourage a mis à nu
/// reprend un trait sombre de sa teinte. Ce qu'il a pris pour du fond au-dedans (l'intérieur des
/// oreilles, des traînées dans la fourrure), un trou que le fond du dehors n'atteint pas, est
/// rempli des couleurs autour, du bord vers le centre.
fn reparer(w: usize, rgba: &mut [[f32; 4]], fond: Fond) {
    let h = rgba.len() / w;
    let vide = |p: &[f32; 4]| p[3] <= 40.0;
    let voisins = |i: usize| {
        let (x, y) = (i % w, i / w);
        [(-1, 0), (1, 0), (0, -1), (0, 1)].into_iter().filter_map(move |(dx, dy): (i32, i32)| {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            ((0..w as i32).contains(&nx) && (0..h as i32).contains(&ny)).then(|| ny as usize * w + nx as usize)
        })
    };
    let huit = |i: usize| {
        let (x, y) = (i % w, i / w);
        [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)].into_iter().filter_map(move |(dx, dy): (i32, i32)| {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            ((0..w as i32).contains(&nx) && (0..h as i32).contains(&ny)).then(|| ny as usize * w + nx as usize)
        })
    };
    for p in rgba.iter_mut() {
        match fond {
            Fond::Magenta => {
                let m = p[0].min(p[2]) - p[1];
                if m > 0.0 {
                    p[0] -= m;
                    p[2] -= m;
                }
            }
            Fond::Vert => {
                let m = p[1] - p[0].max(p[2]);
                if m > 0.0 {
                    p[1] -= m;
                }
            }
        }
        // plein ou vide
        if p[3] < 128.0 {
            *p = [0.0; 4];
        } else {
            let k = 255.0 / p[3];
            *p = [p[0] * k, p[1] * k, p[2] * k, 255.0];
        }
    }
    let lum = |p: &[f32; 4]| (p[0] + p[1] + p[2]) / 3.0;
    // les points perdus : un pixel plein qui n'a qu'un voisin plein au plus
    for _ in 0..2 {
        let seuls: Vec<usize> = (0..rgba.len()).filter(|&i| !vide(&rgba[i]) && huit(i).filter(|&v| !vide(&rgba[v])).count() < 2).collect();
        for i in seuls {
            rgba[i] = [0.0; 4];
        }
    }
    // les piqûres du bord : un vide pris sur trois côtés au moins, comblé de ses voisins
    for _ in 0..2 {
        let comble: Vec<(usize, [f32; 4])> = (0..rgba.len())
            .filter(|&i| vide(&rgba[i]))
            .filter_map(|i| {
                let pleins: Vec<usize> = voisins(i).filter(|&v| !vide(&rgba[v])).collect();
                (pleins.len() >= 3).then(|| (i, moyenne(rgba, &pleins)))
            })
            .collect();
        for (i, p) in comble {
            rgba[i] = p;
        }
    }
    // les éclats sombres restés sur un bord clair : la couleur des voisins clairs
    let bord = |rgba: &[[f32; 4]], i: usize| !vide(&rgba[i]) && voisins(i).any(|v| vide(&rgba[v]));
    let eclats: Vec<(usize, [f32; 4])> = (0..rgba.len())
        .filter(|&i| bord(rgba, i) && lum(&rgba[i]) < 70.0)
        .filter_map(|i| {
            let pleins: Vec<usize> = huit(i).filter(|&v| !vide(&rgba[v])).collect();
            let clairs: Vec<usize> = pleins.iter().copied().filter(|&v| lum(&rgba[v]) >= 70.0).collect();
            (!pleins.is_empty() && lum(&moyenne(rgba, &pleins)) > 150.0 && !clairs.is_empty()).then(|| (i, moyenne(rgba, &clairs)))
        })
        .collect();
    for (i, p) in eclats {
        rgba[i] = p;
    }
    // le contour : autour d'un bord clair, un trait sombre de sa teinte
    let traits: Vec<(usize, [f32; 4])> = (0..rgba.len())
        .filter(|&i| vide(&rgba[i]))
        .filter_map(|i| {
            let clairs: Vec<usize> = voisins(i).filter(|&v| bord(rgba, v) && lum(&rgba[v]) > 110.0).collect();
            (!clairs.is_empty()).then(|| {
                let m = moyenne(rgba, &clairs);
                (i, [m[0] * 0.28 + 14.0 * 0.72, m[1] * 0.28 + 14.0 * 0.72, m[2] * 0.28 + 30.0 * 0.72, 255.0])
            })
        })
        .collect();
    for (i, p) in traits {
        rgba[i] = p;
    }
    // le fond du dehors : le vide qui touche le bord de la planche
    let mut dehors = vec![false; rgba.len()];
    let mut pile: Vec<usize> = (0..rgba.len()).filter(|&i| (i % w == 0 || i % w == w - 1 || i < w || i >= w * (h - 1)) && vide(&rgba[i])).collect();
    pile.iter().for_each(|&i| dehors[i] = true);
    while let Some(i) = pile.pop() {
        for n in voisins(i) {
            if !dehors[n] && vide(&rgba[n]) {
                dehors[n] = true;
                pile.push(n);
            }
        }
    }
    let mut trous: Vec<usize> = (0..rgba.len()).filter(|&i| !dehors[i] && vide(&rgba[i])).collect();
    let mut plein: Vec<bool> = rgba.iter().map(|p| !vide(p)).collect();
    while !trous.is_empty() {
        let mut faits = Vec::new();
        for &i in &trous {
            let (mut somme, mut n) = ([0.0f32; 3], 0.0);
            for v in voisins(i).filter(|&v| plein[v]) {
                let p = rgba[v];
                for c in 0..3 {
                    somme[c] += p[c] / p[3] * 255.0;
                }
                n += 1.0;
            }
            if n > 0.0 {
                faits.push((i, [somme[0] / n, somme[1] / n, somme[2] / n, 255.0]));
            }
        }
        if faits.is_empty() {
            break;
        }
        for &(i, p) in &faits {
            rgba[i] = p;
            plein[i] = true;
        }
        trous.retain(|&i| !plein[i]);
    }
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

fn case(rgba: &[[f32; 4]], largeur: usize, x0: usize, y0: usize, echelle: f32, perso: Perso) -> Case {
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
            // net : plein passé la moitié, rien en deçà ; ni voile ni liseré sur le décor
            if al < 128.0 {
                continue;
            }
            let c = |v: f32| (v / al * 255.0).clamp(0.0, 255.0).round() as u32;
            out[o * sw + x] = 0xFF00_0000 | (c(s[0]) << 16) | (c(s[1]) << 8) | c(s[2]);
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
            let k = relief(perso, if b > a { (i as f32 - a as f32) / (b - a) as f32 } else { 0.5 }, haut);
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
    /// la lettre claire de sa cape (Monde.cape[1]), son accessoire
    pub cape: u8,
    pub accessoire: Accessoire,
    pub perso: Perso,
}

/// Peint `a` dans `pixels` (0xAARRGGBB prémultiplié, `w` × `h`) ; `x0` : la première colonne
/// de forge montrée ; (cw, ch) : la cellule. Rend le rectangle peint.
pub fn peindre(a: &Acteur, pixels: &mut [u32], w: usize, h: usize, x0: i32, cw: i32, ch: i32) -> Option<(i32, i32, i32, i32)> {
    let cases = cases(a.perso, ch as u32, a.cape, a.accessoire)?;
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
        let c = cases(Perso::Scorpheus, 19, b'O', Accessoire::Aucun).expect("la planche se décode");
        let repos = c.case(Rangee::Repos, 0).unwrap();
        // 26 pixels de forge de 9,5 px : ~247 px de haut, posés au sol
        assert!((235..=260).contains(&repos.h), "hauteur {}", repos.h);
        assert!(repos.dy + repos.h as i32 <= 2 && repos.dy + repos.h as i32 >= -4, "pieds à {}", repos.dy + repos.h as i32);
        for (r, n) in [(Rangee::Travail, 6), (Rangee::Droite, 8), (Rangee::Echec, 8), (Rangee::Frappe, 4), (Rangee::Envoi, 7)] {
            for i in 0..n {
                assert!(c.case(r, i).is_some_and(|k| k.w > 0), "{r:?} {i}");
            }
        }
    }

    /// Le fond magenta ne se voit plus : ni trou au-dedans du personnage, ni liseré rose autour.
    #[test]
    fn le_fond_ne_reste_ni_dedans_ni_autour() {
        for planche in PLANCHES {
            let (w, mut rgba) = decoder(planche).unwrap();
            reparer(w, &mut rgba, Fond::Magenta);
            let rose = rgba.iter().filter(|p| p[3] > 0.0 && p[0].min(p[2]) - p[1] > 40.0 * p[3] / 255.0).count();
            assert_eq!(rose, 0, "pixels rosés");
            // le vide que le fond du dehors n'atteint pas
            let h = rgba.len() / w;
            let mut dehors = vec![false; rgba.len()];
            let mut pile: Vec<usize> = (0..w).chain((0..h).map(|y| y * w)).collect();
            while let Some(i) = pile.pop() {
                if dehors[i] || rgba[i][3] > 40.0 {
                    continue;
                }
                dehors[i] = true;
                let (x, y) = (i % w, i / w);
                if x > 0 { pile.push(i - 1) }
                if x + 1 < w { pile.push(i + 1) }
                if y > 0 { pile.push(i - w) }
                if y + 1 < h { pile.push(i + w) }
            }
            let trous = (0..rgba.len()).filter(|&i| !dehors[i] && rgba[i][3] <= 40.0).count();
            assert_eq!(trous, 0, "pixels de trou");
        }
    }

    /// La planche réparée, en RGBA brut non prémultiplié, pour la regarder : PLANCHE_SORTIE=<fichier>.
    #[test]
    #[ignore]
    fn ecrire_la_planche_reparee() {
        let sortie = std::env::var("PLANCHE_SORTIE").unwrap();
        for (k, planche) in PLANCHES.iter().enumerate() {
            let (w, mut rgba) = decoder(planche).unwrap();
            reparer(w, &mut rgba, Fond::Magenta);
            let brut: Vec<u8> = rgba
                .iter()
                .flat_map(|p| {
                    let a = p[3].max(1e-3) / 255.0;
                    [(p[0] / a) as u8, (p[1] / a) as u8, (p[2] / a) as u8, p[3] as u8]
                })
                .collect();
            std::fs::write(format!("{sortie}.{k}.{w}.rgba"), brut).unwrap();
        }
    }

    #[test]
    fn un_accessoire_dessine_se_pose_sur_le_pet() {
        let nu = cases(Perso::Scorpheus, 19, b'O', Accessoire::Aucun).unwrap();
        for (acc, calque) in CALQUES {
            let dessine = decoder(calque).is_some_and(|(w, px)| w == COLONNES * CASE.0 && px.iter().any(|p| p[3] > 0.0));
            let habille = cases(Perso::Scorpheus, 19, b'O', acc).unwrap();
            let pareil = |r: Rangee, i: usize| nu.case(r, i).map(|c| &c.px) == habille.case(r, i).map(|c| &c.px);
            let change = RANGEES.iter().flat_map(|&r| (0..r.ou().2).map(move |i| (r, i))).filter(|&(r, i)| !pareil(r, i)).count();
            assert_eq!(change > 0, dessine, "{acc:?} : {change} cases changées");
        }
    }
}
