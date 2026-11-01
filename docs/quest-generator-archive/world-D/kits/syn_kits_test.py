"""Throughline test harness for the v6c Old Hands and Night Market story kits.
Encodes the tags in test/world/D/kits/old-hands.md and night-market.md (v6c drafts), enumerates every legal
(enemy, variant, stake, move, blow, crisis) combination under faction-stories §11a (points 2-5
and the needs), and samples 8 per kit with random.Random(92). Paired combinations couple both kits.
"""
import random, itertools, collections, sys

def E(shape, who, want, wantp, means, reach, scheme, var, kind, paired=False, leader=False):
    return dict(shape=shape, who=who, want=want, wantp=wantp, means=set(means.split()), reach=reach,
                scheme=scheme, var=var, kind=kind, paired=paired, leader=leader)

def S(hit, needs, phrase, only=None):
    return dict(hit=hit, needs=set(needs.split()), phrase=phrase, only=only)

def M(needs, hits, phrase, sites=None, notunder=(), plot=False, shapes=None):
    # sites: when the move hits place, the place stakes it can hit; plot: not under 'the cost' (§11a pt 5)
    return dict(needs=set(needs.split()), hits=set(hits.split()), phrase=phrase, sites=sites,
                notunder=set(notunder), plot=plot, shapes=shapes)

def C(needs, hits, fam, prem, phrase, sites=None, mode="unpaired", shapes=None, notunder=(), also=None):
    # sites: {hit tag: stakes it can decide} where a hit is narrower than its tag
    # needs: list of alternatives, each a set that must all be met ('Above' counts as a means for authority)
    return dict(needs=[set(a.split()) for a in needs], hits=set(hits.split()), fam=fam, prem=prem, phrase=phrase,
                sites=sites, mode=mode, shapes=shapes, notunder=set(notunder), also=also)

REACH = {"local": 0, "region": 1, "realm": 2}

# ---------------------------------------------------------------- the Old Hands
OH = dict(
 name="Old Hands", leader_strike="the old don", leader_enemy="OH1",
 enemies={
  "OH1": E("Within", "the old don", "revenge", "revenge on the houses that stopped answering his letters",
           "name paper inside", "region", "mean to burn the old houses with every favour they have recorded",
           {"a": "in letters to the houses' rivals", "b": "in letters to the houses' own heirs",
            "c": "on a page nailed to each house's gate at the new moon"}, "the old debt", leader=True),
  "OH2": E("Beside", "a Night Market fixer", "ambition", "the old houses' secrets for the Market",
           "coin inside", "region", "are buying the Old Hands' stewards one by one",
           {"a": "with coin", "b": "with passages out on Far Sails boats", "c": "with places in the Market for their sons"},
           "hidden hand", paired=True),
  "OH3": E("Above", "a Seated House lord", "cover-up", "his house's old use of the Old Hands buried",
           "men paper name", "region", "want every record of the Old Hands' work for his house gone, and the Old Hands with it",
           {"a": "burning a steward's hall with its books", "b": "having a magistrate call the records in",
            "c": "setting the watch on the back rooms"}, "the old debt"),
  "OH4": E("Within", "a smuggling-cache keeper", "desperation", "the syndicate fed now the houses' work is drying up",
           "inside men coin", "region", "have made the quiet crossings a branch of the Walking Road, for Qotiro captives sold to the old houses",
           {"a": "carried as a house's servants", "b": "carried inside funeral corteges", "c": "carried by the river crossing at night"},
           "the cost"),
  "OH5": E("Above", "a True Line schemer", "ambition", "her house's claim to the oldest line",
           "coin faith kin", "realm", "need the forgers her house's line was forged by silenced before the Seated Houses ask for proof",
           {"a": "buying them into silence", "b": "selling them into bond with the Binders",
            "c": "naming them grove-goers to an inquisitor"}, "the old debt"),
  "OH6": E("Beside", "a settled Qotiro widow married into a river house", "revenge",
           "revenge for the land a forged will took from her father", "kin name", "local",
           "are turning the old houses' heirs against the Old Hands with the true will",
           {"a": "shown to each heir before a betrothal", "b": "carried into three houses by her married daughters",
            "c": "read out at the betrothal feasts of the houses that profited"}, "the old debt"),
  "OH7": E("Within", "a back-room master", "preservation", "the houses to need the Old Hands again",
           "inside name craft", "local", "forge the scandals the houses then pay the Old Hands to bury",
           {"a": "love letters in a dead wife's hand", "b": "debts in a dead lord's hand", "c": "an heir who was never born"},
           "hidden hand"),
  "OH8": E("Beside", "a Sharp Hands moneylender", "greed", "the old houses' debts, and their custom with them",
           "coin craft", "region", "are buying the old houses' debts and forgiving them for the work the Old Hands used to do",
           {"a": "his clerks forging their papers", "b": "his carters taking their loads across", "c": "his clerks writing their letters of favour"},
           "hidden hand"),
  "OH9": E("Beside", "a Night Market crew boss", "revenge", "revenge for a brother the Old Hands gave up at a crossing",
           "men craft", "region", "raid the quiet crossings and caches and leave Old Hands seals behind for the watch",
           {"a": "the brother hanged", "b": "the brother sold to the Binders", "c": "the brother drowned at the ford"},
           "the old debt", paired=True),
 },
 stakes={
  "S1": S("place", "men paper kin inside", "the Old Gate House"),
  "S2": S("thing", "inside name", "the old don's favour book", only={"OH1"}),
  "S3": S("name", "name kin coin", "the Old Hands' name with the houses"),
  "S4": S("place", "paper kin coin men", "a safehouse estate"),
  "S5": S("place", "men inside coin paper", "the quiet crossings"),
  "S6": S("trade", "coin name kin craft", "the oldest house's custom"),
 },
 moves={
  "M1": M("coin", "trade people", "buy a house's steward away from the Old Hands, and he says so at his house's table", plot=True, shapes={"Above", "Beside"}),
  "M2": M("men paper", "place", "shut a quiet crossing", sites={"S5"}, notunder={"OH1"}),
  "M3": M("name kin coin", "name trade", "have a house send its work elsewhere and say why before witnesses", plot=True, shapes={"Above", "Beside"}),
  "M4": M("paper", "thing name", "have a lord's court call in a lineage roll the Old Hands made", plot=True, shapes={"Above", "Beside"}),
  "M5": M("coin craft", "trade", "open a back room across the lane from one of the Old Hands', selling the houses the same work", plot=True, shapes={"Above", "Beside"}),
  "M6": M("name paper", "name thing", "have an old favour the Old Hands did a house named before the house's guests", plot=True, notunder={"OH7"}),
  "M7": M("inside", "people trade", "have a steward put out of a back room before the others, on their word"),
  "M8": M("men kin paper inside", "place", "have {place} barred to the Old Hands for a night", sites={"S1", "S4"}, plot=True, shapes={"Above", "Beside"}),
  "M9": M("name", "name", "have a scandal the Old Hands were paid to bury cried at a house's feast", plot=True),
 },
 blows=[
  ("heart", "captive", "men coin paper", "people", "hold the heart"),
  ("heart", "disgraced", "name inside", "people name", "have the heart shamed as one who went to the law"),
  ("heart", "estranged", "kin name", "people", "turn the heart's kin against her"),
  ("player", "framed", "inside name", "people name", "frame you for shaming a house in public"),
  ("player", "stripped of rank", "inside name", "people", "have you put down a rank on false talk"),
  ("player", "arrested", "paper men", "people", "have you arrested"),
  ("guild", "hall seized", "men paper", "place", "have a back room shut"),
  ("guild", "members taken", "men paper coin", "people", "have stewards and forgers taken"),
  ("guild", "outlawed", "paper", "trade place", "get a lord's writ against the back rooms in one lordship"),
  ("guild", "leader struck down", "men inside", "people", "have the old don struck down"),
 ],
 crises={
  "C1": C(["name", "inside"], "name trade people", "Reckoning", "Betrayal",
          "letters to three houses' rivals tell what the Old Hands carried for whom, naming the heart as the one who carried them",
          also="link", sites={"trade": {"S6"}}),
  "C2": C([], "people name trade", "Reckoning", "Blood debt",
          "the truce meeting ends {truce}, one dead on each side, and the Market's leader shields the killer", mode="paired",
          sites={"trade": {"S6"}}),
  "C3": C(["name", "kin", "coin"], "name trade people", "Reckoning", "Ruin from above",
          "the house disowns the Old Hands in public and hands the heart to the watch as their go-between",
          shapes={"Above", "Beside"}, sites={"trade": {"S6"}}),
  "C4": C(["paper"], "thing name people", "Acquire", "Won't sell",
          "his letters are sealed at the Old Gate House, to go out at the new moon", shapes={"Within"}, also="OH1"),
  "C5": C(["men"], "place people", "Passage", "Fugitive",
          "the enemy's men wait at the gate that has never closed for the heart's kin, who saw the scheme at work, and the Old Hands must get them out",
          sites={"place": {"S1"}}, mode="both", shapes={"Above", "Beside"}),
  "C6": C(["kin"], "place people", "Dispute", "Contested inheritance",
          "the head of the house that lends the Old Hands {place} dies, and the enemy's claim, which would shut it to them, meets the heart's in the Old Gate House's inner court",
          sites={"place": {"S1", "S4"}}),
  "C7": C(["coin"], "place people", "Rescue", "Bound by debt",
          "the enemy holds the crossing-keepers' debts, and the heart's kin is held for one", sites={"place": {"S5"}}),
  "C8": C(["inside"], "people name", "Reckoning", "Schism",
          "the enemy has the heart named before the back rooms as the one who did their work, and the old don calls the Old Hands to judge by their own law",
          shapes={"Within"}, notunder={"OH1"}),
 },
 c5_paired="the truce meeting ends {truce}, one dead on each side; the Market blames the heart's kin, and they must be got out of the Old Gate House before the reprisal",
 endings={
  "make the houses need us again": {"name", "trade", "place", "thing"},
  "open the gate": {"name", "place", "trade"},
  "give back the pages": {"thing", "name"},
  "sell to the lords": {"trade", "place", "thing"},
  "make peace with the Market": {"PAIRED"},
 },
)

# ---------------------------------------------------------------- the Night Market
NM = dict(
 name="Night Market", leader_strike="the leader", leader_enemy="N5",
 enemies={
  "N1": E("Beside", "an Old Hands steward", "ambition", "the Market broken and its buyers back with the old houses",
          "craft name men", "region", "have set the score everyone is whispering about as a trap to break the Market",
          {"a": "the house's own guards waiting inside", "b": "the thing a fake and the buyer Old Hands",
           "c": "the house empty, and the crews to be named for a theft done elsewhere that night"}, "hidden hand", paired=True),
  "N2": E("Within", "the Market's paymaster", "preservation", "the Market's people bought back from the Binders",
          "coin inside", "local", "have been skimming every crew's share for a year to buy back Market people sold for debt",
          {"a": "choosing who earns most", "b": "choosing whom he loves", "c": "choosing who knows his secrets"}, "the cost"),
  "N3": E("Above", "a Sharp Hands broker", "ambition", "the Open Council's credit sunk and the council taken",
          "coin paper name", "realm", "are buying up the Market's forged notes to cry them down at the right hour",
          {"a": "in one market town's Council hall", "b": "at a port's exchange", "c": "through a toll town's counting-house"},
          "hidden hand"),
  "N4": E("Above", "a toll lord", "ambition", "the Market in his service",
          "paper men", "region", "let the Market run their trails for years so they could blackmail it into service",
          {"a": "holding its toll records", "b": "holding a hostage kept as a guest", "c": "holding sealed charges at the magistrate's"},
          "the old debt"),
  "N5": E("Within", "the leader", "desperation", "a way out from under the Market's founding debt",
          "inside coin craft", "region", "are sending the crews after the biggest score to pay a founding debt now called in, though it will get them killed",
          {"a": "owed to a Far Sails house", "b": "owed to the Binders", "c": "owed to a lord who paid the first market's fine"},
          "the old debt", leader=True),
  "N6": E("Beside", "a Lantern bounty captain", "greed", "the bounties on Market heads",
          "coin men paper", "region", "pay a fence for the names of crews on the move and collect the lords' bounty on each",
          {"a": "the fence a card-game keeper", "b": "the fence the eldest fence's own nephew", "c": "the names sent by pigeon from a cove"},
          "hidden hand"),
  "N7": E("Beside", "a Road Crews carter, eldest of the Market's carters", "revenge",
          "revenge for a son lost to a load the Market hid in his cart", "name craft inside", "region",
          "are showing the toll men the trails one at a time and turning the carters away",
          {"a": "the son hanged at a toll gate", "b": "the son branded and sold for the fine", "c": "the son drowned fleeing the toll men"},
          "the old debt"),
  "N8": E("Beside", "an Old Hands forger", "greed", "the price of the Market's notes for himself",
          "craft inside", "region", "forge the Market's notes better than the Market does, through a fence they own, so the bad ones are traced to the Market",
          {"a": "the fence a pawn-counter man", "b": "the fence a card-game keeper", "c": "the fence a widow who keeps the books"},
          "hidden hand", paired=True),
  "N9": E("Above", "a Tide House elder", "cover-up", "his house's runaways never traced to him",
          "kin men", "region", "are closing the coves that carried his house's young to the Far Sails and silencing the crews who rowed them",
          {"a": "through his kin in the toll houses", "b": "with his house's guards", "c": "by marrying a daughter to the coves' toll lord"},
          "the old debt"),
 },
 stakes={
  "S1": S("place", "men paper inside", "Under the Market Road"),
  "S2": S("name", "coin name paper", "\"no one owns the Night Market\""),
  "S3": S("thing", "inside coin", "the fences' list"),
  "S4": S("place", "men kin paper", "a smugglers' cove"),
  "S5": S("trade", "men paper inside name", "the toll-dodge trails"),
 },
 moves={
  "M1": M("coin craft", "trade", "open another market the same night under another road, with the Market's own buyers", plot=True, shapes={"Above", "Beside"}, notunder={"N6"}),
  "M2": M("men paper inside", "trade", "shut a toll-dodge trail", shapes={"Above", "Beside"}),
  "M3": M("coin", "name people", "buy a fence at a card table, in front of the crews", plot=True, shapes={"Above", "Beside"}),
  "M4": M("men kin inside", "place", "show the toll men {place}", sites={"S1", "S4"}, plot=True, shapes={"Above", "Beside"}),
  "M5": M("coin paper name", "trade name", "have an exchange refuse the Market's notes", plot=True, shapes={"Above", "Beside"}),
  "M6": M("inside paper", "thing name", "have a page of the fences' list read out in a dockside tavern", plot=True, shapes={"Above", "Beside"}),
  "M7": M("name craft", "name people", "call the Market out before the crews at a dockside card game", plot=True, shapes={"Above", "Beside"}),
  "M8": M("coin", "place", "buy {place} out from under the Market", sites={"S1", "S4"}, plot=True, shapes={"Above", "Beside"}),
  "M9": M("inside", "people name", "have a crew paid short in front of the others", shapes={"Within"}),
  "M10": M("inside craft", "people thing", "name the big score openly at the market and send crews after it", plot=True, notunder={"N7", "N8"}),
 },
 blows=[
  ("heart", "captive", "men coin paper", "people", "hold the heart"),
  ("heart", "disgraced", "name inside", "people name", "have the heart shamed as an informer"),
  ("heart", "estranged", "kin name", "people", "turn the heart's kin against her"),
  ("player", "framed", "inside name", "people name", "frame you for holding back the take"),
  ("player", "stripped of rank", "inside name", "people", "have you put down a rank"),
  ("player", "arrested", "paper men", "people", "have you arrested"),
  ("guild", "hall seized", "men paper", "place", "have a fence's shop shut"),
  ("guild", "members taken", "men paper coin", "people", "have a crew taken"),
  ("guild", "outlawed", "paper", "trade place", "get a toll lord's writ against the trails in his lordship"),
  ("guild", "leader struck down", "men inside", "people", "have the leader struck down"),
 ],
 crises={
  "C1": C([], "people name trade", "Reckoning", "Blood debt",
          "the truce meeting ends {truce}, one dead on each side, and the old don shields the killer", mode="paired",
          sites={"trade": {"S5"}}),
  "C2": C(["men", "paper", "inside"], "place people", "Passage", "The market moves",
          "a raid comes for Under the Market Road tonight with the enemy riding along, and the whole market must move by dawn",
          sites={"place": {"S1"}}, mode="both", shapes={"Above", "Beside"}),
  "C3": C(["coin inside"], "people name", "Reckoning", "Betrayal",
          "the crews learn their shares have been short for a year, and the heart's share paid for someone else's way out",
          shapes={"Within"}),
  "C4": C(["coin", "paper", "name"], "name people", "Reckoning", "Ruin from above",
          "the Market's notes are cried down and the heart is named as the forger", shapes={"Above"}),
  "C5": C(["coin"], "thing people", "Acquire", "The score",
          "the enemy, holding the crews' spent advance on the big score and the fences' list as surety, wants the score by the new moon",
          shapes={"Above", "Beside"}),
  "C6": C(["craft"], "trade people", "Contest", "Champion's bout",
          "the enemy swears a race for the trails under the Market's oldest rule, and the heart's crew must win it",
          shapes={"Above", "Beside"}),
  "C7": C(["men paper", "men Above"], "place trade people", "Rescue", "Imprisoned unjustly",
          "the enemy's men take the crews at {place}, the heart's kin among them, to be judged at dawn", sites={"place": {"S1", "S4"}, "trade": {"S5"}}),
  "C8": C(["inside"], "people name", "Reckoning", "Schism",
          "the enemy lays the short shares on the heart before the crews, and the leader calls the Market to judge by its own rule",
          shapes={"Within"}, notunder={"N5"}),
 },
 c2_paired="the truce meeting ends {truce}, one dead on each side, and the reprisal comes as a raid on Under the Market Road tonight, the enemy riding with the watch",
 endings={
  "pull the big one": {"thing", "trade"},
  "sell to the Sharp Hands": {"trade", "name", "thing"},
  "go straight": {"name", "place", "thing"},
  "put it on the water": {"place", "trade"},
  "share it all out": {"name", "place", "trade"},
  "make peace with the Old Hands": {"PAIRED"},
 },
)

TRUCE = {"a": ("with a knife at the truce table", {"men", "craft"}),
         "b": ("with a fall from the crossing's bridge", {"men", "inside"}),
         "c": ("with a poisoned cup", {"inside", "craft"})}
FEUD = {"F1": ("a contested tavern both syndicates use", {"OH2", "OH9", "N1", "N8"}),
        "F2": ("a quiet crossing the Market's crews began using without leave", {"OH9"}),
        "F3": ("an old house that began buying from the Market", {"OH2", "N1"}),
        "F4": ("a forger who changed sides", {"OH2", "N8"})}
CALLER = {"T1": "the old don, by letter", "T2": "the leader, over a card table",
          "T3": "the lord of the town where the tavern stands", "T4": "the carters' foreman whose crews haul for both"}

def meets(needs, means): return bool(needs & means)

def stake_ok(k, eid, sid):
    e, s = k["enemies"][eid], k["stakes"][sid]
    if s["only"] and eid not in s["only"]: return False
    if eid == "OH1" and sid not in ("S2", "S3"): return False
    if (eid, sid) in {("OH5", "S5"), ("N9", "S5")}: return False  # their schemes are the forgers / the coves
    return meets(s["needs"], e["means"])

def hits_stake(part_hits, sites, sid, k):
    hit = k["stakes"][sid]["hit"]
    if hit not in part_hits: return False
    if isinstance(sites, set): sites = {"place": sites}
    if sites and hit in sites and sid not in sites[hit]: return False
    return True

def move_ok(k, eid, sid, mid):
    e, m = k["enemies"][eid], k["moves"][mid]
    if eid in m["notunder"] or (m["plot"] and e["kind"] == "the cost"): return False
    if m["shapes"] and e["shape"] not in m["shapes"]: return False
    if not meets(m["needs"], e["means"]): return False
    return hits_stake(m["hits"], m["sites"], sid, k) or "people" in m["hits"]

def blow_ok(k, eid, sid, b):
    e = k["enemies"][eid]; tgt, ev, needs, hits, _ = b
    needs = set(needs.split()); hits = set(hits.split())
    if ev == "outlawed":
        if not ("paper" in e["means"] and REACH[e["reach"]] >= 1): return False
    elif not meets(needs, e["means"]): return False
    if ev == "leader struck down" and e["leader"]: return False
    return (k["stakes"][sid]["hit"] in hits) or ("people" in hits)

def crisis_ok(k, eid, sid, cid, paired):
    e, c = k["enemies"][eid], k["crises"][cid]
    if paired and c["mode"] == "unpaired": return False
    if not paired and c["mode"] == "paired": return False
    if eid in c["notunder"]: return False
    if c["shapes"] and e["shape"] not in c["shapes"]: return False
    if c["also"] == "OH1" and eid != "OH1": return False
    if c["also"] == "link" and not (e["shape"] == "Within" or "kin" in e["means"]): return False
    if not paired or c["mode"] == "unpaired":
        means = e["means"] | ({"Above"} if e["shape"] == "Above" else set())
        if c["needs"] and not any(a <= means for a in c["needs"]): return False
    return hits_stake(c["hits"], c["sites"], sid, k)

def endings_for(k, sid, cid, paired):
    hit, fam = k["stakes"][sid]["hit"], k["crises"][cid]["fam"]
    out = [n for n, f in k["endings"].items() if (hit in f or fam in f) and ("PAIRED" not in f or paired)]
    out += [n for n, f in k["endings"].items() if "PAIRED" in f and paired and n not in out]
    return out

def story_options(k, eid, paired, forced=None):
    out = []
    for sid in k["stakes"]:
        if not stake_ok(k, eid, sid): continue
        for cid in (forced or k["crises"]):
            if not crisis_ok(k, eid, sid, cid, paired): continue
            if len(endings_for(k, sid, cid, paired)) < 3: continue
            for mid in k["moves"]:
                if not move_ok(k, eid, sid, mid): continue
                for b in k["blows"]:
                    if blow_ok(k, eid, sid, b): out.append((sid, cid, mid, b))
    return out

def unpaired_combos(k):
    res = []
    for eid, e in k["enemies"].items():
        if e["paired"]: continue
        for o in story_options(k, eid, False): res.append((eid,) + o)
    return res

def paired_combos():
    res = []
    for oe, ne in itertools.product([x for x in OH["enemies"] if OH["enemies"][x]["paired"]],
                                    [x for x in NM["enemies"] if NM["enemies"][x]["paired"]]):
        om, nm = OH["enemies"][oe]["means"], NM["enemies"][ne]["means"]
        for blood in ("OH", "NM"):
            ofc = ["C2"] if blood == "OH" else ["C5"]
            nfc = ["C1"] if blood == "NM" else ["C2"]
            oo, no = story_options(OH, oe, True, ofc), story_options(NM, ne, True, nfc)
            if not oo or not no: continue
            for tv, (_, tn) in TRUCE.items():
                if not (meets(tn, om) and meets(tn, nm)): continue
                for fid, (_, fn) in FEUD.items():
                    for first in ("OH-side", "NM-side"):  # OH-side: the Old Hands' man (NM story's enemy) struck first
                        striker = ne if first == "OH-side" else oe
                        if striker not in fn: continue
                        res.append(dict(oe=oe, ne=ne, blood=blood, truce=tv, feud=fid, first=first, oo=oo, no=no))
    return res

def uses(rng, needs, means):
    pool = sorted(needs & means) if needs else sorted(means)
    return rng.choice(pool) if pool else "-"

def sentence(k, eid, var, sid, cid, mid, b, paired=None, rng=None):
    e = k["enemies"][eid]; s = k["stakes"][sid]; m = k["moves"][mid]; c = k["crises"][cid]
    place = s["phrase"]
    scheme = f'{e["scheme"]}, {e["var"][var]}'
    if paired:
        me_first = (paired["first"] == "NM-side") == (k is OH)
        cause = FEUD[paired["feud"]][0]
        scheme += ("; they struck the feud's first blow, over " if me_first else "; they took up the feud the other side began, over ") + cause
    mv = m["phrase"].format(place=place)
    blow = b[4]
    if paired:
        other = "the Old Hands'" if k is OH else "the Market's"
        blow = f'{blow}, in answer to {other} own move against them'
    if paired:
        tv = TRUCE[paired["truce"]][0]
        if k is OH and cid == "C5": cr = OH["c5_paired"].format(truce=tv)
        elif k is NM and cid == "C2": cr = NM["c2_paired"].format(truce=tv)
        else: cr = c["phrase"].format(truce=tv, place=place)
        cr += f' (the truce called by {CALLER[paired["caller"]]})'
    else:
        cr = c["phrase"].format(place=place)
    return f'Because {e["who"]} wants {e["wantp"]}, they {scheme}; they {mv}, then {blow}, and it comes to a head when {cr}.'

def report(k, label):
    up = unpaired_combos(k)
    print(f"== {label}: {len(up)} legal unpaired spines (enemy, stake, crisis, move, blow)")
    by_e = collections.Counter(x[0] for x in up)
    for eid in k["enemies"]:
        if k["enemies"][eid]["paired"]: continue
        crs = sorted({x[2] for x in up if x[0] == eid}); sts = sorted({x[1] for x in up if x[0] == eid})
        print(f"   {eid}: {by_e[eid]:5d}  stakes {sts}  crises {crs}")
    # crisis share under the roller's order: enemy uniform, then stake, then crisis uniform among legal
    share = collections.Counter(); n = 0
    for eid in [x for x in k["enemies"] if not k["enemies"][x]["paired"]]:
        sts = sorted({x[1] for x in up if x[0] == eid})
        for sid in sts:
            crs = sorted({x[2] for x in up if x[0] == eid and x[1] == sid})
            for cid in crs: share[cid] += 1 / len(sts) / len(crs)
        n += 1
    print("   crisis share (enemy, then stake, then crisis uniform):", {c: f"{v / n:.0%}" for c, v in sorted(share.items())})
    mv = collections.Counter(x[3] for x in up)
    print("   moves used:", dict(sorted(mv.items())))
    return up

def sample(k, label, up, pc, seed=92):
    rng = random.Random(seed)
    print(f"\n== 8 samples for {label}, random.Random({seed})")
    out = []
    for i in range(8):
        paired = rng.random() < 1 / 3
        # roller order: pairing, enemy, stake, crisis, move, blow; each uniform among what is still legal
        def pick(opts, ix):
            v = rng.choice(sorted({o[ix] for o in opts}, key=str)); return [o for o in opts if o[ix] == v], v
        if paired:
            p = rng.choice(pc); p = dict(p); p["caller"] = rng.choice(sorted(CALLER))
            eid = p["oe"] if k is OH else p["ne"]; opts = [(eid,) + o for o in (p["oo"] if k is OH else p["no"])]
        else:
            p = None; opts, eid = pick(up, 0)
        opts, sid = pick(opts, 1); opts, cid = pick(opts, 2); opts, mid = pick(opts, 3)
        b = rng.choice(sorted({o[4] for o in opts}))
        var = rng.choice(sorted(k["enemies"][eid]["var"]))
        e = k["enemies"][eid]
        u = (uses(rng, k["moves"][mid]["needs"], e["means"]), uses(rng, set(b[2].split()), e["means"]),
             uses(rng, set().union(*k["crises"][cid]["needs"]) if k["crises"][cid]["needs"] else set(), e["means"]))
        ends = endings_for(k, sid, cid, bool(p)); rng.shuffle(ends)
        tag = f"{'PAIRED ' + p['oe'] + '+' + p['ne'] + ' blood=' + p['blood'] + ' truce ' + p['truce'] + ' ' + p['feud'] + ' ' + p['first'] + ' ' + p['caller'] if p else 'unpaired'}"
        line = sentence(k, eid, var, sid, cid, mid, b, p, rng)
        rec = f"{i+1}. [{tag}] {eid}{var} · {sid} · {mid} · {b[0]} {b[1]} · {cid} · uses {u} · endings {ends[:3]}"
        print(rec); print("   " + line)
        out.append((rec, line))
    return out

if __name__ == "__main__":
    pc = paired_combos()
    print(f"== paired couplings: {len(pc)}; enemy pairs {sorted({(p['oe'], p['ne'], p['blood']) for p in pc})}")
    print("   truce variants:", collections.Counter(p["truce"] for p in pc), " feud:", collections.Counter(p["feud"] for p in pc))
    uo = report(OH, "Old Hands"); un = report(NM, "Night Market")
    sample(OH, "Old Hands", uo, pc); sample(NM, "Night Market", un, pc)
