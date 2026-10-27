#!/usr/bin/env python3
"""Merge agent outputs into one world file.
Usage: python3 merge.py <out.json> <part1.json> [part2.json ...]"""
import json, sys, collections, os
out_path, parts = sys.argv[1], sys.argv[2:]
mp = json.load(open(os.environ.get("QMAP", "map.json"))); canon = json.load(open(os.environ.get("QCANON", "canon.json")))
W = {"map": mp, "registry": {"people": [], "places": [], "things": [], "factions": []}, "facts": [], "cards": [],
     "story": None, "faction_stories": [], "parts": parts, "merge_notes": []}
seen = collections.defaultdict(dict)
def add(kind, e, src):
    i = e["id"]
    if i in seen[kind]:
        if seen[kind][i][0] != e and kind not in ("factions",) and not e.get("premade"):
            W["merge_notes"].append(f"id clash {kind} {i} between {seen[kind][i][1]} and {src}")
        return
    seen[kind][i] = (e, src); W["registry"][kind].append(e)
for f in canon["factions"]: add("factions", f, "canon")
for p in canon["people"]: add("people", p, "canon")
for p in mp["places"]:
    q = dict(p); q.setdefault("owner", None); q.setdefault("access", "open"); q.setdefault("tags", []); q.setdefault("facts", [])
    add("places", q, "map")
fact_seen = {}
for path in parts:
    d = json.load(open(path)); src = d.get("agent", path)
    reg = d.get("registry", {})
    for kind in ("people", "places", "things", "factions"):
        for e in reg.get(kind, []) or []:
            if kind == "places" and e["id"] in seen["places"] and seen["places"][e["id"]][1] == "map":
                # agent copied a map place: keep map version, merge facts
                mpl = seen["places"][e["id"]][0]
                mpl.setdefault("facts", []).extend(e.get("facts", []) or [])
                for k in ("holder", "titles", "context", "descriptors"):
                    if e.get(k): mpl[k] = e[k] if not isinstance(e[k], list) else (mpl.get(k) or []) + [x for x in e[k] if x not in (mpl.get(k) or [])]
                continue
            if kind == "people" and e["id"] in seen["people"] and seen["people"][e["id"]][1] == "canon":
                old = seen["people"][e["id"]][0]; old.clear(); old.update(e); seen["people"][e["id"]] = (old, src)
                continue
            add(kind, e, src)
    for u in d.get("registry_updates", []) or []:
        tgt = None
        for kind in ("people", "places", "things", "factions"):
            if u.get("entry") in seen[kind]: tgt = seen[kind][u["entry"]][0]
        if tgt is None: W["merge_notes"].append(f"update for unknown entry {u.get('entry')} from {src}"); continue
        for k, v in (u.get("set") or {}).items():
            if tgt.get(k) not in (None, v): W["merge_notes"].append(f"update would change {u['entry']}.{k} ({tgt.get(k)} -> {v}) from {src}")
            else: tgt[k] = v
        tgt.setdefault("links", []).extend(u.get("add_links") or [])
        tgt.setdefault("facts", []).extend(u.get("add_facts") or [])
        tgt.setdefault("traits", []).extend(u.get("add_traits") or [])
    for f in d.get("facts", []) or []:
        if f["id"] in fact_seen:
            if fact_seen[f["id"]] != src: W["merge_notes"].append(f"fact id clash {f['id']}")
            continue
        fact_seen[f["id"]] = src; W["facts"].append(f)
    for c in d.get("cards", []) or []:
        c.setdefault("_agent", src); W["cards"].append(c)
    if d.get("story"):
        if W["story"]: W["merge_notes"].append(f"second main story in {src}")
        else: W["story"] = d["story"]
    W["faction_stories"] += d.get("faction_stories", []) or []
ids = collections.Counter(c["id"] for c in W["cards"])
for i, k in ids.items():
    if k > 1: W["merge_notes"].append(f"card id {i} appears {k} times")
names = collections.Counter(p["name"] for p in W["registry"]["people"])
for n, k in names.items():
    if k > 1: W["merge_notes"].append(f"person name '{n}' used {k} times across parts")
json.dump(W, open(out_path, "w"), indent=1, ensure_ascii=False)
print(f"{out_path}: {len(W['cards'])} cards, {len(W['registry']['people'])} people, {len(W['registry']['places'])} places, "
      f"{len(W['faction_stories'])} faction stories, main story: {'yes' if W['story'] else 'no'}; {len(W['merge_notes'])} merge notes")
for n in W["merge_notes"][:40]: print("  -", n)
