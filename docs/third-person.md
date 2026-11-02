# Third-person control (Laz, 2 Nov 2026)

Laz is pivoting the player's control from Kenshi-style point-and-click to a
KOTOR-like scheme: you move one character yourself in third person with WASD,
and the squad mates act on behaviours you set, plus orders you can queue while
paused. **Built and tuned in dungeons first**, then taken outside.

This file holds his answers (2 Nov) and the defaults Claude chose where he
didn't say. A line marked *(default)* is Claude's call, open to change.

## Camera and aim

- **Behind the character**, following at a short distance. *(default: about
  4 m back, a little over the shoulder; the wheel zooms 2–8 m.)*
- **Crosshair; the mouse turns the camera.** The pointer is hidden and
  attacks go where the crosshair at the screen's centre points. Esc frees the
  pointer for menus; any open panel (pack, journal…) frees it while it's open.
  *(default)*
- **W goes the way the camera faces**; S back, A and D sideways.

## Keys

| Key | Does |
|---|---|
| W A S D | move the controlled character |
| Mouse | turn the camera (and the aim) |
| Left click | attack toward the crosshair: one swing or shot per click |
| Right button (hold) | block / parry with what's carried |
| Shift (hold) | sprint: faster, costs stamina by the metre (Laz, 2 Nov, later) |
| Space | jump |
| Q | dodge backwards (costs stamina, briefly harder to hit) |
| C | sneak, on and off (the game's stealth rules) |
| E | use what the crosshair points at, within reach: open a door, climb a ladder, open a chest, read a plaque, talk; a small label says what E will do |
| Alt | pause and unpause |
| Tab | the next squad member; clicking a portrait takes that one |
| B | the squad's behaviours panel |
| Esc | free the pointer / the game menu |

- **Alt currently shows labels on everything** (the long tooltips). In
  third-person mode Alt pauses instead, and the E label replaces hover
  labels. *(default; outside third-person mode Alt is unchanged)*
- Q turned the camera before; the mouse does that now.

## Fighting

- **True aim.** A melee swing covers an arc in front of the character, its
  reach and width from the weapon's existing data (`Space`: short, medium,
  long, sweep). An arrow, bolt, thrown weapon or spell flies along the
  crosshair's line and hits the first body in its path. Area spells burst
  where they land.
- **Friendly fire: ranged and spells only.** Melee swings pass by squad mates;
  arrows, bolts and spells can hit them if they're in the way.
- **Left click attacks, right button blocks.** Each click is one attack, at
  the weapon's existing wind-up and recovery. Holding the right button blocks
  or parries, using the existing off-hand and parry rules, at a stamina cost.
  *(default)*
- **Damage, armour, wounds, bleeding, stagger and knockouts stay the game's
  existing rules.** Only who decides the controlled character's actions
  changes: the player instead of the fight AI. This is the player exception
  in CLAUDE.md, like the squad's own walking.
- **Jump** uses the generator's prototype numbers so its jumpable gaps agree:
  up at 6.2 m/s under 18 m/s² (about a metre up, a couple of metres across).
  *(default)*

## Squad mates

- **Switching (KOTOR):** Tab cycles and a portrait click takes that member.
  The camera glides to them, and the one left goes back to their behaviours.
- **Behaviours, per mate:**
  - **Stance:** aggressive (seeks fights), defensive (fights what attacks them
    or you), passive (avoids fighting).
  - **Role and range:** close in to melee, keep distance and shoot, or stay
    back and support with spells and healing.
  - **Follow distance or hold:** close, medium or far behind you, or hold a
    spot you set.
  - **Healing and items:** when to drink a draught or bind wounds (e.g. below
    half health), and whether they may use scrolls and bottles.
- **Pause orders, for mates only.** Paused (Alt), switch to a mate and line up
  a few actions for them (move here, attack that, use this), up to 3
  *(default)*, shown as numbered marks. Unpaused, they carry them out in
  order, then go back to their behaviours. Your own character takes no queued
  orders; it acts only on live input.
  *(default: switching back to your own character before unpausing keeps you
  in control of them; unpausing while on a mate hands control to that mate,
  and the orders queued for them are dropped.)*

## Testing

- **Dungeons first.** The overworld keeps today's controls until this feels
  right there.
- **You plus one mate** go into the test dungeon.
- **Foes: the game's own**, spawned by a test option into the dungeon away
  from the arrival (bandits or creatures). Nothing is asked of the
  generator's agent, which hasn't been approved to place enemies.

## Build order *(proposed)*

1. Third-person camera and crosshair, pointer lock, WASD, jump, Q dodge,
   C sneak, E to use, Alt pause, Tab switching; you plus one mate in a
   dungeon, the mate simply following.
2. Mates' behaviours (the settings panel) and following or holding.
3. Foes spawned in dungeons; real-time fighting for the controlled
   character (true-aim melee arcs, ranged and spell rays, block); foes and
   mates on the existing fight AI and its rules.
4. Pause orders for mates.
5. Mates' fighting under their behaviours (stance, role, healing).

## Added 2 Nov, later (Laz)

- **Sprint** on Shift.
- **The pointer stays in the window while playing.** It's confined to the
  window everywhere in play, and captured and hidden in third person. The
  title screen and the game menu free it; screenshot runs never take it.
- **The view changed, so the drawing changed.** Walls now stand to the
  ceiling, and a ceiling covers every walkable cell. The whole site is drawn,
  not only what's in sight. The camera stops short of walls and ceilings.
  The sim still works out what the squad sees: 40 m, not past walls (for
  foes, later).
- **Step 2 done:** the behaviours panel (B). Stance, role and range, follow
  close, medium or far or hold here, heal below a share, bind wounds, use
  scrolls and bottles. Following and holding act now; the rest wait for the
  fighting steps.

## Step 3 done (2 Nov, evening)

- **Foes:** "Test a dungeon" puts three of the game's own bandits out of
  sight of the arrival (`World::spawn_site_foes`, their camp taken away).
  A foe who sees one of the squad within 25 m (no wall or closed door
  between) starts a fight with everyone of the squad in the site.
- **Fights keep to the rooms:** a site's walls, closed doorways, pits, water
  and ledges are in the solid lists (`Site::fight_blocks`, `solid::Id::Site`),
  so the fight rules' movement goes round them. A fight is fought on one
  level: no ladders or jumps mid-fight.
- **You drive your fighter** (`combat::Drive`). The AI leaves them alone;
  the keys move them at their fighting pace (half while blocking).
  - **Left click strikes the way you face.** Whoever of the other side is in
    the weapon's arc when it lands is hit: the nearest, or everyone there
    with a sweeping weapon. The arc is narrow for a thrust, wide for a sweep.
    The blow is the same blow as ever: hit chance, armour, wounds, bleeding,
    stagger.
  - **With a bow, left click shoots along the line** and hits the first body
    on it, friend or foe. Arrows stop at walls.
  - **Hold the right button to block.** A blow from in front is turned at
    least 55% of the time, more with Block skill, a shield or a parrying
    weapon, at most 90%. Each block costs breath. Arrows are turned only
    with a shield.
  - **Q dodges back** 1.6 m, and you're harder to hit for 0.4 s.
- **Mates and foes:** the fight rules as before. Mates' stances and roles
  come in step 5.
- **Not yet:** spells by click (they need a way to choose the spell),
  jumping and ladders mid-fight, sight blocked by height, and foes who
  search or move about before they see you.
