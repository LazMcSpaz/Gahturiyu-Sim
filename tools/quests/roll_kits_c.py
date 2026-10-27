#!/usr/bin/env python3
"""Roll the five guild story kits for one world (guild-kits.md v5; faction-stories.md v5.2 §9-§12).
Usage: python3 roll_kits_c.py <seed> <paired|unpaired|auto> [ledger.json] > rolls.json
Ledger: {"crisis_premises": [["Reckoning","Betrayal"], ...], "levers": ["debt", ...], "main_lever": "debt"}"""
import json, random, sys
from collections import Counter

# ---- data tables -----------------------------------------------------------------------------
# enemy: shape, revelation kind, leader is the enemy, enemy's side (faction-stories §8: a faction is
# the enemy's side in at most one story; Within = the guild itself), paired-only
E = lambda shape, kind, leader=0, side=None, paired=0: dict(shape=shape, kind=kind, leader=leader, side=side, paired=paired)
# crisis: allowed enemies, family, premise, givers by variant (a, b, c), last move kills someone, paired-only
C = lambda en, fam, prem, givers, kills=0, paired=0: dict(enemies=set(en.split()), family=fam, premise=prem,
                                                         givers=givers, kills=kills, paired=paired)
TRUCE = "truce"  # the shared paired last move (one move, one death on each side; counted once)
K = {
 "writers": dict(leaders={"the leader"},
  enemies={"W1": E("Within", "the cost", side="Writers"), "W2": E("Above", "hidden hand", side="Free Lords"),
           "W3": E("Beside", "hidden hand", side="Night Market"), "W4": E("Within", "hidden hand", side="Writers"),
           "W5": E("Above", "the old debt", side="Holy House"), "W6": E("Within", "the cost", 1, "Writers")},
  stakes={"S1": "place", "S2": "thing", "S3": "reputation", "S4": "place", "S5": "thing"},
  stake_ok=lambda e, s: (s != "S5" or e == "W2") and (e != "W2" or s in ("S5", "S2")),
  hearts={"H1": "mentor", "H2": "fellow recruit", "H3": "client", "H4": "veteran"},
  odd={"O1": "Acquire/Copy it before it burns", "O2": "Investigate/The ledger doesn't add up", "O3": "Passage/Smuggling"},
  crises={"C1": C("W1 W4 W6", "Reckoning", "Betrayal", ["the eldest copyist", "the reading room's keeper", "the heart's fallback"]),
          "C2": C("W2 W5", "Acquire", "Copy it before it burns", ["the eldest copyist", "a carter whose letters hang on the Wall", "the heart"]),
          "C3": C("W1", "Reckoning", "Schism", ["the eldest copyist", "a survivor from the village", "the heart's kin"]),
          "C4": C("W3", "Reckoning", "Swindle", ["the eldest copyist", "the heart", "the heart's fallback"])},
  endings=["open the ledgers", "keep the balance", "burn your copy"], seat=("W6", "take the road", None)),
 "lantern": dict(leaders={"the quartermaster"},
  enemies={"L1": E("Above", "the old debt", side="Free Lords"), "L2": E("Within", "the old debt", side="Lantern Company"),
           "L3": E("Within", "hidden hand", side="Lantern Company"), "L4": E("Beside", "hidden hand"),
           "L5": E("Above", "the cost", side="the Throne")},
  stakes={"S1": "place", "S2": "thing", "S3": "reputation", "S4": "thing", "S5": "place"},
  stake_ok=lambda e, s: (s != "S4" or e == "L1") and (e != "L1" or s in ("S4", "S3")),
  hearts={"H1": "mentor", "H2": "fellow recruit", "H3": "client", "H4": "member's kin"},
  odd={"O1": "Passage/The contract's other side", "O2": "Rescue/Kidnapped", "O3": "Investigate/Suspicious death"},
  crises={"C1": C("L1 L5 L3", "Rescue", "Imprisoned unjustly", ["the quartermaster", "the refusing captain's second", "the heart"]),
          "C2": C("L2 L3", "Reckoning", "Betrayal", ["the quartermaster", "the one survivor", "the heart"], kills=1),
          "C3": C("L4", "Reckoning", "Swindle", ["the quartermaster", "the heart", "the client who switched"]),
          "C4": C("L3", "Reckoning", "Schism", ["the quartermaster", "the heart's chapter captain", "a client caught between"])},
  endings=["honor it and march", "put out the lamp", "split the chapters", "pick a side"], seat=None),
 "old-hands": dict(leaders={"the old don"},
  enemies={"OH1": E("Within", "the old debt", 1, "Old Hands"), "OH2": E("Beside", "hidden hand", side="Night Market", paired=1),
           "OH3": E("Above", "the old debt", side="Seated Houses"), "OH4": E("Within", "the cost", side="Old Hands"),
           "OH5": E("Above", "the old debt", side="True Line")},
  stakes={"S1": "place", "S2": "thing", "S3": "reputation", "S4": "place", "S5": "place"},
  stake_ok=lambda e, s: (s != "S2" or e == "OH1") and (e != "OH1" or s in ("S2", "S3")),
  hearts={"H1": "mentor", "H2": "veteran", "H3": "member's kin", "H4": "client"},
  odd={"O1": "Passage/The guest who isn't allowed", "O2": "Reckoning/The favor called in", "O3": "Acquire/Passed on"},
  crises={"C1": C("OH1 OH4", "Reckoning", "Betrayal", [{"OH4": "the old don", "OH1": "the eldest steward"}, "a house steward", "the heart"]),
          "C2": C("OH2", "Reckoning", "Blood debt", ["the old don", "the heart", "the dead man's kin"], TRUCE, 1),
          "C3": C("OH3 OH5", "Reckoning", "Ruin from above", ["the old don", "the heart's fallback", "the steward who brought you in"]),
          "C4": C("OH1", "Acquire", "Won't sell", ["the eldest steward", "the forger's daughter", "the heart's fallback"]),
          "C5": C("OH2", "Passage", "Fugitive", ["the old don", "the heart", "the eldest steward"], TRUCE, 1)},
  endings=["make the houses need us again", "open the gate", "give back the pages"], seat=("OH1", "take the gate", None)),
 "night-market": dict(leaders={"the leader"},
  enemies={"N1": E("Beside", "hidden hand", side="Old Hands", paired=1), "N2": E("Within", "the cost", side="Night Market"),
           "N3": E("Above", "hidden hand", side="Sharp Hands"), "N4": E("Above", "the old debt"),
           "N5": E("Within", "the old debt", 1, "Night Market")},
  stakes={"S1": "place", "S2": "reputation", "S3": "thing", "S4": "place"},
  stake_ok=lambda e, s: True,
  hearts={"H1": "mentor", "H2": "fellow recruit", "H3": "client", "H4": "member's kin"},
  odd={"O1": "Acquire/Passed on", "O2": "Passage/The market moves", "O3": "Rescue/Imprisoned unjustly"},
  crises={"C1": C("N1", "Reckoning", "Blood debt", ["the leader", "the heart", "the dead man's kin"], TRUCE, 1),
          "C2": C("N4 N1", "Passage", "The market moves", ["the leader", "the eldest fence", "the heart"]),
          "C3": C("N2 N5", "Reckoning", "Betrayal", ["the eldest fence", "a crew boss", "the heart"]),
          "C4": C("N3", "Reckoning", "Ruin from above", ["the leader", "a Far Sails captain", "the heart"])},
  endings=["pull the big one", "sell to the Sharp Hands", "go straight", "put it on the water"],
  seat=("N5", "take the shop", "sell to the Sharp Hands")),
 "dueling": dict(leaders={"the showman", "the sister"},
  enemies={"D1": E("Beside", "hidden hand"), "D2": E("Above", "the cost", side="Open Council"),
           "D3": E("Within", "hidden hand", 1, "Dueling Society"), "D4": E("Beside", "hidden hand", side="Night Market"),
           "D5": E("Beside", "the old debt")},
  stakes={"S1": "place", "S2": "thing", "S3": "reputation", "S4": "thing"},
  stake_ok=lambda e, s: not (e == "D5" and s == "S2"),
  hearts={"H1": "mentor", "H2": "fellow recruit", "H3": "veteran", "H4": "member's kin"},
  odd={"O1": "Contest/The fix", "O2": "Investigate/Sabotage", "O3": "Contest/The ladder"},
  crises={"C1": C("D1 D4", "Contest", "First blood or last", ["the sister", "the dead fighter's trainer", "the heart"], kills=1),
          "C2": C("D2 D4", "Reckoning", {"D2": "Ruin from above", "D4": "Swindle"}, ["the sister", "the showman", "the heart"]),
          "C3": C("D3", "Reckoning", "Betrayal", ["the sister", "the heart's fallback", "the heart"]),
          "C4": C("D5", "Contest", "First blood or last", ["the showman", "the heart", "a ring-keeper"])},
  endings=["keep it sport", "teach the deadly half", "back to the tavern yards", "let the showman run the show"],
  seat=("D3", "take the ring", "let the showman run the show")),
}
# cast-step notes the roll hands on (guild-kits compatibility; faction-stories §5)
NOTES = {("writers", "C1"): "create an employer link enemy->heart if missing", ("lantern", "C2"): "employer link, created at cast if needed",
         ("dueling", "C3"): "employer link showman->heart, created at cast if needed", ("old-hands", "C1"): "employer link",
         ("night-market", "C3"): "employer link", ("writers", "C3"): "Schism: plant a fault line; the leader judges by letter",
         ("lantern", "C4"): "Schism: plant a fault line; the quartermaster judges"}
EVENTS = ["captive", "disgraced", "estranged"]
GUILDS = ["old-hands", "night-market", "writers", "lantern", "dueling"]
V = "abc"

# ---- §9 limits; each reason names the pool that broke it (§9: reroll only that pool) ----------
def broken(world, ledger):
    w = list(world.values()); bad = []
    n = lambda f: sum(1 for p in w if f(p))
    if n(lambda p: p["leader_enemy"]) > 1: bad.append(("enemy", "leader as enemy >1"))
    if n(lambda p: p["shape"] == "Within") > 2: bad.append(("enemy", "Within >2"))
    if n(lambda p: p["revelation_kind"] == "hidden hand") > 2: bad.append(("enemy", "hidden hand >2"))
    if any(v > 1 for k, v in Counter(p["side"] for p in w if p["side"]).items()): bad.append(("enemy", "faction is enemy side twice"))
    if n(lambda p: p.get("stake_kind") == "reputation") > 2: bad.append(("stake", "reputation stake >2"))
    if any(v > 2 for v in Counter(p["heart_type"] for p in w if "heart_type" in p).values()): bad.append(("heart", "heart type >2"))
    if any(v > 1 for v in Counter(p["odd_job_premise"] for p in w if "odd_job_premise" in p).values()):
        bad.append(("odd_job", "odd-job premise twice in one phase position"))
    cr = [p for p in w if "crisis_family" in p]
    allc = [(p["crisis_family"], p["crisis_premise"]) for p in cr] + ledger
    if sum(f == "Reckoning" for f, _ in allc) > 3: bad.append(("crisis", "Reckoning crises >3 (with ledger)"))
    if any(v > 2 for v in Counter(allc).values()): bad.append(("crisis", "crisis family+premise >2 (with ledger)"))
    prem = [p["odd_job_premise"] for p in w if "odd_job_premise" in p] + [f + "/" + q for f, q in allc[:len(cr)]]
    if any(v > 2 for v in Counter(prem).values()): bad.append(("crisis", "family+premise >2 across guild stories"))
    if len({p["kill"] for p in cr if p["kill"]}) > 2: bad.append(("crisis", "last moves that kill >2 (the truce is one move)"))
    if n(lambda p: p.get("giver_is_leader")) > 2: bad.append(("crisis variant", "crises given by own leader >2"))
    ev = Counter(p["ordeal_event"] for p in w if "ordeal_event" in p)
    if ev["captive"] > 1 or any(v > 2 for v in ev.values()): bad.append(("event", "chapter-2 event limit"))
    return bad

def run(seed, mode, ledger_doc):
    R = random.Random(seed); log = []
    ledger = [tuple(x) for x in ledger_doc.get("crisis_premises", [])]
    paired = {"paired": True, "unpaired": False, "auto": R.random() < 0.5}[mode]
    log.append(f"seed {seed}; syndicate pairing: {'PAIRED' if paired else 'unpaired'}"
               + (f"; ledger: {len(ledger)} earlier crisis premises" if ledger else ""))
    lev = ledger_doc.get("levers", []); main = ledger_doc.get("main_lever")
    log.append("levers: kit entries carry no lever (levers are rolled with the moves, §12 step 4), so the kit roll can't be "
               "constrained by them; budget handed on to the moves pass: " + (
               f"main lever '{main}' at most once across all guild stories (§9; rulebook §9a: once in the world, ledger already "
               f"shows {lev.count(main)}); " if main else "no main lever in ledger; ") +
               "each lever at most 3 across guild moves" + (f" (ledger counts: {dict(Counter(lev))})" if lev else ""))
    log.append("not rolled at kit time, so not checked here: reveal family (<=2 per family), twists (<=2 each), "
               "move levers, phase-3 premise positions")

    def pick(g, pool, cands, test):  # roll, and on a broken limit reroll this pool only
        cands = list(cands)
        while cands:
            c = R.choice(cands); cands.remove(c); bad = test(c)
            if not bad: return c
            log.append(f"  {g}: reroll {pool} {c} ({'; '.join(bad)})")
        return None

    def try_world():
        world = {}
        blood = R.choice(["old-hands", "night-market"]) if paired else None
        truce = R.choice(V) if paired else None
        if paired: log.append(f"  pair: {blood} takes Blood debt; shared truce variant {truce}")
        # §12 sets no guild order; a fixed one starves the last guild of capped pools (D3 never rolled), so shuffle,
        # with the syndicate pair first when paired (their crises are forced, §10)
        order = GUILDS[2:] if paired else GUILDS[:]; R.shuffle(order)
        if paired: order = GUILDS[:2] + order
        log.append("  order: " + ", ".join(order))
        for g in order:
            k = K[g]
            def check(cand, pool):
                return [r for p, r in broken({**world, g: cand}, ledger) if p == pool]
            def build(e):  # enemy test: its own limits, then every later pool must fit
                en = k["enemies"][e]
                cand = dict(enemy=e, shape=en["shape"], revelation_kind=en["kind"], leader_enemy=bool(en["leader"]), side=en["side"])
                bad = check(cand, "enemy")
                if bad: return bad
                s = pick(g, "stake", [s for s in k["stakes"] if k["stake_ok"](e, s)],
                         lambda s: check({**cand, "stake_kind": k["stakes"][s]}, "stake"))
                if not s: return ["no stake fits"]
                cand.update(stake=s + R.choice(V), stake_kind=k["stakes"][s])
                h = pick(g, "heart", k["hearts"], lambda h: check({**cand, "heart_type": k["hearts"][h]}, "heart"))
                if not h: return ["no heart fits"]
                cand.update(heart=h + R.choice(V), heart_type=k["hearts"][h])
                o = pick(g, "odd job", k["odd"], lambda o: check({**cand, "odd_job_premise": k["odd"][o]}, "odd_job"))
                if not o: return ["no odd job fits"]
                cand.update(odd_job=o + R.choice(V), odd_job_premise=k["odd"][o])
                if paired and g == "old-hands": cs = ["C2" if blood == g else "C5"]
                elif paired and g == "night-market": cs = ["C1" if blood == g else "C2"]
                else: cs = [c for c, x in k["crises"].items() if e in x["enemies"] and not x["paired"]]
                def crisis(c):
                    x = k["crises"][c]; prem = x["premise"]; prem = prem[e] if isinstance(prem, dict) else prem
                    shared = paired and g in ("old-hands", "night-market")
                    base = dict(crisis_family=x["family"], crisis_premise=prem,
                                kill=TRUCE if shared else (g + c if x["kills"] else 0))
                    bad = check({**cand, **base}, "crisis")  # entry-level limits; the giver is the variant's
                    if bad: return bad
                    # paired: both stories use the truce letter, except Night Market C2, whose own letter is how the raid comes
                    vs = [truce] if shared and not (g == "night-market" and c == "C2") else list(V)
                    def variant(v):
                        gv = x["givers"][V.index(v)]; gv = gv[e] if isinstance(gv, dict) else gv
                        cand.update(base, crisis=c + v, crisis_giver=gv, giver_is_leader=gv in k["leaders"])
                        return check(cand, "crisis variant")
                    if not pick(g, f"{c} variant", vs, variant): return ["no variant fits"]
                    return []
                if not pick(g, "crisis", cs, crisis): return ["no crisis fits"]
                if paired and g in ("old-hands", "night-market"): cand["truce_variant"] = truce
                ev = pick(g, "chapter-2 event", EVENTS, lambda v: check({**cand, "ordeal_event": v}, "event"))
                if not ev: return ["no chapter-2 event fits"]
                cand["ordeal_event"] = ev
                end = list(k["endings"])
                if k["seat"] and k["seat"][0] == e:
                    if k["seat"][2]: end[end.index(k["seat"][2])] = k["seat"][1]
                    else: end.append(k["seat"][1])
                cand["faction_endings"] = end
                note = NOTES.get((g, cand["crisis"][:2]))
                if note: cand["cast_notes"] = note
                cand["enemy"] = e + R.choice(V)
                world[g] = cand
                return []
            ens = [x for x, en in k["enemies"].items() if bool(en["paired"]) == paired] if paired and g in ("old-hands", "night-market") \
                else [x for x, en in k["enemies"].items() if not en["paired"]]
            if not pick(g, "enemy", ens, build):
                log.append(f"  {g}: nothing fits -> restart world"); return None
        shapes = {p["shape"] for p in world.values()}
        if not {"Above", "Beside"} <= shapes:
            log.append("  world: no Above or no Beside enemy -> restart"); return None
        return world

    for restart in range(100):
        w = try_world()
        if w: break
        log.append(f"restart {restart + 1}")
    for p in (w or {}).values(): p["last_move_kills"] = bool(p.pop("kill")); p.pop("side"); p.pop("stake_kind")
    return {"seed": seed, "paired": paired, "rolls": w, "log": log}

if __name__ == "__main__":
    led = json.load(open(sys.argv[3])) if len(sys.argv) > 3 else {}
    print(json.dumps(run(int(sys.argv[1]), sys.argv[2], led), indent=1, ensure_ascii=False))
