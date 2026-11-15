//! Le dragon du ciel de la cour (user, 13-11 : « un meilleur dessin du dragon du ciel »), dessiné à la main en lettres
//! comme la chouette mécanique, dans le style des engrenages : aplats francs, contour sombre, peu de teintes, pas de
//! halo. Rouge sombre au ventre d'or en plaques, tête cornue, queue en S jusqu'à la pointe en fer de lance, ailes de
//! chauve-souris (le bras, trois doigts, la membrane festonnée) en trois poses de battement, l'aile de derrière plus
//! sombre. La nuit, une silhouette où rougeoie la braise du ventre, l'œil d'or ; et la gueule ouverte, une gerbe de
//! feu en trois tons. Tourné vers la droite ; paysage.rs le fait passer au-dessus des monts.

pub const HAUT: [&str; 47] = [
    "...........................................k................",
    "...................kkkkkkk................kdk...............",
    "..................kdddddddkkkkkkkk.......kdk................",
    "...................kDDDDDDddddddddkkkkkkkkdk................",
    ".................kkkkkkkDDDDDDDDDDddddddkHkk................",
    "................kRRRRRRRkkkkkkkkDDddddDkHkdk................",
    ".................kMMMMMMRRRRRRRRkkkkkkkRRkdk................",
    "..................kkMMMMMMMMMMMMRRRRRRRRRkdk................",
    "..................kkkMMMMMMMMMMMRRRRMRRRRkdk................",
    ".................kDdkkMMMMMMRRRRMMMMRMMRRkk.................",
    ".................kkkMMMMRRRRMMMMMMRRMMMRRkkk....kk..........",
    "................kkMMRRRRMMMMMMMMMRMMMMMRRkkkHk..kHk.........",
    "...............kMRRRMMMMMMMMMMMRRMMMMMRRkdk.kHk..kHk........",
    "...............kRMMMMMMMMMMMMMRMMMMMMMRRkk...kHkkkHk........",
    "................kkMMMMMMMMMMRRMMMMMMMMRRkk...kRRRRRkkkk.....",
    "..................kkkMMMMMMRMMMMMMMMMMRRkk..kRSSSSSRRRRkk...",
    ".....................kkkMRRMMMMMMMMMMRRkdk.kRRRRkekSSSSRRk..",
    ".....................kkMRMMMMMMMMMMMMRRkk..kRRRRRRRRRRRRRkkk",
    ".....................kRRMMMMMMMMMMMMMRRkk..krRRRRRRRRRRRRRRk",
    "....................kRMMMMMMMMMMMMMMMRRkk..kkrRRRRrrrrrrrkk.",
    "...................kRMMMMMMMMMMMMMMMRRkdk.kSkYrrrkkkkkkkk...",
    "....................kkkMMMMMMMMMMMMMRRkk.kSRRkYYk...........",
    ".......................kkkMMMMMMMMMMRRkkkSRRYYkk............",
    "..........................kMMMMMMMMMRRkkSRRrYk..............",
    ".........................kRkMMMMMMMRRkkSRRrYk...............",
    "......................kkRSSSkMMMMMMRRkSSRRRrYk..............",
    "....................kkRRRRRRRRRRRRRRRRRRRRRrYYk.............",
    "...kk..............kRRRRRRRRRRRRRRRRRRRRRRRrYYk.............",
    "..kSSk............kRRRRRRRRRRrrrrrrrrrrrrrYYk...............",
    ".kSRRRk..........kkrRRRRRRrrkYYyYYyYYyYYyYYk................",
    "kSRRRRRkkk.....kkRRkrrrrrrrkyYYyYYyYYyYYyYk.................",
    ".krRRRrRRRkk.kkRRRrrkkrrrrrkkyyyyyyyyyyyykk.................",
    "..krrrk.kkrRRkRRrrrkrrkkkkk..kkkkkkkkrkkkk..................",
    "...kkk....kkrrrrkk.krrrk...........krrk.....................",
    "............kkkk....krrrk.........krHk......................",
    ".....................kHkHk........kHk.......................",
    "......................k.k..........k........................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
];

pub const MILIEU: [&str; 47] = [
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "..........................................kk....kk..........",
    ".................................k.........kHk..kHk.........",
    "....................kkkkkkkkkkkkkdk.........kHk..kHk........",
    "........kkkkkkkkkkkkdddddddddddddkk..........kHkkkHk........",
    ".......kddddddddddddDDDDDDDDDdkddddk.........kRRRRRkkkk.....",
    "........kkkkDDDDDkkkkkkkkkkkkkHkDddk........kRSSSSSRRRRkk...",
    ".....kkkkkkkkkkkkRRRRRRRRRRRRRkkDDddk......kRRRRkekSSSSRRk..",
    "....kRRRRRRRRRRRRMMMMMMMMMRRRRRRkDDddk.....kRRRRRRRRRRRRRkkk",
    ".....kkkkMMMMMMMMMMMMMRRRRMRMMRRkDDddk.....krRRRRRRRRRRRRRRk",
    ".........kkkMMMMMMRRRRMMMMRMMMMRRkDDddk....kkrRRRRrrrrrrrkk.",
    "........kkMMMMRRRRMMMMMMRRMMMMMMRRkDDddk..kSkYrrrkkkkkkkk...",
    ".......kMMRRRRMMMMMMMMRRMMMMMMMMRRkDDddk.kSRRkYYk...........",
    "......kRRRMMMMMMMMMMMRMMMMMMMMMMMRRkDDddkSRRYYkk............",
    ".......kkkkkkkMMMMMRRMMMMMMMMMMMMMRRkDDkSRRrYk..............",
    "..............kMMRRMMMMMMMMMMMMMMMRRkkkSRRrYk...............",
    "..............kRRMMMMkkkMMMMMMMMMMMRRkSSRRRrYk..............",
    "..............kRkkkkkkRRkkkMMMMMMMMMkRRRRRRrYYk.............",
    "...kk..........k...kRRRRRRRkkkkkkkkkRRRRRRRrYYk.............",
    "..kSSk............kRRRRRRRRRRrrrrrrrrrrrrrYYk...............",
    ".kSRRRk..........kkrRRRRRRrrkYYyYYyYYyYYyYYk................",
    "kSRRRRRkkk.....kkRRkrrrrrrrkyYYyYYyYYyYYyYk.................",
    ".krRRRrRRRkk.kkRRRrrkkrrrrrkkyyyyyyyyyyyykk.................",
    "..krrrk.kkrRRkRRrrrkrrkkkkk..kkkkkkkkrkkkk..................",
    "...kkk....kkrrrrkk.krrrk...........krrk.....................",
    "............kkkk....krrrk.........krHk......................",
    ".....................kHkHk........kHk.......................",
    "......................k.k..........k........................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
];

pub const BAS: [&str; 47] = [
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "..........................................kk....kk..........",
    "...........................................kHk..kHk.........",
    "............................................kHk..kHk........",
    ".............................................kHkkkHk........",
    ".............................................kRRRRRkkkk.....",
    "............................................kRSSSSSRRRRkk...",
    "...........................................kRRRRkekSSSSRRk..",
    "...........................................kRRRRRRRRRRRRRkkk",
    "...........................................krRRRRRRRRRRRRRRk",
    "...........................................kkrRRRRrrrrrrrkk.",
    "..........................................kSkYrrrkkkkkkkk...",
    ".........................................kSRRkYYk...........",
    "........................................kSRRYYkk............",
    "..........................k.k.k.k.k.k..kSRRrYk..............",
    ".........................kRkRkRkRkRkRkkSRRrYk...............",
    "......................kkRSSSSSSSSSSSSSSSRRRrYk..............",
    "....................kkRRRRRRRRRRRRRRRRRRRRRrYYk.............",
    "...kk..............kRRRRRRRRRRRRRRRRRRRRRRRrYYk.............",
    "..kSSk............kRRRRRRRRRRrrrrrrrrrrrrrYYk...............",
    ".kSRRRk..........kkrRRRRRRrrkYYyYYyYYyYYyYYk................",
    "kSRRRRRkkk.....kkRRkrrrrkMMMMMMMMMMRRkYYyYk.................",
    ".krRRRrRRRkk.kkRRRrrkkrrkMMMMMMMMMMRRkyyykk.................",
    "..krrrk.kkrRRkRRrrrkrrkkkMMMMMMMMMMRRkkkkk..................",
    "...kkk....kkrrrrkk.krrrkkMMMMMMMMMMMRRkddk..................",
    "............kkkk....krkkMMMMMMMMMMMMRRkddk..................",
    "....................kkMMMMMMMMMMMMMMRRkDddk.................",
    "....................RRRRRRMMMMMMMMMMMRRkddk.................",
    "....................kkMMMMRRRRRRMMMMMRRkddk.................",
    "......................kkMMMMMMMMRRRRRRRkkk..................",
    "........................kkMMMMMRRRMMRkkk....................",
    "........................kMMMRRRMMMMRkdk.....................",
    ".......................kMRRRMMMMMMRkdk......................",
    "......................kRRkkkkMMMMRkddk......................",
    ".......................kk....kMMRRkdk.......................",
    "..............................kMRkdk........................",
    "..............................kRk.k.........................",
    "...............................k............................",
];

pub const GUEULE: [&str; 47] = [
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "..........................................kk....kk..........",
    ".................................k.........kHk..kHk.........",
    "....................kkkkkkkkkkkkkdk.........kHk..kHk........",
    "........kkkkkkkkkkkkdddddddddddddkk..........kHkkkHk........",
    ".......kddddddddddddDDDDDDDDDdkddddk.........kRRRRRkkkk.....",
    "........kkkkDDDDDkkkkkkkkkkkkkHkDddk........kRSSSSSRRRRkk...",
    ".....kkkkkkkkkkkkRRRRRRRRRRRRRkkDDddk......kRRRRkekSSSSRRk..",
    "....kRRRRRRRRRRRRMMMMMMMMMRRRRRRkDDddk.....kRRRRRRRRRRRRRkkk",
    ".....kkkkMMMMMMMMMMMMMRRRRMRMMRRkDDddk.....krRRRRRRRkkkkk...",
    ".........kkkMMMMMMRRRRMMMMRMMMMRRkDDddk....kkrRRRRk.........",
    "........kkMMMMRRRRMMMMMMRRMMMMMMRRkDDddk..kSkYrrrrkkkk......",
    ".......kMMRRRRMMMMMMMMRRMMMMMMMMRRkDDddk.kSRRkYYkrrrrrkk....",
    "......kRRRMMMMMMMMMMMRMMMMMMMMMMMRRkDDddkSRRYYkk.kkkkkk.....",
    ".......kkkkkkkMMMMMRRMMMMMMMMMMMMMRRkDDkSRRrYk..............",
    "..............kMMRRMMMMMMMMMMMMMMMRRkkkSRRrYk...............",
    "..............kRRMMMMkkkMMMMMMMMMMMRRkSSRRRrYk..............",
    "..............kRkkkkkkRRkkkMMMMMMMMMkRRRRRRrYYk.............",
    "...kk..........k...kRRRRRRRkkkkkkkkkRRRRRRRrYYk.............",
    "..kSSk............kRRRRRRRRRRrrrrrrrrrrrrrYYk...............",
    ".kSRRRk..........kkrRRRRRRrrkYYyYYyYYyYYyYYk................",
    "kSRRRRRkkk.....kkRRkrrrrrrrkyYYyYYyYYyYYyYk.................",
    ".krRRRrRRRkk.kkRRRrrkkrrrrrkkyyyyyyyyyyyykk.................",
    "..krrrk.kkrRRkRRrrrkrrkkkkk..kkkkkkkkrkkkk..................",
    "...kkk....kkrrrrkk.krrrk...........krrk.....................",
    "............kkkk....krrrk.........krHk......................",
    ".....................kHkHk........kHk.......................",
    "......................k.k..........k........................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
];

/// La gerbe de feu, deux images qui alternent, de la gueule vers l'avant.
pub const FLAMME: [[&str; 9]; 2] = [
    [
        "..........gg......",
        ".......ggffgg..g..",
        "....ggffffffgg....",
        "ffFFFFFFFFffffgg..",
        "FFFFFFFFFFFFFffg.g",
        "ffFFFFFFFFffffgg..",
        "....ggffffffgg....",
        ".......ggffgg.....",
        "..........gg..g...",
    ],
    [
        ".........gg...g...",
        "......ggffgg......",
        "....ggfffffggg..g.",
        "ffFFFFFFFFfffffg..",
        "FFFFFFFFFFFFFFffgg",
        "ffFFFFFFFFfffffg..",
        "....ggfffffggg....",
        "......ggffgg..g...",
        ".........gg.......",
    ],
];

/// Le milieu du corps dans les images (colonne, rang) : ce que paysage.rs pose sur sa trajectoire.
pub const CORPS: (i32, i32) = (30, 23);
/// Le coin haut gauche de la flamme dans les images, devant la gueule ouverte.
pub const GUEULE_FEU: (i32, i32) = (57, 15);

/// La couleur d'une lettre, de jour ou de nuit, et si elle luit d'elle-même (l'œil, la braise, le feu).
pub fn couleur(l: u8, nuit: bool) -> Option<(u32, bool)> {
    Some(match (l, nuit) {
        (b'e', _) => (0xffd23a, true),
        (b'F', _) => (0xfff0a0, true),
        (b'f', _) => (0xffa03a, true),
        (b'g', _) => (0xe0482a, true),
        (b'k', false) => (0x24141a, false),
        (b'R', false) => (0xb83a28, false),
        (b'r', false) => (0x7c2420, false),
        (b'S', false) => (0xe0603c, false),
        (b'Y', false) => (0xecb850, false),
        (b'y', false) => (0xb07830, false),
        (b'M', false) => (0xd8743c, false),
        (b'D', false) => (0x6c2a22, false),
        (b'd', false) => (0x4a1c1a, false),
        (b'H', false) => (0xeee0bc, false),
        // la nuit : une silhouette d'encre, le ventre de braise
        (b'k', true) => (0x08080e, false),
        (b'R', true) => (0x221a2c, false),
        (b'r', true) => (0x161220, false),
        (b'S', true) => (0x2e2438, false),
        (b'Y', true) => (0xff8a2a, true),
        (b'y', true) => (0xc8461c, true),
        (b'M', true) => (0x2c2236, false),
        (b'D', true) => (0x141020, false),
        (b'd', true) => (0x0e0a16, false),
        (b'H', true) => (0x403a50, false),
        _ => return None,
    })
}

/// L'image du battement à ce tic : ailes hautes, à mi-course, basses, à mi-course, trois tics chacune (un grand
/// dragon bat lentement) ; `feu` : la gueule ouverte, ailes à mi-course, planées.
pub fn image(tic: i64, feu: bool) -> &'static [&'static str] {
    if feu {
        return &GUEULE;
    }
    match tic.div_euclid(3).rem_euclid(4) {
        0 => &HAUT,
        2 => &BAS,
        _ => &MILIEU,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn images_droites_et_lettres_connues() {
        for img in [&HAUT[..], &MILIEU[..], &BAS[..], &GUEULE[..], &FLAMME[0][..], &FLAMME[1][..]] {
            let large = img[0].len();
            for l in img {
                assert_eq!(l.len(), large, "{l}");
                for c in l.bytes() {
                    assert!(c == b'.' || (couleur(c, false).is_some() && couleur(c, true).is_some()), "lettre {}", c as char);
                }
            }
        }
        assert_eq!(HAUT.len(), BAS.len());
        assert_eq!(HAUT[0].len(), BAS[0].len());
    }
}
