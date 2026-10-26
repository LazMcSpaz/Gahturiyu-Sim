#!/usr/bin/env python3
"""Shared canon registry for the world tests: factions and pre-made leaders (from factions.md / leaders.md).
Every test agent reuses these ids. Writes canon.json."""
import json
F = [  # id, name, size, parent, leaders, seat, reach, want
 ("f-first-peoples","The First Peoples","major",None,["p-tuhudo"],"pl-ninth-ring","realm","preservation"),
 ("f-seated-houses","The Seated Houses","sub","f-first-peoples",["p-tuhudo"],"pl-ninth-ring","region","preservation"),
 ("f-true-line","The True Line","sub","f-first-peoples",["p-garotaqa","p-ruo"],"pl-firststone","region","ambition"),
 ("f-tide-houses","The Tide Houses","sub","f-first-peoples",["p-niralea"],"pl-reefhold","region","preservation"),
 ("f-great-house","The Great House","major",None,["p-gahiru"],"pl-highcrossing","realm","ambition"),
 ("f-throne","The Throne","sub","f-great-house",["p-litita"],"pl-highcrossing","realm","preservation"),
 ("f-banner-lords","The Banner Lords","sub","f-great-house",["p-haahoqo","p-shahayuh"],"pl-longwall","region","ambition"),
 ("f-free-lords","The Free Lords","sub","f-great-house",["p-kotqot"],"pl-breakwall","region","ambition"),
 ("f-open-hand","The Open Hand","major",None,["p-waarua"],"pl-openharbour","realm","ambition"),
 ("f-open-council","The Open Council","sub","f-open-hand",["p-waarua"],"pl-openharbour","region","ambition"),
 ("f-sharp-hands","The Sharp Hands","sub","f-open-hand",["p-feyesh","p-dorog","p-kodor"],"pl-counting-house","region","greed"),
 ("f-road-crews","The Road Crews","sub","f-open-hand",["p-doruduqu"],"pl-carters-rest","region","preservation"),
 ("f-holy-house","The Holy House","minor",None,["p-hotorito","p-aloqi"],"pl-mount","region","preservation"),
 ("f-last-wave","The Last Wave","minor",None,["p-textrim","p-totkit"],"pl-textrim","region","desperation"),
 ("f-far-sails","The Far Sails","minor",None,["p-nemu"],"pl-anchorage","region","preservation"),
 ("f-binders","The Binders","minor",None,["p-paqax"],"pl-chainmarket","region","ambition"),
 ("f-wardens","The Wardens","minor",None,["p-hao","p-hoqa"],"pl-green-wall","community","desperation"),
 ("f-highfolk","The Highfolk","minor",None,["p-shihi"],"pl-high-snows","community","preservation"),
 ("f-rooted","The Rooted","minor",None,[],"pl-soft-hollow","region","ambition"),
 ("f-writers","The Writers","guild",None,["p-fiyeweya"],"pl-highcrossing","realm","preservation"),
 ("f-lantern","The Lantern Company","guild",None,["p-lamawina"],"pl-lanternhall","realm","preservation"),
 ("f-old-hands","The Old Hands","guild",None,["p-yitolao"],"pl-ninth-ring","region","preservation"),
 ("f-night-market","The Night Market","guild",None,["p-yaweth"],"pl-openharbour","region","greed"),
 ("f-dueling","The Dueling Society","guild",None,["p-lio","p-kitim"],"pl-openharbour","community","ambition"),
]
REL = [("f-first-peoples","rival","f-great-house"),("f-first-peoples","rival","f-open-hand"),("f-great-house","rival","f-open-hand"),
 ("f-holy-house","ally","f-first-peoples"),("f-last-wave","at war","f-first-peoples"),("f-binders","serves","f-free-lords"),
 ("f-binders","rival","f-open-hand"),("f-old-hands","rival","f-night-market"),("f-far-sails","rival","f-tide-houses"),
 ("f-banner-lords","rival","f-free-lords"),("f-sharp-hands","rival","f-road-crews"),("f-highfolk","rival","f-great-house")]
P = [  # id, name, standing, faction, sub, home, traits
 ("p-tuhudo","Tuhuḍo of the Ninth-Ring House","High","f-first-peoples","f-seated-houses","pl-ninth-ring",["blind archivist-lord","sharp memory"]),
 ("p-garotaqa","Garoṭaqa of the First-Stone House","High","f-first-peoples","f-true-line","pl-firststone",["schemer","charismatic"]),
 ("p-ruo","Ruʻo of the First-Stone House","High","f-first-peoples","f-true-line","pl-firststone",["the face of the True Line"]),
 ("p-niralea","Niralea, daughter of the Reef Line","High","f-first-peoples","f-tide-houses","pl-reefhold",["season's speaker","divided loyalties"]),
 ("p-gahiru","Gahìru of the Great House","Top","f-great-house","f-throne","pl-highcrossing",["the emperor","fading"]),
 ("p-litita","Litiṭaʻa of the Great House","High","f-great-house","f-throne","pl-highcrossing",["masterful liar","lonely"]),
 ("p-haahoqo","Haʻahoqo the General","High","f-great-house","f-banner-lords","pl-longwall",["old general","warm to commoners"]),
 ("p-shahayuh","Shahayuh of the South Road","High","f-great-house","f-banner-lords","pl-southroad",["Ṭaḍoro-born lord","social climber"]),
 ("p-kotqot","Kotqot Wall-breaker","High","f-great-house","f-free-lords","pl-breakwall",["paranoid","known fighter"]),
 ("p-waarua","Waarua of the Harbour","High","f-open-hand","f-open-council","pl-openharbour",["freed bondswoman","inspiring speaker"]),
 ("p-feyesh","Feyesh of the Silver Road","High","f-open-hand","f-sharp-hands","pl-counting-house",["broker","flatterer"]),
 ("p-dorog","Dorog Coin-counter","High","f-open-hand","f-sharp-hands","pl-counting-house",["twin","smiles and deals"]),
 ("p-kodor","Kodor Debt-taker","High","f-open-hand","f-sharp-hands","pl-counting-house",["twin","counts and threatens"]),
 ("p-doruduqu","Doruḍoqu the Carter","Middle","f-open-hand","f-road-crews","pl-carters-rest",["elected foreman","knows every road"]),
 ("p-hotorito","Hotorìṭo the High Priest","High","f-holy-house",None,"pl-mount",["militant","political"]),
 ("p-aloqi","Aloqi the Priestess","Middle","f-holy-house",None,"pl-elderhollow",["sincere","old"]),
 ("p-textrim","Textrim, High Priestess","High","f-last-wave",None,"pl-textrim",["protective","pragmatic"]),
 ("p-totkit","Totkit Sea-crosser","High","f-last-wave",None,"pl-proving",["war-chief","known fighter"]),
 ("p-nemu","Nemu of the Wide Bay","High","f-far-sails",None,"pl-anchorage",["fleet speaker","homesick"]),
 ("p-paqax","Paqax Chain-breaker","High","f-binders",None,"pl-chainmarket",["ex-slave slaver","self-justifying"]),
 ("p-hao","Haʻo of the Green Wall","Middle","f-wardens",None,"pl-green-wall",["talks at the gate","unbreakable"]),
 ("p-hoqa","Hoqa of the Green Wall","Middle","f-wardens",None,"pl-green-wall",["scouts the green","knows more than she says"]),
 ("p-shihi","Shihi of the High Snows","High","f-highfolk",None,"pl-high-snows",["hospitable","archaic"]),
 ("p-fiyeweya","Fiyeweya of No Road","High","f-writers",None,"pl-highcrossing",["always travelling","puppeteer"]),
 ("p-lamawina","Lamawina of the Lantern","High","f-lantern",None,"pl-lanternhall",["quartermaster","lives by the code"]),
 ("p-yitolao","Yitolaʻo of the Old Gate House","High","f-old-hands",None,"pl-ninth-ring",["politely menacing"]),
 ("p-yaweth","Yaweth of the Market Road","High","f-night-market",None,"pl-openharbour",["cheeky","generous"]),
 ("p-lio","Liʻo of the Ring","High","f-dueling",None,"pl-openharbour",["showman"]),
 ("p-kitim","Kitim Duel-ender","High","f-dueling",None,"pl-openharbour",["known fighter","once killed in a duel"]),
]
factions=[{"id":i,"name":n,"size":s,"parent":p,"leaders":l,"seat":seat,"reach":r,"want":w,
           "relations":[{"type":t,"to":b} for a,t,b in REL if a==i]+[{"type":t,"to":a} for a,t,b in REL if b==i and t in ("rival","at war")],
           "facts":[]} for (i,n,s,p,l,seat,r,w) in F]
people=[{"id":i,"name":n,"standing":st,"traits":tr,"links":[],"faction":{"id":f,"sub":sub},"facts":[],"premade":True,"dead":False,"home":h,"fallback":None}
        for (i,n,st,f,sub,h,tr) in P]
json.dump({"note":"Canon factions and pre-made leaders. Top roster: Gahìru only (pre-made). Reuse these ids; never invent leaders.",
           "factions":factions,"people":people},open("canon.json","w"),indent=1,ensure_ascii=False)
print(len(factions),"factions",len(people),"leaders")
