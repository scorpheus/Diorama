"""Les habitants faits d'un même gabarit : le corps et les poses du marin (habitants/marin.json),
la tête, les couleurs et l'ouvrage propres à chacun. Écrit habitants/<nom>.json (valide: false).

    python tools/habitants/gabarits.py            # tous
    python tools/habitants/gabarits.py forgeron   # un seul

Un nouvel habitant (une demande de habitants/demandes/) : une entrée de plus dans HABITANTS, avec
ses trois têtes (profil, dort, face et sa bouche ouverte), sa palette et son ouvrage, puis
tools/habitants/apercu.py pour la planche.

Lettres du gabarit : k contour, C c le vêtement et son ombre, b le détail (bouton, ceinture),
S le foulard (remplacé par la lettre que l'habitant choisit, O par défaut : la couleur du profil),
p la peau des mains, n les jambes, a les chaussures, u les Z, x X la fumée, O o la couleur du profil.
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from ranger import ranger  # noqa: E402

RACINE = os.path.join(os.path.dirname(__file__), '..', '..', 'habitants')
VIDE = '.' * 16

# -- le corps, pose par pose (rangées du bas ; la tête se pose au-dessus) ---------------------------

# au travail, debout, tourné vers la droite : le bras haut, puis bas (rangées 9 à 15)
TRAVAIL_HAUT = [
    "..kcSSSSSck.....",
    ".kcCCCCbCCpk....",
    ".kCCCCCCCck.....",
    "..kCCCbCCk......",
    "..kcccccck......",
    "..knnk.knk......",
    ".kaaak.kaak.....",
]
TRAVAIL_BAS = TRAVAIL_HAUT[:1] + [
    ".kcCCCCbCCk.....",
    ".kCCCCCCCcpk....",
] + TRAVAIL_HAUT[3:]

# l'attente, le bras levé qui tient le drapeau (rangées 9 à 15, et la main en 7 et 8)
ATTENTE = [
    "..kcSSSSScCk....",
    "..kCCCCbCCk.....",
    ".kpCCCCCCck.....",
    "..kCCCbCCck.....",
    "..kccccccck.....",
    "...knnk.knk.....",
    "..kaaak.kaak....",
]
BRAS_LEVE = [(11, 7, ["p"]), (10, 8, ["pk"])]
DRAPEAU = [
    [".........k......",
     ".........kOOOw..",
     ".........kOOww..",
     ".........kOww...",
     ".........kw.....",
     ".........k......",
     "..........k....."],
    ["...........k....",
     "..........kk.OO.",
     "..........kOOOww",
     "..........kOOww.",
     "..........kw....",
     "..........k.....",
     "..........k....."],
]

# le repos : assis, la tête basse (rangées 11 à 15 ; la tête en 6 à 10)
REPOS = [
    ".kcSSSSSSSck....",
    ".kCCCCbCCCCk....",
    ".kccccccccpkk...",
    "..knnnnnnnnaak..",
    "..kkkkkkkkkkkk..",
]
Z = ["uuuu", "...u", "..u.", ".u..", "uuuu"]
ZZZ = [[], [(10, 3, ["u"])], [(10, 1, Z)], [(12, 0, Z), (9, 4, ["u"])]]

# l'échec : assis par terre, de face, dans la fumée (rangées 10 à 15 ; la tête en 4 à 9)
ECHEC = [
    "..kcSSSSSSSck...",
    ".kpCCCCbCCCCpk..",
    ".kkccccccccckk..",
    "..knnnnnnnnnnk..",
    ".kaakkkkkkkkaak.",
    ".kkk........kkk.",
]
FUMEE = [
    [(7, 2, [".x.", "xXx", "..."]), (10, 4, ["x"])],
    [(7, 1, ["..X", ".xXx", "xx.."])],
    [(8, 0, [".XX", "xXx", ".x.", "x.."]), (11, 4, ["x"])],
]

# parle : debout, de face (rangées 9 à 15 ; la tête en 3 à 8)
PARLE = [
    "...kcSSSSSSSck..",
    "..kcCCCCbCCCCk..",
    "..kpCCCCCCCcpk..",
    "...kCCCCbCCCk...",
    "...kcccccccck...",
    "....knnk.knnk...",
    "...kaaak.kaaak..",
]

# -- les ouvrages (l'état travail) : chaque image dit son bras (haut ou bas) et ce qu'on pose ---------

CLAVIER = [
    ('haut', [(9, 11, [".kkkkkk", "kdgsgdk", "krrrrrk", "krRRRrk", "kkkkkkk"])]),
    ('bas', [(9, 11, [".pkkkkk", "kdggsdk", "krrrrrk", "krRRRrk", "kkkkkkk"])]),
]
# le marteau levé, puis abattu sur l'enclume, et l'étincelle
ENCLUME = [
    ('haut', [(10, 6, ["..kk.", ".kAAk", "rkAk.", "r....", "....."]),
              (10, 12, [".kkkkk", "kAAAAk", ".kgk..", "kggggk"])]),
    ('bas', [(10, 9, ["......", ".....", ".rrkkk", "..kAAk"]),
             (10, 12, [".kkkkk", "kAAAAk", ".kgk..", "kggggk"]), (13, 10, ["s.s"])]),
    ('bas', [(10, 9, ["......", ".....", ".rrkkk", "..kAAk"]),
             (10, 12, [".kkkkk", "kAAAAk", ".kgk..", "kggggk"]), (12, 9, ["s...s"]), (14, 11, ["s"])]),
]
# la clé sur le moyeu d'une roue
ROUE = [
    ('haut', [(10, 9, ["..kkk.", ".kaaak", "kaAAak", "kaAkAa", "kaAAak", ".kaaak", "..kkk."]),
              (11, 10, ["g", "g"])]),
    ('bas', [(10, 9, ["..kkk.", ".kaaak", "kaAAak", "kaAkAa", "kaAAak", ".kaaak", "..kkk."]),
             (11, 11, ["gg"])]),
]
# le ciseau et la massette sur un bloc où s'écrit une lettre, les éclats
PIERRE = [
    ('haut', [(10, 11, ["kkkkkk", "kWQQQk", "kQVQVk", "kQVVQk", "kkkkkk"]), (10, 9, ["rk", ".g"])]),
    ('bas', [(10, 11, ["kkkkkk", "kWQQQk", "kQVQVk", "kQVVQk", "kkkkkk"]), (10, 10, ["rk"]),
             (12, 9, ["W.W"]), (14, 10, ["W"])]),
]
# les mains sur l'orbe qui luit, posé sur un pied
ORBE = [
    ('haut', [(10, 8, [".kkk.", "kfFfk", "kFsFk", "kfFfk", ".kkk.", "..r..", "..r..", ".rrr."])]),
    ('bas', [(10, 8, [".kkk.", "kFfFk", "kfYfk", "kFfFk", ".kkk.", "..r..", "..r..", ".rrr."]),
             (9, 7, ["s"]), (15, 9, ["s"])]),
]
# le volant tenu, qui tourne d'un coup à l'autre
VOLANT = [
    ('haut', [(10, 8, [".kkk.", "kA.Ak", "k.y.k", "k.A.k", ".kkk.", "..k..", "..k.."])]),
    ('bas', [(10, 8, [".kkk.", "kA..k", "k.yAk", "kA..k", ".kkk.", "..k..", "..k.."])]),
]

# -- les habitants -------------------------------------------------------------------------------------

BASE = {
    'k': '#1b1a24', 'p': '#ecb48a', 'q': '#b97a55', 'w': '#f4f1ea', 'm': '#141018', 'M': '#7a2a2a',
    'u': '#bff3ff', 'x': '#5e5966', 'X': '#8d8896', 'O': '#3f80dc', 'o': '#1f4f9e',
}

HABITANTS = {
    'voyageur': {
        'description': "le voyageur générique, en cape à capuche à la couleur du profil : tout profil sans habitant",
        'palette': {'n': '#5a4636', 'a': '#2e343a', 'b': '#7a4a24', 'r': '#7a4a24', 'R': '#4a2c16',
                    'g': '#b4b7bf', 'd': '#7c808c', 's': '#fff3b0'},
        'lettres': {'C': 'O', 'c': 'o', 'S': 'o'},
        'ouvrage': CLAVIER,
        'profil': [
            "....kkkkk.......",
            "...kOOOOOk......",
            "..kOoooooOk.....",
            "..kokppmpk......",
            "..kokppppqk.....",
            "...kokqppk......",
        ],
        'dort': [
            "....kkkkk.......",
            "...kOOOOOk......",
            "..kOoooooOk.....",
            "..koOOOOOok.....",
            "...kkqppqk......",
        ],
        'face': [
            ".....kkkkkk.....",
            "....kOOOOOOk....",
            "...kOooooooOk...",
            "...kOkmppmkOk...",
            "...kokppppkok...",
            "....kkqppqkk....",
        ],
        'bouche': (7, 8, ["MM"]),
    },
    'galadriel': {
        'profil_couleur': '#9a6ad0',
        'description': "Galadriel, l'orchestratrice : cheveux d'or, robe blanche, l'orbe pour ouvrage",
        'palette': {'h': '#f0d27a', 'H': '#b8923a', 'C': '#eceaf4', 'c': '#a9a6c8', 'b': '#e3a63c',
                    'y': '#ffffff', 'r': '#8a90a0', 'f': '#bfe8ff', 'F': '#7fc8f0', 'Y': '#ffffff',
                    's': '#fff3b0'},
        'lettres': {'n': 'c', 'a': 'c', 'S': 'O'},
        'robe': True,
        'ouvrage': ORBE,
        'profil': [
            "....kkkk........",
            "...khhhhk.......",
            "..khHHyHhk......",
            "..khhppmpk......",
            "..khhpppppk.....",
            "..khhkqppk......",
        ],
        'dort': [
            "....kkkk........",
            "...khhhhk.......",
            "..khHHyHhk......",
            "..khhhqpqk......",
            "..khhkppk.......",
        ],
        'face': [
            "......kkkk......",
            ".....khhhhk.....",
            "....khHyyHhk....",
            "...khpmppmphk...",
            "...khppppppqk...",
            "...khhkqppkhhk..",
        ],
        'bouche': (7, 8, ["M"]),
        'dos': [(2, 9, ["h", "h", "H"])],
    },
    'forgeron': {
        'profil_couleur': '#d04a2a',
        'description': "le forgeron des tickets d'Endurance : tablier de cuir, barbe brune, marteau et enclume",
        'palette': {'h': '#4a2c16', 'C': '#8a5a30', 'c': '#5a3818', 'b': '#c9c4ba', 'n': '#3a3440',
                    'a': '#2e2420', 'r': '#7a4a24', 'g': '#4a4e5c', 'A': '#a8aebc', 's': '#ffd75e'},
        'lettres': {'S': 'O'},
        'ouvrage': ENCLUME,
        'profil': [
            "....kkkk........",
            "...khhhhk.......",
            "..khhhhhhk......",
            "..khppppmpk.....",
            "..kkppppppk.....",
            "...khhhhhk......",
        ],
        'dort': [
            "....kkkk........",
            "...khhhhk.......",
            "..khhhhhhk......",
            "..khpqpqpk......",
            "...khhhhhk......",
        ],
        'face': [
            "......kkkk......",
            ".....khhhhk.....",
            "....khhhhhhk....",
            "....kpmppmpk....",
            "....kppppppk....",
            "....khhhhhhk....",
        ],
        'bouche': (7, 8, ["MM"]),
    },
    'mecano': {
        'profil_couleur': '#e0902a',
        'description': "le mécano de vehigraph : bleu de travail, casquette à la couleur du profil, clé et roue",
        'palette': {'h': '#3a2a1e', 'C': '#2f5a9e', 'c': '#1e3a6e', 'b': '#c9c4ba', 'n': '#2f5a9e',
                    'a': '#2e343a', 'A': '#8a90a0', 'g': '#c9ccd4'},
        'lettres': {'S': 'w'},
        'ouvrage': ROUE,
        'profil': [
            "....kkkkk.......",
            "...kOOOOOk......",
            "..kooooooookk...",
            "..khppppmpk.....",
            "..kkppppppk.....",
            "...kqpppqk......",
        ],
        'dort': [
            "....kkkkk.......",
            "...kOOOOOk......",
            "..kooooooookk...",
            "..khpqpqpk......",
            "...kqpppqk......",
        ],
        'face': [
            "......kkkkk.....",
            ".....kOOOOOk....",
            "...kkoooooookk..",
            "....kpmppmpk....",
            "....kppppppk....",
            ".....kqppqk.....",
        ],
        'bouche': (7, 8, ["MM"]),
    },
    'tailleur': {
        'profil_couleur': '#3f8a4a',
        'description': "le tailleur de pierre de LaPierreDesMots : béret à la couleur du profil, ciseau, une lettre au bloc",
        'palette': {'h': '#9a948c', 'C': '#c8bfae', 'c': '#8e8676', 'b': '#7a4a24', 'n': '#5a5048',
                    'a': '#3a2a1e', 'r': '#7a4a24', 'g': '#c9ccd4', 'Q': '#b0aab8', 'V': '#6e6876', 'W': '#e4e0ea'},
        'lettres': {'S': 'b'},
        'ouvrage': PIERRE,
        'profil': [
            "................",
            "...kkkkkk.......",
            "..kOOOOOOk......",
            "..khkppmpk......",
            "..kkppppppk.....",
            "...khhhhk.......",
        ],
        'dort': [
            "................",
            "...kkkkkk.......",
            "..kOOOOOOk......",
            "..khkqpqpk......",
            "...khhhhk.......",
        ],
        'face': [
            "................",
            ".....kkkkkk.....",
            "....kOOOOOOk....",
            "....kpmppmpk....",
            "....kppppppk....",
            ".....khhhhk.....",
        ],
        'bouche': (7, 8, ["MM"]),
    },
    'europa': {
        'profil_couleur': '#d04a2a',
        'description': "la pilote de rallye d'Europa (VRallye) : casque à la couleur du profil, combinaison, volant",
        'palette': {'C': '#e8e8ee', 'c': '#a8acb8', 'b': '#f0c020', 'n': '#a8acb8', 'a': '#2e343a',
                    'e': '#22304a', 'E': '#7fd4ff', 'A': '#b4b7bf', 'y': '#f0c020'},
        'lettres': {'S': 'O'},
        'ouvrage': VOLANT,
        'profil': [
            "....kkkkk.......",
            "...kOOOOOk......",
            "..kOOOOOOOk.....",
            "..kOOkeeEek.....",
            "..kOOkeeeek.....",
            "...koOOOok......",
        ],
        'dort': [
            "....kkkkk.......",
            "...kOOOOOk......",
            "..kOOOOOOOk.....",
            "..kOkeeeEk......",
            "...koOOOok......",
        ],
        'face': [
            "......kkkkk.....",
            ".....kOOOOOk....",
            "....kOOOOOOOk...",
            "....kOkmppmkOk..",
            "....kOkppppkOk..",
            ".....kokqqkok...",
        ],
        'bouche': (8, 8, ["MM"]),
    },
    'claudeterminal': {
        'profil_couleur': '#3fb0a8',
        'description': "le petit robot de ClaudeTerminal : un écran cathodique pour tête, le curseur qui clignote",
        'palette': {'W': '#e6dcc0', 'V': '#a89a74', 'z': '#10241a', 'G': '#74e07a', 'C': '#8a90a0',
                    'c': '#4a4e5c', 'b': '#74e07a', 'n': '#4a4e5c', 'a': '#2e343a', 'r': '#7a4a24',
                    'R': '#4a2c16', 'g': '#b4b7bf', 'd': '#7c808c', 's': '#fff3b0', 'B': '#2b5fd8'},
        'lettres': {'S': 'O', 'p': 'c'},
        'ouvrage': CLAVIER,
        'profil': [
            "..kkkkkkkk......",
            "..kWWWWWWVk.....",
            "..kWzGzzzVk.....",
            "..kWzzGzzVk.....",
            "..kWzGzGGVk.....",
            "..kkVVVVVkk.....",
        ],
        'dort': [
            "..kkkkkkkk......",
            "..kWWWWWWVk.....",
            "..kWzzzzzVk.....",
            "..kWzzzzzVk.....",
            "..kkVVVVVkk.....",
        ],
        'face': [
            "....kkkkkkkk....",
            "....kWWWWWWVk...",
            "....kWzGzzGVk...",
            "....kWzzzzzVk...",
            "....kWzzzzzVk...",
            "....kkVVVVVkk...",
        ],
        'bouche': (7, 7, ["GG"]),
        'triste': [(5, 5, ["BBBBB", "BwBwB", "BBBBB", "BwwwB"])],
    },
}


def poser(img, x, y, motif):
    for dy, ligne in enumerate(motif):
        if not 0 <= y + dy < 16:
            continue
        l = list(img[y + dy])
        for dx, c in enumerate(ligne):
            if c != '.' and 0 <= x + dx < 16:
                l[x + dx] = c
        img[y + dy] = ''.join(l)


def toile(corps, haut):
    img = [VIDE] * 16
    for i, l in enumerate(corps):
        img[haut + i] = l
    return img


def lettres(img, sub):
    return [''.join(sub.get(c, c) for c in l) for l in img]


def robe(img, rangees):
    """Les jambes d'une robe : de bord à bord, sans jour entre elles, et l'ourlet en contour."""
    for y in rangees:
        l = img[y]
        g, d = l.find('k'), l.rfind('k')
        if g < 0:
            continue
        mil = 'k' if y == 15 else 'c'
        img[y] = l[:g] + 'k' + mil * (d - g - 1) + 'k' + l[d + 1:]


def habitant(nom, h):
    sub = {'S': 'O'} | h.get('lettres', {})
    dos = h.get('dos', [])

    def fini(img, debout=False):
        if h.get('robe') and debout:
            robe(img, [14, 15])
        return lettres(img, sub)

    etats = {}
    images = []
    for bras, objets in h['ouvrage']:
        img = toile(TRAVAIL_HAUT if bras == 'haut' else TRAVAIL_BAS, 9)
        for x, y, m in dos:
            poser(img, x, y, m)
        poser(img, 0, 3, h['profil'])
        for x, y, m in objets:
            poser(img, x, y, m)
        images.append(fini(img, True))
    etats['travail'] = {'every': 2, 'images': images}

    images = []
    for zs in ZZZ:
        img = toile(REPOS, 11)
        for x, y, m in dos:
            poser(img, x, y + 2, m)
        poser(img, 0, 6, h['dort'])
        for x, y, m in zs:
            poser(img, x, y, m)
        images.append(fini(img))
    etats['repos'] = {'every': 4, 'images': images}

    images = []
    for drapeau in DRAPEAU:
        img = toile(ATTENTE, 9)
        for x, y, m in dos:
            poser(img, x, y, m)
        poser(img, 0, 0, drapeau)
        poser(img, 0, 3, h['profil'])
        for x, y, m in BRAS_LEVE:
            poser(img, x, y, m)
        images.append(fini(img, True))
    etats['attente'] = {'every': 2, 'images': images}

    images = []
    for fumee in FUMEE:
        img = toile(ECHEC, 10)
        for x, y, m in fumee:
            poser(img, x, y, m)
        poser(img, -1, 4, h['face'])
        bx, by, bm = h['bouche']
        poser(img, bx - 1, by + 1, bm)
        for x, y, m in h.get('triste', []):
            poser(img, x - 1, y + 1, m)
        images.append(fini(img))
    etats['echec'] = {'every': 3, 'images': images}

    images = []
    for ouverte in (False, True):
        img = toile(PARLE, 9)
        poser(img, 0, 3, h['face'])
        if ouverte:
            poser(img, *h['bouche'])
        images.append(fini(img, True))
    etats['parle'] = {'every': 2, 'images': images}

    for e in etats.values():
        for img in e['images']:
            assert all(len(l) == 16 for l in img) and len(img) == 16, nom
    utilisees = {c for e in etats.values() for img in e['images'] for l in img for c in l} - {'.'}
    pal = BASE | h['palette']
    if 'profil_couleur' in h:
        o = h['profil_couleur']
        pal |= {'O': o, 'o': '#' + ''.join(f'{round(int(o[i:i + 2], 16) * 0.62):02x}' for i in (1, 3, 5))}
    manque = utilisees - set(pal)
    assert not manque, f'{nom} : lettres sans couleur {sorted(manque)}'
    return {
        'nom': nom,
        'description': h['description'],
        'valide': False,
        'taille': [16, 16],
        'palette': {k: v for k, v in pal.items() if k in utilisees or k in 'Oo'},
        'etats': etats,
    }


if __name__ == '__main__':
    noms = sys.argv[1:] or list(HABITANTS)
    for nom in noms:
        chemin = os.path.join(RACINE, f'{nom}.json')
        ancien = json.load(open(chemin, encoding='utf-8')) if os.path.exists(chemin) else None
        h = habitant(nom, HABITANTS[nom])
        # un dessin validé par le user le reste : le gabarit ne le repasse pas à false en silence
        if ancien and ancien.get('valide'):
            sys.exit(f'{nom} est validé : le refaire demande de remettre valide à false à la main')
        open(chemin, 'w', encoding='utf-8', newline='\n').write(ranger(h))
        print(nom)
