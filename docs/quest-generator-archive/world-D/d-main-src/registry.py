"""Registry for the realm layer: created people, sites, things, the spine facts and the moves."""
from lib import *
import rolls as RO

N = name
# ---------------------------------------------------------------- the Gate Mill's people (Crownsgate)
person("p-d-main-heart", "Middle", ["miller of the Gate Mill", "earlier-wave Qotiro"], "pl-crownsgate", fallback="p-d-main-brother")
person("p-d-main-uncle", "Middle", ["the old miller of the Gate Mill"], "pl-crownsgate", dead=True)
person("p-d-main-brother", "Middle", ["carter", "Road Crews carter who hauls the Gate Mill's flour"], "pl-crownsgate",
       faction={"id": "f-open-hand", "sub": "f-road-crews"})
person("p-d-main-cousin", "Middle", ["under-clerk of the crown sealing office", "owes money"], "pl-highcrossing",
       faction={"id": "f-great-house", "sub": "f-throne"})
person("p-d-main-widow", "Middle", ["the old miller's widow"], "pl-crownsgate")
person("p-d-main-millhand", "Low", ["mill-hand", "lives in the mill loft"], "pl-crownsgate")
person("p-d-main-hand", "Middle", ["crown clerk of the sealing office", "holds legal authority by crown warrant"], "pl-highcrossing",
       faction={"id": "f-great-house", "sub": "f-throne"})
person("p-d-main-bridgekeeper", "Middle", ["keeps the toll bridge for its grant-holders"], "pl-crownsgate")
person("p-d-main-bridgeson", "Low", ["the bridge-keeper's son"], "pl-crownsgate")
person("p-d-main-sisterinlaw", "Low", ["carter's wife"], "pl-crownsgate")
person("p-d-main-sergeant", "Middle", ["sergeant of the emperor's new company", "holds legal authority by crown warrant"], "pl-oathford",
       faction={"id": "f-great-house", "sub": "f-throne"})
link("p-d-main-heart", "p-d-main-brother", "family", "sibling", "sibling")
link("p-d-main-uncle", "p-d-main-cousin", "family", "parent", "child")
link("p-d-main-widow", "p-d-main-cousin", "family", "parent", "child")
link("p-d-main-uncle", "p-d-main-widow", "spouse", "spouse", "spouse")
link("p-d-main-heart", "p-d-main-uncle", "kin", "niece", "uncle")
link("p-d-main-heart", "p-d-main-cousin", "kin", "cousin", "cousin")
link("p-d-main-heart", "p-d-main-widow", "kin", "niece", "aunt")
link("p-d-main-heart", "p-d-main-uncle", "family", "heir", "deceased", note="named the Gate Mill's heir in the village grant")
link("p-d-main-heart", "p-d-main-millhand", "employer/employee", "employer", "employee")
link("p-d-main-brother", "p-d-main-sisterinlaw", "spouse", "spouse", "spouse")
link("p-d-main-bridgekeeper", "p-d-main-bridgeson", "family", "parent", "child")
link("p-d-main-hand", "p-litita", "employer/employee", "employee", "employer", note="a clerk of the court she keeps")
link("p-d-main-hand", "p-d-main-cousin", "employer/employee", "employer", "employee")
link("p-d-main-hand", "p-d-main-cousin", "creditor/debtor", "creditor", "debtor", note="the cousin's dice debts")
link("p-d0-oathford-lord", "p-litita", "employer/employee", "employee", "employer", when="at m-d-main-3",
     note="the Throne's captain of the emperor's new company (made public by m-d-main-3)")
link("p-d-main-sergeant", "p-d0-oathford-lord", "employer/employee", "employee", "employer")
# ---------------------------------------------------------------- the roads (threshold)
person("p-d-main-carter", "Low", ["Road Crews carter"], "pl-wainyard", faction={"id": "f-open-hand", "sub": "f-road-crews"})
link("p-d-main-carter", "p-d0-wainyard-head", "employer/employee", "employee", "employer")
# ---------------------------------------------------------------- trials 3
person("p-d-main-stallwife", "Low", ["keeps the pilgrim stalls at the Broken Shrine"], "pr-shrine-ruin")
person("p-d-main-towerwarden", "Middle", ["warden of Longwall's outer towers"], "pl-longwall", faction={"id": "f-great-house", "sub": "f-banner-lords"})
link("p-d-main-towerwarden", "p-haahoqo", "employer/employee", "employee", "employer")
person("p-d-main-crossingkeeper", "Middle", ["takes a coin a cart at the Reed Crossing's plank causeway"], "pr-reed-crossing")
person("p-d-main-raider", "Middle", ["lawless", "leads the Reedbank band"], "pr-raid-camp", faction={"id": "f-binders", "sub": None})
person("p-d-main-informer", "Low", ["carter", "has a secret"], "pl-carters-rest")
person("p-d-main-fortsergeant", "Middle", ["Lantern sergeant at the Brennock road fort"], "pr-road-fort", faction={"id": "f-lantern", "sub": None})
person("p-d-main-deserter", "Low", ["bondsman pressed into the emperor's new company", "deserter"], "pl-bogford",
       faction={"id": "f-great-house", "sub": "f-throne"})
link("p-d-main-deserter", "p-d0-oathford-lord", "employer/employee", "employee", "employer", note="pressed into the company")
# ---------------------------------------------------------------- trials 4
person("p-d-main-tollreeve", "Middle", ["toll-reeve of the Great Bridge"], "pl-southroad", faction={"id": "f-great-house", "sub": "f-banner-lords"})
link("p-d-main-tollreeve", "p-shahayuh", "employer/employee", "employee", "employer")
person("p-d-main-paymaster", "Middle", ["paymaster of the emperor's new company"], "pl-longwall", faction={"id": "f-great-house", "sub": "f-throne"})
link("p-d-main-paymaster", "p-d0-oathford-lord", "employer/employee", "employee", "employer")
person("p-d-main-ladderkeeper", "Middle", ["ladder-keeper", "keeps the town's ranking list at the tavern-yard ring"], "pl-southroad",
       faction={"id": "f-dueling", "sub": None}, cls="ladder-keeper")
person("p-d-main-oldkeeper", "Low", ["old toll-keeper at the Great Bridge"], "pl-southroad")
person("p-d-main-paidfighter", "Low", ["young toll-keeper", "fights on the town's ladder"], "pl-southroad")
link("p-d-main-oldkeeper", "p-d-main-paidfighter", "family", "parent", "child")
link("p-d-main-paidfighter", "p-d-main-tollreeve", "employer/employee", "employee", "employer")
person("p-d-main-sapper", "Middle", ["sapper of the emperor's new company"], "pl-longwall", faction={"id": "f-great-house", "sub": "f-throne"})
link("p-d-main-sapper", "p-d0-oathford-lord", "employer/employee", "employee", "employer")
person("p-d-main-hurtcarter", "Low", ["carter"], "pl-southroad")
person("p-d-main-drover", "Low", ["headwoman of the drovers who graze the river meadows"], "pl-highcrossing")
person("p-d-main-chclerk", "Low", ["clerk of the Counting-House"], "pl-counting-house")
link("p-d-main-chclerk", "p-feyesh", "employer/employee", "employee", "employer")
link("p-d-main-hand", "p-feyesh", "creditor/debtor", "debtor", "creditor", when="at m-d-main-4", note="the company's advance, drawn in the clerk's name")
# ---------------------------------------------------------------- ordeal
person("p-d-main-ferrywoman", "Middle", ["keeps the Oathford ferry"], "pl-oathford")
person("p-d-main-ferryson", "Low", ["carter", "hired to the emperor's new company"], "pl-oathford", faction={"id": "f-great-house", "sub": "f-throne"})
link("p-d-main-ferrywoman", "p-d-main-ferryson", "family", "parent", "child")
link("p-d-main-ferrywoman", "p-d-main-heart", "ally", "friend", "friend", note="twenty years of the Gate Mill's flour over the ford")
person("p-d-main-rivalferry", "Middle", ["ferryman"], "pl-oathford")
link("p-d-main-rivalferry", "p-d-main-ferrywoman", "rival", "rival", "rival")
person("p-d-main-lad", "Low", ["ferry-lad"], "pl-oathford")
link("p-d-main-lad", "p-d-main-ferrywoman", "employer/employee", "employee", "employer")
person("p-d-main-bandleader", "Middle", ["lawless", "leads a band of writ-riders"], "pl-oathford")
person("p-d-main-paperdealer", "Middle", ["dealer in old papers and vellum"], "pl-saltmarket")
person("p-d-main-notary", "Middle", ["notary"], "pl-saltmarket")
person("p-d-main-saltcarter", "Low", ["salt-carter"], "pl-saltmarket")
person("p-d-main-apprentice", "Low", ["notary's apprentice"], "pl-saltmarket")
link("p-d-main-apprentice", "p-d-main-notary", "employer/employee", "employee", "employer")
person("p-d-main-factor", "Middle", ["crown factor at the Unchained Quay"], "pl-openharbour", faction={"id": "f-great-house", "sub": "f-throne"})
link("p-d-main-factor", "p-d-main-hand", "employer/employee", "employee", "employer", note="the sealing office's man at the port")
link("p-d-main-notary", "p-d-main-factor", "trade", None, None)
# ---------------------------------------------------------------- climax
person("p-d-main-fordwarden", "Middle", ["head of the ford-wardens"], "pl-oathford")
person("p-d-main-remounter", "Middle", ["horse-dealer of the Marches"], "pl-bogford")
link("p-d-main-remounter", "p-d-main-paymaster", "trade", None, None)
link("p-d-main-bridgeson", "p-d-main-sergeant", "employer/employee", "employee", "employer", when="at m-d-main-6", note="took the company's coin")

# ---------------------------------------------------------------- places (sites; casting §2.4: none of these existed)
place("pl-d-main-gate-mill", "the Gate Mill", "mill", "p-d-main-uncle", "open", "Middle", "pl-crownsgate")
place("pl-d-main-gate-house", "the crown gate-house", "gate-house", "p-d0-crownsgate-warden", "guarded", "Middle", "pl-crownsgate", tags=["landmark"])
place("pl-d-main-toll-bridge", "the Crownsgate toll bridge", "toll bridge", "p-d-main-bridgekeeper", "open", "Middle", "pl-crownsgate")
place("pl-d-main-bogford-yard", "the Free Lord's yard at Bogford", "lord's yard", "p-d0-bogford-lord", "guarded", "Low", "pl-bogford")
place("pl-d-main-oathford-tower", "the lord's tower at Oathford", "lord's tower", "p-d0-oathford-lord", "guarded", "Middle", "pl-oathford", tags=["landmark"])
place("pl-d-main-long-shed", "the Long Shed", "stable hall", "p-doruduqu", "open", "Middle", "pl-carters-rest", tags=["landmark"])
place("pl-d-main-longwall-hall", "the general's hall inside the inner wall", "hall", "p-haahoqo", "fortified", "High", "pl-longwall")
place("pl-d-main-outer-towers", "Longwall's outer towers", "wall towers", "p-haahoqo", "fortified", "High", "pl-longwall", tags=["landmark"])
place("pl-d-main-carters-camp", "the carters' camp under the Brennock road fort", "carters' camp", None, "open", "Low", "pr-road-fort")
place("pl-d-main-breach", "the breach at Breakwall", "breach", "p-kotqot", "guarded", "High", "pl-breakwall", tags=["landmark"])
place("pl-d-main-southroad-ring", "the tavern-yard ring at Southroad", "dueling ring", "p-d-main-ladderkeeper", "open", "Middle", "pl-southroad")
place("pl-d-main-toll-hut", "the west toll hut of the Great Bridge", "toll hut", "p-d-main-oldkeeper", "open", "Low", "pl-southroad")
place("pl-d-main-great-bridge", "the Great Bridge", "bridge", "p-shahayuh", "guarded", "High", "pl-southroad", tags=["landmark"])
place("pl-d-main-meadows", "the river meadows below the Empty Gallery", "common grazing", "p-gahiru", "open", "Middle", "pl-highcrossing")
place("pl-d-main-crown-road", "the crown road from Highcrossing to Crownsgate", "road", None, "open", "Middle", "pl-highcrossing")
place("pl-d-main-scales-hall", "the Scales Hall", "counting hall", "p-feyesh", "guarded", "High", "pl-counting-house", tags=["landmark"])
place("pl-d-main-ferry", "the Oathford ferry stair", "ferry", "p-d-main-ferrywoman", "open", "Middle", "pl-oathford")
place("pl-d-main-burnt-tower", "the burned toll-tower on the Elderhollow road", "ruined tower", None, "remote", "Low", "pl-oathford")
place("pl-d-main-archive", "the archive of the Ninth Ring", "archive", "p-tuhudo", "guarded", "Top", "pl-ninth-ring", tags=["landmark"])
place("pl-d-main-paper-shop", "the paper-dealer's shop on the salt quay", "shop", "p-d-main-paperdealer", "open", "Middle", "pl-saltmarket")
place("pl-d-main-notary-stall", "the notary's stall under the salt-hall arches", "stall", "p-d-main-notary", "open", "Middle", "pl-saltmarket")
place("pl-d-main-quay", "the Unchained Quay", "quay", "p-waarua", "open", "Top", "pl-openharbour", tags=["landmark"])
place("pl-d-main-muster", "the muster field below the lord's tower", "muster field", "p-d0-oathford-lord", "open", "Middle", "pl-oathford")
place("pl-d-main-gallery", "the Empty Gallery", "hall", "p-gahiru", "guarded", "Top", "pl-highcrossing", tags=["landmark"])

# ---------------------------------------------------------------- things
thing("t-d-main-will", "the crown-sealed will that leaves the Gate Mill to Giktrok", "document", "p-d-main-cousin", "Middle", "pl-crownsgate", "p-d-main-cousin", "m-d-main-1")
thing("t-d-main-wagon", "the Gate Mill's flour wagon and its oxen", "goods", "p-d-main-heart", "Middle", "pl-d-main-bogford-yard", "p-d0-bogford-lord", "m-d-main-2")
thing("t-d-main-contract", "the Road Crews' haulage contract with the crown", "document", "p-doruduqu", "Middle", "pl-d-main-long-shed", "p-doruduqu", "before the story")
thing("t-d-main-commission", "the captain's commission of the emperor's new company", "document", "p-d0-oathford-lord", "High", "pl-d-main-outer-towers", "p-d0-oathford-lord", "m-d-main-3")
thing("t-d-main-founding-grant", "the founding grant of the Great Bridge", "document", "p-d-main-oldkeeper", "High", "pl-d-main-toll-hut", "p-d-main-oldkeeper", "before the story")
thing("t-d-main-grant", "the village grant naming Kemed Oath-keeper miller of the Gate Mill", "document", "p-d-main-heart", "Middle", "pl-d-main-paper-shop", "p-d-main-paperdealer", "m-d-main-5")
thing("t-d-main-ledger", "the company's tenth-ledger for Crownsgate", "document", "p-d0-oathford-lord", "Middle", "pl-d-main-gate-house", "p-d-main-sergeant", "m-d-main-6")
thing("t-d-main-oath", "the crown's tenth-roll of the oaths Crownsgate swore", "document", "p-d0-oathford-lord", "Middle", "pl-d-main-muster", "p-d-main-bridgeson", "m-d-main-6")

# ---------------------------------------------------------------- spine facts: the moves (main-quest §5)
H, U, C, B = N("p-d-main-heart"), N("p-d-main-uncle"), N("p-d-main-cousin"), N("p-d-main-brother")
LT, LI, HD = N("p-litita"), N("p-d0-oathford-lord"), N("p-d-main-hand")
fact("x-d-main-uncle-death", f"{U}, the old miller of the Gate Mill, died at Crownsgate.", kind="past event", when="40 days before the story", on=["p-d-main-uncle"])
fact("x-d-main-m1", f"At the reading after {U}'s death, his son {C}, an under-clerk of the crown sealing office, laid down a will sealed at "
     f"Highcrossing that leaves him the Gate Mill, against the village grant that names his cousin {H} its miller.",
     kind="move", when="30 days before the story", on=["pl-d-main-gate-mill"])
fact("x-d-main-m2", "Crown clerks out of the sealing office at Highcrossing have called a 'sealing tenth' on every crown-sealed grant along the "
     "Crossing and Marches roads, owed by midsummer in coin, grain or men; holders who cannot pay are taken into bond-service in the emperor's new company.",
     kind="move", when="after m-d-main-1")
fact("x-d-main-m3", f"{LI}, lord of Oathford, has marched the emperor's new company up the wall road and quartered it in Longwall's empty outer "
     "towers by the Throne's leave, and the company's riders now stop every cart on the Crossing and march roads to take the sealing tenth by force.",
     kind="move", when="after m-d-main-2")
fact("x-d-main-m4", "The crown has re-granted the tolls of the Great Bridge at Southroad to the emperor's new company under a fresh seal, over the head "
     "of the lord who built it; the Silver Road Counting-House has advanced the company a year's pay against those tolls; and crown coin is buying "
     "Southroad's toll-keepers one by one.", kind="move", when="after m-d-main-3")
fact("x-d-main-m5", f"At {LT}'s word, an imperial writ read out in the Empty Gallery and nailed up in every Great House town names {H} a forger of "
     "crown seals: no sealing office, market court or archive is to take a paper from her hand, and whoever shelters her answers for it.",
     kind="move", when="after m-d-main-4", on=["p-d-main-heart"])
fact("x-d-main-m6", "The emperor's new company has seized Crownsgate for the crown 'from a forger's kin', taken the Gate Mill and the village's winter "
     "grain into crown keeping, and holds the village's grant-holders in the old gate-house until they swear their grants to the crown; and by a "
     "proclamation read in every Great House town the Throne has named the company the emperor's own guard.", kind="move", when="after m-d-main-5")
# despite-you faces (Wins stick)
DESPITE = {
 "m-d-main-2": f"Though the Gate Mill's claim was settled for {H}, the sealing office says every crown-sealed grant on the roads now owes its tenth, the Gate Mill's with the rest.",
 "m-d-main-3": "Though the Long Shed held the crews' contract good and the carter from Crownsgate walked out of the lord's tower, the company now takes the tenth at the roadside, contract or no contract.",
 "m-d-main-5": f"Though the Scales Hall heard the crown clerk answer for his pledge, {LT} had the writ read in the Empty Gallery before the news could reach the court.",
 "m-d-main-6": f"Though {H} came home cleared by the Ninth Ring's archive or the Open Council's court, the company took Crownsgate 'from a forger's kin' all the same, under the Throne's writ, not theirs.",
}
# the revelation (main-quest §10 step 3), stored on the Hand who acted
fact("x-d-main-revelation",
     f"{LT} is raising the emperor's new company to hold Highcrossing for the Throne when her brother dies, before Haʻahoqo the General can count "
     f"Longwall's soldiers as his own. She pays for it by re-granting mills, fords and tolls to crown-bound holders under papers she seals herself in "
     f"Gahìru's name, because his hands can no longer hold the seal; the will that took the Gate Mill was the first paper she sealed so.",
     vis="secret", kind="state", when="before the story", on=["p-d-main-hand"])
FACTS["x-d-main-revelation"]["published_by"] = ["c-d-main-15", "c-d-main-18"]
