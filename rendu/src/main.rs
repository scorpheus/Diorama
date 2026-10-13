//! Diorama, l'appli de rendu : elle dessine le bandeau de chaque session de Claude Code, en vrais
//! pixels, sur une fenêtre posée par-dessus le terminal ; le terminal n'en garde qu'un rectangle
//! noir et fixe, marqué de deux repères. Une seule appli pour la machine, lancée par le plugin
//! (rendu.ps1), qui s'en va quand il ne reste plus de bandeau.
//!
//!   rendu.exe [--cadence <ms>] [--scene simple]
//!       --cadence : une image toutes les <ms> (150 par défaut, la cadence du plugin)
//!       --scene simple : un carré qui glisse au lieu de la gerbe (pour mesurer ce que coûte l'appli seule)
//!
//! Elle note ce qu'elle fait dans %TEMP%\diorama-rendu.log.

#![windows_subsystem = "windows"]

mod banc;
mod ecoute;
mod etincelles;
mod journal;
mod veille;
mod peintre;
mod reperes;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use windows::core::w;
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HWND};
use windows::Win32::Graphics::Direct2D::Common::{D2D1_COLOR_F, D2D_RECT_F};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, IsIconic, IsWindowVisible, SetTimer, TranslateMessage, MSG, WM_TIMER,
};

use etincelles::Gerbe;
use journal::note;
use peintre::{Peintre, Toile};
use reperes::{Capture, Place};

const REGARD_MS: u32 = 250; // un coup d'œil aux repères, quand il est dû
const SURETE: u64 = 8; // et un, de toute façon, tous les 8 (2 s)
const MINUTERIE_IMAGE: usize = 1;
const MINUTERIE_REGARD: usize = 2;
const SEUL_AU_PLUS: Duration = Duration::from_secs(60); // sans bandeau, l'appli s'en va passé ce temps

/// Le bandeau d'une session.
struct Bande {
    session: String,
    pid: u32,
    reperes: [u32; 4],
    cols: i32,
    rows: i32,
    terminal: Option<HWND>,
    /// où sont les repères, par rapport au coin de la fenêtre du terminal : la fenêtre peut
    /// bouger sans qu'on les cherche à nouveau
    place: Option<Place>,
    toile: Option<Toile>,
    perdu_depuis: Option<Instant>, // depuis quand on ne voit plus ses repères
    cherche: u64,                  // le coup d'œil de la dernière recherche
    scene: Gerbe,
    vue: Instant, // la dernière image de cette scène
}

impl Bande {
    fn cacher(&mut self) {
        if let Some(t) = self.toile.as_mut() {
            t.cacher();
        }
    }
}

struct App {
    peintre: Peintre,
    capture: Capture,
    veille: veille::Veille,
    bandes: Vec<Bande>,
    cadence_ms: u32,
    simple: Option<Option<windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush>>,
    regards: u64,
    seule_depuis: Option<Instant>,
    // pour la ligne de chaque minute
    minute: Instant,
    images: u64,
    ms: f64,
    recherches: u64,
}

impl App {
    /// Une ligne du plugin : « bande <session> <colonnes> <lignes> », ou « quitter ».
    fn ligne(&mut self, texte: &str) -> bool {
        let mots: Vec<&str> = texte.split_whitespace().collect();
        match mots.as_slice() {
            ["quitter"] => {
                note("on me demande de partir");
                return false;
            }
            ["bande", session, cols, rows] => {
                let (Ok(cols), Ok(rows)) = (cols.parse::<i32>(), rows.parse::<i32>()) else { return true };
                if cols < 2 || rows < 1 {
                    return true;
                }
                if let Some(b) = self.bandes.iter_mut().find(|b| b.session == *session) {
                    if (b.cols, b.rows) != (cols, rows) {
                        note(&format!("{session} : bandeau de {cols} × {rows} cellules"));
                        b.cols = cols;
                        b.rows = rows;
                        b.place = None; // une autre taille : chercher à nouveau
                        b.cacher();
                    }
                    return true;
                }
                let Some(pid) = reperes::processus_de(session) else {
                    note(&format!("{session} : pas de processus connu"));
                    return true;
                };
                note(&format!("{session} (processus {pid}) : bandeau de {cols} × {rows} cellules"));
                let mut scene = Gerbe::new(reperes::hash(session) as u64);
                scene.cadence_ms = self.cadence_ms;
                self.bandes.push(Bande {
                    session: session.to_string(),
                    pid,
                    reperes: reperes::reperes(session),
                    cols,
                    rows,
                    terminal: None,
                    place: None,
                    toile: None,
                    perdu_depuis: None,
                    cherche: 0,
                    scene,
                    vue: Instant::now(),
                });
                self.seule_depuis = None;
            }
            _ => note(&format!("ligne inconnue : {}", texte.chars().take(80).collect::<String>())),
        }
        true
    }

    /// Le coup d'œil : les sessions vivent-elles, leurs repères sont-ils là, sinon où sont-ils.
    fn regarder(&mut self) -> bool {
        self.regards += 1;
        if self.regards % 8 == 0 {
            let avant = self.bandes.len();
            self.bandes.retain(|b| {
                let vit = reperes::vivant(b.pid);
                if !vit {
                    note(&format!("{} : la session est partie", b.session));
                }
                vit
            });
            if self.bandes.is_empty() && avant > 0 {
                self.seule_depuis = Some(Instant::now());
            }
        }
        if self.bandes.is_empty() {
            return self.seule_depuis.is_none_or(|t| t.elapsed() < SEUL_AU_PLUS);
        }
        // on ne regarde qu'après un signe de Windows, et sinon toutes les deux secondes
        let du = veille::alerte() || self.regards % SURETE == 0;
        // d'abord, là où on les a vus : deux colonnes d'un pixel par bandeau
        let mut a_chercher: Vec<usize> = Vec::new();
        for (i, b) in self.bandes.iter_mut().enumerate() {
            if !du && b.terminal.is_some() {
                continue;
            }
            let terminal = match b.terminal {
                Some(t) if unsafe { IsWindowVisible(t).as_bool() } => t,
                _ => {
                    b.cacher();
                    b.terminal = None;
                    if self.regards % 4 != 1 {
                        continue; // s'attacher à une console coûte : une fois par seconde
                    }
                    match reperes::fenetre_de(b.pid) {
                        Some(t) => {
                            self.veille.suivre(t);
                            b.terminal = Some(t);
                            b.toile = None; // une toile appartient à son terminal
                            t
                        }
                        None => continue,
                    }
                }
            };
            if unsafe { IsIconic(terminal).as_bool() } {
                b.cacher();
                continue;
            }
            let Some(fenetre) = reperes::rectangle(terminal) else { continue };
            if let Some(p) = b.place {
                let ici = Place { x: fenetre.left + p.x, y: fenetre.top + p.y, ..p };
                if reperes::tient(&mut self.capture, &ici, b.cols, b.rows, &b.reperes) {
                    b.perdu_depuis = None;
                    poser(&self.peintre, b, terminal, &ici);
                    continue;
                }
            }
            b.cacher();
            // perdu : une capture de toute la fenêtre coûte ; une par seconde au plus, puis une
            // toutes les 6 s passé une minute
            let pas = if b.perdu_depuis.is_some_and(|t| t.elapsed() > Duration::from_secs(60)) { 24 } else { 4 };
            if b.perdu_depuis.is_none() || self.regards - b.cherche >= pas {
                b.perdu_depuis.get_or_insert_with(Instant::now);
                b.cherche = self.regards;
                a_chercher.push(i);
            }
        }
        // puis une capture par fenêtre de terminal, pour tous ses bandeaux perdus
        while let Some(&premier) = a_chercher.first() {
            let terminal = self.bandes[premier].terminal.unwrap();
            let (miens, autres): (Vec<usize>, Vec<usize>) = a_chercher.iter().partition(|&&i| self.bandes[i].terminal == Some(terminal));
            a_chercher = autres;
            let Some(r) = reperes::rectangle(terminal) else { continue };
            self.recherches += 1;
            if !self.capture.prendre(r.left, r.top, r.right - r.left, r.bottom - r.top) {
                continue;
            }
            for i in miens {
                let b = &mut self.bandes[i];
                if let Some(ici) = reperes::chercher(&self.capture, (r.left, r.top), b.cols, b.rows, &b.reperes) {
                    note(&format!("{} : bandeau trouvé en ({}, {}), cellule {} × {}", b.session, ici.x, ici.y, ici.cw, ici.ch));
                    b.place = Some(Place { x: ici.x - r.left, y: ici.y - r.top, ..ici });
                    b.perdu_depuis = None;
                    poser(&self.peintre, b, terminal, &ici);
                }
            }
        }
        true
    }

    /// Une image pour chaque bandeau visible.
    fn peindre(&mut self) {
        for b in self.bandes.iter_mut() {
            let (Some(toile), Some(p)) = (b.toile.as_mut(), b.place) else { continue };
            if !toile.visible {
                continue;
            }
            let debut = Instant::now();
            let dt = debut.duration_since(b.vue).as_secs_f32();
            b.vue = debut;
            let scene = &mut b.scene;
            let (w, h, cw, ch) = (toile.w as f32, toile.h as f32, p.cw as f32, p.ch as f32);
            let trous = [D2D_RECT_F { left: 0.0, top: 0.0, right: cw, bottom: ch }, D2D_RECT_F { left: w - cw, top: h - ch, right: w, bottom: h }];
            if let Some(pinceau) = self.simple.as_mut() {
                // la mesure de l'appli seule : un fond, un carré qui glisse, les trous
                if pinceau.is_none() {
                    *pinceau = unsafe { self.peintre.dc.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.9, g: 0.5, b: 0.2, a: 1.0 }, None) }.ok();
                }
                let x = (self.images as f32 * 4.0) % (w - 20.0);
                let _ = toile.peindre(&self.peintre, |dc| unsafe {
                    dc.Clear(Some(&D2D1_COLOR_F { r: 0.05, g: 0.05, b: 0.08, a: 1.0 }));
                    if let Some(b) = pinceau.as_ref() {
                        dc.FillRectangle(&D2D_RECT_F { left: x, top: h / 2.0 - 10.0, right: x + 20.0, bottom: h / 2.0 + 10.0 }, b);
                    }
                    for t in &trous {
                        dc.PushAxisAlignedClip(t, windows::Win32::Graphics::Direct2D::D2D1_ANTIALIAS_MODE_ALIASED);
                        dc.Clear(Some(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }));
                        dc.PopAxisAlignedClip();
                    }
                });
                self.images += 1;
                self.ms += debut.elapsed().as_secs_f64() * 1000.0;
                continue;
            }
            if let Err(e) = scene.preparer(&self.peintre.dc, &self.peintre.d2d, &self.peintre.ecriture, toile.w, toile.h) {
                note(&format!("{} : la scène ne se prépare pas : {e}", b.session));
                continue;
            }
            scene.avancer(dt);
            if let Err(e) = toile.peindre(&self.peintre, |dc| scene.dessiner(dc, &trous)) {
                note(&format!("{} : l'image ne se peint pas : {e}", b.session));
            }
            let ms = debut.elapsed().as_secs_f64() * 1000.0;
            scene.cout_ms = scene.cout_ms * 0.9 + ms as f32 * 0.1;
            self.images += 1;
            self.ms += ms;
        }
        if self.minute.elapsed() >= Duration::from_secs(60) {
            let visibles = self.bandes.iter().filter(|b| b.toile.as_ref().is_some_and(|t| t.visible)).count();
            note(&format!(
                "une minute : {} bandeaux, {} visibles, {} images ({:.3} ms chacune), {} recherches",
                self.bandes.len(),
                visibles,
                self.images,
                if self.images > 0 { self.ms / self.images as f64 } else { 0.0 },
                self.recherches
            ));
            self.minute = Instant::now();
            self.images = 0;
            self.ms = 0.0;
            self.recherches = 0;
        }
    }
}

/// Pose (ou laisse) la toile du bandeau sur ses repères, à l'écran.
fn poser(peintre: &Peintre, b: &mut Bande, terminal: HWND, ici: &Place) {
    let (w, h) = ((b.cols * ici.cw) as u32, (b.rows * ici.ch) as u32);
    if b.toile.is_none() {
        match Toile::new(peintre, Some(terminal), w, h) {
            Ok(t) => b.toile = Some(t),
            Err(e) => {
                note(&format!("{} : pas de toile : {e}", b.session));
                return;
            }
        }
    }
    let toile = b.toile.as_mut().unwrap();
    if toile.visible && toile.w == w && toile.h == h && toile_en(toile, ici) {
        return;
    }
    b.scene.cellule = (ici.cw, ici.ch);
    if let Err(e) = toile.poser(ici.x, ici.y, w, h) {
        note(&format!("{} : la toile ne se pose pas : {e}", b.session));
    }
}

fn toile_en(toile: &Toile, ici: &Place) -> bool {
    let mut r = Default::default();
    unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowRect(toile.fenetre, &mut r).is_ok() && r.left == ici.x && r.top == ici.y }
}

fn main() {
    let mut cadence_ms = 150u32;
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--cadence") {
        cadence_ms = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(150).clamp(10, 1000);
    }
    let logiciel = args.iter().any(|a| a == "--warp");
    let simple = args.windows(2).any(|a| a[0] == "--scene" && a[1] == "simple");
    let banc = args.iter().position(|a| a == "--banc").map(|i| args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(20u64));
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        if let Err(e) = Peintre::inscrire(peintre::procedure_par_defaut) {
            note(&format!("pas de classe de fenêtre : {e}"));
            return;
        }
        let messages = match Peintre::fenetre_des_messages() {
            Ok(f) => f,
            Err(e) => {
                note(&format!("pas de fenêtre des messages : {e}"));
                return;
            }
        };
        if let Some(secondes) = banc {
            match Peintre::new(logiciel) {
                Ok(p) => banc::banc(&p, messages, secondes, logiciel, args.iter().any(|a| a == "images")),
                Err(e) => note(&format!("banc : pas de quoi peindre : {e}")),
            }
            return;
        }
        // une seule appli pour la machine
        let _verrou = CreateMutexW(None, true, w!("Local\\DioramaRendu"));
        if GetLastError() == ERROR_ALREADY_EXISTS {
            return;
        }
        let (envoi, lignes) = mpsc::channel();
        if let Err(e) = ecoute::ecouter(envoi, messages) {
            note(&format!("le port {} est pris ({e}) : une autre appli tourne", ecoute::PORT));
            return;
        }
        let peintre = match Peintre::new(logiciel) {
            Ok(p) => p,
            Err(e) => {
                note(&format!("pas de quoi peindre : {e}"));
                return;
            }
        };
        note(&format!("l'appli prend son poste, une image toutes les {cadence_ms} ms{}", if simple { ", scène simple" } else { "" }));
        let mut app = App {
            peintre,
            capture: Capture::new(),
            veille: veille::Veille::default(),
            bandes: Vec::new(),
            cadence_ms,
            simple: simple.then_some(None),
            regards: 0,
            seule_depuis: Some(Instant::now()),
            minute: Instant::now(),
            images: 0,
            ms: 0.0,
            recherches: 0,
        };
        SetTimer(Some(messages), MINUTERIE_IMAGE, cadence_ms, None);
        SetTimer(Some(messages), MINUTERIE_REGARD, REGARD_MS, None);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.hwnd == messages {
                if msg.message == ecoute::WM_LIGNE {
                    let mut reste = true;
                    while let Ok(l) = lignes.try_recv() {
                        reste &= app.ligne(&l);
                    }
                    if !reste {
                        break;
                    }
                    continue;
                }
                if msg.message == WM_TIMER && msg.wParam.0 == MINUTERIE_IMAGE {
                    app.peindre();
                    continue;
                }
                if msg.message == WM_TIMER && msg.wParam.0 == MINUTERIE_REGARD {
                    if !app.regarder() {
                        note("plus aucun bandeau : l'appli s'en va");
                        break;
                    }
                    continue;
                }
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
