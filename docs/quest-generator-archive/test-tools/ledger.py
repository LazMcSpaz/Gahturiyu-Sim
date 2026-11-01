#!/usr/bin/env python3
"""World ledger state (rulebook, *World ledger*, rules v6) for a merged world, plus reserved shares for parallel writers.

Usage: python3 ledger.py <world.json> <planned N> [<out.json> [<writers> ...]]
  e.g. python3 ledger.py wp.json 160 shares.json l0b l1b l2b l3b
Counts what the world already uses and how much of each cap is left. With writer names, splits what is left of every
cap that is nearly spent (left < 2 x writers) between them, round-robin, so parallel writers can't overrun a cap.
Keys that aren't nearly spent are open to everyone (shared: true)."""
import json, sys, math, collections, re
W = json.load(open(sys.argv[1])); N = int(sys.argv[2])
out = sys.argv[3] if len(sys.argv) > 3 else None; writers = sys.argv[4:]
caps = {"premise": max(2, round(0.04 * N)), "truth_row": max(3, math.ceil(N / 40)), "shape": max(4, math.ceil(N / 30)),
        "twist": None, "ordeal_event": 2, "way_opening": 3}
cnt = {k: collections.Counter() for k in caps}
for c in W["cards"]:
    cnt["premise"][f"{c['family']}: {c['premise']}"] += 1
    tr = c.get("truth_row") or (c.get("details") or {}).get("truth_row")
    if tr and not c.get("plain"): cnt["truth_row"][tr] += 1
    for s in c.get("shapes") or []: cnt["shape"][s] += 1
    t = c.get("twist"); tn = t.get("name") if isinstance(t, dict) else t
    if tn and tn != "none": cnt["twist"][tn] += 1
    for i, w in enumerate(c.get("ways_pool") or []):
        if i in (c.get("ways_rolled") or []) and w.get("text"):
            cnt["way_opening"][" ".join(re.findall(r"[a-z']+", w["text"].lower())[:4])] += 1
st = W.get("story") or {}
for m in st.get("moves", []):
    if m.get("event"): cnt["ordeal_event"][m["event"]] += 1
for fs in W.get("faction_stories", []):
    for m in fs.get("moves", []):
        if m.get("event"): cnt["ordeal_event"][m["event"]] += 1
levers = collections.Counter(m.get("lever") for m in st.get("moves", []) if m.get("lever"))
reserved = sorted({l for l, k in levers.items() if k >= 2} | ({st.get("main_lever")} if st.get("main_lever") else set()))
plain = sum(1 for c in W["cards"] if c.get("plain"))
side = sum(1 for c in W["cards"] if c.get("relatedness") in ("independent", "echo", "fallout"))
R = {"planned_N": N, "caps": caps, "cards_so_far": len(W["cards"]), "reserved_main_levers": reserved,
     "plain_side_cards": [plain, side], "used": {k: dict(v.most_common()) for k, v in cnt.items()}, "left": {}, "shares": {}}
for k, cap in caps.items():
    if cap is None: continue
    R["left"][k] = {key: cap - n for key, n in cnt[k].items()}
if writers:
    for k, left in R["left"].items():
        for key, n in left.items():
            if n < 2 * len(writers):
                share = {w: 0 for w in writers}
                for i in range(max(0, n)): share[writers[i % len(writers)]] += 1
                R["shares"].setdefault(k, {})[key] = share
print(f"N={N} caps: premise {caps['premise']}, truth row {caps['truth_row']}, shape {caps['shape']}; cards so far {len(W['cards'])}")
print("reserved main levers:", reserved, "| plain side cards", plain, "/", side)
for k in ("premise", "truth_row", "shape", "ordeal_event", "way_opening"):
    full = [key for key, n in R["left"].get(k, {}).items() if n <= 0]
    print(f"{k}: {len(cnt[k])} used, {len(full)} at cap" + (": " + "; ".join(full[:12]) if full else ""))
if out: json.dump(R, open(out, "w"), indent=1, ensure_ascii=False); print("wrote", out)
