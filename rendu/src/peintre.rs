//! Peindre : Direct2D sur la carte graphique, posé à l'écran par DirectComposition.
//!
//! Une seule chaîne pour toute l'appli (Direct3D, Direct2D, DirectComposition, DirectWrite) ;
//! une « toile » par bandeau visible : une fenêtre sans bord, traversée par la souris, jamais
//! active, possédée par la fenêtre du terminal (elle se tient juste au-dessus d'elle, et la suit
//! quand on la réduit ou qu'une autre passe devant), et une chaîne d'images à alpha prémultiplié
//! que le compositeur de Windows pose sur l'écran, sans copie par le processeur.

use windows::core::{w, Interface, Result, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HMODULE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Direct2D::Common::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_PIXEL_FORMAT};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Bitmap1, ID2D1DeviceContext, ID2D1Image, ID2D1Factory1, D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
    D2D1_BITMAP_OPTIONS_TARGET, D2D1_BITMAP_PROPERTIES1, D2D1_DEVICE_CONTEXT_OPTIONS_NONE, D2D1_FACTORY_OPTIONS,
    D2D1_FACTORY_TYPE_SINGLE_THREADED,
};
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP};
use windows::Win32::Graphics::Direct3D11::{D3D11CreateDevice, ID3D11Device, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION};
use windows::Win32::Graphics::DirectComposition::{DCompositionCreateDevice, IDCompositionDevice, IDCompositionTarget, IDCompositionVisual};
use windows::Win32::Graphics::DirectWrite::{DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory2, IDXGIDevice, IDXGIFactory2, IDXGISurface, IDXGISwapChain1, DXGI_CREATE_FACTORY_FLAGS, DXGI_PRESENT,
    DXGI_PRESENT_PARAMETERS,
    DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_CHAIN_FLAG, DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
    DXGI_USAGE_RENDER_TARGET_OUTPUT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassExW, SetLayeredWindowAttributes, SetWindowPos, ShowWindow,
    HWND_MESSAGE, LWA_ALPHA, SWP_NOACTIVATE, SWP_NOOWNERZORDER, SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE, WNDCLASSEXW,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
};

const CLASSE: PCWSTR = w!("DioramaRendu");

/// La chaîne de peinture, une pour toute l'appli.
pub struct Peintre {
    d3d: ID3D11Device,
    fabrique: IDXGIFactory2,
    pub dc: ID2D1DeviceContext,
    pub d2d: ID2D1Factory1,
    pub ecriture: IDWriteFactory,
    composition: IDCompositionDevice,
}

impl Peintre {
    /// `logiciel` : le moteur logiciel de Windows (WARP) au lieu de la carte graphique.
    pub fn new(logiciel: bool) -> Result<Self> {
        unsafe {
            let mut d3d = None;
            D3D11CreateDevice(
                None,
                if logiciel { D3D_DRIVER_TYPE_WARP } else { D3D_DRIVER_TYPE_HARDWARE },
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut d3d),
                None,
                None,
            )?;
            let d3d: ID3D11Device = d3d.unwrap();
            let dxgi: IDXGIDevice = d3d.cast()?;
            let fabrique: IDXGIFactory2 = CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0))?;
            let d2d: ID2D1Factory1 = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, Some(&D2D1_FACTORY_OPTIONS::default()))?;
            let dc = d2d.CreateDevice(&dxgi)?.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
            let composition: IDCompositionDevice = DCompositionCreateDevice(&dxgi)?;
            let ecriture: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            Ok(Peintre { d3d, fabrique, dc, d2d, ecriture, composition })
        }
    }

    /// La classe de fenêtre des toiles et de la fenêtre des messages.
    pub fn inscrire(procedure: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT) -> Result<()> {
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
}

pub unsafe extern "system" fn procedure_par_defaut(f: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    DefWindowProcW(f, msg, w, l)
}

/// La toile d'un bandeau : sa fenêtre, sa chaîne d'images, et ce qui l'attache au compositeur.
pub struct Toile {
    pub fenetre: HWND,
    chaine: IDXGISwapChain1,
    _cible: IDCompositionTarget,
    _visuel: IDCompositionVisual,
    image: Option<ID2D1Bitmap1>,
    pub w: u32,
    pub h: u32,
    pub visible: bool,
}

impl Toile {
    pub fn new(p: &Peintre, proprietaire: Option<HWND>, w: u32, h: u32) -> Result<Self> {
        unsafe {
            let fenetre = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOREDIRECTIONBITMAP,
                CLASSE,
                w!("Diorama"),
                WS_POPUP,
                0,
                0,
                w as i32,
                h as i32,
                proprietaire, // le terminal : la toile se tient au-dessus de lui, et le suit
                None,
                Some(GetModuleHandleW(None)?.into()),
                None,
            )?;
            // en couches et transparente : la souris traverse jusqu'au terminal
            SetLayeredWindowAttributes(fenetre, COLORREF(0), 255, LWA_ALPHA)?;
            let description = DXGI_SWAP_CHAIN_DESC1 {
                Width: w,
                Height: h,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                Scaling: DXGI_SCALING_STRETCH,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
                AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
                ..Default::default()
            };
            let chaine = p.fabrique.CreateSwapChainForComposition(&p.d3d, &description, None)?;
            let cible = p.composition.CreateTargetForHwnd(fenetre, true)?;
            let visuel = p.composition.CreateVisual()?;
            visuel.SetContent(&chaine)?;
            cible.SetRoot(&visuel)?;
            p.composition.Commit()?;
            Ok(Toile { fenetre, chaine, _cible: cible, _visuel: visuel, image: None, w, h, visible: false })
        }
    }

    /// Pose la toile à l'écran (pixels physiques), à la taille dite.
    pub fn poser(&mut self, x: i32, y: i32, w: u32, h: u32) -> Result<()> {
        unsafe {
            if (w, h) != (self.w, self.h) {
                self.image = None; // la chaîne ne change de taille que sans personne qui la tienne
                self.p_redimensionner(w, h)?;
            }
            SetWindowPos(self.fenetre, None, x, y, w as i32, h as i32, SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_SHOWWINDOW)?;
        }
        self.visible = true;
        Ok(())
    }

    unsafe fn p_redimensionner(&mut self, w: u32, h: u32) -> Result<()> {
        self.chaine.ResizeBuffers(2, w, h, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SWAP_CHAIN_FLAG(0))?;
        self.w = w;
        self.h = h;
        Ok(())
    }

    pub fn cacher(&mut self) {
        if self.visible {
            unsafe {
                let _ = ShowWindow(self.fenetre, SW_HIDE);
            }
            self.visible = false;
        }
    }

    /// Dessine une image : `dessin` reçoit le contexte Direct2D, déjà pointé sur la toile.
    pub fn peindre(&mut self, p: &Peintre, dessin: impl FnOnce(&ID2D1DeviceContext)) -> Result<()> {
        self.peindre_zone(p, None, dessin)
    }

    /// Dessine une image dont seule `zone` a changé (le reste est celui de l'image d'avant) :
    /// le compositeur ne reprend que cette zone.
    pub fn peindre_zone(&mut self, p: &Peintre, zone: Option<RECT>, dessin: impl FnOnce(&ID2D1DeviceContext)) -> Result<()> {
        unsafe {
            if self.image.is_none() {
                let surface: IDXGISurface = self.chaine.GetBuffer(0)?;
                let proprietes = D2D1_BITMAP_PROPERTIES1 {
                    pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
                    dpiX: 96.0,
                    dpiY: 96.0,
                    bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                    ..Default::default()
                };
                self.image = Some(p.dc.CreateBitmapFromDxgiSurface(&surface, Some(&proprietes))?);
            }
            let cible: ID2D1Image = self.image.as_ref().unwrap().cast()?;
            p.dc.SetTarget(&cible);
            p.dc.BeginDraw();
            dessin(&p.dc);
            let fin = p.dc.EndDraw(None, None);
            p.dc.SetTarget(None::<&ID2D1Image>);
            // l'image de la cible se garde d'une image à l'autre : Direct3D 11 fait tourner les
            // tampons sous elle ; elle n'est refaite qu'au changement de taille
            fin?;
            match zone {
                None => self.chaine.Present(0, DXGI_PRESENT(0)).ok(),
                Some(mut z) => {
                    let parametres = DXGI_PRESENT_PARAMETERS { DirtyRectsCount: 1, pDirtyRects: &mut z, ..Default::default() };
                    self.chaine.Present1(0, DXGI_PRESENT(0), &parametres).ok()
                }
            }
        }
    }
}

impl Drop for Toile {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.fenetre);
        }
    }
}
