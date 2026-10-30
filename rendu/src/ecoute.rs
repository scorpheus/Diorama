//! L'écoute : le plugin parle à l'appli en HTTP sur 127.0.0.1 (le moteur de Claude lui refuse
//! les tubes nommés). Chaque requête porte une ligne de texte, « verbe arguments… » ; elle est
//! remise au fil principal, qu'on réveille par un message posté à sa fenêtre.
//!
//! Le moteur garde ses connexions ouvertes : une connexion, un fil, ses requêtes l'une après
//! l'autre. Il y en a une par session tout au plus, et elles dorment entre deux requêtes.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::Mutex;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_APP};

/// Le port de l'appli, le même que celui du plugin (register.tsx : RENDER_PORT).
pub const PORT: u16 = 41917;
/// Le message qui réveille le fil principal : une ligne attend dans le canal.
pub const WM_LIGNE: u32 = WM_APP + 1;

/// La réponse à une requête : « vu » si le bandeau de sa session se voit et que quelqu'un est
/// devant l'écran (le clavier ou la souris ont servi depuis moins de PRESENCE_MS), « absent »
/// sinon. Le plugin attend « vu » pour jouer un spectacle.
fn reponse(vu: bool) -> &'static [u8] {
    if vu {
        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nvu"
    } else {
        b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\n\r\nabsent"
    }
}

/// Les sessions dont le bandeau se voit, tenues à jour par le fil principal à chaque image.
pub static VUES: Mutex<Vec<String>> = Mutex::new(Vec::new());
const PRESENCE_MS: u32 = 120_000;

/// Quelqu'un est-il devant l'écran : le clavier ou la souris ont servi il y a peu.
fn present() -> bool {
    let mut l = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    unsafe { GetLastInputInfo(&mut l).as_bool() && GetTickCount().wrapping_sub(l.dwTime) < PRESENCE_MS }
}

/// La session dont parle une requête : le deuxième mot de sa première ligne (« bande <session> … »).
fn session_de(corps: &[u8]) -> Option<String> {
    let texte = std::str::from_utf8(corps).ok()?;
    texte.lines().next()?.split_whitespace().nth(1).map(str::to_string)
}
const TAILLE_MAX: usize = 64 * 1024;

/// Écoute dans un fil à part ; `Err` si le port est déjà pris (une autre appli tourne).
pub fn ecouter(lignes: Sender<String>, fenetre: HWND) -> std::io::Result<()> {
    let ecoute = TcpListener::bind((Ipv4Addr::LOCALHOST, PORT))?;
    let fenetre = fenetre.0 as isize; // une poignée de fenêtre se poste de n'importe quel fil
    std::thread::spawn(move || {
        for client in ecoute.incoming().flatten() {
            let lignes = lignes.clone();
            std::thread::spawn(move || servir(client, lignes, fenetre));
        }
    });
    Ok(())
}

fn servir(client: TcpStream, lignes: Sender<String>, fenetre: isize) {
    let _ = client.set_nodelay(true);
    let Ok(mut sortie) = client.try_clone() else { return };
    let mut entree = BufReader::new(client);
    let mut ligne = String::new();
    loop {
        // l'en-tête, jusqu'à la ligne vide ; seule sa longueur nous importe
        let mut longueur = 0usize;
        loop {
            ligne.clear();
            match entree.read_line(&mut ligne) {
                Ok(0) | Err(_) => return,
                Ok(_) => {}
            }
            let l = ligne.trim_end();
            if l.is_empty() {
                break;
            }
            if let Some((nom, valeur)) = l.split_once(':') {
                if nom.eq_ignore_ascii_case("content-length") {
                    longueur = valeur.trim().parse().unwrap_or(0);
                }
            }
        }
        if longueur > TAILLE_MAX {
            return;
        }
        let mut corps = vec![0u8; longueur];
        if entree.read_exact(&mut corps).is_err() {
            return;
        }
        let vu = session_de(&corps).is_some_and(|s| VUES.lock().is_ok_and(|v| v.contains(&s))) && present();
        if sortie.write_all(reponse(vu)).is_err() {
            return;
        }
        if corps.is_empty() {
            continue;
        }
        if lignes.send(String::from_utf8_lossy(&corps).into_owned()).is_err() {
            return;
        }
        unsafe {
            let _ = PostMessageW(Some(HWND(fenetre as *mut _)), WM_LIGNE, WPARAM(0), LPARAM(0));
        }
    }
}
