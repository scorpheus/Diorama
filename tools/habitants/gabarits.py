"""Les habitants des onglets, dessinés un par un, de face, à la manière des créatures de Dwarf
Fortress : trapus, deux masses franches (coiffe, corps), l'outil qui sort de la silhouette,
contour sombre, peu de couleurs. Pas forcément des humains : un nain, un escargot, un golem,
une voiture. Écrit habitants/<nom>.json (valide: false) ; la nouvelle version d'un habitant
validé va à habitants/revue/<nom>-nouveau.json, à côté.

    python tools/habitants/gabarits.py            # tous
    python tools/habitants/gabarits.py forgeron   # un seul

Chaque habitant donne son dessin debout (16 lignes, ou 8 colonnes de gauche que `miroir`
complète pour un dessin symétrique), puis ce qui change d'un état à l'autre : les yeux fermés,
la bouche ouverte, la mine triste, la main qui lève le drapeau, et son ouvrage image par image.
Un nouvel habitant (une demande de habitants/demandes/) : une entrée de plus dans HABITANTS.
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from ranger import ranger  # noqa: E402

RACINE = os.path.join(os.path.dirname(__file__), '..', '..', 'habitants')
VIDE = '.' * 16


def miroir(moities):
    """Un dessin symétrique : chaque ligne donne ses 8 colonnes de gauche."""
    return [m + m[::-1] for m in moities]


def bas(lignes):
    """Posé sur la dernière ligne : complété de vide par le haut jusqu'à 16."""
    return [VIDE] * (16 - len(lignes)) + list(lignes)


Z = ["uuuu", "...u", "..u.", ".u..", "uuuu"]
ZZZ = [[], [(12, 3, ["u"])], [(12, 0, Z)], [(12, 0, Z), (10, 3, ["u"])]]
FUMEE = [
    [(6, 1, [".x.", "xXx"]), (1, 9, ["x"]), (14, 8, ["X"])],
    [(6, 0, ["..X", ".xXx", "xx.."]), (0, 8, ["X"]), (14, 7, ["x"])],
    [(7, 0, [".XX", "xXx"]), (1, 7, ["x"]), (15, 9, ["x"]), (10, 2, ["x"])],
]


def drapeau(main, f, lettre_bras=None):
    """La hampe tenue par la main (x, y), le drapeau qui flotte à son sommet ; image f."""
    x, y = main
    haut = max(0, y - 9)
    hampe = [(x, haut + i, ["r"]) for i in range(y - haut)]
    tissu = [["OOOw", "OOww", "Oww."], [".OOw", "OOOw", "OOww"]][f]
    return hampe + [(x + 1, haut, tissu)] + ([(x, y, [lettre_bras])] if lettre_bras else [])


BASE = {
    'k': '#1b1a24', 'p': '#ecb48a', 'q': '#b97a55', 'w': '#f4f1ea', 'm': '#141018', 'M': '#7a2a2a',
    'u': '#bff3ff', 'x': '#5e5966', 'X': '#8d8896', 'O': '#3f80dc', 'o': '#1f4f9e', 'r': '#7a4a24',
    'R': '#4a2c16', 'g': '#b4b7bf', 'd': '#7c808c', 's': '#ffd75e', 'c': '#7fd4ff',
}

HABITANTS = {
    # le marin de face : suroît et ciré jaunes, grosse barbe blanche, l'écharpe à la couleur du profil
    'marin': {
        'description': "le marin en ciré jaune et suroît, grosse barbe blanche, l'écharpe à la couleur du profil (Diorama, Route du Rhum)",
        'palette': {'J': '#f8da1c', 'j': '#c9a40e', 'y': '#8a6a10', 'w': '#f4f1ea', 'l': '#c9c4ba',
                    'n': '#1e2a55', 'a': '#2e343a', 'r': '#7a4a24', 'R': '#4a2c16', 'g': '#b4b7bf',
                    'd': '#7c808c', 's': '#fff3b0'},
        'debout': miroir([
            "........",
            "........",
            "......kk",
            ".....kJJ",
            "....kJJJ",
            "..kkjjjj",
            "..kjkppm",
            "...kpppq",
            "...klwww",
            "..kOOOww",
            ".kJJJOlw",
            ".kpJJJJJ",
            "..kJJyJJ",
            "..kjjjjj",
            "...knnk.",
            "..kaaak.",
        ]),
        'yeux': [(7, 6), (8, 6)],
        'ferme': 'q',
        'bouche': (7, 9, ["MM"]),
        'triste': [(6, 6, [".mm."]), (11, 5, ["c"])],
        'main': (13, 11),
        'ouvrage': [
            [(13, 10, ["p"]), (12, 11, ["kkkk"]), (12, 12, ["dgsk"]), (11, 13, ["krrrk"]), (11, 14, ["kRRRk"]), (11, 15, ["kkkkk"])],
            [(13, 11, ["p"]), (12, 11, ["kpkk"]), (12, 12, ["dsgk"]), (11, 13, ["krrrk"]), (11, 14, ["kRRRk"]), (11, 15, ["kkkkk"])],
        ],
    },
    # le nain forgeron : casque de fer, gros nez, barbe rousse jusqu'aux genoux, tablier, marteau
    'forgeron': {
        'profil_couleur': '#d04a2a',
        'description': "le nain forgeron des tickets d'Endurance : casque de fer, barbe rousse, tablier de cuir, marteau et enclume",
        'palette': {'A': '#9aa0b0', 'G': '#d8dce6', 'h': '#c0602a', 'H': '#e08a4a', 'r': '#6a4024',
                    'n': '#3a3440', 'a': '#2e2420'},
        'debout': miroir([
            "........",
            "........",
            ".....kkk",
            "....kAAA",
            "...kAAAA",
            "...kkkkk",
            "...kppmp",
            "...khppq",
            "..khhhhh",
            "..khhHhh",
            ".kphhhhh",
            "..kOhhhh",
            "..kOOhhh",
            "..kOOOOh",
            "...knnk.",
            "..kaaak.",
        ]),
        'plus': [(5, 3, ["G"]), (13, 9, ["r"]), (13, 8, ["r"]), (12, 6, ["kAAk"]), (12, 7, ["kAAk"])],
        'yeux': [(6, 6), (9, 6)],
        'ferme': 'q',
        'bouche': (7, 9, ["MM"]),
        'triste': [(6, 6, ["m..m"]), (11, 5, ["c"])],
        'main': (13, 10),
        'ouvrage': [
            [(13, 8, ["p"]), (13, 9, ["h"]), (14, 7, ["r"]), (13, 5, ["kAk", "kAk"]), (12, 13, ["kAAk", ".kk.", "kkkk"])],
            [(13, 11, ["p"]), (14, 11, ["r"]), (14, 12, ["A"]), (12, 13, ["kAAk", ".kk.", "kkkk"]), (11, 11, ["s"]), (15, 10, ["s"])],
            [(13, 11, ["p"]), (14, 11, ["r"]), (14, 12, ["A"]), (12, 13, ["kAAk", ".kk.", "kkkk"]), (12, 10, ["s"]), (15, 12, ["s"])],
        ],
    },
    # Galadriel comme son pet du Diorama : capuche bleu nuit, diadème d'or, visage dans l'ombre,
    # yeux cyan, pendentif ; la cape s'évase en vagues au milieu et retombe (pas une fusée)
    'galadriel': {
        'profil_couleur': '#9a6ad0',
        'description': "Galadriel, l'orchestratrice, comme son pet du Diorama : capuche et cape bleu nuit étoilées, diadème d'or, visage dans l'ombre, yeux cyan",
        'palette': {'N': '#2a3478', 'n': '#161c46', 'y': '#e8c46a', 'z': '#0c0e1e', 'e': '#7ff0ff',
                    'E': '#2a8aa0', 'f': '#d0fbff'},
        'debout': miroir([
            "........",
            "........",
            "......kk",
            "....kkNN",
            "...kNNNy",
            "...kNzzz",
            "...kNzez",
            "...kNnzz",
            "..kNnNNe",
            "..kNnNNN",
            "..kNnNNN",
            "..kNnNNN",
            ".kNNnNNN",
            ".kNnNNNN",
            "kNNnNNNN",
            "kkykkkyk",
        ]),
        # un croissant d'or d'un seul côté de la cape, comme sur son pet
        'plus': [(10, 11, ["y", ".y"]), (11, 12, ["y"])],
        'yeux': [(6, 6), (9, 6)],
        'ferme': 'E',
        'bouche': (7, 7, ["ee"]),
        'triste': [(6, 6, ["E..E"])],
        'main': (12, 9),
        'lettre_bras': 'N',
        'ouvrage': [
            [(7, 10, ["fe"]), (6, 11, ["kEEk"])],
            [(7, 10, ["ef"]), (6, 11, ["kEEk"]), (5, 8, ["s"]), (11, 5, ["s"])],
            [(7, 10, ["fe"]), (6, 11, ["kEEk"]), (10, 3, ["s"]), (3, 7, ["s"])],
        ],
    },
    # le voyageur : un escargot, sa maison sur le dos à la couleur du profil, un baluchon
    'voyageur': {
        'description': "le voyageur : un escargot qui porte sa maison, la coquille à la couleur du profil, un baluchon",
        'palette': {'b': '#b8c48a', 'B': '#7e8a58', 'W': '#efe2c0', 'r': '#7a4a24'},
        'debout': bas([
            "...........m.m..",
            "...........b.b..",
            "....kkkk...b.b..",
            "...kOOOOk.kbbbk.",
            "..kOoooOOkkbbbbk",
            "..kOoOOoOkkbbbbk",
            "..kOoOoOOkkbbbMk",
            "..kOOooOOkbbbbk.",
            "...kOOOOkbbbbbk.",
            ".kkkkkkkkbbbbk..",
            "kbbbbbbbbbbbk...",
            ".kBBBBBBBBBk....",
            "..kkkkkkkkk.....",
        ]),
        'plus': [(6, 1, ["kWWk", "kWWWk"]), (6, 3, ["r"]), (5, 4, ["r"])],
        'yeux': [(11, 3), (13, 3)],
        'ferme': '.',
        'bouche': (14, 9, ["M"]),
        'triste': [(11, 3, ["."]), (13, 3, ["."]), (11, 4, ["m.m"]), (11, 5, ["b.b"])],
        'main': (7, 7),
        'ouvrage': [
            [(0, 12, ["c"]), (14, 9, ["k"])],
            [(1, 13, ["c"]), (0, 12, ["."])],
        ],
        'decale': [0, 1],
    },
    # le tailleur de LaPierreDesMots : un golem de pierre, les yeux et la rune à la couleur du profil
    'tailleur': {
        'profil_couleur': '#3fb07a',
        'description': "le golem de pierre de LaPierreDesMots : bloc taillé, yeux et rune qui luisent à la couleur du profil, une tablette où il grave",
        'palette': {'Q': '#9a94a2', 'V': '#6a6474', 'W': '#c8c2d0', 'L': '#4a4654'},
        'debout': bas([
            "...kkkkkkk......",
            "..kWQQQQQVk.....",
            "..kQOQQQOVk.....",
            "..kQQkkkQVk.....",
            "..kkkkkkkkk.....",
            ".kWQkWQQQkQVk...",
            ".kQVkQOOQkQVk...",
            ".kQVkQQOQkQVk...",
            ".kQVkQOQQkQVk...",
            ".kkkkQQQQkkkk...",
            ".kWQkVVVVkQVk...",
            ".kkkkQVkQVkkk...",
            "...kQVk.kQVk....",
            "..kLLLk.kLLLk...",
        ]),
        'yeux': [(4, 4), (8, 4)],
        'ferme': 'V',
        'bouche': (5, 5, ["kOk"]),
        'triste': [(4, 4, ["V...V"]), (5, 5, ["k.k"])],
        'main': (12, 10),
        'ouvrage': [
            [(12, 9, ["Q"]), (13, 8, ["V"]), (12, 11, ["kkkk", "kWQk", "kOQk", "kQOk", "kkkk"])],
            [(12, 11, ["kkkk", "kWQk", "kOQk", "kQOk", "kkkk"]), (12, 10, ["QQ"]), (14, 9, ["W"]), (15, 10, ["W"])],
        ],
    },
    # Europa : une voiture de rallye de face, phares pour yeux, rampe de phares sur le toit
    'europa': {
        'profil_couleur': '#d04a2a',
        'description': "la voiture de rallye d'Europa : de face, carrosserie à la couleur du profil, phares pour yeux, rampe de phares sur le toit",
        'palette': {'E': '#9fe0ff', 'e': '#5aa6d0', 'Y': '#fff3b0', 'a': '#2e343a', 'A': '#8a90a0'},
        'debout': bas(miroir([
            "....kYkY",
            "...kkkkk",
            "...kOOOO",
            "..kEEEEE",
            "..kEwEEE",
            ".kOOOOOO",
            "kOwwOOOO",
            "kOwmOkkk",
            "kOOOOkwk",
            "kkkkkkkk",
            ".kak....",
            ".kkk....",
        ])),
        'yeux': [(3, 11), (12, 11)],
        'ferme': 'o',
        'yeux_bis': [(2, 10), (3, 10), (12, 10), (13, 10)],
        'bouche': (6, 12, ["kMMMMk"]),
        'triste': [(2, 10, ["oo"]), (12, 10, ["oo"])],
        'main': (13, 6),
        'ouvrage': [
            [(0, 15, ["X"]), (15, 15, ["X"])],
            [(1, 14, ["x"]), (14, 14, ["x"])],
        ],
        'decale': [0, 1],
    },
    # le mécano de vehigraph, de face : casquette à la couleur du profil, moustache, bleu de travail
    'mecano': {
        'profil_couleur': '#e0902a',
        'description': "le mécano de vehigraph : casquette à la couleur du profil, moustache, bleu de travail, clé et roue",
        'palette': {'h': '#3a2a1e', 'C': '#2f5a9e', 'c': '#1e3a6e', 'a': '#2e343a', 'A': '#8a90a0'},
        'debout': miroir([
            "........",
            "........",
            ".....kkk",
            "....kOOO",
            "...kOOOO",
            "...koooo",
            "...kpmpp",
            "...kphhh",
            "....kppp",
            "...kCwCC",
            "..kCCCCC",
            ".kpkCCCC",
            "..kCCCCC",
            "...kCck.",
            "...kCCk.",
            "..kaaak.",
        ]),
        'plus': [(9, 10, ["w"])],
        'yeux': [(5, 6), (10, 6)],
        'ferme': 'q',
        'bouche': (7, 8, ["MM"]),
        'triste': [(5, 6, ["m....m"]), (11, 5, ["c"])],
        'main': (14, 11),
        'ouvrage': [
            [(14, 10, ["p"]), (13, 9, ["g"]), (12, 8, ["g"]), (11, 11, [".kkk.", "kaaak", "kaAak", "kaaak", ".kkk."])],
            [(14, 11, ["p"]), (13, 12, ["g"]), (12, 13, ["g"]), (11, 11, [".kkk.", "kaaak", "kaAak", "kaaak", ".kkk."])],
        ],
    },
    # ClaudeTerminal : une pieuvre qui jongle avec les sessions, le corps à la couleur du profil
    # (l'orange de Claude par défaut), des yeux noirs qui brillent ; elle tape de tous ses bras
    # et crache de l'encre quand ça casse
    'claudeterminal': {
        'profil_couleur': '#d97757',
        'description': "la pieuvre de ClaudeTerminal, qui jongle avec les sessions : corps à la couleur du profil, yeux noirs brillants, elle tape de tous ses bras",
        'palette': {'z': '#6a58a8', 'P': '#ffc2a8'},
        'debout': [
            "................",
            ".....kkkkkk.....",
            "...kkOOOOOOkk...",
            "..kOPPOOOOOOOk..",
            ".kOPOOOOOOOOOok.",
            ".kOOOOOOOOOOOok.",
            ".kOOmwOOOOmwOok.",
            ".kOOmmOOOOmmOok.",
            ".kOPOOOOOOOOPok.",
            "..kOOOOOOOOOok..",
            "..kOOOOOOOOOok..",
            ".kOOkOOkkOOkOok.",
            "kOOkkOOkkOOkkOok",
            "kOk.kOOkkOOk.kok",
            "kOOkkOk..kOkkOok",
            "kkkkOOk..kOokkkk",
        ],
        'yeux': [(4, 6), (5, 6), (10, 6), (11, 6)],
        'ferme': 'O',
        'bouche': (7, 9, ["MM"]),
        'triste': [(4, 6, ["OO....OO"]), (7, 9, ["kk"]),
                   (0, 10, ["zz", "z"]), (14, 9, ["zz", ".z"]), (1, 8, ["z"]), (13, 7, ["z"])],
        'main': (14, 10),
        'lettre_bras': 'O',
        'ouvrage': [
            [(1, 12, ["..O..O..O..O.."]), (1, 13, ["kkkkkkkkkkkkkk"]), (1, 14, ["kdgdgsgdgdgdgk"]), (1, 15, ["kkkkkkkkkkkkkk"])],
            [(1, 12, [".O..O..O..O..."]), (1, 13, ["kkkkkkkkkkkkkk"]), (1, 14, ["kdgsgdgdgsgdgk"]), (1, 15, ["kkkkkkkkkkkkkk"])],
        ],
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


def effacer(img, x, y):
    l = img[y]
    img[y] = l[:x] + '.' + l[x + 1:]


def decaler(img, dy):
    """Le dessin monté de dy lignes (un cahot), le bas complété de vide."""
    return img[dy:] + [VIDE] * dy if dy else img


def habitant(nom, h):
    def debout():
        img = list(h['debout'])
        for x, y, m in h.get('plus', []):
            poser(img, x, y, m)
        return img

    def fermer(img):
        for x, y in h['yeux']:
            if h['ferme'] == '.':
                effacer(img, x, y)
            else:
                poser(img, x, y, [h['ferme']])
        for x, y in h.get('yeux_bis', []):
            poser(img, x, y, [h['ferme']])

    etats = {}
    images = []
    for i, objets in enumerate(h['ouvrage']):
        img = debout()
        for x, y, m in objets:
            for dy, ligne in enumerate(m):
                for dx, c in enumerate(ligne):
                    if c == '.' and len(m) == 1 and len(ligne) == 1:
                        effacer(img, x + dx, y + dy)
            poser(img, x, y, m)
        images.append(decaler(img, h.get('decale', [0] * 9)[i]))
    etats['travail'] = {'every': 2, 'images': images}

    images = []
    for zs in ZZZ:
        img = debout()
        fermer(img)
        for x, y, m in zs:
            poser(img, x, y, m)
        images.append(img)
    etats['repos'] = {'every': 4, 'images': images}

    images = []
    for f in range(2):
        img = debout()
        for x, y, m in drapeau(h['main'], f, h.get('lettre_bras')):
            poser(img, x, y, m)
        images.append(img)
    etats['attente'] = {'every': 2, 'images': images}

    images = []
    for fumee in FUMEE:
        img = debout()
        for x, y, m in h.get('triste', []):
            for dy, ligne in enumerate(m):
                for dx, c in enumerate(ligne):
                    if c == '.' and len(m) == 1 and len(ligne) == 1:
                        effacer(img, x + dx, y + dy)
            poser(img, x, y, m)
        for x, y, m in fumee:
            poser(img, x, y, m)
        images.append(img)
    etats['echec'] = {'every': 3, 'images': images}

    images = []
    for ouverte in (False, True):
        img = debout()
        if ouverte:
            x, y, m = h['bouche']
            poser(img, x, y, m)
        images.append(img)
    etats['parle'] = {'every': 2, 'images': images}

    for e in etats.values():
        for img in e['images']:
            assert len(img) == 16 and all(len(l) == 16 for l in img), (nom, [len(l) for l in img])
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
        # un dessin validé par le user le reste : sa nouvelle version va à la revue, à côté
        if ancien and ancien.get('valide'):
            chemin = os.path.join(RACINE, 'revue', f'{nom}-nouveau.json')
            os.makedirs(os.path.dirname(chemin), exist_ok=True)
        open(chemin, 'w', encoding='utf-8', newline='\n').write(ranger(h))
        print(nom)
