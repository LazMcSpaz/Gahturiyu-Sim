"""Laz's map marks into the game's town list.

    python tools/map/marks_to_towns.py          # write assets/world/towns.ron and landmarks.ron
    python tools/map/marks_to_towns.py --check  # say whether they're current (exit 1 if not)

Reads `assets/world/marks-export.json` (Laz's marks as pulled from the Map
Marks page: his data, never edited by hand) and writes:

- `assets/world/towns.ron`: every settlement mark (capital, city, town,
  village, hamlet, fort, outpost and their kinds), which the game reads to
  place its towns;
- `assets/world/landmarks.ron`: every other mark (mines, ports, ruins,
  lookouts...), which nothing reads yet.

Keys are a slug of the label ("Last Wave outpost" -> `last_wave_outpost`).
A key, once given, stays with its mark: the mark's id is written beside it
and a later run keeps it even if the label changes, so a roster or a place
file never moves to another town. When two marks would have the same key,
the first made keeps it and the rest get `_<mark id>`; an unlabelled mark is
`mark_<id>`.
"""

import json
import os
import re
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
EXPORT = os.path.join(ROOT, "assets", "world", "marks-export.json")
TOWNS = os.path.join(ROOT, "assets", "world", "towns.ron")
LANDMARKS = os.path.join(ROOT, "assets", "world", "landmarks.ron")

# A mark's kind on the page -> the town's kind in the game (`atlas::Kind`).
SETTLEMENTS = {
    "capital": "Capital",
    "city": "City",
    "town": "Town",
    "village": "Village",
    "hamlet": "Hamlet",
    "fort": "Fort",
    "imperial_fort": "Fort",
    "outpost": "Outpost",
    "warden_outpost": "Outpost",
    "qotiro_outpost": "Outpost",
    "highfolk_hold": "Town",
    "stilt_village": "Village",
}

PEOPLES = {"roduro": "Roduro", "qotiro": "Qotiro", "horaro": "Horaro", "tadoro": "Tadoro", "ṭaḍoro": "Tadoro"}


def slug(label):
    words = re.split(r"[^0-9A-Za-z]+", label)
    return "_".join(w.lower() for w in words if w)


def clean(label):
    # The page's export turned some apostrophes into U+FFFD.
    return label.replace("�", "’").strip()


def founders_of(m):
    """Who founded it, and where that came from: the page's Built by, else
    Laz's notes ("Horaro founders"), else a guess from the kind."""
    f = (m.get("founders") or "").strip().lower()
    if f in PEOPLES:
        return PEOPLES[f], "built by, as set on the page"
    hit = re.search(r"\b(roduro|qotiro|horaro|tadoro|ṭaḍoro)\s+founders\b", (m.get("notes") or "").lower())
    if hit:
        return PEOPLES[hit.group(1)], "founders from the notes"
    kind = m.get("kind", "")
    if kind == "qotiro_outpost":
        return "Qotiro", "founders GUESSED from the kind (Built by is blank)"
    if kind == "stilt_village":
        return "Horaro", "founders GUESSED from the kind (Built by is blank)"
    return "Roduro", "founders GUESSED (Built by is blank)"


def previous(path):
    """What an earlier run wrote: (id, key, px, label) per line."""
    out = []
    if not os.path.exists(path):
        return out
    for line in open(path, encoding="utf-8"):
        k = re.search(r'\bkey:\s*"([^"]*)"', line)
        if not k:
            continue
        i = re.search(r'\bid:\s*"([^"]*)"', line)
        px = re.search(r"\bpx:\s*\((\d+),\s*(\d+)\)", line)
        lab = re.search(r'\blabel:\s*"([^"]*)"', line)
        out.append((i.group(1) if i else None, k.group(1), (int(px.group(1)), int(px.group(2))) if px else None, lab.group(1) if lab else ""))
    return out


def assign_keys(marks, prev):
    """A key for every mark: kept from before (by id, then by spot, then by
    label), else made from the label."""
    keys = {}
    taken = set()
    old = [p for p in prev]
    # By id: a key once given stays.
    for m in marks:
        for p in old:
            if p[0] == m["id"] and p[1] not in taken:
                keys[m["id"]] = p[1]
                taken.add(p[1])
                break
    # The first run over the old placeholder list (no ids yet): the same spot, then the same label.
    for match in ("px", "label"):
        for m in marks:
            if m["id"] in keys:
                continue
            for p in old:
                if p[0] is not None or p[1] in taken:
                    continue
                same = tuple(m["px"]) == p[2] if match == "px" else (slug(clean(m.get("label", ""))) == p[1] and p[1] != "")
                if same:
                    keys[m["id"]] = p[1]
                    taken.add(p[1])
                    break
    # New marks: the label's slug, the first made keeping it plain.
    for m in marks:
        if m["id"] in keys:
            continue
        base = slug(clean(m.get("label", ""))) or "mark_" + m["id"]
        key = base if base not in taken else base + "_" + m["id"]
        keys[m["id"]] = key
        taken.add(key)
    return keys


def ron_str(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def build():
    data = json.load(open(EXPORT, encoding="utf-8"))
    marks = sorted(data["marks"], key=lambda m: (m.get("created", 0), m["id"]))
    keys = assign_keys(marks, previous(TOWNS) + previous(LANDMARKS))
    towns = [m for m in marks if m.get("kind") in SETTLEMENTS]
    others = [m for m in marks if m.get("kind") not in SETTLEMENTS]

    t = []
    t.append("// The towns: Laz's settlement marks from the Map Marks page")
    t.append("// (`assets/world/marks-export.json`), written by `tools/map/marks_to_towns.py`.")
    t.append("// Don't edit by hand: change the mark on the page, pull the export again and")
    t.append("// rerun the script (`tests/marks.rs` fails if this file is stale).")
    t.append("//")
    t.append("// key: the town's name in data (its roster is `assets/world/people/<key>.ron`,")
    t.append("// its place `assets/world/places/<key>.json`); it stays with the mark's `id`")
    t.append("// even if the label changes. label: what the map says (the town's own name is")
    t.append("// made by the naming module). px: the pixel, 150 m a pixel. kind: Capital,")
    t.append("// City, Town, Village, Hamlet, Outpost, Fort. holder: a faction key.")
    t.append("// The comment on each line says where its founders came from.")
    t.append("[")
    for m in towns:
        f, why = founders_of(m)
        t.append(
            "    (key: {}, id: {}, label: {}, px: ({}, {}), kind: {}, founders: {}, holder: {}),  // {}".format(
                ron_str(keys[m["id"]]), ron_str(m["id"]), ron_str(clean(m.get("label", ""))), m["px"][0], m["px"][1], SETTLEMENTS[m["kind"]], f, ron_str(m.get("holder", "") or ""), why
            )
        )
    t.append("]")

    l = []
    l.append("// Landmarks: Laz's marks that aren't settlements (mines, ports, ferries,")
    l.append("// lookouts, ruins, temples...), from `assets/world/marks-export.json`, written")
    l.append("// by `tools/map/marks_to_towns.py`. NOTHING IN THE GAME READS THIS YET: it keeps")
    l.append("// them, with stable keys, for when mines, ports and ruins are placed from")
    l.append("// Laz's map. Don't edit by hand.")
    l.append("[")
    for m in others:
        l.append(
            "    (key: {}, id: {}, label: {}, px: ({}, {}), kind: {}, holder: {}, notes: {}),".format(
                ron_str(keys[m["id"]]), ron_str(m["id"]), ron_str(clean(m.get("label", ""))), m["px"][0], m["px"][1], ron_str(m.get("kind", "")), ron_str(m.get("holder", "") or ""), ron_str(clean(m.get("notes", "") or ""))
            )
        )
    l.append("]")
    return "\n".join(t) + "\n", "\n".join(l) + "\n"


def main():
    towns, landmarks = build()
    if "--check" in sys.argv:
        stale = []
        for path, text in ((TOWNS, towns), (LANDMARKS, landmarks)):
            have = open(path, encoding="utf-8").read().replace("\r\n", "\n") if os.path.exists(path) else ""
            if have != text:
                stale.append(os.path.relpath(path, ROOT))
        if stale:
            print("stale: " + ", ".join(stale) + " (run python tools/map/marks_to_towns.py)")
            sys.exit(1)
        print("current")
        return
    for path, text in ((TOWNS, towns), (LANDMARKS, landmarks)):
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
    print("{} towns, {} landmarks".format(towns.count("(key:"), landmarks.count("(key:")))


if __name__ == "__main__":
    main()
