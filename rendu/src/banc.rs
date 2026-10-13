//! Le banc : ce que coûte l'appli, poste par poste, sans rien montrer à l'écran.
//!
//!   rendu.exe --banc [<secondes>]
//!
//! Une toile hors de l'écran (posée et présentée, mais sur aucun moniteur), de la taille d'un
//! bandeau de 144 × 16 cellules de 9 × 19 ; chaque variante tourne <secondes> (20 par défaut)
//! et le banc note au journal le processeur pris pendant ce temps, en % d'un cœur.
//!
//! Le temps se compte en cycles, exactement : celui que Windows donne en temps n'est qu'un
//! échantillon pris à chaque top d'horloge (15,6 ms), qui perd ou grossit au hasard les petits
//! réveils d'une appli comme celle-ci (±45 % sur 40 s).

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::{GetCurrentProcess, GetCurrentThread};
use windows::Win32::System::WindowsProgramming::{QueryProcessCycleTime, QueryThreadCycleTime};
use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, KillTimer, SetTimer, MSG, WM_TIMER};

use crate::journal::note;
use crate::reperes::Capture;
use crate::forge::monde::{Activite, Lettre, Monde, Petit, Recadrage};
use crate::forge::{Etat, SceneForge};
use crate::scene::{Image, Mire, Scene};
use crate::toile::Toile;

const W: u32 = 144 * 9;
const H: u32 = 16 * 19;

/// Les cycles que compte un cœur en une seconde, mesurés une fois sur une boucle d'un tiers de seconde.
fn cadence_coeur() -> f64 {
    static CADENCE: OnceLock<f64> = OnceLock::new();
    *CADENCE.get_or_init(|| {
        let (t0, c0) = (Instant::now(), cycles_fil());
        let mut x = 1u64;
        while t0.elapsed() < Duration::from_millis(330) {
            x = std::hint::black_box(x.wrapping_mul(6364136223846793005).wrapping_add(1));
        }
        (cycles_fil() - c0) as f64 / t0.elapsed().as_secs_f64()
    })
}

fn cycles_fil() -> u64 {
    let mut c = 0u64;
    unsafe {
        let _ = QueryThreadCycleTime(GetCurrentThread(), &mut c);
    }
    c
}

/// Le temps de processeur de tout le processus, en secondes.
fn cpu() -> f64 {
    let mut c = 0u64;
    unsafe {
        let _ = QueryProcessCycleTime(GetCurrentProcess(), &mut c);
    }
    c as f64 / cadence_coeur()
}

#[derive(Clone, Copy, PartialEq)]
enum Quoi {
    Rien,
    Regards,
    Mire,
    Forge(Activite),
}

/// Combien de coups d'œil sur 8 lisent l'écran : au repos, un (le coup de sûreté, toutes les
/// 2 s) ; quand Windows fait signe sans cesse (on tape dans le terminal), tous.
const REPOS: u64 = 1;
const AGITE: u64 = 8;

pub fn banc(messages: HWND, secondes: u64) {
    note(&format!("banc : un cœur compte {:.0} millions de cycles par seconde", cadence_coeur() / 1e6));
    let mut toile = match Toile::new(None, W, H) {
        Ok(t) => t,
        Err(e) => {
            note(&format!("banc : pas de toile : {e}"));
            return;
        }
    };
    let _ = toile.poser(-32000, -32000, W, H);
    let mut capture = Capture::new();
    let variantes = [
        (Quoi::Rien, REPOS, "à vide (minuteries seules)"),
        (Quoi::Regards, REPOS, "coups d'œil au repos (toutes les 2 s)"),
        (Quoi::Regards, AGITE, "coups d'œil agités (4 par seconde)"),
        (Quoi::Mire, REPOS, "coups d'œil au repos + la mire, à 150 ms"),
        (Quoi::Forge(Activite::Idle), REPOS, "coups d'œil au repos + la forge au repos, à 150 ms"),
        (Quoi::Forge(Activite::Running), REPOS, "coups d'œil au repos + la forge au travail, à 150 ms"),
    ];
    for (quoi, lus, nom) in variantes {
        let mut scene: Box<dyn Scene> = match quoi {
            Quoi::Forge(activite) => Box::new(forge(activite)),
            _ => Box::new(Mire::new()),
        };
        let mut neuve = true;
        let mut regards = 0u64;
        unsafe {
            SetTimer(Some(messages), 1, 150, None);
            SetTimer(Some(messages), 2, 250, None);
        }
        let (debut, cpu0) = (Instant::now(), cpu());
        let mut vue = Instant::now();
        let (mut images, mut ms) = (0u64, 0f64);
        let mut msg = MSG::default();
        while debut.elapsed() < Duration::from_secs(secondes) && unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
            if msg.message != WM_TIMER || msg.hwnd != messages {
                unsafe { DispatchMessageW(&msg) };
                continue;
            }
            match (msg.wParam.0, quoi) {
                (2, Quoi::Rien) => {}
                (2, _) => {
                    regards += 1;
                    if regards % 8 < lus {
                        capture.prendre(4, 4, 1, 19);
                        capture.prendre(1200, 300, 1, 19);
                    }
                }
                (1, Quoi::Mire | Quoi::Forge(_)) => {
                    let t0 = Instant::now();
                    let mut image = Image { pixels: toile.pixels(), w: W, h: H, cellule: (9, 19) };
                    let zone = scene.peindre(&mut image, t0.duration_since(vue).as_secs_f32(), std::mem::take(&mut neuve));
                    vue = t0;
                    toile.presenter(zone);
                    images += 1;
                    ms += t0.elapsed().as_secs_f64() * 1000.0;
                }
                _ => {}
            }
        }
        unsafe {
            let _ = KillTimer(Some(messages), 1);
            let _ = KillTimer(Some(messages), 2);
        }
        let pc = 100.0 * (cpu() - cpu0) / debut.elapsed().as_secs_f64();
        let par_image = if images > 0 { format!(", {:.3} ms d'horloge par image ({images} images)", ms / images as f64) } else { String::new() };
        note(&format!("banc : {nom} : {pc:.3} % d'un cœur{par_image}"));
    }
}

/// La forge du bandeau du banc : 144 colonnes, la cour élargie au plus, trois louveteaux.
fn forge(activite: Activite) -> SceneForge {
    let mut monde = Monde { clock: 1, ..Monde::default() };
    monde.apprentices = (0..3).map(|i| Petit { seed: 3 + 4 * i, colour: Lettre(b"PbG"[i as usize]), age: 20 + 30 * i, leaving: None }).collect();
    // les cases du pet, faites une fois pour toute l'appli : hors de la mesure
    let t = std::time::Instant::now();
    if crate::forge::pet::cases(19, b'O').is_some() && t.elapsed().as_millis() > 1 {
        note(&format!("banc : la planche du pet décodée et mise à l'échelle, une fois : {} ms", t.elapsed().as_millis()));
    }
    let mut s = SceneForge::new();
    s.recevoir(Etat { recadrage: Recadrage::Full, extra: 72, lit: true, activite, tick: 0, monde });
    s
}
