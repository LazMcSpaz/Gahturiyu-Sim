#!/usr/bin/env python3
"""Plan the location pass (quest-locations.md §3): roll each place's target, subtract world-pass cards pinned there,
and split places into agent batches by region. Usage: python3 plan_locations.py worldpass.json <seed> <n_batches> > plan.json"""
import json, sys, random, collections
W = json.load(open(sys.argv[1])); R = random.Random(int(sys.argv[2])); nb = int(sys.argv[3])
mp = W["map"]; MAPP = {p["id"]: p for p in mp["places"]}; PL = {p["id"]: p for p in W["registry"]["places"]}
def home(pid):
    for _ in range(6):
        if pid in MAPP: return pid
        p = PL.get(pid)
        if not p: return None
        pid = p.get("at") or p.get("in") or p.get("parent")
    return None
pinned = collections.Counter()
for c in W["cards"]:
    h = home(c.get("place")) or home(c.get("offered_at"))
    if h: pinned[h] += 1
rows = []
for p in mp["places"]:
    if p.get("secret"): continue
    lo, hi = p["quota"]
    target = R.randint(lo, hi) if hi else 0
    if p["size"] == "presence": target = 1 if R.random() < 0.5 else 0
    rem = max(0, target - pinned[p["id"]])
    if p["size"] in ("town", "large", "great") and rem < 1: rem = 1
    rows.append({"id": p["id"], "name": p["name"], "size": p["size"], "region": p["region"], "territory": p["territory"],
                 "target": target, "world_pass": pinned[p["id"]], "location_pass": rem})
# batch by region, balancing totals
byreg = collections.defaultdict(list)
for r in rows: byreg[r["region"]].append(r)
regs = sorted(byreg, key=lambda k: -sum(x["location_pass"] for x in byreg[k]))
batches = [[] for _ in range(nb)]; tot = [0] * nb
for k in regs:
    i = tot.index(min(tot)); batches[i] += byreg[k]; tot[i] += sum(x["location_pass"] for x in byreg[k])
print(json.dumps({"rows": rows, "total_location_pass": sum(r["location_pass"] for r in rows),
                  "batches": [{"places": b, "cards": t, "regions": sorted({x['region'] for x in b})} for b, t in zip(batches, tot)]}, indent=1, ensure_ascii=False))
