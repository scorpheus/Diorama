---
name: nouveau-theme
description: Crée un thème Diorama à soi (un personnage et son décor en pixel art, ses mots) d'après une description, avec des aperçus PNG à chaque étape, puis le branche dans le bandeau. À utiliser quand la personne veut un autre personnage que le forgeron, retoucher un thème, ou demande « nouveau thème Diorama ».
---

# Créer un thème Diorama

Diorama dessine au-dessus du prompt un personnage qui suit le travail de Claude. Un thème est **un seul fichier JSON** : la scène, le personnage et ses poses, les mots. Le format complet est dans `THEMES.md`, à la racine du plugin. Ce fichier `SKILL.md` est dans `skills/nouveau-theme/` : la racine est donc deux dossiers au-dessus. Le thème `themes/sorciere/theme.json` y sert d'exemple complet.

**Lis `THEMES.md` en entier avant de dessiner**, surtout « Ce que le moteur fait au personnage » et « Relire chaque moment ». Puis lis l'exemple de la sorcière pour voir les proportions ; sa pose `sweep` montre des pas propres à un moment (`walk`).

## 1. Comprendre ce que la personne veut

Une à trois questions suffisent, et seulement sur ce qui manque à la description :
- qui est le personnage ;
- où il travaille, et ce qu'il fait quand Claude travaille (il martèle, touille, tape, jardine…) ;
- qui sont ses aides pour les sous-agents, et s'il y a un animal ;
- le ton des mots.

Propose un ensemble cohérent plutôt qu'une liste d'options : le métier donne la pose de travail, le décor et les mots.

## 2. Dessiner avec un générateur, pas à la main dans le JSON

Écris un petit script (Python ou Node) dans un dossier de travail. Il part d'**une pose de base** du personnage et fabrique les autres par retouches : bras levé, yeux fermés, pas de marche. Il écrit ensuite `theme.json`. C'est ainsi que la sorcière a été faite, et c'est le seul moyen de garder des poses cohérentes entre elles.

Règles de dessin qui comptent dans un terminal :
- **Taille.** Personnage d'environ 14 à 18 pixels de large et 22 à 26 de haut, pieds sur la dernière ligne (`home.y + hauteur = height`). Laisse au moins 6 pixels au-dessus de la tête pour les bulles.
- **Contraste.** Un contour `k` autour de chaque forme. Pas de grandes zones noires : sur un terminal sombre, elles disparaissent.
- **Visage.** Des yeux de 1 ou 2 pixels, un pixel de bouche. Une rangée sombre continue sous les yeux se lit comme une moustache. Une pupille noire seule se lit comme un trou, un œil tout blanc comme un aveugle.
- **Mouvement.** Chaque pose doit se distinguer **à la silhouette** : bras, outil, inclinaison. Deux images qui ne diffèrent que d'un pixel au milieu du corps ne se voient pas.
- **Marche.** 4 images, les pieds alternés. Le moteur avance d'une image par pixel parcouru : les pieds restent posés.
- **Un moment, un geste et un objet.** Chaque moment tient l'objet de son action (pas l'outil d'un autre moment), et son geste produit un effet visible : poussière, vapeur, étincelles.
- **Les moments qui marchent.** Au repos, en réflexion et au balayage, le moteur fait marcher le personnage, et c'est alors la marche qui s'affiche. Si la pose tient un objet, donne-lui ses propres pas (`walk`) : le générateur prend le haut des images du moment et les pieds de la marche.
- **Tenue.** Les deux lettres de `recolor` (par défaut `o`/`O`) sur le vêtement principal : chaque session aura sa couleur.

## 3. Vérifier et regarder à chaque étape

Trois outils, du plus rapide au plus complet. Tous vérifient d'abord le fichier (lettres hors palette, positions, poses inconnues, personnage hors cadre). Si une erreur sort, corrige le générateur, jamais le JSON à la main. Il faut Node 22.6 ou plus.

**La planche, pour toi, à chaque retouche :**

```
node <racine>/tools/apercu.mts <theme.json> <apercu.png>
```

Elle dessine chaque moment du moteur, puis le décor dans ses situations, dans l'ordre que la commande affiche. **Ouvre le PNG et regarde-le** avant de continuer. Pour regarder de près, `--echelle 8 --recadrage solo` grossit l'image.

**Les animations déroulées, pour toi :**

```
node <racine>/tools/apercu.mts <theme.json> <anim.png> --anime tout
```

Chaque moment s'étale sur une ligne, image par image, et les moments où le personnage marche ont deux lignes de plus, en marche. Une seule image par moment ne dit pas si l'animation marche : relis ces lignes avec la liste de contrôle de `THEMES.md` (« Relire chaque moment »). Pour un seul moment : `--anime <moment>`, par exemple `--anime balaie`.

Relis comme la personne verra, en te posant ces questions :
- se lit-il au premier coup d'œil ?
- chaque moment est-il différent des autres ?
- chaque moment tient-il le bon objet, et son geste produit-il un effet visible ?
- en marche, garde-t-il son objet dans les moments qui marchent ?
- les bulles, les aides et l'animal tiennent-ils dans la scène ?

**La mire en direct, pour la personne, qui valide :**

```
node <racine>/tools/mire.mts <theme.json>
```

Elle joue le thème dans un terminal, en vraies couleurs, au rythme du bandeau. La personne passe d'un moment à l'autre avec les flèches et change la marche avec `m` (« comme en vrai » montre les déplacements du moteur). Elle change l'heure avec `h`, met en pause avec espace et quitte avec `q`. Ouvre-la dans une **nouvelle fenêtre**, car elle a besoin d'un vrai terminal :
- sous Windows : `Start-Process cmd.exe -ArgumentList '/k', 'node <racine>/tools/mire.mts <theme.json>'` (PowerShell) ;
- ailleurs : donne la commande à la personne pour qu'elle la lance dans un autre terminal.

**Quand il y a un choix à faire** (deux silhouettes, deux gestes, deux palettes), ne décris pas les options : fais-les. Le générateur écrit une variante par fichier, et la mire les montre côte à côte, nommées A, B, C, D :

```
node <racine>/tools/mire.mts variante-a.json variante-b.json variante-c.json
```

La personne répond par lettre ; garde la variante choisie et supprime les autres fichiers, que tu as créés toi-même.

Montre la planche ou la mire à la personne et demande-lui ce qu'elle en pense. Recommence jusqu'à ce qu'elle soit contente. Avant de passer à l'étape 4, fais-lui parcourir dans la mire **tous** les moments, y compris ceux qu'on oublie : tests réussis et ratés, commit, push, balayage, réveil.

Sans Node, branche directement le thème (étape 4) : le bandeau le dessine en direct, et le journal sous les jauges dit pourquoi un thème est refusé.

## 4. Brancher le thème

1. Enregistre le fichier sous un chemin **absolu** stable, par exemple `~/.claude/diorama/themes/<nom>/theme.json`, en écrivant le dossier personnel en toutes lettres.
2. Dis à la personne d'ouvrir `/config` et de mettre ce chemin dans *Diorama · Thème*.
3. Une fois branché, le bandeau se redessine tout seul à chaque enregistrement du fichier : on peut retoucher en regardant.

## 5. Les mots

Remplis `words` pour que le bandeau parle comme le thème :
- `persona` et `helper`, que lit Haiku ;
- les `labels` et `atWork`, qui s'affichent sous le nom à longueur de journée ;
- quelques `lines` par moment ;
- les `notes` qui nomment le messager et l'animal.

Ce qui n'est pas fourni reste neutre. Garde les marques `{n}`, `{who}`, `{cmd}`, `{project}` là où les valeurs neutres les ont.

## Partager

Si la personne veut partager son thème, elle peut ouvrir une demande de fusion sur `scorpheus/Diorama` avec `themes/<nom>/theme.json`. Le thème se choisira alors par son nom seul.
