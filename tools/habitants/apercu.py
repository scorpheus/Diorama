"""Aperçu des habitants des onglets (habitants/*.json, contrat : habitants/LISEZMOI.md).

    python tools/habitants/apercu.py habitants/marin.json planche.png [--anime anim.gif] [--teinte #e05050]

Vérifie le fichier (taille des images, lettres de la palette, états obligatoires) et dessine une
planche : une rangée par état, ses images agrandies, puis l'habitant à sa vraie taille et deux
fois plus grand, sur une barre d'onglets claire et une sombre. --anime écrit un GIF qui joue tous
les états côte à côte, à leur vrai rythme (un tic = 150 ms). --teinte change la couleur du profil.
"""
import json
import sys

from PIL import Image, ImageDraw

TIC_MS = 150
ETATS = ['travail', 'repos', 'attente', 'echec', 'parle']
LOUPE = 10


def hexa(c):
    c = c.lstrip('#')
    return (int(c[0:2], 16), int(c[2:4], 16), int(c[4:6], 16), 255)


def assombrir(rgba, f):
    return tuple(round(v * f) for v in rgba[:3]) + (255,)


def charger(chemin, teinte=None):
    h = json.load(open(chemin, encoding='utf-8'))
    w, ht = h['taille']
    pal = {k: hexa(v) for k, v in h['palette'].items()}
    if teinte:
        pal['O'] = hexa(teinte)
        pal['o'] = assombrir(pal['O'], 0.62)
    fautes = []
    if h.get('taille') != [16, 16]:
        fautes.append(f"taille {h.get('taille')} : un habitant fait [16, 16]")
    if not isinstance(h.get('valide'), bool):
        fautes.append('valide doit être true ou false')
    for e in ETATS:
        if e not in h['etats']:
            fautes.append(f'état « {e} » absent')
    for nom, etat in h['etats'].items():
        if not isinstance(etat.get('every'), int) or etat['every'] < 1:
            fautes.append(f'{nom} : every doit être un entier ≥ 1')
        for i, img in enumerate(etat['images']):
            if len(img) != ht:
                fautes.append(f'{nom}[{i}] : {len(img)} lignes, {ht} attendues')
            for j, ligne in enumerate(img):
                if len(ligne) != w:
                    fautes.append(f'{nom}[{i}] ligne {j} : {len(ligne)} pixels, {w} attendus')
                for c in ligne:
                    if c != '.' and c not in pal:
                        fautes.append(f'{nom}[{i}] ligne {j} : lettre « {c} » hors palette')
    if fautes:
        sys.exit(f'{chemin} refusé :\n  ' + '\n  '.join(sorted(set(fautes))[:20]))
    return h, pal


def image(img, pal, loupe=1):
    w, h = len(img[0]), len(img)
    im = Image.new('RGBA', (w, h), (0, 0, 0, 0))
    for y, ligne in enumerate(img):
        for x, c in enumerate(ligne):
            if c != '.':
                im.putpixel((x, y), pal[c])
    return im.resize((w * loupe, h * loupe), Image.NEAREST) if loupe > 1 else im


def planche(h, pal):
    w, ht = h['taille']
    etats = list(h['etats'].items())
    n = max(len(e['images']) for _, e in etats)
    marge, titre = 16, 110
    cw, ch = w * LOUPE + 8, ht * LOUPE + 8
    W = titre + n * cw + marge * 2
    H = marge + len(etats) * (ch + 6) + 120
    out = Image.new('RGBA', (W, H), (40, 42, 50, 255))
    d = ImageDraw.Draw(out)
    d.text((marge, 2), f"{h['nom']} — {h.get('description', '')}", fill=(230, 230, 230))
    for r, (nom, e) in enumerate(etats):
        y = marge + r * (ch + 6)
        d.text((marge, y + ch // 2 - 6), f"{nom} ×{e['every']}", fill=(220, 220, 220))
        for i, img in enumerate(e['images']):
            x = titre + i * cw
            d.rectangle([x, y, x + cw - 4, y + ch - 4], fill=(62, 66, 78))
            out.alpha_composite(image(img, pal, LOUPE), (x + 2, y + 2))
    # à vraie taille et doublé, sur deux barres d'onglets
    y0 = marge + len(etats) * (ch + 6) + 8
    for b, fond in enumerate([(232, 228, 220, 255), (30, 31, 38, 255)]):
        y = y0 + b * 54
        d.rectangle([marge, y, W - marge, y + 48], fill=fond)
        x = marge + 8
        for nom, e in etats:
            im = image(e['images'][0], pal)
            out.alpha_composite(im, (x, y + 48 - ht - 4))
            out.alpha_composite(im.resize((w * 2, ht * 2), Image.NEAREST), (x + w + 6, y + 48 - ht * 2 - 4 + 2))
            x += w * 3 + 20
    return out


def anime(h, pal, chemin):
    w, ht = h['taille']
    etats = list(h['etats'].items())
    loupe = 6
    W = len(etats) * (w * loupe + 12) + 12
    H = ht * loupe + 40
    # assez de tics pour que chaque état fasse au moins deux tours
    tics = max(2 * e['every'] * len(e['images']) for _, e in etats)
    images = []
    for t in range(tics):
        im = Image.new('RGBA', (W, H), (232, 228, 220, 255))
        d = ImageDraw.Draw(im)
        for k, (nom, e) in enumerate(etats):
            x = 12 + k * (w * loupe + 12)
            f = e['images'][(t // e['every']) % len(e['images'])]
            im.alpha_composite(image(f, pal, loupe), (x, 8))
            d.text((x, ht * loupe + 16), nom, fill=(40, 40, 40))
        images.append(im.convert('RGB'))
    images[0].save(chemin, save_all=True, append_images=images[1:], duration=TIC_MS, loop=0)


if __name__ == '__main__':
    a = sys.argv[1:]
    teinte = a[a.index('--teinte') + 1] if '--teinte' in a else None
    h, pal = charger(a[0], teinte)
    planche(h, pal).save(a[1])
    if '--anime' in a:
        anime(h, pal, a[a.index('--anime') + 1])
    print('ok')
