"""The map site's close-up pictures: the game's own land, photographed from above.

    python tools/map/render.py                  # render everything, then pack it into tools/map/render/
    python tools/map/render.py --pack DIR       # only pack an earlier render's PNGs from DIR

Runs the game window in its map-render mode (`src/view/maprender.rs`,
`GAHT_MAPRENDER`): an orthographic camera straight down, no towns, people or
panels, the whole map at 4 m a pixel and every mark at 1 m (its place left
without plants: the site draws what's built there). It needs the graphics card,
so it runs on a PC with a screen: a game window is open while it works (about
half an hour; it closes itself). Then each PNG is turned into a WebP (about a
twentieth of the size) in `tools/map/render/`, with `index.json` (each
picture's name, level, metres a pixel and the world box it shows) for the
page (`src/world.tpl.html`, `drawRender`).

Run it again whenever the land changes (the map, its reading in `atlas.rs`,
the ground colours in `palette.rs`, the plants in `sim/plants.rs`).
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile

from PIL import Image

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
OUT = os.path.join(ROOT, "tools", "map", "render")
QUALITY = 80


def render(into):
    env = dict(os.environ, GAHT_MAPRENDER=into, GAHT_WEATHER_QUALITY="off")
    target = os.environ.get("CARGO_TARGET_DIR", os.path.join(ROOT, "target"))
    exe = os.path.join(target, "release", "gahturiyu.exe" if os.name == "nt" else "gahturiyu")
    subprocess.run(["cargo", "build", "--release", "--bin", "gahturiyu"], cwd=ROOT, check=True)
    subprocess.run([exe], cwd=ROOT, env=env, check=True)


def pack(src):
    index = json.load(open(os.path.join(src, "index.json"), encoding="utf-8"))
    if os.path.isdir(OUT):
        shutil.rmtree(OUT)
    os.makedirs(OUT)
    total = 0
    for t in index["tiles"]:
        im = Image.open(os.path.join(src, t["name"] + ".png")).convert("RGB")
        path = os.path.join(OUT, t["name"] + ".webp")
        im.save(path, "WEBP", quality=QUALITY, method=6)
        t["ext"] = "webp"
        total += os.path.getsize(path)
    with open(os.path.join(OUT, "index.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump(index, f, indent=1)
    print("{} pictures, {:.1f} MB, in {}".format(len(index["tiles"]), total / 1e6, os.path.relpath(OUT, ROOT)))


def main():
    if "--pack" in sys.argv:
        pack(sys.argv[sys.argv.index("--pack") + 1])
        return
    with tempfile.TemporaryDirectory() as tmp:
        render(tmp)
        pack(tmp)


if __name__ == "__main__":
    main()
