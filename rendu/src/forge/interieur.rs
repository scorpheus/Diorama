//! L'intérieur de la forge médiévale-fantastique (style Poudlard et Terre du Milieu), peint en
//! pixel art stylisé `2×2` (`1 pixel d'art = 4 vrais pixels`) et détails `1×1`, sans aucun tramage
//! (dithering), sous les cellules transparentes de `0..YARD` :
//! - `F1` : murs d'appareil gothique, nervures de voûte, poutres de chêne, dalles de granit usées ;
//! - `F2` : fenêtre gothique en ogive à vitrail au plomb, volets de chêne ferré, citrouille d'Halloween ;
//! - `F3` : grand four nain de Khazad-dûm, linteau à quatre runes de feu, bûches, flammes, soufflet clouté ;
//! - `F4` : enclume runique ciselée sur billot de chêne cerclé de fer, lingot, baquet de trempe, débris,
//!          tabouret, trésor d'or et gemmes, et le canard en plastique jaune en pixel art `2×2` ;
//! - `F5` : coffre runique de chêne et d'acier sur l'enclume, scellé à la cire rouge et or ;
//! - `F6` & `E4` : astrolabe mural de laiton suivant l'heure du monde, sabliers de tâches et sablier au repos ;
//! - `F7` : arche gothique de la porte vers la cour, lanterne de fer forgé, cloche de bronze oscillante ;
//! - `F8` : quatre chandelles flottantes façon Poudlard sous la voûte (`x < 40`) ;
//! - `F9` : étagère d'alchimiste aux grimoires reliés et fioles de potions bouillonnantes ;
//! - `F10` : râtelier d'armes légendaires (*Dard*, hache naine, dague elfique, masse de cristal, *Andúril*)
//!           et bannière médiévale frangée d'or ;
//! - `E5` : chapeau pointu de sorcier étoilé (anniversaire) et chaperon d'hiver au houx (Noël) sur Scorpheus.

use std::f32::consts::TAU;

use super::accessoires::PALETTE;
use super::atelier::{ANVIL_AT, BANNER, BELL_AT, BUCKET_AT, C, COIN_STEPS, CRATE_AT, DUCK_AT, HOURGLASS_X, LANTERN, MOUTH, RACK, RACK_SLOTS, ROWS, SCRAPS, WINDOW, YARD};
use super::lumiere::{GrilleLumiere, CHANDELLES_FORGE};
use super::monde::{Activite, Arme, Banniere, Fete, Monde, Perso, Saison, Temps};
use super::pet::{self, Acteur};
use super::pixels::hasard;
use super::poses::{Caisse, Droite, Gauche, Lingot, Pose};

/// Taille d'un pixel d'art de l'intérieur en vrais pixels d'écran (`2×2`).
pub const GRAIN: usize = 2;

/// Le haut de l'appui de la fenêtre, en demi-rangées : la panthère roulée s'y pose (chat.rs).
pub(super) const V_APPUI: f32 = 9.4;

const CONTOUR: [u8; 3] = [0x1b, 0x19, 0x24];

const fn rgb(hex: u32) -> [u8; 3] {
    [((hex >> 16) & 0xff) as u8, ((hex >> 8) & 0xff) as u8, (hex & 0xff) as u8]
}

#[inline]
fn melange(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t).round() as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t).round() as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t).round() as u8,
    ]
}

#[inline]
fn echelle(c: [u8; 3], k: f32) -> [u8; 3] {
    [
        (c[0] as f32 * k).clamp(0.0, 255.0).round() as u8,
        (c[1] as f32 * k).clamp(0.0, 255.0).round() as u8,
        (c[2] as f32 * k).clamp(0.0, 255.0).round() as u8,
    ]
}

#[inline]
fn hash_f(seed: i64) -> f32 {
    hasard(seed) as f32
}

/// Mode d'éclairage d'un pixel d'art.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Mode {
    /// Pierre de mur et voûte : sombre à l'ambiance (`MUR * amb + gather_ombre`).
    Mur = 0,
    /// Sol, étagères et mobilier recevant les ombres (`amb + gather_ombre`).
    Sol = 1,
    /// Mobilier de premier plan portant ombre (`amb + gather`).
    Plein = 2,
    /// Source lumineuse propre (flamme, braise, rune ignée, vitrail rétroéclairé, potion).
    Lueur = 3,
}

#[derive(Clone, Copy)]
struct Mat {
    rgb: [u8; 3],
    mode: Mode,
}

impl Default for Mat {
    fn default() -> Self {
        Self { rgb: [72, 62, 72], mode: Mode::Mur }
    }
}

/// Canevas en pixels d'art (`aw × ah`, chacun `2×2` pixels d'écran) couvrant `0..YARD × 0..SH`.
struct Canevas<'a> {
    buf: &'a mut [Mat],
    aw: i32,
    ah: i32,
    cw: f32,
    demi: f32,
}

impl<'a> Canevas<'a> {
    #[inline]
    fn ax(&self, u: f32) -> i32 {
        ((u * self.cw) / GRAIN as f32).round() as i32
    }

    #[inline]
    fn ay(&self, v: f32) -> i32 {
        ((v * self.demi) / GRAIN as f32).round() as i32
    }

    #[inline]
    fn u_de(&self, ax: i32) -> f32 {
        ((ax as f32 + 0.5) * GRAIN as f32) / self.cw
    }

    #[inline]
    fn v_de(&self, ay: i32) -> f32 {
        ((ay as f32 + 0.5) * GRAIN as f32) / self.demi
    }

    #[inline]
    fn poser(&mut self, ax: i32, ay: i32, c: [u8; 3], mode: Mode) {
        if ax >= 0 && ax < self.aw && ay >= 0 && ay < self.ah {
            self.buf[(ay * self.aw + ax) as usize] = Mat { rgb: c, mode };
        }
    }

    fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: [u8; 3], mode: Mode) {
        let (xa, xb) = (x0.max(0), x1.min(self.aw));
        let (ya, yb) = (y0.max(0), y1.min(self.ah));
        for y in ya..yb {
            let row = (y * self.aw) as usize;
            for x in xa..xb {
                self.buf[row + x as usize] = Mat { rgb: c, mode };
            }
        }
    }

    /// Rectangle encadré d'un contour net d'un pixel d'art.
    fn boite(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, fond: [u8; 3], bord: [u8; 3], mode: Mode) {
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        self.rect(x0, y0, x1, y1, bord, mode);
        if x1 - x0 > 2 && y1 - y0 > 2 {
            self.rect(x0 + 1, y0 + 1, x1 - 1, y1 - 1, fond, mode);
        }
    }

    fn ligne(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, c: [u8; 3], mode: Mode) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.poser(x0, y0, c, mode);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn disque(&mut self, cx: i32, cy: i32, r: i32, c: [u8; 3], mode: Mode) {
        let r2 = r * r;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r2 {
                    self.poser(cx + dx, cy + dy, c, mode);
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct CleStatique {
    aw: i32,
    ah: i32,
    cw: i32,
    ch: i32,
}

/// État mis en cache de l'intérieur fin : la maçonnerie, les voûtes, les poutres, le four de pierre,
/// l'étagère, le râtelier, le billot et l'enclume sont calculés une seule fois par taille de cellule.
#[derive(Default)]
pub struct Interieur {
    cle: Option<CleStatique>,
    statique: Vec<Mat>,
    tampon: Vec<Mat>,
}

impl Interieur {
    #[allow(clippy::too_many_arguments)]
    pub fn peindre(
        &mut self,
        pixels: &mut [u32],
        w: usize,
        h: usize,
        x0: i32,
        cols: usize,
        cw: i32,
        ch: i32,
        monde: &Monde,
        activite: Activite,
        pose: &Pose,
        tick: i64,
        grille: &GrilleLumiere,
    ) -> Option<(i32, i32, i32, i32)> {
        let debut_col = (0 - x0).max(0);
        let fin_col = (YARD - x0).min(cols as i32);
        if debut_col >= fin_col || cw <= 0 || ch <= 0 {
            return None;
        }
        let rh = ((ROWS * ch) as usize).min(h);
        let rw = (YARD * cw) as usize;
        if rh == 0 || rw == 0 {
            return None;
        }
        let aw = rw.div_ceil(GRAIN) as i32;
        let ah = rh.div_ceil(GRAIN) as i32;
        let cle = CleStatique { aw, ah, cw, ch };
        let demi = ch as f32 / 2.0;

        if self.cle != Some(cle) {
            self.statique = vec![Mat::default(); (aw * ah) as usize];
            let mut c = Canevas { buf: &mut self.statique, aw, ah, cw: cw as f32, demi };
            batir_architecture(&mut c);
            self.tampon = vec![Mat::default(); (aw * ah) as usize];
            self.cle = Some(cle);
        }

        self.tampon.copy_from_slice(&self.statique);
        {
            let mut c = Canevas { buf: &mut self.tampon, aw, ah, cw: cw as f32, demi };
            peindre_fenetre_ogive(&mut c, monde, monde.clock);
            peindre_etagere_alchimie(&mut c, monde.clock);
            peindre_astrolabe(&mut c, monde.hour as f32);
            peindre_ratelier_et_anduril(&mut c, monde, activite, pose);
            if let Some(b) = &monde.banner {
                peindre_banniere(&mut c, b, monde.clock);
            }
            peindre_lanterne_murale(&mut c, monde.lantern_lit, monde.clock);
            peindre_cloche_bronze(&mut c, monde.bell_age);
            peindre_machine_vapeur(&mut c, monde, pose, tick);
            peindre_foyer_et_soufflet(&mut c, monde, pose, tick);
            peindre_enclume_et_accessoires(&mut c, monde, pose, tick);
            peindre_chandelles_poudlard(&mut c, monde.clock);
        }

        // Projection éclairée sur `pixels`, uniquement sous les cellules transparentes (`*d >> 24 == 0`).
        let sx0 = (debut_col * cw) as usize;
        let sx1 = ((fin_col * cw) as usize).min(w);
        if sx0 >= sx1 {
            return None;
        }
        let dec_x = (x0 * cw) as i32; // décalage écran -> pixel de pièce (u=0 -> rx=0)
        let ax_min = ((sx0 as i32 + dec_x).max(0) as usize) / GRAIN;
        let ax_max = (((sx1 as i32 + dec_x) as usize).div_ceil(GRAIN)).min(aw as usize);

        for ay in 0..ah as usize {
            let v = ((ay as f32 + 0.5) * GRAIN as f32) / demi;
            let sy_a = ay * GRAIN;
            let sy_b = (sy_a + GRAIN).min(rh);
            let row_mat = ay * aw as usize;
            for ax in ax_min..ax_max {
                let mat = self.tampon[row_mat + ax];
                let u = ((ax as f32 + 0.5) * GRAIN as f32) / cw as f32;
                let k = match mat.mode {
                    Mode::Mur => GrilleLumiere::lire(&grille.mur, u, v),
                    Mode::Sol => GrilleLumiere::lire(&grille.sol, u, v),
                    Mode::Plein => GrilleLumiere::lire(&grille.plein, u, v),
                    Mode::Lueur => [1.0, 1.0, 1.0],
                };
                let r = (mat.rgb[0] as f32 * k[0]).clamp(0.0, 255.0) as u32;
                let g = (mat.rgb[1] as f32 * k[1]).clamp(0.0, 255.0) as u32;
                let b = (mat.rgb[2] as f32 * k[2]).clamp(0.0, 255.0) as u32;
                let coul = 0xFF00_0000 | (r << 16) | (g << 8) | b;

                let rx_a = (ax * GRAIN) as i32 - dec_x;
                let rx_b = rx_a + GRAIN as i32;
                let px_a = rx_a.max(sx0 as i32) as usize;
                let px_b = (rx_b.min(sx1 as i32)).max(0) as usize;
                for sy in sy_a..sy_b {
                    let dst_row = sy * w;
                    for px in px_a..px_b {
                        let d = &mut pixels[dst_row + px];
                        if *d >> 24 == 0 {
                            *d = coul;
                        }
                    }
                }
            }
        }

        // Détails fins `1×1` (mèches des chandelles, éclats d'étoiles et reflets spéculaires)
        peindre_details_fins_1x1(pixels, w, rh, sx0, sx1, dec_x, cw as f32, demi, monde, pose);

        Some((sx0 as i32, 0, sx1 as i32, rh as i32))
    }
}

// ============================================================================
// F1, F3 (maçonnerie), F4 (billot & enclume), F7 (arche de porte) — Statique
// ============================================================================

fn batir_architecture(c: &mut Canevas) {
    let sol_y = c.ay(29.2);
    let plinthe_y = c.ay(27.8);

    // 1. Murs d'appareil gothique à joints francs et pierres taillées (sans tramage)
    for ay in 0..sol_y {
        let v = c.v_de(ay);
        let cours = (v / 2.1).floor() as i64;
        let fy = (v / 2.1).fract();
        let joint_h = fy < 0.13;
        for ax in 0..c.aw {
            let u = c.u_de(ax);
            let dec = if cours % 2 != 0 { 1.9 } else { 0.0 };
            let bloc = ((u + dec) / 3.8).floor() as i64;
            let fx = ((u + dec) / 3.8).fract();
            let joint_v = fx < 0.09;

            let base = if joint_h || joint_v {
                rgb(0x2f2833)
            } else {
                let h = hash_f(cours * 137 + bloc * 31 + 11);
                let teinte = if h < 0.30 {
                    rgb(0x463c48)
                } else if h < 0.68 {
                    rgb(0x504552)
                } else if h < 0.90 {
                    rgb(0x5a4d5a)
                } else {
                    rgb(0x4a4352)
                };
                // Biseau franc en haut de pierre
                if fy < 0.26 && !joint_v {
                    echelle(teinte, 1.12)
                } else if fy > 0.88 {
                    echelle(teinte, 0.88)
                } else {
                    teinte
                }
            };
            c.poser(ax, ay, base, Mode::Mur);
        }
    }

    // Assise basse de soubassement en pierre plus dense (`v = 27.8..29.2`)
    for ay in plinthe_y..sol_y {
        for ax in 0..c.aw {
            let u = c.u_de(ax);
            let joint = ((u / 4.5).fract() < 0.08) || ay == plinthe_y;
            let col = if joint { rgb(0x28222c) } else if ay == plinthe_y + 1 { rgb(0x5c505e) } else { rgb(0x433a46) };
            c.poser(ax, ay, col, Mode::Mur);
        }
    }

    // 2. Nervures de voûte gothique et pilastres muraux (`u = 11.8, 28.2, 45.2`)
    // Attention : dans `45..53 × 2..10`, les nervures restent en Mode::Mur sombre pour préserver le contraste nocturne !
    for &pu in &[11.8f32, 28.2, 45.2] {
        let px = c.ax(pu);
        // Pilastre : une demi-colonne de pierre de cinq pixels, éclairée à gauche, sa base et son chapiteau débordants
        // (un filet d'un pixel se lisait comme une couture du mur)
        let (haut_p, bas_p) = (c.ay(5.6), plinthe_y);
        for (dx, col) in [(-3, rgb(0x2a2430)), (-2, rgb(0x6a5e6e)), (-1, rgb(0x5a4e5e)), (0, rgb(0x52475a)), (1, rgb(0x463c4c)), (2, rgb(0x383040)), (3, rgb(0x2a2430))] {
            c.rect(px + dx, haut_p, px + dx + 1, bas_p, col, Mode::Mur);
        }
        c.rect(px - 4, bas_p - 3, px + 5, bas_p, rgb(0x4e4452), Mode::Mur);
        c.rect(px - 4, bas_p - 3, px + 5, bas_p - 2, rgb(0x665a6a), Mode::Mur);
        // Chapiteau taillé
        c.rect(px - 4, c.ay(5.0), px + 5, haut_p, rgb(0x635666), Mode::Mur);
        c.rect(px - 4, haut_p - 1, px + 5, haut_p, rgb(0x3a3040), Mode::Mur);
        // Deux arcs-doubleaux qui montent en ogive vers la poutre maîtresse
        for dir in [-1.0f32, 1.0] {
            let mut st = 0;
            while st <= 24 {
                let t = st as f32 / 24.0;
                let ru = pu + dir * t * 7.2;
                let rv = 5.0 - (1.0 - (1.0 - t) * (1.0 - t)) * 4.0;
                if (0.0..YARD as f32).contains(&ru) && rv >= 1.0 {
                    let rx = c.ax(ru);
                    let ry = c.ay(rv);
                    c.poser(rx, ry - 1, rgb(0x5d5060), Mode::Mur);
                    c.poser(rx, ry, rgb(0x52465a), Mode::Mur);
                    c.poser(rx, ry + 1, rgb(0x362e3a), Mode::Mur);
                }
                st += 1;
            }
        }
    }

    // Poutre maîtresse de chêne ancien au plafond (`v = 0.0..1.3`) et corbeaux ferrés
    let poutre_b = c.ay(1.25).max(2);
    c.rect(0, 0, c.aw, poutre_b, rgb(0x4b3120), Mode::Mur);
    c.rect(0, poutre_b - 1, c.aw, poutre_b, rgb(0x2c1b11), Mode::Mur);
    for &bu in &[6.0f32, 18.0, 30.0, 42.0] {
        let bx = c.ax(bu);
        c.rect(bx, 0, bx + 1, poutre_b, rgb(0x6e7382), Mode::Mur);
    }

    // 3. Dalles de granit usées au sol (`v = 29.2..32.0`), mousses près du seuil
    for ay in sol_y..c.ah {
        let v = c.v_de(ay);
        let rang = ((v - 29.2) / 1.35).floor() as i64;
        let fv = ((v - 29.2) / 1.35).fract();
        for ax in 0..c.aw {
            let u = c.u_de(ax);
            let dec = if rang % 2 == 0 { 0.0 } else { 2.6 };
            let dalle = ((u + dec) / 5.2).floor() as i64;
            let fu = ((u + dec) / 5.2).fract();
            let joint = ay == sol_y || fv < 0.18 || fu < 0.07;
            let col = if joint {
                // Mousse fraîche entre les dalles près de la porte de la cour
                if u > 49.5 && hash_f(dalle * 19 + ay as i64 * 7) > 0.45 {
                    rgb(0x3b6234)
                } else {
                    rgb(0x26222c)
                }
            } else {
                let h = hash_f(dalle * 73 + rang * 41 + 3);
                let pierre = if h < 0.35 {
                    rgb(0x5d5666)
                } else if h < 0.75 {
                    rgb(0x686072)
                } else {
                    rgb(0x544e5d)
                };
                if fv < 0.36 { echelle(pierre, 1.10) } else { pierre }
            };
            c.poser(ax, ay, col, Mode::Sol);
        }
    }

    // 4. F7 — Arche de pierre gothique de la porte vers la cour (`u = 51.4..54.0`)
    let porte_x = c.ax(51.6);
    for ay in poutre_b..sol_y {
        let v = c.v_de(ay);
        // Courbure d'ébrasement gothique en haut de la porte (`v = 1..8`)
        let surplomb = if v < 8.5 { ((8.5 - v) / 7.5).powi(2) * 3.0 } else { 0.0 };
        let x_arch = c.ax(51.6 - surplomb);
        let bloc = (v / 2.6).floor() as i64;
        let joint = (v / 2.6).fract() < 0.12;
        for ax in x_arch..c.aw {
            let col = if ax == x_arch {
                CONTOUR
            } else if joint {
                rgb(0x322b38)
            } else if ax == x_arch + 1 {
                rgb(0x7c7386)
            } else if bloc % 2 == 0 {
                rgb(0x675f72)
            } else {
                rgb(0x5b5366)
            };
            c.poser(ax, ay, col, Mode::Mur);
        }
    }
    // Gonds de fer forgé sur le montant droit
    for &gv in &[10.5f32, 22.5] {
        let gy = c.ay(gv);
        c.rect(porte_x, gy, c.aw, gy + 2, rgb(0x2a2d38), Mode::Sol);
        c.poser(porte_x + 1, gy, rgb(0x8a90a2), Mode::Sol);
    }

    // 5. F3 — Maçonnerie naine du grand four de Khazad-dûm (`u = 0..13.6, v = 0..31.0`)
    batir_four_nain(c, sol_y);

    // 6. F4 — Billot de chêne cerclé de fer et enclume runique ciselée (`ANVIL_AT = (16, 25)`)
    batir_billot_et_enclume(c);
}

fn batir_four_nain(c: &mut Canevas, sol_y: i32) {
    // Conduit de cheminée cannelé montant jusqu'à la voûte (`u = 3.8..9.2, v = 1.2..8.2`)
    let (ch_x0, ch_x1) = (c.ax(3.8), c.ax(9.2));
    let (ch_y0, ch_y1) = (c.ay(1.2), c.ay(8.2));
    c.boite(ch_x0, ch_y0, ch_x1, ch_y1, rgb(0x4b4654), CONTOUR, Mode::Plein);
    for ax in ch_x0 + 1..ch_x1 - 1 {
        let u = c.u_de(ax);
        let col = if u < 5.4 {
            rgb(0x645e6e)
        } else if (u - 6.5).abs() < 0.35 {
            rgb(0x393442)
        } else {
            rgb(0x504a5a)
        };
        for ay in ch_y0 + 1..ch_y1 - 1 {
            let joint = ((c.v_de(ay) / 2.2).fract()) < 0.14;
            c.poser(ax, ay, if joint { rgb(0x322d3a) } else { col }, Mode::Plein);
        }
    }
    // Bague de bronze rivée autour du conduit (`v = 4.6..5.4`)
    let bag_y = c.ay(4.7);
    c.rect(ch_x0, bag_y, ch_x1, bag_y + 2, rgb(0x7c5a34), Mode::Plein);
    c.poser(ch_x0 + 2, bag_y, rgb(0xd4a44c), Mode::Plein);
    c.poser(ch_x1 - 3, bag_y, rgb(0xd4a44c), Mode::Plein);

    // Hotte trapézoïdale à degrés nains (`v = 8.2..15.6`)
    for ay in ch_y1..c.ay(15.6) {
        let v = c.v_de(ay);
        let t = ((v - 8.2) / 7.4).clamp(0.0, 1.0);
        let u_droite = 10.2 + (t * 3.4).floor() * 1.0;
        let x1 = c.ax(u_droite.min(13.6));
        let joint_h = ((v - 8.2) / 1.85).fract() < 0.16;
        for ax in 0..x1 {
            let u = c.u_de(ax);
            let joint_v = ((u + if ((v / 1.85) as i32) % 2 == 0 { 0.0 } else { 1.6 }) / 3.2).fract() < 0.10;
            let col = if ax == x1 - 1 || ay == ch_y1 {
                CONTOUR
            } else if joint_h || joint_v {
                rgb(0x2e2936)
            } else if ax <= 2 {
                rgb(0x6b6476)
            } else if ax >= x1 - 3 {
                rgb(0x45404e)
            } else {
                rgb(0x585262)
            };
            c.poser(ax, ay, col, Mode::Plein);
        }
    }

    // Corps massif du four (`u = 0..13.6, v = 15.6..30.8`)
    let corps_x1 = c.ax(13.6);
    let corps_y0 = c.ay(15.6);
    let corps_y1 = sol_y + 2;
    for ay in corps_y0..corps_y1 {
        let v = c.v_de(ay);
        let joint_h = (v / 2.2).fract() < 0.13;
        for ax in 0..corps_x1 {
            let u = c.u_de(ax);
            let joint_v = ((u + 1.1 * ((v / 2.2).floor() % 2.0)) / 3.4).fract() < 0.09;
            let col = if ax == corps_x1 - 1 || ay == corps_y1 - 1 {
                CONTOUR
            } else if joint_h || joint_v {
                rgb(0x2b2633)
            } else if u < 2.2 {
                rgb(0x686173)
            } else if u > 11.4 {
                rgb(0x46414f)
            } else {
                rgb(0x575060)
            };
            c.poser(ax, ay, col, Mode::Plein);
        }
    }

    // Grand linteau angulaire de Khazad-dûm (`u = 1.4..12.6, v = 15.6..18.0`) à chevrons de bronze
    let (lx0, lx1) = (c.ax(1.4), c.ax(12.6));
    let (ly0, ly1) = (c.ay(15.6), c.ay(18.0));
    c.boite(lx0, ly0, lx1, ly1, rgb(0x423c4a), CONTOUR, Mode::Plein);
    c.rect(lx0 + 1, ly0 + 1, lx1 - 1, ly0 + 2, rgb(0x7c6e78), Mode::Plein);
    // Filets d'or nain aux extrémités du linteau
    c.rect(lx0 + 1, ly0 + 1, lx0 + 3, ly1 - 1, rgb(0x9e7838), Mode::Plein);
    c.rect(lx1 - 3, ly0 + 1, lx1 - 1, ly1 - 1, rgb(0x9e7838), Mode::Plein);

    // Chambre du foyer (`MOUTH = (3, 18, 8, 8)`) : fond de brique réfractaire noircie de suie
    let (mx0, mx1) = (c.ax(MOUTH.0 as f32 - 0.2), c.ax((MOUTH.0 + MOUTH.2) as f32 + 0.2));
    let (my0, my1) = (c.ay(MOUTH.1 as f32), c.ay((MOUTH.1 + MOUTH.3) as f32 + 0.8));
    c.boite(mx0 - 1, my0 - 1, mx1 + 1, my1 + 1, rgb(0x1b151c), CONTOUR, Mode::Plein);
    for ay in my0..my1 {
        let jr = (ay - my0) % 3 == 0;
        for ax in mx0..mx1 {
            let jc = ((ax + (ay / 3) * 2) % 4) == 0;
            let brique = if jr || jc { rgb(0x141016) } else { rgb(0x281d22) };
            c.poser(ax, ay, brique, Mode::Plein);
        }
    }

    // Seuil de pierre de l'âtre (`v = 26.8..28.4`)
    c.boite(c.ax(1.2), my1, c.ax(12.8), my1 + 3, rgb(0x645c6e), CONTOUR, Mode::Plein);
}

fn batir_billot_et_enclume(c: &mut Canevas) {
    let ax_g = ANVIL_AT.0 as f32; // 16.0
    let ay_h = ANVIL_AT.1 as f32; // 25.0

    // 1. Billot de chêne massif (`u = 18.2..26.8, v = 28.0..31.6`) cerclé d'une frette de fer rivée
    let (bx0, bx1) = (c.ax(ax_g + 2.2), c.ax(ax_g + 10.8));
    let (by0, by1) = (c.ay(ay_h + 3.0), c.ay(31.6));
    c.boite(bx0, by0, bx1, by1, rgb(0x6d4428), CONTOUR, Mode::Plein);
    for ax in bx0 + 1..bx1 - 1 {
        let u = c.u_de(ax);
        let fibre = ((u * 2.4).floor() as i32) % 3;
        let bois = if ax < bx0 + 4 {
            if fibre == 0 { rgb(0x885836) } else { rgb(0x784c2e) }
        } else if ax > bx1 - 4 {
            rgb(0x52321c)
        } else if fibre == 1 {
            rgb(0x5d3922)
        } else {
            rgb(0x6d4428)
        };
        for ay in by0 + 1..by1 - 1 {
            c.poser(ax, ay, bois, Mode::Plein);
        }
    }
    // Frette de fer forgé et rivets brillants au milieu du billot
    let fy = (by0 + by1) / 2;
    c.rect(bx0, fy - 1, bx1, fy + 1, CONTOUR, Mode::Plein);
    c.rect(bx0 + 1, fy, bx1 - 1, fy + 1, rgb(0x5f6678), Mode::Plein);
    for k in 0..4 {
        let rx = bx0 + 3 + k * ((bx1 - bx0 - 5).max(4) / 3);
        if rx < bx1 - 1 {
            c.poser(rx, fy, rgb(0xb8c0d4), Mode::Plein);
        }
    }

    // 2. Enclume runique naine, de profil (`u = 16.0..28.8`, table à `v = 25.0`) : la bigorne en cône vers le foyer, la
    // table épaisse et son talon en surplomb à droite, la taille cintrée, les pieds évasés sur le billot. La silhouette
    // est décrite rangée par rangée (bords gauche et droit), puis cernée d'un trait net.
    let y0 = c.ay(ay_h);
    let (pointe, table0, talon) = (c.ax(ax_g), c.ax(ax_g + 3.2), c.ax(ax_g + 12.8));
    let (corps0, corps1) = (c.ax(ax_g + 3.8), c.ax(ax_g + 10.4));
    let bas = by0; // la semelle pose sur le billot
    let rangs = (bas - y0).max(8);
    let bord = |r: i32| -> (i32, i32) {
        let f = r as f32 / rangs as f32;
        let g = match r {
            0 => pointe + 4,
            1 => pointe + 1,
            2 => pointe,
            3 => pointe + 3,
            4 => pointe + 7,
            5 => table0 + 1,
            _ => {
                // la taille se creuse au milieu, les pieds s'évasent en bas
                let creux = (1.0 - ((f - 0.62) / 0.3).powi(2)).max(0.0);
                corps0 + (creux * 4.0).round() as i32 - if f > 0.85 { ((f - 0.85) / 0.15 * 2.0).round() as i32 } else { 0 }
            }
        };
        let d = match r {
            0..=2 => talon,
            3 => talon - 2,
            4 => corps1 + 3,
            5 => corps1 + 1,
            _ => {
                let creux = (1.0 - ((f - 0.62) / 0.3).powi(2)).max(0.0);
                corps1 - (creux * 4.0).round() as i32 + if f > 0.85 { ((f - 0.85) / 0.15 * 2.0).round() as i32 } else { 0 }
            }
        };
        (g, d)
    };
    let dedans = |x: i32, r: i32| -> bool {
        if r < 0 || r >= rangs {
            return false;
        }
        let (g, d) = bord(r);
        x >= g && x < d
    };
    for r in 0..rangs {
        let (g, d) = bord(r);
        for x in g..d {
            let cerne = !dedans(x - 1, r) || !dedans(x + 1, r) || !dedans(x, r - 1) || !dedans(x, r + 1);
            let col = if cerne {
                CONTOUR
            } else if r == 1 {
                rgb(0xd4dced) // le dessus poli de la table et de la bigorne
            } else if r <= 3 {
                if x < table0 { rgb(0x9aa3b8) } else { rgb(0x8f98ac) }
            } else if x < g + 3 {
                rgb(0x8a93a8) // le flanc tourné vers le feu
            } else if x > d - 4 {
                rgb(0x4c5466)
            } else {
                rgb(0x646d82)
            };
            c.poser(x, y0 + r, col, Mode::Plein);
        }
    }
    // le trou carré de la tranche, près du talon
    c.poser(talon - 5, y0 + 1, rgb(0x2a2c38), Mode::Plein);

    // Trois runes naines gravées au flanc (des Cirth) : sombres au repos, elles s'allument quand il forge
    // (`peindre_enclume_et_accessoires`)
    for (x, y) in runes_enclume(c) {
        c.poser(x, y, rgb(0x3a4050), Mode::Plein);
    }
}

/// Les runes de l'enclume : trois Cirth de 3 × 4 au milieu du flanc, là où la taille se creuse.
const RUNES_ENCLUME: [[&str; 4]; 3] = [["x..", "xx.", "x.x", "x.."], [".x.", "x.x", ".x.", ".x."], ["x.x", "xxx", "x.x", "x.x"]];

/// Les pixels des runes de l'enclume (ceux de `batir_billot_et_enclume`).
fn runes_enclume(c: &Canevas) -> Vec<(i32, i32)> {
    let (ax_g, ay_h) = (ANVIL_AT.0 as f32, ANVIL_AT.1 as f32);
    let y0 = c.ay(ay_h);
    let (corps0, corps1) = (c.ax(ax_g + 3.8), c.ax(ax_g + 10.4));
    let rangs = (c.ay(ay_h + 3.0) - y0).max(8);
    let (rx, ry) = ((corps0 + corps1) / 2, y0 + rangs * 3 / 5);
    let mut out = Vec::new();
    for (n, rune) in RUNES_ENCLUME.iter().enumerate() {
        let x0 = rx - 5 + n as i32 * 4;
        for (j, ligne) in rune.iter().enumerate() {
            for (i, ch) in ligne.bytes().enumerate() {
                if ch == b'x' {
                    out.push((x0 + i as i32, ry - 1 + j as i32));
                }
            }
        }
    }
    out
}

// ============================================================================
// F2 — Fenêtre gothique en ogive à vitrail, ciel vivant, volets & citrouille
// ============================================================================

fn peindre_fenetre_ogive(c: &mut Canevas, w: &Monde, clock: i64) {
    let u0 = WINDOW.0 as f32; // 13.0
    let u1 = (WINDOW.0 + WINDOW.2) as f32; // 25.0
    let uc = (u0 + u1) * 0.5; // 19.0
    let v_haut = 0.6f32;
    let v_imposte = 4.6f32; // naissance de l'arc brisé
    let v_appui = V_APPUI;

    let nuit = w.nuit();
    let ciel_h = w.heure_ciel() as f32;
    let crepuscule = !nuit && !(8.0..18.0).contains(&w.heure_ciel());
    let fermes = w.hour >= 23.0 || w.hour < 6.0;

    let ax0 = c.ax(u0 - 0.5);
    let ax1 = c.ax(u1 + 0.5);
    let ay0 = c.ay(v_haut - 0.3);
    let ay_appui = c.ay(v_appui);

    // Fonction de demi-largeur de la baie gothique en ogive à la hauteur `v`
    let demi_largeur = |v: f32| -> f32 {
        if v >= v_imposte {
            5.5
        } else {
            let t = ((v - v_haut) / (v_imposte - v_haut)).clamp(0.0, 1.0);
            5.5 * (1.0 - (1.0 - t).powf(1.85))
        }
    };

    for ay in ay0..ay_appui {
        let v = c.v_de(ay);
        let dl = demi_largeur(v);
        if dl <= 0.15 {
            continue;
        }
        let dl_int = (dl - 0.85).max(0.0);
        for ax in ax0..=ax1 {
            let u = c.u_de(ax);
            let du = (u - uc).abs();
            if du > dl + 0.35 {
                continue;
            }
            if du >= dl_int {
                // Pied-droit et voussure de pierre taillée en ogive
                let bord_ext = du > dl - 0.15;
                let pierre = if bord_ext {
                    CONTOUR
                } else if u < uc {
                    rgb(0x887e92)
                } else {
                    rgb(0x685f72)
                };
                c.poser(ax, ay, pierre, Mode::Mur);
                continue;
            }

            // Intérieur de la baie : tympan de vitrail au plomb en haut (`v < v_imposte`), lancettes en bas
            let meneau_central = du < 0.32 && v > v_haut + 1.4;
            let traverse = (v - v_imposte).abs() < 0.28;
            if meneau_central || traverse {
                c.poser(ax, ay, rgb(0x282530), Mode::Mur);
                continue;
            }

            if v < v_imposte {
                // Rosace trilobée et vitrail médiéval au plomb (rubis, saphir, ambre, émeraude)
                let rosace_d = ((u - uc) * 0.95).hypot(v - (v_haut + 2.35));
                // des losanges de verre d'environ six pixels d'art, sertis de plomb (plus serrés, ils faisaient un damier)
                let plomb_losange = (((u - uc) * 0.72 + v * 0.72).rem_euclid(1.0) < 0.16) || (((u - uc) * 0.72 - v * 0.72).rem_euclid(1.0) < 0.16);
                let verre = if (rosace_d - 1.25).abs() < 0.22 || plomb_losange {
                    rgb(0x23202b)
                } else if rosace_d < 1.25 {
                    if rosace_d < 0.52 {
                        rgb(0xf8cf58) // cœur d'or
                    } else if (u - uc) * (v - (v_haut + 2.35)) >= 0.0 {
                        rgb(0xdc4250) // rubis
                    } else {
                        rgb(0x3c84eb) // saphir
                    }
                } else if u < uc {
                    rgb(0x38b46e) // émeraude
                } else {
                    rgb(0xe89a38) // ambre
                };
                let k = if verre == rgb(0x23202b) { 1.0 } else if nuit { 0.58 } else if crepuscule { 0.88 } else { 1.0 };
                c.poser(ax, ay, echelle(verre, k), if verre == rgb(0x23202b) { Mode::Mur } else { Mode::Lueur });
            } else if fermes {
                // Volets intérieurs de chêne ferré clos la nuit tardive (`23h..6h`), percés d'un trèfle
                let planche = ((u * 1.8).fract()) < 0.12;
                let ferrure = (v - (v_imposte + 1.1)).abs() < 0.25 || (v - (v_appui - 1.1)).abs() < 0.25;
                let judas = ((u - (uc - 2.4)).abs() + (v - 6.5).abs() < 0.55) || ((u - (uc + 2.4)).abs() + (v - 6.5).abs() < 0.55);
                if judas {
                    c.poser(ax, ay, rgb(0x16224a), Mode::Lueur);
                } else if ferrure {
                    c.poser(ax, ay, rgb(0x4a4f5e), Mode::Mur);
                } else if planche {
                    c.poser(ax, ay, rgb(0x2e1c11), Mode::Mur);
                } else {
                    c.poser(ax, ay, rgb(0x523420), Mode::Mur);
                }
            } else {
                // Lancettes ouvertes sur le ciel de l'heure, résille de plomb discrète
                let tf = ((v - v_imposte) / (v_appui - v_imposte)).clamp(0.0, 1.0);
                let mut sky = if nuit {
                    melange(rgb(0x0b1130), rgb(0x223568), tf)
                } else if crepuscule {
                    melange(rgb(0x3e4888), rgb(0xf09e58), tf)
                } else {
                    melange(rgb(0x4c9ce6), rgb(0xbfe4f8), tf)
                };

                // Arc-en-ciel pastel après une série de réussites
                if w.weather == Some(Temps::Rainbow) && !nuit {
                    let dr = ((u - uc).hypot(v - (v_appui + 1.2))) - 4.2;
                    if (0.0..1.6).contains(&dr) {
                        let arc_c = match (dr * 2.5) as usize {
                            0 => rgb(0x74b0ff),
                            1 => rgb(0x84e296),
                            2 => rgb(0xffe878),
                            _ => rgb(0xff8474),
                        };
                        sky = melange(sky, arc_c, 0.65);
                    }
                }

                // Soleil ou croissant de lune + étoiles
                let course = if nuit { ((ciel_h + 3.0).rem_euclid(24.0)) / 9.0 } else { (ciel_h - 6.0) / 15.0 }.clamp(0.05, 0.95);
                let su = (u0 + 1.6) + course * (u1 - u0 - 3.2);
                let sv = v_imposte + 1.1 + (course - 0.5).abs() * 2.4;
                let da = (u - su).hypot(v - sv);
                if nuit {
                    if da < 0.85 && (u - (su + 0.32)).hypot(v - (sv - 0.15)) > 0.68 {
                        sky = rgb(0xf4f0d8);
                    } else if hash_f((ax * 97 + ay * 31) as i64) > 0.965
                        && hash_f((ax + ay + (clock / 6) as i32) as i64) > 0.3
                    {
                        sky = rgb(0xeef2ff);
                    }
                } else if da < 0.95 {
                    sky = if da < 0.55 { rgb(0xfff8c8) } else { rgb(0xffcf58) };
                }

                // Flocons d'hiver derrière la vitre
                if w.saison() == Saison::Hiver {
                    let fl = ((ax * 19 + ((ay as i64 + clock / 2) as i32) * 37) as i64).rem_euclid(53);
                    if fl == 0 {
                        sky = rgb(0xf6faff);
                    }
                }

                c.poser(ax, ay, sky, Mode::Lueur);
            }
        }
    }

    // Volets ouverts rabattus de part et d'autre des lancettes (`6h..23h`)
    if !fermes {
        for &(vu0, vu1) in &[(u0 - 1.7, u0 - 0.5), (u1 + 0.5, u1 + 1.7)] {
            let (vx0, vx1) = (c.ax(vu0), c.ax(vu1));
            let (vy0, vy1) = (c.ay(v_imposte + 0.2), ay_appui);
            c.boite(vx0, vy0, vx1, vy1, rgb(0x5a3922), CONTOUR, Mode::Mur);
            let f1 = vy0 + 2;
            let f2 = vy1 - 3;
            c.rect(vx0 + 1, f1, vx1 - 1, f1 + 1, rgb(0x6c7282), Mode::Mur);
            c.rect(vx0 + 1, f2, vx1 - 1, f2 + 1, rgb(0x6c7282), Mode::Mur);
        }
    }

    // Appui de pierre saillant (`v = 9.4..10.3`)
    let (sx0, sx1) = (c.ax(u0 - 1.0), c.ax(u1 + 1.0));
    let sy1 = c.ay(10.35).max(ay_appui + 2);
    c.boite(sx0, ay_appui, sx1, sy1, rgb(0x7c7286), CONTOUR, Mode::Sol);
    c.rect(sx0 + 1, ay_appui + 1, sx1 - 1, ay_appui + 2, rgb(0x9c92a6), Mode::Sol);

    // Citrouille sculptée illuminée sur l'appui à Halloween
    if w.fete() == Some(Fete::Halloween) {
        // une citrouille ronde et côtelée, cernée, sa queue verte, ses yeux et sa bouche qui luisent (un disque de
        // rayon 2 se lisait comme une croix)
        const CITROUILLE: [&str; 6] = ["...g...", ".kkgkk.", "kOoOoOk", "kyOoOyk", "kOyyyOk", ".kkkkk."];
        let (px, py) = (c.ax(u1 - 2.6) - 3, ay_appui - 6);
        for (j, ligne) in CITROUILLE.iter().enumerate() {
            for (i, b) in ligne.bytes().enumerate() {
                let (col, mode) = match b {
                    b'k' => (CONTOUR, Mode::Plein),
                    b'g' => (rgb(0x3c6e2c), Mode::Plein),
                    b'O' => (rgb(0xe8742a), Mode::Plein),
                    b'o' => (rgb(0xc45a1e), Mode::Plein),
                    b'y' => (rgb(0xffd85a), Mode::Lueur),
                    _ => continue,
                };
                c.poser(px + i as i32, py + j as i32, col, mode);
            }
        }
    }
}

// ============================================================================
// F9 — Étagère d'alchimiste : grimoires reliés et potions bouillonnantes
// ============================================================================

fn peindre_etagere_alchimie(c: &mut Canevas, clock: i64) {
    // Placée en `u = 29.5..43.2, v = 6.0..10.2` (hors de `45..53 × 2..10`)
    let (u0, u1) = (29.5f32, 43.2f32);
    let v_planche = 9.4f32;
    let (sx0, sx1) = (c.ax(u0), c.ax(u1));
    let sy = c.ay(v_planche);

    // Consoles en fer forgé sous la tablette
    for &bu in &[30.8f32, 41.8] {
        let bx = c.ax(bu);
        c.rect(bx, sy + 2, bx + 2, sy + 5, rgb(0x323644), Mode::Sol);
        c.poser(bx + 1, sy + 3, rgb(0x788094), Mode::Sol);
    }
    // Tablette de chêne moulurée
    c.boite(sx0, sy, sx1, sy + 3, rgb(0x6e4528), CONTOUR, Mode::Sol);
    c.rect(sx0 + 1, sy + 1, sx1 - 1, sy + 2, rgb(0x8c5a36), Mode::Sol);

    // 1. Trois grimoires anciens à nerfs d'or (`u = 30.2..34.8`)
    for &(gu, gw, gh, reliure, nerf) in &[
        (30.3f32, 1.3f32, 3.0f32, rgb(0x8e2430), rgb(0xe8be54)), // grimoire rubis
        (31.7, 1.2, 2.7, rgb(0x245838), rgb(0xd8b048)),          // herbier émeraude
        (33.0, 1.3, 3.1, rgb(0x283c7a), rgb(0xf0c85c)),          // codex astral saphir
    ] {
        let (gx0, gx1) = (c.ax(gu), c.ax(gu + gw).max(c.ax(gu) + 3));
        let gy0 = c.ay(v_planche - gh);
        c.boite(gx0, gy0, gx1, sy + 1, reliure, CONTOUR, Mode::Sol);
        c.rect(gx0 + 1, gy0 + 2, gx1 - 1, gy0 + 3, nerf, Mode::Sol);
        if sy - gy0 >= 5 {
            c.rect(gx0 + 1, sy - 2, gx1 - 1, sy - 1, nerf, Mode::Sol);
        }
    }

    // 2. Parchemin roulé scellé d'un ruban vermeil (`u = 34.8..36.4`)
    let px0 = c.ax(34.8);
    c.boite(px0, sy - 3, px0 + 4, sy + 1, rgb(0xe6d6b0), CONTOUR, Mode::Sol);
    c.poser(px0 + 2, sy - 2, rgb(0xd43838), Mode::Sol);

    // 3. Fiole pansue de potion d'émeraude bouillonnante (`u = 37.6..39.8`)
    let fx = c.ax(38.6);
    c.disque(fx, sy - 2, 3, CONTOUR, Mode::Sol);
    c.disque(fx, sy - 2, 2, rgb(0x1f8a52), Mode::Lueur);
    c.rect(fx - 1, sy - 2, fx + 2, sy, rgb(0x42e888), Mode::Lueur);
    // Col de verre et bouchon de liège
    c.rect(fx - 1, sy - 6, fx + 2, sy - 4, rgb(0x9ec4d4), Mode::Sol);
    c.rect(fx, sy - 7, fx + 1, sy - 6, rgb(0xb8844c), Mode::Sol);
    // Bulles animées qui montent dans l'élixir vert
    let phase_b = (clock.div_euclid(3) % 4) as i32;
    c.poser(fx - 1 + (phase_b % 2), sy - 1 - (phase_b.min(2)), rgb(0xb8ffce), Mode::Lueur);
    c.poser(fx + 1, sy - 3, rgb(0xd8ffe8), Mode::Lueur);

    // 4. Haute fiole facettée d'élixir d'ambre phénix (`u = 40.6..42.4`)
    let ax_f = c.ax(41.4);
    c.boite(ax_f - 2, sy - 5, ax_f + 2, sy + 1, rgb(0xd86824), CONTOUR, Mode::Lueur);
    c.rect(ax_f - 1, sy - 3, ax_f + 1, sy, rgb(0xffbe44), Mode::Lueur);
    c.poser(ax_f - 1, sy - 4, rgb(0xfff0a8), Mode::Lueur);
    c.rect(ax_f - 1, sy - 7, ax_f + 1, sy - 5, rgb(0xa8c8d8), Mode::Sol);
    c.poser(ax_f, sy - 8, rgb(0xc48c50), Mode::Sol);
}

// ============================================================================
// F6 (partie 1) — Astrolabe mural astronomique de laiton suivant `world.hour`
// ============================================================================

fn peindre_astrolabe(c: &mut Canevas, heure: f32) {
    // Placé en `u = 27.4, v = 8.4`, entre la fenêtre et la tête de Scorpheus (`C.0 = 30`) : jamais derrière elle, où
    // son anneau d'or faisait une auréole
    let cx = c.ax(27.4);
    let cy = c.ay(8.4);
    let r = c.ay(1.7).max(5);

    // Anneau de suspension en haut
    c.boite(cx - 1, cy - r - 3, cx + 2, cy - r, rgb(0xd4a646), CONTOUR, Mode::Sol);

    // Limbe de laiton ciselé et fond céleste bleu nuit émaillé
    c.disque(cx, cy, r + 1, CONTOUR, Mode::Sol);
    c.disque(cx, cy, r, rgb(0xc89838), Mode::Sol);
    c.disque(cx, cy, (r - 2).max(2), rgb(0x1e2c54), Mode::Sol);

    // Cercle excentré de l'araignée céleste (rete)
    let ri = (r - 3).max(2);
    for k in 0..16 {
        let a = k as f32 * TAU / 16.0;
        let rx = cx + (a.cos() * ri as f32 * 0.75).round() as i32;
        let ry = cy - 1 + (a.sin() * ri as f32 * 0.75).round() as i32;
        c.poser(rx, ry, rgb(0x9e762a), Mode::Sol);
    }

    // Quatre repères cardinaux d'or clair
    for &(dx, dy) in &[(0, -r + 1), (r - 1, 0), (0, r - 1), (-r + 1, 0)] {
        c.poser(cx + dx, cy + dy, rgb(0xf4d26a), Mode::Sol);
    }

    // Alidade tournante calée sur l'heure du monde (`12h` en haut, tour complet en 24h)
    let angle = ((heure - 12.0) / 24.0) * TAU - std::f32::consts::FRAC_PI_2;
    let lx = cx + (angle.cos() * (r - 1) as f32).round() as i32;
    let ly = cy + (angle.sin() * (r - 1) as f32).round() as i32;
    let ox = cx - (angle.cos() * 2.0).round() as i32;
    let oy = cy - (angle.sin() * 2.0).round() as i32;
    c.ligne(ox, oy, lx, ly, rgb(0xfce07c), Mode::Sol);

    // Rivet central à cabochon saphir
    c.poser(cx, cy, rgb(0x64b8ff), Mode::Lueur);
}

// ============================================================================
// F10 — Râtelier d'armes légendaires, *Andúril* & Bannière médiévale frangée
// ============================================================================

fn peindre_ratelier_et_anduril(c: &mut Canevas, w: &Monde, activite: Activite, p: &Pose) {
    let u0 = RACK.0 as f32; // 13.0
    let u1 = (RACK.0 + 16) as f32; // 29.0
    let v_rail = 12.2f32;
    let (rx0, rx1) = (c.ax(u0), c.ax(u1));
    let ry = c.ay(v_rail);

    // Traverse de chêne sculpté et sept chevilles de laiton (toujours présentes sous l'ogive)
    c.boite(rx0, ry, rx1, ry + 3, rgb(0x684024), CONTOUR, Mode::Sol);
    c.rect(rx0 + 1, ry + 1, rx1 - 1, ry + 2, rgb(0x885634), Mode::Sol);
    for &slot in &RACK_SLOTS {
        let sx = c.ax(slot as f32 + 0.5);
        c.poser(sx, ry + 1, rgb(0xdcb04c), Mode::Sol);
    }

    // *Andúril*, la Flamme de l'Ouest reforgée : couchée à l'horizontale sur crochets d'or quand `masterpiece` ; sa garde
    // en lettres (`ANDURIL_GARDE`), puis la longue lame de mithril, ses runes qui luisent une à une
    if w.masterpiece {
        let ay = c.ay(10.95);
        let (ax_g, ax_d) = (c.ax(15.2), c.ax(26.8));
        c.poser(c.ax(19.0), ay + 3, rgb(0xd4a43c), Mode::Sol);
        c.poser(c.ax(24.0), ay + 3, rgb(0xd4a43c), Mode::Sol);
        dessin_arme(c, ax_g, ay - 4, &ANDURIL_GARDE, &|l| teinte_arme(l, Etat::Froide));
        let (debut, pointe) = (ax_g + 11, ax_d);
        let eclat = ((w.clock / 3) % 12) as i32;
        for x in debut..=pointe {
            let reste = pointe - x;
            let demi: i32 = if reste <= 1 { 0 } else if reste <= 4 { 1 } else { 2 };
            for dy in -demi..=demi {
                let col = if dy.abs() == demi {
                    CONTOUR
                } else if dy < 0 {
                    rgb(0xf4f8ff)
                } else if dy > 0 {
                    rgb(0x9aa6c0)
                } else if (x - debut) % 4 == 2 && reste > 5 {
                    // les runes : chacune s'allume quand l'éclat passe
                    if ((x - debut) / 4 - eclat).abs() <= 1 { rgb(0xffffff) } else { rgb(0x8ad8ff) }
                } else {
                    rgb(0xd4dcec)
                };
                let mode = if dy == 0 && (x - debut) % 4 == 2 { Mode::Lueur } else { Mode::Sol };
                c.poser(x, ay + dy, col, mode);
            }
        }
    }

    if w.rack.is_empty() {
        return;
    }
    let derniers = &w.rack[w.rack.len().saturating_sub(RACK_SLOTS.len())..];
    // Lueur elfique bleue de *Dard* quand la forge bat son plein la nuit, en forge active ou après un échec
    let lueur_dard = w.forging || activite == Activite::Failed || (w.nuit() && p.fire >= 3);

    for (i, &arme) in derniers.iter().enumerate() {
        let su = RACK_SLOTS[i] as f32 + 0.5;
        let sx = c.ax(su);
        let etat = if w.forging && i + 1 == derniers.len() {
            Etat::Chaude
        } else if lueur_dard && arme == Arme::Sword {
            Etat::Dard
        } else {
            Etat::Froide
        };
        // pendue à sa cheville par le haut, le milieu de l'image (sa colonne 4) sous la cheville
        let image: &[&str] = match arme {
            Arme::Sword => &EPEE,
            Arme::Axe => &HACHE,
            Arme::Dagger => &DAGUE,
            Arme::Mace => &MASSE,
        };
        let x0 = if arme == Arme::Axe { sx - 6 } else { sx - 4 };
        dessin_arme(c, x0, ry + 1, image, &|l| teinte_arme(l, etat));
    }
}

/// L'état d'une arme du râtelier : froide, chauffée au rouge (celle du jour, encore à la forge), ou luisante d'un bleu
/// elfique (*Dard*, la nuit quand la forge bat son plein ou après un échec).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Etat {
    Froide,
    Chaude,
    Dard,
}

/// La couleur d'une lettre des armes : l'acier (`H S s d`, `u` les runes) suit l'état, le reste est fixe.
fn teinte_arme(l: u8, etat: Etat) -> Option<([u8; 3], Mode)> {
    let acier = |froid: u32, chaud: u32, bleu: u32| match etat {
        Etat::Froide => (rgb(froid), Mode::Sol),
        Etat::Chaude => (rgb(chaud), Mode::Lueur),
        Etat::Dard => (rgb(bleu), Mode::Lueur),
    };
    Some(match l {
        b'k' => (CONTOUR, Mode::Sol),
        b'H' => acier(0xeef4ff, 0xfff4a0, 0xe8faff),
        b'S' => acier(0xc4d0e4, 0xffc040, 0x9ce0ff),
        b's' => acier(0x8a96b0, 0xff8a20, 0x5cb8ff),
        b'd' => acier(0x56607a, 0xc04a10, 0x2a78c8),
        b'u' => match etat {
            Etat::Froide => (rgb(0x7ab8d8), Mode::Sol),
            Etat::Chaude => (rgb(0xffffff), Mode::Lueur),
            Etat::Dard => (rgb(0xffffff), Mode::Lueur),
        },
        b'G' => (rgb(0xf4d060), Mode::Sol),
        b'g' => (rgb(0xc89030), Mode::Sol),
        b'y' => (rgb(0x8a5c1c), Mode::Sol),
        b'L' => (rgb(0x7a4a2a), Mode::Sol),
        b'l' => (rgb(0x4e2c18), Mode::Sol),
        b'W' => (rgb(0x8a5a32), Mode::Sol),
        b'w' => (rgb(0x5a3820), Mode::Sol),
        b'B' => (rgb(0x64c8ff), Mode::Lueur),
        b'b' => (rgb(0x2a70c8), Mode::Sol),
        b'R' => (rgb(0xe04050), Mode::Lueur),
        b'r' => (rgb(0x8a1a2a), Mode::Sol),
        _ => return None,
    })
}

/// Pose une image en lettres, son coin haut gauche en (`x0`, `y0`), pixel de canevas pour lettre.
fn dessin_arme(c: &mut Canevas, x0: i32, y0: i32, image: &[&str], teinte: &dyn Fn(u8) -> Option<([u8; 3], Mode)>) {
    for (j, ligne) in image.iter().enumerate() {
        for (i, l) in ligne.bytes().enumerate() {
            if let Some((col, mode)) = teinte(l) {
                c.poser(x0 + i as i32, y0 + j as i32, col, mode);
            }
        }
    }
}

// Les armes du râtelier, dessinées à la main (9 de large, la cheville au-dessus de la colonne 4 ; la hache au-dessus
// de son manche, colonne 6) : l'épée elfique à lame en feuille, garde recourbée et pommeau de saphir, ses runes sur la
// lame ; la hache naine à barbe incrustée d'or ; la dague au pommeau de rubis ; la masse à ailettes au cœur de saphir.
const EPEE: [&str; 28] = [
    "...kkk...",
    "..kGBGk..",
    "..kgGgk..",
    "...kGk...",
    "...kLk...",
    "...klk...",
    "...kLk...",
    "...klk...",
    "...kLk...",
    "kk.kgk.kk",
    "kGkkGkkgk",
    "kgGGBGGyk",
    ".kkgGykk.",
    "...kHk...",
    "..kHSsk..",
    "..kHusk..",
    "..kHSsk..",
    "..kHSsk..",
    ".kHHusdk.",
    ".kHHSssk.",
    ".kHHSssk.",
    ".kHHusdk.",
    "..kHSsk..",
    "..kHSsk..",
    "...kHsk..",
    "...kHk...",
    "...kSk...",
    "....k....",
];
const HACHE: [&str; 26] = [
    "......k..",
    ".....kGk.",
    ".....kgk.",
    ".....kWk.",
    ".kk..kwk.",
    "kHSk.kWk.",
    "kHSskkWkk",
    "kHSsddWdk",
    "kHSgGdWdk",
    "kHSsgdWdk",
    "kHSssdWk.",
    ".kHSsdWk.",
    "..kHsdWk.",
    "...kkdwk.",
    "....kkWk.",
    ".....kwk.",
    ".....kWk.",
    ".....kwk.",
    ".....kWk.",
    ".....kLk.",
    ".....klk.",
    ".....kLk.",
    ".....klk.",
    ".....kgk.",
    ".....kGk.",
    "......k..",
];
const DAGUE: [&str; 18] = [
    "...kkk...",
    "..kRrRk..",
    "...krk...",
    "...kGk...",
    "...kLk...",
    "...klk...",
    "...kLk...",
    ".kkkgkkk.",
    "kGGGRGGyk",
    ".kkgGykk.",
    "...kHsk..",
    "...kHsk..",
    "...kHuk..",
    "...kHsk..",
    "...kHsk..",
    "...kHsk..",
    "...kHk...",
    "....k....",
];
const MASSE: [&str; 27] = [
    "...kkk...",
    "..kl.lk..",
    "..k...k..",
    "...klk...",
    "...kLk...",
    "...klk...",
    "...kLk...",
    "...klk...",
    "...kLk...",
    "..kgGyk..",
    "...kSk...",
    "...ksk...",
    "...kSk...",
    "...ksk...",
    "...kSk...",
    "..kgGyk..",
    "..kkkkk..",
    ".kHSkSsk.",
    "kHHSkSsdk",
    "kHSkBkSdk",
    "kHSBbBsdk",
    "kHSkBkSdk",
    "kHHSkSsdk",
    ".kHSkSsk.",
    "..kkSkk..",
    "...kdk...",
    "....k....",
];
const ANDURIL_GARDE: [&str; 9] = [
    "........kk..",
    ".......kGGk.",
    "kkk....kgGk.",
    "kBGkkkkkgGkk",
    "kGLlLlLlGBGk",
    "kgGkkkkkgGkk",
    "kkk....kgGk.",
    ".......kGyk.",
    "........kk..",
];


fn couleur_palette(lettre: u8, defaut: u32) -> [u8; 3] {
    PALETTE.get(lettre as usize).copied().flatten().map(rgb).unwrap_or_else(|| rgb(defaut))
}

fn peindre_banniere(c: &mut Canevas, b: &Banniere, clock: i64) {
    let u0 = BANNER.0 as f32 - 0.3; // 46.7
    let u1 = BANNER.0 as f32 + 5.7; // 52.7
    let v0 = BANNER.1 as f32 + 0.6; // 8.6
    let v1 = BANNER.1 as f32 + 9.2; // 17.2

    let drap = couleur_palette(b.cloth.0, 0x8a2432);
    let drap_ombre = echelle(drap, 0.78);
    let embleme = couleur_palette(b.emblem.0, 0xf2c94c);
    let or = rgb(0xdcb044);

    let (x0, x1) = (c.ax(u0), c.ax(u1));
    let (y0, y1) = (c.ay(v0), c.ay(v1));

    // Hampe horizontale de fer forgé à fleurons d'or
    c.rect(x0 - 2, y0 - 1, x1 + 2, y0 + 1, rgb(0x363a48), Mode::Sol);
    c.poser(x0 - 2, y0 - 1, or, Mode::Sol);
    c.poser(x1 + 1, y0 - 1, or, Mode::Sol);

    // Drap tissé bordé d'un galon d'or, animé d'une ondulation légère en bas
    let onde = ((clock.div_euclid(8) % 2) as i32) * 2 - 1;
    let xm = (x0 + x1) / 2;
    for ay in y0 + 1..=y1 {
        let t = (ay - y0) as f32 / (y1 - y0).max(1) as f32;
        let dec = if t > 0.7 { onde } else { 0 };
        for ax in x0..x1 {
            // Découpe en queue d'aronde au bas du pennon
            let encoche = ((ax - xm).abs() * 2) < (ay - (y1 - 3)) * 3;
            if ay >= y1 - 3 && encoche {
                continue;
            }
            let px = ax + dec;
            let col = if ax == x0 || ax == x1 - 1 {
                or
            } else if ax > xm + 1 {
                drap_ombre
            } else {
                drap
            };
            c.poser(px, ay, col, Mode::Sol);
        }
    }
    // Pompons de frange d'or aux deux pointes du bas
    c.poser(x0 + 1 + onde, y1 + 1, rgb(0xf4d264), Mode::Sol);
    c.poser(x1 - 2 + onde, y1 + 1, rgb(0xf4d264), Mode::Sol);

    // Blason symétrique `4 × 5` issu de `b.bits`
    let bx0 = xm - 4;
    let by0 = y0 + 3;
    for j in 0..5 {
        for i in 0..2 {
            if (b.bits >> (j * 2 + i)) & 1 != 0 {
                c.rect(bx0 + i as i32 * 2, by0 + j as i32 * 2, bx0 + i as i32 * 2 + 2, by0 + j as i32 * 2 + 2, embleme, Mode::Sol);
                c.rect(bx0 + (3 - i as i32) * 2, by0 + j as i32 * 2, bx0 + (3 - i as i32) * 2 + 2, by0 + j as i32 * 2 + 2, embleme, Mode::Sol);
            }
        }
    }
}

// ============================================================================
// F7 (suite) — Lanterne murale de fer forgé & Cloche de bronze oscillante
// ============================================================================

fn peindre_lanterne_murale(c: &mut Canevas, allumee: bool, clock: i64) {
    let lx = c.ax(LANTERN.0 as f32 + 1.2);
    let ly = c.ay(LANTERN.1 as f32 + 1.4);

    // Potence en volute de fer forgé scellée au mur
    c.rect(lx - 2, ly, lx + 2, ly + 1, rgb(0x2f3340), Mode::Sol);
    c.poser(lx, ly + 1, rgb(0x52586a), Mode::Sol);
    // Chapeau conique et cage hexagonale
    c.rect(lx - 2, ly + 2, lx + 3, ly + 3, CONTOUR, Mode::Sol);
    c.rect(lx - 1, ly + 1, lx + 2, ly + 2, rgb(0x4a5062), Mode::Sol);
    let vitre = if allumee { rgb(0xf49830) } else { rgb(0x586274) };
    let mode_v = if allumee { Mode::Lueur } else { Mode::Sol };
    c.boite(lx - 2, ly + 3, lx + 3, ly + 8, vitre, CONTOUR, mode_v);
    if allumee {
        let vif = if (clock % 5) < 3 { rgb(0xfff4a8) } else { rgb(0xffcf54) };
        c.rect(lx - 1, ly + 4, lx + 2, ly + 7, rgb(0xffbe42), Mode::Lueur);
        c.poser(lx, ly + 5, vif, Mode::Lueur);
    }
    c.rect(lx - 1, ly + 8, lx + 2, ly + 9, CONTOUR, Mode::Sol);
}

fn peindre_cloche_bronze(c: &mut Canevas, bell_age: Option<i64>) {
    let bx = c.ax(BELL_AT.0 as f32 + 1.6);
    let by = c.ay(1.2);
    let sonne = bell_age.filter(|&a| (0..40).contains(&a));
    let osc = sonne.map_or(0, |a| ((a as f32 * 0.7).sin() * 2.0).round() as i32);

    // Joug de chêne fixé sous la poutre
    c.rect(bx - 2, by - 1, bx + 3, by + 1, rgb(0x5c3a20), Mode::Sol);
    // Robe évasée de bronze doré
    let cx = bx + osc;
    c.boite(cx - 2, by + 1, cx + 3, by + 4, rgb(0xc89638), CONTOUR, Mode::Sol);
    c.boite(cx - 3, by + 3, cx + 4, by + 6, rgb(0xdea844), CONTOUR, Mode::Sol);
    c.rect(cx - 1, by + 2, cx + 1, by + 5, rgb(0xf6d472), Mode::Sol);
    // Battant de bronze
    c.poser(cx + osc.signum(), by + 6, rgb(0x8c6424), Mode::Sol);

    // Ondes sonores dorées quand la cloche tinte
    if let Some(a) = sonne {
        if a % 4 < 2 {
            c.poser(cx - 5, by + 3, rgb(0xffe680), Mode::Lueur);
            c.poser(cx + 5, by + 3, rgb(0xffe680), Mode::Lueur);
            c.poser(cx - 6, by + 4, rgb(0xf4be48), Mode::Lueur);
            c.poser(cx + 6, by + 4, rgb(0xf4be48), Mode::Lueur);
        }
    }
}

// ============================================================================
// Machine à vapeur murale (touche steampunk) : conduite de cuivre, manomètre, sifflet et deux engrenages
// ============================================================================

/// Dans le coin droit, sous la bannière (`u = 48.5..54, v = 18..29`) : une conduite de cuivre monte du sol jusqu'à un
/// sifflet de laiton, un manomètre y est vissé, et deux engrenages engrenés tournent au mur. La pression suit le feu
/// et le soufflet : l'aiguille monte, les roues accélèrent, et le sifflet crache des bouffées de vapeur.
fn peindre_machine_vapeur(c: &mut Canevas, w: &Monde, p: &Pose, tick: i64) {
    let feu = if w.exhausted { 0 } else { p.fire.clamp(0, 4) };
    let pression = (feu as f32 / 4.0 * 0.7 + if p.bellows { 0.15 } else { 0.0 } + if w.forging { 0.12 } else { 0.0 }).min(0.97);
    let t = tick as f32;
    let pi = std::f32::consts::PI;

    // 1. La conduite de cuivre, du sol au sifflet, cerclée de brides rivées
    let px = c.ax(52.6);
    let (haut, sol) = (c.ay(17.8), c.ay(29.2));
    for y in haut..sol {
        for (i, col) in [CONTOUR, rgb(0xeeac70), rgb(0xc4743e), rgb(0x84442a), CONTOUR].iter().enumerate() {
            c.poser(px - 2 + i as i32, y, *col, Mode::Sol);
        }
    }
    let pas = c.ay(2.4).max(6);
    let mut y = sol - pas / 2;
    while y > haut + 3 {
        c.boite(px - 3, y, px + 4, y + 3, rgb(0x9a6430), CONTOUR, Mode::Sol);
        c.poser(px - 2, y + 1, rgb(0xf0c468), Mode::Sol);
        y -= pas;
    }

    // 2. Le sifflet de laiton au bout de la conduite
    c.boite(px - 3, haut - 5, px + 4, haut + 1, rgb(0xd4a446), CONTOUR, Mode::Sol);
    c.rect(px - 2, haut - 4, px - 1, haut, rgb(0xf8dc84), Mode::Sol);
    c.boite(px - 1, haut - 8, px + 2, haut - 4, rgb(0xb88a36), CONTOUR, Mode::Sol);

    // 3. Le volant de la vanne, rouge, à cinq rayons, vissé sur la conduite
    let (vx, vy) = (px, c.ay(24.6));
    c.disque(vx, vy, 3, CONTOUR, Mode::Sol);
    c.disque(vx, vy, 2, rgb(0xc03028), Mode::Sol);
    c.disque(vx, vy, 1, rgb(0x5a1414), Mode::Sol);
    c.poser(vx - 1, vy - 2, rgb(0xff7a5a), Mode::Sol);
    c.poser(vx, vy, rgb(0xe8c060), Mode::Sol);

    // 4. Le manomètre vissé à la conduite : cadran crème, zone rouge, aiguille qui tremble sous la pression
    let (gx, gy) = (px, c.ay(20.2));
    let gr = c.ay(1.0).max(4);
    c.disque(gx, gy, gr + 1, CONTOUR, Mode::Sol);
    c.disque(gx, gy, gr, rgb(0xc89838), Mode::Sol);
    c.disque(gx, gy, gr - 1, rgb(0xf0e6cc), Mode::Sol);
    c.poser(gx - gr + 1, gy - 1, rgb(0xfff8e8), Mode::Sol);
    c.poser(gx + gr - 2, gy - gr + 2, rgb(0xd03030), Mode::Sol);
    c.poser(gx + gr - 1, gy - 1, rgb(0xd03030), Mode::Sol);
    let frisson = 0.04 * (t * 1.9).sin() * (0.3 + pression);
    let a = pi * 0.75 + (pression + frisson).clamp(0.0, 1.0) * pi * 1.5;
    let l = (gr - 1) as f32;
    c.ligne(gx, gy, gx + (a.cos() * l).round() as i32, gy + (a.sin() * l).round() as i32, rgb(0xb01818), Mode::Sol);
    c.poser(gx, gy, CONTOUR, Mode::Sol);

    // 5. Deux engrenages engrenés, laiton et cuivre ; la vitesse suit la pression (immobiles feu éteint)
    let rb = c.ay(1.9).max(6);
    let rs = ((rb as f32) * 0.62).round() as i32;
    let (nb, ns) = (8.0f32, 5.0f32);
    let (bx, by) = (c.ax(49.3), c.ay(21.9));
    let theta = 1.3f32; // l'axe des centres, vers le bas, un peu à droite
    let d = rb as f32 + rs as f32 + 1.6;
    let (sx, sy) = (bx + (theta.cos() * d).round() as i32, by + (theta.sin() * d).round() as i32);
    let vitesse = if feu == 0 && !w.forging { 0.0 } else { 0.012 + 0.06 * pression };
    let rot_b = t * vitesse;
    let phase_b = (theta - rot_b) * nb / TAU;
    let rot_s = theta + pi - (1.0 - phase_b) * TAU / ns;
    engrenage(c, sx, sy, rs, ns, rot_s, rgb(0xc8804a), 0);
    engrenage(c, bx, by, rb, nb, rot_b, rgb(0xd8a84c), 4);

    // 6. Bouffées de vapeur au sifflet quand la pression monte : elles s'élèvent, s'élargissent et s'effacent
    if pression > 0.3 {
        let periode = (44.0 - 26.0 * pression) as i64;
        for k in 0..3i64 {
            let age = (tick + k * periode / 3).rem_euclid(periode) as f32;
            let vie = age / periode as f32;
            let yy = haut - 9 - (age * 0.5) as i32;
            let xx = px - (vie * 6.0 + (age * 0.25 + k as f32 * 2.0).sin() * 1.3).round() as i32;
            let r = 1.2 + vie * 2.4;
            let ri = r.ceil() as i32;
            for dy in -ri..=ri {
                for dx in -ri..=ri {
                    let q = ((dx * dx + dy * dy) as f32).sqrt() / r;
                    if q > 1.0 || (vie > 0.7 && hash_f(k * 977 + dx as i64 * 31 + dy as i64 * 7 + tick / 3) < (vie - 0.7) * 3.0) {
                        continue;
                    }
                    let col = melange(rgb(0xfafcff), rgb(0x9aa0b0), vie * 0.8 + q * 0.25 + if dy > 0 { 0.12 } else { 0.0 });
                    c.poser(xx + dx, yy + dy, col, Mode::Sol);
                }
            }
        }
    }
}

/// Un engrenage de `n` dents carrées au rayon `r` tourné de `rot`, éclairé d'en haut à gauche, à `trous` ajours ronds.
#[allow(clippy::too_many_arguments)]
fn engrenage(c: &mut Canevas, cx: i32, cy: i32, r: i32, n: f32, rot: f32, metal: [u8; 3], trous: i32) {
    let rf = r as f32;
    let largeur = (pi_r(rf) / n * 0.5).max(1.0); // la demi-largeur d'une dent
    for dy in -(r + 4)..=(r + 4) {
        for dx in -(r + 4)..=(r + 4) {
            let (fx, fy) = (dx as f32, dy as f32);
            let dist = (fx * fx + fy * fy).sqrt();
            // la dent la plus proche, un rectangle dressé sur la jante
            let a = fy.atan2(fx) - rot;
            let k = (a * n / TAU - 0.25).round();
            let ac = rot + (k + 0.25) * TAU / n;
            let (le_long, en_travers) = (fx * ac.cos() + fy * ac.sin(), -fx * ac.sin() + fy * ac.cos());
            let dent = le_long > rf - 1.0 && le_long <= rf + 2.0 && en_travers.abs() <= largeur;
            let dent_bord = le_long > rf - 1.0 && le_long <= rf + 3.0 && en_travers.abs() <= largeur + 1.0;
            let corps = dist <= rf + 0.3;
            if !(corps || dent) {
                if dent_bord || dist <= rf + 1.3 {
                    c.poser(cx + dx, cy + dy, CONTOUR, Mode::Sol);
                }
                continue;
            }
            let lum = 1.0 + 0.3 * (-(fx + fy) / (rf * 1.4)).clamp(-1.0, 1.0);
            let mut col = echelle(metal, lum);
            if corps && dist > rf - 1.2 && !dent {
                col = echelle(metal, 0.72 * lum); // la jante
            }
            if dent && le_long > rf + 1.2 && en_travers < 0.0 {
                col = echelle(metal, 1.2 * lum); // l'arête éclairée des dents
            }
            if dist < 1.3 {
                col = rgb(0x2a2630); // l'axe
            } else if dist < 2.6 {
                col = echelle(metal, 1.28); // le moyeu
            }
            for j in 0..trous {
                let at = rot + (j as f32 + 0.5) * TAU / trous as f32;
                let (hx, hy) = (at.cos() * rf * 0.56, at.sin() * rf * 0.56);
                let dh = ((fx - hx).powi(2) + (fy - hy).powi(2)).sqrt();
                if dh < rf * 0.24 {
                    col = echelle(metal, 0.28);
                } else if dh < rf * 0.24 + 0.9 && (fy - hy) > 0.0 {
                    col = echelle(metal, 1.15 * lum); // le bord éclairé de l'ajour, en bas
                }
            }
            c.poser(cx + dx, cy + dy, col, Mode::Sol);
        }
    }
}

/// `π r`, la demi-circonférence.
fn pi_r(r: f32) -> f32 {
    std::f32::consts::PI * r
}

// ============================================================================
// F3 (dynamique) — Runes naines, bûches, flammes, fumée, bouilloire & soufflet
// ============================================================================

fn peindre_foyer_et_soufflet(c: &mut Canevas, w: &Monde, p: &Pose, tick: i64) {
    let feu = if w.exhausted { 0 } else { p.fire.clamp(0, 4) };

    // 1. Quatre runes angulaires de Khazad-dûm gravées dans le linteau (`0..=4` allumées selon `feu`) : des
    // glyphes à hampe et branches obliques, à la manière des Cirth, jamais des croix
    const RUNES: [[&str; 5]; 4] = [
        ["X.X", "XX.", "X..", "XX.", "X.X"],
        [".X.", "X.X", ".X.", ".X.", ".X."],
        ["X.X", ".X.", "X.X", ".X.", "X.X"],
        ["XX.", "X.X", "XX.", "X.X", "XX."],
    ];
    let ly = c.ay(16.1);
    for (idx, &ru) in [3.2f32, 5.7, 8.2, 10.6].iter().enumerate() {
        let rx = c.ax(ru) - 1;
        let active = (idx as i32) < feu;
        let (col, mode) = if active {
            let pulse = ((tick + idx as i64 * 3) % 6) < 3;
            (if pulse { rgb(0xffd85c) } else { rgb(0xff9a30) }, Mode::Lueur)
        } else {
            (rgb(0x262130), Mode::Plein)
        };
        for (j, ligne) in RUNES[idx].iter().enumerate() {
            for (i, b) in ligne.bytes().enumerate() {
                if b == b'X' {
                    c.poser(rx + i as i32, ly + j as i32, col, mode);
                }
            }
        }
    }

    // 2. Réserve hebdomadaire de bûches (`world.fuel7d`, `0..=10`) : des rondins vus en bout, empilés en pyramide sur
    // une planche à consoles de fer scellée à gauche de la hotte
    let pile = match w.fuel7d {
        None => 10usize,
        Some(f) => ((10.0 * (1.0 - f.min(100.0) / 100.0)).round() as i32).clamp(0, 10) as usize,
    };
    let planche_y = c.ay(14.9);
    let (px0, px1) = (c.ax(0.0), c.ax(5.6));
    c.boite(px0, planche_y, px1, planche_y + 2, rgb(0x7a4c2a), CONTOUR, Mode::Plein);
    c.rect(px0, planche_y, px1 - 1, planche_y + 1, rgb(0x96643a), Mode::Plein);
    for &kx in &[px0 + 3, px1 - 4] {
        c.ligne(kx, planche_y + 2, kx + 2, planche_y + 4, rgb(0x2e3240), Mode::Plein);
    }
    const RONDIN: [&str; 5] = [".bbb.", "bttTb", "btctb", "bTttb", ".bbb."];
    let rang_x = |k: usize| -> (i32, i32) {
        // quatre rondins, puis trois, deux et un au sommet ; cinq pixels d'art de large, jointifs
        let (rang, i) = match k {
            0..=3 => (0, k as i32),
            4..=6 => (1, (k - 4) as i32),
            7..=8 => (2, (k - 7) as i32),
            _ => (3, 0),
        };
        (px0 + 1 + rang * 5 / 2 + i * 5, planche_y - 5 - rang * 4)
    };
    for k in 0..pile {
        let (bx, by) = rang_x(k);
        for (j, ligne) in RONDIN.iter().enumerate() {
            for (i, b) in ligne.bytes().enumerate() {
                let col = match b {
                    b'b' => rgb(0x4a2e1a),
                    b't' => rgb(0xc89a62),
                    b'T' => rgb(0xe0b67c),
                    b'c' => rgb(0x8a5e34),
                    _ => continue,
                };
                c.poser(bx + i as i32, by + j as i32, col, Mode::Plein);
            }
        }
    }

    // 3. L'âtre (`world.fuel5h`, `0..=8` bûches) : la brique qui rougit, les flammes qui montent DERRIÈRE les bûches
    // couchées (le bois se découpe en sombre sur le feu), le lit de braises dessous
    let logs = match w.fuel5h {
        None => 8usize,
        Some(f) => ((8.0 * (1.0 - f.min(100.0) / 100.0)).round() as i32).clamp(0, 8) as usize,
    };
    let (mx0, mx1) = (c.ax(MOUTH.0 as f32), c.ax((MOUTH.0 + MOUTH.2) as f32));
    let my_sol = c.ay(26.2);
    let haut0 = c.ay(MOUTH.1 as f32);
    let t = tick as f32 * 0.55;

    // la brique réfractaire rougit au-dessus du feu
    if feu > 0 {
        for ay in haut0..my_sol {
            let k = ((ay - haut0) as f32 / (my_sol - haut0).max(1) as f32).powf(1.6) * (0.35 + 0.15 * feu as f32);
            for ax in mx0..mx1 {
                let centre = 1.0 - ((ax as f32 - (mx0 + mx1) as f32 * 0.5).abs() / ((mx1 - mx0) as f32 * 0.5)).min(1.0);
                let a = (k * (0.5 + 0.5 * centre)).min(0.85);
                if a > 0.08 {
                    let fond = c.buf[(ay * c.aw + ax) as usize].rgb;
                    c.poser(ax, ay, melange(fond, rgb(0x8a2a12), a), Mode::Lueur);
                }
            }
        }
    }

    // les langues de flamme : cinq fuseaux qui s'effilent et ondulent, plus hauts au centre ; le cœur blanc-or, puis
    // l'or, l'orange et un liseré vermeil au bord
    if feu > 0 {
        let h_max = (my_sol - c.ay(MOUTH.1 as f32 + 0.5)) as f32;
        let large = (mx1 - mx0) as f32;
        let lw = (mx1 - mx0) as usize;
        let mut chaleur = vec![0.0f32; lw * (my_sol - haut0).max(0) as usize];
        let langues = 5;
        for i in 0..langues {
            let fi = i as f32;
            let place = (fi + 0.5) / langues as f32;
            let centre = 1.0 - (place - 0.5).abs() * 1.3;
            let souffle = (t * 1.7 + fi * 2.1).sin() * 0.12 + (t * 2.9 + fi * 1.3).sin() * 0.08;
            let h = h_max * (0.4 + 0.6 * feu as f32 / 4.0) * (0.55 + 0.45 * centre + souffle).clamp(0.35, 1.0);
            let cx = mx0 as f32 + large * place + (t * 1.1 + fi).sin() * 0.8;
            let bw = large / langues as f32 * 0.85 + 1.0;
            for dy in 0..(h as i32) {
                let s = dy as f32 / h;
                let demi = bw * (1.0 - s).powf(0.8) * (0.7 + 0.3 * (s * 5.0).min(1.0));
                let x_c = cx + (t * 2.3 + fi * 1.3 + s * 4.0).sin() * s * 2.6;
                let ay = my_sol - 1 - dy;
                if ay < haut0 {
                    break;
                }
                for ax in (x_c - demi).floor() as i32..=(x_c + demi).ceil() as i32 {
                    if ax < mx0 || ax >= mx1 {
                        continue;
                    }
                    let dx = (ax as f32 + 0.5 - x_c).abs() / demi.max(0.5);
                    if dx > 1.0 {
                        continue;
                    }
                    let q = (1.0 - dx) * 0.75 + (1.0 - s) * 0.55 - 0.1;
                    let j = (ay - haut0) as usize * lw + (ax - mx0) as usize;
                    if q > chaleur[j] {
                        chaleur[j] = q;
                    }
                }
            }
        }
        for ay in haut0..my_sol {
            for ax in mx0..mx1 {
                let q = chaleur[(ay - haut0) as usize * lw + (ax - mx0) as usize];
                if q <= 0.0 {
                    continue;
                }
                let col = if q > 0.86 {
                    rgb(0xfff6c4)
                } else if q > 0.66 {
                    rgb(0xffd84a)
                } else if q > 0.42 {
                    rgb(0xffa02a)
                } else if q > 0.2 {
                    rgb(0xf0601c)
                } else {
                    rgb(0xb02a18)
                };
                c.poser(ax, ay, col, Mode::Lueur);
            }
        }
    }

    // le lit de braises
    for ax in mx0..mx1 {
        for (k, ay) in [my_sol, my_sol + 1].into_iter().enumerate() {
            let vif = hash_f(ax as i64 * 7 + k as i64 * 3 + tick / 3) > 0.55;
            let br = match (feu == 0, vif) {
                (true, true) => rgb(0x7a2618),
                (true, false) => rgb(0x3a1814),
                (false, true) => rgb(0xffb040),
                (false, false) => rgb(0xd04a1c),
            };
            c.poser(ax, ay, if k == 1 { echelle(br, 0.75) } else { br }, Mode::Lueur);
        }
    }

    // les bûches : deux couchées au fond, puis d'autres croisées en tipi par-dessus (dans l'ordre où elles brûlent : les
    // dernières de la liste partent les premières) ; écorce sombre cernée, bout coupé clair, braises sur le dessus
    const BUCHES: [(i32, i32, i32, i32); 8] = [
        (1, 14, 0, 0),
        (21, 34, 0, 0),
        (10, 25, 1, 1),
        (3, 15, 2, 7),
        (33, 21, 2, 7),
        (13, 24, 5, 5),
        (8, 17, 7, 11),
        (28, 19, 7, 11),
    ];
    let echelle_x = (mx1 - mx0) as f32 / 36.0;
    for (i, &(xa, xb, ya, yb)) in BUCHES.iter().enumerate().take(logs) {
        let (xa, xb) = (mx0 + (xa as f32 * echelle_x) as i32, mx0 + (xb as f32 * echelle_x) as i32);
        let (g, d) = (xa.min(xb), xa.max(xb));
        for x in g..=d {
            let f = if xb == xa { 0.0 } else { (x - xa) as f32 / (xb - xa) as f32 };
            let y = my_sol - 2 - (ya as f32 + (yb - ya) as f32 * f).round() as i32;
            c.poser(x, y - 2, CONTOUR, Mode::Plein);
            // sombres : le feu qui les éclaire par derrière les ferait virer à l'orange
            c.poser(x, y - 1, rgb(0x4a2c18), Mode::Plein);
            c.poser(x, y, rgb(0x2a1810), Mode::Plein);
            c.poser(x, y + 1, CONTOUR, Mode::Plein);
            if feu > 0 && hash_f(x as i64 * 13 + i as i64 * 5 + tick / 2) > 0.82 {
                c.poser(x, y - 1, rgb(0xff8a2a), Mode::Lueur);
            }
        }
        // le bout coupé, clair, du côté où la bûche finit
        let yb_ = my_sol - 2 - yb;
        c.poser(xb, yb_ - 1, rgb(0xd8aa6c), Mode::Plein);
        c.poser(xb, yb_, rgb(0xb08048), Mode::Plein);
    }

    // de petites langues devant les bûches, qui lèchent le bois
    if feu >= 2 {
        for i in 0..3 {
            let lx = mx0 + 4 + i * (mx1 - mx0 - 8) / 2;
            let hh = 2 + ((t * 2.0 + i as f32 * 2.4).sin() * 1.5 + 1.5) as i32;
            for dy in 0..hh {
                c.poser(lx, my_sol - 5 - dy, if dy == 0 { rgb(0xffd84a) } else { rgb(0xffa02a) }, Mode::Lueur);
            }
        }
    }

    // Escarbilles montantes quand le foyer rugit (`p.embers`)
    if p.embers && feu > 0 {
        for i in 0..4i64 {
            let age = (tick + i * 3).rem_euclid(9);
            let eu = 4.2 + hash_f(i * 17 + (tick + i * 3).div_euclid(9)) * 5.2;
            let ev = 24.5 - age as f32 * 0.85;
            if ev > 18.2 {
                c.poser(c.ax(eu), c.ay(ev), if age < 4 { rgb(0xffe868) } else { rgb(0xff6e24) }, Mode::Lueur);
            }
        }
    }

    // Volutes légères de fumée sous la hotte (`p.smoke_every`, quand la cheminée tire)
    if !w.exhausted && p.smoke_every > 0 {
        for k in 0..3i64 {
            let ph = (tick + k * 5).rem_euclid(14);
            let su = 5.8 + ((ph as f32 * 0.35 + k as f32).sin()) * 0.9;
            let sv = 8.0 - ph as f32 * 0.48;
            if sv > 1.5 {
                c.poser(c.ax(su), c.ay(sv), rgb(0x7c788a), Mode::Mur);
            }
        }
    }

    // 5. Bouilloire de cuivre qui mijote au bord de l'âtre (`w.tea`, hors gorgée)
    if w.tea && p.right != Droite::Tea {
        let (kx, ky) = (c.ax(11.4), c.ay(26.4));
        c.disque(kx, ky, 2, CONTOUR, Mode::Plein);
        c.disque(kx, ky, 1, rgb(0xd0783c), Mode::Plein);
        c.poser(kx, ky - 2, rgb(0x8890a4), Mode::Plein);
        let vap = (tick.div_euclid(3) % 4) as i32;
        c.poser(kx + 1, ky - 3 - vap, rgb(0xdce2ee), Mode::Sol);
    }

    // 6. Le soufflet, accroché au flanc du four, vu de face : une poire de chêne cernée de clous de laiton, sa soupape
    // de cuir au milieu, sa buse de laiton qui entre dans le four, sa poignée percée à droite ; le cuir plissé dépasse
    // dessous, épais quand il est plein d'air, mince quand Scorpheus l'écrase (`p.bellows`). Quand il le relâche, la
    // planche remonte avec sa patte (la rangée `Soufflet` du pet : la patte haute, puis basse sur la poignée) et le cuir
    // s'étire dessous en se remplissant d'air ; la buse, fixée au four, ne bouge pas.
    let by = c.ay(23.0);
    let leve = if p.left == Gauche::Pull && !p.bellows { 2.4 } else { 0.0 };
    let leve_px = (by - c.ay(23.0 - leve)) as f32;
    let (nez, cx, poignee) = (c.ax(12.6), c.ax(16.3), c.ax(18.9));
    let r = ((poignee - cx) as f32 * 0.62).max(4.0);
    let rx_cone = nez + 2;
    let demi = |ax: i32| -> f32 {
        let dx = (ax - cx) as f32;
        let rond = if dx.abs() <= r { (r * r - dx * dx).sqrt() } else { -1.0 };
        let cone = if ax >= rx_cone && ax <= cx { 1.6 + (ax - rx_cone) as f32 / (cx - rx_cone).max(1) as f32 * (r * 0.8 - 1.6) } else { -1.0 };
        rond.max(cone)
    };
    let cuir = if p.bellows { 1 } else { 3 };
    // la buse : un cône de laiton qui s'effile vers le feu
    for ax in c.ax(11.4)..=rx_cone {
        let e = if ax < nez - 1 { 0 } else { 1 };
        c.rect(ax, by - 1 - e, ax + 1, by + 2 + e, CONTOUR, Mode::Plein);
        c.poser(ax, by - e, rgb(0xe0b04c), Mode::Plein);
        c.poser(ax, by + 1, rgb(0xa8782c), Mode::Plein);
    }
    let fin = cx + r as i32;
    // la planche pivote sur la buse : elle se lève d'autant plus qu'on va vers la poignée
    let haut_de = |ax: i32| -> i32 { by - (leve_px * ((ax - rx_cone) as f32 / (poignee - rx_cone).max(1) as f32).clamp(0.0, 1.0)).round() as i32 };
    let bb = haut_de(cx);
    let bp = haut_de(poignee);
    // le cuir qui dépasse sous la planche, plissé
    for ax in rx_cone..=fin {
        let d = demi(ax);
        if d < 0.0 {
            continue;
        }
        // le cuir en accordéon : des plis horizontaux, un trait sombre tous les trois, un reflet au-dessus
        let (y0, y1) = (haut_de(ax) + d.round() as i32, by + d.round() as i32 + cuir);
        for y in y0..=y1 {
            let col = if y == y1 || ax == fin || ax == rx_cone {
                CONTOUR
            } else {
                match (y1 - y).rem_euclid(3) {
                    0 => rgb(0x3a1a18),
                    1 => rgb(0x6a2e26),
                    _ => rgb(0x8a4232),
                }
            };
            c.poser(ax, y, col, Mode::Plein);
        }
    }
    // la planche de chêne, éclairée du côté du feu (en haut, à gauche), son contour net
    for ax in rx_cone..=fin {
        let d = demi(ax);
        if d < 0.0 {
            continue;
        }
        let dr = d.round() as i32;
        let voisin = |x: i32| { let e = demi(x); if e < 0.0 { -1 } else { e.round() as i32 } };
        let (dg, dd) = (voisin(ax - 1), voisin(ax + 1));
        let bc = haut_de(ax);
        for ay in bc - dr..=bc + dr {
            let dy = (ay - bc).abs();
            let bord = dy == dr || dy > dg || dy > dd;
            let col = if bord {
                CONTOUR
            } else {
                let fil = ((ay * 3 + ax / 4) % 5) == 0;
                let clair = ((ay - bc) as f32) < -d * 0.35 || ax < cx - (r * 0.5) as i32;
                match (clair, fil) {
                    (true, false) => rgb(0x9a6638),
                    (true, true) => rgb(0x84542c),
                    (false, false) => rgb(0x7a4c28),
                    (false, true) => rgb(0x643c20),
                }
            };
            c.poser(ax, ay, col, Mode::Plein);
        }
    }
    // les clous de laiton autour de la planche
    for (k, ax) in (rx_cone + 2..fin - 1).enumerate() {
        let d = demi(ax);
        if d >= 2.0 && k % 3 == 0 {
            c.poser(ax, haut_de(ax) - d.round() as i32 + 1, rgb(0xf0c860), Mode::Plein);
            c.poser(ax, haut_de(ax) + d.round() as i32 - 1, rgb(0xc8983c), Mode::Plein);
        }
    }
    // la soupape : un rond de cuir cousu au milieu
    c.disque(cx, bb, 2, rgb(0x5a2622), Mode::Plein);
    c.poser(cx - 1, bb - 1, rgb(0x8a3e30), Mode::Plein);
    c.poser(cx, bb, CONTOUR, Mode::Plein);
    // la poignée, percée, qui dépasse à droite
    c.boite(fin - 1, bp - 2, poignee + 1, bp + 3, rgb(0x8a5a32), CONTOUR, Mode::Plein);
    c.rect(poignee - 2, bp, poignee, bp + 1, rgb(0x2a1a12), Mode::Plein);
    // le souffle : des étincelles qui jaillissent de la buse dans le feu
    if p.bellows && feu > 0 {
        for i in 0..3i64 {
            let a = (tick + i * 2).rem_euclid(6) as i32;
            c.poser(c.ax(11.0) - a * 2, by - (i as i32 - 1) * a / 2, if a < 3 { rgb(0xfff2a0) } else { rgb(0xffa83a) }, Mode::Lueur);
        }
    }
}

// ============================================================================
// F4 (dynamique), F5, F6 (sabliers), E4 — Enclume, coffre, canard, trésor, etc.
// ============================================================================

fn peindre_enclume_et_accessoires(c: &mut Canevas, w: &Monde, p: &Pose, tick: i64) {
    let table_y = c.ay(ANVIL_AT.1 as f32);
    let caisse_active = p.crate_.is_some_and(|st| st != Caisse::Gone);

    // 1. Lingot incandescent sur la table de l'enclume (`p.ingot`)
    if p.ingot != Lingot::None && !caisse_active {
        let (ix0, ix1) = (c.ax(21.0), c.ax(25.0));
        let (fond_l, coeur_l) = match p.ingot {
            Lingot::Flash => (rgb(0xffe868), rgb(0xffffff)),
            Lingot::Hot => (rgb(0xf46e22), rgb(0xffcf4c)),
            Lingot::Cold | Lingot::None => (rgb(0x9e3424), rgb(0xd85828)),
        };
        c.boite(ix0, table_y - 3, ix1, table_y + 1, fond_l, CONTOUR, Mode::Lueur);
        c.rect(ix0 + 1, table_y - 2, ix1 - 1, table_y - 1, coeur_l, Mode::Lueur);
    }

    // Les runes de l'enclume s'éveillent quand il forge, d'un blanc de lune comme l'ithildin ; chaque coup les fait
    // flamboyer, puis elles retombent à une lueur qui respire
    let coup = p.sparks_age.filter(|&a| a < 6);
    if matches!(p.ingot, Lingot::Hot | Lingot::Flash) || coup.is_some() {
        let eclat = coup.map_or(0.0, |a| 1.0 - a as f32 / 6.0);
        let souffle = 0.5 + 0.5 * ((tick as f32) * 0.35).sin();
        let k = (0.35 + 0.25 * souffle + 0.4 * eclat).min(1.0);
        let col = melange([0x46, 0x6c, 0x9a], [0xe8, 0xfa, 0xff], k);
        for (x, y) in runes_enclume(c) {
            c.poser(x, y, col, Mode::Lueur);
        }
    }

    // Marteau posé sur l'enclume pendant que Scorpheus actionne le soufflet (`Gauche::Pull`)
    if p.left == Gauche::Pull && !caisse_active {
        let hx = c.ax(21.2);
        c.ligne(hx - 2, table_y - 1, hx + 2, table_y - 3, rgb(0x7c4c2a), Mode::Plein);
        c.boite(hx + 1, table_y - 4, hx + 5, table_y - 1, rgb(0x949eb4), CONTOUR, Mode::Plein);
    }

    // 2. F6 — Sabliers actifs de tâches parallèles (`w.hourglasses = 1..=3`)
    if w.hourglasses > 0 && !caisse_active {
        let nb = w.hourglasses.clamp(0, 3) as usize;
        for &hu in HOURGLASS_X.iter().take(nb) {
            let hx = c.ax(hu as f32 + 1.2);
            let hy1 = table_y;
            let hy0 = table_y - 7;
            // Monture de laiton haut et bas
            c.rect(hx - 2, hy0, hx + 3, hy0 + 1, rgb(0xdcb048), Mode::Plein);
            c.rect(hx - 2, hy1 - 1, hx + 3, hy1, rgb(0xdcb048), Mode::Plein);
            // Deux ampoules de verre et filet de sable d'or
            c.boite(hx - 2, hy0 + 1, hx + 3, hy0 + 4, rgb(0x7ca6bc), CONTOUR, Mode::Plein);
            c.boite(hx - 2, hy0 + 4, hx + 3, hy1 - 1, rgb(0x7ca6bc), CONTOUR, Mode::Plein);
            c.poser(hx, hy0 + 2, rgb(0xf4c84c), Mode::Lueur);
            if (tick + hu as i64) % 2 == 0 {
                c.poser(hx, hy0 + 4, rgb(0xffe478), Mode::Lueur);
            }
            c.rect(hx - 1, hy1 - 2, hx + 2, hy1 - 1, rgb(0xf4c84c), Mode::Lueur);
        }
    } else if w.exhausted && !caisse_active {
        // 3. E4 — Sablier ancien couché au repos sur l'enclume (tout le sable d'or endormi)
        let hx0 = c.ax(19.0);
        let hx1 = c.ax(24.6);
        let hy0 = table_y - 5;
        c.rect(hx0, hy0, hx0 + 1, table_y, rgb(0xd4a440), Mode::Plein);
        c.rect(hx1 - 1, hy0, hx1, table_y, rgb(0xd4a440), Mode::Plein);
        let hm = (hx0 + hx1) / 2;
        c.boite(hx0 + 1, hy0 + 1, hm, table_y, rgb(0x6e94aa), CONTOUR, Mode::Plein);
        c.boite(hm, hy0 + 1, hx1 - 1, table_y, rgb(0x6e94aa), CONTOUR, Mode::Plein);
        // Sable d'or déposé au fond des bulbes couchés
        c.rect(hx0 + 2, table_y - 2, hm - 1, table_y - 1, rgb(0xf2c44c), Mode::Lueur);
        c.poser(hm + 1, table_y - 2, rgb(0xd8a638), Mode::Plein);
    }

    // 4. Canard en plastique jaune en pixel art `2×2` perché au bout droit de l'enclume (`w.duck`)
    if w.duck && !caisse_active {
        let dx = c.ax(DUCK_AT.0 as f32 + 1.8);
        let dy = table_y - 1;
        // Corps rond jaune vif + queue relevée
        c.boite(dx - 3, dy - 3, dx + 3, dy + 1, rgb(0xf8d230), CONTOUR, Mode::Plein);
        c.poser(dx + 2, dy - 3, rgb(0xf8d230), Mode::Plein);
        // Aile ciselée d'ambre doré
        c.rect(dx - 1, dy - 2, dx + 2, dy - 1, rgb(0xe0a81e), Mode::Plein);
        // Tête ronde, œil noir et bec orange tourné vers Scorpheus / le foyer
        c.boite(dx - 3, dy - 6, dx + 1, dy - 2, rgb(0xffe248), CONTOUR, Mode::Plein);
        c.poser(dx - 2, dy - 5, CONTOUR, Mode::Plein); // œil
        c.rect(dx - 5, dy - 4, dx - 3, dy - 3, rgb(0xf47820), Mode::Plein); // bec orange
    }

    // 5. F5 — Coffre runique de chêne ferré d'acier et d'or sur l'enclume (`p.crate_`)
    if let Some(etat_c) = p.crate_.filter(|&st| st != Caisse::Gone) {
        let (cx0, cx1) = (c.ax(CRATE_AT.0 as f32), c.ax((CRATE_AT.0 + 10) as f32));
        let cy1 = table_y + 1;
        let cy0 = table_y - 5;
        c.boite(cx0, cy0, cx1, cy1, rgb(0x6a4024), CONTOUR, Mode::Plein);
        // Ferrures d'angle en acier et bande runique d'or au milieu
        c.rect(cx0 + 1, cy0 + 1, cx0 + 3, cy1 - 1, rgb(0x7c8498), Mode::Plein);
        c.rect(cx1 - 3, cy0 + 1, cx1 - 1, cy1 - 1, rgb(0x7c8498), Mode::Plein);
        let cm = (cx0 + cx1) / 2;
        c.rect(cm - 2, cy0 + 1, cm + 2, cy1 - 1, rgb(0xdcb044), Mode::Plein);

        match etat_c {
            Caisse::Open => {
                // Couvercle dressé verticalement à gauche
                c.boite(cx0, cy0 - 7, cx0 + 3, cy0 + 1, rgb(0x7c4c2c), CONTOUR, Mode::Plein);
                c.poser(cx0 + 1, cy0 - 4, rgb(0xe4b84c), Mode::Plein);
            }
            Caisse::Closing => {
                // Couvercle incliné qui se rabat
                c.ligne(cx0, cy0, cx1 - 2, cy0 - 4, CONTOUR, Mode::Plein);
                c.ligne(cx0 + 1, cy0, cx1 - 2, cy0 - 3, rgb(0x885634), Mode::Plein);
            }
            Caisse::Shut | Caisse::Sealed => {
                c.boite(cx0 - 1, cy0 - 2, cx1 + 1, cy0 + 1, rgb(0x7c4c2c), CONTOUR, Mode::Plein);
                c.rect(cm - 2, cy0 - 1, cm + 2, cy0 + 1, rgb(0xe8be50), Mode::Plein);
            }
            Caisse::Gone => {}
        }

        // Sceau de cire vermeille frappé d'or
        if etat_c == Caisse::Sealed || p.seal_age.is_some_and(|s| s >= 0) {
            c.disque(cm - 1, cy0 + 1, 2, rgb(0xc8242c), Mode::Plein);
            c.poser(cm - 1, cy0 + 1, rgb(0xffd85c), Mode::Lueur);
        }
    }

    // 6. Baquet de trempe en douelles de chêne cerclé de fer (`p.bucket`) + vapeur (`p.steam`)
    if p.bucket {
        let (bx0, bx1) = (c.ax(BUCKET_AT.0 as f32), c.ax((BUCKET_AT.0 + 8) as f32));
        let (by0, by1) = (c.ay(BUCKET_AT.1 as f32), c.ay(31.8));
        c.boite(bx0, by0, bx1, by1, rgb(0x6e4426), CONTOUR, Mode::Plein);
        for ax in (bx0 + 2..bx1 - 1).step_by(3) {
            c.rect(ax, by0 + 2, ax + 1, by1 - 1, rgb(0x4c2d18), Mode::Plein);
        }
        // Eau vive miroitante en haut + deux cercles d'acier
        let eau = if p.left == Gauche::Dip { rgb(0xff9838) } else { rgb(0x52a8dc) };
        c.rect(bx0 + 1, by0 + 1, bx1 - 1, by0 + 2, eau, if p.left == Gauche::Dip { Mode::Lueur } else { Mode::Plein });
        c.rect(bx0, by0 + 2, bx1, by0 + 3, rgb(0x8892a6), Mode::Plein);
        c.rect(bx0, by1 - 3, bx1, by1 - 2, rgb(0x747e92), Mode::Plein);
    }
    if let Some(st) = p.steam {
        for i in 0..6i64 {
            let a = st as f32 - i as f32 * 0.7;
            if (0.0..=8.0).contains(&a) {
                let su = (BUCKET_AT.0 + 4) as f32 + (hash_f(i * 13) * 2.0 - 1.0) * 2.2 + (a + i as f32).sin() * 0.6;
                let sv = (BUCKET_AT.1 - 1) as f32 - a * 1.15;
                let vx = c.ax(su);
                let vy = c.ay(sv);
                let col = if a < 3.0 { rgb(0xeef4fc) } else { rgb(0x9ca4b8) };
                c.poser(vx, vy, col, Mode::Sol);
                if a < 4.5 {
                    c.poser(vx + 1, vy, rgb(0xbcc4d6), Mode::Sol);
                }
            }
        }
    }

    // 7. Tabouret d'atelier en chêne tourné (`p.stool`)
    if p.stool && !w.absent {
        let su0 = (C.0 + w.x + 2) as f32;
        let su1 = (C.0 + w.x + 14) as f32;
        let (sx0, sx1) = (c.ax(su0), c.ax(su1));
        let sy0 = c.ay((C.1 + 23) as f32);
        let sy1 = c.ay(31.6);
        c.boite(sx0, sy0, sx1, sy0 + 2, rgb(0x845230), CONTOUR, Mode::Sol);
        c.rect(sx0 + 2, sy0 + 2, sx0 + 4, sy1, rgb(0x683e22), Mode::Sol);
        c.rect(sx1 - 4, sy0 + 2, sx1 - 2, sy1, rgb(0x683e22), Mode::Sol);
        c.rect(sx0 + 3, (sy0 + sy1) / 2, sx1 - 3, (sy0 + sy1) / 2 + 1, rgb(0x58341c), Mode::Sol);
    }

    // 8. Copeaux de fer et escarbilles refroidies au sol (`w.ctx > 50`, balayés pendant `Sweep`)
    if p.sweep.is_none() && w.ctx > 50.0 {
        let n = (((w.ctx - 50.0) / 50.0 * SCRAPS.len() as f64).round() as usize).min(SCRAPS.len());
        for &(ux, uy, ch) in SCRAPS.iter().take(n) {
            let (sx, sy) = (c.ax(ux as f32 + 0.4), c.ay(uy as f32 + 0.2));
            let col = match ch {
                b'E' => rgb(0xd05828),
                b'r' | b'R' => rgb(0x8a542e),
                _ => rgb(0x8c96aa),
            };
            c.poser(sx, sy, col, Mode::Sol);
            c.poser(sx + 1, sy, CONTOUR, Mode::Sol);
        }
    }

    // 9. Trésor de pièces d'or et gemmes près du seuil (`w.usd`, `u = 48.8..53.6, v = 27.0..31.8`)
    let marches = match w.usd {
        Some(usd) if usd > 0.0 => COIN_STEPS.min(1 + ((1.0 + usd).log2() * 2.0).floor() as i32),
        _ => 0,
    };
    if marches > 0 {
        let total = ((marches as f32 / COIN_STEPS as f32) * 26.0).round() as usize;
        let base_y = c.ay(31.5);
        let centre_x = c.ax(51.2);
        for i in 0..total {
            let rang = (i / 6) as i32;
            let col_i = (i % 6) as i32 - 3 + (rang % 2);
            let px = centre_x + col_i * 2;
            let py = base_y - rang * 2;
            let scintille = ((i as i64 + w.clock.div_euclid(9)) % 11) == 0;
            let or_c = if scintille {
                rgb(0xfff294)
            } else if i == 9 && marches >= 6 {
                rgb(0xe83848) // rubis dans le trésor
            } else if i == 16 && marches >= 8 {
                rgb(0x38d878) // émeraude
            } else if (i + rang as usize) % 2 == 0 {
                rgb(0xf0c244)
            } else {
                rgb(0xc8962c)
            };
            c.rect(px, py - 1, px + 2, py + 1, or_c, if scintille { Mode::Lueur } else { Mode::Sol });
            c.poser(px, py + 1, CONTOUR, Mode::Sol);
        }
    }
}

// ============================================================================
// F8 — Quatre chandelles flottantes façon Poudlard (`u < 40`)
// ============================================================================

fn position_chandelle(idx: usize, clock: i64) -> (f32, f32) {
    let (u, v) = CHANDELLES_FORGE[idx];
    let flottement = (clock as f32 * 0.11 + idx as f32 * 1.7).sin() * 0.42;
    (u, v + flottement)
}

fn peindre_chandelles_poudlard(c: &mut Canevas, clock: i64) {
    for idx in 0..CHANDELLES_FORGE.len() {
        let (u, v) = position_chandelle(idx, clock);
        let cx = c.ax(u);
        let cy = c.ay(v);
        // Fût de cire d'abeille ivoire, ombré à droite, une coulure, et sa coupelle de laiton
        c.boite(cx - 1, cy, cx + 3, cy + 7, rgb(0xf2e8cf), CONTOUR, Mode::Sol);
        c.rect(cx, cy + 1, cx + 1, cy + 6, rgb(0xfff8e8), Mode::Sol);
        c.rect(cx + 1, cy + 1, cx + 2, cy + 6, rgb(0xd8c8a4), Mode::Sol);
        c.poser(cx - 1, cy + 2, rgb(0xf6ecd4), Mode::Sol);
        c.rect(cx - 2, cy + 6, cx + 4, cy + 7, rgb(0xc89a40), Mode::Sol);

        // Flamme en goutte, vacillante (`Mode::Lueur`) : large et orangée en bas, blanche au cœur, effilée en haut
        let danse = ((clock + idx as i64 * 5) % 6 < 2) as i32;
        c.poser(cx, cy - 1, rgb(0xff8e2c), Mode::Lueur);
        c.poser(cx + 1, cy - 1, rgb(0xe8782a), Mode::Lueur);
        c.poser(cx, cy - 2, rgb(0xfff6c8), Mode::Lueur);
        c.poser(cx + 1, cy - 2, rgb(0xffcf48), Mode::Lueur);
        c.poser(cx + danse, cy - 3, rgb(0xffe27a), Mode::Lueur);
        c.poser(cx + danse, cy - 4, rgb(0xffb838), Mode::Lueur);
    }
}

/// Détails `1×1` au vrai pixel d'écran : mèches fines des chandelles flottantes, étincelle sur le canard
/// et éclat fin sur *Andúril* ou l'astrolabe.
#[allow(clippy::too_many_arguments)]
fn peindre_details_fins_1x1(
    pixels: &mut [u32],
    w: usize,
    rh: usize,
    sx0: usize,
    sx1: usize,
    dec_x: i32,
    cw: f32,
    demi: f32,
    monde: &Monde,
    pose: &Pose,
) {
    let mut point = |u: f32, v: f32, hex: u32| {
        let px = (u * cw).round() as i32 - dec_x;
        let py = (v * demi).round() as i32;
        if px >= sx0 as i32 && px < sx1 as i32 && py >= 0 && (py as usize) < rh {
            pixels[py as usize * w + px as usize] = 0xFF00_0000 | hex;
        }
    };

    // Mèche noire `1×1` et cœur blanc pur `1×1` sur chaque chandelle flottante
    for idx in 0..CHANDELLES_FORGE.len() {
        let (u, v) = position_chandelle(idx, monde.clock);
        point(u, v - 0.15, 0x1b1924);
        point(u, v - 0.55, 0xfffef0);
    }

    // Reflet blanc `1×1` dans l'œil du canard en plastique jaune
    let caisse_active = pose.crate_.is_some_and(|st| st != Caisse::Gone);
    if monde.duck && !caisse_active {
        let du = DUCK_AT.0 as f32 + 1.45;
        let dv = ANVIL_AT.1 as f32 - 1.35;
        point(du, dv, 0xffffff);
    }
}

// ============================================================================
// E5 — Chapeau de sorcier étoilé (Anniversaire) & Chaperon au houx (Noël)
// ============================================================================

/// Peint en pixel art `2×2` le chapeau festif sur la tête de Scorpheus (quand `pet::peindre` l'a dessiné)
/// et rend le rectangle touché pour que l'effacement incrémental (`peints`) n'en laisse aucune trace.
#[allow(clippy::too_many_arguments)]
pub fn peindre_chapeau_acteur(
    a: &Acteur,
    fete: Option<Fete>,
    pixels: &mut [u32],
    w: usize,
    h: usize,
    x0: i32,
    cw: i32,
    ch: i32,
) -> Option<(i32, i32, i32, i32)> {
    let fete = fete.filter(|f| matches!(f, Fete::Anniversaire | Fete::Noel))?;
    if a.perso != Perso::Scorpheus || cw <= 0 || ch <= 0 {
        return None;
    }
    let cases = pet::cases(a.perso, ch as u32, a.cape, a.accessoire)?;
    let case = cases.case(a.rangee, a.image)?;
    if case.w == 0 || case.h == 0 {
        return None;
    }

    let demi = ch as f32 / 2.0;
    let ox = ((a.x - x0 as f32) * cw as f32).round() as i32 + case.dx;
    let oy = (a.y * demi).round() as i32 + case.dy;

    // On cherche le sommet du crâne près de l'axe vertical des pattes (`-case.dx`), en ignorant le marteau levé à gauche
    let axe = (-case.dx).clamp(0, case.w as i32 - 1);
    let demi_tete = (cw * 3 / 5).max(3);
    let i_min = (axe - demi_tete).max(0) as usize;
    let i_max = ((axe + demi_tete + 1).min(case.w as i32)).max(0) as usize;
    if i_min >= i_max {
        return None;
    }

    let mut sommet_j = None;
    let mut somme_i = 0i32;
    let mut nb_i = 0i32;
    for j in 0..case.h {
        let row = &case.px[j * case.w..(j + 1) * case.w];
        for (i, &px) in row.iter().enumerate().take(i_max).skip(i_min) {
            if px >> 24 >= 128 {
                if sommet_j.is_none() {
                    sommet_j = Some(j as i32);
                }
                if sommet_j == Some(j as i32) || sommet_j == Some(j as i32 - 1) {
                    somme_i += i as i32;
                    nb_i += 1;
                }
            }
        }
        if sommet_j.is_some_and(|sj| j as i32 > sj + 1) {
            break;
        }
    }
    let sj = sommet_j?;
    let centre_x = ox + if nb_i > 0 { somme_i / nb_i } else { axe };
    let base_y = oy + sj + GRAIN as i32; // posé juste sur la crête

    // Facteur lumineux local du personnage en haut de sa tête
    let k = match &a.eclairage {
        Some(e) if e.w > 0 && e.h > 0 => {
            let lx = (a.x.round() as i32 - e.x0).clamp(0, e.w - 1);
            let ly = (((base_y as f32 / demi).round() as i32) - e.y0).clamp(0, e.h - 1);
            let v = e.k[(ly * e.w + lx) as usize];
            [v[0].max(0.28), v[1].max(0.28), v[2].max(0.28)]
        }
        _ => [1.0, 1.0, 1.0],
    };

    // Motifs `13 × 10` en pixels d'art `2×2`
    const SORCIER: [&str; 10] = [
        ".......kk....",
        "......kBBk...",
        ".....kByBBk..",
        ".....kBBBBk..",
        "....kBBBBk...",
        "...kBByBBk...",
        "...kBBBBBBk..",
        "..kyyyyyyyyk.",
        ".kBBBBBBBBBBk",
        "kkkkkkkkkkkkk",
    ];
    const NOEL: [&str; 10] = [
        "............",
        "......kkkk..",
        "....kkRRRRkk",
        "...kRRRRRRRk",
        "..kRRRRRRRRk",
        "..kRRRRRkwwk",
        ".kGGRRRRkwwk",
        ".kwOwwwwkkkk",
        "kwwwwwwwwwwk",
        "kkkkkkkkkkkk",
    ];

    let grille = if fete == Fete::Anniversaire { &SORCIER } else { &NOEL };
    let haut = grille.len() as i32;
    let larg = grille[0].len() as i32;
    let gx0 = centre_x - (larg * GRAIN as i32) / 2;
    let gy0 = base_y - (haut - 1) * GRAIN as i32;

    let (mut l_min, mut t_min, mut r_max, mut b_max) = (w as i32, h as i32, 0i32, 0i32);
    for (rj, ligne) in grille.iter().enumerate() {
        for (ri, ch) in ligne.bytes().enumerate() {
            if ch == b'.' {
                continue;
            }
            let (brut, lueur) = match ch {
                b'k' => (CONTOUR, false),
                b'B' => (rgb(0x2848a8), false),
                b'y' => (rgb(0xffdf64), true),
                b'R' => (rgb(0xc82834), false),
                b'w' => (rgb(0xf4f0ea), false),
                b'G' => (rgb(0x2c8a44), false),
                b'O' => (rgb(0xf43838), true),
                _ => continue,
            };
            let fk = if lueur { [k[0].max(0.85), k[1].max(0.85), k[2].max(0.85)] } else { k };
            let r = (brut[0] as f32 * fk[0]).clamp(0.0, 255.0) as u32;
            let g = (brut[1] as f32 * fk[1]).clamp(0.0, 255.0) as u32;
            let b = (brut[2] as f32 * fk[2]).clamp(0.0, 255.0) as u32;
            let coul = 0xFF00_0000 | (r << 16) | (g << 8) | b;

            let px0 = gx0 + ri as i32 * GRAIN as i32;
            let py0 = gy0 + rj as i32 * GRAIN as i32;
            for sy in py0.max(0)..(py0 + GRAIN as i32).min(h as i32) {
                for sx in px0.max(0)..(px0 + GRAIN as i32).min(w as i32) {
                    pixels[sy as usize * w + sx as usize] = coul;
                    l_min = l_min.min(sx);
                    t_min = t_min.min(sy);
                    r_max = r_max.max(sx + 1);
                    b_max = b_max.max(sy + 1);
                }
            }
        }
    }

    (l_min < r_max && t_min < b_max).then_some((l_min, t_min, r_max, b_max))
}
