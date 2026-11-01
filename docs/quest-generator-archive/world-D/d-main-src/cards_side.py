"""Minor quests (3) and fallout (16)."""
from cards_main import *

GROUP.update({"needs it more": "The other side is right"}); TWIST_NAME.update({"needs it more": "Needs it more"})
def side(cid, rel, pl, ways, **c):
    rolled = roll_ways(cid, ways)
    removed = c.pop("removed", [])
    return card(id=cid, relatedness=rel, place=pl, ways_pool=ways, ways_rolled=rolled, ways_removed=removed, position=None,
                chapter=None, **c)
BF = "never offered before it: the card stands on this move"
def rcf(f): return [{"fact": f, "before": BF}]

# =============================================================== MINORS
# M1 -- attached to Trials 3a (Longwall); the veterans' captain from c-d-main-8
person("p-d-main-wallreeve", "Middle", ["reeve of the wall-village under Longwall's inner wall"], "pl-longwall")
place("pl-d-main-wall-village", "the wall-village under Longwall's inner wall", "village quarter", None, "open", "Low", "pl-longwall")
place("pl-d-main-ditch", "the new inner ditch below Longwall's inner wall", "earthwork", "p-d-main-veteran", "open", "Low", "pl-longwall")
fact("x-d-main-mi1-truth", "The inner ditch is cut too low: it runs under the mill-stream's bed and drains the stream into the ditch, as anyone "
     "can see who walks the cut at dusk.", vis="public", when="after m-d-main-3")
fact("x-d-main-mi1-tw", f"{n('p-d-main-wallreeve')} asked the veterans for the ditch herself, to keep the company's riders out of the wall-village.",
     vis="secret", on=["p-d-main-wallreeve"])
fact("x-d-main-adv-1", f"{given('p-d-main-wallreeve')} can put a cart through Longwall's inner gate unasked, and the veterans on it will carry "
     "word down the crown road to Highcrossing's lower town.", vis="secret", when="after m-d-main-3")
side("c-d-main-minor-1", "minor", "pl-d-main-wall-village", [
    W(f"stand in the cut at the mill-stream and stop {n('p-d-main-veteran')}'s diggers until the stream is answered for", "confront", "encroacher"),
    W("bargain a culvert under the ditch for the stream, in exchange for village hands on the digging", "bargain", "encroacher", truth=True),
    W("walk the cut at dusk with the diggers and show them where the water goes", "undermine", "project", truth=True),
    W("dam a holding pond above the ditch and keep the mill turning on what it holds", "protect", "requester"),
], family="Threat", premise="Encroachment", scope="community", offered_at="pl-longwall", attached_to="ch-d-main-3a",
   move="m-d-main-3", slots={"requester": "p-d-main-wallreeve", "community": "pl-d-main-wall-village", "encroacher": "p-d-main-veteran",
                             "project": "pl-d-main-ditch"},
   opposing_slot="encroacher", truth="x-d-main-mi1-truth", truth_row="Threat: project flawed", shapes=["built-to-fail"],
   twist=twist("one of us caused it", "x-d-main-mi1-tw"),
   pressure=f"Since the company took the outer towers, {n('p-d-main-veteran')}'s veterans have been cutting a ditch below the inner wall, and the "
            "wall-village's mill-stream is running into it. The village mill has stopped, and the veterans won't stop digging while crown riders sleep on the Wall.",
   pressure_level=1, separation="n/a", plants=[], depends_on=["x-d-main-m3"], reads_context=rcf("x-d-main-m3"),
   advantage={"fact": "x-d-main-adv-1", "for_node": "n-d-main-6", "ending": "the stream is answered for and the veterans keep their ditch"},
   details={"why_here": "the company in the outer towers", "truth_public": True},
   removed=[{"way": "petition the encroacher's lord", "reason": "the lord is the general, whose story cards are at the reuse cap (rulebook §4)"}],
   theme="the Wall", place_tie="the wall-village that grew up in the shadow of Longwall's inner wall",
   cast_log={"requester": {"how": "created", "why": "the wall-village's reeve: on demand, created, holds the post"},
             "community": {"how": "created", "why": "the wall-village, a new site"},
             "encroacher": {"how": "reused", "why": "the minor's person from the chapter: the veterans' captain of c-d-main-8"},
             "project": {"how": "created", "why": "the ditch, a new site"}})

# M2 -- attached to Trials 3b (Breakwall); the fort sergeant and Kotqot from c-d-main-11 / -12
thing("t-d-main-tally", "the company's tally of the tenth taken at the Brennock road fort", "document", "p-d0-oathford-lord", "Middle",
      "pl-d-main-breach", "p-kotqot", "m-d-main-3")
fact("x-d-main-mi2-seal", f"{n('p-kotqot')}'s gate seized the company's tally-cart at the breach for Breakwall's wall-toll, unpaid, and holds "
     "its tally under his seal until the toll is paid.", kind="past event", when="after m-d-main-3", on=["t-d-main-tally"])
fact("x-d-main-mi2-truth", "The tally in Breakwall's strongroom is the fair copy for the paymaster at Longwall; the tenth-clerk on the march "
     "road kept the original, with every carter's mark on it.", vis="secret", on=["t-d-main-tally"])
fact("x-d-main-adv-2", f"The tally names the company's paymaster, {n('p-d-main-paymaster')}, on every line: his own mark shows where the "
     "tenth-coin went.", vis="secret", when="after m-d-main-3")
side("c-d-main-minor-2", "minor", "pl-d-main-breach", [
    W(f"pay Breakwall's wall-toll and ask {n('p-kotqot')} for the tally in return", "earn", "holder"),
    W(f"show {given('p-kotqot')} what the tally would prove against the company that is quartered a day from his wall", "trade", "holder", truth=True),
    W("swap the sealed tally for a blank one with the same company seal-tag", "deceive", "target"),
    W("lift the tally out of the breach-gate's strongroom", "take", "holding_place"),
], family="Acquire", premise="Under seal", scope="community", offered_at="pr-road-fort", attached_to="ch-d-main-3b", move="m-d-main-3",
   slots={"requester": "p-d-main-fortsergeant", "target": "t-d-main-tally", "holder": "p-kotqot", "holding_place": "pl-d-main-breach"},
   opposing_slot="holder", reasons={"requester": "the carters' coin back", "holder": "the toll his wall is owed"},
   truth="x-d-main-mi2-truth", truth_row="Acquire: a copy", shapes=["a-copy"], twist=None,
   pressure=f"The company's riders took the stranded carters' tenth at the Brennock road fort, and its tally went off to Breakwall, where "
            f"{n('p-kotqot')} seized the cart for an unpaid wall-toll. {n('p-d-main-fortsergeant')} wants the tally: it is the only reckoning of what each carter paid.",
   pressure_level=1, separation="exempt (acquisition)", plants=[], depends_on=["x-d-main-m3"], reads_context=rcf("x-d-main-m3"),
   advantage={"fact": "x-d-main-adv-2", "for_node": "n-d-main-4a", "ending": "obtained and handed over"},
   details={"seal_type": "Fine unpaid", "seal": "x-d-main-mi2-seal"},
   theme="the tenth", place_tie="the breach Kotqot smashed in his own wall, where every cart into Breakwall pays",
   cast_log={"requester": {"how": "reused", "why": "the minor's person from the chapter: the fort sergeant of c-d-main-11"},
             "target": {"how": "created", "why": "the tally, a document thing"},
             "holder": {"how": "reused", "why": "holds Breakwall, on c-d-main-12 (+2)"},
             "holding_place": {"how": "reused", "why": "the breach, the site of c-d-main-12"}})

# M3 -- attached to Ordeal 5a (Oathford); the rival ferryman from c-d-main-19
fact("x-d-main-mi3-agree", f"When {n('p-d-main-rivalferry')} took Oathford's ford, he shook hands with the ford-wardens on the ferry stair: their "
     "men and toll-boxes cross free, and the ford is his.", kind="past event", when="after m-d-main-5")
fact("x-d-main-mi3-truth", "No one wrote the handshake down: the ford-wardens' clerk was at the muster that day, and the stair keeps no book.",
     vis="secret")
fact("x-d-main-adv-3", f"{n('p-d-main-fordwarden')} will tell you which remount lines came in from the Marches, and when.", vis="secret",
     when="after m-d-main-5")
side("c-d-main-minor-3", "minor", "pl-d-main-ferry", [
    W("find who stood on the ferry stair the day the hands were shaken", "truth", "party_b", truth=True),
    W("broker a toll: the wardens' men cross free, their boxes at half fare", "deal", "party_b"),
    W(f"back the ford-wardens' claim before the lord's steward, and let {given('p-d-main-rivalferry')} answer for his word", "side", "party_a"),
], family="Dispute", premise="Broken agreement", scope="community", offered_at="pl-oathford", attached_to="ch-d-main-5a", move="m-d-main-5",
   slots={"requester": "p-d-main-fordwarden", "party_a": "p-d-main-fordwarden", "party_b": "p-d-main-rivalferry"},
   opposing_slot="party_b", reasons={"party_a": "duty", "party_b": "livelihood"},
   truth="x-d-main-mi3-truth", truth_row="Dispute: no record", shapes=["no-record"], twist=None,
   pressure=f"Since {n('p-d-main-rivalferry')} took the ford, he has charged the ford-wardens full fare for every crossing, and their toll-boxes "
            "sit on the far bank. Without them the wardens can't keep the ford's peace when the company's columns cross.",
   pressure_level=1, separation="applies (trade: both parties Middle)", plants=[], depends_on=["x-d-main-m5"], reads_context=rcf("x-d-main-m5"),
   advantage={"fact": "x-d-main-adv-3", "for_node": "n-d-main-6", "ending": "A wins"},
   details={"agreement": "x-d-main-mi3-agree", "player_position": "advocate", "requester_is": "party_a", "arbiter_absent": True},
   theme="the ford", place_tie="the Oathford ferry stair, where the ford's bargains are shaken on",
   cast_log={"requester": {"how": "created", "why": "the head of the ford-wardens: on demand, created, holds the post"},
             "party_a": {"how": "created", "why": "the same"},
             "party_b": {"how": "reused", "why": "the minor's person from the chapter: the rival ferryman of c-d-main-19"}})

# =============================================================== FALLOUT (fallout §3)
def fall(cid, src, face, fills, pl, ways, checkpoint=None, **c):
    return side(cid, "fallout", pl, ways, source_move=src, face=face, move_fills=fills, aftershock_of=None,
                checkpoint=checkpoint, scope="community", depends_on=[MOVEFACT[src]], reads_context=rcf(MOVEFACT[src]), **c)
MOVEFACT = {f"m-d-main-{i}": f"x-d-main-m{i}" for i in range(1, 7)}
FALL_AT = {}

# F1 -- m1 (community): endure
person("p-d-main-f1-baker", "Low", ["baker at the gate oven"], "pl-crownsgate")
person("p-d-main-f1-breadcarter", "Low", ["bread-carter out of Highcrossing"], "pl-highcrossing")
place("pl-d-main-f1-oven", "the gate oven at Crownsgate", "bakehouse", "p-d-main-f1-baker", "open", "Low", "pl-crownsgate")
fact("x-d-main-f1-truth", f"Crownsgate's households have got used to {n('p-d-main-f1-breadcarter')}'s flatbread off the Highcrossing carts: "
     "it is cheaper than the oven's loaves, and they don't mean to go back.", vis="secret")
fall("c-d-main-fall-1", "m-d-main-1", "endure", "cause: with the Gate Mill's key held under the crown's seal, no flour comes to the oven",
     "pl-d-main-f1-oven", [
    W("win the households back with what the carts can't bring: a loaf hot from the oven at dawn", "craft", "requester", truth=True),
    W("buy flour from the Longwall millers until there is flour nearer home", "substitute", "requester"),
    W(f"talk {n('p-d-main-f1-breadcarter')} into carrying her loaves on his round as well as his own", "persuade", "rival", truth=True),
    W("gather the households who owe the oven a winter's credit and bake for them first", "gather", "requester"),
], family="Restore", premise="Lost livelihood", offered_at="pl-crownsgate",
   slots={"requester": "p-d-main-f1-baker", "broken": "pl-d-main-f1-oven", "rival": "p-d-main-f1-breadcarter"},
   opposing_slot=None, reasons={"requester": "survival"}, truth="x-d-main-f1-truth", truth_row="Restore: buyers went elsewhere",
   shapes=["trade-moved"], twist=None,
   pressure=f"Since the will put the Gate Mill's key under the crown's seal, no flour has come to the gate oven. {n('p-d-main-f1-baker')} "
            "has baked nothing for a month, and Crownsgate buys its bread off the Highcrossing carts.",
   pressure_level=1, separation="n/a", plants=[], details={"cause": "buyers went elsewhere (side-effect cause, fallout rule 1a)"},
   theme="bread", place_tie="the gate oven by the crown gate, where Crownsgate has baked for a hundred years",
   cast_log={"requester": {"how": "created", "why": "fallout: no one outside the main story fits"},
             "broken": {"how": "created", "why": "a new site"}, "rival": {"how": "created", "why": "no one fits"}})

# F2a -- m2: the taking's Rescue (rule 7a), endure
person("p-d-main-f2a-wife", "Low", ["smallholder's wife"], "pl-bogford")
person("p-d-main-f2a-debtor", "Low", ["smallholder", "taken into bond-service for the tenth"], "pl-bogford")
link("p-d-main-f2a-wife", "p-d-main-f2a-debtor", "spouse", "wife", "husband")
person("p-d-main-f2a-keeper", "Middle", ["keeper of the Sedge pens"], "pr-pens", faction={"id": "f-binders", "sub": None})
person("p-d-main-f2a-neighbour", "Low", ["holds a crown-sealed field beside the smallholder's"], "pl-bogford")
fact("x-d-main-f2a-truth", f"The tenth-roll is wrong: it names {n('p-d-main-f2a-debtor')} for the crown-sealed field of his neighbour "
     f"{n('p-d-main-f2a-neighbour')}; his own field is held by an old Free Lords' grant that owes the crown no tenth.", vis="secret")
fall("c-d-main-fall-2a", "m-d-main-2", "endure", "the debt fact: the sealing tenth, and bond-service for those who cannot pay", "pr-pens", [
    W("pay the tenth the roll says he owes, and take him home", "price", "captor"),
    W(f"work the tenth off in his place at the pens until the company's bond-lines come", "price", "captive"),
    W(f"find out whose field the roll really means, and bring {given('p-d-main-f2a-neighbour')} to the pens to answer for it", "leverage", "real_debtor", truth=True),
    W("cut him out of the pens at night", "force", "captive"),
], family="Rescue", premise="Bound by debt", offered_at="pl-bogford",
   slots={"requester": "p-d-main-f2a-wife", "captive": "p-d-main-f2a-debtor", "captor": "p-d-main-f2a-keeper", "place": "pr-pens",
          "real_debtor": "p-d-main-f2a-neighbour"},
   opposing_slot="captor", truth="x-d-main-f2a-truth", truth_row="Rescue: misassigned", shapes=["wrong-name"], twist=None,
   pressure=f"{n('p-d-main-f2a-debtor')} could not pay the sealing tenth by midsummer, and the clerks handed him to the Sedge pens to wait for "
            f"the company's bond-lines. {n('p-d-main-f2a-keeper')} feeds him on what his wife can bring, and the lines come at the month's end.",
   pressure_level=2, separation="applies (keeper Middle, captive Low)", plants=[],
   details={"rule_7a": "the taking's Rescue: the captive was taken by the move and fills no main or minor slot"},
   theme="the tenth", place_tie="the Sedge pens, where the Marches' bonded wait to be sold on",
   cast_log={"requester": {"how": "created", "why": "fallout: no one fits"}, "captive": {"how": "created", "why": "one the move took (rule 7a)"},
             "captor": {"how": "created", "why": "no keeper at the pens yet"}, "place": {"how": "reused", "why": "the Sedge pens, a map presence"},
             "real_debtor": {"how": "created", "why": "no one fits"}})

# F2b -- m2: profit
person("p-d-main-f2b-broker", "Middle", ["debt broker at the Chainmarket"], "pl-chainmarket", faction={"id": "f-binders", "sub": None})
person("p-d-main-f2b-daughter", "Low", ["smallholder's daughter"], "pl-chainmarket")
place("pl-d-main-f2b-room", "the debt broker's counting room at the Chainmarket", "counting room", "p-d-main-f2b-broker", "guarded", "Middle", "pl-chainmarket")
thing("t-d-main-f2b-grant", "an old Free Lords' grant to a Chainmarket smallholding", "document", "p-d-main-f2b-broker", "Middle",
      "pl-d-main-f2b-room", "p-d-main-f2b-broker", "m-d-main-2")
fact("x-d-main-f2b-truth", f"It is as {given('p-d-main-f2b-daughter')} says: her father sold the grant to pay the sealing tenth, at a tenth of "
     "its worth, and the broker simply won't sell it back.", vis="secret")
fact("x-d-main-f2b-tw", f"{given('p-d-main-f2b-broker')} means to burn the old grant, so the field can be re-sealed under the crown and sold to a "
     "buyer from the Chainmarket's pens.", vis="secret", on=["p-d-main-f2b-broker"])
fall("c-d-main-fall-2b", "m-d-main-2", "profit", "reason: her father sold the grant to the broker to pay the sealing tenth",
     "pl-d-main-f2b-room", [
    W(f"work for {n('p-d-main-f2b-broker')} until the grant is paid for twice over", "earn", "holder"),
    W(f"outbid the buyer {given('p-d-main-f2b-broker')} has waiting, with the field's own next harvest", "trade", "holder", truth=True),
    W("swap the grant for a burned scrap and let the broker think it gone", "deceive", "target"),
    W("take the grant from the counting room before it goes in the brazier", "take", "holding_place"),
], family="Acquire", premise="Won't sell", offered_at="pl-chainmarket",
   slots={"requester": "p-d-main-f2b-daughter", "target": "t-d-main-f2b-grant", "holder": "p-d-main-f2b-broker", "holding_place": "pl-d-main-f2b-room"},
   opposing_slot="holder", reasons={"requester": "legacy", "holder": "profit"},
   truth="x-d-main-f2b-truth", truth_row="Acquire: as told", shapes=[], twist=twist("meant for destruction", "x-d-main-f2b-tw"),
   pressure="Debt brokers at the Chainmarket are buying the grants of smallholders who can't meet the sealing tenth. "
            f"{n('p-d-main-f2b-daughter')}'s father sold theirs for a tenth of its worth; the broker won't sell it back at any price.",
   pressure_level=2, separation="exempt (acquisition)", plants=[], details={},
   theme="the tenth", place_tie="the Binder debt brokers' row at the Chainmarket",
   cast_log={"requester": {"how": "created", "why": "no one fits"}, "target": {"how": "created", "why": "a document thing"},
             "holder": {"how": "created", "why": "the Chainmarket's debt broker presence, created as a person"},
             "holding_place": {"how": "created", "why": "a new site"}})

# F2late -- m2: resist, late fallout
person("p-d-main-f2l-grainwife", "Low", ["grain-wife of the Breakwall fields"], "pl-breakwall")
person("p-d-main-f2l-carrier", "Low", ["carrier"], "pl-breakwall")
person("p-d-main-f2l-postguard", "Low", ["guard at the company's tenth-post", "new to the company"], "pl-breakwall",
       faction={"id": "f-great-house", "sub": "f-throne"})
place("pl-d-main-f2l-post", "the company's tenth-post on the Breakwall road", "toll post", None, "guarded", "Low", "pl-breakwall")
place("pl-d-main-f2l-market", "Breakwall's grain market", "market", None, "open", "Low", "pl-breakwall")
thing("t-d-main-f2l-grain", "a harvest of barley in sacks", "goods", "p-d-main-f2l-grainwife", "Low", "pl-breakwall", "p-d-main-f2l-grainwife", "before the story")
fact("x-d-main-f2l-truth", f"{n('p-d-main-f2l-carrier')}, who carts her sacks, sells the tenth-post word of which loads are hers.", vis="secret")
fall("c-d-main-fall-2late", "m-d-main-2", "resist", "the ban fact: the sealing tenth, taken at the post in coin, grain or men", "pl-d-main-f2l-market", [
    W("hide the sacks under a dung-cart's load", "cover", "cargo"),
    W("take the barley in by the bog paths, a sack at a time", "route", "cargo"),
    W(f"change carriers and tell the new one nothing until the road", "cover", "carrier", truth=True),
    W(f"pay {n('p-d-main-f2l-postguard')} to count her sacks short", "arrangement", "pursuer"),
], checkpoint=1, family="Passage", premise="Smuggling", offered_at="pl-breakwall",
   slots={"requester": "p-d-main-f2l-grainwife", "cargo": "t-d-main-f2l-grain", "pursuer": "p-d-main-f2l-postguard", "origin": "pl-d-main-f2l-post",
          "destination": "pl-d-main-f2l-market", "carrier": "p-d-main-f2l-carrier"},
   opposing_slot="pursuer", truth="x-d-main-f2l-truth", truth_row="Passage: the pursuer was told", shapes=["sold-from-inside"], twist=None,
   pressure=f"The company has set a tenth-post on the Breakwall road, and every load of grain into the market leaves a tenth there. "
            f"{n('p-d-main-f2l-grainwife')} means to get this year's barley past it whole, and the post has stopped her twice.",
   pressure_level=1, separation="exempt (crime)", plants=[], details={"late": True},
   theme="the tenth", place_tie="Breakwall's grain market, inside the wall Kotqot broke",
   cast_log={"requester": {"how": "created", "why": "no one fits"}, "cargo": {"how": "created", "why": "goods"},
             "pursuer": {"how": "created", "why": "a bystander member of the company (fallout rule 5)"},
             "origin": {"how": "created", "why": "a new site"}, "destination": {"how": "created", "why": "a new site"},
             "carrier": {"how": "created", "why": "no one fits"}})

# F3a -- m3: the enemy's own (a bystander member is the requester)
person("p-d-main-f3a-rider", "Low", ["rider of the emperor's new company", "new to the company"], "pl-highcrossing",
       faction={"id": "f-great-house", "sub": "f-throne"})
person("p-d-main-f3a-brother", "Low", ["carter on the Crossing road"], "pl-highcrossing")
link("p-d-main-f3a-rider", "p-d-main-f3a-brother", "family", "brother", "brother")
person("p-d-main-f3a-mother", "Low", ["carter's widow on the Crossing road"], "pl-highcrossing")
link("p-d-main-f3a-mother", "p-d-main-f3a-rider", "family", "parent", "child")
link("p-d-main-f3a-mother", "p-d-main-f3a-brother", "family", "parent", "child")
place("pl-d-main-f3a-row", "the riders' row at Highcrossing", "barracks row", None, "open", "Low", "pl-highcrossing")
fact("x-d-main-f3a-truth", f"{given('p-d-main-f3a-rider')} told his captain which quiet roads his brother's carts used, to win his place in the "
     "company; the riders took his brother's cart on the first of them.", vis="secret")
fall("c-d-main-fall-3a", "m-d-main-3", "endure", "cause: the company's riders stop every cart on the Crossing road to take the tenth by force",
     "pl-d-main-f3a-row", [
    W(f"carry {given('p-d-main-f3a-rider')}'s words to his brother at the carters' fires", "persuade", "estranged_b"),
    W(f"get {given('p-d-main-f3a-rider')} to say aloud what he told his captain, before his brother hears it from someone else", "persuade", "estranged_a", truth=True),
    W(f"bring them both to {given('p-d-main-f3a-mother')}'s hearth on her naming-day", "gather", "requester"),
    W("help them part on decent terms before the company rides north", "substitute", "estranged_b"),
], family="Restore", premise="Estrangement", offered_at="pl-highcrossing",
   slots={"requester": "p-d-main-f3a-mother", "estranged_a": "p-d-main-f3a-rider", "estranged_b": "p-d-main-f3a-brother"},
   opposing_slot=None, reasons={"requester": "kin"}, truth="x-d-main-f3a-truth", truth_row="Restore: a secret that got told",
   shapes=["secret-told"], twist=None,
   pressure=f"{n('p-d-main-f3a-rider')} took the company's coat in spring; since its riders took his brother's cart on the Crossing road, "
            f"his brother won't speak to him. Their mother, {n('p-d-main-f3a-mother')}, wants both sons at her hearth on her naming-day, ten days off.",
   pressure_level=1, separation="n/a", plants=[], details={"cause": "a secret that got told", "hard_part": "a deadline: the naming-day"},
   theme="kin", place_tie="the riders' row, the new barracks the company was given in the capital",
   cast_log={"requester": {"how": "created", "why": "their mother: no one fits"}, "estranged_a": {"how": "created", "why": "a bystander member of the company (fallout rule 5)"},
             "estranged_b": {"how": "created", "why": "his brother, created with the family link"}})

# F3b -- m3: endure (Investigate: a bystander culprit, fallout rule 8)
person("p-d-main-f3b-carter", "Low", ["carter laid up at Southroad"], "pl-southroad")
person("p-d-main-f3b-potboy", "Low", ["pot-boy at the carters' inn"], "pl-southroad")
person("p-d-main-f3b-drover", "Low", ["stranger drover"], "pl-southroad")
place("pl-d-main-f3b-inn", "the carters' inn at Southroad", "inn", None, "open", "Low", "pl-southroad")
fact("x-d-main-f3b-truth", f"{n('p-d-main-f3b-potboy')}, the inn's pot-boy, took the box from the stable loft while the carters slept off "
     "the wait; it is under the inn's ash-pit.", vis="secret", on=["p-d-main-f3b-potboy"])
fall("c-d-main-fall-3b", "m-d-main-3", "endure", "the scene: carters lie up at Southroad for days because the riders stop every cart on the Crossing road",
     "pl-d-main-f3b-inn", [
    W("look over the stable loft for how the box went out", "examine", "scene", truth=True),
    W("question the carters laid up in the inn", "question", "scene", truth=True),
    W(f"ask {n('p-d-main-f3b-drover')}, whom the carters blame, where he slept", "question", "red_herring", truth=True),
    W("watch who goes out to the ash-pit after dark", "follow", "scene", truth=True),
], family="Investigate", premise="Theft", offered_at="pl-southroad",
   slots={"requester": "p-d-main-f3b-carter", "victim": "p-d-main-f3b-carter", "culprit": "p-d-main-f3b-potboy", "scene": "pl-d-main-f3b-inn",
          "red_herring": "p-d-main-f3b-drover"},
   opposing_slot="culprit", motive="greed (an opportunist)", truth="x-d-main-f3b-truth", truth_row="Investigate: a bystander",
   shapes=["bystander-did-it"], twist=None,
   pressure=f"Carters wait days at Southroad for the Crossing road to open, and {n('p-d-main-f3b-carter')}'s road-money box has gone from the "
            "inn's stable loft. The carters blame the stranger drover; without the box she can't pay the next toll.",
   pressure_level=1, separation="n/a (crime)", plants=[], possible_endings=["confront privately", "blackmail", "cover it up", "let it go", "truth not found"],
   details={"arbiter_absent": True}, theme="the road", place_tie="the carters' inn by the Southroad toll line",
   cast_log={"requester": {"how": "created", "why": "no one fits"}, "victim": {"how": "created", "why": "the same"},
             "culprit": {"how": "created", "why": "a bystander opportunist"}, "scene": {"how": "created", "why": "a new site"},
             "red_herring": {"how": "created", "why": "no one fits"}})

# F3late -- m3: resist, late fallout
person("p-d-main-f3l-freeman", "Low", ["freeholder of the Breakwall fields"], "pl-breakwall")
person("p-d-main-f3l-rider", "Low", ["rider of the emperor's new company", "a bondswoman in the company's coat", "new to the company"], "pl-breakwall",
       faction={"id": "f-great-house", "sub": "f-throne"})
person("p-d-main-f3l-keeper", "Low", ["keeps Breakwall's dueling ladder"], "pl-breakwall", faction={"id": "f-dueling", "sub": None})
place("pl-d-main-f3l-yard", "Breakwall's ladder yard", "dueling yard", "p-d-main-f3l-keeper", "open", "Low", "pl-breakwall")
fact("x-d-main-f3l-truth", f"The fight is for what {given('p-d-main-f3l-freeman')} says: the ox the riders took at the roadside, and the "
     "right to say so to a rider's face.", vis="secret")
fact("x-d-main-f3l-tw", f"{n('p-d-main-f3l-rider')} is bonded into the company; if she loses a challenge in its coat, her bond is doubled.",
     vis="secret", on=["p-d-main-f3l-rider"])
fall("c-d-main-fall-3late", "m-d-main-3", "resist", "reason: the company's riders took his ox for the tenth by force at the roadside", "pl-d-main-f3l-yard", [
    W(f"train {given('p-d-main-f3l-freeman')} for first blood: a short blade and a long reach", "prepare", "principal"),
    W("raise the stakes: the ox back if he wins, his cart to the company if he loses", "stakes", "opponent", truth=True),
    W(f"hold the riders to the ladder's own rules before {n('p-d-main-f3l-keeper')}: no second, no armour", "rules", "ring_keeper"),
], checkpoint=1, family="Contest", premise="First blood or last", offered_at="pl-breakwall",
   slots={"requester": "p-d-main-f3l-freeman", "principal": "p-d-main-f3l-freeman", "opponent": "p-d-main-f3l-rider",
          "ring_keeper": "p-d-main-f3l-keeper"},
   opposing_slot="opponent", truth="x-d-main-f3l-truth", truth_row="Contest: the stated reason", shapes=[],
   twist=twist("needs it more", "x-d-main-f3l-tw"),
   pressure=f"The company's riders took {n('p-d-main-f3l-freeman')}'s ox for the tenth at the roadside, and he has called the rider who did it to "
            "first blood on Breakwall's ladder. The yard is full every evening to see whether a rider will answer.",
   pressure_level=2, separation="n/a", plants=[], details={"terms": "first blood", "late": True},
   theme="the tenth", place_tie="Breakwall's ladder yard, inside the broken wall",
   cast_log={"requester": {"how": "created", "why": "no one fits"}, "principal": {"how": "created", "why": "the same"},
             "opponent": {"how": "created", "why": "a bystander member of the company (fallout rule 5)"},
             "ring_keeper": {"how": "created", "why": "Breakwall's ladder-keeper presence, created as a person"}})

# F4a -- m4: profit
person("p-d-main-f4a-boss", "Middle", ["crew boss of a Carter's Rest hauling crew"], "pl-carters-rest")
place("pl-d-main-f4a-yard", "the crew boss's wagon yard at Carter's Rest", "wagon yard", "p-d-main-f4a-boss", "open", "Middle", "pl-carters-rest")
fact("x-d-main-f4a-wrong", f"{n('p-dorog')} has called in {n('p-d-main-f4a-boss')}'s notes a season early and taken his crew's wagons against them.",
     kind="past event", when="after m-d-main-4")
fact("x-d-main-f4a-truth", f"{given('p-d-main-f4a-boss')} had already missed two quarter-days on the notes; the house was within its terms, only early.",
     vis="secret")
fall("c-d-main-fall-4a", "m-d-main-4", "profit", "reason: the Counting-House advanced the company a year's pay against the bridge's tolls, and calls in "
     "the crews' notes early to cover it", "pl-d-main-f4a-yard", [
    W(f"shame {n('p-dorog')} before the crews at Carter's Rest, whose hauling the house lives on", "ruin", "wrongdoer"),
    W("show the crews the two missed quarter-days, and make the house give the wagons back for one paid in full", "amends", "wrongdoer", truth=True),
    W(f"break {given('p-dorog')}'s hold on the wagons by hauling them out of the house's yard by night", "violence", "wrongdoer"),
], family="Reckoning", premise="Ruin from above", offered_at="pl-carters-rest",
   slots={"requester": "p-d-main-f4a-boss", "wronged": "p-d-main-f4a-boss", "wrongdoer": "p-dorog"},
   opposing_slot="wrongdoer", reasons={"requester": "redress"}, truth="x-d-main-f4a-truth", truth_row="Reckoning: shared blame",
   shapes=["shared-blame"], twist=None,
   pressure=f"With the Counting-House's silver lent to the company against the Great Bridge's tolls, {n('p-dorog')} has called in "
            f"{n('p-d-main-f4a-boss')}'s notes a season early and taken his crew's wagons. Eight carters sit idle in his yard.",
   pressure_level=2, separation="exempt", plants=[],
   details={"shield": "Rank: the wrongdoer is High and the requester Middle", "wrong": "x-d-main-f4a-wrong",
            "no_answers_to": "the one the debt-taker answers to holds a main card: the family-rule way is left out (fallout rule 4)"},
   theme="debt", place_tie="the wagon yards of Carter's Rest, where the crews winter",
   cast_log={"requester": {"how": "created", "why": "no one fits"}, "wronged": {"how": "created", "why": "the same"},
             "wrongdoer": {"how": "reused", "why": "the Counting-House's other twin, on no main card, not linked to the cast (fallout rule 4)"}})

# F4b -- m4: resist
person("p-d-main-f4b-tollwife", "Low", ["toll-keeper of the Great Bridge", "would not take crown coin"], "pl-southroad")
person("p-d-main-f4b-brother", "Low", ["carter at Wainyard"], "pl-wainyard")
link("p-d-main-f4b-tollwife", "p-d-main-f4b-brother", "family", "sister", "brother")
person("p-d-main-f4b-tollman", "Low", ["crown toll-man", "new to the company"], "pl-southroad", faction={"id": "f-great-house", "sub": "f-throne"})
place("pl-d-main-f4b-loft", "the carters' loft at Wainyard", "loft", "p-d-main-f4b-brother", "open", "Low", "pl-wainyard")
place("pl-d-main-f4b-toll-row", "the toll-keepers' row at Southroad", "houses", None, "open", "Low", "pl-southroad")
place("pl-d-main-f4b-road", "the Silver Road from Southroad to Wainyard", "road", None, "open", "Low", "pl-wainyard")
fact("x-d-main-f4b-truth", f"{given('p-d-main-f4b-tollwife')} carries the toll-house's takings-book, with every keeper's crown coin written in "
     "it; that's why the toll-man follows, not the coin she refused.", vis="secret")
fall("c-d-main-fall-4b", "m-d-main-4", "resist", "reason: crown coin is buying Southroad's toll-keepers one by one, and she refused it",
     "pl-d-main-f4b-loft", [
    W("dress her as a Road Crews carter and drive her out with a load", "cover", "cargo"),
    W(f"lay a false trail toward the Counting-House for {n('p-d-main-f4b-tollman')}", "cover", "pursuer"),
    W("go by the Four Ways stone and the drovers' tracks, slow and unseen", "route", "cargo"),
    W(f"find out what {given('p-d-main-f4b-tollman')} is really sent after, and give him a reason to stop", "arrangement", "pursuer", truth=True),
], family="Passage", premise="Fugitive", offered_at="pl-southroad",
   slots={"requester": "p-d-main-f4b-brother", "cargo": "p-d-main-f4b-tollwife", "pursuer": "p-d-main-f4b-tollman", "origin": "pl-d-main-f4b-toll-row",
          "destination": "pl-d-main-f4b-loft", "route": "pl-d-main-f4b-road"},
   opposing_slot="pursuer", reasons={"cargo": "refused the crown's coin"}, truth="x-d-main-f4b-truth", truth_row="Passage: not what it seems",
   shapes=["more-than-said"], twist=None,
   pressure=f"{n('p-d-main-f4b-tollwife')} would not take the crown's coin to turn her toll-gate, and the crown's toll-men have been at her door "
            "every night since. Her brother at Wainyard asks for her to be got to him.",
   pressure_level=2, separation="exempt (authority)", plants=[], details={"why_flee": "refused crown coin"},
   theme="tolls", place_tie="the carters' lofts at Wainyard, where the Road Crews' yards end",
   cast_log={"requester": {"how": "created", "why": "her brother, who will take her in: no one fits"}, "cargo": {"how": "created", "why": "no one fits"},
             "pursuer": {"how": "created", "why": "a bystander member of the company (fallout rule 5)"},
             "origin": {"how": "created", "why": "a new site"}, "destination": {"how": "created", "why": "a new site"},
             "route": {"how": "created", "why": "a route place"}})

# F4late -- m4: endure, late fallout
person("p-d-main-f4l-fighter", "Low", ["carter who fights on the ladder"], "pl-carters-rest")
person("p-d-main-f4l-opponent", "Low", ["ladder fighter"], "pl-carters-rest")
person("p-d-main-f4l-keeper", "Low", ["keeps the ladder at the tavern-yard ring"], "pl-carters-rest", faction={"id": "f-dueling", "sub": None})
place("pl-d-main-f4l-ring", "the tavern-yard ring at Carter's Rest", "dueling ring", "p-d-main-f4l-keeper", "open", "Low", "pl-carters-rest")
fact("x-d-main-f4l-truth", f"{given('p-d-main-f4l-fighter')} wants off the ladder: his crew boss takes his purses for the bridge's tolls, and he would "
     "sooner lose cleanly and be struck off than keep fighting for them.", vis="secret", on=["p-d-main-f4l-fighter"])
fall("c-d-main-fall-4late", "m-d-main-4", "endure", "reason: the Great Bridge's tolls under the company's grant have doubled what a crew pays to cross",
     "pl-d-main-f4l-ring", [
    W(f"train {given('p-d-main-f4l-fighter')} for the next rung against {n('p-d-main-f4l-opponent')}", "prepare", "principal"),
    W("ask him plainly whose purse his winnings go into", "edge", "principal", truth=True),
    W(f"get {n('p-d-main-f4l-keeper')} to let him fight for his own purse, not his crew's", "rules", "ring_keeper"),
], checkpoint=2, family="Contest", premise="The ladder", offered_at="pl-carters-rest",
   slots={"requester": "p-d-main-f4l-fighter", "principal": "p-d-main-f4l-fighter", "opponent": "p-d-main-f4l-opponent",
          "ring_keeper": "p-d-main-f4l-keeper"},
   opposing_slot="opponent", truth="x-d-main-f4l-truth", truth_row="Contest: a way out", shapes=["wants-out"], twist=None,
   pressure=f"Since the bridge's tolls doubled, {n('p-d-main-f4l-fighter')} fights on the Carter's Rest ladder every quarter-day to pay what his crew "
            "owes at the Great Bridge. He asks for help climbing the next rung.",
   pressure_level=1, separation="n/a", plants=[], details={"late": True},
   theme="tolls", place_tie="the tavern-yard ring where the carters of Carter's Rest settle their quarrels",
   cast_log={"requester": {"how": "created", "why": "no one fits"}, "principal": {"how": "created", "why": "the same"},
             "opponent": {"how": "created", "why": "no one fits"}, "ring_keeper": {"how": "created", "why": "no one fits"}})

# F5a -- m5: the mirror (a new person in the heart's place; door family Dispute)
person("p-d-main-f5a-sealcutter", "Middle", ["seal-cutter", "named in the writ"], "pl-longwall")
person("p-d-main-f5a-halfbrother", "Middle", ["seal-cutter's half-brother"], "pl-longwall")
person("p-d-main-f5a-father", "Middle", ["old seal-cutter"], "pl-longwall", dead=True)
link("p-d-main-f5a-sealcutter", "p-d-main-f5a-halfbrother", "family", "brother", "brother")
link("p-d-main-f5a-father", "p-d-main-f5a-sealcutter", "family", "parent", "child")
link("p-d-main-f5a-father", "p-d-main-f5a-halfbrother", "family", "parent", "child")
place("pl-d-main-f5a-shop", "the seal-cutter's shop on Longwall's wall street", "workshop", "p-d-main-f5a-father", "open", "Middle", "pl-longwall")
fact("x-d-main-f5a-truth", f"The old seal-cutter's will survives in the copy-book of Longwall's tithe-house, in the tithe-clerk's hand, and leaves "
     f"the shop to {given('p-d-main-f5a-sealcutter')}.", vis="secret")
fall("c-d-main-fall-5a", "m-d-main-5", "mirror", "reason: the writ names him alongside the forger, so no court takes a paper from his hand, and his "
     "half-brother claims the shop as a forger's goods", "pl-d-main-f5a-shop", [
    W("find the old seal-cutter's will where the writ can't reach it: in another clerk's hand", "truth", "stake", truth=True),
    W(f"prove at the bench who cut the shop's seals these ten years", "truth", "party_a"),
    W(f"broker a division: the shop to {given('p-d-main-f5a-sealcutter')}, half its custom to his brother until the writ is lifted", "deal", "party_b"),
    W(f"back {given('p-d-main-f5a-halfbrother')}'s claim, and keep the shop open under a name the writ doesn't bar", "side", "party_b"),
], family="Dispute", premise="Contested inheritance", offered_at="pl-longwall",
   slots={"requester": "p-d-main-f5a-sealcutter", "party_a": "p-d-main-f5a-sealcutter", "party_b": "p-d-main-f5a-halfbrother",
          "stake": "pl-d-main-f5a-shop", "deceased": "p-d-main-f5a-father"},
   opposing_slot="party_b", reasons={"party_a": "livelihood", "party_b": "profit"}, truth="x-d-main-f5a-truth",
   truth_row="Dispute: record survives", shapes=["record-survives"], twist=None,
   pressure=f"The writ that names the Gate Mill's miller a forger names {n('p-d-main-f5a-sealcutter')} too, for cutting seals she used. His half-brother "
            "claims their father's shop as a forger's goods, and no court will take a paper from his hand to answer.",
   pressure_level=2, separation="applies (both heirs Middle)", plants=[],
   details={"mirror": "same move, same door family (Dispute), a different premise from c-d-main-24; unlinked to the heart", "arbiter_absent": True},
   theme="the writ", place_tie="the seal-cutters' street under Longwall's inner wall",
   cast_log={"requester": {"how": "created", "why": "the mirror: a new person (fallout rule 14)"}, "party_a": {"how": "created", "why": "the same"},
             "party_b": {"how": "created", "why": "his half-brother"}, "stake": {"how": "created", "why": "a new site"},
             "deceased": {"how": "created", "why": "the dead are newly created"}})

# F5b -- m5: endure
person("p-d-main-f5b-weaver", "Low", ["weaver"], "pl-reefhold")
person("p-d-main-f5b-reeve", "Middle", ["keeps the market seals at Reefhold"], "pl-reefhold")
place("pl-d-main-f5b-sealhouse", "the market-seal house at Reefhold", "seal house", "p-d-main-f5b-reeve", "guarded", "Middle", "pl-reefhold")
thing("t-d-main-f5b-bales", "a season's bales of reef-cloth", "goods", "p-d-main-f5b-weaver", "Middle", "pl-d-main-f5b-sealhouse", "p-d-main-f5b-reeve", "m-d-main-5")
fact("x-d-main-f5b-seal", f"{n('p-d-main-f5b-reeve')} sealed the bales in the market-seal house until a court rules on them: they came in on a "
     "flour-ticket written in the writ-named miller's hand.", kind="past event", when="after m-d-main-5", on=["t-d-main-f5b-bales"])
fact("x-d-main-f5b-truth", "It is as the weaver says: the bales are hers, and only the ticket is the trouble.", vis="secret")
fact("x-d-main-f5b-tw", f"{given('p-d-main-f5b-reeve')} wants the bales gone: they fill half his seal house, and he can't release them without a "
     "ruling no court will make.", vis="secret", on=["p-d-main-f5b-reeve"])
fall("c-d-main-fall-5b", "m-d-main-5", "endure", "the seal type: held until a ruling, because the writ bars any paper from the forger's hand",
     "pl-d-main-f5b-sealhouse", [
    W(f"carry the bales out of {given('p-d-main-f5b-reeve')}'s seal house for him, under a new ticket in the weaver's own hand", "trade", "holder", truth=True),
    W("earn the reeve's ruling by finding him a court that will hear it", "earn", "holder"),
    W("swap the ticket on the bales for a clean one", "deceive", "target"),
], family="Acquire", premise="Under seal", offered_at="pl-reefhold",
   slots={"requester": "p-d-main-f5b-weaver", "target": "t-d-main-f5b-bales", "holder": "p-d-main-f5b-reeve", "holding_place": "pl-d-main-f5b-sealhouse"},
   opposing_slot="holder", reasons={"requester": "livelihood", "holder": "duty"}, truth="x-d-main-f5b-truth", truth_row="Acquire: as told",
   shapes=[], twist=twist("wants to be rid of it", "x-d-main-f5b-tw"),
   pressure=f"{n('p-d-main-f5b-weaver')}'s season of reef-cloth came into Reefhold with a load of Gate Mill flour, on a ticket in the miller's "
            "hand; since the writ, Reefhold's market-seal house holds the bales until a ruling, and the selling season is going by.",
   pressure_level=1, separation="exempt (acquisition)", plants=[], details={"seal_type": "Held until a ruling", "seal": "x-d-main-f5b-seal"},
   theme="the writ", place_tie="Reefhold's market-seal house, where every bale off the reef road is stamped",
   cast_log={"requester": {"how": "created", "why": "no one fits"}, "target": {"how": "created", "why": "goods"},
             "holder": {"how": "created", "why": "no one fits"}, "holding_place": {"how": "created", "why": "a new site"}})

# F5late -- m5: profit, late fallout
person("p-d-main-f5l-sister", "Low", ["lodging-keeper's sister"], "pl-ashring")
person("p-d-main-f5l-dead", "Low", ["lodging-keeper"], "pl-ashring", dead=True)
link("p-d-main-f5l-sister", "p-d-main-f5l-dead", "family", "sister", "brother")
person("p-d-main-f5l-taker", "Low", ["takes the writ's reward for names"], "pl-ashring")
place("pl-d-main-f5l-lodge", "the lodging-house at Ashring", "lodging-house", "p-d-main-f5l-sister", "open", "Low", "pl-ashring")
fact("x-d-main-f5l-wrong", f"{n('p-d-main-f5l-taker')} named {n('p-d-main-f5l-dead')} to the writ's officers as one who had sheltered the forger, for "
     "their reward; the lodging-keeper died of cold in their cell before he was heard.", kind="past event", when="after m-d-main-5")
fact("x-d-main-f5l-truth", f"{given('p-d-main-f5l-taker')} is still at it: he has named two more households on the Vale road since, and the officers "
     "pay him for every one.", vis="secret")
fall("c-d-main-fall-5late", "m-d-main-5", "profit", "reason: the writ makes sheltering the forger a crime, and its officers pay for names",
     "pl-d-main-f5l-lodge", [
    W(f"bring the blood-debt before {n('p-d0-ashring-head')}, who keeps Ashring's peace", "justice", "arbiter"),
    W(f"follow {given('p-d-main-f5l-taker')} to the next house he names, and let the Vale road see what he does", "ruin", "wrongdoer", truth=True),
    W(f"make {given('p-d-main-f5l-taker')} pay the dead man's price to his sister", "amends", "wrongdoer"),
], checkpoint=2, family="Reckoning", premise="Blood debt", offered_at="pl-ashring",
   slots={"requester": "p-d-main-f5l-sister", "wronged": "p-d-main-f5l-dead", "wrongdoer": "p-d-main-f5l-taker", "arbiter": "p-d0-ashring-head"},
   opposing_slot="wrongdoer", reasons={"requester": "blood price"}, truth="x-d-main-f5l-truth", truth_row="Reckoning: still profiting",
   shapes=["still-profiting"], twist=None,
   pressure=f"{n('p-d-main-f5l-taker')} named {n('p-d-main-f5l-dead')} to the writ's officers for their reward, and the lodging-keeper died of cold "
            "in their cell. His sister claims the blood-debt; the man who named him sleeps three doors down.",
   pressure_level=2, separation="exempt (crime)", plants=[], details={"wrong": "x-d-main-f5l-wrong", "demand": "blood price", "late": True},
   theme="the writ", place_tie="the lodging-house on the Vale road where Ashring's travellers sleep",
   cast_log={"requester": {"how": "created", "why": "no one fits"}, "wronged": {"how": "created", "why": "the dead are newly created"},
             "wrongdoer": {"how": "created", "why": "no one fits"},
             "arbiter": {"how": "reused", "why": "the headman of Ashring (premade, authority slot)"}})

# F6a -- m6: resist (Threat Raiders: never none)
person("p-d-main-f6a-farmwife", "Low", ["farm-wife of the Firststone outlying farms"], "pl-firststone")
person("p-d-main-f6a-son", "Low", ["rides with the old levies"], "pl-firststone")
link("p-d-main-f6a-farmwife", "p-d-main-f6a-son", "family", "parent", "child")
person("p-d-main-f6a-leader", "Low", ["lawless", "once a palace levy"], "pl-firststone")
place("pl-d-main-f6a-farms", "the outlying farms of Firststone", "farms", None, "open", "Low", "pl-firststone")
fact("x-d-main-f6a-truth", "When the company was named the emperor's own guard, the old palace levies were turned out of Highcrossing's barracks "
     "unpaid; this band took to the roads, and has come as far as Firststone.", vis="public", when="after m-d-main-6")
fact("x-d-main-f6a-tw", f"{n('p-d-main-f6a-son')}, {given('p-d-main-f6a-farmwife')}'s own son, rides with the levies, and showed them where the farms keep their stores.",
     vis="secret", on=["p-d-main-f6a-son"])
fall("c-d-main-fall-6a", "m-d-main-6", "resist", "why-here: driven out, when the company's naming as the emperor's guard turned the old levies out",
     "pl-d-main-f6a-farms", [
    W(f"face {n('p-d-main-f6a-leader')}'s levies at the farm gates", "confront", "leader"),
    W("teach the farms to bring the stores in behind one wall", "protect", "requester"),
    W(f"get {given('p-d-main-f6a-son')} away from the band, and the band's knowledge of the stores with him", "undermine", "member", truth=True),
    W("offer the levies what the palace never paid: a winter's work at the farms", "bargain", "leader"),
], family="Threat", premise="Raiders", offered_at="pl-firststone",
   slots={"requester": "p-d-main-f6a-farmwife", "community": "pl-d-main-f6a-farms", "leader": "p-d-main-f6a-leader", "camp": "pr-survey",
          "member": "p-d-main-f6a-son"},
   opposing_slot="leader", truth="x-d-main-f6a-truth", truth_row="Threat: driven out", shapes=["pushed-here"],
   twist=twist("one of us caused it", "x-d-main-f6a-tw"),
   pressure=f"A band of the old palace levies, turned out of Highcrossing's barracks, has struck Firststone's outlying farms three nights in "
            f"five. {n('p-d-main-f6a-farmwife')}'s farm has lost its seed-grain, and the next farm means to fight.",
   pressure_level=2, separation="exempt (crime)", plants=[],
   removed=[{"way": "cut off their patron or buyer", "reason": "truth is not 'paid'"}], details={"why_here": "driven out (public)"},
   theme="the guard", place_tie="the outlying farms on the edge of the First Stone's land",
   cast_log={"requester": {"how": "created", "why": "no one fits"}, "community": {"how": "created", "why": "a new site"},
             "leader": {"how": "created", "why": "no one fits"}, "camp": {"how": "reused", "why": "the Edge survey camp, a map presence near Firststone, abandoned for the winter"},
             "member": {"how": "created", "why": "her son, created with the family link"}})

# F6b -- m6: endure (Investigate: a bystander culprit)
person("p-d-main-f6b-widow", "Low", ["grain-porter's widow"], "pl-highcrossing")
person("p-d-main-f6b-dead", "Low", ["grain-porter at the river wharf"], "pl-highcrossing", dead=True)
link("p-d-main-f6b-widow", "p-d-main-f6b-dead", "spouse", "wife", "husband")
person("p-d-main-f6b-culprit", "Low", ["wharf tallyman"], "pl-highcrossing")
place("pl-d-main-f6b-wharf", "the river wharf at Highcrossing", "wharf", None, "open", "Low", "pl-highcrossing")
fact("x-d-main-f6b-truth", f"{n('p-d-main-f6b-culprit')}, a wharf tallyman, has been shorting the crown's grain-sacks as they come off the "
     f"boats; {given('p-d-main-f6b-dead')} caught him at it, and he pushed the porter off the wharf's end.", vis="secret", on=["p-d-main-f6b-culprit"])
fall("c-d-main-fall-6b", "m-d-main-6", "endure", "the scene: Crownsgate's seized winter grain comes in by boat at the wharf, and is tallied there",
     "pl-d-main-f6b-wharf", [
    W("look at the wharf's end where he went in, and at the sacks stacked there", "examine", "scene", truth=True),
    W("question the porters who unloaded the grain-boats that night", "question", "scene", truth=True),
    W("count the sacks against the tally and see whose count runs short", "follow", "culprit", truth=True),
], family="Investigate", premise="Suspicious death", offered_at="pl-highcrossing",
   slots={"requester": "p-d-main-f6b-widow", "victim": "p-d-main-f6b-dead", "culprit": "p-d-main-f6b-culprit", "scene": "pl-d-main-f6b-wharf"},
   opposing_slot="culprit", motive="greed (an opportunist)", truth="x-d-main-f6b-truth", truth_row="Investigate: a bystander",
   shapes=["bystander-did-it"], twist=None,
   pressure=f"{n('p-d-main-f6b-dead')} was pulled out of the river under the wharf where the crown's grain-boats unload. The wharf says he was drunk; "
            "his widow says he never drank, and that he'd been counting the sacks.",
   pressure_level=2, separation="n/a (crime)", plants=[], possible_endings=["confront privately", "blackmail", "cover it up", "let it go", "truth not found"],
   details={"arbiter_absent": True}, theme="grain", place_tie="the river wharf under the Empty Gallery, where the capital's grain comes in",
   cast_log={"requester": {"how": "created", "why": "no one fits"}, "victim": {"how": "created", "why": "the dead are newly created"},
             "culprit": {"how": "created", "why": "a bystander opportunist"}, "scene": {"how": "created", "why": "a new site"}})

# F6late -- m6: the enemy's own, late fallout
person("p-d-main-f6l-captain", "Middle", ["captain of the old palace guard"], "pl-highcrossing", faction={"id": "f-great-house", "sub": None})
person("p-d-main-f6l-quartermaster", "Middle", ["quartermaster of the emperor's new company", "new to the company"], "pl-highcrossing",
       faction={"id": "f-great-house", "sub": "f-throne"})
place("pl-d-main-f6l-yard", "the old palace guard-yard", "guard-yard", None, "guarded", "Middle", "pl-highcrossing")
fact("x-d-main-f6l-truth", f"{given('p-d-main-f6l-captain')} lies about the stores: the old guard bought them with crown coin, not from its own "
     "purse, and the yard's tally-board shows it.", vis="secret", on=["p-d-main-f6l-captain"])
fall("c-d-main-fall-6late", "m-d-main-6", "enemy's own", "reason: the company was named the emperor's own guard, and claims the old guard's yard with the name",
     "pl-d-main-f6l-yard", [
    W("read the yard's tally-board to see who paid for the stores", "truth", "stake", truth=True),
    W("broker a sharing: the well to both, the stores to whoever paid for them", "deal", "party_b"),
    W(f"back {given('p-d-main-f6l-quartermaster')}'s claim before the old guard can empty the stores", "side", "party_a"),
], checkpoint=3, family="Dispute", premise="Shared resource", offered_at="pl-highcrossing",
   slots={"requester": "p-d-main-f6l-quartermaster", "party_a": "p-d-main-f6l-quartermaster", "party_b": "p-d-main-f6l-captain",
          "stake": "pl-d-main-f6l-yard"},
   opposing_slot="party_b", reasons={"party_a": "duty", "party_b": "pride"}, truth="x-d-main-f6l-truth", truth_row="Dispute: one side lies",
   shapes=["one-side-lies"], twist=None,
   pressure=f"Since the company was named the emperor's own guard, {n('p-d-main-f6l-quartermaster')} has been told to feed it from the old palace "
            f"guard-yard; {n('p-d-main-f6l-captain')} says the yard's well and stores are the old guard's own and won't give up the keys.",
   pressure_level=1, separation="applies (both Middle)", plants=[],
   details={"player_position": "advocate", "requester_is": "party_a", "arbiter_absent": True,
            "enemy_own": "a bystander member of the company is the requester (fallout rule 13)"},
   theme="the guard", place_tie="the old palace guard-yard under the Empty Gallery",
   cast_log={"requester": {"how": "created", "why": "a bystander member of the company (fallout rule 5)"}, "party_a": {"how": "created", "why": "the same"},
             "party_b": {"how": "created", "why": "no one fits"}, "stake": {"how": "created", "why": "a new site"}})
