#!/usr/bin/env python3
"""Throughline test for the widened Writers and Lantern kits (fixlist-v6c; faction-stories §11a).
Enumerates every legal (enemy, stake, chapter-1 move, chapter-2 blow, crisis) under the kits' needs,
means, stake lists, shape and want filters, then draws 8 with random.Random(91) per kit and writes
the one-sentence throughline. Usage: python3 throughline.py [writers|lantern] [--all N]"""
import random, sys
from collections import Counter

ANY = None
# ---------------------------------------------------------------- shared
WANT = {"greed": "money", "revenge": "revenge", "preservation": "things kept as they are",
        "desperation": "a way out of ruin", "cover-up": "an old wrong kept buried",
        "ambition": "more power", "faith": "what the house demands"}
BLOW_NEEDS = {  # faction-stories §11a, What each blow needs
    ("heart", "captive"): ["men", "coin", "paper"], ("heart", "disgraced"): ["name", "inside"],
    ("heart", "estranged"): ["kin", "name"], ("player", "framed"): ["inside", "name"],
    ("player", "stripped of rank"): ["inside", "name"], ("player", "arrested"): ["paper", "men"],
    ("guild", "hall seized"): ["men", "paper"], ("guild", "members taken"): ["men", "paper", "coin"],
    ("guild", "outlawed"): ["paper"], ("guild", "leader struck down"): ["men", "inside"]}
PREF = ["men", "inside", "name", "coin", "kin", "craft", "faith", "paper"]  # `uses`: paper last (writ fatigue)

def uses(needs, means):
    c = [m for m in PREF if m in needs and m in means]
    return c[0] if c else None

# ---------------------------------------------------------------- the Writers
W = dict(
 name="Writers",
 enemies={  # shape, who, pronoun, want, means, reach, stakes, kind, scheme
  "W1": ("Within", "a hall master", "she", "preservation", ["paper", "name", "inside"], "region", ["S2", "S4", "S7"], "the cost",
         "releases two rival lords' true debts to each other through the Lending Wall, one letter at a time, keeping their feud at the simmer that suits the guild"),
  "W2": ("Above", "a Free Lord's archivist", "he", "ambition", ["coin", "men", "paper"], "region", ["S5"], "hidden hand",
         "buys the old imperial survey rolls out of Writers' digs and has the diggers who sold elsewhere killed, so his lord can file a 'restored' claim to three valleys"),
  "W3": ("Beside", "a Night Market forger", "she", "greed", ["craft", "name", "coin"], "local", ["S3", "S7"], "hidden hand",
         "copies what the halls keep back and sells each family its own file as a 'forgery'"),
  "W4": ("Within", "the hall clerk who runs the letter-writers", "he", "greed", ["inside", "coin", "name"], "region", ["S2", "S4"], "hidden hand",
         "has the letter-writers' stalls sell what they read, and sells his list"),
  "W5": ("Above", "a Holy House inquisitor", "he", "cover-up", ["faith", "paper", "men"], "region", ["S1", "S2"], "the old debt",
         "burns as heresy the texts that record how the Writers helped the Mount draw up its land grants"),
  "W6": ("Within", "the Writers' head herself", "she", "preservation", ["inside", "craft", "name"], "realm", ["S1", "S4", "S7"], "the cost",
         "has let every other copy of the region's land rolls go astray, so whoever holds the Writers decides who owns the land"),
  "W7": ("Beside", "a copyist struck off the Wayhouse's roll", "she", "revenge", ["craft", "name", "inside"], "local", ["S3", "S7"], "the old debt",
         "enters false lines in Writers' copies, in a hand no hall can tell from its own, so that no Writers' copy will stand again"),
  "W8": ("Above", "a Warden captain whose towers are failing", "he", "desperation", ["men", "kin", "name"], "region", ["S6"], "hidden hand",
         "has the survey camps' edge readings redrawn to put the green a mile nearer the towns, so frightened lords will pay for his wall"),
 },
 leader_enemy="W6",
 stakes={"S1": ("the Wayhouse", "place", ["paper", "men", "coin", "inside"]),
         "S2": ("the Lending Wall's letters", "thing", ["paper", "inside", "faith", "men"]),
         "S3": ("the name for true copies", "name", ["name", "craft", "paper"]),
         "S4": ("a reading room", "place", ["paper", "men", "coin", "inside"]),
         "S5": ("the old imperial survey rolls", "thing", ["coin", "men"]),
         "S6": ("the edge maps", "thing", ["craft", "men", "kin"]),
         "S7": ("the keeping of a lord's court rolls", "trade", ["coin", "name", "craft"])},
 # moves: needs, hits, stakes, shape ('W' Within only, '!W' never Within), modes ok under the cost, writ, text by stake
 moves={
  "M1": (["coin", "craft"], ["trade", "place"], ["S1", "S4", "S7"], "!W", False, False,
         {"S1": "opens a copy house across the street from the Wayhouse at half the halls' price and takes its clients",
          "S4": "opens a copy house beside the reading room at half its price and takes its clients",
          "S7": "underbids the halls for the lord's court rolls with a copy house of {pos} own"}),
  "M2": (["paper"], ["thing", "name", "trade"], ["S2", "S3", "S5", "S6", "S7"], "!W", True, True,
         {"S2": "has a court seize one of the Lending Wall's letters as evidence, under seal",
          "S5": "has a lord's court rule that rolls bought at a dig belong to the buyer, not the halls",
          "S6": "has a lord's court call one camp's edge map as evidence and rule it unproven",
          "S3": "has a magistrate rule in open court that a Writers' copy doesn't stand for the original",
          "S7": "has the lord's court refuse a Writers' copy of its own rolls"}),
  "M3": (["inside"], ["place", "thing", "trade"], ["S1", "S2", "S4", "S7"], "W", True, False,
         {"S1": "openly takes the Wayhouse strongroom's keys into {pos} own keeping, over the halls' objection",
          "S2": "openly takes the sorting of the Lending Wall into {pos} own hands",
          "S4": "openly takes the reading room's master copies into {pos} own keeping",
          "S7": "openly takes the lord's court rolls into {pos} own keeping, so no other copyist touches them"}),
  "M4": (["coin"], ["people", "trade"], ["S4", "S7"], ANY, True, False,
         {"S4": "buys the reading room's best copyist away from the hall in public, copying book and all",
          "S7": "buys the copyist who keeps the lord's court rolls away from the hall in public, copying book and all"}),
  "M5": (["name"], ["name", "thing"], ["S3", "S6", "S7"], "!W", False, False,
         {"S3": "has it cried in a market square that a Writers' copy was forged",
          "S6": "has it cried in the edge towns that the Writers' maps put the green too far off",
          "S7": "has it cried at the lord's court that the Writers' copies of its rolls are false"}),
  "M6": (["men", "faith"], ["place", "thing", "people"], ["S5", "S6"], ANY, False, False,
         {"S5": "closes the dig the rolls came from and turns its diggers off",
          "S6": "closes a survey camp at the green's edge and keeps its journals"}),
  "M7": (["inside", "name"], ["thing", "name"], ["S2"], ANY, False, False,
         {"S2": "has a letter from the Lending Wall read aloud in a market square, so everyone knows the Wall is open"}),
  "M8": (["men", "faith", "paper"], ["place", "thing"], ["S1", "S2", "S4"], "!W", False, False,
         {"S1": "has the Wayhouse searched for forbidden texts and its doors sealed for a week",
          "S2": "has the Lending Wall searched for forbidden letters and its pigeonholes sealed for a week",
          "S4": "has the reading room searched for forbidden texts and its doors sealed for a week"}),
  "M9": (["inside"], ["people", "name"], ["S1", "S2", "S4", "S7"], "W", True, False,
         {"S1": "dismisses, before the whole hall, the Wayhouse copyist who asked where the strongroom's copies went",
          "S2": "dismisses, before the whole hall, the Lending Wall clerk who asked who reads the letters",
          "S4": "dismisses, before the whole hall, the reading room's copyist who asked where its master copies went",
          "S7": "dismisses, before the whole hall, the copyist who kept the lord's court rolls"}),
 },
 # blows the kit allows: (target, event): fragment by uses-means; extra filters
 blows={
  ("heart", "captive"): {"men": "{pos} men take the heart and hold her", "coin": "{p} buys the heart's debts and has her held for them",
                         "paper": "the heart is held under a charge {p} laid"},
  ("heart", "disgraced"): {"name": "{p} has the heart cried down for destroying a text",
                           "inside": "{pos} people in the hall accuse the heart of taking a side in public"},
  ("heart", "estranged"): {"kin": "{p} turns the heart's kin against her", "name": "a rumour {p} spreads turns the heart's kin against her"},
  ("player", "framed"): {"inside": "{pos} people in the hall frame you for destroying a text",
                         "name": "{p} has it spread that you took a side in public"},
  ("player", "stripped of rank"): {"inside": "{pos} people in the hall have you put down a rank", "name": "{pos} talk has you put down a rank in the hall"},
  ("player", "arrested"): {"men": "{pos} men hand you to a lord's cells", "paper": "you are arrested on a charge {p} laid"},
  ("guild", "hall seized"): {"men": "{pos} men shut a reading room", "paper": "a writ {p} obtained shuts a reading room",
                             "inside": "{p} shuts a reading room by {pos} own order"},
  ("guild", "members taken"): {"men": "{pos} men march a field camp's diggers off", "coin": "copyists are bonded for debts {p} bought",
                               "paper": "copyists are arrested on {pos} charge"},
  ("guild", "outlawed"): {"paper": "a lord's writ {p} obtained closes the halls in his lordship"},
  ("guild", "leader struck down"): {"men": "the Writers' head is wounded on the road by {pos} men",
                                    "inside": "{pos} people inside disgrace the Writers' head"},
 },
 leader_never=["W6"],
 # crises: family/premise, needs, hits, stakes, shape, wants, writ-if-paper, text by stake
 crises={
  "C1": ("Reckoning", "Betrayal", ["inside"], ["thing", "place", "trade", "people"], ["S1", "S2", "S4", "S7"], "W", ANY, False,
         {"S1": "the Wayhouse strongroom's copies are burned and the heart is named as the one who let the burners in",
          "S2": "the Lending Wall's letters are burned and the heart is named as the one who let the burners in",
          "S4": "the reading room's master copies are burned and the heart is named as the one who let the burners in",
          "S7": "the halls' copies of the lord's court rolls are burned and the heart is named as the one who let the burners in"}),
  "C2": ("Acquire", "Copy it before it burns", ["paper"], ["thing"], ["S2", "S6"], "!W", ANY, True,
         {"S2": "a Highcrossing magistrate's writ, on a charge {p} laid, puts the Lending Wall's letters in {pos} hands, to burn at dawn",
          "S6": "a Highcrossing magistrate's writ, on a charge {p} laid, puts the edge maps in {pos} hands, to burn at dawn"}),
  "C3": ("Reckoning", "Schism", ["inside"], ["people", "thing", "place", "trade"], ["S2", "S4", "S7"], "W-notleader", ANY, False,
         {"S2": "a letter let out through the Lending Wall at the wrong moment brings armed men to the heart's village, and it burns",
          "S4": "what leaks out of the reading room brings armed men to the heart's village, and it burns",
          "S7": "a debt read out of the lord's court rolls turns his quarrel into a feud, and the heart's village burns"}),
  "C4": ("Reckoning", "Swindle", ["craft", "coin", "inside"], ["people", "name", "thing", "trade"], ["S2", "S3", "S7"], ANY,
         ["greed", "revenge", "desperation"], False,
         {"S2": "{p} sells the heart's own letters to the man they ruin and is gone, and the heart loses her post",
          "S3": "{p} sells the heart's own file, in a copy no hall can tell from its own, to the man it ruins, and is gone",
          "S7": "{p} sells the heart's copy of the court rolls to the lord's rival and is gone, and the heart loses her place at court"}),
  "C5": ("Rescue", "Imprisoned unjustly", ["paper", "faith"], ["people", "place", "thing"], ["S1", "S2", "S5"], "Above", ANY, True,
         {"S1": "on {charge} {p} laid, {holders} hold the Wayhouse's copyists in its own reading hall, the heart's kin among them",
          "S2": "on {charge} {p} laid, {holders} hold the Lending Wall's clerks at the Wayhouse until its letters are handed over, the heart's kin among them",
          "S5": "on {charge} {p} laid, {holders} hold the Wayhouse's copyists until the rolls lodged there are handed over, the heart's kin among them"}),
  "C6": ("Passage", "Fugitive", ["men"], ["people", "thing"], ["S2", "S5", "S6", "S7"], "!W", ANY, False,
         {"S2": "the heart must carry the one letter that proves it out of Highcrossing, with {pos} men on every road",
          "S5": "the heart must carry the rolls lodged at the Wayhouse out of Highcrossing, with {pos} men on every road",
          "S6": "the heart must carry the camp's true readings out of Highcrossing, with {pos} men on every road",
          "S7": "the heart must carry the halls' true copy of the court rolls out of Highcrossing, with {pos} men on every road"}),
  "C7": ("Dispute", "Public offense", ["name"], ["name", "people", "trade", "thing"], ["S3", "S6", "S7"], ANY, ANY, False,
         {"S3": "{p} accuses the heart before a lord's court of a false copy, and the guild's name rides on the hearing",
          "S6": "{p} accuses the heart before a lord's court of drawing the edge maps false, and the guild's name rides on the hearing",
          "S7": "{p} accuses the heart before the lord's court of a false copy of its rolls, and the halls' keeping of them rides on the hearing"}),
  "C8": ("Acquire", "Won't sell", ["coin"], ["place", "trade", "people"], ["S1", "S4", "S7"], "!W", ANY, False,
         {"S1": "{p} has bought the debt on the Wayhouse fair and square, means to close it at the season's end, and won't sell it back",
          "S4": "{p} has bought the reading room's lease fair and square, means to turn the copyists out, and won't sell it back",
          "S7": "{p} has bought the lord's court commission fair and square and won't sell it back"}),
 },
 endings={"open the ledgers": ["thing", "name", "Reckoning"], "keep the balance": ["place", "trade", "Dispute", "Reckoning"],
          "burn your copy": ["thing", "Acquire", "Passage"], "a copy in every hall": ["thing", "place", "Acquire", "Rescue"],
          "trade what we know": ["name", "trade", "Dispute", "Rescue"], "take a side at last": ["name", "Reckoning", "Dispute", "Passage"]},
 seat=("W6", "take the road"),
 charge={"W5": ("a heresy charge", "his inquisitors"), "W2": ("a charge of theft", "a magistrate's men")},
 move_stake_wants={("M3", "S1"): ["preservation"], ("M3", "S4"): ["preservation"], ("M3", "S7"): ["preservation"]},
 hq_bar=("C5", "inside"),  # the Wayhouse isn't seized under C5 (its reading hall holds the copyists)
)

# ---------------------------------------------------------------- the Lantern Company
L = dict(
 name="Lantern",
 enemies={
  "L1": ("Above", "a Free Lord who holds the Company's oldest contract", "he", "ambition", ["paper", "men", "name"], "region", ["S3", "S4"], "the old debt",
         "means to call the Company's oldest contract in for his own feud with a neighbouring lord, knowing the code won't let it refuse"),
  "L2": ("Within", "a chapter-house captain", "he", "cover-up", ["inside", "coin", "kin"], "local", ["S2", "S5"], "the old debt",
         "has kept the lamps of two comrades his chapter abandoned years ago burning by hand, listed them 'still on contract', and paid their kin out of other contracts"),
  "L3": ("Within", "the quartermaster's deputy", "she", "ambition", ["inside", "paper"], "region", ["S1", "S3", "S6"], "hidden hand",
         "steers contracts so the Company ends up on one side, because she believes neutrality is cowardice"),
  "L4": ("Beside", "a Proving Ground sellsword captain", "he", "desperation", ["craft", "men", "coin"], "region", ["S6"], "hidden hand",
         "undercuts the Lanterns and frames them for the same contracts, because his people won't survive the winter otherwise"),
  "L5": ("Above", "the crown's recruiter for one march", "he", "ambition", ["coin", "paper", "men"], "region", ["S3", "S5"], "the cost",
         "buys the march's chapter contracts through go-betweens, so those chapters stand wherever the march's muster needs them"),
  "L6": ("Beside", "a hill-village headman whose son died when a chapter held his village for a lord", "he", "revenge", ["name", "kin", "men"], "local", ["S3", "S6"], "the old debt",
         "sends the Company false calls for help from the hill villages and turns the villages against any chapter that comes"),
  "L7": ("Above", "a prior of the Holy House's armed order", "he", "faith", ["faith", "men", "name"], "region", ["S2", "S3"], "hidden hand",
         "has the shrines tell the Company's clients that a Lantern escort carries the house's curse, because a company that guards heretics' carts and keeps pagan lamps for its dead is damned"),
  "L8": ("Beside", "the Sharp Hands factor who is the Company's biggest client", "he", "greed", ["coin", "paper", "name"], "region", ["S1", "S3", "S6"], "the cost",
         "holds the Lamp Room's debts, bought from the Company's own creditors, and takes his interest as contracts at cost, so chapters ride his caravans for nothing"),
 },
 leader_enemy=None,
 stakes={"S1": ("Lanternhall", "place", ["men", "paper", "coin", "inside"]),
         "S2": ("the Lamp Room's lamps", "thing", ["faith", "inside", "men"]),
         "S3": ("the Company's neutrality", "name", ["name", "paper", "coin", "inside", "faith"]),
         "S4": ("the old contract", "thing", ["paper"]),
         "S5": ("a chapter house", "place", ["men", "paper", "coin", "inside"]),
         "S6": ("the road contract", "trade", ["coin", "craft", "men", "name", "inside"])},
 moves={
  "M1": (["craft", "coin"], ["trade"], ["S5", "S6"], "!W", False, False,
         {"S6": "puts {pos} own blades on the caravan road at half the Company's price",
          "S5": "puts {pos} own blades on the chapter house's contracts at half its price"}),
  "M2": (["name"], ["name"], ["S3"], "!W", False, False,
         {"S3": "has it cried in every market that the Company has taken a side"}),
  "M3": (["men"], ["trade", "place"], ["S5", "S6"], ANY, False, False,
         {"S6": "closes the caravan road to Lantern escorts with {pos} men",
          "S5": "closes the roads round the chapter house to its riders with {pos} men"}),
  "M4": (["paper"], ["thing", "name"], ["S3", "S4"], "!W", True, False,
         {"S4": "calls in the old contract before the contract board, in public",
          "S3": "calls in, before the contract board, a contract the Company can't refuse, binding it to one side"}),
  "M5": (["coin"], ["people", "name"], ["S3", "S5"], "!W", True, False,
         {"S3": "buys a well-known chapter captain into {pos} own service in front of the contract board",
          "S5": "buys the chapter house's captain into {pos} own service in front of its board"}),
  "M6": (["inside"], ["place", "thing", "name", "trade"], ["S1", "S2", "S3", "S5", "S6"], "W", True, False,
         {"S1": "openly takes Lanternhall's contract board into {pos} own hands",
          "S2": "openly takes the lamps of one chapter's dead into {pos} own keeping and lets no one trim them",
          "S3": "openly posts only one side's contracts on the board, against the quartermaster's word",
          "S5": "openly shuts the chapter house's books to the quartermaster's auditors",
          "S6": "openly gives the road contract to the side {p} favours, against the quartermaster's word"}),
  "M7": (["faith"], ["thing", "name"], ["S2", "S3"], ANY, False, False,
         {"S2": "has the shrines preach that the Lamp Room's lamps are pagan rites",
          "S3": "has the shrines preach that a Lantern who guards a heretic's cart shares the heresy"}),
  "M8": (["coin"], ["thing", "place", "trade"], ["S1", "S5", "S6"], ANY, True, False,
         {"S1": "buys up the last of the Company's debts and posts {pos} terms on Lanternhall's own contract board",
          "S5": "buys up the chapter house's debts and says so in front of its board",
          "S6": "buys up the debts the road contract was pledged against and says so at the contract board"}),
  "M9": (["inside"], ["people", "name"], ["S1", "S2", "S3", "S5", "S6"], "W", True, False,
         {"S1": "throws the clerk who asked who signs Lanternhall's contracts out of the hall, before the board",
          "S2": "throws out, before the contract board, the lamp-keeper who asked whose lamps those are",
          "S3": "throws out, before the contract board, the captain who spoke for the Company's neutrality",
          "S5": "throws out, before the chapter house's board, the rider who asked where its old dead lie",
          "S6": "takes the road contract from its captain before the board, for asking whose side it serves"}),
 },
 move_wants={"M8": ["greed", "ambition"]},
 blows={
  ("heart", "captive"): {"men": "{pos} men take the heart and hold her", "coin": "{p} buys the heart's debt and has her held for it",
                         "paper": "the heart is held under a writ {p} obtained"},
  ("heart", "disgraced"): {"name": "{p} has the heart cried down for abandoning a comrade",
                           "inside": "{pos} people in the chapter accuse the heart of abandoning a contract"},
  ("heart", "estranged"): {"kin": "{p} turns the heart's kin against her", "name": "a rumour {p} spreads turns the heart's kin against her"},
  ("player", "framed"): {"inside": "{pos} people in the chapter frame you for abandoning a contract",
                         "name": "{p} has it spread that you left a comrade behind"},
  ("player", "stripped of rank"): {"inside": "{pos} people in the chapter have you put down a rank at the contract board", "name": "{pos} talk has you put down a rank"},
  ("player", "arrested"): {"men": "{pos} men take you to a lord's cells", "paper": "you are arrested on a charge {p} laid"},
  ("guild", "hall seized"): {"men": "{pos} men shut a chapter house", "paper": "a writ {p} obtained shuts a chapter house",
                             "inside": "{p} shuts a chapter house by {pos} own order"},
  ("guild", "members taken"): {"men": "{pos} men take a chapter in a fight and hold it", "paper": "a chapter is arrested on {pos} charge"},
  ("guild", "outlawed"): {"paper": "a march's writ {p} obtained bars the Company's chapters there"},
  ("guild", "leader struck down"): {"men": "the quartermaster is wounded on the road by {pos} men",
                                    "inside": "{pos} people inside disgrace the quartermaster"},
 },
 leader_never=[],
 crises={
  "C1": ("Rescue", "Imprisoned unjustly", ["paper", "faith"], ["people", "place", "name", "thing"], ["S1", "S2", "S3", "S4"], "AND men|inside", ANY, True,
         {"S1": "{writ} names one chapter; it refuses, and is held in Lanternhall's own yard on the Company's right to judge its people, the heart among those pleading for them",
          "S2": "{writ} names the chapter that keeps the lamps, its lamps to be put out; it refuses, and {pos} men hold it in Lanternhall's yard, the heart among those pleading for them",
          "S3": "{writ} names one chapter {side}; it refuses, and {pos} men hold it in Lanternhall's yard, the heart among those pleading for them",
          "S4": "{writ} names one chapter under the old contract; it refuses, and {pos} men hold it in Lanternhall's yard, the heart among those pleading for them"}),
  "C2": ("Reckoning", "Betrayal", ["inside"], ["people", "thing", "place", "name", "trade"], ["S2", "S3", "S5", "S6"], "W", ANY, False,
         {"S2": "on {pos} orders a chapter is led into an ambush and dies holding a road, the heart's kin among them, and their lamps join the ones {p} kept lit",
          "S5": "on {pos} orders the chapter house's riders who know the old story are led into an ambush and die holding a road, the heart's kin among them",
          "S3": "on {pos} orders a chapter is led into an ambush and dies holding a road for the side {p} chose, the heart's kin among them",
          "S6": "on {pos} orders a chapter is led into an ambush and dies holding the caravan road, the heart's kin among them"}),
  "C3": ("Reckoning", "Swindle", ["craft", "coin"], ["trade", "people"], ["S6"], "!W", ["desperation", "greed"], False,
         {"S6": "{p} wins the Company's road contract with the Company's own stolen route maps, and the caravan the heart rides with is ruined"}),
  "C4": ("Reckoning", "Schism", ["inside"], ["people", "name", "place", "thing", "trade"], ["S1", "S2", "S3", "S5", "S6"], "W", ANY, False,
         {"S3": "{pos} steering puts two chapters on opposite sides of one fight, and the heart is disowned by her chapter",
          "S6": "{pos} steering puts two chapters on opposite sides of the caravan road, and the heart is disowned by her chapter",
          "S1": "two chapters stand at Lanternhall's gate, one for {obj} and one against, and the heart is disowned by her chapter",
          "S2": "the kin {p} paid come to Lanternhall for their men, {pos} chapter stands by {obj}, and the heart is disowned by her chapter",
          "S5": "the kin {p} paid come to the chapter house for their men, {pos} chapter stands by {obj}, and the heart is disowned by her chapter"}),
  "C5": ("Threat", "Raiders", ["men"], ["trade", "people", "place"], ["S6"], "Beside", ANY, False,
         {"S6": "{pos} men take to the caravan road out of Carter's Rest as a band, and the caravan the heart rides with is the first they strike"}),
  "C6": ("Rescue", "Bound by debt", ["coin"], ["people", "place", "thing", "trade"], ["S1", "S2", "S5", "S6"], ANY, ["greed", "ambition"], False,
         {"S1": "{p} calls in the Company's debts, and the chapter that can't pay is held at Lanternhall to work them off, the heart's kin among them",
          "S2": "{p} calls in the debts the founding lamp was pledged against, and the chapter that can't pay is held at Lanternhall to work them off, the heart's kin among them",
          "S5": "{p} calls in the chapter house's debts, and its chapter is held at Lanternhall to work them off, the heart's kin among them",
          "S6": "{p} calls in the debts the road contract was pledged against, and the chapter that rode it is held at Lanternhall to work them off, the heart's kin among them"}),
  "C7": ("Passage", "The contract's other side", ["coin", "paper"], ["thing", "people", "name", "trade"], ["S3", "S6"], ANY, ["ambition", "greed", "faith"], False,
         {"S3": "{p} sets a Lantern chapter {underc} against the Company's own escort leaving Lanternhall, the caravan the heart rides with in its charge",
          "S6": "{p} sets a Lantern chapter {underc} against the Company's escort on the caravan road, the caravan the heart rides with in its charge"}),
  "C8": ("Dispute", "Public offense", ["name", "faith"], ["name", "people", "thing"], ["S2", "S3"], ANY, ANY, False,
         {"S3": "{p} accuses the Company before Carter's Rest's judge of {offense}, naming the heart's chapter as the ones who did it",
          "S2": "{p} accuses the Company before Carter's Rest's judge of keeping pagan lamps for its dead, naming the heart's chapter as their keepers"}),
 },
 writ={"L1": "his seizure order under the old contract", "L5": "the march magistrate's levy writ",
       "L3": "a lord's writ she steered for", "L7": "a heresy writ from the house's court"},
 side={"L1": "to march in his feud", "L5": "for the march's muster", "L3": "to march for the side she chose", "L7": "for judgment, as the guard of heretics' carts"},
 crisis_not_cost=["C3", "C7"],
 offense={"L8": "breaking its contracts with him", "L1": "fighting for his rival", "L6": "the killing at his village", "L7": "selling its swords to heretics"},
 endings={"honor it and march": ["thing", "name", "Rescue", "Passage"], "put out the lamp": ["thing", "Reckoning", "Rescue"],
          "split the chapters": ["place", "name", "Reckoning", "Threat"], "pick a side": ["name", "trade", "Passage", "Dispute"],
          "buy the contract back": ["thing", "trade", "Rescue"], "hold the road for nothing": ["trade", "place", "Threat", "Passage"]},
 seat=None,
 hq_bar=None,
)

def shape_ok(rule, shape, eid, kit):
    if rule is ANY: return True
    if rule == "W": return shape == "Within"
    if rule == "!W": return shape != "Within"
    if rule == "W-notleader": return shape == "Within" and eid != kit["leader_enemy"]
    if rule == "Above": return shape == "Above"
    if rule == "Beside": return shape == "Beside"
    if rule.startswith("AND"): return True
    return True

def legal(kit):
    out = []
    for eid, (shape, who, p, want, means, reach, estakes, kind, scheme) in kit["enemies"].items():
        for sid in estakes:
            sname, hit, sneeds = kit["stakes"][sid]
            if not set(sneeds) & set(means): continue
            for mid, (mneeds, mhits, mst, mshape, cost_ok, mwrit, mtext) in kit["moves"].items():
                if sid not in mst or not shape_ok(mshape, shape, eid, kit): continue
                mu = uses(mneeds, means)
                if not mu: continue
                if kind == "the cost" and not cost_ok: continue
                if want not in kit.get("move_wants", {}).get(mid, [want]): continue
                if want not in kit.get("move_stake_wants", {}).get((mid, sid), [want]): continue
                if hit not in mhits and "people" not in mhits: continue
                for cid, (fam, prem, cneeds, chits, cst, cshape, cwants, cwrit, ctext) in kit["crises"].items():
                    if sid not in cst or not shape_ok(cshape, shape, eid, kit): continue
                    cu = uses(cneeds, means)
                    if not cu: continue
                    if cshape and cshape.startswith("AND") and not (set(cshape[4:].split("|")) & set(means)): continue
                    if cwants and want not in cwants: continue
                    if kind == "the cost" and cid in kit.get("crisis_not_cost", []): continue
                    if hit not in chits: continue
                    ends = [e for e, f in kit["endings"].items() if hit in f or fam in f]
                    if len(ends) < 3: continue
                    for (t, ev), frag in kit["blows"].items():
                        needs = BLOW_NEEDS[(t, ev)] + (["inside"] if ev == "hall seized" and shape == "Within" else [])
                        if ev == "hall seized" and shape == "Within": needs = ["inside"]  # a Within enemy shuts a hall by order
                        if ev == "outlawed" and (shape == "Within" or reach == "local"): continue
                        if ev == "leader struck down" and eid in kit["leader_never"]: continue
                        bu = uses([n for n in needs if n in frag], means)
                        if not bu: continue
                        x = dict(e=eid, s=sid, m=mid, mu=mu, b=(t, ev), bu=bu, c=cid, cu=cu, ends=ends)
                        if writs(kit, x) > 1: continue  # at most one writ part per story (Claude)
                        out.append(x)
    return out

def writs(kit, x):
    n = 0
    mw = kit["moves"][x["m"]][5]
    if mw and x["mu"] == "paper": n += 1
    if x["bu"] == "paper" and x["b"] in [("guild", "hall seized"), ("guild", "members taken"), ("guild", "outlawed")]: n += 1
    if kit["crises"][x["c"]][7] and x["cu"] == "paper": n += 1
    return n

def sentence(kit, x):
    shape, who, p, want, means, reach, estakes, kind, scheme = kit["enemies"][x["e"]]
    pos = {"he": "his", "she": "her"}[p]; obj = {"he": "him", "she": "her"}[p]
    f = lambda s: s.format(p=p, pos=pos, obj=obj, writ=kit.get("writ", {}).get(x["e"], "a writ"),
                           offense=kit.get("offense", {}).get(x["e"], "taking a side"), side=kit.get("side", {}).get(x["e"], "to march for one side"), underc=("under a contract " + p + " signed") if shape == "Within" else ("under contract to " + obj),
                           charge=kit.get("charge", {}).get(x["e"], ("a charge", "a magistrate's men"))[0],
                           holders=kit.get("charge", {}).get(x["e"], ("a charge", "a magistrate's men"))[1]).replace("'s men take you", "'s men take you")
    mv = f(kit["moves"][x["m"]][6][x["s"]])
    bl = f(kit["blows"][x["b"]][x["bu"]])
    cr = f(kit["crises"][x["c"]][8][x["s"]])
    want_txt = WANT[want] if not (want == "faith" and p == "she") else "what she believes the gods demand"
    return (f"Because {who} wants {want_txt} ({want}), {p} {scheme}; {p} {mv}, then {bl}, "
            f"and it comes to a head when {cr}.")

def tag(kit, x):
    fam, prem = kit["crises"][x["c"]][:2]
    return f"[{x['e']} · {x['s']} · {x['m']} ({x['mu']}) · {x['b'][0]}: {x['b'][1]} ({x['bu']}) · {x['c']} {fam} *{prem}* ({x['cu']}) · writs {writs(kit, x)}]"

if __name__ == "__main__":
    which = sys.argv[1]; kit = W if which == "writers" else L
    combos = legal(kit)
    # stats
    print(f"legal combinations: {len(combos)}")
    print("enemy+crisis pairs:", len({(x['e'], x['c']) for x in combos}))
    for k in ("e", "s", "m", "c"):
        print(k, dict(sorted(Counter(x[k] for x in combos).items())))
    print("blows", dict(Counter(x["b"][1] for x in combos)))
    print("writ>=2:", sum(1 for x in combos if writs(kit, x) >= 2))
    # per enemy: crises and stakes reachable
    for e in kit["enemies"]:
        cs = sorted({x["c"] for x in combos if x["e"] == e}); ss = sorted({x["s"] for x in combos if x["e"] == e})
        ms = sorted({x["m"] for x in combos if x["e"] == e})
        print(f"  {e}: stakes {ss} moves {ms} crises {cs}")
    # drawn the way the kit roller draws: the enemy first, then stake, move, crisis and blow, each among
    # what is still legal (a flat draw over combinations over-weights the enemies with the most blows)
    R = random.Random(91); pick = []
    for _ in range(8):
        pool = combos
        for key in ("e", "s", "m", "c", "b"):
            v = R.choice(sorted({x[key] for x in pool}, key=str)); pool = [x for x in pool if x[key] == v]
        pick.append(R.choice(pool))
    print()
    for i, x in enumerate(pick, 1):
        print(f"{i}. {sentence(kit, x)}\n   {tag(kit, x)} endings: {x['ends']}")
    if "--all" in sys.argv:
        n = int(sys.argv[sys.argv.index("--all") + 1]); R2 = random.Random(5)
        print("\n--- extra review sample ---")
        for x in R2.sample(combos, min(n, len(combos))):
            print("-", sentence(kit, x), tag(kit, x))
