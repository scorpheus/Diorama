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
        // Pilastre vertical discret
        c.rect(px - 1, c.ay(5.5), px + 2, plinthe_y, rgb(0x3a323e), Mode::Mur);
        c.rect(px, c.ay(5.5), px + 1, plinthe_y, rgb(0x584c5a), Mode::Mur);
        // Chapiteau taillé
        c.rect(px - 2, c.ay(5.0), px + 3, c.ay(5.8), rgb(0x635666), Mode::Mur);
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
                    c.poser(rx, ry, rgb(0x5d5060), Mode::Mur);
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

    // 2. Enclume runique naine (`u = 16.0..28.8, v = 25.0..28.2`)
    // Table d'acier poli + bigorne pointue à gauche + talon carré à droite
    let table_y0 = c.ay(ay_h);
    let table_y1 = c.ay(ay_h + 1.35).max(table_y0 + 3);
    let bigorne_x = c.ax(ax_g);
    let corps_x0 = c.ax(ax_g + 2.8);
    let talon_x1 = c.ax(ax_g + 12.8);

    // Table principale et talon
    c.boite(corps_x0, table_y0, talon_x1, table_y1, rgb(0x8f98ac), CONTOUR, Mode::Plein);
    // Reflet spéculaire net sur le dessus de la table d'enclume
    c.rect(corps_x0 + 1, table_y0 + 1, talon_x1 - 2, table_y0 + 2, rgb(0xcbd4e6), Mode::Plein);

    // Bigorne effilée vers le foyer (à gauche)
    for ax in bigorne_x..corps_x0 + 1 {
        let t = ((ax - bigorne_x) as f32 / (corps_x0 - bigorne_x).max(1) as f32).clamp(0.0, 1.0);
        let yb = table_y0 + 1 + ((1.0 - t) * 1.6).round() as i32;
        let yh = table_y0;
        c.poser(ax, yh, CONTOUR, Mode::Plein);
        for ay in yh + 1..=yb.min(table_y1) {
            c.poser(ax, ay, if ay == yh + 1 { rgb(0xd4dced) } else { rgb(0x7c8498) }, Mode::Plein);
        }
        c.poser(ax, yb.min(table_y1) + 1, CONTOUR, Mode::Plein);
    }

    // Taille cintrée de l'enclume et patins rivés sur le billot
    let pied_y1 = by0 + 1;
    let taille_x0 = c.ax(ax_g + 3.8);
    let taille_x1 = c.ax(ax_g + 10.6);
    c.boite(taille_x0, table_y1 - 1, taille_x1, pied_y1, rgb(0x636b7e), CONTOUR, Mode::Plein);
    c.rect(taille_x0 + 1, table_y1, taille_x0 + 4, pied_y1 - 1, rgb(0x838ca0), Mode::Plein);
    // Semelle évasée
    c.boite(c.ax(ax_g + 2.8), pied_y1 - 2, c.ax(ax_g + 11.4), pied_y1, rgb(0x555d70), CONTOUR, Mode::Plein);

    // Rune naine d'or incrustée au flanc de l'enclume (losange Khuzdul)
    let (rx, ry) = ((taille_x0 + taille_x1) / 2, (table_y1 + pied_y1) / 2);
    c.poser(rx, ry - 1, rgb(0xe4b448), Mode::Plein);
    c.poser(rx - 1, ry, rgb(0xf4cc64), Mode::Plein);
    c.poser(rx + 1, ry, rgb(0xc89632), Mode::Plein);
    c.poser(rx, ry + 1, rgb(0xe4b448), Mode::Plein);
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
    let v_appui = 9.4f32;

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
                let plomb_losange = (((u - uc) * 1.6 + v * 1.6).fract() < 0.14) || (((u - uc) * 1.6 - v * 1.6).fract().abs() < 0.14);
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
        let px = c.ax(u1 - 2.6);
        let py = ay_appui - 2;
        c.disque(px, py, 2, rgb(0xe46d24), Mode::Plein);
        c.poser(px, py - 3, rgb(0x3c6e2c), Mode::Plein);
        c.poser(px - 1, py - 1, rgb(0xffea64), Mode::Lueur);
        c.poser(px + 1, py - 1, rgb(0xffea64), Mode::Lueur);
        c.poser(px, py + 1, rgb(0xffb838), Mode::Lueur);
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
    // Placé en `u = 42.4, v = 13.6`, rayon ~ 2.1 (`v = 11.2..16.0`, hors de `45..53 × 2..10`)
    let cx = c.ax(42.4);
    let cy = c.ay(13.6);
    let r = c.ay(2.15).max(5);

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

    // *Andúril*, la Flamme de l'Ouest reforgée : couchée à l'horizontale sur crochets d'or quand `masterpiece`
    if w.masterpiece {
        let ay = c.ay(10.95);
        let (ax_g, ax_d) = (c.ax(15.2), c.ax(26.8));
        // Crochets d'or
        c.poser(c.ax(18.0), ay + 1, rgb(0xd4a43c), Mode::Sol);
        c.poser(c.ax(24.0), ay + 1, rgb(0xd4a43c), Mode::Sol);
        // Pommeau saphir, fusée et grande garde elfique d'or
        c.poser(ax_g, ay, rgb(0x54a8ff), Mode::Lueur);
        c.rect(ax_g + 1, ay, ax_g + 4, ay + 1, rgb(0x6e3b22), Mode::Sol);
        c.rect(ax_g + 4, ay - 2, ax_g + 5, ay + 3, rgb(0xf4ce5e), Mode::Lueur);
        // Longue lame de mithril aux runes stellaires
        c.rect(ax_g + 5, ay - 1, ax_d, ay + 2, CONTOUR, Mode::Sol);
        c.rect(ax_g + 5, ay, ax_d - 1, ay + 1, rgb(0xe8f2ff), Mode::Lueur);
        c.poser(ax_d - 1, ay, rgb(0xffffff), Mode::Lueur);
        let eclat = ((w.clock / 3) % 8) as i32;
        if ax_g + 6 + eclat * 2 < ax_d - 1 {
            c.poser(ax_g + 6 + eclat * 2, ay, rgb(0xffffff), Mode::Lueur);
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
        let hot = w.forging && i + 1 == derniers.len();
        let y_top = ry + 2;
        let y_bas = c.ay(17.6);

        match arme {
            Arme::Sword => {
                // Épée elfique à lame foliacée (*Dard / Glamdring*), pointe en bas, garde d'or en haut
                let (lame, coeur, mode) = if hot {
                    (rgb(0xff9428), rgb(0xfff078), Mode::Lueur)
                } else if lueur_dard {
                    (rgb(0x5cc8ff), rgb(0xdff8ff), Mode::Lueur)
                } else {
                    (rgb(0x9ca8be), rgb(0xdce4f4), Mode::Sol)
                };
                // Pommeau + poignée + garde en haut
                c.poser(sx, y_top - 2, rgb(0xf4cc58), Mode::Sol);
                c.poser(sx, y_top - 1, rgb(0x5e341e), Mode::Sol);
                c.rect(sx - 2, y_top, sx + 3, y_top + 1, rgb(0xe8bc48), Mode::Sol);
                // Lame avec contour
                c.rect(sx - 1, y_top + 1, sx + 2, y_bas, CONTOUR, Mode::Sol);
                for ay in y_top + 1..y_bas {
                    c.poser(sx, ay, if (ay - y_top) % 3 == 0 { coeur } else { lame }, mode);
                }
                c.poser(sx, y_bas, coeur, mode);
            }
            Arme::Axe => {
                // Hache de bataille naine à fer croissant damasquiné d'or
                for ay in y_top - 1..=y_bas {
                    c.poser(sx, ay, rgb(0x744628), Mode::Sol);
                }
                let fer = if hot { rgb(0xffa838) } else { rgb(0xa8b4ca) };
                let mode = if hot { Mode::Lueur } else { Mode::Sol };
                c.boite(sx - 3, y_top + 1, sx + 1, y_top + 5, fer, CONTOUR, mode);
                c.poser(sx - 2, y_top + 2, if hot { rgb(0xfff088) } else { rgb(0xe4ecfc) }, mode);
                c.poser(sx - 1, y_top + 3, rgb(0xdcb044), Mode::Sol);
            }
            Arme::Dagger => {
                // Dague elfique courbe sertie de mithril
                let y_fin = (y_top + y_bas) / 2 + 2;
                c.poser(sx, y_top - 1, rgb(0x52301c), Mode::Sol);
                c.rect(sx - 1, y_top, sx + 2, y_top + 1, rgb(0xd8b450), Mode::Sol);
                let lame = if hot { rgb(0xffb43c) } else { rgb(0xc4d0e6) };
                let mode = if hot { Mode::Lueur } else { Mode::Sol };
                for ay in y_top + 1..=y_fin {
                    c.poser(sx, ay, lame, mode);
                }
            }
            Arme::Mace => {
                // Masse d'armes à ailettes d'acier et cœur de cristal saphir
                for ay in y_top..=y_bas {
                    c.poser(sx, ay, rgb(0x683e22), Mode::Sol);
                }
                let ym = y_bas - 1;
                c.boite(sx - 2, ym - 2, sx + 3, ym + 2, rgb(0x8892a8), CONTOUR, Mode::Sol);
                c.poser(sx, ym, if hot { rgb(0xffe868) } else { rgb(0x64ccff) }, Mode::Lueur);
            }
        }
    }
}

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
// F3 (dynamique) — Runes naines, bûches, flammes, fumée, bouilloire & soufflet
// ============================================================================

fn peindre_foyer_et_soufflet(c: &mut Canevas, w: &Monde, p: &Pose, tick: i64) {
    let feu = if w.exhausted { 0 } else { p.fire.clamp(0, 4) };

    // 1. Quatre runes angulaires de Khazad-dûm gravées dans le linteau (`0..=4` allumées selon `feu`)
    let ly = c.ay(16.3);
    for (idx, &ru) in [3.2f32, 5.7, 8.2, 10.6].iter().enumerate() {
        let rx = c.ax(ru);
        let active = (idx as i32) < feu;
        let (col, mode) = if active {
            let pulse = ((tick + idx as i64 * 3) % 6) < 3;
            (if pulse { rgb(0xffd85c) } else { rgb(0xff8e28) }, Mode::Lueur)
        } else {
            (rgb(0x282330), Mode::Plein)
        };
        // Glyphe runique nain ciselé (`3×4` pixels d'art)
        c.poser(rx, ly, col, mode);
        c.poser(rx - 1, ly + 1, col, mode);
        c.poser(rx + 1, ly + 1, col, mode);
        c.poser(rx, ly + 2, col, mode);
        if idx % 2 == 0 {
            c.poser(rx - 1, ly + 3, col, mode);
            c.poser(rx + 1, ly + 3, col, mode);
        } else {
            c.poser(rx, ly + 3, col, mode);
        }
    }

    // 2. Réserve hebdomadaire de bûches sur le degré gauche de la hotte (`world.fuel7d`, `0..=10`)
    let pile = match w.fuel7d {
        None => 10usize,
        Some(f) => ((10.0 * (1.0 - f.min(100.0) / 100.0)).round() as i32).clamp(0, 10) as usize,
    };
    const TAS_BOIS: [(f32, f32); 10] = [
        (0.8, 14.5), (2.1, 14.5), (3.4, 14.5), (4.7, 14.5),
        (1.2, 13.4), (2.5, 13.4), (3.8, 13.4),
        (1.8, 12.3), (3.1, 12.3),
        (2.4, 11.2),
    ];
    for &(bu, bv) in TAS_BOIS.iter().take(pile) {
        let (bx, by) = (c.ax(bu), c.ay(bv));
        c.disque(bx, by, 2, CONTOUR, Mode::Plein);
        c.disque(bx, by, 1, rgb(0x6e4426), Mode::Plein);
        c.poser(bx, by, rgb(0xd8b07a), Mode::Plein); // tranche claire du cœur de chêne
    }

    // 3. Bûches ardentes et lit de braises dans l'âtre (`world.fuel5h`, `0..=8`)
    let logs = match w.fuel5h {
        None => 8usize,
        Some(f) => ((8.0 * (1.0 - f.min(100.0) / 100.0)).round() as i32).clamp(0, 8) as usize,
    };
    let (mx0, mx1) = (c.ax(MOUTH.0 as f32), c.ax((MOUTH.0 + MOUTH.2) as f32));
    let my_sol = c.ay(26.2);

    // Tapis de braises incandescentes
    for ax in mx0..mx1 {
        let vif = ((ax as i64 + tick / 2) % 3) == 0;
        let br = if feu == 0 {
            if vif { rgb(0x9e321e) } else { rgb(0x521c18) }
        } else if vif {
            rgb(0xff7824)
        } else {
            rgb(0xc8381c)
        };
        c.poser(ax, my_sol, br, Mode::Lueur);
        c.poser(ax, my_sol - 1, echelle(br, 0.8), Mode::Lueur);
    }
    for i in 0..logs {
        let lu = MOUTH.0 as f32 + 0.5 + i as f32 * 0.95;
        let lx = c.ax(lu);
        c.boite(lx - 1, my_sol - 3, lx + 2, my_sol, rgb(0x4a2c1a), CONTOUR, Mode::Plein);
        if feu > 0 && (i + (tick as usize / 3)) % 2 == 0 {
            c.poser(lx, my_sol - 2, rgb(0xff6820), Mode::Lueur);
        }
    }

    // 4. Langues de flammes étagées en pixel art net (`Mode::Lueur`)
    if feu > 0 {
        let my_haut = c.ay(MOUTH.1 as f32 + 0.4);
        let h_max = (my_sol - 2 - my_haut).max(3);
        for ax in mx0..mx1 {
            let centre = 1.0 - ((ax as f32 - (mx0 + mx1 - 1) as f32 * 0.5).abs() / ((mx1 - mx0).max(2) as f32 * 0.55)).min(1.0);
            let danse = hash_f(tick * 29 + ax as i64 * 13) * 0.42 - 0.14;
            let frac_h = ((feu as f32 / 4.0) * (0.45 + 0.65 * centre) + danse).clamp(0.12, 1.0);
            let h_col = ((h_max as f32 * frac_h).round() as i32).max(1);
            for k in 0..h_col {
                let f = k as f32 / h_col.max(1) as f32;
                let col = if f < 0.24 {
                    rgb(0xfff49c) // cœur blanc-or
                } else if f < 0.52 {
                    rgb(0xffbe34) // or vif
                } else if f < 0.80 {
                    rgb(0xf2641e) // orange forge
                } else {
                    rgb(0xb8281e) // crête vermeille
                };
                c.poser(ax, my_sol - 2 - k, col, Mode::Lueur);
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

    // 6. Soufflet médiéval fuselé en poire : cuir sombre plissé, joues de chêne et clous de laiton (`u = 11.4..18.4`)
    let bx0 = c.ax(12.4);
    let bx1 = c.ax(17.8);
    let by_mil = c.ay(23.2);
    let demi_max = if p.bellows { 3 } else { 5 };

    // Buse conique de laiton fichée dans le flanc du four
    c.rect(c.ax(11.3), by_mil - 1, bx0 + 1, by_mil + 1, CONTOUR, Mode::Plein);
    c.rect(c.ax(11.4), by_mil - 1, bx0 + 1, by_mil, rgb(0xd4a442), Mode::Plein);
    if p.bellows {
        c.poser(c.ax(10.4), by_mil - 1, rgb(0xfff0a0), Mode::Lueur);
        c.poser(c.ax(9.6), by_mil - 2, rgb(0xffb844), Mode::Lueur);
    }

    // Corps en soufflet triangulaire (étroit à la buse à gauche, large aux poignées à droite)
    let larg = (bx1 - bx0).max(2);
    for ax in bx0..=bx1 {
        let t = (ax - bx0) as f32 / larg as f32;
        let demi = (1.0 + t * (demi_max - 1) as f32).round() as i32;
        let yh = by_mil - demi;
        let yb = by_mil + demi;
        for ay in yh..=yb {
            let col = if ax == bx0 || ax == bx1 || ay == yh || ay == yb {
                CONTOUR
            } else if ay == yh + 1 || ay == yb - 1 {
                // Joues de chêne ferré et clous de laiton
                if (ax - bx0) % 2 == 1 && t > 0.25 {
                    rgb(0xe2b64c)
                } else {
                    rgb(0x4e2f1a)
                }
            } else if (ax - bx0) % 2 == 0 {
                rgb(0x3b1e22) // pli creux du cuir
            } else {
                rgb(0x642a2c) // cuir tanné grenat sombre
            };
            c.poser(ax, ay, col, Mode::Plein);
        }
    }
    // Deux poignées de chêne tournées vers la droite
    let lev_x = c.ax(18.7);
    let ecart_p = if p.bellows { 2 } else { 4 };
    c.ligne(bx1, by_mil - demi_max + 1, lev_x, by_mil - ecart_p, rgb(0x6e4426), Mode::Plein);
    c.ligne(bx1, by_mil + demi_max - 1, lev_x, by_mil + ecart_p - 1, rgb(0x52321b), Mode::Plein);
    c.poser(lev_x, by_mil - ecart_p, rgb(0xdcb04c), Mode::Plein);
    c.poser(lev_x, by_mil + ecart_p - 1, rgb(0xdcb04c), Mode::Plein);
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
        // Fût de cire d'abeille ivoire + coulure de cire
        c.boite(cx - 1, cy, cx + 2, cy + 5, rgb(0xf2e8cf), CONTOUR, Mode::Sol);
        c.poser(cx, cy + 1, rgb(0xfff8e8), Mode::Sol);
        c.poser(cx - 1, cy + 3, rgb(0xe4d4b2), Mode::Sol);

        // Flamme dorée vacillante au sommet (`Mode::Lueur`)
        let danse = ((clock + idx as i64 * 5) % 6 < 2) as i32;
        c.poser(cx, cy - 2, rgb(0xff9e2c), Mode::Lueur);
        c.poser(cx, cy - 3, rgb(0xfff6b0), Mode::Lueur);
        if danse == 0 {
            c.poser(cx, cy - 4, rgb(0xffcf48), Mode::Lueur);
        } else {
            c.poser(cx + 1, cy - 3, rgb(0xffb838), Mode::Lueur);
        }
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
