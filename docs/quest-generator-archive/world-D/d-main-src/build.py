"""Assemble d-main.json and d-main-log.md. Run: python3 build.py"""
import json, os, re, collections, math
from cards_side import *
import rolls as RO
import people as PEO

OUTJ = os.path.join(WD, "D", "d-main.json"); OUTL = os.path.join(WD, "D", "d-main-log.md")

# ---------------------------------------------------------------- the story object
CAST = {"heart": H, "fallback": B, "antagonist": LT, "hand": HD, "lieutenant": LI}
LAYERS = [
    {"layer": 1, "role": "Hook", "scope": "community", "node ids": ["n-d-main-1"], "move": "m-d-main-1"},
    {"layer": 2, "role": "Threshold", "scope": "community", "node ids": ["n-d-main-2a", "n-d-main-2b"], "move": "m-d-main-2"},
    {"layer": 3, "role": "Trials", "scope": "region", "node ids": ["n-d-main-3a", "n-d-main-3b"], "move": "m-d-main-3"},
    {"layer": 4, "role": "Trials (revelation)", "scope": "region", "node ids": ["n-d-main-4a", "n-d-main-4b"], "move": "m-d-main-4"},
    {"layer": 5, "role": "Ordeal", "scope": "realm", "node ids": ["n-d-main-5a", "n-d-main-5b"], "move": "m-d-main-5"},
    {"layer": 6, "role": "Climax", "scope": "realm", "node ids": ["n-d-main-6"], "move": "m-d-main-6"},
    {"layer": 7, "role": "Return", "scope": "community", "node ids": ["n-d-main-ret"], "move": None},
]
CH_ORDER = ["ch-d-main-hook", "ch-d-main-2a", "ch-d-main-2b", "ch-d-main-3a", "ch-d-main-3b", "ch-d-main-4a", "ch-d-main-4b",
            "ch-d-main-5a", "ch-d-main-5b", "ch-d-main-6", "ch-d-main-ret"]
chapters = [dict(id=k, **CH[k]) for k in CH_ORDER]
nodes = [{"id": ch["node"], "chapter": ch["id"], "role": ch["role"], "region": ch["region"], "settlement": ch["settlement"]} for ch in chapters]
edges = []
for a, b in zip(LAYERS, LAYERS[1:]):
    for x in a["node ids"]:
        for y in b["node ids"]:
            edges.append({"from": x, "to": y, "thread": "the next move lands" if b["move"] else "home"})
MOVES = [
    {"id": "m-d-main-1", "layer": 1, "fact": "x-d-main-m1", "scope": "community", "lever": "kin", "kind": "harm", "harm_type": "claim challenged (unfixable)",
     "faces": {"plain": "x-d-main-m1"}},
    {"id": "m-d-main-2", "layer": 2, "fact": "x-d-main-m2", "scope": "region", "lever": "debt", "kind": "harm",
     "faces": {"plain": "x-d-main-m2", "despite_you": DESPITE["m-d-main-2"]},
     "note": "the move's reach is region (the Crossing and Marches roads) though its chapters are community: fallout counted by reach"},
    {"id": "m-d-main-3", "layer": 3, "fact": "x-d-main-m3", "scope": "region", "lever": "force", "kind": "harm",
     "faces": {"plain": "x-d-main-m3", "despite_you": DESPITE["m-d-main-3"]}},
    {"id": "m-d-main-4", "layer": 4, "fact": "x-d-main-m4", "scope": "region", "lever": "bribery", "kind": "harm", "faces": {"plain": "x-d-main-m4"}},
    {"id": "m-d-main-5", "layer": 5, "fact": "x-d-main-m5", "scope": "realm", "lever": "law", "kind": "ordeal event", "event": "disgraced", "target": "heart",
     "faces": {"plain": "x-d-main-m5", "despite_you": DESPITE["m-d-main-5"]}},
    {"id": "m-d-main-6", "layer": 6, "fact": "x-d-main-m6", "scope": "realm", "lever": "force", "kind": "last move",
     "faces": {"plain": "x-d-main-m6", "despite_you": DESPITE["m-d-main-6"]}},
]
STORY = {
    "id": "d-main", "arc": "Rising", "shape": "Above", "L": 6, "W": 2, "cast": CAST, "stake": "pl-d-main-gate-mill",
    "want": "preservation", "heart_secret": None, "heart_secret_roll": "no (0.8191 vs 0.5)",
    "factions": {"antagonist_side": "f-throne", "ally": {"faction": "f-banner-lords", "person": GEN, "price": "the company's arms and the bondsmen's oaths go to Longwall",
                                                         "plant": "x-d-main-p-ally"},
                 "fault_line": {"person": DU, "virtue": "the emperor's lawful peace: the gate opens on his own hand, not his sister's seal",
                                "plant": "x-d-main-p-faultline"}},
    "revelation": {"fact": "x-d-main-revelation", "stored_on": HD, "published_by": ["c-d-main-15", "c-d-main-18"], "layer": 4},
    "moves": MOVES, "main_lever": "force", "reserved_levers": ["force"],
    "layers": LAYERS, "nodes": nodes, "edges": edges, "open_threads": [], "chapters": chapters, "return": "ch-d-main-ret",
    "doors": {"n-d-main-5a": "Disgraced → Passage (Unwelcome cargo), at c-d-main-21", "n-d-main-5b": "Disgraced → Dispute (Public offense), at c-d-main-24"},
    "return_variants": {
        "won": "Crownsgate is free of the company and the writ is torn up: the Return is the village learning to trust its miller again, with the lieutenant still holding what grants he can.",
        "let go": "The heart gave the Gate Mill to the crown for the writ: she comes home a free woman with no mill, and the Return is her name, not her stones.",
        "lost": "The company still holds Crownsgate and the writ stands: the Return is a homecoming under guard, the heart's name read out at every muster.",
    },
    "minors": ["c-d-main-minor-1", "c-d-main-minor-2", "c-d-main-minor-3"],
    "fallout": [c["id"] for c in CARDS if c.get("relatedness") == "fallout"],
}

# ---------------------------------------------------------------- context (locations §2)
ctx("x-d-main-m1", ["pl-crownsgate"])
ctx("x-d-main-m2", settlements_in("r-crossing", "r-marches"))
ctx("x-d-main-m3", settlements_in("r-crossing", "r-marches"))
ctx("x-d-main-m4", settlements_in("r-crossing", "r-roads"))
ctx("x-d-main-m5", all_settlements()); ctx("x-d-main-m6", all_settlements())
ctx("x-d-main-ctx-long-shed", ["pl-carters-rest"])
for c in CARDS:
    hs = home_settlement(c["place"]) or home_settlement(c["offered_at"])
    if c.get("source_move"):
        ctx(MOVEFACT[c["source_move"]], [hs] if MAPP[hs]["size"] != "presence" else [])
    for p in c.get("plants") or []:
        f = FACTS.get(p["fact"])
        if f and f["visibility"] == "public" and f.get("when") and not p["fact"].startswith("x-d-main-m") and MAPP[hs]["size"] != "presence":
            ctx(p["fact"], [hs])
    if c.get("truth") and FACTS[c["truth"]]["visibility"] == "public" and FACTS[c["truth"]].get("when") and MAPP[hs]["size"] != "presence":
        ctx(c["truth"], [hs])
# private plants (a knower's word, a document) stay out of context: mark them secret
for f in ["x-d-main-p-lead", "x-d-main-p-company", "x-d-main-p-ally", "x-d-main-p-captain-word", "x-d-main-p-deserter-hides", "x-d-main-p-founding",
          "x-d-main-p-chainpost", "x-d-main-p-cousin-hides", "x-d-main-p-pledge", "x-d-main-p-clearances", "x-d-main-p-factor-stall",
          "x-d-main-p-oath-name"]:
    FACTS[f]["visibility"] = "secret"
    for s in CONTEXT:
        if f in CONTEXT[s]: CONTEXT[s].remove(f)
TITLES = {
    "pl-crownsgate": [{"title": "keeper of the toll bridge", "holder": "p-d-main-bridgekeeper"}],
    "pl-longwall": [{"title": "warden of the outer towers", "holder": "p-d-main-towerwarden"},
                    {"title": "reeve of the wall-village", "holder": "p-d-main-wallreeve"}],
    "pl-southroad": [{"title": "toll-reeve of the Great Bridge", "holder": "p-d-main-tollreeve"},
                     {"title": "ladder-keeper", "holder": "p-d-main-ladderkeeper"}],
    "pl-oathford": [{"title": "head of the ford-wardens", "holder": "p-d-main-fordwarden"}],
    "pr-road-fort": [{"title": "Lantern sergeant of the road fort", "holder": "p-d-main-fortsergeant"}],
    "pl-breakwall": [{"title": "ladder-keeper", "holder": "p-d-main-f3l-keeper"}],
    "pl-carters-rest": [{"title": "keeper of the tavern-yard ladder", "holder": "p-d-main-f4l-keeper"}],
}
map_entries = []
for pid in sorted(set(CONTEXT) | set(TITLES)):
    e = {"id": pid}
    if CONTEXT.get(pid): e["context"] = list(CONTEXT[pid])
    if TITLES.get(pid): e["titles"] = TITLES[pid]
    map_entries.append(e)

# ---------------------------------------------------------------- card finishing: ids, endings, depends, shapes/plain present
for c in CARDS:
    c.setdefault("shapes", [])
    c["possible_endings"] = [e for e in c["possible_endings"]]
    c["chapter"] = c.get("chapter")
    for k in ("_confirming",): c.pop(k, None)
    if c["relatedness"] == "main":
        c["layer"] = next(l["layer"] for l in LAYERS if CH[c["chapter"]]["node"] in l["node ids"])
    c["reads_context"] = c.get("reads_context") or []
# published_by for the revelation
FACTS["x-d-main-revelation"]["published_by"] = ["c-d-main-15", "c-d-main-18"]

# drop people created for conditional slots that ended up off every card
used = {e for c in CARDS for v in c["slots"].values() for e in (v if isinstance(v, list) else [v]) if isinstance(e, str)}
for c in CARDS:
    for w in [c["ways_pool"][i] for i in c["ways_rolled"]]:
        pass
named_in_facts = set()
for f in FACTS.values():
    for pid in PEOPLE:
        if PEOPLE[pid]["name"] in f["text"]: named_in_facts.add(pid)
for pid in list(PEOPLE):
    if pid not in used and pid not in CAST.values() and pid not in named_in_facts:
        print("dropping unused person", pid); PEOPLE.pop(pid)

OUT = {"agent": "d-main", "registry": {"people": list(PEOPLE.values()), "places": list(PLACES.values()) + map_entries,
                                       "things": list(THINGS.values())},
       "facts": list(FACTS.values()), "story": STORY, "cards": CARDS, "registry_updates": UPDATES}
STORY["registry_updates"] = UPDATES

# ---------------------------------------------------------------- self-checks before writing
IDRE = re.compile(r"\b(?:x|p|pl|t|c|f|m)-[a-z0-9]+-[a-z0-9-]+\b")
probs = []
for c in CARDS:
    for k in ("pressure", "place_tie"):
        if IDRE.search(c.get(k) or ""): probs.append(f"{c['id']} {k} has an id")
    for i in c["ways_rolled"]:
        if IDRE.search(c["ways_pool"][i]["text"]): probs.append(f"{c['id']} way {i} has an id")
    for r, e in c["slots"].items():
        if isinstance(e, str) and e not in PEOPLE and e not in EXIST and e not in PLACES and e not in MAPP and e not in THINGS and e not in FACTS:
            probs.append(f"{c['id']} slot {r} -> {e} unknown")
    for f in c.get("depends_on") or []:
        if f not in FACTS and f not in {x["id"] for x in L0["facts"]}: probs.append(f"{c['id']} depends_on {f} unknown")
    for p in c.get("plants") or []:
        if p["fact"] not in FACTS: probs.append(f"{c['id']} plant {p['fact']} unknown")
prem = collections.Counter((c["family"], c["premise"]) for c in CARDS)
rows = collections.Counter(c["truth_row"] for c in CARDS if not c["plain"])
shp = collections.Counter(s for c in CARDS for s in c["shapes"])
print("premises >2:", {k: v for k, v in prem.items() if v > 2})
print("rows >3:", {k: v for k, v in rows.items() if v > 3}); print("shapes >4:", {k: v for k, v in shp.items() if v > 4})
for p in probs: print("PROBLEM", p)
json.dump(OUT, open(OUTJ, "w"), indent=1, ensure_ascii=False)
print("wrote", OUTJ, len(CARDS), "cards", len(PEOPLE), "people", len(PLACES), "places", len(THINGS), "things", len(FACTS), "facts")

# ---------------------------------------------------------------- the log
L = ["# d-main log: Test World D, layer 2 (the realm): main story, Return, minors, fallout", "",
     "Seed `random.Random(6601)` for the spine and plan (rolls.py, in order); keyed rolls `K[6601, …]` (casting §8) for people and ways.",
     "Rules v6, card schema v1.11. Build: `D/d-main-src/` (rolls.py → people.py → lib.py → registry.py → cards_main.py → cards_side.py → build.py).", "",
     "## Spine", "",
     f"- Arc **Rising**, shape **Above** (forced by Rising), **L=6 × W=2** (diamond 1-2-2-2-2-1 + Return). Antagonist {name(LT)} (Throne), want **preservation**.",
     f"- Heart **{name(H)}** (Middle miller of the Gate Mill, Crownsgate; earlier-wave Qotiro); fallback {name(B)}; stake **the Gate Mill**; no heart secret (rolled no).",
     f"- Hand {name(HD)} (crown clerk of the sealing office); lieutenant {name(LI)}; ally {name(GEN)} (price: the company's arms and bondsmen's oaths); fault line {name(DU)}.",
     "- Moves: m1 kin (the will) · m2 debt (the sealing tenth) · m3 force (company in Longwall's towers) · m4 bribery (the Great Bridge re-granted, the house's advance) · "
     "m5 law (the writ: disgraced) · m6 force (Crownsgate seized; the company named the emperor's guard). Reserved lever: force.",
     "- Revelation (layer 4, published by c-d-main-15 and c-d-main-18): Litiṭaʻa seals in the emperor's name because his hands can no longer hold the seal, to raise a company that holds Highcrossing when he dies.", "",
     "## Rolls (seeded, in order)", ""]
for x in RO.LOG:
    s = f"- {x['roll']}: **{x.get('pick')}**"
    if "r" in x: s += f" (r={x['r']}" + (f", p={x['p']}" if "p" in x else "") + (f", of {x['options']}" if "options" in x and len(x['options']) < 9 else "") + ")"
    if x.get("note"): s += f" — {x['note']}"
    L.append(s)
L += ["", "## Way rolls (keyed K[6601, card, 'ways'])", ""]
for cid, n_, steps, out in WAYLOG:
    L.append(f"- {cid}: target {n_}; " + ", ".join(f"{s} {i}" if not isinstance(i, list) else f"{s} {i}" for s, i in steps) + f" → rolled {out}")
L += ["", "## People (keyed K[6601, id, 'make', …]: people, age, manner, habit; names from the naming system, stems unique)", ""]
for pid, g in PEO.OUT.items():
    if pid in PEOPLE: L.append(f"- {pid}: {g['name']} ({g['people']}, {g['age']}, {g['voice']['manner']}/{g['voice']['habit']})")
L += ["", "## Cards", ""]
for c in CARDS:
    tw = c["twist"]["name"] if c.get("twist") else "none"
    ways = "; ".join(c["ways_pool"][i]["kind"] for i in c["ways_rolled"])
    where = home_settlement(c["place"])
    tag = c["relatedness"] + (f" L{c.get('layer')} {c.get('position')}" if c["relatedness"] == "main" else
                              f" → {c.get('attached_to')}" if c["relatedness"] == "minor" else f" {c.get('source_move')} {c.get('face')}" + (f" checkpoint {c['checkpoint']}" if c.get('checkpoint') else ""))
    L.append(f"- **{c['id']}** ({tag}) {c['family']}/{c['premise']} at {pname(c['place']) if c['place'] in PLACES else MAPP[c['place']]['name']} ({MAPP[where]['name']}): "
             f"truth *{c['truth_row']}*, twist {tw}, ways [{ways}]{' — plain' if c['plain'] else ''}.")
how = collections.Counter(v["how"] for c in CARDS for v in c["cast_log"].values())
L += ["", f"Cast how: {dict(how)}; reuse rate among reused+created: {how['reused'] / max(1, how['reused'] + how['created']):.0%}.", ""]
L += ["## Rule problems (doc and section)", "",
      "1. **main-quest §6 (Rising journey) + locations §3:** each layer's chapters must open farther than *near* from the last turn's place and within ~3 days; on map D the Crossing is so tight that layer 5 and the climax had one legal opening each (Oathford ~3.06 days). The two constraints together nearly force geography.",
      "2. **main-quest §6, landmark standing:** community landmarks are 'Middle' exactly, but most villages' best site is Low or High; exact match had no solution, so I read it as a floor.",
      "3. **main-quest §8 (heart's secret):** no odds are given for whether the heart has a secret; I rolled 50%.",
      "4. **family-rescue §6 (Justified captor) vs rulebook (truth reachable):** on Imprisoned unjustly the twist removes the prove ways, which are the only ways that reach a *guilty* truth (c-d-main-5). I kept a crowd way and marked it truth-reaching; the family needs a non-prove way that can surface guilt.",
      "5. **family-rescue (Kidnapped, why taken: as leverage) + assembly §2 (no way harms the heart):** 'give them what they took him for' would hand over the heart, so it is removed (c-d-main-20). The family should say what replaces it.",
      "6. **fallout §3 rule 8 + family-investigate:** a bystander culprit fits Theft and Suspicious death but not Disappearance (needs a link) or Sabotage (needs a rival or grudge), so fallout Investigate has two premises.",
      "7. **rulebook §4 once-per-family vs main-quest (ally planted on a Reckoning card before a Reckoning climax):** the crisis exception covers faction stories only; the general's two Reckoning uses (c-d-main-8, c-d-main-27) are legal only because c-d-main-8 uses him in an authority slot.",
      "8. **world_check vs rulebook (World ledger):** the checker sets caps from the cards it sees (N≈48 here: premise 2, row 3, shape 4), not the planned N=220 (premise 9, row 6, shape 8). I held to the tighter caps.",
      "9. **main-quest §9 (Ordeal is realm) vs door cards:** c-d-main-19 (Estrangement) is personal by nature; I made it realm through the writ's proof (mother and son's quarrel is what the writ cites).",
      "10. **locations §2 (Settlement context) vs layer 0:** the Long Shed court state from layer 0 has no `when`, so it can't sit in a context list; I restated it as x-d-main-ctx-long-shed with `when`.",
      "11. **casting (fallback in the heart's place):** whether the fallback counts as cast when he isn't acting for the heart is unstated; I treated him as cast everywhere (he's in story.cast).",
      "12. **main-quest §5 (move scope):** a move's scope is not said to follow its layer; m2 reaches two regions from community chapters, so I set its scope by reach (region) and counted fallout 2 + late.",
      "13. **No blank card / plain:** whether main non-turn cards may be plain is unstated; c-d-main-13 and c-d-main-14 rolled confirming truths and no twist in the same chapter (both plain).",
      "14. **fallout §3 rule 4 + family-reckoning (Rank, answers-to):** on fallout Ruin from above the one the wrongdoer answers to holds a main card, so the family-rule way can't be offered (c-d-main-fall-4a).",
      "15. **locations §6:** `checkpoint: k` has no numbering; I used 1 for the threshold and trials-3 moves, 2 for trials-4 and the ordeal, 3 for the last move.",
      "16. **world_check reuse (one person, two slots) vs the families:** requester + party is allowed, but requester + estranged party (Restore, Estrangement) or requester + cargo (Passage, Fugitive) is flagged, though the families' natural requester is that person. I moved the requester to someone else on three cards (c-d-main-19, -fall-3a, -fall-4b).",
      "17. **main-quest §9 (Return reuses shown people):** no shown person can be the bridge-keeper's son who comes home in c-d-main-28, so the cargo (and the toll-bridge site and river road) are created in the Return; the requester and the one left behind are shown people.",
      "", "## Story notes", "",
      "- The will (m1) and the seal-proofs (c-d-main-2) carry the same new seal: the lead plant, paid at the Long Shed's strongbox (c-d-main-6).",
      "- Layer 3 is the lieutenant's: he holds the commission (c-d-main-9) and hunts the deserter (c-d-main-12); he returns as captor at Crownsgate (c-d-main-26), behind the notary (c-d-main-23) and as the Return's profiteer.",
      "- The revelation is published twice, once per path: by Shahayuh at the Great Bridge (his own grant bears the emperor's true hand) and by the cousin at the Scales Hall.",
      "- The climax rolled 4 ways by overflow: the forced ally and sacrifice ways plus the fault line and revelation ways.",
      "- Minors: the veterans' ditch (Longwall), the tenth tally under Kotqot's seal (Breakwall), the ford-wardens' handshake (Oathford). Advantages land on the climax and on 4a.",
      "- Fallout: 16 cards (m1 1; m2–m6 2 each + 1 late), faces profit/resist/endure/enemy's own/mirror all present (enemy's own: the company quartermaster in the guard-yard dispute); mirror is the Longwall seal-cutter named in the same writ (Dispute, Contested inheritance)."]
open(OUTL, "w").write("\n".join(L) + "\n")
print("wrote", OUTL)
