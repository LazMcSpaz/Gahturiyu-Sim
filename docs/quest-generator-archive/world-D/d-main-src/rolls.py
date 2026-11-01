"""World D, layer 2 (realm): every roll for the main story, in order, from random.Random(6601).
Append-only: new rolls go at the end so earlier ones never change. Run to print the log and write rolls.json."""
import random, json, math
R = random.Random(6601)
LOG = []
def roll(name, options, weights=None, note=""):
    r = R.random()
    if weights is None: weights = [1] * len(options)
    tot = sum(weights); acc = 0
    for o, w in zip(options, weights):
        acc += w / tot
        if r < acc: break
    LOG.append({"roll": name, "r": round(r, 4), "options": [str(o) for o in options], "weights": weights, "pick": str(o), "note": note})
    return o
def chance(name, p, note=""):
    r = R.random(); ok = r < p
    LOG.append({"roll": name, "r": round(r, 4), "p": p, "pick": ok, "note": note}); return ok

# 1. arc (main-quest §6 weights)
arc = roll("arc", ["Rising", "Ripple", "Inward", "Deep"], [35, 25, 20, 20])
# 2. enemy shape: Rising allows Above only (no roll)
LOG.append({"roll": "shape", "pick": "Above", "note": "Rising allows only Above (main-quest §6)"})
# 3. L (Rising 5-8, uniform), then W in 2..floor((L+1)/2) (assembly §3)
L = roll("L", [5, 6, 7, 8], note="Rising range 5-8, uniform (main-quest §3)")
W = roll("W", list(range(2, (L + 1) // 2 + 1)), note="2 <= W <= floor((L+1)/2)")
# 4. antagonist's side: a faction of realm reach, or a sub-faction borrowing its parent's (Rising); never a faction with a
#    faction story (the five guilds, the Holy House: faction-stories §8)
SIDES = ["f-great-house", "f-throne", "f-banner-lords", "f-free-lords", "f-first-peoples", "f-seated-houses", "f-true-line",
         "f-tide-houses", "f-open-hand", "f-open-council", "f-sharp-hands", "f-road-crews"]
side = roll("antagonist side", SIDES, note="equal weights")
# 5. want (shared list; the faction's own want weighted x2)
want = roll("want", ["greed", "revenge", "cover-up", "desperation", "ambition", "preservation"], [1, 1, 1, 1, 1, 2],
            note="the Throne's own want (preservation) x2")
# 6. hook settlement: Middle standing (a community landmark, main-quest §4), where a crown due can fall: Great House land, or an
#    Open Hand town on the old imperial road
hook_town = roll("hook settlement", ["pl-oathford", "pl-crownsgate", "pl-saltmarket", "pl-carters-rest"])

# 7. hook card family (assembly §4 hook table; Reckoning Betrayal is Within only, so out), then its premise
hook_family = roll("hook family", ["Investigate", "Threat", "Rescue", "Dispute", "Acquire", "Passage"])
HOOK_PREM = {"Investigate": ["Theft", "Disappearance", "Suspicious death", "Sabotage"],
             "Threat": ["Raiders", "Protected beast", "Sickness or blight", "Encroachment"],
             "Rescue": ["Kidnapped", "Captured in battle", "Imprisoned unjustly", "Bound by debt"],
             "Dispute": ["Contested inheritance", "Shared resource"], "Acquire": ["Under seal"], "Passage": ["Fugitive"]}
hook_premise = roll("hook premise", HOOK_PREM[hook_family])
# 8. ordeal event: the hook's harm (claim challenged) is unfixable, so the event must be disgraced or estranged (assembly §4,
#    repair guarantee); the main Ordeal stays on the heart (main-quest §9)
ordeal_event = roll("ordeal event", ["disgraced", "estranged"])
# 9. climax family: the shape's (Above: Reckoning, Ruin from above) or Schism (main-quest §9)
climax_premise = roll("climax premise", ["Ruin from above", "Schism"])
# 10. the heart's secret (main-quest §7, Plant what the climax asks for): with or without one, even odds
heart_secret = chance("heart has a secret", 0.5)
# 11. hook turn card family: a turn card, so assembly §5 governs it (main-quest §3): Investigate, Threat, Passage, Reckoning
#     (Dispute is the opening's family; Rescue's captive would be the heart's kin, a second harm; Acquire, Restore, Contest barred)
hook_turn_family = roll("hook turn family", ["Investigate", "Threat", "Passage", "Reckoning"])
hook_turn_premise = roll("hook turn premise", ["Betrayal", "Ruin from above", "Blood debt", "Swindle"])
# 12. turn-card families by layer (assembly §4-§5, §7; main-quest §9): threshold from Passage / Dispute / Rescue, two distinct
thr_turns = []
thr_turns.append(roll("threshold turn A family", ["Passage", "Dispute", "Rescue"]))
thr_turns.append(roll("threshold turn B family", [f for f in ["Passage", "Dispute", "Rescue"] if f not in thr_turns]))
# trials-4 (the revelation layer) turn cards must carry the revelation: Investigate or Reckoning (main-quest §3); both, one each
#   (two siblings never share a family). Which chapter gets which is a coin.
t4_first = roll("trials-4 chapter A family", ["Investigate", "Reckoning"])
# trials-3 turn families: any but Contest and Restore (assembly §5), not a threshold family nor a trials-4 family (linked)
T3_OK = [f for f in ["Rescue", "Investigate", "Dispute", "Threat", "Passage", "Acquire", "Reckoning"] if f not in thr_turns and f not in ("Investigate", "Reckoning")]
t3_turns = []
t3_turns.append(roll("trials-3 turn A family", T3_OK))
t3_turns.append(roll("trials-3 turn B family", [f for f in T3_OK if f not in t3_turns]))
# 13. turn-card premises
thrA_premise = roll("threshold A (Rescue) premise", ["Kidnapped", "Captured in battle", "Imprisoned unjustly", "Bound by debt"],
                    [0, 0, 1, 2], note="the move takes men for an unpaid tenth: Bound by debt x2; no lawless band or battle on the card's ground")
thrB_premise = roll("threshold B (Dispute) premise", ["Broken agreement", "Shared resource"],
                    note="Contested inheritance is the hook's, Public offense the Ordeal door's")
t3A_premise = roll("trials-3 A (Acquire) premise", ["Won't sell", "Passed on", "Under seal", "Unknown worth"], [1, 1, 2, 1],
                   note="the lieutenant holds by crown warrant: Under seal x2")
t3B_premise = roll("trials-3 B (Passage) premise", ["Smuggling", "Fugitive"])
t4A_premise = roll("trials-4 A (Investigate) premise", ["Theft", "Disappearance", "Suspicious death", "Sabotage"])
t4B_premise = roll("trials-4 B (Reckoning) premise", ["Betrayal", "Ruin from above", "Blood debt"],
                   note="Swindle is the hook turn's; Ruin from above is also the climax's (allowed, premise <= 3)")
ordeal_doors = roll("which ordeal chapter gets which door", ["A Dispute / B Passage", "A Passage / B Dispute"])
# 14. non-turn card families, weighted by what the chapter's move did (main-quest §7, Non-turn families follow the chapter problem);
#     excluded: the chapter's turn family, an earlier card's family in the chapter, and (openings) the turn families of linked chapters
NT = {}
NT["c2a-open"] = roll("ch2a opening family", ["Passage", "Acquire", "Investigate", "Threat", "Restore", "Dispute", "Contest"],
                      [2, 2, 1, 1, 1, 1, 0.5], note="m2 takes men and levies a tenth: Passage, Acquire lean")
NT["c2b-open"] = roll("ch2b opening family", ["Rescue", "Passage", "Acquire", "Investigate", "Threat", "Restore", "Contest"],
                      [2, 2, 2, 1, 1, 1, 0.5], note="a taking leans Rescue; a levy Acquire")
def fam(key, fams, w, note=""):
    NT[key] = roll(key + " family", fams, w, note); return NT[key]
fam("c3a-open", ["Threat", "Passage", "Investigate", "Restore", "Reckoning", "Contest"], [2, 1, 1, 1, 1, 0.5],
    "m3 is force on the road: Threat x2; Acquire (turn), Rescue and Dispute (linked turns) out")
fam("c3a-mid", [f for f in ["Dispute", "Rescue", "Threat", "Passage", "Investigate", "Restore", "Reckoning", "Contest"] if f != NT["c3a-open"]],
    None, "")
fam("c3b-open", ["Acquire", "Threat", "Investigate", "Restore", "Reckoning", "Contest"], [2, 1, 1, 1, 1, 0.5],
    "carts stopped and goods taken: Acquire x2; Passage (turn), Rescue, Dispute out")
fam("c3b-mid", [f for f in ["Dispute", "Acquire", "Rescue", "Threat", "Investigate", "Restore", "Reckoning", "Contest"] if f != NT["c3b-open"]], None)
fam("c4a-open", ["Dispute", "Threat", "Rescue", "Restore", "Reckoning", "Contest"], [2, 1, 1, 1, 1, 0.5],
    "m4 re-grants tolls and buys keepers: Dispute x2; Investigate (turn), Acquire, Passage (linked turns) out")
fam("c4a-mid", [f for f in ["Acquire", "Dispute", "Threat", "Rescue", "Restore", "Reckoning", "Passage", "Contest"] if f != NT["c4a-open"]], None)
fam("c4b-open", ["Dispute", "Threat", "Rescue", "Restore", "Investigate", "Contest"], [2, 1, 1, 1, 1, 0.5],
    "Reckoning (turn), Acquire, Passage out")
fam("c4b-mid", [f for f in ["Acquire", "Dispute", "Threat", "Rescue", "Restore", "Investigate", "Passage", "Contest"] if f != NT["c4b-open"]], None)
fam("c5a-open", ["Acquire", "Dispute", "Restore", "Rescue", "Threat", "Contest"], [2, 1, 1, 1, 0.5, 1],
    "the writ bars her papers: Acquire x2; Passage (door), Investigate, Reckoning (linked turns) out")
fam("c5a-mid", [f for f in ["Acquire", "Dispute", "Restore", "Rescue", "Threat", "Contest", "Investigate", "Reckoning"] if f != NT["c5a-open"]], None)
fam("c5b-open", ["Acquire", "Passage", "Restore", "Rescue", "Threat", "Contest"], [2, 1, 1, 1, 0.5, 1],
    "Dispute (door), Investigate, Reckoning out")
fam("c5b-mid", [f for f in ["Acquire", "Passage", "Restore", "Rescue", "Threat", "Contest", "Investigate", "Reckoning"] if f != NT["c5b-open"]], None)
fam("c6-open", ["Acquire", "Threat", "Restore", "Investigate", "Contest"], [2, 2, 1, 1, 0.5],
    "m6 seizes a village: Acquire, Threat x2; Rescue (the forced middle), Reckoning (turn), Dispute, Passage (linked turns) out")
fam("ret-open", ["Acquire", "Dispute", "Threat", "Investigate", "Rescue", "Passage", "Contest"], [2, 2, 1, 1, 1, 1, 0.5],
    "the aftermath of a seizure: Acquire, Dispute x2; Restore (turn), Reckoning (linked climax turn) out")
# 15. non-turn premises (requirements filtered by hand; no premise past 2 in the storyline)
PR = {}
def prem(key, opts, w=None, note=""):
    PR[key] = roll(key + " premise", opts, w, note); return PR[key]
prem("c2a-open", ["Won't sell", "Passed on", "Under seal", "Unknown worth"], [1, 1, 2, 0], "crown clerks hold Bogford's goods for the tenth: Under seal x2; nothing of unknown worth there")
prem("c2b-open", ["Kidnapped", "Imprisoned unjustly", "Bound by debt"], [1, 1, 2], "men taken for an unpaid tenth: Bound by debt x2")
prem("c3a-open", ["Broken works", "Estrangement", "Lost livelihood"], [1, 1, 2], "riders on the road drive trade off it: Lost livelihood x2")
prem("c3b-open", ["Broken works", "Estrangement", "Lost livelihood"], [2, 1, 1], "carts dodging the riders load the quiet crossing: Broken works x2")
prem("c3a-mid", ["Betrayal", "Ruin from above", "Blood debt", "Swindle"])
prem("c3b-mid", ["Raiders", "Protected beast", "Sickness or blight", "Encroachment"], [2, 0, 1, 1], "a Binder raid camp lies within reach; no preserve by the march road")
prem("c4a-open", ["Champion's bout", "The fix", "First blood or last", "The ladder"])
prem("c4a-mid", ["Passed on", "Under seal", "Unknown worth"], None, "Won't sell is the trials-3 turn's")
prem("c4b-open", ["Shared resource", "Contested inheritance", "Broken agreement"], None, "Public offense is the Ordeal door's")
prem("c4b-mid", ["Smuggling", "Unwilling passenger"], None, "Fugitive is the trials-3 turn's; Unwelcome cargo the Ordeal door's")
prem("c5a-open", ["Broken works", "Estrangement", "Lost livelihood"], None, "Lost good name left to the Return, which repairs the disgrace")
prem("c5a-mid", ["Kidnapped", "Bound by debt"], None, "Imprisoned unjustly already twice (threshold A turn, threshold B opening): out")
prem("c5b-open", ["Won't sell", "Passed on", "Under seal", "Unknown worth"], [1, 1, 2, 1], "the writ bars her papers: Under seal x2")
prem("c5b-mid", ["Betrayal", "Ruin from above", "Blood debt", "Swindle"])
prem("c6-open", ["Raiders", "Sickness or blight", "Encroachment"], None, "no preserve near Oathford")
prem("ret-open", ["Smuggling", "Fugitive", "Unwilling passenger", "Unwelcome cargo"])
# 16. twists, in the rulebook's roll order (§9): climax, Return, Ordeal, chapter turn cards, then the rest.
#     Storyline budget: max(2, one per 10 storyline cards) per group = 3 (29 main cards plus minors); never two of a group in one
#     chapter or in linked chapters (adjacent layers; every node links to every node of the next layer, W = 2).
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
         "the champion was promised something": "Innocents pay"}
LAYER = {"c1": 1, "c2": 1, "c3": 2, "c4": 2, "c5": 2, "c6": 2, "c7": 3, "c8": 3, "c9": 3, "c10": 3, "c11": 3, "c12": 3,
         "c13": 4, "c14": 4, "c15": 4, "c16": 4, "c17": 4, "c18": 4, "c19": 5, "c20": 5, "c21": 5, "c22": 5, "c23": 5, "c24": 5,
         "c25": 6, "c26": 6, "c27": 6, "c28": 7, "c29": 7}
CHAP = {"c1": "h", "c2": "h", "c3": "2a", "c4": "2a", "c5": "2b", "c6": "2b", "c7": "3a", "c8": "3a", "c9": "3a", "c10": "3b", "c11": "3b",
        "c12": "3b", "c13": "4a", "c14": "4a", "c15": "4a", "c16": "4b", "c17": "4b", "c18": "4b", "c19": "5a", "c20": "5a", "c21": "5a",
        "c22": "5b", "c23": "5b", "c24": "5b", "c25": "6", "c26": "6", "c27": "6", "c28": "r", "c29": "r"}
ELIG = {  # twists each card can carry, filtered by hand against the family tables and what the card can make true
 "c27": ["the wronged don't want it", "changed"],
 "c25": ["one of us caused it", "divided community", "passed along", "protector's price"],
 "c26": ["unwilling captive", "hidden motive"],
 "c29": ["can't go back", "can't go back", "someone wants it broken"],   # weighted to Can't go back (assembly, Return)
 "c28": ["not wanted at the end", "the one left behind"],
 "c21": ["not wanted at the end", "pursuer has a point", "the one left behind"],
 "c24": ["old wound", "law on the wrong side"],
 "c19": ["someone wants it broken", "it broke for a reason", "can't go back", "one person pays"],
 "c20": ["unwilling captive", "hidden motive", "not alone"],
 "c22": ["load-bearing", "wants to be rid of it", "meant for destruction"],
 "c23": ["the wronged don't want it", "changed", "hand not head", "disproportionate demand"],
 "c2": ["the wronged don't want it", "changed", "disproportionate demand"],
 "c4": ["unwilling captive", "hidden motive", "justified captor", "not alone"],
 "c6": ["caught in the middle", "hollow prize", "old wound", "law on the wrong side"],
 "c9": ["load-bearing", "wants to be rid of it", "meant for destruction", "rightful keeper"],
 "c12": ["not wanted at the end", "the one left behind"],
 "c15": ["sympathetic culprit", "wrong target"],
 "c18": ["the wronged don't want it", "changed", "hand not head", "disproportionate demand"],
 "c1": ["caught in the middle", "hollow prize", "old wound", "law on the wrong side"],
 "c3": ["wants to be rid of it", "meant for destruction"],
 "c5": ["unwilling captive", "hidden motive", "justified captor", "not alone"],
 "c7": ["someone wants it broken", "it broke for a reason", "can't go back", "one person pays"],
 "c8": ["the wronged don't want it", "changed", "hand not head", "disproportionate demand"],
 "c10": ["someone wants it broken", "it broke for a reason", "can't go back", "one person pays"],
 "c11": ["one of us caused it", "divided community", "passed along", "protector's price"],
 "c13": ["the fix protects someone", "the champion was promised something"],
 "c14": ["load-bearing", "meant for destruction"],
 "c16": ["caught in the middle", "hollow prize", "old wound", "law on the wrong side"],
 "c17": ["not wanted at the end", "the one left behind"],
}
ORDER = ["c27", "c25", "c26", "c29", "c28", "c21", "c24", "c19", "c20", "c22", "c23",
         "c2", "c4", "c6", "c9", "c12", "c15", "c18",
         "c1", "c3", "c5", "c7", "c8", "c10", "c11", "c13", "c14", "c16", "c17"]
TURNING = set(ORDER[:18])
FORCED = {"c29", "c11"}   # the Return's Restore card (assembly §4) and Raiders (Threat §6) never roll none
TW = {}; GUSE = {}
def linked(a, b): return a == b or abs(LAYER[a] - LAYER[b]) <= 1 and (CHAP[a] == CHAP[b] or LAYER[a] != LAYER[b])
for c in ORDER:
    p = 1.0 if c in FORCED else (0.7 if c in TURNING else 0.35)
    if not chance(f"{c} twist?", p, "forced" if c in FORCED else ("turning card" if c in TURNING else "")):
        TW[c] = None; continue
    ok = [t for t in ELIG[c] if len(GUSE.get(GROUP[t], [])) < 3 and not any(linked(c, o) for o in GUSE.get(GROUP[t], []))]
    if not ok:
        LOG.append({"roll": f"{c} twist", "pick": "none", "note": "every eligible group spent or used in a linked chapter"}); TW[c] = None; continue
    t = roll(f"{c} twist", ok)
    TW[c] = t; GUSE.setdefault(GROUP[t], []).append(c)
# 17. truths (truth rows with shapes, family truth tables). Turning cards with no twist take a non-confirming row
#     (rulebook, No blank card). Shape counts start from the twists that carry one; rows <= 3 and shapes <= 4 in this pass.
TRUTH = {}; SHAPE_N = {"still-profiting": 2, "someone-above": 1, "rigged-to-fall": 1}; ROW_N = {}
TABLE = {
 "Dispute": [("Dispute: record survives", "record-survives"), ("Dispute: no record", "no-record"), ("Dispute: one side lies", "one-side-lies"), ("Dispute: both partly right", "both-partly-right")],
 "Swindle": [("Reckoning: as told", None), ("Reckoning: a witness exists", "unheard-witness"), ("Reckoning: shared blame", "shared-blame"), ("Reckoning: still profiting", "still-profiting")],
 "Acquire": [("Acquire: as told", None), ("Acquire: promised elsewhere", "another-buyer"), ("Acquire: a copy", "a-copy")],
 "Imprisoned": [("Rescue: innocent", None), ("Rescue: guilty", "held-rightly")],
 "Bound": [("Rescue: misassigned", "wrong-name"), ("Rescue: paid", "paid-already"), ("Rescue: inflated", "made-bigger"), ("Rescue: real and fair", "held-rightly")],
 "Kidnapped": [("Rescue: inside help", "sold-from-inside"), ("Rescue: a better buyer", "another-buyer")],
 "Livelihood": [("Restore: the trade road moved", "trade-moved"), ("Restore: buyers went elsewhere", "trade-moved")],
 "Works": [("Restore: worn out and neglected", "neglected"), ("Restore: built badly", "built-to-fail"), ("Restore: the ground moved", "ground-moved"), ("Restore: stones carted off for someone's wall", "stripped-for-parts")],
 "Estrangement": [("Restore: a missed deathbed", "absent-when-needed"), ("Restore: a spouse one of them disapproved of", "disapproved-match"), ("Restore: a side taken in someone else's quarrel", "took-a-side"), ("Restore: a secret that got told", "secret-told")],
 "Raiders": [("Threat: hunger", "ran-out"), ("Threat: driven out", "pushed-here"), ("Threat: paid", "someone-above"), ("Threat: opportunity", None)],
 "Sickness": [("Threat: bad water", "harm-from-upstream"), ("Threat: waste from upstream", "harm-from-upstream"), ("Threat: a sick animal traded in", "brought-in"), ("Threat: carried in by travelers", "brought-in"), ("Threat: spoiled stores", "neglected")],
 "Passage": [("Passage: as declared", None), ("Passage: not what it seems", "more-than-said"), ("Passage: the pursuer was told", "sold-from-inside")],
 "Fix": [("Contest: the money", None), ("Contest: someone above", "someone-above")],
}
PLAN = [  # card, table, may it be confirming?
 ("c27", "Swindle", True), ("c25", "Sickness", False), ("c26", "Bound", False), ("c28", "Passage", True), ("c21", "Passage", True),
 ("c24", "Dispute", True), ("c19", "Estrangement", True), ("c20", "Kidnapped", False), ("c22", "Acquire", False), ("c23", "Swindle", True),
 ("c2", "Swindle", False), ("c4", "Imprisoned", True), ("c6", "Dispute", True), ("c9", "Acquire", False), ("c12", "Passage", False),
 ("c18", "Swindle", True), ("c1", "Dispute", True), ("c3", "Acquire", True), ("c7", "Livelihood", True), ("c8", "Swindle", True),
 ("c10", "Works", True), ("c11", "Raiders", True), ("c13", "Fix", True), ("c14", "Acquire", True), ("c16", "Dispute", True), ("c17", "Passage", True),
]
# fixed by rule: c5 justified captor -> guilty (rolled before the truth); c15 trials culprit -> the story's own; c29 Return cause
TRUTH["c5"] = ("Rescue: guilty", "held-rightly"); TRUTH["c15"] = ("Investigate: the story's own", "the-enemy-did-it")
TRUTH["c29"] = ("Restore: false, spread by someone off the card", "rumour-planted")
for c, k in ((c, v) for c, v in TRUTH.items()):
    ROW_N[k[0]] = ROW_N.get(k[0], 0) + 1
    if k[1]: SHAPE_N[k[1]] = SHAPE_N.get(k[1], 0) + 1
for c, tab, conf_ok in PLAN:
    opts = [(r, sh) for r, sh in TABLE[tab] if (conf_ok or sh) and ROW_N.get(r, 0) < 3 and (sh is None or SHAPE_N.get(sh, 0) < 4)]
    pick = roll(f"{c} truth row", [r for r, _ in opts])
    sh = dict(opts)[pick]
    TRUTH[c] = (pick, sh); ROW_N[pick] = ROW_N.get(pick, 0) + 1
    if sh: SHAPE_N[sh] = SHAPE_N.get(sh, 0) + 1
# 18. Reckoning shields (family-reckoning §5: only those whose requirements the card can meet)
SHIELD = {}
SHIELD["c2"] = roll("c2 shield", ["No law against it", "Fled"], note="Bought needs the arbiter linked to the clerk: no such link")
SHIELD["c8"] = roll("c8 shield", ["No law against it", "Fled"], note="Rank and Protected fail on standing")
SHIELD["c18"] = roll("c18 shield", ["No law against it", "Bought", "Protected"])
SHIELD["c23"] = roll("c23 shield", ["No law against it", "Fled"])
SHIELD["c27"] = "Rank"; LOG.append({"roll": "c27 shield", "pick": "Rank", "note": "only the emperor outranks Litiṭaʻa, and he doesn't hear such matters (fixed by standing)"})
# 19. minor quests: 50% per main node (assembly §10); none on the Return
MINOR = {}
for n in ["h", "2a", "2b", "3a", "3b", "4a", "4b", "5a", "5b", "6"]:
    MINOR[n] = chance(f"minor on chapter {n}", 0.5)
# 20. fallout counts for the realm moves (fallout §3: realm 2-3)
FALL5 = roll("fallout count, ordeal move (realm)", [2, 3])
FALL6 = roll("fallout count, last move (realm)", [2, 3])
# 21. minors (3): families fixed by what the chapter has near it (see log); twists from the groups still open to each
#     (storyline budget 3 per group; never in a linked chapter). 3b has no open group.
M_TW = {}
M_TW["M1"] = roll("minor 3a twist", ["one of us caused it", "none"], note="You started it is the only group open to 3a; Encroachment never rolls none (Threat §6) -> forced") if False else "one of us caused it"
LOG.append({"roll": "minor 3a twist", "pick": "one of us caused it", "note": "Threat Encroachment never rolls none; only You started it is open to 3a"})
M_TR = {}
M_TR["M1"] = roll("minor 3a truth (Encroachment, public)", ["Threat: project flawed", "Threat: project sound"])
M_TR["M2"] = roll("minor 3b truth (Under seal)", ["Acquire: promised elsewhere", "Acquire: a copy"], note="no open twist group: a non-confirming row")
M_SEAL = roll("minor 3b seal type", ["Fine unpaid", "Ownership unproven", "Held as surety for someone else's debt", "Held until a ruling"])
M_TW["M3"] = roll("minor 5a twist", ["law on the wrong side", "old wound", "none"], [1, 1, 1])
M_TR["M3"] = roll("minor 5a truth (Broken agreement)", ["Dispute: no record", "Dispute: one side lies"], note="record survives and both partly right are at 1 each; spread the rows")
# 22. fallout (16 cards; fallout §3): twists, one per group across the story's fallout (rule 11); Raiders never none
F_ELIG = {
 "F6a": ["one of us caused it", "divided community", "passed along", "protector's price"],
 "F1": ["someone wants it broken", "can't go back", "one person pays"],
 "F2a": ["unwilling captive", "hidden motive", "justified captor"],
 "F2b": ["wants to be rid of it", "meant for destruction", "rightful keeper", "load-bearing"],
 "F2late": ["not wanted at the end", "pursuer has a point", "need along the way", "the one left behind"],
 "F3a": ["someone wants it broken", "it broke for a reason", "can't go back", "one person pays"],
 "F3b": ["sympathetic culprit", "the truth costs the requester"],
 "F3late": ["needs it more", "old blood"],
 "F4a": ["the wronged don't want it", "changed", "hand not head"],
 "F4b": ["not wanted at the end", "the one left behind"],
 "F4late": ["needs it more", "the champion was promised something"],
 "F5a": ["caught in the middle", "hollow prize", "old wound", "law on the wrong side"],
 "F5b": ["wants to be rid of it", "meant for destruction", "rightful keeper"],
 "F5late": ["changed", "hand not head"],
 "F6b": ["sympathetic culprit", "the truth costs the requester", "no crime"],
 "F6late": ["caught in the middle", "hollow prize", "old wound", "law on the wrong side"],
}
GROUP.update({"needs it more": "The other side is right", "old blood": "You started it", "no crime": "Not what it seems",
              "the truth costs the requester": "They don't want it", "need along the way": "Innocents pay"})
F_TW = {}; FG = set()
for c in ["F6a", "F1", "F2a", "F2b", "F2late", "F3a", "F3b", "F3late", "F4a", "F4b", "F4late", "F5a", "F5b", "F5late", "F6b", "F6late"]:
    forced = c == "F6a"
    if not chance(f"fallout {c} twist?", 1.0 if forced else 0.4, "Raiders never none" if forced else ""):
        F_TW[c] = None; continue
    ok = [t for t in F_ELIG[c] if GROUP[t] not in FG]
    if not ok: F_TW[c] = None; LOG.append({"roll": f"fallout {c} twist", "pick": "none", "note": "groups spent"}); continue
    t = roll(f"fallout {c} twist", ok); F_TW[c] = t; FG.add(GROUP[t])
# 23. fallout truths: non-confirming rows preferred (the world's plain share is 1 in 4 of side cards); rows <= 3, shapes <= 4
for k, sh in [("Acquire: a copy", "a-copy"), ("Threat: project flawed", "built-to-fail"), ("Dispute: no record", "no-record")]:
    ROW_N[k] = ROW_N.get(k, 0) + 1; SHAPE_N[sh] = SHAPE_N.get(sh, 0) + 1
F_TR = {"F1": ("Restore: buyers went elsewhere", "trade-moved"), "F3b": ("Investigate: a bystander", "bystander-did-it"),
        "F6b": ("Investigate: a bystander", "bystander-did-it")}
for k in F_TR.values():
    ROW_N[k[0]] = ROW_N.get(k[0], 0) + 1; SHAPE_N[k[1]] = SHAPE_N.get(k[1], 0) + 1
TABLE["Smuggling"] = TABLE["Passage"]; TABLE["Fugitive"] = TABLE["Passage"]
TABLE["FirstBlood"] = [("Contest: the stated reason", None), ("Contest: a cover", "hidden-aim"), ("Contest: a way out", "wants-out")]
TABLE["Ladder"] = [("Contest: the prize", None), ("Contest: the sponsor's price", "owes-a-backer"), ("Contest: a way out", "wants-out")]
F_PLAN = [("F2a", "Bound", False), ("F2b", "Acquire", True), ("F2late", "Smuggling", False), ("F3a", "Estrangement", False),
          ("F3late", "FirstBlood", True), ("F4a", "Swindle", False), ("F4b", "Fugitive", False), ("F4late", "Ladder", False),
          ("F5a", "Dispute", False), ("F5b", "Acquire", True), ("F5late", "Swindle", False), ("F6a", "Raiders", True),
          ("F6late", "Dispute", False)]
for c, tab, conf_ok in F_PLAN:
    opts = [(r, sh) for r, sh in TABLE[tab] if (conf_ok or sh) and ROW_N.get(r, 0) < 3 and (sh is None or SHAPE_N.get(sh, 0) < 4)]
    pick = roll(f"fallout {c} truth row", [r for r, _ in opts]); sh = dict(opts)[pick]
    F_TR[c] = (pick, sh); ROW_N[pick] = ROW_N.get(pick, 0) + 1
    if sh: SHAPE_N[sh] = SHAPE_N.get(sh, 0) + 1

if __name__ == "__main__":
    for x in LOG: print(x)
    json.dump(LOG, open("rolls.json", "w"), indent=1, ensure_ascii=False)
