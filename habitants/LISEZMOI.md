# Les habitants des onglets de ClaudeTerminal

Un personnage par profil, dans le style du Diorama, que la barre d'onglets de ClaudeTerminal pose et anime selon
l'état de la session. Ce fichier est le contrat entre les deux (copie dans `D:\KT\.claude\projets\onglets-habitants.md`).

- **Où** : dépôt Diorama, dossier `habitants/`. Un fichier par personnage, `habitants/<nom>.json` (UTF-8). Les
  accessoires (pigeon, toiles, bougie, bourse, météo, mineurs, bulle) viendront dans `habitants/accessoires.json`, mêmes
  images, format écrit ici avant d'être dessinés.
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
  | `travail` | la session travaille | tape au clavier |
  | `repos` | la session est au repos | dort, des Z montent |
  | `attente` | la session attend le user | agite un drapeau |
  | `echec` | erreur, échec | assis dans la fumée |
  | `parle` | la ligne « Dit : » est prononcée | 2 images : bouche fermée, ouverte ; ClaudeTerminal peut les alterner au rythme de la voix plutôt qu'à `every` |
- **Placement** : l'image se pose par le milieu de son bord bas (les pieds sur la dernière ligne). Agrandissement
  **entier** au plus proche (× 1, × 2…), choisi par ClaudeTerminal selon la hauteur de la barre, sans lissage. Ce qui
  accompagne l'état (Z, drapeau, fumée, clavier) est **dans** les images : le lecteur n'ajoute rien.
- **Voyageur** : `habitants/voyageur.json`, mêmes états, teint par `O`/`o` ; un profil sans habitant validé le prend,
  un fichier refusé aussi.
- **Vérifier** : `python tools/habitants/apercu.py habitants/<nom>.json planche.png [--anime anim.gif] [--teinte #rrggbb]`
  refuse un fichier faux (taille, lettre hors palette, état obligatoire absent, `every` non entier ≥ 1) et dessine
  la planche. Le lecteur de ClaudeTerminal refuse les mêmes fautes et prend alors le voyageur.
- **Profil → habitant** : à ClaudeTerminal (sa config ; par défaut, le nom du profil en minuscules sans accents).
  Fichiers prévus : `galadriel` (Orchestratrice), `forgeron` (Endurance), `marin` (Diorama, Route du Rhum), `mecano`
  (vehigraph), `tailleur` (LaPierreDesMots), `voyageur`.
- **File des demandes** (le paragraphe « Si aucune session Diorama n'est ouverte ») : `habitants/demandes/<nom>.json`,
  écrit par ClaudeTerminal ou l'orchestratrice, `{ "nom", "profil", "couleur": "#rrggbb", "sujet": "ce que fait le
  profil, en une phrase", "le": "AAAA-MM-JJ" }`. La session Diorama dessine `habitants/<nom>.json` avec `valide: false`,
  et ne supprime jamais la demande ; elle est servie quand l'habitant existe avec `valide: true`, et son auteur la
  retire alors. **En attente** = la demande existe et `habitants/<nom>.json` n'existe pas encore (dessiné mais pas
  validé : il attend le user, pas une session). `<nom>` : minuscules, sans accents ni espaces (`[a-z0-9-]+`), le même
  que l'habitant. Exemple : `habitants/demandes/vehigraph.json` →
  `{ "nom": "vehigraph", "profil": "vehigraph", "couleur": "#4a9e5c", "sujet": "outil de diagnostic véhicule", "le": "2026-11-03" }`.
  La file n'est pas suivie par git (`.gitignore`) : elle vit sur le disque de la machine, pas dans l'historique.
- **Fait** : `marin.json` (premier habitant, `valide: false` en attendant l'avis du user).
