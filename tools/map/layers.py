"""Turn Laz's clean map export into the atlas's layers (assets/world/).

    python tools/map/layers.py [map.png] [roads.png] [out_dir]

Defaults: docs/map/laz-map-2026-10-13.png, docs/map/laz-roads-2026-10-13.png,
assets/world/. Writes, each the map's own size (one pixel = 150 m):

- terrain.png  every pixel one flat colour: the sea, or the land's class
               (the legend's colours exactly, the hill-shading taken off).
- water.png    rivers and lakes, white on transparent.
- cliffs.png   the cliff ticks, white on transparent.
- roads.png    the lifted roads thinned to 1 px centrelines, white on
               transparent.

The colours are in tools/map/README.md. Needs Pillow and numpy only; the
same map gives the same layers, byte for byte.
"""

import os
import sys

import numpy as np
from PIL import Image, ImageDraw

# The legend's swatches, exactly as drawn (the classes, in this order).
CLASSES = [
    ("plains", (164, 186, 112)),
    ("forest", (78, 116, 72)),
    ("coast", (220, 206, 158)),
    ("bog", (122, 128, 92)),
    ("rock", (150, 138, 124)),
    ("snow", (242, 244, 248)),
    ("overgrowth", (64, 82, 70)),
]
# One colour for the sea in terrain.png.
SEA = (74, 111, 165)
# Strokes drawn over the land: the coast's outline (and the lakes' rims),
# the cliff ticks, the legend's paper.
OUTLINE = (40, 44, 52)
CLIFF = (70, 52, 40)
PAPER = (246, 240, 226)

# How far the hill-shading may darken or lighten a class's colour.
SHADE = (0.4, 1.7)
# A pixel this far from every class (after shading) is a stroke or an icon:
# it takes its class from the nearest pixel that isn't.
FAR = 34.0
# Darker than this (mean of r, g, b): tree and tuft icons, never ground.
ICON_DARK = 62.0
# Icons are cleaned up by the most common class within this many pixels.
MODE_R = 2
# Gaps in a lake up to about twice this across are filled.
LAKE_R = 3

UNKNOWN, BLOCKED = -1, -2


def near(img, rgb, tol=0):
    return (np.abs(img - np.array(rgb)) <= tol).all(axis=2)


def neighbours(a, fill):
    """The four neighbours of every pixel (up, down, left, right)."""
    up = np.full_like(a, fill)
    up[1:] = a[:-1]
    down = np.full_like(a, fill)
    down[:-1] = a[1:]
    left = np.full_like(a, fill)
    left[:, 1:] = a[:, :-1]
    right = np.full_like(a, fill)
    right[:, :-1] = a[:, 1:]
    return (up, down, left, right)


def fill_nearest(labels):
    """Every UNKNOWN pixel takes the label of the nearest labelled one,
    growing out a pixel at a time (BLOCKED pixels neither give nor take).
    Ties go to the first neighbour in (up, down, left, right) order."""
    labels = labels.copy()
    while True:
        unknown = labels == UNKNOWN
        if not unknown.any():
            return labels
        taken = np.full_like(labels, UNKNOWN)
        for n in neighbours(labels, UNKNOWN):
            pick = (taken == UNKNOWN) & (n >= 0)
            taken[pick] = n[pick]
        grow = unknown & (taken >= 0)
        if not grow.any():
            return labels
        labels[grow] = taken[grow]


def dilate(mask, steps):
    m = mask.copy()
    for _ in range(steps):
        grown = m.copy()
        for n in neighbours(m, False):
            grown |= n
        m = grown
    return m


def box_sum(a, r):
    """Sum over the (2r+1)-square round each pixel (edges count what's there)."""
    c = np.pad(a, ((r + 1, r), (r + 1, r))).cumsum(0).cumsum(1)
    k = 2 * r + 1
    return c[k:, k:] - c[:-k, k:] - c[k:, :-k] + c[:-k, :-k]


def mode_filter(labels, land, n, r):
    """The most common land class within r pixels of each land pixel."""
    counts = np.stack([box_sum(((labels == k) & land).astype(np.int32), r) for k in range(n)])
    best = counts.argmax(axis=0)
    return np.where(land, best, labels)


def thin(mask):
    """Zhang-Suen thinning: strokes down to 1 px centrelines."""
    m = np.pad(mask.astype(np.uint8), 1)
    while True:
        changed = False
        for step in (0, 1):
            p2 = m[:-2, 1:-1]
            p3 = m[:-2, 2:]
            p4 = m[1:-1, 2:]
            p5 = m[2:, 2:]
            p6 = m[2:, 1:-1]
            p7 = m[2:, :-2]
            p8 = m[1:-1, :-2]
            p9 = m[:-2, :-2]
            ring = [p2, p3, p4, p5, p6, p7, p8, p9, p2]
            b = sum(x.astype(np.int32) for x in ring[:8])
            a = sum(((ring[i] == 0) & (ring[i + 1] == 1)).astype(np.int32) for i in range(8))
            if step == 0:
                c = (p2 * p4 * p6) == 0
                d = (p4 * p6 * p8) == 0
            else:
                c = (p2 * p4 * p8) == 0
                d = (p2 * p6 * p8) == 0
            core = m[1:-1, 1:-1]
            gone = (core == 1) & (b >= 2) & (b <= 6) & (a == 1) & c & d
            if gone.any():
                core[gone] = 0
                changed = True
        if not changed:
            return m[1:-1, 1:-1].astype(bool)


def white_on_clear(mask):
    out = np.zeros(mask.shape + (4,), np.uint8)
    out[mask] = (255, 255, 255, 255)
    return Image.fromarray(out, "RGBA")


def layers(map_path, roads_path, out_dir):
    img = np.asarray(Image.open(map_path).convert("RGB")).astype(np.int32)
    h, w, _ = img.shape

    # The legend: its paper box in the bottom-left, taken as sea.
    paper = near(img, PAPER, 2)
    paper[: h * 2 // 3] = False
    paper[:, w // 3 :] = False
    legend = np.zeros((h, w), bool)
    if paper.any():
        ys, xs = np.nonzero(paper)
        legend[max(ys.min() - 8, 0) : ys.max() + 9, max(xs.min() - 8, 0) : xs.max() + 9] = True

    # The sea: everything reached from the edge without crossing the
    # coast's outline (rivers end at it, so lakes and rivers stay inland).
    outline = near(img, OUTLINE, 6) & ~legend
    walls = Image.fromarray(np.where(outline, 255, 0).astype(np.uint8), "L").copy()
    ImageDraw.floodfill(walls, (0, 0), 128, thresh=0)
    sea = np.asarray(walls) == 128
    # The outline itself is split down its middle between sea and land.
    split = np.where(sea, 0, np.where(outline, UNKNOWN, 1))
    sea = fill_nearest(split) == 0
    land = ~sea

    # Water on the land: the blue strokes and lakes, and the lakes' rims.
    r, g, b = img[..., 0], img[..., 1], img[..., 2]
    blue = land & (b - r > 40) & (b - g > 15)
    rims = land & outline & ~dilate(sea, 3)
    water = blue | (rims & dilate(blue, 2))
    # Icons drawn over a lake leave holes in it: inside a body of water (most
    # of the square round a pixel is water) a gap is water too.
    closed = ~dilate(~dilate(water, LAKE_R), LAKE_R)
    water |= land & closed & (box_sum(water.astype(np.int32), LAKE_R + 1) * 2 >= (2 * LAKE_R + 3) ** 2)

    # Each land pixel's class: the legend colour that, darkened or lightened
    # by the hill-shading, comes closest.
    cols = np.array([c for _, c in CLASSES], np.float64)
    px = img.astype(np.float64)
    best = np.full((h, w), np.inf)
    label = np.zeros((h, w), np.int32)
    for k, c in enumerate(cols):
        s = np.clip((px @ c) / (c @ c), *SHADE)
        d = np.sqrt(((px - s[..., None] * c) ** 2).sum(axis=2))
        better = d < best
        best[better] = d[better]
        label[better] = k
    cliff = land & near(img, CLIFF, 6)
    purple = (r > g + 10) & (b > g + 10)
    strokes = water | outline | cliff | purple | (img.mean(axis=2) < ICON_DARK) | (best > FAR)
    label = np.where(sea, BLOCKED, np.where(strokes, UNKNOWN, label))
    label = fill_nearest(label)
    label = np.where(sea, BLOCKED, label)
    label = mode_filter(np.maximum(label, 0), land, len(CLASSES), MODE_R)

    terrain = np.zeros((h, w, 3), np.uint8)
    terrain[sea] = SEA
    for k, (_, c) in enumerate(CLASSES):
        terrain[land & (label == k)] = c

    roads_img = np.asarray(Image.open(roads_path).convert("RGBA"))
    if roads_img.shape[:2] != (h, w):
        raise SystemExit(f"{roads_path} is {roads_img.shape[1]}x{roads_img.shape[0]}, the map {w}x{h}")
    roads = thin(roads_img[..., 3] > 0)
    # Specks left from lifting the roads off the draft: a lone pixel goes.
    r8 = np.pad(roads, 1).astype(np.int32)
    touching = sum(r8[1 + dy : 1 + dy + h, 1 + dx : 1 + dx + w] for dy in (-1, 0, 1) for dx in (-1, 0, 1) if (dy, dx) != (0, 0))
    roads &= touching > 0

    os.makedirs(out_dir, exist_ok=True)
    Image.fromarray(terrain, "RGB").save(os.path.join(out_dir, "terrain.png"), optimize=True)
    white_on_clear(water).save(os.path.join(out_dir, "water.png"), optimize=True)
    white_on_clear(cliff).save(os.path.join(out_dir, "cliffs.png"), optimize=True)
    white_on_clear(roads).save(os.path.join(out_dir, "roads.png"), optimize=True)

    n = land.sum()
    print(f"{w}x{h}: land {100 * n / (w * h):.1f}% of the picture")
    for k, (name, _) in enumerate(CLASSES):
        print(f"  {name:11s} {100 * (land & (label == k)).sum() / n:5.1f}% of the land")
    print(f"  water {water.sum()} px, cliffs {cliff.sum()} px, roads {roads.sum()} px (from {(roads_img[..., 3] > 0).sum()})")


if __name__ == "__main__":
    a = sys.argv[1:]
    layers(
        a[0] if len(a) > 0 else "docs/map/laz-map-2026-10-13.png",
        a[1] if len(a) > 1 else "docs/map/laz-roads-2026-10-13.png",
        a[2] if len(a) > 2 else "assets/world",
    )
