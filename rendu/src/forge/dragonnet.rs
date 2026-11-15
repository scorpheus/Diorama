//! Les dragonnets de la cour (user, 12-11 : il retient « dragonnets » parmi les pistes ; 13-11 : « pour faire les
//! dragonnets […] oui, tu peux bien sûr y aller »), dessinés à la main en lettres, de la famille des louveteaux à
//! lunettes : de face, trapus, seize lignes de seize lettres, contour sombre, peu de teintes, les lunettes de laiton
//! relevées sur le front. La tête en écu (large au front, le museau étroit et ses naseaux), les cornes d'ivoire, les
//! ailes de chauve-souris entrouvertes, le ventre en plaques. Deux frères : Braise, de cuivre, et Mousse, d'émeraude.
//! paysage.rs les fait vivre sur le toit du puits : ils clignent, soufflent des ronds de fumée, s'envolent ensemble
//! faire une boucle au-dessus de la cour, et dorment la nuit.

pub const ASSIS: [&str; 16] = [
    ".kk..........kk.",
    ".kHk........kHk.",
    "..kHk......kHk..",
    "..kOHkkkkkkHOk..",
    ".kYyeeyyyyeeyYk.",
    ".kOOOOOOOOOOOOk.",
    "k.kOwnOOOOwnOk.k",
    "kokOnnOOOOnnOkok",
    "kMokOOccccOOkoMk",
    "kMMokcnccnckoMMk",
    "kMMMokcccckoMMMk",
    "kMMMMokkkkoMMMMk",
    "kMkkOOccccOOkkMk",
    ".k.kOkcddckOk.k.",
    "...kOOccccOOk...",
    "...kkOkkkkOkk...",
];

pub const CLIGNE: [&str; 16] = [
    ".kk..........kk.",
    ".kHk........kHk.",
    "..kHk......kHk..",
    "..kOHkkkkkkHOk..",
    ".kYyeeyyyyeeyYk.",
    ".kOOOOOOOOOOOOk.",
    "k.kOOOOOOOOOOk.k",
    "kokOkkOOOOkkOkok",
    "kMokOOccccOOkoMk",
    "kMMokcnccnckoMMk",
    "kMMMokcccckoMMMk",
    "kMMMMokkkkoMMMMk",
    "kMkkOOccccOOkkMk",
    ".k.kOkcddckOk.k.",
    "...kOOccccOOk...",
    "...kkOkkkkOkk...",
];

pub const SOUFFLE: [&str; 16] = [
    ".kk..........kk.",
    ".kHk........kHk.",
    "..kHk......kHk..",
    "..kOHkkkkkkHOk..",
    ".kYyeeyyyyeeyYk.",
    ".kOOOOOOOOOOOOk.",
    "k.kOwnOOOOwnOk.k",
    "kokOnnOOOOnnOkok",
    "kMokOOccccOOkoMk",
    "kMMokcnccnckoMMk",
    "kMMMokcccckoMMMk",
    "kMMMMokrrkoMMMMk",
    "kMkkOOccccOOkkMk",
    ".k.kOkcddckOk.k.",
    "...kOOccccOOk...",
    "...kkOkkkkOkk...",
];

pub const VOL_HAUT: [&str; 16] = [
    ".kk..........kk.",
    ".kHk........kHk.",
    "k.kHk......kHk.k",
    "kokOHkkkkkkHOkok",
    "kkYyeeyyyyeeyYkk",
    "kkOOOOOOOOOOOOkk",
    "kMkOwnOOOOwnOkMk",
    "kMkOnnOOOOnnOkMk",
    "kMokOOccccOOkoMk",
    "kkMokcnccnckoMkk",
    ".kMkokcccckokMk.",
    "..k.kokkkkok.k..",
    "....kOccccOk....",
    "...kOkcddckOk...",
    "...kOkccccOOk...",
    "....k.kkkk.k....",
];

pub const VOL_BAS: [&str; 16] = [
    ".kk..........kk.",
    ".kHk........kHk.",
    "..kHk......kHk..",
    "..kOHkkkkkkHOk..",
    ".kYyeeyyyyeeyYk.",
    ".kOOOOOOOOOOOOk.",
    "..kOwnOOOOwnOk..",
    "..kOnnOOOOnnOk..",
    "...kOOccccOOk...",
    "....kcnccnck....",
    "..kk.kcccck.kk..",
    ".kMokkkkkkkkoMk.",
    "kMMokOccccOkoMMk",
    "kMMokOcddcOkoMMk",
    "kMokkOccccOkkoMk",
    "kok..kkkkkk..kok",
];

/// Un des deux frères : ses écailles (`O`, `o`) et ses ailes (`M`).
#[derive(Clone, Copy)]
pub struct Robe {
    pub corps: u32,
    pub ombre: u32,
    pub ailes: u32,
}

pub const BRAISE: Robe = Robe { corps: 0xd0583a, ombre: 0x8c3024, ailes: 0xf0a050 };
pub const MOUSSE: Robe = Robe { corps: 0x4aa858, ombre: 0x26683a, ailes: 0x9ad870 };

/// La couleur d'une lettre pour cette robe.
pub fn couleur(l: u8, r: &Robe) -> Option<u32> {
    Some(match l {
        b'O' => r.corps,
        b'o' => r.ombre,
        b'M' => r.ailes,
        b'k' => 0x1b1a24,
        b'c' => 0xf4e2b0,
        b'd' => 0xc8a466,
        b'H' => 0xeee0bc,
        b'y' => 0xe0b040,
        b'Y' => 0x9a6a1c,
        b'e' => 0x58c8ff,
        b'n' => 0x141018,
        b'w' => 0xffffff,
        b'r' => 0x8a2030,
        _ => return None,
    })
}

/// Où sortent les naseaux dans l'image (colonne, rang), d'où monte la fumée.
pub const NASEAUX: (i32, i32) = (7, 9);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seize_sur_seize_et_lettres_connues() {
        for img in [&ASSIS, &CLIGNE, &SOUFFLE, &VOL_HAUT, &VOL_BAS] {
            for l in img.iter() {
                assert_eq!(l.len(), 16, "{l}");
                for c in l.bytes() {
                    assert!(c == b'.' || couleur(c, &BRAISE).is_some(), "lettre {}", c as char);
                }
            }
        }
    }
}
