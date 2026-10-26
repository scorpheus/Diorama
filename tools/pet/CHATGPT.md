# Faire redessiner le pet par ChatGPT ou Gemini

Deux chantiers, dans cet ordre :

1. **Les gestes de la forge** (marteau, soufflet, trempe, envoi, balai, sommeil). Je les ai
   dessinés à la main dans `gestes.py`, mais ils sont plus raides que le pet d'origine. Une
   génération peut les refaire tous : on donne mes 21 poses comme guide, l'IA les redessine dans
   le style du personnage, et `chatgpt.py` les remet à leur place dans `rendu/pet/forge.webp`.
2. **Les accessoires** (lunettes, bandeau, tablier) : on demande à l'IA d'ajouter l'accessoire sur
   les poses telles qu'elles sont. `chatgpt.py` ne garde que ce qui a changé, l'accessoire, et en
   fait un calque (`rendu/pet/lunettes.webp`, etc.) que l'appli pose sur le pet. Le dessin
   d'origine reste intact.

Les accessoires se font **après** les gestes : leur calque est découpé sur les poses du moment.
Si les gestes sont refaits ensuite, il faut refaire les planches de tenue 3 et 4, qui les
contiennent.

Gemini sait bien modifier une image sans toucher au reste, et tu as du crédit : il convient bien
aux accessoires. Le reste marche avec l'un comme avec l'autre. Dans les deux cas, il faut
télécharger l'image en pleine taille (le bouton de téléchargement), pas une capture d'écran.

Toutes les commandes se lancent à la racine du dépôt.

---

## 1. Les gestes

### Préparer les images

```
python tools/pet/chatgpt.py --guide
```

Cela écrit dans `tools/pet/chatgpt/`, un dossier ignoré par git :
- `guide.png` : les 21 poses, 6 par rangée, sur un fond magenta ;
- `reference.png` : le personnage d'origine, en grand.

### Envoyer les deux images, avec ce texte

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

### Reprendre ce qu'il rend

```
python tools/pet/chatgpt.py chemin\vers\son-image.png
```

Le script :
- retire le fond (le magenta, un damier, ou rien s'il a rendu un vrai fond transparent) ;
- découpe les poses rangée par rangée et vérifie qu'il y en a 21. Sinon il refuse et n'écrit rien ;
- met chacune à la taille et à la place de la mienne : même hauteur, les pattes au même endroit ;
- écrit `rendu/pet/forge.webp`, avec `tools/pet/chatgpt/apercu.png` pour comparer (les miennes
  en haut, les siennes en bas).

Pour revenir à mes dessins : `python tools/pet/gestes.py`, ou `git checkout rendu/pet/forge.webp`.

---

## 2. Les accessoires

### Préparer les planches

```
python tools/pet/chatgpt.py --tenue
```

Cela écrit `tenue-1.png` à `tenue-4.png` dans `tools/pet/chatgpt/`. Ce sont les 78 poses que
l'appli peint, 20 par planche (18 sur la dernière), 5 par rangée, sur du magenta :
- planches 1 à 3 : le pet de Codex (repos, marche, salut, saut, échec, attente, travail,
  relecture) ;
- fin de la planche 3 et planche 4 : les gestes de la forge.

Il faut **une demande par accessoire et par planche**, soit 12 en tout. Pour qu'un accessoire
garde le même dessin d'une planche à l'autre, fais ses quatre planches dans la même conversation,
l'une après l'autre. Recommence une conversation pour l'accessoire suivant.

### Le texte à envoyer avec chaque planche

Le texte commun, où `[ACCESSOIRE]` se remplace par l'un des trois paragraphes plus bas :

```
Edit this image. It is a sprite sheet of one pixel-art character, a chibi white wolf blacksmith
with a blue crest of hair and a green hooded tunic, in many poses on a flat magenta background,
5 poses per row.

Add the accessory described below to the character in EVERY pose, and change nothing else:
- every pose stays exactly where it is, at the same size, with the same outline, colours,
  shading and pixel-art style; do not redraw, move, crop or re-centre anything;
- the background stays one flat solid magenta (#FF00FF); no text, no numbers, no grid, no
  shadow, no border;
- the image keeps the same width and height ratio.

Draw the accessory in the same pixel-art style as the character, with the same dark outline,
the light coming from the upper left. It follows each pose: when the head turns, tilts or bows,
the accessory turns with it; when part of it would be hidden (behind the head, an arm, the
hood), leave that part out.

The accessory: [ACCESSOIRE]
```

**Lunettes** :

```
blacksmith goggles pushed up on the forehead, just below the blue crest of hair and above the
eyes: two round lenses of dark teal glass with a small white glint, in brass rims, joined by a
short brass bridge, and a dark brown leather strap going around the head.
```

**Bandeau** :

```
a cloth headband tied around the forehead, just below the blue crest of hair and above the eyes,
made of exactly the same olive-green cloth as his hood (same greens, same shading), with a small
knot and two short loose tails on one side of the head.
```

Le bandeau doit être du même vert que la capuche : l'appli lui donne alors la couleur de la cape
de la session, comme à la capuche.

**Tablier** :

```
a blacksmith's leather apron worn over the green tunic: medium-brown leather with darker edges
and a few light stitches, covering the chest and belly down to just above the knees, held by a
thin strap around the neck. His belt, pouches, rope, scroll and flask stay visible on top of the
apron, and his arms stay in front of it.
```

### Reprendre chaque planche

```
python tools/pet/chatgpt.py --tenue lunettes 1 chemin\vers\son-image.png
```

L'accessoire est `lunettes`, `bandeau` ou `tablier`, puis vient le numéro de la planche (1 à 4).
Le script :
- vérifie que l'image a les proportions de la planche envoyée, sinon il refuse : l'IA a changé
  la mise en page ;
- la ramène à la taille de la planche, recale chaque pose sur la mienne (à 12 pixels près), et ne
  garde que ce qui a changé, en taches de plus de deux pixels ;
- épure ce calque jusqu'à l'accessoire seul. Quand l'IA redessine une pose au lieu de l'habiller
  (la tête redressée, la queue ailleurs, une gourde ou un manche de balai ajoutés), tout ce qui
  diffère passe dans le calque, et l'appli montrerait le pet en double. Sont ôtés : la fourrure
  (blanc, gris clair, bleu de la crête et de la queue), les miettes, et tout ce qui n'est pas la
  pièce de la tête (lunettes, bandeau, au-dessus du menton) ou le cuir brun du corps (tablier) ;
- complète `rendu/pet/<accessoire>.webp`, planche par planche (les cases des autres planches sont
  gardées), et écrit `tools/pet/chatgpt/apercu-<accessoire>-<n>.png`. Chaque pose y apparaît
  trois fois : la mienne, la sienne, puis la mienne avec le calque ;
- dit, pour chaque pose, de combien il l'a recalée, combien de pixels compte son calque et
  combien en sont gardés ; liste les poses où il n'a rien trouvé, et celles dont plus de la moitié
  a été ôtée (il les a redessinées : les regarder dans l'aperçu).

Les calques déjà faits se repassent à l'épuration, sans rien redemander à l'IA :

```
python tools/pet/chatgpt.py --epurer
```

Elle ne fait qu'ôter, et la repasser ne change plus rien. `tools/pet/chatgpt/epure-<accessoire>.png`
montre chaque pose habillée avant (à gauche) et après.

Une planche ratée se refait seule : relancer la commande avec une meilleure image remplace ses
poses. Pour retirer un accessoire : `git checkout rendu/pet/<accessoire>.webp`, qui remet le
calque vide (une image d'un pixel).

---

## 3. Regarder

Les planches et les calques sont compilés dans l'appli. Pour les voir jouer, moment par moment :

```
cd rendu
cargo run --release -- --vitrine
```

Les touches :
- ← → : changer de moment ;
- espace : pause ;
- R : reprendre le moment depuis le début ;
- C : changer la cape (forêt, braise, nuit, prune, ocre, ardoise) ;
- A : changer l'accessoire.

Pour les mettre dans l'appli du bandeau : `rendu\rendu.ps1 -Lancer`. Il recompile si `pet/` a
changé, puis remplace l'appli qui tourne.

## Ce qui peut clocher

- **Gestes : il a fusionné deux poses, ou en a ajouté une.** Le compte n'est pas bon et le script
  le dit. Lui demander de séparer davantage les poses, ou corriger l'image à la main.
- **Gestes : un poing est loin de là où le guide le met.** La forge dessine le levier, la lame, le
  sceau et le balai à la place de mes poings : sa patte tombera à côté. Le repérer dans la
  vitrine.
- **Gestes : la taille des poses varie.** Le script prend une seule échelle, la médiane : une pose
  dessinée plus grande ou plus petite le restera.
- **Accessoires : il a retouché autre chose** (les yeux, une couleur, un contour). L'épuration ôte
  la fourrure et ce qui n'est pas l'accessoire ; ce qui resterait se voit dans l'aperçu, en
  troisième image. Refaire la planche en insistant sur « change nothing else ».
- **Accessoires : l'accessoire est ôté avec le reste.** Un tablier d'une autre couleur que le cuir
  brun moyen du prompt, ou un accessoire de tête plus bas que le menton, ne passe pas l'épuration
  (`cuir` et `TETE` dans `chatgpt.py`) : les ajuster, puis relancer `--epurer`.
- **Accessoires : il a déplacé ou redimensionné les poses.** Le recalage rattrape 12 pixels.
  Au-delà, le calque prend des morceaux de corps décalés : refaire la planche.
- **Accessoires : un calque vide** (« sans accessoire trouvé ») : l'accessoire est caché dans
  cette pose, ou l'IA l'a oublié.
