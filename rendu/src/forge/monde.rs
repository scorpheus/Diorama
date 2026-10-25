//! Le monde : ce qui se passe, en faits simples (hooks/world.ts). Le moteur le tient à jour,
//! la forge le change en pixels.

use serde::Deserialize;

macro_rules! sorte {
    ($(#[$m:meta])* $nom:ident { $($v:ident = $s:literal),* $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
        pub enum $nom { $(#[serde(rename = $s)] $v),* }

        #[allow(dead_code)]
        impl $nom {
            /// Le nom que lui donne le plugin.
            pub fn de(s: &str) -> Option<Self> {
                match s { $($s => Some(Self::$v),)* _ => None }
            }
        }
    };
}

sorte!(
    /// Ce que fait le personnage.
    Activite {
        Idle = "idle", Thinking = "thinking", Running = "running", Review = "review", Waiting = "waiting",
        Failed = "failed", Jumping = "jumping", Waving = "waving", Quench = "quench", Ship = "ship",
        Sweep = "sweep", Sleep = "sleep", Waking = "waking", Away = "away",
    }
);
sorte!(
    /// La sorte de travail : elle colore les étincelles.
    Saveur { Forge = "forge", Build = "build", Test = "test", Shell = "shell", Git = "git", Web = "web", Agent = "agent", Mcp = "mcp" }
);
sorte!(Issue { Pass = "pass", Fail = "fail" });
sorte!(Envoi { Commit = "commit", Push = "push" });
sorte!(Temps { Storm = "storm", Rainbow = "rainbow" });
sorte!(Marteau { Small = "small", Normal = "normal", Sledge = "sledge" });
sorte!(Arme { Sword = "sword", Axe = "axe", Dagger = "dagger", Mace = "mace" });
sorte!(Accessoire { Aucun = "aucun", Lunettes = "lunettes", Tablier = "tablier", Bandeau = "bandeau" });
sorte!(Routine { Coffee = "coffee", Lunch = "lunch", Lantern = "lantern" });
sorte!(Humeur { Calme = "calme", Jovial = "jovial", Grognon = "grognon", Reveur = "reveur", Applique = "applique" });
sorte!(ModeChat { Roof = "roof", Walk = "walk", Flee = "flee" });
sorte!(Cadeau { Coin = "coin", Cookie = "cookie", Ore = "ore" });
sorte!(Nouvelle { Done = "done", Waiting = "waiting" });
sorte!(Rarete { Star = "star", Dragon = "dragon" });
sorte!(
    /// Le spectacle de la Route du Rhum au lancement d'une session : le départ rejoué, ou une arrivée.
    FeteRhum { Depart = "start", Arrivee = "finish" }
);
sorte!(Recadrage { Full = "full", Compact = "compact", Solo = "solo" });
sorte!(
    /// La Route du Rhum, dans la session qui la montre : la flotte avant le départ, ou en course.
    PhaseRhum { Avant = "before", Course = "race" }
);
sorte!(
    Reve {
        Voiture = "voiture", Fusee = "fusee", Bug = "bug", Circuit = "circuit", Livre = "livre", Montagne = "montagne",
        Poisson = "poisson", Etoile = "etoile", Gateau = "gateau", Dragon = "dragon", Drapeau = "drapeau",
        Carburant = "carburant", Cle = "cle", Ecran = "ecran", Musique = "musique", Base = "base", Reseau = "reseau",
        Manette = "manette", Graphique = "graphique", Photo = "photo", Carte = "carte", Maison = "maison",
        Horloge = "horloge", Cadenas = "cadenas", Nuage = "nuage", Ampoule = "ampoule", Engrenage = "engrenage",
        Robot = "robot", Plante = "plante", Cafe = "cafe",
    }
);

/// Une lettre de la palette.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Lettre(pub u8);

impl<'de> Deserialize<'de> for Lettre {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(Lettre(s.bytes().next().unwrap_or(b'.')))
    }
}

/// Un aide pour un sous-agent : `seed` fixe son pas et son chemin, `colour` sa couleur,
/// `age` les tics depuis son arrivée, `leaving` ceux depuis la fin de son travail.
#[derive(Clone, Debug, Deserialize)]
pub struct Petit {
    pub seed: i64,
    pub colour: Lettre,
    pub age: i64,
    pub leaving: Option<i64>,
}

/// Un personnage en visite : où il se tient (comme World.x), son allure, son propre temps.
#[derive(Clone, Debug, Deserialize)]
pub struct Invite {
    pub x: i32,
    pub walking: bool,
    pub facing: i32,
    pub cape: [Lettre; 2],
    pub accessory: Accessoire,
    pub tick: i64,
    pub waving: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Banniere {
    pub cloth: Lettre,
    pub emblem: Lettre,
    pub bits: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Chat {
    pub mode: ModeChat,
    pub x: i32,
    pub age: i64,
    pub dir: Option<i32>,
    pub step: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Don {
    pub kind: Cadeau,
    pub age: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Messager {
    pub kind: Nouvelle,
    pub age: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Rare {
    pub kind: Rarete,
    pub age: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpectacleRhum {
    pub kind: FeteRhum,
    pub age: i64,
}

/// Ce que fait le personnage du travail, quoi qu'on dessine.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monde {
    pub flavor: Saveur,
    /// La jauge la plus pleine, en % : l'inquiétude se voit passé 85.
    pub pressure: f64,
    /// Le contexte rempli, en %.
    pub ctx: f64,
    pub fuel5h: Option<f64>,
    pub fuel7d: Option<f64>,
    pub apprentices: Vec<Petit>,
    pub hourglasses: i32,
    /// Tics depuis le début du tour principal ; 0 hors d'un tour.
    pub turn_ticks: i64,
    /// Tics depuis le début de la session.
    pub clock: i64,
    pub bell_age: Option<i64>,
    pub outcome: Option<Issue>,
    pub ship: Option<Envoi>,
    /// L'heure locale, 0..24, minutes comprises.
    pub hour: f64,
    pub month: i32,
    pub day: i32,
    pub weather: Option<Temps>,
    pub hammer: Marteau,
    pub effort: i32,
    pub usd: Option<f64>,
    pub x: i32,
    pub walking: bool,
    pub facing: i32,
    pub rack: Vec<Arme>,
    pub forging: bool,
    pub banner: Option<Banniere>,
    pub speaking: bool,
    pub duck: bool,
    pub exhausted: bool,
    pub tea: bool,
    pub cape: [Lettre; 2],
    pub accessory: Accessoire,
    pub absent: bool,
    pub guests: Vec<Invite>,
    pub banquet: bool,
    pub cat: Option<Chat>,
    pub gift: Option<Don>,
    pub messenger: Option<Messager>,
    pub routine: Option<Routine>,
    pub lantern_lit: bool,
    pub dream: Option<Reve>,
    pub personality: Humeur,
    pub masterpiece: bool,
    pub rare: Option<Rare>,
    /// La session nommée pour la Route du Rhum : la mer dans la cour.
    pub rhum: Option<PhaseRhum>,
    /// Où en est la course, de 0 au départ à 1 aux premières arrivées : le décor voyage.
    pub rhum_progress: Option<f64>,
    /// La flotte au dernier pointage, pour la carte : (classe, latitude, longitude, rang dans sa
    /// classe) ; les classes : 0 Ultim, 1 Ocean Fifty, 2 IMOCA, 3 Class40, 4 Rhum Multi, 5 Rhum Mono.
    pub rhum_flotte: Option<Vec<[f32; 4]>>,
    /// Les couleurs du premier de chaque classe, pour les bateaux de la baie : (classe, coque, bande).
    pub rhum_tetes: Option<Vec<[u32; 3]>>,
    /// Le spectacle en cours, et depuis combien de tics.
    pub rhum_fete: Option<SpectacleRhum>,
    /// La trace du premier de chaque classe, pour la carte : (classe, latitude, longitude,
    /// latitude, longitude…), des plus anciens points au dernier pointage.
    pub rhum_traces: Option<Vec<Vec<f32>>>,
}

impl Default for Monde {
    /// CALM : rien ne se passe.
    fn default() -> Self {
        Monde {
            flavor: Saveur::Forge,
            pressure: 0.0,
            ctx: 0.0,
            fuel5h: None,
            fuel7d: None,
            apprentices: Vec::new(),
            hourglasses: 0,
            turn_ticks: 0,
            clock: 0,
            bell_age: None,
            outcome: None,
            ship: None,
            hour: 12.0,
            month: 6,
            day: 15,
            weather: None,
            hammer: Marteau::Normal,
            effort: 0,
            usd: None,
            x: 0,
            walking: false,
            facing: 0,
            rack: Vec::new(),
            forging: false,
            banner: None,
            speaking: false,
            duck: false,
            exhausted: false,
            tea: false,
            cape: [Lettre(b'o'), Lettre(b'O')],
            accessory: Accessoire::Aucun,
            absent: false,
            guests: Vec::new(),
            banquet: false,
            cat: None,
            gift: None,
            messenger: None,
            routine: None,
            lantern_lit: false,
            dream: None,
            personality: Humeur::Calme,
            masterpiece: false,
            rare: None,
            rhum: None,
            rhum_progress: None,
            rhum_flotte: None,
            rhum_tetes: None,
            rhum_fete: None,
            rhum_traces: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Saison {
    Hiver,
    Printemps,
    Ete,
    Automne,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Fete {
    Anniversaire,
    Halloween,
    Noel,
    NouvelAn,
}

impl Monde {
    pub fn saison(&self) -> Saison {
        match self.month {
            12 | ..=2 => Saison::Hiver,
            ..=5 => Saison::Printemps,
            ..=8 => Saison::Ete,
            _ => Saison::Automne,
        }
    }

    pub fn fete(&self) -> Option<Fete> {
        let (m, d) = (self.month, self.day);
        if m == 10 && d == 2 {
            Some(Fete::Anniversaire) // le jour où la forge fut allumée, 2026
        } else if m == 10 && d >= 25 {
            Some(Fete::Halloween)
        } else if m == 12 && (15..=26).contains(&d) {
            Some(Fete::Noel)
        } else if (m == 12 && d == 31) || (m == 1 && d == 1) {
            Some(Fete::NouvelAn)
        } else {
            None
        }
    }

    pub fn nuit(&self) -> bool {
        self.hour < 6.0 || self.hour >= 21.0
    }
}
