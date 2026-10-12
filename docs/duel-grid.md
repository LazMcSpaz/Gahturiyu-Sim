# The duel grid

Every weapon (Common, its usual material) against every armour set (the items design, 0.2 item 10). Made by `cargo run --release --bin duel_grid > docs/duel-grid.md`; a report for tuning, not a test.

- **Seconds to down**: an average wielder (Strength 35, skill 30) against someone in that armour who doesn't fight back (median of 3; `-` if not within the time).
- **Wins of 8**: the weapon against a Common Forgeiron longsword, both wearing that armour, both fighting.

## Seconds to down

| Weapon | none | cloth | leather + padding | mail + coat | scale | plate | stone |
|---|---|---|---|---|---|---|---|
| Knife | 125 | 146 | 182 | 356 | 233 | - | 214 |
| Dirk | 105 | 118 | 140 | 250 | 181 | - | 166 |
| Hatchet | 33 | 33 | 42 | 100 | 42 | 243 | 71 |
| Short sword | 33 | 33 | 42 | 82 | 43 | 137 | 46 |
| Cutlass | 24 | 29 | 34 | 49 | 36 | 90 | 41 |
| Longsword | 19 | 27 | 29 | 28 | 23 | 43 | 28 |
| Greatsword | 40 | 42 | 48 | 54 | 48 | 62 | 48 |
| Battle axe | 21 | 26 | 42 | 48 | 31 | 58 | 31 |
| Sap | 420 | 438 | - | 472 | 438 | - | - |
| Club | 38 | 46 | 74 | 46 | 44 | 74 | - |
| Mace | 28 | 28 | 45 | 29 | 28 | 38 | 294 |
| War-pick | 62 | 72 | 139 | 86 | 36 | 36 | 48 |
| Stone maul | 21 | 21 | 30 | 28 | 28 | 32 | 109 |
| Knuckle guards | 148 | 148 | 182 | 103 | 69 | 160 | - |
| Spear | 78 | 78 | 105 | 273 | 234 | 384 | 159 |
| Short spear | 33 | 34 | 43 | 133 | 50 | 213 | 57 |
| Trident | 57 | 66 | 109 | 152 | 107 | 329 | 82 |
| Glaive | 36 | 36 | 40 | 62 | 40 | 83 | 40 |
| Weighted net | - | - | - | - | - | - | - |
| Staff | 206 | 244 | 553 | 263 | 375 | 491 | - |
| Short bow | 429 | 443 | - | - | - | - | - |
| Longbow | 186 | 262 | - | - | - | - | - |
| Crossbow | 45 | 52 | 55 | 79 | 75 | 117 | 79 |
| Hand crossbow | 354 | 513 | - | - | - | - | - |
| Sling | 464 | 580 | - | - | - | - | - |
| Throwing knives | 89 | 111 | 185 | 353 | 229 | - | 186 |
| Javelin | 33 | 33 | 63 | 143 | 131 | 421 | 113 |

## Wins of 8 against a longsword

| Weapon | none | cloth | leather + padding | mail + coat | scale | plate | stone | Best in |
|---|---|---|---|---|---|---|---|---|
| Knife | 0 | 0 | 0 | 0 | 0 | 0 | 0 | never |
| Dirk | 0 | 0 | 0 | 0 | 0 | 0 | 0 | never |
| Hatchet | 1 | 1 | 0 | 0 | 1 | 0 | 0 | none, cloth, scale |
| Short sword | 0 | 0 | 0 | 0 | 0 | 0 | 0 | never |
| Cutlass | 2 | 1 | 1 | 1 | 0 | 0 | 0 | none |
| Longsword | 3 | 4 | 1 | 1 | 2 | 3 | 1 | cloth |
| Greatsword | 2 | 2 | 5 | 4 | 4 | 4 | 5 | leather + padding, stone |
| Battle axe | 6 | 5 | 6 | 6 | 6 | 4 | 5 | none, leather + padding, mail + coat, scale |
| Sap | 0 | 0 | 0 | 0 | 0 | 0 | 0 | never |
| Club | 1 | 1 | 0 | 1 | 0 | 1 | 0 | none, cloth, mail + coat, plate |
| Mace | 2 | 2 | 1 | 4 | 3 | 3 | 1 | mail + coat |
| War-pick | 4 | 4 | 3 | 5 | 6 | 6 | 2 | scale, plate |
| Stone maul | 5 | 4 | 4 | 5 | 4 | 5 | 1 | none, mail + coat, plate |
| Knuckle guards | 2 | 4 | 0 | 1 | 1 | 0 | 0 | cloth |
| Spear | 8 | 8 | 8 | 8 | 7 | 5 | 7 | none, cloth, leather + padding, mail + coat |
| Short spear | 1 | 1 | 0 | 1 | 0 | 0 | 0 | none, cloth, mail + coat |
| Trident | 2 | 2 | 1 | 0 | 0 | 0 | 0 | none, cloth |
| Glaive | 8 | 8 | 8 | 8 | 8 | 7 | 8 | none, cloth, leather + padding, mail + coat, scale, stone |
| Weighted net | 0 | 0 | 0 | 0 | 0 | 0 | 0 | never |
| Staff | 0 | 0 | 0 | 1 | 0 | 0 | 0 | mail + coat |
| Short bow | 0 | 0 | 0 | 0 | 0 | 0 | 0 | never |
| Longbow | 0 | 0 | 0 | 0 | 0 | 0 | 0 | never |
| Crossbow | 0 | 0 | 0 | 0 | 0 | 0 | 0 | never |
| Hand crossbow | 0 | 0 | 0 | 0 | 0 | 0 | 0 | never |
| Sling | 0 | 0 | 0 | 0 | 0 | 0 | 0 | never |
| Throwing knives | 0 | 0 | 0 | 0 | 0 | 0 | 0 | never |
| Javelin | 1 | 0 | 0 | 0 | 0 | 0 | 0 | none |

Not yet in these fights: the strength each weapon wants (B1), sneak attacks, tight spaces, and fighting in the dark. Ranged weapons start 12 m off, hand weapons close.
