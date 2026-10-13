# Plan : Diorama devient une appli native

Objectif : **une seule solution, meilleure**. Une appli native dessine la forge sur le bandeau,
avec la même cadence et les mêmes pulsations, la porte ouverte à un dessin plus fin, pour une
fraction du CPU actuel. Le plugin ne garde que ce que seul Claude peut lui donner. Le dessin dans
le terminal, le prototype « voile » et le passeur disparaissent : l'appli les remplace tous les trois.

## 1. Audit : où part le CPU aujourd'hui

Mesures du 2026-10-13, une session au repos, au premier plan, en % d'un cœur.

| poste | coût | d'où il vient |
|---|---|---|
| Claude seul, sans plugin | ~0,4 | — |
| minuterie du plugin à 150 ms | ~0,8 | `$.clock.every(TICK_MS)` : un aller-retour moteur par tic |
| calcul de l'image | ~0,4 | `theme.frame` dans le bac à sable du moteur |
| affichage par le terminal | ~2,5 à 3,5 | `$.ui.blit` → texte ANSI → Ink → Windows Terminal |
| coups d'œil aux autres sessions | ~0,5 | visites, chat, banquet : relire le store toutes les 2 et 5 s |
| **total plugin** | **~5,0** | mesuré : 5,02 |
| Windows Terminal | ~7,5 à 8 | partagé entre toutes les sessions, redessine chaque bande 6 à 7 fois/s |

Le prototype « voile » n'a sorti que l'affichage : le plugin calculait et expédiait encore 6 à 7
images par seconde (3,3 à 3,9 % dans Claude, 1 % dans le voile). **Ce qui coûte, c'est que le
plugin vive à 150 ms. Il faut que plus rien ne tourne dans Claude entre deux événements.**

Ce que le plugin utilise vraiment de Claude (`register.tsx`) :

- les événements : `session.start`, `prompt.submit`, `tool.call`, `turn.complete`,
  `agent.spawn`, `session.compact`, `session.measure`, `turn.step` ;
- `$.session.usage`, `$.session.model`, `$.agent.list` : les jauges, l'équipe ;
- `$.model.complete` : les répliques et le statut écrits par Haiku ;
- `$.ui.render` / `$.ui.blit` : la place du bandeau dans le terminal ;
- `$.store` : seulement pour que les sessions se voient entre elles, ce qu'une appli unique
  voit sans fichier.

Tout le reste (horloge, marche, routines, sieste, thé, visites, chat, banquet, corbeaux, dessin,
thèmes) n'a pas besoin de Claude.

## 2. Le langage : Rust

- **Natif, sans moteur ni ramasse-miettes** : au repos, l'appli dort entre deux images. On vise
  quelques centièmes de % par bande visible, contre 0,5 % mesuré pour une fenêtre Bun.
- **Un exécutable de 1 à 2 Mo**, quelques Mo de mémoire, sans rien à installer pour le faire tourner.
- **Accès direct à Windows et au GPU** (crate `windows`) : Direct2D sur DirectComposition. Le GPU
  agrandit l'image, la transparence est gérée par la composition de Windows, et le texte se
  dessine avec DirectWrite. C'est ce qui permet le dessin fin, des pixels plus petits que la cellule.
- **Déjà là** : `cargo` 1.93 et Visual Studio 18 (l'éditeur de liens).
- Écartés :
  - Bun/TypeScript : un moteur JS qui tourne en permanence, des rappels FFI fragiles pour la
    procédure de fenêtre ;
  - C# (comme le passeur) : JIT et ramasse-miettes, et C# 5 seulement avec le `csc` de Windows ;
  - C++ : aussi rapide que Rust, mais plus risqué pour un programme qui tourne en permanence.

**Distribution** : le hook `SessionStart` compile `rendu/` avec `cargo build --release` quand la
source change (comme `passeur.ps1` avec `csc`), dans `%LOCALAPPDATA%\Diorama`. Pour quelqu'un sans
Rust, un exécutable publié dans les versions GitHub, téléchargé une fois (plus tard).

## 3. Qui fait quoi

| | plugin (TypeScript, dans Claude) | appli `rendu/` (Rust) |
|---|---|---|
| événements de Claude | ✔ les relaie, tels quels | ✔ en tire l'activité, les jauges, l'xp |
| répliques et statut par Haiku | ✔ (`$.model.complete`), relaie le texte | ✔ les affiche |
| la place du bandeau | ✔ un rectangle noir de la bonne taille, 2 repères, **fixe** | |
| horloge, marche, routines, sieste, thé | | ✔ à 150 ms, les mêmes règles |
| vie entre sessions (visites, chat, banquet, corbeaux) | | ✔ directement : il voit toutes les sessions |
| vols d'une fenêtre à l'autre (le passeur) | | ✔ c'est déjà sa fenêtre |
| image, texte à côté (jauges, statut, journal) | | ✔ Direct2D + DirectWrite |
| trouver le bandeau, suivre onglets, volets, défilement | | ✔ par les repères |
| thèmes (forge, sorcière, `theme.json` à soi) | | ✔ |

**Le plugin ne tourne plus du tout entre deux événements** : pas de minuterie, pas de store.
Chaque événement part en une requête HTTP locale (127.0.0.1) vers l'appli. C'est quelques
requêtes par seconde au plus pendant un tour, et aucune au repos.

**Une seule appli pour la machine** : lancée par le premier `SessionStart`, une fenêtre par
bandeau visible. Elle s'en va quand il ne reste plus aucune session.

## 4. Étapes

Chaque étape a son banc, vu rouge avant d'être vert.

1. **Le squelette, et sa mesure.**
   - L'appli Rust : serveur local, une fenêtre posée sur le bandeau grâce aux repères, une image
     d'essai animée à 150 ms.
   - Le plugin, derrière une option : le rectangle noir marqué, les événements relayés.
   - **Seuil pré-enregistré** : l'appli au repos avec une bande visible doit coûter ≤ 0,1 %
     d'un cœur. Sinon on s'arrête là et on regarde pourquoi, avant d'écrire le reste.
2. **Le dessin porté**, pixel pour pixel.
   - Avant de toucher au TypeScript, il produit un jeu d'images de référence : chaque activité,
     plusieurs tics, plusieurs mondes, les deux thèmes.
   - Le Rust doit les redonner **à l'octet près**. C'est le contrat qui garantit la même cadence
     et les mêmes pulsations.
   - L'outil d'aperçu (`tools/apercu`, `mire`) devient `rendu --apercu`, qui écrit les PNG.
3. **Le cerveau porté** : activité, jauges, marche, routines, vie entre sessions, vols.
   - Les 80 tests actuels sont réécrits en tests Rust, scène par scène. Un test ne disparaît que
     s'il porte sur le relais du plugin.
4. **La bascule** : l'option devient le seul mode. Le plugin est réduit au relais, et on supprime
   le dessin TypeScript, le voile, le passeur et le store des visites.
5. **La mesure finale**, au protocole du tableau : plugin, appli, Windows Terminal.
   - Seuil pré-enregistré : le total par session doit passer de 5,0 % à ≤ 1 %, sans perte de
     cadence visible.
6. **Ensuite** : le dessin fin, avec des pixels plus petits que la cellule.

## 5. Estimation, à vérifier par la mesure

| | aujourd'hui | avec l'appli |
|---|---|---|
| plugin dans Claude | ~5,0 % par session | ~0,4 à 0,5 % (Claude seul, plus quelques requêtes par tour) |
| appli de rendu | — | quelques centièmes de % par bande visible, un seul processus |
| passeur | un processus de plus | absorbé |
| Windows Terminal | redessine chaque bande 6 à 7 fois par seconde | ne redessine plus les bandes |

## 6. Ce que ça coûte, et les risques

- **Windows seulement** : une seule solution veut dire qu'il n'y a plus de dessin dans le
  terminal. Sur Mac et Linux, le bandeau resterait vide tant que l'appli n'y est pas portée.
- **Le portage est gros** : environ 4 800 lignes de TypeScript (dessin 3 300, cerveau 1 800). Le
  contrat « à l'octet près » de l'étape 2 et les tests de l'étape 3 évitent qu'il dérive.
- **Une fenêtre qui ne couvre qu'un repère** : l'appli se cache, alors que le reste du bandeau est visible.
- **Changement d'onglet** : jusqu'à 250 ms de rectangle noir avant que l'appli se pose.
- **Écrans à échelles différentes** : gérés par la conscience DPI par écran, comme le passeur.
- **L'appli plantée** : le bandeau reste noir. Le hook `SessionStart` suivant la relance, et le
  plugin peut la relancer s'il voit ses requêtes refusées.
