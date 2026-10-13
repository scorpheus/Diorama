"""Faire redessiner les gestes par ChatGPT, et reprendre ce qu'il rend (cf. CHATGPT.md).

python tools/pet/chatgpt.py --guide            → tools/pet/chatgpt/guide.png et reference.png,
                                                  à lui envoyer
python tools/pet/chatgpt.py <son image.png>     → rendu/pet/forge.webp refait de ses poses, et
                                                  tools/pet/chatgpt/apercu.png pour comparer

Ses poses sont prises dans l'ordre du guide (de gauche à droite, rangée par rangée), chacune à la
taille et à la place de la mienne : même hauteur, les pattes au même endroit. Ce qu'il dessine
autour (le fond, un damier « transparent ») est retiré d'après les couleurs du bord.
"""
import os
import sys
from statistics import median
from PIL import Image, ImageDraw

from outils import ICI, PET, charger
import gestes

COLONNES = 6
FOND = (255, 0, 255, 255)  # un magenta franc : rien de tel sur le personnage
MARGE = 24
SORTIE = os.path.join(ICI, 'chatgpt')  # ignoré par git


def noms():
    return [n for r in gestes.RANGEES for n in r if n]


def franc(im):
    """Chaque pixel opaque ou vide : un demi-transparent virerait au rose sur le magenta. Et les
    trous du dessin d'origine (des pixels vides au milieu de la fourrure) bouchés de la couleur de
    leurs voisins, pour qu'il n'y voie pas du fond."""
    im = im.copy()
    a = im.getchannel('A').point(lambda v: 255 if v > 127 else 0)
    dehors = a.copy()
    ImageDraw.floodfill(dehors, (0, 0), 128)
    im.putalpha(a)
    px, w, h = im.load(), im.width, im.height
    trous = {(x, y) for y in range(h) for x in range(w) if dehors.getpixel((x, y)) == 0}
    while trous:
        bouches = {}
        for x, y in trous:
            v = [px[x + i, y + j] for i, j in ((-1, 0), (1, 0), (0, -1), (0, 1)) if 0 <= x + i < w and 0 <= y + j < h and px[x + i, y + j][3] and (x + i, y + j) not in trous]
            if v:
                bouches[x, y] = tuple(sum(c[k] for c in v) // len(v) for k in range(3)) + (255,)
        if not bouches:
            break
        for p, c in bouches.items():
            px[p] = c
        trous -= set(bouches)
    return im


def guide():
    ns = noms()
    lignes = (len(ns) + COLONNES - 1) // COLONNES
    w, h = 192 + MARGE, 208 + MARGE
    out = Image.new('RGBA', (COLONNES * w + MARGE, lignes * h + MARGE), FOND)
    for k, n in enumerate(ns):
        out.alpha_composite(franc(gestes.image(n)), (MARGE + (k % COLONNES) * w, MARGE + (k // COLONNES) * h))
    os.makedirs(SORTIE, exist_ok=True)
    out.convert('RGB').save(os.path.join(SORTIE, 'guide.png'))
    ref = Image.new('RGBA', (192 + 2 * MARGE, 208 + 2 * MARGE), FOND)
    ref.alpha_composite(franc(charger()), (MARGE, MARGE))
    ref.convert('RGB').resize((ref.width * 4, ref.height * 4), Image.NEAREST).save(os.path.join(SORTIE, 'reference.png'))
    print(f'{SORTIE} : guide.png ({len(ns)} poses en {COLONNES} colonnes), reference.png (le personnage d\'origine)')


def detourer(im):
    """Le personnage seul : ce qui ressemble aux couleurs du bord (le fond, ou les deux cases
    d'un damier) devient transparent."""
    im = im.convert('RGBA')
    w, h = im.size
    px = im.load()
    bord = [px[x, y] for x in range(0, w, 3) for y in (0, 1, h - 2, h - 1)] + [px[x, y] for y in range(0, h, 3) for x in (0, 1, w - 2, w - 1)]
    if sum(p[3] < 128 for p in bord) > len(bord) * 0.8:
        return im  # il a rendu un vrai fond transparent
    # les couleurs du bord, grossièrement : celles qui en font l'essentiel
    compte = {}
    for p in bord:
        q = tuple(c // 16 for c in p[:3])
        compte[q] = compte.get(q, 0) + 1
    fonds = []
    for q, n in sorted(compte.items(), key=lambda e: -e[1]):
        if sum(compte[f] for f in fonds) > len(bord) * 0.9:
            break
        fonds.append(q)
    teintes = [tuple(c * 16 + 8 for c in q) for q in fonds]
    for y in range(h):
        for x in range(w):
            p = px[x, y]
            if any(sum((p[k] - t[k]) ** 2 for k in range(3)) < 40 ** 2 for t in teintes):
                px[x, y] = (0, 0, 0, 0)
    # le liseré : au bord du personnage, ce que le magenta a teinté (plus de rouge et de bleu
    # que de vert, ce que ni la fourrure, ni la cape, ni la crête n'ont)
    for _ in range(2):
        bord = [(x, y) for y in range(1, h - 1) for x in range(1, w - 1)
                if px[x, y][3] and (not px[x - 1, y][3] or not px[x + 1, y][3] or not px[x, y - 1][3] or not px[x, y + 1][3])]
        for x, y in bord:
            r, g, b, _ = px[x, y]
            if min(r, b) - g > 40:
                px[x, y] = (0, 0, 0, 0)
    return im


def plages(occupe, ecart, minimum):
    """Les plages occupées d'une projection, les trous plus courts que `ecart` comblés."""
    out, debut, vide = [], None, 0
    for i, o in enumerate(occupe + [False] * (ecart + 1)):
        if o:
            if debut is None:
                debut = i
            vide = 0
        elif debut is not None:
            vide += 1
            if vide > ecart:
                fin = i - vide + 1
                if fin - debut >= minimum:
                    out.append((debut, fin))
                debut, vide = None, 0
    return out


def poses(im):
    """Ses poses, rangée par rangée, de gauche à droite : chacune rognée à ce qu'elle montre."""
    a = im.getchannel('A').point(lambda v: 255 if v > 96 else 0)
    w, h = im.size
    lignes = [a.crop((0, y, w, y + 1)).getbbox() is not None for y in range(h)]
    out = []
    for y0, y1 in plages(lignes, h // 60, h // 20):
        bande = a.crop((0, y0, w, y1))
        cols = [bande.crop((x, 0, x + 1, y1 - y0)).getbbox() is not None for x in range(w)]
        for x0, x1 in plages(cols, w // 150, w // 40):
            boite = a.crop((x0, y0, x1, y1)).getbbox()
            out.append(im.crop((x0 + boite[0], y0 + boite[1], x0 + boite[2], y0 + boite[3])))
    return out


def pied(im):
    """Le milieu de ce qui touche le sol : les colonnes occupées dans le bas de l'image."""
    a = im.getchannel('A')
    h = im.height
    bas = a.crop((0, h - max(2, h // 12), im.width, h))
    xs = [x for x in range(im.width) if bas.crop((x, 0, x + 1, bas.height)).getbbox()]
    return (xs[0] + xs[-1]) / 2 if xs else im.width / 2


def importer(chemin):
    ns = noms()
    siennes = poses(detourer(Image.open(chemin)))
    if len(siennes) != len(ns):
        sys.exit(f'{len(siennes)} poses trouvées dans {chemin}, il en faut {len(ns)} ({COLONNES} par rangée, '
                 'bien séparées) : rien d\'écrit')
    miennes = {n: gestes.image(n) for n in ns}
    boites = {n: miennes[n].getbbox() for n in ns}
    # une seule échelle pour toutes : il les a dessinées à la même taille
    k = median((boites[n][3] - boites[n][1]) / s.height for n, s in zip(ns, siennes))
    print(f'{len(siennes)} poses, mises à l\'échelle {k:.3f}')
    refaites = {}
    for n, s in zip(ns, siennes):
        s = s.resize((max(1, round(s.width * k)), max(1, round(s.height * k))), Image.LANCZOS)
        b = boites[n]
        mien = miennes[n].crop(b)
        # les pattes au sol au même endroit que les miennes, le bas sur le même bas
        x = round(b[0] + pied(mien) - pied(s))
        y = b[3] - s.height
        case = Image.new('RGBA', (192, 208), (0, 0, 0, 0))
        case.alpha_composite(s, (max(0, x), max(0, y)), (max(0, -x), max(0, -y)))
        refaites[n] = case
    out = Image.new('RGBA', (8 * 192, len(gestes.RANGEES) * 208), (0, 0, 0, 0))
    for r, rangee in enumerate(gestes.RANGEES):
        for c, n in enumerate(rangee):
            out.paste(refaites[n] if n else charger(), (c * 192, r * 208))
    out.save(os.path.join(PET, 'forge.webp'), lossless=True, quality=100, method=6)
    # les miennes au-dessus, les siennes dessous
    ap = Image.new('RGBA', (len(ns) * 100, 220), (40, 36, 40, 255))
    for i, n in enumerate(ns):
        ap.alpha_composite(miennes[n].resize((96, 104)), (i * 100, 2))
        ap.alpha_composite(refaites[n].resize((96, 104)), (i * 100, 112))
    os.makedirs(SORTIE, exist_ok=True)
    ap.save(os.path.join(SORTIE, 'apercu.png'))
    print(f'rendu/pet/forge.webp écrit ; {os.path.join(SORTIE, "apercu.png")} : les miennes en haut, les siennes en bas')


if __name__ == '__main__':
    if '--guide' in sys.argv:
        guide()
    elif len(sys.argv) > 1:
        importer(sys.argv[1])
    else:
        print(__doc__)
