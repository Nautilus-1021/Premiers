# Optimisations de l'algorithme de recherche de nombres premiers

Pistes pour accélérer `run_search` (`primes_rust/src/main.rs`) **sans radicalement changer
l'algorithme** : on garde la division par essais, les threads scopés, la collecte dans
l'`AtomicVec`, le tri et la passe de vérification.

L'algorithme actuel, pour chaque nombre impair `n <= limite` :

```rust
for divi in (3..=sqrt(n)).step_by(2) {
    if n % divi == 0 { /* composé */ }
}
```

## 1. Ne diviser que par des nombres premiers — le plus gros gain (~3 à 4× sur la boucle interne)

Aujourd'hui, chaque candidat est testé contre **tous les impairs** jusqu'à √n.
Pour une limite de 10⁷, √n ≈ 3163, soit ~1581 diviseurs, alors qu'il n'y a que
**446 nombres premiers** ≤ 3163.

- Précalculer une fois (en séquentiel, coût négligeable) la liste des premiers ≤ √limite.
- Dans la boucle parallèle, ne tester la divisibilité que par cette liste.

Les pires cas (les nombres premiers eux-mêmes) passent alors d'environ 1581 à 446
divisions, soit ~3,5× moins d'opérations de modulo sur la partie dominante du coût.

## 2. Roue de factorisation mod 30 (~1,9× moins de candidats)

La boucle ne teste déjà que les impairs (50 % des nombres). En ne retenant que les
résidus **1, 7, 11, 13, 17, 19, 23, 29 mod 30** (8 sur 30, soit 26,7 %), on élimine
d'emblée les multiples de 2, 3 et 5.

Mise en œuvre : remplacer `fetch_add(2)` par un compteur sur l'indice `k` de la roue et
calculer `n = roue(k)` (incréments cycliques `4, 2, 4, 2, 4, 6, 2, 6` en partant de 7).
`roue(k)` étant croissante, `fetch_add(1)` convient et chaque `k` donne un candidat unique.

Combiné au point 1 : jusqu'à ~6-7× moins d'opérations de modulo au total.

## 3. Réduire la contention sur le compteur partagé

- `fetch_add(2, SeqCst)` à chaque itération sérialise tous les threads sur une seule
  ligne de cache. Distribuer des **blocs** (`fetch_add(2 * 256)` une fois, puis itérer
  localement dans le bloc) divise le trafic atomique par 256. `Relaxed` suffit : on a
  besoin d'unicité des candidats, pas d'ordre mémoire.
- Le `progress.current.store(...)` exécuté à **chaque** candidat écrit la même ligne de
  cache depuis tous les threads (ralentissement réel, lié à la TUI) : ne le mettre à jour
  que tous les ~1024 candidats.

## 4. Pousser les résultats par lots

Chaque `primes.push()` acquiert le `RwLock` en lecture. Bufferiser les premiers trouvés
localement dans chaque thread, puis les pousser par paquets, réduit fortement les prises
de lock et les réallocations.

## 5. Alléger la phase de vérification

Elle parcourt aujourd'hui **tous les impairs** jusqu'au plus grand premier avec division
par essais — presque aussi coûteuse que la génération elle-même. Sans toucher au
générateur :

- ne vérifier que la primalité des nombres trouvés (coût ~π(n) au lieu de ~n), ou
- faire la vérification par **crible d'Ératosthène** (O(n log log n)) uniquement pour
  cette phase.

## 6. Détails mineurs

- Remplacer le `sqrt()` flottant par une comparaison `divi * divi <= n` (en `u64` pour
  éviter le débordement près de `u32::MAX`) : le modulo domine, mais le `sqrt` flottant
  par candidat a un coût.
- `sort_unstable` sur ~664 k éléments (pour 10⁷) est négligeable devant la génération.
- Hors algorithme mais significatif : compiler avec `target-cpu=native` — le modulo est
  très sensible au débit de l'unité de division.

## Récapitulatif

| Amélioration | Gain estimé | Changement structurel |
|---|---|---|
| Diviseurs = premiers ≤ √n uniquement | ~3,5× sur la boucle interne | faible (précalcul + liste) |
| Roue mod 30 pour les candidats | ~1,9× moins de candidats | faible (compteur sur indice de roue) |
| Blocs pour le compteur atomique + `Relaxed` | moins de contention | faible |
| Mise à jour de la progression par paliers | moins de contention (TUI) | faible |
| Push par lots dans l'`AtomicVec` | moins de locks | moyen |
| Vérification par crible ou sur les seuls trouvés | ~2× sur le temps total | moyen (vérification seule) |

Les points 1 + 2 sont ceux qui changent vraiment la donne tout en restant de la
« division par essais classique » ; les points 3 et 4 sont des gains de parallélisme
gratuits. Aucun ne remet en cause l'architecture (threads scopés, `AtomicVec`, tri,
vérification).
