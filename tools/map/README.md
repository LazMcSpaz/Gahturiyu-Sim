# The atlas's layers

`layers.py` turns Laz's clean map export into the layers the atlas reads
(`sim/atlas.rs`), in `assets/world/`:

```
python tools/map/layers.py [map.png] [roads.png] [out_dir]
```

The defaults are `docs/map/laz-map-2026-10-13.png`, `docs/map/laz-roads-2026-10-13.png`
and `assets/world/`. It needs Pillow and numpy only, takes about 20 seconds, and
gives the same bytes from the same map. When Laz sends a new export, run it again.

Every layer is the map's own size (2576 × 1932); one pixel is 150 m.

## terrain.png (RGB)

Every pixel is exactly one of these colours: the legend's swatches as drawn,
and one colour for the sea.

| Class | RGB |
|---|---|
| Sea | 74, 111, 165 |
| Rolling plains | 164, 186, 112 |
| Forest | 78, 116, 72 |
| Coast | 220, 206, 158 |
| Bog | 122, 128, 92 |
| Rocky ground | 150, 138, 124 |
| Snowy mountains | 242, 244, 248 |
| The Overgrowth | 64, 82, 70 |

How it's read:

- **The sea** is everything reached from the picture's edge without crossing
  the coast's dark outline (40, 44, 52). The outline is split down its middle
  between land and sea. Lakes and rivers are inside the outline, so they are
  land here. The legend's box is sea.
- **A land pixel's class** is the swatch that comes closest once darkened or
  lightened by the hill-shading (a factor of 0.4 to 1.7 on all three channels).
- **Strokes and icons take the class of the nearest ground round them.** These
  are rivers, lakes, outlines, cliff ticks, tree and tuft icons, and the
  Overgrowth's purple tinge. A most-common-class pass over 5 × 5 pixels then
  smooths what the icons left.
- **Under a river or a lake** the class is the ground round it. `water.png`
  says where the water is.

## water.png (RGBA)

Rivers and lakes: white (255, 255, 255, 255) where there is water, transparent
elsewhere.

- **Rivers** are the blue strokes as drawn. That is 1 px for a stream and up to
  6 to 8 px for the big rivers, so up to about a kilometre wide at 150 m a pixel.
- **Lakes** are filled. Gaps left by icons drawn over them are closed, and their
  dark rims count as water.
- Rivers and lakes are not told apart. A lake is the wide blob, a river the stroke.

## cliffs.png (RGBA)

The cliff ticks (70, 52, 40) as drawn, white on transparent. Each tick is a
short stroke across the coast, where a cliff stands. The legend's note
"contours 250 m" refers to the thin lines over the hills and mountains, which
aren't lifted.

## roads.png (RGBA)

The roads lifted off the draft (`docs/map/laz-roads-2026-10-13.png`), thinned
to 1 px centrelines (Zhang-Suen), white on transparent, with lone specks
removed. Branches meet at single pixels, so they stay joined.
