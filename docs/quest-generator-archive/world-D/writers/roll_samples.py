"""Roll Writers sample stories from the authored spines (v8 placer, draft 5).

Draft 5 (review-v7d: coherence 3.0, the breaks cross-field contradictions
inside a spine): blows, endings and flavour are no longer kept here; they are
read from each spine's fields 9, 11 and 14 in spines.md by spine5.py, with the
shared cards of field 18. A spine has one heart, at most two blow branches
(each written whole), exactly three endings (plus W6's seat ending, all
offered; only the shown one is chosen) and description-only flavour. The
static checks add the pairwise ones (spine5.check: an ending against the blow
dealt, a branch's marks against everything outside it, standing against the
blow, every proof against Held, an odd job's object and what is told,
the want against the tell, a person lost comes back, one sentence per line);
the per-story checks add faction leaders only where a spine asks
(lord_ok), a seat's map size (size_ok), a captor's lordship against the
crisis, and a Wayhouse crisis's clock from the dealt door.

      python3 roll_samples.py --render-all   (every branch x ending of every
                                              spine, to proof/<spine>.md)

Draft 4 (review-v7c): slots carry ownership as well as distance (a lord's
land is the lordship of his seat: the nearest lord's seat in the same
territory; towers keep out of hostile enclaves; W1's two lords share a major
faction; a market woman's town has a market); a clock sums every leg its
crisis needs (W2-B's summons rides out before she can run); the reveal's
giver, its closing form and chapter 1's shape are spine data, kept in the
set's memory with caps; the stories that deal a card at the Wayhouse are
capped (Highcrossing was in five of eight); the lost-surveyor motif, not the
H5 type, is the once-a-set heart; and flavour options are checked for place
names, season words and unglossed outside factions.

Reads `map-D.json` (places, coordinates: 1 unit = a day; roads) and
`D/d-l0.json` (every settlement's seat holder and titles, people, factions),
and applies the *Placing a spine* rules, the slot checks and *Fresh stories*
from `spines.md`.

One story per seed, each with its own random.Random(seed). The seeds of a set
are read as playthroughs in a row; Fresh stories keeps every earlier sample
of the set in memory: spine, enemy, lords and courts, main town, heart type,
stake, blow, chapter-opening premises, chapter-1 move, routine premises and
endings (the caps are the named constants below).

Per seed, in order:
  1. spine: from those memory allows, shuffled by the seed; a spine is taken
     only if the rest of the set can still be filled under the caps (a
     look-ahead over the enemies left), and only if the placer can fill it
  2. chapter-2 blow, among the spine's options, least used first
  3. every flavour list, each rolled on its own (each names the card that
     voices it)
  4. the heart's casting (sex, unless the spine fixes it)
  5. slots: lords and courts from d-l0's holders; places from the map by kind,
     owner and feature, weighted by nearness; a site the map lacks is added
     with its reason; day slots, the region word and `{house}` filled
  6. routine premises (a card's swaps, by the Writers' profile weights and
     the set's memory)
  7. the clock: `{days}` = the road from where the parties are to where the
     crisis falls, rounded up, plus one day
  8. 3 endings from the pool (W6's seat ending always a fourth), least used
     first, and the one shown (never one shown earlier in the set)
Then every check runs (placement bounds, the crisis where the story is, slot
kinds and owners, holders from d-l0, parties set up before they are used,
hidden-hand and shaming bars, flavour voiced, routine phase rules, chapter
families, the set's variety caps). Any break prints FAIL and the script exits 1.

Run:  python3 roll_samples.py                 (seeds 7401-7408)
      python3 roll_samples.py 7201 7208       (any seed range, one set)
      python3 roll_samples.py --sets 9001 20  (20 sets of 8 seeds from 9001;
                                               one summary line per set)
      python3 roll_samples.py --placing W1-A  (print a spine's field 17)
"""
import heapq
import json
import math
import os
import random
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MAP = json.load(open(os.path.join(HERE, "..", "..", "map-D.json")))
L0 = json.load(open(os.path.join(HERE, "..", "d-l0.json")))
SPINES_MD = os.path.join(HERE, "spines.md")

# --- the rules' numbers (spines.md, Placing a spine) -------------------------
BOUND = 1.25     # a card bound to a slot sits this near it
STRETCH = 1.6    # ...or this near, only when a needed feature is nowhere nearer
FREE = 3.0       # a card bound to no slot, from the centre
JOURNEY = 3.5    # a card the spine marks as a journey, from the centre
SEATS_APART = 2.8  # W1: two lordships adjoin when their seats are this near (draft 4: was 2.6,
LENDERS = 2.8      # W1: the lenders' hall is this near both seats      which left one triple once lords share a faction)
OFFROAD = 1.35     # the clock: a stretch off the roads costs this much more...
OFFROAD_REACH = 1.6  # ...and is taken only between places this near

# --- Fresh stories inside a set (spines.md, Fresh stories) -------------------
HEART_CAP = {"H6": 1}            # a heart type at most this often in a set (else 2)
HEART_GAP = 2                    # ...and never again within this many samples
MOTIF_CAP = {"lost surveyor": 1}  # draft 4: the lost surveyor's child (W5-A, W8-B) once a set
SHAPE_CAP = 4                    # draft 4: a chapter-1 shape (1, 2, 3) at most four times in a set
REVEAL_CAP = {"leader": 3}       # draft 4: the leader gives at most three reveals in a set (others: four)
FORM_CAP = 4                     # draft 4: a reveal's closing form at most four times in a set
HQ_CAP = 3                       # draft 4: stories that deal any card at the Wayhouse, at most three
SHAPES = {1: "the price at the turn", 2: "the ally first", 3: "the early turn"}
GIVERS = {"leader": "the leader", "heart": "the heart", "elder": "an elder copyist",
          "ally": "the ally", "letter": "a found letter"}
FORMS = {"side": "laid side by side", "chain": "a chain", "question": "the knower's question",
         "names": "the thing names them"}
STAKE_CAP = 3                    # a stake (S1-S7) at most three times, never twice running
BLOW_CAP = 2                     # a blow (target + event) at most twice, never twice running
OPEN_CAP = {"Passage · Unwelcome cargo": 1}  # a chapter opening's premise at most this often in its
                                              # position (chapter 1 or chapter 2; else 2); Unwelcome cargo once in all
MOVE_CAP = 3                     # a chapter-1 move (M1-M9) at most three times, never twice running
LEDGER = "Investigate · The ledger doesn't add up"
COPY = "Acquire · Copy it before it burns"
SIGNATURE = (LEDGER, COPY, "Passage · Samples from the green")
ROUTINE_CAP = {LEDGER: 3, COPY: 5}  # stories whose routine uses the premise (others: no cap)
WEIGHT = {"Investigate": 3, "Acquire": 3, "Dispute": 2, "Passage": 2, "Restore": 2,
          "Rescue": 1, "Threat": 1, "Reckoning": 1, "Contest": 0}  # the Writers' profile

# --- places -----------------------------------------------------------------
BASE = {}
for p in MAP["places"]:
    BASE[p["id"]] = dict(p)
for r in L0["registry"]["places"]:
    BASE[r["id"]]["holder"] = r["holder"]
    BASE[r["id"]]["titles"] = r.get("titles", [])
PL = {}


def reset_places():
    PL.clear()
    for k, v in BASE.items():
        PL[k] = dict(v)


reset_places()
PEOPLE = {p["id"]: p for p in L0["registry"]["people"]}
LEADERS = {pid for v in L0["faction_leaders"].values() for pid in v}
REGIONS = {r["id"]: r["name"] for r in MAP["regions"]}
ROADS = {frozenset(r) for r in MAP["roads"]}
ROADS |= {frozenset(p) for p in zip(MAP["walking_road"], MAP["walking_road"][1:])}
WAYHOUSE = "pl-highcrossing"
MOUNT = "pl-mount"
SETTLE = ("village", "town", "large", "great")


def name(pid):
    return PL[pid]["name"]


def xy(pid):
    return PL[pid]["x"], PL[pid]["y"]


def dist(a, b):
    (ax, ay), (bx, by) = xy(a), xy(b)
    return math.hypot(ax - bx, ay - by)


def road(a, b):
    """Days from a to b: along the roads at their length, off them at OFFROAD
    times the straight line, between places no more than OFFROAD_REACH apart."""
    if a == b:
        return 0.0
    nodes = [p for p in PL if not PL[p].get("secret")]
    best = {a: 0.0}
    q = [(0.0, a)]
    while q:
        d, u = heapq.heappop(q)
        if u == b:
            return d
        if d > best.get(u, 1e9):
            continue
        for v in nodes:
            if v == u:
                continue
            s = dist(u, v)
            if frozenset((u, v)) in ROADS:
                c = s
            elif s <= OFFROAD_REACH:
                c = s * OFFROAD
            else:
                continue
            if d + c < best.get(v, 1e9):
                best[v] = d + c
                heapq.heappush(q, (d + c, v))
    return dist(a, b) * OFFROAD


def days_between(a, b):
    return days_for(road(a, b))


def days_for(total):
    """{days}: the road's days, rounded up (from the two decimals the roller
    prints, so the printed sum and the clock agree), plus one day."""
    return max(2, math.ceil(round(total, 2) - 1e-9) + 1)


def pres(pid):
    return PL[pid].get("presences_in_town") or []


def has(pid, word):
    return any(word in s for s in pres(pid))


def settlements():
    return [p for p in PL if PL[p].get("size") in SETTLE and not PL[p].get("secret")
            and not p.startswith("new-")]


def halls(wayhouse=False):
    out = [p for p in PL if has(p, "Writers reading room")]
    return out + ([WAYHOUSE] if wayhouse else [])


def stalls():
    return [p for p in PL if has(p, "letter-writer's stall")]


def holder(pid):
    h = PL[pid].get("holder")
    return PEOPLE.get(h) if h else None


def lord_kind(pid):
    """What sort of court-holder a seat's d-l0 holder is (None if not one)."""
    h = holder(pid)
    if not h or h["standing"] not in ("High", "Top"):
        return None
    f, sub = h["faction"]["id"], h["faction"]["sub"]
    return {"f-banner-lords": "Banner Lord", "f-free-lords": "Free Lord",
            "f-seated-houses": "Seated House head", "f-tide-houses": "Tide House speaker",
            "f-open-council": "Open Council head", "f-throne": "the Throne",
            "f-true-line": "True Line head"}.get(sub) or \
        {"f-binders": "the Binders' head", "f-holy-house": "the Holy House"}.get(f)


LANDLORDS = ("Banner Lord", "Free Lord", "Seated House head")
COURTS = LANDLORDS + ("Tide House speaker", "the Binders' head", "Open Council head")


def seats(kinds):
    return [p for p in PL if lord_kind(p) in kinds]


SIZES = ("site", "village", "town", "large", "great")


def lord_ok(sid, slot, pid):
    """Draft 5 (review-v7d): a lord slot takes no faction leader unless its
    spine asks for one (`leaders_ok`), and never one whose canon the spine
    names as answering its scheme."""
    h = PL[pid].get("holder")
    if h not in LEADERS:
        return True
    ok = SPINES[sid].get("leaders_ok", {}).get(slot)
    return bool(ok) and h not in ok[1]


def size_ok(sid, slot, pid):
    need = SPINES[sid].get("seat_size", {}).get(slot)
    return not need or SIZES.index(PL[pid]["size"]) >= SIZES.index(need[0])


def who(pid):
    h = holder(pid)
    return f"{h['name']} ({lord_kind(pid) or h.get('class') or 'holder'}, d-l0 {h['id']})" if h else "no holder"


def rivals(a, b):
    ha, hb = PL[a].get("holder"), PL[b].get("holder")
    return any(l["type"] == "rival" and l["to"] == hb for l in PEOPLE[ha]["links"]) if ha and hb else False


def region_word(pid):
    return REGIONS.get(PL[pid].get("region"), "the region")


# --- ownership (draft 4: slots carry whose land, not only how far) ------------
HOSTILE = ("t-wave", "t-high", "t-green", "t-sails")  # enclaves nobody hides a hoard in


def base_places():
    return [p for p in PL if not p.startswith("new-") and not PL[p].get("secret")]


def territory_at(x, y):
    """Whose land a point is: the territory of the nearest map place."""
    return PL[min(base_places(), key=lambda p: math.hypot(PL[p]["x"] - x, PL[p]["y"] - y))]["territory"]


def lord_seats():
    return [p for p in base_places() if lord_kind(p)]


def lordship_at(x, y, terr=None):
    """Whose lordship a point is in: the nearest lord's seat in its territory."""
    terr = terr or territory_at(x, y)
    cands = [s for s in lord_seats() if PL[s]["territory"] == terr]
    return min(cands, key=lambda s: math.hypot(PL[s]["x"] - x, PL[s]["y"] - y)) if cands else None


def lordship_of(pid):
    return pid if lord_kind(pid) else lordship_at(PL[pid]["x"], PL[pid]["y"], PL[pid]["territory"])


def major(pid):
    """A seat holder's major faction (f-great-house, f-first-peoples, ...)."""
    return holder(pid)["faction"]["id"]


def market_town(pid):
    """A town with a market: no keep, fort, counting-house, enclave or camp."""
    k = PL[pid]["kind"]
    return PL[pid]["size"] in ("town", "large", "great") and not any(
        w in k for w in ("keep", "fort", "counting-house", "enclave", "camp", "hold", "shrine"))


# --- created sites ----------------------------------------------------------
class Created(Exception):
    pass


def nearest_settlement(x, y, skip=()):
    return min((p for p in settlements() if p not in skip),
               key=lambda p: math.hypot(PL[p]["x"] - x, PL[p]["y"] - y))


def make(slots, key, nm, kind, x, y, reason, territory=None):
    pid = f"new-{key}"
    near = nearest_settlement(x, y)
    PL[pid] = dict(id=pid, name=nm, kind=kind, size="site", x=round(x, 2), y=round(y, 2),
                   territory=territory or PL[near]["territory"], region=PL[near]["region"],
                   owner_faction=None, created=reason, presences_in_town=[])
    slots[key] = pid
    return pid


def toward(a, b, d):
    (ax, ay), (bx, by) = xy(a), xy(b)
    n = math.hypot(bx - ax, by - ay) or 1
    return ax + (bx - ax) / n * d, ay + (by - ay) / n * d


def away(a, b, d):
    (ax, ay), (bx, by) = xy(a), xy(b)
    n = math.hypot(ax - bx, ay - by) or 1
    return ax + (ax - bx) / n * d, ay + (ay - by) / n * d


def remote(a, d, rng, clear=0.7, terrs=None):
    """A point d from a, at least `clear` from every settlement (for towers);
    with `terrs`, only in those territories (draft 4: never in an enclave)."""
    for _ in range(144):
        t = rng.random() * 2 * math.pi
        x, y = xy(a)[0] + d * math.cos(t), xy(a)[1] + d * math.sin(t)
        if 0.2 < x < 11 and 0.2 < y < 8.8 and all(
                math.hypot(PL[p]["x"] - x, PL[p]["y"] - y) >= clear for p in settlements()) \
                and (terrs is None or territory_at(x, y) in terrs):
            return x, y
    raise Created("no remote ground in the land it must be in")


def inland(seat, d, rng):
    """A point d from a seat, inside its own territory (the nearest map place
    shares it) and its own lordship (draft 4: no other lord's seat is nearer
    in that territory), clear of other settlements; drawn among the three
    most inland of 36 bearings."""
    terr = PL[seat]["territory"]
    base = base_places()
    out = []
    for k in range(36):
        t = 2 * math.pi * k / 36
        x, y = PL[seat]["x"] + d * math.cos(t), PL[seat]["y"] + d * math.sin(t)
        if not (0.2 < x < 11 and 0.2 < y < 8.8):
            continue
        if PL[min(base, key=lambda p: math.hypot(PL[p]["x"] - x, PL[p]["y"] - y))]["territory"] != terr:
            continue
        if lord_kind(seat) and lordship_at(x, y, terr) != seat:
            continue
        if any(math.hypot(PL[p]["x"] - x, PL[p]["y"] - y) < 0.45 for p in settlements() if p != seat):
            continue
        out.append((min(math.hypot(PL[p]["x"] - x, PL[p]["y"] - y) for p in base
                        if PL[p]["territory"] != terr), x, y))
    if not out:
        raise Created("no inland ground")
    out.sort(reverse=True)
    _, x, y = rng.choice(out[:3])
    return x, y


def add_feature(pid, feature, reason):
    PL[pid].setdefault("added", []).append((feature, reason))


# --- picking ----------------------------------------------------------------
class NoFit(Exception):
    pass


def pick(rng, cands, anchor=None, maxd=None, why="", avoid=()):
    """Weighted by nearness to `anchor` (1 / (0.2 + d)^2); none beyond maxd.
    A town already holding another of the story's cards (`avoid`) weighs a
    quarter, so routine cards spread where the bounds allow."""
    cands = [c for c in dict.fromkeys(cands)]
    if anchor and maxd is not None:
        cands = [c for c in cands if dist(c, anchor) <= maxd + 1e-9]
    if not cands:
        raise NoFit(why or "no candidate")
    w = [(1.0 / (0.2 + dist(c, anchor)) ** 2 if anchor else 1.0) * (0.25 if c in avoid else 1.0)
         for c in cands]
    return rng.choices(cands, weights=w)[0]


def nearest(cands, anchor):
    return min(cands, key=lambda c: dist(c, anchor))


def house_near(rng, c, s, skip=()):
    """`{house}`: a registry house near the court (not a faction leader, not
    the court's own holder), else one made on demand."""
    lord = PL[c].get("holder")
    cand = [p for p in PEOPLE.values() if " of " in p["name"] and p["id"] not in LEADERS
            and p["id"] != lord and p["id"] not in skip and p["standing"] in ("High", "Middle")
            and p["home"] in PL and dist(p["home"], c) <= FREE]
    if not cand:
        return "a house of the court's rolls (made on demand)", None
    cand.sort(key=lambda p: (dist(p["home"], c), p["id"]))
    p = rng.choice(cand[:3])
    return "the house of " + p["name"].split(" of ", 1)[1] + f" ({p['name']}, d-l0 {p['id']}, of {name(p['home'])})", p["id"]


# --- the spines --------------------------------------------------------------
# Card tuple: (id, what, place-slot, anchors, mode, note)
#   mode: "bound" (within BOUND of each anchor; STRETCH where the slot says why)
#         "journey" (within JOURNEY of the centre; note says why)
#         "free" (within FREE of the centre)
#         "hq" (at the Wayhouse; note says why the slot people there are there)
# A door is ("DOOR", {blow: place-slot}, anchors, mode, note).
# The crisis card ("CR") is set where the story is (Laz, 27 Oct); only a spine
# whose `about` names the Wayhouse or the leader may set it at the Wayhouse.
B, J, F, HQ = "bound", "journey", "free", "hq"

SPINES = {}


def spine(sid, **kw):
    kw["id"] = sid
    kw.setdefault("alts", {})
    kw.setdefault("blows", {})      # draft 5: blows, endings and flavour are read from spines.md (load_d5)
    kw.setdefault("endings", [])
    kw.setdefault("flavour", {})
    kw.setdefault("days_slots", {})
    kw.setdefault("clock", None)
    kw.setdefault("about", None)
    kw.setdefault("parties", {})
    kw.setdefault("door_needs", {})
    kw.setdefault("ending_needs", {})
    kw.setdefault("owns", [])     # draft 4: (slot, test, other) ownership needs, checked per story
    kw.setdefault("motif", None)
    kw.setdefault("seasons", ())  # season words the spine's own text dates the story by
    kw["hq_forced"] = kw["centre"] == "wayhouse" or any(c[0] != "DOOR" and c[4] == HQ for c in kw["cards"])
    SPINES[sid] = kw


DAYS = {
    "truce_day": ["the spring quarter day", "the first day of the harvest fair", "the new moon after the sowing"],
    "hearing_day": ["the court's quarter sitting", "the court's harvest sitting", "the first market day of the month"],
    "feast": ["the feast of first fruits", "the feast of lamps", "the midwinter vigil"],
    "fair": ["the salt fair", "the wool fair", "the horse fair"],
    "vote_day": ["the month's end", "the quarter day", "the next new moon"],
}

# draft 4: season words. Flavour options carry none (they must hold wherever
# and whenever they are placed); a story's day names and its spine's own
# season words must fall in one season or two that follow each other.
SEASON_WORDS = [(r"before winter", "autumn"), (r"midwinter", "winter"), (r"midsummer", "summer"),
                (r"first fruits", "summer"),
                (r"\bspring (?:quarter|fair|day|sowing)|\b(?:in|this|last|next|by|before|of) spring\b", "spring"),
                (r"sowing", "spring"), (r"summer", "summer"), (r"harvest", "autumn"), (r"frost", "autumn"),
                (r"autumn", "autumn"), (r"winter", "winter"), (r"\bsnow\b", "winter")]
SEASON_RING = ["spring", "summer", "autumn", "winter"]


def seasons_in(text):
    """Season words in a text (a water spring is not a season)."""
    t, out = text.lower(), set()
    for w, s in SEASON_WORDS:
        if re.search(w, t):
            out.add(s)
            t = re.sub(w, "", t)
    return out


def seasons_fit(ss):
    ss = set(ss)
    if len(ss) <= 1:
        return True
    if len(ss) > 2:
        return False
    a, b = sorted(SEASON_RING.index(s) for s in ss)
    return b - a == 1 or (a, b) == (0, 3)


# W1-A -----------------------------------------------------------------------
def lenders_hall_ok(h, a):
    """W1's lenders' hall: a town of the lords' own major faction, or of no
    landholding faction (a Binders' or Open Hand market), never a rival's."""
    majors = {major(p) for p in seats(LANDLORDS)}
    of = PL[h]["owner_faction"]
    return of not in majors or of == major(a)


def fill_w1(rng, s, sid):
    triples = []
    for a in seats(LANDLORDS):
        for b in seats(LANDLORDS):
            if a >= b or dist(a, b) > SEATS_APART:
                continue
            if not (lord_ok(sid, "lord_a", a) and lord_ok(sid, "lord_b", b)):
                continue  # draft 5: no faction leader the spine hasn't asked for
            if major(a) != major(b):
                continue  # draft 4: two lords of one major faction, or it is a treaty, not a ford feud
            for h in halls():
                if h in (a, b) or dist(h, a) > LENDERS or dist(h, b) > LENDERS:
                    continue
                if not lenders_hall_ok(h, a):
                    continue  # draft 4: their lenders lodge in their own faction's town or a neutral market
                mx, my = (PL[a]["x"] + PL[b]["x"]) / 2, (PL[a]["y"] + PL[b]["y"]) / 2
                dm = math.hypot(PL[h]["x"] - mx, PL[h]["y"] - my)
                triples.append(((a, b, h), (3.0 if rivals(a, b) else 1.0) / (0.2 + dm) ** 2))
    triples = [t for t in triples if t[0][0] not in s["_lords"] and t[0][1] not in s["_lords"]
               and t[0][2] not in s["_towns"]]
    if not triples:
        raise NoFit("no two adjoining lordships with a lenders' hall that is neither's seat")
    (a, b, h) = rng.choices([t[0] for t in triples], weights=[t[1] for t in triples])[0]
    if rng.random() < 0.5:
        a, b = b, a
    s["lord_a"], s["lord_b"], s["reading_room"] = a, b, h
    s["_why"]["reading_room"] = (f"the hall both houses' lenders lodge at: {dist(h, a):.2f} from "
                                 f"{name(a)}, {dist(h, b):.2f} from {name(b)}, neither's seat")
    mx, my = (PL[a]["x"] + PL[b]["x"]) / 2, (PL[a]["y"] + PL[b]["y"]) / 2
    if sid == "W1-A":
        make(s, "ford_mill", "the ford mill on the border", "mill (spine site)", mx, my,
             "the debt object stands on the border, half-way between the seats; the map marks no mill")
        s["mill_market"] = pick(rng, settlements(), s["ford_mill"], BOUND, "no market near the mill")
    else:
        vill = [p for p in settlements() if PL[p]["size"] in ("village", "town")
                and p not in (a, b) and not lord_kind(p)
                and math.hypot(PL[p]["x"] - mx, PL[p]["y"] - my) <= BOUND]
        if vill:
            s["border_village"] = pick(rng, vill, None)
        else:
            make(s, "border_village", "the border village", "village (spine site)", mx, my,
                 "no unseated village stands on this border; the placer adds one, with its parish chapel")
        s["free_hall"] = pick(rng, [x for x in halls() if x != h], h, FREE, "no other hall within three days")
    return h


W1_LEADERS = ("both lords stay off the cards, seen through their stewards; a paranoid lord believes every unsigned "
              "warning, which suits the scheme", ("p-tuhudo",))  # the blind archivist-lord remembers every debt and name

spine("W1-A", shape=1, reveal="leader", form="side", title="The Truce Held for a Debt", enemy="W1", kind="the cost",
      heart=("H4", "a", "cast"), centre="reading_room", fill=fill_w1, lords=("lord_a", "lord_b"),
      stake="S2", move="M3",
      slots=[("lord_a", "seat", "a High holder with tenants: Banner Lord, Free Lord or Seated House head (d-l0)", "seats ≤2.8 apart"),
             ("lord_b", "seat", "as lord_a, of the same major faction (draft 4); adjoining (seats ≤2.8 apart)", "—"),
             ("reading_room", "hall", "a reading room in a large town, neither lord's seat, ≤2.8 from both seats (their lenders' hall, where the truce is lodged), in the lords' own faction's town or a neutral market's, never a rival faction's", "centre"),
             ("ford_mill", "site", "on the border between the seats (added: the map marks no mill)", "midpoint of the seats"),
             ("mill_market", "town", "any settlement", "≤1.25 of ford_mill"),
             ("truce_day", "day", "the lodging day both lords set (roller: a day name; `{days}` days off at the last move)", "—"),
             ("hand", "person", "her Wall sorter at reading_room (made on demand; the hall's)", "—"),
             ("steward_clerk", "person", "lord_b's household (made on demand)", "at lord_b's seat")],
      cards=[("R1", "Investigate · The ledger doesn't add up (heart's card)", "reading_room", ["reading_room"], B, ""),
             ("R2", "Passage · Smuggling (odd job)", "lord_b", ["lord_b"], J, "carried from reading_room's Wall branch to lord_b's steward's gate"),
             ("R3", "Acquire · Unknown worth (the miller's tally)", "mill_market", ["ford_mill"], B, ""),
             ("RV", "Reveal · Acquire Won't sell (lord_b's retired steward)", "lord_b", ["lord_b"], B, ""),
             ("C1-1", "Restore · Broken works (the mill race)", "ford_mill", ["ford_mill"], J, "the border, a ride out"),
             ("C1-2", "Investigate · Watch them (the branch)", "reading_room", ["reading_room"], B, ""),
             ("C1-3", "Dispute · Broken agreement (the steward's gatehouse)", "lord_b", ["lord_b"], B, ""),
             ("C2-0", "Passage · Unwilling passenger (the miller)", "ford_mill", ["ford_mill"], J, "back to the ford"),
             ("DOOR", {"a": "reading_room", "b": "reading_room", "c": "reading_room"}, ["reading_room"], B, ""),
             ("CR", "Acquire · Under seal (the hall's strongroom)", "reading_room", ["reading_room"], B, "")],
      acted="reading_room", giver=("the heart, who keeps the strongroom's key-book", "reading_room"),
      clock=([("lord_b", "reading_room")], None, "truce_day", "the steward rides in from his master's seat to object"),
      days_slots={"truce_day": DAYS["truce_day"]},
      alts={"R1": ["Acquire · Copy it before it burns (heart's card)"]},
      plants={},
      leaders_ok={"lord_a": W1_LEADERS, "lord_b": W1_LEADERS},
      parties={"the master": "RV", "the leader": "RV", "the sorter (the Hand)": "C1-3", "lord_b's steward": "C1-3",
               "the steward's clerk": "C1-3", "the steward's wife": "C1-3", "the old miller's widow": "C1-1",
               "the ford's tenants": "C1-1", "lord_b's house": "R2"},
      door_needs={"all": ["the old miller's widow"], "a": ["lord_b's steward"], "b": ["the master"]},
      ending_needs={"Lodged on the ruling": ["the steward's clerk"], "Burn your copy": ["lord_b's house"],
                    "Keep the balance": ["the ford's tenants", "the steward's clerk"]},
      crisis_blames_heart=False)

spine("W1-B", shape=2, reveal="heart", form="chain", title="The Stone That Walked", enemy="W1", kind="the cost",
      heart=("H2", "c", "cast"), centre="reading_room", fill=fill_w1, lords=("lord_a", "lord_b"),
      stake="S7", move="M9",
      slots=[("lord_a", "seat", "a High holder with a border: Banner Lord, Free Lord or Seated House head (d-l0)", "seats ≤2.8 apart"),
             ("lord_b", "seat", "as lord_a, of the same major faction (draft 4)", "—"),
             ("reading_room", "hall", "the survey hall: a reading room in a large town, neither seat, ≤2.8 from both, in the lords' own faction's town or a neutral market's", "centre"),
             ("border_village", "village", "Low or Middle, unseated, on the border, with a parish chapel (added if none)", "≤1.25 of the seats' midpoint"),
             ("hand", "person", "the survey hall's copyist (made on demand)", "—"),
             ("ally", "person", "lord_b's surveyor (made on demand; his)", "—"),
             ("fault", "person", "the survey hall's eldest surveyor (made on demand; the hall's)", "—"),
             ("free_hall", "hall", "any reading room", "≤3.0 of the centre")],
      cards=[("R1", "Investigate · The ledger doesn't add up", "free_hall", [], F, ""),
             ("R2", "Dispute · Broken agreement (heart's card)", "reading_room", ["reading_room"], B, ""),
             ("R3", "Acquire · Copy it before it burns (odd job: the chapel's tithe book)", "border_village", ["border_village"], B, ""),
             ("RV", "Reveal · Restore Lost good name (lord_a's retired surveyor)", "lord_a", ["lord_a"], B, ""),
             ("C1-1", "Investigate · Watch them (the masons; lord_b's surveyor met)", "border_village", ["border_village"], J, "the border, a ride out"),
             ("C1-2", "Dispute · Shared resource", "border_village", ["border_village"], B, ""),
             ("C1-3", "Acquire · Won't sell", "reading_room", ["reading_room"], B, ""),
             ("C2-0", "Threat · Go and look", "border_village", ["border_village"], J, "the border at night"),
             ("DOOR", {"a": "reading_room", "b": "border_village", "c": "reading_room"}, [], B, ""),
             ("CR", "Reckoning · Schism (at the burned village)", "border_village", ["border_village"], B, "")],
      acted="border_village", giver=("the heart's mother, among the ashes of her house", "border_village"),
      alts={"R1": ["Acquire · Take the readings"]},
      plants={},
      leaders_ok={"lord_a": W1_LEADERS, "lord_b": W1_LEADERS},
      parties={"the master": "RV", "the heart's mother": "R2", "the village": "R3", "fault (the eldest surveyor)": "C1-1",
               "the Hand": "C1-3", "lord_b's surveyor": "C1-1", "lord_a": "RV"},
      door_needs={"all": ["fault (the eldest surveyor)"], "a": ["the village"], "b": ["the master"]},
      ending_needs={"Open the ledgers": ["the master", "the heart's mother"], "Keep the balance": ["fault (the eldest surveyor)"],
                    "Take a side at last": ["lord_a"]},
      crisis_blames_heart=False)


# W2 -------------------------------------------------------------------------
def free_lord_dig(rng, s, label):
    lords = [p for p in seats(("Free Lord",)) if p not in s["_lords"] and p not in s["_towns"]
             and lord_ok(s["_sid"], "lord", p) and size_ok(s["_sid"], "lord", p)]
    if not lords:
        raise NoFit("no Free Lord left")
    lord = rng.choice(lords)
    s["lord"] = lord
    x, y = inland(lord, 0.8, rng)
    make(s, "dig", f"the {label} dig, at an imperial ruin", "Writers dig (spine site)", x, y,
         f"the map's only dig (the Old Road dig) is in Great House land; the placer adds one at a ruin in "
         f"{name(lord)}'s march, 0.8 from his seat", territory=PL[lord]["territory"])
    return lord


def fill_w2a(rng, s, sid):
    lord = free_lord_dig(rng, s, "march")
    s["court"] = lord
    dig = s["dig"]
    make(s, "hamlet", "the diggers' hamlet", "hamlet (spine site)", *toward(dig, lord, 0.25),
         "the diggers live below the dig", territory=PL[lord]["territory"])
    make(s, "valley", "the three valleys", "farmland (spine site)", *inland(lord, 1.0, rng),
         "the three valleys of the claim lie past the dig, in his march; the heart's farm is in one", territory=PL[lord]["territory"])
    s["lodge_hall"] = pick(rng, halls(), dig, STRETCH, "no reading room within reach of the dig")
    if dist(s["lodge_hall"], dig) > BOUND:
        s["_why"]["lodge_hall"] = f"stretched to {dist(s['lodge_hall'], dig):.2f}: the nearest reading room to the dig"
    lan = [p for p in settlements() if has(p, "Lantern") and dist(p, dig) <= STRETCH]
    s["lantern_post"] = pick(rng, lan, dig, STRETCH, "no Lantern post near the dig", avoid=(s["lodge_hall"],))
    if dist(s["lantern_post"], dig) > BOUND:
        s["_why"]["lantern_post"] = f"stretched to {dist(s['lantern_post'], dig):.2f}: the nearest Lantern post (the dig's contract chapter)"
    pawn = [p for p in settlements() if has(p, "pawn house") and dist(p, dig) <= STRETCH]
    if pawn:
        s["pawn_town"] = pick(rng, pawn, dig, STRETCH, avoid=(s["lodge_hall"], s["lantern_post"]))
        if dist(s["pawn_town"], dig) > BOUND:
            s["_why"]["pawn_town"] = f"stretched to {dist(s['pawn_town'], dig):.2f}: the nearest pawn house"
    else:
        s["pawn_town"] = s["lodge_hall"]
        add_feature(s["lodge_hall"], "a pawnbroker", "no pawn house near the dig; a large town can hold one")
    s["r1_hall"] = pick(rng, [h for h in halls() if h != s["lodge_hall"]], dig, FREE,
                        "no second hall within three days of the dig", avoid=(s["pawn_town"], s["lantern_post"]))
    return dig


spine("W2-A", shape=1, reveal="letter", form="names", title="The Last of the Four", enemy="W2", kind="the hidden hand",
      heart=("H6", "a", "cast"), centre="dig", main="lord", fill=fill_w2a, lords=("lord",),
      stake="S5", move="M6",
      slots=[("lord", "seat", "a Free Lord (d-l0 holder, sub f-free-lords); his court is at his seat", "—"),
             ("dig", "site", "a Writers' dig at an imperial ruin in the lord's territory (added: the map has none there)", "centre; 0.8 from the seat"),
             ("valley", "site", "the three valleys, in the lord's territory; the heart's farm is in one, and the boy hides there", "1.0 from the seat"),
             ("hamlet", "site", "the diggers' hamlet", "≤1.25 of the dig"),
             ("lodge_hall", "hall", "the reading room nearest the dig, where its finds are catalogued", "≤1.25 of the dig, else ≤1.6 (nearest)"),
             ("lantern_post", "town", "a Lantern chapter house or contract board (the dig's guards' chapter)", "≤1.25 of the dig, else ≤1.6 (nearest)"),
             ("pawn_town", "town", "a pawn house (presence); else the placer adds a pawnbroker at lodge_hall", "≤1.25 of the dig, else ≤1.6"),
             ("r1_hall", "hall", "a reading room (not lodge_hall) keeping the year's copy of the Wayhouse's lodging book, as every hall does", "≤3.0 of the dig"),
             ("court", "seat", "the lord's court, at his seat (= lord)", "—"),
             ("hearing_day", "day", "the court's sitting (roller: a day name; `{days}` days off at the last move)", "—"),
             ("hand", "person", "the dig's foreman (made on demand)", "—"),
             ("sergeant", "person", "the Lantern sergeant of the dig's camp (made on demand; the Lantern's)", "—"),
             ("son", "person", "the heart's son, a digger at the dig (cast with the heart's household)", "at the dig")],
      cards=[("R1", "Investigate · The ledger doesn't add up (the lodging book's year copy)", "r1_hall", [], F, ""),
             ("R2", "Dispute · Broken agreement (heart's card: he stands witness)", "lantern_post", ["dig"], B, ""),
             ("R3", "Investigate · Disappearance (odd job: the dig's reader)", "lodge_hall", ["dig"], B, ""),
             ("RV", "Reveal · Acquire Passed on (the pawned tool roll)", "pawn_town", ["dig"], B, ""),
             ("C1-1", "Threat · Go and look (the closed dig)", "dig", ["dig"], J, "out to the dig"),
             ("C1-2", "Dispute · Public offense (the wage-roll)", "hamlet", ["dig"], B, ""),
             ("C1-3", "Acquire · Under seal (the Lantern camp)", "dig", ["dig"], B, ""),
             ("C2-0", "Dispute · Shared resource (the hamlet's grazing)", "hamlet", ["dig"], B, ""),
             ("DOOR", {"a": "lodge_hall", "b": "lord", "c": "hamlet"}, ["dig"], B, ""),
             ("CR", "Passage · Unwilling passenger (the heart's farm)", "valley", ["valley"], B, "")],
      acted="lord", giver=("the heart, at the farm where the boy hides", "valley"),
      clock=([("valley", "lord")], None, "hearing_day", "the boy must be got from the farm to the court"),
      days_slots={"hearing_day": DAYS["hearing_day"]},
      plants={},
      parties={"the son": "R2", "the sergeant": "C1-3", "the lord": "C1-3", "the archivist": "RV", "the valleys' holders": "R2",
               "the eldest copyist": "R1", "the reader": "R3"},
      owns=[("dig", "lordship", "lord"), ("valley", "lordship", "lord"), ("hamlet", "lordship", "lord")],
      door_needs={"all": ["the son", "the sergeant"], "a": ["the lord"], "b": ["the archivist"]},
      ending_needs={"Delivered": ["the son", "the sergeant"], "Burn your copy": ["the valleys' holders"],
                    "Released": ["the valleys' holders", "the son"]},
      crisis_blames_heart=False)


def fill_w2b(rng, s, sid):
    lord = free_lord_dig(rng, s, "border")
    dig = s["dig"]
    near = [p for p in settlements() if dist(p, dig) <= BOUND]
    # draft 4: the stall is in the lord's own land, where his men can serve a summons
    own = [p for p in near if lordship_of(p) == lord]
    if not own:
        raise NoFit("no settlement of the lord's own within a day of the dig")
    st = [p for p in own if has(p, "letter-writer's stall")]
    if st:
        s["stall"] = pick(rng, st, dig, BOUND)
    else:
        s["stall"] = pick(rng, own, dig, BOUND)
        add_feature(s["stall"], "a letter-writer's stall (the heart's)",
                    f"no stall in {name(lord)}'s land within a day of the dig; a settlement of his can hold one")
    for d in (0.7, 0.55, 0.4, 0.3):
        x, y = toward(dig, WAYHOUSE, d)
        if lordship_at(x, y, PL[lord]["territory"]) == lord:
            break
    else:
        raise NoFit("the dig's track leaves the lord's land at once")
    make(s, "toll", f"{name(lord)}'s toll on the dig's track", "toll (spine site)",
         x, y, "where the dig's track leaves his land, toward the high road (no map road reaches the dig)",
         territory=PL[lord]["territory"])
    s["reveal_town"] = pick(rng, near, dig, BOUND)
    hv = [p for p in PL if PL[p].get("holder") and p not in (lord, WAYHOUSE, s["stall"])
          and PL[p]["territory"] != PL[lord]["territory"]
          and lord_kind(p) in ("Open Council head", "Banner Lord", "Seated House head")
          and dist(p, s["stall"]) <= FREE and lord_ok(sid, "haven", p)]
    s["haven"] = pick(rng, hv, s["stall"], FREE, "no haven outside his land within three days of her stall")
    return dig


spine("W2-B", shape=2, reveal="leader", form="question", title="Sold by the Wayhouse", enemy="W2", kind="the hidden hand",
      heart=("H1", "c", "cast"), centre="dig", main="lord", fill=fill_w2b, lords=("lord",),
      stake="S5", move="M4",
      slots=[("lord", "seat", "a Free Lord (d-l0 holder, sub f-free-lords)", "—"),
             ("dig", "site", "a Writers' dig whose diggers can't write, in the lord's territory (added: none on the map there)", "centre; 0.8 from the seat"),
             ("stall", "town", "the heart's letter-writer's stall, in the lord's own land (his seat or a place nearer it than any other lord's seat), where his men can serve a summons (presence; else added, with reason)", "≤1.25 of the dig"),
             ("toll", "site", "the lord's toll where the dig's track leaves his land, toward the high road (added; in his lordship)", "≤0.7 from the dig"),
             ("reveal_town", "town", "any settlement (the dead digger's sons)", "≤1.25 of the dig"),
             ("haven", "seat", "a d-l0 seat outside the lord's land and the magistrate's writ, held by an Open Council head, a Banner Lord or a Seated House head", "≤3.0 of the stall"),
             ("hearing_day", "day", "the suit's hearing in Highcrossing (roller: a day name; `{days}` days off at the last move)", "—"),
             ("magistrate", "person", "Highcrossing's magistrate (the Throne's officer, made on demand)", "Highcrossing"),
             ("hand", "person", "the archivist's buyer (made on demand)", "—"),
             ("ally", "person", "the diggers' headman's daughter (made on demand)", "at the dig")],
      cards=[("R1", "Dispute · Broken agreement (heart's card: her bill)", "stall", ["stall"], B, ""),
             ("R2", "Passage · Smuggling (odd job: a bill of sale past the toll)", "toll", ["toll"], J, "carried from her stall past the lord's toll on the dig's track"),
             ("R3", "Investigate · The ledger doesn't add up (the lodging book)", "wayhouse", [], HQ, "the dig's lodgings are kept at the Wayhouse"),
             ("RV", "Reveal · Dispute Contested inheritance (a digger's sons)", "reveal_town", ["dig"], B, ""),
             ("C1-1", "Investigate · Watch them (the dig's gate; the ally met)", "dig", ["dig"], J, "out to the dig"),
             ("C1-2", "Dispute · Public offense (the headman)", "dig", ["dig"], B, ""),
             ("C1-3", "Acquire · Won't sell (her cart, held at the toll)", "toll", ["toll"], B, ""),
             ("C2-0", "Restore · Lost livelihood (her stall)", "stall", ["stall"], B, ""),
             ("DOOR", {"a": "lord", "b": "toll"}, ["dig"], B, ""),
             ("CR", "Passage · Fugitive (from her stall)", "stall", ["stall"], B, "")],
      acted="haven", giver=("the heart, at her stall, the summons in her hand", "stall"),
      clock=([("wayhouse", "stall"), ("stall", "haven")], (1, "the haven's lofts carry her sworn statement up to the hearing"), "hearing_day", "the summons rides from Highcrossing to her stall; she must then reach the haven and swear, and its lofts carry her statement up in a day"),
      days_slots={"hearing_day": DAYS["hearing_day"]},
      alts={"R3": ["Acquire · Copy it before it burns (the lodging book)"]},
      plants={"Trade what we know": "R3 'a late fee for the magistrate's own house, hidden'"},
      parties={"the headman": "C1-2", "the headman's daughter": "C1-1", "the toll keeper": "C1-3", "the toll's clerk": "C1-3",
               "the magistrate's household": "R3", "the lord": "R2", "the diggers": "C1-2", "the haven's holder": "CR",
               "the archivist": "RV"},
      owns=[("dig", "lordship", "lord"), ("stall", "lordship", "lord"), ("toll", "lordship", "lord")],
      door_needs={"all": [], "a": ["the lord", "the archivist"], "b": ["the toll's clerk"]},
      ending_needs={"Delivered": ["the haven's holder"], "Burn your copy": ["the diggers"],
                    "Trade what we know": ["the magistrate's household"]},
      leaders_ok={"haven": ("the haven's holder only takes a sworn statement through a clerk and sends it by the lofts", ())},
      crisis_blames_heart=False)


# W3 -------------------------------------------------------------------------
def fill_w3a(rng, s, sid):
    rr = [p for p in halls() if p not in s["_towns"]]
    s["reading_room"] = pick(rng, rr, None)
    c = s["reading_room"]
    other = nearest([p for p in settlements() if p != c], c)
    make(s, "barn", "the night market's barn", "barn (spine site)", *away(c, other, 0.9),
         "a barn a day out, where the night market meets")
    near = [p for p in settlements() if p != c and dist(p, c) <= BOUND]
    s["widow_town"] = pick(rng, near, c, BOUND) if near else c
    return c


spine("W3-A", shape=3, reveal="ally", form="chain", title="Every Family Its Own File", enemy="W3", kind="the hidden hand",
      heart=("H3", "c", "man (he writes to his wife)"), centre="reading_room", fill=fill_w3a, lords=(),
      stake="S3", move="M5",
      slots=[("reading_room", "hall", "a reading room in a large town that keeps families' files", "centre"),
             ("barn", "site", "the night market's barn (added)", "0.9 from the centre"),
             ("widow_town", "town", "any settlement whose letters go through reading_room's Wall", "≤1.25 of the centre (the centre itself if none)"),
             ("hand", "person", "her runner (made on demand)", "—"),
             ("wall_clerk", "person", "reading_room's Wall clerk (made on demand; the hall's)", "—")],
      cards=[("R1", "Dispute · Broken agreement (heart's card)", "reading_room", ["reading_room"], B, ""),
             ("R2", "Passage · Smuggling (odd job: the love letter)", "barn", ["reading_room"], B, ""),
             ("R3", "Acquire · Copy it before it burns (the widow)", "widow_town", ["reading_room"], B, ""),
             ("RV", "Reveal · Investigate Theft (the Wall clerk asks; the night porter knows)", "reading_room", ["reading_room"], B, ""),
             ("C1-1", "Threat · Go and look (the night market)", "barn", ["reading_room"], J, "the barn a day out"),
             ("C1-2", "Dispute · Public offense (the carters' yard; the turn)", "reading_room", ["reading_room"], B, ""),
             ("C1-3", "Passage · Fugitive (the road out; the Wall clerk asks)", "reading_room", ["reading_room"], B, ""),
             ("C2-0", "Acquire · Unknown worth", "reading_room", ["reading_room"], B, ""),
             ("DOOR", {"a": "reading_room", "b": "reading_room", "c": "reading_room"}, ["reading_room"], B, ""),
             ("CR", "Reckoning · Swindle (the booth, at the Wall)", "reading_room", ["reading_room"], B, "")],
      acted="reading_room", giver=("the heart, at the Wall his letters went through", "reading_room"),
      plants={},
      parties={"the heart's wife": "R1", "the Wall clerk": "RV", "the copyist": "RV", "the families": "C1-2", "the forger": "RV"},
      door_needs={"all": ["the Wall clerk"], "a": ["the heart's wife"], "b": ["the forger"]},
      ending_needs={"Amends made": ["the copyist", "the forger"], "Burn your copy": ["the families"],
                    "Wrongdoer ruined": ["the forger"]},
      crisis_blames_heart=False)


def court_town(rng, s):
    ct = [p for p in halls() if lord_kind(p) in COURTS and p not in s["_lords"] and p not in s["_towns"]
          and lord_ok(s["_sid"], "lord", p)]
    if not ct:
        raise NoFit("no court town left")
    return pick(rng, ct, None)


def fill_w3b(rng, s, sid):
    c = court_town(rng, s)
    s["lord"] = s["reading_room"] = c
    pawn = [p for p in settlements() if has(p, "pawn house") and dist(p, c) <= BOUND]
    if pawn:
        s["pawn_town"] = pick(rng, pawn, c, BOUND)
    else:
        s["pawn_town"] = c
        add_feature(c, "a pawnbroker", "no pawn house within a day; a large town can hold one")
    make(s, "record_house", "the court's record house outside the walls", "record house (spine site)",
         *away(c, nearest([p for p in settlements() if p != c], c), 0.5), "the court's hall of record stands outside the town walls")
    s["bindery_hall"] = pick(rng, [h for h in halls() if h != c], c, FREE, "no other hall within three days")
    s["house"], hid = house_near(rng, c, s, skip=tuple(PL[l].get("holder") for l in s["_lords"]))
    s["_house_id"] = hid
    return c


spine("W3-B", shape=1, reveal="heart", form="question", title="The Copy House Across the Street", enemy="W3", kind="the hidden hand",
      heart=("H1", "b", "cast"), centre="reading_room", fill=fill_w3b, lords=("lord",),
      stake="S7", move="M1",
      slots=[("lord", "seat", "a court holder whose seat is a large town with a reading room: Banner/Free Lord, Seated or Tide House head, the Binders' head, Open Council head (d-l0)", "= reading_room's town"),
             ("reading_room", "hall", "that town's reading room", "centre"),
             ("record_house", "site", "the court's record house outside the walls (added)", "0.5 from the centre"),
             ("pawn_town", "town", "a pawn house (presence); else a pawnbroker added at the centre", "≤1.25 of the centre"),
             ("bindery_hall", "hall", "another reading room (the old bindery's town)", "≤3.0 of the centre"),
             ("house", "house", "the house the rolls shame most, the steward's creditor: a d-l0 house within three days (no faction leader, not the court's holder), else made on demand", "≤3.0 of the centre"),
             ("hand", "person", "the copyist's kinswoman (flavour d), a clerk in the court's counting room (made on demand; the court's)", "—"),
             ("copyist", "person", "a copyist of reading_room, her proxy (made on demand; the hall's)", "—"),
             ("binder", "person", "the old bookbinder of bindery_hall's town (made on demand)", "—")],
      cards=[("R1", "Dispute · Contested inheritance (heart's card)", "reading_room", ["reading_room"], B, ""),
             ("R2", "Investigate · The ledger doesn't add up (odd job: the court's bill, at its counting room)", "lord", ["lord"], B, ""),
             ("R3", "Restore · Lost livelihood (the bindery)", "bindery_hall", [], F, ""),
             ("RV", "Reveal · Acquire Unknown worth (the pawned sheets, read aloud to the heart)", "pawn_town", ["reading_room"], B, ""),
             ("C1-1", "Dispute · Broken agreement (the court's steward)", "record_house", ["lord"], J, "out to the record house"),
             ("C1-2", "Investigate · Watch them (the lane)", "reading_room", ["reading_room"], B, ""),
             ("C1-3", "Rescue · Bound by debt (the copy house)", "reading_room", ["reading_room"], B, ""),
             ("C2-0", "Acquire · Copy it before it burns (the archive room)", "record_house", ["lord"], B, ""),
             ("DOOR", {"a": "record_house", "b": "record_house"}, ["reading_room"], B, ""),
             ("CR", "Acquire · Won't sell (the commission)", "reading_room", ["reading_room"], B, "")],
      acted="reading_room", giver=("the heart, at the reading room, the court's letter in hand", "reading_room"),
      plants={"Trade what we know": "C2-0 'a margin note on the steward's own debt'"},
      parties={"the house": "C2-0", "the steward": "C1-1", "the binder": "C1-3", "the bound copyists": "C1-3",
               "the court": "C1-1", "the forger": "RV"},
      door_needs={"all": ["the binder"], "a": ["the court"], "b": ["the steward"]},
      ending_needs={"Obtained and handed over": ["the house", "the bound copyists"],
                    "Burn your copy": ["the forger"], "Trade what we know": ["the house", "the steward"]},
      leaders_ok={"lord": ("the court's copying is let by its steward and counting room; the lord never appears",
                           ("p-tuhudo",))},
      crisis_blames_heart=False)


# W4 -------------------------------------------------------------------------
def clerk_and_stall(rng, s, wayhouse_ok):
    pairs = [(h, st) for h in halls(wayhouse_ok) for st in stalls()
             if st != h and dist(h, st) <= STRETCH and h not in s["_towns"]]
    if not pairs:
        raise NoFit("no hall with a stall in reach")
    h, st = rng.choice(pairs)
    s["clerk_hall"], s["stall"] = h, st
    if dist(h, st) > BOUND:
        s["_why"]["stall"] = f"stretched to {dist(h, st):.2f}: the nearest stall to any hall is 1.3–1.4 days out"
    return h


def fill_w4a(rng, s, sid):
    h = clerk_and_stall(rng, s, False)
    st = s["stall"]
    s["town"] = st
    make(s, "loft", "the relay loft on the hill", "relay loft (spine site)",
         *toward(h, st, 0.5), "a pigeon loft on high ground beside the clerk's hall")
    f = [p for p in settlements() if has(p, "fence's shop") and dist(p, h) <= FREE]
    if f:
        s["fence_town"] = pick(rng, f, h, FREE)
    else:
        f = [p for p in settlements() if PL[p]["size"] in ("large", "great") and has(p, "Night Market")
             and dist(p, h) <= FREE and p not in (h, WAYHOUSE)]
        s["fence_town"] = pick(rng, f, h, FREE, "no Night Market town in reach")
        add_feature(s["fence_town"], "a Night Market fence's shop", "the map's only fence's shop is out of reach; a Night Market town can hold one")
    fair = [p for p in settlements() if PL[p]["size"] in ("town", "large", "great")
            and p not in (st, WAYHOUSE) and dist(p, st) <= 2.5]
    w = [(3.0 if frozenset((p, st)) in ROADS else 1.0) / (0.2 + dist(p, st)) ** 2 for p in fair]
    if not fair:
        raise NoFit("no fair town in reach")
    s["fair_town"] = rng.choices(fair, weights=w)[0]
    s["robbed_town"] = pick(rng, [p for p in halls() + stalls() if p not in (st,)], h, FREE)
    hide = [p for p in settlements() if dist(p, s["loft"]) <= BOUND and p != h]
    s["hideout"] = pick(rng, hide, s["loft"], BOUND) if hide else h
    return h


spine("W4-A", shape=3, reveal="leader", form="names", title="The Stalls' Oath", enemy="W4", kind="the hidden hand",
      heart=("H3", "a", "cast"), centre="clerk_hall", fill=fill_w4a, lords=(),
      stake="S2", move="M7",
      slots=[("clerk_hall", "hall", "the reading room whose clerk runs its district's stalls (the enemy's seat)", "centre"),
             ("stall", "stall", "a letter-writer's stall (presence) run from clerk_hall", "≤1.25 of the centre, else ≤1.6 (nearest)"),
             ("town", "town", "the heart's own town = the stall's", "—"),
             ("loft", "site", "the relay loft on high ground (added)", "0.5 from the centre"),
             ("fence_town", "town", "a Night Market fence's shop (presence); else a large Night Market town, the shop added", "≤3.0 of the centre"),
             ("fair_town", "town", "a town the heart hauls to (road-linked preferred)", "≤2.5 of the heart's town"),
             ("fair", "day", "the fair at fair_town (roller: a fair's name)", "—"),
             ("robbed_town", "hall/stall", "any town with a Writers' presence", "≤3.0 of the centre"),
             ("hideout", "town", "where the loft keeper hides", "≤1.25 of the loft"),
             ("hand", "person", "his list-writer (made on demand)", "—"),
             ("ally", "person", "someone who has seen a list (made on demand; see flavour c)", "in the heart's town"),
             ("painter", "person", "the old sign-painter of the heart's town (made on demand)", "—")],
      cards=[("R1", "Dispute · Broken agreement (heart's card: his wage)", "stall", ["stall"], B, ""),
             ("R2", "Investigate · Disappearance (odd job: the loft)", "loft", ["clerk_hall"], B, ""),
             ("R3", "Acquire · Copy it before it burns (a robbed merchant)", "robbed_town", [], F, ""),
             ("RV", "Reveal · Passage Unwilling passenger (the loft keeper)", "hideout", ["loft"], B, ""),
             ("C1-1", "Restore · Lost good name (the turned-off letter-writer)", "town", ["stall"], B, ""),
             ("C1-2", "Acquire · Unknown worth (the fence's boy; the turn)", "fence_town", ["fence_town"], J, "to the fence's shop"),
             ("C1-3", "Passage · Fugitive (the road out; the ally asks)", "fence_town", ["fence_town"], J, "the road out of the fence's town"),
             ("C2-0", "Threat · Go and look (which streets the crews walked)", "town", ["stall"], B, ""),
             ("DOOR", {"a": "town", "b": "clerk_hall", "c": "clerk_hall"}, [], B, ""),
             ("CR", "Dispute · Broken agreement (does the oath bind the halls?)", "town", ["stall"], B, "")],
      acted="town", giver=("the heart, in his robbed house in his own town", "town"),
      days_slots={"fair": DAYS["fair"]},
      plants={},
      parties={"the painter": "C2-0", "the ally": "C1-3", "the leader": "RV", "the clerk": "RV", "the heart's daughter": "R1"},
      door_needs={"all": ["the painter"], "a": ["the clerk"], "b": ["the leader"]},
      ending_needs={"The heart wins": ["the clerk"], "The clerk wins": ["the clerk"], "Take a side at last": ["the ally"]},
      crisis_blames_heart=False)


def fill_w4b(rng, s, sid):
    h = clerk_and_stall(rng, s, True)
    st = s["stall"]
    s["town"] = st
    br = [p for p in settlements() if has(p, "Binder debt broker") and dist(p, st) <= STRETCH]
    if br:
        s["broker"] = pick(rng, br, st, STRETCH)
    else:
        s["broker"] = st
        add_feature(st, "a Binders' debt broker" + (" (hidden: an Open Hand town)" if PL[st]["territory"] == "t-open" else ""),
                    "the map lists brokers only in the Marches; the Binders lend wherever there is debt")
    make(s, "hamlet", "the weavers' hamlet", "hamlet (spine site)", *away(st, h, 0.7),
         "a hamlet whose debts were bought, out from the heart's town")
    stw = [p for p in seats(LANDLORDS) if p != st and p not in s["_lords"] and lord_ok(sid, "steward_seat", p)]
    s["steward_seat"] = pick(rng, stw, h, FREE, "no lord's seat in reach")
    # draft 5 (review-v7d 5.4): R3 is drawn by nearness to the heart's town, where the story's people are,
    # not to the centre: within two days of it where any hall is, else the nearest other hall to it
    r3 = [x for x in halls() if x != h and dist(x, h) <= FREE]
    if not r3:
        raise NoFit("no other hall within three days")
    near = [x for x in r3 if dist(x, st) <= 2.0]
    s["r3_hall"] = pick(rng, near, st, 2.0) if near else nearest(r3, st)
    if dist(s["r3_hall"], st) > 2.0:
        s["_why"]["r3_hall"] = f"{dist(s['r3_hall'], st):.2f} from the heart's town: the nearest other hall to it"
    add_feature(st, "a bindery (the bookbinder's widow's), whose presses the heart works by night",
                "the heart lives and works in his own town; the spine puts the bindery there")
    return h


spine("W4-B", shape=2, reveal="heart", form="side", title="Who Is Poor", enemy="W4", kind="the hidden hand",
      heart=("H5", "b", "man (the son)"), centre="clerk_hall", main="town", fill=fill_w4b, lords=("steward_seat",),
      stake="S2", move="M9",
      slots=[("clerk_hall", "hall", "the reading room (or the Wayhouse) whose clerk runs its stalls", "centre"),
             ("stall", "stall", "a letter-writer's stall (presence) run from it", "≤1.25, else ≤1.6 (nearest)"),
             ("town", "town", "the heart's town = the stall's; he works its bindery's presses by night; his father's creditor lives there", "—"),
             ("broker", "town", "a Binders' debt broker (presence); else one added in the heart's town (hidden if Open Hand)", "≤1.25 of the heart's town, else ≤1.6"),
             ("hamlet", "site", "the weavers' hamlet whose debts were bought (added)", "0.7 from the heart's town"),
             ("steward_seat", "seat", "a High holder (Banner/Free Lord, Seated House head) whose steward keeps his late predecessor's accounts", "≤3.0 of the centre"),
             ("r3_hall", "hall", "any other hall, drawn by nearness to the heart's town (draft 5: within two days of it where any hall is, else the nearest)", "≤3.0 of the centre"),
             ("hand", "person", "the clerk's letter-writer (made on demand)", "—"),
             ("uncle", "person", "the heart's uncle, a carter of the heart's town (cast with the heart's household)", "in the heart's town"),
             ("old_creditor", "person", "his father's old creditor, in the heart's town (made on demand)", "—")],
      cards=[("R1", "Restore · Lost livelihood (heart's card: the bindery)", "town", ["stall"], B, ""),
             ("R2", "Acquire · Copy it before it burns (odd job: the steward's accounts)", "steward_seat", [], F, ""),
             ("R3", "Investigate · The ledger doesn't add up", "r3_hall", [], F, ""),
             ("RV", "Reveal · Dispute Shared resource (two stall-keepers; the heart brings the player)", "town", ["stall"], B, ""),
             ("C1-1", "Acquire · Unknown worth (the weavers' letter; the old creditor met)", "hamlet", ["town"], J, "out to the hamlet"),
             ("C1-2", "Threat · Go and look (the broker's yard)", "broker", ["town"], B, ""),
             ("C1-3", "Dispute · Public offense (the stall)", "stall", ["stall"], B, ""),
             ("C2-0", "Acquire · Under seal (his father's writing case)", "clerk_hall", ["clerk_hall"], B, ""),
             ("DOOR", {"a": "town", "b": "clerk_hall", "c": "clerk_hall"}, [], B, ""),
             ("CR", "Rescue · Bound by debt (the clerk's rooms)", "clerk_hall", ["clerk_hall"], B, "")],
      acted="clerk_hall", giver=("the heart, at the clerk's door", "clerk_hall"),
      alts={"R3": ["Investigate · Watch them"]},
      plants={},
      parties={"the uncle": "R1", "the heart's sister": "R1", "the old creditor": "C1-1", "the moneylender's clerk": "R1",
               "the leader": "RV", "the clerk": "RV", "the old stall-keeper": "RV"},
      door_needs={"all": ["the moneylender's clerk"], "a": ["the uncle"], "b": ["the clerk"]},
      leaders_ok={"steward_seat": ("the lord never appears: his steward burns his predecessor's accounts on the clerk's "
                                   "man's word", ("p-tuhudo",))},  # the blind archivist-lord remembers every debt forgiven
      ending_needs={"Freed": ["the old creditor", "the heart's sister"], "Burn your copy": ["the clerk"],
                    "Exchanged": ["the heart's sister", "the clerk"]},
      crisis_blames_heart=False)


# W5 -------------------------------------------------------------------------
def fill_w5a(rng, s, sid):
    vh = [p for p in halls() + stalls() if PL[p]["territory"] == "t-first" and p != MOUNT
          and dist(p, WAYHOUSE) <= JOURNEY]
    s["branch_hall"] = pick(rng, vh, WAYHOUSE, JOURNEY, "no First Peoples Wall branch in reach")
    make(s, "inq_camp", "the inquisitors' camp", "camp (spine site)", *toward(s["branch_hall"], MOUNT, 0.5),
         "on the Mount road out of the sealed branch's town, toward the Mount the inquisitors came from")
    s["lodging"] = resolve_lodging("an inn by that town's gate", WAYHOUSE)  # draft 5: fixed, not flavour
    th = [h for h in halls() if has(h, "tithe-house") and dist(h, WAYHOUSE) <= FREE]
    if th:
        s["tithe_hall"] = pick(rng, th, WAYHOUSE, FREE)
    else:
        s["tithe_hall"] = nearest(halls(), MOUNT)
        add_feature(s["tithe_hall"], "a tithe-house book copied at the hall", "no hall town with a tithe-house in reach")
    s["widow_hall"] = pick(rng, [h for h in halls() if h != s["tithe_hall"]], WAYHOUSE, FREE)
    s["mount"] = MOUNT
    return WAYHOUSE


spine("W5-A", shape=2, reveal="letter", form="names", motif="lost surveyor", title="The Letters Never Collected", enemy="W5", kind="the old debt",
      heart=("H5", "c", "man (the son)"), centre="wayhouse", fill=fill_w5a, lords=(),
      stake="S2", move="M8", about="the Wayhouse (its Wall's oldest pigeonholes, burned in its own yard)",
      slots=[("wayhouse", "fixed", "the Wayhouse's Lending Wall, oldest section", "centre"),
             ("branch_hall", "hall/stall", "a Wall branch in First Peoples land where the house's word holds; not the Mount", "≤3.5 of Highcrossing (a journey)"),
             ("inq_camp", "site", "the inquisitors' camp on the road beyond (added)", "0.5 beyond branch_hall"),
             ("tithe_hall", "hall", "a reading room in a town with a tithe-house (presence)", "≤3.0 of Highcrossing"),
             ("widow_hall", "hall", "any other hall", "≤3.0 of Highcrossing"),
             ("mount", "fixed", "the Mount (the Holy House's seat; its stall)", "—"),
             ("inquisitor", "person", "a Holy House inquisitor, the Mount's (made on demand under its holder)", "—"),
             ("hand", "person", "a pious copyist in the inquisitor's pay (made on demand)", "—"),
             ("magistrate", "person", "Highcrossing's magistrate (made on demand)", "—"),
             ("ally", "person", "the dead copyist's widow (made on demand)", "—"),
             ("lodging", "place", "where the inquisitors sleep before a burning: an inn by Highcrossing's gate (roller)", "Highcrossing")],
      cards=[("R1", "Dispute · Broken agreement (heart's card: the pigeonhole fee)", "wayhouse", [], HQ, "his letters wait in the Wayhouse's Wall"),
             ("R2", "Acquire · Copy it before it burns (odd job: the widow)", "widow_hall", [], F, ""),
             ("R3", "Investigate · The ledger doesn't add up (the tithe book)", "tithe_hall", [], F, ""),
             ("RV", "Reveal · Acquire Unknown worth (the old desk; the drafters' own copy of their letter)", "tithe_hall", [], F, ""),
             ("C1-1", "Passage · Smuggling (the sealed branch's letters; the ally met)", "branch_hall", ["branch_hall"], J, "the road to the branch, where the inquisitor searches"),
             ("C1-2", "Threat · Go and look (the camp)", "inq_camp", ["branch_hall"], J, "the road beyond the branch"),
             ("C1-3", "Acquire · Won't sell (the Mount's stall)", "mount", [], F, ""),
             ("C2-0", "Restore · Lost livelihood", "branch_hall", ["branch_hall"], J, "the branch's town"),
             ("DOOR", {"a": "branch_hall", "b": "branch_hall"}, ["branch_hall"], J, "the branch's town"),
             ("CR", "Acquire · Copy it before it burns (the Wayhouse's yard)", "wayhouse", [], HQ, "the burning is in the Wayhouse's own yard, where his letters wait; he comes to it as he comes every month")],
      acted="wayhouse", giver=("the heart, at the Wall where his letters wait", "wayhouse"),
      clock=([("@door", "wayhouse")], None, None, "the writ's dawn: the player comes from the door's town to the Wayhouse's yard"),
      alts={"R3": ["Investigate · Watch them (the tithe-house)"]},
      plants={"A copy in every hall": "C1-2 'they sleep at {lodging}, and burn at dawn'", "Trade what we know": "RV the inquisitor burns for silence (the old debt published); silence serves him as well"},
      parties={"the branch's old Wall clerk": "C1-1", "the ally": "C1-1", "the Hand": "C1-3", "the inquisitor": "RV"},
      door_needs={"all": ["the branch's old Wall clerk"], "a": [], "b": ["the Hand"]},
      ending_needs={"A copy in every hall": ["the inquisitor"], "Trade what we know": ["the inquisitor"],
                    "Left with the holder": ["the inquisitor"]},
      crisis_blames_heart=False)


def fill_w5b(rng, s, sid):
    vh = [h for h in halls() if PL[h]["territory"] == "t-first" and h not in s["_towns"]]
    s["home_hall"] = pick(rng, vh, MOUNT, JOURNEY, "no First Peoples hall within a journey of the Mount")
    c = s["home_hall"]
    x, y = away(c, nearest([p for p in settlements() if p != c], c), 0.8)
    make(s, "dig", "the scribes' house dig", "Writers dig (spine site)", x, y,
         "no dig on the map in First Peoples land; the placer adds one at the drafting house's ruin", territory="t-first")
    lan = [p for p in settlements() if has(p, "Lantern chapter house") and dist(p, s["dig"]) <= STRETCH]
    s["lantern_hall"] = pick(rng, lan, s["dig"], STRETCH, "no Lantern chapter house near the dig")
    if dist(s["lantern_hall"], s["dig"]) > BOUND:
        s["_why"]["lantern_hall"] = f"stretched to {dist(s['lantern_hall'], s['dig']):.2f}: the nearest chapter house"
    dl = lordship_of(s["dig"])  # draft 4: the lord whose lordship the dig stands in, not the nearest of a kind
    if lord_kind(dl) not in ("Seated House head", "Tide House speaker") or dist(dl, s["dig"]) > STRETCH \
            or not lord_ok(sid, "dig_lord", dl):
        raise NoFit("the dig stands in no Seated or Tide House's lordship")
    s["dig_lord"] = dl
    if dist(s["dig_lord"], s["dig"]) > BOUND:
        s["_why"]["dig_lord"] = f"stretched to {dist(s['dig_lord'], s['dig']):.2f}: the nearest seat (the lordship the dig stands in)"
    rl = [p for p in seats(("Seated House head",)) if p != s["dig_lord"] and p not in s["_lords"]
          and dist(p, c) <= FREE and lord_ok(sid, "reader_lord", p)]
    s["reader_lord"] = pick(rng, rl, c, FREE, "no Seated House with an archive within three days")
    s["mount"] = MOUNT
    return c


spine("W5-B", shape=3, reveal="heart", form="chain", title="The Last Reader", enemy="W5", kind="the old debt",
      heart=("H1", "a", "cast"), centre="home_hall", fill=fill_w5b, lords=("dig_lord", "reader_lord"),
      stake="S1", move="M6",
      slots=[("home_hall", "hall", "a reading room in First Peoples land, where the heart teaches", "centre; ≤3.5 of the Mount"),
             ("dig", "site", "a Writers' dig at the drafting house's ruin, in First Peoples land (added: none there)", "0.8 from the centre"),
             ("lantern_hall", "town", "a Lantern chapter house (the dig's camp's chapter)", "≤1.25 of the dig, else ≤1.6"),
             ("dig_lord", "seat", "the Seated or Tide House head whose lordship holds the dig: the nearest lord's seat in its territory (d-l0)", "≤1.25 of the dig, else ≤1.6"),
             ("reader_lord", "seat", "a Seated House head with an archive, not dig_lord (d-l0); his court wants the roll's words", "≤3.0 of the centre"),
             ("mount", "fixed", "the Mount and its cells", "—"),
             ("feast", "day", "a feast of the Mount's calendar (roller: a feast's name; `{days}` days off when word comes)", "—"),
             ("hand", "person", "the inquisitor's clerk (made on demand; the Mount's)", "—"),
             ("fallback", "person", "her elder pupil (cast)", "—"),
             ("digger", "person", "a digger who heard her read (made on demand)", "at the dig")],
      cards=[("R1", "Acquire · Copy it before it burns (heart's card)", "home_hall", ["home_hall"], B, ""),
             ("R2", "Passage · Smuggling (the lent finds, home from the lord's archive)", "reader_lord", [], F, ""),
             ("R3", "Acquire · Copy it before it burns (odd job: the captain's day-book)", "lantern_hall", ["dig"], B, ""),
             ("RV", "Reveal · Restore Estrangement (the heart asks; her sister from the Mount)", "home_hall", ["home_hall"], J, "her sister comes down from the Mount"),
             ("C1-1", "Dispute · Shared resource (the closed dig)", "dig", ["dig"], J, "out to the dig"),
             ("C1-2", "Acquire · Take the readings (the trench, before it is filled; the turn)", "dig", ["dig"], B, ""),
             ("C1-3", "Passage · Unwilling passenger (the digger who heard her; he asks)", "dig", ["dig"], B, ""),
             ("C2-0", "Investigate · Watch them (the parish chapel)", "dig", ["dig"], B, ""),
             ("DOOR", {"a": "dig", "b": "dig_lord"}, ["dig"], B, ""),
             ("CR", "Rescue · Imprisoned unjustly (at the Mount's gate)", "mount", ["mount"], J, "her pupil follows the inquisitor's riders to the Mount's gate")],
      acted="mount", giver=("her pupil, who has followed her to the Mount's gate", "mount"),
      clock=([("home_hall", "mount")], None, "feast", "word of the recanting comes to her hall; the Mount is that far"),
      days_slots={"feast": DAYS["feast"]},
      alts={"R1": ["Dispute · Public offense (heart's card)"]},
      plants={},
      parties={"the reader lord": "R2", "the dig's lord": "C1-1", "her sister": "RV", "the digger": "C1-3",
               "her pupil": "R1", "the leader": "RV", "the inquisitor": "RV", "the Hand": "C1-2"},
      owns=[("dig", "lordship", "dig_lord")],
      door_needs={"all": ["her sister"], "a": ["the leader", "the reader lord"], "b": ["the dig's lord", "the inquisitor"]},
      ending_needs={"Freed": ["the digger"], "Captive stays by choice": ["the inquisitor"], "Burn your copy": ["the inquisitor"]},
      leaders_ok={"dig_lord": ("he acts once, by his court's writ at the house's asking, as any lord in the house's land may", ()),
                  "reader_lord": ("the spine asks for the blind archivist-lord, whose archive is read to him", ())},
      crisis_blames_heart=False)


# W6 -------------------------------------------------------------------------
def fill_w6a(rng, s, sid):
    pairs = [(h, c) for h in halls() for c in seats(LANDLORDS + ("Tide House speaker",))
             if dist(h, c) <= STRETCH and h not in s["_towns"] and c not in s["_lords"] and lord_ok(sid, "court", c)]
    if not pairs:
        raise NoFit("no hall with a court in reach")
    h, c = rng.choice(pairs)
    s["reading_room"], s["court"] = h, c
    if dist(h, c) > BOUND:
        s["_why"]["court"] = f"stretched to {dist(h, c):.2f}: the nearest court to the hall"
    make(s, "valley", "the valley", "farmland (spine site)", *inland(c, 0.6, rng),
         "the valley of the claims, in the court's lordship", territory=PL[c]["territory"])
    terrs = {PL[h]["territory"], "t-great"} - set(HOSTILE)
    make(s, "tower", "the hoard's tower", "lone tower (spine site)", *remote(h, 1.8, rng, terrs=terrs),
         "a lone, dry building on remote ground, in the hall's own land or Great House land; the map has no scholar's tower here")
    s["region"] = region_word(h)
    return h


spine("W6-A", shape=1, reveal="elder", form="names", title="The Copies Sent for Mending", enemy="W6", kind="the cost",
      heart=("H2", "a", "cast"), centre="reading_room", fill=fill_w6a, lords=("court",), seat_ending="take the road",
      stake="S4", move="M3", about="the Wayhouse and the leader (her order keeps the copy in, at the Wayhouse's door)",
      slots=[("reading_room", "hall", "a reading room in a large town keeping its region's land rolls (S4)", "centre"),
             ("region", "region", "the region of reading_room (roller: the map's region name)", "—"),
             ("court", "seat", "a court holder whose writ binds the valley: Banner/Free Lord, Seated or Tide House head (d-l0); his archivist is his", "≤1.25 of the centre, else ≤1.6 (nearest)"),
             ("valley", "site", "the valley of the claims, in the court's territory (added)", "0.6 from the court's seat"),
             ("tower", "site", "the hoard: a lone dry building on remote ground, with a keeper (added), in the centre's territory or Great House land, never an enclave", "1.8 from the centre"),
             ("hand", "person", "the leader's courier (made on demand)", "—"),
             ("keeper", "person", "the hoard's keeper, a pensioned copyist in the leader's pay (made on demand)", "at the tower"),
             ("tutor", "person", "the old reader at reading_room who taught the leader (made on demand)", "—"),
             ("claimant", "person", "the valley's headwoman (made on demand)", "—")],
      cards=[("R1", "Acquire · Won't sell (the court's archivist)", "court", ["court"], B, ""),
             ("R2", "Dispute · Broken agreement (heart's card: her night-copy pay)", "wayhouse", [], HQ, "she copies the Wayhouse's land rolls by night, for mending"),
             ("R3", "Investigate · The ledger doesn't add up (odd job: the mending book)", "reading_room", ["reading_room"], B, ""),
             ("RV", "Reveal · Acquire Passed on (the fallen roll-case)", "reading_room", ["reading_room"], B, ""),
             ("C1-1", "Acquire · Unknown worth (the hoard)", "tower", [], J, "out to the tower, by the tolls the courier paid"),
             ("C1-2", "Investigate · Watch them (the hall's strongroom)", "reading_room", ["reading_room"], B, ""),
             ("C1-3", "Dispute · Public offense (the courier, on the road out)", "reading_room", ["reading_room"], B, ""),
             ("C2-0", "Acquire · Under seal (the land rolls under her seal)", "wayhouse", [], HQ, "the land rolls and their night copyists are the Wayhouse's"),
             ("DOOR", {"a": "reading_room", "b": "wayhouse"}, [], B, "b: she is found copying the Wayhouse's roll, where she works"),
             ("CR", "Passage · Smuggling (out of the Wayhouse door)", "wayhouse", [], HQ, "the copy must leave the Wayhouse past her courier; the heart works there, and the eldest reads the order there")],
      acted="court", giver=("the eldest copyist, who must read the order aloud", "wayhouse"),
      clock=([("@door", "wayhouse"), ("wayhouse", "court")], None, None,
             "the player comes from the dealt door to the Wayhouse, and the copy must ride from there to the court before it sits"),
      plants={"A copy in every hall": "R2 'the shape-sheets under her bed' and C1-2 'a master who'd sooner copy than send'"},
      parties={"the headwoman": "C1-3", "the tutor": "RV", "the courier (the Hand)": "C1-3", "the eldest copyist": "CR",
               "the hall master": "C1-2", "the leader": "RV", "the archivist": "R1"},
      door_needs={"all": [], "a": ["the courier (the Hand)"], "b": ["the leader"]},
      ending_needs={"Delivered": ["the eldest copyist", "the headwoman"], "A copy in every hall": ["the courier (the Hand)", "the hall master"],
                    "Seized": ["the courier (the Hand)"], "take the road": ["the leader"]},
      owns=[("valley", "lordship", "court"), ("tower", "territory", "centre|t-great")],
      crisis_blames_heart=False)


def fill_w6b(rng, s, sid):
    pairs = [(c, h) for c in seats(LANDLORDS) for h in halls()
             if h != c and dist(h, c) <= STRETCH and holder(c)["age"] == "old"
             and holder(c)["id"] not in LEADERS and c not in s["_lords"] and c not in s["_towns"]
             and size_ok(sid, "lord", c)]
    if not pairs:
        raise NoFit("no old lord (not a faction leader) whose seat can hold an archive, with a hall in reach")
    c, h = rng.choice(pairs)
    s["lord"], s["reading_room"] = c, h
    if dist(h, c) > BOUND:
        s["_why"]["reading_room"] = f"stretched to {dist(h, c):.2f}: the nearest hall to the seat (it keeps his rolls)"
    s["r3_hall"] = pick(rng, [x for x in halls() if x not in (h,)], c, FREE)
    return c


spine("W6-B", shape=3, reveal="elder", form="side", title="The Archive Nobody Copied", enemy="W6", kind="the cost",
      heart=("H4", "c", "cast"), centre="lord", fill=fill_w6b, lords=("lord",), seat_ending="take the road",
      stake="S7", move="M9",
      slots=[("lord", "seat", "an old High holder (d-l0 age 'old'; Banner/Free Lord or Seated House head) with an archive at his seat, a town or larger (draft 5: it must hold the archive tower, a seat's hall and two heirs' households); never a faction leader", "centre"),
             ("reading_room", "hall", "the hall that keeps his rolls: a reading room in another town", "≤1.25 of the seat, else ≤1.6 (nearest)"),
             ("r3_hall", "hall", "any other hall", "≤3.0 of the centre"),
             ("heirs", "people", "his two heirs (made on demand as his kin)", "at the seat"),
             ("hand", "person", "the leader's courier (made on demand)", "—"),
             ("steward", "person", "his steward (made on demand; his)", "at the seat"),
             ("tutor", "person", "reading_room's old reader (made on demand)", "—")],
      cards=[("R1", "Investigate · The ledger doesn't add up (heart's card)", "reading_room", ["reading_room"], B, ""),
             ("R2", "Acquire · Copy it before it burns (odd job: the steward's clerk)", "lord", ["lord"], B, ""),
             ("R3", "Dispute · Contested inheritance (a burned will)", "r3_hall", [], F, ""),
             ("RV", "Reveal · Restore Lost good name (the steward)", "reading_room", ["reading_room", "lord"], B, ""),
             ("C1-1", "Restore · Broken works (the archive tower)", "lord", ["lord"], J, "out to the seat"),
             ("C1-2", "Acquire · Won't sell (the seat's hall; the turn)", "lord", ["lord"], B, ""),
             ("C1-3", "Investigate · Watch them (the archive door; the steward asks)", "lord", ["lord"], B, ""),
             ("C2-0", "Passage · Unwilling passenger (the old lord's ward)", "lord", ["lord"], B, ""),
             ("DOOR", {"a": "reading_room", "b": "reading_room", "c": "reading_room"}, ["reading_room"], B, ""),
             ("CR", "Reckoning · Betrayal (the letter read before the hall)", "reading_room", ["reading_room"], B, "")],
      acted="reading_room", giver=("the old reader who taught the leader, who reads her letter before the hall", "reading_room"),
      alts={"R1": ["Dispute · Broken agreement (heart's card)"]},
      plants={},
      parties={"the steward": "RV", "the heirs": "C1-3", "the tutor": "RV", "the courier (the Hand)": "C1-2", "the leader": "RV"},
      door_needs={"all": [], "a": ["the leader"], "b": ["the leader"]},
      ending_needs={"Amends made": ["the leader", "the steward"], "Open the ledgers": ["the leader"],
                    "Take a side at last": ["the heirs", "the steward"], "take the road": ["the leader"]},
      seat_size={"lord": ("town", "an archive tower, a seat's hall, a steward and two heirs' households")},
      crisis_blames_heart=True)


# W7 -------------------------------------------------------------------------
def fill_w7a(rng, s, sid):
    c = court_town(rng, s)
    s["lord"] = s["reading_room"] = c
    near = [p for p in settlements() if p != c and dist(p, c) <= BOUND]
    own = [p for p in near if lordship_of(p) == c and PL[p]["size"] in ("village", "town")]
    if own:
        s["tenant_village"] = pick(rng, own, c, BOUND)
    else:
        make(s, "tenant_village", "the tenants' village", "village (spine site)", *inland(c, 0.7, rng),
             "no village of the court's own tenants within a day; the placer adds one in its lordship",
             territory=PL[c]["territory"])
    rest = [p for p in near if p != s["tenant_village"]]
    s["son_town"] = pick(rng, rest, c, BOUND) if rest else (near[0] if near else c)
    s["r3_hall"] = pick(rng, [h for h in halls() if h != c], c, FREE, "no other hall within three days")
    return c


spine("W7-A", shape=3, reveal="ally", form="question", title="The Court's New Clerk", enemy="W7", kind="the old debt",
      heart=("H4", "spec", "cast"), centre="reading_room", fill=fill_w7a, lords=("lord",),
      stake="S7", move="M5",
      slots=[("lord", "seat", "a court holder whose seat is a large town with a reading room (S7 a/b/c): Banner/Free Lord, Seated or Tide House head, the Binders' head, Open Council head (d-l0)", "= reading_room's town"),
             ("reading_room", "hall", "that town's reading room ({master}'s hall)", "centre"),
             ("tenant_village", "village", "a village of the court's own tenants, in its lordship (added if none)", "≤1.25 of the centre"),
             ("son_town", "town", "where the master's estranged son lives, within a day of the court (he pays his carting dues at its counter)", "≤1.25 of the centre"),
             ("r3_hall", "hall", "another hall, whose bookbinder bought the Wayhouse's scrap", "≤3.0 of the centre"),
             ("hand", "person", "the court's under-clerk (made on demand; the court's)", "—"),
             ("master", "person", "reading_room's hall master (made on demand; the hall's)", "—"),
             ("kin", "person", "the heart's daughter, a copyist at the same hall (cast with the heart's household)", "—"),
             ("client", "person", "a tenant of the court whose lease rests on the copy (made on demand)", "in tenant_village")],
      cards=[("R1", "Dispute · Contested inheritance (heart's card)", "reading_room", ["reading_room"], B, ""),
             ("R2", "Investigate · The ledger doesn't add up (odd job: the court's day-book)", "lord", ["lord"], B, ""),
             ("R3", "Acquire · Passed on (the cut leaf)", "r3_hall", [], F, ""),
             ("RV", "Reveal · Restore Estrangement (the master and his son)", "son_town", ["reading_room"], B, ""),
             ("C1-1", "Dispute · Shared resource (a field moved by one line)", "tenant_village", ["lord"], J, "out to the tenants' village"),
             ("C1-2", "Passage · Smuggling (the court's gate; the turn)", "lord", ["lord"], B, ""),
             ("C1-3", "Investigate · Watch them (the check room; the son asks)", "lord", ["lord"], B, ""),
             ("C2-0", "Acquire · Unknown worth (the tenant's first copy)", "tenant_village", ["lord"], B, ""),
             ("DOOR", {"a": "reading_room", "b": "reading_room"}, ["reading_room"], B, ""),
             ("CR", "Dispute · Public offense (the hearing at the court)", "lord", ["lord"], B, "")],
      acted="lord", giver=("the tenant whose lease rests on the copy, come to the hall for an advocate", "reading_room"),
      plants={},
      parties={"the tenant (client)": "C1-1", "the heart's daughter": "R1", "the master": "RV", "the master's son": "RV",
               "the commissioner": "C1-2", "the clerk": "RV"},
      owns=[("tenant_village", "lordship", "lord")],
      door_needs={"all": [], "a": ["the clerk"], "b": ["the heart's daughter", "the clerk"]},
      ending_needs={"The heart wins": ["the clerk"], "A deal is struck": ["the master"],
                    "Take a side at last": ["the master", "the tenant (client)"]},
      leaders_ok={"lord": ("the court acts through its commissioner; the lord never hears his own court's copying",
                           ("p-tuhudo",))},
      crisis_blames_heart=True)


def fill_w7b(rng, s, sid):
    c = pick(rng, [h for h in halls() if h not in s["_towns"]], None)
    s["reading_room"] = c
    terrs = {PL[c]["territory"], "t-great"} - set(HOSTILE)
    make(s, "tower", "the struck-off copyist's tower", "lone tower (spine site)", *remote(c, 1.0, rng, 0.5, terrs=terrs),
         "a lone tower on remote ground within reach of the market, in the hall's own land; the map has none")
    near = [p for p in settlements() if p != c and dist(p, c) <= BOUND and market_town(p)]
    s["mother_town"] = pick(rng, near, c, BOUND) if near else c
    s["r3_hall"] = pick(rng, [h for h in halls(True) if h != c], c, FREE, "no other hall within three days")
    return c


spine("W7-B", shape=1, reveal="elder", form="chain", title="The Apprentice's Hand", enemy="W7", kind="the old debt",
      heart=("H2", "W7c", "cast"), centre="reading_room", fill=fill_w7b, lords=(),
      stake="S3", move="M5", about="the Wayhouse's own judging (its hearing bars her hand from every hall, as it struck her teacher off)",
      slots=[("reading_room", "hall", "a reading room in a large town (its market, its door)", "centre"),
             ("tower", "site", "her lone tower on remote ground (added), in the hall's territory or Great House land, never an enclave", "1.0 from the centre"),
             ("mother_town", "town", "where the heart's mother keeps her market stall: a town with a market (no keep, fort, counting-house or enclave); else the centre", "≤1.25 of the centre"),
             ("r3_hall", "hall", "another hall", "≤3.0 of the centre"),
             ("hand", "person", "the tower's agent who sells exemplars (made on demand)", "—"),
             ("scribe", "person", "the Wayhouse's old hearing-scribe (made on demand; the Wayhouse's)", "—")],
      cards=[("R1", "Restore · Lost livelihood (heart's card)", "reading_room", ["reading_room"], B, ""),
             ("R2", "Acquire · Copy it before it burns (odd job: the exemplars)", "reading_room", ["reading_room"], B, ""),
             ("R3", "Investigate · The ledger doesn't add up", "r3_hall", [], F, ""),
             ("RV", "Reveal · Passage Unwilling passenger (the shepherd boy; the hall's master)", "reading_room", ["tower"], B, ""),
             ("C1-1", "Threat · Go and look (the tower)", "tower", ["reading_room"], J, "up to the tower"),
             ("C1-2", "Dispute · Public offense (the market)", "reading_room", ["reading_room"], B, ""),
             ("C1-3", "Acquire · Won't sell (the paper stall)", "reading_room", ["reading_room"], B, ""),
             ("C2-0", "Dispute · Broken agreement (the twelve copies' owners)", "reading_room", ["reading_room"], B, ""),
             ("DOOR", {"a": "mother_town", "b": "reading_room"}, ["reading_room"], B, ""),
             ("CR", "Reckoning · Ruin from above (the Wayhouse's hearing)", "wayhouse", [], HQ, "the Wayhouse's hearing bars her hand: summoned to it, she rides up with the hearing-scribe, and the eldest reads the teacher's letter at the Wall that night")],
      acted="wayhouse", giver=("the eldest copyist, who read the letter at the Wall", "wayhouse"),
      clock=([("@door", "wayhouse")], None, None, "the summons: she rides up from the door's town to the hearing with the scribe"),
      alts={"R3": ["Investigate · Watch them (the exemplars' carrier)"]},
      plants={},
      parties={"the heart's mother": "R1", "the hearing-scribe": "C1-3", "the teacher": "RV", "the old hall master": "RV",
               "the twelve copies' owners": "C1-2"},
      door_needs={"all": ["the hearing-scribe"], "a": ["the heart's mother"], "b": []},
      ending_needs={"Amends made": ["the old hall master", "the teacher"],
                    "Burn your copy": ["the twelve copies' owners"], "The requester forgives": ["the teacher"]},
      owns=[("tower", "territory", "centre|t-great"), ("mother_town", "market", None)],
      crisis_blames_heart=False)


# W8 -------------------------------------------------------------------------
def edge(rng, s):
    camps = [p for p in PL if PL[p]["kind"] == "Writers survey camp"]
    s["camp"] = c = rng.choice(camps)
    s["cairns"] = nearest([p for p in PL if PL[p]["kind"] == "marker cairns"], c)
    s["green_wall"] = "pl-green-wall"
    ev = [p for p in settlements() if PL[p]["size"] == "village" and PL[p]["owner_faction"] != "f-wardens"
          and holder(p) and holder(p)["standing"] in ("Low", "Middle") and dist(p, c) <= BOUND]
    s["edge_village"] = pick(rng, ev, c, BOUND, "no Low or Middle village within a day of the camp")
    lh = [h for h in halls() if dist(h, c) <= STRETCH]
    s["lodge_hall"] = pick(rng, lh, c, STRETCH, "no hall in reach of the camp")
    if dist(s["lodge_hall"], c) > BOUND:
        s["_why"]["lodge_hall"] = f"stretched to {dist(s['lodge_hall'], c):.2f}: the nearest hall, where the camp lodges its surveys"
    return c


def fill_w8a(rng, s, sid):
    c = edge(rng, s)
    # draft 5 (review-v7d 6.3): branch b holds the player at the captain's own watchtower, not the Green Wall
    s["watchtower"] = nearest([p for p in PL if PL[p]["kind"] == "Warden watchtower"], c)
    gt = [p for p in settlements() if PL[p]["size"] in ("large", "great") and dist(p, s["edge_village"]) <= STRETCH]
    s["gate_town"] = pick(rng, gt, s["edge_village"], STRETCH, "no gated town near the village")
    if dist(s["gate_town"], s["edge_village"]) > BOUND:
        s["_why"]["gate_town"] = "stretched: the nearest gated town"
    pt = [p for p in settlements() if (has(p, "Warden recruiting post") or PL[p]["owner_faction"] == "f-wardens")
          and dist(p, c) <= FREE and p != WAYHOUSE]
    used = (s["gate_town"], s["lodge_hall"])
    if pt:
        s["post_town"] = pick(rng, pt, c, FREE, avoid=used)
        if PL[s["post_town"]]["owner_faction"] == "f-wardens" and not has(s["post_town"], "Warden recruiting post"):
            s["_why"]["post_town"] = "the Wardens' own fort, where every widow of the towers draws her pension"
    else:
        s["post_town"] = nearest([h for h in halls() if h not in used] or halls(), c)
        add_feature(s["post_town"], "a Warden recruiting post",
                    "the map's posts are over three days from the edge; the Wardens recruit where the fear is")
    make(s, "timber_yard", "the wall's timber yard", "yard (spine site)", *toward(s["edge_village"], c, 0.4),
         "the yard where the wall's timber is cut, below the village")
    return c


spine("W8-A", shape=2, reveal="leader", form="side", title="The Wall Through the Spring", enemy="W8", kind="the hidden hand",
      heart=("H3", "b", "cast"), centre="camp", main="edge_village", fill=fill_w8a, lords=(),
      stake="S6", move="M5",
      slots=[("camp", "site", "a Writers' survey camp at the green's edge (presence); his sister heads it", "centre"),
             ("cairns", "site", "the Wardens' marker cairns", "nearest to the camp"),
             ("edge_village", "village", "a Low or Middle village (d-l0 holder) within a day of the camp, not a Warden fort", "≤1.25 of the camp"),
             ("green_wall", "fixed", "the Green Wall (f-wardens; held by d-l0's Haʻo)", "—"),
             ("gate_town", "town", "a large town with a gate", "≤1.25 of the village, else ≤1.6"),
             ("lodge_hall", "hall", "the hall where the camp lodges its surveys, which must certify the season's maps", "≤1.25 of the camp, else ≤1.6 (nearest)"),
             ("post_town", "town", "a Warden recruiting post (presence) or the Wardens' own fort, not the Wayhouse's town; else one added at the nearest hall (draft 4: not a town the routine already uses)", "≤3.0 of the camp"),
             ("timber_yard", "site", "the wall's timber yard (added)", "0.4 from the village"),
             ("watchtower", "site", "the captain's Warden watchtower, nearest the camp (f-wardens)", "nearest"),
             ("captain", "person", "a Warden captain of the towers here, under the Green Wall's holder (made on demand)", "—"),
             ("hand", "person", "lodge_hall's checker (made on demand; the hall's)", "—"),
             ("surveyor", "person", "a camp surveyor who signed under protest (made on demand; the camp's)", "—")],
      cards=[("R1", "Passage · Unwelcome cargo (heart's card: the gate)", "gate_town", ["edge_village"], B, ""),
             ("R2", "Investigate · The ledger doesn't add up (odd job: the survey)", "lodge_hall", ["camp"], B, ""),
             ("R3", "Acquire · Unknown worth (the Warden's widow)", "post_town", [], F, ""),
             ("RV", "Reveal · Acquire Won't sell (the sister's daughter, come out to the camp)", "camp", ["camp"], B, ""),
             ("C1-1", "Acquire · Take the readings (the cairns; the surveyor met)", "cairns", ["camp"], J, "out to the cairns"),
             ("C1-2", "Dispute · Shared resource (the spring meadow)", "edge_village", ["camp"], B, ""),
             ("C1-3", "Rescue · Imprisoned unjustly (the chain-boy held at the camp)", "camp", ["camp"], B, ""),
             ("C2-0", "Dispute · Broken agreement (the timber yard's hiring)", "timber_yard", ["edge_village"], B, ""),
             ("DOOR", {"a": "edge_village", "b": "green_wall"}, ["camp"], B, ""),
             ("CR", "Threat · Encroachment (the spring meadow)", "edge_village", ["camp"], B, "")],
      acted="edge_village", giver=("the hall master who must certify the maps, come out to see the line", "edge_village"),
      clock=([("edge_village", "lodge_hall")], None, None, "the maps must be certified at the hall before the ditch reaches the spring"),
      plants={},
      parties={"the heart's son": "R1", "the oldest herder": "C1-2", "the headman": "C1-2", "the surveyor": "C1-1",
               "the captain's sister": "RV", "the captain": "RV"},
      door_needs={"all": ["the oldest herder"], "a": ["the heart's son", "the captain"], "b": ["the headman", "the captain"]},
      ending_needs={"Threat ended": ["the surveyor", "the captain"], "Bargain struck": ["the captain's sister"],
                    "Protected, the threat remains": ["the captain"]},
      crisis_blames_heart=False)


def fill_w8b(rng, s, sid):
    c = edge(rng, s)
    s["watchtower"] = nearest([p for p in PL if PL[p]["kind"] == "Warden watchtower"], c)
    make(s, "hut", "the old surveyor's hut beyond the towers", "hut (spine site)", *toward(s["watchtower"], "pl-wellstone", 0.6),
         "beyond the tower line, toward the green")
    make(s, "true_edge", "the true edge", "edge (spine site)", *toward(c, "pl-wellstone", 0.6),
         "half a day's walk out from the redrawn line")
    lh = [h for h in halls() if has(h, "Lantern chapter house") and dist(h, c) <= STRETCH]
    s["lantern_hall"] = pick(rng, lh, c, STRETCH, "no Lantern chapter house in reach of the camp")
    if dist(s["lantern_hall"], c) > BOUND:
        s["_why"]["lantern_hall"] = f"stretched to {dist(s['lantern_hall'], c):.2f}: the nearest Lantern chapter house"
    wt = [p for p in settlements() if dist(p, c) <= BOUND and PL[p]["owner_faction"] != "f-wardens"]
    s["widow_town"] = pick(rng, wt, c, BOUND, "no village near the camp")
    near = [h for h in halls() if dist(h, c) <= FREE]
    s["r3_hall"] = pick(rng, [h for h in near if h != s["lodge_hall"]] or near, c, FREE,
                        "no hall within three days of the camp", avoid=(s["lantern_hall"],))
    eb = [p for p in wt if p != s["edge_village"] and PL[p]["size"] in ("village", "town")]
    if eb:
        s["edge_village_b"] = pick(rng, eb, c, BOUND)
    else:
        make(s, "edge_village_b", "the edge hamlet on the road out", "hamlet (spine site)",
             *toward(c, s["lodge_hall"], 0.5), "no second village within a day of the camp; the placer adds a hamlet on the road to the hall")
    return c


spine("W8-B", shape=1, reveal="heart", form="question", motif="lost surveyor", title="The Mother's Journal", enemy="W8", kind="the hidden hand",
      heart=("H5", "a", "man (the son)"), centre="camp", main="edge_village", fill=fill_w8b, lords=(),
      stake="S6", move="M6",
      slots=[("camp", "site", "a Writers' survey camp at the green's edge (presence)", "centre"),
             ("watchtower", "site", "a Warden watchtower above the camp (f-wardens)", "nearest"),
             ("green_wall", "fixed", "the Green Wall (f-wardens; held by d-l0's Haʻo)", "—"),
             ("edge_village", "village", "a Low or Middle village within a day of the camp", "≤1.25 of the camp"),
             ("edge_village_b", "village", "a second settlement within a day of the camp, not a Warden fort (added as a hamlet on the road to the hall if none)", "≤1.25 of the camp"),
             ("hut", "site", "the old surveyor's hut beyond the towers (added)", "0.6 past the watchtower"),
             ("true_edge", "site", "the true edge (added)", "0.6 from the camp"),
             ("lodge_hall", "hall", "the hall nearest the camp, where the camps lodge their journals and the lords' stewards vote the levy", "≤1.25, else ≤1.6"),
             ("lantern_hall", "hall", "a reading room with a Lantern chapter house (the old escorts)", "≤1.25, else ≤1.6"),
             ("widow_town", "town", "a settlement near the camp, not a Warden fort", "≤1.25 of the camp"),
             ("r3_hall", "hall", "a hall keeping the year's copy of the Wayhouse's edge-map register (every hall keeps the year's copy), not lodge_hall where one is in reach", "≤3.0 of the camp"),
             ("vote_day", "day", "the day the stewards vote the levy (roller: a day name; `{days}` days off at the last move)", "—"),
             ("captain", "person", "a Warden captain of the towers here (made on demand)", "—"),
             ("hand", "person", "the captain's escort sergeant (made on demand; the Wardens')", "—"),
             ("uncle", "person", "the heart's mother's brother, a sergeant of the Green Wall (cast with the heart's household)", "at the Green Wall"),
             ("old_surveyor", "person", "an old surveyor who learned the mother's cipher on the surveys they walked (made on demand)", "at the hut")],
      cards=[("R1", "Restore · Lost good name (heart's card: his mother)", "lodge_hall", ["camp"], B, ""),
             ("R2", "Acquire · Copy it before it burns (odd job: the widow)", "widow_town", ["camp"], B, ""),
             ("R3", "Investigate · The ledger doesn't add up (the register's year copy)", "r3_hall", [], F, ""),
             ("RV", "Reveal · Dispute Broken agreement (the Lantern escorts; the heart asks)", "lantern_hall", ["camp"], B, ""),
             ("C1-1", "Investigate · Watch them (the tower)", "watchtower", ["camp"], J, "up to the tower above the camp"),
             ("C1-2", "Acquire · Take the readings (the true edge)", "true_edge", ["camp"], B, ""),
             ("C1-3", "Dispute · Public offense (the Green Wall)", "green_wall", ["camp"], B, ""),
             ("C2-0", "Passage · Unwelcome cargo (the second village's inn)", "edge_village_b", ["camp"], B, ""),
             ("DOOR", {"a": "green_wall", "b": "edge_village"}, ["camp"], B, ""),
             ("CR", "Passage · Samples from the green (from the hut)", "hut", ["camp"], B, "")],
      acted="hut", giver=("the heart, at the hut, his mother's last page just read to him", "hut"),
      clock=([("hut", "edge_village_b"), ("edge_village_b", "edge_village"), ("edge_village", "lodge_hall")], None, "vote_day", "the journals must go from the hut, through both villages, to the hall's door"),
      days_slots={"vote_day": DAYS["vote_day"]},
      alts={"R3": ["Dispute · Broken agreement (the register's carriage fee)"]},
      plants={},
      parties={"the uncle": "C1-3", "the old surveyor": "C1-3", "the second village": "C2-0", "the widow": "R2",
               "the sergeant (the Hand)": "C1-3", "the captain": "RV"},
      door_needs={"all": ["the old surveyor"], "a": ["the sergeant (the Hand)"], "b": ["the uncle"]},
      ending_needs={"Delivered": ["the old surveyor", "the captain"], "Delivered late, or in part": ["the second village"],
                    "Seized": ["the captain"]},
      crisis_blames_heart=False)

ORDER = list(SPINES)
FIXED_SLOTS = {"wayhouse", "wall", "strongroom", "eldest", "leader", "enemy", "heart", "days", "mount",
               "magistrate", "green_wall", "region"}


# --- small readers ------------------------------------------------------------
def premise(what):
    """'Investigate · Watch them (the branch)' -> 'Investigate · Watch them'."""
    return what.split(" (")[0].strip()


def family(what):
    return premise(what).split(" · ")[0]


def blow_key(text):
    t = text.lower()
    for k in ("heart disgraced", "heart estranged", "heart captive", "player framed", "player stripped",
              "player arrested", "hall seized", "members taken", "outlawed", "leader struck down",
              "stalls' room shut"):
        if k in t:
            return {"stalls' room shut": "hall seized"}.get(k, k)
    return t


def card(sp, cid):
    for c in sp["cards"]:
        if c[0] == cid:
            return c
    return None


def openings(sp):
    return [premise(card(sp, "C1-1")[1]), premise(card(sp, "C2-0")[1])]


def opening_keys(sp):
    """Memory keys for the chapter openings: each premise in its position."""
    a, b = openings(sp)
    return ["ch1 " + a, "ch2 " + b]


def open_count(xs, key):
    p = key[4:]
    if p in OPEN_CAP:  # Unwelcome cargo: counted in either position
        return sum(1 for x in xs if x[4:] == p)
    return xs.count(key)


def forced_routine(sp):
    """Routine premises a spine always uses (cards without swaps)."""
    return {premise(c[1]) for c in sp["cards"] if c[0] in ("R1", "R2", "R3") and c[0] not in sp["alts"]}


def routine_options(sp, cid):
    c = card(sp, cid)
    return [c[1]] + sp["alts"].get(cid, [])


# --- static checks (the spine set) ------------------------------------------
HARD_CLOCK = re.compile(r"\b(?:one|two|three|four|five|six|seven|\d+)\s+days?\s+(?:off|away|later|hence)\b"
                        r"|\bin\s+(?:one|two|three|four|five|six|seven|\d+)\s+days\b"
                        r"|\bthe night before (?:the|`?\{)", re.I)


def spine_sections():
    md = open(SPINES_MD, encoding="utf-8").read()
    out = {}
    for m in re.finditer(r"^## (W\d-[AB]) · .*?$(.*?)(?=^## |\Z)", md, re.M | re.S):
        out[m.group(1)] = m.group(2)
    return md, out


def static_checks():
    fails = []
    types = {}
    md, sections = spine_sections()
    for sid, sp in SPINES.items():
        types.setdefault(sp["heart"][0], []).append(sid)
        for k, (txt, _door) in sp["blows"].items():
            if sp["kind"] == "the hidden hand" and "wounded-men" in txt:
                fails.append(f"{sid} blow {k}: a hidden-hand enemy openly wounds (wounded-men)")
            if sp["crisis_blames_heart"] and "cried-down" in txt:
                fails.append(f"{sid} blow {k}: cried-down, and the crisis blames the heart again")
        ids = {c[0] for c in sp["cards"]}
        for ax, (voice, opts) in sp["flavour"].items():
            if len(opts) < 2:
                fails.append(f"{sid} flavour {ax}: one option only")
            if voice not in ids and voice != "MOVE":
                fails.append(f"{sid} flavour {ax}: voiced on {voice}, which is no card of the spine")
        for e in sp["endings"]:
            if e in ("A copy in every hall", "Trade what we know") and e not in sp["plants"]:
                fails.append(f"{sid} ending '{e}' has no earlier plant")
        if len(sp["endings"]) != 3:
            fails.append(f"{sid}: {len(sp['endings'])} endings; draft 5 has exactly three")
        routine = [c for c in sp["cards"] if c[0] in ("R1", "R2", "R3")]
        hearts = [c[0] for c in routine if "heart's card" in c[1]]
        odd = [c[0] for c in routine if "odd job" in c[1]]
        if len(hearts) != 1 or len(odd) != 1:
            fails.append(f"{sid}: routine needs one heart's card and one odd job ({hearts}, {odd})")
        elif odd[0] == "R1" or odd == hearts:
            fails.append(f"{sid}: the odd job is the first routine card or the heart's")
        for cid in sp["alts"]:
            if cid in odd:
                fails.append(f"{sid}: {cid} is the odd job, whose premise the kit forces; it has no swaps")
        chapter = {premise(card(sp, c)[1]) for c in ("C1-1", "C1-2", "C1-3", "C2-0")}
        for c in ("R1", "R2", "R3"):
            for w in routine_options(sp, c):
                if premise(w) in chapter:
                    fails.append(f"{sid}: routine {c} ({premise(w)}) repeats a chapter card's premise")
        if not any(routine_ok(sp, combo) for combo in routine_combos(sp)):
            fails.append(f"{sid}: no routine combination keeps the phase rules")
        # chapter families
        ch1 = [family(card(sp, c)[1]) for c in ("C1-1", "C1-2", "C1-3")]
        if len(set(ch1)) < 3:
            fails.append(f"{sid}: chapter 1 repeats a family ({ch1})")
        turn = "C1-2" if sp["shape"] == 3 else "C1-3"
        if family(card(sp, turn)[1]) == "Investigate":
            fails.append(f"{sid}: chapter 1's turn card ({turn}) is Investigate")
        c20 = family(card(sp, "C2-0")[1])
        for k, (_t, door) in sp["blows"].items():
            if door.split()[0] == c20:
                fails.append(f"{sid}: chapter 2's opening and door {k} share a family ({c20})")
        # the crisis where the story is
        cr = card(sp, "CR")
        if cr[4] == HQ and not sp["about"]:
            fails.append(f"{sid}: the crisis is pinned to the Wayhouse, and the spine is not about the Wayhouse or the leader")
        if cr[4] == HQ and not cr[5]:
            fails.append(f"{sid}: the crisis is at the Wayhouse with no reason its people are there")
        # parties set up first
        for opt, needs in list(sp["door_needs"].items()) + list(sp["ending_needs"].items()):
            for p in needs:
                if p not in sp["parties"]:
                    fails.append(f"{sid}: '{p}' ({opt}) has no card where it is first set up")
        for p, cid in sp["parties"].items():
            if cid.split(":")[0] not in ids:
                fails.append(f"{sid}: party '{p}' is set up on {cid}, which is no card of the spine")
        # the spine's text: slots, hard clocks, region names
        text = sections.get(sid)
        if text is None:
            fails.append(f"{sid}: no section in spines.md")
            continue
        known = {s[0] for s in sp["slots"]} | FIXED_SLOTS | set(sp["days_slots"]) | set(sp.get("flavour_slots", []))
        for slot in sorted(set(re.findall(r"\{([a-z_0-9]+)\}", text))):
            if slot not in known:
                fails.append(f"{sid}: spine text uses {{{slot}}}, which the roller neither fills nor casts")
        for m in HARD_CLOCK.finditer(text):
            fails.append(f"{sid}: a hard-numbered deadline in the text ('{m.group(0)}'); use {{days}}")
        if re.search(r"\bVale\b", text):
            fails.append(f"{sid}: a region name hard-coded ('Vale'); use {{region}} or a slot")
        if sp["clock"] and "{days}" not in text:
            fails.append(f"{sid}: the spine has a clock but its text never says {{days}}")
        fails += draft4_checks(sid, sp, text)
        fails += d5_static(sid, sp)
    for t, ids in types.items():
        if len(ids) > 3:
            fails.append(f"heart type {t} in {len(ids)} spines ({', '.join(ids)}); cap 3")
    for key, table, cap in (("reveal", GIVERS, 5), ("form", FORMS, 4), ("shape", SHAPES, 6)):
        for k in table:
            n = sum(1 for sp in SPINES.values() if sp[key] == k)
            if n > cap:
                fails.append(f"{key} '{k}' in {n} of 16 spines; cap {cap}")
    shape_copy = [sid for sid, t in sections.items() if re.search(r"copies[^.;]{0,40}\bby shape\b", t)]
    if len(shape_copy) > 1:
        fails.append(f"a copyist who copies by shape in {len(shape_copy)} spines ({', '.join(shape_copy)}); keep one")
    fails = [f for f in fails if f]
    return fails, types


PLACE_WORDS = sorted({re.sub(r"^the ", "", p["name"]) for p in MAP["places"]}
                     | {re.sub(r"^the ", "", r["name"]) for r in MAP["regions"]}, key=len, reverse=True)
FACTION_WORDS = ("Sharp Hands", "Binders", "Open Hand", "Night Market", "Market notes", "Market", "Lantern",
                 "Last Wave", "Old Hands", "Far Sails", "Highfolk", "Rooted", "Holy House")


def field(text, n):
    """A spine's field n ('**7 · ...' up to the next numbered field)."""
    m = re.search(rf"^\*\*{n} · (.*?)(?=^\*\*\d+ · |\Z)", text, re.M | re.S)
    return m.group(1) if m else ""


def draft4_checks(sid, sp, text):
    """review-v7c's patterns, checked on the spine's own text and data."""
    f = []
    # pattern 2: the reveal shows, then adds (with who and how); its giver and form are data
    f7 = field(text, 7)
    m = re.match(r"Reveal\*\* \(giver: \*([^*]+)\*; form: \*([^*]+)\*\)", f7)
    if not m:
        f.append(f"{sid}: field 7 does not open '(giver: *…*; form: *…*)'")
    else:
        if m.group(1) != GIVERS[sp["reveal"]]:
            f.append(f"{sid}: field 7's giver '{m.group(1)}' is not the roller's '{GIVERS[sp['reveal']]}'")
        if m.group(2) != FORMS[sp["form"]]:
            f.append(f"{sid}: field 7's form '{m.group(2)}' is not the roller's '{FORMS[sp['form']]}'")
    for w in ("**Shows:**", "**Adds:**", "**Repeats:**", "**Closes:**"):
        if w not in f7:
            f.append(f"{sid}: field 7 lacks {w}")
    if "all point at" in text:
        f.append(f"{sid}: the formula reveal sentence ('all point at')")
    # chapter 1's shape: the links and where the ally is met
    f8 = field(text, 8)
    if f"*Shape:* {sp['shape']} · {SHAPES[sp['shape']]}" not in f8:
        f.append(f"{sid}: field 8 does not name shape {sp['shape']} · {SHAPES[sp['shape']]}")
    rows = {r.split("|")[1].strip().replace(" turn", ""): r for r in f8.splitlines() if r.startswith("| C1-")}
    links = {1: ("therefore", "but"), 2: ("therefore", "but"), 3: ("but", "therefore")}[sp["shape"]]
    for cid, want in zip(("C1-2", "C1-3"), links):
        cells = [c.strip() for c in rows.get(cid, "").split("|")]
        if len(cells) < 5 or cells[4] != want:
            f.append(f"{sid}: {cid}'s link is not '{want}' (shape {sp['shape']})")
    turn = "C1-2" if sp["shape"] == 3 else "C1-3"
    if " turn |" not in rows.get(turn, ""):
        f.append(f"{sid}: {turn} is shape {sp['shape']}'s turn and is not marked 'turn'")
    ally_at = {1: ("C1-3", "**Ally plant:**"), 2: ("C1-1", "**Ally plant:**"), 3: ("C1-3", "**Ally asks:**")}[sp["shape"]]
    if ally_at[1] not in rows.get(ally_at[0], ""):
        f.append(f"{sid}: shape {sp['shape']} meets the ally on {ally_at[0]} ({ally_at[1]}), and it isn't there")
    for cid, r in rows.items():
        if cid != ally_at[0] and ("**Ally plant" in r or "**Ally asks" in r):
            f.append(f"{sid}: an ally plant on {cid}, outside shape {sp['shape']}'s place for it")
    # pattern 3: what the player holds; pattern 5: the stripped player's standing
    f10 = field(text, 10)
    if "**Held:**" not in f10 and "- **HELD:**" not in f10:
        f.append(f"{sid}: field 10 has no **Held:** line (why the payload lands though the player holds a warning)")
    if not sp.get("d5") and any("(stripped)" in v[0] for v in sp["blows"].values()) and "**Standing:**" not in f10:
        f.append(f"{sid}: a blow strips the player, and field 10 has no **Standing:** line")
    # pattern 6: flavour holds wherever it is placed (draft 5: spine5.flavour_fault, stricter, instead)
    for ax, (_v, opts) in ({} if sp.get("d5") else sp["flavour"]).items():
        for o in opts:
            for w in PLACE_WORDS:
                if re.search(rf"\b{re.escape(w)}\b", o):
                    f.append(f"{sid}: flavour {ax} option '{o}' names a place ('{w}')")
                    break
            if seasons_in(o):
                f.append(f"{sid}: flavour {ax} option '{o}' carries a season word")
            for w in FACTION_WORDS:
                if w in o and "(" not in o and f", the " not in o:
                    f.append(f"{sid}: flavour {ax} option '{o}' names the {w} without a gloss")
                    break
    return f


def routine_combos(sp):
    opts = [routine_options(sp, c) for c in ("R1", "R2", "R3")]
    return [(a, b, c) for a in opts[0] for b in opts[1] for c in opts[2]]


def routine_ok(sp, combo):
    fams = [family(w) for w in combo]
    if any(fams[i] == fams[i + 1] for i in range(2)):
        return False
    if any(fams.count(f) > 2 for f in fams):
        return False
    return any(premise(w) in SIGNATURE for w in combo)


# --- memory and the look-ahead -----------------------------------------------
def heart_cap(t):
    return HEART_CAP.get(t, 2)


def open_cap(p):
    return OPEN_CAP.get(p, 2)


def allowed(sp, mem, tier=0):
    """Can this spine come next, given the set so far? Tier 0 keeps every
    memory rule; tier 1 drops the 'not running' rules; tier 2 the counts."""
    if sp["id"] in mem["spines"]:
        return False
    if tier < 3 and sp["enemy"] in mem["enemies"]:
        return False
    t = sp["heart"][0]
    if tier < 2:
        if mem["hearts"].count(t) >= heart_cap(t):
            return False
        if sp["motif"] and mem["motifs"].count(sp["motif"]) >= MOTIF_CAP.get(sp["motif"], 2):
            return False
        if mem["shapes"].count(sp["shape"]) >= SHAPE_CAP:
            return False
        if mem["reveals"].count(sp["reveal"]) >= REVEAL_CAP.get(sp["reveal"], 4):
            return False
        if mem["forms"].count(sp["form"]) >= FORM_CAP:
            return False
        if sp["hq_forced"] and sum(mem["hq"]) >= HQ_CAP:
            return False
        if mem["stakes"].count(sp["stake"]) >= STAKE_CAP:
            return False
        if mem["moves"].count(sp["move"]) >= MOVE_CAP:
            return False
        if mem["moves"] and mem["moves"][-1] == sp["move"] and tier < 1:
            return False
        for k in opening_keys(sp):
            if open_count(mem["openings"] + opening_keys(sp), k) > open_cap(k[4:]):
                return False
        for p in forced_routine(sp):
            if p in ROUTINE_CAP and mem["routine"].count(p) >= ROUTINE_CAP[p]:
                return False
        if not [k for k in blow_keys(sp) if mem["blows"].count(k) < BLOW_CAP
                and (tier >= 1 or not mem["blows"] or mem["blows"][-1] != k)]:
            return False
    if tier < 1:
        if t in mem["hearts"][-HEART_GAP:]:
            return False
        if mem["stakes"] and mem["stakes"][-1] == sp["stake"]:
            return False
    return True


def blow_keys(sp):
    return [blow_key(v[0]) for v in sp["blows"].values()]


def take(mem, sp, blow, routine):
    m = {k: list(v) for k, v in mem.items()}
    m["spines"].append(sp["id"])
    m["enemies"].append(sp["enemy"])
    m["hearts"].append(sp["heart"][0])
    m["stakes"].append(sp["stake"])
    m["moves"].append(sp["move"])
    m["openings"] += opening_keys(sp)
    m["blows"].append(blow)
    m["routine"] += sorted({premise(w) for w in routine})
    m["shapes"].append(sp["shape"])
    m["reveals"].append(sp["reveal"])
    m["forms"].append(sp["form"])
    m["motifs"] += [sp["motif"]] if sp["motif"] else []
    m["hq"].append(sp["hq_forced"])  # story_of sets the story's real value once it is placed
    return m


def _state(mem):
    def counts(xs):
        return tuple(sorted({x: xs.count(x) for x in xs}.items()))
    return (frozenset(mem["enemies"]), frozenset(mem["spines"]), counts(mem["hearts"]), tuple(mem["moves"][-1:]),
            tuple(mem["hearts"][-HEART_GAP:]), counts(mem["stakes"]), tuple(mem["stakes"][-1:]),
            counts(mem["moves"]), counts(mem["openings"]), counts(mem["blows"]), tuple(mem["blows"][-1:]),
            counts(mem["routine"]), counts(mem["shapes"]), counts(mem["reveals"]), counts(mem["forms"]),
            counts(mem["motifs"]), sum(mem["hq"]))


def can_finish(mem, left, memo=None):
    """Is there some order and choice of spines (one per enemy left) that keeps
    every tier-0 rule? Placement is not looked at here."""
    if left <= 0:
        return True
    memo = {} if memo is None else memo
    key = (_state(mem), left)
    if key in memo:
        return memo[key]
    ok = False
    for sid in ORDER:
        sp = SPINES[sid]
        if not allowed(sp, mem, 0):
            continue
        for k in dict.fromkeys(blow_keys(sp)):
            if mem["blows"].count(k) >= BLOW_CAP or (mem["blows"] and mem["blows"][-1] == k):
                continue
            if can_finish(take(mem, sp, k, list(forced_routine(sp))), left - 1, memo):
                ok = True
                break
        if ok:
            break
    memo[key] = ok
    return ok


def empty_memory():
    return dict(spines=[], enemies=[], hearts=[], stakes=[], moves=[], openings=[], blows=[], routine=[],
                lords=[], towns=[], endings=[], shown=[], shapes=[], reveals=[], forms=[], motifs=[], hq=[])


# --- the per-story checks ----------------------------------------------------
def place_of(s, slot):
    return WAYHOUSE if slot == "wayhouse" else s[slot]


def cards_of(sp):
    """Every card as (id, what, place-slot, anchors, mode, note); a door's
    place-slot is its {blow: slot} map."""
    for c in sp["cards"]:
        if c[0] == "DOOR" and len(c) == 5:
            cid, what, anchors, mode, note = c
            yield cid, what, None, anchors, mode, note
        else:
            yield c


DEALT = ["R1", "R2", "R3", "RV", "C1-1", "C1-2", "C1-3", "C2-0", "DOOR", "CR"]


def check_story(sp, s, blow, endings_rolled):
    """Return (report rows, fails)."""
    rows, fails = [], []
    centre = place_of(s, sp["centre"])
    for c in cards_of(sp):
        cid, what, where, anchors, mode, note = c
        if cid == "DOOR":
            where = what[blow]
            what = f"door (blow {blow}): {sp['blows'][blow][1]}"
        elif cid in s["_routine"]:
            what = s["_routine"][cid]
        p = place_of(s, where)
        dc = dist(p, centre)
        mark = []
        if mode == HQ or p == WAYHOUSE and where == "wayhouse":
            if p != WAYHOUSE:
                fails.append(f"{cid}: an hq card off the Wayhouse")
            if not note:
                fails.append(f"{cid}: at the Wayhouse with no reason for its people to be there")
            if cid == "CR" and not sp["about"]:
                fails.append("the crisis is at the Wayhouse in a story that is not about the Wayhouse or the leader")
            mark.append("Wayhouse (fixed)" + (f": {note}" if cid == "CR" else ""))
        else:
            for a in anchors:
                da = dist(p, place_of(s, a))
                stretched = any(k in s["_why"] for k in (where, a))
                if da <= BOUND + 1e-9:
                    mark.append(f"{da:.2f} from {a} ✓")
                elif da <= STRETCH + 1e-9 and stretched:
                    mark.append(f"{da:.2f} from {a} ✓ stretched")
                else:
                    mark.append(f"{da:.2f} from {a} ✗")
                    fails.append(f"{cid} at {name(p)} is {da:.2f} from {a} ({name(place_of(s, a))}); bound {BOUND}")
            lim = JOURNEY if mode == J else FREE
            if dc > lim + 1e-9:
                fails.append(f"{cid} at {name(p)} is {dc:.2f} from the centre; {mode} limit {lim}")
            if mode == J and not note:
                fails.append(f"{cid}: a journey with no reason")
            if mode == F:
                mark.append(f"free, ≤{FREE}")
            if mode == J:
                mark.append(f"journey, ≤{JOURNEY}: {note}")
        if cid == "CR":
            if p == WAYHOUSE and centre != WAYHOUSE and not sp["about"]:
                fails.append("the crisis falls at the Wayhouse, away from the story")
            gp = place_of(s, sp["giver"][1])
            if dist(gp, p) > BOUND + 1e-9:
                fails.append(f"the crisis giver is at {name(gp)}, {dist(gp, p):.2f} from the crisis at {name(p)}")
            mark.append(f"giver there: {sp['giver'][0]}")
        rows.append((cid, what, name(p), dc, "; ".join(mark)))
    ap = place_of(s, sp["acted"])
    rows.append(("CR*", "crisis acted at", name(ap), dist(ap, centre),
                 "where its action falls" + ("" if ap != place_of(s, card(sp, "CR")[2]) else " (the crisis card's own place)")))
    if dist(ap, centre) > JOURNEY + 1e-9 and not (sp["about"] and ap == WAYHOUSE):
        fails.append(f"the crisis's action at {name(ap)} is {dist(ap, centre):.2f} from the centre; limit {JOURNEY}")
    # slots: kinds, owners, holders from d-l0; every non-person slot filled
    for slot, kind, need, _bound in sp["slots"]:
        if slot not in s:
            if kind not in ("person", "people", "place", "fixed"):
                fails.append(f"slot {slot} ({kind}) left unfilled")
            continue
        p = s[slot]
        if kind in ("day", "house", "region"):
            if not p:
                fails.append(f"slot {slot} left empty")
            continue
        if kind == "seat":
            h = holder(p)
            if not h or h["id"] not in PEOPLE:
                fails.append(f"slot {slot}: {name(p)} has no d-l0 holder")
            elif lord_kind(p) is None:
                fails.append(f"slot {slot}: {name(p)}'s holder is not a lord or court")
        if kind == "hall" and not (has(p, "Writers reading room") or p == WAYHOUSE):
            fails.append(f"slot {slot}: {name(p)} has no reading room")
        if kind == "stall" and not has(p, "letter-writer's stall"):
            fails.append(f"slot {slot}: {name(p)} has no letter-writer's stall")
        if p.startswith("new-") and not PL[p].get("created"):
            fails.append(f"slot {slot}: added without a reason")
    if "lodging" in [x[0] for x in sp["slots"]] and "lodging" not in s:
        fails.append("slot lodging (from flavour b) left unfilled")
    # the clock
    if sp["clock"]:
        if "days" not in s or s["days"] < 2:
            fails.append("the clock: {days} not filled from the map")
        else:
            total, _legs = clock_total(sp, s)
            if s["days"] < total:
                fails.append(f"the clock: {s['days']} days is less than its legs ({total:.2f})")
            if s["days"] != days_for(total):
                fails.append(f"the clock: {s['days']} days is not its legs ({total:.2f}) rounded up + 1")
    # parties set up before they are used
    before_door = DEALT[:DEALT.index("DOOR")]
    for p in sp["door_needs"].get("all", []) + sp["door_needs"].get(blow, []):
        cid = sp["parties"].get(p, "?")
        if cid not in before_door:
            fails.append(f"door {blow}: '{p}' is first set up on {cid}, not on a card dealt before the door")
    for e in endings_rolled:
        for p in sp["ending_needs"].get(e, []):
            cid = sp["parties"].get(p, "?")
            if cid not in DEALT and cid != f"DOOR:{blow}":
                fails.append(f"ending '{e}': '{p}' is first set up on {cid}, which was not dealt")
    # flavour voiced on a dealt card
    for ax, (voice, _o) in sp["flavour"].items():
        if voice not in DEALT and voice != "MOVE":
            fails.append(f"flavour {ax}: voiced on {voice}, not dealt")
    # draft 4: the story's season words fit one season or two that follow each other
    if not seasons_fit(s.get("_seasons", ())):
        fails.append(f"season words clash: {s['_seasons']}")
    # routine phase rules
    combo = tuple(s["_routine"][c] for c in ("R1", "R2", "R3"))
    if not routine_ok(sp, combo):
        fails.append(f"routine breaks the phase rules: {[premise(w) for w in combo]}")
    sid = sp["id"]
    # draft 5: lord slots take no faction leader unless the spine asks; seats hold what the plot needs
    for slot, kind, _need, _b in sp["slots"]:
        if kind == "seat" and slot in s and not lord_ok(sid, slot, s[slot]):
            fails.append(f"slot {slot}: {name(s[slot])}'s holder is a faction leader, and the spine doesn't ask for one there")
        if slot in s and kind in ("seat", "hall", "town", "village") and not size_ok(sid, slot, s[slot]):
            fails.append(f"slot {slot}: {name(s[slot])} is a {PL[s[slot]]['size']}, too small for "
                         f"{sp['seat_size'][slot][1]}")
    # draft 5: the blow's captor against where the crisis falls
    d5 = sp.get("d5")
    if d5 and blow in d5["branches"]:
        b5 = d5["branches"][blow]
        cap = b5["tags"].get("captor", [])
        if cap:
            cp = place_of(s, cap[0])
            for where in (card(sp, "CR")[2], sp["acted"]):
                wp = place_of(s, where)
                if (wp == cp or lordship_of(wp) == lordship_of(cp)) and "at-captor" not in b5["lines"]["STANDING"][1]:
                    fails.append(f"branch {blow}: the player is held by {name(cp)}, the crisis is at {name(wp)} in "
                                 f"the same lordship, and STANDING doesn't say on what footing the player is there")
    # spine-specific owner checks
    if sid in ("W2-A", "W2-B"):
        if PL[s["dig"]]["territory"] != PL[s["lord"]]["territory"] or lord_kind(s["lord"]) != "Free Lord":
            fails.append("the dig is not in the Free Lord's land")
    if sid == "W2-B" and PL[s["haven"]]["territory"] == PL[s["lord"]]["territory"]:
        fails.append("the haven is inside the lord's own land")
    if sid in ("W5-A",) and PL[s["branch_hall"]]["territory"] != "t-first":
        fails.append("branch_hall not in First Peoples land")
    if sid == "W5-B" and PL[s["dig"]]["territory"] != "t-first":
        fails.append("the dig not in First Peoples land")
    if sid == "W5-B" and dist(s["home_hall"], MOUNT) > JOURNEY:
        fails.append("the heart's sister can't come down from the Mount within a journey")
    if sid == "W5-B" and s["reader_lord"] == s["dig_lord"]:
        fails.append("the reader lord is the dig's lord")
    if sid in ("W1-A", "W1-B"):
        if s["reading_room"] in (s["lord_a"], s["lord_b"]):
            fails.append("the lenders' hall is a lord's own seat")
        if major(s["lord_a"]) != major(s["lord_b"]):
            fails.append("the two lords are of different major factions (a treaty, not a ford feud)")
        if not lenders_hall_ok(s["reading_room"], s["lord_a"]):
            fails.append("the lenders' hall is a rival faction's town")
    # draft 4: ownership needs (whose land, which territory, what kind of town)
    for slot, test, other in sp["owns"]:
        p = s.get(slot)
        if p is None:
            fails.append(f"slot {slot}: an ownership need, and the slot is unfilled")
            continue
        if test == "lordship":
            o = s[other]
            lp = lordship_of(p)
            if lp != o:
                fails.append(f"slot {slot}: {name(p)} lies in {name(lp) if lp else 'no'} lordship, not {name(o)}'s")
        elif test == "territory":
            ok_t = {PL[place_of(s, sp["centre"])]["territory"] if t == "centre" else t for t in other.split("|")}
            ta = territory_at(PL[p]["x"], PL[p]["y"])
            if ta not in ok_t or ta in HOSTILE:
                fails.append(f"slot {slot}: {name(p)} is in {ta}, not {' or '.join(sorted(ok_t))}")
        elif test == "market":
            if p != place_of(s, sp["centre"]) and not market_town(p):
                fails.append(f"slot {slot}: {name(p)} ({PL[p]['kind']}) has no market")
    if sid == "W6-B":
        h = holder(s["lord"])
        if h["age"] != "old" or h["id"] in LEADERS:
            fails.append("the dying lord is not old, or is a faction leader")
    if sid == "W6-A" and PL[s["valley"]]["territory"] != PL[s["court"]]["territory"]:
        fails.append("the valley is not in the court's land")
    if sid == "W8-B" and s["edge_village_b"] == s["edge_village"]:
        fails.append("the two edge villages are one")
    return rows, fails


# --- the roll ----------------------------------------------------------------
def heart_cast(rng, sp):
    t, v, sex = sp["heart"]
    if sex == "cast":
        sex = rng.choice(["woman", "man"])
    return f"{t}{v if v not in ('spec', 'W7c') else ' (' + v + ')'} · {sex}"


def main_town(s, sp):
    c = place_of(s, sp.get("main", sp["centre"]))
    return c if not c.startswith("new-") else nearest_settlement(PL[c]["x"], PL[c]["y"])


def lords_of(s, sp):
    return [s[k] for k in sp["lords"] if k in s]


def place_spine(sp, mem, rng, tier, passed):
    """Fill a spine's slots on the map (40 tries); None if it can't be placed."""
    sid = sp["id"]
    for attempt in range(40):
        snap = snapshot()
        s = {"_why": {}, "_sid": sid, "_lords": set(mem["lords"]), "_towns": set(mem["towns"]) if tier < 3 else set()}
        try:
            sp["fill"](rng, s, sid)
        except (NoFit, Created) as e:
            restore(snap)
            if attempt == 39:
                passed.append(f"{sid}: {e}")
            continue
        mt = main_town(s, sp)
        if any(l in mem["lords"] for l in lords_of(s, sp)) or (tier < 3 and mt in mem["towns"]) \
                or (s.get("_house_id") and s["_house_id"] in mem["lords"]):
            restore(snap)
            if attempt == 39:
                passed.append(f"{sid}: its lord or main town is in Fresh stories' memory")
            continue
        s["_added"] = {k: v["added"][len(snap[1].get(k, [])):] for k, v in PL.items() if v.get("added")
                       and len(v["added"]) > len(snap[1].get(k, []))}
        return s
    return None


def snapshot():
    return set(PL), {k: list(v.get("added", [])) for k, v in PL.items()}


def restore(snap):
    keys, added = snap
    for k in set(PL) - keys:
        del PL[k]
    for k, v in PL.items():
        if added.get(k):
            v["added"] = list(added[k])
        else:
            v.pop("added", None)


def looks_ahead(sp, mem, left):
    for k in dict.fromkeys(blow_keys(sp)):
        if mem["blows"].count(k) < BLOW_CAP and (not mem["blows"] or mem["blows"][-1] != k):
            if can_finish(take(mem, sp, k, list(forced_routine(sp))), left - 1):
                return True
    return False


def roll_blow(rng, sp, mem):
    keys = sorted(sp["blows"])
    ok = [k for k in keys if mem["blows"].count(blow_key(sp["blows"][k][0])) < BLOW_CAP
          and (not mem["blows"] or mem["blows"][-1] != blow_key(sp["blows"][k][0]))]
    pool = ok or keys
    w = [1.0 / (1 + 2 * mem["blows"].count(blow_key(sp["blows"][k][0]))) for k in pool]
    return rng.choices(pool, weights=w)[0]


def roll_routine(rng, sp, mem):
    combos = [c for c in routine_combos(sp) if routine_ok(sp, c)]
    def weight(combo):
        w = 1.0
        for x in combo:
            p = premise(x)
            n = mem["routine"].count(p)
            if p in ROUTINE_CAP and n >= ROUTINE_CAP[p]:
                w *= 0.0
            w *= WEIGHT[family(x)] / (1 + 2 * n)
        return w
    ws = [weight(c) for c in combos]
    if sum(ws) == 0:
        ws = [1.0] * len(combos)
    combo = rng.choices(combos, weights=ws)[0]
    return dict(zip(("R1", "R2", "R3"), combo))


def roll_endings(rng, sp, mem):
    """Draft 5: a spine has exactly three endings, all offered; only the one
    shown is chosen (never one shown earlier in the set)."""
    picks = list(range(len(sp["endings"])))
    fresh = [i for i in picks if sp["endings"][i] not in mem["shown"]]
    shown = rng.choice(fresh or picks)
    return picks, shown


def clock_total(sp, s):
    """Every leg the crisis card's events need, summed (draft 4), plus any
    fixed extra (a day for the lofts)."""
    door = card(sp, "DOOR")[1].get(s.get("_blow")) if s.get("_blow") else None
    legs = [(place_of(s, door if a == "@door" else a), place_of(s, door if b == "@door" else b))
            for a, b in sp["clock"][0]]
    legs = [(a, b, road(a, b)) for a, b in legs]
    extra = sp["clock"][1]
    return sum(d for _a, _b, d in legs) + (extra[0] if extra else 0), legs


def fill_extras(rng, sp, s, flav):
    for slot, names in sp["days_slots"].items():
        fit = [n for n in names if seasons_fit(seasons_in(n) | set(sp["seasons"]))]
        s[slot] = rng.choice(fit or names)
    if sp["clock"]:
        total, legs = clock_total(sp, s)
        s["days"] = days_for(total)
        extra = sp["clock"][1]
        s["_clock"] = (f"{s['days']} = " + " + ".join(f"{name(a)}→{name(b)} {d:.2f}" for a, b, d in legs)
                       + (f" + {extra[0]} ({extra[1]})" if extra else "")
                       + f" = {round(total, 2):.2f}, rounded up {math.ceil(round(total, 2) - 1e-9)}, + 1: {sp['clock'][3]}")
    s["region"] = s.get("region") or region_word(place_of(s, sp["centre"]))
    for ax, v in flav.items():
        if "lodges the night before" in ax:
            s["lodging"] = resolve_lodging(v, place_of(s, sp["acted"]))
    s["_seasons"] = sorted(seasons_in(" ".join(str(s[k]) for k in sp["days_slots"])) | set(sp["seasons"]))


def resolve_lodging(option, town):
    """W5-A's flavour b is said placeless on C1-2 ("the chapel of the town they
    burn in"); the crisis names it for the town the burning is in (draft 4)."""
    if "chapel" in option:
        return f"{name(town)}'s chapel"
    if "inn" in option:
        return f"an inn by {name(town)}'s gate"
    shrines = [p for p in PL if "wayside shrine" in PL[p]["kind"] and "ruined" not in PL[p]["kind"]]
    return name(nearest(shrines, town))


def story_of(seed, sid, s, rng, mem, passed, tier):
    """Roll the rest of one story (blow, flavour, heart, routine, clock,
    endings), check it, and return its report lines, fails and the new memory."""
    out = []
    sp = SPINES[sid]
    bk = roll_blow(rng, sp, mem)
    flav = {ax: rng.choice(opts) for ax, (_v, opts) in sp["flavour"].items()}
    heart = heart_cast(rng, sp)
    s["_routine"] = roll_routine(rng, sp, mem)
    s["_blow"] = bk
    fill_extras(rng, sp, s, flav)
    picks, shown = roll_endings(rng, sp, mem)
    rolled = [sp["endings"][i] for i in picks] + ([sp["seat_ending"]] if sp.get("seat_ending") else [])
    rows, fails = check_story(sp, s, bk, rolled)
    if tier:
        fails.append(f"Fresh stories relaxed to tier {tier}: nothing else fitted")
    centre = place_of(s, sp["centre"])
    mt = main_town(s, sp)
    o = out.append
    o("\n" + "=" * 78)
    o(f"seed {seed} · {sid} {sp['title']} · enemy {sp['enemy']} ({sp['kind']})")
    if passed:
        o("  passed over: " + "; ".join(passed))
    o(f"  heart: {heart}")
    o(f"  stake {sp['stake']} · chapter-1 move {sp['move']} · openings: {' / '.join(openings(sp))}")
    o(f"  chapter-1 shape {sp['shape']} ({SHAPES[sp['shape']]}) · reveal given by {GIVERS[sp['reveal']]}, "
      f"closing form: {FORMS[sp['form']]}" + (f" · heart motif: {sp['motif']}" if sp["motif"] else ""))
    o(f"  blow {bk}: {sp['blows'][bk][0]} -> door {sp['blows'][bk][1]}")
    for ax, v in flav.items():
        o(f"  flavour ({ax}; voiced on {sp['flavour'][ax][0]}): {v}")
    o("  routine: " + " · ".join(f"{c} {premise(s['_routine'][c])}" + (" (swapped)" if s["_routine"][c] != card(sp, c)[1] else "")
                                 for c in ("R1", "R2", "R3")))
    o(f"  endings rolled: " + ", ".join(f"{i + 1}. {sp['endings'][i]}" for i in picks)
      + (f" + seat ending: {sp['seat_ending']}" if sp.get("seat_ending") else ""))
    o(f"  ending shown: {shown + 1}. {sp['endings'][shown]}")
    o(f"  centre: {sp['centre']} = {name(centre)} (main town: {name(mt)}; region: {s['region']})")
    cr = card(sp, "CR")
    o(f"  crisis: at {cr[2]} = {name(place_of(s, cr[2]))}" + (f" (about {sp['about']})" if sp["about"] else " (where the story is)")
      + f"; giver: {sp['giver'][0]}; acted at {name(place_of(s, sp['acted']))}")
    if sp["clock"]:
        o(f"  {{days}} = {s['_clock']}")
    o("  slots:")
    for slot, kind, need, _b in sp["slots"]:
        if slot in s:
            p = s[slot]
            if kind in ("day", "house", "region"):
                o(f"    {slot:13} {p}" + (f"  ({s['days']} days off at the last move)" if kind == "day" and sp['clock'] and sp['clock'][2] == slot else ""))
                continue
            if kind == "place":
                o(f"    {slot:13} {p}  (from flavour)")
                continue
            extra = []
            if PL[p].get("holder"):
                extra.append("holder " + who(p))
            if PL[p].get("created"):
                extra.append(f"ADDED at ({PL[p]['x']}, {PL[p]['y']}): {PL[p]['created']}")
            if s.get("_added", {}).get(p):
                extra += [f"feature added: {f} ({r})" for f, r in s["_added"][p]]
            if slot in s["_why"]:
                extra.append(s["_why"][slot])
            for oslot, test, other in sp["owns"]:
                if oslot == slot:
                    if test == "lordship":
                        extra.append(f"in {name(lordship_of(p))}'s lordship (needs {name(s[other])}'s)")
                    elif test == "territory":
                        extra.append(f"in {territory_at(PL[p]['x'], PL[p]['y'])} (needs {other.replace('centre', 'the centre')})")
                    else:
                        extra.append(f"a market town ({PL[p]['kind']})")
            own = PL[p].get("owner_faction") or PL[p]["territory"]
            o(f"    {slot:13} {name(p)} [{PL[p]['kind']}; {own}; {dist(p, centre):.2f} from centre]"
              + ("".join("\n" + " " * 18 + "· " + e for e in extra)))
        else:
            o(f"    {slot:13} (role: {need})")
    o("  placement report (card · place · from centre · bound):")
    mx = 0.0
    for cid, what, pname, dc, mark in rows:
        o(f"    {cid:5} {pname:38} {dc:5.2f}  {mark}   [{what}]")
        if "Wayhouse (fixed)" not in mark and cid != "CR*":
            mx = max(mx, dc)
    rt = {r[2] for r in rows if r[0] in ("R1", "R2", "R3", "RV")}
    o(f"  routine and reveal towns: {len(rt)} distinct ({', '.join(sorted(rt))})")
    for f in fails:
        o("  FAIL " + f)
    o(f"  max distance from centre (Wayhouse cards aside): {mx:.2f}" + ("" if fails else " · all checks pass"))
    summ = ((seed, sid, name(mt), mx, len(fails), heart.split(" · ")[0], sp["stake"],
                    blow_key(sp["blows"][bk][0]), sp["endings"][shown]))
    hq = mt == WAYHOUSE or any(r[2] == name(WAYHOUSE) for r in rows if r[0] in DEALT)
    o(f"  Highcrossing on a dealt card: {'yes' if hq else 'no'}" + (f" · seasons: {', '.join(s['_seasons'])}" if s.get("_seasons") else ""))
    mem = take(mem, sp, blow_key(sp["blows"][bk][0]), [s["_routine"][c] for c in ("R1", "R2", "R3")])
    mem["hq"][-1] = hq
    mem["lords"] += lords_of(s, sp) + ([s["_house_id"]] if s.get("_house_id") else [])
    mem["towns"].append(mt)
    mem["endings"] += rolled
    mem["shown"].append(sp["endings"][shown])
    return dict(out=out, fails=[f"seed {seed} {sid}: {f}" for f in fails], summary=summ, mem=mem)


BUDGET = 4000  # placements tried per pass before Fresh stories relaxes a tier


def run_set(seeds, verbose=True):
    """Roll a set as playthroughs in a row. Each sample takes the first spine
    in its seed's shuffle that memory allows and the map can place; if a later
    sample then finds nothing, the set backs up and the earlier sample takes
    its next spine. Only if no whole set fits does Fresh stories relax a tier."""
    seeds = list(seeds)
    res = None
    for max_tier in (0, 1, 2, 3):
        reset_places()
        budget = [BUDGET]

        def go(i, mem):
            if i == len(seeds):
                return []
            seed = seeds[i]
            rng = random.Random(seed)
            order = ORDER[:]
            rng.shuffle(order)
            passed = []
            for tier in range(max_tier + 1):
                for sid in order:
                    sp = SPINES[sid]
                    if budget[0] <= 0:
                        return None
                    if not allowed(sp, mem, tier):
                        continue
                    if tier == 0 and not looks_ahead(sp, mem, len(seeds) - i):
                        passed.append(f"{sid}: would leave the rest of the set no spine under the caps")
                        continue
                    budget[0] -= 1
                    snap = snapshot()
                    s = place_spine(sp, mem, rng, tier, passed)
                    if s is None:
                        continue
                    st = story_of(seed, sid, s, rng, mem, list(passed), tier)
                    if tier == 0 and variety_fails(st["mem"]):
                        restore(snap)
                        passed.append(f"{sid}: its routine or blow would break the set's caps")
                        continue
                    rest = go(i + 1, st["mem"])
                    if rest is not None:
                        return [st] + rest
                    restore(snap)
                    passed.append(f"{sid}: the later samples could not then be placed")
            return None

        res = go(0, empty_memory())
        if res:
            break
    if not res:
        raise SystemExit(f"FAIL seeds {seeds[0]}-{seeds[-1]}: no set can be placed")
    out, allfails, summary = [], [], []
    for st in res:
        out += st["out"]
        allfails += st["fails"]
        summary.append(st["summary"])
    mem = res[-1]["mem"]
    # the set's variety
    vf = variety_fails(mem)
    allfails += [f"set: {f}" for f in vf]
    o = out.append
    o("\n" + "=" * 78)
    o("Set summary")
    for seed, sid, town, mx, nf, ht, st, bl, sh in summary:
        o(f"  {seed} {sid:5} {ht:4} {st} {bl:18} main town {town:18} max {mx:.2f}  shown: {sh:26} {'FAIL x' + str(nf) if nf else 'pass'}")
    o("  variety: " + variety_line(mem))
    for f in vf:
        o("  FAIL " + f)
    if verbose:
        print("\n".join(out))
    return allfails, mem


def variety_fails(mem):
    f = []
    if len(set(mem["spines"])) != len(mem["spines"]):
        f.append("a spine twice")
    if len(set(mem["enemies"])) != len(mem["enemies"]):
        f.append("an enemy twice")
    for t in set(mem["hearts"]):
        if mem["hearts"].count(t) > heart_cap(t):
            f.append(f"heart type {t} ×{mem['hearts'].count(t)}")
    for i, t in enumerate(mem["hearts"]):
        if t in mem["hearts"][max(0, i - HEART_GAP):i]:
            f.append(f"heart type {t} again within {HEART_GAP} samples")
    for x in set(mem["stakes"]):
        if mem["stakes"].count(x) > STAKE_CAP:
            f.append(f"stake {x} ×{mem['stakes'].count(x)}")
    if any(a == b for a, b in zip(mem["stakes"], mem["stakes"][1:])):
        f.append("a stake twice running")
    for x in set(mem["blows"]):
        if mem["blows"].count(x) > BLOW_CAP:
            f.append(f"blow '{x}' ×{mem['blows'].count(x)}")
    if any(a == b for a, b in zip(mem["blows"], mem["blows"][1:])):
        f.append("a blow twice running")
    for x in set(mem["moves"]):
        if mem["moves"].count(x) > MOVE_CAP:
            f.append(f"move {x} ×{mem['moves'].count(x)}")
    if any(a == b for a, b in zip(mem["moves"], mem["moves"][1:])):
        f.append("a move twice running")
    for x in set(mem["openings"]):
        if open_count(mem["openings"], x) > open_cap(x[4:]):
            f.append(f"opening '{x}' ×{open_count(mem['openings'], x)}")
    for x in set(mem["routine"]):
        if x in ROUTINE_CAP and mem["routine"].count(x) > ROUTINE_CAP[x]:
            f.append(f"routine premise '{x}' in {mem['routine'].count(x)} stories")
    if len(set(mem["shown"])) != len(mem["shown"]):
        f.append("an ending shown twice")
    if len(set(mem["towns"])) != len(mem["towns"]):
        f.append("a main town twice")
    if len(set(mem["lords"])) != len(mem["lords"]):
        f.append("a lord, court or house twice")
    for x in set(mem["motifs"]):
        if mem["motifs"].count(x) > MOTIF_CAP.get(x, 2):
            f.append(f"heart motif '{x}' ×{mem['motifs'].count(x)}")
    for key, cap, label in (("shapes", lambda x: SHAPE_CAP, "chapter-1 shape"),
                            ("reveals", lambda x: REVEAL_CAP.get(x, 4), "reveal giver"),
                            ("forms", lambda x: FORM_CAP, "reveal form")):
        for x in set(mem[key]):
            if mem[key].count(x) > cap(x):
                f.append(f"{label} '{x}' ×{mem[key].count(x)}")
    if sum(mem["hq"]) > HQ_CAP:
        f.append(f"{sum(mem['hq'])} stories deal a card at the Wayhouse; cap {HQ_CAP}")
    return f


def variety_line(mem):
    def top(xs):
        c = {}
        for x in xs:
            c[x] = c.get(x, 0) + 1
        return max(c.values()) if c else 0
    return (f"hearts {'/'.join(mem['hearts'])} · stakes max ×{top(mem['stakes'])} · blows max ×{top(mem['blows'])} · "
            f"ch1 shapes {'/'.join(str(x) for x in mem['shapes'])} · reveals {'/'.join(mem['reveals'])} · "
            f"forms max ×{top(mem['forms'])} · Highcrossing in {sum(mem['hq'])} · "
            f"openings max ×{top(mem['openings'])} · ledger in {mem['routine'].count(LEDGER)} routines, "
            f"copy-it in {mem['routine'].count(COPY)} · Burn your copy rolled {mem['endings'].count('Burn your copy')}, "
            f"endings shown all different: {len(set(mem['shown'])) == len(mem['shown'])}")


def run(seeds):
    sfails, types = static_checks()
    print("Static checks over the 16 spines:")
    print("  heart types: " + ", ".join(f"{t} ×{len(v)} ({', '.join(v)})" for t, v in sorted(types.items())))
    for f in sfails:
        print("  FAIL", f)
    if not sfails:
        print("  all pass (hidden-hand bar, shaming guard, flavour voiced, plants for endings, parties set up, "
              "heart-type cap, routine and chapter families, the crisis where the story is, slots in the text "
              "filled, no hard-numbered deadline, no region name)")
    fails, _mem = run_set(seeds)
    fails = sfails + fails
    if fails:
        print(f"\nFAIL: {len(fails)} rule break(s)")
        for f in fails:
            print("  " + f)
        sys.exit(1)
    print("\nAll stories pass.")


def run_sets(start, count):
    sfails, _ = static_checks()
    for f in sfails:
        print("  FAIL", f)
    total = list(sfails)
    for k in range(count):
        seeds = range(start + 8 * k, start + 8 * k + 8)
        fails, mem = run_set(seeds, verbose=False)
        total += fails
        print(f"set {k + 1:2} (seeds {seeds[0]}-{seeds[-1]}): {'pass' if not fails else 'FAIL x' + str(len(fails))} · "
              f"{' '.join(mem['spines'])} · {variety_line(mem)}")
        for f in fails:
            print("    " + f)
    if total:
        print(f"\nFAIL: {len(total)} rule break(s)")
        sys.exit(1)
    print(f"\nAll {count} sets pass.")


def placing_md(sid):
    sp = SPINES[sid]
    mt = sp.get("main", sp["centre"])
    out = [f"**17 · Placing.** Centre: `{{{sp['centre']}}}`. Main town (for *Fresh stories*): "
           + (f"`{{{mt}}}`" if mt != sp["centre"] else "the centre's town")
           + (". Lords and courts kept in memory: " + ", ".join(f"`{{{l}}}`" for l in sp["lords"]) if sp["lords"] else "")
           + f". Kept in memory too: stake {sp['stake']}, move {sp['move']}, openings {', '.join(openings(sp))}."
           + " Checked by `roll_samples.py` (*Placing a spine*)."]
    cr = card(sp, "CR")
    out.append("")
    out.append(f"*Crisis:* at `{{{cr[2]}}}`" + (f", about {sp['about']}" if sp["about"] else ", where the story is")
               + f"; the giver is there ({sp['giver'][0]}); its action falls at `{{{sp['acted']}}}`."
               + (" *Clock:* `{days}` = " + ", plus ".join(f"the road from {'the dealt door' if a == '@door' else f'`{{{a}}}`'} to `{{{b}}}`" for a, b in sp["clock"][0])
                  + (f", plus {sp['clock'][1][0]} day ({sp['clock'][1][1]})" if sp["clock"][1] else "")
                  + f", rounded up, + 1 ({sp['clock'][3]})"
                  + (f"; `{{{sp['clock'][2]}}}` is that many days off at the last move." if sp['clock'][2] else ".")
                  if sp["clock"] else ""))
    if sp["alts"]:
        out.append("*Swaps* (field 5): " + "; ".join(f"{c} may be {', '.join(premise(a) for a in v)}" for c, v in sp["alts"].items()) + ".")
    out.append("")
    out.append("| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |")
    out.append("|---|---|---|---|")
    for slot, kind, need, bound in sp["slots"]:
        out.append(f"| `{{{slot}}}` | {kind} | {need} | {bound} |")
    out.append("")
    out.append("| Card | Place | Within 1.25 of | Mode |")
    out.append("|---|---|---|---|")
    for cid, what, where, anchors, mode, note in cards_of(sp):
        if cid == "DOOR":
            where = " · ".join(f"{k}: `{{{v}}}`" for k, v in what.items())
        else:
            where = f"`{{{where}}}`"
        an = ", ".join(f"`{{{a}}}`" for a in anchors) or "—"
        md = {B: "bound", J: f"journey (≤3.5 of the centre): {note}", F: "free (≤3.0 of the centre)",
              HQ: f"the Wayhouse: {note}"}[mode]
        out.append(f"| {cid if cid != 'CR' else 'crisis'} | {where} | {an} | {md} |")
    out.append(f"| crisis, acted | `{{{sp['acted']}}}` | — | where its action falls |")
    out.append("")
    out.append("*Parties set up first* (checked): " + "; ".join(f"{p} ({c})" for p, c in sp["parties"].items()) + ".")
    return "\n".join(out)


# --- draft 5: the spine text is the source -----------------------------------
import spine5  # noqa: E402


def load_d5():
    """Blows, endings and flavour come from each spine's fields 9, 11 and 14
    in spines.md (draft 5), never from a second copy here."""
    _md, sections = spine_sections()
    for sid, sp in SPINES.items():
        d5 = spine5.parse(sid, sections.get(sid, ""))
        sp["d5"] = d5 if d5["branches"] else None
        if not sp["d5"]:
            continue
        sp["blows"] = {k: (t.replace("`", ""), door) for k, (t, door) in spine5.blows(d5).items()}
        sp["cards"] = [("DOOR", {k: b["door_place"] for k, b in d5["branches"].items()}) + tuple(c[2:])
                       if c[0] == "DOOR" else c for c in sp["cards"]]
        sp["endings"] = [e["name"] for e in d5["endings"]]
        if d5["seat"]:
            sp["seat_ending"] = d5["seat"]["name"]
        sp["flavour"] = {f"{slot} · {what}": (voice, opts) for slot, (voice, opts, what) in d5["flavour"].items()}
        sp["flavour_slots"] = list(d5["flavour"])
        cr = card(sp, "CR")
        d5["_cr_place"] = {cr[2], sp["acted"]}


def d5_static(sid, sp):
    f = []
    d5 = sp["d5"]
    if not d5:
        return [f"{sid}: not yet in draft 5 (no branches in field 9)"]
    f += spine5.check(d5, known_seat=bool(SPINES_SEAT.get(sid)), place_words=PLACE_WORDS)
    for k in sp["door_needs"]:
        if k != "all" and k not in sp["blows"]:
            f.append(f"{sid}: door_needs names branch {k}, which the spine doesn't have")
    names = set(sp["endings"]) | ({sp["seat_ending"]} if sp.get("seat_ending") else set())
    for e in sp["ending_needs"]:
        if e not in names:
            f.append(f"{sid}: ending_needs names '{e}', which is not one of its endings")
    for e in sp["endings"]:
        if e not in sp["ending_needs"]:
            f.append(f"{sid}: ending '{e}' lists no parties it needs (ending_needs)")
    for e in sp["plants"]:
        if e not in names:
            f.append(f"{sid}: a plant is kept for '{e}', which is not one of its endings")
    # a seat slot's needs and the leader rule are data, checked per story; the spine must say why it asks
    for slot, (why, _ex) in sp.get("leaders_ok", {}).items():
        if not why:
            f.append(f"{sid}: {slot} may be a faction leader, and the spine gives no reason")
    have = {k for k, _t, _g in d5["cards"]}
    for cid in sp["alts"]:
        if f"{cid} swap" not in have:
            f.append(f"{sid}: {cid} has a swap and field 18 has no '{cid} swap' line")
    # the Wayhouse exemption keeps a clock (review-v7d 3d)
    if card(sp, "CR")[4] == HQ and not (sp["clock"] and any(a == "@door" for a, _b in sp["clock"][0])):
        f.append(f"{sid}: the crisis is at the Wayhouse and no clock runs from the door to it")
    return f


SPINES_SEAT = {sid: sp.get("seat_ending") for sid, sp in SPINES.items()}
load_d5()


def render_all():
    """--render-all: every branch x ending of every spine, to proof/<spine>.md."""
    out_dir = os.path.join(HERE, "proof")
    os.makedirs(out_dir, exist_ok=True)
    total, lines = 0, []
    for sid in ORDER:
        sp = SPINES[sid]
        if not sp["d5"]:
            raise SystemExit(f"FAIL {sid}: not in draft 5")
        places = {}
        for c in cards_of(sp):
            cid, what, where = c[0], c[1], c[2]
            if cid in ("R1", "R2", "R3", "C1-1", "C1-2", "C1-3", "C2-0"):
                places[cid] = f"`{{{where}}}`"
            if cid == "RV":
                places["RV"] = f"`{{{where}}}`"
        cr = card(sp, "CR")
        crp = f"at `{{{cr[2]}}}`" + (f", acted at `{{{sp['acted']}}}`" if sp["acted"] != cr[2] else "")
        text, n = spine5.render_spine(sp["d5"], sp["title"], places, crp)
        open(os.path.join(out_dir, f"{sid}.md"), "w", encoding="utf-8").write(text)
        lines.append(f"  {sid} {sp['title']}: {len(sp['d5']['branches'])} branch(es) x "
                     f"{len(sp['d5']['endings']) + (1 if sp['d5']['seat'] else 0)} endings = {n}")
        total += n
    print("\n".join(lines))
    print(f"Total combinations: {total} (written to proof/<spine>.md)")
    return total


if __name__ == "__main__":
    a = sys.argv[1:]
    if a and a[0] == "--render-all":
        sf, _ = static_checks()
        for x in sf:
            print("  FAIL", x)
        render_all()
        if sf:
            sys.exit(1)
    elif a and a[0] == "--placing":
        for sid in (a[1:] or ORDER):
            print(f"<!-- {sid} -->\n" + placing_md(sid) + "\n")
    elif a and a[0] == "--sets":
        run_sets(int(a[1]), int(a[2]))
    else:
        lo, hi = (int(a[0]), int(a[1])) if len(a) == 2 else (7401, 7408)
        run(range(lo, hi + 1))
