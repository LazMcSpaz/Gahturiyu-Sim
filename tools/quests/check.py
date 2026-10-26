#!/usr/bin/env python3
"""Mechanical rule checker for a generated quest set (shape layer).
Usage: python3 check.py questset.json > check-report.md
Checks only what can be verified from the data; premise-level judgement is left to review."""
import json, sys, collections, re

d = json.load(open(sys.argv[1]))
reg = d.get("registry", {})
people = {p["id"]: p for p in reg.get("people", [])}
places = {p["id"]: p for p in reg.get("places", [])}
things = {t["id"]: t for t in reg.get("things", [])}
factions = {f["id"]: f for f in reg.get("factions", [])}
facts = {f["id"]: f for f in d.get("facts", [])}
story = d.get("story", {})
cards = d.get("cards", [])
byid = {c["id"]: c for c in cards}
entities = {**people, **places, **things, **factions}

issues = collections.defaultdict(list)   # category -> list of messages
def bad(cat, msg): issues[cat].append(msg)

SCOPE = {"household": 0, "community": 1, "region": 2, "realm": 3}
AUTH_SLOTS = {"arbiter", "authority", "petition_target", "petition target", "answers_to", "answers-to"}
MAINLIKE = {"main", "minor"}

cast = story.get("cast", {}) or {}
cast_ids = {v for k, v in cast.items() if isinstance(v, str) and v.startswith("p-")}
stake = cast.get("stake")

def slot_items(c):
    s = c.get("slots") or {}
    out = []
    for role, v in s.items():
        if isinstance(v, list):
            out += [(role, x) for x in v if isinstance(x, str)]
        elif isinstance(v, str):
            out.append((role, v))
    return out

# 1. referential integrity ------------------------------------------------
for c in cards:
    for role, eid in slot_items(c):
        if eid not in entities:
            bad("references", f"{c['id']}: slot '{role}' points to missing id {eid}")
    if c.get("place") and c["place"] not in places:
        bad("references", f"{c['id']}: place {c['place']} missing")
    for f in c.get("depends_on", []) or []:
        fid = f if isinstance(f, str) else f.get("fact")
        if fid not in facts:
            bad("references", f"{c['id']}: depends_on {fid} not in facts")

# 2. names -----------------------------------------------------------------
names = collections.Counter(e.get("name", "").strip().lower() for e in entities.values() if e.get("name"))
for n, k in names.items():
    if k > 1: bad("names", f"name used {k} times: {n}")

# 3. reuse caps ------------------------------------------------------------
uses = collections.defaultdict(list)  # eid -> [(card, family, role)]
for c in cards:
    for role, eid in slot_items(c):
        uses[eid].append((c["id"], c.get("family"), role, c.get("relatedness")))
for eid, us in uses.items():
    if eid in factions: continue
    e = entities.get(eid, {})
    is_cast = eid in cast_ids or eid == stake
    counted = []
    for (cid, fam, role, rel) in us:
        if is_cast and rel in MAINLIKE: continue
        if e.get("standing") == "Top" and role.lower() in AUTH_SLOTS: continue
        counted.append((cid, fam))
    if is_cast:
        outside = [u for u in us if u[3] not in MAINLIKE]
        if outside: bad("cast", f"cast/stake {eid} used outside the main storyline: {[u[0] for u in outside]}")
    if len({cid for cid, _ in counted}) > 3:
        bad("reuse cap", f"{eid} ({e.get('name')}) on {len({x for x,_ in counted})} cards: {[x for x,_ in counted]}")
    famc = collections.Counter(f for _, f in {(cid, fam) for cid, fam in counted})
    for fam, k in famc.items():
        if k > 1: bad("once per family", f"{eid} ({e.get('name')}) appears {k}x in {fam}")

# 4. Top roster -------------------------------------------------------------
tops = [p for p in people.values() if p.get("standing") == "Top"]
if len(tops) > 4: bad("top roster", f"{len(tops)} Top people (roster 4)")
for p in tops:
    if not p.get("premade"): bad("top roster", f"{p['id']} is Top but not premade")

# 5. chapters ----------------------------------------------------------------
chapters = {ch["id"]: ch for ch in story.get("chapters", [])}
nodes = {n["id"]: n for n in story.get("nodes", [])}
node_of_ch = {ch["id"]: ch.get("node") for ch in chapters.values()}
layer_of_node = {n["id"]: n.get("layer") for n in nodes.values()}
for chid, ch in chapters.items():
    cs = [byid[x] for x in ch.get("cards", []) if x in byid]
    fams = [c.get("family") for c in cs]
    if len(fams) != len(set(fams)): bad("chapter variety", f"{chid}: repeated family {fams}")
    # cast once per chapter
    cnt = collections.Counter()
    for c in cs:
        for role, eid in slot_items(c):
            if eid in cast_ids: cnt[eid] += 1
    for eid, k in cnt.items():
        if k > 1: bad("cast per chapter", f"{chid}: {eid} fills {k} slots")
    if cs:
        if cs[0].get("position") != "opening": bad("chapter shape", f"{chid}: first card not 'opening'")
        if cs[-1].get("position") != "turn": bad("chapter shape", f"{chid}: last card not 'turn'")

def turn_card(chid):
    cs = chapters.get(chid, {}).get("cards", [])
    return byid.get(cs[-1]) if cs else None
ch_of_node = {v: k for k, v in node_of_ch.items()}

# edges / diamond variety
edges = story.get("edges", [])
indeg = collections.Counter(); outdeg = collections.Counter()
for e in edges:
    outdeg[e.get("from")] += 1; indeg[e.get("to")] += 1
    if not e.get("thread"): bad("edges", f"edge {e.get('from')}->{e.get('to')} has no thread")
    a, b = ch_of_node.get(e.get("from")), ch_of_node.get(e.get("to"))
    ta, tb = turn_card(a) if a else None, turn_card(b) if b else None
    if ta and tb and ta.get("family") == tb.get("family"):
        bad("diamond variety", f"linked turn cards share family {ta.get('family')}: {e.get('from')}->{e.get('to')}")
    if ta and b:
        ob = byid.get(chapters[b]["cards"][0]) if chapters.get(b, {}).get("cards") else None
        if ob and ob.get("family") == ta.get("family"):
            bad("chapter variety", f"opening of {b} shares family with linked turn of {a} ({ta.get('family')})")
role_of_layer = {l.get("layer"): (l.get("role") or "").lower() for l in story.get("layers", [])}
ret_layers = {L for L,r in role_of_layer.items() if r=="return"}
maxlayer = max([L for L,r in role_of_layer.items() if r=="climax"] or [max([n.get("layer", 0) for n in nodes.values()] or [0])])
for nid, n in nodes.items():
    L = n.get("layer")
    if L in ret_layers: continue
    if L != 1 and indeg[nid] == 0: bad("edges", f"{nid} has no incoming edge")
    if L != maxlayer and outdeg[nid] == 0: bad("edges", f"{nid} has no outgoing edge")
for lay in story.get("layers", []):
    nids = lay.get("node_ids") or lay.get("node ids") or lay.get("nodes") or []
    fams = [ (turn_card(ch_of_node.get(x)) or {}).get("family") for x in nids]
    fams = [f for f in fams if f]
    if len(fams) != len(set(fams)): bad("diamond variety", f"layer {lay.get('layer')}: turn cards share a family {fams}")

# 6. ways --------------------------------------------------------------------
for c in cards:
    pool = c.get("ways_pool") or []
    rolled = c.get("ways_rolled") or []
    if not (2 <= len(rolled) <= 3): bad("ways", f"{c['id']}: rolled {len(rolled)} ways")
    kinds = set()
    for i in rolled:
        try: kinds.add(pool[i].get("kind"))
        except Exception: bad("ways", f"{c['id']}: rolled index {i} not in pool")
    if len(kinds) < 2: bad("ways", f"{c['id']}: rolled ways cover {len(kinds)} kind(s)")
    extra = [w for w in pool if (w.get("source") or "premise") != "premise"]
    base = len(pool) - len(extra)
    if base > 6: bad("ways", f"{c['id']}: premise pool {base} > 6")
    if len(extra) > 3: bad("ways", f"{c['id']}: {len(extra)} added ways > cap")

# 7. scope -------------------------------------------------------------------
layer_scope = {lay.get("layer"): lay.get("scope") for lay in story.get("layers", [])}
for c in cards:
    sc = SCOPE.get(c.get("scope"), None)
    if sc is None: bad("scope", f"{c['id']}: missing/unknown scope '{c.get('scope')}'"); continue
    if c.get("relatedness") not in ("main",) and sc > 1:
        bad("scope", f"{c['id']} ({c.get('relatedness')}): scope {c.get('scope')} exceeds community cap")
    if c.get("relatedness") == "main" and c.get("chapter") in chapters:
        lay = layer_of_node.get(node_of_ch.get(c["chapter"]))
        ls = SCOPE.get(layer_scope.get(lay), None)
        if ls is not None and sc > ls: bad("scope", f"{c['id']}: card scope above chapter scope")

# 8. plants ------------------------------------------------------------------
fact_planted_by = collections.defaultdict(set)   # fact -> set(chapter ids)
for c in cards:
    for pl in c.get("plants", []) or []:
        fid = pl if isinstance(pl, str) else pl.get("fact")
        if c.get("chapter"): fact_planted_by[fid].add(c["chapter"])
move_layer = {m["id"]: m.get("layer") for m in story.get("moves", [])}
move_fact = {}
for m in story.get("moves", []):
    f = m.get("fact"); fid = f if isinstance(f, str) else (f or {}).get("id")
    if fid: move_fact[fid] = m.get("layer")
chapters_by_layer = collections.defaultdict(set)
for chid, nid in node_of_ch.items():
    chapters_by_layer[layer_of_node.get(nid)].add(chid)
rev = (story.get("revelation") or {}).get("fact")
rev = rev if isinstance(rev, str) else (rev or {}).get("id")
doc_plants = 0
for c in cards:
    for pl in c.get("plants", []) or []:
        if isinstance(pl, dict) and pl.get("document"): doc_plants += 1
if doc_plants > 1: bad("plants", f"{doc_plants} document plants (limit 1)")
for c in cards:
    if c.get("relatedness") not in ("main", "minor"): continue
    ch = c.get("chapter") or c.get("attached_to")
    lay = layer_of_node.get(node_of_ch.get(ch))
    if lay is None: continue
    for f in c.get("depends_on", []) or []:
        fid = f if isinstance(f, str) else f.get("fact")
        if fid in move_fact and (move_fact[fid] or 99) <= lay: continue
        ok = False
        for L, chs in chapters_by_layer.items():
            if L is not None and L < lay and chs and chs <= fact_planted_by.get(fid, set()): ok = True
        # same-chapter payoff for turn cards
        if not ok and c.get("position") == "turn" and ch in fact_planted_by.get(fid, set()): ok = True
        if not ok and c.get("relatedness") == "minor" and ch in fact_planted_by.get(fid, set()): ok = True  # assembly §6/§8: minor quests may use their chapter's unique plant
        if not ok and (facts.get(fid, {}).get("source") in (c["id"],)): ok = True
        if not ok: bad("plant rule", f"{c['id']}: depends on {fid} not planted in every chapter of an earlier layer")
# climax payoffs
climax_layer = maxlayer
for chid in chapters_by_layer.get(climax_layer, []):
    t = turn_card(chid)
    if not t: continue
    deps = [f if isinstance(f, str) else f.get("fact") for f in t.get("depends_on", []) or []]
    src_layers = set()
    for fid in deps:
        for L, chs in chapters_by_layer.items():
            if L and L < climax_layer and chs and chs <= fact_planted_by.get(fid, set()): src_layers.add(L)
    if len(src_layers) < 2: bad("climax", f"{t['id']}: pays off plants from {len(src_layers)} layer(s)")
    if rev and rev not in deps: bad("climax", f"{t['id']}: doesn't depend on the revelation {rev}")

# 9. fallout -----------------------------------------------------------------
main_entities = set()
for c in cards:
    if c.get("relatedness") in ("main", "minor"):
        for _, eid in slot_items(c): main_entities.add(eid)
        if c.get("place"): main_entities.add(c["place"])
fam_prem_by_move = collections.defaultdict(set)
for c in cards:
    if c.get("relatedness") == "main" and c.get("move"):
        fam_prem_by_move[c["move"]].add((c.get("family"), c.get("premise")))
fo = [c for c in cards if c.get("relatedness") == "fallout"]
for c in fo:
    for role, eid in slot_items(c):
        if eid in main_entities and eid not in factions: bad("fallout", f"{c['id']}: uses main/minor entry {eid} as {role}")
    if c.get("place") in main_entities: bad("fallout", f"{c['id']}: set at main-card place {c['place']}")
    if (c.get("family"), c.get("premise")) in fam_prem_by_move.get(c.get("source_move"), set()):
        bad("fallout", f"{c['id']}: same family+premise as a main card from move {c.get('source_move')}")
    if not c.get("move_fills"): bad("fallout", f"{c['id']}: no move_fills (delete test)")
fpre = collections.Counter(c.get("premise") for c in fo)
for p, k in fpre.items():
    if k > 1: bad("fallout", f"premise '{p}' repeats {k}x across fallout")
ffam = collections.Counter(c.get("family") for c in fo)
for f, k in ffam.items():
    if k > 2: bad("fallout", f"family {f} used {k}x across fallout (max 2)")
faces = {c.get("face") for c in fo}
if len(fo) >= 3:
    for need in ("profit", "resist", "endure"):
        if need not in faces: bad("fallout", f"no '{need}' face")
if sum(1 for c in fo if c.get("face") == "mirror" or c.get("mirror")) != 1: bad("fallout", "needs exactly one mirror card")

# 10. twist groups -----------------------------------------------------------
def tgroup(c):
    t = c.get("twist")
    return (t or {}).get("group") if isinstance(t, dict) else None
for name, sel in (("main storyline", lambda c: c.get("relatedness") in ("main", "minor")), ("fallout", lambda c: c.get("relatedness") == "fallout")):
    g = collections.Counter(tgroup(c) for c in cards if sel(c) and tgroup(c))
    for grp, k in g.items():
        if k > (2 if name == "main storyline" else 1): bad("twist groups", f"{name}: group '{grp}' used {k}x")

# report -----------------------------------------------------------------------
rel = collections.Counter(c.get("relatedness") for c in cards)
print("# Rule-check report\n")
print(f"- Cards: {len(cards)} — " + ", ".join(f"{k} {v}" for k, v in rel.most_common()))
print(f"- Registry: {len(people)} people, {len(places)} places, {len(things)} things, {len(factions)} factions, {len(facts)} facts")
print(f"- Chapters: {len(chapters)}; nodes: {len(nodes)}; edges: {len(edges)}; moves: {len(story.get('moves', []))}")
total = sum(len(v) for v in issues.values())
print(f"- **Issues found: {total}**\n")
for cat in sorted(issues):
    print(f"## {cat} ({len(issues[cat])})")
    for m in issues[cat][:40]: print(f"- {m}")
    if len(issues[cat]) > 40: print(f"- …and {len(issues[cat]) - 40} more")
    print()
json.dump({k: v for k, v in issues.items()}, open(sys.argv[1].replace(".json", "-issues.json"), "w"), indent=1)
