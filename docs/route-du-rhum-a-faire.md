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
  
  **Personne n'écrit encore ce fichier** : c'est le travail du §2.
- Sans fichier, le décor avance au rythme d'un Ultim (7 jours pour la traversée), à partir de l'heure du départ.

## 1. Vérifier sur Windows, avant tout

Rien de l'appli n'a tourné sous Windows : elle a été compilée pour Windows (`cargo check --target
x86_64-pc-windows-gnu`), et son dessin testé sous Linux avec un faux module `windows`.

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

**Trouver la source** (la VM n'y avait pas accès) :
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
- **Le plus simple** : un outil Node, `tools/route-du-rhum.mts --suivre` par exemple. Il va chercher la source, la convertit, écrit le fichier, et recommence toutes les 15 minutes. Une tâche planifiée Windows ou l'orchestrateur le lance.
- **Si la source est un simple JSON ou CSV**, la session désignée peut aussi la chercher elle-même : `$.http.fetch` dans `hooks/register.tsx`, à côté de `watchRhumData`. Toujours à un rythme raisonnable, et en respectant les conditions d'utilisation du site.

Tester : écrire le fichier à la main avec deux ou trois bateaux. La carte doit apparaître dans les
deux secondes, et le journal doit dire « Route du Rhum : pointage de 14h00, 3 bateaux ».

## 3. Les idées de dessins

Faites :
- la baie au vrai pixel, le voyage de Saint-Malo à la Guadeloupe, la carte ;
- le bateau-comité et son canon, le feu d'artifice, les grains, les sillages phosphorescents.

À faire, dans l'ordre où elles rendraient le plus :
1. **Les arrivées** : quand un bateau a `dtf` à 0, le personnage saute de joie, un toast le nomme,
   et la baie de Guadeloupe montre la ligne d'arrivée et un bateau qui la coupe.
2. **Un nouveau premier** dans une classe : une ligne au journal, une réplique du personnage.
3. **Les abandons** (si la source les donne) : grisés sur la carte.
4. **La trace du premier** sur la carte : garder ses positions des derniers pointages et tracer son sillage.
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
