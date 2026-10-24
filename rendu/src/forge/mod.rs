//! La forge de Scorpheus, dessinée par l'appli : le monde changé en cellules (des demi-blocs
//! et des glyphes plus fins, à la couleur près), comme hooks/themes/forge.ts les dessinait
//! pour le terminal.

pub mod accessoires;
pub mod atelier;
pub mod baie;
pub mod cellules;
pub mod cour;
pub mod gerbe;
pub mod lumiere;
pub mod monde;
pub mod pet;
pub mod pixels;
pub mod planche;
pub mod poses;
pub mod trace;


use atelier::{compose, Contexte, C, GROW, IMPACT, ROWS, SH, SW, YARD};
use monde::{Accessoire, Activite, Monde, Recadrage};

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
            pet::Acteur { rangee, image, x: (x + 8) as f32 + pet::decalage(a, w), y: SH as f32, eclairage: None, cape: w.cape[1].0, accessoire: w.accessory }
        };
        let mut acteurs: Vec<pet::Acteur> = Vec::new();
        if pet {
            if !world.absent {
                acteurs.push(acteur(activity, tick, world, C.0 + world.x));
            }
            acteurs.extend(self.ctx.invites.iter().map(|v| acteur(v.activite, v.tick, &v.monde, v.x)));
        }
        if !lit {
            return (pixels::to_cells(&scene, &accessoires::PALETTE, x0, fin, None), acteurs, coup);
        }
        let actor = if pet { pixels::Grille::new(SW, SH) } else { compose(&mut self.ctx, &mut self.cour, &p, world, tick, true, SW) };
        let dehors = self.cour.outdoors(world, activity == Activite::Running, YARD, width);
        let mut lumiere = lumiere::LumiereForge::new(&mut self.chutes, &p, world, tick, &actor, &dehors);
        for a in &mut acteurs {
            a.eclairage = Some(lumiere.eclairage(a.x as i32 - 15, 0, 30, SH));
        }
        (pixels::to_cells(&scene, &accessoires::PALETTE, x0, fin, Some(&mut lumiere)), acteurs, coup)
    }
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

/// La forge d'un bandeau : le dernier état reçu, avancé d'un tic par image, et les cellules
/// déjà peintes (seules celles qui changent sont repeintes, et celles sous Scorpheus).
pub struct SceneForge {
    forge: Forge,
    etat: Option<Etat>,
    /// tics passés depuis le dernier état reçu
    depuis: i64,
    avant: Vec<u32>,
    gerbe: gerbe::Gerbe,
    baie: baie::Baie,
    /// le tic du dernier coup de marteau : un message du plugin qui le redonne ne frappe pas deux fois
    frappe: i64,
    pub images: u64,
}

impl SceneForge {
    pub fn new() -> Self {
        SceneForge { forge: Forge::default(), etat: None, depuis: 0, avant: Vec::new(), gerbe: gerbe::Gerbe::default(), baie: baie::Baie::default(), frappe: i64::MIN, images: 0 }
    }

    pub fn recevoir(&mut self, etat: Etat) {
        self.etat = Some(etat);
        self.depuis = 0;
    }
}

impl crate::scene::Scene for SceneForge {
    fn peindre(&mut self, image: &mut crate::scene::Image, dt: f32, neuve: bool) -> Option<windows::Win32::Foundation::RECT> {
        let etat = self.etat.as_ref()?;
        let e = self.depuis;
        self.depuis += 1;
        let monde = etat.monde.plus_tard(e);
        let (cw, ch) = (image.cellule.0 as i32, image.cellule.1 as i32);
        let tick = etat.tick + e;
        // les cases du pet, faites une fois par hauteur de cellule et par cape
        let pet = pet::cases(image.cellule.1, b'O', Accessoire::Aucun).is_some();
        let (cells, acteurs, coup) = self.forge.image(etat.recadrage, etat.activite, tick, &monde, etat.lit, etat.extra, pet);
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
            let debut = (YARD - x0).max(0) as usize;
            for rangee in cells.chunks_mut(cols * 3) {
                for cell in rangee.chunks_mut(3).skip(debut) {
                    cell[0] = u32::MAX;
                }
            }
        }
        let mut peints: Vec<_> = acteurs.iter().filter_map(|a| pet::peindre(a, image.pixels, w, h, x0, cw, ch)).collect();
        peints.extend(self.gerbe.peindre(image.pixels, w, h));
        for (l, t, r, b) in peints {
            zone = Some(zone.map_or((l, t, r, b), |z| (z.0.min(l), z.1.min(t), z.2.max(r), z.3.max(b))));
            // sous lui et sous la gerbe, les cellules ne sont plus ce que `cells` en dit : à
            // l'image suivante elles se repeignent, qu'il soit resté ou parti
            let rows = (cells.len() / 3 / cols.max(1)) as i32;
            for cy in (t / ch).max(0)..((b + ch - 1) / ch).min(rows) {
                for cx in (l / cw).max(0)..((r + cw - 1) / cw).min(cols as i32) {
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
