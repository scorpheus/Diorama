# Écrire un thème Diorama

Un thème, c'est un seul fichier `theme.json`. Il dit à quoi ressemblent la scène et son personnage, et avec quels mots le bandeau raconte ce qui se passe. Le moteur fournit tout le reste : il suit le travail de Claude, il anime le personnage et il ajoute les bulles, les aides et les visites des autres sessions.

Le plus simple est de laisser Claude le dessiner avec la compétence `/diorama:nouveau-theme`. Ce document décrit le format, pour le lire ou pour le retoucher à la main.

- **Brancher un thème :** `/config`, ligne *Diorama · Thème*. On y met le chemin **absolu** du fichier, par exemple `C:/Users/moi/.claude/diorama/themes/pirate/theme.json`, ou le nom d'un thème livré (`forge`, `sorciere`).
- **Voir ses changements :** dès que le fichier change, le bandeau se redessine en deux secondes environ.
- **Si le thème est refusé :** le bandeau reprend la forge, et le journal sous les jauges dit pourquoi (`thème « … » refusé : …`).
- **Aperçu sans lancer de session :** `node tools/apercu.mts theme.json apercu.png` vérifie le fichier et dessine une planche de vingt moments (il faut Node 22.6 ou plus).

## Les dessins

Un dessin est une liste de lignes de texte, **une lettre par pixel**, de haut en bas. Le point `.` est transparent.

Chaque lettre désigne une couleur de la palette. Le terminal affiche deux pixels par case (`▀` / `▄`). Une scène de 32 pixels de haut occupe donc 16 lignes de terminal.

```json
"idle": [[
  "..kkkk..",
  ".kwwwwk.",
  ".kwkwkk."
]]
```

Une pose est une liste d'images, jouées en boucle. Une image dure `every` tics, et un tic dure 150 ms.

## La palette

La palette par défaut est toujours disponible. Elle est définie dans `hooks/props.ts`, avec un commentaire par lettre. Les bulles, les rêves, les étincelles et le ciel s'en servent. Un thème peut **ajouter** des lettres ou en **changer** :

```json
"palette": { "6": "#f2c7a0", "%": "#4d4170" }
```

Une clé de la palette est un seul caractère, n'importe lequel sauf le point. Les chiffres `6` à `9` et les signes `% & + = * ~ @ #` sont libres.

Voici les lettres par défaut les plus utiles :

| lettre | couleur |
|---|---|
| `k` | contour quasi noir |
| `w` | blanc |
| `l` `g` `d` | gris clair, moyen, sombre |
| `y` | or |
| `r` `R` | bois, bois sombre |
| `a` `A` | acier, acier clair |
| `q` `Q` | pierre |
| `F` `f` `Y` `E` | flamme, flamme vive, chaleur blanche, braise |
| `c` | eau, ciel |
| `G` `j` | vert, vert clair |
| `P` `i` | violet, violet clair |
| `M` | rose |
| `b` `B` `n` | bleu clair, bleu, bleu nuit |
| `o` `O` | vert sombre, vert : la couleur que la session change |
| `C` `D`, `N` `L`, `U` `I`, `T` `J`, `Z` `v` | les autres tenues : braise, nuit, prune, ocre, ardoise |

**Sur un terminal sombre, le noir pur disparaît.** Un contour `k` autour d'une forme sombre la fait ressortir.

## Le fichier, champ par champ

Seuls `diorama`, `name`, `width` et `character` (avec sa pose `idle`) sont obligatoires.

```jsonc
{
  "diorama": 1,                       // version du format
  "name": "L'antre de la sorcière",
  "width": 72,                        // pixels de large (24 à 120)
  "height": 32,                       // pixels de haut, pair, 8 à 32 (32 par défaut)
  "crops": {                          // cadrages, du plus large au plus étroit
    "full": [0, 72],                  // le bandeau prend le plus large qui tient
    "compact": [6, 72],
    "solo": [6, 50]
  },
  "palette": { "6": "#f2c7a0" },
  "sky": { "at": [2, 1], "size": [10, 8], "shutters": false },
  "background": ["…"],                // décor fixe, dessiné par-dessus le ciel
  "foreground": ["…"],                // dessiné par-dessus le personnage
  "character": { … },
  "accessories": { … },
  "variants": { … },
  "stage": { … },
  "effects": [ … ],
  "gauges": [ … ],
  "trophies": { … },
  "helpers": { … },
  "pet": { … },
  "messenger": { "fly": [ … ] },
  "gift": { "at": [64, 28] },
  "feast": { "at": [36, 26], "draw": ["…"] },
  "names": ["Morgane", "Viviane"],
  "helperNames": ["Crapoti", "Rainette"],
  "words": { … }
}
```

Les positions s'écrivent `[x, y]` en pixels, à partir du coin haut gauche de la scène.

### `sky` : le ciel

Le ciel remplit un rectangle, peint **sous** le décor. Laissez donc des `.` dans `background` à l'endroit des carreaux.

Il change avec l'heure : soleil, crépuscule, lune et étoiles. Il suit aussi la saison (neige, feuilles, pétales) et la série en cours : orage après des échecs, arc-en-ciel deux minutes après trois réussites. De temps en temps y passent une étoile filante ou un dragon.

Avec `"shutters": true`, des volets ferment le ciel de 23 h à 6 h.

### `character` : le personnage

```jsonc
"character": {
  "home": [22, 6],            // coin haut gauche du personnage à sa place
  "every": 5,                 // tics par image (4 par défaut)
  "mirror": false,            // true : les images de marche sont retournées pour aller à gauche
  "recolor": ["o", "O"],      // lettres [sombre, claire] remplacées par la tenue de la session
  "bubble": [16, -6],         // où vont les bulles, depuis home (par défaut : à droite de la tête)
  "poses": {
    "idle": [ [ "…" ], [ "…" ] ],                    // obligatoire
    "walk": [ … ],                                   // une image par pixel parcouru
    "running": { "frames": [ … ], "every": 2, "at": [-6, 0] }
  }
}
```

Une pose s'écrit soit comme une liste d'images, soit comme `{ "frames", "every", "at" }`. Le décalage `at` permet à une pose de déborder de la place du personnage, par exemple un bras qui touille un chaudron à sa gauche.

Voici les poses, et ce qu'affiche une pose absente :

| pose | quand | sinon |
|---|---|---|
| `idle` | au repos | — |
| `walk` | en marche | l'image de la pose en cours |
| `thinking` | le modèle réfléchit | `idle` |
| `running` | un outil travaille (écriture, commande, compilation) | `idle` |
| `review` | lecture, recherche | `thinking` |
| `waiting` | Claude attend la personne | `idle` |
| `failed` | un outil a échoué | `idle` |
| `jumping` | tour réussi, niveau gagné | `waving` |
| `waving` | arrivée, accueil d'une visite | `jumping` |
| `quench` | des tests viennent de finir | `running` |
| `ship` | commit ou push | `running` |
| `sweep` | compactage du contexte | `running` |
| `sleep` | sieste après dix minutes de calme | `idle` |
| `waking` | réveil par un message | `waving` |

Le moteur ajoute lui-même plusieurs éléments par-dessus les poses :
- les bulles : `!` quand Claude attend, la parole, et le nuage de réflexion ;
- pendant la sieste, les `z` et la bulle du rêve ;
- la sueur quand les jauges dépassent 85 % ;
- la poussière d'un échec, et le feu d'artifice d'une réussite.

Laissez au moins 6 pixels libres au-dessus de la tête pour les bulles.

### `accessories`, `variants` : la tenue de chaque session

Chaque session porte une tenue. Haiku la choisit d'après le sujet de la conversation, et deux sessions ouvertes ne portent jamais la même. Une tenue se compose de deux choses :

- **une variante de couleurs**, qui remplace les deux lettres de `recolor`. Sans `variants`, ce sont les six tenues par défaut (forêt, braise, nuit, prune, ocre, ardoise) :

  ```json
  "variants": { "rouge": { "colors": ["C", "D"], "meaning": "urgence, correction pressée" } }
  ```

  `meaning` dit à Haiku pour quel sujet porter cette tenue.

- **un accessoire**, collé sur le personnage. Sa position est relative au coin du personnage :

  ```json
  "accessories": { "lunettes": { "at": [4, 9], "draw": ["AkA..AkA"] } }
  ```

### `stage` : où il va

Ce sont des pixels, comptés depuis `home` ; le négatif est à gauche.

```jsonc
"stage": {
  "min": -12, "max": 14,        // jusqu'où il se promène
  "idle": [-12, -6, 0, 6, 14],  // ses endroits favoris au repos
  "work": 0,                    // où il travaille
  "build": 0,                   // où il compile
  "offstage": 52,               // hors de la scène, quand il part en visite
  "guests": [32, 20]            // où se tiennent les visiteurs
}
```

### `effects` : ce qui bouge dans le décor

```jsonc
"effects": [
  { "type": "fire",   "at": [11, 29], "size": [8, 3], "colors": ["f", "F", "E", "E"] },
  { "type": "smoke",  "at": [14, 20], "colors": ["j", "G"] },
  { "type": "sparks", "at": [15, 21] },
  { "type": "glow",   "at": [35, 1],  "colors": ["Y", "f"], "night": true }
]
```

- **`fire`** brûle plus fort au travail, encore plus en compilation et avec un effort élevé, puis retombe en braises pendant la sieste. Ses couleurs vont du cœur vers la pointe.
- **`smoke`** monte, plus dense au travail.
- **`sparks`** jaillit au travail. Sa couleur dépend du genre de travail : or pour l'écriture, vert pour les tests, bleu pour les commandes, violet pour les sous-agents…
- **`glow`** est un pixel qui vacille. Avec `"night": true`, il ne s'allume que le soir.

### `gauges` : les jauges dans le décor

Des pixels apparaissent ou disparaissent selon une jauge :

```json
"gauges": [ { "of": "5h", "points": [[0, 31, "R"], [1, 31, "r"]] } ]
```

| `of` | ce que montrent les points |
|---|---|
| `ctx` | le contexte qui se remplit : un point de plus à chaque cran |
| `5h` | ce qui reste de la fenêtre de 5 h : un point de moins à chaque cran |
| `7d` | ce qui reste de la fenêtre de 7 jours |
| `cost` | le coût de la session, sur une échelle logarithmique |

Les points apparaissent dans l'ordre de la liste.

### `trophies` : une trace par jour travaillé

Un emplacement par jour, sept au plus. Le dessin dépend de la journée :

```jsonc
"trophies": {
  "slots": [[39, 8], [42, 8], [45, 8]],
  "draw": {
    "tests":  ["…"],   // jour de tests
    "builds": ["…"],   // jour de compilations
    "short":  ["…"],   // journée courte
    "other":  ["…"]    // le reste
  },
  "honour": { "at": [61, 8], "draw": ["…"] }   // le chef-d'œuvre : un long tour fini par des tests verts
}
```

### `helpers` : les aides (les sous-agents)

Chaque sous-agent lancé par Claude devient une petite créature qui va et vient dans la zone, travaille un moment, puis repart quand sa mission est finie.

```jsonc
"helpers": {
  "walk": [ … ],            // images de marche, tournées vers la droite
  "work": [ … ],            // images au travail (facultatives)
  "area": [50, 71, 31],     // x de début, x de fin, y des pieds
  "recolor": "C"            // lettre remplacée par la couleur du type d'agent
}
```

### `pet` : l'animal de la machine

Un seul animal pour toutes les sessions ouvertes sur la machine. Il passe d'une session à l'autre et s'enfuit quand le feu ronfle trop fort. Sans `pet`, le thème n'en a pas.

```jsonc
"pet": {
  "walk": [ … ],            // tourné vers la gauche (sinon "facing": "right")
  "stand": ["…"],
  "sleep": ["…"],           // endormi sur son perchoir
  "perch": [3, 5],
  "floor": 31,              // y de ses pattes
  "gate": 4                 // x où il saute du perchoir
}
```

### `messenger`, `gift`, `feast`

- **`messenger.fly`** : l'oiseau qui apporte les nouvelles des autres sessions (tâche finie, personne attendue). Il est tourné vers la gauche. Par défaut, c'est un corbeau.
- **`gift.at`** : l'endroit où un visiteur laisse un petit cadeau.
- **`feast`** : la table de fête, quand trois sessions ou plus sont au calme en même temps.

### `words` : les mots du thème

Les mots qui portent l'image du thème se changent un par un. Ceux qu'on ne change pas restent neutres (fichier `hooks/themes/neutral-words.ts`), et le français ordinaire (« réfléchit », « erreur API ») reste au moteur. Les marques `{n}`, `{who}`, `{cmd}`… sont remplies par le moteur.

```jsonc
"words": {
  "persona": "une sorcière qui touille ses potions de code",    // pour Haiku
  "helper": "grenouille apprentie",                              // pour Haiku, qui nomme les aides
  "placeOf": "l'antre {project}",
  "lines":  { "answer": ["Abracadabra !"], "push": ["Vole, hibou !"] },  // les répliques, par moment
  "labels": { "running": "touille le chaudron" },                // le mot sous le nom, par moment
  "atWork": { "build": "attise le feu", "test": "goûte la potion" },
  "moods":  { "cold": "à court de bois" },
  "details": { "pass": "la potion est bonne" },
  "notes":  { "newsDone": "hibou : {who} a fini sa tâche" },     // les lignes du journal
  "teaToast": "deux heures de sorts, une infusion ?",
  "today":  { "tests": "{n} potions goûtées" },
  "helpers": "grenouilles {n}"
}
```

La liste complète des clés, avec leurs valeurs neutres, est dans `hooks/themes/neutral-words.ts`. La forge, qui est le thème de référence, a les siennes dans `hooks/themes/forge-words.ts`.

## Ce que le moteur vérifie

Avant d'afficher un thème, le moteur le vérifie et le refuse au besoin. Il contrôle les points suivants :
- chaque lettre de chaque dessin existe dans la palette ;
- les positions tombent dans la scène ;
- les poses portent des noms connus ;
- la pose de repos peint au moins 20 cases : un personnage hors du cadre est refusé, plutôt que d'afficher une scène vide.

Chaque erreur dit où elle se trouve, par exemple :

```
character.poses.idle[0], ligne 3 : la lettre « Q » n'est pas dans la palette
```

## Partager un thème

Ouvrez une demande de fusion sur [scorpheus/Diorama](https://github.com/scorpheus/Diorama) avec un dossier `themes/<nom>/theme.json`. Ce thème se choisit ensuite par son nom seul.
