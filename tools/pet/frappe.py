"""Le coup de marteau de Scorpheus, en pixel art, sur la case « waiting » n° 0 : le corps gardé,
le bras et le marteau effacés puis redessinés dans chaque position, aux proportions de l'original
(un gros poing serré juste sous la tête d'un marteau court).

python tools/pet/frappe.py          → rendu/pet/forge.webp (la rangée de la frappe)
python tools/pet/frappe.py --voir   → frappe-x2.png et zooms.png ici, pour regarder
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


from cartes import TETE, POING, EPAULE as TOUFFE, tourner

EPAULE = (58, 106)
# le poing et la tête du marteau (leurs centres), et de combien de quarts de tour le manche penche
POSES = {
    'levé': dict(poing=(48, 80), tete=(47, 49), quarts=0),
    'abattu': dict(poing=(42, 102), tete=(17, 101), quarts=1),
    'frappe': dict(poing=(40, 127), tete=(15, 129), quarts=1),
}


def poser(im, carte, centre, pal):
    h, w = len(carte), len(carte[0])
    carte_ = carte
    carte(im, round(centre[0] - w / 2), round(centre[1] - h / 2), carte_, pal) if False else None
    from outils import carte as dessiner
    dessiner(im, round(centre[0] - w / 2), round(centre[1] - h / 2), carte_, pal)


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


def pose(nom):
    im = charger()
    effacer(im, [(0, 84), (52, 84), (56, 100), (58, 112), (60, 124), (52, 132), (44, 138), (26, 138), (0, 120)])
    reparer(im)
    p = POSES[nom]
    f, t, q = p['poing'], p['tete'], p['quarts']
    manche(im, f, t)
    poser(im, tourner(TETE, q), t, dict(FER, k=BOIS['c']))
    avant_bras(im, (EPAULE[0] + 4, EPAULE[1] - 2), f, 16)
    # le bras sort de sous la cape : le corps repasse par-dessus sa racine
    corps = charger()
    effacer(corps, [(0, 84), (52, 84), (56, 100), (58, 112), (60, 124), (52, 132), (44, 138), (26, 138), (0, 120)])
    reparer(corps)
    masque = Image.new('L', im.size, 0)
    ImageDraw.Draw(masque).polygon([(57, 92), (80, 92), (80, 124), (57, 124)], fill=255)
    a = corps.getchannel('A').point(lambda v: 255 if v > 0 else 0)
    from PIL import ImageChops
    im.paste(corps, (0, 0), ImageChops.multiply(masque, a))
    poser(im, tourner(POING, q), f, PATTE)
    return im


def planche(sortie=os.path.join(PET, 'forge.webp')):
    """La rangée de la forge : au repos (la case d'origine), levé, abattu, frappe."""
    images = [charger()] + [pose(n) for n in ('levé', 'abattu', 'frappe')]
    out = Image.new('RGBA', (8 * 192, 208), (0, 0, 0, 0))
    for k, im in enumerate(images):
        out.paste(im, (k * 192, 0))
    out.save(sortie, lossless=True, quality=100, method=6)


def voir():
    noms = ['levé', 'abattu', 'frappe']
    images = [charger()] + [pose(n) for n in noms]
    out = Image.new('RGBA', (len(images) * 200, 212), (40, 36, 40, 255))
    for k, im in enumerate(images):
        out.alpha_composite(im, (k * 200, 2))
    out.resize((out.width * 2, out.height * 2), Image.NEAREST).save('frappe-x2.png')
    zs = []
    for n, im in zip(noms, images[1:]):
        loupe(im, (0, 30, 100, 160), 6, f'zoom-{n}.png', grille=False)
        zs.append(Image.open(f'zoom-{n}.png'))
    o = Image.new('RGB', (sum(z.width for z in zs) + 20, zs[0].height))
    x = 0
    for z in zs:
        o.paste(z, (x, 0))
        x += z.width + 10
    o.save('zooms.png')


if __name__ == '__main__':
    import sys
    voir() if '--voir' in sys.argv else planche()
