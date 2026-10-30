"""Les habitants faits d'un même gabarit : un corps élancé (2,5 têtes, tête de 5 px, torse de 4,
bras d'un pixel, jambes séparées), la tête, les couleurs et l'ouvrage propres à chacun. Écrit
habitants/<nom>.json (valide: false). Galadriel et le robot, qui ne sont pas faits comme les
autres, donnent leurs corps entiers.

    python tools/habitants/gabarits.py            # tous
    python tools/habitants/gabarits.py forgeron   # un seul

Un nouvel habitant (une demande de habitants/demandes/) : une entrée de plus dans HABITANTS, puis
tools/habitants/apercu.py pour la planche.

Repères (petits personnages lisibles, cf. pixnote.net/en/learn/character) : silhouette qui se
reconnaît remplie de noir (chapeau, outil, barbe), bras d'un pixel, yeux de 1 × 2, visage qui
montre la peau, 6 à 12 couleurs.

Lettres du corps : k contour, C c le vêtement et son ombre (le bras), w le col, b un détail,
p la main, n les jambes, a les chaussures ; chacun les remplace par les siennes (`corps`).
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from ranger import ranger  # noqa: E402

RACINE = os.path.join(os.path.dirname(__file__), '..', '..', 'habitants')
VIDE = '.' * 16

# -- le corps commun, tourné vers la droite -----------------------------------------------------------

# debout : rangées 8 à 15 (la tête en 2 à 7)
DEBOUT = [
    "...kCwwCk.......",
    "...kCCcCk.......",
    "...kCbcCk.......",
    "...kCCpCk.......",
    "...knnnnk.......",
    "....knknk.......",
    "....knknk.......",
    "...kaakaak......",
]
# le bras qui pend devant le torse : effacé quand il se lève ou travaille
BRAS = [(6, 9), (6, 10), (6, 11)]
EPAULE = (8, 8)

# assis par terre, les jambes devant : rangées 11 à 15 (la tête descend de 3)
ASSIS = [
    "...kCwwCk.......",
    "...kCCcCk.......",
    "...kCCpCk.......",
    "...knnnnnnnaak..",
    "...kkkkkkkkkkk..",
]

Z = ["uuuu", "...u", "..u.", ".u..", "uuuu"]
ZZZ = [[], [(11, 3, ["u"])], [(11, 1, Z)], [(12, 0, Z), (10, 4, ["u"])]]
FUMEE = [
    [(8, 2, [".x.", "xXx"]), (1, 9, ["x"]), (12, 10, ["X"])],
    [(8, 1, ["..X", ".xXx", "xx.."]), (1, 8, ["X"]), (12, 9, ["x"])],
    [(9, 0, [".XX", "xXx", ".x."]), (0, 9, ["x"]), (13, 10, ["x"]), (10, 4, ["x"])],
]


def drapeau(x, y, f):
    """Le bras levé (épaule en x, y), la main, la hampe et le drapeau qui flotte ; image f."""
    m = [(x + 1, y, ["C"]), (x + 2, y - 1, ["C"]), (x + 2, y - 2, ["p"])]
    hampe = [(x + 2, y - 8 + i, ["r"]) for i in range(6)]
    tissu = [["OOOw", "OOww", "Oww."], [".OOw", "OOOw", "OOww"]][f]
    return hampe + m + [(x + 3, y - 8, tissu)]


# -- les ouvrages (l'état travail) : le bras tendu et l'objet, image par image --------------------------

def bras_devant(main):
    """Le bras tendu de l'épaule (8, 9) à la main ; main = (x, y)."""
    x, y = main
    return [(8, 9, ["C"])] + ([(9, y, ["C"])] if x > 9 else []) + [(x, y, ["p"])]


CLAVIER = [
    bras_devant((10, 10)) + [(10, 11, ["kkkkkk"]), (10, 12, ["dgsgdk"]), (9, 13, ["krrrrrk"]), (9, 14, ["krRRRrk"]), (9, 15, ["kkkkkkk"])],
    bras_devant((10, 11)) + [(11, 11, ["kkkkk"]), (10, 12, ["dggsdk"]), (9, 13, ["krrrrrk"]), (9, 14, ["krRRRrk"]), (9, 15, ["kkkkkkk"])],
]
_ENCLUME = [(10, 12, [".kkkkk"]), (10, 13, ["kAAAAk"]), (11, 14, ["kgk"]), (10, 15, ["kggggk"])]
ENCLUME = [
    bras_devant((9, 8)) + [(9, 5, [".kk", "kAAk", ".rk", ".r."])] + _ENCLUME,
    bras_devant((10, 10)) + [(11, 10, ["rr"]), (13, 10, ["kAk"]), (13, 11, ["kAk"])] + _ENCLUME + [(12, 9, ["s.s"])],
    bras_devant((10, 10)) + [(11, 10, ["rr"]), (13, 10, ["kAk"]), (13, 11, ["kAk"])] + _ENCLUME + [(11, 8, ["s...s"]), (15, 11, ["s"])],
]
_ROUE = [(11, 9, [".kkk.", "kaaak", "kaAak", "kaaak", ".kkk."])]
ROUE = [
    bras_devant((10, 10)) + _ROUE + [(11, 10, ["g"]), (12, 11, ["g"])],
    bras_devant((10, 12)) + _ROUE + [(11, 12, ["g"]), (12, 11, ["g"])],
]
_BLOC = [(10, 11, ["kkkkkk", "kWQQQk", "kQVQVk", "kQVVQk", "kkkkkk"])]
PIERRE = [
    bras_devant((10, 9)) + _BLOC + [(10, 8, ["r"]), (11, 10, ["g"])],
    bras_devant((10, 10)) + _BLOC + [(11, 9, ["r"]), (12, 8, ["W.W"]), (15, 9, ["W"])],
]
VOLANT = [
    bras_devant((10, 9)) + [(10, 8, [".kkk.", "kA.Ak", "k.y.k", "k.A.k", ".kkk.", "..k..", "..k..", ".kkk."])],
    bras_devant((10, 10)) + [(10, 8, [".kkk.", "kA..k", "k.yAk", "kA..k", ".kkk.", "..k..", "..k..", ".kkk."])],
]

# -- les habitants -------------------------------------------------------------------------------------

BASE = {
    'k': '#1b1a24', 'p': '#ecb48a', 'q': '#b97a55', 'w': '#f4f1ea', 'm': '#141018', 'M': '#7a2a2a',
    'u': '#bff3ff', 'x': '#5e5966', 'X': '#8d8896', 'O': '#3f80dc', 'o': '#1f4f9e', 'r': '#7a4a24',
    'R': '#4a2c16', 'g': '#b4b7bf', 'd': '#7c808c', 's': '#ffd75e', 'c': '#7fd4ff',
}

HABITANTS = {
    'marin': {
        'description': "le marin en ciré jaune et suroît, barbe blanche, l'écharpe à la couleur du profil (Diorama, Route du Rhum)",
        'palette': {'J': '#f8da1c', 'j': '#c9a40e', 'y': '#8a6a10', 'n': '#1e2a55', 'a': '#2e343a'},
        'corps': {'C': 'J', 'c': 'j', 'w': 'O', 'b': 'y'},
        'tete': [
            "....kkkk........",
            "...kJJJJk.......",
            "..kjjjjjjkk.....",
            "..kjkppmpk......",
            "...kpppmppk.....",
            "....kwwwk.......",
        ],
        'yeux': [(7, 5), (7, 6)],
        'bouche': (6, 7),
        'ouvrage': CLAVIER,
    },
    'voyageur': {
        'description': "le voyageur générique : chapeau à large bord, cape à la couleur du profil, sac au dos",
        'palette': {'h': '#5a3a22', 'H': '#8a6238', 'n': '#5a4636', 'a': '#2e2420', 'y': '#e3a63c'},
        'corps': {'C': 'O', 'c': 'o', 'w': 'y', 'b': 'O'},
        'tete': [
            ".....kkk........",
            "....kHHHk.......",
            "..kkhhhhhkk.....",
            "...khppmpk......",
            "...kpppmppk.....",
            "....kpppk.......",
        ],
        'yeux': [(7, 5), (7, 6)],
        'bouche': (7, 7),
        'dos': [(0, 8, [".kk", "khh", "khH", ".kk"])],
        'ouvrage': CLAVIER,
    },
    'forgeron': {
        'profil_couleur': '#d04a2a',
        'description': "le forgeron des tickets d'Endurance : crâne nu, barbe noire, bras nus, tablier de cuir, marteau et enclume",
        'palette': {'P': '#f6d0b0', 'h': '#2a1e1a', 'A': '#4a3a34', 'n': '#3a3440', 'a': '#2e2420',
                    'G': '#8a90a0'},
        'corps': {'C': 'A', 'c': 'p', 'w': 'h', 'b': 'O'},
        'tete': [
            "................",
            "....kkkkk.......",
            "...kpPPppk......",
            "...kpphhhk......",
            "...kpppmpPk.....",
            "....khhhhk......",
        ],
        'yeux': [(7, 6)],
        'bouche': (6, 7),
        # les épaules nues, la barbe sur la poitrine
        'plus': [(4, 8, ["p"]), (7, 8, ["p"])],
        'ouvrage': ENCLUME,
        'lettres_outil': {'A': 'G'},
    },
    'mecano': {
        'profil_couleur': '#e0902a',
        'description': "le mécano de vehigraph : bleu de travail, casquette à la couleur du profil, clé et roue",
        'palette': {'h': '#3a2a1e', 'C': '#2f5a9e', 'c': '#1e3a6e', 'n': '#264a84', 'a': '#2e343a',
                    'b': '#c9c4ba', 'A': '#8a90a0'},
        'corps': {},
        'tete': [
            "....kkkkk.......",
            "...kOOOOOk......",
            "...koooookkk....",
            "...khppmpk......",
            "...kpppmppk.....",
            "....kpppk.......",
        ],
        'yeux': [(7, 5), (7, 6)],
        'bouche': (7, 7),
        'ouvrage': ROUE,
    },
    'tailleur': {
        'profil_couleur': '#3f8a4a',
        'description': "le tailleur de pierre de LaPierreDesMots : béret à la couleur du profil, moustache grise, blouse, ciseau et bloc",
        'palette': {'h': '#a8a29a', 'C': '#c8bfae', 'c': '#8e8676', 'b': '#7a4a24', 'n': '#5a5048',
                    'a': '#3a2a1e', 'Q': '#b0aab8', 'V': '#6e6876', 'W': '#e4e0ea'},
        'corps': {},
        'tete': [
            "................",
            "....kkkkkk......",
            "...kOOOOOOk.....",
            "...khppmpkk.....",
            "...kpppmppk.....",
            "....khhhk.......",
        ],
        'yeux': [(7, 5), (7, 6)],
        'bouche': (6, 7),
        'ouvrage': PIERRE,
    },
    'europa': {
        'profil_couleur': '#d04a2a',
        'description': "la pilote de rallye d'Europa : casque ouvert à la couleur du profil, visière relevée, combinaison blanche, volant",
        'palette': {'C': '#eceef4', 'c': '#a8acb8', 'n': '#d4d6de', 'a': '#2e343a', 'G': '#3a3f4c',
                    'E': '#9fe0ff', 'A': '#b4b7bf', 'y': '#f0c020', 'h': '#5a3a22'},
        'corps': {'w': 'O', 'b': 'O', 'p': 'G'},
        'tete': [
            "....kkkkk.......",
            "...kOOOOOk......",
            "..kOOEEEEk......",
            "..kOOppmpk......",
            "..kOOppmppk.....",
            "...kOkppk.......",
        ],
        'yeux': [(7, 5), (7, 6)],
        'bouche': (7, 7),
        'ouvrage': VOLANT,
    },
    'galadriel': {
        'profil_couleur': '#9a6ad0',
        'description': "Galadriel, l'orchestratrice, comme dans le Diorama : capuche bleu nuit étoilée, liserés d'or, visage dans l'ombre, yeux cyan",
        'palette': {'N': '#26306a', 'n': '#161c46', 'y': '#e8c46a', 'z': '#0c0e1e', 'e': '#7ff0ff',
                    'E': '#2a8aa0', 'f': '#bff8ff'},
        'debout': [
            "................",
            ".......k........",
            "......kNk.......",
            ".....kNyNk......",
            "....kNNNNNk.....",
            "....kNzzzNk.....",
            "....kNezeNk.....",
            "....knNzNnk.....",
            "...knNyeyNnk....",
            "...kNnNeNnNk....",
            "..kNNnNENnNNk...",
            "..kNnNNNNNnNk...",
            ".kNNnNNNNNnNNk..",
            ".kNnNNyNNNnNNk..",
            ".kNNnNNNyNnNNk..",
            ".kyyyyyyyyyyyk..",
        ],
        'assis': [
            "................",
            "................",
            "................",
            "................",
            "................",
            ".......k........",
            "......kNk.......",
            ".....kNyNk......",
            "....kNNNNNk.....",
            "....kNzzzNk.....",
            "....kNezeNk.....",
            "...knnNzNnnk....",
            "..knNNyeyNNnk...",
            ".kNNnNNNNNnNNk..",
            "kNNnNNyNNNNnNNk.",
            "kyyyyyyyyyyyyyk.",
        ],
        'yeux': [(6, 6), (8, 6)],
        'ferme': 'E',
        'bouche': (7, 7),
        'bouche_lettre': 'e',
        'epaule': (10, 9),
        # l'orbe entre ses mains, qui luit, et les étincelles qui montent
        'ouvrage': [
            [(6, 10, ["kek"]), (6, 9, [".f."])],
            [(6, 10, ["kfk"]), (6, 9, [".e."]), (5, 7, ["s"]), (10, 5, ["s"])],
            [(6, 10, ["kek"]), (6, 9, [".f."]), (9, 4, ["s"]), (4, 6, ["s"])],
        ],
    },
    'claudeterminal': {
        'profil_couleur': '#d97757',
        'description': "le petit ordinateur de ClaudeTerminal : un écran cathodique qui marche, deux yeux orange, le clavier",
        'palette': {'W': '#ece4cc', 'V': '#b8ab88', 'z': '#1a2440', 'F': '#ff9a5a', 'a': '#4a4e5c',
                    'B': '#2b5fd8', 'C': '#ece4cc', 'p': '#4a4e5c'},
        'debout': [
            "................",
            "................",
            "...kkkkkkkk.....",
            "...kWWWWWWk.....",
            "...kWzzzzVk.....",
            "...kWFzFzVk.....",
            "...kWFzFzVk.....",
            "...kWzzzzVk.....",
            "...kWWWWWVk.....",
            "...kWWkkWVk.....",
            "...kVVVVVVk.....",
            "...kkkkkkkk.....",
            ".....a..a.......",
            ".....a..a.......",
            "....kak.kak.....",
            "....kkk.kkk.....",
        ],
        'assis': [
            "................",
            "................",
            "................",
            "................",
            "................",
            "...kkkkkkkk.....",
            "...kWWWWWWk.....",
            "...kWzzzzVk.....",
            "...kWFzFzVk.....",
            "...kWFzFzVk.....",
            "...kWzzzzVk.....",
            "...kWWWWWVk.....",
            "...kWWkkWVk.....",
            "...kVVVVVVk.....",
            "...kkkkkkkkkak..",
            ".........kkkkk..",
        ],
        'yeux': [(5, 5), (7, 5)],
        'ferme': 'z',
        'bouche': (5, 7),
        'bouche_lettre': 'FFF',
        'epaule': (9, 8),
        'triste': [(5, 4, ["BBBB", "BwBw", "BBBB", "wwwB"])],
        'ouvrage': [
            [(10, 8, ["a"]), (11, 9, ["a"]), (10, 11, ["kkkkkk"]), (10, 12, ["dgsgdk"]), (9, 13, ["kWWWWWk"]), (9, 14, ["kVVVVVk"]), (9, 15, ["kkkkkkk"])],
            [(10, 9, ["a"]), (11, 10, ["a"]), (10, 11, ["kkkkkk"]), (10, 12, ["dggsdk"]), (9, 13, ["kWWWWWk"]), (9, 14, ["kVVVVVk"]), (9, 15, ["kkkkkkk"])],
        ],
        'bras_attente': 'a',
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


def sub(lignes, table):
    return [''.join(table.get(c, c) for c in l) for l in lignes]


def corps_commun(h, rangees, haut, dy_tete, sans_bras=False):
    """Le corps commun aux couleurs de l'habitant, sa tête au-dessus (descendue de dy_tete)."""
    corps = list(rangees)
    if sans_bras:
        for x, y in BRAS:
            l = corps[y - 8]
            corps[y - 8] = l[:x] + 'C' + l[x + 1:]
    img = toile(sub(corps, h.get('corps', {})), haut)
    for x, y, m in h.get('dos', []):
        poser(img, x, y + dy_tete, m)
    poser(img, 0, 2 + dy_tete, h['tete'])
    for x, y, m in h.get('plus', []):
        poser(img, x, y + dy_tete, m)
    return img


def habitant(nom, h):
    entier = 'debout' in h
    ex, ey = h.get('epaule', EPAULE)
    tc = h.get('corps', {})
    ferme = h.get('ferme', 'q')
    # de combien la tête descend quand il s'assoit
    dy = {'galadriel': 3, 'claudeterminal': 3}.get(nom, 3)

    def debout(sans_bras=False):
        return list(h['debout']) if entier else corps_commun(h, DEBOUT, 8, 0, sans_bras)

    def assis():
        return list(h['assis']) if entier else corps_commun(h, ASSIS, 11, 3)

    def yeux(img, lettre):
        for x, y in h['yeux']:
            poser(img, x, y + dy, [lettre])

    def outil(objets):
        # le bras prend les couleurs du corps ; l'outil garde les siennes (sauf lettres_outil)
        if entier:
            return objets
        return [(x, y, sub(m, (tc if m in (["C"], ["p"]) else {}) | h.get('lettres_outil', {}))) for x, y, m in objets]

    etats = {}
    images = []
    for objets in h['ouvrage']:
        img = debout(sans_bras=True)
        for x, y, m in outil(objets):
            poser(img, x, y, m)
        images.append(img)
    etats['travail'] = {'every': 2, 'images': images}

    images = []
    for zs in ZZZ:
        img = assis()
        yeux(img, ferme)
        for x, y, m in zs:
            poser(img, x, y, m)
        images.append(img)
    etats['repos'] = {'every': 4, 'images': images}

    images = []
    for f in range(2):
        img = debout(sans_bras=True)
        for x, y, m in drapeau(ex, ey, f):
            if m == ["C"]:
                m = [h.get('bras_attente', 'N')] if entier else [tc.get('C', 'C')]
            elif m == ["p"]:
                m = [h.get('bras_attente', 'e')] if entier else [tc.get('p', 'p')]
            poser(img, x, y, m)
        images.append(img)
    etats['attente'] = {'every': 2, 'images': images}

    images = []
    for fumee in FUMEE:
        img = assis()
        for x, y, m in fumee:
            poser(img, x, y, m)
        for x, y, m in h.get('triste', []):
            poser(img, x, y + dy, m)
        if not h.get('triste'):
            bx, by = h['bouche']
            poser(img, bx, by + dy, [h.get('bouche_lettre', 'M')[0]])
        images.append(img)
    etats['echec'] = {'every': 3, 'images': images}

    images = []
    for ouverte in (False, True):
        img = debout()
        if ouverte:
            bx, by = h['bouche']
            poser(img, bx, by, [h.get('bouche_lettre', 'M')])
        images.append(img)
    etats['parle'] = {'every': 2, 'images': images}

    for e in etats.values():
        for img in e['images']:
            assert len(img) == 16 and all(len(l) == 16 for l in img), nom
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
        # un dessin validé par le user le reste : le gabarit ne le repasse pas à false en silence
        h = habitant(nom, HABITANTS[nom])
        if ancien and ancien.get('valide'):
            # un validé ne se remplace pas : sa nouvelle version va à la revue, à côté
            chemin = os.path.join(RACINE, 'revue', f'{nom}-nouveau.json')
            os.makedirs(os.path.dirname(chemin), exist_ok=True)
        open(chemin, 'w', encoding='utf-8', newline='\n').write(ranger(h))
        print(nom)
