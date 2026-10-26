#!/usr/bin/env python3
"""Build the world map artifact from merged worlds plus findings.
Usage: python3 build_viz.py out.html <world.json> <title> <tab> <blurb> <findings.md|-> <check.md|-> [<world2.json> ...repeat]"""
import json, sys, html, re
out = sys.argv[1]; args = sys.argv[2:]
def md(path):
    if path == "-": return ""
    t = open(path).read()
    o = []
    for line in t.splitlines():
        e = html.escape(line)
        e = re.sub(r"\*\*(.+?)\*\*", r"<b>\1</b>", e); e = re.sub(r"\*(.+?)\*", r"<i>\1</i>", e); e = re.sub(r"`(.+?)`", r"<code>\1</code>", e)
        if line.startswith("### "): o.append(f"<h3>{e[4:]}</h3>")
        elif line.startswith("## "): o.append(f"<h3>{e[3:]}</h3>")
        elif line.startswith("# "): o.append(f"<h2>{e[2:]}</h2>")
        elif line.startswith("- "): o.append(f"<ul><li>{e[2:]}</li></ul>")
        elif line.strip().startswith("|"): o.append(f"<div class='mono'>{e}</div>")
        elif line.strip(): o.append(f"<p>{e}</p>")
    return re.sub(r"</ul>\s*<ul>", "", "\n".join(o))
worlds = []
for i in range(0, len(args), 6):
    w, title, tab, blurb, find, chk = args[i:i + 6]
    d = json.load(open(w))
    keep = {"map": d["map"], "registry": d["registry"], "facts": d["facts"], "cards": d["cards"], "story": d.get("story"), "faction_stories": d.get("faction_stories", [])}
    worlds.append({"title": title, "tab": tab, "blurb": blurb, "data": keep, "findings": md(find), "check": "" if chk == "-" else open(chk).read()})
t = open(__import__("os").path.join(__import__("os").path.dirname(__file__), "world-viz-template.html")).read().replace("/*DATA*/null", json.dumps(worlds, ensure_ascii=False))
open(out, "w").write(t); print(out, len(t))
