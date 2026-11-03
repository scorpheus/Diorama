# Les habitants des onglets de ClaudeTerminal

Un personnage par profil, dans le style du Diorama, que la barre d'onglets de ClaudeTerminal pose et anime selon
l'état de la session. Ce fichier est le contrat entre les deux (copie dans `D:\KT\.claude\projets\onglets-habitants.md`).

- **Où** : dépôt Diorama, dossier `habitants/`. Un fichier par personnage, `habitants/<nom>.json` (UTF-8). Les
  accessoires (pigeon, toiles, bougie, bourse, météo, mineurs, bulle) tous dans `habitants/accessoires.json` (plus bas).
- **Format** : le dessin à lettres des thèmes du Diorama (THEMES.md), une lettre par pixel, `.` transparent.
  ```json
  { "nom": "marin", "description": "…", "valide": false, "taille": [16, 16],
    "palette": { "k": "#1b1a24", "O": "#3f80dc", "o": "#1f4f9e", … },
    "etats": { "travail": { "every": 2, "images": [ [ "16 lettres", … 16 lignes ], … ] }, … } }
  ```
  - `taille` : `[largeur, hauteur]`, **16 × 16** pour tout habitant ; toutes ses images ont cette taille.
  - `valide` : `true` une fois le dessin validé par le user. Tant qu'il vaut `false`, ClaudeTerminal garde le
    voyageur (il peut montrer l'habitant en aperçu, pas l'installer).
  - `palette` : complète et propre au fichier (aucune palette par défaut à connaître), `"#rrggbb"`.
  - **`O` et `o` sont la couleur du profil** et son ombre : le lecteur remplace `O` par la couleur de l'onglet et `o`
    par la même × 0,62 (chaque canal, arrondi). Les valeurs du fichier sont celles de l'aperçu.
  - `etats` : nom → `{ every, images }` ; une image dure `every` tics de **150 ms**, les images tournent en boucle.
- **États obligatoires** (tout habitant les a ; le lecteur ignore un état qu'il ne connaît pas) :

  | état | quand | ce qu'on voit |
  |---|---|---|
  | `travail` | la session travaille | à son ouvrage : clavier (marin, voyageur, claudeterminal), enclume, roue, pierre, orbe, volant |
  | `repos` | la session est au repos | dort, des Z montent |
  | `attente` | la session attend le user | agite un drapeau |
  | `echec` | erreur, échec | assis dans la fumée |
  | `parle` | la ligne « Dit : » est prononcée | 2 images : bouche fermée, ouverte ; ClaudeTerminal peut les alterner au rythme de la voix plutôt qu'à `every` |
- **État facultatif `marche`** (ClaudeTerminal, 02-11) : quand le personnage d'une session rend visite à une
  autre dans le Diorama, l'habitant de son onglet va jusqu'à l'onglet de l'hôte et en revient. S'il a un état
  `marche` (de face, un dandinement, images en boucle), ClaudeTerminal le joue en chemin ; sinon il prend
  l'image bouche fermée de `parle`, à petits sauts. Chez l'hôte, il joue `parle`.
- **Placement** : l'image se pose par le milieu de son bord bas (les pieds sur la dernière ligne). Agrandissement
  **entier** au plus proche (× 1, × 2…), choisi par ClaudeTerminal selon la hauteur de la barre, sans lissage. Ce qui
  accompagne l'état (Z, drapeau, fumée, clavier) est **dans** les images : le lecteur n'ajoute rien.
- **Voyageur** : `habitants/voyageur.json`, mêmes états, teint par `O`/`o` ; un profil sans habitant validé le prend,
  un fichier refusé aussi.
- **Vérifier** : `python tools/habitants/apercu.py habitants/<nom>.json planche.png [--anime anim.gif] [--teinte #rrggbb]`
  refuse un fichier faux (taille, lettre hors palette, état obligatoire absent, `every` non entier ≥ 1) et dessine
  la planche. Le lecteur de ClaudeTerminal refuse les mêmes fautes et prend alors le voyageur.
- **Profil → habitant** : à ClaudeTerminal (sa config ; par défaut, le nom du profil en minuscules sans accents).
  Les habitants, de face, trapus, à la manière des créatures de Dwarf Fortress, et pas forcément humains :
  `galadriel` (Orchestratrice, comme son pet du Diorama), `forgeron` (Endurance, un nain), `marin` (Diorama, Route du
  Rhum), `mecano` (vehigraph), `tailleur` (LaPierreDesMots, un golem de pierre), `europa` (Europa, une voiture de
  rallye), `claudeterminal` (ClaudeTerminal, une pieuvre qui tape de tous ses bras), `voyageur` (tous les autres, dont « Claude » : un petit
  marcheur des contes, capuche et cape à la couleur du profil, bâton ; il a remplacé l'escargot le 02-11). Tous dans `tools/habitants/gabarits.py` (le dessin debout, puis ce qui change d'un
  état à l'autre) : un nouvel habitant y est une entrée.
- **Accessoires** : `habitants/accessoires.json`, une palette pour tous (complète, `"#rrggbb"`, `O`/`o` à la couleur
  du profil comme chez les habitants), et chaque accessoire nommé :
  ```json
  { "nom": "accessoires", "valide": false, "palette": { … },
    "accessoires": { "pigeon": { "taille": [8, 6], "every": 1, "usage": "…", "niveaux": [ [ image, image, … ] ] }, … } }
  ```
  - `niveaux` : une liste de niveaux, chacun une liste d'images jouées en boucle à `every` tics de 150 ms. Un
    accessoire animé seul n'a qu'un niveau. Un accessoire qui suit une grandeur (la bougie, les toiles, la bourse) en a
    plusieurs, du plus bas au plus haut : le lecteur prend le niveau `min(n − 1, ⌊f × n⌋)` pour une fraction `f` de 0
    à 1, ou l'indice qu'il tient lui-même (les toiles : 0, 1, 2 selon ses seuils de repos).
  - Placement : par le milieu du bord bas, agrandi du même facteur entier que l'habitant. Les accessoires dessinés
    tournés vers la droite (`pigeon`, `mineur`, `mineur-lingot`) se retournent en miroir pour aller à gauche.
  - **Le vol du pigeon** reprend celui du passeur (`passeur/passeur.cs`, `Step`) : durée
    `min(2,2 ; max(1,0 ; 0,8 + distance / 3000))` s (distance en pixels d'écran) ; avec `t` de 0 à 1 et
    `e = 0,5 − 0,5 cos(π t)` (il part et se pose en douceur), `x = x0 + (x1 − x0) e`,
    `y = y0 + (y1 − y0) e − h · 4 e (1 − e)`, `h` la hauteur d'arc qui ne sort pas de la fenêtre ; opacité
    `0,92 × min(1, t / 0,08, (1 − t) / 0,1)`. Ses ailes battent à `every` (un tic, 150 ms) pendant le vol.
    `toiles` fait 16 × 16 et se pose **sur** l'habitant, au même point.
  - `usage` dit en une phrase quand le montrer ; la liste : `pigeon` (vol d'un onglet à l'autre), `toiles` (repos
    long : 0 poussière, 1 une toile, 2 toiles et araignée), `bougie` (contexte : niveaux du plus fondu au plus neuf,
    flamme qui vacille), `piece` (une pièce qui tourne en tombant), `bourse` (le coût : plate, ronde, pleine),
    `meteo-soleil`, `meteo-averse`, `meteo-orage`, `meteo-arc` (16 × 8, au-dessus de l'onglet), `mineur` (part au
    diagnostic de fond), `mineur-lingot` (revient avec le rapport), `bulle` (la ligne « Dit : », trois points qui
    s'animent).
- **File des demandes** (le paragraphe « Si aucune session Diorama n'est ouverte ») : `habitants/demandes/<nom>.json`,
  écrit par ClaudeTerminal ou l'orchestratrice, `{ "nom", "profil", "couleur": "#rrggbb", "sujet": "ce que fait le
  profil, en une phrase", "le": "AAAA-MM-JJ" }`. La session Diorama dessine `habitants/<nom>.json` avec `valide: false`,
  et ne supprime jamais la demande ; elle est servie quand l'habitant existe avec `valide: true`, et son auteur la
  retire alors. **En attente** = la demande existe et `habitants/<nom>.json` n'existe pas encore (dessiné mais pas
  validé : il attend le user, pas une session). `<nom>` : minuscules, sans accents ni espaces (`[a-z0-9-]+`), le même
  que l'habitant. Exemple : `habitants/demandes/vehigraph.json` →
  `{ "nom": "vehigraph", "profil": "vehigraph", "couleur": "#4a9e5c", "sujet": "outil de diagnostic véhicule", "le": "2026-11-03" }`.
  La file n'est pas suivie par git (`.gitignore`) : elle vit sur le disque de la machine, pas dans l'historique.
- **Fait** (30-10) : tous validés — `marin` (redessiné de face), `voyageur`, `galadriel`, `forgeron`, `mecano`,
  `tailleur`, `europa`, `claudeterminal` (la pieuvre) et `accessoires.json` ; revue d'ensemble
  (`python tools/habitants/apercu.py --revue revue.png habitants/*.json`).
