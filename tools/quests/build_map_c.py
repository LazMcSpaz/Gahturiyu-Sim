#!/usr/bin/env python3
"""Hand-placed test island for the world-generation tests (a WORLD ASSUMPTION, not land-generator output).
Placement follows faction-places.md §1 relationships. 1 map unit = about one day's travel.
Writes map.json."""
import json, math, random
random.seed(26)

regions = {
 "r-vale": "the Elder Vale", "r-reefcoast": "the Reef Coast", "r-crossing": "the Crown Crossing",
 "r-marches": "the Marches", "r-harbour": "the Harbour Coast", "r-roads": "the Carter Roads",
 "r-rim": "the Broken Rim", "r-snows": "the High Snows", "r-green": "the Greenmarch", "r-sea": "the Wide Bay"}
territories = {
 "t-first": ("f-first-peoples", None), "t-great": ("f-great-house", None), "t-free": ("f-great-house", "f-free-lords"),
 "t-open": ("f-open-hand", None), "t-wave": ("f-last-wave", None), "t-high": ("f-highfolk", None),
 "t-green": ("f-rooted", None), "t-sails": ("f-far-sails", None)}

# id, name, kind, size, territory, region, x, y, standing, fixed(canon)?, owner faction
P = [
 # First Peoples
 ("pl-ninth-ring","the Ninth Ring","great town","great","t-first","r-vale",2.0,4.0,"Top",True,"f-first-peoples"),
 ("pl-firststone","Firststone","hill estate","village","t-first","r-vale",2.7,3.1,"High",True,"f-first-peoples"),
 ("pl-mount","the Mount","shrine-town","town","t-first","r-vale",3.1,4.7,"High",True,"f-holy-house"),
 ("pl-reefhold","Reefhold","stilt town","large","t-first","r-reefcoast",0.9,6.4,"High",True,"f-first-peoples"),
 ("pl-elderhollow","Elderhollow","old town","town","t-first","r-vale",2.3,5.6,"Middle",False,"f-first-peoples"),
 ("pl-ashring","Ashring","old village","village","t-first","r-vale",1.1,3.2,"Middle",False,"f-first-peoples"),
 ("pl-shellring","Shellring","reef village","village","t-first","r-reefcoast",0.5,5.3,"Low",False,"f-first-peoples"),
 # Great House (Throne / Banner)
 ("pl-highcrossing","Highcrossing","great town (capital)","great","t-great","r-crossing",5.4,4.1,"Top",True,"f-great-house"),
 ("pl-longwall","Longwall","fortress town","large","t-great","r-crossing",7.0,2.9,"High",True,"f-great-house"),
 ("pl-southroad","Southroad","toll town","large","t-great","r-crossing",5.6,5.6,"High",True,"f-great-house"),
 ("pl-oathford","Oathford","river town","town","t-great","r-crossing",4.3,5.0,"Middle",False,"f-great-house"),
 ("pl-crownsgate","Crownsgate","gate village","village","t-great","r-crossing",6.3,3.4,"Middle",False,"f-great-house"),
 # Free Lords frontier
 ("pl-breakwall","Breakwall","frontier keep","town","t-free","r-marches",8.2,4.6,"High",True,"f-free-lords"),
 ("pl-chainmarket","the Chainmarket","market town","large","t-free","r-marches",8.8,5.5,"High",True,"f-binders"),
 ("pl-bogford","Bogford","bog village","village","t-free","r-marches",7.6,5.4,"Low",False,"f-free-lords"),
 # Open Hand
 ("pl-openharbour","Openharbour","great port","great","t-open","r-harbour",4.9,8.0,"Top",True,"f-open-hand"),
 ("pl-counting-house","the Silver Road Counting-House","fortified counting-house","village","t-open","r-roads",5.5,6.8,"High",True,"f-sharp-hands"),
 ("pl-carters-rest","Carter's Rest","waystation town at the march crossroads (Lanternhall)","large","t-open","r-roads",7.3,7.0,"Middle",True,"f-road-crews"),
 ("pl-saltmarket","Saltmarket","salt town","town","t-open","r-harbour",3.5,7.8,"Middle",False,"f-open-hand"),
 ("pl-wainyard","Wainyard","cart village","village","t-open","r-roads",8.6,7.8,"Low",False,"f-open-hand"),
 # Last Wave
 ("pl-textrim","Textrim's Height","stepped enclave","town","t-wave","r-rim",10.4,3.2,"High",True,"f-last-wave"),
 ("pl-proving","the Proving Ground","fortified camp","village","t-wave","r-rim",9.4,3.9,"Middle",True,"f-last-wave"),
 ("pl-ironstep","Ironstep","enclave village","village","t-wave","r-rim",11.0,4.4,"Middle",False,"f-last-wave"),
 # Highfolk
 ("pl-high-snows","the High Snows","cliff hold","village","t-high","r-snows",5.8,0.8,"High",True,"f-highfolk"),
 ("pl-womb","the Womb of Creation","hidden temple (SECRET)","site","t-high","r-snows",6.7,0.3,"Top",True,"f-highfolk"),
 # Overgrowth / Wardens
 ("pl-green-wall","the Green Wall","wall-fort","village","t-first","r-green",2.4,1.9,"Middle",True,"f-wardens"),
 ("pl-wellstone","Wellstone","lost village (in the green)","site","t-green","r-green",1.9,1.3,"Low",True,"f-wardens"),
 ("pl-soft-hollow","the Soft Hollow","Rooted village","village","t-green","r-green",1.3,0.9,"Low",True,"f-rooted"),
 ("pl-heart","the Heart","the Overgrowth's centre (SECRET)","site","t-green","r-green",0.6,0.4,"Top",True,"f-rooted"),
 # Far Sails
 ("pl-anchorage","the Anchorage of the Wide Bay","ring of anchored ships","village","t-sails","r-sea",0.1,7.4,"High",True,"f-far-sails"),
 ("pl-lantern-rock","the Lantern Rock","sea-rock tower","site","t-sails","r-sea",0.5,6.9,"Middle",True,"f-far-sails"),
]
# presences in the land (faction-places §5b), placed by their rules
L = [
 ("pr-shrine-vale","the Vale wayside shrine","wayside shrine","f-holy-house","t-first","r-vale",2.6,4.4),
 ("pr-shrine-crossing","the Crossroads shrine","wayside shrine","f-holy-house","t-great","r-crossing",4.9,4.6),
 ("pr-shrine-ruin","the Broken Shrine","wayside shrine (ruined)","f-holy-house","t-great","r-crossing",6.2,4.9),
 ("pr-ward-tower-1","Watchtower Ash","Warden watchtower","f-wardens","t-first","r-green",1.6,2.2),
 ("pr-ward-tower-2","Watchtower Thorn","Warden watchtower","f-wardens","t-first","r-green",3.2,1.6),
 ("pr-green-cairn","the Year Cairns","marker cairns","f-wardens","t-first","r-green",2.9,2.4),
 ("pr-road-fort","Brennock road fort","Lantern road fort","f-lantern","t-free","r-marches",7.9,6.2),
 ("pr-lamp-grave","Owl Cross lamp grave","lamp grave","f-lantern","t-great","r-crossing",6.7,4.4),
 ("pr-raid-camp","the Reedbank camp","Binder raid camp","f-binders","t-free","r-marches",9.6,6.0),
 ("pr-pens","the Sedge pens","Binder holding pen","f-binders","t-free","r-marches",9.2,6.7),
 ("pr-waystation","the Last Well","Walking Road waystation","f-binders","t-free","r-marches",8.1,5.9),
 ("pr-landing","Gull Landing","Far Sails landing","f-far-sails","t-first","r-reefcoast",1.4,7.2),
 ("pr-cove","Smugglers' Notch","smugglers' cove","f-night-market","t-open","r-harbour",2.6,8.4),
 ("pr-survey","the Edge survey camp","Writers survey camp","f-writers","t-first","r-green",2.0,2.6),
 ("pr-dig","the Old Road dig","Writers dig (a Lantern contract camp guards it)","f-writers","t-great","r-crossing",4.4,2.7),
 ("pr-challenge","the Four Ways stone","challenge stone","f-dueling","t-open","r-roads",6.6,7.3),
 ("pr-cairn-wave","the Rim watch cairn","Last Wave watch cairn","f-last-wave","t-wave","r-rim",9.0,3.2),
 ("pr-pass-hut","the Gap pass hut","Highfolk pass hut","f-highfolk","t-high","r-snows",5.5,1.6),
 ("pr-foothill","the Foothill post","Highfolk trading post","f-highfolk","t-great","r-snows",5.1,2.2),
 ("pr-grove","the Hush grove","Rooted listening grove","f-rooted","t-first","r-green",1.2,2.8),
 ("pr-crossing","the Quiet Crossing","Old Hands quiet crossing","f-old-hands","t-first","r-vale",3.6,4.0),
 ("pr-reed-crossing","the Reed Crossing","Old Hands quiet crossing past the Southroad toll line","f-old-hands","t-great","r-crossing",6.6,5.6),
 ("pr-hunting","the Ash Gully camp","Last Wave hunting camp","f-last-wave","t-wave","r-rim",10.0,4.6),
]
# in-town presences (share the town's quota)
TOWN = {
 "pl-highcrossing": ["Writers HQ: the Wayhouse", "Lantern chapter house", "Dueling Society ring (tavern yard)", "Old Hands back rooms (a lineage registry)", "Night Market contact (a card game)", "parish shrine", "Warden recruiting post"],
 "pl-openharbour": ["Night Market HQ: Under the Market Road", "Dueling Society HQ: the Ring", "Lantern chapter house", "Writers reading room", "Old Hands contact (an old family's factor)", "Far Sails pilots' hall", "Warden recruiting post"],
 "pl-ninth-ring": ["Old Hands HQ: the Old Gate House", "Writers reading room", "Lantern chapter house", "Dueling ring (tolerated by the house)", "parish shrine"],
 "pl-elderhollow": ["the Low Shrine (Aloqi)", "Old Hands back rooms (a pawn house)", "Writers letter-writer's stall", "Lantern contract board"],
 "pl-reefhold": ["Far Sails pilots' hall", "Old Hands back rooms (a lineage registry)", "Writers reading room", "Lantern chapter house", "Dueling ladder-keeper", "Night Market contact (a dockhand)"],
 "pl-longwall": ["Lantern chapter house", "Writers reading room", "Old Hands back rooms (a pawn house)", "Dueling ring (tolerated by the lord)", "parish shrine", "tithe-house"],
 "pl-southroad": ["Old Hands back rooms (a lineage registry)", "Writers reading room", "Lantern chapter house", "Night Market contact (a toll-dodge trail starts here)", "Dueling ladder-keeper", "parish shrine"],
 "pl-chainmarket": ["Binder debt broker", "Lantern chapter house", "Writers reading room", "Dueling ring (tavern yard)", "Night Market contact"],
 "pl-carters-rest": ["Lantern Company HQ: Lanternhall", "Writers reading room", "Dueling ring (tavern yard)", "Night Market contact (a card game)"],
 "pl-saltmarket": ["Night Market fence's shop", "Old Hands contact (an old family's factor)", "Dueling ring (tavern yard)", "Writers letter-writer's stall"],
 "pl-oathford": ["parish shrine", "Writers letter-writer's stall", "Lantern contract board", "Old Hands contact (a steward)", "Night Market contact (a toll-dodge trail)"],
 "pl-breakwall": ["Binder debt broker", "Lantern contract board", "Dueling ladder-keeper"],
 "pl-mount": ["the Mount (Holy House seat)", "tithe-house", "Writers letter-writer's stall"],
 "pl-crownsgate": ["Lantern contract board", "Old Hands contact (a steward)"],
 "pl-wainyard": ["Night Market contact (a dockhand)", "Dueling ladder-keeper"],
}
POOLS = {
 "f-first-peoples": (["a house grown so big it swallowed its neighbor","a sealed archive room","a stump-ring of a vanished house"],["records","stone milk","old-family goods"],["a forged lineage","a house going quiet","a Horaro family planning to leave"]),
 "f-great-house": (["an empty plinth","a lord's tower","a toll bridge"],["tolls","soldiers","grain"],["a lord backing the wrong heir","a slave-bought field","a forged imperial seal"]),
 "f-free-lords": (["an empty plinth","a lord's tower","a toll bridge"],["tolls","soldiers","grain"],["a lord backing the wrong heir","a slave-bought field","a forged imperial seal"]),
 "f-open-hand": (["a new quay","a caravan yard","a freed family's shop"],["trade goods","loans","hauling"],["a debt bought up by the Sharp Hands","a strike brewing","a Binder contract hidden in a deal"]),
 "f-last-wave": (["a dark-stone step-hall","a duel circle","a smoke hall"],["forge salt","weapons","guards"],["a hungry winter","a raid planned","a mixed marriage"]),
}
SIZE_QUOTA = {"great": (10,15), "large": (6,10), "town": (3,5), "village": (1,2), "site": (0,1)}

places = []
for (pid,name,kind,size,t,r,x,y,st,fixed,owner) in P:
    d = {"id":pid,"name":name,"kind":kind,"size":size,"territory":t,"region":r,"x":x,"y":y,"standing":st,
         "canon":fixed,"owner_faction":owner,"quota":SIZE_QUOTA[size],"presences_in_town":TOWN.get(pid,[])}
    if "SECRET" in kind: d["secret"]=True; d["quota"]=(0,0)
    if not fixed and owner in POOLS:
        lm,tr,hk = POOLS[owner]
        d["landmark"]=random.choice(lm); d["trade"]=random.choice(tr); d["hook"]=random.choice(hk)
    else:
        d["details_from"]="faction-places.md (fixed place: landmark, trade, hook)"
    places.append(d)
for (pid,name,kind,fac,t,r,x,y) in L:
    places.append({"id":pid,"name":name,"kind":kind,"size":"presence","territory":t,"region":r,"x":x,"y":y,
                   "standing":"Low","canon":False,"owner_faction":fac,"quota":(0,1),"presence_of":fac,
                   "details_from":"faction-places.md §5b (roll a condition and a nearby detail)"})
def dist(a,b): return math.hypot(a["x"]-b["x"], a["y"]-b["y"])
for a in places:
    a["near"] = sorted(b["id"] for b in places if b is not a and dist(a,b) <= 1.25 and not b.get("secret"))
roads = [("pl-highcrossing","pl-southroad"),("pl-southroad","pl-counting-house"),("pl-counting-house","pl-openharbour"),
 ("pl-highcrossing","pl-oathford"),("pl-oathford","pl-elderhollow"),("pl-elderhollow","pl-ninth-ring"),("pl-ninth-ring","pl-reefhold"),
 ("pl-highcrossing","pl-crownsgate"),("pl-crownsgate","pl-longwall"),("pl-longwall","pl-breakwall"),("pl-breakwall","pl-chainmarket"),
 ("pl-counting-house","pl-carters-rest"),("pl-carters-rest","pl-wainyard"),("pl-openharbour","pl-saltmarket"),
 ("pl-carters-rest","pl-chainmarket"),("pl-ninth-ring","pl-green-wall"),("pl-ninth-ring","pl-firststone"),("pl-highcrossing","pr-foothill"),
 ("pl-breakwall","pl-proving"),("pl-proving","pl-textrim"),("pl-mount","pl-ninth-ring"),("pl-mount","pl-oathford")]
walking_road = ["pr-pens","pr-raid-camp","pr-waystation","pl-chainmarket","pl-breakwall"]
out = {"assumption":"Hand-placed test island (world assumption, not land-generator output). Placement follows faction-places.md §1. 1 unit ≈ one day's travel; 'near' = within 1.25 units.",
 "size_days":"about 11 days across, 9 days top to bottom",
 "regions":[{"id":k,"name":v} for k,v in regions.items()],
 "territories":[{"id":k,"holder":v[0],"sub":v[1]} for k,v in territories.items()],
 "places":places,"roads":roads,"walking_road":walking_road,
 "notes":["Southroad is the [Dir]road toll town: the busiest road out of Highcrossing runs south to the coast.",
          "The Walking Road runs from the Marches' bog camps into Free Lords land and never crosses Open Hand land.",
          "Secret places (the Womb, the Heart) get no quests in tests."]}
json.dump(out, open(__import__("sys").argv[1] if len(__import__("sys").argv)>1 else "map.json","w"), indent=1, ensure_ascii=False)
q = [p["quota"] for p in places]
print(len(places), "places;", "quota range", sum(a for a,b in q), "-", sum(b for a,b in q))
