# Gahturiyu Sim — placeholder sounds

Placeholder sound effects for the game. Every one is **synthesised by us** in
`tools/sounds/make_sounds.py` from noise, sines, simple filters and envelopes — no recordings,
no samples, no third-party material, so there is no licence to track.

- Game files: `*.ogg` (Ogg Vorbis, 44.1 kHz mono), loaded by `src/view/sound.rs`.
  Previews: `tools/sounds/preview/wav/*.wav` (16-bit PCM), written when you regenerate.
- `manifest.json`: name, file, seconds, peak and RMS level (dBFS), group, description, recipe.
- `tools/sounds/preview/listen.html`: open in a browser to play everything (uses the wav copies).
- Levels: interface peaks at −18 dBFS (hover −24, release −20), screens −14 (talk −17),
  world −10, the fire loop −20. Mix from there in the game.
- `_1/_2/_3` files are takes of the same sound; pick one at random each time.
- `coins_1..3` are the trade screen's opening sound (and any coins-on-a-surface moment).

## Regenerate

```
pip install numpy scipy soundfile   # libsndfile 1.0.29+ for Vorbis
python3 tools/sounds/make_sounds.py   # from the repo root
```

It is seeded (`SEED` at the top, plus each file's name), so it writes the same
audio every time. Change a recipe function or the seed and run it again.

## Interface

| Sound | What it is | How it's made |
|---|---|---|
| `ui_hover` | Softest tick as the mouse passes over a button. (35 ms) | 3 ms band-passed noise click plus a 2.3 kHz ping, very quiet. |
| `ui_press` ×3 | Button pressed: a soft woody/brass click. (75 ms) | Noise click plus damped 680/1550 Hz wood modes and faint inharmonic brass partials, detuned per take. |
| `ui_release` | Button released: a lighter, higher click. (55 ms) | The press recipe pitched up ~22%, shorter and quieter. |
| `ui_open` | Panel opening: a short soft paper/cloth swish. (200 ms) | Noise through a band-pass sweeping 600 to 2600 Hz under a smooth hump. |
| `ui_close` | Panel closing: a shorter downward swish. (150 ms) | Noise through a band-pass sweeping 2200 down to 650 Hz, front-weighted. |
| `ui_cant` | Action not possible: two quick dull low thuds. (180 ms) | Two pitch-dropping low sines (170 to 120 Hz) with low-passed noise puffs, 85 ms apart, low-passed at 1.2 kHz. |
| `ui_select` | Squad member selected: a small bright brass tink. (150 ms) | Bell-like inharmonic partials on 1.9 kHz (x1, 2.32, 4.07) with a soft attack. |
| `ui_order` ×3 | Move order given: a soft low thup. (120 ms) | Sine dropping 190 to 95 Hz plus a low-passed noise puff, detuned per take. |

## Screens

| Sound | What it is | How it's made |
|---|---|---|
| `talk_open` | Conversation opens: a very soft breath/rustle. (300 ms) | Broad noise band sweeping 700 to 1300 Hz plus sparse rustle grains. |
| `coins` ×3 | Trade screen opens (trade_open): coins clinking onto wood. (400 ms) | 5-7 coin strikes (4 inharmonic modes each, random pitch) settling faster and quieter, over wooden taps. |
| `coin_single` | One coin set down. (150 ms) | One coin strike on a wood tap with a small bounce. |
| `pack_open` | Pack opens: leather creak then a buckle clink. (300 ms) | Friction grains through 380/860/1500 Hz resonators, then a small metal clink. |
| `craft_open` | Crafting opens: tools set on a bench. (400 ms) | Four metal strikes at different pitches, each with a wooden thump. |
| `lid_open` | Chest lid creaking open. (600 ms) | Stick-slip pulse train (35 to 90 per second) ringing wooden resonators at 420/950/1700 Hz. |
| `lid_close` | Wooden lid thump shut. (250 ms) | Low sine thud (120 to 75 Hz), wood modes and a small rattle. |
| `take_item` ×3 | Picking something up: a soft cloth/leather shuffle. (150 ms) | Short swept noise band plus rustle grains, timing varied per take. |
| `page_turn` | A page turning (journal, spell book). (350 ms) | Rising noise band with crinkle grains and a soft low flap at the end. |

## World

| Sound | What it is | How it's made |
|---|---|---|
| `step_dirt` ×3 | Footstep on earth. (120 ms) | Heel and toe: low-passed noise thuds with gritty grains on top. |
| `step_stone` ×3 | Footstep on stone. (100 ms) | Heel and toe: sharp band-passed clicks with a small low body and scuff. |
| `swing` ×3 | A weapon swishing through the air. (250 ms) | Noise through a band-pass that rises to ~1.5 kHz and falls, under a bell envelope. |
| `hit_flesh` ×3 | A blow landing: a muffled thud. (150 ms) | Sine dropping 95 to 55 Hz, low-passed noise mass and a faint slap. |
| `hit_block` ×3 | A blow blocked: a wood/metal knock. (150 ms) | Wood modes (450/1120 Hz) plus quieter inharmonic metal partials and a click. |
| `bow_release` | A bowstring twang. (300 ms) | Karplus-Strong plucked string at 98 Hz, heavily damped, plus a falling swish and a wood knock. |
| `cast` | A spell starting: a soft rising shimmer. (600 ms) | Detuned sine partials gliding 420 to 640 Hz with vibrato, light FM and airy rising noise, swelling in. |
| `door_open` | A wooden door: latch, then a creak. (700 ms) | Two metal latch clicks, then a slow stick-slip creak through 230/560/1150 Hz resonators. |
| `knock_down` | Someone falling: a heavy body thud. (400 ms) | Sine dropping 72 to 42 Hz with low noise mass, two smaller follow-up impacts and faint gear rattle. |
| `fire_loop` | Quiet crackling fire, seamless 4 s loop. (4000 ms) | Breathing brown-noise roar, faint hiss, random crackles and pops; tail cross-faded into the head. |
