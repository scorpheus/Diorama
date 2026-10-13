"""Les pièces dessinées à la main, pixel par pixel, une lettre par pixel ('.' rien).
Lumière en haut à gauche (le foyer). Dessinées manche vers le haut ; tournées d'un quart de tour
sans perte pour les autres positions."""

# La tête du marteau, en travers, l'œil (où le manche entre) en bas au milieu.
# o contour · C reflet · c arête claire · M face · m corps · S ombre · s ombre profonde · k cuivre
TETE = [
    "....ooooooooooooooooooooooo....",
    "..ooCCCcccccccckkkcccccccMMoo..",
    ".oCCccccccccccckkkccccccccMMmo.",
    ".oCcccccccccccccccccccccccMMmo.",
    "oCcMMMMMMMMMMMMMMMMMMMMMMMMMmmo",
    "ocMMmmkkmmmmmmmmmmmmmmmkkmmMmSo",
    "ocMmmmkkmmmMmmmmmmmmmmmkkmmmmSo",
    "ocMmmmkkmmmmmmmmmmmmMmmkkmmmmSo",
    "ocMmmmkkmmmmmmmmmmmmmmmkkmmmmSo",
    "ocMmmmkkmmmmmmmmmmmmmmmkkmmmmSo",
    "ocMMmmkkmmmmmmMmmmmmmmmkkmmmmSo",
    "ocMmmmkkmmmmmmmmmmmmmmmkkmmmmSo",
    "ocMmmmkkmmmmmmmmmmmmmmmkkmmmmSo",
    "oMmSSSkkSSSSSSSSSSSSSSSkkSSSmSo",
    "oMSssssssssssssssssssssssssssSo",
    "oMSssssssssssssssssssssssssssSo",
    ".oSsssssssssssssssssssssssssso.",
    "..oosssssssssssssssssssssssoo..",
    "....ooooooooooooooooooooooo....",
]

# Le poing serré sur un manche vertical (colonnes 8 à 11) : le pouce par-dessus en haut à gauche,
# trois doigts repliés en bourrelets, l'ombre en bas à droite.
# o contour · C reflet · c clair · M fourrure · m ombre · S ombre profonde · s sillon
POING = [
    ".....oooooooooooooo.....",
    "...ooCCccccMMMMMMMMoo...",
    "..oCcccMMMMMMMMMMMMMMo..",
    ".oCcMMMMooooooMMMMMMmmo.",
    ".ocMMMMoCCcccMoMMMMMmmo.",
    "ocMMMMMocMMMMMoMMMMMmmmo",
    "ocMMMMMMoooooooMMMMmmmmo",
    "oMcMMMMMMMMMMMMMMMMmmmmo",
    "oMMssssssssssssssMMmmmmo",
    "ocMcMMMMMMMMMMMMMMmmmmSo",
    "oMMMMMMMMMMMMMMMMmmmmmSo",
    "oMMssssssssssssssMmmmmSo",
    "ocMcMMMMMMMMMMMMMmmmmSSo",
    "oMMMMMMMMMMMMMMMmmmmmSSo",
    ".oMssssssssssssMmmmmSSo.",
    ".oMcMMMMMMMMMMMmmmmmSSo.",
    "..oMmmmmmmmmmmmmmmmSSo..",
    "...oommmmmmmmmmmmSSoo...",
    ".....oooooooooooooo.....",
]

# La touffe blanche de l'épaule, au bord de la cape.
EPAULE = [
    "..ooooo..",
    ".oCCccmo.",
    "oCCcccmmo",
    "oCccccmso",
    "occccmmso",
    ".occmmso.",
    "..omosmo.",
    "...o.o...",
]


def tourner(carte, quarts):
    """La carte tournée de `quarts` quarts de tour dans le sens inverse des aiguilles."""
    for _ in range(quarts % 4):
        h, w = len(carte), len(carte[0])
        carte = [''.join(carte[j][w - 1 - i] for j in range(h)) for i in range(w)]
    return carte


if __name__ == '__main__':
    for nom, c in (('TETE', TETE), ('POING', POING), ('EPAULE', EPAULE)):
        print(nom, set(len(l) for l in c), len(c))
