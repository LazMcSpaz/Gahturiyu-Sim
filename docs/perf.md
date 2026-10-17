# Where the window's time goes

## Measuring

`GAHT_PROF=<file>` turns on the window's profiler (`src/view/prof.rs`):
named spans of code (`prof::span("scene.ground")`) add up their time, and
every 30 frames a report is appended to the file: the frame time (average and
worst) with what's going on (`[play]`, `[editor]`, `[stroke]`), then each
span's milliseconds a frame. Off, a span costs one flag check.
`prof::count(name)` counts things instead (shown as 1 ms each).

`GAHT_STROKE=<seconds>` (with `GAHT_PROF`) scripts a land-editor session:
the editor opens 3 s after the first frame (`[editor]`), the camera is
tilted, turned and zoomed in and out for 4 s (`[camera]`), a Raise stroke
is held for that long with the brush circling the middle of the screen
(`[stroke]`), and the window closes 2 s after it's let go (`[after]`). The same stroke every run, so a change can be measured before and
after. From Git Bash on the PC:

```
GAHT_PROF=prof.txt GAHT_STROKE=8 GAHT_START=town ./target/release/gahturiyu.exe
```

Without `GAHT_STROKE` it runs until closed (or `timeout 45 ...`). Foliage
loads for the first ~15 s, so read idle numbers after that.

## The first pass (17 Oct, Laz's "jittery" land editor)

Ryzen 7 3700X / RTX 2070 SUPER, town start, defaults. Frames are capped at
60 Hz by vsync, so 16.7 ms is the floor.

| | before | after |
|---|---|---|
| frame mid-stroke | 128–153 ms avg, worst 600–800 ms | ~22 ms avg, worst ~30 ms |
| frame idle in play | ~28 ms | ~17 ms (vsync) |
| whole-ground rebuild (panning across a cell; stroke end; editor open) | 250–460 ms | 24 ms in play, 44 ms in the editor |

What it was, and what changed:

- **Every dab redrew the whole land** (the ground mesh was keyed to the hand
  edits' version, which every dab bumps), and the rebuild itself was slow.
  - The fine patch round the camera is now 6×6 tiles, each its own mesh.
    Mid-stroke only the tiles under the brush are redrawn (`scene.tiles`,
    ~4 ms); the roads and the far ring are redrawn once when the stroke
    ends. The editor hands the area touched each frame in
    `Editor::fresh`.
  - `ground_cells` works out each corner once (it was four times, once per
    quad touching it), reads slopes from the same height grid (it was four
    extra terrain lookups a corner), and the colour asks the atlas once a
    corner (`Atlas::shares_at`) instead of five wobbled lookups.
- **Roads were draped over the true terrain every rebuild** (52 ms): each
  point asked `Grid::height`, four terrain lookups plus a sea test. They now
  use `Grid::memo`, which keeps each grid corner's height once asked
  (5 ms).
- **The band rings (500 m and 2.5 km round the squad, on by default) cost
  10 ms every frame**: ~1,000 pieces each asking the ground five times. A
  ring now asks twice a spoke, shared by the pieces either side, and the
  band rings use `Grid::memo` (0.9 ms).
- The foliage readout's count of every drawn plant runs only while the
  readout (L) is shown.

What's left in a play frame (~6 ms of our own code): people, groups and
animals ~1.5 ms each, foliage 0.2 ms once loaded, the UI ~0.4 ms. The
remaining whole-ground rebuild in the editor is mostly the fine patch
(22,500 cells, ~32 ms); spreading its tiles over a few frames would hide it
if panning still hitches.

## Second pass: moving the camera in the editor (17 Oct)

Laz edits while tilting and zooming to see the heights. Every zoom step
changed the ground's key (the exact draw radius was in it), so the whole
ground was redrawn: a 44 ms stall about every half second while the camera
moved (250–460 ms each before the first pass).

- The fine patch is one tile per coarse cell, fixed to the world
  (`Scene3d::fine`): a zoom or tilt keeps every tile; a pan draws only the
  cells it brings into reach; a stroke redraws the tiles it touched. The
  far ring is kept unless its cell, reach or the land changed. Roads are
  their own mesh, redrawn on any change (~4.5 ms).
- The ground is drawn for the draw radius rounded up to steps of 1.2×
  (`RADIUS_STEP`), so zooming far out redraws it now and then, not every
  frame.
- When a stroke ends, only towns it reached are redrawn.
- The weather's land grid (`view/weather/land.rs`) was rebuilt for the
  whole world after every stroke: 79 ms. While the editor is open it now
  waits until the editor closes.

| editor, scripted | before this pass | after |
|---|---|---|
| camera moving: worst frame | 60–67 ms (a ground redraw ~2× a second) | ~24 ms |
| letting go of a stroke | ~120 ms | ~41 ms |
| mid-stroke frame | ~22 ms | ~23 ms |

