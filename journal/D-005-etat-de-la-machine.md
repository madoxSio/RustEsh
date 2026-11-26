# D-005 — L'état de la machine : tableaux fixes, gaz payé d'avance, calcul enveloppant

| Champ    | Valeur                                                                                   |
|----------|------------------------------------------------------------------------------------------|
| ID       | D-005                                                                                    |
| Date     | 2026-11-26                                                                               |
| Auteurs  | Mahamadou                                                                                |
| Statut   | adoptée                                                                                  |
| Commits  | `ffdf5f3`, `a2e8995`, `3e68835`, `374bb3a`, `e5a6592`, `3c1cbc1`, `b6ef38c`, `a1e47fa`, `1a56fc3` |

## 1. Contexte

Après le chargeur (D-003) et le décodeur (D-004), il faut la machine qui exécute : un `pc`, du gaz,
une pile de 64 valeurs et une mémoire de 256 cases (B.1). Le sujet impose **aucune allocation
pendant un tour** et aucun `panic`. La spec impose aussi l'ordre du cycle (B.2) : décoder, **payer
le gaz**, exécuter, avancer. Un manque de gaz donne `OutOfGas` **sans exécuter** l'instruction.
Enfin, toute l'arithmétique est enveloppante : un dépassement n'est jamais une faute.

## 2. Options envisagées

### Option A — Des `Vec` pour la pile et la mémoire

- Avantages : syntaxe familière (`push`/`pop` de `Vec`, comme une `ArrayList` Java).
- Inconvénients : un `Vec` peut réallouer quand il grandit, ce qui est interdit pendant un tour, et
  il faut vérifier la limite de 64 à la main à chaque `push`.

### Option B — Des tableaux de taille fixe

La pile est un `[i64; 64]` avec un compteur `height`. La mémoire est un `[i64; 256]`. Les deux
sont créés une fois, avec la machine.

- Avantages : aucune allocation possible ; la limite fait partie du type. Une case qui n'existe pas
  donne `None` avec `get`/`get_mut`, qu'on transforme directement en faute (`StackOverflow`,
  `OutOfBounds`).
- Inconvénients : il faut écrire soi-même le petit type `Stack`.

### Sous-choix — Comment payer le gaz

Soustraire puis vérifier qu'on n'est pas en dessous de zéro, ou utiliser `checked_sub`, qui renvoie
`None` si le gaz ne suffit pas, **sans modifier** le compteur.

## 3. Ce que l'assistant IA a proposé

L'assistant a proposé l'option B, une `struct Machine` avec une méthode `step()` qui fait un seul
tour du cycle B.2 (testable instruction par instruction) et une méthode `run()` qui appelle `step()`
jusqu'à `HALT` ou une faute. Il a proposé `checked_sub(...).ok_or(Fault::OutOfGas)?` pour le gaz,
les méthodes `wrapping_*` pour l'arithmétique, un contrôle explicite `b == 0` avant `DIV`/`MOD`, une
fonction `shift_count` pour la règle des décalages hors de `0..=63` (B.6), et une méthode `cell`
qui transforme toute adresse invalide en `OutOfBounds`. Il a fait avancer les instructions par
familles (pile, calcul, comparaisons, bits, décalages, mémoire), avec un commit et des tests à
chaque fois.

Ce que j'ai vérifié : 23 tests dans `vm`, dont les exemples chiffrés de la spec (`i64::MIN / -1`,
`-1 SHR 1`, `-7 SAR 1`), les cas 010 et 063 du corpus (gaz consommé et faute identiques au fichier
attendu), les adresses 256, -1, `i64::MIN`, `i64::MAX`, et le non-paiement du gaz sur `OutOfGas`.

## Ce qui n'a pas marché

- J'ai écrit seul les manipulations de pile, et j'ai dépilé dans le mauvais ordre : `Swap` et `Rot`
  ne changeaient rien. Le test `manipulations_de_pile` l'aurait détecté. J'en retiens la règle :
  **le premier `pop()` donne la dernière lettre de la notation** (`a b -- …` : d'abord `b`).
- J'avais ajouté `machine.rs` sans `mod machine;` dans `lib.rs` : le fichier n'était pas compilé et
  ses tests ne tournaient pas, sans aucune erreur (corrigé dans `3e68835`). Depuis, je compte les
  tests affichés après chaque ajout.

## 4. Ce que nous avons décidé, et pourquoi

Option B et `checked_sub`. La règle « pas d'allocation » est garantie par les types au lieu d'être
une discipline, et chaque faute de la spec correspond à un `None` transformé en `Err`, ce qui est
facile à montrer en soutenance. La séparation `step()` / `run()` permet de tester une instruction à
la fois.

Point de vigilance : les branches `_ =>` de `step()` (renvoie `BadOpcode`) et de `gas_cost()`
(renvoie 0) sont **temporaires**. Elles doivent disparaître quand toutes les instructions seront
écrites, pour que le compilateur signale toute instruction oubliée.

## 5. Ce qui nous ferait changer d'avis

- Si la spec v2 rendait la taille de la pile ou de la mémoire configurable : il faudrait un tableau
  dimensionné au chargement, alloué une seule fois avant le tour.
- Si les mesures (`criterion`) montraient que `step()` coûte trop cher par instruction.
- Si un autre groupe, à J4, obtenait un `gas_used` différent du nôtre sur le même programme.

## Historique

- 2026-11-26 : création, après la famille mémoire.
