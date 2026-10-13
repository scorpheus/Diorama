//! L'écoute : le plugin parle à l'appli en HTTP sur 127.0.0.1 (le moteur de Claude lui refuse
//! les tubes nommés). Chaque requête porte une ligne de texte, « verbe arguments… » ; elle est
//! remise au fil principal, qu'on réveille par un message posté à sa fenêtre.
//!
//! Le moteur garde ses connexions ouvertes : une connexion, un fil, ses requêtes l'une après
//! l'autre. Il y en a une par session tout au plus, et elles dorment entre deux requêtes.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::mpsc::Sender;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_APP};

/// Le port de l'appli, le même que celui du plugin (register.tsx : RENDER_PORT).
pub const PORT: u16 = 41917;
/// Le message qui réveille le fil principal : une ligne attend dans le canal.
pub const WM_LIGNE: u32 = WM_APP + 1;

const REPONSE: &[u8] = b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n";
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
        if entree.read_exact(&mut corps).is_err() || sortie.write_all(REPONSE).is_err() {
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
