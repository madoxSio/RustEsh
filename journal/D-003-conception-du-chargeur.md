# D-003 — Concevoir le chargeur `.pbc` autour d'un lecteur d'octets

| Champ    | Valeur                                         |
|----------|------------------------------------------------|
| ID       | D-003                                          |
| Date     | 2026-11-22                                     |
| Auteurs  | Mahamadou                                      |
| Statut   | adoptée                                        |
| Commits  | à compléter avec le hash du commit du chargeur |

## 1. Contexte

Le chargeur lit un fichier `.pbc` octet par octet selon l'annexe A : signature `PBC1`, version, flags,
nombre de constantes, constantes (`i64`), taille du code, code. Les entiers sur plusieurs octets sont en
big-endian. Toute anomalie doit produire `Fault::BadHeader`, sans `panic`, sans lecture hors du
fichier et **sans allocation non bornée**, quelle que soit l'entrée, même aléatoire ou construite pour
nuire (A.2). C'est l'invariant 5 du sujet et le premier critère du MVP.

## 2. Options envisagées

### Option A — Calculer les positions à la main

Lire chaque champ par sa position fixe (`bytes.get(6..8)`, `bytes.get(8..16)`…).

- Avantages : pas de type supplémentaire, tout est visible dans une seule fonction.
- Inconvénients : les décalages dépendent du nombre de constantes, donc il faut les recalculer à la
  main. Une erreur d'un octet est vite arrivée, et chaque lecture répète la vérification de taille.

### Option B — Un lecteur séquentiel (`Reader`) qui avance comme un marque-page

Un petit type interne qui garde « les octets qui restent » et offre `take(n)`, `read_u8`, `read_u16`,
`read_u32`, `read_i64` et `is_empty`. Chaque lecture renvoie `None` s'il ne reste pas assez d'octets.

- Avantages : la règle « ne jamais lire hors du tableau » est écrite à **un seul endroit**
  (`split_at_checked`) et testée seule. Le chargeur se lit dans l'ordre exact de la spec.
- Inconvénients : un type de plus, et un lifetime (`'a`) à comprendre, puisque le lecteur emprunte les
  octets.

### Sous-choix — Où produire `BadHeader`

- Soit chaque ligne convertit son erreur (`.ok_or(Fault::BadHeader)?` partout) ;
- soit une fonction interne `parse` renvoie `Option<Program>`, et `load` fait la conversion **une
  seule fois** (`parse(bytes).ok_or(Fault::BadHeader)`).

## 3. Ce que l'assistant IA a proposé

L'assistant a proposé l'option B, avec un `Reader` en `pub(crate)` (invisible des autres crates), le
découpage `load` / `parse` avec une seule conversion vers `BadHeader`, et ces garde-fous contre
l'allocation non bornée :

- `const_count` est comparé à 4095 **avant** de réserver la place des constantes (au plus 32 Ko) ;
- `code_len` est comparé à 1..=65 535, et le code n'est copié (`to_vec`) qu'après que `take` a vérifié
  que les octets existent vraiment. Un fichier de 10 octets qui annonce 4 milliards d'octets de code
  ne déclenche donc aucune allocation.

Il a d'abord expliqué trop de notions d'un coup (octets, hexadécimal, big-endian, `Result`, conception)
et j'ai été perdu. Je lui ai demandé de reprendre une idée à la fois, avec des analogies Java.

Ce que j'ai vérifié : 12 tests unitaires passent (lecteur et chargeur : programme minimal, constantes
dont une négative, mauvaise signature, version, flags, octet en trop, code vide, trop de constantes,
et **toutes** les troncatures possibles d'un fichier valide). L'assistant a aussi rejoué le corpus
officiel (65 programmes) avec un test temporaire : 62 se chargent et les 3 cas `header` du corpus sont
bien refusés en `BadHeader`.

## 4. Ce que nous avons décidé, et pourquoi

Option B, avec la conversion unique dans `load`. Le chargeur se lit ligne à ligne dans l'ordre de
l'annexe A, ce qui le rend facile à défendre et à modifier (spec v2). La sûreté mémoire ne repose que
sur `Reader::take`, petit et testé. Le `?` sur les `Option` rend chaque contrôle d'une ligne.

`Program` garde ses champs privés et expose `constants()` et `code()` en lecture seule : une fois
chargé, un programme ne peut pas être modifié par erreur par la VM.

## 5. Ce qui nous ferait changer d'avis

- Si la spec v2 demandait de distinguer plusieurs erreurs de chargement : renvoyer `Option` ne
  suffirait plus, il faudrait que `parse` renvoie directement un `Result` avec la bonne faute.
- Si le fuzzing (bonus) trouvait une entrée qui fait paniquer ou allouer démesurément le chargeur.
- Si un autre groupe, à J4, refusait un fichier que nous acceptons (ou l'inverse).

## Historique

- 2026-11-22 : création, avec le commit du chargeur.
