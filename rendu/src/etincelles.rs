//! La gerbe d'étincelles : une mire de finesse, dessinée au vrai pixel de l'écran.
//!
//! Une forge de nuit. Le forgeron frappe l'enclume, et chaque coup lance une gerbe d'étincelles
//! qui partent en arcs, traînent derrière elles leur mouvement, refroidissent du blanc au rouge
//! et rebondissent au sol. Des braises montent du foyer, dont la lueur palpite ; la lune et les
//! étoiles veillent sur des montagnes. Tout se tient en unités de la hauteur du bandeau : la scène
//! est la même à toute taille, et son mouvement ne dépend pas de la cadence des images.

use windows::core::{w, Result};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_COLOR_F, D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_END_CLOSED, D2D1_GRADIENT_STOP, D2D_RECT_F,
};
use windows::Win32::Graphics::Direct2D::{
    ID2D1DeviceContext, ID2D1Factory1, ID2D1GradientStopCollection, ID2D1LinearGradientBrush, ID2D1PathGeometry1, ID2D1RadialGradientBrush, ID2D1RenderTarget, ID2D1Factory,
    ID2D1SolidColorBrush, ID2D1StrokeStyle, D2D1_ANTIALIAS_MODE_ALIASED, D2D1_CAP_STYLE_ROUND, D2D1_DRAW_TEXT_OPTIONS_NONE,
    D2D1_ELLIPSE, D2D1_EXTEND_MODE_CLAMP, D2D1_GAMMA_2_2, D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES, D2D1_PRIMITIVE_BLEND_ADD,
    D2D1_PRIMITIVE_BLEND_SOURCE_OVER, D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES, D2D1_STROKE_STYLE_PROPERTIES,
};
use windows::Win32::Graphics::DirectWrite::{
    IDWriteFactory, IDWriteTextFormat, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_ITALIC, DWRITE_FONT_STYLE_NORMAL,
    DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_WEIGHT_SEMI_LIGHT, DWRITE_MEASURING_MODE_NATURAL, DWRITE_TEXT_ALIGNMENT_TRAILING,
};
use std::cell::RefCell;

use windows::core::Interface;
use windows::Win32::Graphics::Direct2D::Common::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COMPOSITE_MODE_SOURCE_COPY, D2D1_COMPOSITE_MODE_SOURCE_OVER, D2D1_PIXEL_FORMAT, D2D_SIZE_U};
use windows::Win32::Graphics::Direct2D::{ID2D1Bitmap1, ID2D1Image, D2D1_BITMAP_OPTIONS_TARGET, D2D1_BITMAP_PROPERTIES1, D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR};
use windows::Win32::Graphics::DirectWrite::IDWriteTextLayout;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows_numerics::{Matrix3x2, Vector2};

const PAS: f32 = 1.0 / 120.0; // le pas de la physique, quelle que soit la cadence des images
const TRAINE: usize = 10; // positions gardées par étincelle : un douzième de seconde de mouvement
const PAS_TRAINE: usize = 3; // la traîne se dessine d'un trait toutes les trois positions
const PERIODE: f32 = 1.45; // un coup de marteau
const IMPACT: f32 = 0.92; // l'instant du coup, dans la période

fn c(r: f32, g: f32, b: f32, a: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F { r, g, b, a }
}
fn v(x: f32, y: f32) -> Vector2 {
    Vector2 { X: x, Y: y }
}
fn rect(l: f32, t: f32, r: f32, b: f32) -> D2D_RECT_F {
    D2D_RECT_F { left: l, top: t, right: r, bottom: b }
}
fn lisse(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Un hasard reproductible (xorshift).
struct Hasard(u64);
impl Hasard {
    fn f(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
    fn entre(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
}

struct Etincelle {
    p: Vector2,
    vit: Vector2,
    age: f32,
    vie: f32,
    rebonds: u8,
    traine: [Vector2; TRAINE],
    n: usize, // positions de la traîne déjà remplies
}

struct Braise {
    p: Vector2,
    vy: f32,
    phase: f32,
    age: f32,
    vie: f32,
}

struct Etoile {
    p: Vector2,
    taille: f32,
    phase: f32,
    vitesse: f32,
}

/// Ce qui ne dépend que de la taille du bandeau : les formes, les positions.
struct Decor {
    w: f32,
    h: f32,
    sol: f32,
    foyer: Vector2,
    impact: Vector2,
    epaule: Vector2,
    bras: f32, // de l'épaule à la tête du marteau
    angle_impact: f32,
    enclume: ID2D1PathGeometry1,
    forgeron: ID2D1PathGeometry1,
    hotte: ID2D1PathGeometry1,
    montagnes: [ID2D1PathGeometry1; 2],
    pierres: Vec<(Vector2, Vector2)>,
    charbons: Vec<(Vector2, f32, f32)>, // centre, rayon, phase
    etoiles: Vec<Etoile>,
    texte: IDWriteTextFormat,
    titre: IDWriteTextFormat,
    ecriture: IDWriteFactory,
    derriere: Option<ID2D1Bitmap1>,
    devant: Option<ID2D1Bitmap1>,
}

/// Les pinceaux, liés au contexte de dessin.
struct Pinceaux {
    uni: ID2D1SolidColorBrush,
    ciel: ID2D1LinearGradientBrush,
    lueur: ID2D1RadialGradientBrush,
    eclat: ID2D1RadialGradientBrush,
    halo: ID2D1RadialGradientBrush,
    rond: ID2D1StrokeStyle,
}

pub struct Gerbe {
    hasard: Hasard,
    t: f32,
    reste: f32, // le temps pas encore donné à la physique
    etincelles: Vec<Etincelle>,
    braises: Vec<Braise>,
    a_braise: f32,
    eclat: f32,
    decor: Option<Decor>,
    pinceaux: Option<Pinceaux>,
    /// ce que coûte une image, en ms (moyenne glissante), et la cadence : pour l'affichage
    pub cout_ms: f32,
    pub cadence_ms: u32,
    pub cellule: (i32, i32),
    mesure: RefCell<Option<(u32, IDWriteTextLayout)>>,
}

impl Gerbe {
    pub fn new(graine: u64) -> Self {
        Gerbe {
            hasard: Hasard(graine | 1),
            t: 0.0,
            reste: 0.0,
            etincelles: Vec::with_capacity(256),
            braises: Vec::with_capacity(64),
            a_braise: 0.0,
            eclat: 0.0,
            decor: None,
            pinceaux: None,
            cout_ms: 0.0,
            cadence_ms: 150,
            cellule: (0, 0),
            mesure: RefCell::new(None),
        }
    }

    /// Fait avancer la scène de `dt` secondes, par petits pas.
    pub fn avancer(&mut self, dt: f32) {
        let Some(d) = self.decor.as_ref() else { return };
        let (h, sol) = (d.h, d.sol);
        let (impact, foyer) = (d.impact, d.foyer);
        self.reste += dt.min(0.5);
        while self.reste >= PAS {
            self.reste -= PAS;
            let avant = (self.t / PERIODE).fract();
            self.t += PAS;
            let apres = (self.t / PERIODE).fract();
            if avant < IMPACT && apres >= IMPACT {
                self.frapper(impact, h);
            }
            self.eclat *= (-PAS * 9.0).exp();
            // les étincelles : la pesanteur, l'air, le sol
            let g = 2.9 * h;
            let air = (-PAS * 0.9).exp();
            for e in &mut self.etincelles {
                e.age += PAS;
                e.vit.Y += g * PAS;
                e.vit.X *= air;
                e.vit.Y *= air;
                e.p.X += e.vit.X * PAS;
                e.p.Y += e.vit.Y * PAS;
                if e.p.Y > sol && e.vit.Y > 0.0 {
                    e.p.Y = sol;
                    e.vit.Y *= -0.32;
                    e.vit.X *= 0.55;
                    e.rebonds += 1;
                    if e.rebonds > 1 {
                        e.vie = e.vie.min(e.age + 0.12);
                    }
                }
                e.traine.copy_within(0..TRAINE - 1, 1);
                e.traine[0] = e.p;
                e.n = (e.n + 1).min(TRAINE);
            }
            self.etincelles.retain(|e| e.age < e.vie);
            // les braises : elles montent du foyer, en ondoyant
            self.a_braise += PAS * 5.0;
            while self.a_braise >= 1.0 {
                self.a_braise -= 1.0;
                let x = foyer.X + self.hasard.entre(-0.12, 0.12) * h;
                self.braises.push(Braise {
                    p: v(x, foyer.Y - 0.02 * h),
                    vy: -self.hasard.entre(0.10, 0.26) * h,
                    phase: self.hasard.entre(0.0, 6.28),
                    age: 0.0,
                    vie: self.hasard.entre(1.6, 3.4),
                });
            }
            for b in &mut self.braises {
                b.age += PAS;
                b.p.Y += b.vy * PAS;
                b.p.X += (self.t * 2.3 + b.phase).sin() * 0.05 * h * PAS;
            }
            self.braises.retain(|b| b.age < b.vie && b.p.Y > 0.0);
        }
    }

    fn frapper(&mut self, impact: Vector2, h: f32) {
        self.eclat = 1.0;
        let n = 40 + (self.hasard.f() * 24.0) as usize;
        for _ in 0..n {
            // un demi-cercle vers le haut, plus nourri sur les côtés
            let angle = self.hasard.entre(-std::f32::consts::PI + 0.12, -0.12);
            let vitesse = self.hasard.entre(0.5, 2.4) * h * (0.6 + 0.4 * angle.cos().abs());
            let p = v(impact.X + self.hasard.entre(-0.02, 0.02) * h, impact.Y);
            self.etincelles.push(Etincelle {
                p,
                vit: v(angle.cos() * vitesse, angle.sin() * vitesse),
                age: 0.0,
                vie: self.hasard.entre(0.45, 1.35),
                rebonds: 0,
                traine: [p; TRAINE],
                n: 1,
            });
        }
    }

    /// L'angle du bras : levé lentement, tenu, abattu d'un coup, posé sur l'enclume.
    fn angle_bras(&self, d: &Decor) -> f32 {
        let phi = (self.t / PERIODE).fract();
        let leve = if phi < 0.62 {
            lisse(phi / 0.62)
        } else if phi < 0.84 {
            1.0
        } else if phi < IMPACT {
            let u = (phi - 0.84) / (IMPACT - 0.84);
            1.0 - u * u
        } else {
            0.04 * ((phi - IMPACT) / (1.0 - IMPACT) * std::f32::consts::PI).sin() // le rebond
        };
        d.angle_impact + leve * 1.95
    }

    /// Prépare le décor et les pinceaux pour cette taille.
    pub fn preparer(&mut self, dc: &ID2D1DeviceContext, d2d: &ID2D1Factory1, ecriture: &IDWriteFactory, w: u32, h: u32) -> Result<()> {
        if self.decor.as_ref().is_some_and(|d| d.w == w as f32 && d.h == h as f32) {
            return Ok(());
        }
        let (w, h) = (w as f32, h as f32);
        let sol = h * 0.86;
        let foyer = v(0.42 * h, sol - 0.15 * h);
        let enclume_x = 1.22 * h;
        let dessus = sol - 0.24 * h;
        let impact = v(enclume_x - 0.02 * h, dessus);
        let epaule = v(enclume_x + 0.31 * h, sol - 0.48 * h);
        let demi_tete = 0.075 * h;
        let dx = impact.X - epaule.X;
        let dy = impact.Y - epaule.Y;
        let bras = ((dx * dx + dy * dy) - demi_tete * demi_tete).max(0.0).sqrt();
        // la face qui frappe est (bras, -demi_tete) dans le repère du bras : on la tourne sur l'impact
        let angle_impact = dy.atan2(dx) - (-demi_tete).atan2(bras);
        let u = h;
        unsafe {
            let figure = |points: &[Vector2]| -> Result<ID2D1PathGeometry1> {
                let g = d2d.CreatePathGeometry()?;
                let s = g.Open()?;
                s.BeginFigure(points[0], D2D1_FIGURE_BEGIN_FILLED);
                s.AddLines(&points[1..]);
                s.EndFigure(D2D1_FIGURE_END_CLOSED);
                s.Close()?;
                Ok(g)
            };
            let x = enclume_x;
            let enclume = figure(&[
                v(x - 0.20 * u, dessus),
                v(x + 0.17 * u, dessus),
                v(x + 0.17 * u, dessus + 0.055 * u),
                v(x + 0.08 * u, dessus + 0.085 * u),
                v(x + 0.065 * u, dessus + 0.16 * u),
                v(x + 0.15 * u, sol - 0.02 * u),
                v(x + 0.16 * u, sol),
                v(x - 0.16 * u, sol),
                v(x - 0.15 * u, sol - 0.02 * u),
                v(x - 0.065 * u, dessus + 0.16 * u),
                v(x - 0.08 * u, dessus + 0.085 * u),
                v(x - 0.19 * u, dessus + 0.065 * u),
                v(x - 0.26 * u, dessus + 0.04 * u),
                v(x - 0.315 * u, dessus + 0.012 * u),
                v(x - 0.27 * u, dessus + 0.003 * u),
            ])?;
            let (sx, sy) = (epaule.X, epaule.Y);
            let hanche = sol - 0.27 * u;
            let forgeron = figure(&[
                v(sx - 0.075 * u, sy - 0.01 * u),
                v(sx + 0.05 * u, sy - 0.02 * u),
                v(sx + 0.075 * u, sy + 0.03 * u),
                v(sx + 0.07 * u, hanche),
                v(sx + 0.075 * u, hanche + 0.02 * u),
                v(sx + 0.10 * u, sol - 0.02 * u),
                v(sx + 0.115 * u, sol),
                v(sx + 0.05 * u, sol),
                v(sx + 0.02 * u, hanche + 0.06 * u),
                v(sx - 0.01 * u, sol),
                v(sx - 0.085 * u, sol),
                v(sx - 0.06 * u, sol - 0.02 * u),
                v(sx - 0.055 * u, hanche + 0.02 * u),
                v(sx - 0.065 * u, hanche),
                v(sx - 0.09 * u, sy + 0.04 * u),
            ])?;
            let fx = foyer.X;
            let hotte = figure(&[
                v(fx - 0.34 * u, sol - 0.36 * u),
                v(fx - 0.14 * u, 0.0),
                v(fx + 0.14 * u, 0.0),
                v(fx + 0.34 * u, sol - 0.36 * u),
            ])?;
            let mut hasard = Hasard(0x9e3779b97f4a7c15);
            let mut chaine = |base: f32, ecart: f32, pas: f32| -> Result<ID2D1PathGeometry1> {
                let mut points = vec![v(1.55 * u, sol)];
                let mut x = 1.55 * u;
                while x < w + pas {
                    points.push(v(x, sol - base * u - hasard.entre(0.0, ecart) * u));
                    x += pas * hasard.entre(0.5, 1.4);
                }
                points.push(v(w + pas, sol));
                figure(&points)
            };
            let montagnes = [chaine(0.30, 0.22, 0.16 * u)?, chaine(0.14, 0.10, 0.07 * u)?];
            // les joints des pierres du foyer : des traits d'un pixel
            let mut pierres = Vec::new();
            let (g, d, haut) = (fx - 0.31 * u, fx + 0.31 * u, sol - 0.36 * u);
            let mut y = haut + 0.06 * u;
            let mut rang = 0;
            while y < sol {
                pierres.push((v(g, y), v(d, y)));
                let decale = if rang % 2 == 0 { 0.0 } else { 0.07 * u };
                let mut x = g + 0.05 * u + decale;
                while x < d {
                    pierres.push((v(x, y - 0.06 * u), v(x, y)));
                    x += 0.14 * u;
                }
                y += 0.06 * u;
                rang += 1;
            }
            let charbons = (0..11)
                .map(|i| {
                    let x = fx - 0.15 * u + i as f32 * 0.03 * u;
                    (v(x, sol - 0.115 * u - (i % 3) as f32 * 0.008 * u), (0.018 + (i % 4) as f32 * 0.004) * u, i as f32 * 1.7)
                })
                .collect();
            let etoiles = (0..90)
                .map(|_| Etoile {
                    p: v(hasard.entre(1.6 * u, w), hasard.entre(0.0, sol - 0.48 * u)),
                    taille: hasard.entre(0.6, 1.6),
                    phase: hasard.entre(0.0, 6.28),
                    vitesse: hasard.entre(0.6, 2.6),
                })
                .collect();
            let taille_texte = (0.052 * h).max(11.0);
            let texte = ecriture.CreateTextFormat(w!("Segoe UI"), None, DWRITE_FONT_WEIGHT_SEMI_LIGHT, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL, taille_texte, w!("fr-fr"))?;
            texte.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_TRAILING)?;
            let titre = ecriture.CreateTextFormat(w!("Georgia"), None, DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_STYLE_ITALIC, DWRITE_FONT_STRETCH_NORMAL, (0.09 * h).max(14.0), w!("fr-fr"))?;
            titre.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_TRAILING)?;
            self.decor = Some(Decor {
                w,
                h,
                sol,
                foyer,
                impact,
                epaule,
                bras,
                angle_impact,
                enclume,
                forgeron,
                hotte,
                montagnes,
                pierres,
                charbons,
                etoiles,
                texte,
                titre,
                ecriture: ecriture.clone(),
                derriere: None,
                devant: None,
            });
            if self.pinceaux.is_none() {
                let degrade = |arrets: &[(f32, D2D1_COLOR_F)]| -> Result<ID2D1GradientStopCollection> {
                    let arrets: Vec<_> = arrets.iter().map(|&(position, color)| D2D1_GRADIENT_STOP { position, color }).collect();
                    // la version de Direct2D 1.0 : quelques arrêts, en gamma 2.2
                    let rt: &ID2D1RenderTarget = dc;
                    rt.CreateGradientStopCollection(&arrets, D2D1_GAMMA_2_2, D2D1_EXTEND_MODE_CLAMP)
                };
                let radial = |arrets: &[(f32, D2D1_COLOR_F)]| -> Result<ID2D1RadialGradientBrush> {
                    let p = D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES { center: v(0.0, 0.0), gradientOriginOffset: v(0.0, 0.0), radiusX: 1.0, radiusY: 1.0 };
                    dc.CreateRadialGradientBrush(&p, None, &degrade(arrets)?)
                };
                self.pinceaux = Some(Pinceaux {
                    uni: dc.CreateSolidColorBrush(&c(1.0, 1.0, 1.0, 1.0), None)?,
                    ciel: dc.CreateLinearGradientBrush(
                        &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES { startPoint: v(0.0, 0.0), endPoint: v(0.0, 1.0) },
                        None,
                        &degrade(&[(0.0, c(0.015, 0.02, 0.07, 1.0)), (1.0, c(0.09, 0.055, 0.09, 1.0))])?,
                    )?,
                    lueur: radial(&[(0.0, c(1.0, 0.42, 0.10, 0.55)), (0.35, c(0.85, 0.22, 0.05, 0.22)), (1.0, c(0.5, 0.1, 0.0, 0.0))])?,
                    eclat: radial(&[(0.0, c(1.0, 0.95, 0.8, 0.9)), (0.25, c(1.0, 0.6, 0.2, 0.35)), (1.0, c(1.0, 0.3, 0.0, 0.0))])?,
                    halo: radial(&[(0.0, c(0.75, 0.82, 1.0, 0.30)), (1.0, c(0.6, 0.7, 1.0, 0.0))])?,
                    rond: (d2d as &ID2D1Factory).CreateStrokeStyle(
                        &D2D1_STROKE_STYLE_PROPERTIES { startCap: D2D1_CAP_STYLE_ROUND, endCap: D2D1_CAP_STYLE_ROUND, ..Default::default() },
                        None,
                    )?,
                });
            }
            let (derriere, devant) = self.peindre_decor(dc, self.decor.as_ref().unwrap(), self.pinceaux.as_ref().unwrap())?;
            let d = self.decor.as_mut().unwrap();
            d.derriere = Some(derriere);
            d.devant = Some(devant);
            *self.mesure.borrow_mut() = None;
        }
        Ok(())
    }

    /// Le décor qui ne bouge pas, peint une fois par taille : derrière (opaque), et devant le feu
    /// (l'enclume et le forgeron, à contre-jour, sur fond transparent).
    unsafe fn peindre_decor(&self, dc: &ID2D1DeviceContext, d: &Decor, p: &Pinceaux) -> Result<(ID2D1Bitmap1, ID2D1Bitmap1)> {
        let (w, h, u) = (d.w, d.h, d.h);
        let uni = &p.uni;
        let proprietes = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
            dpiX: 96.0,
            dpiY: 96.0,
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET,
            ..Default::default()
        };
        let taille = D2D_SIZE_U { width: w as u32, height: h as u32 };
        let derriere = dc.CreateBitmap(taille, None, 0, &proprietes)?;
        let devant = dc.CreateBitmap(taille, None, 0, &proprietes)?;
        dc.SetTarget(&derriere.cast::<ID2D1Image>()?);
        dc.BeginDraw();
        dc.SetTransform(&Matrix3x2::identity());
        // le ciel, du bleu de nuit au brun près du sol
        p.ciel.SetEndPoint(v(0.0, d.sol));
        dc.FillRectangle(&rect(0.0, 0.0, w, h), &p.ciel);
        // la lune, son halo, ses mers
        let lune = v(w - 0.95 * u, 0.27 * u);
        p.halo.SetCenter(lune);
        p.halo.SetRadiusX(0.42 * u);
        p.halo.SetRadiusY(0.42 * u);
        dc.FillRectangle(&rect(lune.X - 0.42 * u, lune.Y - 0.42 * u, lune.X + 0.42 * u, lune.Y + 0.42 * u), &p.halo);
        uni.SetColor(&c(0.93, 0.92, 0.84, 1.0));
        dc.FillEllipse(&D2D1_ELLIPSE { point: lune, radiusX: 0.095 * u, radiusY: 0.095 * u }, uni);
        uni.SetColor(&c(0.78, 0.77, 0.70, 1.0));
        for (dx, dy, r) in [(-0.03, -0.02, 0.022), (0.03, 0.025, 0.016), (0.012, -0.045, 0.011), (-0.045, 0.035, 0.009)] {
            dc.FillEllipse(&D2D1_ELLIPSE { point: v(lune.X + dx * u, lune.Y + dy * u), radiusX: r * u, radiusY: r * u }, uni);
        }
        // les montagnes, la lointaine puis la proche
        uni.SetColor(&c(0.055, 0.055, 0.10, 1.0));
        dc.FillGeometry(&d.montagnes[0], uni, None);
        uni.SetColor(&c(0.035, 0.03, 0.06, 1.0));
        dc.FillGeometry(&d.montagnes[1], uni, None);
        // le sol
        uni.SetColor(&c(0.07, 0.05, 0.045, 1.0));
        dc.FillRectangle(&rect(0.0, d.sol, w, h), uni);
        // la hotte et le foyer, ses pierres jointées d'un pixel, sa bouche
        uni.SetColor(&c(0.10, 0.075, 0.07, 1.0));
        dc.FillGeometry(&d.hotte, uni, None);
        let (fx, haut) = (d.foyer.X, d.sol - 0.36 * u);
        uni.SetColor(&c(0.20, 0.15, 0.13, 1.0));
        dc.FillRectangle(&rect(fx - 0.31 * u, haut, fx + 0.31 * u, d.sol), uni);
        uni.SetColor(&c(0.08, 0.055, 0.05, 1.0));
        for (a, b) in &d.pierres {
            dc.DrawLine(*a, *b, uni, 1.0, None);
        }
        uni.SetColor(&c(0.12, 0.03, 0.01, 1.0));
        dc.FillRectangle(&rect(fx - 0.18 * u, d.sol - 0.25 * u, fx + 0.18 * u, d.sol - 0.08 * u), uni);
        // le titre, net au pixel
        uni.SetColor(&c(0.82, 0.78, 0.68, 0.85));
        let titre: Vec<u16> = "la gerbe d’étincelles".encode_utf16().collect();
        dc.DrawText(&titre, &d.titre, &rect(0.0, 0.04 * u, w - 0.25 * u, 0.2 * u), uni, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        dc.EndDraw(None, None)?;
        dc.SetTarget(&devant.cast::<ID2D1Image>()?);
        dc.BeginDraw();
        dc.Clear(Some(&c(0.0, 0.0, 0.0, 0.0)));
        // l'enclume, son arête éclairée
        uni.SetColor(&c(0.11, 0.11, 0.13, 1.0));
        dc.FillGeometry(&d.enclume, uni, None);
        uni.SetColor(&c(0.85, 0.45, 0.2, 0.6));
        dc.DrawGeometry(&d.enclume, uni, 1.0, None);
        // le forgeron, bordé par le feu
        uni.SetColor(&c(0.025, 0.02, 0.025, 1.0));
        dc.FillGeometry(&d.forgeron, uni, None);
        dc.FillEllipse(&D2D1_ELLIPSE { point: v(d.epaule.X + 0.02 * u, d.epaule.Y - 0.085 * u), radiusX: 0.05 * u, radiusY: 0.058 * u }, uni);
        uni.SetColor(&c(0.9, 0.42, 0.15, 0.4));
        dc.DrawGeometry(&d.forgeron, uni, 1.0, None);
        dc.EndDraw(None, None)?;
        dc.SetTarget(None::<&ID2D1Image>);
        Ok((derriere, devant))
    }

    /// Dessine l'image, puis rend transparentes les cellules repères (`trous`).
    pub fn dessiner(&self, dc: &ID2D1DeviceContext, trous: &[D2D_RECT_F]) {
        let (Some(d), Some(p)) = (self.decor.as_ref(), self.pinceaux.as_ref()) else { return };
        let (Some(derriere), Some(devant)) = (d.derriere.as_ref(), d.devant.as_ref()) else { return };
        let (w, h, u) = (d.w, d.h, d.h);
        let t = self.t;
        let uni = &p.uni;
        let feu = 0.78 + 0.12 * (t * 7.3).sin() + 0.07 * (t * 13.1 + 1.0).sin() + 0.04 * (t * 29.0).sin();
        unsafe {
            dc.SetTransform(&Matrix3x2::identity());
            // le décor de derrière, recopié tel quel par la carte graphique
            dc.DrawImage(derriere, None, None, D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR, D2D1_COMPOSITE_MODE_SOURCE_COPY);
            // les étoiles, qui scintillent
            for e in &d.etoiles {
                let a = 0.35 + 0.65 * (0.5 + 0.5 * (t * e.vitesse + e.phase).sin());
                uni.SetColor(&c(0.85, 0.88, 1.0, a));
                let r = e.taille * 0.5;
                dc.FillRectangle(&rect(e.p.X - r, e.p.Y - r, e.p.X + r, e.p.Y + r), uni);
            }
            // les charbons qui respirent
            for &(centre, r, phase) in &d.charbons {
                let k = (0.55 + 0.45 * (t * 3.1 + phase).sin() * (t * 1.7 + phase * 0.5).cos()).clamp(0.0, 1.0) * feu;
                uni.SetColor(&c(0.45 + 0.55 * k, 0.07 + 0.55 * k * k, 0.02 + 0.2 * k * k * k, 1.0));
                dc.FillEllipse(&D2D1_ELLIPSE { point: centre, radiusX: r * 1.3, radiusY: r }, uni);
            }
            // la lueur du feu, ajoutée à ce qu'elle éclaire
            dc.SetPrimitiveBlend(D2D1_PRIMITIVE_BLEND_ADD);
            let fx = d.foyer.X;
            let r = 1.05 * u * (0.92 + 0.08 * feu);
            p.lueur.SetCenter(v(fx, d.sol - 0.16 * u));
            p.lueur.SetRadiusX(r);
            p.lueur.SetRadiusY(r * 0.8);
            p.lueur.SetOpacity(feu.min(1.0));
            dc.FillRectangle(&rect(fx - r, d.sol - 0.16 * u - r, fx + r, h), &p.lueur);
            dc.SetPrimitiveBlend(D2D1_PRIMITIVE_BLEND_SOURCE_OVER);
            // l'enclume et le forgeron, à contre-jour
            dc.DrawImage(devant, None, None, D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR, D2D1_COMPOSITE_MODE_SOURCE_OVER);
            // le bras et le marteau, tournés autour de l'épaule
            let a = self.angle_bras(d);
            let (s, co) = a.sin_cos();
            let (ex, ey) = (d.epaule.X, d.epaule.Y);
            dc.SetTransform(&Matrix3x2 { M11: co, M12: s, M21: -s, M22: co, M31: ex - ex * co + ey * s, M32: ey - ex * s - ey * co });
            uni.SetColor(&c(0.03, 0.025, 0.03, 1.0));
            dc.DrawLine(v(ex, ey), v(ex + 0.22 * u, ey), uni, 0.05 * u, &p.rond);
            uni.SetColor(&c(0.30, 0.18, 0.10, 1.0));
            dc.DrawLine(v(ex + 0.2 * u, ey), v(ex + d.bras, ey), uni, 0.02 * u, &p.rond);
            let (hx, dt) = (ex + d.bras, 0.075 * u);
            uni.SetColor(&c(0.24, 0.24, 0.27, 1.0));
            dc.FillRectangle(&rect(hx - 0.035 * u, ey - dt, hx + 0.035 * u, ey + dt), uni);
            uni.SetColor(&c(0.95, 0.6, 0.3, 0.7));
            dc.FillRectangle(&rect(hx - 0.035 * u, ey - dt, hx + 0.035 * u, ey - dt + 1.0), uni);
            dc.SetTransform(&Matrix3x2::identity());
            // l'éclat du coup, puis les étincelles et les braises, toutes ajoutées à la lumière
            dc.SetPrimitiveBlend(D2D1_PRIMITIVE_BLEND_ADD);
            if self.eclat > 0.02 {
                let r = 0.5 * u * (0.6 + 0.4 * self.eclat);
                p.eclat.SetCenter(d.impact);
                p.eclat.SetRadiusX(r);
                p.eclat.SetRadiusY(r);
                p.eclat.SetOpacity(self.eclat);
                dc.FillRectangle(&rect(d.impact.X - r, d.impact.Y - r, d.impact.X + r, d.impact.Y + r), &p.eclat);
            }
            let epais = (u / 260.0).clamp(1.0, 2.2);
            for e in &self.etincelles {
                let k = e.age / e.vie; // 0 : blanche, 1 : éteinte
                let (r, g, b) = if k < 0.25 {
                    (1.0, 1.0 - 0.3 * k / 0.25, 0.85 - 0.6 * k / 0.25)
                } else {
                    let m = (k - 0.25) / 0.75;
                    (1.0 - 0.45 * m, 0.7 - 0.6 * m, 0.25 - 0.23 * m)
                };
                let a = (1.0 - k).powf(1.4);
                // la traîne en trois traits, de plus en plus pâles
                let mut i = 0;
                while i + PAS_TRAINE < e.n {
                    let queue = 1.0 - (i + PAS_TRAINE) as f32 / TRAINE as f32;
                    uni.SetColor(&c(r, g, b, a * queue));
                    dc.DrawLine(e.traine[i], e.traine[i + PAS_TRAINE], uni, epais, None);
                    i += PAS_TRAINE;
                }
                uni.SetColor(&c(1.0, 0.95, 0.8, a));
                dc.FillRectangle(&rect(e.p.X - epais * 0.5, e.p.Y - epais * 0.5, e.p.X + epais * 0.5, e.p.Y + epais * 0.5), uni);
            }
            for b in &self.braises {
                let k = b.age / b.vie;
                let a = (k * 6.0).min(1.0) * (1.0 - k) * (0.6 + 0.4 * (t * 9.0 + b.phase).sin());
                uni.SetColor(&c(1.0, 0.45, 0.12, a));
                let r = epais * 0.75;
                dc.FillEllipse(&D2D1_ELLIPSE { point: b.p, radiusX: r, radiusY: r }, uni);
            }
            dc.SetPrimitiveBlend(D2D1_PRIMITIVE_BLEND_SOURCE_OVER);
            // la mesure, remise en page une fois par seconde
            let seconde = t as u32;
            let mut mesure = self.mesure.borrow_mut();
            if mesure.as_ref().is_none_or(|(s, _)| *s != seconde) {
                let texte: Vec<u16> = format!(
                    "rendu natif · {} × {} px · cellule {} × {} · {} étincelles · {:.2} ms par image · une image toutes les {} ms",
                    w as u32, h as u32, self.cellule.0, self.cellule.1, self.etincelles.len(), self.cout_ms, self.cadence_ms
                )
                .encode_utf16()
                .collect();
                *mesure = d.ecriture.CreateTextLayout(&texte, &d.texte, w - 0.25 * u, 0.085 * u).ok().map(|l| (seconde, l));
            }
            if let Some((_, l)) = mesure.as_ref() {
                uni.SetColor(&c(0.62, 0.62, 0.70, 0.9));
                dc.DrawTextLayout(v(0.0, h - 0.085 * u), l, uni, D2D1_DRAW_TEXT_OPTIONS_NONE);
            }
            // les trous : les repères du terminal restent visibles à travers
            for t in trous {
                dc.PushAxisAlignedClip(t, D2D1_ANTIALIAS_MODE_ALIASED);
                dc.Clear(Some(&c(0.0, 0.0, 0.0, 0.0)));
                dc.PopAxisAlignedClip();
            }
        }
    }
}
