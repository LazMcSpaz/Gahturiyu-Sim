"""The 29 main-story cards (layers 1-6 and the Return)."""
from registry import *
import rolls as RO

H, B, C, HD, LI, LT = "p-d-main-heart", "p-d-main-brother", "p-d-main-cousin", "p-d-main-hand", "p-d0-oathford-lord", "p-litita"
DU, GEN = "p-d0-crownsgate-warden", "p-haahoqo"
n = name
def plant(f, typ, knower=None, doc=False):
    d = {"fact": f, "type": typ}
    if knower: d["knower"] = knower
    if doc: d["document"] = True
    return d
BEFORE = "never offered before it: the card grows from this move"
def rc(*fs): return [{"fact": f, "before": BEFORE if f.startswith("x-d-main-m") else "the vote has not been called: the Long Shed is quiet, and the crews haul as usual"} for f in fs]
CH = {}
def ch(cid, **k): CH[cid] = k
def main(cid, chapter, position, move, pl, ways, turn=False, climax=False, **c):
    rolled = roll_ways(cid, ways, turn=turn, climax=climax)
    removed = c.pop("removed", []) + c.pop("ways_removed", [])
    return card(id=cid, chapter=chapter, position=position, move=move, place=pl, ways_pool=ways, ways_rolled=rolled,
                ways_removed=removed, landmark=(position == "turn"), **c)

# =============================================================== the plants and truths, written as facts
fact("x-d-main-p-lodged", f"{n(HD)}, a crown clerk out of Highcrossing, carried {n(C)}'s will to Crownsgate and lodged in the crown "
     "gate-house the week of the reading.", kind="plant", when="30 days before the story", on=[HD])
fact("x-d-main-p-lead", f"The seal-proofs {n(HD)} sold at Crownsgate and the will that claims the Gate Mill carry the same new crown seal, "
     "pressed in the sealing office at Highcrossing.", kind="plant", when="30 days before the story")
fact("x-d-main-p-company", f"The tenth's coin and its bondsmen go to the emperor's new company, which {n(LI)}, lord of Oathford, is raising "
     "in the Throne's pay.", kind="plant", when="after m-d-main-2")
fact("x-d-main-p-xipid", f"{n(B)}, who drove the Gate Mill's wagon, struck the company's sergeant at Bogford ford and was carried off to the "
     "lord's tower at Oathford under the crown clerk's warrant.", kind="plant", when="after m-d-main-2", on=[B])
fact("x-d-main-p-contract-read", "At the Last Well the tenth-clerk read out the Road Crews' haulage contract with the crown as the paper the "
     "crews' tenth is owed on.", kind="plant", when="after m-d-main-2")
fact("x-d-main-p-faultline", f"{n(DU)}, the old crown gate-warden at Crownsgate, refused to open the crown gate to the company's riders "
     "without the emperor's own hand on the order, and says the seal they carried is not his.", kind="plant", when="after m-d-main-3", on=[DU])
fact("x-d-main-p-ally", f"{n(GEN)} will stand with {n(H)} against the emperor's new company, if the company's arms and the bondsmen's oaths "
     "come to Longwall once it is broken.", kind="plant", when="after m-d-main-3", on=[GEN])
fact("x-d-main-p-riders-dawn", "The carts that go round by the quiet ways say the company's riders come out of Longwall's outer towers every "
     "dawn, and that the towers were opened to them from inside.", kind="plant", when="after m-d-main-3")
fact("x-d-main-p-captain-word", f"{n('p-d-main-towerwarden')} opened the outer towers on {n(LI)}'s word that he came as the emperor's own captain.",
     kind="plant", when="after m-d-main-3")
fact("x-d-main-p-stranded", "The carts that went round by the Reed Crossing are stranded under the Brennock road fort, and riders out of the "
     "Reedbank camp have begun to strike them at night.", kind="plant", when="after m-d-main-3")
fact("x-d-main-p-deserter-hides", f"Among the stranded carts hides {n('p-d-main-deserter')}, a bondsman who deserted the company, and he says he "
     "will tell what he saw at Highcrossing to whoever gets him to Breakwall.", kind="plant", when="after m-d-main-3")
fact("x-d-main-p-founding", f"{n('p-d-main-paidfighter')}'s father, the old toll-keeper {n('p-d-main-oldkeeper')}, keeps a paper from the "
     "bridge's building in the west toll hut, and doesn't know what it is.", kind="plant", when="after m-d-main-4")
fact("x-d-main-p-chainpost", f"{given('p-d-main-oldkeeper')} says the Great Bridge's toll-gate chain-post was sound at dusk on the night before "
     "the crown's re-grant was read on the bridge.", kind="plant", when="after m-d-main-4")
fact("x-d-main-p-cousin-hides", f"{n(C)}, the under-clerk who claimed the Gate Mill, has been turned out of the sealing office and hides "
     "with the drovers on the river meadows.", kind="plant", when="after m-d-main-4", on=[C])
fact("x-d-main-p-pledge", f"{given(C)} says {n(HD)} has pledged the Gate Mill's grain to the Counting-House against the company's pay, and "
     "goes to the Scales Hall himself on quarter-day.", kind="plant", when="after m-d-main-4")
fact("x-d-main-p-lad-taken", f"Writ-riders took {n('p-d-main-lad')}, {given('p-d-main-ferrywoman')}'s ferry-lad, who had hidden {given(H)} in "
     "his boat, and rode up the Elderhollow road.", kind="plant", when="after m-d-main-5")
fact("x-d-main-p-headman-sworn", f"The riders meant to sell word of {given(H)}'s road to the head of Elderhollow's ruling house, who has "
     "sworn to bar her at the town's bridge.", kind="plant", when="after m-d-main-5")
fact("x-d-main-p-clearances", f"The writ's papers came to {n('p-d-main-paperdealer')} from {n('p-d-main-notary')}, a Saltmarket notary "
     "selling 'clearances' to those the writ names.", kind="plant", when="after m-d-main-5")
fact("x-d-main-p-factor-stall", f"{given('p-d-main-notary')}'s clearances are stamped at the crown factor's stall on the Unchained Quay "
     "at Openharbour.", kind="plant", when="after m-d-main-5")
fact("x-d-main-p-short-guard", "With the company's columns held sick at Oathford, the grant-holders it took at Crownsgate are kept in the old "
     "gate-house under a short guard.", kind="plant", when="after m-d-main-6")
fact("x-d-main-p-oath-name", f"{n('p-d-main-bridgekeeper')} says the company's captain made the grant-holders swear to the Throne's keeper by "
     "name, not to the emperor.", kind="plant", when="after m-d-main-6")
fact("x-d-main-p-name-read", f"{n('p-d-main-bridgeson')} says the company's clerks still read {given(H)}'s name out as a forger at every muster.",
     kind="plant", when="after m-d-main-6")

# =============================================================== CHAPTER: Hook (layer 1, community) at Crownsgate
ch("ch-d-main-hook", node="n-d-main-1", role="Hook", problem="x-d-main-m1", cards=["c-d-main-1", "c-d-main-2"], region="r-crossing",
   settlement="pl-crownsgate")
fact("x-d-main-t1", f"The will is not the old miller's. {n(C)} put his father's name to it at {n(HD)}'s word, to clear what he owed the clerk; "
     f"{n(DU)} sat with {n('p-d-main-uncle')} in his last month and heard him promise the mill to {given(H)}.", vis="secret", kind="state",
     when="30 days before the story", on=[C])
main("c-d-main-1", "ch-d-main-hook", "opening", "m-d-main-1", "pl-d-main-gate-mill", [
    W(f"have {n(DU)}, who sat with {given('p-d-main-uncle')} in his last month, tell the hearing what he meant for the mill", "truth", "confidant", truth=True),
    W(f"prove at the millstones who kept the Gate Mill turning while {given('p-d-main-uncle')} was dying", "truth", "party_a"),
    W(f"broker a division: the mill to {given(H)}, its grinding fees to {given(C)} for ten years", "deal", "party_b"),
    W(f"buy {given(C)} out with what he wants more than a mill: a clean slate with the clerk he owes", "deal", "party_b"),
    W(f"back {given(H)}'s grant before {n(GEN)}, whose wall Crownsgate's grain feeds", "side", "arbiter"),
], family="Dispute", premise="Contested inheritance", scope="community", offered_at="pl-crownsgate",
   slots={"requester": H, "party_a": H, "party_b": C, "stake": "pl-d-main-gate-mill", "deceased": "p-d-main-uncle", "arbiter": GEN,
          "occupant": "p-d-main-millhand", "confidant": DU},
   opposing_slot="party_b", reasons={"party_a": "dependents", "party_b": "obligation"},
   truth="x-d-main-t1", truth_row="Dispute: one side lies", shapes=["one-side-lies"],
   twist=twist("caught in the middle", f"{n('p-d-main-millhand')} has slept in the mill loft forty years on {given('p-d-main-uncle')}'s word. "
               f"{given(H)} means to give the loft to her brother's household; {given(C)} means to sell the mill to the crown. Whoever wins, the old man loses his place."),
   pressure=f"Until the will is settled, {n(C)} holds the Gate Mill's key on the crown's seal, and the grain of Crownsgate's households waits "
            "unground in their lofts while the first frost comes on.",
   pressure_level=1, separation="applies (inheritance: both heirs Middle)",
   plants=[plant("x-d-main-m1", "harm"), plant("x-d-main-p-lodged", "unique")], depends_on=["x-d-main-m1"], reads_context=rc("x-d-main-m1"),
   details={"deceased": "p-d-main-uncle", "player_position": "advocate", "requester_is": "party_a", "stake": "the Gate Mill (place)",
            "will": "t-d-main-will"},
   cast_present=[H], proxies_present=[C], theme="inheritance",
   place_tie="the Gate Mill grinds the grain Crownsgate trades in, at the foot of the toll bridge",
   cast_log={"requester": {"how": "cast", "why": "the heart"}, "party_a": {"how": "cast", "why": "the heart"},
             "party_b": {"how": "created", "why": "no claimant on the map: created as the dead miller's son, an under-clerk of the crown sealing office (proxy: the Hand's employee)"},
             "stake": {"how": "created", "why": "no mill site at Crownsgate yet: created (casting §2.4)"},
             "deceased": {"how": "created", "why": "the dead are newly created (rulebook §4)"},
             "arbiter": {"how": "reused", "why": "premade High lord who outranks both heirs, lives near Crownsgate (+1), same region (+2); authority slot, no cap"},
             "occupant": {"how": "created", "why": "no one fits: the old mill-hand, created with the heart as employer"},
             "confidant": {"how": "reused", "why": "holds Crownsgate's crown gate (lives at the card's settlement, +1), same region (+2), a crown servant who heard the miller (prefers)"}})

fact("x-d-main-t2", f"{n('p-d-main-bridgekeeper')} gathered his neighbours' silver for the seal-proofs himself, and kept a tenth of it for "
     "his trouble.", vis="secret", kind="state", when="30 days before the story", on=["p-d-main-bridgekeeper"])
fact("x-d-main-swindle-hook", f"{n(HD)} sold eleven of Crownsgate's grant-holders 'seal-proofs' at three silver each, to guard their grants "
     "against claims like the Gate Mill's, and took the silver back to Highcrossing.", kind="past event", when="30 days before the story")
main("c-d-main-2", "ch-d-main-hook", "turn", "m-d-main-1", "pl-d-main-gate-house", [
    W(f"con {given(HD)} with a grant that looks forged, and let him try to sell a proof against it", "ruin", "wrongdoer"),
    W(f"find the other grant-holders {given(HD)} sold to on the Crownsgate road, and come to the gate-house together", "ruin", "wronged"),
    W(f"make {n(HD)} repay before the gate-house's own copy-clerks, whom a crown clerk can't be seen to ignore", "amends", "wrongdoer",
      truth=True, pays="x-d-main-p-lodged"),
    W(f"frighten {given(HD)} off the Crownsgate road before he sells to the next village", "violence", "wrongdoer"),
    W(f"follow {given(HD)} to the sealing office at Highcrossing and settle it under his own masters' roof", "amends", "wrongdoer", source="family rule"),
], turn=True, family="Reckoning", premise="Swindle", scope="community", offered_at="pl-crownsgate",
   slots={"requester": "p-d-main-bridgekeeper", "wronged": "p-d-main-bridgekeeper", "wrongdoer": HD},
   opposing_slot="wrongdoer", reasons={"requester": "repayment for the cheated grant-holders"},
   truth="x-d-main-t2", truth_row="Reckoning: shared blame", shapes=["shared-blame"], twist=None,
   pressure=f"Eleven of Crownsgate's grant-holders paid {n(HD)} three silver each for 'seal-proofs' that would guard their grants against "
            "claims like the one on the Gate Mill. The papers are worthless, the silver went with him to Highcrossing, and the next claim could come for any of them.",
   pressure_level=2, separation="exempt (crime)",
   plants=[plant("x-d-main-p-lead", "lead", doc=True)], depends_on=["x-d-main-m1", "x-d-main-p-lodged"], reads_context=rc("x-d-main-m1"),
   details={"shield": "Fled: the clerk went home to the sealing office at Highcrossing, his second place", "demand": "repayment",
            "wrong": "x-d-main-swindle-hook", "arbiter_absent": True, "requester_is": "wronged (speaks for the eleven)"},
   ways_removed=[], cast_present=[HD], proxies_present=[], theme="seals",
   place_tie="the crown gate-house, where Crownsgate's papers are read and copied for the road",
   cast_log={"requester": {"how": "created", "why": "no one fits: the toll bridge's keeper, created, holds that keeping"},
             "wronged": {"how": "created", "why": "the same: he speaks for the eleven"},
             "wrongdoer": {"how": "cast", "why": "the Hand"}})

# =============================================================== Threshold (layer 2, community), move m2
ch("ch-d-main-2a", node="n-d-main-2a", role="Threshold", problem="x-d-main-m2", cards=["c-d-main-3", "c-d-main-4"], region="r-marches",
   settlement="pl-oathford")
fact("x-d-main-taking-wagon", f"When the company's riders came to Bogford ford for the Gate Mill's tenth, the sergeant {n('p-d-main-sergeant')} "
     f"took the mill's flour wagon and its oxen in place of the coin, and {n('p-d0-bogford-lord')} took them as his share for letting the "
     "crown's clerks work his land.", kind="past event", when="after m-d-main-2")
fact("x-d-main-t3", f"{n('p-d0-bogford-lord')} has already promised the wagon and its oxen to the overseer of the bond-field below Bogford, "
     "to haul for the slave-bought field.", vis="secret", when="after m-d-main-2")
main("c-d-main-3", "ch-d-main-2a", "opening", "m-d-main-2", "pl-d-main-bogford-yard", [
    W(f"walk {n('p-d0-bogford-lord')} back along the wagon's road, from the ford where it was taken to his own yard, until he sees what he was paid with", "earn", "holder"),
    W(f"repay {given('p-d0-bogford-lord')} what the crown's share was worth to him", "trade", "holder", truth=True),
    W(f"make {n('p-d-main-sergeant')}, who cut the oxen from the traces, buy them back from the Free Lord himself", "earn", "original_taker"),
    W("swap the oxen for a team the Free Lord's drovers can't tell apart", "deceive", "target"),
    W("drive the Gate Mill's wagon out of the Free Lord's yard at night", "take", "holding_place"),
], family="Acquire", premise="Passed on", scope="community", offered_at="pl-bogford",
   slots={"requester": H, "target": "t-d-main-wagon", "holder": "p-d0-bogford-lord", "holding_place": "pl-d-main-bogford-yard",
          "original_taker": "p-d-main-sergeant"},
   opposing_slot="holder", reasons={"requester": "survival", "holder": "livelihood"},
   truth="x-d-main-t3", truth_row="Acquire: promised elsewhere", shapes=["another-buyer"], twist=None,
   pressure="Since the tenth was taken at Bogford ford, the Gate Mill's wagon stands in the Free Lord's yard; the flour of Crownsgate's households, "
            "and of the three Bogford farms it carried for, has no way to the Marches market before the roads close.",
   pressure_level=1, separation="exempt (acquisition)",
   plants=[plant("x-d-main-p-xipid", "unique")], depends_on=["x-d-main-m2"], reads_context=rc("x-d-main-m2"),
   details={"past_taking": "x-d-main-taking-wagon", "original_taker_alive": True},
   cast_present=[H], proxies_present=["p-d-main-sergeant"], theme="the tenth",
   place_tie="Bogford's ford, where the march road crosses the bog, and the Free Lord's yard above it",
   factions=[{"faction": "f-free-lords", "role": "backer"}],
   cast_log={"requester": {"how": "cast", "why": "the heart"},
             "target": {"how": "created", "why": "the Gate Mill's own wagon and team (the heart's)"},
             "holder": {"how": "reused", "why": "holds Bogford (lives at the card's place, +1), same region (+2); the Free Lords take a share of what crosses their land (prefers)"},
             "holding_place": {"how": "created", "why": "no yard at Bogford yet: created beside the holder's seat"},
             "original_taker": {"how": "created", "why": "no crown sergeant on the march road: created (proxy: the lieutenant's man)"}})

fact("x-d-main-t4", f"{n(B)} did strike {n('p-d-main-sergeant')} at Bogford ford, hard enough to break his jaw, when the sergeant cut the "
     "oxen from the wagon's traces.", vis="secret", when="after m-d-main-2", on=[B])
person("p-d-main-tenthclerk", "Middle", ["tenth-clerk of the sealing office on the march road", "holds legal authority by crown warrant"], "pl-bogford",
       faction={"id": "f-great-house", "sub": "f-throne"})
main("c-d-main-4", "ch-d-main-2a", "turn", "m-d-main-2", "pl-d-main-oathford-tower", [
    W(f"go over the charge with {n('p-d-main-sergeant')}, who swore it with his jaw still bound", "leverage", "witness", truth=True),
    W(f"bribe {n(HD)} to lose the warrant", "price", "captor"),
    W(f"bring the Road Crews' carters, with {n('p-doruduqu')} at their head, to crowd the tower gate until holding {given(B)} costs the clerk more than he's worth",
      "leverage", "representative"),
    W(f"break {given(B)} out of the lord's tower", "force", "captive"),
    W(f"spring {given(B)} on the road when he's sent on to the company's bond-lines", "stealth", "captive"),
], turn=True, family="Rescue", premise="Imprisoned unjustly", scope="community", offered_at="pl-crownsgate",
   slots={"requester": "p-d-main-sisterinlaw", "captive": B, "captor": HD, "place": "pl-d-main-oathford-tower",
          "witness": "p-d-main-sergeant", "representative": "p-doruduqu"},
   opposing_slot="captor", truth="x-d-main-t4", truth_row="Rescue: guilty", shapes=["held-rightly"],
   twist=twist("unwilling captive", f"{given(B)} won't go quietly out of the tower: he means to stand before the lord of Oathford and say, in "
               "front of the whole ford, what the crown's clerks are doing on the roads."),
   pressure=f"{n(B)} is held in the lord's tower at Oathford on a crown clerk's warrant for striking a crown sergeant, to be sent into the "
            "company's bond-service; his wife and child at Crownsgate live on what his wagon earns, and the wagon is gone.",
   pressure_level=2, separation="exempt (authority)",
   plants=[plant("x-d-main-p-company", "shared", knower=B)], depends_on=["x-d-main-m2", "x-d-main-p-xipid"],
   reads_context=rc("x-d-main-m2"), details={"charge": "striking a crown sergeant", "access": "guarded"},
   cast_present=[B, HD], proxies_present=["p-d-main-sergeant", "p-d-main-sisterinlaw"], theme="the tenth",
   place_tie="the lord's tower at Oathford, which holds the ford and its tolls",
   cast_log={"requester": {"how": "created", "why": "the captive's wife: created with the spouse link (kin of the fallback)"},
             "captive": {"how": "cast", "why": "the heart's fallback (not in her place here; normal reuse rules)"},
             "captor": {"how": "cast", "why": "the Hand, holding by crown warrant"},
             "place": {"how": "created", "why": "the lord's tower: Oathford's own landmark, made a site"},
             "witness": {"how": "reused", "why": "on c-d-main-3 (+2, same story), struck by the captive (+3, linked)"},
             "representative": {"how": "reused", "why": "the Road Crews' foreman: on another card of the story (+2), in a live court tension (+2)"}})

ch("ch-d-main-2b", node="n-d-main-2b", role="Threshold", problem="x-d-main-m2", cards=["c-d-main-5", "c-d-main-6"], region="r-roads",
   settlement="pl-carters-rest")
fact("x-d-main-t5", f"{n('p-d-main-carter')} did fire the crown's tally-shed at Bogford ford when the clerks took his team, with the "
     "tenth-clerk's boy asleep in its loft; the boy lived, burned.", vis="secret", when="after m-d-main-2", on=["p-d-main-carter"])
main("c-d-main-5", "ch-d-main-2b", "opening", "m-d-main-2", "pr-waystation", [
    W(f"go over the charge with the tenth-clerk's burned boy, who saw the fire set", "leverage", "witness", truth=True),
    W(f"bribe {n('p-d-main-tenthclerk')} to lose the carter's warrant", "price", "captor"),
    W(f"bring the stranded crews to the Last Well's gate, {n('p-d-main-informer')} among them, until the clerk would sooner be rid of his prisoner",
      "leverage", "representative", truth=True),
    W(f"break {given('p-d-main-carter')} out of the Last Well's chained cells", "force", "captive"),
    W(f"take {given('p-d-main-carter')} off the road when he's walked to the company's bond-lines", "stealth", "captive"),
], family="Rescue", premise="Imprisoned unjustly", scope="community", offered_at="pr-waystation",
   slots={"requester": "p-d0-wainyard-head", "captive": "p-d-main-carter", "captor": "p-d-main-tenthclerk", "place": "pr-waystation",
          "representative": "p-d-main-informer"},
   opposing_slot="captor", truth="x-d-main-t5", truth_row="Rescue: guilty", shapes=["held-rightly"],
   twist=twist("justified captor", "The clerk's case is the better one: a child was burned in the shed the carter fired."),
   pressure=f"Since the tenth was called on the march road, {n('p-d-main-tenthclerk')} holds a Road Crews carter in the Last Well's "
            "chained cells, rented from the Binders' keeper, and the crews' wagons stand at the waystation with him until the tenth is paid. "
            "Every crew on the march road is waiting to see whether the Crews pay or fight.",
   pressure_level=1, separation="exempt (authority)",
   plants=[plant("x-d-main-p-contract-read", "unique")], depends_on=["x-d-main-m2"], reads_context=rc("x-d-main-m2"),
   details={"charge": "firing the crown's tally-shed at Bogford ford"},
   cast_present=[], proxies_present=["p-d-main-tenthclerk"], theme="the tenth",
   place_tie="the Last Well, where the Walking Road's captives get their last drink and families leave tokens",
   cast_log={"requester": {"how": "reused", "why": "the carter's yard-boss: employer of the captive (+3, linked), Road Crews"},
             "captive": {"how": "created", "why": "no one fits: a Road Crews carter, created as the yard-boss's man"},
             "captor": {"how": "created", "why": "no crown clerk at the waystation: created (proxy, Throne)"},
             "place": {"how": "reused", "why": "the Last Well, a map presence on the march road"},
             "representative": {"how": "created", "why": "no one fits: a stranded carter, created"}},
   removed=[{"way": "go over the charge with the tenth-clerk's burned boy", "reason": "justified captor removes the prove ways (family-rescue §6)"}])
# the justified-captor removal: drop the way from the pool's rolled set if drawn
c4 = CARDS[-2]
if 2 not in c4["ways_rolled"]: c4["slots"].pop("representative"); c4["cast_log"].pop("representative")
c5 = CARDS[-1]; c5["ways_rolled"] = [i for i in c5["ways_rolled"] if i != 0]
if len(c5["ways_rolled"]) < 2: c5["ways_rolled"] = sorted(set(c5["ways_rolled"]) | {2, 3})
c5["ways_pool"][0]["removed"] = True

fact("x-d-main-t6", "The haulage contract in the Long Shed's strongbox, in the old foreman's hand, says the crown pays each load on delivery "
     "and owes the crews' road no tenth; the crews stopped hauling only after a season's pay went missing.", vis="secret")
fact("x-d-main-defect-chest", "The crown's haulage chest is empty: the sealing office has been paying old loads with new tenths.", vis="secret",
     on=["t-d-main-contract"])
person("p-d-main-oldcarter", "Low", ["carter", "shook on the crown's haulage contract twenty years ago"], "pl-carters-rest")
link(HD, "p-doruduqu", "trade", None, None, note="the crown's haulage contract, which the clerk keeps for the crown")
main("c-d-main-6", "ch-d-main-2b", "turn", "m-d-main-2", "pl-d-main-long-shed", [
    W(f"find {n('p-d-main-oldcarter')}, who shook on the contract with the crown's clerk twenty years back, and hear what was promised", "truth", "witness"),
    W("open the Long Shed's strongbox and read the contract itself, old seal and all, beside the new seal on the tenth", "truth", "stake",
      truth=True, pays="x-d-main-p-lead"),
    W(f"renegotiate with {given(HD)}: the crews haul for half the tenth, paid in loads, not coin", "deal", "party_b"),
    W(f"find a third party to cover the season's missing pay: {n('p-kodor')}, at the Counting-House's rates", "deal", "third_party"),
    W("enforce the contract for the crews, wagons and all, in front of the Long Shed", "side", "party_a"),
], turn=True, family="Dispute", premise="Broken agreement", scope="community", offered_at="pl-carters-rest",
   slots={"requester": H, "party_a": "p-doruduqu", "party_b": HD, "stake": "t-d-main-contract", "witness": "p-d-main-oldcarter",
          "third_party": "p-kodor"},
   opposing_slot="party_b", reasons={"party_a": "dependents", "party_b": "pride"},
   truth="x-d-main-t6", truth_row="Dispute: record survives", shapes=["record-survives"],
   twist=twist("hollow prize", "Whoever wins, the crown's haulage chest is empty: the season's back pay won't come, and the tenth was always meant to fill the chest."),
   pressure=f"{n(HD)} says the Road Crews broke the crown's haulage contract by refusing the tenth, and claims the crews' wagons on the march "
            f"road as forfeit; {n('p-doruduqu')} says the crown broke it first by not paying a season's loads. With the Long Shed already split over "
            f"the strike vote, no crew hauls crown grain until it's settled, and none of {given(H)}'s flour moves.",
   pressure_level=2, separation="applies (trade: both parties Middle)",
   plants=[plant("x-d-main-p-company", "shared", knower="p-doruduqu")],
   depends_on=["x-d-main-m2", "x-d-main-p-contract-read", "x-d-main-p-lead"], reads_context=rc("x-d-main-m2", "x-d-main-ctx-long-shed"),
   details={"player_position": "advocate", "requester_is": "neither party: the heart, whose flour rides on the crews' wagons, backs party A",
            "arbiter_absent": True, "agreement": "t-d-main-contract (a document thing)", "defect": "x-d-main-defect-chest"},
   cast_present=[H, HD], proxies_present=[], theme="hauling",
   place_tie="the Long Shed, where the crews vote by standing on one side of it or the other",
   cast_log={"requester": {"how": "cast", "why": "the heart"},
             "party_a": {"how": "looked up", "why": "the Road Crews stand through their elected foreman, who holds Carter's Rest"},
             "party_b": {"how": "cast", "why": "the Hand"},
             "stake": {"how": "created", "why": "the haulage contract: created as a document thing"},
             "witness": {"how": "created", "why": "no one fits: an old carter, created"},
             "third_party": {"how": "reused", "why": "in the live Long Shed tension (+2), grudge-linked to party A (+3)"}})

c6 = CARDS[-1]
if 3 not in c6["ways_rolled"]: c6["slots"].pop("third_party"); c6["cast_log"].pop("third_party")
if 0 not in c6["ways_rolled"]: c6["slots"].pop("witness"); c6["cast_log"].pop("witness"); PEOPLE.pop("p-d-main-oldcarter")
fact("x-d-main-ctx-long-shed", "Doruḍoqu the Carter has called the Road Crews to vote in the Long Shed on striking against the Counting-House's rates.",
     kind="state", when="before the story")
# =============================================================== Trials (layer 3, region), move m3: the lieutenant's layer
ch("ch-d-main-3a", node="n-d-main-3a", role="Trials", problem="x-d-main-m3", cards=["c-d-main-7", "c-d-main-8", "c-d-main-9"],
   region="r-crossing", settlement="pl-longwall")
person("p-d-main-merchant", "Middle", ["grain-merchant of the Crossing road"], "pl-southroad")
fact("x-d-main-t7", "The trade road moved: since the company's riders took the Crossing road, the carts of three lordships go round by the "
     "Reed Crossing, and no one passes the Broken Shrine's stalls.", vis="secret")
main("c-d-main-7", "ch-d-main-3a", "opening", "m-d-main-3", "pr-shrine-ruin", [
    W(f"talk {n('p-d-main-merchant')} into sending his grain-carts back past the Broken Shrine, riders or no riders", "persuade", "merchant", truth=True),
    W("gather the stall-families and the carters who ate there to roof the stalls again before the frost", "gather", "requester"),
    W("turn the empty stalls into a carters' stable on the quiet road the carts use now", "substitute", "requester"),
], family="Restore", premise="Lost livelihood", scope="region", offered_at="pr-shrine-ruin",
   slots={"requester": "p-d-main-stallwife", "broken": "pr-shrine-ruin", "merchant": "p-d-main-merchant"},
   opposing_slot=None, reasons={"requester": "survival"}, truth="x-d-main-t7", truth_row="Restore: the trade road moved", shapes=["trade-moved"],
   twist=None,
   pressure="The stalls at the Broken Shrine, which have fed the carters and pilgrims of the Crossing road for three generations, have stood "
            "empty since midsummer. The families who keep them winter on what they take before the frost, and so do the carters of three lordships who ate there cheap.",
   pressure_level=1, separation="n/a",
   plants=[plant("x-d-main-p-riders-dawn", "unique")], depends_on=["x-d-main-m3", "x-d-main-p-company"], reads_context=rc("x-d-main-m3"),
   details={"cause": "the trade road moved", "hard_part": "a deadline: the frost"},
   removed=[{"way": "find the apprentice who learned half the method", "reason": "cause is not 'the master died'"},
            {"way": "rebuild the method by trial", "reason": "cause is not 'the master died'"},
            {"way": "find a new seam or source", "reason": "cause is not 'resource ran out'"}],
   cast_present=[], proxies_present=[], theme="the road", place_tie="the ruined wayside shrine whose pilgrim stalls fed the Crossing road",
   cast_log={"requester": {"how": "created", "why": "no one fits: the stalls' keeper, created"},
             "broken": {"how": "reused", "why": "the Broken Shrine, a map presence"},
             "merchant": {"how": "created", "why": "no merchant of the Crossing road in the registry with a reason to be here: created"}})

person("p-d-main-veteran", "Middle", ["captain of the veterans who walk the Wall"], "pl-longwall", faction={"id": "f-great-house", "sub": "f-banner-lords"})
fact("x-d-main-betrayal-towers", f"{n('p-d-main-towerwarden')}, warden of Longwall's outer towers, opened them to the emperor's new company.",
     kind="past event", when="after m-d-main-3")
fact("x-d-main-t8", "The veterans had a hand in it: they stopped keeping the outer towers' watch when the pay stopped at midwinter, and left "
     "the warden alone on the wall with the keys.", vis="secret")
main("c-d-main-8", "ch-d-main-3a", "middle", "m-d-main-3", "pl-d-main-longwall-hall", [
    W(f"expose {n('p-d-main-towerwarden')} to the veterans who walk the Wall each year, whose trust he still needs", "ruin", "wrongdoer"),
    W(f"take back what the betrayal got {given('p-d-main-towerwarden')}: the company's silver in his strongbox", "amends", "wrongdoer"),
    W(f"bring {n('p-d-main-veteran')} and the warden face to face on the Wall", "amends", "wronged", truth=True),
    W(f"bring it before {n(GEN)} in his hall inside the inner wall", "justice", "arbiter"),
    W(f"settle it with {given('p-d-main-towerwarden')} by a beating at the inner gate, or a duel on the Wall", "violence", "wrongdoer", contest=True),
    W("warn the other wardens along the Wall, whom the company will try next, and dry up the warden's trade in keys", "ruin", "wrongdoer", source="family rule"),
], family="Reckoning", premise="Betrayal", scope="community", offered_at="pl-longwall",
   slots={"requester": "p-d-main-veteran", "wronged": "p-d-main-veteran", "wrongdoer": "p-d-main-towerwarden", "arbiter": GEN},
   opposing_slot="wrongdoer", reasons={"requester": "formal punishment"},
   truth="x-d-main-t8", truth_row="Reckoning: shared blame", shapes=["shared-blame"], twist=None,
   pressure=f"{n('p-d-main-towerwarden')}, warden of Longwall's outer towers, opened them to the emperor's new company; now crown riders sleep on "
            "the Wall the veterans walk every year and ride out against the Crossing every dawn, and the general's own men eye the inner gate.",
   pressure_level=2, separation="exempt",
   plants=[plant("x-d-main-p-ally", "ally", knower=GEN), plant("x-d-main-p-captain-word", "unique")],
   depends_on=["x-d-main-m3", "x-d-main-p-riders-dawn"], reads_context=rc("x-d-main-m3"),
   details={"shield": "No law against it: opening the emperor's towers to the emperor's captain breaks no law", "demand": "formal punishment",
            "wrong": "x-d-main-betrayal-towers", "link": "employer/employee (the warden serves the general; the veterans keep the Wall with him)"},
   cast_present=[], proxies_present=[], theme="the Wall", place_tie="Longwall's inner wall, where the veterans name their dead at each tower",
   cast_log={"requester": {"how": "created", "why": "no one fits: the veterans' captain, created"},
             "wronged": {"how": "created", "why": "the same"},
             "wrongdoer": {"how": "created", "why": "no warden of the outer towers yet: created, holds the post"},
             "arbiter": {"how": "reused", "why": "holds Longwall (the card's settlement), on c-d-main-1 (+2); authority slot, no cap; knower of the ally plant"}})

fact("x-d-main-t9", f"The commission {n(LI)} carries is a fair copy; the sealed original lies in {n(LT)}'s own rooms at Highcrossing, behind "
     "the Empty Gallery.", vis="secret", on=["t-d-main-commission"])
main("c-d-main-9", "ch-d-main-3a", "turn", "m-d-main-3", "pl-d-main-outer-towers", [
    W(f"give {given(LI)} what the commission does for him another way: {n(GEN)}'s word that the outer towers are his to hold till spring",
      "earn", "ally", pays="x-d-main-p-ally"),
    W(f"borrow the commission from {given(LI)} for one evening, under his eye, to set its seal beside another", "trade", "holder", truth=True),
    W(f"convince {given(LI)} that carrying it puts him at risk, that the general's men mean to take it off his body", "deceive", "holder"),
    W(f"become the one {given(LI)} would trust it to: the company's own clerk for the winter", "earn", "holder"),
    W(f"take the commission off {given(LI)} in the outer towers", "take", "holder"),
], turn=True, family="Acquire", premise="Won't sell", scope="region", offered_at="pl-longwall",
   slots={"requester": H, "target": "t-d-main-commission", "holder": LI, "holding_place": "pl-d-main-outer-towers", "ally": GEN},
   opposing_slot="holder", reasons={"requester": "evidence", "holder": "security"},
   truth="x-d-main-t9", truth_row="Acquire: a copy", shapes=["a-copy"], twist=None,
   pressure=f"The emperor's new company holds Longwall's outer towers on the strength of the commission {n(LI)} carries; whoever can set its "
            "seal beside the emperor's own can show the whole Crossing on whose word the company rides. He keeps it on his body and will not part with it.",
   pressure_level=3, separation="exempt (acquisition)",
   plants=[plant("x-d-main-p-faultline", "fault line", knower=H)],
   depends_on=["x-d-main-m3", "x-d-main-p-captain-word", "x-d-main-p-company"], reads_context=rc("x-d-main-m3"),
   details={"access": "fortified"},
   cast_present=[H, LI], proxies_present=[], theme="seals", place_tie="Longwall's empty outer towers, at the end of the ruined imperial wall",
   cast_log={"requester": {"how": "cast", "why": "the heart"}, "target": {"how": "created", "why": "the commission, created as a document thing"},
             "holder": {"how": "cast", "why": "the lieutenant"},
             "holding_place": {"how": "created", "why": "Longwall's outer towers, made a site"},
             "ally": {"how": "reused", "why": "only if way 0 is rolled: the general, the ally plant's knower on this chapter (+2)"}},
   removed=[{"way": "take the commission off him in the outer towers", "reason": "fortified: a take way that requires entering the place (family-acquire §8)"}])
c9 = CARDS[-1]
c9["ways_pool"][4]["removed"] = True
if 4 in c9["ways_rolled"]:
    c9["ways_rolled"] = [i for i in c9["ways_rolled"] if i != 4]
    for j in (3, 2, 0):
        if j not in c9["ways_rolled"]: c9["ways_rolled"] = sorted(c9["ways_rolled"] + [j]); break
if 0 not in c9["ways_rolled"]: c9["slots"].pop("ally"); c9["cast_log"].pop("ally")

ch("ch-d-main-3b", node="n-d-main-3b", role="Trials", problem="x-d-main-m3", cards=["c-d-main-10", "c-d-main-11", "c-d-main-12"],
   region="r-marches", settlement="pl-breakwall")
person("p-d-main-reedcutter", "Low", ["reed-cutter", "laid the plank causeway at the Reed Crossing"], "pr-reed-crossing")
fact("x-d-main-t10", "The causeway was built badly for this: laid for mules and smugglers' handcarts, a plank wide on reed bundles, it was never "
     "meant to bear loaded wagons, and every wagon since midsummer has sunk it further.", vis="secret")
main("c-d-main-10", "ch-d-main-3b", "opening", "m-d-main-3", "pr-reed-crossing", [
    W(f"find {n('p-d-main-reedcutter')}, who laid the causeway, and talk her into laying it again", "persuade", "holdout"),
    W("work out why the causeway sinks before relaying it to sink again", "craft", "broken", truth=True),
    W("trade the stranded carters' labour now for free passage over the crossing at harvest", "gather", "requester"),
    W("find another way over the reed-beds: the old drove-ford upstream", "substitute", "broken"),
], family="Restore", premise="Broken works", scope="region", offered_at="pr-reed-crossing",
   slots={"requester": "p-d-main-crossingkeeper", "broken": "pr-reed-crossing", "holdout": "p-d-main-reedcutter"},
   opposing_slot=None, reasons={"requester": "survival"}, truth="x-d-main-t10", truth_row="Restore: built badly", shapes=["built-to-fail"],
   twist=None,
   pressure="Since midsummer the carts of the Crossing and the Marches have gone round the toll road by the Reed Crossing, and its plank "
            "causeway has given way under them; a score of wagons are stuck or turned back, and when the autumn water comes the crossing goes under altogether.",
   pressure_level=1, separation="n/a",
   plants=[plant("x-d-main-p-stranded", "unique")], depends_on=["x-d-main-m3", "x-d-main-p-company"], reads_context=rc("x-d-main-m3"),
   details={"cause": "built badly", "hard_part": "a deadline: the autumn water"},
   removed=[{"way": "shame the neglectful owner into paying", "reason": "the requester is the owner"}],
   cast_present=[], proxies_present=[], theme="the road", place_tie="the Reed Crossing, the quiet way past the Southroad toll line",
   cast_log={"requester": {"how": "created", "why": "no one fits: the crossing's keeper, created"},
             "broken": {"how": "reused", "why": "the Reed Crossing, a map presence"},
             "holdout": {"how": "created", "why": "no one fits: the reed-cutter who laid it, created"}})

person("p-d-main-raidsecond", "Low", ["lawless", "the Reedbank leader's second"], "pr-raid-camp", faction={"id": "f-binders", "sub": None})
fact("x-d-main-t11", "The Reedbank band was driven out of its old ground in the bog camps when the company's riders swept the march road for "
     "bondsmen.", vis="public", kind="past event", when="after m-d-main-3")
fact("x-d-main-tw11", f"{n('p-d-main-informer')}, a carter among the stranded, has been selling the Reedbank riders word of which carts carry "
     "the tenth-coin, to see the crown's silver taken.", vis="secret", on=["p-d-main-informer"])
main("c-d-main-11", "ch-d-main-3b", "middle", "m-d-main-3", "pl-d-main-carters-camp", [
    W(f"drive {n('p-d-main-raider')}'s riders off in a fight at the camp's edge", "confront", "leader"),
    W("arm the stranded carters with the road fort's spare spears and teach them to hold a wagon ring", "protect", "requester"),
    W(f"turn {n('p-d-main-raidsecond')} against the Reedbank leader", "undermine", "band_lieutenant"),
    W(f"offer {given('p-d-main-raider')}'s band honest work hauling the stranded carts through to Breakwall", "bargain", "leader"),
], family="Threat", premise="Raiders", scope="community", offered_at="pr-road-fort",
   slots={"requester": "p-d-main-fortsergeant", "community": "pl-d-main-carters-camp", "leader": "p-d-main-raider", "camp": "pr-raid-camp",
          "member": "p-d-main-informer", "band_lieutenant": "p-d-main-raidsecond"},
   opposing_slot="leader", truth="x-d-main-t11", truth_row="Threat: driven out", shapes=["pushed-here"],
   twist=twist("one of us caused it", "x-d-main-tw11"),
   pressure="Two score carts stranded under the Brennock road fort are struck every night by riders out of the Reedbank camp; three carters "
            "are hurt and one cart burned, and the Lantern fort has six spears for the whole camp.",
   pressure_level=2, separation="exempt (crime)",
   plants=[plant("x-d-main-p-deserter-hides", "unique")], depends_on=["x-d-main-m3", "x-d-main-p-stranded"], reads_context=rc("x-d-main-m3"),
   details={"why_here": "driven out (public)", "secret_member": "x-d-main-tw11"},
   removed=[{"way": "cut off their patron or buyer", "reason": "truth is not 'paid'"}],
   cast_present=[], proxies_present=[], theme="the road", place_tie="the Brennock road fort, which guards the worst stretch of the march road",
   cast_log={"requester": {"how": "created", "why": "the road fort's Lantern sergeant: on demand, created, holds the post"},
             "community": {"how": "created", "why": "the stranded carts' camp, a new site under the fort"},
             "leader": {"how": "created", "why": "no lawless leader at the Reedbank camp yet: created"},
             "camp": {"how": "reused", "why": "the Reedbank camp, a map presence within reach"},
             "member": {"how": "reused", "why": "on c-d-main-5 (+2, same story)"},
             "band_lieutenant": {"how": "created", "why": "no one fits: created"}})

fact("x-d-main-t12", f"{n('p-d-main-crossingkeeper')}, who keeps the Reed Crossing, sold {given(LI)}'s riders word of which road the deserter "
     "would take.", vis="secret")
place("pl-d-main-march-road", "the march road from Bogford to Breakwall", "road", None, "open", "Low", "pl-bogford")
main("c-d-main-12", "ch-d-main-3b", "turn", "m-d-main-3", "pl-d-main-breach", [
    W(f"dress {n('p-d-main-deserter')} as a Road Crews carter and drive him in with the stranded carts", "cover", "cargo"),
    W(f"lay a false trail toward the Chainmarket for {given(LI)}'s riders", "cover", "pursuer"),
    W(f"get {given('p-d-main-deserter')} through the breach at Breakwall, if {n('p-kotqot')} will take a crown deserter in, and pay what he asks",
      "route", "haven_owner", price="that the deserter tells Breakwall what he saw at Highcrossing before he tells anyone else"),
    W(f"hire {n('p-d-main-fortsergeant')}'s spears to hold the march road behind him", "force", "guard"),
    W(f"buy {given(LI)}'s riders off the trail, or trade them something they want more", "arrangement", "pursuer", truth=True),
], turn=True, family="Passage", premise="Fugitive", scope="region", offered_at="pr-road-fort",
   slots={"requester": H, "cargo": "p-d-main-deserter", "pursuer": LI, "origin": "pl-d-main-carters-camp", "destination": "pl-d-main-breach",
          "route": "pl-d-main-march-road", "haven_owner": "p-kotqot", "guard": "p-d-main-fortsergeant"},
   opposing_slot="pursuer", reasons={"cargo": "deserter"}, truth="x-d-main-t12", truth_row="Passage: the pursuer was told",
   shapes=["sold-from-inside"], twist=None,
   pressure=f"{n('p-d-main-deserter')} deserted the emperor's new company with what he saw at Highcrossing in his head, and {n(LI)}'s own riders "
            "are a day behind him on the march road. If they take him, he goes back to the company's bond-lines in chains, and what he knows goes with him.",
   pressure_level=3, separation="exempt (crime and authority)",
   plants=[plant("x-d-main-p-faultline", "fault line", knower="p-kotqot"), plant("x-d-main-p-ally", "ally", knower="p-d-main-deserter")],
   depends_on=["x-d-main-m3", "x-d-main-p-deserter-hides", "x-d-main-p-company"], reads_context=rc("x-d-main-m3"),
   details={"why_flee": "deserter", "pursuer_link": "his master: the deserter was pressed into the lieutenant's company"},
   cast_present=[H, LI], proxies_present=["p-d-main-deserter"], theme="desertion", place_tie="the breach Kotqot smashed in his own wall and left open",
   cast_log={"requester": {"how": "cast", "why": "the heart"}, "cargo": {"how": "created", "why": "no one fits: a pressed bondsman, created (proxy: the lieutenant's man)"},
             "pursuer": {"how": "cast", "why": "the lieutenant"}, "origin": {"how": "reused", "why": "the carters' camp from c-d-main-11"},
             "destination": {"how": "created", "why": "the breach at Breakwall, Breakwall's own landmark made a site"},
             "route": {"how": "created", "why": "the march road, created as a route place"},
             "haven_owner": {"how": "looked up", "why": "holds Breakwall"},
             "guard": {"how": "reused", "why": "on c-d-main-11 (+2), lives at the road fort the road runs past (+1)"}})
c12 = CARDS[-1]
if 2 not in c12["ways_rolled"]: c12["slots"].pop("haven_owner"); c12["cast_log"].pop("haven_owner")
if 3 not in c12["ways_rolled"]: c12["slots"].pop("guard"); c12["cast_log"].pop("guard")
if "haven_owner" not in c12["slots"]:
    # the fault line needs a knower slot on this card: Kotqot stands as the haven at the destination whatever way is rolled
    c12["slots"]["haven_owner"] = "p-kotqot"; c12["cast_log"]["haven_owner"] = {"how": "looked up", "why": "holds Breakwall, the haven (the fault line's knower)"}

# =============================================================== Trials (layer 4, region, the revelation layer), move m4
ch("ch-d-main-4a", node="n-d-main-4a", role="Trials", problem="x-d-main-m4", cards=["c-d-main-13", "c-d-main-14", "c-d-main-15"],
   region="r-crossing", settlement="pl-southroad")
fact("x-d-main-oath13", f"{n('p-shahayuh')} and {n(LT)} have sworn before Southroad's ladder-keeper that a champions' bout at the tavern-yard "
     "ring decides who holds the Great Bridge's toll-house until the court hears his appeal.", kind="past event", when="after m-d-main-4",
     on=["p-shahayuh", LT])
fact("x-d-main-t13", f"{n('p-d-main-paymaster')} has bet the company's own silver on the crown's champion at long odds, and means to pay the "
     "company's arrears from the winnings.", vis="secret", on=["p-d-main-paymaster"])
main("c-d-main-13", "ch-d-main-4a", "opening", "m-d-main-4", "pl-d-main-southroad-ring", [
    W(f"stand in for {given('p-d-main-paidfighter')} against {n('p-d-main-sapper')} and win the bout anyway, against everything the fix arranged", "prepare", "opponent"),
    W(f"make {given('p-d-main-paymaster')}'s bet ride the other way: lay the company's own silver against the crown's champion", "stakes", "fixer"),
    W("expose the fix in front of the tavern-yard crowd, mid-bout", "rules", "ring_keeper", truth=True),
    W(f"turn {n('p-d-main-paidfighter')}, the toll-keeper who took the crown's coin to fall", "edge", "champion", truth=True),
], family="Contest", premise="The fix", scope="region", offered_at="pl-southroad",
   slots={"requester": "p-d-main-tollreeve", "principal_a": "p-shahayuh", "principal_b": LT, "champion": "p-d-main-paidfighter",
          "opponent": "p-d-main-sapper", "fixer": "p-d-main-paymaster", "ring_keeper": "p-d-main-ladderkeeper"},
   opposing_slot="principal_b", reasons={"requester": "a promise", "fixer": "money"},
   truth="x-d-main-t13", truth_row="Contest: the money", shapes=[], twist=None, _confirming=True,
   pressure="Since the crown re-granted the bridge's tolls, the Great Bridge's toll-house has been fought over in the street; whoever wins the bout "
            f"at the tavern-yard ring holds it, and the tolls of the whole Crossing road with it, until the court hears the appeal. "
            f"{n('p-d-main-tollreeve')} knows her lord's champion has been paid to fall.",
   pressure_level=1, separation="exempt (crime)",
   plants=[plant("x-d-main-p-founding", "unique")], depends_on=["x-d-main-m4", "x-d-main-p-faultline"], reads_context=rc("x-d-main-m4"),
   details={"contest_kind": "bout", "terms": "yield", "ring_keeper": "p-d-main-ladderkeeper", "oath": "x-d-main-oath13", "fix_type": "paid fighter",
            "champion": "p-d-main-paidfighter (the requester's side)", "rides_on": "the Great Bridge's toll-house until the appeal",
            "note": "both principals High: champions on both sides (family-contest §8); the fixer is the other side's paymaster"},
   cast_present=[LT], proxies_present=["p-d-main-paymaster", "p-d-main-sapper"], theme="tolls",
   place_tie="the tavern-yard ring where Southroad keeps its ladder, a street from the Great Bridge",
   cast_log={"requester": {"how": "created", "why": "the Great Bridge's toll-reeve: on demand, created, holds the post"},
             "principal_a": {"how": "looked up", "why": "the Great Bridge's lord, who holds Southroad"},
             "principal_b": {"how": "cast", "why": "the antagonist, in person for the crown's claim (met before the climax)"},
             "champion": {"how": "created", "why": "no one fits: a young toll-keeper who fights on the ladder, created"},
             "opponent": {"how": "created", "why": "no crown champion yet: the company's sapper, created (proxy)"},
             "fixer": {"how": "created", "why": "no paymaster yet: created (proxy, the lieutenant's man)"},
             "ring_keeper": {"how": "created", "why": "Southroad's ladder-keeper: an on-demand contact, created, holds the post"},
             })

fact("x-d-main-t14", f"The founding grant is what it looks like: {n('p-gahiru')}'s grant of the bridge-right to {n('p-shahayuh')}, sealed by the "
     "emperor's own hand in the year the bridge was raised.", vis="secret", on=["t-d-main-founding-grant"])
main("c-d-main-14", "ch-d-main-4a", "middle", "m-d-main-4", "pl-d-main-toll-hut", [
    W(f"buy the 'old tally' from {n('p-d-main-oldkeeper')} for a few coppers and say nothing", "trade", "holder"),
    W(f"tell {given('p-d-main-oldkeeper')} what lies under his crock of nails and offer him a fair price", "trade", "holder", truth=True),
    W(f"tell him, and help him sell it to {n('p-shahayuh')} for a share", "earn", "buyer"),
    W("swap the grant for a better crock and a newer tally", "deceive", "target"),
    W(f"take it from the west hut while {given('p-d-main-oldkeeper')} is out at the tolls", "take", "holding_place"),
], family="Acquire", premise="Unknown worth", scope="community", offered_at="pl-southroad",
   slots={"requester": H, "target": "t-d-main-founding-grant", "holder": "p-d-main-oldkeeper", "holding_place": "pl-d-main-toll-hut",
          "buyer": "p-shahayuh"},
   opposing_slot="holder", reasons={"requester": "evidence", "holder": "sentiment (his father kept it)"},
   truth="x-d-main-t14", truth_row="Acquire: as told", shapes=[], twist=None, _confirming=True,
   pressure=f"In the west toll hut of the Great Bridge, under a crock of nails, lies the bridge's founding grant, sealed in the year the bridge was "
            f"raised; {n('p-d-main-oldkeeper')} thinks it an old tally. With crown riders taking the tolls, the company means to clear the hut within the week.",
   pressure_level=2, separation="exempt (acquisition)",
   plants=[plant("x-d-main-p-chainpost", "unique")], depends_on=["x-d-main-m4", "x-d-main-p-founding"], reads_context=rc("x-d-main-m4"),
   details={"holder_state": "unaware", "value": "High (two tiers above the holder)"},
   cast_present=[H], proxies_present=[], theme="seals", place_tie="the west toll hut, where the Great Bridge's keepers have counted coin for forty years",
   cast_log={"requester": {"how": "cast", "why": "the heart"},
             "target": {"how": "created", "why": "the founding grant, created as a document thing"},
             "holder": {"how": "created", "why": "the old toll-keeper: first named in the last card's plant (not a use), created here"},
             "holding_place": {"how": "created", "why": "the west toll hut, a new site by the bridge"},
             "buyer": {"how": "reused", "why": "only if way 2 is rolled: on c-d-main-13 (+2), lord of the bridge (+1 near)"}})
c14 = CARDS[-1]
if 2 not in c14["ways_rolled"]: c14["slots"].pop("buyer"); c14["cast_log"].pop("buyer")

fact("x-d-main-t15", f"{n('p-d-main-sapper')}, the company's sapper, drew the toll-gate's chain-post in the night on {n('p-d-main-paymaster')}'s "
     f"word, so the gate would fall on {n('p-d-main-tollreeve')}, the toll-reeve who would not take crown coin; it fell on {n('p-d-main-hurtcarter')}'s cart instead.",
     vis="secret", on=["p-d-main-sapper"])
fact("x-d-main-toll-gate-fall", f"The Great Bridge's toll-gate came down at first light and crushed {n('p-d-main-hurtcarter')}'s cart, and her leg with it.",
     kind="past event", when="after m-d-main-4")
main("c-d-main-15", "ch-d-main-4a", "turn", "m-d-main-4", "pl-d-main-great-bridge", [
    W("look over the fallen chain-post to see whether it rusted through or was drawn", "examine", "scene", truth=True, pays="x-d-main-p-chainpost"),
    W("question the toll-keepers who were on the bridge before dawn", "question", "scene", truth=True),
    W(f"ask {n('p-d-main-oldkeeper')}, whom the company's men blame, what he saw from the west hut", "question", "red_herring_2", truth=True),
    W("see whose riders take the tolls while the gate lies in the road, and who paid them to wait", "follow", "scene", truth=True),
    W("catch the hand that drew it trying the east gate's post the same way", "provoke", "culprit", truth=True),
], turn=True, family="Investigate", premise="Sabotage", scope="region", offered_at="pl-southroad",
   slots={"requester": "p-shahayuh", "victim": "p-d-main-hurtcarter", "culprit": "p-d-main-sapper", "scene": "pl-d-main-great-bridge",
          "red_herring_1": "p-kodor", "red_herring_2": "p-d-main-oldkeeper", "intended": "p-d-main-tollreeve"},
   opposing_slot="culprit", motive="greed (paid by the company's paymaster)",
   truth="x-d-main-t15", truth_row="Investigate: the story's own", shapes=["the-enemy-did-it", "rigged-to-fall"],
   twist=twist("wrong target", f"The chain-post was drawn to drop the gate on {n('p-d-main-tollreeve')}, the toll-reeve who refused crown coin; it fell on a carter instead.",
               shape="rigged-to-fall"),
   pressure=f"The Great Bridge's toll-gate came down at first light and crushed {n('p-d-main-hurtcarter')}'s cart and her leg with it. The company's "
            f"men say the old keeper left the chain-post loose; {n('p-shahayuh')} says the Counting-House's debt-taker, {n('p-kodor')}, wants the tolls kept flowing to its loan; and while the gate lies in the road, "
            "crown riders take the bridge's tolls.",
   pressure_level=3, separation="n/a (crime)",
   plants=[plant("x-d-main-revelation", "shared", knower="p-shahayuh")],
   depends_on=["x-d-main-m4", "x-d-main-p-chainpost"], reads_context=rc("x-d-main-m4"),
   details={"red_herrings": [
       {"person": "p-kodor", "reason": "benefits from the outcome: the Counting-House lent the company a year's pay against these tolls, and its debt-taker wants them flowing", "clearing_fact": "the debt-taker sat in the Scales Hall all that night over the quarter-day ledgers, as the house's night-clerks swear", "already_blamed": False},
       {"person": "p-d-main-oldkeeper", "reason": "seen near the scene: he sleeps in the west hut", "clearing_fact": "the post was drawn from the east side, where his hut can't see", "already_blamed": True}],
       "arbiter_absent": True, "hard_proof_needed": False, "revelation_knower": "p-shahayuh"},
   ways_removed=[], possible_endings=["confront privately", "blackmail", "cover it up", "let it go", "truth not found"],
   cast_present=[], proxies_present=["p-d-main-sapper"], theme="tolls", place_tie="the Great Bridge, with its lord's name cut into every stone",
   cast_log={"requester": {"how": "reused", "why": "on c-d-main-13 (+2), lord of the bridge (+1 near); the revelation's knower (he holds his own grant in the emperor's true hand)"},
             "victim": {"how": "created", "why": "no one fits: a carter, created"},
             "culprit": {"how": "reused", "why": "on c-d-main-13 (+2), the paymaster's man (+3, linked to someone on the card's chapter)"},
             "scene": {"how": "created", "why": "the Great Bridge, Southroad's landmark made a site"},
             "red_herring_1": {"how": "reused", "why": "the Counting-House's debt-taker: in a live court tension (+2), the house holds the company's loan on these tolls (+3, linked to the move)"},
             "red_herring_2": {"how": "reused", "why": "on c-d-main-14 (+2), sleeps by the bridge (+1)"},
             "intended": {"how": "reused", "why": "on c-d-main-13 (+2), the requester's employee (+3)"}})

ch("ch-d-main-4b", node="n-d-main-4b", role="Trials", problem="x-d-main-m4", cards=["c-d-main-16", "c-d-main-17", "c-d-main-18"],
   region="r-roads", settlement="pl-counting-house")
person("p-d-main-olddrover", "Low", ["the oldest drover on the river"], "pl-highcrossing")
fact("x-d-main-t16", "Each claim holds in part: the meadows' old grant gives the crown their grass in a time of muster, and the drovers have "
     "grazed them by custom for sixty years without one.", vis="secret")
main("c-d-main-16", "ch-d-main-4b", "opening", "m-d-main-4", "pl-d-main-meadows", [
    W(f"walk the meadows' old bounds with {n('p-d-main-olddrover')}, who remembers the last muster", "truth", "oldest_local", truth=True),
    W("prove from the wear on the river banks who has grazed the meadows since the last muster", "truth", "stake", truth=True),
    W("broker a sharing: the company's remounts by day, the drovers' beasts by night", "deal", "party_b"),
    W("find the remounts other grass on the crown road's wide verges", "deal", "party_b"),
    W(f"secure the meadows for {n('p-d-main-drover')}'s drovers before the court", "side", "party_a"),
], family="Dispute", premise="Shared resource", scope="region", offered_at="pl-highcrossing",
   slots={"requester": "p-d-main-drover", "party_a": "p-d-main-drover", "party_b": LT, "stake": "pl-d-main-meadows",
          "oldest_local": "p-d-main-olddrover"},
   opposing_slot="party_b", reasons={"party_a": "survival", "party_b": "pride"},
   truth="x-d-main-t16", truth_row="Dispute: both partly right", shapes=["both-partly-right"], twist=None,
   pressure=f"The Counting-House's advance is paying for the company's remounts, and {n(LT)} has had them turned out on the river meadows below the "
            "Empty Gallery, where the capital's drovers have grazed their beasts for sixty years. The drovers' herds go hungry into autumn, and the "
            "drovers say they'll drive them into the Gallery's own court.",
   pressure_level=1, separation="exempt (shared resource)",
   plants=[plant("x-d-main-p-cousin-hides", "unique")], depends_on=["x-d-main-m4", "x-d-main-p-faultline"], reads_context=rc("x-d-main-m4"),
   details={"player_position": "advocate", "requester_is": "party_a", "arbiter_absent": True,
            "why_no_arbiter": "a party is High and only the emperor outranks her; he doesn't hear grazing quarrels"},
   cast_present=[LT], proxies_present=[], theme="the company", place_tie="the river meadows below the Empty Gallery, at the island's biggest crossing",
   cast_log={"requester": {"how": "created", "why": "no one fits: the drovers' headwoman, created"},
             "party_a": {"how": "created", "why": "the same"}, "party_b": {"how": "cast", "why": "the antagonist, for the crown's claim (met before the climax)"},
             "stake": {"how": "created", "why": "the meadows, a new site at Highcrossing (the Throne's seat appears before the climax)"},
             "oldest_local": {"how": "created", "why": "no one fits: created"}})

person("p-d-main-sweetheart", "Low", ["drover's daughter"], "pl-highcrossing")
fact("x-d-main-t17", f"{given(C)} carries a leaf torn from the sealing office's day-book, the page for the day his father's will was sealed, sewn "
     "into his coat; it is why the clerks turned him out, and why he will not be taken anywhere the sealing office can find him.", vis="secret", on=[C])
main("c-d-main-17", "ch-d-main-4b", "middle", "m-d-main-4", "pl-d-main-crown-road", [
    W(f"earn {given(C)}'s trust on the crown road, a mile at a time", "arrangement", "cargo", truth=True),
    W("keep him under close escort past the sealing office's runners", "force", "cargo"),
    W(f"let {given(C)} choose the road and the pace, and pay for it in days", "route", "cargo", truth=True, price="days on the road"),
    W(f"bring along {n('p-d-main-sweetheart')}, the drover's daughter {given(C)} has been courting on the meadows", "arrangement", "loved_one"),
], family="Passage", premise="Unwilling passenger", scope="community", offered_at="pl-highcrossing",
   slots={"requester": "p-d-main-widow", "cargo": C, "origin": "pl-d-main-meadows", "destination": "pl-crownsgate", "route": "pl-d-main-crown-road",
          "loved_one": "p-d-main-sweetheart"},
   opposing_slot=None, reasons={"cargo": "expects punishment on arrival"},
   truth="x-d-main-t17", truth_row="Passage: not what it seems", shapes=["more-than-said"], twist=None,
   pressure=f"{n('p-d-main-widow')} has come down to the capital to bring her son home; {given(C)} won't go, and the sealing office's runners have "
            "been asking for him along the crown road. Every day he stays on the meadows, they come closer.",
   pressure_level=2, separation="exempt (authority)",
   plants=[plant("x-d-main-p-pledge", "unique")], depends_on=["x-d-main-m4", "x-d-main-p-cousin-hides"], reads_context=rc("x-d-main-m4"),
   details={"requester_link": "parent of the cargo"},
   cast_present=[], proxies_present=[C], theme="kin", place_tie="the crown road from the capital to its gate village",
   cast_log={"requester": {"how": "created", "why": "the cargo's mother: first on a story card here; created with the family link at the spine"},
             "cargo": {"how": "reused", "why": "on c-d-main-1 (+2), son of the requester (+3)"},
             "origin": {"how": "reused", "why": "the meadows from c-d-main-16"},
             "destination": {"how": "reused", "why": "Crownsgate, the map settlement, his home"},
             "route": {"how": "created", "why": "the crown road, created as a route place"},
             "loved_one": {"how": "created", "why": "only if way 3 is rolled: no one fits, created"}})
c17 = CARDS[-1]
if 3 not in c17["ways_rolled"]: c17["slots"].pop("loved_one"); c17["cast_log"].pop("loved_one"); PEOPLE.pop("p-d-main-sweetheart")

fact("x-d-main-t18", f"{n('p-d-main-chclerk')}, the Counting-House clerk who wrote the pledge, saw {n(HD)} sign the Gate Mill's grain over to "
     "the house in his own name, and has told no one.", vis="secret", on=["p-d-main-chclerk"])
fact("x-d-main-betrayal-cousin", f"{n(HD)}, {given(C)}'s master at the sealing office, swore the Gate Mill would be his, then pledged its grain to the "
     "Counting-House against the company's pay and had him turned out of the office.", kind="past event", when="after m-d-main-4")
main("c-d-main-18", "ch-d-main-4b", "turn", "m-d-main-4", "pl-d-main-scales-hall", [
    W(f"expose {given(HD)}'s pledge to the Road Crews and the Crossing millers, whose trust the sealing office still needs", "ruin", "wrongdoer", truth=True),
    W("take back what the betrayal got him: the Gate Mill's grain, before the house pays out on it", "amends", "wrongdoer", pays="x-d-main-p-pledge"),
    W(f"bring {given(C)} and {given(HD)} face to face under the brass scales", "amends", "wronged"),
    W(f"bring it before {n('p-feyesh')} in the Scales Hall", "justice", "arbiter"),
    W(f"meet {given(HD)} on the Silver Road with a cudgel, or in a duel at the Four Ways stone", "violence", "wrongdoer", contest=True),
    W(f"show {n('p-waarua')} what the Counting-House lends the crown, and let {given('p-feyesh')} fear the Open Council", "justice", "feared", source="family rule"),
], turn=True, family="Reckoning", premise="Betrayal", scope="community", offered_at="pl-counting-house",
   slots={"requester": H, "wronged": C, "wrongdoer": HD, "arbiter": "p-feyesh", "feared": "p-waarua", "witness": "p-d-main-chclerk"},
   opposing_slot="wrongdoer", reasons={"requester": "confession"},
   truth="x-d-main-t18", truth_row="Reckoning: a witness exists", shapes=["unheard-witness"],
   twist=twist("the wronged don't want it", f"{given(C)} wants it left alone: the clerk who betrayed him is the sealing office's man, and {given(C)} would sooner lose the mill than be found."),
   pressure=f"On quarter-day {n(HD)} comes to the Scales Hall to draw the company's pay against the Gate Mill's grain. {given(C)}, whom he swore "
            f"the mill to and then turned out of the sealing office, is ruined, and {given(H)}'s mill is pledged to the house by a crown that does not own it.",
   pressure_level=3, separation="exempt",
   plants=[plant("x-d-main-revelation", "shared", knower=C)], depends_on=["x-d-main-m4", "x-d-main-p-pledge"], reads_context=rc("x-d-main-m4"),
   details={"shield": f"Bought: {given('p-feyesh')}, who would judge it, is the clerk's creditor for the company's advance", "demand": "confession",
            "wrong": "x-d-main-betrayal-cousin", "link": "employer/employee (the clerk was his master)", "revelation_knower": C},
   cast_present=[H, HD], proxies_present=[C], theme="debt", place_tie="the Scales Hall, where the Counting-House weighs debts in public on its brass scales",
   cast_log={"requester": {"how": "cast", "why": "the heart, acting for her cousin"},
             "wronged": {"how": "reused", "why": "on c-d-main-1 and c-d-main-17 (+2), the heart's cousin (+3)"},
             "wrongdoer": {"how": "cast", "why": "the Hand"},
             "arbiter": {"how": "looked up", "why": "holds the Counting-House; premade High in an authority slot"},
             "feared": {"how": "reused", "why": "only if the shield way is rolled: the Open Council's head, the one the house fears"},
             "witness": {"how": "created", "why": "the clerk the truth names: no one fits, created"}})
c18 = CARDS[-1]
if 5 not in c18["ways_rolled"]: c18["slots"].pop("feared"); c18["cast_log"].pop("feared")

# =============================================================== Ordeal (layer 5, realm): disgraced (main-quest §9 doors)
ch("ch-d-main-5a", node="n-d-main-5a", role="Ordeal", problem="x-d-main-m5", cards=["c-d-main-19", "c-d-main-20", "c-d-main-21"],
   region="r-vale", settlement="pl-ninth-ring")
fact("x-d-main-t19", f"{n('p-d-main-ferryson')} told the company's clerk that his mother had carried {given(H)}'s letters about the crown seals over "
     "the ford for a season; the writ cites those letters, read in her own hand, as its proof, and she has not spoken to him since.", vis="secret")
fact("x-d-main-tw19", f"{n('p-d-main-rivalferry')} has carried Oathford's ford since {given('p-d-main-ferrywoman')} was shunned for a forger's letters, "
     "and the longer mother and son stay apart, the longer the company's custom is his.", vis="secret", on=["p-d-main-rivalferry"])
main("c-d-main-19", "ch-d-main-5a", "opening", "m-d-main-5", "pl-d-main-ferry", [
    W(f"carry words between {given('p-d-main-ferrywoman')} and {given('p-d-main-ferryson')} across the muster field", "persuade", "estranged_b"),
    W(f"find the thing that matters to both: the ferry-rope {given('p-d-main-ferryson')}'s father spliced", "craft", "estranged_a", truth=True),
    W("arrange a crossing neither can walk out of: the two of them alone on the ferry, mid-river", "persuade", "estranged_a"),
    W("bring in the ford-folk who loved them both", "gather", "requester"),
    W("help them part on decent terms before the column marches", "substitute", "estranged_b"),
], family="Restore", premise="Estrangement", scope="realm", offered_at="pl-oathford",
   slots={"requester": "p-d-main-lad", "estranged_a": "p-d-main-ferrywoman", "estranged_b": "p-d-main-ferryson",
          "profiteer": "p-d-main-rivalferry"},
   opposing_slot=None, reasons={"requester": "loyalty (he works the ferry for the mother and grew up with the son)"},
   truth="x-d-main-t19", truth_row="Restore: a secret that got told", shapes=["secret-told", "still-profiting"],
   twist=twist("someone wants it broken", "x-d-main-tw19", shape="still-profiting"),
   pressure=f"The writ names {given(H)} a forger on the strength of letters {n('p-d-main-ferrywoman')} carried for her across the ford, and it was "
            f"her son {given('p-d-main-ferryson')} who told the company's clerk of them. Mother and son haven't spoken since. He marches with the company's "
            "column at the month's end, and while his word stands, the writ's proof stands in every town it is nailed up in.",
   pressure_level=1, separation="n/a",
   plants=[plant("x-d-main-p-lad-taken", "unique")], depends_on=["x-d-main-m5"], reads_context=rc("x-d-main-m5"),
   details={"cause": "a secret that got told", "hard_part": "a deadline: the column marches at the month's end", "bond": "parent and child"},
   cast_present=[], proxies_present=["p-d-main-ferryson"], theme="the writ", place_tie="the Oathford ferry stair, where the Gate Mill's flour crossed for twenty years",
   cast_log={"requester": {"how": "created", "why": "the ferry-lad: no one fits, created as the ferrywoman's man (he is taken on the next card)"},
             "estranged_a": {"how": "created", "why": "the heart's old friend at the ford: no one fits, created with the friend link"}, "estranged_b": {"how": "created", "why": "her son, created with the family link"},
             "profiteer": {"how": "created", "why": "a rival ferryman: no one fits, created"}})

person("p-d-main-writrider", "Low", ["writ-rider", "wants out"], "pl-oathford")
person("p-d-main-towerguard", "Low", ["writ-rider who keeps the tower door"], "pl-oathford")
fact("x-d-main-t20", f"The band already has a better buyer for {given('p-d-main-lad')} than any writ reward: the overseer of {n('p-d0-bogford-lord')}'s "
     "bond-field will pay for a strong young ferry-lad, and the band means to move him there before the new moon.", vis="secret")
main("c-d-main-20", "ch-d-main-5a", "middle", "m-d-main-5", "pl-d-main-burnt-tower", [
    W(f"fight {n('p-d-main-bandleader')}'s writ-riders at the burned toll-tower", "force", "captor"),
    W(f"slip into the toll-tower at night and bring {given('p-d-main-lad')} out", "stealth", "captive"),
    W(f"bribe {n('p-d-main-towerguard')}, who keeps the tower's door", "price", "guard"),
    W(f"give the band what it took {given('p-d-main-lad')} for", "price", "captor"),
    W(f"turn {n('p-d-main-writrider')}, the rider who wants out, before the band moves the lad", "leverage", "band_member", truth=True),
], family="Rescue", premise="Kidnapped", scope="community", offered_at="pl-oathford",
   slots={"requester": "p-d-main-ferrywoman", "captive": "p-d-main-lad", "captor": "p-d-main-bandleader", "hideout": "pl-d-main-burnt-tower",
          "guard": "p-d-main-towerguard", "band_member": "p-d-main-writrider"},
   opposing_slot="captor", truth="x-d-main-t20", truth_row="Rescue: a better buyer", shapes=["another-buyer"], twist=None,
   pressure=f"Writ-riders took {n('p-d-main-lad')}, {given('p-d-main-ferrywoman')}'s ferry-lad, for hiding {given(H)} in his boat, and hold him at the "
            "burned toll-tower on the Elderhollow road; they say they'll trade him for word of where the forger sleeps. Every ferry family on the river "
            "is watching to see what sheltering her costs.",
   pressure_level=2, separation="exempt (crime)",
   plants=[plant("x-d-main-p-headman-sworn", "unique")], depends_on=["x-d-main-m5", "x-d-main-p-lad-taken"], reads_context=rc("x-d-main-m5"),
   details={"why_taken": "as leverage (over the heart: they'll trade him for word of her)"},
   removed=[{"way": "give the band what it took him for", "reason": "it would hand them the heart: no way may harm the heart (assembly §2)"}],
   cast_present=[], proxies_present=[], theme="the writ", place_tie="the burned toll-tower on the old Elderhollow road",
   cast_log={"requester": {"how": "reused", "why": "on c-d-main-19 (+2), employer of the captive (+3)"},
             "captive": {"how": "reused", "why": "the ferry-lad, requester of c-d-main-19 (+2), her employee (+3)"},
             "captor": {"how": "created", "why": "no lawless band on the Elderhollow road: created"},
             "hideout": {"how": "created", "why": "a ruined tower on the road, created"},
             "guard": {"how": "created", "why": "no one fits: created"}, "band_member": {"how": "created", "why": "no one fits: created"}})
c20 = CARDS[-1]
c20["ways_pool"][3]["removed"] = True
if 3 in c20["ways_rolled"]:
    c20["ways_rolled"] = [i for i in c20["ways_rolled"] if i != 3]
    for j in (1, 0, 2):
        if j not in c20["ways_rolled"]: c20["ways_rolled"] = sorted(c20["ways_rolled"] + [j]); break
if 2 not in c20["ways_rolled"]: c20["slots"].pop("guard"); c20["cast_log"].pop("guard"); PEOPLE.pop("p-d-main-towerguard")

person("p-d-main-host", "Low", ["innkeeper on the Elderhollow road"], "pl-elderhollow")
person("p-d-main-farmer", "Low", ["outlying farmer who knows the Elderhollow farm tracks"], "pl-elderhollow")
place("pl-d-main-vale-road", "the road from Oathford through Elderhollow to the Ninth Ring", "road", None, "open", "Middle", "pl-elderhollow")
fact("x-d-main-t21", f"{n('p-d-main-rivalferry')} sold word of {given(H)}'s road to {n('p-d0-elderhollow-head')}, whose house bars her at Elderhollow's bridge.",
     vis="secret")
main("c-d-main-21", "ch-d-main-5a", "turn", "m-d-main-5", "pl-d-main-archive", [
    W(f"find a host on the Elderhollow road, {n('p-d-main-host')}'s inn perhaps, willing to take a writ-named Qotiro under the roof, and learn what it will cost",
      "arrangement", "host"),
    W(f"go round Elderhollow by the farm tracks {n('p-d-main-farmer')} knows, at the cost of days", "route", "farmer", price="days the cargo may not have"),
    W(f"pass {given(H)} off as a mill-hand bringing flour to the Ninth Ring, risking every host who doesn't know", "cover", "cargo"),
    W("force the bridge at Elderhollow", "force", "headman"),
    W(f"win {n('p-d0-elderhollow-head')} over with something Elderhollow needs", "arrangement", "headman", truth=True, pays="x-d-main-p-headman-sworn"),
], turn=True, family="Passage", premise="Unwelcome cargo", scope="realm", offered_at="pl-oathford",
   slots={"requester": B, "cargo": H, "recipient": "p-tuhudo", "origin": "pl-d-main-ferry", "destination": "pl-d-main-archive",
          "route": "pl-d-main-vale-road", "waypoint": "pl-elderhollow", "headman": "p-d0-elderhollow-head", "host": "p-d-main-host",
          "farmer": "p-d-main-farmer", "left_behind": "p-d-main-millhand"},
   opposing_slot="headman", reasons={"cargo": "under the writ's ban (x)"},
   truth="x-d-main-t21", truth_row="Passage: the pursuer was told", shapes=["sold-from-inside"],
   twist=twist("the one left behind", f"When {given(H)} leaves Oathford, {n('p-d-main-millhand')}, the old mill-hand who came with her, is taken up by the writ's officers in her place, as one who sheltered her."),
   pressure=f"The writ bars {given(H)} from every archive, and the one place her hand and her house's line can be sworn true is the archive of the Ninth "
            f"Ring, where {n('p-tuhudo')} reads old seals by touch. The road runs through Elderhollow, whose ruling house has sworn to bar a Qotiro forger at its "
            "bridge, and anyone who lodges her on the way answers to the writ.",
   pressure_level=3, separation="n/a", door="Disgraced → Passage (Unwelcome cargo)",
   plants=[], depends_on=["x-d-main-m5", "x-d-main-p-headman-sworn", "x-d-main-revelation"], reads_context=rc("x-d-main-m5"),
   details={"ban_fact": "x-d-main-m5", "waypoints": ["pl-elderhollow"]},
   cast_present=[H, B], proxies_present=[], theme="the writ", place_tie="the archive of the Ninth Ring, where every house on the island has its line",
   cast_log={"requester": {"how": "cast", "why": "the heart's fallback, sending her"}, "cargo": {"how": "cast", "why": "the heart, under the ban"},
             "recipient": {"how": "looked up", "why": "the archive's keeper, who holds the Ninth Ring"},
             "origin": {"how": "reused", "why": "the ferry stair from c-d-main-19"},
             "destination": {"how": "created", "why": "the Ninth Ring's archive, made a site"},
             "route": {"how": "created", "why": "the Vale road, created as a route place"},
             "waypoint": {"how": "reused", "why": "Elderhollow, the map town the road runs through"},
             "headman": {"how": "looked up", "why": "the head of Elderhollow's ruling house, who holds the town"},
             "host": {"how": "created", "why": "only if way 0 is rolled: no one fits, created"},
             "farmer": {"how": "created", "why": "only if way 1 is rolled: no one fits, created"},
             "left_behind": {"how": "reused", "why": "on c-d-main-1 (+2), the cargo's employee (+3)"}})
c21 = CARDS[-1]
if 0 not in c21["ways_rolled"]: c21["slots"].pop("host"); c21["cast_log"].pop("host"); PEOPLE.pop("p-d-main-host")
if 1 not in c21["ways_rolled"]: c21["slots"].pop("farmer"); c21["cast_log"].pop("farmer"); PEOPLE.pop("p-d-main-farmer")

ch("ch-d-main-5b", node="n-d-main-5b", role="Ordeal", problem="x-d-main-m5", cards=["c-d-main-22", "c-d-main-23", "c-d-main-24"],
   region="r-harbour", settlement="pl-openharbour")
fact("x-d-main-taking-grant", f"When the writ was read, {n(HD)} took every paper from {given(H)}'s hand at Crownsgate, the village grant among them, and "
     "the sealing office sold the forger's papers off as scrap vellum.", kind="past event", when="after m-d-main-5")
fact("x-d-main-t22", f"The grant {n('p-d-main-paperdealer')} holds is Crownsgate's own copy; the original, sealed when the village gave {given(H)} the "
     "mill, was sent to the Ninth Ring's archive for keeping, as every crown village's grants are.", vis="secret", on=["t-d-main-grant"])
main("c-d-main-22", "ch-d-main-5b", "opening", "m-d-main-5", "pl-d-main-paper-shop", [
    W(f"trace the grant back with {n('p-d-main-paperdealer')} through every hand that sold it, to the sealing office that stripped it", "earn", "holder", truth=True),
    W(f"repay {given('p-d-main-paperdealer')} what the bundle cost him", "trade", "holder"),
    W(f"make {n(HD)}, who took it, buy it back from the dealer", "earn", "original_taker"),
    W("swap the grant for a scrap of vellum with the same seal-tag", "deceive", "target", truth=True),
    W("take it from the salt-quay shop", "take", "holding_place"),
], family="Acquire", premise="Passed on", scope="realm", offered_at="pl-saltmarket",
   slots={"requester": B, "target": "t-d-main-grant", "holder": "p-d-main-paperdealer", "holding_place": "pl-d-main-paper-shop",
          "original_taker": HD},
   opposing_slot="holder", reasons={"requester": "legacy", "holder": "livelihood"},
   truth="x-d-main-t22", truth_row="Acquire: a copy", shapes=["a-copy"], twist=None,
   pressure=f"The writ's officers stripped every paper from {given(H)}'s hand and the sealing office sold them on as scrap. The village grant that names "
            "her the Gate Mill's miller now lies in a Saltmarket paper-dealer's shop, bought fair by the bundle; without it she cannot answer the writ in any "
            "court, and the same is happening to the papers of everyone the writ names, in every Great House town.",
   pressure_level=1, separation="exempt (acquisition)",
   plants=[plant("x-d-main-p-clearances", "unique")], depends_on=["x-d-main-m5"], reads_context=rc("x-d-main-m5"),
   details={"past_taking": "x-d-main-taking-grant", "original_taker_alive": True},
   cast_present=[B, HD], proxies_present=[], theme="the writ", place_tie="the salt quay, where Saltmarket's hauling trade changes hands",
   cast_log={"requester": {"how": "cast", "why": "the heart's fallback, in her place (the grant is hers)"},
             "target": {"how": "created", "why": "the village grant, created as a document thing"},
             "holder": {"how": "created", "why": "no one fits: a paper-dealer, created"},
             "holding_place": {"how": "created", "why": "his shop, a new site"}, "original_taker": {"how": "cast", "why": "the Hand"}})

fact("x-d-main-t23", f"{n('p-d-main-apprentice')}, the notary's apprentice, copied the clearances from a letter in {n(LI)}'s hand, and has told no one.",
     vis="secret", on=["p-d-main-apprentice"])
fact("x-d-main-tw23", f"{given('p-d-main-notary')} sells the clearances on {n(LI)}'s orders, and the silver goes to the company's chest.", vis="secret")
fact("x-d-main-swindle-clear", f"{n('p-d-main-notary')} sold 'clearances' of the writ to salt-carters, ferrymen and lodging-keepers who had carried for "
     f"{given(H)}, at a month's wage each; the writ's officers tore them up at the first gate.", kind="past event", when="after m-d-main-5")
link("p-d-main-notary", LI, "employer/employee", "employee", "employer", vis="secret", when="after m-d-main-5")
main("c-d-main-23", "ch-d-main-5b", "middle", "m-d-main-5", "pl-d-main-notary-stall", [
    W(f"con {given('p-d-main-notary')} with a clearance of her own to sign, made out to the crown factor", "ruin", "wrongdoer"),
    W(f"find the others {given('p-d-main-notary')} sold to along the salt road and come to her stall together", "ruin", "wronged", truth=True),
    W(f"make {given('p-d-main-notary')} repay before {n('p-d0-saltmarket-head')} and the Saltmarket council, whom she can't afford to ignore", "amends", "council"),
    W(f"frighten {given('p-d-main-notary')} off the salt road before she sells to Openharbour's carriers", "violence", "wrongdoer"),
    W("warn the carriers of the salt road and dry up the notary's trade before it reaches the next town", "ruin", "wrongdoer", source="family rule"),
], family="Reckoning", premise="Swindle", scope="community", offered_at="pl-saltmarket",
   slots={"requester": "p-d-main-saltcarter", "wronged": "p-d-main-saltcarter", "wrongdoer": "p-d-main-notary", "head": LI,
          "council": "p-d0-saltmarket-head"},
   opposing_slot="wrongdoer", reasons={"requester": "repayment"},
   truth="x-d-main-t23", truth_row="Reckoning: a witness exists", shapes=["unheard-witness", "someone-above"],
   twist=twist("hand not head", "x-d-main-tw23", shape="someone-above"),
   pressure=f"{n('p-d-main-notary')} sold 'clearances' of the writ to the salt-carters, ferrymen and lodging-keepers who had carried for {given(H)}, at a "
            "month's wage each; the writ's officers tore them up at the first gate. The silver is gone, and the notary is already selling to the next town's carriers.",
   pressure_level=2, separation="exempt (crime)",
   plants=[plant("x-d-main-p-factor-stall", "unique")], depends_on=["x-d-main-m5", "x-d-main-p-clearances"], reads_context=rc("x-d-main-m5"),
   details={"shield": "No law against it: a paper that promises what it can't keep is no crime under crown law", "demand": "repayment",
            "wrong": "x-d-main-swindle-clear", "head": LI},
   cast_present=[LI], proxies_present=["p-d-main-notary"], theme="the writ", place_tie="the salt-hall arches, where Saltmarket's notaries write for the quay",
   cast_log={"requester": {"how": "created", "why": "no one fits: a salt-carter named in the writ's lists, created"},
             "wronged": {"how": "created", "why": "the same"},
             "wrongdoer": {"how": "created", "why": "first named in the last card's plant (not a use): created"},
             "head": {"how": "cast", "why": "the lieutenant, the hand behind the notary"},
             "council": {"how": "reused", "why": "only if way 2 is rolled: heads the Saltmarket council (lives at the card's place, +1), same region (+2)"}})
c23 = CARDS[-1]
if 2 not in c23["ways_rolled"]: c23["slots"].pop("council"); c23["cast_log"].pop("council")

fact("x-d-main-offense", f"On the Unchained Quay, before the market crowd, {n(H)} called the crown factor {n('p-d-main-factor')}'s seals forgeries "
     "pressed in another's name.", kind="past event", when="after m-d-main-5")
fact("x-d-main-t24", f"Nothing settles what {given(H)} said on the quay: the crowd's tellings disagree, and the market clerk who wrote the day's "
     "quarrels lost the page to the tide.", vis="secret")
main("c-d-main-24", "ch-d-main-5b", "turn", "m-d-main-5", "pl-d-main-quay", [
    W("find out from the quay crowd what was really said", "truth", "party_a", truth=True),
    W(f"make mercy cheaper than punishment for {given('p-d-main-factor')}: find what the factor gains by letting it drop, and what the notary's stamp costs him if it doesn't",
      "deal", "party_b", pays="x-d-main-p-factor-stall"),
    W("arrange a public apology, or a payment, on the Unchained Quay", "deal", "party_a"),
    W(f"get {given(H)} out of {given('p-d-main-factor')}'s reach: lodge her in the Road Crews' yards, or find a patron who outranks a crown factor", "side", "party_a"),
], turn=True, family="Dispute", premise="Public offense", scope="realm", offered_at="pl-openharbour",
   slots={"requester": H, "party_a": H, "party_b": "p-d-main-factor", "arbiter": "p-waarua"},
   opposing_slot="party_b", reasons={"party_a": "survival", "party_b": "pride"},
   truth="x-d-main-t24", truth_row="Dispute: no record", shapes=["no-record"],
   twist=twist("law on the wrong side", f"{given('p-d-main-factor')} has the lawful claim (the writ makes her words a crime in any market that honours the crown) and wants his pride; "
               f"{given(H)}, who is legally wrong, has her living and her name at stake."),
   pressure=f"{n('p-d-main-factor')}, the crown factor on the Unchained Quay, demands that {given(H)} answer before the Open Council's market court for "
            "calling his seals false in the open market, under a writ that names her the forger. If she's found against, every market that honours the "
            "crown's seal will turn her away, and Openharbour's own will have honoured the writ.",
   pressure_level=3, separation="exempt (public offense)", door="Disgraced → Dispute (Public offense)",
   plants=[], depends_on=["x-d-main-m5", "x-d-main-p-factor-stall", "x-d-main-revelation"], reads_context=rc("x-d-main-m5"),
   details={"player_position": "advocate", "requester_is": "party_a", "offense": "x-d-main-offense"},
   cast_present=[H], proxies_present=["p-d-main-factor"], theme="the writ", place_tie="the Unchained Quay, railed with melted shackles, where Openharbour trades",
   cast_log={"requester": {"how": "cast", "why": "the heart"}, "party_a": {"how": "cast", "why": "the heart"},
             "party_b": {"how": "created", "why": "the crown factor at the port: no one fits, created (proxy, the Hand's man)"},
             "arbiter": {"how": "looked up", "why": "the Open Council's head, who holds Openharbour; premade High in an authority slot"}})

# =============================================================== Climax (layer 6, realm): the last move
ch("ch-d-main-6", node="n-d-main-6", role="Climax", problem="x-d-main-m6", cards=["c-d-main-25", "c-d-main-26", "c-d-main-27"],
   region="r-crossing", settlement="pl-highcrossing")
fact("x-d-main-t25", f"The cough came in with the remounts {n('p-d-main-remounter')} sold the company out of the Marches, bought cheap from a bog "
     "farm where it had already killed half a herd.", vis="secret", on=["p-d-main-remounter"])
main("c-d-main-25", "ch-d-main-6", "opening", "m-d-main-6", "pl-d-main-muster", [
    W("trace the cough back along the remount lines to the Marches farm it came from", "undermine", "source_owner", truth=True),
    W(f"talk {n('p-d-main-remounter')} into taking his remounts off the muster field", "bargain", "source_owner"),
    W("pen the coughing beasts apart and find ford-folk willing to tend them", "protect", "requester"),
    W(f"cull the sick remounts on the muster field and face {given('p-d-main-remounter')}, who loses the price", "confront", "source_owner"),
], family="Threat", premise="Sickness or blight", scope="realm", offered_at="pl-oathford",
   slots={"requester": "p-d-main-fordwarden", "community": "pl-d-main-muster", "source_owner": "p-d-main-remounter"},
   opposing_slot="source_owner", truth="x-d-main-t25", truth_row="Threat: a sick animal traded in", shapes=["brought-in"], twist=None,
   pressure="Since the company was named the emperor's own guard, its columns muster at Oathford before marching out on every road from Highcrossing, "
            "and a cough has gone through their remounts and into Oathford's own oxen and draught horses. It travels with every column, and the ford-folk's "
            "teams are dying in their stalls.",
   pressure_level=1, separation="n/a",
   plants=[plant("x-d-main-p-short-guard", "unique")], depends_on=["x-d-main-m6"], reads_context=rc("x-d-main-m6"),
   details={"source": "a sick animal traded in", "never_deliberate": True},
   cast_present=[], proxies_present=[], theme="the company", place_tie="the muster field below the lord's tower, where the river road meets the ford",
   cast_log={"requester": {"how": "created", "why": "the head of Oathford's ford-wardens: no one fits, created, holds the post"},
             "community": {"how": "created", "why": "the muster field, a new site"},
             "source_owner": {"how": "created", "why": "a Marches horse-dealer: no one fits, created"}})

fact("x-d-main-t26", "The tenth is real and fair: every Crownsgate grant was a crown grant with a tenth written into its seal-clause, and none of them "
     "has paid it in forty years.", vis="secret")
main("c-d-main-26", "ch-d-main-6", "middle", "m-d-main-6", "pl-d-main-gate-house", [
    W("pay the grant-holders' tenth into the company's chest", "price", "captor"),
    W(f"work off the tenth in their place, hauling the company's grain to Highcrossing", "price", "captive"),
    W("steal the company's tenth-ledger from the gate-house strongbox", "stealth", "ledger", truth=True),
    W("find the real debtor", "leverage", "captive"),
    W("prove the debt was paid or inflated, through a witness", "leverage", "captive"),
], family="Rescue", premise="Bound by debt", scope="community", offered_at="pl-crownsgate",
   slots={"requester": DU, "captive": "p-d-main-bridgekeeper", "captor": LI, "place": "pl-d-main-gate-house", "ledger": "t-d-main-ledger"},
   opposing_slot="captor", truth="x-d-main-t26", truth_row="Rescue: real and fair", shapes=["held-rightly"], twist=None,
   pressure=f"The company holds {n('p-d-main-bridgekeeper')} and Crownsgate's other grant-holders in the old gate-house, fed on their own seized grain, "
            f"until each swears his grant over to the crown or pays forty years of tenth. {n(DU)}, whose gate-house it is, has the keys the company didn't think to ask for.",
   pressure_level=2, separation="applies (the captor's side and the captives' representative are a tier apart: High through his men, Middle)",
   plants=[plant("x-d-main-p-oath-name", "unique")], depends_on=["x-d-main-m6", "x-d-main-p-short-guard"], reads_context=rc("x-d-main-m6"),
   details={"captives": "Crownsgate's grant-holders, through their representative", "debt": "the crown's tenth, forty years unpaid"},
   removed=[{"way": "find the real debtor", "reason": "requires truth = misassigned"}, {"way": "prove the debt was paid or inflated", "reason": "requires truth = paid or inflated"}],
   cast_present=[LI], proxies_present=[DU], theme="the tenth", place_tie="the crown gate-house, where the company keeps the village it took",
   cast_log={"requester": {"how": "reused", "why": "on c-d-main-1 (+2), owns the gate-house (+1); met on the hook, so on every path"},
             "captive": {"how": "reused", "why": "on c-d-main-2 (+2), lives at the card's place (+1); met on the hook, so on every path"},
             "captor": {"how": "cast", "why": "the lieutenant (main-quest §9: on a non-turn climax card)"},
             "place": {"how": "reused", "why": "the gate-house from c-d-main-2 (the hook's landmark)"},
             "ledger": {"how": "created", "why": "the tenth-ledger, created as a document thing (only if way 2 is rolled)"}})
c26 = CARDS[-1]
for j in (3, 4): c26["ways_pool"][j]["removed"] = True
c26["ways_rolled"] = [i for i in c26["ways_rolled"] if i not in (3, 4)]
if len(c26["ways_rolled"]) < 2 or len({c26["ways_pool"][i]["kind"] for i in c26["ways_rolled"]}) < 2:
    c26["ways_rolled"] = sorted(set(c26["ways_rolled"]) | {2, 0})
if 2 not in c26["ways_rolled"]: c26["slots"].pop("ledger"); c26["cast_log"].pop("ledger")

fact("x-d-main-t27", f"The wrong is as {given(B)} tells it: the writ that named his sister a forger, and the seizure that took her mill as a forger's "
     f"goods, were both sealed at {n(LT)}'s word.", vis="secret")
fact("x-d-main-wrong27", f"{n(LT)} had {n(H)} named a forger by imperial writ, and her mill seized as a forger's kin's.", kind="past event",
     when="after m-d-main-6")
pool27 = [
    W(f"turn {given(LT)}'s own servants: {n(DU)}, who has kept the emperor's gate forty years and says the new seal is not his", "ruin", "servant",
      pays="x-d-main-p-faultline"),
    W(f"shame {given(LT)} before the court in the Empty Gallery, among the empty plinths, where every lord can see", "ruin", "wrongdoer"),
    W(f"force amends with what she can least afford said aloud: that she seals in her brother's name because his hands can no longer hold the seal",
      "amends", "wrongdoer", truth=True, pays="x-d-main-revelation"),
    W(f"break the company she built from it, at the Gallery's doors", "violence", "wrongdoer"),
    W(f"take it to the one {given(LT)} still answers to: {n('p-gahiru')}, her brother, whose seal she presses", "justice", "answers_to", source="family rule"),
    W(f"call {n(GEN)}'s veterans down from Longwall to the Gallery's doors, at his price", "violence", "ally", source="ally", forced=True,
      price="the company's arms and the bondsmen's oaths go to Longwall once it is broken", pays="x-d-main-p-ally"),
    W(f"give the Gate Mill to the crown for good, in exchange for the writ torn up in the Empty Gallery", "amends", "stake", source="sacrifice", forced=True),
]
main("c-d-main-27", "ch-d-main-6", "turn", "m-d-main-6", "pl-d-main-gallery", pool27, climax=True,
   family="Reckoning", premise="Ruin from above", scope="realm", offered_at="pl-highcrossing",
   slots={"requester": B, "wronged": H, "wrongdoer": LT, "servant": DU, "ally": GEN, "stake": "pl-d-main-gate-mill"},
   opposing_slot="wrongdoer", reasons={"requester": "ruin"},
   truth="x-d-main-t27", truth_row="Reckoning: as told", shapes=[],
   twist=twist("the wronged don't want it", f"{given(H)} wants her mill and her name, not the emperor's sister brought down in the Empty Gallery: she has seen "
               "what the general would do with the company's arms, and what Breakwall would do with the proof."),
   pressure=f"The company is named the emperor's own guard, Crownsgate is the crown's, and the writ still hangs in every Great House town; {n(LT)} holds "
            "court in the Empty Gallery with the company at its doors. Whatever is to be answered for is answered here, before the court she keeps, or not at all.",
   pressure_level=3, separation="exempt (authority)",
   plants=[], depends_on=["x-d-main-m5", "x-d-main-m6", "x-d-main-revelation", "x-d-main-p-faultline", "x-d-main-p-ally", "x-d-main-p-oath-name"],
   reads_context=rc("x-d-main-m6"),
   details={"shield": "Rank: only the emperor outranks her, and he doesn't hear such matters", "demand": "ruin", "wrong": "x-d-main-wrong27",
            "payoffs": ["x-d-main-p-faultline (layer 3)", "x-d-main-p-ally (layer 3)", "x-d-main-revelation (layer 4)", "x-d-main-m5 (layer 5)"],
            "sacrifice_way": 6, "ally_way": 5, "overflow": "forced ally and sacrifice ways plus two plant-backed ways: a 4th way (rulebook, Forced ways overflow)"},
   cast_present=[H, B, LT], proxies_present=[DU], theme="the Throne", place_tie="the Empty Gallery, where the provinces' plinths stand bare",
   cast_log={"requester": {"how": "cast", "why": "the heart's fallback, acting for her"}, "wronged": {"how": "cast", "why": "the heart"},
             "wrongdoer": {"how": "cast", "why": "the antagonist"},
             "servant": {"how": "reused", "why": "the fault line, met on the hook (every path)"},
             "ally": {"how": "reused", "why": "the planted ally, met on the hook (every path)"},
             "stake": {"how": "reused", "why": "the Gate Mill, the stake"}})

# =============================================================== Return (layer 7: the heart's home community)
ch("ch-d-main-ret", node="n-d-main-ret", role="Return", problem="x-d-main-m6", cards=["c-d-main-28", "c-d-main-29"], region="r-crossing",
   settlement="pl-crownsgate")
person("p-d-main-bridgedaughter", "Low", ["the bridge-keeper's daughter"], "pl-crownsgate")
link("p-d-main-bridgekeeper", "p-d-main-bridgedaughter", "family", "parent", "child")
place("pl-d-main-river-road", "the river road from Oathford to Crownsgate", "road", None, "open", "Middle", "pl-oathford")
fact("x-d-main-t28", f"{given('p-d-main-bridgeson')} carries the crown's tenth-roll of every oath Crownsgate swore, taken from the muster clerk's "
     "chest the night he decided to come home; it is why he won't be seen on the road.", vis="secret", on=["p-d-main-bridgeson"])
main("c-d-main-28", "ch-d-main-ret", "opening", "m-d-main-6", "pl-d-main-toll-bridge", [
    W(f"win {given('p-d-main-bridgeson')} round on the river road, one milestone at a time", "arrangement", "cargo", truth=True),
    W("keep the boy under close escort past the company's pickets", "force", "cargo"),
    W(f"let {given('p-d-main-bridgeson')} choose the road home and pay for it in days", "route", "cargo", truth=True, price="days on the road"),
    W(f"bring along {n('p-d-main-bridgedaughter')}, the sister he hasn't seen since he swore", "arrangement", "loved_one"),
], family="Passage", premise="Unwilling passenger", scope="household", offered_at="pl-crownsgate",
   slots={"requester": "p-d-main-bridgekeeper", "cargo": "p-d-main-bridgeson", "origin": "pl-d-main-muster", "destination": "pl-d-main-toll-bridge",
          "route": "pl-d-main-river-road", "left_behind": "p-d-main-sergeant", "loved_one": "p-d-main-bridgedaughter"},
   opposing_slot=None, reasons={"cargo": "expects punishment on arrival"},
   truth="x-d-main-t28", truth_row="Passage: not what it seems", shapes=["more-than-said"],
   twist=twist("the one left behind", f"When {given('p-d-main-bridgeson')} leaves the muster, {n('p-d-main-sergeant')}, the sergeant who vouched for him, is flogged for the boy's going."),
   pressure=f"Since the company took Crownsgate, {n('p-d-main-bridgekeeper')}'s son {given('p-d-main-bridgeson')} has worn its tabard at the Oathford muster. "
            "His father wants him home across the toll bridge before the company marches him away; the boy will not come, for shame of what he swore.",
   pressure_level=1, separation="exempt (authority)",
   plants=[plant("x-d-main-p-name-read", "unique")], depends_on=["x-d-main-m6"], reads_context=rc("x-d-main-m6"),
   details={"requester_link": "parent of the cargo"},
   cast_present=[], proxies_present=["p-d-main-bridgeson", "p-d-main-sergeant"], theme="kin", place_tie="the toll bridge that Crownsgate is named for",
   cast_log={"requester": {"how": "reused", "why": "Return casting: on c-d-main-2 and c-d-main-26 (+2), the cargo's father (+3)"},
             "cargo": {"how": "created", "why": "Return casting: no shown person can be the bridge-keeper's son; created with the family link"},
             "origin": {"how": "reused", "why": "the muster field from c-d-main-25"},
             "destination": {"how": "created", "why": "the toll bridge, Crownsgate's own landmark made a site (Crownsgate itself is already a Passage destination, c-d-main-17)"},
             "route": {"how": "created", "why": "the river road, created as a route place"},
             "left_behind": {"how": "reused", "why": "Return casting: on c-d-main-3 (+2), the cargo's sergeant (+3)"},
             "loved_one": {"how": "created", "why": "only if way 3 is rolled: Return casting found no shown person, created"}})
c28 = CARDS[-1]
if 3 not in c28["ways_rolled"]: c28["slots"].pop("loved_one"); c28["cast_log"].pop("loved_one"); PEOPLE.pop("p-d-main-bridgedaughter")

fact("x-d-main-t29", f"The stain is false and was made so on purpose, off this card: {n(LT)} had the writ sealed against {given(H)} because hers was "
     "the hand setting the new seals beside the old.", vis="secret")
fact("x-d-main-tw29", f"{n(LI)} holds Crownsgate and the Gate Mill for the crown on the strength of 'a forger's kin'; the longer the stain stays on "
     f"{given(H)}, the longer his company's hold on the village stands.", vis="secret", on=[LI])
main("c-d-main-29", "ch-d-main-ret", "turn", "m-d-main-6", "pl-d-main-gate-house", [
    W("gather the grant-holders who were held in the gate-house and have them say aloud what was really sealed there", "gather", "subject", truth=True),
    W(f"stage a public act at the toll bridge that shows the village who {given(H)} is now", "craft", "subject"),
    W(f"get {n(GEN)} to vouch for her at the gate-house door", "persuade", "voucher"),
    W(f"help {given(H)} start fresh where no one has read the writ", "substitute", "subject"),
], turn=True, family="Restore", premise="Lost good name", scope="community", offered_at="pl-crownsgate",
   slots={"requester": H, "subject": H, "profiteer": LI, "voucher": GEN},
   opposing_slot=None, reasons={"requester": "pride"},
   truth="x-d-main-t29", truth_row="Restore: false, spread by someone off the card", shapes=["rumour-planted", "still-profiting"],
   twist=twist("someone wants it broken", "x-d-main-tw29", shape="still-profiting"),
   pressure=f"Whatever was decided in the Empty Gallery, Crownsgate still calls its miller a forger: the company's clerks read her name at every muster, "
            "the gate-house still has the writ on its door, and no one in the village will buy flour from the Gate Mill while her name is on it.",
   pressure_level=2, separation="n/a",
   plants=[], depends_on=["x-d-main-m6", "x-d-main-p-name-read", "x-d-main-revelation", "x-d-main-m5"], reads_context=rc("x-d-main-m6"),
   details={"cause": "false, spread by someone off the card (the writ; Return use, main-quest §9)", "hard_part": "pride"},
   removed=[{"way": "find where the story started and answer it at the source", "reason": "cause is 'spread off the card'"}],
   cast_present=[H, LI], proxies_present=[], theme="good name", place_tie="the crown gate-house, the hook's landmark, in the state the company left it",
   cast_log={"requester": {"how": "cast", "why": "the heart"}, "subject": {"how": "cast", "why": "the heart"},
             "profiteer": {"how": "cast", "why": "the lieutenant, who holds the village"},
             "voucher": {"how": "reused", "why": "only if way 2 is rolled: Return casting, the general, shown on every path"}})
c29 = CARDS[-1]
if 2 not in c29["ways_rolled"]: c29["slots"].pop("voucher"); c29["cast_log"].pop("voucher")
