//! Trouver le bandeau d'une session à l'écran.
//!
//! Le plugin dessine dans le terminal un bandeau noir et fixe, marqué de deux cellules repères :
//! la première (moitié haute a, basse b) et la dernière (c, d), de couleurs sombres tirées de la
//! session. On les cherche une fois dans une capture de la fenêtre du terminal ; ensuite, on
//! vérifie seulement qu'elles sont toujours là, deux colonnes d'un pixel de large. Si on ne les
//! voit plus (un autre onglet, le défilement, une fenêtre posée dessus), le bandeau est caché.

use windows::Win32::Foundation::{CloseHandle, HWND, RECT, STILL_ACTIVE};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject, BITMAPINFO,
    BITMAPINFOHEADER, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ, SRCCOPY,
};
use windows::Win32::System::Console::{AttachConsole, FreeConsole, GetConsoleWindow, SetConsoleCtrlHandler};
use windows::Win32::System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::WindowsAndMessaging::{GetAncestor, GA_ROOTOWNER};

/// FNV-1a sur les unités UTF-16, comme le plugin (register.tsx : hashOf).
pub fn hash(texte: &str) -> u32 {
    let mut h: u32 = 0x811c9dc5;
    for u in texte.encode_utf16() {
        h = (h ^ u as u32).wrapping_mul(0x01000193);
    }
    h
}

/// Les quatre repères d'une session : des couleurs sombres (chaque canal 0x00, 0x11 ou 0x22),
/// jamais le noir pur (register.tsx : markOf).
pub fn reperes(session: &str) -> [u32; 4] {
    let h = hash(session);
    std::array::from_fn(|n| {
        let v = ((h >> (n * 8)) & 0xff) % 26 + 1;
        (v / 9 % 3 * 0x11) << 16 | (v / 3 % 3 * 0x11) << 8 | (v % 3 * 0x11)
    })
}

/// Où se tient le bandeau à l'écran, en pixels physiques.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Place {
    pub x: i32,
    pub y: i32,
    /// largeur et hauteur d'une cellule
    pub cw: i32,
    pub ch: i32,
    /// la hauteur de la moitié haute d'une cellule
    pub moitie: i32,
}

/// Une image 32 bits en mémoire, où l'on copie un morceau de l'écran.
pub struct Capture {
    dc: HDC,
    bitmap: HBITMAP,
    ancien: HGDIOBJ,
    pixels: *mut u32,
    pub w: i32,
    pub h: i32,
}

impl Capture {
    pub fn new() -> Self {
        let mut c = Capture { dc: HDC::default(), bitmap: HBITMAP::default(), ancien: HGDIOBJ::default(), pixels: std::ptr::null_mut(), w: 0, h: 0 };
        c.taille(1, 1);
        c
    }

    fn liberer(&mut self) {
        if !self.dc.is_invalid() {
            unsafe {
                SelectObject(self.dc, self.ancien);
                let _ = DeleteObject(self.bitmap.into());
                let _ = DeleteDC(self.dc);
            }
            self.dc = HDC::default();
        }
    }

    fn taille(&mut self, w: i32, h: i32) {
        if w == self.w && h == self.h && !self.dc.is_invalid() {
            return;
        }
        self.liberer();
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h, // de haut en bas
                biPlanes: 1,
                biBitCount: 32,
                ..Default::default()
            },
            ..Default::default()
        };
        unsafe {
            let ecran = GetDC(None);
            self.dc = CreateCompatibleDC(Some(ecran));
            let mut bits = std::ptr::null_mut();
            self.bitmap = CreateDIBSection(Some(ecran), &info, DIB_RGB_COLORS, &mut bits, None, 0).unwrap_or_default();
            ReleaseDC(None, ecran);
            self.pixels = bits as *mut u32;
            self.ancien = SelectObject(self.dc, self.bitmap.into());
        }
        self.w = w;
        self.h = h;
    }

    /// Copie le rectangle (x, y, w, h) de l'écran en (0, 0).
    pub fn prendre(&mut self, x: i32, y: i32, w: i32, h: i32) -> bool {
        self.taille(w.max(1), h.max(1));
        if self.pixels.is_null() {
            return false;
        }
        unsafe {
            let ecran = GetDC(None);
            let ok = BitBlt(self.dc, 0, 0, w, h, Some(ecran), x, y, SRCCOPY).is_ok();
            ReleaseDC(None, ecran);
            ok
        }
    }

    #[inline]
    pub fn en(&self, x: i32, y: i32) -> u32 {
        unsafe { *self.pixels.add((y * self.w + x) as usize) & 0xFF_FFFF }
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        self.liberer();
    }
}

/// Si les repères sont toujours là où on les a vus : deux colonnes d'un pixel, au milieu de la
/// première et de la dernière cellule.
pub fn tient(cap: &mut Capture, p: &Place, cols: i32, rows: i32, r: &[u32; 4]) -> bool {
    let (haut, bas) = (p.moitie / 2, (p.moitie + p.ch) / 2);
    if !cap.prendre(p.x + p.cw / 2, p.y, 1, p.ch) || cap.en(0, haut) != r[0] || cap.en(0, bas) != r[1] {
        return false;
    }
    let (x2, y2) = (p.x + (cols - 1) * p.cw + p.cw / 2, p.y + (rows - 1) * p.ch);
    cap.prendre(x2, y2, 1, p.ch) && cap.en(0, haut) == r[2] && cap.en(0, bas) == r[3]
}

/// Cherche le repère haut-gauche dans la capture d'une fenêtre (en `origine` à l'écran), puis le
/// bas-droit à la distance que disent les colonnes et les lignes du bandeau.
pub fn chercher(cap: &Capture, origine: (i32, i32), cols: i32, rows: i32, r: &[u32; 4]) -> Option<Place> {
    let (w, h) = (cap.w, cap.h);
    let [a, b, c, d] = *r;
    for y in 0..h - 1 {
        for x in 0..w {
            if cap.en(x, y) != a || (x > 0 && cap.en(x - 1, y) == a) || (y > 0 && cap.en(x, y - 1) == a) {
                continue;
            }
            // le coin haut-gauche de la zone a : sa largeur, sa hauteur, puis la zone b dessous
            let cw = (x..w).take_while(|&i| cap.en(i, y) == a).count() as i32;
            let moitie = (y..h).take_while(|&j| cap.en(x, j) == a).count() as i32;
            let bas = (y + moitie..h).take_while(|&j| cap.en(x, j) == b).count() as i32;
            if bas == 0 || cw < 2 || moitie < 2 {
                continue;
            }
            let ch = moitie + bas;
            let (x2, y2) = (x + (cols - 1) * cw + cw / 2, y + (rows - 1) * ch);
            if x2 >= w || y2 + ch > h || cap.en(x2, y2 + moitie / 2) != c || cap.en(x2, y2 + (moitie + ch) / 2) != d {
                continue;
            }
            return Some(Place { x: origine.0 + x, y: origine.1 + y, cw, ch, moitie });
        }
    }
    None
}

/// Le processus d'une session, d'après ~/.claude/sessions/<pid>.json.
pub fn processus_de(session: &str) -> Option<u32> {
    let dossier = std::path::Path::new(&std::env::var_os("USERPROFILE")?).join(".claude").join("sessions");
    let cherche = format!("\"{session}\"");
    std::fs::read_dir(dossier).ok()?.flatten().find_map(|f| {
        let chemin = f.path();
        let pid: u32 = chemin.file_stem()?.to_str()?.parse().ok()?;
        std::fs::read_to_string(&chemin).ok()?.contains(&cherche).then_some(pid)
    })
}

/// Si le processus vit encore, sans garder de poignée sur lui.
pub fn vivant(pid: u32) -> bool {
    unsafe {
        let Ok(p) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else { return false };
        let mut code = 0u32;
        let ok = GetExitCodeProcess(p, &mut code).is_ok();
        let _ = CloseHandle(p);
        ok && code == STILL_ACTIVE.0 as u32
    }
}

/// La fenêtre du terminal d'un processus : on s'attache un instant à sa console, dont la
/// fenêtre (cachée, chez Windows Terminal) a pour propriétaire la vraie fenêtre.
pub fn fenetre_de(pid: u32) -> Option<HWND> {
    unsafe {
        let _ = FreeConsole();
        AttachConsole(pid).ok()?;
        let _ = SetConsoleCtrlHandler(None, true); // un Ctrl+C de la session ne nous touche pas
        let console = GetConsoleWindow();
        let _ = FreeConsole();
        if console.is_invalid() {
            return None;
        }
        let racine = GetAncestor(console, GA_ROOTOWNER);
        (!racine.is_invalid()).then_some(racine)
    }
}

/// La ligne de commande d'un processus.
pub fn ligne_de_commande(pid: u32) -> Option<String> {
    use windows::Wdk::System::Threading::{NtQueryInformationProcess, ProcessCommandLineInformation};
    use windows::Win32::Foundation::UNICODE_STRING;
    unsafe {
        let p = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        // un UNICODE_STRING suivi de son texte
        let mut tampon = vec![0u64; 4096];
        let mut n = 0u32;
        let ok = NtQueryInformationProcess(p, ProcessCommandLineInformation, tampon.as_mut_ptr().cast(), (tampon.len() * 8) as u32, &mut n).is_ok();
        let _ = CloseHandle(p);
        if !ok {
            return None;
        }
        let u = &*(tampon.as_ptr() as *const UNICODE_STRING);
        (!u.Buffer.is_null()).then(|| String::from_utf16_lossy(std::slice::from_raw_parts(u.Buffer.0, u.Length as usize / 2)))
    }
}

/// Les processus `claude attach <cible>` qui montrent la session `session` (processus `pid`) :
/// une session lancée en arrière-plan (`claude --bg`) n'a pas de fenêtre à elle, elle se montre
/// dans les onglets qui s'y rattachent. La cible est son identifiant, ses huit premiers
/// caractères, ou son nom (`--name`).
pub fn rattaches(session: &str, pid: u32) -> Vec<u32> {
    use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
    let la_sienne = ligne_de_commande(pid).unwrap_or_default();
    let mots: Vec<&str> = la_sienne.split_whitespace().collect();
    let nom = mots.iter().position(|&m| m == "--name").and_then(|i| mots.get(i + 1)).map(|n| n.trim_matches('"').to_string());
    let court = session.get(..8).unwrap_or(session);
    let mut out = Vec::new();
    unsafe {
        let Ok(photo) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return out };
        let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut encore = Process32FirstW(photo, &mut e).is_ok();
        while encore {
            let exe = String::from_utf16_lossy(&e.szExeFile[..e.szExeFile.iter().position(|&c| c == 0).unwrap_or(0)]);
            if exe.eq_ignore_ascii_case("claude.exe") && e.th32ProcessID != pid {
                if let Some(ligne) = ligne_de_commande(e.th32ProcessID) {
                    let mots: Vec<&str> = ligne.split_whitespace().collect();
                    let cible = mots.iter().position(|&m| m == "attach").and_then(|i| mots.get(i + 1)).map(|c| c.trim_matches('"'));
                    if cible.is_some_and(|c| c == session || c == court || nom.as_deref() == Some(c)) {
                        out.push(e.th32ProcessID);
                    }
                }
            }
            encore = Process32NextW(photo, &mut e).is_ok();
        }
        let _ = CloseHandle(photo);
    }
    out
}

/// Le rectangle d'une fenêtre à l'écran.
pub fn rectangle(f: HWND) -> Option<RECT> {
    let mut r = RECT::default();
    unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowRect(f, &mut r).ok()? };
    (r.right > r.left && r.bottom > r.top).then_some(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_hachage_est_celui_du_plugin() {
        // FNV-1a 32 bits de référence
        assert_eq!(hash(""), 0x811c9dc5);
        assert_eq!(hash("a"), 0xe40c292c);
    }

    /// Le double d'un onglet rattaché : ce binaire de test, copié sous le nom claude.exe et lancé
    /// avec « attach <cible> » ; les mots de sa ligne sont pour lui des filtres, « dort » le tient
    /// en vie le temps de le chercher.
    #[test]
    fn un_onglet_rattache_se_retrouve_par_sa_cible() {
        let dossier = std::env::temp_dir().join(format!("diorama-rattache-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).unwrap();
        let claude = dossier.join("claude.exe");
        std::fs::copy(std::env::current_exe().unwrap(), &claude).unwrap();
        let session = "0123abcd-0000-4000-8000-000000000000";
        let mut cherches = Vec::new();
        for cible in ["0123abcd", session] {
            let mut p = std::process::Command::new(&claude).args(["attach", cible, "dort", "--ignored", "--quiet"]).spawn().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(400));
            cherches.push(rattaches(session, std::process::id()).contains(&p.id()));
            let _ = p.kill();
            let _ = p.wait();
        }
        // une autre session ne le voit pas
        let mut p = std::process::Command::new(&claude).args(["attach", "ffffffff", "dort", "--ignored", "--quiet"]).spawn().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(400));
        let autre = rattaches(session, std::process::id()).contains(&p.id());
        let _ = p.kill();
        let _ = p.wait();
        let _ = std::fs::remove_file(&claude);
        let _ = std::fs::remove_dir(&dossier);
        assert_eq!(cherches, [true, true], "par l'identifiant court, puis entier");
        assert!(!autre);
        assert!(ligne_de_commande(std::process::id()).is_some_and(|l| l.contains("rendu")));
    }

    #[test]
    #[ignore]
    fn dort() {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    #[test]
    fn les_reperes_sont_sombres_et_jamais_noirs() {
        for s in ["sess-1", "4f2a", "une autre"] {
            for m in reperes(s) {
                assert_ne!(m, 0);
                for canal in [m >> 16, (m >> 8) & 0xff, m & 0xff] {
                    assert!(matches!(canal, 0 | 0x11 | 0x22));
                }
            }
        }
    }
}
