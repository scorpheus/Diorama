//! La veille : Windows nous dit quand une fenêtre de terminal bouge, change de titre (un autre
//! onglet), passe au premier plan, se réduit ou revient. Les repères ne sont regardés qu'après
//! un tel signe, et sinon toutes les deux secondes seulement : lire l'écran coûte.

use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowThreadProcessId, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE, EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND,
    OBJID_WINDOW, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};

static ALERTE: AtomicBool = AtomicBool::new(true);

/// S'il s'est passé quelque chose depuis la dernière fois qu'on a demandé.
pub fn alerte() -> bool {
    ALERTE.swap(false, Ordering::Relaxed)
}

unsafe extern "system" fn signe(_: HWINEVENTHOOK, _: u32, _: HWND, objet: i32, enfant: i32, _: u32, _: u32) {
    // la fenêtre elle-même, pas son curseur ni ses contrôles
    if objet == OBJID_WINDOW.0 && enfant == 0 {
        ALERTE.store(true, Ordering::Relaxed);
    }
}

/// Les processus de terminal déjà suivis.
#[derive(Default)]
pub struct Veille {
    suivis: Vec<u32>,
}

impl Veille {
    pub fn suivre(&mut self, terminal: HWND) {
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(terminal, Some(&mut pid)) };
        if pid == 0 || self.suivis.contains(&pid) {
            return;
        }
        self.suivis.push(pid);
        let drapeaux = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;
        unsafe {
            // les signes arrivent par la boucle des messages, sur ce fil
            SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND, None, Some(signe), pid, 0, drapeaux);
            SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE, None, Some(signe), pid, 0, drapeaux);
        }
        ALERTE.store(true, Ordering::Relaxed);
    }
}
