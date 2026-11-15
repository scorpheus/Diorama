//! Diorama : les outils pour regarder la forge sans ClaudeTerminal. La forge (forge/) et la scène
//! (scene.rs) sont compilées telles quelles par ClaudeTerminal (son build.rs), qui peint le Diorama
//! de chaque session ; ce programme ne sert qu'à dessiner et à vérifier.
//!
//!   rendu --planche <png> [--large <colonnes>]    la forge dans chaque moment (forge/planche.rs)
//!   rendu --galadriel <dossier> [--large <colonnes>] [--scorpheus] [--scenes a,b]
//!                                                  Galadriel dans chaque moment (forge/planche.rs)
//!   rendu --vitrine [--cadence <ms>]              ces moments joués dans une fenêtre (vitrine.rs)

mod forge;
mod scene;
mod vitrine;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let apres = |nom: &str| args.iter().position(|a| a == nom).map(|i| args.get(i + 1).and_then(|s| s.parse::<u64>().ok()));
    let large = apres("--large").flatten().unwrap_or(0) as i32;
    let resultat = if let Some(i) = args.iter().position(|a| a == "--planche") {
        let sortie = args.get(i + 1).map_or("planche.png", |s| s.as_str());
        forge::planche::planche(sortie, large).map_err(|e| format!("planche : {e}"))
    } else if let Some(i) = args.iter().position(|a| a == "--galadriel") {
        let dossier = args.get(i + 1).map_or(".", |s| s.as_str());
        let perso = if args.iter().any(|a| a == "--scorpheus") { None } else { Some(forge::monde::Perso::Galadriel) };
        let filtre: Vec<String> = args.iter().position(|a| a == "--scenes").and_then(|i| args.get(i + 1)).map(|s| s.split(',').map(String::from).collect()).unwrap_or_default();
        forge::planche::galadriel(dossier, large, perso, &filtre).map_err(|e| format!("galadriel : {e}"))
    } else if args.iter().any(|a| a == "--vitrine") {
        let cadence_ms = apres("--cadence").flatten().unwrap_or(150).clamp(10, 1000) as u32;
        vitrine::vitrine(cadence_ms).map_err(|e| format!("vitrine : {e}"))
    } else {
        Err("rendu --planche <png> | --galadriel <dossier> | --vitrine (voir l'en-tête de src/main.rs)".into())
    };
    if let Err(e) = resultat {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
