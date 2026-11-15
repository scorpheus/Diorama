//! Ce qu'on peint sur une toile : une scène avance d'image en image, et ne repeint que ce qui
//! a changé.

use windows::Win32::Foundation::RECT;

/// L'image d'une toile, telle qu'une scène la reçoit.
pub struct Image<'a> {
    /// ligne après ligne : 0xAARRGGBB, alpha prémultiplié
    pub pixels: &'a mut [u32],
    pub w: u32,
    pub h: u32,
    /// la taille d'une cellule du terminal, en pixels
    pub cellule: (u32, u32),
}

pub trait Scene {
    /// Avance de `dt` secondes et peint ce qui a changé ; `neuve` : l'image est à peindre en
    /// entier. Rend la zone peinte, ou `None` si rien n'a changé.
    fn peindre(&mut self, image: &mut Image, dt: f32, neuve: bool) -> Option<RECT>;
}
