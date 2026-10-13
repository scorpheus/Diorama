//! Le journal : %TEMP%\diorama-rendu.log, une ligne par chose qui compte, recommencé passé 256 Ko.

use std::io::Write;
use windows::Win32::System::SystemInformation::GetLocalTime;

const LIMITE: u64 = 256 * 1024;

pub fn note(ligne: &str) {
    let chemin = std::env::temp_dir().join("diorama-rendu.log");
    if std::fs::metadata(&chemin).is_ok_and(|m| m.len() > LIMITE) {
        let _ = std::fs::remove_file(&chemin);
    }
    let t = unsafe { GetLocalTime() };
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&chemin) {
        let _ = writeln!(f, "{:02}:{:02}:{:02} {}", t.wHour, t.wMinute, t.wSecond, ligne);
    }
}
