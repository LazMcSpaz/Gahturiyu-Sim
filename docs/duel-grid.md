# The duel grid

Written by `cargo test --release --test duel_grid -- --ignored` (items design, 0.2 item 10).
Each cell: the share of 16 duels the weapon won against an average fighter with a Common Forgeiron longsword wearing that armour (draws count half). Both fighters average (Strength, Agility, Toughness 35; 40 in the weapon skills). "in": on the floor of a town building; "sneak": the reference doesn't see the first blow coming. Bows start 14 m off, thrown weapons 9 m, the rest 3 m (1 m from behind).

| Weapon | none out | none out sneak | none in | none in sneak | light out | light out sneak | light in | light in sneak | mail out | mail out sneak | mail in | mail in sneak | plate out | plate out sneak | plate in | plate in sneak |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Knife | 0 | 6 | 0 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Dirk | 6 | 12 | 6 | 12 | 6 | 6 | 6 | 6 | 0 | 0 | 6 | 0 | 0 | 0 | 0 | 0 |
| Short sword | 38 | 62 | 56 | 56 | 28 | 44 | 34 | 44 | 0 | 12 | 25 | 6 | 0 | 0 | 6 | 0 |
| Cutlass | 56 | 69 | 56 | 69 | 44 | 56 | 44 | 56 | 12 | 12 | 12 | 19 | 0 | 0 | 0 | 0 |
| Longsword | 69 | 69 | 69 | 69 | 62 | 56 | 62 | 56 | 44 | 44 | 44 | 44 | 6 | 25 | 6 | 25 |
| Greatsword | 38 | 56 | 6 | 31 | 44 | 56 | 0 | 25 | 25 | 12 | 0 | 6 | 19 | 0 | 0 | 0 |
| Hatchet | 25 | 44 | 62 | 50 | 19 | 31 | 38 | 25 | 6 | 0 | 6 | 0 | 0 | 0 | 0 | 0 |
| Battle axe | 69 | 62 | 6 | 62 | 62 | 50 | 19 | 62 | 50 | 38 | 6 | 19 | 31 | 19 | 0 | 0 |
| Sap | 0 | 50 | 0 | 44 | 0 | 44 | 0 | 44 | 0 | 44 | 0 | 44 | 0 | 44 | 0 | 44 |
| Club | 25 | 44 | 31 | 44 | 12 | 12 | 12 | 19 | 19 | 12 | 19 | 31 | 0 | 12 | 6 | 6 |
| Mace | 50 | 69 | 50 | 62 | 19 | 19 | 25 | 38 | 31 | 38 | 38 | 38 | 25 | 31 | 31 | 31 |
| War-pick | 56 | 62 | 56 | 62 | 38 | 69 | 38 | 69 | 19 | 44 | 19 | 44 | 25 | 50 | 25 | 50 |
| Stone maul | 12 | 69 | 19 | 19 | 6 | 19 | 0 | 12 | 12 | 19 | 0 | 19 | 6 | 19 | 0 | 12 |
| Staff | 31 | 31 | 0 | 0 | 0 | 6 | 0 | 0 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 |
| Knuckle guards | 31 | 56 | 31 | 56 | 12 | 22 | 25 | 28 | 19 | 19 | 19 | 19 | 6 | 6 | 19 | 6 |
| Spear | 100 | 94 | 19 | 31 | 100 | 88 | 19 | 19 | 94 | 69 | 6 | 0 | 100 | 94 | 0 | 0 |
| Short spear | 31 | 44 | 28 | 62 | 19 | 31 | 25 | 44 | 0 | 6 | 19 | 12 | 0 | 0 | 12 | 0 |
| Trident | 31 | 38 | 0 | 25 | 19 | 19 | 0 | 12 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Glaive | 75 | 88 | 12 | 6 | 75 | 81 | 12 | 0 | 62 | 75 | 0 | 0 | 88 | 81 | 0 | 0 |
| Short bow | 38 | 38 | 0 | 0 | 38 | 38 | 0 | 0 | 38 | 38 | 0 | 0 | 44 | 47 | 0 | 0 |
| Longbow | 25 | 28 | 0 | 0 | 25 | 28 | 0 | 0 | 25 | 25 | 0 | 0 | 25 | 19 | 0 | 0 |
| Crossbow | 28 | 31 | 0 | 6 | 22 | 25 | 0 | 0 | 22 | 22 | 0 | 0 | 31 | 19 | 0 | 0 |
| Hand crossbow | 25 | 16 | 0 | 0 | 25 | 16 | 0 | 0 | 28 | 16 | 0 | 0 | 16 | 22 | 0 | 0 |
| Sling | 38 | 47 | 0 | 0 | 38 | 47 | 0 | 0 | 41 | 47 | 0 | 0 | 44 | 44 | 0 | 0 |
| Throwing knives | 6 | 0 | 50 | 50 | 6 | 0 | 31 | 38 | 0 | 0 | 6 | 19 | 0 | 0 | 0 | 0 |
| Javelin | 50 | 75 | 94 | 62 | 44 | 56 | 88 | 56 | 6 | 12 | 25 | 12 | 0 | 0 | 0 | 0 |
| Weighted net | 25 | 31 | 31 | 31 | 12 | 31 | 25 | 25 | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 |

## The best in each case

| Case | Win rate | Weapon |
|---|---|---|
| none out | 100% | Spear |
| none out sneak | 94% | Spear |
| none in | 94% | Javelin |
| none in sneak | 69% | Cutlass, Longsword |
| light out | 100% | Spear |
| light out sneak | 88% | Spear |
| light in | 88% | Javelin |
| light in sneak | 69% | War-pick |
| mail out | 94% | Spear |
| mail out sneak | 75% | Glaive |
| mail in | 44% | Longsword |
| mail in sneak | 44% | Longsword, Sap, War-pick |
| plate out | 100% | Spear |
| plate out sneak | 94% | Spear |
| plate in | 31% | Mace |
| plate in sneak | 50% | War-pick |

## Never the best choice

Knife, Dirk, Short sword, Greatsword, Hatchet, Battle axe, Club, Stone maul, Staff, Knuckle guards, Short spear, Trident, Short bow, Longbow, Crossbow, Hand crossbow, Sling, Throwing knives, Weighted net
