#!/usr/bin/env python3
"""World D, layers 0 and 1 (casting v1.2 §1, §3, §4, §5, §7, §8; locations §7a).
Adapted from World C's c-l0-build.py, brought up to casting v1.2:
- a generated filler REPLACES the canon person (casting §4.3): faction leaders, seats, links and foils point at it;
- one office per person; co-leaders on one seat = one holder plus a title (casting §3; rulebook *Seats*);
- a tension counts against every major faction whose land or people it involves (casting §5);
- kin of generated fillers keep the canon relation and share a family name (casting §4.3);
- guild hall and contact keepers are on demand, not pre-made (casting §3);
- every archetype row lists its keeps explicitly (traits, facts, links), plus age, voice, peoples, sex (casting §4);
- people, age and voice on everyone created (casting §7).
Every roll is keyed (casting §8): Random("4401|<key>|..."). Names come from the naming module's lists and
byname rules (gahturiyu-sim assets/lang/names/*.ron and people.ron), by people (casting §7).
Writes D/d-l0.json."""
import json, random, os, re, unicodedata

HERE = os.path.dirname(os.path.abspath(__file__))
W = os.path.dirname(HERE)
canon = json.load(open(os.path.join(W, "canon-D.json")))
mp = json.load(open(os.path.join(W, "map-D.json")))
SEED = 4401
PLACES = {p["id"]: p for p in mp["places"]}
CANON_P = {p["id"]: p for p in canon["people"]}
LANG = "/home/claude/gahturiyu-sim/assets/lang"

def K(*parts):
    return random.Random("|".join([str(SEED)] + [str(p) for p in parts]))

def wpick(rng, weights):
    items = sorted(weights.items())
    tot = sum(w for _, w in items)
    r = rng.random() * tot
    for k, w in items:
        r -= w
        if r < 0:
            return k
    return items[-1][0]

LOG = []
RULES = []        # rule problems found while building
facts = []
def fact(fid, text, vis="public", kind="state", when=None, source="world"):
    f = {"id": fid, "text": text, "visibility": vis, "kind": kind, "source": source}
    if when: f["when"] = when
    assert all(x["id"] != fid for x in facts), fid
    facts.append(f)
    return fid

# ---------------------------------------------------------------- voices (voices-ref.md)
MANNERS = ["blunt", "careful", "warm", "wry", "weary", "proud"]
HABITS = {
    "roduro": ["traveller", "names_the_dead", "so_first", "counts_seasons", "stone_keep_you", "speaks_of_the_house"],
    "qotiro": ["one_word", "decide", "curses_the_damp", "eat", "no_goodbye"],
    "horaro": ["have_you_eaten", "out_of_the_wind", "we_for_all", "talks_tide", "another_day", "waits_for_a_sign"],
    "tadoro": ["have_you_noticed", "guesses_where_from", "corrects_self", "writes_it_down", "listen_first", "what_would_you_do"],
}
def roll_voice(key, people):
    ms = {m: 1 for m in MANNERS}
    if people == "tadoro":
        ms["solitary"] = 1
    return {"manner": wpick(K(key, "make", "manner"), ms),
            "habit": K(key, "make", "habit").choice(HABITS[people])}

# ---------------------------------------------------------------- names (naming module lists + byname rules)
def plain(s):
    s = unicodedata.normalize("NFD", s)
    s = "".join(c for c in s if unicodedata.category(c) != "Mn")
    return s.replace("ʻ", "").replace("'", "").lower()

NAMES = {}
for ppl in ("roduro", "qotiro", "horaro", "tadoro"):
    txt = open(os.path.join(LANG, "names", ppl + ".ron"), encoding="utf-8").read()
    NAMES[ppl] = re.findall(r'\(name: "([^"]+)", gender: (Male|Female|Either)', txt)
CANON_GIVEN = ["tuhuḍo", "garoṭaqa", "ruʻo", "niralea", "gahìru", "litiṭaʻa", "haʻahoqo", "shahayuh", "kotqot", "waarua",
               "feyesh", "dorog", "kodor", "doruḍoqu", "hotorìṭo", "aloqi", "textrim", "totkit", "nemu", "paqax", "haʻo",
               "hoqa", "shihi", "fiyeweya", "lamawina", "yitolaʻo", "yaweth", "liʻo", "kitim"]
CANON_STEMS = {plain(n)[:4] for n in CANON_GIVEN}
# byname parts (people.ron)
HOUSE_RINGS = ["three", "four", "five", "six", "seven", "eight", "nine", "ten", "hundred"]
HOUSE_MARKS = ["moss", "vine", "grass", "thorn", "tree", "spring", "rain", "frost", "gull", "smoke", "ember", "moon", "star", "herb", "bark", "wind"]
HOUSE_SITES = ["ledge", "slope", "cliff", "crag", "hollow", "hill", "bank", "ridge", "brow", "stream", "field", "forest", "shore", "cave"]
DEEDS = ["Iron-breaker", "Stone-breaker", "Shield-breaker", "Spear-breaker", "Bone-breaker", "Gate-holder", "Pass-holder",
         "Wall-warden", "Hearth-warden", "Road-warden", "Law-keeper", "Oath-keeper", "Storm-walker", "Night-walker", "Sand-walker",
         "Fire-walker", "Beast-hunter", "Hound-hunter", "Thunder-striker", "First-striker", "Spear-bearer", "Torch-bearer",
         "Shield-bearer", "Fire-binder", "Beast-binder", "Ore-finder", "Spring-finder", "Way-finder", "Far-seer", "Horn-cutter",
         "Rope-cutter", "Gorge-leaper", "Wall-leaper", "Spear-thrower", "Ember-bearer", "Crag-runner", "Dawn-runner", "Sun-servant",
         "Hall-builder", "Stair-builder", "Curse-breaker"]   # "Wall-breaker" left out: canon (Kotqot)
SEA_MARKS = ["kelp", "pearl", "gull", "foam", "moon", "deep", "calm", "shell", "eel", "rain", "far", "blue", "green", "white", "reed",
             "salt", "storm", "star", "old", "quiet", "long", "red"]
SEA_SITES = ["reef", "bay", "cove", "shore", "stack", "island", "harbour", "pool", "headland", "sand", "mouth"]
ROADS = ["north", "east", "west", "high", "far", "long", "old", "wind", "star", "dawn", "dusk", "hill", "salt", "silk", "sea",
         "stone", "rain", "quiet", "white"]   # "south" left out: canon (Shahayuh of the South Road)
PLACE_NAMES = sorted({plain(p["name"]).replace("the ", "") for p in mp["places"]}, key=len, reverse=True)

used_full, used_stem, used_byname = set(), set(), set()
for p in canon["people"]:
    used_full.add(p["name"].lower())

def byname(key, ppl, i):
    r = K(key, "make", "byname", i)
    if ppl == "roduro":
        if r.random() < 0.25:
            return f"of the {r.choice(HOUSE_RINGS).capitalize()}-Ring House"
        return f"of the {r.choice(HOUSE_MARKS).capitalize()}-{r.choice(HOUSE_SITES).capitalize()} House"
    if ppl == "qotiro":
        return r.choice(DEEDS)
    if ppl == "horaro":
        return f"of the {r.choice(SEA_MARKS).capitalize()} {r.choice(SEA_SITES).capitalize()}"
    return f"of the {r.choice(ROADS).capitalize()} Road"

def roll_name(key, ppl, sex, family=None):
    """Given name from the people's list (by sex, or Either); byname by the people's pattern (people.ron).
    Full name unique world-wide; given-name stems (first four plain letters) unique world-wide among generated
    people and kept off canon stems (stricter than rulebook *Name check*, which checks stems locally);
    unrelated people never share a byname; kin share theirs (`family`). On a clash, roll again with the next key."""
    want = {"man": ("Male", "Either"), "woman": ("Female", "Either")}[sex]
    pool = sorted(n for n, g in NAMES[ppl] if g in want and n not in CANON_GIVEN)
    for i in range(500):
        g = K(key, "make", "name", i).choice(pool)
        stem = plain(g)[:4]
        if stem in used_stem or stem in CANON_STEMS:
            continue
        b = family if family else byname(key, ppl, i)
        if not family and b in used_byname:
            continue
        full = g[0].upper() + g[1:] + " " + b
        if full.lower() in used_full:
            continue
        if any(f"of {pn}" in plain(full) for pn in PLACE_NAMES):
            continue
        used_stem.add(stem); used_full.add(full.lower()); used_byname.add(b)
        if i:
            LOG.append(f"name {key}: {i} clash re-roll(s)")
        return full
    raise SystemExit("name pool exhausted for " + key)

# ---------------------------------------------------------------- people mixes (no culture blend wired: the map place's region, voices-ref.md)
MIX = {
    "vale":    {"roduro": 6, "horaro": 2, "qotiro": 1, "tadoro": 1},
    "reef":    {"horaro": 6, "roduro": 3, "tadoro": 1, "qotiro": 0.5},
    "great":   {"roduro": 5, "tadoro": 2, "qotiro": 2, "horaro": 1},
    "free":    {"qotiro": 4, "roduro": 3, "tadoro": 1, "horaro": 1},
    "harbour": {"horaro": 3, "roduro": 2, "qotiro": 2, "tadoro": 2},
    "roads":   {"roduro": 3, "qotiro": 2, "tadoro": 2, "horaro": 2},
    "wave":    {"qotiro": 1},
    "sea":     {"horaro": 1},
    "snows":   {"roduro": 1},
    "green":   {"roduro": 3, "horaro": 1, "qotiro": 1, "tadoro": 1},
}
REGION_MIX = {"r-vale": "vale", "r-reefcoast": "reef", "r-crossing": "great", "r-marches": "free", "r-harbour": "harbour",
              "r-roads": "roads", "r-rim": "wave", "r-sea": "sea", "r-snows": "snows", "r-green": "green"}
def place_mix(pid):
    return dict(MIX[REGION_MIX[PLACES[pid]["region"]]])

# ---------------------------------------------------------------- archetype rows (data/quests/rules/archetypes.ron, written from leaders.md)
# Each row: role, default, keeps.traits, peoples allowed, sex (a keep: fixed where the canon role is gendered:
# mother/son, the emperor and his sister, husband/wife, brother/sister, priestess, bondswoman, don, a male war-chief;
# None where only the canon person's description is gendered),
# canon age, canon voice, canon sex (for the canon filler's pronouns), guaranteed.
# keeps.facts and keeps.links are listed below (ARCH_FACTS, LINKS) and copied into each row's keeps.
ROWS = {
 "a-seated-head": dict(role="head of the Seated Houses, the blind archivist-lord who rules the Ninth Ring from memory", default="p-tuhudo",
     traits=["blind archivist-lord", "sharp memory"], peoples={"roduro": 1}, sex=None, age="old", voice=("careful", "speaks_of_the_house"), csex="man"),
 "a-true-line-schemer": dict(role="the True Line's schemer at Firststone, digging under the Founding Stone for the older claim", default="p-garotaqa",
     traits=["schemer", "charismatic"], peoples={"roduro": 1}, sex="woman", age="grown", voice=("proud", "names_the_dead"), csex="woman"),
 "a-true-line-face": dict(role="the True Line's public face, the schemer's child", default="p-ruo",
     traits=["the face of the True Line"], peoples={"roduro": 1}, sex="man", age="young", voice=("proud", "so_first"), csex="man"),
 "a-tide-speaker": dict(role="the Tide Houses' season's speaker at Reefhold, loyal in public and torn in private", default="p-niralea",
     traits=["season's speaker", "divided loyalties"], peoples={"horaro": 1}, sex=None, age="grown", voice=("warm", "waits_for_a_sign"), csex="woman"),
 "a-emperor": dict(role="the emperor of the Great House, fading on a hollow Throne", default="p-gahiru",
     traits=["the emperor", "fading"], peoples={"roduro": 1}, sex="man", age="old", voice=("weary", "counts_seasons"), csex="man"),
 "a-throne-voice": dict(role="the emperor's sibling, the Throne's masterful liar, who keeps the court's face up", default="p-litita",
     traits=["masterful liar", "lonely"], peoples={"roduro": 1}, sex="woman", age="grown", voice=("careful", "stone_keep_you"), csex="woman"),
 "a-banner-general": dict(role="the Banner Lords' old general at Longwall, beloved of the soldiers", default="p-haahoqo",
     traits=["old general", "warm to commoners"], peoples={"roduro": 1}, sex=None, age="old", voice=("warm", "names_the_dead"), csex="man"),
 "a-banner-climber": dict(role="the Ṭaḍoro-born Banner Lord of the toll road, who built the Great Bridge from his own purse", default="p-shahayuh",
     traits=["Ṭaḍoro-born lord", "social climber"], peoples={"tadoro": 1}, sex=None, age="grown", voice=("proud", "guesses_where_from"), csex="man"),
 "a-free-lord-breaker": dict(role="the Qotiro-blooded Free Lord who took Breakwall through its own wall", default="p-kotqot",
     traits=["paranoid", "known fighter", "Qotiro-blooded"], peoples={"qotiro": 2, "roduro": 1}, sex=None, age="grown", voice=("blunt", "decide"), csex="man"),
 "a-council-head": dict(role="head of the Open Council at Openharbour, once in bondage", default="p-waarua",
     traits=["freed bondswoman", "inspiring speaker"], peoples={"horaro": 3, "roduro": 1, "qotiro": 1, "tadoro": 1}, sex="woman", age="grown", voice=("warm", "we_for_all"), csex="woman"),
 "a-sharp-broker": dict(role="the Sharp Hands' broker at the Silver Road Counting-House, who holds the notes", default="p-feyesh",
     traits=["broker", "flatterer"], peoples={"tadoro": 3, "roduro": 1, "qotiro": 1, "horaro": 1}, sex=None, age="grown", voice=("warm", "what_would_you_do"), csex=None),
 "a-sharp-twin-deals": dict(role="the Sharp Hands twin who smiles and deals", default="p-dorog",
     traits=["twin", "smiles and deals"], peoples={"qotiro": 1}, sex=None, age="grown", voice=("warm", "eat"), csex=None),
 "a-sharp-twin-counts": dict(role="the Sharp Hands twin who counts and threatens", default="p-kodor",
     traits=["twin", "counts and threatens"], peoples={"qotiro": 1}, sex=None, age="grown", voice=("blunt", "one_word"), csex="woman"),
 "a-road-foreman": dict(role="the Road Crews' elected foreman at Carter's Rest", default="p-doruduqu",
     traits=["elected foreman", "knows every road"], peoples={"roduro": 2, "qotiro": 1, "tadoro": 1, "horaro": 1}, sex=None, age="grown", voice=("blunt", "counts_seasons"), csex="man"),
 "a-high-priest": dict(role="the militant high priest of the Holy House at the Mount", default="p-hotorito",
     traits=["militant", "political"], peoples={"roduro": 4, "horaro": 1}, sex=None, age="old", voice=("proud", "so_first"), csex="man"),
 "a-sincere-priestess": dict(role="the sincere priestess who keeps the Low Shrine's kitchen in the old way", default="p-aloqi",
     traits=["sincere", "old"], peoples={"roduro": 4, "horaro": 1}, sex="woman", age="old", voice=("warm", "stone_keep_you"), csex="woman"),
 "a-wave-priestess": dict(role="the Last Wave's High Priestess at Textrim's Height (priestesses rule)", default="p-textrim",
     traits=["High Priestess", "protective", "pragmatic"], peoples={"qotiro": 1}, sex="woman", age="grown", voice=("careful", "decide"), csex="woman"),
 "a-wave-warchief": dict(role="the Last Wave's war-chief at the Proving Ground, the High Priestess's champion", default="p-totkit",
     traits=["war-chief", "known fighter"], peoples={"qotiro": 1}, sex="man", age="grown", voice=("blunt", "no_goodbye"), csex="man"),
 "a-fleet-speaker": dict(role="the Far Sails' fleet speaker at the Anchorage", default="p-nemu",
     traits=["fleet speaker", "homesick"], peoples={"horaro": 1}, sex=None, age="old", voice=("wry", "talks_tide"), csex="man"),
 "a-binder-head": dict(role="head of the Binders at the Chainmarket, a slave who became a slaver", default="p-paqax",
     traits=["ex-slave slaver", "self-justifying"], peoples={"qotiro": 1}, sex=None, age="grown", voice=("careful", "curses_the_damp"), csex="man"),
 "a-warden-gate": dict(role="the Warden who holds the Green Wall's gate and does the talking", default="p-hao",
     traits=["talks at the gate", "unbreakable"], peoples={"roduro": 3, "horaro": 1, "qotiro": 1, "tadoro": 1}, sex="man", age="old", voice=("wry", "so_first"), csex="man"),
 "a-warden-scout": dict(role="the Warden who has scouted the green deeper than anyone (guaranteed: hidden-lore)", default="p-hoqa",
     traits=["scouts the green", "knows more than {she} says"], peoples={"roduro": 3, "horaro": 1, "qotiro": 1, "tadoro": 1}, sex="woman", age="old", voice=("careful", "names_the_dead"), csex="woman"),
 "a-highfolk-head": dict(role="head of the Highfolk clans at the High Snows, who guards the way to the Womb (guaranteed: hidden-lore)", default="p-shihi",
     traits=["hospitable", "archaic"], peoples={"roduro": 1}, sex=None, age="grown", voice=("warm", "traveller"), csex="woman"),
 "a-writers-leader": dict(role="the Writers' head, always on the road", default="p-fiyeweya",
     traits=["always travelling", "puppeteer"], peoples={"tadoro": 4, "roduro": 1, "horaro": 1, "qotiro": 1}, sex=None, age="grown", voice=("wry", "writes_it_down"), csex="woman"),
 "a-lantern-quartermaster": dict(role="the Lantern Company's quartermaster at Lanternhall", default="p-lamawina",
     traits=["quartermaster", "lives by the code"], peoples={"horaro": 1, "roduro": 1, "qotiro": 1, "tadoro": 1}, sex=None, age="grown", voice=("blunt", "another_day"), csex="woman"),
 "a-old-don": dict(role="the old don of the Old Hands at the Old Gate House", default="p-yitolao",
     traits=["politely menacing"], peoples={"roduro": 1}, sex="man", age="old", voice=("careful", "speaks_of_the_house"), csex="man"),
 "a-market-fence": dict(role="the charming fence who runs the Night Market", default="p-yaweth",
     traits=["cheeky", "generous"], peoples={"tadoro": 2, "horaro": 1, "roduro": 1, "qotiro": 1}, sex=None, age="grown", voice=("wry", "have_you_noticed"), csex="man"),
 "a-dueling-showman": dict(role="the Dueling Society co-founder who runs the show", default="p-lio",
     traits=["showman"], peoples={"roduro": 2, "horaro": 1, "qotiro": 1, "tadoro": 1}, sex="man", age="young", voice=("warm", "traveller"), csex="man"),
 "a-dueling-sister": dict(role="the Dueling Society co-founder, the showman's adopted sibling, who once killed in a duel", default="p-kitim",
     traits=["known fighter", "once killed in a duel"], peoples={"qotiro": 2, "roduro": 1, "horaro": 1, "tadoro": 1}, sex="woman", age="young", voice=("blunt", "one_word"), csex="woman"),
}
GUARANTEED = {"a-warden-scout", "a-highfolk-head"}

# load-bearing facts (keeps.facts): canon hooks on the person (faction-places §2-§4), leaders.md secrets.
# {n} name; {he}/{his}/{him}/{she} follow the filler.
ARCH_FACTS = {
 "a-seated-head": [("{n} rules the Ninth Ring's archive from memory; it has gaps only {he} knows about.", "public", "trait"),
                   ("{n} has begun to forget.", "secret", "state"),
                   ("{n} knows the Alliance is dying, and is worn out from holding it up.", "secret", "trait")],
 "a-true-line-schemer": [("{n} is digging under the Founding Stone for older records to prove the True Line's claim.", "public", "state")],
 "a-tide-speaker": [("{n} knows that boats slip out from Reefhold to the Far Sails at night, and allows it.", "secret", "state")],
 "a-emperor": [("A double sits in for {n} at public audiences when {his} hands shake.", "secret", "state")],
 "a-throne-voice": [("{n} knows how weak the emperor has grown, and hides it behind the court's ceremony.", "secret", "state")],
 "a-banner-general": [("{n} has quietly begun counting how many soldiers would follow {him}, rather than the Throne, if the emperor died.", "public", "state")],
 "a-banner-climber": [("{n} paid for the Great Bridge from {his} own pocket; {his} name is cut into every stone, which the old blood finds vulgar.", "public", "past event", "years before the story"),
                      ("Old-blood lords keep 'accidentally' damaging the Great Bridge, and {n} suspects {his} fellow Banner Lord.", "public", "state")],
 "a-free-lord-breaker": [("{n} took Breakwall by smashing a hole through its outer wall, and left it open: whoever can hold the breach can have the keep.", "public", "past event", "years before the story"),
                         ("{n} keeps a written record of every lie the Throne has told, to publish the day {he} declares independence.", "secret", "state"),
                         ("{n} defies the Throne because {he} thinks it a lie, not out of greed.", "secret", "trait")],
 "a-council-head": [("{n} no longer has keys to all of the council hall's locked ledger rooms.", "public", "state")],
 "a-road-foreman": [("The Road Crews are close to voting to strike against Sharp Hands rates; one bought vote would decide it.", "public", "state")],
 "a-high-priest": [("{n} no longer believes, and has stopped performing the inner rites; no one has checked in years.", "secret", "state")],
 "a-sincere-priestess": [("The Mount wants the Low Shrine closed, and {n} has more followers than {he} realizes.", "public", "state")],
 "a-wave-priestess": [("Textrim's Height is one hard winter from hunger; {n} would trade peace for grain if anyone offered it with respect.", "public", "state")],
 "a-wave-warchief": [("Young warriors at the Proving Ground want war with the lowlands; {n} is holding them back by a thread.", "public", "state")],
 "a-fleet-speaker": [("The Binders have offered {n} a price for the list of runaways sheltered at the Anchorage.", "public", "state")],
 "a-binder-head": [("{n} had the shackles {he} wore as a slave set into the Block.", "public", "past event", "years before the story")],
 "a-warden-scout": [("{n} has mapped a path deeper into the green than anyone has gone, and found something that walks there with purpose.", "secret", "state")],
 "a-highfolk-head": [("{n} guards the way to the Womb of Creation, whose location is never spoken.", "secret", "state")],
 "a-writers-leader": [("{n} is never at the Wayhouse; {he} judges by letter, read out by the eldest copyist.", "public", "trait")],
 "a-old-don": [("{n} has kept a copy of every favor done for the old houses, and they've stopped answering {his} letters.", "secret", "state")],
 "a-market-fence": [("{n} is planning the biggest score in the island's history.", "public", "state")],
 "a-dueling-sister": [("{n} once killed in a duel, and started the Dueling Society so no one else would have to.", "public", "past event", "years before the story")],
}
# one fact shared by three archetypes (leaders.md: the Sharp Hands' secret Binder ties; faction-places §2 hook)
SHARED_FACTS = [("sharp-books", ["a-sharp-broker", "a-sharp-twin-deals", "a-sharp-twin-counts"],
                 "The Sharp Hands' three partners, {a-sharp-broker}, {a-sharp-twin-deals} and {a-sharp-twin-counts}, quietly profit from the Binder trade; a cellar below the Counting-House keeps the second set of books.",
                 "secret", "state")]

# links between archetypes (keeps.links): kept whoever fills either side (casting §4.3)
LINKS = [
 # foils (leaders.md Notes; faction-places §6 set pieces and answering places)
 ("a-council-head", "a-binder-head", "rival", None, None, "foil: the freed bondswoman and the ex-slave Binder"),
 ("a-banner-general", "a-banner-climber", "rival", None, None, "foil: the two Banner Lords, Longwall vs Southroad"),
 ("a-high-priest", "a-sincere-priestess", "rival", None, None, "foil: the Mount vs the Low Shrine"),
 ("a-seated-head", "a-true-line-schemer", "rival", None, None, "foil: Succession set piece, the Ninth Ring vs Firststone"),
 ("a-tide-speaker", "a-fleet-speaker", "rival", None, None, "foil: Reefhold vs the Anchorage"),
 ("a-old-don", "a-market-fence", "rival", None, None, "foil: the rival syndicates"),
 ("a-sharp-broker", "a-road-foreman", "rival", None, None, "the Road Crews grumble about the Sharp Hands (world-factions)"),
 # kin (canon relations)
 ("a-emperor", "a-throne-voice", "family", "sibling", "sibling", "canon: the emperor and his sister"),
 ("a-true-line-schemer", "a-true-line-face", "family", "parent", "child", "canon: mother and son"),
 ("a-sharp-twin-deals", "a-sharp-twin-counts", "family", "sibling", "sibling", "canon: twins"),
 ("a-dueling-showman", "a-dueling-sister", "family", "sibling", "sibling", "canon: adopted siblings, co-founders"),
 ("a-warden-gate", "a-warden-scout", "spouse", "spouse", "spouse", "canon: the old married pair"),
 # working ties between leaders who lead together
 ("a-sharp-broker", "a-sharp-twin-deals", "trade", None, None, "canon: three-way partnership"),
 ("a-sharp-broker", "a-sharp-twin-counts", "trade", None, None, "canon: three-way partnership"),
 ("a-wave-priestess", "a-wave-warchief", "employer/employee", "employer", "employee", "canon: her champion, who fights for her"),
]
KIN_NAME = {  # kin share a family name (casting §4.3; rulebook *Name check*): the shared part, when one exists
 "a-emperor": "a-throne-voice", "a-throne-voice": "a-emperor",
 "a-true-line-schemer": "a-true-line-face", "a-true-line-face": "a-true-line-schemer",
 "a-sharp-twin-deals": "a-sharp-twin-counts", "a-sharp-twin-counts": "a-sharp-twin-deals",
 "a-dueling-showman": "a-dueling-sister", "a-dueling-sister": "a-dueling-showman",
}
SAME_PEOPLE = {"a-sharp-twin-deals": "a-sharp-twin-counts", "a-sharp-twin-counts": "a-sharp-twin-deals",
               "a-emperor": "a-throne-voice", "a-throne-voice": "a-emperor"}
HOUSE_OF = {"a-emperor": "of the Great House", "a-throne-voice": "of the Great House",
            "a-true-line-schemer": "of the First-Stone House", "a-true-line-face": "of the First-Stone House"}

PRON = {"man": {"he": "he", "his": "his", "him": "him", "she": "he"}, "woman": {"he": "she", "his": "her", "him": "her", "she": "she"}}
def fmt(text, pid):
    sex = people[pid]["_sex"] or "man"
    nm = people[pid]["name"]
    if ", " in nm:   # "Textrim, High Priestess" mid-sentence takes a closing comma
        text = re.sub(r"\{n\}(?=\s)", nm.replace("\\", "") + ",", text)
    return text.replace("{n}", nm).replace("{he}", PRON[sex]["he"]).replace("{his}", PRON[sex]["his"]) \
               .replace("{him}", PRON[sex]["him"]).replace("{she}", PRON[sex]["she"])

people = {}
fill = {}
arch_rows = []
for aid in sorted(ROWS):
    row = ROWS[aid]
    r = K(aid, "canon").random()
    is_canon = r < 0.70
    LOG.append(f"archetype {aid}: K[{SEED},{aid},canon] = {r:.4f} -> {'canon' if is_canon else 'generated'}")
    cp = CANON_P[row["default"]]
    if is_canon:
        pid = row["default"]
        p = dict(cp)
        p["traits"] = list(cp["traits"]) + [t for t in row["traits"] if t not in cp["traits"]]
        p["people"] = None  # set below from the row's canon people
        p["_sex"] = row["csex"]
        p["age"] = row["age"]; p["voice"] = {"manner": row["voice"][0], "habit": row["voice"][1]}
        CANON_PEOPLE = {"p-tuhudo": "roduro", "p-garotaqa": "roduro", "p-ruo": "roduro", "p-niralea": "horaro", "p-gahiru": "roduro",
          "p-litita": "roduro", "p-haahoqo": "roduro", "p-shahayuh": "tadoro", "p-kotqot": "qotiro", "p-waarua": "horaro",
          "p-feyesh": "tadoro", "p-dorog": "qotiro", "p-kodor": "qotiro", "p-doruduqu": "roduro", "p-hotorito": "roduro",
          "p-aloqi": "roduro", "p-textrim": "qotiro", "p-totkit": "qotiro", "p-nemu": "horaro", "p-paqax": "qotiro",
          "p-hao": "roduro", "p-hoqa": "roduro", "p-shihi": "roduro", "p-fiyeweya": "tadoro", "p-lamawina": "horaro",
          "p-yitolao": "roduro", "p-yaweth": "tadoro", "p-lio": "roduro", "p-kitim": "qotiro"}
        p["people"] = CANON_PEOPLE[pid]
        p["traits"] = [t.replace("{she}", "she") for t in p["traits"]]
    else:
        pid = "p-d0-" + aid[2:]
        partner = SAME_PEOPLE.get(aid)
        if partner and partner in fill:
            ppl = people[fill[partner]]["people"]
        else:
            ppl = wpick(K(aid, "make", "people"), row["peoples"])
        sex = row["sex"] or ("woman" if K(aid, "make", "sex").random() < 0.5 else "man")
        kin = KIN_NAME.get(aid)
        fam = HOUSE_OF.get(aid)
        if fam is None and kin and kin in fill:
            other = people[fill[kin]]
            if other["id"].startswith("p-d0-") and other.get("_family"):
                fam = other["_family"]
                LOG.append(f"  kin share a family name: {aid} takes '{fam}' from {kin}")
            elif other["id"].startswith("p-d0-") or not other.get("_family"):
                RULES.append(f"KIN-NAME: {aid} is generated, its kin {kin} ({other['name']}) carries a personal byname, not a family name; nothing to share (casting §4.3 vs rulebook *Name check*, canon names exempt)")
        name = roll_name(aid, ppl, sex, family=fam)
        traits = [t.replace("{she}", "she" if sex == "woman" else "he") for t in row["traits"]]
        p = {"id": pid, "name": name, "standing": cp["standing"], "traits": traits, "links": [],
             "faction": dict(cp["faction"]), "facts": [], "premade": True, "dead": False, "home": cp["home"],
             "dies_at": None, "fallback": None, "people": ppl, "age": row["age"], "voice": roll_voice(aid, ppl), "_sex": sex}
        LOG.append(f"  generated {pid}: {name} ({ppl}, {sex}, {row['age']}); voice {p['voice']['manner']}/{p['voice']['habit']}; "
                   f"standing {cp['standing']}, home {cp['home']}, replaces {cp['id']} ({cp['name']})")
    # family name, for kin of a later generated filler: a house byname ("of the X House") is a family name
    m = re.search(r"(of the [^,]*House)$", p["name"])
    p["_family"] = m.group(1) if m else None
    p["archetype"] = aid; p["class"] = None
    p["links"] = []; p["facts"] = []; p["dies_at"] = None; p["fallback"] = None
    people[pid] = p
    fill[aid] = pid
    arch_rows.append({"id": aid, "role": row["role"], "default": row["default"], "filled_by": pid,
                      "canon": "yes" if is_canon else "no", "roll": round(r, 4), "guaranteed": aid in GUARANTEED,
                      "keeps": {"traits": list(row["traits"]), "facts": [], "links": [], "sex": row["sex"]},
                      "age": row["age"], "voice": {"manner": row["voice"][0], "habit": row["voice"][1]},
                      "peoples": row["peoples"], "sex": row["sex"]})
ROWI = {a["id"]: a for a in arch_rows}
for a, b, t, asa, asb, note in LINKS:
    if t != "family": continue
    pa, pb = people[fill[a]], people[fill[b]]
    ga, gb = pa["id"].startswith("p-d0-"), pb["id"].startswith("p-d0-")
    if ga != gb:
        g, c = (pa, pb) if ga else (pb, pa)
        shared = c.get("_family") and c["_family"] in g["name"]
        LOG.append(f"  kin: {g['name']} (generated) and {c['name']} (canon), {note}: family name {'shared: ' + c['_family'] if shared else 'not shared'}")
        if not shared:
            RULES.append(f"KIN-NAME: generated {g['id']} ({g['name']}) is kin of canon {c['id']} ({c['name']}), whose byname is personal, not a family name; casting §4.3 asks for a shared family name, rulebook *Name check* exempts canon names, so nothing can be shared. Kept the generated filler's own byname.")
for a in arch_rows:   # the row's keeps.traits read the role-neutral form
    a["keeps"]["traits"] = [t.replace("{she}", "she/he") for t in a["keeps"]["traits"]]

# ---- facts
for aid, rows in sorted(ARCH_FACTS.items()):
    pid = fill[aid]
    for i, rw in enumerate(rows):
        text, vis, kind = rw[0], rw[1], rw[2]
        when = rw[3] if len(rw) > 3 else None
        fid = fact(f"x-d0-{aid[2:]}-{i+1}", fmt(text, pid), vis, kind, when)
        people[pid]["facts"].append(fid)
        ROWI[aid]["keeps"]["facts"].append(fid)
for slug, aids, text, vis, kind in SHARED_FACTS:
    t = text
    for a in aids:
        t = t.replace("{" + a + "}", people[fill[a]]["name"])
    fid = fact(f"x-d0-{slug}", t, vis, kind)
    for a in aids:
        people[fill[a]]["facts"].append(fid); ROWI[a]["keeps"]["facts"].append(fid)

# ---- links
def link(a, b, typ, as_a=None, as_b=None, vis="public", when="before the story", note=None):
    la = {"type": typ, "to": b, "as": as_a, "visibility": vis, "when": when}
    lb = {"type": typ, "to": a, "as": as_b, "visibility": vis, "when": when}
    if note: la["note"] = note; lb["note"] = note
    people[a]["links"].append(la); people[b]["links"].append(lb)

for a, b, t, asa, asb, note in LINKS:
    # sibling words follow the filler: a "sister" relation is kept as a sibling link whatever the filler's sex
    link(fill[a], fill[b], t, asa, asb, note=note)
    ROWI[a]["keeps"]["links"].append({"type": t, "to": b, "as": asa, "note": note})
    ROWI[b]["keeps"]["links"].append({"type": t, "to": a, "as": asb, "note": note})

# ---------------------------------------------------------------- classes (casting §3), data rows (classes.ron)
# Only the pre-made classes are made here: one per seat on the map. Hall and contact keepers (chapter captains,
# reading-room masters, ring-masters, ladder-keepers, back-room keepers, market contacts, letter-writers,
# contract-board veterans, parish priests, tithe-masters, pilots' hall masters, recruiters, stewards, factors,
# fences, debt brokers, toll captains, harbour-masters, eldest copyist) are on demand (casting §3).
CLASSES = {
 # name: standing, faction (id, sub) or None, traits, people weights (None = place mix), age weights, population
 "Banner Lord":       ("High", ("f-great-house", "f-banner-lords"), ["noble", "holds legal authority"], {"roduro": 4, "tadoro": 1, "qotiro": 1}, {"grown": 2, "old": 2, "young": 0.5}, "pre-made"),
 "Free Lord":         ("High", ("f-great-house", "f-free-lords"), ["noble", "holds legal authority", "buys bonded labour"], {"roduro": 3, "qotiro": 2, "tadoro": 1}, {"grown": 2, "old": 2, "young": 0.5}, "pre-made"),
 "crown gate-warden": ("Middle", ("f-great-house", "f-throne"), ["holds legal authority"], {"roduro": 4, "tadoro": 1, "qotiro": 1}, {"grown": 3, "old": 1, "young": 1}, "pre-made"),
 "house head":        ("High", ("f-first-peoples", "f-seated-houses"), ["noble", "holds legal authority"], {"roduro": 5, "horaro": 1}, {"grown": 2, "old": 3, "young": 0.5}, "pre-made"),
 "headman":           ("Middle", None, ["holds legal authority"], None, {"grown": 2, "old": 2, "young": 0.5}, "pre-made"),
 "town council head": ("Middle", ("f-open-hand", "f-open-council"), ["holds legal authority", "merchant"], None, {"grown": 3, "old": 1, "young": 0.5}, "pre-made"),
 "enclave priestess": ("Middle", ("f-last-wave", None), ["priestess", "holds legal authority"], {"qotiro": 1}, {"grown": 2, "old": 2, "young": 0.5}, "pre-made"),
 "rock-keeper":       ("Middle", ("f-far-sails", None), ["pilot", "keeps the beacon"], {"horaro": 1}, {"grown": 2, "old": 1, "young": 1}, "pre-made"),
}
ON_DEMAND = ["chapter captain", "contract-board veteran", "reading-room master", "eldest copyist", "letter-writer", "ring-master",
             "ladder-keeper", "back-room keeper", "house steward", "house factor", "market contact", "fence", "parish priest",
             "tithe-master", "debt broker", "pilots' hall master", "Warden recruiter", "toll captain", "harbour-master"]

def make(cls, slug, place, title, sexfix=None, faction=None, extra_traits=(), peoples=None):
    st, fac, traits, pmix, ages, _ = CLASSES[cls]
    pid = "p-d0-" + slug
    assert pid not in people, pid
    mix = dict(peoples) if peoples else (dict(pmix) if pmix else place_mix(place))
    ppl = wpick(K(pid, "make", "people"), mix)
    sex = sexfix or ("woman" if K(pid, "make", "sex").random() < 0.5 else "man")
    name = roll_name(pid, ppl, sex)
    age = wpick(K(pid, "make", "age"), ages)
    fac = faction or fac
    p = {"id": pid, "name": name, "standing": st, "archetype": None, "class": cls,
         "traits": list(traits) + list(extra_traits) + [title], "links": [],
         "faction": ({"id": fac[0], "sub": fac[1]} if fac else None), "facts": [], "premade": True, "dead": False,
         "home": place, "dies_at": None, "fallback": None, "people": ppl, "age": age,
         "voice": roll_voice(pid, ppl), "_sex": sex, "_family": None}
    people[pid] = p
    LOG.append(f"class {cls}: {pid} {name} ({ppl}, {sex}, {age}) at {place}: {title}")
    return pid

# ---------------------------------------------------------------- seats and offices
seats = {}
def seat(place, holder):
    seats.setdefault(place, {"holder": None, "titles": []})
    assert seats[place]["holder"] is None, place
    seats[place]["holder"] = holder
def title(place, t, holder, since="before the story"):
    seats.setdefault(place, {"holder": None, "titles": []})
    assert all(x["title"] != t for x in seats[place]["titles"]), (place, t)
    seats[place]["titles"].append({"title": t, "holder": holder, "since": since})

F = fill
# First Peoples
seat("pl-ninth-ring", F["a-seated-head"])
title("pl-ninth-ring", "head of the Old Hands (the Old Gate House)", F["a-old-don"])
seat("pl-firststone", F["a-true-line-schemer"])
title("pl-firststone", "face of the True Line (co-leader)", F["a-true-line-face"])
seat("pl-mount", F["a-high-priest"])
seat("pl-reefhold", F["a-tide-speaker"])
eh = make("house head", "elderhollow-head", "pl-elderhollow", "head of Elderhollow's ruling house")
seat("pl-elderhollow", eh)
title("pl-elderhollow", "keeper of the Low Shrine", F["a-sincere-priestess"])
seat("pl-ashring", make("headman", "ashring-head", "pl-ashring", "headman of Ashring",
                        faction=("f-first-peoples", "f-seated-houses"), extra_traits=("of an old Vale house",), peoples={"roduro": 1}))
seat("pl-shellring", make("headman", "shellring-head", "pl-shellring", "headman of Shellring",
                          faction=("f-first-peoples", "f-tide-houses"), peoples={"horaro": 1}))
seat("pl-green-wall", F["a-warden-gate"])
title("pl-green-wall", "first scout of the Wall (co-leader)", F["a-warden-scout"])
# Great House
seat("pl-highcrossing", F["a-emperor"])
title("pl-highcrossing", "keeper of the Throne's court", F["a-throne-voice"])
title("pl-highcrossing", "head of the Writers (the Wayhouse; never there)", F["a-writers-leader"])
seat("pl-longwall", F["a-banner-general"])
seat("pl-southroad", F["a-banner-climber"])
oath_lord = make("Banner Lord", "oathford-lord", "pl-oathford", "Banner Lord of Oathford")
seat("pl-oathford", oath_lord)
cg = make("crown gate-warden", "crownsgate-warden", "pl-crownsgate", "gate-warden of Crownsgate")
seat("pl-crownsgate", cg)
link(F["a-emperor"], cg, "employer/employee", "employer", "employee", note="class: a crown gate-warden serves the Throne")
# Free Lords' frontier
seat("pl-breakwall", F["a-free-lord-breaker"])
seat("pl-chainmarket", F["a-binder-head"])
seat("pl-bogford", make("Free Lord", "bogford-lord", "pl-bogford", "Free Lord of Bogford"))
# Open Hand
seat("pl-openharbour", F["a-council-head"])
title("pl-openharbour", "head of the Night Market (Under the Market Road)", F["a-market-fence"])
title("pl-openharbour", "head of the Dueling Society (the Ring)", F["a-dueling-showman"])
title("pl-openharbour", "co-head of the Dueling Society (the Ring)", F["a-dueling-sister"])
seat("pl-counting-house", F["a-sharp-broker"])
title("pl-counting-house", "partner who deals (Sharp Hands co-leader)", F["a-sharp-twin-deals"])
title("pl-counting-house", "partner who keeps the count (Sharp Hands co-leader)", F["a-sharp-twin-counts"])
seat("pl-carters-rest", F["a-road-foreman"])
title("pl-carters-rest", "head of the Lantern Company (Lanternhall)", F["a-lantern-quartermaster"])
sm = make("town council head", "saltmarket-head", "pl-saltmarket", "head of the Saltmarket council")
seat("pl-saltmarket", sm)
wy = make("headman", "wainyard-head", "pl-wainyard", "yard-boss of Wainyard",
          faction=("f-open-hand", "f-road-crews"), extra_traits=("Road Crews carter",))
seat("pl-wainyard", wy)
link(F["a-road-foreman"], wy, "employer/employee", "employer", "employee", note="class: the yard-boss answers to the crews' foreman")
# Last Wave, Highfolk, Far Sails
seat("pl-textrim", F["a-wave-priestess"])
seat("pl-proving", F["a-wave-warchief"])
ip = make("enclave priestess", "ironstep-priestess", "pl-ironstep", "priestess of Ironstep", sexfix="woman")
seat("pl-ironstep", ip)
link(F["a-wave-priestess"], ip, "employer/employee", "employer", "employee", note="class: an enclave priestess answers to the High Priestess")
seat("pl-high-snows", F["a-highfolk-head"])
seat("pl-anchorage", F["a-fleet-speaker"])
rk = make("rock-keeper", "rock-keeper", "pl-lantern-rock", "keeper of the Lantern Rock beacon")
seat("pl-lantern-rock", rk)
link(F["a-fleet-speaker"], rk, "employer/employee", "employer", "employee", note="class: the beacon's keeper serves the fleet")
# the Soft Hollow (the Rooted have no fixed leader), Wellstone (lost), the Womb and the Heart (secret): no seat.

# ---------------------------------------------------------------- layer 1: court state (casting §5), data rows (tensions.ron)
def N(aid):
    nm = people[fill[aid]]["name"]
    return nm + "," if ", " in nm else nm   # every tension template puts the name before a verb
TENSIONS = [
 {"id": "t-banner-race", "name": "The Banner Lords' succession race", "archetypes": ["a-banner-general", "a-banner-climber", "a-emperor", "a-throne-voice"],
  "majors": ["f-great-house"], "weight": 3, "touches": ["f-great-house", "f-banner-lords", "f-throne"],
  "why_majors": "Longwall, Southroad and Highcrossing are Great House land",
  "facts": [lambda: f"{N('a-emperor')} has named no heir, and the court talks openly of whose banner would stand first if the emperor died: that of {N('a-banner-general')} at Longwall, or that of {N('a-banner-climber')} at Southroad.",
            lambda: f"{N('a-banner-climber')} has said before the court that friends of {N('a-banner-general')} are behind the damage to the Great Bridge.",
            lambda: f"Both Banner Lords now send gifts to {N('a-throne-voice')}, and the court waits to see whose she answers.".replace("she answers", ("she" if people[fill['a-throne-voice']]['_sex'] == 'woman' else "he") + " answers")],
  "links": [("a-banner-climber", "a-banner-general", "grudge", "holder", "target")],
  "tagged": ["a-banner-general", "a-banner-climber", "a-throne-voice"]},
 {"id": "t-schism", "name": "The Mount against the Low Shrine", "archetypes": ["a-high-priest", "a-sincere-priestess"],
  "majors": ["f-first-peoples"], "weight": 3, "touches": ["f-holy-house", "f-first-peoples"],
  "why_majors": "the Mount and Elderhollow are First Peoples land, and the house leans on the Alliance",
  "facts": [lambda: f"{N('a-high-priest')} has ordered the Low Shrine's soup kitchen at Elderhollow closed; {N('a-sincere-priestess')} keeps it open every evening.",
            lambda: f"Pilgrims who stopped climbing to the Mount now leave their coin at the Low Shrine, and the Mount's preachers call {N('a-sincere-priestess')}'s way a heresy."],
  "links": [("a-high-priest", "a-sincere-priestess", "grudge", "holder", "target")],
  "tagged": ["a-high-priest", "a-sincere-priestess"]},
 {"id": "t-tide-term", "name": "The Tide speaker's season runs out", "archetypes": ["a-tide-speaker", "a-fleet-speaker"],
  "majors": ["f-first-peoples"], "weight": 3, "touches": ["f-first-peoples", "f-tide-houses", "f-far-sails"],
  "why_majors": "Reefhold is First Peoples land; the Far Sails are Horaro who left it",
  "facts": [lambda: f"{N('a-tide-speaker')}'s season on the Speaker's Deck ends at the next turning, and the Tide Houses will choose again.",
            lambda: f"The captains on the Raft have voted to send {N('a-fleet-speaker')}'s word to whoever sits the Deck next, and the Seated Houses have warned Reefhold not to hear it."],
  "links": [],
  "tagged": ["a-tide-speaker", "a-fleet-speaker"]},
 {"id": "t-firststone-claim", "name": "Firststone's claim on the Ninth Ring", "archetypes": ["a-seated-head", "a-true-line-schemer", "a-true-line-face"],
  "majors": ["f-first-peoples"], "weight": 2, "touches": ["f-first-peoples", "f-seated-houses", "f-true-line"],
  "why_majors": "both houses are First Peoples",
  "facts": [lambda: f"The True Line has petitioned to read the Ninth Ring's oldest house rolls, and {N('a-seated-head')} has refused them.",
            lambda: f"{N('a-true-line-face')} has told the pilgrims at the Founding Stone that the Seated Houses keep the archive shut because it would not bear reading."],
  "links": [("a-true-line-schemer", "a-seated-head", "grudge", "holder", "target")],
  "tagged": ["a-seated-head", "a-true-line-schemer", "a-true-line-face"]},
 {"id": "t-long-shed", "name": "The Long Shed vote", "archetypes": ["a-road-foreman", "a-sharp-twin-counts"],
  "majors": ["f-open-hand"], "weight": 2, "touches": ["f-open-hand", "f-road-crews", "f-sharp-hands"],
  "why_majors": "Carter's Rest and the Counting-House are Open Hand",
  "facts": [lambda: f"{N('a-road-foreman')} has called the Road Crews to vote in the Long Shed on striking against the Counting-House's rates.",
            lambda: f"{N('a-sharp-twin-counts')} has let it be known that any crew that strikes will have its notes called in."],
  "links": [("a-road-foreman", "a-sharp-twin-counts", "grudge", "holder", "target")],
  "tagged": ["a-road-foreman", "a-sharp-twin-counts"]},
 {"id": "t-quay-block", "name": "The Quay against the Block", "archetypes": ["a-council-head", "a-binder-head"],
  "majors": ["f-open-hand", "f-great-house"], "weight": 3, "touches": ["f-open-hand", "f-open-council", "f-binders", "f-free-lords"],
  "why_majors": "Openharbour is Open Hand; the Chainmarket stands on the Free Lords' frontier, Great House land",
  "facts": [lambda: f"{N('a-council-head')} has asked the Open Council to bar from Open Hand roads any caravan that has passed through the Chainmarket.",
            lambda: f"{N('a-binder-head')} has answered that the Block remembers every Open Hand debtor sold on it."],
  "links": [("a-binder-head", "a-council-head", "grudge", "holder", "target")],
  "tagged": ["a-council-head", "a-binder-head"]},
 {"id": "t-syndicates", "name": "The syndicates' cold war", "archetypes": ["a-old-don", "a-market-fence"],
  "majors": ["f-first-peoples", "f-open-hand"], "weight": 3, "touches": ["f-old-hands", "f-night-market", "f-first-peoples", "f-open-hand"],
  "why_majors": "the Old Gate House is in the Ninth Ring (First Peoples); Under the Market Road is in Openharbour (Open Hand)",
  "facts": [lambda: f"Old Hands and Night Market men no longer drink in the same houses, and {N('a-old-don')} has put it about that {N('a-market-fence')}'s people are not to be served behind the Old Gate."],
  "links": [("a-old-don", "a-market-fence", "grudge", "holder", "target")],
  "tagged": ["a-old-don", "a-market-fence"]},
 {"id": "t-rim-war", "name": "War talk on the Rim", "archetypes": ["a-wave-warchief", "a-wave-priestess"],
  "majors": ["f-great-house"], "weight": 2, "touches": ["f-last-wave", "f-free-lords"],
  "why_majors": "the lowlands below the Rim are the Free Lords' frontier, Great House land",
  "facts": [lambda: f"Young warriors at the Proving Ground call for a raid below the Rim; {N('a-wave-warchief')} has forbidden it until the Height's stores are counted.",
            lambda: f"{N('a-wave-priestess')} has sent word below the Rim that the Height will hear any offer of grain made with respect."],
  "links": [],
  "tagged": ["a-wave-warchief", "a-wave-priestess"]},
]
cr = K("court", "count").random()
count = 3 if cr < 0.5 else 4
LOG.append(f"court: K[{SEED},court,count] = {cr:.4f} -> {count} tensions")
picked, used_majors = [], set()
for slot in range(1, count + 1):
    cand = {t["id"]: t["weight"] for t in TENSIONS
            if t["id"] not in [p["id"] for p in picked]
            and not (set(t["majors"]) & used_majors)
            and all(a in fill for a in t["archetypes"])}
    if not cand:
        LOG.append(f"court slot {slot}: no candidate left (majors used: {sorted(used_majors)})")
        RULES.append(f"COURT-COUNT: the count roll asked for {count} tensions but slot {slot} had no candidate: with three majors and every tension touching one or two, 'at most one per major' caps the court at 3 when a two-major tension is picked early (casting §5)")
        break
    rng = K("court", slot)
    tid = wpick(rng, cand)
    t = next(x for x in TENSIONS if x["id"] == tid)
    LOG.append(f"court slot {slot}: K[{SEED},court,{slot}] candidates {sorted(cand.items())} -> {tid} (majors {t['majors']})")
    picked.append(t); used_majors |= set(t["majors"])

tensions_out = []
for t in picked:
    fids = []
    for i, f in enumerate(t["facts"]):
        fids.append(fact(f"x-d0-{t['id']}-{i+1}", f(), "public", "state", source="world"))
    for a, b, typ, asa, asb in t["links"]:
        link(fill[a], fill[b], typ, asa, asb, note=f"court state {t['id']}")
    ppl = [fill[a] for a in t["tagged"]]
    for pid in ppl:
        people[pid]["traits"].append(f"tension: {t['id']}")
        for fid in fids:
            if people[pid]["name"] in next(x["text"] for x in facts if x["id"] == fid) and fid not in people[pid]["facts"]:
                people[pid]["facts"].append(fid)
    tensions_out.append({"id": t["id"], "name": t["name"], "people": ppl, "archetypes": t["archetypes"], "facts": fids,
                         "links": [{"type": typ, "from": fill[a], "to": fill[b], "as": [asa, asb]} for a, b, typ, asa, asb in t["links"]],
                         "touches": t["touches"], "majors": t["majors"], "why_majors": t["why_majors"]})

# ---------------------------------------------------------------- checks
office = {}
for pl, s in seats.items():
    for h in ([s["holder"]] if s["holder"] else []) + [x["holder"] for x in s["titles"]]:
        assert h in people, (pl, h)
        office.setdefault(h, []).append(pl)
two = {h: v for h, v in office.items() if len(v) > 1}
assert not two, two
names = [p["name"] for p in people.values()]
assert len(names) == len(set(names)), "full name clash"
gen = [p for p in people.values() if p["id"].startswith("p-d0-")]
stems = [plain(p["name"].split(" ")[0])[:4] for p in gen]
assert len(stems) == len(set(stems)), "stem clash"
settlements = [p["id"] for p in mp["places"] if p["size"] not in ("presence", "site")]
missing = [s for s in settlements if s not in seats]
faction_seats = sorted({f["seat"] for f in canon["factions"]})
missing_seats = [s for s in faction_seats if s not in seats]
LOG.append(f"check: settlements without a holder: {missing}; faction seats without a holder: {missing_seats}")
LOG.append(f"check: Top people: {[p['id'] for p in people.values() if p['standing'] == 'Top']}")
replaced = [a["default"] for a in arch_rows if a["canon"] == "no"]
for p in people.values():
    for l in p["links"]:
        assert l["to"] not in replaced, (p["id"], l)

# ---------------------------------------------------------------- write
ORDER = ["id", "name", "standing", "archetype", "class", "traits", "links", "faction", "facts", "premade", "dead",
         "home", "dies_at", "fallback", "people", "age", "voice"]
out_people = [{k: people[pid].get(k) for k in ORDER} for pid in sorted(people)]
out_places = []
for pl in sorted(seats):
    s = seats[pl]
    e = {"id": pl, "name": PLACES[pl]["name"]}
    if s["holder"]: e["holder"] = s["holder"]
    if s["titles"]: e["titles"] = s["titles"]
    out_places.append(e)
faction_leaders = {}
BY_DEFAULT = {a["default"]: a["filled_by"] for a in arch_rows}
for f in canon["factions"]:
    faction_leaders[f["id"]] = [BY_DEFAULT[x] for x in f["leaders"]]
out = {
 "world": "D", "layer": "0-1", "seed": SEED,
 "registry": {"people": out_people, "places": out_places},
 "facts": sorted(facts, key=lambda f: f["id"]),
 "tensions": tensions_out,
 "archetypes": arch_rows,
 "faction_leaders": faction_leaders,
}
json.dump(out, open(os.path.join(HERE, "d-l0.json"), "w"), ensure_ascii=False, indent=1)
genA = [a for a in arch_rows if a["canon"] == "no"]
print(f"archetypes {len(arch_rows)}: canon {len(arch_rows)-len(genA)}, generated {len(genA)}")
for a in genA: print("  generated", a["id"], "->", people[a["filled_by"]]["name"], "| replaces", a["default"])
print(f"seat holders {sum(1 for s in seats.values() if s['holder'])}, titles {sum(len(s['titles']) for s in seats.values())}, places {len(out_places)}")
print(f"people {len(people)}: canon {len(people)-len(gen)}, generated {len(gen)} ({len(genA)} fillers, {len([p for p in gen if p['class']])} class)")
print("tensions:", [(t['id'], t['majors']) for t in tensions_out])
print("facts:", len(facts))
for l in LOG: print(l)
for r in RULES: print("RULE", r)
