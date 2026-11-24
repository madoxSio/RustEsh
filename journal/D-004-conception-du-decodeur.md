# D-004 — Décoder une instruction à la fois, vers un `enum Instruction`

| Champ    | Valeur                                   |
|----------|------------------------------------------|
| ID       | D-004                                    |
| Date     | 2026-11-24                               |
| Auteurs  | Mahamadou                                |
| Statut   | adoptée                                  |
| Commits  | à compléter avec le hash du décodeur     |

## 1. Contexte

Le chargeur (D-003) vérifie le fichier une seule fois. Ensuite, à chaque pas, la machine doit lire
l'instruction située au `pc` : un octet d'opcode, puis 0, 1 ou 2 octets d'opérande (annexe B).
La spec impose un ordre de fautes strict au décodage (B.2) : `PcOutOfRange` si le `pc` sort du
code, puis `BadOpcode` si l'octet n'est pas un opcode connu, puis `Truncated` si l'opérande est
coupé par la fin du code. Elle dit aussi (A.3) qu'il n'y a **aucune analyse préalable** : un
octet invalide ne pose problème que si la machine arrive dessus.

Au début, je confondais chargeur et décodeur. Le découpage qui m'a aidé : le chargeur lit le
fichier une fois, le décodeur lit **une instruction** à chaque pas, l'interpréteur (crate `vm`)
l'exécute.

## 2. Options envisagées

### Option A — L'interpréteur travaille directement sur les octets

Un grand `match` sur l'octet d'opcode dans la boucle de la VM, qui lit lui-même ses opérandes.

- Avantages : un seul endroit, pas de type intermédiaire.
- Inconvénients : lecture des octets et exécution mélangées ; les contrôles `Truncated` se
  répètent dans chaque branche ; impossible de tester le décodage sans faire tourner la VM.

### Option B — Une fonction `decode(code, pc)` qui renvoie un `enum Instruction`

Un `enum` de 43 variantes, dont certaines portent leur opérande (`PushI16(i16)`, `Jmp(i16)`,
`Load(u8)`…), et une méthode `size()` qui donne la taille de l'instruction (1, 2 ou 3 octets).

- Avantages : la VM reçoit une instruction déjà propre, avec un opérande typé ; le compilateur
  oblige à traiter chaque variante dans son `match` ; le décodeur se teste seul, dans la crate
  `pbc`, sans VM.
- Inconvénients : un type de plus, et les opcodes sont écrits à deux endroits (`decode` et la
  documentation des variantes).

### Sous-choix — Décoder tout le programme d'avance, ou à la demande

Décoder tout le code au chargement dans un `Vec<Instruction>` irait contre A.3 (un octet inconnu
jamais atteint ne doit pas faire échouer le programme) et casserait les sauts qui visent le milieu
d'une instruction. Je décode donc à la demande, au `pc` courant.

## 3. Ce que l'assistant IA a proposé

L'assistant a proposé l'option B avec décodage à la demande, et ce code pour respecter l'ordre des
fautes :

1. lire l'opcode avec `code.get(pc).ok_or(Fault::PcOutOfRange)?` ;
2. créer un `Reader` (celui de D-003) sur les octets **après** l'opcode ;
3. faire le `match` sur l'opcode : un opcode inconnu tombe dans `_ => Err(Fault::BadOpcode)` avant
   toute lecture d'opérande, et un opérande manquant donne `.ok_or(Fault::Truncated)?`.

Il a ajouté `read_i16` au `Reader` pour les déplacements de saut, qui sont signés.

Ce que j'ai vérifié : les tests de `pbc` passent (19 au total), dont le décodage complet du
programme d'exemple « carrés » de la spec, un opérande négatif, un `pc` hors du code, un opcode
inconnu, un opérande tronqué, le cas « opcode inconnu en fin de code » (doit donner `BadOpcode`
et non `Truncated`), et un balayage des 256 valeurs d'octet : chacune est soit décodée (les 43
opcodes connus) soit refusée, **sans jamais paniquer**. `clippy` et `fmt` sont propres.

## 4. Ce que nous avons décidé, et pourquoi

Option B, décodage à la demande. La séparation suit les trois étapes que je sais expliquer
(charger, décoder, exécuter), et l'ordre des fautes de B.2 se lit directement dans les trois
étapes de `decode`. Le décodeur reste dans la crate `pbc`, car il dépend du format et pas de la
machine : la VM ne verra jamais un octet brut.

Point de vigilance noté : dans `size()`, la branche `_ => 1` couvre toutes les instructions sans
opérande. Si la spec v2 ajoute une instruction **avec** opérande, il faudra penser à l'ajouter à
`size()`, car le compilateur ne le signalera pas.

## 5. Ce qui nous ferait changer d'avis

- Si la spec v2 changeait la taille des opérandes ou ajoutait des instructions de taille variable.
- Si les mesures de performance (bonus `criterion`) montraient que décoder à chaque pas coûte trop
  cher : on pourrait alors mettre en cache les instructions décodées, en gardant l'ordre des fautes.
- Si un autre groupe, à J4, obtenait une faute différente de la nôtre sur un même programme.

## Historique

- 2026-11-24 : création, avec le commit du décodeur.
