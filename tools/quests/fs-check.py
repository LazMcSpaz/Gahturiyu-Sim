#!/usr/bin/env python3
"""Mechanical checker for one generated faction story (quest-faction-stories.md).
Usage: python3 fs-check.py faction-story.json > fs-check-report.md
Checks what the data can show; premise-level judgement is left to review."""
import json, sys, collections

d = json.load(open(sys.argv[1]))
reg = d.get("registry", {})
people = {p["id"]: p for p in reg.get("people", [])}
places = {p["id"]: p for p in reg.get("places", [])}
things = {t["id"]: t for t in reg.get("things", [])}
factions = {f["id"]: f for f in reg.get("factions", [])}
facts = {f["id"]: f for f in d.get("facts", [])}
cards = d["cards"]
byid = {c["id"]: c for c in cards}
ents = {**people, **places, **things, **factions}
issues = collections.defaultdict(list)
def bad(cat, msg): issues[cat].append(msg)

SCOPE = {"household": 0, "community": 1, "region": 2, "realm": 3}
ORDEAL_DOORS = {
    "captive": {("Rescue", "Kidnapped"), ("Rescue", "Bound by debt"), ("Acquire", "Won't sell"), ("Acquire", "Under seal")},
    "disgraced": {("Dispute", "Public offense"), ("Passage", "Unwelcome cargo"), ("Restore", "Lost good name")},
    "estranged": {("Reckoning", "Betrayal"), ("Restore", "Estrangement"), ("Dispute", "Contested inheritance")},
}
TWIST_GROUPS = {
    "the other side is right": {"justified captor", "sympathetic culprit", "law on the wrong side", "the law is on the wrong side",
        "rightful keeper", "pursuer has a point", "needs it more", "the fix protects someone",
        "the contract is on the wrong side", "the old heads were right once"},
    "you started it": {"old wound", "one of us caused it", "old blood", "someone was told first", "a lamp still lit", "old debts"},
    "innocents pay": {"caught in the middle", "load-bearing", "one person pays", "protector's price", "the one left behind",
        "changed", "not alone", "the champion was promised something", "someone's name is in it",
        "someone's going to talk", "the goods are evidence", "better than they know"},
    "they don't want it": {"unwilling captive", "the wronged don't want it", "the house won't thank you"},
}
def group_of(t):
    t = (t or "").lower().strip().rstrip(".")
    for g, s in TWIST_GROUPS.items():
        if t in s: return g
    return None

def slots(c):
    out = []
    for r, v in (c.get("slots") or {}).items():
        for x in (v if isinstance(v, list) else [v]):
            if isinstance(x, str): out.append((r, x))
    return out

# references
for c in cards:
    for r, e in slots(c):
        if e not in ents and e not in facts: bad("references", f"{c['id']}: slot {r} -> missing {e}")
    for k in ("place", "offered_at"):
        if c.get(k) and c[k] not in places: bad("references", f"{c['id']}: {k} {c[k]} missing")
    for f in c.get("depends_on", []) or []:
        if f not in facts: bad("references", f"{c['id']}: depends_on {f} missing")

# names
nm = collections.Counter(e.get("name", "").lower() for e in ents.values() if e.get("name"))
for n, k in nm.items():
    if k > 1: bad("names", f"name used {k}x: {n}")

stories = d["faction_stories"]
for fs in stories:
    tag = fs["faction"]
    def bad(cat, msg, tag=tag): issues[cat].append(f"[{tag}] {msg}")
    order = fs["phases"]
    mine = [byid[x] for x in (order["routine"] + [order["reveal"]] + [x for ch in order["chapters"] for x in ch["cards"]] + order["crisis"] + (order.get("epilogue") or [])) if x in byid]
    # length and phase order
    seq = order["routine"] + [order["reveal"]] + [x for ch in order["chapters"] for x in ch["cards"]] + order["crisis"] + (order.get("epilogue") or [])
    if not 9 <= len(seq) <= 12: bad("shape", f"length {len(seq)} outside 9-12")
    if not 3 <= len(order["routine"]) <= 4: bad("shape", f"routine has {len(order['routine'])} cards")
    if len(order["chapters"]) != 2: bad("shape", "need exactly 2 chapters")
    for ch in order["chapters"]:
        if not 2 <= len(ch["cards"]) <= 3: bad("shape", f"chapter {ch['id']} has {len(ch['cards'])} cards")
        fams = [byid[x]["family"] for x in ch["cards"]]
        if len(fams) != len(set(fams)): bad("variety", f"chapter {ch['id']} repeats a family: {fams}")
    if len(order["crisis"]) != 1: bad("shape", "crisis must be exactly 1 card")
    for x in seq:
        if x not in byid: bad("references", f"phase list names missing card {x}")

    # routine rules
    rfam = [byid[x]["family"] for x in order["routine"]]
    for a, b in zip(rfam, rfam[1:]):
        if a == b: bad("variety", f"back-to-back routine cards share family {a}")
    for f, k in collections.Counter(rfam).items():
        if k > 2: bad("variety", f"routine uses {f} {k} times")
    if not any(byid[x].get("signature") for x in order["routine"]): bad("routine", "no routine card uses a signature premise")
    quiet = {}
    for x in order["routine"]:
        c = byid[x]
        q = [p for p in c.get("plants", []) if p.get("type") == "quiet"]
        if len(q) != 1: bad("routine", f"{x}: needs exactly one quiet plant, has {len(q)}")
        for p in q: quiet[p["fact"]] = x
        if SCOPE.get(c.get("scope"), 9) > 1: bad("scope", f"{x}: routine card above community")
    odd = [x for x in order["routine"] if byid[x].get("odd_job")]
    if len(odd) != 1: bad("routine", f"need exactly one odd job, found {len(odd)}")
    elif odd[0] == order["routine"][0]: bad("routine", "the odd job is the first routine card")
    else:
        oc = byid[odd[0]]; hand = fs["cast"]["hand"]
        hidden_ids = {e for r, e in slots(oc)} | set(oc.get("details", {}).get("named_in_truth", []) or [])
        nit = set(oc.get("details", {}).get("named_in_truth", []) or [])
        secret_link = any(l.get("to") in (hand, fs["cast"]["enemy"]) and l.get("visibility") == "secret" for r, e in slots(oc) for l in people.get(e, {}).get("links", []))
        proxy_in_truth = any(any(l.get("to") in (hand, fs["cast"]["enemy"]) for l in people.get(e, {}).get("links", [])) for e in nit)
        if not (hand in nit or proxy_in_truth or secret_link): bad("routine", f"odd job {odd[0]}: Hand/proxy not in a hidden role (named in truth, or a secret link)")

    # reveal
    rv = byid[order["reveal"]]
    rev = fs["revelation"]
    rf = facts.get(rev["fact"], {})
    if order["reveal"] not in (rf.get("published_by") or []): bad("reveal", "revelation not published_by the reveal card")
    if rf.get("visibility") != "secret": bad("reveal", "revelation fact must be stored secret")
    paid = [f for f in rv.get("depends_on", []) if f in quiet]
    if len(set(quiet[f] for f in paid)) < 2: bad("reveal", f"reveal pays off quiet plants from {len(set(quiet[f] for f in paid))} routine cards (need 2)")
    if not any(p.get("fact") == rev["fact"] and p.get("knower") for p in rv.get("plants", [])):
        bad("reveal", "reveal card doesn't carry the revelation as a plant with a knower")

    # moves
    mv = {m["id"]: m for m in fs["moves"]}
    kinds = [m["kind"] for m in fs["moves"]]
    if kinds != ["move", "ordeal event", "last move"]: bad("moves", f"move kinds {kinds}, want move / ordeal event / last move")
    for m in fs["moves"]:
        if SCOPE.get(m["scope"], 9) > 2: bad("scope", f"move {m['id']} above region")
        if m["fact"] not in facts or facts[m["fact"]].get("visibility") != "public": bad("moves", f"move {m['id']} fact missing or not public")
    ch1, ch2 = order["chapters"]
    ev = fs["moves"][1].get("event")
    turn2 = byid[ch2["cards"][-1]]
    if ev in ORDEAL_DOORS and (turn2["family"], turn2["premise"]) not in ORDEAL_DOORS[ev]:
        bad("moves", f"chapter 2 turn {turn2['id']} ({turn2['family']} {turn2['premise']}) is not a door for '{ev}'")
    for ch, m in ((ch1, fs["moves"][0]), (ch2, fs["moves"][1])):
        op = byid[ch["cards"][0]]
        if m["fact"] not in op.get("depends_on", []): bad("moves", f"chapter {ch['id']} opening card doesn't grow from its move")
    t1 = byid[ch1["cards"][-1]]
    if not any(p.get("type") == "ally" for p in t1.get("plants", [])): bad("plants", "chapter 1 turn card has no ally plant")
    if t1["family"] == "Investigate": bad("story", "chapter 1 turn card is Investigate")
    if not [p for p in turn2.get("plants", []) if p.get("type") in ("shared", "unique")]:
        bad("plants", "chapter 2 turn card plants nothing for the crisis")
    if not any(byid[x].get("private_side") for ch in order["chapters"] for x in ch["cards"]):
        bad("story", "no phase-3 card draws on the faction's wound or hidden side")

    # crisis
    cr = byid[order["crisis"][0]]
    fac = factions[fs["faction"]]
    def _home(pid):
        for _ in range(6):
            if pid == fac.get("seat"): return pid
            q = places.get(pid)
            if not q: return pid
            pid = q.get("at") or q.get("in") or q.get("parent")
        return pid
    if _home(cr.get("place")) != fac.get("seat"): bad("crisis", f"crisis not at headquarters ({cr.get('place')} vs {fac.get('seat')})")
    opp = cr.get("opposing_slot")
    enemy_in = fs["cast"]["enemy"] in [e for r, e in slots(cr)]
    if not enemy_in: bad("crisis", "enemy doesn't fill a slot on the crisis card")
    if (cr.get("slots") or {}).get(opp) != fs["cast"]["enemy"]: bad("crisis", f"enemy is not in the opposing slot ({opp})")
    hf = {fs["cast"]["heart"], fs["cast"].get("fallback")}
    if not any((cr.get("slots") or {}).get(r) in hf for r in ("requester", "wronged", "party_a", "party", "principal")): bad("crisis", "heart or fallback not in requester/wronged/party slot")
    deps = set(cr.get("depends_on", []))
    if rev["fact"] not in deps: bad("crisis", "crisis doesn't pay off the revelation")
    p3 = {p["fact"] for ch in order["chapters"] for x in ch["cards"] for p in byid[x].get("plants", [])}
    if not deps & p3: bad("crisis", "crisis pays off no phase-3 plant")
    if fs["moves"][2]["fact"] not in deps: bad("crisis", "crisis doesn't grow from the last move")
    pool = cr.get("ways_pool", []); rolled = cr.get("ways_rolled", [])
    cb = [i for i, w in enumerate(pool) if w.get("source") == "code_break"]
    if len(cb) != 1 or cb[0] not in rolled: bad("crisis", "crisis must roll exactly one code-break way")
    if not 3 <= len(fs.get("faction_endings", [])) <= 4: bad("crisis", "need 3-4 faction endings")

    # ways rolled: 2-3, >=2 kinds, no code-forbidden
    for c in mine:
        pool = c.get("ways_pool", []); r = c.get("ways_rolled", [])
        if not 2 <= len(r) <= 3: bad("ways", f"{c['id']}: rolled {len(r)} ways")
        ks = {pool[i]["kind"] for i in r if i < len(pool)}
        if len(ks) < 2: bad("ways", f"{c['id']}: rolled ways cover {len(ks)} kind(s)")
        extra = [w for w in pool if w.get("source") not in ("premise", None)]
        cap = 3 if c.get("phase") == "crisis" else 2
        if len(extra) > cap: bad("ways", f"{c['id']}: pool grew by {len(extra)} beyond premise")
        for i in r:
            if i < len(pool) and pool[i].get("code_forbidden"): bad("code", f"{c['id']}: rolled a code-forbidden way")
        for e in ([] if c.get("phase") == "crisis" else c.get("possible_endings", [])):
            if e in [x if isinstance(x, str) else x.get("ending") for x in (fs.get("code_endings") or [])]: bad("code", f"{c['id']}: code-forbidden ending '{e}' still offered")

    # scope, foundational, twists
    grp = collections.Counter()
    for c in mine:
        if SCOPE.get(c.get("scope"), 9) > 2: bad("scope", f"{c['id']}: above region")
        for r, e in slots(c):
            if "foundational" in (ents.get(e, {}).get("tags") or []): bad("scope", f"{c['id']}: uses foundational {e}")
        tw = (c.get("twist") or {}).get("name") if isinstance(c.get("twist"), dict) else c.get("twist")
        if tw:
            g = group_of(tw)
            if g: grp[g] += 1
    for g, k in grp.items():
        if k > 2: bad("twists", f"group '{g}' used {k} times (max 2 per faction story)")

    # heart
    heart = fs["cast"]["heart"]
    if people.get(heart, {}).get("standing") not in ("Low", "Middle"): bad("cast", "heart must be Low or Middle")
    if people.get(heart, {}).get("dead"): bad("cast", "heart is dead")
    # leader appears
    ldr = fs["cast"]["leader"]
    if not any(ldr == e for c in mine for r, e in slots(c)) and not any(c.get("giver") == ldr for c in mine): bad("cast", "leader neither fills a slot nor gives a card")

    # reuse: non-cast cap 3, once per family (within this story)
    castset = {v for k, v in fs["cast"].items() if isinstance(v, str)}
    uses = collections.defaultdict(list)
    for c in mine:
        for r, e in slots(c):
            if e in castset or e in factions: continue
            uses[e].append((c["id"], c["family"]))
        if c.get("place") and c["place"] not in castset: uses[c["place"]].append((c["id"], c["family"]))
    for e, us in uses.items():
        cs = {u[0] for u in us}
        if len(cs) > 3 and people.get(e, {}).get("premade") is not True:
            bad("reuse", f"{e} used on {len(cs)} cards")
        fam = collections.Counter(f for cid, f in {(u[0], u[1]) for u in us})
        for f, k in fam.items():
            if k > 1 and people.get(e, {}).get("premade") is not True and e not in places:
                bad("reuse", f"{e} appears in {k} {f} cards")

    # v4 story craft
    p3 = [x for ch in order["chapters"] for x in ch["cards"]]
    p3slots = {e for x in p3 for r, e in slots(byid[x])}
    if fs["cast"]["enemy"] not in p3slots: bad("craft", "enemy fills no slot on a phase-3 card")
    if fs["cast"]["hand"] not in p3slots: bad("craft", "Hand fills no slot on a phase-3 card")
    if not any(fs["cast"]["heart"] in [e for r, e in slots(byid[x])] for x in order["routine"]): bad("craft", "heart doesn't appear on a routine card")
    for ch in order["chapters"]:
        ids = ch["cards"]
        lv = [byid[x].get("pressure_level") for x in ids]
        if None in lv: bad("craft", f"chapter {ch['id']}: pressure_level missing")
        elif any(b < a for a, b in zip(lv, lv[1:])): bad("craft", f"chapter {ch['id']}: pressure falls {lv}")
        for a_, b_ in zip(ids, ids[1:]):
            prev_plants = {p_["fact"] for p_ in byid[a_].get("plants", [])}
            if not prev_plants & set(byid[b_].get("depends_on", [])): bad("craft", f"{b_} doesn't depend on a plant from {a_} (chain)")
    lv1 = [byid[order["chapters"][0]["cards"][-1]].get("pressure_level"), byid[order["chapters"][1]["cards"][0]].get("pressure_level")]
    if None not in lv1 and lv1[1] < lv1[0]: bad("craft", "chapter 2 opens below chapter 1's turn level")
    levers = [m.get("lever") for m in fs["moves"]]
    if None in levers: bad("craft", "a move has no lever")
    elif len(set(levers)) < 2: bad("craft", f"moves use one lever only: {levers}")
    crp = cr.get("ways_pool", []); crr = cr.get("ways_rolled", [])
    if not any(crp[i].get("source") == "ally" for i in crr if i < len(crp)): bad("craft", "crisis doesn't roll the ally way")
    if len(crr) < 3: bad("craft", f"crisis rolls {len(crr)} ways (need 3)")
    for e_ in people.values():
        if e_.get("dies_at"):
            mv_ids = [m["id"] for m in fs["moves"]]
            if e_["dies_at"] in mv_ids:
                k = mv_ids.index(e_["dies_at"]); before = (order["routine"] + [order["reveal"]] + [x for ch in order["chapters"][:k] for x in ch["cards"]]) if k < 2 else seq[:-1 - len(order.get("epilogue") or [])]
                if not any(e_["id"] in [e for r, e in slots(byid[x])] for x in before if x in byid): bad("craft", f"{e_['id']} dies at {e_['dies_at']} but is never met before")
    # pinning
    towns = {byid[x].get("offered_at") for x in order["routine"]}
    if len(towns) < 2: bad("pinning", "routine jobs offered in fewer than 2 places")
    far = [x for ch in order["chapters"] for x in ch["cards"] if byid[x].get("far")]
    if not far: bad("pinning", "no story card is a journey away")


# world variety ledger (v4)
rv_fam = collections.Counter(byid[fs["phases"]["reveal"]]["family"] for fs in stories if fs["phases"]["reveal"] in byid)
for f_, k in rv_fam.items():
    if k > 2: bad("ledger", f"{k} reveals are {f_}")
fp = collections.Counter((c["family"], c["premise"]) for c in cards if c.get("relatedness") == "faction")
for k_, v in fp.items():
    if v > 2: bad("ledger", f"{k_[0]}/{k_[1]} used {v}x across guild stories")
twc = collections.Counter(((c.get("twist") or {}).get("name") if isinstance(c.get("twist"), dict) else c.get("twist") or "").lower() for c in cards if c.get("relatedness") == "faction")
for k_, v in twc.items():
    if k_ and v > 2: bad("ledger", f"twist '{k_}' used {v}x")
lev = collections.Counter(m.get("lever") for fs in stories for m in fs["moves"])
for k_, v in lev.items():
    if k_ and v > 3: bad("ledger", f"lever '{k_}' used {v}x")
kills = sum(1 for fs in stories if any(p_.get("dies_at") == fs["moves"][2]["id"] for p_ in people.values()))
if kills > 2: bad("ledger", f"{kills} last moves kill someone")

# world spread across stories
def bad(cat, msg): issues[cat].append(msg)
leaders_enemy = sum(1 for fs in stories if fs["cast"]["enemy"] == fs["cast"]["leader"])
if leaders_enemy > 1: bad("spread", f"{leaders_enemy} stories have the leader as enemy")
ht = collections.Counter(fs["cast"]["heart_type"] for fs in stories)
for h, k in ht.items():
    if k > 2: bad("spread", f"heart type {h} used by {k} stories")
cap = sum(1 for fs in stories if fs["moves"][1].get("event") == "captive")
if cap > 1: bad("spread", f"{cap} stories roll captive")
within = sum(1 for fs in stories if fs["cast"]["shape"] == "Within")
if within > 2: bad("spread", f"{within} Within enemies")
casts = [set(v for v in fs["cast"].values() if isinstance(v, str) and v.startswith("p-")) for fs in stories]
for i in range(len(casts)):
    for j in range(i+1, len(casts)):
        if casts[i] & casts[j]: bad("spread", f"stories share cast: {casts[i] & casts[j]}")
print("# Faction story check\n")
total = sum(len(v) for v in issues.values())
print(f"Cards: {len(cards)} · Issues: {total}\n")
for k in sorted(issues):
    print(f"## {k} ({len(issues[k])})")
    for m in issues[k]: print(f"- {m}")
    print()
if not total: print("All mechanical checks pass.")
