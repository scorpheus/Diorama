"""Les gestes de Scorpheus à la forge, en pixel art, sur la case « waiting » n° 0 : le corps
gardé, le bras et le marteau effacés puis redessinés dans chaque position, aux proportions de
l'original (un gros poing serré, un avant-bras sombre qui sort de sous la cape).

python tools/pet/gestes.py                  → rendu/pet/forge.webp (une rangée par geste)
python tools/pet/gestes.py --voir [noms]    → gestes-x2.png ici, pour regarder
"""
import math
from PIL import Image, ImageDraw
import os
from outils import *

# la palette du pet, relevée sur la case
FOURRURE = {'o': '#1e1613', 'O': '#2e261e', 's': '#65564e', 'm': '#cebdb5', 'c': '#e8ddd3', 'C': '#fdf9ed'}
PATTE = {'o': '#090503', 'O': '#15100b', 's': '#201810', 'S': '#2c261e', 'm': '#3d2d21', 'M': '#4b3828', 'c': '#694a2d', 'C': '#8a6a48'}
FER = {'o': '#0c0607', 's': '#1a1714', 'S': '#251e1d', 'm': '#34302e', 'M': '#55443b', 'c': '#a19384', 'C': '#d8d0c4'}
BOIS = {'o': '#110b09', 's': '#361b0e', 'm': '#542f14', 'M': '#8a4b15', 'c': '#b66518'}
CEINTURE = {'o': '#0b0703', 's': '#322212', 'm': '#5a2707', 'M': '#794613', 'c': '#a49267'}
CORDE = {'o': '#0a0302', 's': '#4c270d', 'm': '#8c4914', 'M': '#b0712f', 'c': '#d8a76c'}
TUNIQUE = {'o': '#0b0703', 's': '#181a0c', 'm': '#404419', 'M': '#706729'}


def ell(d, x, y, rx, ry, c):
    d.ellipse([x - rx, y - ry, x + rx, y + ry], fill=hexa(c))


def cote_lumiere(dx, dy):
    """La normale d'un trait, tournée vers la lumière (en haut à gauche : le foyer)."""
    n = math.hypot(dx, dy) or 1
    nx, ny = -dy / n, dx / n
    return (nx, ny) if nx + ny <= 0 else (-nx, -ny)


def avant_bras(im, a, b, w):
    """L'avant-bras, fourrure sombre, de `a` à `b` : contour, fond, une bande d'ombre, un reflet."""
    d = ImageDraw.Draw(im)
    for c, ww in ((PATTE['o'], w + 2), (PATTE['M'], w)):
        d.line([a, b], fill=hexa(c), width=ww)
        ell(d, *b, ww / 2, ww / 2, c)
    nx, ny = cote_lumiere(b[0] - a[0], b[1] - a[1])
    k = w / 2 - 2
    d.line([(a[0] - nx * k, a[1] - ny * k), (b[0] - nx * k, b[1] - ny * k)], fill=hexa(PATTE['m']), width=3)
    d.line([(a[0] - nx * (k + 1), a[1] - ny * (k + 1)), (b[0] - nx * (k + 1), b[1] - ny * (k + 1))], fill=hexa(PATTE['S']), width=1)
    d.line([(a[0] + nx * (k - 1), a[1] + ny * (k - 1)), (b[0] + nx * (k - 1), b[1] + ny * (k - 1))], fill=hexa(PATTE['c']), width=2)


def epaule(im, x, y):
    """La touffe blanche de l'épaule, là où le bras sort de la cape."""
    d = ImageDraw.Draw(im)
    ell(d, x, y, 8, 7, FOURRURE['o'])
    ell(d, x, y, 7, 6, FOURRURE['m'])
    ell(d, x - 1, y - 1, 5, 4, FOURRURE['c'])
    ell(d, x - 2, y - 2, 2, 2, FOURRURE['C'])
    # des mèches qui dépassent en bas
    for i in (-5, -1, 3):
        point(im, x + i, y + 7, FOURRURE['o'])
        point(im, x + i, y + 6, FOURRURE['m'])


def poing(im, x, y, vers):
    """Le poing serré autour du manche (qui part vers `vers`, un angle) : une grosse boule,
    quatre doigts repliés en bourrelets, le pouce par-dessus, la lumière en haut à gauche."""
    d = ImageDraw.Draw(im)
    ell(d, x, y, 13, 11, PATTE['o'])
    ell(d, x, y, 12, 10, PATTE['M'])
    ell(d, x + 3, y + 3, 9, 7, PATTE['m'])
    ell(d, x + 5, y + 5, 5, 4, PATTE['S'])
    ell(d, x - 4, y - 4, 5, 4, PATTE['c'])
    ell(d, x - 5, y - 5, 2, 1.5, PATTE['C'])
    # les doigts : des sillons perpendiculaires au manche, du côté du manche
    ca, sa = math.cos(vers), math.sin(vers)
    for t in (-6, -2, 2, 6):
        p0 = (x + ca * 4 + (-sa) * t, y + sa * 4 + ca * t)
        p1 = (x + ca * 11 + (-sa) * t, y + sa * 11 + ca * t)
        d.line([p0, p1], fill=hexa(PATTE['s']), width=1)
    # le pouce, un bourrelet clair par-dessus
    tx, ty = x + ca * 6 + sa * 6, y + sa * 6 - ca * 6
    ell(d, tx, ty, 4, 3, PATTE['o'])
    ell(d, tx, ty, 3, 2, PATTE['c'])


def marteau(im, poing_xy, tete_xy):
    """Le manche du poing à la tête, et la tête en travers : 30 de long, 18 d'épais, des faces
    biseautées, une arête claire côté foyer, le coin de cuivre dans l'œil."""
    d = ImageDraw.Draw(im)
    (x, y), (cx, cy) = poing_xy, tete_xy
    dx, dy = cx - x, cy - y
    n = math.hypot(dx, dy)
    ca, sa = dx / n, dy / n
    # le manche dépasse un peu sous le poing
    q = (x - ca * 14, y - sa * 14)
    d.line([q, (cx, cy)], fill=hexa(BOIS['o']), width=7)
    d.line([q, (cx, cy)], fill=hexa(BOIS['M']), width=5)
    nx, ny = cote_lumiere(dx, dy)
    d.line([(q[0] + nx * 1.5, q[1] + ny * 1.5), (cx + nx * 1.5, cy + ny * 1.5)], fill=hexa(BOIS['c']), width=1)
    d.line([(q[0] - nx * 2, q[1] - ny * 2), (cx - nx * 2, cy - ny * 2)], fill=hexa(BOIS['m']), width=1)
    L, E = 15, 9
    u = (-sa, ca)  # le long de la tête
    v = (ca, sa)   # son épaisseur, le long du manche
    P = lambda a, b: (cx + u[0] * a + v[0] * b, cy + u[1] * a + v[1] * b)
    d.polygon([P(-L - 1, -E), P(-L, -E - 1), P(L, -E - 1), P(L + 1, -E), P(L + 1, E), P(L, E + 1), P(-L, E + 1), P(-L - 1, E)], fill=hexa(FER['o']))
    d.polygon([P(-L, -E), P(L, -E), P(L, E), P(-L, E)], fill=hexa(FER['m']))
    # le corps de la tête, un peu plus étroit que ses faces
    d.polygon([P(-L + 4, -E), P(L - 4, -E), P(L - 4, E), P(-L + 4, E)], fill=hexa(FER['S']))
    d.polygon([P(-L + 4, -E + 2), P(L - 4, -E + 2), P(L - 4, E - 2), P(-L + 4, E - 2)], fill=hexa(FER['m']))
    # les faces de frappe : un biseau clair
    for s in (-1, 1):
        d.line([P(s * L, -E + 1), P(s * L, E - 1)], fill=hexa(FER['c']), width=1)
        d.line([P(s * (L - 1), -E + 1), P(s * (L - 1), E - 1)], fill=hexa(FER['M']), width=1)
    # l'arête vers la lumière, claire ; l'autre, sombre
    haut = min((-E + 1, E - 1), key=lambda b: (P(0, b)[0] + P(0, b)[1]))
    bas = -haut
    d.line([P(-L + 1, haut), P(L - 1, haut)], fill=hexa(FER['c']), width=2)
    d.line([P(-L + 2, haut), P(-L + 6, haut)], fill=hexa(FER['C']), width=1)
    d.line([P(-L + 1, bas), P(L - 1, bas)], fill=hexa(FER['s']), width=2)
    # l'œil : le bout du manche et son coin de cuivre
    ex, ey = P(0, -E + 3)
    ell(d, ex, ey, 2.5, 2.5, BOIS['o'])
    ell(d, ex, ey, 1.5, 1.5, BOIS['c'])


def reparer(im):
    """Ce que le bras cachait : le bord de la tunique, le bout de la ceinture, le haut de la corde."""
    # le bord de la tunique, de l'épaule à la ceinture (le blanc de l'ancienne manche part avec)
    remplir(im, [(58, 96), (62, 96), (62, 124), (56, 124), (56, 104)], TUNIQUE['m'])
    ligne(im, [(57, 97), (56, 104), (55, 123)], TUNIQUE['o'])
    ligne(im, [(58, 100), (57, 122)], TUNIQUE['M'])
    remplir(im, [(55, 124), (62, 124), (62, 133), (55, 133)], CEINTURE['m'])
    ligne(im, [(54, 125), (54, 132)], CEINTURE['o'])
    ligne(im, [(55, 124), (62, 124)], CEINTURE['o'])
    ligne(im, [(55, 133), (62, 133)], CEINTURE['o'])
    ligne(im, [(56, 126), (61, 126)], CEINTURE['M'])
    ligne(im, [(56, 131), (61, 131)], CEINTURE['s'])
    d = ImageDraw.Draw(im)
    for (bx, by, rx, ry, c) in ((53, 141, 9, 8, 'm'), (54, 142, 6, 6, 'M')):
        d.arc([bx - rx, by - ry, bx + rx, by + ry], 180, 360, fill=hexa(CORDE['o']), width=4)
        d.arc([bx - rx, by - ry, bx + rx, by + ry], 180, 360, fill=hexa(CORDE[c]), width=2)
    d.arc([44, 133, 62, 149], 200, 260, fill=hexa(CORDE['c']), width=1)
    ligne(im, [(56, 133), (57, 135)], CORDE['o'], 3)
    ligne(im, [(56, 134), (56, 135)], CORDE['M'], 1)


from cartes import TETE, POING, tourner

EPAULE = (58, 106)

# Du monde à la case : le pied du pet (96, 203) se pose au bord gauche de la colonne de forge
# où il s'ancre, au bas de la scène ; 26 pixels de forge de haut pour ses 198 dans la case, sur
# des cellules de 9 × 19 (rendu/src/forge/pet.rs, PIED et HAUTEUR).
PAS = (9 / (26 * 9.5 / 198), 9.5 / (26 * 9.5 / 198))
SH = 32


def depuis_forge(gx, gy, ancre):
    """Le centre du pixel de forge (gx, gy) dans la case, le pet ancré à la colonne `ancre`."""
    return (96 + (gx + 0.5 - ancre) * PAS[0], 203 - (SH - gy - 0.5) * PAS[1])


# Où se tient le pet, en colonnes de forge (atelier.rs : C.0 = 30, + 8 pour son milieu, plus son
# décalage dans pet.rs) : au soufflet (BELLOWS_SPOT − 8), et à l'enclume, approché de 4.
AU_SOUFFLET = 30
A_L_ENCLUME = 34

# Les gestes du coup de marteau, dans la case : le poing, la tête du marteau, de combien de quarts
# de tour le manche penche.
FRAPPE = {
    'levé': dict(poing=(48, 80), tete=(47, 49), quarts=0),
    'abattu': dict(poing=(42, 102), tete=(17, 101), quarts=1),
    'frappe': dict(poing=(40, 127), tete=(15, 129), quarts=1),
}

# Les gestes sans marteau, où la forge pose le bout de son bras (atelier.rs, le bras gauche) : le
# poing s'y referme, sur ce que la forge y dessine (le levier, la garde de la lame, le sceau).
GESTES = {
    # le levier du soufflet, son bout en (BELLOWS.0 + 5, BELLOWS.1), deux plus bas pressé
    'soufflet-haut': dict(forge=(18, 20.5), ancre=AU_SOUFFLET, quarts=0),
    'soufflet-bas': dict(forge=(18, 22.5), ancre=AU_SOUFFLET, quarts=0),
    # la lame chaude levée devant lui, sa garde en (x − 2, y + 10) ; puis au seau, la pointe en bas
    'lame': dict(forge=(28, 17.5), ancre=A_L_ENCLUME, quarts=0),
    'bain': dict(forge=(22, 19.8), ancre=A_L_ENCLUME, quarts=0),
    # la lame du jour, à plat, descendue dans la caisse (son talon en x 25, de y 15 à 18)
    'range-1': dict(forge=(26, 15.5), ancre=A_L_ENCLUME, quarts=1),
    'range-2': dict(forge=(26, 16.5), ancre=A_L_ENCLUME, quarts=1),
    'range-3': dict(forge=(26, 17.5), ancre=A_L_ENCLUME, quarts=1),
    'range-4': dict(forge=(26, 18.5), ancre=A_L_ENCLUME, quarts=1),
    # le couvercle rabattu, puis le sceau (son manche en x 21) levé et frappé sur la jointure
    'couvercle': dict(forge=(25, 18.5), ancre=A_L_ENCLUME, quarts=1),
    'sceau-haut': dict(forge=(22, 11.5), ancre=A_L_ENCLUME, quarts=0),
    'sceau-bas': dict(forge=(22, 16.5), ancre=A_L_ENCLUME, quarts=0),
}

# Le balai, que la forge dessine (atelier.rs, draw_broom) : la patte du haut (la gauche de l'écran,
# en travers de la poitrine) et celle du bas (la droite) sur son manche, à chaque allonge du coup,
# là où la forge les met, le pet décalé de 2 vers la cour (x + 8 + 2, x le bord de son dessin).
AU_BALAI = 10
BALAI = {f'balai-{r}': dict(haut=(h, 19.5), bas=(b, 23.5)) for r, (h, b) in enumerate([(15, 17), (16, 18), (16, 19), (17, 20), (18, 20)])}

# Les rangées de rendu/pet/forge.webp, dans l'ordre de pet.rs (Rangee::ou) : None, la case
# d'origine, le marteau au repos.
RANGEES = [
    [None, 'levé', 'abattu', 'frappe'],
    ['soufflet-haut', 'soufflet-bas'],
    ['lame', 'bain'],
    ['range-1', 'range-2', 'range-3', 'range-4', 'couvercle', 'sceau-haut', 'sceau-bas'],
    list(BALAI),
    ['sommeil', 'sommeil-tete'],
]


def poser(im, c, centre, pal):
    h, w = len(c), len(c[0])
    carte(im, round(centre[0] - w / 2), round(centre[1] - h / 2), c, pal)


def manche(im, f, t):
    d = ImageDraw.Draw(im)
    dx, dy = t[0] - f[0], t[1] - f[1]
    n = math.hypot(dx, dy)
    ca, sa = dx / n, dy / n
    q = (f[0] - ca * 12, f[1] - sa * 12)
    d.line([q, t], fill=hexa(BOIS['o']), width=6)
    d.line([q, t], fill=hexa(BOIS['M']), width=4)
    nx, ny = cote_lumiere(dx, dy)
    d.line([(q[0] + nx, q[1] + ny), (t[0] + nx, t[1] + ny)], fill=hexa(BOIS['c']), width=1)
    d.line([(q[0] - nx * 1.5, q[1] - ny * 1.5), (t[0] - nx * 1.5, t[1] - ny * 1.5)], fill=hexa(BOIS['m']), width=1)


# le bras et le marteau de la case d'origine
BRAS = [(0, 84), (52, 84), (56, 100), (58, 112), (60, 124), (52, 132), (44, 138), (26, 138), (0, 120)]


def sans_bras():
    im = charger()
    effacer(im, BRAS)
    reparer(im)
    return im


def geste(poing, quarts, tete=None):
    """Le bras gauche (de l'écran) tendu jusqu'à `poing`, le marteau en main si `tete`."""
    im = sans_bras()
    if tete:
        manche(im, poing, tete)
        poser(im, tourner(TETE, quarts), tete, dict(FER, k=BOIS['c']))
    avant_bras(im, (EPAULE[0] + 4, EPAULE[1] - 2), poing, 16)
    # le bras sort de sous la cape : le corps repasse par-dessus sa racine
    corps = sans_bras()
    masque = Image.new('L', im.size, 0)
    ImageDraw.Draw(masque).polygon([(57, 92), (80, 92), (80, 124), (57, 124)], fill=255)
    a = corps.getchannel('A').point(lambda v: 255 if v > 0 else 0)
    from PIL import ImageChops
    im.paste(corps, (0, 0), ImageChops.multiply(masque, a))
    poser(im, tourner(POING, quarts), poing, PATTE)
    return im


# la patte droite (de l'écran) qui pend devant la sacoche, et son coude
PATTE_DROITE = [(107, 115), (132, 115), (134, 147), (107, 147)]
COUDE = (121, 112)


def sans_patte_droite(im):
    """La patte droite effacée, et ce qu'elle cachait : la tunique, la ceinture qui file vers la
    hanche, le haut de la gourde."""
    effacer(im, PATTE_DROITE)
    remplir(im, [(107, 115), (132, 115), (133, 140), (107, 140)], TUNIQUE['m'])
    ligne(im, [(110, 117), (131, 117)], TUNIQUE['M'])
    ligne(im, [(108, 138), (132, 139)], TUNIQUE['s'])
    remplir(im, [(107, 124), (133, 125), (133, 133), (107, 132)], CEINTURE['m'])
    ligne(im, [(107, 123), (133, 124)], CEINTURE['o'])
    ligne(im, [(107, 133), (133, 134)], CEINTURE['o'])
    ligne(im, [(107, 125), (133, 126)], CEINTURE['M'])
    ligne(im, [(107, 131), (133, 132)], CEINTURE['s'])
    d = ImageDraw.Draw(im)
    ell(d, 125, 146, 9, 6, TUNIQUE['o'])
    ell(d, 125, 146, 8, 5, TUNIQUE['M'])
    ell(d, 122, 144, 3, 2, '#9a9a4a')


def balai(nom):
    p = BALAI[nom]
    haut, bas = depuis_forge(*p['haut'], AU_BALAI), depuis_forge(*p['bas'], AU_BALAI)
    im = sans_bras()
    sans_patte_droite(im)
    # la droite, du coude vers le bas du manche
    avant_bras(im, COUDE, bas, 15)
    poser(im, tourner(POING, 0), bas, PATTE)
    # la gauche, de l'épaule en travers de la poitrine, vers le haut du manche
    avant_bras(im, (EPAULE[0] + 4, EPAULE[1] - 2), haut, 13)
    poser(im, tourner(POING, 0), haut, PATTE)
    return im


# Assis sur son tabouret (atelier.rs : la forge le descend de 2 pixels, ~15 de la case) : le haut
# du corps descend, le haut des cuisses passe sous la tunique, la queue suit le corps.
ASSIS = 14
TAILLE = 152  # au-dessus : ce qui descend avec lui ; dessous, ses pattes, qui restent au sol
QUEUE = 136   # à droite de cette colonne, la queue descend tout entière


def assis(im):
    out = Image.new('RGBA', im.size, VIDE)
    out.paste(im.crop((0, TAILLE + ASSIS, QUEUE, 208)), (0, TAILLE + ASSIS))
    out.alpha_composite(im.crop((0, 0, QUEUE, TAILLE)), (0, ASSIS))
    out.alpha_composite(im.crop((QUEUE, 0, 192, 208 - ASSIS)), (QUEUE, ASSIS))
    return out


def yeux_fermes(im):
    """Les yeux fermés : la fourrure du visage repeinte sur l'œil, une paupière en arc, un cil au coin."""
    for (x0, x1), (a, b, c) in (((65, 75), ((64, 61), (70, 67), (75, 62))), ((89, 101), ((88, 62), (95, 67), (102, 60)))):
        # la fourrure claire d'autour (ni les cils, ni les marques), ombrée sous le sourcil
        autour = [im.getpixel((x, y)) for x in range(x0 - 2, x1 + 3) for y in (51, 52, 71, 72)]
        autour = [p for p in autour if min(p[:3]) > 150]
        f = [sum(p[k] for p in autour) / len(autour) for k in range(3)]
        for x in range(x0, x1 + 1):
            for y in range(53, 71):
                t = 0.86 + 0.14 * min(1, (y - 53) / 9)
                im.putpixel((x, y), tuple(round(f[k] * t) for k in range(3)) + (255,))
        # la paupière, une courbe qui tombe au milieu, épaissie au creux ; un cil au coin
        pts = [(round((1 - t) ** 2 * a[0] + 2 * t * (1 - t) * b[0] + t * t * c[0]), round((1 - t) ** 2 * a[1] + 2 * t * (1 - t) * (b[1] + 1) + t * t * c[1])) for t in [i / 24 for i in range(25)]]
        for x, y in pts:
            point(im, x, y, FOURRURE['o'])
        for x, y in pts[6:19]:
            point(im, x, y - 1, FOURRURE['O'])
            point(im, x, y + 1, FOURRURE['s'])
        coin = a if a[0] < 80 else c
        point(im, coin[0] - (1 if a[0] < 80 else -1), coin[1] - 1, FOURRURE['o'])


def sommeil(nom):
    """Il somnole sur son tabouret, les pattes sur les genoux ; la tête dodeline (sommeil-tete)."""
    im = sans_bras()
    yeux_fermes(im)
    im = assis(im)
    # la patte gauche posée sur le genou
    genou = (84, 150)
    avant_bras(im, (EPAULE[0] + 4, EPAULE[1] - 2 + ASSIS), genou, 15)
    poser(im, tourner(POING, 1), genou, PATTE)
    if nom == 'sommeil-tete':
        im.alpha_composite(im.crop((0, 0, 192, 92 + ASSIS)), (0, 4))
    return im


def image(nom):
    if nom is None:
        return charger()
    if nom.startswith('sommeil'):
        return sommeil(nom)
    if nom in BALAI:
        return balai(nom)
    if nom in FRAPPE:
        p = FRAPPE[nom]
        return geste(p['poing'], p['quarts'], p['tete'])
    p = GESTES[nom]
    return geste(depuis_forge(*p['forge'], p['ancre']), p['quarts'])


def planche(sortie=os.path.join(PET, 'forge.webp')):
    out = Image.new('RGBA', (8 * 192, len(RANGEES) * 208), (0, 0, 0, 0))
    for r, noms in enumerate(RANGEES):
        for k, nom in enumerate(noms):
            out.paste(image(nom), (k * 192, r * 208))
    out.save(sortie, lossless=True, quality=100, method=6)


def voir(noms):
    images = [image(n) for n in noms]
    out = Image.new('RGBA', (len(images) * 200, 212), (40, 36, 40, 255))
    for k, im in enumerate(images):
        out.alpha_composite(im, (k * 200, 2))
    out.resize((out.width * 2, out.height * 2), Image.NEAREST).save('gestes-x2.png')


if __name__ == '__main__':
    import sys
    noms = [a for a in sys.argv[1:] if not a.startswith('-')]
    voir(noms or [n for r in RANGEES for n in r if n]) if '--voir' in sys.argv else planche()
