#!/usr/bin/env python3
"""Roll the five guild story kits for one world, following quest-faction-stories.md §9–§12 and quest-guild-kits.md.
Usage: python3 roll_kits.py <seed> <paired|unpaired|auto> > rolls.json"""
import json, random, sys
seed = int(sys.argv[1]); mode = sys.argv[2]
R = random.Random(seed)
log = []

# enemy: (shape, revelation kind, leader?, extra)
K = {
 "writers": {"enemies": {"W1":("Within","hidden hand",0),"W2":("Above","hidden hand",0),"W3":("Beside","the cost",0),
                          "W4":("Within","hidden hand",0),"W5":("Above","the old debt",0),"W6":("Within","the cost",1)},
             "stakes": ["S1","S2","S3","S4","S5"], "hearts": {"H1":"mentor","H2":"fellow recruit","H3":"client","H4":"veteran"},
             "odd": ["O1","O2","O3"],
             "crises": {"C1":({"W1","W4","W6"},"Reckoning"),"C2":({"W2","W5"},"Acquire"),"C3":({"W1"},"Reckoning"),"C4":({"W3"},"Reckoning")},
             "endings": ["open the ledgers","keep the balance","burn your copy"], "seat_ending": ("take the road","W6"),
             "stake_rules": lambda e,s: (s!="S5" or e=="W2") and (e!="W2" or s in ("S5","S2"))},
 "lantern": {"enemies": {"L1":("Above","the old debt",0),"L2":("Within","the old debt",0),"L3":("Within","hidden hand",0),
                         "L4":("Beside","hidden hand",0),"L5":("Above","the cost",0)},
             "stakes": ["S1","S2","S3","S4","S5"], "hearts": {"H1":"mentor","H2":"fellow recruit","H3":"client","H4":"member's kin"},
             "odd": ["O1","O2","O3"],
             "crises": {"C1":({"L1","L5"},"Rescue"),"C2":({"L2","L3"},"Reckoning"),"C3":({"L4"},"Reckoning"),"C4":({"L3"},"Reckoning")},
             "endings": ["honor it and march","put out the lamp","split the chapters","pick a side"], "seat_ending": None,
             "stake_rules": lambda e,s: (s!="S4" or e=="L1") and (e!="L1" or s in ("S4","S3"))},
 "old-hands": {"enemies": {"OH1":("Within","the old debt",1),"OH2":("Beside","hidden hand",0),"OH3":("Above","the old debt",0),
                           "OH4":("Within","the cost",0),"OH5":("Above","the old debt",0)},
             "stakes": ["S1","S2","S3","S4","S5"], "hearts": {"H1":"mentor","H2":"veteran","H3":"member's kin","H4":"client"},
             "odd": ["O1","O2","O3"],
             "crises": {"C1":({"OH1","OH4"},"Reckoning"),"C2":({"OH2"},"Reckoning (paired)"),"C3":({"OH3","OH5"},"Reckoning"),"C4":({"OH1"},"Acquire")},
             "endings": ["make the houses need us again","open the gate","give back the pages"], "seat_ending": ("take the gate","OH1"),
             "stake_rules": lambda e,s: (s!="S2" or e=="OH1") and (e!="OH1" or s in ("S2","S3"))},
 "night-market": {"enemies": {"N1":("Beside","hidden hand",0),"N2":("Within","the cost",0),"N3":("Above","hidden hand",0),
                              "N4":("Above","the old debt",0),"N5":("Within","the old debt",1)},
             "stakes": ["S1","S2","S3","S4"], "hearts": {"H1":"mentor","H2":"fellow recruit","H3":"client","H4":"member's kin"},
             "odd": ["O1","O2","O3"],
             "crises": {"C1":({"N1"},"Reckoning (paired)"),"C2":({"N4"},"Passage"),"C3":({"N2","N5"},"Reckoning"),"C4":({"N3"},"Reckoning")},
             "endings": ["pull the big one","sell to the Sharp Hands","go straight","put it on the water"], "seat_ending": ("take the shop","N5"),
             "stake_rules": lambda e,s: True},
 "dueling": {"enemies": {"D1":("Beside","hidden hand",0),"D2":("Above","the cost",0),"D3":("Within","hidden hand",1),
                         "D4":("Beside","hidden hand",0),"D5":("Beside","the old debt",0)},
             "stakes": ["S1","S2","S3","S4"], "hearts": {"H1":"mentor","H2":"fellow recruit","H3":"veteran","H4":"member's kin"},
             "odd": ["O1","O2","O3"],
             "crises": {"C1":({"D1"},"Contest"),"C2":({"D2","D4"},"Reckoning"),"C3":({"D3"},"Reckoning"),"C4":({"D5"},"Contest")},
             "endings": ["keep it sport","teach the deadly half","back to the tavern yards","let Liʻo run the show"], "seat_ending": ("take the ring","D3"),
             "stake_rules": lambda e,s: not (e=="D5" and s=="S2")},
}
ORDER = ["old-hands","night-market","writers","lantern","dueling"]
paired = {"paired":True,"unpaired":False,"auto":R.random()<0.5}[mode]
log.append(f"seed {seed}; syndicate pairing: {'PAIRED' if paired else 'unpaired'}")

def try_world():
    out = {}
    for g in ORDER:
        k = K[g]
        for attempt in range(200):
            if paired and g=="old-hands": e="OH2"
            elif paired and g=="night-market": e="N1"
            else:
                choices=[x for x in k["enemies"] if not (not paired and x in ("OH2","N1"))]
                e=R.choice(choices)
            shape,kind,leader=k["enemies"][e]
            crises=[c for c,(es,f) in k["crises"].items() if e in es and ("paired" not in f or paired)]
            if not crises: continue
            c=R.choice(crises); fam=k["crises"][c][1]
            stakes=[s for s in k["stakes"] if k["stake_rules"](e,s)]
            s=R.choice(stakes)
            hk=R.choice(list(k["hearts"])); ht=k["hearts"][hk]
            ev=R.choice(["captive","disgraced","estranged"])
            cand=dict(enemy=e,shape=shape,revelation_kind=kind,leader_enemy=bool(leader),crisis=c,crisis_family=fam,stake=s,
                      heart=hk,heart_type=ht,odd_job=R.choice(k["odd"]),ordeal_event=ev)
            # world-spread checks against stories already rolled (§9)
            prev=list(out.values())+[cand]
            bad=[]
            if sum(p["leader_enemy"] for p in prev)>1: bad.append("leader")
            if sum(p["shape"]=="Within" for p in prev)>2: bad.append("within")
            if sum(p["revelation_kind"]=="hidden hand" for p in prev)>2: bad.append("hidden hand")
            import collections as _c
            if max(_c.Counter(p["heart_type"] for p in prev).values())>2: bad.append("heart type")
            if max(_c.Counter(p["ordeal_event"] for p in prev).values())>2: bad.append("event")
            if sum(p["ordeal_event"]=="captive" for p in prev)>1: bad.append("captive")
            if sum(p["crisis_family"].startswith("Reckoning") for p in prev)>3: bad.append("reckoning")
            if bad:
                log.append(f"  {g}: reroll ({', '.join(bad)}) from {e}/{c}/{ht}/{ev}"); continue
            endings=list(k["endings"])
            if k["seat_ending"] and k["seat_ending"][1]==e: endings.append(k["seat_ending"][0])
            cand["faction_endings"]=endings
            out[g]=cand; break
        else:
            return None
    shapes={p["shape"] for p in out.values()}
    if not ({"Above","Beside"} <= shapes):
        log.append("  world: missing Above or Beside -> restart"); return None
    return out

for restart in range(50):
    w=try_world()
    if w: break
    log.append(f"restart {restart+1}")
print(json.dumps({"seed":seed,"paired":paired,"rolls":w,"log":log},indent=1,ensure_ascii=False))
