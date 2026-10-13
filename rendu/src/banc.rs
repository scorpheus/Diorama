//! Le banc : ce que coûte l'appli, poste par poste, sans rien montrer à l'écran.
//!
//!   rendu.exe --banc [<secondes>] [images]
//!
//! « images » : seulement les variantes qui comparent les chemins de l'image (carte graphique,
//! toute l'image ou sa zone changée ; fenêtre en couches peinte par le processeur, idem).
//!
//! Une toile hors de l'écran (posée, présentée, mais sur aucun moniteur), de la taille d'un
//! bandeau de 144 × 16 cellules de 9 × 19 ; chaque variante tourne <secondes> (20 par défaut)
//! et le banc note au journal le CPU du processus pendant ce temps, en % d'un cœur.

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use windows::Win32::Graphics::Direct2D::Common::{D2D1_COLOR_F, D2D_RECT_F};
use windows::Win32::System::Threading::{GetCurrentProcess, GetCurrentThread};
use windows::Win32::System::WindowsProgramming::{QueryProcessCycleTime, QueryThreadCycleTime};
use windows::core::w;
use windows::Win32::Foundation::{HWND, POINT, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, GetDC, ReleaseDC, SelectObject, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER,
    BLENDFUNCTION, DIB_RGB_COLORS, HDC,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DispatchMessageW, GetMessageW, KillTimer, SetTimer, ShowWindow, UpdateLayeredWindowIndirect, MSG, SW_SHOWNOACTIVATE,
    UPDATELAYEREDWINDOWINFO, ULW_ALPHA, WM_TIMER, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
};

use crate::etincelles::Gerbe;
use crate::journal::note;
use crate::peintre::{Peintre, Toile};
use crate::reperes::Capture;

const W: u32 = 144 * 9;
const H: u32 = 16 * 19;

// Le temps de processeur se compte en cycles, exactement : celui que Windows donne en temps
// n'est qu'un échantillon pris à chaque top d'horloge (15,6 ms), qui perd ou grossit au hasard
// les petits réveils d'une appli comme celle-ci.

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

/// Le temps de processeur du fil principal (qui dessine), en secondes.
fn cpu_fil() -> f64 {
    cycles_fil() as f64 / cadence_coeur()
}

/// Le temps de processeur de tout le processus, ses fils et ceux du pilote graphique, en secondes.
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
    Simple,
    SimpleZone,
    Couche,
    CoucheZone,
    Gerbe,
}

/// Une fenêtre en couches peinte par le processeur : une image en mémoire, remise au
/// compositeur de Windows par UpdateLayeredWindow.
struct Couche {
    fenetre: HWND,
    dc: HDC,
    pixels: *mut u32,
}

impl Couche {
    fn new() -> Option<Self> {
        unsafe {
            let fenetre = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED,
                w!("DioramaRendu"),
                w!("Diorama"),
                WS_POPUP,
                -32000,
                -32000,
                W as i32,
                H as i32,
                None,
                None,
                Some(GetModuleHandleW(None).ok()?.into()),
                None,
            )
            .ok()?;
            let info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER { biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32, biWidth: W as i32, biHeight: -(H as i32), biPlanes: 1, biBitCount: 32, ..Default::default() },
                ..Default::default()
            };
            let ecran = GetDC(None);
            let dc = CreateCompatibleDC(Some(ecran));
            let mut bits = std::ptr::null_mut();
            let image = CreateDIBSection(Some(ecran), &info, DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
            ReleaseDC(None, ecran);
            SelectObject(dc, image.into());
            let _ = ShowWindow(fenetre, SW_SHOWNOACTIVATE);
            Some(Couche { fenetre, dc, pixels: bits as *mut u32 })
        }
    }

    /// Remplit le rectangle de la couleur (prémultipliée, opaque).
    fn remplir(&self, x0: u32, y0: u32, x1: u32, y1: u32, couleur: u32) {
        for y in y0..y1.min(H) {
            let ligne = unsafe { std::slice::from_raw_parts_mut(self.pixels.add((y * W) as usize), W as usize) };
            ligne[x0 as usize..x1.min(W) as usize].fill(couleur);
        }
    }

    fn presenter(&self, zone: Option<RECT>) {
        let (taille, origine) = (SIZE { cx: W as i32, cy: H as i32 }, POINT { x: 0, y: 0 });
        let melange = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA as u8 };
        let z = zone.unwrap_or(RECT { left: 0, top: 0, right: W as i32, bottom: H as i32 });
        let info = UPDATELAYEREDWINDOWINFO {
            cbSize: std::mem::size_of::<UPDATELAYEREDWINDOWINFO>() as u32,
            psize: &taille,
            hdcSrc: self.dc,
            pptSrc: &origine,
            pblend: &melange,
            dwFlags: ULW_ALPHA,
            prcDirty: &z,
            ..Default::default()
        };
        unsafe {
            let _ = UpdateLayeredWindowIndirect(self.fenetre, &info);
        }
    }
}

/// Combien de coups d'œil sur 8 lisent l'écran : au repos, un (le coup de sûreté, toutes
/// les 2 s) ; quand Windows fait signe sans cesse (on tape dans le terminal), tous.
const REPOS: u64 = 1;
const AGITE: u64 = 8;

pub fn banc(peintre: &Peintre, messages: HWND, secondes: u64, logiciel: bool, images_seulement: bool) {
    note(&format!("banc : un cœur compte {:.0} millions de cycles par seconde", cadence_coeur() / 1e6));
    let mut toile = match Toile::new(peintre, None, W, H) {
        Ok(t) => t,
        Err(e) => {
            note(&format!("banc : pas de toile : {e}"));
            return;
        }
    };
    let _ = toile.poser(-32000, -32000, W, H);
    let pinceau = unsafe { peintre.dc.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.9, g: 0.5, b: 0.2, a: 1.0 }, None) }.ok();
    let trous = [D2D_RECT_F { left: 0.0, top: 0.0, right: 9.0, bottom: 19.0 }, D2D_RECT_F { left: W as f32 - 9.0, top: H as f32 - 19.0, right: W as f32, bottom: H as f32 }];
    let mut capture = Capture::new();
    let couche = Couche::new();
    let variantes_images = [
        (Quoi::Simple, 150, REPOS, "carré qui glisse, carte graphique, toute l'image"),
        (Quoi::SimpleZone, 150, REPOS, "carré qui glisse, carte graphique, la zone changée"),
        (Quoi::Couche, 150, REPOS, "carré qui glisse, fenêtre en couches, toute l'image"),
        (Quoi::CoucheZone, 150, REPOS, "carré qui glisse, fenêtre en couches, la zone changée"),
    ];
    let variantes = [
        (Quoi::Rien, 150, REPOS, "à vide (minuteries seules)"),
        (Quoi::Regards, 150, REPOS, "coups d'œil au repos (toutes les 2 s)"),
        (Quoi::Regards, 150, AGITE, "coups d'œil agités (4 par seconde)"),
        (Quoi::Simple, 150, REPOS, "coups d'œil au repos + un carré qui glisse, à 150 ms"),
        (Quoi::Gerbe, 150, REPOS, "coups d'œil au repos + la gerbe, à 150 ms"),
        (Quoi::Gerbe, 33, REPOS, "coups d'œil au repos + la gerbe, à 33 ms"),
    ];
    let liste: &[(Quoi, u32, u64, &str)] = if images_seulement { &variantes_images } else { &variantes };
    for &(quoi, cadence, lus, nom) in liste {
        let mut regards = 0u64;
        let mut scene = Gerbe::new(7);
        scene.cadence_ms = cadence;
        scene.cellule = (9, 19);
        if quoi == Quoi::Gerbe {
            let _ = scene.preparer(&peintre.dc, &peintre.d2d, &peintre.ecriture, W, H);
        }
        unsafe {
            SetTimer(Some(messages), 1, cadence, None);
            SetTimer(Some(messages), 2, 250, None);
        }
        let (debut, cpu0, fil0) = (Instant::now(), cpu(), cpu_fil());
        let mut vue = Instant::now();
        let (mut images, mut ms_peinture) = (0u64, 0f64);
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
                (1, Quoi::Simple) => {
                    let t0 = Instant::now();
                    let x = (images as f32 * 4.0) % (W as f32 - 20.0);
                    let _ = toile.peindre(peintre, |dc| unsafe {
                        dc.Clear(Some(&D2D1_COLOR_F { r: 0.05, g: 0.05, b: 0.08, a: 1.0 }));
                        if let Some(b) = pinceau.as_ref() {
                            dc.FillRectangle(&D2D_RECT_F { left: x, top: 140.0, right: x + 20.0, bottom: 160.0 }, b);
                        }
                    });
                    images += 1;
                    ms_peinture += t0.elapsed().as_secs_f64() * 1000.0;
                }
                (1, Quoi::SimpleZone) => {
                    let t0 = Instant::now();
                    let (avant, x) = ((images.max(1) - 1) as f32 * 4.0 % (W as f32 - 20.0), (images as f32 * 4.0) % (W as f32 - 20.0));
                    let zone = RECT { left: avant.min(x) as i32, top: 140, right: avant.max(x) as i32 + 21, bottom: 160 };
                    let _ = toile.peindre_zone(peintre, Some(zone), |dc| unsafe {
                        if images < 2 {
                            dc.Clear(Some(&D2D1_COLOR_F { r: 0.05, g: 0.05, b: 0.08, a: 1.0 }));
                        }
                        if let Some(b) = pinceau.as_ref() {
                            dc.FillRectangle(&D2D_RECT_F { left: x, top: 140.0, right: x + 20.0, bottom: 160.0 }, b);
                        }
                    });
                    images += 1;
                    ms_peinture += t0.elapsed().as_secs_f64() * 1000.0;
                }
                (1, Quoi::Couche | Quoi::CoucheZone) => {
                    let Some(c) = couche.as_ref() else { continue };
                    let t0 = Instant::now();
                    let (avant, x) = ((images.max(1) - 1) * 4 % (W as u64 - 20), images * 4 % (W as u64 - 20));
                    let (avant, x) = (avant as u32, x as u32);
                    let fond = 0xFF0D0D14;
                    if quoi == Quoi::Couche || images < 2 {
                        c.remplir(0, 0, W, H, fond);
                        c.remplir(x, 140, x + 20, 160, 0xFFE68033);
                        c.presenter(None);
                    } else {
                        c.remplir(avant, 140, avant + 20, 160, fond);
                        c.remplir(x, 140, x + 20, 160, 0xFFE68033);
                        c.presenter(Some(RECT { left: avant.min(x) as i32, top: 140, right: (avant.max(x) + 20) as i32, bottom: 160 }));
                    }
                    images += 1;
                    ms_peinture += t0.elapsed().as_secs_f64() * 1000.0;
                }
                (1, Quoi::Gerbe) => {
                    let t0 = Instant::now();
                    scene.avancer(t0.duration_since(vue).as_secs_f32());
                    vue = t0;
                    let _ = toile.peindre(peintre, |dc| scene.dessiner(dc, &trous));
                    images += 1;
                    ms_peinture += t0.elapsed().as_secs_f64() * 1000.0;
                }
                _ => {}
            }
        }
        unsafe {
            let _ = KillTimer(Some(messages), 1);
            let _ = KillTimer(Some(messages), 2);
        }
        let duree = debut.elapsed().as_secs_f64();
        let pc = 100.0 * (cpu() - cpu0) / duree;
        let fil = 100.0 * (cpu_fil() - fil0) / duree;
        let par_image = if images > 0 { format!(", {:.3} ms d'horloge par image ({images} images)", ms_peinture / images as f64) } else { String::new() };
        note(&format!("banc{} : {nom} : {pc:.3} % d'un cœur, dont {fil:.3} % au fil principal{par_image}", if logiciel { " (WARP)" } else { "" }));
    }
}
