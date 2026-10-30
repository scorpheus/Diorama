//! La forge de Scorpheus, dessinée par l'appli : le monde changé en cellules (des demi-blocs
//! et des glyphes plus fins, à la couleur près), comme hooks/themes/forge.ts les dessinait
//! pour le terminal.

pub mod accessoires;
pub mod atelier;
pub mod baie;
pub mod bateaux;
pub mod belvedere;
pub mod cellules;
pub mod cour;
pub mod gerbe;
pub mod jauges;
pub mod lumiere;
pub mod monde;
pub mod pet;
pub mod pixels;
pub mod planche;
pub mod poses;
pub mod trace;


use atelier::{compose, Contexte, C, GROW, IMPACT, ROWS, SH, SW, YARD};
use monde::{Accessoire, Activite, Monde, Perso, Recadrage};

/// D'elle au plus proche de ses invités, en pixels de forge : sa demi-largeur, robes comprises,
/// et celle d'un loup.
const A_COTE: f32 = 21.0;

/// Les fenêtres sur la scène, de la plus large : le bandeau montre la plus large qui tient.
pub const fn crop(c: Recadrage) -> (i32, i32) {
    match c {
        Recadrage::Full => (0, SW),
        Recadrage::Compact => (14, SW), // l'enclume, Scorpheus et la cour
        Recadrage::Solo => (29, YARD),  // Scorpheus seul
    }
}

/// Ce que la forge garde d'une image à l'autre : la cour immobile, la chute des lampes.
#[derive(Default)]
pub struct Forge {
    ctx: Contexte,
    cour: cour::Cour,
    chutes: lumiere::Chutes,
}

impl Forge {
    /// Les cellules d'une image : trois mots par cellule (caractère, encre, fond), rangée après
    /// rangée. `extra` : les colonnes que la fenêtre a en plus, pour un recadrage qui finit
    /// par la cour : elle s'élargit d'autant (GROW au plus).
    #[cfg(test)]
    pub fn frame(&mut self, c: Recadrage, activity: Activite, tick: i64, world: &Monde, lit: bool, extra: i32) -> Vec<u32> {
        self.image(c, activity, tick, world, lit, extra, false).0
    }

    /// Comme `frame` ; avec `pet`, la scène sans Scorpheus en lettres ni étincelles en lettres,
    /// et ce qui se peint par-dessus : lui et ses invités tels que pet.rs les peint (la case du
    /// moment, la place, la lumière autour), et le coup de marteau qui tombe à ce tic (combien
    /// d'étincelles).
    #[allow(clippy::too_many_arguments)]
    pub fn image(&mut self, c: Recadrage, activity: Activite, tick: i64, world: &Monde, lit: bool, extra: i32, pet: bool) -> (Vec<u32>, Vec<pet::Acteur>, Option<usize>) {
        self.ctx.pet = pet;
        let (x0, x1) = crop(c);
        let width = if x1 == SW { SW + extra.clamp(0, GROW) } else { SW };
        let p = poses::pose_of(activity, tick, world);
        let scene = compose(&mut self.ctx, &mut self.cour, &p, world, tick, false, width);
        let fin = if x1 == SW { width } else { x1 };
        let coup = if pet && p.sparks_age == Some(0) { p.spark_count.map(|n| n as usize) } else { None };
        // au milieu de la place du personnage en lettres (16 de large), les pattes au sol ; ses
        // invités ensuite, par-dessus lui, comme la scène les dessinait
        let acteur = |a: Activite, t: i64, w: &Monde, x: i32| {
            let (rangee, image) = pet::moment(a, t, w);
            pet::Acteur { rangee, image, x: (x + 8) as f32 + pet::decalage(a, w), y: SH as f32, eclairage: None, cape: w.cape[1].0, accessoire: w.accessory, perso: Perso::Scorpheus }
        };
        let gala = world.galadriel();
        let mut acteurs: Vec<pet::Acteur> = Vec::new();
        if pet {
            // la session qui montre la Route du Rhum : lui en ciré jaune, ses invités dans leurs capes
            let cire = world.rhum.is_some();
            if !world.absent && gala {
                // elle flotte : un pixel d'écran de haut en bas, lentement
                let (rangee, image) = pet::moment_galadriel(activity, tick, world);
                let houle = 0.5 + 0.5 * (world.clock as f32 * 0.35).sin();
                acteurs.push(pet::Acteur { rangee, image, x: (C.0 + world.x + 8) as f32, y: SH as f32 - 0.7 - 0.45 * houle, eclairage: None, cape: b'O', accessoire: Accessoire::Aucun, perso: Perso::Galadriel });
            } else if !world.absent {
                let mut lui = acteur(activity, tick, world, C.0 + world.x);
                if cire {
                    lui.cape = CIRE;
                }
                acteurs.push(lui);
            }
            let mut invites: Vec<_> = self.ctx.invites.iter().map(|v| acteur(v.activite, v.tick, &v.monde, v.x)).collect();
            // ses robes sont larges : un invité dont la place tomberait sur elle se tient à côté,
            // et les autres d'autant, pour garder leur rang
            if let Some(elle) = acteurs.first().filter(|a| a.perso == Perso::Galadriel).map(|a| a.x) {
                let d = invites.iter().map(|a| elle + A_COTE - a.x).fold(0.0, f32::max);
                for a in &mut invites {
                    a.x += d;
                }
            }
            acteurs.extend(invites);
        }
        // chez Galadriel, le belvédère se peint au vrai pixel sur les colonnes de la forge : ses
        // cellules restent vides, ni dessinées ni éclairées
        let debut = if gala && pet { x0.max(belvedere::FIN).min(fin) } else { x0 };
        if !lit {
            return (vides_avant(pixels::to_cells(&scene, &accessoires::PALETTE, debut, fin, None), debut - x0, fin - debut), acteurs, coup);
        }
        let actor = if pet { pixels::Grille::new(SW, SH) } else { compose(&mut self.ctx, &mut self.cour, &p, world, tick, true, SW) };
        let dehors = self.cour.outdoors(world, activity == Activite::Running, YARD, width);
        let mut lumiere = lumiere::LumiereForge::new(&mut self.chutes, &p, world, tick, &actor, &dehors);
        let (ciel, porte) = cour::lumiere_dehors(world);
        for a in &mut acteurs {
            let mut e = lumiere.eclairage(a.x as i32 - 15, 0, 30, SH);
            dehors_au_ciel(&mut e, ciel, porte, a.perso);
            a.eclairage = Some(e);
        }
        (vides_avant(pixels::to_cells(&scene, &accessoires::PALETTE, debut, fin, Some(&mut lumiere)), debut - x0, fin - debut), acteurs, coup)
    }
}

/// Dehors, la lumière du ciel : passé la porte, celle de la pièce (bornée à son dernier pixel)
/// s'efface en quatre pixels de forge, et seule en reste la part qui passe la porte. Galadriel a
/// sa lueur à elle : la nuit ne l'éteint pas.
fn dehors_au_ciel(e: &mut pet::Eclairage, ciel: [f32; 3], porte: f32, perso: Perso) {
    let lueur = if perso == Perso::Galadriel { [0.66, 0.7, 0.86] } else { [0.0; 3] };
    for (i, k) in e.k.iter_mut().enumerate() {
        let x = e.x0 + (i as i32 % e.w);
        if x < YARD {
            continue;
        }
        let t = ((x - YARD + 1) as f32 / 4.0).min(1.0);
        let t = t * t * (3.0 - 2.0 * t);
        for c in 0..3 {
            let dehors = ciel[c] + (k[c] - ciel[c]).max(0.0) * porte * 0.5;
            k[c] = (k[c] + (dehors - k[c]) * t).max(lueur[c] * t);
        }
    }
}

/// Des cellules vides (transparentes) devant chaque rangée : `n` colonnes, puis les `large` données.
fn vides_avant(cells: Vec<u32>, n: i32, large: i32) -> Vec<u32> {
    if n <= 0 {
        return cells;
    }
    let (n, large) = (n as usize, large.max(0) as usize);
    let rows = if large == 0 { ROWS as usize } else { cells.len() / 3 / large };
    let mut out = Vec::with_capacity((n + large) * rows * 3);
    for r in 0..rows {
        for _ in 0..n {
            out.extend_from_slice(&[0x20, pixels::DEFAUT, pixels::DEFAUT]);
        }
        out.extend_from_slice(&cells[r * large * 3..(r + 1) * large * 3]);
    }
    out
}

/// Où se peint la baie de la Route du Rhum dans le bandeau : la cour, de la porte au bord ;
/// rien quand le monde ne la montre pas ou que le recadrage n'a pas de cour.
pub fn cadre_baie(monde: &Monde, x0: i32, cols: usize, cw: i32, ch: i32, h: usize) -> Option<baie::Cadre> {
    let debut = YARD - x0;
    if monde.rhum.is_none() || debut < 0 || debut >= cols as i32 {
        return None;
    }
    Some(baie::Cadre {
        left: debut * cw,
        w: ((cols as i32 - debut) * cw) as usize,
        h: ((ROWS * ch) as usize).min(h),
        cw: cw as f32,
        demi: ch as f32 / 2.0,
        from: YARD as f32,
        to: (x0 + cols as i32) as f32,
    })
}

impl Monde {
    /// Le monde `e` tics plus tard, tel que l'appli le voit passer entre deux messages du
    /// plugin : l'horloge et tous les âges avancent (le pendant de `later` dans register.tsx ;
    /// un écart entre les deux ne coûte qu'un message de plus).
    pub fn plus_tard(&self, e: i64) -> Monde {
        let mut w = self.clone();
        if e == 0 {
            return w;
        }
        w.clock += e;
        if w.turn_ticks > 0 {
            w.turn_ticks += e;
        }
        w.bell_age = w.bell_age.map(|a| a + e);
        for c in &mut w.apprentices {
            match c.leaving.as_mut() {
                Some(l) => *l += e,
                None => c.age += e,
            }
        }
        for g in &mut w.guests {
            g.tick += e;
        }
        if let Some(c) = w.cat.as_mut() {
            c.age += e;
        }
        if let Some(g) = w.gift.as_mut() {
            g.age += e;
        }
        if let Some(m) = w.messenger.as_mut() {
            m.age += e;
        }
        if let Some(r) = w.rare.as_mut() {
            r.age += e;
        }
        if let Some(f) = w.rhum_fete.as_mut() {
            f.age += e;
        }
        w
    }
}

/// Ce que le plugin dit du bandeau d'une session : le recadrage, la lumière, le moment.
pub struct Etat {
    pub recadrage: Recadrage,
    pub extra: i32,
    pub lit: bool,
    pub activite: Activite,
    pub tick: i64,
    pub monde: Monde,
}

impl Etat {
    /// « <recadrage> <extra> <lumière 0|1> <moment> <tic> <monde en JSON> ».
    pub fn lire(texte: &str) -> Result<Etat, String> {
        let mut m = texte.splitn(6, ' ');
        let mut mot = || m.next().ok_or_else(|| "ligne courte".to_string());
        let recadrage = Recadrage::de(mot()?).ok_or("recadrage inconnu")?;
        let extra = mot()?.parse::<i32>().map_err(|e| e.to_string())?;
        let lit = mot()? == "1";
        let activite = Activite::de(mot()?).ok_or("moment inconnu")?;
        let tick = mot()?.parse::<i64>().map_err(|e| e.to_string())?;
        let monde = serde_json::from_str::<Monde>(mot()?).map_err(|e| e.to_string())?;
        Ok(Etat { recadrage, extra, lit, activite, tick, monde })
    }
}

/// La lettre du ciré jaune (accessoires::PALETTE) : sa tenue pendant la Route du Rhum.
const CIRE: u8 = b'7';

/// Un tic du monde, en secondes : celui du plugin (register.tsx : TICK_MS).
const TIC: f32 = 0.15;
/// Au-delà de cet écart (en tics) entre l'horloge montrée et celle du plugin, on la recale.
const ECART_MAX: f64 = 100.0;

/// La forge d'un bandeau : le dernier état reçu, avancé d'un tic toutes les 150 ms, et les cellules
/// déjà peintes (seules celles qui changent sont repeintes, et celles sous Scorpheus).
pub struct SceneForge {
    forge: Forge,
    etat: Option<Etat>,
    /// les secondes passées depuis le dernier état reçu : le monde avance d'un tic toutes les
    /// TIC secondes, au temps réel et non à l'image peinte. Une appli qui peint un peu moins vite
    /// que le plugin ne compte pas (WM_TIMER à 150 ms en donne moins de 6,67 par seconde) : sinon
    /// chaque état reçu la ramenait de quelques tics en avant, et la baie sautait.
    temps: f32,
    /// l'horloge montrée, en tics : elle suit celle du plugin sans jamais sauter. Le plugin compte
    /// ses tics au minuteur de Claude, qui prend du retard ; l'appli compte au temps réel. Recalée
    /// d'un coup à chaque état reçu, la baie sautait de quelques tics, en avant ou en arrière :
    /// elle rattrape l'écart en douceur, et ne recule jamais.
    lisse: Option<f64>,
    /// la dernière horloge montrée, et quand l'appli a dit un saut au journal
    montree: Option<i64>,
    pub saute: Option<(i64, i64)>,
    avant: Vec<u32>,
    gerbe: gerbe::Gerbe,
    baie: baie::Baie,
    belvedere: belvedere::Belvedere,
    /// le tic du dernier coup de marteau : un message du plugin qui le redonne ne frappe pas deux fois
    frappe: i64,
    pub images: u64,
}

impl SceneForge {
    pub fn new() -> Self {
        SceneForge { forge: Forge::default(), etat: None, temps: 0.0, lisse: None, montree: None, saute: None, avant: Vec::new(), gerbe: gerbe::Gerbe::default(), baie: baie::Baie::default(), belvedere: belvedere::Belvedere::default(), frappe: i64::MIN, images: 0 }
    }

    pub fn recevoir(&mut self, etat: Etat) {
        // un écart de plus de 15 s (une session reprise, un plugin rechargé) : on la recale
        if self.lisse.is_some_and(|l| (etat.monde.clock as f64 - l).abs() > ECART_MAX) {
            self.lisse = None;
            self.montree = None;
        }
        self.etat = Some(etat);
        self.temps = 0.0;
    }

    /// L'horloge montrée à cette image : celle du plugin avancée au temps réel, rejointe en
    /// douceur (l'écart comblé en 1,5 s, de 4 fois moins vite à 3 fois plus vite que le temps).
    fn horloge_lisse(&mut self, cible: f64, dt: f32) -> i64 {
        let pas = (dt / TIC) as f64;
        let l = match self.lisse {
            None => cible,
            Some(l) => l + (pas + (cible - l) * (dt as f64 / 1.5).min(1.0)).clamp(pas * 0.25, pas * 3.0),
        };
        self.lisse = Some(l);
        (l + 1e-3).floor() as i64
    }

    /// Les tics passés depuis le dernier état reçu (le millième rattrape la somme des dt en f32).
    fn depuis(&self) -> i64 {
        (self.temps / TIC + 1e-3).floor() as i64
    }

    /// L'horloge du monde à la prochaine image, telle que l'appli l'a avancée depuis le dernier
    /// état ; rien pour un bandeau qu'elle n'a pas peint depuis (caché, il n'avance pas).
    #[cfg(test)]
    pub fn horloge(&self) -> Option<i64> {
        self.etat.as_ref().filter(|_| self.temps > 0.0).map(|e| e.monde.clock + self.depuis())
    }
}

impl crate::scene::Scene for SceneForge {
    fn peindre(&mut self, image: &mut crate::scene::Image, dt: f32, neuve: bool) -> Option<windows::Win32::Foundation::RECT> {
        let etat = self.etat.as_ref()?;
        let e = self.depuis();
        let cible = etat.monde.clock as f64 + (self.temps / TIC) as f64;
        self.temps += dt;
        let mut monde = etat.monde.plus_tard(e);
        let horloge = self.horloge_lisse(cible, dt);
        if let Some(avant) = self.montree.filter(|&a| horloge < a || horloge > a + 10) {
            self.saute = Some((avant, horloge));
        }
        self.montree = Some(horloge);
        monde.clock = horloge;
        let etat = self.etat.as_ref()?;
        let (cw, ch) = (image.cellule.0 as i32, image.cellule.1 as i32);
        let tick = etat.tick + e;
        let gala = monde.galadriel();
        // les cases du pet, faites une fois par hauteur de cellule et par cape
        let pet = pet::cases(if gala { Perso::Galadriel } else { Perso::Scorpheus }, image.cellule.1, b'O', Accessoire::Aucun).is_some();
        // chez Galadriel, le chat, la messagère et les aides se peignent dans son belvédère : la
        // scène en lettres ne les dessine pas
        let cellulaire = if gala && pet {
            let mut m = monde.clone();
            m.cat = None;
            m.messenger = None;
            m.apprentices.clear();
            m
        } else {
            monde.clone()
        };
        let (cells, mut acteurs, coup) = self.forge.image(etat.recadrage, etat.activite, tick, &cellulaire, etat.lit, etat.extra, pet);
        let gala = gala && pet;
        let coup = coup.filter(|_| !gala);
        let (x0, x1) = crop(etat.recadrage);
        // le coup tombe sur l'enclume : la gerbe part de là, et retombe au sol de la forge
        let hb = (ROWS * ch) as f32;
        if let Some(n) = coup.filter(|_| tick != self.frappe) {
            self.frappe = tick;
            self.gerbe.frapper(((IMPACT.0 + 1 - x0) * cw) as f32 + cw as f32 / 2.0, IMPACT.1 as f32 * ch as f32 / 2.0, hb, n * 4);
        }
        self.gerbe.avancer(dt, hb, (SH - 1) as f32 * ch as f32 / 2.0);
        let cols = if x1 == SW { (SW + etat.extra.clamp(0, GROW) - x0) as usize } else { (x1 - x0) as usize };
        // un autre recadrage, une autre largeur : tout à repeindre
        let neuve = neuve || self.avant.len() != cells.len();
        let (w, h) = (image.w as usize, image.h as usize);
        let mut cible = cellules::Cible { pixels: image.pixels, w, h };
        let mut zone = cellules::peindre(&mut cible, &cells, if neuve { None } else { Some(&self.avant) }, cols, 0, 0, cw, ch);
        let mut cells = cells;
        // la Route du Rhum : la baie, au vrai pixel, sous les cellules que la cour laisse
        // transparentes ; elle bouge à chaque image, ces cellules se repeignent donc toujours
        if let Some(cadre) = cadre_baie(&monde, x0, cols, cw, ch, h) {
            if let Some(z) = self.baie.peindre(image.pixels, w, h, cadre, &monde) {
                zone = Some(zone.map_or(z, |a| (a.0.min(z.0), a.1.min(z.1), a.2.max(z.2), a.3.max(z.3))));
            }
            let debut = ((if gala { belvedere::FIN } else { YARD }) - x0).max(0) as usize;
            for rangee in cells.chunks_mut(cols * 3) {
                for cell in rangee.chunks_mut(3).skip(debut) {
                    cell[0] = u32::MAX;
                }
            }
        }
        // le belvédère de Galadriel, sur les cellules vides de la forge
        let fin_belvedere = (belvedere::FIN - x0).clamp(0, cols as i32);
        if gala {
            let elle = acteurs.first().filter(|a| a.perso == Perso::Galadriel).map(|a| a.x);
            let vue = belvedere::Vue { monde: &monde, activite: etat.activite, tick, x0, cols: cols as i32, cw, ch, elle };
            let (z, dehors) = self.belvedere.peindre(image.pixels, w, h, &vue, neuve);
            if let Some(z) = z {
                zone = Some(zone.map_or(z, |a| (a.0.min(z.0), a.1.min(z.1), a.2.max(z.2), a.3.max(z.3))));
            }
            // ce qui a débordé sur la cour : ses cellules se repeignent à l'image suivante
            if let Some((l, t, r, b)) = dehors {
                zone = Some(zone.map_or((l, t, r, b), |a| (a.0.min(l), a.1.min(t), a.2.max(r), a.3.max(b))));
                let rows = (cells.len() / 3 / cols.max(1)) as i32;
                for cy in (t / ch).max(0)..((b + ch - 1) / ch).min(rows) {
                    for cx in (l / cw).max(fin_belvedere)..((r + cw - 1) / cw).min(cols as i32) {
                        cells[(cy as usize * cols + cx as usize) * 3] = u32::MAX;
                    }
                }
            }
            // la lumière du belvédère sur qui s'y tient
            for a in &mut acteurs {
                if (a.x as i32) < belvedere::FIN {
                    if let Some(e) = self.belvedere.eclairage(a.x as i32 - 15, 30) {
                        a.eclairage = Some(e);
                    }
                }
            }
        }
        // les jauges au sol, sous lui quand il passe dessus
        let mut peints: Vec<_> = jauges::peindre(image.pixels, w, h, cw, ch, &monde).into_iter().collect();
        peints.extend(acteurs.iter().filter_map(|a| pet::peindre(a, image.pixels, w, h, x0, cw, ch)));
        peints.extend(self.gerbe.peindre(image.pixels, w, h));
        // chez Galadriel, ce qui passe devant elle
        if gala {
            let elle = acteurs.first().filter(|a| a.perso == Perso::Galadriel).map(|a| a.x);
            let vue = belvedere::Vue { monde: &monde, activite: etat.activite, tick, x0, cols: cols as i32, cw, ch, elle };
            let (z, dehors) = self.belvedere.devant(image.pixels, &vue);
            if let Some(z) = z {
                zone = Some(zone.map_or(z, |a| (a.0.min(z.0), a.1.min(z.1), a.2.max(z.2), a.3.max(z.3))));
            }
            peints.extend(dehors);
        }
        for (l, t, r, b) in peints {
            zone = Some(zone.map_or((l, t, r, b), |z| (z.0.min(l), z.1.min(t), z.2.max(r), z.3.max(b))));
            // sous lui et sous la gerbe, les cellules ne sont plus ce que `cells` en dit : à
            // l'image suivante elles se repeignent, qu'il soit resté ou parti
            let rows = (cells.len() / 3 / cols.max(1)) as i32;
            // chez Galadriel, sous elle le belvédère se reprend de son fond ; ses cellules vides
            // n'ont pas à se repeindre
            let premiere = if gala {
                if l < fin_belvedere * cw {
                    self.belvedere.salir((l, t, r, b));
                }
                fin_belvedere
            } else {
                0
            };
            for cy in (t / ch).max(0)..((b + ch - 1) / ch).min(rows) {
                for cx in (l / cw).max(premiere)..((r + cw - 1) / cw).min(cols as i32) {
                    cells[(cy as usize * cols + cx as usize) * 3] = u32::MAX;
                }
            }
        }
        self.avant = cells;
        self.images += 1;
        zone.map(|(left, top, right, bottom)| windows::Win32::Foundation::RECT { left, top, right, bottom })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une appli qui peint moins d'images que de tics ne prend pas de retard sur le plugin : le
    /// monde avance au temps réel (deux tics pour une image de 0,3 s), pas à l'image.
    #[test]
    fn le_monde_avance_au_temps_reel_et_non_a_l_image() {
        use crate::scene::{Image, Scene};
        let mut s = SceneForge::new();
        s.recevoir(Etat { recadrage: Recadrage::Full, extra: 0, lit: false, activite: Activite::Idle, tick: 0, monde: Monde { clock: 100, ..Monde::default() } });
        let (w, h) = (SW as usize * 9, ROWS as usize * 19);
        let mut px = vec![0u32; w * h];
        for k in 0..3 {
            s.peindre(&mut Image { pixels: &mut px, w: w as u32, h: h as u32, cellule: (9, 19) }, 0.3, k == 0);
        }
        assert_eq!(s.horloge(), Some(106));
    }

    /// Un plugin en retard ou en avance sur le temps réel : l'horloge montrée ne recule jamais et
    /// n'avance jamais de plus de 3 tics d'une image à l'autre, puis rejoint la sienne.
    #[test]
    fn l_horloge_montree_rejoint_celle_du_plugin_sans_sauter() {
        use crate::scene::{Image, Scene};
        let mut s = SceneForge::new();
        let etat = |clock| Etat { recadrage: Recadrage::Full, extra: 0, lit: false, activite: Activite::Idle, tick: 0, monde: Monde { clock, ..Monde::default() } };
        let (w, h) = (SW as usize * 9, ROWS as usize * 19);
        let mut px = vec![0u32; w * h];
        let mut montree = Vec::new();
        s.recevoir(etat(1000));
        for k in 0..40 {
            // le plugin, ses tics en retard : 1 tic tous les 0,2 s, redit toutes les 4 images
            if k > 0 && k % 4 == 0 {
                s.recevoir(etat(1000 + k * 3 / 4));
            }
            s.peindre(&mut Image { pixels: &mut px, w: w as u32, h: h as u32, cellule: (9, 19) }, 0.15, k == 0);
            montree.push(s.montree.unwrap());
        }
        s.recevoir(etat(1060)); // puis en avance
        for _ in 0..40 {
            s.peindre(&mut Image { pixels: &mut px, w: w as u32, h: h as u32, cellule: (9, 19) }, 0.15, false);
            montree.push(s.montree.unwrap());
        }
        for p in montree.windows(2) {
            assert!(p[1] >= p[0] && p[1] <= p[0] + 3, "saut de {} à {}", p[0], p[1]);
        }
        assert!((montree.last().unwrap() - (1060 + 40)).abs() <= 2, "{:?}", montree.last());
        assert_eq!(s.saute, None);
    }

    /// Une ligne telle que le plugin l'envoie (tests/diorama.test.tsx, rendu natif).
    const LIGNE: &str = r#"full 0 1 idle 993 {"hour":14,"month":10,"day":2,"hammer":"sledge","effort":0,"x":-12,"walking":false,"facing":1,"apprentices":[],"cape":["Z","v"],"accessory":"aucun","absent":false,"guests":[],"banquet":false,"cat":{"mode":"roof","x":14,"age":84},"lanternLit":false,"personality":"reveur","masterpiece":false,"rack":[],"forging":false,"banner":{"cloth":"E","emblem":"s","bits":259},"speaking":false,"duck":false,"exhausted":false,"tea":false,"flavor":"forge","pressure":0,"ctx":0,"hourglasses":0,"turnTicks":0,"clock":1012}"#;

    #[test]
    fn le_monde_du_plugin_se_lit_et_se_dessine() {
        let etat = Etat::lire(LIGNE).expect("la ligne du plugin se lit");
        assert_eq!((etat.recadrage, etat.extra, etat.lit, etat.activite, etat.tick), (Recadrage::Full, 0, true, Activite::Idle, 993));
        assert_eq!((etat.monde.x, etat.monde.clock, etat.monde.cape[0].0), (-12, 1012, b'Z'));
        let cells = Forge::default().frame(etat.recadrage, etat.activite, etat.tick, &etat.monde, etat.lit, etat.extra);
        assert_eq!(cells.len(), (SW * atelier::ROWS * 3) as usize);
        // le chat dort sur l'appui, et le temps passe sans message : son âge avance avec l'horloge
        let plus = etat.monde.plus_tard(10);
        assert_eq!((plus.clock, plus.cat.map(|c| c.age)), (1022, Some(94)));
    }

    /// Peinte par morceaux image après image, la forge est celle qu'on peindrait en entier : rien
    /// de lui ni de ses invités ne reste où ils étaient, quand il marche ou qu'ils passent.
    #[test]
    fn rien_ne_reste_ou_il_etait() {
        use crate::scene::{Image, Scene};
        let (cw, ch) = (9u32, 19u32);
        for recadrage in [Recadrage::Full, Recadrage::Compact, Recadrage::Solo] {
            for extra in [0, 12] {
                let mut base = Etat::lire(LIGNE).unwrap();
                base.recadrage = recadrage;
                base.extra = extra;
                let (x0, x1) = crop(recadrage);
                let cols = if x1 == SW { SW + extra.clamp(0, GROW) - x0 } else { x1 - x0 };
                let (w, h) = (cols as u32 * cw, atelier::ROWS as u32 * ch);
                let mut morceaux = SceneForge::new();
                let mut garde = vec![0u32; (w * h) as usize];
                for t in 0..48i64 {
                    let mut etat = Etat::lire(LIGNE).unwrap();
                    etat.recadrage = recadrage;
                    etat.extra = extra;
                    etat.tick = base.tick + t;
                    etat.monde.clock = base.monde.clock + t;
                    // il marche d'un bout à l'autre et revient ; un invité arrive de la coulisse
                    let aller = t < 24;
                    etat.monde.walking = true;
                    etat.monde.facing = if aller { 1 } else { -1 };
                    etat.monde.x = if aller { -12 + t as i32 } else { 12 - (t as i32 - 24) };
                    etat.monde.guests = vec![monde::Invite {
                        x: atelier::GUEST_SPOT + 40 - t as i32,
                        walking: true,
                        facing: -1,
                        cape: [monde::Lettre(b'N'), monde::Lettre(b'L')],
                        accessory: Accessoire::Aucun,
                        tick: t,
                        waving: false,
                    }];
                    let mut entiere = SceneForge::new();
                    entiere.recevoir(clone_etat(&etat));
                    morceaux.recevoir(clone_etat(&etat));
                    let mut neuve = vec![0u32; (w * h) as usize];
                    entiere.peindre(&mut Image { pixels: &mut neuve, w, h, cellule: (cw, ch) }, 0.15, true);
                    morceaux.peindre(&mut Image { pixels: &mut garde, w, h, cellule: (cw, ch) }, 0.15, t == 0);
                    let ecarts = garde.iter().zip(&neuve).filter(|(a, b)| a != b).count();
                    assert_eq!(ecarts, 0, "{recadrage:?} +{extra}, tic {t} : {ecarts} pixels restés");
                }
            }
        }
    }

    fn clone_etat(e: &Etat) -> Etat {
        Etat { recadrage: e.recadrage, extra: e.extra, lit: e.lit, activite: e.activite, tick: e.tick, monde: e.monde.clone() }
    }

    #[test]
    fn la_route_du_rhum_se_lit_du_monde_et_se_garde_entre_deux_messages() {
        let ligne = LIGNE.replace(r#""clock":1012}"#, r#""clock":1012,"rhum":"race"}"#);
        let etat = Etat::lire(&ligne).expect("la ligne du plugin se lit");
        assert_eq!(etat.monde.rhum, Some(monde::PhaseRhum::Course));
        assert_eq!(etat.monde.plus_tard(10).rhum, Some(monde::PhaseRhum::Course));
        assert_eq!(Etat::lire(LIGNE).expect("sans elle aussi").monde.rhum, None);
        // et le pointage : la flotte pour la carte, les couleurs des premiers
        let ligne = LIGNE.replace(r#""clock":1012}"#, r#""clock":1012,"rhum":"race","rhumProgress":0.5,"rhumFlotte":[[0,30,-34,1],[2,40.5,-15.2,1]],"rhumTetes":[[0,1916810,15909198]]}"#);
        let m = Etat::lire(&ligne).expect("le pointage se lit").monde;
        assert_eq!(m.rhum_progress, Some(0.5));
        assert_eq!(m.rhum_flotte, Some(vec![[0.0, 30.0, -34.0, 1.0], [2.0, 40.5, -15.2, 1.0]]));
        assert_eq!(m.rhum_tetes, Some(vec![[0, 0x1d3f8a, 0xf2c14e]]));
    }

    #[test]
    fn une_ligne_abimee_ne_casse_rien() {
        assert!(Etat::lire("full 0 1 idle").is_err());
        assert!(Etat::lire("large 0 1 idle 3 {}").is_err());
        assert!(Etat::lire(r#"full 0 1 idle 3 {"hour":1}"#).is_err());
    }
}
