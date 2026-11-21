# D-001 — Découper le projet en un workspace de quatre crates

| Champ    | Valeur                                  |
|----------|-----------------------------------------|
| ID       | D-001                                   |
| Date     | 2026-10-17                              |
| Auteurs  | Mahamadou                               |
| Statut   | adoptée                                 |
| Commits  | `55adcc7`                               |

## 1. Contexte

Au démarrage, il fallait choisir comment organiser le code. Le projet a trois parties de natures très
différentes : lire et valider un fichier `.pbc` (annexe A), exécuter le bytecode (annexes B et D), et
servir l'exécution par le réseau (annexe K). Le barème évalue explicitement « la séparation nette entre
le format PBC, le moteur d'exécution et la couche réseau ». Le moteur ne doit dépendre ni du réseau ni
des fichiers, pour rester déterministe et testable seul.

## 2. Options envisagées

### Option A — Une seule crate, avec un dossier (module) par partie

- Avantages : configuration minimale, un seul `Cargo.toml`, rien à apprendre sur les workspaces.
- Inconvénients : la séparation n'est qu'une convention. Rien n'empêche le moteur d'importer du code
  réseau par erreur, et on ne s'en rendrait compte qu'en relecture.

### Option B — Un workspace Cargo avec quatre crates

`pbc` (format), `vm` (moteur), `exec` (serveur réseau) et `palestre` (le binaire en ligne de commande),
avec des dépendances dans un seul sens : `palestre → exec → vm → pbc`.

- Avantages : les frontières sont imposées par le compilateur, une crate ne voit que ce dont elle
  dépend. On peut compiler et tester chaque partie seule (`cargo test -p vm`). Les versions et les
  réglages communs sont écrits une seule fois dans le `Cargo.toml` racine.
- Inconvénients : un peu plus de configuration, et il faut décider ce qui est public (`pub`) pour être
  utilisé d'une crate à l'autre.

## 3. Ce que l'assistant IA a proposé

J'ai demandé à l'assistant (Claude, dans le navigateur, en conversation et pas en agent autonome) ce
qui se fait en général dans un projet Rust professionnel. Il a proposé l'option B avec ces quatre noms
et le sens unique des dépendances, en citant le critère du barème. Il a aussi présenté l'option A
comme alternative plus simple.

Il avait d'abord commencé à tout générer d'un coup (lints, CI, journal) sans m'expliquer. Je l'ai
arrêté : j'ai demandé à avancer étape par étape et à taper le code moi-même pour le comprendre.

Ce que j'ai vérifié : le workspace compile (`cargo build`) et `cargo run -p palestre` fonctionne, ce
qui montre que les crates se voient entre elles.

## 4. Ce que nous avons décidé, et pourquoi

Option B. Les éléments (`pbc`, `vm`, `exec`) ne doivent pas être mélangés : chacun a une
responsabilité unique, et le compilateur garantit que le moteur ne peut pas dépendre du réseau. C'est
aussi ce qui permettra de localiser rapidement une modification demandée en soutenance, par exemple
une nouvelle instruction de la spec v2 ne touche que `vm`.

## 5. Ce qui nous ferait changer d'avis

- Si on se retrouvait à dupliquer du code entre deux crates parce qu'il n'a pas sa place dans l'une
  ou l'autre.
- Si on devait rendre publics beaucoup d'éléments internes juste pour traverser une frontière : le
  découpage serait alors mal placé.
- Si une évolution du sujet (nouvelles directives, spec v2) demandait une partie qui ne rentre dans
  aucune des quatre crates.

## Historique

- 2026-10-17 : décision prise et workspace créé.
- 2026-11-21 : fiche rédigée a posteriori, la décision ayant été prise avant la mise en place du journal.
