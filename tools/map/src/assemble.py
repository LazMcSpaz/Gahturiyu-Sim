"""Assemble tools/map/index.html (the one-map page) from src/world.tpl.html and the blocks
reused from the earlier two pages: the mark kinds and glyphs, the people screen and the RON
export (from the old map page), the catalogue and the drawing of things (from the builder).

    python tools/map/src/assemble.py            # writes tools/map/index.html
    python tools/map/src/assemble.py --site DIR # also stages a site like the Pages workflow does
"""
import os, re, sys, shutil, json

HERE = os.path.dirname(os.path.abspath(__file__))
MAP = os.path.dirname(HERE)
ROOT = os.path.dirname(os.path.dirname(MAP))

def block(path, start, end):
    s = open(path, encoding="utf-8").read()
    i = s.index(start); j = s.index(end, i)
    return s[i:j]

old_index = os.path.join(MAP, "src", "map-page.html")
builder = os.path.join(MAP, "build.html")
A = block(old_index, "  // Standard icons: what a place is, not who it is.", "  const $ = (id) => document.getElementById(id);")
B = block(old_index, "  // ---- people: the roster of a place", "  // ---- a first look ----")
C = block(builder, "  // ---- the catalogue:", "  // ---- the place ----")
D = block(builder, "  const STONE = ", "  // ---- the tray:")
H = block(old_index, '<div id="people" hidden>', '<div id="dock">')

# The people block's own copies of what the core now provides.
B = B.replace("  let GAME_KEYS = {};\n", "")
B = re.sub(r'  fetch\("grounds/index\.json".*?\n', "", B, count=1)
B = re.sub(r'  // The place builder \(build\.html\).*?\n  \$\("f-build"\)\.addEventListener.*?\n', "", B, count=1, flags=re.S)
assert '$("f-build")' not in B, "the old build button handler is still there"

tpl = open(os.path.join(HERE, "world.tpl.html"), encoding="utf-8").read()
G = open(os.path.join(HERE, "towngrow.js"), encoding="utf-8").read()
out = tpl.replace("<!--@H@-->", H).replace("/*@G@*/", G).replace("/*@A@*/", A).replace("/*@B@*/", B).replace("/*@C@*/", C).replace("/*@D@*/", D)
assert "/*@" not in out and "<!--@" not in out
open(os.path.join(MAP, "index.html"), "w", encoding="utf-8", newline="\n").write(out)
print("tools/map/index.html:", out.count("\n"), "lines")

if "--site" in sys.argv:
    site = sys.argv[sys.argv.index("--site") + 1]
    os.makedirs(os.path.join(site, "data", "places"), exist_ok=True)
    for f in os.listdir(MAP):
        p = os.path.join(MAP, f)
        if os.path.isfile(p):
            shutil.copy(p, site)
        elif f == "grounds":
            shutil.copytree(p, os.path.join(site, "grounds"), dirs_exist_ok=True)
    head = '<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover"><link rel="manifest" href="manifest.json"><meta name="theme-color" content="#231c14"></head><body>'
    open(os.path.join(site, "index.html"), "w", encoding="utf-8", newline="\n").write(head + out + "</body></html>")
    shutil.copy(os.path.join(ROOT, "docs/map/laz-map-2026-10-13.png"), os.path.join(site, "map.png"))
    shutil.copy(os.path.join(ROOT, "docs/map/laz-map-draft-2026-10-13.png"), os.path.join(site, "draft.png"))
    shutil.copy(os.path.join(ROOT, "docs/map/laz-roads-2026-10-13.png"), os.path.join(site, "roads.png"))
    shutil.copy(os.path.join(ROOT, "assets/world/catalogue.json"), os.path.join(site, "catalogue.json"))
    shutil.copy(os.path.join(ROOT, "assets/world/marks-export.json"), os.path.join(site, "data", "marks-export.json"))
    shutil.copy(os.path.join(ROOT, "assets/world/people-export.json"), os.path.join(site, "data", "people-export.json"))
    pdir = os.path.join(ROOT, "assets/world/places")
    names = []
    for f in sorted(os.listdir(pdir)):
        if f.endswith(".json"):
            shutil.copy(os.path.join(pdir, f), os.path.join(site, "data", "places", f)); names.append(f)
    open(os.path.join(site, "data", "places.txt"), "w", encoding="utf-8", newline="\n").write("\n".join(names) + "\n")
    open(os.path.join(site, "version.txt"), "w").write("local\n")
    print("site staged at", site)
