"""Les accessoires des onglets (habitants/accessoires.json, contrat : habitants/LISEZMOI.md).

    python tools/habitants/accessoires.py

Les petites formes à la main, les rondes (soleil, nuages, arc-en-ciel) calculées ; chaque
accessoire a ses niveaux, chacun ses images. Écrit le fichier avec valide: false, sauf s'il est
déjà validé.
"""
import json
import math
import os
import sys

RACINE = os.path.join(os.path.dirname(__file__), '..', '..', 'habitants')

PALETTE = {
    'k': '#1b1a24', 'w': '#f4f1ea', 'g': '#b4b7bf', 'd': '#7c808c', 'a': '#4a4e5c',
    'O': '#3f80dc', 'o': '#1f4f9e',
    'y': '#e3a63c', 'Y': '#fff3b0', 's': '#ffd75e', 'F': '#ff5a1f', 'f': '#ffb02e',
    'W': '#f2ead6', 'V': '#c8bc9e', 'r': '#7a4a24', 'R': '#4a2c16', 'p': '#ecb48a', 'm': '#141018',
    'c': '#7fd4ff', 'C': '#eef0f6', 'D': '#a8aebc', 'n': '#5a6070', 'N': '#3a3f4c',
    'E': '#e05050', 'H': '#ff8c2a', 'G': '#74e07a', 'B': '#3f80dc', 'P': '#b07cff',
    'u': '#dcdcea', 'x': '#8d8896', 'j': '#5aa08a', 'q': '#e08a6a',
}


def toile(w, h):
    return [['.'] * w for _ in range(h)]


def texte(t):
    return [''.join(l) for l in t]


def poser(t, x, y, motif):
    for dy, ligne in enumerate(motif):
        for dx, c in enumerate(ligne):
            if c != '.' and 0 <= y + dy < len(t) and 0 <= x + dx < len(t[0]):
                t[y + dy][x + dx] = c


def nuage(t, x0, y0, clair, ombre):
    """Un nuage de 12 × 5 : deux bosses, le dessous dans l'ombre, cerné."""
    poser(t, x0, y0, [
        "....kkk.....",
        "..kk" + clair * 3 + "kkk..",
        ".k" + clair * 8 + "k.",
        "k" + clair * 10 + "k",
        ".k" + ombre * 8 + "k.",
    ])


def pigeon():
    images = [
        ["...kk...",
         "..kgdk..",
         ".kkgdkkk",
         "kgggggjq",
         ".kdggggk",
         "...kwk.."],
        ["........",
         "......kk",
         "kddddkgq",
         "kgggggjk",
         ".kdggggk",
         "...kwk.."],
        ["........",
         "......kk",
         ".kkgggjq",
         "kggdgggk",
         ".kdddkk.",
         "..kdwk.."],
    ]
    return {'taille': [8, 6], 'every': 1, 'usage': "vole d'un onglet à l'autre quand une session écrit à une autre, la lettre dans les pattes",
            'niveaux': [images + [images[1]]]}


def toiles():
    poussiere = [(3, 4), (10, 3), (6, 9), (12, 12), (2, 13), (8, 6), (13, 7)]
    coin_g = ["uuuuuu", "uu..u.", "u.uu..", "u.u...", "uu....", "u....."]
    coin_d = ["".join(reversed(l)) for l in coin_g]
    niveaux = []
    for n in range(3):
        t = toile(16, 16)
        for x, y in poussiere[:4 + n]:
            t[y][x] = 'x'
        if n >= 1:
            poser(t, 0, 0, coin_g)
        if n >= 2:
            poser(t, 10, 0, coin_d)
            for y in range(0, 5):
                t[y][12] = 'u'
            poser(t, 11, 5, ["k.k", ".k.", "k.k"])
        niveaux.append([texte(t)])
    return {'taille': [16, 16], 'every': 4, 'usage': "posé sur l'habitant d'une session au repos depuis longtemps : 0 poussière, 1 une toile, 2 toiles et araignée",
            'niveaux': niveaux}


def bougie():
    flammes = [[".f.", "fYf", ".F."], ["..f", ".Yf", ".FF"], [".f.", "fYf", "FYF"]]
    niveaux = []
    for h in (1, 3, 5, 7, 9):
        images = []
        for fl in flammes:
            t = toile(7, 16)
            poser(t, 0, 14, ["kyyyyyk", ".kkkkk."])
            haut = 14 - h
            for y in range(haut, 14):
                t[y] = list(".kWWVk.")
            t[haut] = list(".kkkkk.")
            if h >= 5:
                t[haut + 2][1] = 'W'  # une coulure
            t[haut - 1][3] = 'k'
            poser(t, 2, haut - 4, fl)
            images.append(texte(t))
        niveaux.append(images)
    return {'taille': [7, 16], 'every': 2, 'usage': "le contexte : niveau 0 presque fondue, 4 neuve (après un compactage) ; la flamme vacille",
            'niveaux': niveaux}


def piece():
    images = [[".kk.", "kYyk", "kyyk", ".kk."], [".kk.", ".kYk", ".kyk", ".kk."],
              ["..k.", "..k.", "..k.", "..k."], [".kk.", "kYk.", "kyk.", ".kk."]]
    return {'taille': [4, 4], 'every': 1, 'usage': "une pièce qui tourne en tombant vers la bourse, au fil des jetons dépensés",
            'niveaux': [images]}


def bourse():
    plate = ["..........", "..........", "..........", "..........", "....kk....",
             "...krrk...", "...kyyk...", "..krrrrk..", ".krrrrRRk.", ".kkkkkkkk."]
    ronde = ["..........", "..........", "....kk....", "...krrk...", "...kyyk...",
             "..krrrrk..", ".krrrrrRk.", "krrrrrrRRk", "krrrrrRRRk", ".kkkkkkkk."]
    pleine = ["..k..kk...", ".kYkkYyk..", "..kyk.kk..", "...kyyk...", "..krrrrk..",
              ".krrrrrRk.", "krrrrrrRRk", "krrrrrrRRk", "krrrrRRRRk", ".kkkkkkkk."]
    return {'taille': [10, 10], 'every': 4, 'usage': "le coût de la session : 0 plate, 1 ronde, 2 pleine",
            'niveaux': [[plate], [ronde], [pleine]]}


def soleil():
    images = []
    for f in range(2):
        t = toile(16, 8)
        for y in range(8):
            for x in range(16):
                d = math.hypot(x - 7.5, y - 3.5)
                if d <= 2.2:
                    t[y][x] = 'Y' if (x, y) in ((7, 3), (6, 3), (7, 2)) else 's'
                elif d <= 2.9:
                    t[y][x] = 'f'
        rayons = [(0, -1), (1, 0), (0, 1), (-1, 0)] if f == 0 else [(1, -1), (1, 1), (-1, 1), (-1, -1)]
        for dx, dy in rayons:
            for r in (3.9, 4.6):
                x, y = round(7.5 + dx * r * (0.75 if dx and dy else 1)), round(3.5 + dy * r * (0.75 if dx and dy else 1) * 0.9)
                if 0 <= y < 8 and 0 <= x < 16:
                    t[y][x] = 's'
        images.append(texte(t))
    return {'taille': [16, 8], 'every': 4, 'usage': "la météo : tout va bien", 'niveaux': [images]}


def averse():
    images = []
    for f in range(3):
        t = toile(16, 8)
        nuage(t, 2, 0, 'C', 'D')
        for i, x in enumerate((4, 7, 10, 12, 5, 9)):
            y = 5 + (f + i) % 3
            if y < 8:
                t[y][int(x)] = 'c'
        images.append(texte(t))
    return {'taille': [16, 8], 'every': 1, 'usage': "la météo : une compilation ratée", 'niveaux': [images]}


def orage():
    eclair = [(8, 5), (7, 6), (8, 6), (7, 7)]
    images = []
    for f in range(4):
        t = toile(16, 8)
        nuage(t, 2, 0, 'D' if f == 2 else 'n', 'N')
        if f in (1, 2):
            for x, y in eclair:
                t[y][x] = 'Y'
        images.append(texte(t))
    return {'taille': [16, 8], 'every': 2, 'usage': "la météo : un plantage", 'niveaux': [images]}


def arc():
    bandes = 'EHsGBP'
    images = []
    for f in range(2):
        t = toile(16, 8)
        for y in range(8):
            for x in range(16):
                d = math.hypot((x - 7.5) * 0.92, (y - 8.2) * 1.0)
                i = int(7.6 - d)
                if 0 <= i < len(bandes) and y < 8:
                    t[y][x] = bandes[i]
        if f == 1:
            t[1][4] = 'w'
            t[0][11] = 'w'
        images.append(texte(t))
    return {'taille': [16, 8], 'every': 3, 'usage': "la météo : un test qui passe", 'niveaux': [images]}


def mineur():
    corps = ["..kRkRk.", ".kk..kk."], ["...kRk..", "..kkkk.."]
    images = []
    for jambes in corps:
        images.append(["g...kkk.",
                       "gr.kyyyY",
                       ".rkppmk.",
                       "..rkOOk.",
                       "..kpOOpk",
                       "..kOOOk."] + list(jambes))
    return {'taille': [8, 8], 'every': 2, 'usage': "un diagnostic de fond : part sous la barre d'onglets, tourné vers la droite (miroir pour la gauche)",
            'niveaux': [images]}


def mineur_lingot():
    corps = ["..kRkRk.", ".kk..kk."], ["...kRk..", "..kkkk.."]
    images = []
    for jambes in corps:
        images.append(["..kkkk..",
                       ".kYyyyk.",
                       ".pkyyyYp",
                       "..kppmk.",
                       "..kpOOpk",
                       "..kOOOk."] + list(jambes))
    return {'taille': [8, 8], 'every': 2, 'usage': "le même, qui revient avec un lingot quand son rapport est versé",
            'niveaux': [images]}


def bulle():
    images = []
    for n in range(1, 4):
        points = ''.join('m' if i < n else 'w' for i in range(3))
        images.append(["..kkkkkkkk..",
                       ".kwwwwwwwwk.",
                       "kwwwwwwwwwwk",
                       f"kww{points[0]}ww{points[1]}ww{points[2]}wk",
                       "kwwwwwwwwwwk",
                       ".kwwwwwwwwk.",
                       "..kwkkkkkk..",
                       "..kk........"])
    return {'taille': [12, 8], 'every': 3, 'usage': "la ligne « Dit : » prononcée : sort de l'onglet, trois points qui s'écrivent",
            'niveaux': [images]}


ACCESSOIRES = {
    'pigeon': pigeon, 'toiles': toiles, 'bougie': bougie, 'piece': piece, 'bourse': bourse,
    'meteo-soleil': soleil, 'meteo-averse': averse, 'meteo-orage': orage, 'meteo-arc': arc,
    'mineur': mineur, 'mineur-lingot': mineur_lingot, 'bulle': bulle,
}


def ecrire():
    acc = {nom: f() for nom, f in ACCESSOIRES.items()}
    for nom, a in acc.items():
        w, h = a['taille']
        for niveau in a['niveaux']:
            for img in niveau:
                assert len(img) == h and all(len(l) == w for l in img), nom
    utilisees = {c for a in acc.values() for n in a['niveaux'] for img in n for l in img for c in l} - {'.'}
    assert utilisees <= set(PALETTE), sorted(utilisees - set(PALETTE))
    return {'nom': 'accessoires', 'valide': False,
            'palette': {k: v for k, v in PALETTE.items() if k in utilisees or k in 'Oo'}, 'accessoires': acc}


def ranger(d):
    out = ['{', f'  "nom": "accessoires",', f'  "valide": {json.dumps(d["valide"])},', '  "palette": {']
    pal = list(d['palette'].items())
    out += [f'    {json.dumps(k)}: {json.dumps(v)}' + (',' if i < len(pal) - 1 else '') for i, (k, v) in enumerate(pal)]
    out += ['  },', '  "accessoires": {']
    accs = list(d['accessoires'].items())
    for i, (nom, a) in enumerate(accs):
        out.append(f'    {json.dumps(nom)}: {{')
        out.append(f'      "taille": {json.dumps(a["taille"])},')
        out.append(f'      "every": {a["every"]},')
        out.append(f'      "usage": {json.dumps(a["usage"], ensure_ascii=False)},')
        out.append('      "niveaux": [')
        for j, niv in enumerate(a['niveaux']):
            out.append('        [')
            for k, img in enumerate(niv):
                out.append('          [')
                out += [f'            {json.dumps(l)}' + (',' if m < len(img) - 1 else '') for m, l in enumerate(img)]
                out.append('          ]' + (',' if k < len(niv) - 1 else ''))
            out.append('        ]' + (',' if j < len(a['niveaux']) - 1 else ''))
        out.append('      ]')
        out.append('    }' + (',' if i < len(accs) - 1 else ''))
    out += ['  }', '}']
    return '\n'.join(out) + '\n'


if __name__ == '__main__':
    chemin = os.path.join(RACINE, 'accessoires.json')
    if os.path.exists(chemin) and json.load(open(chemin, encoding='utf-8')).get('valide'):
        sys.exit('accessoires.json est validé : le refaire demande de remettre valide à false à la main')
    open(chemin, 'w', encoding='utf-8', newline='\n').write(ranger(ecrire()))
    print('ok')
