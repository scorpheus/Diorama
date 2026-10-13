//! La toile d'un bandeau : une fenêtre en couches, sans bord, traversée par la souris, jamais
//! active, possédée par la fenêtre du terminal (elle se tient juste au-dessus d'elle, et la suit
//! quand on la réduit ou qu'une autre passe devant). Son image est en mémoire : des pixels
//! 32 bits, bleu-vert-rouge-alpha, alpha prémultiplié ; on n'en remet au compositeur de Windows
//! que la zone qui a changé (UpdateLayeredWindowIndirect).
//!
//! Pas de carte graphique de notre côté : son pilote coûtait, à chaque image, plus que tout le
//! reste de l'appli (docs/plan-rendu.md).

use windows::core::{w, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject, AC_SRC_ALPHA, AC_SRC_OVER,
    BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassExW, ShowWindow, UpdateLayeredWindowIndirect, HWND_MESSAGE,
    SW_HIDE, SW_SHOWNOACTIVATE, ULW_ALPHA, UPDATELAYEREDWINDOWINFO, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT, WS_POPUP,
};

const CLASSE: PCWSTR = w!("DioramaRendu");

unsafe extern "system" fn procedure(f: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    DefWindowProcW(f, msg, w, l)
}

/// La classe de fenêtre des toiles et de la fenêtre des messages, une fois par processus.
pub fn inscrire() -> Result<()> {
    unsafe {
        let classe = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(procedure),
            hInstance: GetModuleHandleW(None)?.into(),
            lpszClassName: CLASSE,
            ..Default::default()
        };
        if RegisterClassExW(&classe) == 0 {
            return Err(windows::core::Error::from_win32());
        }
    }
    Ok(())
}

/// Une fenêtre invisible qui ne sert qu'à recevoir les messages de l'appli (minuteries, réveils).
pub fn fenetre_des_messages() -> Result<HWND> {
    unsafe {
        CreateWindowExW(
            Default::default(),
            CLASSE,
            w!("Diorama"),
            Default::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(GetModuleHandleW(None)?.into()),
            None,
        )
    }
}

pub struct Toile {
    pub fenetre: HWND,
    dc: HDC,
    image: HBITMAP,
    ancien: HGDIOBJ,
    pixels: *mut u32,
    pub w: u32,
    pub h: u32,
    x: i32,
    y: i32,
    pub visible: bool,
}

impl Toile {
    /// Une toile, cachée, possédée par `proprietaire` (le terminal) s'il est donné.
    pub fn new(proprietaire: Option<HWND>, w: u32, h: u32) -> Result<Self> {
        let fenetre = unsafe {
            CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
                CLASSE,
                w!("Diorama"),
                WS_POPUP,
                0,
                0,
                w as i32,
                h as i32,
                proprietaire,
                None,
                Some(GetModuleHandleW(None)?.into()),
                None,
            )?
        };
        let mut t = Toile {
            fenetre,
            dc: HDC::default(),
            image: HBITMAP::default(),
            ancien: HGDIOBJ::default(),
            pixels: std::ptr::null_mut(),
            w: 0,
            h: 0,
            x: 0,
            y: 0,
            visible: false,
        };
        t.allouer(w, h)?;
        Ok(t)
    }

    fn liberer(&mut self) {
        if !self.dc.is_invalid() {
            unsafe {
                SelectObject(self.dc, self.ancien);
                let _ = DeleteObject(self.image.into());
                let _ = DeleteDC(self.dc);
            }
            self.dc = HDC::default();
            self.pixels = std::ptr::null_mut();
        }
    }

    /// L'image en mémoire, à cette taille ; son contenu est à repeindre en entier.
    fn allouer(&mut self, w: u32, h: u32) -> Result<()> {
        self.liberer();
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w as i32,
                biHeight: -(h as i32), // de haut en bas
                biPlanes: 1,
                biBitCount: 32,
                ..Default::default()
            },
            ..Default::default()
        };
        unsafe {
            let ecran = GetDC(None);
            self.dc = CreateCompatibleDC(Some(ecran));
            let mut bits = std::ptr::null_mut();
            let image = CreateDIBSection(Some(ecran), &info, DIB_RGB_COLORS, &mut bits, None, 0);
            ReleaseDC(None, ecran);
            self.image = image?;
            self.pixels = bits as *mut u32;
            self.ancien = SelectObject(self.dc, self.image.into());
        }
        self.w = w;
        self.h = h;
        Ok(())
    }

    /// Les pixels de l'image, ligne après ligne : 0xAARRGGBB, alpha prémultiplié.
    pub fn pixels(&mut self) -> &mut [u32] {
        if self.pixels.is_null() {
            return &mut [];
        }
        unsafe { std::slice::from_raw_parts_mut(self.pixels, (self.w * self.h) as usize) }
    }

    /// Place la toile à l'écran (pixels physiques), à cette taille. `true` si l'image a été
    /// refaite (une autre taille) : elle est alors à repeindre en entier.
    pub fn poser(&mut self, x: i32, y: i32, w: u32, h: u32) -> Result<bool> {
        let neuve = (w, h) != (self.w, self.h);
        if neuve {
            self.allouer(w, h)?;
        }
        self.x = x;
        self.y = y;
        Ok(neuve)
    }

    /// Remet au compositeur la zone qui a changé (toute l'image si `None`), et montre la toile.
    pub fn presenter(&mut self, zone: Option<RECT>) {
        let tout = RECT { left: 0, top: 0, right: self.w as i32, bottom: self.h as i32 };
        let zone = zone.unwrap_or(tout);
        let (taille, origine, place) = (SIZE { cx: self.w as i32, cy: self.h as i32 }, POINT { x: 0, y: 0 }, POINT { x: self.x, y: self.y });
        let melange = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA as u8 };
        let info = UPDATELAYEREDWINDOWINFO {
            cbSize: std::mem::size_of::<UPDATELAYEREDWINDOWINFO>() as u32,
            pptDst: &place,
            psize: &taille,
            hdcSrc: self.dc,
            pptSrc: &origine,
            pblend: &melange,
            dwFlags: ULW_ALPHA,
            prcDirty: &zone,
            ..Default::default()
        };
        unsafe {
            let _ = UpdateLayeredWindowIndirect(self.fenetre, &info);
            if !self.visible {
                let _ = ShowWindow(self.fenetre, SW_SHOWNOACTIVATE);
            }
        }
        self.visible = true;
    }

    pub fn cacher(&mut self) {
        if self.visible {
            unsafe {
                let _ = ShowWindow(self.fenetre, SW_HIDE);
            }
            self.visible = false;
        }
    }

    /// Où la toile se tient à l'écran.
    pub fn place(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl Drop for Toile {
    fn drop(&mut self) {
        self.liberer();
        unsafe {
            let _ = DestroyWindow(self.fenetre);
        }
    }
}
