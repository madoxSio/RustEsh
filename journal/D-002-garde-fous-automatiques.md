# D-002 — Faire respecter les règles du sujet par des outils automatiques

| Champ    | Valeur                                  |
|----------|-----------------------------------------|
| ID       | D-002                                   |
| Date     | 2026-11-21                              |
| Auteurs  | Mahamadou                               |
| Statut   | adoptée                                 |
| Commits  | `563c1af`, `f37ab55`, `0313ba9`         |

## 1. Contexte

Le sujet impose des règles strictes : zéro `unsafe`, aucun `unwrap()`, `expect()` ni `panic!()` dans
le moteur et le décodeur (invariant 5), `cargo clippy -- -D warnings` et `cargo fmt` impeccables, et
une CI dès le jalon J1. Un `panic!` ou un `unsafe` peut faire tomber la note à 0–4/20. Avec plusieurs
personnes et un assistant IA qui propose du code, il faut savoir comment on s'assure que ces règles
sont respectées en permanence.

## 2. Options envisagées

### Option A — Vigilance et relecture

- Avantages : rien à configurer.
- Inconvénients : repose sur l'attention de chacun. Un `unwrap()` ou un `v[i]` passe facilement en
  relecture, surtout dans du code proposé par une IA, et on ne le découvre qu'en soutenance.

### Option B — Lints du compilateur, formatage automatique et CI

- Avantages : le compilateur refuse le code interdit avant même qu'il soit commité, et GitHub vérifie
  chaque push. La règle est écrite une fois dans le `Cargo.toml` racine et s'applique aux quatre crates.
- Inconvénients : un peu de configuration, et il faut écrire le code autrement (par exemple
  `v.get(i)` au lieu de `v[i]`), ce qui est plus verbeux.

## 3. Ce que l'assistant IA a proposé

L'assistant a proposé l'option B avec cette liste précise :

- `unsafe_code = "forbid"` (erreur que personne ne peut désactiver localement) ;
- `unwrap_used`, `expect_used`, `panic` et `indexing_slicing` en `deny` (erreur de compilation) ;
- un `clippy.toml` qui autorise ces constructions dans les tests seulement, comme le tolère le sujet ;
- un `rustfmt.toml` (édition 2024, lignes de 100 caractères) ;
- une CI GitHub Actions qui lance `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
  -- -D warnings` et `cargo test --workspace` à chaque push sur `main` et à chaque pull request.

Dans une première version, il avait aussi mis `arithmetic_side_effects` (qui signale toute addition
pouvant déborder). Il l'a retiré lui-même après avoir constaté que le code d'exemple généré par cargo
(`left + right`) aurait fait échouer la CI. On la réévaluera pour la VM, où l'arithmétique doit être
enveloppante (`wrapping_add`, etc.).

Ce que j'ai vérifié : j'ai ajouté une fonction `pub fn premier(v: &[i64]) -> i64 { v[0] }` dans `vm`.
Clippy l'a refusée avec `error: indexing may panic`, puis l'a acceptée une fois réécrite en
`v.first().copied()`, qui renvoie un `Option<i64>`. La CI passe au vert sur GitHub.

## 4. Ce que nous avons décidé, et pourquoi

Option B. Ces règles sont des critères éliminatoires du barème, donc elles doivent être garanties par
un outil et pas par la mémoire de chacun. Le niveau `forbid` pour `unsafe` traduit le « sans exception »
du sujet. La CI protège `main` : personne ne peut y casser le formatage, les lints ou les tests sans
que ça se voie.

## 5. Ce qui nous ferait changer d'avis

- Si une règle nous obligeait à contourner le code de façon artificielle sans gain de sûreté réel : on
  la passerait en `warn` et on le documenterait ici.
- Si la CI devenait trop lente au point qu'on évite de pousser souvent.
- Si le sujet ou la spec v2 assouplissait une des contraintes.

## Ce qui n'a pas marché

- J'ai d'abord ajouté `[lints] workspace = true` dans les crates sans définir `[workspace.lints]` dans
  le `Cargo.toml` racine. Cargo a refusé avec `workspace.lints was not defined`. Ça m'a fait comprendre
  le mécanisme : la racine **définit**, chaque crate **choisit d'hériter**.
- Le fichier a d'abord été commité sous le nom `clipppy.toml` (trois « p »). Clippy l'ignorait sans
  aucune erreur, donc l'exception pour les tests ne s'appliquait pas. Repéré en relecture du commit,
  corrigé par un renommage. Leçon : un fichier de configuration mal nommé échoue en silence.

## Historique

- 2026-11-21 : décision prise, lints, rustfmt et CI mis en place.
