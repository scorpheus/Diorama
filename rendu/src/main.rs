//! Diorama, l'appli de rendu : elle dessine le bandeau de chaque session de Claude Code, en vrais
//! pixels, sur une fenêtre posée par-dessus le terminal ; le terminal n'en garde qu'un rectangle
//! noir et fixe, marqué de deux repères. Une seule appli pour la machine, lancée par le plugin
//! (rendu.ps1), qui s'en va quand il ne reste plus de bandeau.
//!
//!   rendu.exe [--cadence <ms>]     une image toutes les <ms> (150 par défaut, la cadence du plugin)
//!   rendu.exe --banc [<secondes>]  ce qu'elle coûte, poste par poste (banc.rs)
//!   rendu.exe --planche <png> [--large <colonnes>]  la forge dans chaque moment (forge/planche.rs)
//!   rendu.exe --vitrine            ces moments joués dans une fenêtre, à la cadence (vitrine.rs)
//!
//! Elle note ce qu'elle fait dans %TEMP%\diorama-rendu.log.

#![windows_subsystem = "windows"]

mod banc;
mod forge;
mod ecoute;
mod journal;
mod reperes;
mod scene;
mod toile;
mod veille;
mod vitrine;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use windows::core::w;
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HWND, RECT};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, IsIconic, IsWindowVisible, SetTimer, TranslateMessage, MSG, WM_TIMER,
};

use journal::note;
use reperes::{Capture, Place};
use scene::{Image, Scene};
use toile::Toile;

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
    /// le processus `claude attach` dont l'onglet montre la session, quand elle tourne en
    /// arrière-plan (reperes::rattaches)
    client: Option<u32>,
    /// où sont les repères, par rapport au coin de la fenêtre du terminal : la fenêtre peut
    /// bouger sans qu'on les cherche à nouveau
    place: Option<Place>,
    toile: Option<Toile>,
    /// l'image de la toile est à repeindre en entier (neuve, ou d'une autre taille)
    neuve: bool,
    perdu_depuis: Option<Instant>, // depuis quand on ne voit plus ses repères
    cherche: u64,                  // le coup d'œil de la dernière recherche
    scene: forge::SceneForge,
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
    capture: Capture,
    veille: veille::Veille,
    bandes: Vec<Bande>,
    regards: u64,
    seule_depuis: Option<Instant>,
    // pour la ligne de chaque minute
    minute: Instant,
    images: u64,
    ms: f64,
    recherches: u64,
}

impl App {
    /// Une ligne du plugin : « bande <session> <colonnes> <lignes> », « monde <session> <état> »
    /// (forge::Etat), ou « quitter ».
    fn ligne(&mut self, texte: &str) -> bool {
        if let Some(reste) = texte.strip_prefix("monde ") {
            let Some((session, etat)) = reste.split_once(' ') else { return true };
            let Some(b) = self.bandes.iter_mut().find(|b| b.session == session) else { return true };
            match forge::Etat::lire(etat) {
                Ok(etat) => b.scene.recevoir(etat),
                Err(e) => note(&format!("{session} : monde illisible : {e}")),
            }
            return true;
        }
        let mots: Vec<&str> = texte.split_whitespace().collect();
        match mots.as_slice() {
            ["quitter"] => {
                note("on me demande de partir");
                return false;
            }
            ["bande", session, cols, rows, vivant @ ..] if vivant.len() <= 1 => {
                // après un /clear, le processus va sous un autre identifiant : on le trouve par lui
                let vivant = vivant.first().unwrap_or(session);
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
                let Some(pid) = reperes::processus_de(vivant) else {
                    note(&format!("{session} : pas de processus connu sous {vivant}"));
                    return true;
                };
                // un même processus n'a qu'un bandeau : un module rechargé après un /clear en
                // annonce un sous un autre nom, l'ancien ne se verra plus
                self.bandes.retain(|b| {
                    if b.pid == pid {
                        note(&format!("{} : remplacé par {session}", b.session));
                    }
                    b.pid != pid
                });
                note(&format!("{session} (processus {pid}) : bandeau de {cols} × {rows} cellules"));
                self.bandes.push(Bande {
                    session: session.to_string(),
                    pid,
                    reperes: reperes::reperes(session),
                    cols,
                    rows,
                    terminal: None,
                    client: None,
                    place: None,
                    toile: None,
                    neuve: true,
                    perdu_depuis: None,
                    cherche: 0,
                    scene: forge::SceneForge::new(),
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
        if self.regards % SURETE == 0 {
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
            // l'onglet rattaché s'est fermé : sa fenêtre ne la montre plus
            if b.client.is_some_and(|c| self.regards % SURETE == 0 && !reperes::vivant(c)) {
                b.cacher();
                b.client = None;
                b.terminal = None;
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
                    let visible = |t: &HWND| unsafe { IsWindowVisible(*t).as_bool() };
                    // sa fenêtre ; sinon (lancée en arrière-plan), celle d'un onglet qui s'y
                    // rattache : chercher ces onglets passe tous les processus en revue, une
                    // fois toutes les 4 s
                    b.client = None;
                    let trouvee = reperes::fenetre_de(b.pid).filter(visible).or_else(|| {
                        if self.regards % 16 != 1 {
                            return None;
                        }
                        reperes::rattaches(&b.session, b.pid).into_iter().find_map(|c| {
                            let t = reperes::fenetre_de(c).filter(visible)?;
                            note(&format!("{} : montrée dans l'onglet de « claude attach » (processus {c})", b.session));
                            b.client = Some(c);
                            Some(t)
                        })
                    });
                    match trouvee {
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
                    poser(b, terminal, &ici);
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
                    poser(b, terminal, &ici);
                    // les onglets d'un terminal ont leur bandeau au même endroit : celui qu'on
                    // vient de voir à l'écran a pris la place d'un autre, que son prochain coup
                    // d'œil n'aurait caché qu'après (jusqu'à 2 s, les deux forges l'une sur l'autre)
                    let (ici, cols, rows) = (b.place.unwrap(), b.cols, b.rows);
                    let session = b.session.clone();
                    for (j, o) in self.bandes.iter_mut().enumerate() {
                        let Some(p) = o.place.filter(|_| j != i && o.terminal == Some(terminal)) else { continue };
                        let croise = p.x < ici.x + cols * ici.cw && ici.x < p.x + o.cols * p.cw && p.y < ici.y + rows * ici.ch && ici.y < p.y + o.rows * p.ch;
                        if croise && o.toile.as_ref().is_some_and(|t| t.visible) {
                            note(&format!("{} : caché, {session} est montré à sa place", o.session));
                            o.cacher();
                        }
                    }
                }
            }
        }
        true
    }

    /// Une image pour chaque bandeau visible : la scène peint ce qui a changé, la toile n'en
    /// remet que cette zone au compositeur.
    fn peindre(&mut self) {
        for b in self.bandes.iter_mut() {
            let (Some(toile), Some(p)) = (b.toile.as_mut(), b.place) else { continue };
            if !toile.visible && !b.neuve {
                continue;
            }
            let debut = Instant::now();
            let dt = debut.duration_since(b.vue).as_secs_f32();
            b.vue = debut;
            let (w, h) = (toile.w, toile.h);
            let mut image = Image { pixels: toile.pixels(), w, h, cellule: (p.cw as u32, p.ch as u32) };
            let neuve = std::mem::take(&mut b.neuve);
            let Some(mut zone) = b.scene.peindre(&mut image, dt, neuve) else { continue };
            // les repères du terminal restent visibles à travers : leurs cellules transparentes
            for trou in trous(w, h, p.cw, p.ch) {
                if neuve || croise(zone, trou) {
                    image.remplir(trou, 0);
                    zone = scene::union(zone, trou);
                }
            }
            toile.presenter(Some(zone));
            self.images += 1;
            self.ms += debut.elapsed().as_secs_f64() * 1000.0;
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

/// Les deux cellules repères dans l'image d'un bandeau : la première et la dernière.
fn trous(w: u32, h: u32, cw: i32, ch: i32) -> [RECT; 2] {
    let (w, h) = (w as i32, h as i32);
    [RECT { left: 0, top: 0, right: cw, bottom: ch }, RECT { left: w - cw, top: h - ch, right: w, bottom: h }]
}

fn croise(a: RECT, b: RECT) -> bool {
    a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom
}

/// Pose (ou laisse) la toile du bandeau sur ses repères, à l'écran ; elle se montre à sa
/// prochaine image.
fn poser(b: &mut Bande, terminal: HWND, ici: &Place) {
    let (w, h) = ((b.cols * ici.cw) as u32, (b.rows * ici.ch) as u32);
    if b.toile.is_none() {
        match Toile::new(Some(terminal), w, h) {
            Ok(t) => {
                b.toile = Some(t);
                b.neuve = true;
            }
            Err(e) => {
                note(&format!("{} : pas de toile : {e}", b.session));
                return;
            }
        }
    }
    let toile = b.toile.as_mut().unwrap();
    if toile.visible && (toile.w, toile.h) == (w, h) && toile.place() == (ici.x, ici.y) {
        return;
    }
    match toile.poser(ici.x, ici.y, w, h) {
        Ok(neuve) => b.neuve |= neuve || !toile.visible,
        Err(e) => note(&format!("{} : la toile ne se pose pas : {e}", b.session)),
    }
    if !b.neuve {
        toile.presenter(Some(RECT::default())); // elle bouge seulement : rien à repeindre
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let apres = |nom: &str| args.iter().position(|a| a == nom).map(|i| args.get(i + 1).and_then(|s| s.parse::<u64>().ok()));
    let cadence_ms = apres("--cadence").flatten().unwrap_or(150).clamp(10, 1000) as u32;
    let banc = apres("--banc").map(|s| s.unwrap_or(20));
    if let Some(i) = args.iter().position(|a| a == "--planche") {
        let sortie = args.get(i + 1).map_or("planche.png", |s| s.as_str());
        let large = apres("--large").flatten().unwrap_or(0) as i32;
        if let Err(e) = forge::planche::planche(sortie, large) {
            eprintln!("planche : {e}");
        }
        return;
    }
    if args.iter().any(|a| a == "--vitrine") {
        if let Err(e) = vitrine::vitrine(cadence_ms) {
            note(&format!("vitrine : {e}"));
        }
        return;
    }
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        if let Err(e) = toile::inscrire() {
            note(&format!("pas de classe de fenêtre : {e}"));
            return;
        }
        let messages = match toile::fenetre_des_messages() {
            Ok(f) => f,
            Err(e) => {
                note(&format!("pas de fenêtre des messages : {e}"));
                return;
            }
        };
        if let Some(secondes) = banc {
            banc::banc(messages, secondes);
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
        note(&format!("l'appli prend son poste, une image toutes les {cadence_ms} ms"));
        let mut app = App {
            capture: Capture::new(),
            veille: veille::Veille::default(),
            bandes: Vec::new(),
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
                    while let Ok(corps) = lignes.try_recv() {
                        for l in corps.lines() {
                            reste &= app.ligne(l);
                        }
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
