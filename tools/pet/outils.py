"""Outils de pixel art pour les cases du pet : couleurs, formes sans lissage, cartes en lettres, loupe."""
import os
from PIL import Image, ImageDraw

ICI = os.path.dirname(os.path.abspath(__file__))
PET = os.path.join(ICI, '..', '..', 'rendu', 'pet')
# la case d'où partent les dessins : « waiting », image 0, le marteau au poing
CASE = (0, 6)


def hexa(s):
    s = s.lstrip('#')
    return (int(s[0:2], 16), int(s[2:4], 16), int(s[4:6], 16), 255)


VIDE = (0, 0, 0, 0)


def charger():
    planche = Image.open(os.path.join(PET, 'scorpheus.webp')).convert('RGBA')
    x, y = CASE[0] * 192, CASE[1] * 208
    return planche.crop((x, y, x + 192, y + 208))


def effacer(im, poly):
    m = Image.new('L', im.size, 0)
    ImageDraw.Draw(m).polygon(poly, fill=255)
    vide = Image.new('RGBA', im.size, VIDE)
    im.paste(vide, (0, 0), m)


def remplir(im, poly, c):
    """Un polygone plein, sans lissage."""
    ImageDraw.Draw(im).polygon(poly, fill=hexa(c) if isinstance(c, str) else c)


def ligne(im, pts, c, epais=1):
    ImageDraw.Draw(im).line(pts, fill=hexa(c) if isinstance(c, str) else c, width=epais)


def point(im, x, y, c):
    if 0 <= x < im.width and 0 <= y < im.height:
        im.putpixel((x, y), hexa(c) if isinstance(c, str) else c)


def carte(im, x0, y0, lignes, couleurs):
    """Des pixels écrits en lettres : chaque lettre une couleur de `couleurs`, '.' rien."""
    for j, l in enumerate(lignes):
        for i, ch in enumerate(l):
            if ch in couleurs:
                point(im, x0 + i, y0 + j, couleurs[ch])


def loupe(im, boite, k, sortie, grille=True):
    r = im.crop(boite)
    fond = Image.new('RGBA', r.size, (90, 90, 90, 255))
    fond.alpha_composite(r)
    z = fond.resize((r.width * k, r.height * k), Image.NEAREST)
    if grille:
        d = ImageDraw.Draw(z)
        for x in range(r.width + 1):
            d.line([(x * k, 0), (x * k, z.height)], fill=(0, 0, 0, 255) if (x + boite[0]) % 10 == 0 else (70, 70, 70, 255))
        for y in range(r.height + 1):
            d.line([(0, y * k), (z.width, y * k)], fill=(0, 0, 0, 255) if (y + boite[1]) % 10 == 0 else (70, 70, 70, 255))
    z.save(sortie)
