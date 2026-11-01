"""World D layer 2: people the realm layer creates. Names, people, age and voice are keyed rolls
K[6601, person id, "make", field] (casting §7, §8), from the naming module's lists."""
import json, os, re, random, unicodedata
HERE = os.path.dirname(os.path.abspath(__file__)); W = os.path.dirname(os.path.dirname(HERE))
LANG = "/home/claude/gahturiyu-sim/assets/lang"
def K(*keys): return random.Random("6601|" + "|".join(map(str, keys)))
def plain(s):
    s = unicodedata.normalize("NFD", s); s = "".join(c for c in s if unicodedata.category(c) != "Mn")
    return s.replace("ʻ", "").replace("'", "").lower()
NAMES = {}
for ppl in ("roduro", "qotiro", "horaro", "tadoro"):
    txt = open(os.path.join(LANG, "names", ppl + ".ron"), encoding="utf-8").read()
    NAMES[ppl] = re.findall(r'\(name: "([^"]+)", gender: (Male|Female|Either)', txt)
canon = json.load(open(os.path.join(W, "canon-D.json"))); l0 = json.load(open(os.path.join(W, "D", "d-l0.json")))
existing = [p["name"] for p in canon["people"]] + [p["name"] for p in l0["registry"]["people"]]
used_full = {n.lower() for n in existing}
used_stem = {plain(n.split()[0])[:4] for n in existing}
used_byname = {" ".join(n.split()[1:]) for n in existing}
HOUSE_MARKS = ["moss", "vine", "grass", "thorn", "tree", "spring", "rain", "frost", "gull", "smoke", "ember", "moon", "star", "herb", "bark", "wind"]
HOUSE_SITES = ["ledge", "slope", "cliff", "crag", "hollow", "hill", "bank", "ridge", "brow", "stream", "field", "forest", "shore", "cave"]
HOUSE_RINGS = ["three", "four", "five", "six", "seven", "eight", "nine", "ten", "hundred"]
DEEDS = ["Iron-breaker", "Stone-breaker", "Shield-breaker", "Spear-breaker", "Bone-breaker", "Gate-holder", "Pass-holder", "Wall-warden",
         "Hearth-warden", "Road-warden", "Law-keeper", "Oath-keeper", "Storm-walker", "Night-walker", "Sand-walker", "Fire-walker",
         "Beast-hunter", "Hound-hunter", "Thunder-striker", "First-striker", "Spear-bearer", "Torch-bearer", "Shield-bearer",
         "Fire-binder", "Ore-finder", "Spring-finder", "Way-finder", "Far-seer", "Horn-cutter", "Rope-cutter", "Gorge-leaper",
         "Wall-leaper", "Spear-thrower", "Ember-bearer", "Crag-runner", "Dawn-runner", "Hall-builder", "Stair-builder"]
SEA_MARKS = ["kelp", "pearl", "gull", "foam", "moon", "deep", "calm", "shell", "eel", "rain", "far", "blue", "green", "white", "reed", "salt", "storm", "star", "old", "quiet", "long", "red"]
SEA_SITES = ["reef", "bay", "cove", "shore", "stack", "island", "harbour", "pool", "headland", "sand", "mouth"]
ROADS = ["north", "east", "west", "high", "far", "long", "old", "wind", "star", "dawn", "dusk", "hill", "salt", "silk", "sea", "stone", "rain", "quiet", "white"]
MANNERS = ["blunt", "careful", "warm", "wry", "weary", "proud"]
HABITS = {"roduro": ["traveller", "names_the_dead", "so_first", "counts_seasons", "stone_keep_you", "speaks_of_the_house"],
          "qotiro": ["one_word", "decide", "curses_the_damp", "eat", "no_goodbye"],
          "horaro": ["have_you_eaten", "out_of_the_wind", "we_for_all", "talks_tide", "another_day", "waits_for_a_sign"],
          "tadoro": ["have_you_noticed", "guesses_where_from", "corrects_self", "writes_it_down", "listen_first", "what_would_you_do"]}
MIX = {"great": {"roduro": 5, "tadoro": 2, "qotiro": 2, "horaro": 1}, "free": {"qotiro": 4, "roduro": 3, "tadoro": 1, "horaro": 1},
       "harbour": {"horaro": 3, "roduro": 2, "qotiro": 2, "tadoro": 2}, "roads": {"roduro": 3, "qotiro": 2, "tadoro": 2, "horaro": 2},
       "vale": {"roduro": 6, "horaro": 2, "qotiro": 1, "tadoro": 1}, "wave": {"qotiro": 1}}
def wpick(r, d):
    tot = sum(d.values()); x = r.random() * tot
    for k in sorted(d):
        x -= d[k]
        if x < 0: return k
    return sorted(d)[-1]
def byname(r, ppl):
    if ppl == "roduro":
        if r.random() < 0.25: return f"of the {r.choice(HOUSE_RINGS).capitalize()}-Ring House"
        return f"of the {r.choice(HOUSE_MARKS).capitalize()}-{r.choice(HOUSE_SITES).capitalize()} House"
    if ppl == "qotiro": return r.choice(DEEDS)
    if ppl == "horaro": return f"of the {r.choice(SEA_MARKS).capitalize()} {r.choice(SEA_SITES).capitalize()}"
    return f"of the {r.choice(ROADS).capitalize()} Road"
PLACE_WORDS = ["crownsgate", "oathford", "southroad", "longwall", "breakwall", "bogford", "highcrossing", "saltmarket", "wainyard"]
def make_name(pid, ppl, sex, family=None):
    want = {"man": ("Male", "Either"), "woman": ("Female", "Either")}[sex]
    pool = sorted(n for n, g in NAMES[ppl] if g in want)
    for i in range(500):
        r = K(pid, "make", "name", i); g = r.choice(pool)
        stem = plain(g)[:4]
        if stem in used_stem: continue
        b = family or byname(K(pid, "make", "byname", i), ppl)
        if not family and b in used_byname: continue
        full = g[0].upper() + g[1:] + " " + b
        if full.lower() in used_full: continue
        used_stem.add(stem); used_full.add(full.lower()); used_byname.add(b)
        return full, b
    raise SystemExit("names exhausted " + pid)
SPECS = []  # (id, sex, people or mix, age weights, family-of id or None)
def spec(pid, sex, ppl, age=None, family=None): SPECS.append((pid, sex, ppl, age or {"young": 1, "grown": 2, "old": 1}, family))
# the Gate Mill's house (Crownsgate, the Crown Crossing)
spec("p-d-main-heart", "woman", "great", {"grown": 1})
spec("p-d-main-uncle", "man", "@p-d-main-heart", {"old": 1}, "p-d-main-heart")
spec("p-d-main-brother", "man", "@p-d-main-heart", {"young": 2, "grown": 1}, "p-d-main-heart")
spec("p-d-main-cousin", "man", "@p-d-main-heart", {"grown": 1}, "p-d-main-heart")
spec("p-d-main-widow", "woman", "@p-d-main-heart", {"old": 1}, "p-d-main-heart")
spec("p-d-main-millhand", "man", "great", {"old": 1})
spec("p-d-main-hand", "man", "great", {"grown": 2, "old": 1})
spec("p-d-main-bridgekeeper", "man", "great", {"grown": 1, "old": 1})
spec("p-d-main-bridgeson", "man", "@p-d-main-bridgekeeper", {"young": 1}, "p-d-main-bridgekeeper")
spec("p-d-main-sergeant", "man", "great", {"grown": 1})
spec("p-d-main-sisterinlaw", "woman", "great", {"young": 1, "grown": 1})
spec("p-d-main-carter", "man", "roads")
spec("p-d-main-stallwife", "woman", "great", {"grown": 1, "old": 1})
spec("p-d-main-towerwarden", "man", "great", {"grown": 1, "old": 1})
spec("p-d-main-crossingkeeper", "woman", "great")
spec("p-d-main-raider", "man", "free", {"grown": 1})
spec("p-d-main-informer", "man", "roads")
spec("p-d-main-fortsergeant", "woman", "free")
spec("p-d-main-deserter", "man", "great", {"young": 1})
spec("p-d-main-tollreeve", "woman", "great")
spec("p-d-main-paymaster", "man", "great", {"grown": 1})
spec("p-d-main-ladderkeeper", "woman", "great", {"young": 1, "grown": 1})
spec("p-d-main-oldkeeper", "man", "great", {"old": 1})
spec("p-d-main-paidfighter", "man", "@p-d-main-oldkeeper", {"young": 1}, "p-d-main-oldkeeper")
spec("p-d-main-sapper", "man", "great")
spec("p-d-main-hurtcarter", "woman", "great")
spec("p-d-main-drover", "woman", "great", {"grown": 1, "old": 1})
spec("p-d-main-chclerk", "woman", "roads")
spec("p-d-main-ferrywoman", "woman", "great", {"grown": 1, "old": 1})
spec("p-d-main-ferryson", "man", "@p-d-main-ferrywoman", {"young": 1}, "p-d-main-ferrywoman")
spec("p-d-main-rivalferry", "man", "great")
spec("p-d-main-lad", "man", "great", {"young": 1})
spec("p-d-main-bandleader", "woman", "great")
spec("p-d-main-paperdealer", "man", "harbour")
spec("p-d-main-notary", "woman", "harbour")
spec("p-d-main-saltcarter", "man", "harbour")
spec("p-d-main-apprentice", "woman", "harbour", {"young": 1})
spec("p-d-main-factor", "man", "harbour")
spec("p-d-main-fordwarden", "woman", "great")
spec("p-d-main-remounter", "man", "free")
# ---- added with the card build (appended: earlier names don't move)
spec("p-d-main-tenthclerk", "man", "great", {"grown": 1})
spec("p-d-main-oldcarter", "man", "roads", {"old": 1})
spec("p-d-main-merchant", "man", "great", {"grown": 1, "old": 1})
spec("p-d-main-veteran", "man", "great", {"grown": 1, "old": 2})
spec("p-d-main-reedcutter", "woman", "great", {"old": 1})
spec("p-d-main-raidsecond", "woman", "free")
spec("p-d-main-olddrover", "man", "great", {"old": 1})
spec("p-d-main-sweetheart", "woman", "great", {"young": 1})
spec("p-d-main-writrider", "man", "great", {"young": 1})
spec("p-d-main-towerguard", "man", "great")
spec("p-d-main-host", "woman", "great", {"grown": 1, "old": 1})
spec("p-d-main-farmer", "man", "great")
spec("p-d-main-bridgedaughter", "woman", "@p-d-main-bridgekeeper", {"young": 1}, "p-d-main-bridgekeeper")
# minors
spec("p-d-main-wallreeve", "woman", "great", {"grown": 1, "old": 1})
# fallout
spec("p-d-main-f1-baker", "woman", "great")
spec("p-d-main-f1-breadcarter", "man", "great")
spec("p-d-main-f2a-wife", "woman", "free")
spec("p-d-main-f2a-debtor", "man", "@p-d-main-f2a-wife", None, "p-d-main-f2a-wife")
spec("p-d-main-f2a-keeper", "man", "free", {"grown": 1})
spec("p-d-main-f2a-neighbour", "man", "free")
spec("p-d-main-f2b-broker", "woman", "free", {"grown": 1, "old": 1})
spec("p-d-main-f2b-daughter", "woman", "free", {"young": 1})
spec("p-d-main-f2l-grainwife", "woman", "free")
spec("p-d-main-f2l-carrier", "man", "free")
spec("p-d-main-f2l-postguard", "man", "great")
spec("p-d-main-f3a-rider", "man", "great", {"young": 1})
spec("p-d-main-f3a-brother", "man", "@p-d-main-f3a-rider", None, "p-d-main-f3a-rider")
spec("p-d-main-f3b-carter", "woman", "roads")
spec("p-d-main-f3b-potboy", "man", "great", {"young": 1})
spec("p-d-main-f3b-drover", "man", "great")
spec("p-d-main-f3l-freeman", "man", "free")
spec("p-d-main-f3l-rider", "woman", "great", {"young": 1})
spec("p-d-main-f3l-keeper", "woman", "free")
spec("p-d-main-f4a-boss", "man", "roads", {"grown": 1})
spec("p-d-main-f4b-tollwife", "woman", "great")
spec("p-d-main-f4b-brother", "man", "@p-d-main-f4b-tollwife", None, "p-d-main-f4b-tollwife")
spec("p-d-main-f4b-tollman", "man", "great")
spec("p-d-main-f4l-fighter", "man", "roads", {"young": 1})
spec("p-d-main-f4l-opponent", "woman", "roads")
spec("p-d-main-f4l-keeper", "man", "roads", {"grown": 1, "old": 1})
spec("p-d-main-f5a-sealcutter", "man", "great", {"grown": 1})
spec("p-d-main-f5a-halfbrother", "man", "@p-d-main-f5a-sealcutter", None, "p-d-main-f5a-sealcutter")
spec("p-d-main-f5a-father", "man", "@p-d-main-f5a-sealcutter", {"old": 1}, "p-d-main-f5a-sealcutter")
spec("p-d-main-f5b-weaver", "woman", "harbour")
spec("p-d-main-f5b-reeve", "man", "harbour", {"grown": 1, "old": 1})
spec("p-d-main-f5l-sister", "woman", "vale")
spec("p-d-main-f5l-taker", "man", "vale")
spec("p-d-main-f5l-dead", "man", "@p-d-main-f5l-sister", None, "p-d-main-f5l-sister")
spec("p-d-main-f6a-farmwife", "woman", "vale", {"grown": 1, "old": 1})
spec("p-d-main-f6a-son", "man", "@p-d-main-f6a-farmwife", {"young": 1}, "p-d-main-f6a-farmwife")
spec("p-d-main-f6a-leader", "man", "great", {"grown": 1})
spec("p-d-main-f6b-widow", "woman", "great")
spec("p-d-main-f6b-dead", "man", "great")
spec("p-d-main-f6b-culprit", "man", "great")
spec("p-d-main-f6l-captain", "woman", "great", {"grown": 1, "old": 1})
spec("p-d-main-f6l-quartermaster", "man", "great")
spec("p-d-main-f3a-mother", "woman", "@p-d-main-f3a-rider", {"old": 1}, "p-d-main-f3a-rider")
OUT = {}
for pid, sex, ppl, age, fam in SPECS:
    if ppl.startswith("@"): pp = OUT[ppl[1:]]["people"]
    else: pp = wpick(K(pid, "make", "people"), MIX[ppl])
    famname = OUT[fam]["_by"] if fam else None
    if fam and pp == "qotiro": famname = None   # Qotiro bynames are personal deeds (l0 rule problem 1)
    name, by = make_name(pid, pp, sex, famname)
    a = wpick(K(pid, "make", "age"), age)
    ms = {m: 1 for m in MANNERS}
    if pp == "tadoro": ms["solitary"] = 1
    v = {"manner": wpick(K(pid, "make", "manner"), ms), "habit": K(pid, "make", "habit").choice(HABITS[pp])}
    OUT[pid] = {"name": name, "_by": by, "people": pp, "age": a, "voice": v, "_sex": sex}
if __name__ == "__main__":
    for k, v in OUT.items(): print(k, v["name"], v["people"], v["age"], v["voice"], v["_sex"])
