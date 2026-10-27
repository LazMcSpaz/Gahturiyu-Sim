#!/usr/bin/env python3
"""World-level checker: coherence (references, reuse, states, loose threads), span (coverage per place and region),
depth, place ties, variety, clocks and travel. Usage: python3 world_check.py world.json [report.md] [metrics.json]"""
import json, sys, collections, math, re
W = json.load(open(sys.argv[1]))
rep_path = sys.argv[2] if len(sys.argv) > 2 else None
met_path = sys.argv[3] if len(sys.argv) > 3 else None
reg = W["registry"]
P = {p["id"]: p for p in reg["people"]}; PL = {p["id"]: p for p in reg["places"]}
T = {t["id"]: t for t in reg.get("things", [])}; FA = {f["id"]: f for f in reg["factions"]}
F = {f["id"]: f for f in W["facts"]}; C = {c["id"]: c for c in W["cards"]}
ENT = {**P, **PL, **T, **FA}
mp = W["map"]; MAPP = {p["id"]: p for p in mp["places"]}
issues = collections.defaultdict(list); info = collections.defaultdict(list)
def bad(k, m): issues[k].append(m)
def note(k, m): info[k].append(m)

def slots(c):
    out = []
    for r, v in (c.get("slots") or {}).items():
        for x in (v if isinstance(v, list) else [v]):
            if isinstance(x, str): out.append((r, x))
    return out
def home_place(pid):
    """Resolve a place to the map place it sits in."""
    seen = 0
    while pid and pid not in MAPP and seen < 6:
        p = PL.get(pid); seen += 1
        if not p: return None
        pid = p.get("at") or p.get("in") or p.get("parent")
    return pid if pid in MAPP else None

# ---------- 1. references ----------
for c in W["cards"]:
    for r, e in slots(c):
        if e not in ENT and e not in F: bad("references", f"{c['id']}: slot {r} -> missing {e}")
    for k in ("place", "offered_at"):
        if c.get(k) and c[k] not in PL: bad("references", f"{c['id']}: {k} {c[k]} missing")
    for f in c.get("depends_on") or []:
        if f not in F: bad("references", f"{c['id']}: depends_on {f} missing")
    for p in c.get("plants") or []:
        if p.get("fact") not in F: bad("references", f"{c['id']}: plant {p.get('fact')} missing")
    if c.get("truth") and isinstance(c["truth"], str) and c["truth"] not in F: bad("references", f"{c['id']}: truth {c['truth']} missing")
for p in reg["places"]:
    if p["id"] not in MAPP and not home_place(p["id"]): bad("references", f"place {p['id']} ({p.get('name')}) has no 'at' chain to a map place")

# ---------- 2. names ----------
TITLES = {"captain","lady","lord","master","mistress","old","young","sister","brother","father","mother","high","priest","priestess","wharf-master","ser","dame","inquisitor","factor","steward","widow","granny","goodwife","goodman"}
nm = collections.Counter(p["name"].strip().lower() for p in reg["people"])
for n, k in nm.items():
    if k > 1: bad("names", f"full name used {k}x: {n}")
for c in W["cards"]:
    st = collections.defaultdict(list)
    for r, e in slots(c):
        p = P.get(e)
        if p and not p.get("premade"):
            parts = [w for w in re.split(r"[\s,]+", p["name"].strip()) if w.lower() not in TITLES]
            first = (parts[0] if parts else p["name"]).lower()
            st[first[:4]].append(p["name"])
    for s, ns in st.items():
        if len(set(ns)) > 1: bad("names", f"{c['id']}: shared stem '{s}': {sorted(set(ns))}")

# ---------- 3. reuse ----------
AUTH = {"arbiter", "authority", "petition_target", "answers_to", "answers-to", "route_authority", "ring_keeper", "ring-keeper", "judge"}
cast_of = collections.defaultdict(set)  # person -> story ids where cast
st = W.get("story") or {}
for k, v in (st.get("cast") or {}).items():
    if isinstance(v, str): cast_of[v].add("main")
for fs in W["faction_stories"]:
    for k, v in (fs.get("cast") or {}).items():
        if isinstance(v, str): cast_of[v].add(fs.get("id") or fs["faction"])
def story_of(c):
    if c.get("relatedness") in ("main", "minor") or c.get("chapter") and c.get("relatedness") != "faction": return "main"
    if c.get("relatedness") == "faction": return c.get("faction_story")
    return None
uses = collections.defaultdict(list)
for c in W["cards"]:
    sid = story_of(c)
    for r, e in slots(c):
        if e in FA or e in F: continue
        if e == c.get("place"): continue
        if P.get(e, {}).get("premade") and r in AUTH: continue
        if sid and sid in cast_of.get(e, ()) or (sid == "main" and "main" in cast_of.get(e, ())): continue
        uses[e].append((c["id"], c["family"], r))
    pl = c.get("place")
    if pl and pl not in MAPP or (pl in MAPP and MAPP[pl]["size"] in ("presence",)):
        uses[pl].append((c["id"], c["family"], "place"))
    elif pl in MAPP:
        uses[pl].append((c["id"], c["family"], "place (whole settlement)"))
for e, us in uses.items():
    cards = {u[0] for u in us}
    if len(cards) > 3: bad("reuse", f"{e} ({ENT.get(e, {}).get('name', '?')}) used on {len(cards)} cards: {sorted(cards)[:6]}")
    fam = collections.Counter(f for (cid, f) in {(u[0], u[1]) for u in us})
    for f, k in fam.items():
        if k > 1: bad("reuse", f"{e} ({ENT.get(e, {}).get('name', '?')}) in {k} {f} cards")
    OKPAIR = {"requester", "party_a", "party_b", "party", "wronged", "victim", "subject", "principal", "principal_a"}
    for cid in {u[0] for u in us}:
        rs = [u[2] for u in us if u[0] == cid and not u[2].startswith("place")]
        rs2 = [r for r in rs if r not in {x for x in rs if x in OKPAIR} or rs.count(r) > 1] if set(rs) <= OKPAIR | {"requester"} and "requester" in rs else rs
        if len(rs) > 1 and not (set(rs) <= OKPAIR and "requester" in rs and len(rs) == 2):
            bad("reuse", f"{e} fills {len(rs)} slots on {cid}: {rs}")

# ---------- 4. states ----------
LIVING = {"requester", "party_a", "party_b", "party", "holder", "captor", "pursuer", "wrongdoer", "culprit", "witness", "guide", "champion",
          "opponent", "sponsor", "fixer", "giver", "principal", "principal_a", "principal_b", "seller", "buyer", "keeper"}
for c in W["cards"]:
    for r, e in slots(c):
        if P.get(e, {}).get("dead") and r in LIVING: bad("states", f"{c['id']}: dead person {e} fills living role {r}")

# ---------- 5. loose threads ----------
dep_by = collections.defaultdict(set)
for c in W["cards"]:
    for f in c.get("depends_on") or []: dep_by[f].add(c["id"])
for c in W["cards"]:
    for p in c.get("plants") or []:
        f = p.get("fact"); t = p.get("type")
        users = dep_by.get(f, set()) - {c["id"]}
        if not users:
            if t in ("quiet", "shared", "ally", "fault line", "lead"):
                bad("loose threads", f"{c['id']}: {t} plant {f} is never paid off")
            else:
                note("unpaid unique plants", f"{c['id']}: {t} plant {f}")
for fs in W["faction_stories"]:
    rv = (fs.get("revelation") or {}).get("fact")
    if rv and not (F.get(rv, {}).get("published_by")): bad("loose threads", f"{fs['faction']}: revelation not published")
    for m in fs.get("moves") or []:
        if not dep_by.get(m["fact"]): bad("loose threads", f"{fs['faction']}: move {m['id']} reaches no card")
if st:
    rvf = (st.get("revelation") or {}).get("fact")
    if rvf and not F.get(rvf, {}).get("published_by"): bad("loose threads", "main revelation not published")
    mids = {m["id"]: m for m in st.get("moves") or []}
    for m in mids.values():
        reached = dep_by.get(m.get("fact"), set()) | {c["id"] for c in W["cards"] if c.get("move") == m["id"] or c.get("source_move") == m["id"]}
        if not reached: bad("loose threads", f"main move {m['id']} reaches no card")
    for c in W["cards"]:
        if c.get("source_move") and c["source_move"] not in mids: bad("loose threads", f"{c['id']}: source_move {c['source_move']} not a main move")
for c in W["cards"]:
    lp = c.get("linked_pair")
    if lp and isinstance(lp, dict):
        o = C.get(lp.get("with"))
        if not o or (o.get("linked_pair") or {}).get("with") != c["id"]: bad("loose threads", f"{c['id']}: linked pair partner missing or not reciprocal")
    eo = c.get("echo_of")
    if eo and isinstance(eo, dict):
        if eo.get("person") not in P or eo.get("from_card") not in C: bad("loose threads", f"{c['id']}: echo_of points nowhere")
        elif eo.get("person") not in [e for r, e in slots(c)]: bad("loose threads", f"{c['id']}: echoed person not on the card")
    adv = c.get("advantage")
    if adv and isinstance(adv, dict) and adv.get("for_node") and adv["for_node"] not in C and adv["for_node"] not in {n.get("id") for n in st.get("nodes", [])} and adv["for_node"] not in {ch.get("id") for ch in st.get("chapters", [])}:
        bad("loose threads", f"{c['id']}: advantage for unknown node {adv['for_node']}")

# ---------- 6. coverage / span ----------
by_place = collections.defaultdict(list)
for c in W["cards"]:
    hp = home_place(c.get("place")) or home_place(c.get("offered_at"))
    if hp: by_place[hp].append(c["id"])
    else: bad("coverage", f"{c['id']}: card has no location on the map")
cov = {}
for p in mp["places"]:
    if p.get("secret"): continue
    n = len(by_place.get(p["id"], [])); lo, hi = p["quota"]
    cov[p["id"]] = n
    if p["size"] != "presence" and n == 0: bad("coverage", f"{p['name']} ({p['size']}) has no quests")
    elif n < lo: note("below quota", f"{p['name']} ({p['size']}): {n} < {lo}")
    if n > hi + 3: note("above quota", f"{p['name']} ({p['size']}): {n} > {hi}")
reg_cnt = collections.Counter(); reg_deep = collections.defaultdict(int)
def depth(c):
    if c.get("relatedness") in ("main", "faction"): return 4
    if c.get("relatedness") in ("minor", "fallout") or c.get("linked_pair") or c.get("echo_of"): return 3
    if c.get("twist") or (c.get("truth") and F.get(c["truth"], {}).get("visibility") == "secret"): return 2
    return 1
dep = {c["id"]: depth(c) for c in W["cards"]}
for p in mp["places"]:
    for cid in by_place.get(p["id"], []):
        reg_cnt[p["region"]] += 1
        if dep[cid] >= 3: reg_deep[p["region"]] += 1
for r in mp["regions"]:
    if reg_cnt[r["id"]] == 0: bad("span", f"region {r['name']} has no quests")
    elif reg_deep[r["id"]] == 0: bad("depth", f"region {r['name']} has no level-3+ thread")
for p in mp["places"]:
    if p["size"] == "great" and not any(dep[c] == 4 for c in by_place.get(p["id"], [])): bad("depth", f"great town {p['name']} has no story-arc presence")

# ---------- 7. place ties, variety ----------
tied = sum(1 for c in W["cards"] if c.get("place_tie"))
fam = collections.Counter(c["family"] for c in W["cards"])
prem = collections.Counter((c["family"], c["premise"]) for c in W["cards"])
tw = collections.Counter((c.get("twist") or {}).get("name") if isinstance(c.get("twist"), dict) else c.get("twist") for c in W["cards"])
ways = collections.Counter(w["text"].strip().lower() for c in W["cards"] for i, w in enumerate(c.get("ways_pool") or []) if i in (c.get("ways_rolled") or []))
for (f, pr), k in prem.items():
    if k > max(4, len(W["cards"]) // 12): note("repetition", f"premise {f}/{pr} used {k}x")
for wtext, k in ways.most_common(8):
    if k > 3: note("repetition", f"rolled way used {k}x: {wtext[:70]}")
for p in mp["places"]:
    cs = by_place.get(p["id"], [])
    if len(cs) >= 4:
        f2 = collections.Counter(C[c]["family"] for c in cs).most_common(1)[0]
        if f2[1] / len(cs) > 0.5: note("repetition", f"{p['name']}: {f2[1]}/{len(cs)} cards are {f2[0]}")

# ---------- 8. clocks, travel ----------
side = [c for c in W["cards"] if c.get("relatedness") in ("echo", "independent")]
clocked = [c for c in W["cards"] if c.get("clock")]
ELIG = {("Rescue", "Kidnapped"), ("Rescue", "Bound by debt"), ("Threat", "Raiders"), ("Threat", "Sickness or blight"), ("Restore", "Estrangement"), ("Acquire", "Passed on")}
for c in clocked:
    if c.get("relatedness") in ("main", "faction", "minor") or c.get("linked_pair"): bad("clocks", f"{c['id']}: story/minor/linked card has a clock")
    elif (c["family"], c["premise"]) not in ELIG and c["family"] != "Passage": bad("clocks", f"{c['id']}: {c['family']}/{c['premise']} not clock-eligible")
if side and len(clocked) > 0.1 * len(side) + 0.5: bad("clocks", f"{len(clocked)} clocks on {len(side)} side quests (> 1 in 10)")
def far(a, b):
    a, b = home_place(a), home_place(b)
    if not a or not b or a == b: return False
    return b not in MAPP[a]["near"]
farc = [c["id"] for c in side if c.get("offered_at") and far(c["offered_at"], c["place"])]
if side and len(farc) > 0.2 * len(side) + 0.5: bad("travel", f"{len(farc)}/{len(side)} side quests send the player farther than near (> 1 in 5)")

# ---------- 9. v5 checks (world-test fixes) ----------
last_ch = max((ch.get("id") for ch in (st or {}).get("chapters", []) if ch.get("id")), default=None, key=lambda x: str(x)) if st else None
for c in W["cards"]:
    key = c.get("phase") == "crisis" or c.get("position") == "turn" or (c.get("relatedness") == "main" and c.get("chapter") == last_ch)
    if key:
        pool = c.get("ways_pool") or []; rolled = set(c.get("ways_rolled") or [])
        for i, w in enumerate(pool):
            if w.get("source") in ("ally", "code_break") and i not in rolled:
                bad("plant-backed ways", f"{c['id']}: {w['source']} way in the pool but not rolled")
    tw0 = c.get("twist"); twn = (tw0.get("name") if isinstance(tw0, dict) else tw0)
    if not c.get("truth") and (not twn or twn == "none"): bad("blank cards", f"{c['id']} ({c.get('relatedness')}, {c['family']}): no truth and no twist")
    rolled_ways = [w for i, w in enumerate(c.get("ways_pool") or []) if i in (c.get("ways_rolled") or [])]
    if rolled_ways and not any(w.get("names_slot") for w in rolled_ways): note("generic ways", f"{c['id']}: no rolled way names anything on the card")
main_at = collections.Counter(home_place(c.get("place")) for c in W["cards"] if c.get("relatedness") in ("main", "minor"))
for pid, k in main_at.items():
    if pid and k > 5: bad("main cards per settlement", f"{MAPP[pid]['name']}: {k} main-story cards incl. minors (> 5)")
story_people = collections.defaultdict(set)
for c in W["cards"]:
    if c.get("relatedness") in ("main", "faction", "minor"):
        for r, e in slots(c):
            if e in P:
                hp = home_place(P[e].get("home")) or home_place(c.get("place"))
                parts = [w for w in re.split(r"[\s,]+", P[e]["name"].strip()) if w.lower() not in TITLES | {"headman", "headwoman", "clerk", "elder", "reeve", "warden", "keeper", "mother", "abbot"}]
                if hp and parts and not P[e].get("premade"): story_people[(MAPP[hp].get("region"), parts[0])].add(e)
for (rg, given), ids in story_people.items():
    names = {P[i]["name"] for i in ids}
    if len(ids) > 1 and len(names) > 1: bad("names", f"given name {given} on {len(ids)} story people in one region: {', '.join(sorted(names))}")

# ---------- 10. guild stories meet (faction-stories §8b) ----------
GUILDS = {"Writers": "f-writers", "Lantern": "f-lantern", "Old Hands": "f-old-hands", "Night Market": "f-night-market", "Dueling": "f-dueling"}
HALL = ("HQ", "reading room", "chapter house", "back room", "fence's shop", "ring")
crossroads = set()
for p in mp["places"]:
    gs = {g for x in p.get("presences_in_town") or [] for g in GUILDS if x.startswith(g) and any(h in x for h in HALL)}
    if len(gs) >= 2: crossroads.add(p["id"])
gcards = collections.defaultdict(list)
for c in W["cards"]:
    if c.get("faction_story"): gcards[c["faction_story"]].append(c)
fs_fac = {f.get("id"): f.get("faction") for f in W.get("faction_stories", [])}
uses = collections.defaultdict(set)
for fsid, cs in gcards.items():
    for c in cs:
        hp = home_place(c.get("place"))
        if hp in crossroads: uses[hp].add(fsid)
reg_g = collections.Counter(); presence_regions = {p.get("region") for p in mp["places"] if p.get("presences_in_town") or p.get("size") == "presence"}
for fsid, cs in gcards.items():
    regs = {MAPP[h].get("region") for h in (home_place(c.get("place")) for c in cs) if h}
    for c in cs:
        h = home_place(c.get("place"))
        if h: reg_g[MAPP[h].get("region")] += 1
    if len(regs) < 3: bad("guild placement", f"{fsid}: touches {len(regs)} regions (< 3)")
    shared = [c["id"] for c in cs if home_place(c.get("place")) in crossroads and len(uses[home_place(c.get("place"))]) >= 2]
    if len(shared) < 2: bad("guild placement", f"{fsid}: {len(shared)} cards in shared crossroads (< 2)")
    hosted = [c["id"] for c in cs if c.get("guild_meeting")]
    if len(hosted) > 2: bad("guild placement", f"{fsid}: hosts {len(hosted)} guild meetings (> 2)")
if gcards:
    involved = set()
    for fsid, cs in gcards.items():
        for c in cs:
            gm = c.get("guild_meeting")
            if gm: involved |= {fs_fac.get(fsid), gm.get("guild")}
    for fsid in gcards:
        if fs_fac.get(fsid) and fs_fac.get(fsid) not in involved: bad("guild placement", f"{fsid}: its guild takes part in no guild meeting")
    tot = sum(reg_g.values())
    for r in mp["regions"]:
        if r["id"] in presence_regions and reg_g[r["id"]] == 0: bad("guild placement", f"region {r['name']} has a guild presence but no guild-story card")
        if tot and reg_g[r["id"]] > tot / 3 + 0.5: bad("guild placement", f"region {r['name']} holds {reg_g[r['id']]}/{tot} guild-story cards (> a third)")
    for cid, fss in uses.items():
        if len(fss) == 1: note("guild placement", f"crossroads {MAPP[cid]['name']} used by one guild story only")

# ---------- 11. casting, seats, main-story craft (rules v5.2, casting v1.1) ----------
TITLE_WORDS = ("lord", "lady", "steward", "master", "captain", "speaker", "warden", "headman", "headwoman", "reeve", "magistrate", "abbot", "keeper")
for pl in reg["places"]:
    h = pl.get("holder")
    holders = {h} | {t.get("holder") for t in (pl.get("titles") or []) if isinstance(t, dict)}
    if not h: continue
    pname = pl.get("name", "")
    for p in reg["people"]:
        if p["id"] in holders: continue
        txt = (p.get("name", "") + " " + " ".join(p.get("traits") or [])).lower()
        if pname and f"of {pname.lower()}" in txt and any(w in txt for w in TITLE_WORDS):
            bad("seats", f"{p['id']} ({p['name']}) claims an office of {pname}, held by {h}")
story_people = {e for c in W["cards"] if c.get("relatedness") in ("main", "faction", "minor", "fallout") for r, e in slots(c) if e in P}
missing_voice = [i for i in story_people if not (P[i].get("people") and P[i].get("age") and P[i].get("voice"))]
if story_people: note("person fields", f"{len(missing_voice)}/{len(story_people)} people on story cards lack people/age/voice")
how = collections.Counter()
for c in W["cards"]:
    for r, v in (c.get("cast_log") or {}).items():
        hw = v.get("how") if isinstance(v, dict) else v
        if hw: how[hw] += 1
if how:
    tot = how["reused"] + how["created"]
    note("casting", f"cast_how: {dict(how)}; reuse rate {how['reused'] / max(1, tot):.0%} (aim 25-45%)")
NO_NONE = {("Threat", "Raiders"), ("Threat", "Protected beast"), ("Threat", "Encroachment")}
for c in W["cards"]:
    tw0 = c.get("twist"); twn = tw0.get("name") if isinstance(tw0, dict) else tw0
    if (c["family"], c["premise"]) in NO_NONE and (not twn or twn == "none"): bad("blank cards", f"{c['id']}: {c['premise']} may never roll twist none")
    if isinstance(tw0, dict) and tw0.get("name") and tw0.get("name") != "none" and not tw0.get("group"): bad("twists", f"{c['id']}: twist {tw0['name']} has no group")
if st:
    for t in st.get("open_threads") or []:
        if not t.get("closed_by") or t["closed_by"] not in C: bad("loose threads", f"open thread {t.get('id')} is never closed by a card")
    cast = st.get("cast") or {}
    chs = [ch for ch in st.get("chapters", []) if ch.get("id")]
    climax_ids = set()
    if chs:
        last_main = [ch for ch in chs if ch.get("id") != st.get("return")][-1]
        climax_ids = set(last_main.get("cards") or [])
    for role in ("antagonist", "lieutenant"):
        pid = cast.get(role)
        if pid and not any(pid in [e for r, e in slots(c)] for c in W["cards"] if c.get("relatedness") == "main" and c["id"] not in climax_ids):
            bad("meet before matters", f"main {role} {pid} is on no main card before the climax")
    heart = cast.get("heart"); fb = cast.get("fallback")
    for lay in st.get("layers") or []:
        nodes = set(lay.get("node ids") or [])
        lcards = [c for c in W["cards"] if c.get("relatedness") in ("main", "minor") and any(ch.get("node") in nodes and c["id"] in (ch.get("cards") or []) for ch in chs)]
        if lcards and not any(x in [e for r, e in slots(c)] for c in lcards for x in (heart, fb) if x):
            bad("heart offstage", f"layer {lay.get('layer')} ({lay.get('role')}): heart and fallback on none of its {len(lcards)} cards")

# ---------- report ----------
rel = collections.Counter(c.get("relatedness") for c in W["cards"])
metrics = {"cards": len(W["cards"]), "relatedness": rel, "families": fam, "depth": collections.Counter(dep.values()),
           "place_tie_share": round(tied / max(1, len(W["cards"])), 2), "coverage": cov,
           "regions": {r["name"]: {"cards": reg_cnt[r["id"]], "deep": reg_deep[r["id"]]} for r in mp["regions"]},
           "twists": tw, "clocks": len(clocked), "far_side": len(farc), "issues": {k: len(v) for k, v in issues.items()}}
L = ["# World check", "", f"Cards: {len(W['cards'])} · " + ", ".join(f"{k}: {v}" for k, v in rel.items()),
     f"Depth levels: {dict(sorted(collections.Counter(dep.values()).items()))} · place ties: {metrics['place_tie_share']:.0%} · clocks: {len(clocked)} · far side quests: {len(farc)}", "",
     "## Regions", *[f"- {r['name']}: {reg_cnt[r['id']]} cards, {reg_deep[r['id']]} at depth 3+" for r in mp["regions"]], "",
     "## Places (cards / quota)", *[f"- {p['name']} ({p['size']}): {cov.get(p['id'], 0)} / {p['quota'][0]}–{p['quota'][1]}" for p in mp["places"] if not p.get("secret")], "",
     "## Families", ", ".join(f"{k} {v}" for k, v in fam.most_common()), "",
     f"## Issues ({sum(len(v) for v in issues.values())})"]
for k in sorted(issues):
    L.append(f"### {k} ({len(issues[k])})"); L += [f"- {m}" for m in issues[k][:60]]
    if len(issues[k]) > 60: L.append(f"- … {len(issues[k]) - 60} more")
L += ["", "## Notes (not failures)"]
for k in sorted(info):
    L.append(f"### {k} ({len(info[k])})"); L += [f"- {m}" for m in info[k][:25]]
txt = "\n".join(L)
if rep_path: open(rep_path, "w").write(txt)
if met_path: json.dump(metrics, open(met_path, "w"), indent=1, default=dict)
print(txt[:6000])
