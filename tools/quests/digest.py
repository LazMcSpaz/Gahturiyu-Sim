#!/usr/bin/env python3
"""Render a merged world as readable text for reviewers. Usage: python3 digest.py world.json > digest.md"""
import json, sys, collections
W = json.load(open(sys.argv[1]))
reg = W["registry"]; E = {}
for k in ("people", "places", "things", "factions"):
    for e in reg.get(k, []): E[e["id"]] = e
F = {f["id"]: f for f in W["facts"]}; C = {c["id"]: c for c in W["cards"]}
MAPP = {p["id"]: p for p in W["map"]["places"]}
def nm(i): return E.get(i, {}).get("name", i) if isinstance(i, str) else str(i)
def home(pid):
    for _ in range(6):
        if pid in MAPP: return pid
        p = E.get(pid)
        if not p: return None
        pid = p.get("at") or p.get("in") or p.get("parent")
def where(c):
    pl = c.get("place"); h = home(pl)
    return f"{nm(pl)}" + (f" ({nm(h)})" if h and h != pl else "")
def ft(i): return F.get(i, {}).get("text", i) if i else "—"
def card(c, n=None):
    sl = "; ".join(f"{r}={nm(v) if isinstance(v, str) else ', '.join(nm(x) for x in v)}" + (f" [{E[v]['standing']}]" if isinstance(v, str) and E.get(v, {}).get('standing') else "") for r, v in (c.get("slots") or {}).items())
    ways = [w["text"] + f" ({w['kind']})" for i, w in enumerate(c.get("ways_pool") or []) if i in (c.get("ways_rolled") or [])]
    tw = c.get("twist")
    if isinstance(tw, dict): tw = (tw.get("name") or "none") + (f" — {tw.get('detail') or tw.get('twist_detail')}" if (tw.get('detail') or tw.get('twist_detail')) else "")
    tw = tw or "none"
    if c.get("twist_detail"): tw += f" — {c['twist_detail']}"
    L = [f"**{(str(n)+'. ') if n else ''}{c['family']}: {c['premise']}** `{c['id']}` · {c.get('relatedness')} · {c.get('scope')} · at {where(c)}"]
    L.append(f"  - who: {sl}")
    if c.get("pressure"): L.append(f"  - pressure: {c['pressure']}")
    if c.get("motive"): L.append(f"  - motive: {c['motive'] if isinstance(c['motive'], str) else json.dumps(c['motive'], ensure_ascii=False)}")
    if c.get("details"): L.append(f"  - details: {c['details'] if isinstance(c['details'], str) else json.dumps(c['details'], ensure_ascii=False)[:400]}")
    L.append(f"  - truth: {ft(c.get('truth'))} · twist: {tw}")
    L.append(f"  - ways rolled: {' | '.join(ways)}")
    if c.get("plants"): L.append("  - plants: " + " | ".join(f"[{p.get('type')}] {ft(p.get('fact'))}" for p in c["plants"]))
    if c.get("depends_on"): L.append("  - depends on: " + " | ".join(ft(x)[:110] for x in c["depends_on"]))
    if c.get("place_tie"): L.append(f"  - place tie: {c['place_tie']}")
    L.append(f"  - endings: {', '.join(c.get('possible_endings') or [])}" + (f" · faction endings: {', '.join(c['faction_endings'])}" if c.get('faction_endings') else ""))
    return "\n".join(L)
out = [f"# Digest: {sys.argv[1]}", ""]
st = W.get("story")
if st:
    cast = st.get("cast", {})
    out += ["## Main story", f"arc {st.get('arc')} · shape {st.get('shape')} · L{st.get('L')} W{st.get('W')}",
            "cast: " + "; ".join(f"{k}={nm(v)}" for k, v in cast.items() if isinstance(v, str)),
            f"revelation: {ft((st.get('revelation') or {}).get('fact'))}", ""]
    for m in st.get("moves", []): out.append(f"- move L{m.get('layer')} ({m.get('kind')}, {m.get('scope')}): {ft(m.get('fact'))}")
    out.append("")
    for ch in st.get("chapters", []):
        out.append(f"### Chapter {ch['id']} ({ch.get('role')}) — problem: {ft(ch.get('problem'))}")
        for i, cid in enumerate(ch.get("cards", []), 1):
            if cid in C: out.append(card(C[cid], i))
        out.append("")
    for kind in ("minor", "fallout"):
        cs = [c for c in W["cards"] if c.get("relatedness") == kind]
        if cs:
            out.append(f"### {kind.title()} quests ({len(cs)})")
            out += [card(c) for c in cs]; out.append("")
for fs in W.get("faction_stories", []):
    cast = fs.get("cast", {})
    out += [f"## Faction story: {nm(fs['faction'])}",
            "cast: " + "; ".join(f"{k}={nm(v)}" for k, v in cast.items() if isinstance(v, str)),
            f"revelation ({(fs.get('revelation') or {}).get('kind')}): {ft((fs.get('revelation') or {}).get('fact'))}"]
    for m in fs.get("moves", []): out.append(f"- {m.get('kind')}{' ('+m['event']+')' if m.get('event') else ''}: {ft(m.get('fact'))}")
    p = fs["phases"]; seq = [("Routine", p["routine"]), ("Reveal", [p["reveal"]])] + [(f"Chapter {i+1}", ch["cards"]) for i, ch in enumerate(p["chapters"])] + [("Crisis", p["crisis"]), ("Epilogue", p.get("epilogue") or [])]
    n = 0
    for lab, ids in seq:
        if not ids: continue
        out.append(f"### {lab}")
        for cid in ids:
            n += 1
            if cid in C: out.append(card(C[cid], n))
    out.append(f"faction endings: {', '.join(fs.get('faction_endings') or [])}"); out.append("")
side = [c for c in W["cards"] if c.get("relatedness") in ("echo", "independent")]
if side:
    out.append(f"## Location pass ({len(side)} side quests)")
    by = collections.defaultdict(list)
    for c in side: by[home(c.get("place")) or home(c.get("offered_at"))].append(c)
    for h, cs in sorted(by.items(), key=lambda kv: str(kv[0])):
        out.append(f"### {nm(h)} ({len(cs)})"); out += [card(c) for c in cs]; out.append("")
print("\n".join(out))
