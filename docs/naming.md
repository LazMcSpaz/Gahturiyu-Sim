# Naming: words and names in the four tongues

Second agent's work, branch `agent2/naming`. This file grows with each stage. This is
the state after **stage 1** (canon inventory, First Speech roots, sound rules, cognate
sets). Stage 2 is compounds, the gods, the elements, the peoples' names for each other,
and spelling and pronunciation; then things, people, places.

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
| `src/names/` | the code: `sound.rs` carries out a rule list; `mod.rs` reads the data and answers questions |
| `src/bin/lang.rs` | a small tool to read the tongues (below) |
| `docs/glossary.md` | every root in all four tongues (made from the data; a test keeps it in step) |
| `tests/names.rs` | the checks (below) |

Asking (in code): `names::word("sea", Tongue::Horaro)` gives `moa`.
`names::derive("moʻa", Tongue::Qotiro)` runs any First Speech form through a tongue's
rules. Both are plain functions: the same answer every time, no state, no randomness.

Reading (at a terminal):

```
cargo run --release --bin lang -- glossary      every root in all four tongues
cargo run --release --bin lang -- cognates      the sets below
cargo run --release --bin lang -- say moʻa pora try First Speech forms in every tongue
cargo run --release --bin lang -- clashes       roots that sound alike within a tongue
```

## Where the canon came from

`languages.md` and `pantheon.md` are not in the repo. Both were recovered in full from
the chat they were written in (2026-07-29) and followed as written. `architecture.md`
was not found; it is not needed until stage 3 (buildings).

**The full Gogìḍu dictionary was not available.** Laz gave the language tool's export
to that chat as an attachment, which cannot be read back. The inventory therefore holds
the Gogìḍu words that are *quoted* in the lore: 37 words, 5 bits of grammar, 3 phrases.
Everything built on them is firm. The other 286 roots are new and **stand only until
checked against the dictionary**: where the dictionary already has a word (for
"stone", say), the root must be re-made from that word. That is a change to
`roots.ron` only; the rules and the code don't move.

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

Not yet: the "guess the tongue from the sounds" check, the length, blocklist and
lookalike checks. They belong to generated names (stages 4 and 5).

## Decided without asking (veto any of these)

1. **The First Speech's sound set**, and that Roduro lost its lip and nose sounds
   (it rests on the 45 Gogìḍu words I can see).
2. **The rule lists above.** They are a first cut for the ear. To change a tongue's
   sound, change its list in `sounds.ron`.
3. **Qotiro may have sound-alike roots**; the other three may not.
4. **Ṭaḍoro has no `o`.**
5. **All 286 new roots.** Where they echo a real language (`hale` house, `limu` kelp)
   it is by choice, for a faint familiar ring; say if that should go.
6. **Which words Ṭaḍoro borrowed**, and from whom.
7. **Turiyu** is recorded as the worn-down "holy beast" inside Gahturiyu
   (`hotorì gayugo`). That is a reading, not something written down anywhere.
8. Spelling: `Qotihiqì` as in `pantheon.md` (built on `hiqì`), not `Qotihìqì`.

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
