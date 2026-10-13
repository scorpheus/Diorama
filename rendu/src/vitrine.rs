//! La vitrine : une fenêtre ordinaire où la forge joue chaque moment de la planche, à la taille
//! et à la cadence du bandeau, peinte par la même scène que l'appli. Pour regarder ce qu'on
//! dessine sans passer par une session de Claude Code.
//!
//!   ← →     le moment d'avant, d'après
//!   espace  pause
//!   R       le moment repris de son début

use windows::core::{w, Result, HSTRING};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC, SetDIBitsToDevice, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRectEx, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage, RegisterClassExW, SetTimer,
    SetWindowTextW, ShowWindow, TranslateMessage, CW_USEDEFAULT, MSG, SW_SHOW, WM_DESTROY, WM_KEYDOWN, WM_TIMER, WNDCLASSEXW,
    WS_CAPTION, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU,
};

use crate::forge::atelier::ROWS;
use crate::forge::monde::{Accessoire, Activite, Lettre};
use crate::forge::{planche, Etat, SceneForge};
use crate::scene::{Image, Scene};

const CW: i32 = 9;
const CH: i32 = 19;
const COLONNES: i32 = 144;
const FOND: u32 = 0xFF0C_0C0C; // le fond du terminal, là où la scène est transparente
/// Un moment rejoue depuis son début au bout de ce nombre de tics : les suites (trempe, envoi)
/// se revoient en entier.
const BOUCLE: i64 = 80;

unsafe extern "system" fn procedure(f: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg == WM_DESTROY {
        PostQuitMessage(0);
        return LRESULT(0);
    }
    DefWindowProcW(f, msg, w, l)
}

struct Vitrine {
    scenes: Vec<(Activite, i64, crate::forge::monde::Monde)>,
    n: usize,
    scene: SceneForge,
    tics: i64,
    pause: bool,
    /// sa tenue, essayée par-dessus celle de la scène (C, A) : l'une de CAPES, l'un de ACCESSOIRES
    cape: usize,
    acc: usize,
}

const CAPES: [(&str, [u8; 2]); 6] = [("forêt", *b"oO"), ("braise", *b"CD"), ("nuit", *b"NL"), ("prune", *b"UI"), ("ocre", *b"TJ"), ("ardoise", *b"Zv")];
const ACCESSOIRES: [(&str, Accessoire); 4] =
    [("sans accessoire", Accessoire::Aucun), ("lunettes", Accessoire::Lunettes), ("bandeau", Accessoire::Bandeau), ("tablier", Accessoire::Tablier)];

impl Vitrine {
    fn montrer(&mut self, n: usize) {
        self.n = n % self.scenes.len();
        let (activite, _, mut monde) = self.scenes[self.n].clone();
        let cape = CAPES[self.cape].1;
        monde.cape = [Lettre(cape[0]), Lettre(cape[1])];
        monde.accessory = ACCESSOIRES[self.acc].1;
        self.scene = SceneForge::new();
        self.scene.recevoir(Etat { recadrage: crate::forge::monde::Recadrage::Full, extra: COLONNES - 72, lit: true, activite, tick: 0, monde });
        self.tics = 0;
    }

    fn titre(&self) -> String {
        let (a, _, w) = &self.scenes[self.n];
        let mut quoi = format!("{a:?}");
        for (vrai, mot) in [
            (w.walking, "en marche"),
            (w.nuit(), "la nuit"),
            (!w.apprentices.is_empty(), "les louveteaux"),
            (!w.guests.is_empty(), "des invités"),
            (w.cat.is_some(), "le chat"),
            (w.weather.is_some(), "l'orage"),
            (w.banquet, "le banquet"),
            (w.speaking, "il parle"),
        ] {
            if vrai {
                quoi += &format!(" · {mot}");
            }
        }
        format!(
            "Diorama · vitrine · {}/{} · {quoi} · cape {}, {}{}   (← → moments, espace pause, R reprendre, C cape, A accessoire)",
            self.n + 1,
            self.scenes.len(),
            CAPES[self.cape].0,
            ACCESSOIRES[self.acc].0,
            if self.pause { " · en pause" } else { "" }
        )
    }
}

pub fn vitrine(cadence_ms: u32) -> Result<()> {
    let (w, h) = (COLONNES * CW, ROWS * CH);
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let classe = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(procedure),
            hInstance: instance.into(),
            lpszClassName: w!("DioramaVitrine"),
            ..Default::default()
        };
        RegisterClassExW(&classe);
        let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
        let mut cadre = RECT { left: 0, top: 0, right: w, bottom: h };
        AdjustWindowRectEx(&mut cadre, style, false, Default::default())?;
        let fenetre = CreateWindowExW(
            Default::default(),
            w!("DioramaVitrine"),
            w!("Diorama · vitrine"),
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            cadre.right - cadre.left,
            cadre.bottom - cadre.top,
            None,
            None,
            Some(instance.into()),
            None,
        )?;
        let _ = ShowWindow(fenetre, SW_SHOW);
        let mut v = Vitrine { scenes: planche::scenes(), n: 0, scene: SceneForge::new(), tics: 0, pause: false, cape: 0, acc: 0 };
        v.montrer(0);
        let _ = SetWindowTextW(fenetre, &HSTRING::from(v.titre()));
        let mut pixels = vec![0u32; (w * h) as usize];
        let mut vue = vec![0u32; (w * h) as usize];
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                ..Default::default()
            },
            ..Default::default()
        };
        SetTimer(Some(fenetre), 1, cadence_ms, None);
        let mut neuve = true;
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_KEYDOWN {
                let avant = (v.n, v.pause);
                match msg.wParam.0 {
                    0x25 => v.montrer(v.n + v.scenes.len() - 1), // ←
                    0x27 => v.montrer(v.n + 1),                  // →
                    0x20 => v.pause = !v.pause,                  // espace
                    0x52 => v.montrer(v.n),                      // R
                    0x43 => {
                        v.cape = (v.cape + 1) % CAPES.len(); // C
                        v.montrer(v.n);
                    }
                    0x41 => {
                        v.acc = (v.acc + 1) % ACCESSOIRES.len(); // A
                        v.montrer(v.n);
                    }
                    _ => {}
                }
                if avant != (v.n, v.pause) || [0x52, 0x43, 0x41].contains(&msg.wParam.0) {
                    neuve = true;
                    let _ = SetWindowTextW(fenetre, &HSTRING::from(v.titre()));
                }
                continue;
            }
            if msg.message == WM_TIMER && msg.hwnd == fenetre {
                if v.pause && !neuve {
                    continue;
                }
                if v.tics >= BOUCLE {
                    v.montrer(v.n);
                    neuve = true;
                }
                let dt = cadence_ms as f32 / 1000.0;
                let mut image = Image { pixels: &mut pixels, w: w as u32, h: h as u32, cellule: (CW as u32, CH as u32) };
                v.scene.peindre(&mut image, dt, neuve);
                neuve = false;
                v.tics += 1;
                // la scène sur le fond du terminal
                for (d, &p) in vue.iter_mut().zip(&pixels) {
                    let a = p >> 24;
                    *d = if a == 255 {
                        p
                    } else {
                        let m = |s: u32, f: u32| (s + f * (255 - a) / 255).min(255);
                        0xFF00_0000 | (m((p >> 16) & 255, (FOND >> 16) & 255) << 16) | (m((p >> 8) & 255, (FOND >> 8) & 255) << 8) | m(p & 255, FOND & 255)
                    };
                }
                let dc = GetDC(Some(fenetre));
                SetDIBitsToDevice(dc, 0, 0, w as u32, h as u32, 0, 0, 0, h as u32, vue.as_ptr() as *const _, &info, DIB_RGB_COLORS);
                ReleaseDC(Some(fenetre), dc);
                continue;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}
