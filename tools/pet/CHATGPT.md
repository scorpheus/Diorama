# Faire redessiner les gestes par ChatGPT

Mes gestes (`gestes.py`) tiennent la route mais sont plus raides que le pet d'origine. Une seule
génération ChatGPT peut les refaire toutes : on lui donne mes 21 poses comme guide, il les
redessine dans le style du personnage, et `chatgpt.py` les remet à leur place dans
`rendu/pet/forge.webp`.

## 1. Préparer les deux images

```
python tools/pet/chatgpt.py --guide
```

Cela écrit dans `tools/pet/chatgpt/`, un dossier ignoré par git :
- `guide.png` : les 21 poses, 6 par rangée, sur un fond magenta ;
- `reference.png` : le personnage d'origine, en grand.

## 2. Envoyer les deux images à ChatGPT, avec ce texte

```
Attached: (1) reference.png, the original character, a chibi white wolf blacksmith in pixel art;
(2) guide.png, 21 poses of the same character on a flat magenta background, laid out 6 per row
(rows of 6, 6, 6 and 3).

Redraw guide.png as a polished pixel-art sprite sheet of this exact character, in the exact style,
palette, proportions and level of detail of reference.png. Keep everything else identical to the
guide:
- the same 21 poses, in the same order and the same grid (6, 6, 6, then 3), each pose clearly
  separated from its neighbours by empty magenta space;
- the same body size in every pose; the feet on the same ground line in each row;
- the arm positions of the guide: each closed fist exactly where the guide puts it. The guide's
  arms are crude: redraw them properly, with a dark-furred forearm coming out of the green cape
  and a well-drawn fist;
- the hands hold nothing, except the hammer in poses 1 to 3 (keep that hammer). The lever, the
  blade, the crate, the seal stamp and the broom are drawn by the game around the character,
  so do NOT draw them;
- poses 20 and 21: seated, asleep, eyes closed, the head drooping a little lower in pose 21.

Background: one flat solid magenta (#FF00FF) everywhere. No checkerboard, no shadow on the
ground, no text, no numbers, no grid lines, no border.

The poses, in order:
1 hammer raised above the shoulder; 2 hammer swung forward, horizontal; 3 hammer striking down
low; 4 left fist far to the left at chest height, pulling a lever; 5 same, fist pressed lower;
6 fist raised at the shoulder, holding a sword hilt (blade not drawn); 7 arm reaching down-left,
dipping a sword into a bucket; 8 to 11 fist to the left at chest height, lowered a little in each
pose, laying a blade into a crate; 12 fist to the left, closing a lid; 13 arm raised high to the
upper left, holding a stamp; 14 arm extended left, stamping down; 15 to 19 both fists to the
right side, holding a broom handle, the arms pushing a bit further right from pose to pose;
20 seated on a stool, asleep, a paw on the knee; 21 the same, head nodding lower.
```

## 3. Reprendre ce qu'il rend

Télécharger son image, puis :

```
python tools/pet/chatgpt.py chemin\vers\son-image.png
```

Le script :
- retire le fond (le magenta, un damier, ou rien s'il a rendu un vrai fond transparent) ;
- découpe les poses rangée par rangée et vérifie qu'il y en a 21. Sinon il refuse et n'écrit rien ;
- met chacune à la taille et à la place de la mienne : même hauteur, les pattes au même endroit ;
- écrit `rendu/pet/forge.webp`, avec `tools/pet/chatgpt/apercu.png` pour comparer (les miennes
  en haut, les siennes en bas).

## 4. Regarder

La planche est compilée dans l'appli. Pour la voir jouer, moment par moment (← → pour changer de
moment, espace pour la pause) :

```
cd rendu
cargo run --release -- --vitrine
```

Pour la mettre dans l'appli du bandeau : `rendu\rendu.ps1 -Lancer`. Il recompile si `pet/` a
changé, puis remplace l'appli qui tourne.

Pour revenir à mes dessins : `python tools/pet/gestes.py`, ou `git checkout rendu/pet/forge.webp`.

## Ce qui peut clocher

- **Il a fusionné deux poses, ou en a ajouté une.** Le compte n'est pas bon et le script le dit.
  Lui demander de séparer davantage les poses, ou corriger l'image à la main.
- **Un poing est loin de là où le guide le met.** La forge dessine le levier, la lame, le sceau
  et le balai à la place de mes poings : sa patte tombera à côté. Le repérer dans la vitrine.
- **La taille des poses varie.** Le script prend une seule échelle, la médiane : une pose dessinée
  plus grande ou plus petite le restera.
