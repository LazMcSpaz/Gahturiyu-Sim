# Findings ledger (world tests, 2026-10-26)

## From the kit roller
- R1 Distinct heart types across 5 guilds forced the last guild's heart → loosened to "no type more than twice" (faction-stories §9). FIXED.
- R2 Nothing limited repeated chapter-2 events (4 of 5 'estranged') → "no event more than twice". FIXED.
- R3 Settlements vs reuse cap of 3 made location quotas impossible → settlements are containers (locations §2, rulebook §4). FIXED.

## A-syn (Old Hands + Night Market, paired)
- S1 Old Hands code removes 'justice done formally', the natural ending of their own code-break way (go to the magistrate). Code-break ways need their matching ending restored on the crisis.
- S2 Paired mode kills people met earlier, but 'the dead are newly created' and `dead` is static → need a `dies_at` (card/move) field: a person alive until a named move.
- S3 Once-per-family forces two different magistrates for one killing (paired crises share an event).
- S4 Reckoning 'Rank' shield can never apply while a Top emperor exists (no registry person outranks... except emperor) — shield is effectively dead.
- S5 Old Hands O3 (Won't sell, holder bought from a thief) contradicts Won't sell → should be Passed on.
- S6 Field-site / journey-away undefined for villages.
- S7 Mirrored Blood debt crises read as one card twice → paired crises should differ (one Blood debt, the other a different family on the same event).
- S8 No rule seeds the truce meeting (the paired last move needs a plant earlier).
- S9 One twist per group starved 6/10 Old Hands cards (twists too scarce in faction stories).
- S10 Reckoning dominates syndicate stories; reveals default to Investigate (both).
- Checker: crisis place compared to seat settlement (should resolve site → settlement); pool growth cap should allow shield + ally + code-break on crises.

## A-du (Dueling Society)
- D1 Contest champion default (player) broke crisis C4: the player would throw Kitim's blood bout. Kit crises must name the champion when it's a leader's fight.
- D2 Code-break way leads to a code-forbidden ending (throw it → 'thrown'). The code-break way's own ending must be exempt from the code (same as S1).
- D3 Ally plant unpaid when the optional crisis ally way isn't rolled → make the ally way mandatory on the crisis when an ally plant exists (or let any phase-3/crisis card pay it).
- D4 C4 sits very close to Reckoning Blood debt; 'old blood' needs an unconnected deed but the last move chalks the killing publicly.
- D5 No conflict-of-interest rule for the ring-keeper/arbiter (Liʻo keeps the ring for his sister's bout).
- D6 Reach vs scope: Dueling Society canon reach 'community' vs map presence in six towns vs 'region whatever the reach' (faction-stories §7) vs rulebook §5 'reach caps scope'.
- D7 Disgraced door opposing slot unclear in faction stories; faction proxy rule made a victim a 'proxy' via a rival link (proxy definition too wide: any link incl. rival/grudge).
- D8 Enemy appears only at the crisis — revenge stories need an earlier face-to-face (rule: enemy fills a slot on ≥1 phase-3 card).
- D9 The Hand fills culprit/wrongdoer on 3 cards → reads as the villain (cap the Hand at 2 hidden roles).
- D10 Chapter 2 cards all at one level (money and chalk) — progressive complication not enforced.
- Checker: slots holding fact ids flagged as missing references (fs-check). Fix.

## A-wl (Writers + Lantern)
- W1 Signature rule conflict: §6 'at least one routine card uses a signature' vs §3.2 roll (no signature rolled) → the phase rule should force one routine card's premise to a signature (choose which card at the start).
- W2 Is the leader cast? One-slot limit vs leader as arbiter; giver isn't a slot → leader 'appears' requirement should allow `giver`.
- W3 Fiyeweya is never at the Wayhouse, yet crisis there; kit names no crisis giver for Writers.
- W4 C2 needs the enemy to hold legal authority over the guild's own HQ (invented a writ) → C2 should name the writ as part of the last move.
- W5 Rescue 'Left behind on contract' lacks a truth table needed by 'justified captor'; unclear whether family main-story door filters apply in faction stories.
- W6 Death timing (same as S2): need `dies_at`.
- W7 Stem rule forces kin onto unrelated surnames → kin should share family names (exempt family names from stem rule).
- W8 Duplicate landmark created by two agents (Crownsgate toll bridge) → generated holdings' landmarks should be registry places created at map time, not by quests.
- W9 Same kit rolls (H4 + captive + C2) reproduce nearly the same Lantern story as the previous test → kits too narrow; entries need internal variants (who/why/how options) not just ids.
- W10 Both guilds rolled Threat 'Protected beast'; Writers had two Threats in routine (weights 'low' still allow 2) → low-weight families max once per story.
- W11 Above enemy never appears before the crisis (same as D8); Writers' Hand vanishes after the odd job → Hand should recur (≥1 phase-3 card).

## Story critic, World A (review-A-story.md)
Scores (A/D/E): Old Hands 3/3/3 · Night Market 2/4/4 · Writers 3/3/4 · Lantern 4/4/5 · Dueling 4/3/4.
- K1 Moves must advance the revealed scheme (Night Market abandoned its revealed trap).
- K2 Chapter cards sit side by side instead of chaining and climbing (main-quest §3 'different doors, same problem').
- K3 Meet them before they matter: enemy & Hand on ≥1 phase-3 card; anyone a move kills appears earlier; heart appears innocently on a routine card.
- K4 Paired syndicates are parallel stories, not two views of one feud.
- K5 World variety ledger across guild stories (reveals all Investigate; repeated premises/twists; 3 killing last moves; debt lever everywhere).
- K6 Names collide (3 Hestpers, 2 Dunmores, two black-cord clues).
- K7 Plant what the crisis asks for (heart's secret dangling; endings unsupported; ally way optional).
- K8 State tracking along the chain (cargo in one card, crew member in the next).
- Strengths to keep: old-debt stories re-staging the original wrong in the original place; quiet plants; hearts whose want pulls against the guild (Arlo the would-be toll clerk).

## B-main (Ripple, Above: Dorog Coin-counter, Sharp Hands; heart Brisa Tolland, Saltmarket New Quay) — 69 cards
- M1 Regions: locations §5 'each trials chapter in a different region' (whole story) vs main-quest §6 'per path' → use per path.
- M2 Climax place: locations §5.5 (earlier landmark) vs main-quest §6 (seat on a non-turn card/slot) vs §5 (climax/Return at landmark) disagree → unify.
- M3 Fallout count: 17 fallout vs rule 10 ceiling (each family ≤2 → 9×2=18): one more region move makes rule 10 impossible → scale the family cap with count.
- M4 Home-town quota: Saltmarket 11 cards vs quota 3–5 → story pins should count but quota is a floor not a cap for story-heavy places; heavy concentration at hook/Return town.
- M5 Late fallout: `checkpoint` tag has no schema field (used `late_fallout`).
- M6 Twist groups exhausted by layer 4 → forced 'none' on later cards (2 per group too tight for 44+ card stories) → scale with story size.
- M7 Schism arbiter: villain and lieutenant are the faction's leaders → parent faction judges (should be written).
- M8 Repetition: 11 Rescue (6 Imprisoned unjustly), 4 Raiders, because non-turn families roll uniformly → weight non-turn families by chapter problem and cap per-premise.
- M9 Heart offstage in all 9 trials chapters (Ripple) → heart (or fallback) on ≥1 card per layer, or a 'heart thread' card in trials.
- M10 Ally way never rolled on climax → make it required (as faction v4).
- M11 Minor advantages pile on the climax (5/8) → spread: at most 2 per target node.
- Checker check.py: minor quests depending on own chapter's unique plant flagged (allowed by assembly §6/§8); twist cap still 1 per group (now 2). Fix checker.

## Laz decisions (2026-10-26, mid-test)
- Deferred casting: quests output job descriptions (role specs) for people and places; a casting pass binds them (reuse-first scoring + backtracking), creating missing entries; repeat quest pass ↔ casting pass until settled. Layers become a soft preference. → write quest-casting.md after the test.
- Classes, not fixed names: Banner Lord, Free Lord, steward, captain… are classes with generated members per world.
- Canon leaders optional too (Laz: "why not?"). Proposed: the ROLE (archetype + load-bearing traits/secrets) is guaranteed where stories depend on it; the named canon person is the default filler, not mandatory. Kits reference archetypes, and entries needing a missing archetype are filtered.

## B-g2 (Lantern L5 + Dueling D5, layered v4) — fs-check 0 issues, 19 cards
- G1 Odd job + heart-on-routine + one-cast-slot collide (Lantern odd job needed Hand & heart on one card).
- G2 Leader counts as cast → leader as arbiter breaks one-slot on reveal. (v4 says leader appears via giver/authority — clarify authority-only slots don't count toward the cast slot limit.)
- G3 Nothing guarantees the heart a slot on a routine card → choose that card at phase start (like signature).
- G4 Rulebook §9 still says one twist per group for faction stories; v4 says two → fix rulebook.
- G5 Crossing (max 1) vs realm-layer reuse (§7a) conflict when >1 realm person reused → realm reuse of authorities/clients isn't a 'crossing'; crossing = sharing a main-story card's person in a meaningful role.
- G6 D5 'Beside' though he joined the Society; kin link to heart makes heart a proxy. 'Seized' (epilogue trigger) undefined; C1 'Captured in battle' fits hostage-in-own-yard loosely.
- G7 Realm reuse helped little: main story sits on the south coast with mostly proxies → court-state layer needed (now folded into casting).
- G8 Kits repeat across worlds (same D5 rolls → W9 again); Lantern at 9 cards climbs thinly; both crises same shape (at HQ, leader gives) → vary crisis giver.

## B-g1 (Old Hands OH4, Night Market N5, Writers W3) — fs-check 0, 32 cards
- H1 Kit roller didn't check crisis premises against the world ledger: 3 Betrayal crises in world B (> 2). Roller must include the ledger (crisis family+premise ≤2; ≤3 Reckoning crises).
- H2 Seat ending appended pushed NM to 5 endings (cap 3–4) → seat ending replaces one kit ending.
- H3 'One slot per card in phases 1–2' ambiguous (per card or per cast member) → per cast member.
- H4 Checker wants leader in a slot; §5 allows giver → checker fix.
- H5 No `depends_on` path to main-story facts for layer-3 cards; no opposing slot on Restore doors.
- H6 Reckoning 7/32 cards, all three crises Reckoning; one-twist-per-twist budget left Reckoning crises twistless.
- H7 Writers' code makes every Dispute truth+deal (side removed) → fine but monotone.
- H8 Realm reuse helped as texture but proxy/once-per-family limits pushed realm people into arbiter slots (G7 again → casting/court state).

## B-l3 (location: Rim, Green, Harbour) — 14 cards, 0 own issues
- L1 No profiles for minor factions (Last Wave, Wardens, Rooted) though presences give one-off jobs → write minor-faction profiles (light: weights + vocab + code).
- L2 Some twists missing from rulebook §9 groups → audit every family's twists into the table.
- L3 Premise roll ignores the place (Captured in battle at Saltmarket impossible) → requirements filter must check the location's available entities (a noble stronghold within reach).
- L4 Variety caps only per batch (Saltmarket repeated Imprisoned unjustly from the world pass) → per-location premise cap counting story cards.
- L5 Fixed-place landmarks aren't registry places (agents create ziggurat/Circle/Listening Trees) → same as W8: create landmarks at map time.
- L6 Contest naturally makes the other principal the fixer; 'one entry one role' rejects it → allow principal + fixer pairing in Contest (The fix) explicitly, or forbid by rule.
- L7 Textrim leans on the hungry-winter hook 3/5; two identical-shape Broken agreements (Last Wave/lowlander) → per-place hook usage cap (a place's hook used by ≤2 cards).
- L8 Cards rolled no way that reaches their own truth → truth-reaching way must be among rolled ways when a truth exists (rulebook §2.8).

## B-l1 (location: Vale, Roads) — 15 cards, 0 own issues
- V1 Holy House has no profile (same as L1) → minor-faction profiles.
- V2 Location cards can't use the main story's visible later moves (salt crisis) even where they're the most visible thing → let location cards depend on PUBLIC moves of layers ≤ the one the region first appears in? Or: towns touched by a move get it as context only (not dependency): a 'context' field that never gates availability.
- V3 Ten twists sit in no group (L2 again).
- V4 Lantern faction twist can never fit its own signature Rescue (twist table mismatch).
- V5 A reused person's fallback can't be added when entries must be copied unchanged (casting would handle; fallback belongs to the person-card pair).
- V6 Contest: no way to mark a requester competing in person.
- V7 Place features stack: Ninth Ring's leanings stacked → Reckoning 4/10, no Investigate/Restore → leanings should not stack beyond the strongest two; per-place family cap 30%.
- V8 Theme clustering: 5 Ninth Ring cards turn on lineage/house-name; both Mount cards 'the Armed Nave's men are brutal' → per-place hook/theme cap (L7).

## B-l2 (location: Marches, Reef, Snows) — 15 cards, 0 own issues
- X1 Binders no profile (L1 again) — presence cards had no giving faction.
- X2 May location cards depend on a faction story's public plant? (Brennock relied on a Lantern plant) → decide: yes for public plants of story cards in the same region, treated like hook harm (V2).
- X3 Presence roll 'under contract to a named faction' conflicts with a secret (crown backing) → presence rolls must read secrets and use the public front.
- X4 Fallbacks for copied people (V5); premade High leaders in arbiter slots exceed 3 (Shihi on 4) → authority-only exemption should cover premade High leaders in authority slots, not just Top.
- X5 Binder presences pull Marches cards toward bondage (6/15) → per-region theme cap.

## B-l0 (location: Crown Crossing) — 24 cards, 0 own issues
- C1 Linked pairs 'between near places' impossible when a batch's places aren't near (capital region) → allow within one settlement (sites).
- C2 'Copy reused people unchanged' vs fallbacks/new links (V5, X4) → casting owns this; links live in a link table, not inside people.
- C3 Assembly §8 echoes 'from a main node' vs brief 'any story card' → decide: any story card (main or faction).
- C4 Twists missing from §9 groups (L2, V3).
- C5 Signature premise rate under 1-in-3 in practice (0 of 4) → make it a quota per presence (every 3rd card), not a weight.
- C6 Rescue creeping: 16+ Rescue world-wide, 8 Imprisoned unjustly → world family/premise ledger for side quests too.
- C7 Thematic clustering per town (Highcrossing seals/offices; Southroad 4/8 Great Bridge) (L7, V8) → per-place hook/landmark cap.
- C8 Threat underweighted (1 card after reroll) — leaning weights too weak / Threat requirements too narrow in towns.
- Ops: parallel agents should prefix scratch file names.

## World B — story critic (review-B-story.md). Scores: Depth 4, Units 4, Whole 4 (variety 3, theme 5)
- SC1 Blank cards: 15 have no truth; most minors have no twist; Ordeal has 2 blank bouts → every card carries a truth or a real twist; Contest gets a truth row (what the other side really fights for); minors not exempt (rulebook §2.8, family-contest).
- SC2 Template ways: 2a1 and 3a2 have the same three ways → every rolled way names a person/place/thing from its own card (rulebook §2.3, card-schema).
- SC3 Captors never right: 20/21 Imprisoned/Bound-by-debt have an innocent captive or an unowed debt → 'guilty' and 'real and fair' as plain truths (~1/3); world ledger for truth rows (family-rescue §5).
- SC4 Main story meets-before-matters: Dorog and Quenna Ashby first seen at the climax; people taken by moves (Wainyard, Bogford, Shellring) are never rescuable → main-quest §7.6 introduce rule; every move that takes people gets a downstream Rescue (§9).
- SC5 Plant-backed ways unrolled (Far Sails ally way on 71/73; Lantern crisis uses ~2 of 8 plants) → plant-backed ways roll first on climax/crisis/turn cards (main-quest §7.5, faction-stories phase 4).
- SC6 Chain and climb missing in main chapters (only ch-5a chained, and it's the best) → carry v4 rule into main-quest §3.
- SC7 One world ledger across main/guild/location passes; reserve the main lever (debt) — 'bought a debt and called it early' ×5; ungrouped 'hidden motive' twist ×10 → every twist in a group (faction-stories §9, rulebook §9).
- SC8 Guild story events outweigh the main story (Lantern: Throne edict, war writ) → ceiling on a guild story's event scale (faction-stories §7, guild-kits).
- SC9 Dueling story repeats World A (kit D5 both times) → kit roller avoids repeats across saves? (per-world only; note for later: seed variety).
- SC10 digest.py drops twist_detail, motive, offense text → tooling fix; the best moments were invisible to readers.
- Recheck next run: M8 premise cap, M9 heart every layer, M10 climax ally way (fixed after B was generated).

## World B — playthrough diary (review-B-playthrough.md). Score 6.5/10 (guild stories ~8, main ~5). ~112 days, 72 walking.
- PD1 Heart & villains offstage: Brisa absent day 4→101; Dorog on one card; lieutenant Kodor met only as an absent owner → enforce heart-per-layer PER PATH; main story gets 'meet them before they matter' (= SC4); an owner slot doesn't count as meeting someone (they must be present).
- PD2 Wins don't stick: loan called in right after the player saves the payment; Return says quay still sealed after Dorog is cast out → moves must read player-resolved facts (a move whose target was resolved by the player changes face, not undone silently); Return text has variants keyed to the climax ending (main-quest §5, mechanics §5).
- PD3 Plants don't pay off: Far Sails ally planted 3× never rolled (= SC5); hook's stolen mules never found → the hook's lost thing is a tracked thread with a payoff card; revelation landed on a mastiff Threat card → revelation only on cards whose family can carry it (Investigate/Reckoning/turn), or the card gets a matching way.
- PD4 Commuting: Saltmarket↔Openharbour 8×; climax rescue offered 2 days away for a man from an untaken sibling chapter → per-step max distance for faction stories (next card ≤ ~1 day from the last, or same region); climax cards may only use cast from the player's chosen path (or have a fallback); offered_at may be near the player's path, not only the requester's home.
- PD5 Stories never touch / seams contradict: Lantern war writ reaches no town (faction fallout optional → make ≥1 faction fallout per story-scale event); capital never mentions salt (location cards in touched regions get public moves as context, = V2/X2); a Night Market job mentions salt notes before they exist (card text must only use facts its depends_on/plants make true at its time); a second lord of Breakwall invented (§7a not enforced → checker: no new authority over a place that already has one).
- PD6 Saltmarket hosts 7 main cards (> cap 5) → checker should catch it (did it? add).

## World B — coherence reader (review-B-coherence.md). Score 3/5. Secrets clean.
- CO1 Towns blind to big stories (Textrim no salt crisis; Marches never see the war writ; Chainmarket ignores debtors sent there; Crownsgate's 7 cards never see each other) → per-settlement `context` digest (public facts from moves/events touching it; never gates availability); ≥1 card per touched settlement references it. LOCATION-BRIEF contradicted locations §1/§9.5 — align.
- CO2 Timeline contradictions (Absalom's debt bought & called same day vs 'a year of passes' vs 'worked off long ago'; seal-box sold yet seal used) → past-event facts carry `when`; things carry `held_by` + `since`.
- CO3 Two lords of Breakwall (invented Hawksmoor beside canon Kotqot) → a seat/office binds to its registry holder; no new holder for an occupied seat (= PD5).
- CO4 Ormond Vale in two offices after fleeing (mi6a vs nm-r2/nm-crisis) → Fled/Changed shields write state facts on the person; later cards read them.
- CO5 Revelation's victims never followed (bonded → Chainmarket; Return brings a new villain; mules never found) → a minor quest at the revelation's destination; Return truths may cite the revelation; hook's lost thing is a tracked thread (= PD3).
- CO6 Duplicates (two 'only records' of the crews' debts; invented hamlets Thornby & Thornwick at one site) → reuse a site at the settlement before creating; 'only X' objects become registry things.
- CO7 Endings remove people needed later (Kodor 5a3→72; Sallow ln-rev→c1a/c1b; Sevrin oh-c1b→c2c) → endings that kill/drive off a person needed by a later card are either removed or the later card gets a fallback (casting owns fallbacks).
- CO8 Motives fail after the last move (Yaweth's debt paid by her own last move yet she runs the score; Ambrose's scheme unneeded; named revelation costs never paid) → the last move may not satisfy the villain's want; a revelation's named cost must be paid by a later card or move.
- CO9 Geography text errors (south/north; 7-day fugitive run; country estate in a reef village; 'three weeks' vs 6–9 days); Saltmarket 7 main cards > cap 5 missed by checker → card text directions/durations come from the map (generated, not written); place kind must fit the premise; checker enforces §5.6 cap.
- CO10 Unpaid plants (Ysmay's weigh-book; Far Sails ally on 73; nm-r3; water-walk) → checker enforces main-quest §7.5 ally-way-rolled; plant-backed ways roll first (= SC5).
- CO11 Name collisions touching the cast (two Wennas, two Ilsas, Sallow/Sowle; siblings with three surnames) → given names unique per region among cast-touching people; kin share a surname (W7).
- Working: hook→bond clause→Quenna→Binders price→revelation chain; cross-layer echoes (strike kitchen, mule fair, Names Post, Brennock halt, reed path, Wellstone); Elric/Cador payoff; all five faction crises rolled their ally way; Lantern and Writers tightest.

# World C (rules v5.2, schema v1.10, casting v1.1) — 2026-10-27

## Kit roller (roll_kits_c.py)
- KC1 §9 "never twice in the same phase position" vs "crisis family+premise at most twice": applied to odd jobs only → clarify which pools the phase-position rule covers.
- KC2 §10 truce killing: counted as one killing for "≤2 last moves that kill" → state it.
- KC3 Dueling: does the sister (Liʻo) count as "the leader" for the leader-enemy cap? Roller counted sister and showman both → clarify co-leaders.
- KC4 §8 enemy-side rule also bars W2 with L1 (both Free Lords) → intended? (one enemy side per faction across stories).
- KC5 §9 main-lever cap vs rulebook §9a wording differ; kits carry no levers, so the reserved lever can't constrain kit rolls → move the check to move-writing time (each story's moves), say so.
- KC6 Paired mode: the hidden-hand cap bars W2, W4, L3, L4, D1, D3 (paired already uses 2) → paired worlds are narrow; consider counting the paired pair's reveal as one.

## Layers 0–1 (c-l0): 29 archetypes (22 canon, 7 generated), 28 seats + 76 titles, 85 created, 4 tensions
- L0-1 Casting §8 keys archetype rolls per archetype; a single seeded stream shifts every roll when one is added → the test brief was wrong, not the rule (note for tooling).
- L0-2 No `keeps` list per archetype without leaders.md → archetypes data must list load-bearing traits/secrets explicitly (casting §4 data row says so; needs writing).
- L0-3 Kin share a family name vs canon twins/adopted siblings who don't → canon names exempt (rulebook says canon exempt; generated fillers of canon kin: keep canon relation, generate a shared name).
- L0-4 Co-leaders sharing one seat: which is the seat holder? "Nobody in two offices" isn't written → rulebook *Seats and offices*: add one-office-per-person (except canon) and co-leader seats name one holder + a title for the other.
- L0-5 Casting §3: do guild contacts and land presences count as offices pre-made at layer 0? → state: in-town hall/contact keepers are on-demand class members, not pre-made.
- L0-6 "One tension per major faction" ignores patrons/territory (schism on First Peoples land beside the Firststone claim) → count a tension against every major faction whose land or people it involves.

## Layer 2 (c-main): Rising/Above, L5×W2, 40 cards (21 main, 2 Return, 5 minors, 12 fallout); checker clean; reuse 26% (35% counting cast slots)
- CM1 main-quest §6: Rising needs a realm-reach antagonist faction; the Sharp Hands are region → allow a sub-faction to borrow its parent's reach, or say Rising needs realm reach of the side *or its parent*.
- CM2 rulebook §9 vs assembly §4 / main-quest §7: filling cards in chapter order exhausted twist groups before the climax and Return → roll the climax and Return twists first (state the order).
- CM3 assembly §7 with W=2: linked-node variety always bars a Reckoning ordeal door → loosen for W=2.
- CM4 rulebook §9a: the main story alone fills four truth rows to the cap of 3 → the truth-row cap must scale with world size (like the premise cap ~4% of cards).
- CM5 main-quest §4: landmark standing rule doesn't say site or settlement.
- CM6 assembly §8: a minor on the Return node is impossible (nothing follows it) → exempt the Return from minors.
- CM7 fallout §4: the allowed Restore causes are easy to miss → move into the rule text, not a sub-list.
- Tooling: cast_how gained a 4th value `cast` (story cast slots) → add to the contract; checker shows both rates.

## Layer 3 plan (c-gplan): 5 stories, 50 stubs, all §8b numbers pass (verify 496 checks); busiest region Crown Crossing 28%; no crossings (layer 2 made no High people, so all three Above enemies were created)
- GP1 Cast before placement (faction-stories §12 step 2 vs casting §1/§6): casting scores need a card's region, but cards come later → cast after the placement plan, or score against the story's home region.
- GP2 Higher-layer entries can't be updated (casting §1), yet a lower layer needs dies_at / links / kit facts on reused layer-0 people → allow "add-only" updates (facts, links, dies_at) recorded as registry_updates; say so in casting §1.
- GP3 Paired meeting (§8b.3 vs §10): the meeting's guild and rival roles don't fit each enemy appearing in its own guild's story → word the paired exception exactly.
- GP4 Shared truce move (§9): counts twice for levers and kills; fallout limit unclear → count a shared move once.
- GP5 Field sites beside shared towns count toward the town's caps but are "field" for pinning (§6, §8b.2) → define "beside".
- GP6 Epilogue trigger "seized" undefined (§6 phase 4).
- GP7 Lantern C1 holds the named chapter at Carter's Rest, outside the march whose writ names it (event ceiling §7) → kit text.
- GP8 Layer 2 made no High people, so the crossing / Above-from-layer-2 path never fires → casting should prefer layer-0/2 High people for Above enemies when they fit (it may be working as designed; watch in more worlds).

## Layer 3 cards, Lantern + Dueling (c-g2): 20 cards; world_check clean for its cards; reuse 41% (31% world)
- G2-1 Bouts that can turn deadly (Dueling kit, Contest §7) vs *Endings don't remove people a later card needs* (rulebook §3): a fallback can't make a later plant true → a person who carries a later plant may not stand in a deadly bout (or the plant moves to the fallback too).
- G2-2 Reckoning *Ruin from above* has no legal shield when requester and wrongdoer are both Middle (Reckoning §5, rulebook §4) → the premise needs a standing gap, or allow a Middle-vs-Middle shield (bought, protected).
- G2-3 A reveal on a Restore card clashes with Restore's "no deliberate harm by someone on the card" (Restore §1) → bar Restore as a reveal family, or exempt the reveal.
- G2-4 Two card writers fill the guild ledger caps blind (§9) → the plan must reserve premises per story (or the writers run in sequence).
- G2-5 Truth rows run out (rulebook §9a): Acquire had only "promised elsewhere" left → the truth-row cap must scale with world size (CM4).
- Checker: fs-check ignores the Lantern code's "when hired by the requester" condition.

## Layer 3 cards, Old Hands + Night Market + Writers (c-g1): 30 cards; checker clean for its cards; reuse 45% (36% world)
- G1-1 Two writers share one ledger blind (§9): merging showed 4 guild premises + 1 twist over caps; g1 fixed its 2; Blood debt, Lost good name, First blood or last stand at 3 (> 2) from g2 → left as found (generation order must be sequential, or the plan reserves premises).
- G1-2 Does the main story count toward the guild "family+premise ≤2" cap? (§9 vs rulebook §9a) → say: main counts in the world ledger caps, the guild-only caps count guild stories.
- G1-3 A family's truth rows can run out world-wide (Acquire full) → scale (CM4, G2-5).
- G1-4 Shared truce vs "meet them before they matter": fs-check wants each story to meet both dead; one is at the reuse cap → §10: each story meets its own dead only; fix the checker.
- G1-5 The planner put the fallback on two cards in one chapter (§8: one slot per chapter per cast member) → planner check.
- G1-6 An ally plant on a card of the crisis's family hits once-per-family (§6, rulebook §4) → exempt cast/plant knowers, or plant on another family.
- G1-7 A twist that fixes the truth is rolled after the truth (Contest §5b) → roll order: twist before truth when the twist decides it.
- Reuse observation: the casting score makes a same-region or same-story person beat creation almost always (45% here) → create_score may be low; watch the world total (36%, within 25–45%).

## Layer 4 batch 3 (c-l3b: Marches, Rim, Roads, Snows): 15 cards, 0 own issues; reuse 19% (world 32%)
- L3B-1 **The world ledger's truth-row cap (3) closes whole families** by layer 4: every Acquire and Passage truth row full, Reckoning one left → Contest became the batch's biggest family. Strongest signal so far (CM4, G2-5, G1-3): the truth-row cap must scale with world size (e.g. ~2% of cards, min 3) and parallel writers can't see each other's counts.
- L3B-2 Twist-then-truth order unspecified; twists needing a particular truth clashed (G1-7 again).
- L3B-3 Same-region public plants let a location card rest on a later move's world, though moves should only be context → limit location dependencies to plants made at or before layer 2's hook (time-ordered), or to plants whose card is offered before.
- L3B-4 Reuse is low in remote regions (Rim, Snows have few existing people) → expected; the aim range should be world-wide, not per batch.
- Story: "a blamed bystander" is stock at the Chainmarket (3rd) → per-place theme cap should catch it; check the theme tags.

## Layer 4 batch 0 (c-l0b: Crown Crossing): 16 cards, 0 own issues; reuse 19% (27% counting looked-up holders); world 32%
- L0B-1 Place leanings point at premises already at the world cap (a lord's seat → Imprisoned unjustly, Ruin from above, both at 4) → when a leaning's premises are capped, the leaning moves to the place's next feature (state it).
- L0B-2 The ~4% premise cap can't be shared between parallel batches → a real generator runs batches in sequence; for paper tests, reserve shares per batch.
- L0B-3 Contest *The fix*: one person as fixer and the other side / paid fighter as champion → rulebook allows the pairing (family-contest) but the checker flags two slots → checker exemption.
- L0B-4 Oathford has no `context` list though m4's own fallout card is pinned there → settlement context must be written for fallout-touched towns too (main-story agent omission or rule gap: "touched" should include fallout pins).
- L0B-5 Restore *obligation* forces a newly created creditor where a local moneylender would do → allow reuse.
- L0B-6 Kin share a family name makes Contest's *old blood* twist hard to hide (the challenger carries his dead father's name) → the twist hides through a different name (mother's line / byname); note.
- Story: three Highcrossing cards turn on standing at court; Southroad's second Encroachment repeats the first's water-harm shape → the per-place cap counts premises; add "same harm shape" to the theme cap.
- Reuse: towns with no story cards have no people but office holders → low reuse there is expected.

## Layer 4 batch 2 (c-l2b: Greenmarch, Harbour, Reef Coast): 15 cards, 0 own issues; reuse 26%
- L2B-1 Acquire blocked by the ledger (all three truth rows at cap) → only signature premises can roll Acquire; the Rooted's "What the grove asks for" is dead too (L3B-1 again).
- L2B-2 Parallel location agents overrun caps (each reads only the world-pass ledger) → sequential generation (L0B-2).
- L2B-3 Signature quota at fixed places (the Soft Hollow, the Green Wall give faction jobs) undefined → the per-presence quota also applies to fixed places that give faction jobs.
- L2B-4 Plan gives Wellstone 0 but the checker flags empty places → checker: a place with quota 0–1 and target 0 isn't a coverage failure (only settlements).
- Story: **every card has a twist or a secret truth, so no plain level-1 quest exists for contrast** → *No blank card* may be too strong for side quests: allow a share of plain cards (a simple truth row "as it looks" counts as a truth, so a card can be plain and still not blank — clarify that "as told/as it looks" is a valid truth).
- Story: Openharbour 4/15 Investigate — heavy for a port.
- Reuse made regions feel connected (one Reefhold slaving house through three cards).

## Layer 4 batch 1 (c-l1b: Elder Vale, Wide Bay): 16 cards, 0 own issues; reuse 12%
- L1B-1 Ledger + parallel agents: 9 rerolls to respect the 4% premise cap (L0B-2).
- L1B-2 Truth rows full → two Acquire cards carry a twist and no truth; do signature truth rows share their base family's rows? → state: signature rows count separately.
- L1B-3 Signature quota: a presence offering one card must make it a signature job → would repeat its premise → quota "every third card" should round down for 1–2 cards (the first card of a presence is free).
- L1B-4 Petition/patron ways can't be filled against High parties (only Top person is the emperor) → ways needing an outranking person must be filtered when no one outranks (rulebook *Outranking slots*).
- L1B-5 No route kind for the sea; a linked pair's second card has no valid `when` for a past event; court-state facts not listed as settlement context → small schema/rule gaps.
- L1B-6 Reuse 12%: Vale's Low/Middle people are cast, captive, or at the 3-card cap → reuse is starved in story-heavy regions too; the 3-card cap may be tight for ordinary townsfolk in story regions.
- Story: two Encroachment cards rolled the same three ways; two *Passed on* cards too (*Ways name what's on the card* passes but the way kinds repeat) → a per-place "same way-set" check.
- Story: five Vale cards turn on the Mount's protection-for-a-tithe trade → place hook cap (≤2) not applied to a trade? → per-place caps should include the trade.

## World C — story critic (C/review-C-story.md). Depth 5 (B 4), units 4, whole 4 (variety 3, theme 5, memorable people 5). Main 4 (B 3); guilds W4 OH5 NM4 L4 D4; sampled side quests 3.8
- Showing now: captor can be right, meet before matters (Feyesh on the first card), plant-backed ways first, all 9 main chapters chain and climb, event ceiling holds.
- SC-C1 Confirming truths ("as told/as it looks") + twist none on 14 cards read as blank → mark confirming rows; a card with one needs a twist — **except** a share of plain side quests (tension with L2B: "no plain quests for contrast"). Decide: story turn cards and crises need a non-confirming truth or a twist; side quests may be plain (confirming truth, no twist) up to ~1 in 4.
- SC-C2 Ledger keys split one truth into many (bystander culprit ×7 under suffixes; both partly right ×7; bad water ×6) while small families blank out → count variants under one key; scale caps for small tables; capped cards reroll the premise, never blank the truth.
- SC-C3 Turn cards with no twist (main 26/40 twist none; all ch4b; crises oh/nm/w) → spend twist budget on turn cards and crises first; ≥1 minor in 3 has a twist.
- SC-C4 The heart's secret first appears in the Return → port *plant what the crisis asks for* to main stories (heart's secret planted on trials/Ordeal).
- SC-C5 The main mechanism (debt/notes) isn't reserved (main lever recorded as bribery) → reserve every lever the main story uses on 2+ moves; ledger counts the mechanism incl. ordeal events and side premises built on it.
- SC-C6 Way stems repeat ("go over the charge with X…" ×5) → ledger counts way stems ≤3 per world; filled ways name a place feature or thing when the card has one.
- SC-C7 Same ordeal everywhere (heart disowned/cried down in main + 4 guild stories) → main Ordeal event counts in the guild chapter-2 event cap; *the cost* reveal ≤2 with varied wording.
- SC-C8 Echoed twist actors and shapes (Agnes as "better buyer" twice; "loosened in the night" twice; *the fix protects someone* in a third world) → a background High person named in a truth fills that role once per world; a twist's shape ≤2 per world.
- §8b: every check passes; Old Hands–Night Market reads superbly; other meetings just fill a slot; the Writers–Old Hands trade pair should meet and never does (trade meetings are optional) → fine by rule; consider "trade pairs meet if a shared town allows".

## World C — coherence reader (C/review-C-coherence.md). 3.5/5 (B 3/5). Secrets clean.
- Fixed since B: two lords of one keep, a man in two offices, the debt-timeline muddle, revelation follow-through, unpaid plants, geography text, "only" duplicates.
- CO-C1 **Faction moves reach no town and no other story** (high): settlement `context` holds main-story facts only (Longwall's 7 cards ignore the levy muster; Highcrossing's writ, letters and truce poisonings never notice each other; Lanternhall's occupied yard unseen; the main climax bout at the Ring while the Dueling story revokes its licence) → the world pass adds faction-story moves to `context` (locations *Settlement context* must say faction moves too, and the guild planner writes them); realm scope = every settlement.
- CO-C2 **One truce killing, two crises disagree** (high): NM truth blames Lysander (explains Idris, not Edric); OH crisis calls Thaddeus the killer; Lysander in two places → §10: one crisis fixes the shared move's truth (who killed whom, where), the other cites it.
- CO-C3 Context text states things before they happen; four cards show fact ids/notes in player text → a card that reads a move's context is available only after that move; checker flags ids in text.
- CO-C4 **Doubled canon leaders**: generated fillers (Isolde for Yitolaʻo, the Saltonstalls for Dorog/Kodor, …) but the canon people stay listed as faction leaders → casting §4: a generated filler *replaces* the canon person (faction `leaders` point at the filler; the canon person doesn't exist that world). Also a test-tool bug: merge.py loads every canon person regardless.
- CO-C5 Two removable people without fallbacks (Malcolm Tredgold, a blood bout's stake; Piers Cawdrey before mi-5) → fallback check misses blood-bout stakes and minors.
- Lower: Agnes buys an oldest-hand reader twice; six near-identical bad-water cards; small contradictions; 33 shared first names world-wide (the regional rule allows it).

## World C — playthrough (C/review-C-playthrough.md). 7.5/10 (B 6.5): main 7.5 (B ~5), guilds 8, side quests 6.5. ~125 days, 70 walking; all ten regions visited, but the Rim and the Snows only by choice
- Fixed since B: heart and villains on stage; wins stick. Mostly fixed: plants pay off. Better in the main story only: commuting. Half better: stories touching.
- PD-C1 Same shapes repeat (seven water-spoiled-uphill quests on one path; two "wants off the ladder" minors; Agnes buying a reader twice) → the ledger counts truth rows, not the shared mechanism (= SC-C2, SC-C8).
- PD-C2 Shared sites contradict across stories (the Ring's licence revoked, then the climax bout held there) → a shared site's state is a fact all stories read; whichever story changes it first, the others' text must hold under either order (or the change is a move other stories cite) (= CO-C1).
- PD-C3 Payoffs don't land: the climax's counter-mark way wasn't rolled (forced ways filled the 3 slots); the heart's secret first appears in the Return; Return casting creates new kin → forced ways overflow (core M4 again: when forced ways fill the slots, the climax may roll a 4th); heart's secret planted earlier (= SC-C4); Return reuses established kin.
- PD-C4 Walking and a cold climax: the Writers' escort doubles back; 52 days before the climax's first card, then an 18-day minor loop → a maximum gap between main layers; minors near the node.
- PD-C5 Clocks and late sibling chapters read wrong: a clock ending contradicts its own card; a "lost" village still hosts a quest; ch-2b reads stale on day 115 → a clock's world_ending writes state other cards read; sibling chapters taken late need a "later" face (mechanics).
- Region note: the Rim and the Snows have content but nothing pulls the player there → Laz's "don't exclude regions" is met for availability, not for pull; main/guild paths could route one card through each remote region over the world (watch).

# World D (rules v6 + v6b + v6c) — 2026-10-27

## Layers 0–1 (d-l0): 29 archetypes (23 canon, 6 generated), 27 seats + 12 titles, 39 people, 3 tensions
- DL0-1 Kin share a family name vs canon bynames that are deeds (Kitim) and Qotiro naming with no family name (casting §4.3 vs rulebook *Name check*).
- DL0-2 Do generated fillers keep the canon voice? (casting §4, §7). Canon voices/ages aren't in any doc the generator reads.
- DL0-3 Gendered archetype roles must keep their sex (the sincere priestess) → fixed as a `keeps` item; rule gap in casting §4/§2.1.
- DL0-4 Seat holders' people from the town mix put a Qotiro head over a Seated Houses village → use the seat's faction (casting §7).
- DL0-5 Co-leaders on separate seats can't also hold a title (one office per person); are toll captains, harbour-masters, parish priests pre-made? (casting §3).

## Layer 2 (d-main): Rising/Above, L6×W2, 48 cards (29 main incl. Return, 3 minors, 16 fallout); reuse 25%
- DM-1 Checker set caps from the cards it saw, not planned N → fixed (QN env).
- DM-2 Justified captor on Imprisoned unjustly removed the only ways reaching *guilty* (already decided: witness way stays; the agent read an older copy?).
- DM-3 Rising's "farther than near" plus ~3-day openings nearly fix the geography on this map.
- DM-4 A move's scope isn't tied to its layer (m2 reaches two regions from community chapters).
- DM-5 Checker flags requester = estranged party / cargo where the family's natural requester is that person.
- DM-6 Fallout Investigate with a bystander culprit fits only Theft and Suspicious death.
- DM-7 The Return had to create the bridge-keeper's son (no shown person could be him).
