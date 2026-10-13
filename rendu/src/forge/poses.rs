//! Les moments, une pose par tic de 150 ms (t compte depuis le début du moment) : la fin de
//! hooks/themes/forge.ts.

use super::monde::{Activite, Envoi, Humeur, Issue, Marteau, Monde, Routine, Saveur};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Yeux {
    Open,
    Look,
    Blink,
    Happy,
    Sad,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bouche {
    None,
    Smile,
    Open,
    Frown,
    Grit,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gauche {
    Rest,
    Raise,
    Strike,
    Scroll,
    Drop,
    Pull,
    Blade,
    Dip,
    Stow,
    Lid,
    StampUp,
    StampDown,
    Broom,
    Lap,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Droite {
    Rest,
    Wave1,
    Wave2,
    Up,
    Scroll,
    Scratch,
    Wipe,
    Drink,
    Broom,
    Lap,
    Chin,
    Tea,
    Bread,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lingot {
    Cold,
    Hot,
    Flash,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lame {
    Hot,
    Proven,
    Cracked,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Caisse {
    Open,
    Closing,
    Shut,
    Sealed,
    Gone,
}

/// Une pose : ce que le dessin montre du personnage et de la forge à un tic.
#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub eyes: Yeux,
    pub mouth: Bouche,
    pub worried: bool,
    /// les sourcils froncés : à l'enclume, au soufflet, à la lame
    pub focused: bool,
    pub ear: bool,
    pub ear_r: bool,
    /// la tête tournée : −1 vers la forge .. +1 vers les mots
    pub look: i32,
    pub head_dy: i32,
    /// tout le personnage, pixels vers le bas (négatif : en l'air)
    pub dy: i32,
    pub seated: bool,
    pub tap: bool,
    pub left: Gauche,
    pub right: Droite,
    pub tail: usize,
    /// 0 braises .. 4 il rugit
    pub fire: i32,
    pub ingot: Lingot,
    pub smoke_every: i64,
    pub sparks_age: Option<i64>,
    pub spark_count: Option<i64>,
    pub embers: bool,
    pub bellows: bool,
    pub thought: bool,
    pub glints: bool,
    pub bubble: Option<i32>,
    pub scroll: bool,
    pub puff: Option<i64>,
    pub stars: bool,
    pub fireworks: Option<i64>,
    pub sweat: bool,
    pub bucket: bool,
    pub steam: Option<i64>,
    pub blade: Option<Lame>,
    pub crate_: Option<Caisse>,
    pub stow: Option<i32>,
    pub seal_age: Option<i64>,
    pub lid_puff: bool,
    pub raven: Option<i64>,
    pub sweep: Option<i32>,
    pub broom_up: bool,
    pub dust: Option<i64>,
    pub zzz: bool,
    pub stool: bool,
}

impl Pose {
    /// Ce que toute pose donne : le reste à son défaut.
    fn base(eyes: Yeux, mouth: Bouche, left: Gauche, right: Droite, tail: usize, fire: i32, ingot: Lingot, smoke_every: i64) -> Pose {
        Pose {
            eyes,
            mouth,
            worried: false,
            focused: false,
            ear: false,
            ear_r: false,
            look: 0,
            head_dy: 0,
            dy: 0,
            seated: false,
            tap: false,
            left,
            right,
            tail,
            fire,
            ingot,
            smoke_every,
            sparks_age: None,
            spark_count: None,
            embers: false,
            bellows: false,
            thought: false,
            glints: false,
            bubble: None,
            scroll: false,
            puff: None,
            stars: false,
            fireworks: None,
            sweat: false,
            bucket: false,
            steam: None,
            blade: None,
            crate_: None,
            stow: None,
            seal_age: None,
            lid_puff: false,
            raven: None,
            sweep: None,
            broom_up: false,
            dust: None,
            zzz: false,
            stool: false,
        }
    }
}

fn at<T: Copy>(list: &[T], i: i64) -> T {
    list[i.rem_euclid(list.len() as i64) as usize]
}

/// Les quatre temps de la queue sur deux `every` ; trop vite pour quatre, elle bat à gauche, à droite.
fn sway(t: i64, every: i64) -> usize {
    (if every >= 2 { (2 * t).div_euclid(every) % 4 } else { t % 2 }) as usize
}

// Le repos tourne sur 24 s : il regarde le feu, jette un œil aux mots, se gratte une oreille
// et s'étire, respirant et remuant la queue tout du long. Avec des tâches de fond, il s'assoit
// sur son tabouret et tape du pied, un œil sur le sablier.
fn idle(t: i64, w: &Monde) -> Pose {
    let c = t % 160;
    let within = |a: i64, b: i64| c >= a && c < b;
    let blink = c == 14 || c == 24 || c == 40 || c == 42 || c == 104;
    let mut base = Pose::base(
        if blink { Yeux::Blink } else { Yeux::Open },
        Bouche::None,
        Gauche::Rest,
        Droite::Rest,
        sway(t, if within(90, 100) { 2 } else { 6 }),
        1,
        Lingot::Cold,
        5,
    );
    base.look = if within(16, 32) || within(88, 102) {
        -1
    } else if within(44, 54) {
        1
    } else {
        0
    };
    base.ear = within(52, 54) || within(110, 112);
    base.ear_r = within(30, 32) || c == 100 || within(52, 54);
    base.head_dy = if c % 16 < 8 { 0 } else { 1 };
    if w.hourglasses > 0 {
        return Pose {
            seated: true,
            stool: true,
            tap: t % 4 < 2,
            look: if c % 40 < 20 { -1 } else { 0 },
            left: Gauche::Lap,
            right: Droite::Lap,
            head_dy: 0,
            ..base
        };
    }
    if w.exhausted {
        // plus de charbon : il s'assoit près de la forge froide, un œil sur l'ardoise
        return Pose {
            seated: true,
            stool: true,
            left: Gauche::Lap,
            right: Droite::Lap,
            look: -1,
            eyes: if blink { Yeux::Blink } else { Yeux::Open },
            worried: true,
            fire: 0,
            smoke_every: 0,
            head_dy: 0,
            ..base
        };
    }
    if w.duck {
        // trois échecs de suite : il explique le problème au canard
        return Pose {
            look: -1,
            mouth: if c % 6 < 3 { Bouche::Open } else { Bouche::None },
            eyes: if blink { Yeux::Blink } else { Yeux::Open },
            worried: true,
            ..base
        };
    }
    if w.tea && c % 160 >= 20 && c % 160 < 120 {
        return Pose {
            seated: true,
            stool: true,
            left: Gauche::Lap,
            right: Droite::Tea,
            eyes: if c % 30 < 4 { Yeux::Happy } else { base.eyes },
            head_dy: 0,
            ..base
        };
    }
    match w.routine {
        Some(Routine::Lantern) => {
            // le soir : il se hisse pour allumer la lanterne
            return Pose { left: Gauche::Raise, right: Droite::Up, look: -1, eyes: Yeux::Open, head_dy: 0, ..base };
        }
        Some(Routine::Lunch) => {
            return Pose {
                seated: true,
                stool: true,
                left: Gauche::Lap,
                right: Droite::Bread,
                mouth: if c % 8 < 4 { Bouche::Open } else { Bouche::None },
                head_dy: 0,
                ..base
            };
        }
        Some(Routine::Coffee) => {
            return Pose { right: Droite::Tea, eyes: if c % 30 < 4 { Yeux::Happy } else { base.eyes }, ..base };
        }
        None => {}
    }
    if w.hour < 5.0 && c % 48 >= 40 {
        // les petites heures : il bâille de plus en plus souvent
        return Pose { left: Gauche::Raise, right: Droite::Up, eyes: Yeux::Blink, mouth: Bouche::Open, dy: -1, head_dy: 0, ..base };
    }
    if within(64, 76) {
        base.right = Droite::Scratch;
        base.eyes = Yeux::Happy;
        base.ear = c % 2 == 0;
    }
    if within(120, 128) {
        base.left = Gauche::Raise;
        base.right = Droite::Up;
        base.eyes = Yeux::Blink;
        base.mouth = Bouche::Open;
        base.dy = -1;
        base.head_dy = 0;
    }
    // son humeur se voit quand rien d'autre ne se voit
    if w.personality == Humeur::Jovial && base.mouth == Bouche::None && base.eyes == Yeux::Open {
        base.mouth = Bouche::Smile;
    }
    if w.personality == Humeur::Grognon && base.eyes == Yeux::Open {
        base.focused = true;
    }
    if w.personality == Humeur::Reveur && base.look == 0 && c % 40 < 30 {
        base.look = -1; // vers la fenêtre
    }
    if w.personality == Humeur::Applique && base.look == 0 && c % 40 < 20 {
        base.look = -1;
        base.focused = true;
    }
    if w.pressure >= 85.0 {
        base.worried = true;
        base.sweat = true;
    }
    if w.pressure >= 95.0 {
        base.mouth = Bouche::Frown;
    }
    base
}

// Au travail : le marteau, ou pour une compilation le soufflet. Un long tour le fatigue : passé
// 5 minutes il s'essuie le front de temps en temps, passé 15 il boit à sa gourde.
fn running(t: i64, w: &Monde) -> Pose {
    let c = t % 8;
    let strike = (3..=5).contains(&c);
    let pose = if w.flavor == Saveur::Build {
        let squeeze = t % 6 < 3;
        let heat = (2 + w.effort + t.div_euclid(40) as i32).min(4); // le four monte à mesure que la compilation avance
        Pose {
            focused: true,
            look: -1,
            head_dy: if squeeze { 1 } else { 0 },
            bellows: squeeze,
            embers: true,
            ..Pose::base(
                if t % 30 == 0 { Yeux::Blink } else { Yeux::Open },
                if squeeze { Bouche::Grit } else { Bouche::None },
                Gauche::Pull,
                Droite::Rest,
                sway(t, 3),
                if squeeze { heat } else { heat - 1 },
                Lingot::Hot,
                1,
            )
        }
    } else {
        Pose {
            focused: true,
            look: -1,
            head_dy: if strike { 1 } else { 0 },
            sparks_age: if c >= 3 { Some(c - 3) } else { None },
            spark_count: Some(match w.hammer {
                Marteau::Sledge => 16,
                Marteau::Small => 8,
                Marteau::Normal => 12,
            }),
            ..Pose::base(
                if c == 7 { Yeux::Blink } else { Yeux::Open },
                if c == 3 { Bouche::Grit } else { Bouche::None },
                if strike { Gauche::Strike } else { Gauche::Raise },
                Droite::Rest,
                sway(t, 4),
                (2 + w.effort).min(4),
                if c == 3 { Lingot::Flash } else { Lingot::Hot },
                2,
            )
        }
    };
    let long = w.turn_ticks;
    if long > 6000 && long % 64 < 10 {
        return Pose {
            right: Droite::Drink,
            left: if pose.left == Gauche::Pull { Gauche::Pull } else { Gauche::Rest },
            eyes: Yeux::Blink,
            look: 0,
            sparks_age: None,
            head_dy: 0,
            ..pose
        };
    }
    if long > 2000 && long % 48 < 6 {
        return Pose { right: Droite::Wipe, sweat: true, ..pose };
    }
    pose
}

// Le modèle réfléchit : la patte sous le menton, un engrenage qui tourne dans un nuage.
fn thinking(t: i64, w: &Monde) -> Pose {
    let c = t % 30;
    Pose {
        look: if c % 10 < 6 { 1 } else { 0 },
        head_dy: if c % 20 < 10 { 0 } else { 1 },
        thought: true,
        ..Pose::base(
            if c == 11 {
                Yeux::Blink
            } else if c % 10 < 6 {
                Yeux::Look
            } else {
                Yeux::Open
            },
            Bouche::None,
            Gauche::Rest,
            Droite::Chin,
            sway(t, 6),
            (1 + w.effort).min(3),
            Lingot::Hot,
            4,
        )
    }
}

fn review(t: i64, w: &Monde) -> Pose {
    let c = t % 24;
    Pose {
        glints: matches!(w.flavor, Saveur::Web | Saveur::Mcp | Saveur::Agent),
        head_dy: 1,
        scroll: true,
        ..Pose::base(
            if c == 13 {
                Yeux::Blink
            } else if c % 8 < 4 {
                Yeux::Look
            } else {
                Yeux::Open
            },
            Bouche::None,
            Gauche::Scroll,
            Droite::Scroll,
            sway(t, 6),
            1,
            Lingot::Hot,
            4,
        )
    }
}

// Il attend la personne ; passé 30 s il tape du pied.
fn waiting(t: i64) -> Pose {
    Pose {
        worried: true,
        ear: t % 9 < 2,
        look: if t % 12 < 6 { 1 } else { 0 },
        tap: t > 200 && t % 4 < 2,
        bubble: Some(at(&[0, 0, 1, 1], t)),
        ..Pose::base(if t % 20 == 9 { Yeux::Blink } else { Yeux::Open }, Bouche::None, Gauche::Rest, Droite::Up, sway(t, 3), 1, Lingot::Hot, 5)
    }
}

fn failed(t: i64) -> Pose {
    Pose {
        worried: true,
        head_dy: 1,
        puff: if t < 7 { Some(t) } else { None },
        ..Pose::base(Yeux::Sad, Bouche::Frown, if t < 1 { Gauche::Strike } else { Gauche::Drop }, Droite::Rest, 0, 0, Lingot::Cold, 0)
    }
}

fn jumping(t: i64) -> Pose {
    let c = t % 8;
    Pose {
        dy: at(&[0, -1, -3, -4, -3, -1, 0, 0], c),
        stars: true,
        fireworks: Some(t),
        ..Pose::base(
            Yeux::Happy,
            if c < 4 { Bouche::Open } else { Bouche::Smile },
            Gauche::Raise,
            Droite::Up,
            (c % 2) as usize,
            3,
            Lingot::Flash,
            2,
        )
    }
}

fn waving(t: i64) -> Pose {
    Pose {
        sparks_age: if (4..10).contains(&t) { Some(t - 4) } else { None },
        ..Pose::base(
            Yeux::Happy,
            Bouche::Smile,
            Gauche::Rest,
            if t.div_euclid(2) % 2 != 0 { Droite::Wave1 } else { Droite::Wave2 },
            sway(t, 2),
            t.div_euclid(4).min(2) as i32, // la forge s'allume
            Lingot::Cold,
            if t > 6 { 4 } else { 0 },
        )
    }
}

// Les tests sont finis : la lame va au seau, fume, et tient ou se fend.
fn quench(t: i64, w: &Monde) -> Pose {
    let pass = w.outcome != Some(Issue::Fail);
    let base = Pose {
        focused: true,
        look: -1,
        blade: Some(Lame::Hot),
        bucket: true,
        ..Pose::base(Yeux::Open, Bouche::None, Gauche::Blade, Droite::Rest, sway(t, 4), 2, Lingot::None, 3)
    };
    if t < 3 {
        return base;
    }
    if t < 9 {
        return Pose { left: Gauche::Dip, eyes: Yeux::Blink, steam: Some(t - 3), ..base };
    }
    if pass {
        Pose {
            blade: Some(Lame::Proven),
            eyes: Yeux::Happy,
            mouth: Bouche::Smile,
            look: 0,
            right: Droite::Up,
            steam: Some(t - 3),
            stars: t < 14,
            ..base
        }
    } else {
        Pose {
            blade: Some(Lame::Cracked),
            eyes: Yeux::Sad,
            mouth: Bouche::Frown,
            worried: true,
            head_dy: 1,
            steam: Some(t - 3),
            puff: Some(t - 9),
            ..base
        }
    }
}

// git : il pose la lame du jour dans la caisse, ferme le couvercle et le scelle de cire d'or ;
// sur un push le corbeau descend et l'emporte. L'histoire tient dans les 2,4 s d'un commit.
fn ship(t: i64, w: &Monde) -> Pose {
    let base = Pose {
        focused: true,
        look: -1,
        crate_: Some(Caisse::Sealed),
        seal_age: Some(t - 8),
        ..Pose::base(Yeux::Open, Bouche::None, Gauche::Rest, Droite::Rest, sway(t, 4), 2, Lingot::None, 4)
    };
    if t < 4 {
        return Pose { left: Gauche::Stow, stow: Some((4 - t) as i32), crate_: Some(Caisse::Open), seal_age: None, ..base };
    }
    if t < 6 {
        return Pose {
            left: Gauche::Lid,
            crate_: Some(if t == 4 { Caisse::Closing } else { Caisse::Shut }),
            lid_puff: t == 5,
            seal_age: None,
            ..base
        };
    }
    if t < 8 {
        return Pose {
            left: Gauche::StampUp,
            crate_: Some(Caisse::Shut),
            seal_age: None,
            eyes: if t == 7 { Yeux::Blink } else { Yeux::Open },
            ..base
        };
    }
    if t < 10 {
        return Pose { left: Gauche::StampDown, mouth: Bouche::Grit, head_dy: 1, ..base };
    }
    let proud = Pose { eyes: Yeux::Happy, mouth: Bouche::Smile, focused: false, look: 0, right: Droite::Up, ..base };
    if w.ship != Some(Envoi::Push) || t < 11 {
        return proud;
    }
    // le corbeau : de la droite en trois tics, puis parti avec la caisse
    let raven = t - 14;
    Pose {
        raven: Some(raven),
        crate_: Some(if raven >= 0 { Caisse::Gone } else { Caisse::Sealed }),
        right: if raven >= 0 {
            if t.div_euclid(2) % 2 != 0 {
                Droite::Wave1
            } else {
                Droite::Wave2
            }
        } else {
            Droite::Up
        },
        ..proud
    }
}

// Le contexte se compacte : il balaie l'atelier. Un coup en huit temps : la tête poussée
// sur le sol, la poussière qui vole devant, puis levée et ramenée pour le suivant.
fn sweep(t: i64) -> Pose {
    let c = t % 8;
    Pose {
        look: 1,
        focused: c < 5,
        sweep: Some(at(&[0, 1, 2, 3, 4, 3, 2, 1], c)),
        broom_up: c >= 5,
        dust: if (1..=5).contains(&c) { Some(c - 1) } else { None },
        ..Pose::base(
            if t % 20 == 7 { Yeux::Blink } else { Yeux::Open },
            if c < 5 { Bouche::Grit } else { Bouche::None },
            Gauche::Broom,
            Droite::Broom,
            sway(t, 3),
            1,
            Lingot::Cold,
            5,
        )
    }
}

// Dix minutes sans rien à faire : il somnole sur son tabouret, le feu réduit aux braises.
fn sleep(t: i64) -> Pose {
    Pose {
        head_dy: if t % 24 < 12 { 1 } else { 0 },
        seated: true,
        stool: true,
        zzz: true,
        ..Pose::base(Yeux::Blink, Bouche::None, Gauche::Lap, Droite::Lap, 0, 0, Lingot::Cold, 0)
    }
}

// Un message le réveille en sursaut.
fn waking(t: i64) -> Pose {
    Pose {
        ear: true,
        dy: at(&[-3, -2, -1, 0], t.min(3)),
        bubble: if t < 4 { Some(0) } else { None },
        ..Pose::base(
            Yeux::Open,
            if t < 3 { Bouche::Open } else { Bouche::None },
            Gauche::Rest,
            Droite::Up,
            sway(t, 1),
            t.min(2) as i32,
            Lingot::Cold,
            3,
        )
    }
}

pub fn pose_of(activity: Activite, tick: i64, world: &Monde) -> Pose {
    match activity {
        Activite::Idle | Activite::Away => idle(tick, world), // parti : sa forge reste vide
        Activite::Running => running(tick, world),
        Activite::Thinking => thinking(tick, world),
        Activite::Review => review(tick, world),
        Activite::Waiting => waiting(tick),
        Activite::Failed => failed(tick),
        Activite::Jumping => jumping(tick),
        Activite::Waving => waving(tick),
        Activite::Quench => quench(tick, world),
        Activite::Ship => ship(tick, world),
        Activite::Sweep => sweep(tick),
        Activite::Sleep => sleep(tick),
        Activite::Waking => waking(tick),
    }
}
