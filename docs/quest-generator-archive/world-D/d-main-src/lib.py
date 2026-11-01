"""Helpers for the World D realm-layer build (prefix d-main)."""
import random, json, os, collections, re
import people as PEO

HERE = os.path.dirname(os.path.abspath(__file__)); WD = os.path.dirname(os.path.dirname(HERE))
MAP = json.load(open(os.path.join(WD, "map-D.json"))); CANON = json.load(open(os.path.join(WD, "canon-D.json")))
L0 = json.load(open(os.path.join(WD, "D", "d-l0.json")))
MAPP = {p["id"]: p for p in MAP["places"]}
EXIST = {p["id"]: p for p in CANON["people"]}
EXIST.update({p["id"]: p for p in L0["registry"]["people"]})
for r in ("p-niralea", "p-aloqi", "p-lio", "p-shihi", "p-yitolao", "p-hoqa"):
    EXIST.pop(r, None)   # replaced by generated fillers at layer 0

def K(*keys): return random.Random("6601|" + "|".join(map(str, keys)))
WAYLOG = []      # every way roll, for the log

# ---------------------------------------------------------------- registry
PEOPLE, PLACES, THINGS, FACTS, UPDATES = {}, {}, {}, {}, []
def name(pid):
    if pid in PEOPLE: return PEOPLE[pid]["name"]
    if pid in EXIST: return EXIST[pid]["name"]
    raise KeyError(pid)
def given(pid):
    n = name(pid)
    return n.split(",")[0].split(" ")[0]
def pname(pid):
    if pid in PLACES: return PLACES[pid]["name"]
    return MAPP[pid]["name"]
def person(pid, standing, traits, home, faction=None, links=None, facts=None, dead=False, dies_at=None, fallback=None, cls=None):
    g = PEO.OUT[pid]
    PEOPLE[pid] = {"id": pid, "name": g["name"], "standing": standing, "archetype": None, "class": cls, "traits": traits,
                   "links": links or [], "faction": faction, "facts": facts or [], "premade": False, "dead": dead, "home": home,
                   "dies_at": dies_at, "fallback": fallback, "people": g["people"], "age": g["age"], "voice": g["voice"]}
    return pid
def link(a, b, typ, as_a, as_b, vis="public", when="before the story", note=None):
    for x, y, asx in ((a, b, as_a), (b, a, as_b)):
        L = {"type": typ, "to": y, "as": asx, "visibility": vis, "when": when}
        if note: L["note"] = note
        if x in PEOPLE: PEOPLE[x]["links"].append(L)
        else: upd(x, add_links=[L])
def upd(entry, add_facts=None, add_links=None, set_=None, note=None):
    for u in UPDATES:
        if u["entry"] == entry:
            u["add_facts"] += add_facts or []; u["add_links"] += add_links or []
            if set_: u["set"].update(set_)
            return
    UPDATES.append({"entry": entry, "add_facts": add_facts or [], "add_links": add_links or [], "set": set_ or {}, "note": note or ""})
def place(pid, nm, kind, owner, access, standing, at, region=None, tags=None, facts=None):
    PLACES[pid] = {"id": pid, "name": nm, "kind": kind, "owner": owner, "access": access, "standing": standing,
                   "region": region or MAPP[at]["region"], "at": at, "tags": tags or [], "facts": facts or []}
    return pid
def thing(tid, nm, kind, owner, value, location, held_by, since):
    THINGS[tid] = {"id": tid, "name": nm, "kind": kind, "owner": owner, "value": value, "location": location,
                   "held_by": held_by, "since": since, "tags": [], "facts": []}
    return tid
def fact(fid, text, vis="public", kind="state", when=None, source=None, on=None):
    FACTS[fid] = {"id": fid, "text": text, "visibility": vis, "kind": kind, "source": source or "d-main"}
    if when: FACTS[fid]["when"] = when
    for e in (on or []):
        if e in PEOPLE: PEOPLE[e]["facts"].append(fid)
        elif e in PLACES: PLACES[e]["facts"].append(fid)
        elif e in THINGS: THINGS[e]["facts"].append(fid)
        else: upd(e, add_facts=[fid])
    return fid

# ---------------------------------------------------------------- context (locations §2, Settlement context)
CONTEXT = collections.defaultdict(list)
def ctx(fid, settlements):
    for s in settlements:
        if fid not in CONTEXT[s]: CONTEXT[s].append(fid)
def settlements_in(*regions):
    return [p["id"] for p in MAP["places"] if p["region"] in regions and p["size"] != "presence" and not p.get("secret")]
def all_settlements():
    return [p["id"] for p in MAP["places"] if p["size"] != "presence" and not p.get("secret")]
def home_settlement(pid):
    seen = 0
    while pid and pid not in MAPP and seen < 6:
        pid = PLACES[pid]["at"]; seen += 1
    return pid

# ---------------------------------------------------------------- ways (rulebook §2 rule 3, Roll order)
def W(text, kind, names=None, source="premise", requires=None, price=None, pays=None, forced=False, truth=False, contest=False):
    w = {"text": text, "kind": kind, "source": source, "requires": requires, "names_slot": names, "price": price, "pays": pays,
         "code_forbidden": False, "forced": forced, "reaches_truth": truth}
    if contest: w["contest"] = True
    return w
def roll_ways(cid, pool, turn=False, climax=False, n=None):
    """Forced ways first, then (if the card has a truth) one that reaches it, then plant-backed ways on turn and climax cards,
    then random draws, inside 2-3 ways and at least 2 kinds; a 4th only on the climax (forced ways overflow).
    Keyed roll K[6601, card, 'ways']."""
    r = K(cid, "ways")
    if n is None: n = r.choice([2, 3])
    out = [i for i, w in enumerate(pool) if w.get("forced")]
    steps = [("forced", list(out))]
    if not any(pool[i].get("reaches_truth") for i in out):
        cands = [i for i, w in enumerate(pool) if w.get("reaches_truth") and i not in out]
        # prefer one that also pays a plant (a way that does two jobs)
        two = [i for i in cands if pool[i].get("pays")]
        if two or cands:
            pick = r.choice(two or cands); out.append(pick); steps.append(("truth", pick))
    if turn or climax:
        pb = [i for i, w in enumerate(pool) if w.get("pays") and i not in out]
        r.shuffle(pb)
        for i in pb:
            if len(out) < max(n, 3) or (climax and len(out) < 4):
                out.append(i); steps.append(("plant-backed", i))
    rest = [i for i in range(len(pool)) if i not in out]
    r.shuffle(rest)
    while len(out) < n and rest:
        i = rest.pop(); out.append(i); steps.append(("random", i))
    kinds = {pool[i]["kind"] for i in out}
    if len(kinds) < 2:
        other = [i for i in range(len(pool)) if i not in out and pool[i]["kind"] not in kinds]
        if other:
            j = r.choice(other)
            # replace the last random draw if there is one, else add (still within 3)
            rnd = [i for (s, i) in steps if s == "random"]
            if rnd and len(out) >= 3: out.remove(rnd[-1]); steps.append(("drop for kinds", rnd[-1]))
            out.append(j); steps.append(("kinds", j))
    out = sorted(out)
    WAYLOG.append((cid, n, steps, out))
    return out

# ---------------------------------------------------------------- cards
CARDS = []
GROUP = {"unwilling captive": "They don't want it", "hidden motive": "Someone's hiding something", "justified captor": "The other side is right",
         "not alone": "Innocents pay", "caught in the middle": "Innocents pay", "hollow prize": "Not what it seems", "old wound": "You started it",
         "law on the wrong side": "The other side is right", "the wronged don't want it": "They don't want it", "changed": "Innocents pay",
         "hand not head": "Someone's hiding something", "disproportionate demand": "The other side is right",
         "not wanted at the end": "They don't want it", "pursuer has a point": "The other side is right", "the one left behind": "Innocents pay",
         "load-bearing": "Innocents pay", "wants to be rid of it": "Not what it seems", "meant for destruction": "Someone's hiding something",
         "rightful keeper": "The other side is right", "someone wants it broken": "Someone's hiding something", "it broke for a reason": "You started it",
         "can't go back": "They don't want it", "one person pays": "Innocents pay", "one of us caused it": "You started it",
         "divided community": "They don't want it", "passed along": "Someone's hiding something", "protector's price": "Innocents pay",
         "sympathetic culprit": "The other side is right", "wrong target": "Not what it seems", "the fix protects someone": "The other side is right",
         "the champion was promised something": "Innocents pay", "no crime": "Not what it seems", "the truth costs the requester": "They don't want it",
         "need along the way": "Innocents pay", "load bearing": "Innocents pay"}
TWIST_NAME = {"the wronged don't want it": "The wronged don't want it", "hand not head": "Hand, not head", "law on the wrong side": "Law on the wrong side",
              "the one left behind": "The one left behind", "someone wants it broken": "Someone wants it broken", "caught in the middle": "Caught in the middle",
              "unwilling captive": "Unwilling captive", "justified captor": "Justified captor", "hollow prize": "Hollow prize", "wrong target": "Wrong target",
              "one of us caused it": "One of us caused it", "old wound": "Old wound", "can't go back": "Can't go back", "it broke for a reason": "It broke for a reason",
              "divided community": "Divided community", "not alone": "Not alone", "sympathetic culprit": "Sympathetic culprit", "no crime": "No crime",
              "protector's price": "Protector's price", "passed along": "Passed along", "one person pays": "One person pays", "changed": "Changed",
              "disproportionate demand": "Disproportionate demand", "pursuer has a point": "Pursuer has a point", "not wanted at the end": "Not wanted at the end",
              "need along the way": "Need along the way", "load-bearing": "Load-bearing", "wants to be rid of it": "Wants to be rid of it",
              "meant for destruction": "Meant for destruction", "rightful keeper": "Rightful keeper", "hidden motive": "Hidden motive",
              "the fix protects someone": "The fix protects someone", "the champion was promised something": "The champion was promised something"}
def twist(t, detail=None, shape=None):
    if not t: return None
    d = {"name": TWIST_NAME.get(t, t), "group": GROUP[t]}
    if detail: d["detail"] = detail
    if shape: d["shape"] = shape
    return d
ENDINGS = {
 "Dispute": ["A wins", "B wins", "the stake is divided", "a deal is struck", "the stake is made worthless or abandoned",
             "a hidden defect is revealed, used, or kept quiet", "the player walks away unresolved"],
 "Reckoning": ["justice done formally", "wrongdoer ruined", "amends made", "wrongdoer killed or driven off", "requester talked down, or forgives",
               "player sides with the wrongdoer", "reckoning fails"],
 "Rescue": ["freed", "freed but harmed or late", "captive stays by choice", "exchanged for something", "moved elsewhere", "rescue fails"],
 "Acquire": ["obtained and handed over", "obtained and kept or redirected by the player", "destroyed", "left with the holder", "holder fairly compensated"],
 "Passage": ["delivered", "delivered late, damaged, or partly", "seized or lost", "kept or redirected by the player", "handed to the pursuer"],
 "Restore": ["restored as it was", "restored in a changed form", "replaced by a substitute", "left broken", "abandoned (people move on)"],
 "Threat": ["threat ended", "threat driven off elsewhere", "community protected, but the threat remains", "community relocated", "bargain struck", "community lost"],
 "Investigate": ["accuse before the arbiter", "confront privately", "blackmail", "cover it up", "let it go", "truth not found"],
 "Contest": ["won", "won, terms bent", "lost", "thrown (lost on purpose)", "settled before the contest", "called off", "fix exposed"],
}
def card(**c):
    c.setdefault("relatedness", "main"); c.setdefault("signature", False); c.setdefault("clock", None); c.setdefault("variant", None)
    c.setdefault("giver", None); c.setdefault("giver_faction", None); c.setdefault("fallbacks", {}); c.setdefault("motive", None)
    c.setdefault("factions", []); c.setdefault("open_threads", []); c.setdefault("ways_removed", []); c.setdefault("details", {})
    c.setdefault("reads_context", []); c.setdefault("plants", []); c.setdefault("depends_on", [])
    c.setdefault("possible_endings", list(ENDINGS[c["family"]]))
    tr = c.get("truth_row"); sh = c.get("shapes", [])
    c["plain"] = bool(c.get("_confirming")) and not c.get("twist")
    c.pop("_confirming", None)
    CARDS.append(c)
    return c
