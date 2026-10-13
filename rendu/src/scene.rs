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

impl Image<'_> {
    /// Remplit un rectangle (borné à l'image) d'une couleur.
    pub fn remplir(&mut self, r: RECT, couleur: u32) {
        let (x0, x1) = (r.left.max(0) as usize, (r.right.max(0) as usize).min(self.w as usize));
        let (y0, y1) = (r.top.max(0) as usize, (r.bottom.max(0) as usize).min(self.h as usize));
        if x0 >= x1 {
            return;
        }
        for y in y0..y1 {
            let debut = y * self.w as usize;
            self.pixels[debut + x0..debut + x1].fill(couleur);
        }
    }
}

pub trait Scene {
    /// Avance de `dt` secondes et peint ce qui a changé ; `neuve` : l'image est à peindre en
    /// entier. Rend la zone peinte, ou `None` si rien n'a changé.
    fn peindre(&mut self, image: &mut Image, dt: f32, neuve: bool) -> Option<RECT>;
}

/// Le plus petit rectangle qui couvre les deux.
pub fn union(a: RECT, b: RECT) -> RECT {
    RECT { left: a.left.min(b.left), top: a.top.min(b.top), right: a.right.max(b.right), bottom: a.bottom.max(b.bottom) }
}

/// La mire : un carré qui glisse sur un fond sombre. Ce que coûte l'appli seule, au banc, et ce
/// qu'elle montre en attendant la forge.
pub struct Mire {
    t: f32,
    avant: Option<RECT>,
}

impl Mire {
    pub fn new() -> Self {
        Mire { t: 0.0, avant: None }
    }
}

const FOND: u32 = 0xFF0D0D14;
const CARRE: u32 = 0xFFE68033;

impl Scene for Mire {
    fn peindre(&mut self, image: &mut Image, dt: f32, neuve: bool) -> Option<RECT> {
        self.t += dt;
        let course = (image.w as f32 - 20.0).max(1.0);
        let x = (self.t * 27.0) % course;
        let y = image.h as i32 / 2 - 10;
        let carre = RECT { left: x as i32, top: y, right: x as i32 + 20, bottom: y + 20 };
        let zone = match (neuve, self.avant) {
            (false, Some(avant)) => {
                image.remplir(avant, FOND);
                union(avant, carre)
            }
            _ => {
                let tout = RECT { left: 0, top: 0, right: image.w as i32, bottom: image.h as i32 };
                image.remplir(tout, FOND);
                tout
            }
        };
        image.remplir(carre, CARRE);
        self.avant = Some(carre);
        Some(zone)
    }
}
