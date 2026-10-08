# Gahturiyu Sim

A Kenshi-style world for Gahturiyu, built simulation-first. Right now it is a
21 × 21 km coast with 30 towns and 5,000 people who travel, visit and wander
whether or not you are watching — plus a window to watch them in, built
with the Bevy game engine: sun and moon, firelight, woods and grass.

Your squad of four can fight, sneak, pick locks, go indoors, gather and craft,
talk to people, trade and take on work. Every town has households, jobs and
daily routines, kitchens and gardens, a stockpile, a treasury and caravans. Bandit camps by the roads ambush travellers
across the whole map whether you're there or not. The foundation underneath:
**the far-away world runs cheaply, the nearby world runs in full detail, and
nothing that happens depends on how closely it was being watched.**

## Running it

One-time setup on Windows:

1. Install Rust from <https://rustup.rs>. If it asks to install the Visual Studio
   C++ build tools, say yes.
2. Clone this repo (GitHub Desktop is fine).

Then, in a terminal inside the repo folder:

```
cargo run --release
```

The first build downloads and compiles the Bevy engine and takes a while —
10 to 20 minutes on a laptop. After that, rebuilds take under a minute and it
starts in seconds. `cargo run --release -- 42` builds a different world from
seed 42.

### Controls

The window opens in **3D**. Press **V** to flip to the top-down map and back.

| | |
|---|---|
| Left-click ground | Send the selected squad members there (everyone, if none selected) |
| Left-click a squad member, or F1–F4 | Select them (hold Shift to add or remove) |
| ` or Esc | Select everyone again (Esc also leaves a conversation) |
| Left-click a bandit | Attack (a sneak attack if they haven't noticed you) |
| Left-click anyone else | Walk over and talk to them |
| Left-click something on the ground | Pick it up |
| Left-click a door | Go in, or pick the lock if it's locked (needs a lockpick) |
| Left-click a plant, rock or log | Gather it |
| Left-click someone who's down | Nearest selected member goes and picks them up |
| X | Selected members put down whoever they're carrying |
| N | Selected members rest (they sleep where they stand); press again to get them up |
| Z | Selected members sneak / stop sneaking |
| T | Selected members light their torch, or put it out (takes one from the pack if needed) |
| I, or right-click a squad card | Pack and gear |
| K | Crafting |
| M | Spell book (click a spell to use it; spells aimed at someone or somewhere then wait for a click in the world — right-click cancels) |
| G | Look through a scout spirit (G again or C to come back) |
| J | Journal (jobs) |
| P, or click a town's name | Town panel: its people and customs, food, store, money, which services are open |
| O | Graphics settings |
| F8 / F9 | Save / load (one quick-save slot) |
| Right-drag, Q / E | Turn the camera (map: pan) |
| Middle-drag, or WASD | Pan |
| Mouse wheel | Zoom |
| C | Snap back to following the squad |
| Space | Pause |
| 1 – 5 | Speed: real time, 10×, 1 minute/s, 10 minutes/s, 1 hour/s |
| R | Show / hide the band rings |
| L | Detail readout: what's drawn at what detail, and triangle counts |
| B | (testing) Drop a band of bandits next to the squad |

Hover over anyone or anything for details. A fight breaking out near you drops
the speed to real time.

In the **pack** panel: click something worn to take it off, click something in
the pack to put it on (or drink, eat or use it), right-click to drop it. Equip
the short bow from the pack to shoot (arrows stay in the pack). Click a standing
torch in the pack to set it in the ground.

Each **squad card** shows health (and energy, for casters), then three small bars — food,
stamina and rest — whose labels turn orange when there's trouble ("hungry",
"starving", "worn out"), plus who they're carrying or who's carrying them, and
any limb lost for good (−LA = left arm, and so on). "torch" after a name means
their torch is lit. A violet diamond with "Holding …" means a ritual is held ready.

## Playing

- **Your squad**: a Roduro brawler with a club, a Horaro hunter with a
  spear and lockpicks, a Qotiro fighter with hatchet and buckler, and a
  Ṭaḍoro mage, all in the shared basics (leather, cloth, wood, plain iron) —
  the traditions' work is something to buy, order or make. The mage has (every
  felt spell their feel reaches, Fireball, Lightning bolt, Paralyze, Blind,
  Barrier, Haste and some others, and the Restore ritual) with a mortar and
  pestle and some herbs. Each walks at their own pace: hills, a hurt leg or an
  overloaded pack all slow them down.
- **Fights** are Kenshi-style: six body parts, each with its own health;
  cuts and blunt blows; armour that covers some parts some of the time. A
  ruined arm drops the weapon, a ruined leg slows you, a head or torso at zero
  knocks you out, and only a much worse beating kills. Wounds heal on their
  own over hours. Hits, dodges, blocks and spells train the skills used,
  Morrowind-style. Hurt fighters drink healing draughts if they have them.
- **Food**: hunger climbs with the clock — faster on the move, under a heavy
  load or when wounded, slower asleep. Members eat from their own pack when
  hungry. Go without and they heal slower, then weaken, then waste until they
  collapse (it never kills). Food is found in homes; berries and mussels can be
  gathered.
- **Stamina and sleep**: stamina drains on the march (more uphill and loaded)
  and comes back standing still; a fight starts with what's left. Tiredness
  builds over the day and only sleep clears it: in the open is slow, a tent
  (the hunter carries one) is better, indoors is best. Worn-out members are
  slower and weaker. Sleepers who are attacked take a few seconds to wake.
- **Healing** depends on how they're doing: asleep and fed heals fastest,
  marching heals little, starving heals nothing.
- **Carrying**: knocked-out members no longer get left behind — send someone
  to carry them. A carrier walks at about 60% pace (a strong one carrying a
  light body faster, a weak one with a heavy body slower) and can't fight until
  they put it down.
- **Bedtime**: a member left standing idle after 22:00 who's fairly tired lies
  down by themselves, and gets up at 06:00. Wake them with N and they stay up
  until morning.
- **Roads** are quicker than open ground (sand, rock and scrub are slower than
  grass). Click somewhere far and the squad takes the roads if that's faster.
- **Bows and crossbows** shoot from range and use up ammo; about half is found
  again after a fight. Archers draw a hand weapon if someone gets close (or
  back away if they have none). Some bandits are archers. You can watch each
  arrow fly; misses stick in the ground for a while.
- **Light and dark**: the hour sets how light it is, and campfires, lit town
  windows and torches add to it. In the dark you're harder to see — and harder
  to hit, an arrow most of all.
- **Torches** go in the off hand (everyone starts with two). A lit torch lights
  the ground round you, so you see and shoot better at night — but it can be
  seen from over 200 m away, and sneaking with one lit hides you hardly at all.
  A torch burns about four game hours, then the next one in the pack is lit.
  Put one out early and it keeps what's left. The hunter also carries two
  standing torches: set one in the ground to light an area for eight hours.
  Travellers on the road light a torch after dark too, so bandit camps spot
  them from much further off at night.
- **Lost limbs**: an arm or leg battered badly enough is gone for good. No
  shield or two-handed weapon without a left arm; a lost leg is a permanent
  limp, two mean crawling.
- **Magic**: see below. Scrolls cast their spell once with no energy and
  never fizzle.
- **Bandit camps** sit beside the roads, lit by a campfire. Their lookouts
  notice you by sight (worse in the dark, against someone sneaking) and by
  sound (louder when moving, fighting or in heavy armour). Once they notice,
  they attack. They also attack travellers they think they can beat — all
  across the map.
- **Doors** lock from 20:00 to 06:00. A picked lock stays open until the next
  night. Inside, the walls and roof are cut away; there are things to take,
  but they belong to someone. Being seen picking a lock or stealing earns a
  bounty in that town (pay it off by talking to a local). **Word travels**: a
  traveller leaving a town that knows about your bounty may carry the news to
  where they're going, so other towns hear of it over the following days.
  Locals in any town that has heard treat you coldly, and you can pay it off
  there too. The side panel says how many towns know.
- **Crafting** (K): seven crafts, each with its own station in town —
  leather, cloth and wood (workbench), smithing (forge), armouring (forge or
  bench), weaving and sealing (weaver's frame), stone-tending (grower's bed),
  paper, ink and scrolls (scribe's desk), alchemy (table, or a mortar and
  pestle anywhere). Each member starts with a craft or two; to take up
  another, pay a crafter at work to teach you ("Teach me your trade"), or
  read a manual (slower, and it only gets you started). Practice does the
  rest. What you make comes out crude, common, fine or masterwork, and
  carries your mark. See Materials and crafting below.
- **Wear**: your weapons and armour wear with every blow; at zero a piece
  is gone. A crafter of the right trade mends it for coin, or right-click it
  in the pack panel to mend it yourself (if you know the craft and are at its
  station) — or to seal unsealed reed with pitch.
- **Talking**: people answer from who they are and what's true right now. Some
  have work: break a bandit camp, fetch materials, carry a letter.
- **Towns live by the clock**: people sleep, walk to work, eat, work, spend
  the evening at the hearth, the inn or the deck, and go to bed. A shop is
  open only while its keeper is at work — at night, on their rest day, during
  their meal, or if they've died, it's shut. **Trade** with a merchant at
  their stall ("What have you got?"); **change money** with an exchanger. See
  Society below.

## Society

**Cultures are tendencies, not scripts.** No town is told "you're Roduro, so
you do this". Each people has a list of leanings (`src/sim/culture.rs`): who
cooks, the rhythm of the day, who belongs together, where they like to spend
the evening, what work draws them. A town's culture is the **blend** of its
residents' leanings, weighted by how many of each live there, plus a small
random nudge of its own. Each custom is then picked from the blend by **one
fixed roll** per town. The roll never changes; the blend does — so a town
keeps its ways while its people stay much the same, and drifts into new ones
when enough people die, leave or move in (it takes stock each dawn). Each
person also rolls a few habits of their own (keeping their own people's hours,
lodging with a host, where they spend evenings).

A coastal town is two communities: the town on land, and the stilt village
offshore, each with its own blend and customs.

The customs:

- **Who cooks**: each household its own pot; a hearth kitchen whose cooks
  feed the workers and whose **meal runners** carry stacked pots out to the
  fields and yards at midday; or everyone eating together off a shared deck.
  Ṭaḍoro have no leaning of their own here — their hosts feed them.
- **Rhythm of the day**: **seasonal** hours (steady, sliding with the time of
  year); **bells** (fixed shifts — everyone eats at the same bell, which makes
  a town predictable); **tides** (work starts round low water, about fifty
  minutes later each day, so a stilt village's day comes round in a
  fortnight); or **irregular** (everyone their own hours).
- **Who belongs together**: households by family line (one per home), tier
  blocks (two homes to a household), one household for the whole village, or
  everyone lodging on their own.
- **Services**: in towns no one people dominates, services are **combined** —
  one market square with stalls, one workyard with forge and armourer's bench,
  one healing house, a letters house (exchange, letters, teaching). Towns
  dominated by one people **split** them into separate shops and houses. It's
  a smooth leaning, not a cut-off.
- **Strong minorities** lean toward an institution of their own: a mess hall,
  a cooking deck, a Tenders' yard, a letters house.

**Jobs.** Everyone has one, chosen when the world is made from the town's open
posts, weighted by their calling, temper and people's leanings: farmer, fisher
and diver, forager and hunter, woodcutter and quarrier, Stone Tender (making
rounds of the stone homes), cook, meal runner, merchant, caravaner, innkeeper,
guard (day or night watch), official, priest, healer, teacher, exchanger,
arbiter, labourer and porter, kelp gatherer, boatwright, charcoal burner, and
the crafters (smith, armourer, alchemist, scribe, weaver and sealer, tanner,
leatherworker, tailor, woodworker). How many crafters of each kind a town has
leans on how much its people take to each craft. Restless people with nothing
to do drift. When someone with a post dies or leaves, it stays empty until the
next dawn, when a labourer takes it up. Every home on land has a garden,
tended by whoever in the house has the lightest work (a lodger first, then the
eldest).

**Days.** Everyone's day is a plan read off the clock: wake, walk to work,
eat at midday, work, garden, maybe the market (every fourth day is market
day) or a neighbour's, then the evening, then bed. People rest one day in six
(under bells, a third of a shift at a time). Guards on the night watch work
18:00 to 06:00. At night people are indoors asleep — which makes them poor
witnesses, and they can't be talked to.

**Food** comes three ways, and the cooking custom decides how much each is
leaned on: **gardens** feed their households; the **kitchens** cook from the
town's stock (with cooks and runners, at a deck, or in each home's own pot);
and the **dawn boats** bring the stilt village's catch ashore at first light.
Every dawn the day before is tallied. A path that breaks — the cooks dead, the
runners gone, the village withdrawn — leaves its share unmet, and the others
don't cover for it. The town panel shows how well each path did.

**Goods and money.** Work adds goods to the town's store at simple rates
(grain from the fields, fish from the divers, timber, game, hides, rock and
ore, and the rest — see below); food slowly spoils. What's in store is what the
merchants sell, and how well-stocked and well-fed a town is sets its
**prosperity**, which sets how fast the merchants' coin refills after you've
sold to them (what you and the caravans bring in can lift it above that
level; it just won't refill past it). **Caravaners** carry a town's surplus to a town that wants it,
on the same roads and journeys as everyone else; the goods arrive when they
do, bandits who beat them take the lot, and the coin from the sale comes home
with them. Hover a caravan to see what it carries.

**One coin**, struck by the Ṭaḍoro's magical presses so it can't be faked;
coins weigh something. **Notes** (50 coin on Ṭaḍoro paper) weigh next to
nothing, but are as easily stolen or lost as anything else you carry.
Exchangers in the larger towns swap one for the other, keeping a coin each
time. Each dawn a town taxes its households into a **treasury** that pays its
guards; when it can't, the pay owed is shown in red (Part 3 makes that matter).

### Materials and crafting

**What the land offers.** Each town looks at the land within a day's walk
once, as the world is made: ore and rare veins (gold, Edgeglass seed
crystals) toward the mountains and the plateau's edge, sand on the plateau,
clay in the lowlands, tentsilk cocoons in the hills, and on the coast reed,
pitch from the shore woods, shell and pearls, and whatever lies on the
seabed. Its woodcutters and quarriers split their hours over what's there;
farmers bring in fibre, hunters hides, divers shell and fishskin, kelp
gatherers reed. A town without ore can only work iron if the caravans bring
some. One thing feeds another: charcoal burners turn wood into charcoal and
ash; forges need charcoal and give off ash; Stone Tenders grow stone from rock
and ash; kelp from the store, dug into the gardens, makes them yield more.

**Materials** (`src/sim/materials.rs`) each carry their traits: edge,
weight behind a blow, how much armour they get through, how well they turn
cuts and blows, weight, how long they last, whether they can be mended.
Every tradition's are there: grown Roduro stone (Ringstone, Hearthclay,
Slatewing, Edgeglass — keenest of all and slow to wear, but it can't be
mended and shatters when spent), Qotiro fire-work (Bronze, Forgeiron — best
against armour — Sandglass, gold), Horaro sea-work (Seareed, which rots
unless sealed with pitch; Nacre; Fishskin), and Ṭaḍoro tentsilk. The shared
basics (leather, cloth, wood, plain iron) are made anywhere. Nacre and
Slatewing turn cuts but crack under heavy blows; nets entangle. A piece can
pair a main material with a backing one: an Edgeglass edge on a Forgeiron
spine, Nacre scales on a Slatewing frame — which needs both traditions'
work in one town, or a caravan between two.

**Town crafters** work from the same recipes as you. At the top of each
hour, any with nothing in hand pick something to make from what the store
holds and lacks; it's done when their hours add up. Materials go back to the
store; weapons, armour, scrolls and manuals go on the town's **shelf**, where
the merchants sell them and the townsfolk buy a few each dawn. Grade comes
from the maker's skill, the station and a roll. A fine piece, or one by a
skilled maker, is **stamped with their mark**; every piece records who made
it and where. Stamped pieces sell for more, and more again as the maker's
work gets around.

**Prices follow each town's own store**: a good it has little of costs more
there, one it has plenty of costs less (×0.5 to ×2.5, worked out when
asked). Caravans carry what's cheap at home to where it's dear. The town
panel marks dear goods with ↑ and cheap ones with ↓.

**Orders.** Grown pieces (Slatewing helms and plates, Ringstone mauls and
shields, Edgeglass blades) aren't on any shelf: ask a Stone Tender at work
what they'd grow, pay half, and come back when it's ready — days for
Hearthclay, three weeks for Slatewing, three months for Edgeglass. Stone
grows only while it's tended: each dawn the Tender is away adds a day, and if
the Tender dies the order and the deposit are lost.

**Mending** needs someone who works the material: smiths and armourers for
fire metals, weavers for reed and shell, Tenders for grown stone, any
leather-, cloth- or woodworker for the basics. A town can only mend what its
people know how to work.

`cargo run --release --bin headless -- society 3` prints every town's customs,
jobs, food, store, what the land offers, its shelf and its dearest goods after
three days — the quickest way to see what the dials do.

## Magic

**Three styles are the magic skills.** Anyone can learn any of them; a
person's people only tilts which they lean to (Roduro and Horaro feel their
way, Ṭaḍoro build, Qotiro perform rites), and individuals vary widely.

| | Felt | Structured | Ritual |
|---|---|---|---|
| How big | small | medium | large |
| Casting | instant, even mid-fight | 1–2 seconds; a solid hit spoils it | minutes to hours, never mid-fight |
| Costs | a little energy and a little tiredness | energy | components and/or blood (a wound that heals like any other) |
| Failing | rarely | fizzles sometimes; the energy is spent anyway | backlash: components lost, the caster hurt |
| Learning | comes by itself as the style is used | from a teacher or from notes | from a teacher or from rare texts |

"Energy" is the mana pool (the blue bar).

**Rituals** are performed standing still (walking off breaks it off; the
components are gone either way). Some need a place: a town's **hearth**, a
**shrine** (for now, the Qotiro temples and halls), or a **drawn circle**
(drawing one adds 20 minutes; it stays on the ground for next time). A
finished ritual is **held ready** — one at a time — and released whenever
you like, mid-fight included, where it works at once. Holding one drains
stamina, and one still held when its caster next sleeps slips away. The
squad card shows a violet diamond and "Holding …" while one is held.

**Learning**: felt spells come with use (your squad only). Notes teach a
structured spell and rare texts a ritual, to anyone skilled enough to follow
them; they're kept after reading. Notes turn up now and then in homes, ritual
texts in Qotiro temples. Local mages teach what they know for coin ("Could
you teach me?" in conversation). People you meet already know every felt
spell their feel reaches and some of the structured spells and rituals their
skill would let them follow.

**Domains** sort spells by what they work on: Elemental, Psychic, Illusion,
Vital, Warding, Alteration, Summoning, Necromancy. They have no rules of
their own yet; items or blessings can strengthen a domain's spells or ward
against them (nothing in the game does yet).

**Summoned creatures and raised dead** are fighters like anyone else, on
their caster's side, using all the same fight rules, until their binding
runs out. The raised are mindless (they go for whoever is nearest), a body
can only be raised once, and nothing raises one of the caster's own side —
a fallen squadmate is never brought back.

### The spells

F = felt, S = structured, R = ritual.

| Domain | Spells |
|---|---|
| Elemental | Spark (F, a little fire) · Chill (F, frost and a slow) · Kindle / Douse (F, light or put out a torch, standing torch or campfire — a doused campfire stays dark for 3 hours) · Fireball (S, burst that hits friend and foe) · Lightning bolt (S, ignores armour) · Stone spikes (S, hits only the legs) · Firestorm (R, a big burst) · Tremor (R, throws enemies round you to the ground) |
| Psychic | Daze (F, they lose their next action) · Sense life (F, the living nearby shown through walls) · Nightsight (F) · Calm (S, won't fight until hit) · Fear (S, runs off for a while) · Paralyze (S) · Sway (S, +20 disposition in conversation) · Dominate (R, an enemy fights for you for 20 s) |
| Illusion | Silent step (F) · Glow / Gloom (F, light or darkness round you — it counts for being seen) · Blind (S) · Hide (S, hard to see; enemies lose you beyond arm's reach) · Decoy (S, an illusion that draws the blows) · Disguise (S, half an hour unrecognised: bounties and crimes aren't laid on you) · Veil (R, hours unseen and unheard by lookouts) |
| Vital | Mend (F) · Second wind (F, stamina) · Haste (S) · Might (S, strength and carrying) · Toughen (S, skin as light armour) · Restore (R, at a hearth: heals every wound and all tiredness of the squad) · Sustain (R, a day with no hunger or tiredness) · Regrow (R, at a shrine: a lost limb grows back — the only cure) |
| Warding | Brace (F, turns the next blow) · Tripwire (F, round camp: sleepers wake at once if attacked) · Resist (S, half of elemental harm) · Barrier (S) · Dispel (S, ends every spell on someone; sends a creature back) · Sanctuary (R, enemies can't step in; lookouts won't come for you inside) |
| Alteration | Lighten / Burden (F) · Shatter item (S, their weapon or shield breaks for good) · Unlock (S, a locked door opens until the next night) · Shrink / Enlarge (S) · Rust (S, their armour stops less) · Transmute (R, materials in the pack become others: ore to ingots, hide to leather, salt to storm glass, ash moss to ghostcap) |
| Summoning | Wisp (F, a light at a spot) · Scout (F, a spirit at a spot up to 150 m off; press G to look through it) · Spirit beast (S, fights for you for a minute) · Pack spirit (S, carries 40 kg for four hours) · Guardian (R, waits at a spot for hours and joins the next fight there) · Swarm (R, six small biters, in a fight) |
| Necromancy | Drain (F, takes health into the caster) · Preserve (F, bodies don't rot for a day) · Wither (S, rots one limb) · Raise thrall (S, a body fights for you, mindless, for 45 s) · Grave call (R, every body round you rises) · Blight (R, a patch of rot that eats at everyone in it) |

Not in yet, because what they work on doesn't exist: Far sight (the map
already shows everything), Hold the Overgrowth (no Overgrowth yet), Mend item
and Soften/Harden (items don't wear and walls can't break), Seal (nothing but
your squad opens doors), Bind (no animals), Speak with dead (nothing records
what the dead knew).

The fight AI reads what a spell does from its effects (mend, ward, hinder,
blast, strike, call up help, raise the dead), so new spells are used
sensibly without new code. Mages cast freely; ordinary fighters mostly throw
a spark at someone still out of reach. Your squad saves a held ritual for
when it counts (Restore when someone is down, a blast when three or more are
caught), so hours of work aren't spent on a scratch.

Lasting spells cast outside a fight (Nightsight, Barrier, Haste...) are still
on you when a fight starts, so they can be cast beforehand.

### Saving

F8 saves, F9 loads. There is one quick-save slot, `saves/quick.sav` in the repo
folder (git ignores it). A save holds the world's seed plus everything that has
changed since the world was made — every person, journey, fight in progress,
what's lying on the ground, bounties and gossip. The land and roads aren't
stored; they're rebuilt from the seed, which is why a save is only about 2 MB.
A loaded world carries on exactly as it would have. When an update changes
what's saved, older saves are refused with a message rather than loaded wrong.

### Graphics settings

Press O. Click a row to change it; it's remembered in `settings.txt` next to
the assets folder (delete that file to reset). None of it changes what happens.

| Setting | What it does |
|---|---|
| Shadows | off / low / medium / high: how far sun shadows reach |
| Grass and trees | off / low (no grass, thinner woods) / medium (half the grass) / high |
| Lights at once | 8 / 16 / 28 / 40 fires, torches and windows lighting the scene |
| Glow | the soft halo round bright things at night |

If the game runs slowly, try grass and trees on medium first, then shadows.

## What you are looking at

- **Colours are races.** Stone = Roduro, ember = Qotiro, sea blue = Horaro,
  pale violet = Ṭaḍoro.
- **Towns** follow `architecture.md`: Roduro grow rounded, banded homes of dark
  stone with a lit window; Qotiro towns are stepped sandstone blocks around a
  three-tier temple; Qotiro living elsewhere keep one small dark-stone hall with
  a gold crown; every coastal town has a Horaro stilt village just offshore —
  woven domes on stone pillars with timber decks. Every town has a hearth at its
  centre. Ṭaḍoro build nothing; a resting wanderer pitches a tent.
- **The rings around your squad are the bands.**
  - Inside the inner ring (500 m) is **band 1**: everyone is a person with a
    name, temperament and gear. This is decided **person by person**, by where
    each one stands, so the ring cuts through a town rather than switching the
    whole town on or off.
  - Between the rings (out to 2.5 km) is **band 2**: travelling groups are one
    marker each, updated every few game seconds.
  - Beyond is **band 3**: groups updated once a game-minute.
- In 3D, people are true size up close and drawn larger as you zoom out, so you
  can still pick them out; far off they're a plain shape, and groups beyond
  band 2 are a single marker. Buildings always stay true size.
- **Day and night**: the sun crosses the sky with the game hour and casts
  shadows; dawn and dusk are warm; night is dark blue with moonlight. Lit
  windows, campfires and torches glow and light what's round them — the same
  lights the simulation counts for who sees whom. At night each squad member
  has a faint ring at their feet so you can always find them.
- **Grass, bushes and trees** are decoration: they grow where the ground suits
  (never on roads, in towns or on the dry plateau's bare rock) and sway in the
  wind. They don't change anything that happens. Grass fades out by 80 m,
  bushes by 300 m; trees turn into simpler shapes and then flat pictures with
  distance so woods stay visible to the horizon. Press L to see how many of
  each are being drawn.
- The panel's **"Named so far"** count only goes up when something comes close.

### The land

- **The coast** runs down the west side, mostly low beaches with some stretches
  of low sea cliff. Harbour towns only stand where there's a decent beach.
- **Rolling hills** rise slowly inland.
- **Mountains** wall in the north and east edges, with one big massif in the
  middle of the map.
- **The Qotiro plateau** in the south-east is a flat-topped, cliff-edged
  tableland of dry scrub — their homeland.
- **Roads** link every town to its nearest neighbours. They were found by
  searching the land for the easiest walk, so they follow valleys, climb
  escarpments where the slope eases, and go around mountains. Travellers use
  them, and walk slower uphill and a little faster downhill (Tobler's hiking
  rule), so the slope shapes how long every journey takes.
- Your squad also slows on climbs. The sea is off-limits.

In 3D, the land within a few kilometres is detailed and the rest is coarser
out to the horizon. The map view shows the whole world as shaded relief.

## Adding a model

Models are GLB files from Blender or Meshy with standard materials (colour,
normal, and one combined occlusion/roughness/metallic image). Put them in
`assets/models/`, named by detail level:

```
assets/models/Roduro_Home_5k.glb    about 5,000 triangles, drawn up close
assets/models/Roduro_Home_2k.glb    about 2,000, from 70 m
assets/models/Roduro_Home_500.glb   about 500, from 220 m
```

Only the `_5k` file is needed — any missing lower version is made when the game
starts by simplifying the one above it. The model should stand on its base and
face +X (Blender's red arrow); it's scaled to each building's size, so units
don't matter. Restart the game to pick it up. Press L to check it loaded: the
readout says how many triangles each level has and which were made
automatically. Until the file is there, Roduro homes are drawn as the built-in
grown-stone domes. (More model slots — Qotiro blocks, Horaro stilts — are a
line each in `src/view/models.rs` when you have them.)

## How it works, in plain words

**Everyone exists, but cheaply.** Each person is a seed, a race, four temperament
traits and one cached "might" score. That is all the far-away world reads.

**Details appear only when needed, and then stay.** A name and gear are built
from the seed the first time a person comes within band 1, chosen so they add up
to the might score the world was already using. Once built they are kept forever.
Someone you have met never comes back as a stranger.

**Races set the average, not the person.** Each trait is drawn around the race's
average with a wide spread. Most Roduro are homebodies; some are restless and
travel far. Most Ṭaḍoro roam; some settle in a town.

**Journeys are schedules.** When a group sets out, its whole trip is written
down: leave at 09:12, reach the next town at 11:40, stay until evening, walk
home. Its position at any moment is just looked up from the schedule. A group
in band 3 is checked once a minute and a group beside you sixty times a second,
and both are in exactly the right place. This is the same rule as the old NPC
routine plan — read the clock, set the state.

**Randomness is keyed, not rolled.** "Does anyone leave this town this hour?" is
answered from the world seed plus that town plus that hour. So the answer is the
same no matter how coarsely the world was being stepped or where your squad was.

**Fights far away are the same fights.** When a traveller's route is planned
(a couple of hours ahead), it's checked against every bandit camp: the exact
moment the road brings them into sight is worked out like a line crossing a
circle. Bandits weigh their combined might against the travellers' and decide
whether to attack. If they do, the fight is run blow by blow on the same rules
as your own fights, there and then — it takes well under a millisecond. Nearby,
you watch a copy of it play out in real time; the outcome was already fixed.
Survivors wait until they can stand, then head home. So walking closer never
changes how a fight went.

## Proving it

```
cargo test --release
```

The important tests are in `tests/consistency.rs`. They run the same world
several ways — one-second steps vs one-hour steps, squad here vs squad in the
far corner, squad watching a roadside ambush vs far away — and check that every
journey, route, position, wound, death and ambush comes out identical. They
also check that details, once built, never change. The other test files check
each system does what it says: `combat.rs`, `gear.rs` (every enchantment),
`squad.rs`, `stealth.rs`, `indoors.rs`, `crafting.rs`, `talk.rs`, `terrain.rs`,
`condition.rs`, `carry.rs`, `news.rs`, `save.rs` (a loaded world carries on
exactly like the saved one, even mid-fight), `magic.rs` (styles, costs,
rituals, learning), `spells.rs` (every spell does what it says) and
`society.rs` (customs, routines, services, food paths, caravans, trade).

```
cargo run --release --bin headless -- 3
```

runs three game days with no window and prints what the world is doing and how
long it took (a few seconds, stepping one game second at a time). With
`GAHT_SAVE=some.sav` it saves the world at the end; start the window from it
with `GAHT_LOAD=some.sav`.
`headless fight 3` prints a squad-vs-bandits fight blow by blow.

## Dials

The numbers most worth tuning, all named constants:

| What | Where |
|---|---|
| Population, town counts, who becomes a wanderer | `src/sim/worldgen.rs` |
| Which races favour which towns | `affinity()` in `src/sim/worldgen.rs` |
| Race temperaments, build, walking speed | `src/sim/race.rs` |
| How busy the roads are, day vs night | `DEPARTURE_RATE`, `NIGHT_FACTOR` in `src/sim/world.rs` |
| Band sizes and update rates | `src/sim/bands.rs` |
| Where mountains, the plateau and cliffs are; how tall | `src/sim/terrain.rs` |
| How roads are chosen (steepness limit, how many links per town) | `src/sim/routes.rs` |
| Ground and building colours | `src/view/palette.rs` |
| Stats, skills, how fast they train | `src/sim/stats.rs` |
| Weapons, armour, enchantments, materials, potions, scrolls | `src/sim/items.rs` |
| The spell list: style, domain, costs, aim, range, effects, ritual needs (`SPELLS`); how much of their skill's reach people know at the start (`KNOWN_SHARE`); success chances (`success_chance`) | `src/sim/magic.rs` |
| What each kind of effect is and does (one list for spells, potions, scrolls, worn items) | `src/sim/effects.rs` |
| A mage's study split and each people's leaning style (`MAGE_STUDY`, `race_style`) | `src/sim/stats.rs` |
| Rituals and learning: places (`HEARTH_REACH`, `SHRINE_REACH`, `CIRCLE_MINUTES`), holding (`STAMINA_HOLD`), backlash (`BACKLASH`), reading and teaching (`READ_SKILL`, `TAUGHT_SKILL`, `lesson_price`), `TRANSMUTE`, `PRESERVE_HOURS` | `src/sim/casting.rs` |
| Summoned creatures' strength and numbers (`creature`) | `src/sim/combat.rs` |
| Magical light (`GLOW_REACH`, `GLOOM_REACH`), how long a doused campfire stays out (`DOUSE_HOURS`) | `src/sim/torch.rs` |
| Hit chances, damage, sneak attacks | `src/sim/combat.rs` |
| Bandit camps: how many, how far they see, rest between attacks | `src/sim/encounters.rs` |
| Sight, hearing, light, sneaking | `src/sim/stealth.rs` |
| Locks, lockpicking, what's inside homes | `src/sim/buildings.rs` |
| Recipes, stations, gathering | `src/sim/crafting.rs` |
| Errands and dialogue | `src/sim/quests.rs`, `src/sim/dialogue.rs` |
| **Each people's leanings** (`PROFILES`: cooking, rhythm, belonging, evenings, keeping their own ways, lodging, their minority institution, job leanings); the town nudge (`NUDGE`, `SPLIT_NUDGE`); when a minority sets up its own (`MINORITY`); how big a shift re-blends a town (`BLEND_SHIFT`) | `src/sim/culture.rs` |
| How many of each job a town wants (`posts_for`), and which places it keeps at what size (`places_wanted`); how each cooking custom leans on the food paths (`PATHS`); hours workers can be counted on (`expected_hours`); rest days and market days (`WEEK`, `MARKET_EVERY`); dawn (`DAWN`); roads for an inn (`ROADS_MEET`) | `src/sim/society.rs` |
| Work rates (`FARM_PER_HOUR`, `FISH_PER_HOUR`, `KELP_PER_HOUR`, `MEALS_PER_COOK_HOUR`, `POTS_PER_RUN`, `Job::yields`), who gets the garden (`Job::lightness`), how well callings and tempers suit each job (`Job::fit`), goods' worth and spoiling | `src/sim/jobs.rs` |
| Gardens, tax, guards' pay, the merchants' purse, trade prices, notes and the exchange fee, caravans (`GARDEN_FOOD`, `TAX_PER_HEAD`, `GUARD_WAGE`, `PURSE_PER_MERCHANT`, `PURSE_REFILL`, `BUY_MARKUP`, `SELL_SHARE`, `NOTE_VALUE`, `CARAVAN_CHANCE`, `CARGO_PER_HEAD`, `KEEP_DAYS`) | `src/sim/economy.rs` |
| Working hours by rhythm, the night watch, the runners' round and the dawn boats (`work_hours`, `RUN_HOURS`, `BOAT_OUT`, `BOAT_BACK`, `BED`) | `src/sim/routine.rs` |
| The tide and the season (`TIDE_PERIOD`, `YEAR_DAYS`, `SEASON_SWING`) | `src/sim/tide.rs` |
| Hunger: how fast, stages, when they eat (`HUNGER_PER_HOUR`, `EAT_AT`, `HUNGRY`/`WEAK`/`STARVING`, `STARVE_DRAIN`) | `src/sim/condition.rs` |
| Stamina and tiredness (`STAMINA_WALK`, `STAMINA_CLIMB`, `STAMINA_REST`, `TIRED_PER_HOUR`, `SLEEP_OPEN`/`TENT`/`INDOORS`, `EXHAUSTED`) | `src/sim/condition.rs` |
| Healing by activity (`HEAL_SLEEP_*`, `HEAL_RESTING`, `HEAL_WALKING`; base `HEAL_PER_HOUR` in `body.rs`) | `src/sim/condition.rs` |
| Food nourishment, tent, bows and ammo | `src/sim/items.rs` |
| Carrying: body weights, pace (`body_weight()`, `CARRY_PACE`, `CARRY_BODY`, `CARRY_PACE_RANGE`) | `src/sim/carry.rs` |
| Bedtime (`BEDTIME`, `RISE`, `BED_TIRED`) | `src/sim/condition.rs` |
| How readily bounty news travels (`NEWS_CHANCE`) | `src/sim/news.rs` |
| How dark before travellers light torches (`TRAVEL_TORCH_DARK`) | `src/sim/torch.rs` |
| Road speed and ground types (`ROAD_PACE`, `ROAD_HALF_WIDTH`, `Ground::pace`) | `src/sim/terrain.rs` |
| When a trip goes by road (`ROAD_TRIP`) | `src/sim/buildings.rs` |
| Archers: draw, stow and back-off distances (`ARCHER_DRAW`, `ARCHER_STOW`, `ARCHER_SPACE`) | `src/sim/combat.rs` |
| Ammo found after a fight (`AMMO_FOUND`) | `src/sim/fights.rs` |
| When a limb is lost (`LIMB_LOSS`) | `src/sim/body.rs` |
| Name sounds per race | `src/sim/names.rs` |
| Torches: burn time, light, how far they're seen, sneaking with one (`TORCH_HOURS`, `STANDING_HOURS`, `TORCH_REACH`, `TORCH_POWER`, `TORCH_SEEN`, `TORCH_SNEAK`) | `src/sim/torch.rs` |
| How much darkness spoils aim (`DARK_SHOT`, `DARK_BLOW`) | `src/sim/combat.rs` |
| Materials' traits: edge, weight, piercing, turning cuts and blows, durability, wear, cracking, rot, growing days, worth (`MATERIALS`); grades (`Grade::from`, `power`, `durability`, `worth`), when a mark is stamped (`MARK_SKILL`), rot (`ROT_PER_DAY`) | `src/sim/materials.rs` |
| Every recipe: inputs, skill, difficulty, station, time (`RECIPES`); how many beds a Tender grows at once (`TENDER_BEDS`) | `src/sim/crafting.rs` |
| What the land offers each town and how far quarriers go (`town_sources`, `VEIN_REACH`); what a quarrier digs (`DIG`); what other gatherers bring in (`Job::gathers`) | `src/sim/making.rs`, `src/sim/jobs.rs` |
| Prices: range (`PRICE_RANGE`), how much a town keeps (`KEEP_COIN`), mark and renown (`MARK_WORTH`, `RENOWN_WORTH`, `RENOWN_MAX`) | `src/sim/making.rs`, `src/sim/economy.rs` |
| Shelf size, what locals buy (`SHELF_CAP`, `SHELF_EACH`, `LOCALS_BUY`); charcoal (`BURN_PER_HOUR`, `CHARCOAL_PER_TIMBER`, `ASH_PER_TIMBER`, `FORGE_ASH`); kelp in the gardens (`KELP_PER_GARDEN`, `KELP_BOOST`) | `src/sim/making.rs`, `src/sim/economy.rs` |
| Lessons and manuals (`LESSON_PRICE`, `LESSON_CAP`, `MANUAL_GAIN`, `MANUAL_HOURS`, `MANUAL_CAP`; how each teaching style goes in `Teaching::lesson`); order deposit (`DEPOSIT`) | `src/sim/making.rs`, `src/sim/culture.rs` |
| Mending prices (`MEND_PRICE`, `SELF_MEND`) | `src/sim/wear.rs` |

Drawing only (these never change what happens):

| What | Where |
|---|---|
| How dark night is (`NIGHT_BRIGHTNESS`); sun, moon and sky light (`SUN_LUX`, `MOON_LUX`, `DAY_AMBIENT`, `NIGHT_AMBIENT`) | `src/view/light.rs` |
| Fire, torch and window light; how many point lights (`FIRE_LUMENS`, `WINDOW_LUMENS`, `MAX_LAMPS`) | `src/view/light.rs` |
| Foliage distances (`GRASS_FADE`, `BUSH_FULL_END`, `BUSH_SIMPLE_END`, `TREE_FULL_END`, `TREE_BLOB_END`, `BILLBOARD_FAR`) | `src/view/foliage.rs` |
| Foliage density and woods (`GRASS_GRID`, `BUSH_GRID`, `TREE_GRID`, `WOOD_SCALE`, `TREE_LINE`), wind (`WIND_SWAY`) | `src/view/foliage.rs` |
| Model detail switches (`MODEL_LOD1`, `MODEL_LOD2`, `MODEL_FADE`), model facing (`MODEL_YAW`) | `src/view/models.rs` |
| When people turn into plain shapes (`PERSON_SIMPLE`), arrow flight (`ARROW_SPEED`, `ARROW_LIES`) | `src/view/scene.rs` |
| What each graphics setting means (`foliage_density`, `shadow_reach`, `billboard_far`, `LAMP_STEPS`) | `src/view/settings.rs` |

## Layout

```
src/sim/      the simulation — no graphics, fully testable
  world.rs      the world and its step loop
  worldgen.rs   building a world from a seed
  group.rs      travelling groups and their schedules
  person.rs     people: cheap summary + lazy details
  bands.rs      band assignment by distance from the squad
  race.rs       the four races
  names.rs      per-race name generators
  geo.rs        positions and the coastline
  terrain.rs    the height of the land, and walking speed on slopes
  routes.rs     the road network and every town-to-town route
  rng.rs        deterministic randomness
  stats.rs      attributes, skills, callings
  body.rs       body parts and wounds that heal
  items.rs      the item catalogue
  inventory.rs  gear slots, packs, weight
  magic.rs      the spell list, styles, domains
  effects.rs    the one effect list for spells, potions, scrolls and worn items
  casting.rs    magic outside fights: casting, rituals, held rituals, boons, wards, learning
  combat.rs     a fight, tick by tick
  ai.rs         what fighters decide
  fights.rs     where fights meet the world (the squad's fights)
  encounters.rs bandit camps and ambushes on the world's timeline
  squad.rs      squad members, walking, picking things up
  condition.rs  hunger, stamina, tiredness, sleep, and healing that follows them
  carry.rs      carrying the downed
  stealth.rs    being seen and heard
  buildings.rs  doors, locks, interiors
  crafting.rs   recipes, stations, the squad's crafting, gathering, potions
  materials.rs  materials and their traits, crafts, grades, marks, made pieces
  making.rs     what the land gives, town crafters and their shelf, prices, orders, lessons
  wear.rs       wear, breakage, rot, mending, sealing
  quests.rs     jobs
  dialogue.rs   conversations (and trading)
  culture.rs    each people's leanings, town blends, choosing customs
  society.rs    communities, households, jobs, workplaces, gardens, dawn changes
  routine.rs    day plans looked up from the clock; who's where; what's open
  economy.rs    production, food paths, stock, prosperity, money, caravans, trade
  jobs.rs       jobs, workplaces, goods and services as tables
  tide.rs       the tide clock and the season
  torch.rs      torches, and all light sources
  news.rs       bounty news carried town to town
  save.rs       saving and loading
src/main.rs   starts the window
src/view/     the window (Bevy) — drawing only, never changes the world's rules
  app.rs        the frame: keys and mouse, stepping the world, panels, screenshots
  cam.rs        the 3D camera and the map's pan and zoom
  scene.rs      the 3D view: land, roads, towns, people, fights
  light.rs      sun, moon, sky, fires, torches, lit windows
  foliage.rs    grass, bushes and trees, and the detail readout
  models.rs     GLB models and their detail levels
  mesh.rs       building shapes in code
  palette.rs    colours, shared by the 3D view, the map and the panels
  map.rs        the top-down map
  hud.rs        side panel, hover descriptions, health bars
  squadui.rs    squad cards, pack, crafting, conversation, journal
  townui.rs     the town panel (P)
  shot.rs       headless screenshots (the GAHT_ flags)
  settings.rs   graphics settings (O)
src/bin/headless.rs  the world with no window
assets/       font, and models/ for GLB files
tests/        the consistency checks, and one file per system
```
