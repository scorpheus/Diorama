# Route du Rhum : ce qui reste à faire, sur le PC

Ce qui a été fait l'a été dans une machine virtuelle Linux, sans Windows et sans accès aux sites de
la course. Cette page dit ce qu'il reste à vérifier et à brancher sur le PC, avec un Claude Code
connecté aux vraies informations. Le départ est le **dimanche 1er novembre 2026 à 13h02**, de
Saint-Malo, pour 3 542 milles jusqu'à Pointe-à-Pitre.

## Où on en est

- **Une seule session montre la course** : celle que nomme `%USERPROFILE%\.claude\diorama-route-du-rhum.txt`
  (pid, nom ou identifiant de session ; `node tools/route-du-rhum.mts` liste et désigne), ou celle
  lancée avec `DIORAMA_ROUTE_DU_RHUM=1`. Voir le README, section *La Route du Rhum*.
- **L'appli native peint la cour au vrai pixel** (`rendu/src/forge/baie.rs`, `trace.rs`) :
  - le ciel de l'heure, le soleil bas et son reflet, la lune, les cumulus ;
  - la houle, la flotte (Ultim et Ocean Fifty en vol, IMOCA à foils, Class40 et Rhum sous gennaker), les sillages, les feux de navigation ;
  - le décor qui voyage : Saint-Malo, le large et les Açores, les tropiques, la Guadeloupe ;
  - le bateau-comité et son canon au départ, les grains des tropiques, les sillages phosphorescents la nuit au large, le feu d'artifice des nuits d'arrivée.
- **Les données** : la session désignée lit `%USERPROFILE%\.claude\diorama-route-du-rhum.json`
  (format dans le README). Avec ce fichier :
  - la carte de l'Atlantique s'épingle dans la cour ;
  - les bateaux de la baie prennent les couleurs du premier de leur classe ;
  - le décor avance avec la distance du premier ;
  - une ligne près des jauges nomme le premier de chaque classe.
  
  `tools/route-du-rhum-suivre.mts` l'écrit d'après le tracker GeoVoile (§2) ; la carte 2026 n'est pas
  encore en ligne (25 octobre).
- Sans fichier, le décor avance au rythme d'un Ultim (7 jours pour la traversée), à partir de l'heure du départ.

## 1. Vérifier sur Windows, avant tout

**Fait sur le PC, le 25 octobre** (branche fusionnée dans `main`) : `cargo test` passe (20 tests),
la version release se compile, `claude plugin test .` passe (89 tests). La planche a été regardée :
rien ne déborde, les bateaux de la Guadeloupe sont sur l'eau et les palmiers devant. Le coût est mesuré
hors écran (`cargo test --release chrono_baie -- --ignored --nocapture`) : entre 0,7 et 1,9 ms par image
avec la baie, contre 0,4 ms pour la cour seule, en cellules de 9×19 comme de 14×30. À 150 ms la
cadence, cela fait environ 1 % d'un cœur : rien à optimiser. **Restent à voir à l'œil** : la vitrine
(point 3) et la vraie session (point 4).

1. Les tests et la compilation :
   ```
   cd rendu
   cargo test
   cargo build --release
   cd ..
   claude plugin test .
   ```
   Le test « the visits are looked for in the store read whole » échouait sous Linux, avant ces
   changements, à cause des chemins `C:/…` ; sous Windows, les 86 tests doivent passer.
2. **La planche** : `rendu\target\release\rendu.exe --planche planche.png --large 72`. Les
   dernières scènes sont la Route du Rhum : avant le départ, le soir, le départ et son canon, les Açores,
   le large la nuit, les tropiques avec la carte (sur une flotte d'exemple), la Guadeloupe de jour
   et de nuit. Les regarder à l'œil : couleurs, tailles, rien qui déborde.
3. **En mouvement** : `rendu.exe --vitrine` joue ces scènes dans une fenêtre, à la cadence. Regarder
   les bateaux (la course, le tangage, les sillages), la houle, les nuages, le phare, les dauphins,
   la baleine, les poissons volants, le feu d'artifice.
4. **En vrai** : ouvrir la session désignée.
   ```
   start "Route du Rhum" cmd /k "set DIORAMA_ROUTE_DU_RHUM=1&& claude"
   ```
   Vérifier les points suivants :
   - la cour devient la mer dans cette session seulement ;
   - les petits, les invités et le chat restent devant la baie ;
   - la porte et l'avant-toit sont toujours là ;
   - la ligne « ⛵ Route du Rhum · départ dans … » s'affiche près des jauges.

   Désigner ensuite une autre session avec `node tools/route-du-rhum.mts <pid>` : la mer doit y passer en deux secondes.
5. **Ce que ça coûte** : sur la VM, la baie prend environ 1 ms par image, soit à peu près 1 % d'un cœur
   à 150 ms la cadence. Mesurer sur le PC, et regarder `%TEMP%\diorama-rendu.log`. Si c'est trop :
   - la houle (`Baie::mer`) et la pose sous les cellules transparentes sont les deux postes réguliers ;
   - les nuages se repeignent quand ils ont glissé d'un pixel ;
   - les bateaux et les palmiers sont des vignettes gardées.
6. **D'autres tailles de police et d'autres échelles d'écran** : la baie se dessine à la taille
   d'une cellule (`cw`, `ch`). À essayer avec une police plus grande, et sur un écran à 150 %.

## 2. Brancher les vraies données

But : écrire `%USERPROFILE%\.claude\diorama-route-du-rhum.json` toutes les 15 minutes environ,
dans le format du README :
```json
{
  "at": "2026-11-03T13:00:00Z",
  "bateaux": [
    { "nom": "…", "skipper": "…", "classe": "Ultim", "couleurs": ["#coque", "#bande"],
      "lat": 30.04, "lon": -33.97, "rang": 1, "dtf": 1771.4, "vitesse": 31.2 }
  ]
}
```
Les classes : `Ultim`, `Ocean Fifty`, `IMOCA`, `Class40`, `Rhum Multi`, `Rhum Mono`. `rang` est le
rang dans la classe (déduit de `dtf` s'il manque), `dtf` les milles jusqu'à l'arrivée, `vitesse` en
nœuds. Écrire dans un fichier à côté, puis le renommer : la session ne doit jamais lire un fichier à moitié écrit.

**Fait sur le PC, le 25 octobre** : la source est le tracker GeoVoile,
`routedurhum.geovoile.com/2026/tracker/`. Il répond « Mise en ligne à venir » pour l'instant. Celui de
2022 est encore en ligne, avec les mêmes objets en mémoire :
- `tracker._boats` : nom, skipper, `hullColors`, trace ;
- `tracker._boatClassesByIds` : les classes, dont « Ultim 32/23 » ;
- `tracker._reports` : les pointages, avec `rank` dans la classe, `dtf`, `speed` et `racestatus`
  (`RAC`, `ARV` ou `RET`).

`tools/route-du-rhum-suivre.mts` lit ces objets dans Edge sans fenêtre, par le protocole DevTools,
sans dépendance. Rejoué sur 2022 (`--annee 2022 --instant 2022-11-12T14:00Z`), il donne 138 bateaux
et un premier par classe. **Reste à faire** quand la carte 2026 sera en ligne, sans doute avant le
1er novembre :
- le relancer une fois, pour vérifier les noms des classes et l'état avant le départ ;
- ~~le faire tourner pendant la course~~ : fait, la session désignée le lance tous les quarts
  d'heure (`rhumFetch` dans `hooks/register.tsx`), d'une semaine avant le départ à quarante jours
  après.

Le rejeu a montré un piège, déjà corrigé dans le plugin : un bateau qui abandonne a, chez GeoVoile,
le rang 0 et 0 mille à faire. Il serait devenu premier de sa classe, et le décor serait parti en
Guadeloupe. Le plugin lit maintenant `statut`.

Les pistes de départ, pour mémoire (la VM n'y avait pas accès) :
1. **Le site officiel**, `routedurhum.com` : la cartographie et les classements. Ouvrir les outils
   de développement du navigateur (onglet *Réseau*) sur ces pages, et chercher ce qu'elles chargent :
   du JSON, ou des fichiers de classement (Excel ou CSV, publiés à chaque pointage, comme ceux du
   Vendée Globe). C'est le meilleur cas : un `fetch` suffit.
2. **Le tracker**, historiquement GeoVoile (`routedurhum.geovoile.com`, `carto-prod.routedurhum.com`
   en 2022). Ses données arrivaient en binaire chiffré, que leur propre JavaScript décode. S'il
   n'y a rien d'autre, un navigateur sans fenêtre (Playwright) peut charger la carte et lire l'objet
   décodé en mémoire. Le projet `Yapock22/Sail_data` sur GitHub le fait pour d'autres courses
   GeoVoile.
3. **Les couleurs des bateaux** : le tracker en donne souvent une par bateau (celle de sa trace).
   À défaut, une petite table des livrées connues (au moins les Ultim et les premiers IMOCA).
4. Avant le départ, la liste des inscrits suffit : tous les bateaux à Saint-Malo (48,65 N ; 2,02 O),
   la carte les montre au port.

**Où l'écrire** :
- **Fait** : `tools/route-du-rhum-suivre.mts`. Il va chercher la source, la convertit et écrit le fichier, trace des premiers comprise. La session désignée le lance avec `--une-fois` tous les quarts d'heure : pas de tâche planifiée Windows, donc pas de console qui clignote, et rien ne tourne quand la session est fermée.
- **Si la source est un simple JSON ou CSV**, la session désignée peut aussi la chercher elle-même : `$.http.fetch` dans `hooks/register.tsx`, à côté de `watchRhumData`. Toujours à un rythme raisonnable, et en respectant les conditions d'utilisation du site.

Tester : écrire le fichier à la main avec deux ou trois bateaux. La carte doit apparaître dans les
deux secondes, et le journal doit dire « Route du Rhum : pointage de 14h00, 3 bateaux ».

## 3. Les idées de dessins

Faites :
- la baie au vrai pixel, le voyage de Saint-Malo à la Guadeloupe, la carte ;
- le bateau-comité et son canon, le feu d'artifice, les grains, les sillages phosphorescents ;
- le spectacle du lancement de session (25 octobre) : le départ rejoué, ou l'arrivée avec la ligne,
  un Ultim qui la coupe, le feu d'artifice même de jour, et les arrivés nommés (README). Il couvre
  l'idée 1 ci-dessous. Pour le regarder hors écran :
  `SPECTACLE=start FILM=depart.png cargo test --release film_rhum -- --ignored` (ou `finish`).

À faire, dans l'ordre où elles rendraient le plus :
1. ~~**Les arrivées**~~ : faites, par le spectacle (voir plus haut).
2. ~~**Un nouveau premier**~~ : fait, une ligne au journal et une réplique du personnage.
3. ~~**Les abandons**~~ : faits, en gris sur la carte, sous les autres.
4. ~~**La trace du premier**~~ : faite, lue sur le tracker (`trace`), une quarantaine de points
   par premier. Vue sur le rejeu 2022 :
   `RHUM_JSON=<monde.json> rendu.exe --planche planche.png --large 72` dessine la carte d'un vrai
   pointage (au format du monde : `rhumFlotte`, `rhumTraces`, `rhumTetes`).

Faits aussi, le 25 octobre : la carte toujours affichée (la flotte à l'estime sans pointage), une
rose des vents, une punaise, et un voile de nuit sur le papier.
5. **Le ciré jaune** : un accessoire du personnage pour la semaine (un calque du pet, comme les
   autres accessoires : `tools/pet/chatgpt.py --tenue`, voir `tools/pet/CHATGPT.md`).
6. **Le vrai vent** : la vitesse et la direction du vent sur la flotte (si la source les donne) pour
   la houle, l'angle des voiles, les grains.
7. **Le dessin au terminal** (`rendu` à `non`, `hooks/themes/forge-yard.ts`) : il reste à
   Saint-Malo, en gros pixels. Lui porter au moins les étapes du voyage, si quelqu'un l'utilise.

Pas d'hélicoptère.

## Les fichiers

| fichier | rôle |
|---|---|
| `hooks/register.tsx` | quelle session montre la course, le compte à rebours, le pointage lu, ce qui part à l'appli (`rhum`, `rhumProgress`, `rhumFlotte`, `rhumTetes`) |
| `hooks/world.ts` | ces champs du monde |
| `rendu/src/forge/monde.rs` | les mêmes, côté appli |
| `rendu/src/forge/baie.rs` | la baie, la flotte, le voyage, la faune, la carte |
| `rendu/src/forge/trace.rs` | polygones, traits et disques lissés |
| `rendu/src/forge/cour.rs` | la cour laisse transparent ce que la baie peint dessous |
| `rendu/src/forge/mod.rs` | la baie peinte sous les cellules de la cour, à chaque image |
| `rendu/src/forge/planche.rs` | les scènes de la planche et de la vitrine |
| `hooks/themes/forge-yard.ts` | la version en gros pixels, pour le terminal |
| `tools/route-du-rhum.mts` | désigner la session |
| `tests/diorama.test.tsx` | les tests du plugin (désignation, compte à rebours, pointage) |
