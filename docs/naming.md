# Naming: words and names in the four tongues

Second agent's work, branch `agent2/naming`. This file grows with each stage. This is
the state after **stage 3**. Stage 1: canon inventory, First Speech roots, sound rules,
cognate sets. Stage 2: how each tongue puts words together, the gods and the elements
in all four tongues, what each people calls each people, the words of worship and
rule, and spelling and pronunciation. Stage 3: the names of things (jobs, crafts,
materials, items, buildings, creatures, weather). Next: people, then places.

**Nothing in the game uses this yet.** The game still makes names with
`src/sim/names.rs` (random syllables in each tongue's sounds). The adapter that points
those call sites here comes with the people and place generators (stages 4 and 5).

## The idea

Every word starts as a **root of the First Speech**, the lost ancestor of all four
tongues. Each tongue has an ordered list of **sound rules** that wears a root down its
own way. So one root gives four related words, a new word needs only a root, and
changing a rule changes every word of that tongue at once.

| File | What it is |
|---|---|
| `assets/lang/roots.ron` | the root list: 325 roots, each with its meaning and area of meaning |
| `assets/lang/sounds.ron` | the First Speech's sounds, and each tongue's rule list, with a plain account of every rule |
| `assets/lang/irregular.ron` | words that break the rules, each with its reason |
| `assets/lang/loans.ron` | words one tongue borrowed from another |
| `assets/lang/canon.ron` | everything that already had a name: the inventory |
| `assets/lang/grammar.ron` | each tongue's naming grammar: word order, endings, name endings, stress |
| `assets/lang/sacred.ron` | the gods, the elements, the peoples, the words of worship and rule |
| `src/names/` | the code: `sound.rs` carries out a rule list; `grammar.rs` joins words; `sacred.rs` the gods and peoples; `say.rs` spelling and pronunciation; `mod.rs` reads the data and answers questions |
| `src/bin/lang.rs` | a small tool to read the tongues (below) |
| `docs/glossary.md` | every root in all four tongues (made from the data; a test keeps it in step) |
| `docs/sacred.md` | the gods, elements, peoples and sacred words in all four tongues, each with how to say it (made from the data; a test keeps it in step) |
| `tests/names.rs` | the checks (below) |

Asking (in code): `names::word("sea", Tongue::Horaro)` gives `moa`.
`names::derive("moʻa", Tongue::Qotiro)` runs any First Speech form through a tongue's
rules. Both are plain functions: the same answer every time, no state, no randomness.

Reading (at a terminal):

```
cargo run --release --bin lang -- glossary      every root in all four tongues
cargo run --release --bin lang -- sacred        gods, elements, peoples, sacred words
cargo run --release --bin lang -- pronounce horaro moalanu     how to say a word
cargo run --release --bin lang -- cognates      the sets below
cargo run --release --bin lang -- say moʻa pora try First Speech forms in every tongue
cargo run --release --bin lang -- clashes       roots that sound alike within a tongue
```

## Where the canon came from

`languages.md` and `pantheon.md` are not in the repo. Both were recovered in full from
the chat they were written in (2026-07-29) and followed as written. `architecture.md`
came from Laz for stage 3 and is followed for the buildings.

**The roots here are the language now.** The old Gogìḍu dictionary (the language
tool's export) could never be read back, so the inventory holds the Gogìḍu words that
are *quoted* in the lore: 37 words, 5 bits of grammar, 3 phrases. Everything built on
them is firm and is still checked by the tests. The other roots were made new for this
system, and by Laz's decision (2026-10-09) they **stand as the language and supersede
the old dictionary**: nothing is waiting to be checked against it. If an old word is
ever wanted back, it is one line in `roots.ron` (give the root that word as its
`canon`); the rules and the code don't move.

`canon.ron` also lists: the 18 gods and the Trinity's two names (with the words each is
built from), the four peoples, the tongue's name, Gahturiyu and Turiyu, the names
that were considered and set aside (so they are not reused by accident), the nine
example names from `languages.md`, and the English-only names in the code that still
need native words (22 creatures, 9 materials, Stone Tender, the Overgrowth).

## The First Speech

- **Shape:** every syllable is one consonant and one vowel; a word may start with a
  bare vowel. No consonants together, nothing ends in a consonant.
- **Consonants (18):** `p t ṭ k q ʻ d ḍ g m n s th h r l w y`. The catch `ʻ` only ever
  sits between two vowels.
- **Vowels (6):** `a e i ì o u`.

Why these: the quoted Gogìḍu words use only `g d ḍ t ṭ q h r l th sh y ʻ`. There is
no `p b m n w k s f` in any of them. Roduro is the tongue that changed least, so the
First Speech is Roduro's sounds plus the sounds of the lips and the nose that Roduro
must have lost, because Horaro (`l m n w`) and Ṭaḍoro (`f w`) still have them.

## How each tongue wears a root down

The two branches of `languages.md`: the dry line (Earth, then Fire) and the moist line
(Water, then Air).

**Earth branch (Roduro and Qotiro).** `w` hardens to `g`.

**Roduro** keeps every vowel and the shape of every word. Only the lips and nose go,
pulled back in the mouth: `p → h`, `k → q`, `m → ḍ`, `n → ḍ`, `s → sh`. Result: open
syllables, `ṭ ḍ q ʻ` everywhere. Every canon word comes out exactly.

**Qotiro** hardens and clips.
- Soft sounds harden: `ʻ → q`, `l → r`, `y → d`, `h → x`, `s → x`, `th → t`.
- The curled-back `ṭ ḍ` become r-clusters: `tr-`, `dr-` at the start of a word,
  `-rt-`, `-rd-` inside.
- Stress is on the first vowel; later vowels dull (`e → a`, `i → e`).
- **The last vowel is lost**, so words end hard. In a short word the lost vowel
  colours the one that is left (`a` becomes `e` or `o`).
- In longer words the second vowel is lost too where the consonants can stand
  together (`doruguna → dorgun`).

**Water branch (Horaro and Ṭaḍoro).** The harsh sounds soften: the catch and `q` fall
silent (so vowels meet), `k → h`, `p → f`, `g → w`, `ì → i`.

**Horaro** turns everything liquid: `ṭ → l`, `ḍ → n`, `t → n`, `d → l`, `f → w`,
`s → h`, `th → h`, `y → i`. An `h` between two of the same vowel fades and leaves a
long vowel (`tiki → nihi → nii`). Result: only `l m n w r h`, vowels running together.

**Ṭaḍoro** turns everything to breath: `t → s`, `d → th`, `ṭ → th`, `r → sh`, and the
liquids and nose-sounds thin to glides (`l → y`, `n → y`, `ḍ → y`, `m → w`).
- There is no `o`: it opens into `au` in a word's first syllable and blurs to `e`
  elsewhere.
- Vowels that meet glide together (`ai au ei ae`) or take a breath or glide between.
- A long word sheds its last vowel after `h th s sh` (`ketudu → hesuth`), and its
  middle vowels blur to `e`.

**Borrowed words.** Ṭaḍoro borrows the most. A borrowed word starts from the lender's
finished word, fitted to Ṭaḍoro's consonants; it keeps `u` where the lender had `o`,
which a native word never does. Seven so far (`loans.ron`): mountain, rock, town, law
and tradition from Roduro; iron and fight from Qotiro. Each has its reason beside it.

**Exceptions.** One so far: Roduro's canon word for death, *ṭeyuʻoye*, is far longer
than a root, so the root is its first part (`ṭeyu`) and Roduro's long form stands as
an exception.

## Twenty cognate sets

One root, four tongues. (More in `docs/glossary.md`.)

| Meaning | First Speech | Roduro | Qotiro | Horaro | Ṭaḍoro |
|---|---|---|---|---|---|
| waterway, river, flowing water | pora | hora | por | wora | fausha |
| earth, dirt, the ground | rodu | rodu | rod | rolu | shauthu |
| the sea | moʻa | ḍoʻa | moq | moa | weya |
| fire | ṭaku | ṭaqu | trok | lahu | thahu |
| to twist, to turn; wind | ṭano | ṭaḍo | tron | lano | thaye |
| air, breath | saʻe | shaʻe | xaq | hae | sae |
| a stone | doqo | doqo | doq | loo | thehe |
| island | wahì | gahì | gex | wahi | wahi |
| sky | laʻi | laʻi | req | lai | yai |
| moon | mene | ḍeḍe | men | mene | weye |
| night | yamo | yaḍo | dom | iamo | yawe |
| ray of light | ketho | qetho | ket | heho | hethe |
| tree | ṭeqa | ṭeqa | treq | lea | theya |
| fruit | wuʻe | guʻe | guq | wue | wuwe |
| mother | ama | aḍa | am | ama | awa |
| child | tama | taḍa | tam | nama | sawa |
| house, home | pale | hale | par | wale | faye |
| a people, kind | roʻì | roʻì | roq | roi | shei |
| dream | tìʻa | tìʻa | teq | nia | siya |
| to hunt | lìdì | lìdì | red | lili | yithi |

The example names in `languages.md` are shapes these rules make: `doruguna → Dorgun`
and `qethuruka → Qetruk` in Qotiro, `moʻaṭanu → Moalanu` and `liloma → Liloma` in
Horaro, `ketudu → Hesuth`, `pemaʻi → Fewai` and `taqelika → Saeyih` in Ṭaḍoro.
(Moalanu could even be read as sea-wind: `moa` + `lano`.)

## Stage 2: putting words together

A new compound is made by a tongue's own speakers from its own finished words, so
joining works on finished words and mends the place where they meet
(`grammar.ron`, `src/names/grammar.rs`).

| | Roduro | Qotiro | Horaro | Ṭaḍoro |
|---|---|---|---|---|
| Order | describing word first: `rodu` + `ro`, earth-kind | describing word first | main word first: `roi` + `mawe`, kind of water | main word first |
| Where the words meet | two vowels never touch: the catch goes between; a syllable said twice is said once | consonants may meet, two at most (three if the last two are a stop and r); otherwise an `a` goes between | vowels run together, three at most | a consonant meeting a consonant takes an `e` between; vowels glide |
| Longest compound | 4 syllables; the second word keeps its end | 3; keeps its start | 5; keeps its start | 4; keeps its start |
| A people | `-ro` | `-roq` | `roi-` | `shei-` |
| One who does it | `-qe` | `-ak` | `he-` | `he-` |
| Where it is | `-la` | `-ar` | `la-` | `ya-` |
| Little / great | `-li` / `-ra` | `-ik` / `-rek` | `li-` / `ra-` | `yi-` / `sha-` |
| The highest | `hi-` | `xi-` | `hi-` | `hi-` |
| "of" | `yi` | `di` | `i` | `yi` |

- The Earth tongues put endings *after* a word; the Water tongues put them *before*.
  That one difference makes the two branches look different at a glance.
- **Roduro's rule gives back the canon's own compounds** where they are regular:
  Horahìda (`hora` + `hohìda`), Guʻedeqi, Guʻehiqì, Qotihiqì, Lìdìhoya, Rìthaduya,
  Horaṭaḍo, and Roduro itself. A test holds this.
- `kind`, `small`, `great` come from roots (a people, small, great); `hi-` and `yi` are
  Roduro's own canon words; "one who does it" (First Speech `ke`) and "where it is"
  (`la`) are new.
- **Given-name endings** are in `grammar.ron` as a proposal for stage 4: Roduro men
  `-o -u`, women `-a -i`, either `-e -ì`; Qotiro men end on a hard stop
  (`-k -t -q -x`), women on a soft one (`-n -r -m`), either `-d -g`; Horaro men
  `-u -o`, women `-a -ia`, either `-i -e`; Ṭaḍoro men on a breath (`-th -h -s`), women
  `-ai -a -i`, either `-e -u -sh`.

## Stage 2: the gods, the elements, the peoples

All of it is in `docs/sacred.md`, in every tongue, with how to say each name.

**The gods.** The gods are older than the split of the tongues. So each god's name is
kept as one First Speech form, and each tongue wears it down by its own rules: the
"same god-names eroded differently" that `languages.md` asks for. The Roduro form is
the canon name exactly (a test holds all 20). One exception: *Yohyeʻ* is itself worn
down past Roduro's rules, so the other tongues start from its full form, Yoheʻiʻa.

| Shown as | Roduro (canon) | Qotiro | Horaro | Ṭaḍoro |
|---|---|---|---|---|
| the World's Artery | Horahìda | Porped | Worawila | Faushefith |
| the Enduring Earth | Dodìṭo | Dodert | Lolilo | Thauthith |
| the Wounding Lance | Qotisho | Qotex | Honiho | Hausis |
| the Wayfarer's Stave | Rìthaduya | Retadud | Rihaluia | Shethetheya |
| the Most Radiant | Hiqethoru | Xiktor | Hihehoru | Hihethesh |

- **English names shown first** are the meanings `pantheon.md` gives ("the World's
  Artery"). The Trinity together is "the Highest" (Hiro).
- The Trinity's mark `hi-` survives in every tongue (`xi-` in Qotiro).

**The elements.** Earth `rodu` is canon. Water, fire and air are new roots (see the
dictionary note above).

**What each people calls each people.** Each speaker uses its own word for the element
and its own mark for "a people".

| Speaker | the Earth-kind | the Fire-kind | the Water-kind | the Air-kind |
|---|---|---|---|---|
| Roduro (canon) | Roduro | Qotiro | Horaro | Ṭaḍoro |
| Qotiro | Rodroq | **Trokroq** | Magroq | Tronroq |
| Horaro | Roirolu | Roilahu | **Roimawe** | Roilano |
| Ṭaḍoro | Sheishauthu | Sheithahu | Sheiwawe | **Sheisae** |

- The Roduro row comes out of the rules as the canon four.
- **The canon four stay the names the game shows.** The other twelve are for
  dialogue: a Qotiro says *Trokroq* of her own people.
- The Roduro name the others by a likeness (spear-kind, waterway-kind, wind-kind). The
  other three name everyone by the element itself, and all call the Air-kind after
  the wind, except the Air-kind themselves, who say air.

**Words of worship and rule** (24, from roots): god, rite, omen, offering, blessing,
curse, vow, shrine, temple (god-house), priestess, high priestess (great priestess),
elder, speaker (one who speaks), arbiter (one who judges), ancestor, soul, spirit,
fate, death, tomb, the dead, dream, holy, holy place. The game has no named saints,
rites or holy places yet, so there was nothing more to name.

## Stage 2: spelling and saying

**One spelling per tongue**, with only the canon's marks.

| Letter | Sound |
|---|---|
| `ṭ ḍ` | t and d with the tongue curled back (Roduro only) |
| `ì` | the i of "sit" (Roduro only); plain `i` is "ee" |
| `ʻ` | the catch in "uh-oh" (Roduro only, and only between vowels) |
| `q` | a k made far back in the throat (Roduro, Qotiro) |
| `x` | the ch of "loch" (Qotiro only) |
| `th`, `sh` | one sound each, as in "thin" and "ship" |
| `r` | tapped in Roduro and Horaro, rolled hard in Qotiro |
| `a e i o u` | "ah", "eh", "ee", "oh", "oo" |
| `ai au ei` | Ṭaḍoro glides: "eye", "ow", "ay" |
| a vowel twice (`aa`, `ii`) | Horaro long vowel: said twice |

**Plain letters** (`names::ascii`): `ṭ ḍ ì` lose their marks and the catch becomes an
apostrophe (`Guʻehiqì` is `Gu'ehiqi`); `names::file_name` gives letters only.

**How to say it** (`names::pronounce(word, tongue)`): plain English syllables, the loud
one in capitals. Stress is next to last in Roduro and Horaro, first in Qotiro, last in
Ṭaḍoro. `Qotisho` is "koh-TEE-shoh", `Dorgun` "DOR-goon", `Moalanu` "moh-ah-LAH-noo",
`Hesuth` "heh-SOOTH". The hints agree with the ones written in `pantheon.md` except
where that file is not consistent with itself (it writes `do` as "doo" in Dodìṭo and
as "doh" elsewhere; here it is always "doh").

**The font** (`assets/DejaVuSans.ttf`) has every letter used, capitals included. A
test checks every word, god and people's name against it.

## Stage 3: the names of things

**The full table is `docs/things.md`** (311 things; made by
`cargo run --release --bin lang -- things`). The list itself is
`assets/lang/things.ron`.

**How a thing gets its name.** Nobody types a native name in. Each entry is the
English name the game already shows and a short *recipe* saying which roots the
native name is built from; the tongue's own rules (stage 2) do the rest:

| Recipe | Means | Example |
|---|---|---|
| `stone` | a root | Rock: Roduro *Ḍoqa* |
| `black+water` | the first describes the second | Ink: Qotiro *Kemag* |
| `forge:agent` | an ending: one who does it (also `place`, `small`, `great`) | Smith: Qotiro *Dortak* |
| `board/=Bronze` | two words, "A of B"; `=` is another thing's name | Bronze ingot: *Pirt di trerged* |
| `*turiyu` | an old First Speech name, worn down by each tongue | Turiyu: Qotiro *Tured* |
| `@qotihiqi` | a god | the Overgrowth: *Quʻa yi Qotihiqì* |

So changing a name means changing its recipe, and a change to a root or a sound rule
flows through every name built on it.

**Whose word it is.** Laz's rule: a thing has a native name in the tongue of the
tradition it belongs to, and things every people has get all four.

- **One people's thing** has one native name, in that people's tongue, and everyone
  else uses that word. Ownership is not my opinion wherever the game already says:
  a *material* belongs to its making tradition in `materials.rs` (grown = Roduro,
  fire-made = Qotiro, sea and shore = Horaro, written and carried = Ṭaḍoro); a *made
  thing* belongs to the people whose material it is usually made in (a longsword is
  usually Forgeiron, so it is Qotiro; a harpoon is usually Nacre, so Horaro). A test
  holds both. Buildings follow `architecture.md`; creatures follow where they live;
  jobs and town places follow the craft they serve.
- **Everything else** has a word in each tongue, built by the same recipe from that
  tongue's own roots.

A few from each people (the game shows the English; the native name sits under it):

| English | Whose | Native name | Say it | Word for word |
|---|---|---|---|---|
| Stone Tender | Roduro | **Leḍaqe** | leh-DAH-keh | tend-er |
| Ringstone | Roduro | **Goledoqo** | goh-leh-DOH-koh | ring-stone |
| Grown home | Roduro | **Quʻahale** | koo-ah-HAH-leh | grow-house |
| Ridgehound | Roduro | **Guriḍoro** | goo-ree-DOH-roh | ridge-hound |
| Smith | Qotiro | **Dortak** | DOR-tak | forge-er |
| Forgeiron | Qotiro | **Dortaged** | DOR-tah-ged | forge-iron |
| Temple | Qotiro | **Otpar** | OT-par | god-house |
| Dustrunner | Qotiro | **Xendirak** | KHEN-dee-rak | sand-run-er |
| Fisher and diver | Horaro | **Henume** | heh-NOO-meh | dive-er |
| Seareed | Horaro | **Ionomoa** | ee-oh-noh-MOH-ah | sea-reed |
| Stilt house | Horaro | **Walewohu** | wah-leh-WOH-hoo | pillar-house |
| Deepcoil | Horaro | **Hihonuwu** | hee-hoh-NOO-woo | deep-eel |
| Scribe | Ṭaḍoro | **Hesefi** | heh-seh-FEE | write-er |
| Tentsilk | Ṭaḍoro | **Sesafau** | seh-sah-FOW | tent-silk |
| Scholar's tent | Ṭaḍoro | **Fauwase** | fow-wah-SEH | silk-tent |
| Silk Mother | Ṭaḍoro | **Awasesa** | ah-wah-seh-SAH | silk-mother |

And things everyone has:

| English | Roduro | Qotiro | Horaro | Ṭaḍoro | Word for word |
|---|---|---|---|---|---|
| Bread | Ḍiha | Mip | Miwa | Wifa | bread |
| Boat | Goqa | Gok | Woha | Wauha | boat |
| The sea | Ḍoʻa | Moq | Moa | Weya | sea |
| Merchant | Tuquqe | Tukak | Henuu | Hesuhu | trade-er |
| Inn | Ḍatihale | Metpar | Walemani | Fayewasi | guest-house |
| Charcoal | Qeḍeqaho | Kemkop | Hawoheme | Hafehewe | black-timber |
| Boots | Guṭaheqa | Gurtapek | Wehawula | Fehawutha | foot-shell |
| thunderstorm | Ḍururogu | Drurkrog | Horowunuru | Haushewuyu | thunder-storm |

**What is covered.** Every name in the game's own lists, checked by a test so a new
job or item can't be added without a name: 37 jobs, 9 crafts, 41 materials and goods,
19 weapons and their shot, 15 pieces of armour and clothing, 16 pieces of gear, 19
foods, herbs and draughts, 8 named treasures, 13 books, 33 places in a town, 7 work
stations, 13 things the squad can build, 19 creatures, 5 fish and sea beasts, 19
kinds of weather, and 19 everyday things. Also 18 buildings and parts of buildings
from `architecture.md`, and the Overgrowth (named for Qotihiqì, as `pantheon.md` says).

**Left out, and why.**
- **A spell's notes, rite or scroll** (43 items) is named for its spell, and spells
  have no native names yet. The head words are in the table (*Notes*, *Rite*,
  *Scroll*, *Manual*), ready for when they do.
- **Made variants** ("Nacre helm", "Edgeglass spear") are a material and a form put
  together. Both halves are in the table; armour and clothing are deliberately named
  without a material in the word (*head-shield*, *body-cloth*) so the same word serves
  whatever it is made of. Putting the two together is one small function, due with the
  naming module in stage 5.
- **Fighting and magic skills, services, grades** ("Blade", "Felt magic", "crude").
  Not asked for; say if they should have words.

**Three roots changed, one exception added.** Reading the whole table turned up words
that look rude to an English reader. There is now a list of words to keep clear of
(`assets/lang/avoid.ron`) and a test that holds every root, sacred word and thing to
it; the people and place generators will use the same list.

| Was | Now | Why |
|---|---|---|
| rite `kake` → Qotiro *Kak* | `kase` → *Kax* | reads as a rude word |
| fate `nipo` → Qotiro *Nip* | `nìpo` → *Nep* | reads as a slur |
| scale `sìṭa` → Roduro *Shìṭa* | `seṭa` → *Sheṭa* | reads as a rude word |
| to journey (canon *rìtha*) → Ṭaḍoro *Shitha* | exception: *Shetha* | reads as a rude word; the canon Roduro word is untouched |

The last one also touched one god's name **in Ṭaḍoro only**: the Wayfarer's Stave
was *Shithetheya* and is now *Shethetheya*. The canon name, *Rìthaduya*, is unchanged.

**Fifteen roots added** for things the list needed and the stock lacked: head, hand,
foot, body, neck, scale, shield, bow, board, paper, thing, to throw, to leap, to work,
to shut. 340 roots now, 38 of them canon.

## What the tests hold (`tests/names.rs`)

- 300 to 400 roots, every area of meaning covered, every root a proper First Speech form.
- **The canon comes back:** every root built on a Gogìḍu word gives that word by the
  Roduro rules; every plain Gogìḍu word in the inventory is some root's word or a
  listed exception; every god is built from words in the inventory.
- **Each tongue keeps to its sounds:** Roduro open syllables and its thirteen
  consonants; Qotiro ends hard, clusters of two at most; Horaro only `l m n w r h`
  and open syllables; Ṭaḍoro only `h s sh th f w y`, no consonants together.
- **Roots stay apart:** no two roots share a word in Roduro, Horaro or Ṭaḍoro. Qotiro,
  which loses its last vowel, has 16 groups of sound-alikes (`req` is both sky and
  long); the test keeps them under 19. Compounds tell them apart.
- Everything is a plain function, and `docs/glossary.md` matches the data.

Stage 2 adds:
- **The gods come back:** all 20 canon names from their First Speech forms; each
  keeps to its tongue's sounds in the other three; no two gods share a name in any
  tongue; the Trinity's mark shows in every tongue.
- **The peoples:** the Roduro row is the canon four; all sixteen names differ; each
  row carries its speaker's mark.
- **Compounds keep each tongue's shape** for any two roots and any ending, and
  Roduro's joining gives back the canon compounds.
- **Every word can be said and spelled:** a hint with a loud syllable, a plain-letter
  form, and every letter in the game's font. `docs/sacred.md` matches the data.

Stage 3 adds:
- **Everything the game names has a native name:** every job, good, material, craft,
  item, made form, town place, station, base building, creature and kind of weather
  in the code has an entry (over 300 names, read from the code's own lists).
- **Whose it is follows the game:** a material's people is its making tradition; a
  made thing's people is that of its usual material.
- **Every name holds together:** it keeps its tongue's sounds, runs to four beats a
  word at most (five in Horaro, which is all vowels), no two things share a name in
  any tongue, and nothing reads as a rude word in English. `docs/things.md` matches
  the data.

Not yet: the "guess the tongue from the sounds" check and the lookalike check. They
belong to generated names (stages 4 and 5).

## Decided without asking (veto any of these)

1. **The First Speech's sound set**, and that Roduro lost its lip and nose sounds
   (it rests on the 45 Gogìḍu words I can see).
2. **The rule lists above.** They are a first cut for the ear. To change a tongue's
   sound, change its list in `sounds.ron`.
3. **Qotiro may have sound-alike roots**; the other three may not.
4. **Ṭaḍoro has no `o`.**
5. **All the new roots** (302 now). Where they echo a real language (`hale` house,
   `limu` kelp) it is by choice, for a faint familiar ring; say if that should go.
   (Laz, 2026-10-09: these stand as the language.)
6. **Which words Ṭaḍoro borrowed**, and from whom.
7. **Turiyu** is recorded as the worn-down "holy beast" inside Gahturiyu
   (`hotorì gayugo`). That is a reading, not something written down anywhere.
8. Spelling: `Qotihiqì` as in `pantheon.md` (built on `hiqì`), not `Qotihìqì`.

From stage 2:

9. **The gods' names in the other three tongues are the Roduro names worn down**, not
   new names built from each people's own words. Any single one can be overridden
   (`own` in `sacred.ron`) if a people should have its own name for a god.
10. **The gods' English names** are the meanings from `pantheon.md`. Some are long
    ("Millennium-into-Dust"); say which should be shorter.
11. **The peoples keep their canon names on screen**; the other twelve names are for
    speech. And those twelve are plain element names ("fire-kind"), not nicknames.
12. **Word order:** Earth tongues describe first, Water tongues name the main thing
    first, and put their endings in front.
13. **The endings** for a doer, a place, little and great, and the name endings for
    men and women in each tongue.
14. **Horaro `y`** melts into a neighbouring `i` (one small rule added to stage 1's list).

From stage 3:

15. **Everyone uses the owner's word for an owned thing**, unchanged (a Roduro says
    *Dortaged* for Forgeiron). The alternative is each people bending it to their own
    mouth, as Ṭaḍoro already does with its borrowed words.
16. **Who owns what** where the game doesn't say: jobs, town places and creatures.
    Notably Teacher, Scribe, Alchemist, Exchanger, Drifter, coin and letters are
    Ṭaḍoro; Mason is Qotiro (the Roduro grow, they don't build); marsh, hill and
    mountain beasts are Roduro; the plateau's are Qotiro; everything of the shore and
    sea is Horaro; the silk spinners are Ṭaḍoro (Tentsilk is theirs).
17. **A job is usually "one who does X" from a single root** (Tailor is "cloth-er",
    Carpenter "timber-er"), to keep names to three or four beats. The English name
    carries the detail ("Woodcutter and quarrier" is just *Ṭeqiqe*, "cutter").
18. **Proposed English names for the buildings**, which the code only has as variant
    names: Grown home, Stilt house, Tier house, Temple, Temple-fortress, Gatehouse,
    Sun-disc, Scholar's tent, and the parts (Year-rings for the banding, Stilt, Deck,
    Woven dome, Awning, Stone dock, Tent pole, Script panel). No existing English name
    was changed; I found none that read as a placeholder.
19. **Gold is "sun-ore" and Bronze "gold-iron"** in Qotiro; **the Overgrowth is "the
    growth of Qotihiqì"**; **Turiyu** keeps its name and wears down in the other
    tongues like a god's name (*Tured*, *Nuriu*, *Sushiyu*).
20. **The three root changes and the Ṭaḍoro exception** above.

## Lines added to shared files

| File | Line |
|---|---|
| `src/lib.rs` | `pub mod names;` (and one line of the comment above it) |

`src/bin/lang.rs` is a new program; Cargo finds it by itself, so `Cargo.toml` is
untouched. Nothing else outside `assets/lang/`, `src/names/`, `tests/names.rs` and
`docs/` was changed.

Font: the game's font (`assets/DejaVuSans.ttf`) has every special letter used
(`ṭ ḍ ì ʻ` and their capitals).

For the adapter later: names are made today in `src/sim/names.rs`
(`person_name(race, seed)`, `place_name(race, seed)`), called from `person.rs`,
`worldgen.rs`, `carry.rs`, `encounters.rs`, `dialogue.rs` and `view/townui.rs`.
