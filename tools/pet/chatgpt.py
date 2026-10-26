"""Faire redessiner le pet par ChatGPT ou Gemini, et reprendre ce qu'il rend (cf. CHATGPT.md).

Les gestes de la forge, redessinés :
python tools/pet/chatgpt.py --guide             → tools/pet/chatgpt/guide.png et reference.png
python tools/pet/chatgpt.py <son image.png>      → rendu/pet/forge.webp refait de ses poses, et
                                                   tools/pet/chatgpt/apercu.png pour comparer

Les accessoires, ajoutés sur les poses telles qu'elles sont :
python tools/pet/chatgpt.py --tenue             → tools/pet/chatgpt/tenue-1.png à tenue-4.png
python tools/pet/chatgpt.py --tenue <accessoire> <n> <son image.png>
                                                → rendu/pet/<accessoire>.webp complété des poses
                                                  de tenue-<n>, et apercu-<accessoire>-<n>.png
python tools/pet/chatgpt.py --epurer            → les calques déjà faits repassés à l'épuration, et
                                                  tools/pet/chatgpt/epure-<accessoire>.png

Les gestes : ses poses sont prises dans l'ordre du guide (de gauche à droite, rangée par rangée),
chacune à la taille et à la place de la mienne : même hauteur, les pattes au même endroit. Ce
qu'il dessine autour (le fond, un damier « transparent ») est retiré d'après les couleurs du bord.

Les accessoires : son image est ramenée à la taille de la planche envoyée, chaque pose recalée sur
la mienne, et n'en est gardé que ce qui a changé : un calque que l'appli pose sur le pet. Le dessin
d'origine reste intact. S'il a redessiné une pose (la tête ailleurs, une gourde ajoutée), ce qui a
changé n'est pas que l'accessoire : le calque est épuré (epurer) jusqu'à ne garder que lui.
"""
import os
import sys
from statistics import median
import numpy as np
from PIL import Image, ImageDraw, ImageFilter

from outils import ICI, PET, charger
import gestes

COLONNES = 6
FOND = (255, 0, 255, 255)  # un magenta franc : rien de tel sur le personnage
MARGE = 24
SORTIE = os.path.join(ICI, 'chatgpt')  # ignoré par git


def noms():
    return [n for r in gestes.RANGEES for n in r if n]


# --- les accessoires ---------------------------------------------------------------------------

ACCESSOIRES = ('lunettes', 'bandeau', 'tablier')
# les rangées du pet que la forge montre (pet.rs, Rangee::ou), et leur nombre d'images
RANGEES_PET = [6, 8, 8, 4, 5, 8, 6, 6, 6]
PAR_RANGEE, PAR_PLANCHE = 5, 20
CASE = (192, 208)


def poses_tenue():
    """Toutes les poses que l'appli peint : (planche, rangée, colonne), la première planche puis
    celle de la forge (sans sa case 0, qui est la case d'attente du pet)."""
    out = [(0, r, i) for r, n in enumerate(RANGEES_PET) for i in range(n)]
    out += [(1, r, i) for r, rangee in enumerate(gestes.RANGEES) for i, n in enumerate(rangee) if n]
    return out


def planches_tenue():
    ps = poses_tenue()
    return [ps[k:k + PAR_PLANCHE] for k in range(0, len(ps), PAR_PLANCHE)]


def case_de(p, r, i):
    planche = Image.open(os.path.join(PET, ('scorpheus', 'forge')[p] + '.webp')).convert('RGBA')
    return planche.crop((i * CASE[0], r * CASE[1], (i + 1) * CASE[0], (r + 1) * CASE[1]))


def coin(k):
    """Le coin de la k-ième pose dans une planche de tenue."""
    return (MARGE + (k % PAR_RANGEE) * (CASE[0] + MARGE), MARGE + (k // PAR_RANGEE) * (CASE[1] + MARGE))


def taille_tenue(n):
    return (PAR_RANGEE * (CASE[0] + MARGE) + MARGE, (n + PAR_RANGEE - 1) // PAR_RANGEE * (CASE[1] + MARGE) + MARGE)


def guide_tenue():
    os.makedirs(SORTIE, exist_ok=True)
    for n, ps in enumerate(planches_tenue(), 1):
        out = Image.new('RGBA', taille_tenue(len(ps)), FOND)
        for k, (p, r, i) in enumerate(ps):
            out.alpha_composite(franc(case_de(p, r, i)), coin(k))
        out.convert('RGB').save(os.path.join(SORTIE, f'tenue-{n}.png'))
    print(f'{SORTIE} : tenue-1.png à tenue-{len(planches_tenue())}.png ({len(poses_tenue())} poses, {PAR_RANGEE} par rangée)')


def recaler(mien, sien, x0, y0, portee=12):
    """Le décalage (dx, dy) de `sien` (toute sa planche) qui pose le mieux la case `mien`, dont
    le coin est en (x0, y0) : l'écart moyen de couleur le plus petit sur ce que montre `mien`."""
    m = np.asarray(mien).astype(np.int32)
    s = np.asarray(sien).astype(np.int32)
    plein = m[..., 3] > 128
    h, w = plein.shape
    meilleur = None
    for dy in range(-portee, portee + 1):
        for dx in range(-portee, portee + 1):
            y, x = y0 + dy, x0 + dx
            if y < 0 or x < 0 or y + h > s.shape[0] or x + w > s.shape[1]:
                continue
            c = s[y:y + h, x:x + w]
            d = np.where(c[..., 3] > 128, np.abs(c[..., :3] - m[..., :3]).sum(-1), 300)[plein].mean()
            if meilleur is None or d < meilleur[0]:
                meilleur = (d, dx, dy)
    return meilleur


def calque(mien, sien):
    """Ce que `sien` a de plus que `mien` (deux cases recalées) : là où il montre autre chose,
    en taches plus larges que deux pixels (les contours décalés d'un pixel n'en sont pas)."""
    m = np.asarray(mien).astype(np.int32)
    s = np.asarray(sien).astype(np.int32)
    # pas le fond resté dans un creux : son magenta, plus de rouge et de bleu que de vert
    fond = np.minimum(s[..., 0], s[..., 2]) - s[..., 1] > 60
    change = (s[..., 3] > 128) & ~fond & ((m[..., 3] <= 128) | (np.abs(s[..., :3] - m[..., :3]).sum(-1) > SEUIL))
    masque = Image.fromarray((change * 255).astype(np.uint8))
    masque = masque.filter(ImageFilter.MinFilter(3)).filter(ImageFilter.MaxFilter(3)).filter(ImageFilter.MaxFilter(3))
    garde = (np.asarray(masque) > 0) & (s[..., 3] > 128)
    out = np.zeros_like(s)
    out[garde] = s[garde]
    out[garde, 3] = 255
    return Image.fromarray(out.astype(np.uint8), 'RGBA')


# l'écart de couleur (somme des trois canaux) au-delà duquel un pixel a changé
SEUIL = 90


def taches(masque):
    """Les taches d'un masque (4-voisins) : une liste de masques."""
    vu = np.zeros_like(masque)
    h, w = masque.shape
    out = []
    for y0, x0 in zip(*np.nonzero(masque)):
        if vu[y0, x0]:
            continue
        t, pile = np.zeros_like(masque), [(y0, x0)]
        vu[y0, x0] = True
        while pile:
            y, x = pile.pop()
            t[y, x] = True
            for j, i in ((y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1)):
                if 0 <= j < h and 0 <= i < w and masque[j, i] and not vu[j, i]:
                    vu[j, i] = True
                    pile.append((j, i))
        out.append(t)
    return out


def fourrure(px):
    """Les couleurs du pet qu'aucun accessoire ne porte : le blanc et le gris clair de la
    fourrure, le bleu de la crête et de la queue."""
    r, g, b = (px[..., k] / 255 for k in range(3))
    mx, mn = np.maximum(np.maximum(r, g), b), np.minimum(np.minimum(r, g), b)
    sat = (mx - mn) / np.maximum(mx, 1e-6)
    return ((mx > 0.62) & (sat < 0.22)) | ((b > r + 0.15) & (b > g + 0.05) & (sat > 0.35))


def saturation(px):
    rgb = px[..., :3].astype(float)
    return ((rgb.max(-1) - rgb.min(-1)) / np.maximum(rgb.max(-1), 1)).mean()


# sa tête, jusqu'au menton : ces pixels sous le haut du personnage, couché comme debout. Sur toutes
# les poses, les lunettes et le bandeau ont leur milieu à 73 pixels au plus, et le cuir du tablier
# (sa bavette comprise) à 84 au moins
TETE = 80


def cuir(rgb):
    """Le cuir brun moyen du tablier (cf. CHATGPT.md), à sa teinte : sur toutes les poses, le vert
    à 0,6 du rouge et le bleu à 0,3 ; le bois sombre, le crème, le doré s'en écartent."""
    r, g, b = (float(v) for v in rgb)
    return 80 <= r <= 190 and 0.5 <= g / r <= 0.68 and b / r < 0.42


def epurer(acc, mien, c):
    """Le calque `c` réduit à l'accessoire. Quand l'IA redessine une pose au lieu de l'habiller
    (une tête redressée, la queue ailleurs, une gourde ajoutée), tout ce qui diffère passe dans le
    calque, et le pet se montre en double. On n'en garde que l'accessoire : ni fourrure, ni
    miettes, et la seule pièce de la tête (lunettes, bandeau) ou celles du corps (tablier)."""
    c = np.asarray(c).copy()
    plein = (c[..., 3] > 0) & ~fourrure(c)
    m = Image.fromarray((plein * 255).astype(np.uint8)).filter(ImageFilter.MinFilter(3)).filter(ImageFilter.MaxFilter(3))
    plein &= np.asarray(m) > 0
    for t in taches(plein):
        if t.sum() < 40:
            plein &= ~t
    ys = np.nonzero(np.asarray(mien)[..., 3] > 128)[0]
    if not len(ys):
        return Image.fromarray(np.zeros_like(c), 'RGBA')
    tete = ys.min() + TETE
    rangee = np.arange(c.shape[0])[:, None].repeat(c.shape[1], 1)
    # les morceaux voisins (une lanière coupée par la fourrure) font un même groupe
    proche = np.asarray(Image.fromarray((plein * 255).astype(np.uint8)).filter(ImageFilter.MaxFilter(5))) > 0
    groupes = [g & plein for g in taches(proche) if (g & plein).any()]
    garde = np.zeros_like(plein)
    if acc == 'tablier':
        for g in groupes:
            # le cuir sur le corps ; pas un manche de balai redessiné, la rayure d'une queue, une gourde
            if rangee[g].mean() >= tete and g.sum() >= 150 and cuir(np.median(c[g][:, :3], 0)):
                garde |= g
    else:
        haut = [g for g in groupes if rangee[g].mean() < tete]
        if haut:
            # dans la pièce : ni morceau gris (une pointe d'oreille), ni morceau sous la tête
            for t in taches(max(haut, key=lambda g: g.sum())):
                if rangee[t].mean() < tete and saturation(c[t]) >= 0.15:
                    garde |= t
    c[~garde] = 0
    return Image.fromarray(c, 'RGBA')


def importer_tenue(acc, n, chemin):
    if acc not in ACCESSOIRES:
        sys.exit(f'accessoire inconnu : {acc} ({", ".join(ACCESSOIRES)})')
    planches = planches_tenue()
    if not 1 <= n <= len(planches):
        sys.exit(f'planche {n} : il y en a {len(planches)}')
    ps = planches[n - 1]
    taille = taille_tenue(len(ps))
    brut = Image.open(chemin).convert('RGBA')
    ecart = (brut.width / brut.height) / (taille[0] / taille[1]) - 1
    print(f'{chemin} : {brut.width} × {brut.height}, ramenée à {taille[0]} × {taille[1]} (proportions à {ecart:+.1%})')
    if abs(ecart) > 0.06:
        sys.exit("ses proportions ne sont pas celles de la planche envoyée : il a changé la mise en page, rien d'écrit")
    sien = detourer(brut.resize(taille, Image.LANCZOS))
    sortie = os.path.join(PET, f'{acc}.webp')
    lignes = len(RANGEES_PET) + len(gestes.RANGEES)
    feuille = Image.open(sortie).convert('RGBA') if os.path.exists(sortie) else None
    if feuille is None or feuille.size != (8 * CASE[0], lignes * CASE[1]):
        feuille = Image.new('RGBA', (8 * CASE[0], lignes * CASE[1]), (0, 0, 0, 0))
    ap = Image.new('RGBA', (len(ps) * 3 * 100, 110), (40, 36, 40, 255))
    vides, redessinees = [], []
    for k, (p, r, i) in enumerate(ps):
        mien = franc(case_de(p, r, i))
        x0, y0 = coin(k)
        d, dx, dy = recaler(mien, sien, x0, y0)
        case = sien.crop((x0 + dx, y0 + dy, x0 + dx + CASE[0], y0 + dy + CASE[1]))
        brut_ = calque(mien, case)
        c = epurer(acc, mien, brut_)
        avant, apres = (int(np.count_nonzero(np.asarray(x)[..., 3])) for x in (brut_, c))
        if c.getbbox() is None:
            vides.append(k + 1)
        elif avant > 2 * apres:
            redessinees.append(k + 1)
        for cx, cy in places(p, r, i):
            feuille.paste((0, 0, 0, 0), (cx * CASE[0], cy * CASE[1], (cx + 1) * CASE[0], (cy + 1) * CASE[1]))
            feuille.paste(c, (cx * CASE[0], cy * CASE[1]))
        habille = mien.copy()
        habille.alpha_composite(c)
        for j, im in enumerate((mien, case, habille)):
            ap.alpha_composite(im.resize((96, 104)), ((k * 3 + j) * 100, 3))
        print(f'  pose {k + 1:2} (planche {p}, rangée {r}, case {i}) : recalée de ({dx:+}, {dy:+}), écart {d:.0f} ; '
              f'calque de {avant} pixels, {apres} gardés')
    feuille.save(sortie, lossless=True, quality=100, method=6)
    os.makedirs(SORTIE, exist_ok=True)
    ap.save(os.path.join(SORTIE, f'apercu-{acc}-{n}.png'))
    print(f'rendu/pet/{acc}.webp complété ; apercu-{acc}-{n}.png : la mienne, la sienne, la mienne habillée')
    if vides:
        print(f'  sans accessoire trouvé : poses {vides}')
    if redessinees:
        print(f'  plus de la moitié du calque ôtée (il a redessiné la pose) : poses {redessinees}, à regarder dans l\'aperçu')


def places(p, r, i):
    """Où la pose va dans un calque : sa rangée, et la case d'attente du pet est aussi la case 0
    de la forge."""
    rangee = r if p == 0 else len(RANGEES_PET) + r
    return [(i, rangee)] + ([(0, len(RANGEES_PET))] if (p, r, i) == (0, 6, 0) else [])


def epurer_calques():
    """Les calques déjà faits, épurés comme à l'import ; et tools/pet/chatgpt/epure-<accessoire>.png,
    chaque pose habillée avant (à gauche) et après."""
    os.makedirs(SORTIE, exist_ok=True)
    ps = poses_tenue()
    for acc in ACCESSOIRES:
        sortie = os.path.join(PET, f'{acc}.webp')
        feuille = Image.open(sortie).convert('RGBA')
        if feuille.size != (8 * CASE[0], (len(RANGEES_PET) + len(gestes.RANGEES)) * CASE[1]):
            print(f'{acc} : pas encore dessiné')
            continue
        ap = Image.new('RGBA', (10 * 200, (len(ps) + 9) // 10 * 110), (40, 36, 40, 255))
        ote = []
        for k, (p, r, i) in enumerate(ps):
            mien = franc(case_de(p, r, i))
            for n, (cx, cy) in enumerate(places(p, r, i)):
                boite = (cx * CASE[0], cy * CASE[1], (cx + 1) * CASE[0], (cy + 1) * CASE[1])
                c = feuille.crop(boite)
                e = epurer(acc, mien, c)
                avant, apres = (int(np.count_nonzero(np.asarray(x)[..., 3])) for x in (c, e))
                if avant > 2 * apres:
                    ote.append(k + 1)
                feuille.paste((0, 0, 0, 0), boite)
                feuille.paste(e, boite)
                if n == 0:
                    for j, x in enumerate((c, e)):
                        habille = mien.copy()
                        habille.alpha_composite(x)
                        ap.alpha_composite(habille.resize((96, 104)), ((k % 10) * 200 + j * 100, (k // 10) * 110 + 3))
        feuille.save(sortie, lossless=True, quality=100, method=6)
        ap.save(os.path.join(SORTIE, f'epure-{acc}.png'))
        print(f'rendu/pet/{acc}.webp épuré ; epure-{acc}.png : avant, après'
              + (f' ; plus de la moitié ôtée : poses {sorted(set(ote))}' if ote else ''))


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


def sans_sol(im, epais=12, long=40):
    """Sans les traits de sol qu'il tire sous une rangée (Gemini le fait) : un magenta plus sombre
    que le fond, que le détourage laisse, et ce qui reste plein sur au plus `epais` pixels de haut
    et sur plus de `long` de large. Ils souderaient les poses."""
    px = np.array(im)
    rose = np.minimum(px[..., 0], px[..., 2]).astype(np.int32) - px[..., 1] > 40
    px[rose] = 0
    im = Image.fromarray(px, 'RGBA')
    a = np.asarray(im.getchannel('A')) > 96
    h, w = a.shape
    # la hauteur pleine autour de chaque pixel : ce qui est plein au-dessus et en dessous
    haut = np.zeros((h, w), np.int32)
    bas = np.zeros((h, w), np.int32)
    for y in range(h):
        haut[y] = np.where(a[y], (haut[y - 1] if y else 0) + 1, 0)
    for y in range(h - 1, -1, -1):
        bas[y] = np.where(a[y], (bas[y + 1] if y < h - 1 else 0) + 1, 0)
    mince = a & (haut + bas - 1 <= epais)
    trait = np.zeros_like(mince)
    for y in range(h):
        x, ligne = 0, mince[y]
        while x < w:
            if ligne[x]:
                f = x
                while f < w and ligne[f]:
                    f += 1
                if f - x > long:
                    trait[y, x:f] = True
                x = f
            else:
                x += 1
    px = np.array(im)
    px[trait] = 0
    print(f'fond teinté retiré : {int(rose.sum())} pixels, traits de sol : {int(trait.sum())}')
    return Image.fromarray(px, 'RGBA')


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
    siennes = poses(sans_sol(detourer(Image.open(chemin))))
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
    if '--epurer' in sys.argv:
        epurer_calques()
    elif '--tenue' in sys.argv:
        args = [a for a in sys.argv[1:] if a != '--tenue']
        guide_tenue() if not args else importer_tenue(args[0], int(args[1]), args[2])
    elif '--guide' in sys.argv:
        guide()
    elif len(sys.argv) > 1:
        importer(sys.argv[1])
    else:
        print(__doc__)
