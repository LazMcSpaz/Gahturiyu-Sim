# Factions in the Generator — Locked (v2.6, minimal)

This doc covers only what the quest generator needs from factions. The factions themselves live in two worldbuilding docs, and this spec reads both:
- **`factions.md`:** who each faction is, its sub-factions, voices, and depth (virtue, wound, hidden side).
- **`leaders.md`:** the leader archetypes of every faction and sub-faction, each with its canon person (the default filler), traits, goals, and foils.

## 1. What a faction is to the generator
- **A fourth registry entry type**, alongside person, place and thing.
- **Pre-made only.** Factions come from the world, and quests never create them.
- Faction leaders are archetype fillers, and fillers are pre-made people (§2). **Top-standing leaders count toward the Top roster.**

## 2. Faction entry spec

| Field | What it holds |
|---|---|
| **Name** | e.g. "The Free Lords" |
| **Size** | major, minor, or guild |
| **Parent** | another faction, or none. Covers sub-factions such as the Free Lords under the Great House. |
| **Leaders** | the fillers of one or more leader archetypes from `leaders.md` (several factions are led by pairs or partnerships) |
| **Seat** | a place |
| **Reach** | community, region, or realm. A sub-faction's reach can't exceed its parent's. |
| **Want** | greed, revenge, cover-up, desperation, ambition, or **preservation** (keep things as they are) |
| **Virtue / wound** | one public line each, taken from worldbuilding |
| **Hidden side** | one **secret** fact, taken from worldbuilding (e.g. "many members hate what they do") |
| **Relations** | toward other factions: *ally*, *rival*, *at war*, *trade* (dealings without loyalty; used between guilds, `quest-faction-stories.md` §8b), or *serves* (vassalage to a faction that **isn't** its parent) |

- **Inherited relations:** a sub-faction inherits its parent's relations unless it has its own entry for that faction.
- **Inherited character:** a sub-faction also inherits its parent's virtue, wound and hidden side, unless the worldbuilding gives it its own.
- **Leaders are archetypes from `leaders.md`.** Each world fills each archetype with its canon person (the default) or a generated person who keeps the traits and secrets that stories depend on. A few archetypes are guaranteed (the overview's lore-critical list). Either way the filler is pre-made: it exists before any quest.
  - **A generated filler replaces the canon person:** the faction's `leaders`, its seat, links and foils all point at the filler, and the canon person doesn't exist in that world (casting, *The filler replaces the canon person*).
  - **Co-leaders sharing a seat:** the seat names one holder; the other holds a title at the same place. Both count as "the leader" for leader caps (rulebook, *Seats and offices have one holder*).
- **Seats** come from `faction-places.md` (each faction's fixed seat).
- **Reach** follows from where a faction's places land (faction-places §6). It stays a *world assumption* until a world is generated.

### Reading `leaders.md`
| `leaders.md` gives | Generator field |
|---|---|
| Name and byname | the canon filler's `name` (a generated filler gets a generated name) |
| The archetype itself | the filler's `archetype`; story kits name the archetype, and give a canon name only as the default |
| Goals | the leader's want (matched to the shared want list), and story hooks for the antagonist's want or the ally's price |
| Traits marked *secretly*, or a hidden tie ("secret Binder ties", "lost his faith", "secretly principled") | a **secret fact** on the leader. It may also serve as the faction's hidden side when the faction has none of its own. |
| Other traits | public traits on the person |
| Foil pairs (the freed bondswoman and the ex-slave Binder, the two Banner Lords, and so on) | **rival** links between the leaders, and natural **fault lines** when one of their factions is the antagonist's side |
| Two leaders who compete inside one faction (the Banner Lords, the Holy House) | ready-made **Succession** or **Schism** material |

**Leader standing:** standing belongs to the archetype, so a generated filler keeps it. The emperor is Top. Other leaders are High unless the worldbuilding says otherwise. Top leaders count toward the Top roster.

## 3. Links
- **Member:** a person belongs to one faction, plus that faction's sub-faction if any. Exclusive factions are a world rule, and the generator respects them.
- **Joinable factions** are the five guilds and, since v6b, **the Holy House** (Laz, 27 Oct): a minor faction you can join like a guild, with its own faction story (`quest-faction-stories.md`; its kit in `quest-guild-kits.md` §6). Its size stays *minor*.
- **Exclusive pairs** (the player's memberships; rules in mechanics §9):
  - the Old Hands and the Night Market: joining one closes the other's story and sets its members hostile;
  - **the Holy House and the Dueling Society** (Laz): joining one closes the other's story and its hall jobs, and the reverse. The house bans rings (canon). **Not hostile:** the other's members aren't set against the player. Both stories are still generated;
  - the Rooted: joining is one-way, and removes every other membership.
- **Backed by:** a party on a card can be backed by a faction without being a member.

## 4. How cards use factions
- **Factions act through people.**
  - A faction never fills a slot itself. It acts through one named **representative** (a person slot), and that person follows the normal reuse rules.
  - Any way through that names a faction ("a neutral faction's protection," "an allied faction") fills a representative slot for it.
- **Membership is a tag on a person, not a slot.** Two cards that both involve the same faction aren't **threaded** unless they share a person or a plant.
- **Reach caps scope.** A faction can only drive a card (as attacker, backer, or the antagonist's side) at or below its reach. See main-quest §4 for card scope.
  - **Exception, Rising:** in a main story's Rising arc, a sub-faction may borrow its parent's realm reach for its moves and the cards they drive (main-quest, *Scale arcs*, Rising). Its own reach doesn't change, so it never exceeds its parent's.
- **Relations are requirements.** Set pieces check them.
- **Reuse:**
  - Factions themselves are exempt from the cap and from once-per-family.
  - In any one world generation, a faction can be the antagonist's side in only one main story.
- **Reasons:** a member's or backed party's reason may come from their faction's virtue or wound. **The family's requirements still apply.** For example, *obligation* still needs a creditor/debtor link.
- **Secrets:**
  - A faction's hidden side is a secret fact on the faction entry. Cards may build on it but never state it openly.
  - **Exception:** the main story's revelation may publish the hidden side of the **antagonist's faction**.
  - **Exception:** a faction story's revelation may publish its **own** faction's hidden side (when its kit allows) and its own places' secret hooks.
- **Faction stories:** a faction with a faction story (`quest-faction-stories.md`) is never the main story's antagonist side; since v6b that includes the Holy House. Any faction is the enemy's side in at most one faction story per world, and a *Within* enemy makes its own faction that side (faction-stories, *Enemy factions*).
- **Reputation stakes:** a faction entry may carry facts, including a reputation that a faction story puts at stake.

## 5. Parked (mechanical layer)
- Reputation
- Relations shifting over time
- Lords switching sides
- Joining and leaving factions
- Hostility triggers

## 6. Changes from review
- Added a hidden-side field.
- Removed overlap between *serves* and the Parent field.
- Added inherited relations and a reach limit for sub-factions.
- Factions now act through representatives.
- Sharing a faction no longer counts as a thread.
- Faction reasons still have to meet family requirements.
- Top leaders count toward the Top roster.
- **v2.1 (full test):**
  - sub-factions inherit virtue, wound and hidden side
  - missing seats and leaders are filled as *world assumptions*
- **v2.2:**
  - The generator now reads `leaders.md` for leaders.
  - The leader field became a list.
  - A mapping from leaders' goals, secrets and foils to generator fields.
  - Only seats and reach remain as world assumptions.
- **v2.3:** seats now come from `faction-places.md`; only reach is still an assumption.
- **v2.5 (2026-10-26), audit fixes:** leaders are archetypes, filled per world by the canon person or a generated one who keeps the traits and secrets stories need; fillers are pre-made; kits name archetypes.
- **2026-10-27 (v2.6), v6: World C fixes:** a generated filler replaces the canon person (CO-C4); co-leaders: one seat holder, both count as the leader (L0-4, KC3); one enemy side per faction across faction stories (KC4); Rising's borrowed parent reach is an exception to *Reach caps scope* (CM1). Pointers only.
  - v6b: Laz's guild changes (27 Oct): the Holy House is joinable, with a faction story; joining it closes the Dueling Society, and the reverse, not hostile; exclusive pairs listed (§3); the Holy House can't be the main antagonist's side; a *Within* enemy's own faction is its side (§4).

## 7. Decisions made without Laz
- **v6b:** the Holy House stays a *minor* faction in size, though joinable; the exclusive pairs are gathered in one list (§3); "a *Within* enemy makes its own faction the enemy's side" is stated here as the kits already read it.
- **Open for Laz:** since the Holy House can no longer be the main story's antagonist side, the main story loses a natural zealot villain. Is that wanted, or should a joinable faction's *sub-group* (the inquisitors, say) be allowed as a main antagonist's side?
