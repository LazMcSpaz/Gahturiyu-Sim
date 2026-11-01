"""v6c kit model for Dueling and Holy House: legal combinations and the throughline test.

needs: list of alternatives; each alternative is a set of means that must all be present.
"""
import itertools, random, sys

def N(s):
    return [frozenset(a.split("+")) for a in s.split("|")] if s else [frozenset()]

def ok(needs, means):
    return any(a <= means for a in needs)

# ---------------- DUELING ----------------
D = dict(
 enemies={
  "D1": dict(who="an old-school sword-master", p="he", want="preservation", gloss="the old heads proved right",
             means={"craft","men","inside"}, reach="local", shape="Beside", kind="hidden hand", side="none",
             stakes=["S3"], scheme="has his students sabotage the Society's bouts so that sport fighters bleed on real steel"),
  "D2": dict(who="the council-house head who holds the Ring's lease", p="he", want="ambition", gloss="his son at the top of the ladder",
             means={"paper","men","coin"}, reach="local", shape="Above", kind="the cost", side="Open Council",
             stakes=["S1","S2","S4"], scheme="has every fighter who beats his son fined, banned or pressed into his guard"),
  "D3": dict(who="the showman", p="he", want="preservation", gloss="the Ring kept open",
             means={"inside","coin","men"}, reach="region", shape="Within", kind="hidden hand", side="Dueling Society",
             stakes=["S1","S3"], scheme="sells the Ring's best fighters into lords' private blood matches, billed as exhibitions, to pay the Ring's debts",
             blows={("heart","captive"),("heart","disgraced"),("player","framed"),("player","stripped"),("guild","members taken")}),
  "D4": dict(who="a Night Market bookmaker", p="she", want="greed", gloss="a sure book",
             means={"coin","inside"}, reach="local", shape="Beside", kind="hidden hand", side="Night Market",
             stakes=["S3","S4"], scheme="runs the ladder as a fixed book through a ladder-keeper in her debt"),
  "D5": dict(who="the kin of the man the sister killed", p="he", want="revenge", gloss="the sister broken",
             means={"kin","craft","name"}, reach="local", shape="Beside", kind="the old debt", side="none",
             stakes=["S3"], scheme="climbs the open ladder unsworn, under another name, to be drawn against her"),
  "D6": dict(who="the Ring's senior ring-keeper", p="he", want="ambition", gloss="the Society for himself",
             means={"inside","craft","name"}, reach="region", shape="Within", kind="hidden hand", side="Dueling Society",
             stakes=["S1","S3","S4"], scheme="teaches the ladder's best the deadly half in secret, so that when 'nobody dies' fails the fighters will follow him"),
  "D7": dict(who="a Banner Lord's master-at-arms", p="he", want="preservation", gloss="his lord's march held",
             means={"men","paper"}, reach="region", shape="Above", kind="the cost", side="Banner Lords",
             stakes=["S4"], scheme="keeps giving the tournament's prize, a post in his lord's guard, that has sent the ladder's last champions to die in his lord's feud",
             blows={("heart","captive"),("player","arrested"),("guild","members taken")}),
  "D8": dict(who="the headman of the village that lost the Society's first feud-bout", p="he", want="desperation", gloss="the mill his village lost",
             means={"name","men"}, reach="local", shape="Beside", kind="the old debt", side="none",
             stakes=["S1","S3"], scheme="means to prove that bout was never fair, because the Society lent the other side its deadliest fighter, and take the mill back"),
  "D9": dict(who="a parish priest of the Holy House", p="he", want="faith", gloss="the feast back at his shrine",
             means={"faith","name"}, reach="region", shape="Beside", kind="the cost", side="Holy House",
             stakes=["S2"], scheme="builds a heresy complaint, wound by wound, to have the rings banned, because the feast-day bouts emptied his shrine and closed its soup line"),
 },
 stakes={
  "S1": dict(name="the Ring", hit="place", needs=N("men|paper|inside|faith")),
  "S2": dict(name="the grand tournament's licence", hit="trade", needs=N("paper|coin|faith")),
  "S3": dict(name="the Society's name", hit="name", needs=N("name|craft|inside")),
  "S4": dict(name="the ladder", hit="thing", needs=N("inside|coin|craft|paper")),
 },
 moves={
  "M1": dict(needs=N("craft+men|craft+inside"), hits={"trade","name"}, shapes={"Beside","Within"},
             t="puts real-steel bouts on at the next circuit stop and draws off the Society's crowd"),
  "M2": dict(needs=N("craft"), hits={"name"}, shapes={"Beside","Within","Above"},
             t="calls the Society's way cowardice at a challenge stone and beats the fighter who answers"),
  "M3": dict(needs=N("coin|paper"), hits={"thing"}, shapes={"Above","Within"},
             t="takes the ladder's top fighter off it in front of the crowd, into a paid post"),
  "M4": dict(needs=N("paper|faith"), hits={"place","trade","thing"}, shapes={"Above","Beside"},
             t="gets {stake} suspended pending an inquiry"),
  "M5": dict(needs=N("name|faith"), hits={"name","trade"}, shapes={"Beside"},
             t="cries the Ring down in the taverns as a killers' yard"),
  "M6": dict(needs=N("men"), hits={"place","trade"}, shapes={"Above","Beside"},
             t="posts men at the Ring's gate on bout nights to turn the crowd away"),
  "M7": dict(needs=N("inside"), hits={"name","place"}, shapes={"Within"},
             t="overrules the yield rule in public and puts bouts on the card at first blood"),
  "M8": dict(needs=N("coin"), hits={"thing","name"}, shapes={"Beside"},
             t="opens a book on the ladder in every tavern, at odds that tell the town who will win"),
 },
 blows={
  ("heart","captive"): dict(needs=N("men|coin|paper"), t="has the heart held"),
  ("heart","disgraced"): dict(needs=N("name|inside"), t="has the heart accused of throwing a bout"),
  ("heart","estranged"): dict(needs=N("kin|name"), t="turns the heart's kin against her"),
  ("player","framed"): dict(needs=N("inside|name"), t="frames you for throwing a bout, or for a beating outside the ring"),
  ("player","stripped"): dict(needs=N("inside"), t="has you put down the ladder"),
  ("player","arrested"): dict(needs=N("paper|faith|men"), menshape={"Above"}, t="has you arrested"),
  ("guild","hall seized"): dict(needs=N("men|paper|faith"), t="has a tavern-yard ring shut"),
  ("guild","members taken"): dict(needs=N("men|paper|coin"), menshape={"Above"}, band=True, t="has a group of fighters pressed, bonded or held"),
  ("guild","outlawed"): dict(needs=N("paper|faith"), region=True, shapes={"Above","Beside"}, t="gets rings banned in a lordship"),
  ("guild","leader struck down"): dict(needs=N("men|inside"), t="strikes down {leader}"),
 },
 crises={
  "C1": dict(needs=N("craft+inside|craft+men"), hits={"name"}, shapes={"Beside","Within"}, fam="Contest: First blood or last",
             t="a fighter dies after a bout {p} had a hand in, and {p}r champion lays a blood challenge over the Ring's honour"),
  "C2": dict(needs=N("coin"), hits={"trade"}, shapes={"Above","Beside"}, fam="Reckoning: Ruin from above (Bought)",
             t="the licence is revoked after a riot {p} paid for, and the heart is fined and barred"),
  "C3": dict(needs=N("inside+coin+men"), hits={"name"}, shapes={"Within"}, fam="Reckoning: Betrayal",
             t="the heart is sold into a lord's private match and badly hurt"),
  "C4": dict(needs=N("kin+craft"), hits={"name"}, shapes={"Beside"}, fam="Contest: First blood or last (the sister fights)",
             t="the sister's old duel is chalked on every challenge stone and {p} calls her out on blood terms"),
  "C5": dict(needs=N("inside"), hits={"place","thing"}, shapes={"Within"}, fam="Reckoning: Schism",
             t="the fighters who follow {o} take {stake} and beat the heart for standing against {o}"),
  "C6": dict(needs=N("coin+men|paper+men"), hits={"thing"}, shapes={"Above","Beside","Within"}, fam="Rescue: Bound by debt",
             t="{p} buys up the ladder's entry debts and {p}s men take the heart and the top rungs to work them off"),
  "C7": dict(needs=N("paper|faith"), hits={"place","trade"}, shapes={"Above","Beside"}, fam="Acquire: Under seal",
             t="{stake} is put under {p}s seal on the grand bout's eve, and the heart is barred from the sand"),
  "C8": dict(needs=N("name+men"), hits={"name","place"}, shapes={"Beside"}, fam="Dispute: A bout instead of blood",
             t="{p}s village comes to the Ring for the old winners' blood, the heart's kin among them"),
  "C9": dict(needs=N("coin+inside"), hits={"thing","name"}, shapes={"Beside"}, fam="Reckoning: Swindle (Fled)",
             t="the ladder is cried down as a book at the season's draw, and the heart, whose wins were bought without her knowing, is named the fixer"),
 },
 leaders=lambda e: {"D1":["the showman, wounded","the sister, wounded"],"D2":["the showman, disgraced","the sister, wounded"],
                     "D4":["the showman, disgraced"],"D6":["the showman, wounded","the sister, wounded"],
                     "D8":["the sister, wounded","the showman, wounded"]}.get(e, []),
)

# ---------------- HOLY HOUSE ----------------
H = dict(
 enemies={
  "HH1": dict(who="the high priest", p="he", want="ambition", gloss="an armed house the powers must need",
              means={"faith","men","inside"}, reach="region", shape="Within", kind="hidden hand", side="Holy House",
              stakes=["S2","S3"], scheme="has the heresy lists copied from the tithe-houses' arrears books, so whoever stops paying is named a heretic"),
  "HH2": dict(who="the Mount's tithe-master", p="he", want="preservation", gloss="the order fed",
              means={"paper","inside","men"}, reach="region", shape="Within", kind="the cost", side="Holy House",
              stakes=["S2","S4"], scheme="keeps the protection tithe rising town by town, though it has emptied the kitchens the priestess feeds from"),
  "HH3": dict(who="a Seated House patron", p="he", want="ambition", gloss="the order's riders for his house's quarrel",
              means={"paper","men","kin"}, reach="region", shape="Above", kind="the old debt", side="Seated Houses",
              stakes=["S1","S5"], scheme="means to call in the old grant's last term, which pledges the order's riders to his house"),
  "HH4": dict(who="a relic-seller from the pilgrim stalls", p="he", want="greed", gloss="the house's coin",
              means={"craft","coin","inside"}, reach="local", shape="Beside", kind="hidden hand", side="none",
              stakes=["S6"], scheme="sells the house 'recovered pieces' of the true procession relic he holds, while the one in procession is his copy",
              blows={("heart","captive"),("heart","disgraced"),("player","framed"),("guild","leader struck down")}),
  "HH5": dict(who="a Binder debt broker", p="he", want="greed", gloss="people to sell",
              means={"coin","paper","men"}, reach="region", shape="Beside", kind="the cost", side="Binders",
              band=True, stakes=["S3","S4"], scheme="buys the penance debts the inquisitors' book sets, and takes the families who can't pay"),
  "HH6": dict(who="an inquisitor", p="he", want="cover-up", gloss="an old burning kept buried",
              means={"faith","men"}, reach="region", shape="Within", kind="the old debt", side="Holy House",
              stakes=["S2","S3"], scheme="hunts as heretics the survivors of the shrine he burned with its keeper inside"),
  "HH7": dict(who="a True Line elder", p="he", want="ambition", gloss="his line above the Seated Houses",
              means={"coin","inside","kin"}, reach="region", shape="Above", kind="hidden hand", side="True Line",
              stakes=["S2","S3"], scheme="buys the inquisitors' favour with grain and horses, so heresy charges fall on the Seated Houses' clients"),
  "HH8": dict(who="a Last Wave raid leader", p="he", want="revenge", gloss="the riders paid back for a burned smoke hall",
              means={"men","name"}, reach="region", shape="Beside", kind="the old debt", side="Last Wave",
              band=True, stakes=["S3","S4"], scheme="raids the wayside shrines and the towns that tithe to them, years after the order's riders burned his enclave's smoke hall as heathen"),
  "HH9": dict(who="a Low Shrine priest", p="he", want="faith", gloss="a house that feeds people again",
              means={"inside","name"}, reach="local", shape="Within", kind="hidden hand", side="Holy House",
              stakes=["S2","S4"], scheme="empties the tithe-houses' stores into the hungry's bowls and writes false receipts to hide it",
              blows={("player","framed"),("heart","disgraced"),("guild","leader struck down")}),
 },
 stakes={
  "S1": dict(name="the Mount", hit="place", needs=N("paper|kin"), shapes={"Above","Beside"}),
  "S2": dict(name="the Low Shrine", hit="place", needs=N("men|paper|faith|inside")),
  "S3": dict(name="the house's sanctuary", hit="name", needs=N("men|coin|name")),
  "S4": dict(name="the tithe rolls", hit="trade", needs=N("paper|inside|men")),
  "S5": dict(name="the old grant", hit="thing", needs=N("paper"), only={"HH3"}),
  "S6": dict(name="the procession relic", hit="thing", needs=N("craft"), only={"HH4"}),
 },
 moves={
  "M1": dict(needs=N("paper"), hits={"place","thing"}, shapes={"Above"}, t="lays his house's claim to {stake} before his house's court"),
  "M2": dict(needs=N("men|coin+inside"), hits={"name","place"}, shapes={"Above","Beside","Within"}, nostake={"S1"},
             t="has a man taken from {site}"),
  "M3": dict(needs=N("faith"), hits={"place"}, shapes={"Within"}, t="has a priest of the priestess's way named a heretic from the Mount's steps"),
  "M4": dict(needs=N("paper"), hits={"trade"}, shapes={"Within"}, t="raises one valley's tithe and calls in its arrears at once"),
  "M5": dict(needs=N("coin"), hits={"trade"}, shapes={"Above","Beside"}, t="buys a tithe-house's debts at its door, in front of the town"),
  "M6": dict(needs=N("name"), hits={"name","trade"}, shapes={"Above","Beside","Within"}, t="cries in the markets that the house's shelter is worth nothing"),
  "M7": dict(needs=N("name+inside"), hits={"trade","place"}, shapes={"Within"}, t="feeds the outer ring from a tithe-house's door and calls it the house's duty"),
  "M8": dict(needs=N("craft"), hits={"thing"}, shapes={"Beside"}, t="puts a 'recovered piece' of the procession relic on show at the stalls, a keeper swearing to it"),
 },
 blows={
  ("heart","captive"): dict(needs=N("men|coin|paper|faith"), t="has the heart held"),
  ("heart","disgraced"): dict(needs=N("inside|faith"), t="has the heart accused of shaming the house in public"),
  ("heart","estranged"): dict(needs=N("kin"), t="turns the heart's kin against her"),
  ("player","framed"): dict(needs=N("inside|faith"), t="frames you for shaming the house in public"),
  ("player","stripped"): dict(needs=N("inside|faith"), t="has you stripped of your rank in the order"),
  ("player","arrested"): dict(needs=N("paper|men|faith"), menshape={"Above"}, t="has you arrested on the house's complaint"),
  ("guild","hall seized"): dict(needs=N("men|paper"), t="has a parish shrine shut or taken"),
  ("guild","members taken"): dict(needs=N("men|paper|coin|faith"), menshape={"Above"}, band=True, t="has novices and penitents taken"),
  ("guild","outlawed"): dict(needs=N("paper"), region=True, shapes={"Above","Beside"}, t="gets the house's shrines shut in one lordship"),
  ("guild","leader struck down"): dict(needs=N("men|inside"), t="strikes down {leader}"),
 },
 crises={
  "C1": dict(needs=N("men+inside|men+faith"), hits={"place"}, shapes={"Within"}, fam="Reckoning: Schism",
             t="{p} has the riders shut the Low Shrine's doors on its line, and the heart is named before the parish as the one who stirred it"),
  "C2": dict(needs=N("faith"), hits={"name","place"}, shapes={"Within"}, fam="Rescue: Held for heresy",
             t="the inquisitors take the heart's kin from {site} on a charge {p} laid, to be judged at the Mount on the feast day"),
  "C3": dict(needs=N("coin+men|paper+men"), hits={"trade","name"}, shapes={"Within","Beside","Above"}, fam="Rescue: Bound by debt",
             t="the penance debts are sold, the heart's family's among them, and the debtors are penned in the Mount's yard to march at dawn"),
  "C4": dict(needs=N("paper+men"), hits={"place","thing"}, shapes={"Above"}, fam="Acquire: Under seal",
             t="by his house's court his men take the Mount's gate under the old grant, and the heart's company of riders is the first pledged"),
  "C5": dict(needs=N("craft+coin"), hits={"thing"}, shapes={"Beside"}, fam="Reckoning: Swindle (Fled)",
             t="on the feast day the procession relic is shown to be his copy, he is gone to a lord's town, and the heart, who swore to it, is cast out"),
  "C6": dict(needs=N("men+name"), hits={"trade","name"}, shapes={"Beside"}, fam="Threat: Raiders",
             t="his raiders camp a night from the Mount, the tithe-towns' protection fails, and the heart's village is next"),
  "C7": dict(needs=N("inside+name"), hits={"place","trade"}, shapes={"Within"}, fam="Reckoning: Betrayal",
             t="his false receipts are found, the riders come to the Low Shrine for the grain, and the heart, who carried it, is named the thief"),
  "C8": dict(needs=N("coin+inside"), hits={"name","place"}, shapes={"Above"}, fam="Reckoning: Ruin from above (Bought)",
             t="at the feast-day judgment his bought inquisitors find the heart's kin heretics, taken from {site} where they sheltered, and their fields pass to his house"),
 },
 sites={"S2": "the Low Shrine's line", "S3": "a shrine's steps"},
 leaders=lambda e: {"HH1":["the priestess, disgraced"],"HH2":["the priestess, disgraced"],
                     "HH3":["the high priest, wounded, disgraced or killed"],"HH5":["the priestess, wounded"],
                     "HH6":["the priestess, disgraced"],"HH7":["the priestess, disgraced"],
                     "HH8":["the high priest, wounded or killed"],"HH9":["the high priest, disgraced"]}.get(e, []),
)

def legal(K):
    out = []
    for eid, e in K["enemies"].items():
        for sid in e["stakes"]:
            s = K["stakes"][sid]
            if not ok(s["needs"], e["means"]): continue
            if "shapes" in s and e["shape"] not in s["shapes"]: continue
            if "only" in s and eid not in s["only"]: continue
            for mid, m in K["moves"].items():
                if not ok(m["needs"], e["means"]) or e["shape"] not in m["shapes"]: continue
                if s["hit"] not in m["hits"] or sid in m.get("nostake", ()): continue
                for cid, c in K["crises"].items():
                    if not ok(c["needs"], e["means"]) or e["shape"] not in c["shapes"]: continue
                    if s["hit"] not in c["hits"]: continue
                    for bk, b in K["blows"].items():
                        if not ok(b["needs"], e["means"]): continue
                        if b.get("region") and e["reach"] == "local": continue
                        if "shapes" in b and e["shape"] not in b["shapes"]: continue
                        if "menshape" in b and not ok([a for a in b["needs"] if a != frozenset({"men"})], e["means"]) and e["shape"] not in b["menshape"] and not (b.get("band") and e.get("band")): continue
                        if "blows" in e and bk not in e["blows"]: continue
                        if bk[1] == "leader struck down" and not K["leaders"](eid): continue
                        out.append((eid, sid, mid, bk, cid))
    return out

def sentence(K, combo, rng):
    eid, sid, mid, bk, cid = combo
    e, s = K["enemies"][eid], K["stakes"][sid]
    p = e["p"]; P = p.capitalize()
    poss = {"he": "his", "she": "her"}[p]
    site = K.get("sites", {}).get(sid, s["name"])
    def f(t):
        return (t.replace("{p}s", poss).replace("{p}r", poss).replace("{p}", p).replace("{o}", {"he":"him","she":"her"}[p])
                 .replace("{stake}", s["name"]).replace("{site}", site))
    b = K["blows"][bk]["t"]
    if "{leader}" in b:
        b = b.replace("{leader}", rng.choice(K["leaders"](eid)))
    return (f"Because {e['who']} wants {e['want']} ({e['gloss']}), {p} {f(e['scheme'])}; "
            f"{p} {f(K['moves'][mid]['t'])}, then {f(b)}, and it comes to a head when {f(K['crises'][cid]['t'])}.")

if __name__ == "__main__":
    for name, K in (("dueling", D), ("holy-house", H)):
        L = legal(K)
        print(f"== {name}: {len(L)} legal combinations")
        es = sorted({c[0] for c in L}); print(" enemies used:", es)
        for k in ("enemies", "stakes", "moves", "crises"):
            used = sorted({c[{"enemies":0,"stakes":1,"moves":2,"crises":4}[k]] for c in L})
            unused = sorted(set(K[k]) - set(used))
            if unused: print("  UNUSED", k, unused)
        spines = sorted({(c[0], c[1], c[2], c[4]) for c in L})
        print(" spines (enemy,stake,move,crisis):", len(spines))
        from collections import Counter
        print(" crisis share over spines:", Counter(c[3] for c in spines))
        from collections import Counter as C2
        print(" share of combos per enemy:", dict(C2(c[0] for c in L)))
        # roll in the roller's order: the enemy first, then a legal combination for it
        rng = random.Random(93)
        es = sorted({c[0] for c in L})
        picks = []
        for _ in range(8):
            e = rng.choice(es)
            picks.append(rng.choice(sorted((c for c in L if c[0] == e), key=str)))
        for c in picks:
            print(" ", c[0], c[1], c[2], "/".join(c[3]), c[4])
            print("   ", sentence(K, c, rng))
