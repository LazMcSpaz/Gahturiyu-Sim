#!/usr/bin/env python3
"""Roll the six guild story kits for one world (guild-kits.md v6c + v6d; faction-stories.md §9-§12, §11a),
then deal each story its chapter-2 blow (target + event + how it fell + door), as the placement plan does (§12 step 2).

The kit data is read from guild-kits.md itself (every *Story kit* table: enemies, stakes, hearts and their
`member`/`outside` tags, moves, odd jobs and their `fits`, crises with their `heart` tags and variants, endings,
chapter-2 blows, per-enemy blows, the paired feud, the readings by enemy and variant), and the doors by how
the blow fell from assembly.md §4, so the roller's tables are the docs' tables by construction; `--check`
prints what it read. What isn't a table (which hall a seized hall can be, how a leader is struck) is copied
below from each kit's compatibility notes. The sentence wording (PHRASE) is the roller's own, written from the
kits' readings; where a kit gives a reading by enemy or variant, the kit's words are used.

Usage: python3 roll_kits_d.py <seed> <paired|unpaired|auto> [ledger.json] > rolls.json
       python3 roll_kits_d.py --check            (parse the doc, list every legal enemy + crisis pair)
       python3 roll_kits_d.py --stats 1 400      (roll seeds 1-400, auto pairing, print the shares)
       python3 roll_kits_d.py --joins 1 400      (roll seeds 1-400 and count breaks of the v6d joins, rules 1-8)
Ledger (all optional): {"crisis_premises": [["Reckoning","Betrayal"], ...], "levers": [...], "main_lever": "debt",
  "main_ordeal": "estranged",
  "recent": [ {"writers": {"enemy": "W1", "crisis": "C3", "odd_job": "O2"}, ...},   # the older world
              {...} ]}                                                            # the last world
`recent` (mechanics §13, *Fresh stories*): those entries are unavailable unless nothing else fits;
when nothing else fits, the older world's come back first."""
import json, os, random, re, sys, hashlib
from collections import Counter, defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
DOC = os.path.normpath(os.path.join(HERE, "..", "..", "guild-kits.md"))
ASM = os.path.normpath(os.path.join(HERE, "..", "..", "assembly.md"))
ORDER = ["writers", "lantern", "old-hands", "night-market", "dueling", "holy-house"]
HEADS = {"writers": "## 1. The Writers", "lantern": "## 2. The Lantern Company", "old-hands": "## 3. The Old Hands",
         "night-market": "## 4. The Night Market", "dueling": "## 5. The Dueling Society", "holy-house": "## 6. The Holy House"}
SIDE_OF = {"writers": "Writers", "lantern": "Lantern Company", "old-hands": "Old Hands", "night-market": "Night Market",
           "dueling": "Dueling Society", "holy-house": "Holy House"}
SHAPES = ("Within", "Above", "Beside")
REACH = {"local": 0, "region": 1, "realm": 2}
V = "abc"

# ---- faction-stories §11a, *What each blow needs*: the hits of each target + event -------------------------------
BLOW_HITS = {("heart", "captive"): {"people"}, ("heart", "disgraced"): {"people", "name"}, ("heart", "estranged"): {"people"},
             ("player", "framed"): {"people", "name"}, ("player", "stripped of rank"): {"people"}, ("player", "arrested"): {"people"},
             ("guild", "hall seized"): {"place"}, ("guild", "members taken"): {"people"},
             ("guild", "outlawed"): {"trade", "place"}, ("guild", "leader struck down"): {"people"}}
# The doors are read from assembly §4, *Doors by how the blow fell* (v6d): load_hows() below. Each row is how
# the blow fell; it sets the blow's means, its doors and what fills them.
STRIKE_ROW = {"killed": "killed", "wounded-men": "wounded", "wounded-rigged": "wounded", "disgraced": "disgraced"}
TARGET_CAP = (1, 3)

# ---- copied from each kit's compatibility notes (not tables) -----------------------------------------------------
LEADERS = {"writers": {"the leader"}, "lantern": {"the quartermaster"}, "old-hands": {"the old don"},
           "night-market": {"the leader"}, "dueling": {"the showman", "the sister"}, "holy-house": {"the high priest", "the priestess"}}
LEADER_ENEMY = {("writers", "W6"): "the leader", ("old-hands", "OH1"): "the old don", ("night-market", "N5"): "the leader",
                ("dueling", "D3"): "the showman", ("holy-house", "HH1"): "the high priest"}
GUILD_OF = {v: k for k, v in SIDE_OF.items()}
HALLS = {"writers": (["a reading room", "the Wayhouse"], lambda c, s: c == "C5" or (c == "C8" and s == "S1")),
         "lantern": (["a chapter house", "Lanternhall"], lambda c, s: c in ("C1", "C6")),
         "old-hands": (["a back room", "a cache", "the Old Gate House"], lambda c, s: c == "C5"),
         "night-market": (["a fence's shop", "Under the Market Road"], lambda c, s: c == "C2" or (c == "C7" and s == "S1")),
         "dueling": (["a tavern-yard ring", "the Ring"], lambda c, s: c in ("C1", "C5", "C7")),
         "holy-house": (["a chapel", "a parish shrine", "the Mount"], lambda c, s: c == "C4")}
CODE = {"writers": ["destroying a text", "taking a side in public"], "lantern": ["abandoning a contract", "abandoning a comrade"],
        "old-hands": ["going to the law", "shaming a house"], "night-market": ["informing", "holding back the take"],
        "dueling": ["throwing a bout"], "holy-house": ["shaming the house in public"]}

def strike_options(g, eid, crisis, giver):
    """Who may be struck down, and how (each kit's *leader struck down* row). [(leader, how, successor)]"""
    out = []
    if g == "writers":
        if eid != "W6": out += [("the leader", "wounded"), ("the leader", "disgraced")]
    elif g == "lantern":
        out += [("the quartermaster", "wounded"), ("the quartermaster", "disgraced")]
        if crisis != "C4" and giver != "the quartermaster": out.append(("the quartermaster", "killed"))
    elif g == "old-hands":
        if eid != "OH1": out += [("the old don", "wounded")] + ([] if crisis in ("C6", "C8") else [("the old don", "disgraced")])
    elif g == "night-market":
        if eid != "N5": out += [("the leader", "wounded")] + ([] if crisis == "C8" else [("the leader", "disgraced")])
    elif g == "dueling":
        sh = {"D1": "wounded", "D6": "wounded", "D8": "wounded", "D2": "disgraced", "D4": "disgraced"}.get(eid)
        if sh: out.append(("the showman", sh))
        if eid in ("D1", "D2", "D6", "D8"): out.append(("the sister", "wounded"))
    elif g == "holy-house":
        hp = {"HH3": ["wounded", "disgraced", "killed"], "HH8": ["wounded", "killed"], "HH9": ["disgraced"]}.get(eid, [])
        out += [("the high priest", h) for h in hp if not (h == "killed" and giver == "the high priest")]
        ps = {"HH1": "disgraced", "HH2": "disgraced", "HH6": "disgraced", "HH7": "disgraced", "HH5": "wounded"}.get(eid)
        if ps: out.append(("the priestess", ps))
    succ = {"lantern": "the eldest chapter captain" if eid == "L3" else "her deputy", "holy-house": "the Mount's eldest priest"}
    return [(l, h, succ.get(g) if h == "killed" else None) for l, h in out]

# ================================================================== parsing guild-kits.md
def cells(line):
    return [c.strip() for c in line.strip().strip("|").split("|")]

def tables_after(text, marker):
    """The first markdown table after `marker` in `text`, as a list of dicts keyed by header."""
    i = text.index(marker); lines = text[i:].split("\n")
    j = next(k for k, l in enumerate(lines) if l.lstrip().startswith("|"))
    rows = []
    for l in lines[j:]:
        if not l.lstrip().startswith("|"): break
        rows.append(cells(l))
    head = rows[0]
    return [dict(zip(head, r)) for r in rows[2:]]

def col(row, *names):
    for n in names:
        for k in row:
            if k.lower().startswith(n.lower()): return row[k]
    return ""

def eid(cell): return re.match(r"([A-Z]+\d+)", cell).group(1)
def ticks(cell): return re.findall(r"`([a-z]+)`", cell)
def ids(cell): return set(re.findall(r"\b([A-Z]{1,2}\d+)\b", cell))

def top_split(text, seps):
    out, depth, cur, i = [], 0, "", 0
    while i < len(text):
        ch = text[i]
        if ch == "(": depth += 1
        elif ch == ")": depth -= 1
        hit = next((s for s in seps if depth == 0 and text.startswith(s, i)), None)
        if hit: out.append(cur); cur = ""; i += len(hit); continue
        cur += ch; i += 1
    out.append(cur); return out

def parse_cond(txt):
    """A bracket's limits: shapes or enemy ids (the enemy must be one), `region`, `never ...` (it must be none)."""
    cond = dict(pos=set(), neg=set(), region=False)
    for grp in txt.split(";"):
        grp = re.sub(r"\([^)]*\)", "", grp).strip()
        toks = ids(grp) | {x for x in SHAPES if x in grp}
        if grp.startswith("never"): cond["neg"] |= toks
        elif grp == "region": cond["region"] = True
        else: cond["pos"] |= toks
    return cond

def is_cond(txt):
    return not ticks(txt) and bool(ids(txt) or any(x in txt for x in SHAPES + ("region", "never")))

def parse_needs(cell):
    """`a` or `b` + `c` (Above) ...; whole-cell limits after a top-level ';'. None = no means (the truce letter's)."""
    if not ticks(cell): return None
    parts = top_split(cell, [";"])
    whole = parse_cond(";".join(p for p in parts[1:] if p.strip().startswith("never")))
    alts = []
    for a in top_split(parts[0], [" or "]):
        m = re.search(r"\(([^)]*)\)\s*$", a.strip())
        cond = parse_cond(m.group(1)) if m and is_cond(m.group(1)) else dict(pos=set(), neg=set(), region=False)
        means = frozenset(ticks(a))
        if not means: continue
        cond = dict(cond); cond["neg"] = cond["neg"] | whole["neg"]
        alts.append(dict(means=means, **cond))
    return alts

def alt_ok(a, e):
    if not a["means"] <= e["means"]: return False
    if a["pos"] and not (e["shape"] in a["pos"] or e["id"] in a["pos"]): return False
    if e["shape"] in a["neg"] or e["id"] in a["neg"]: return False
    if a["region"] and REACH[e["reach"]] < 1: return False
    return True

def met(needs, e):
    """The alternatives of `needs` the enemy meets."""
    return [a for a in (needs or []) if alt_ok(a, e)]

def parse_shapes(cell):
    c = cell.strip().lower()
    return set(SHAPES) if c in ("any", "") else {s for s in SHAPES if s.lower() in c}

def parse_limits(cell):
    L = dict(wants=None, never=set(), only=set(), nocost=False, mode="unpaired", stake_wants={}, shuts=False)
    for item in cell.split(";"):
        it = item.strip()
        if not it: continue
        if it.startswith("shuts a place"): L["shuts"] = True; continue          # v6d: a hall isn't seized after it
        m = re.match(r"on ((?:S\d+,?\s*)+): wants (.*)", it)
        if m:
            for s in re.findall(r"S\d+", m.group(1)): L["stake_wants"][s] = {w.strip() for w in m.group(2).split(",")}
        elif it.startswith("wants "): L["wants"] = {w.strip() for w in it[6:].split(",")}
        elif it.startswith("never "): L["never"] |= ids(it) | {s for s in SHAPES if s in it}
        elif it.startswith("only "): L["only"] |= ids(it)
        elif it.startswith("not under the cost"): L["nocost"] = True
        elif it.startswith("paired only"): L["mode"] = "paired"
        elif it.startswith("also paired"): L["mode"] = "both"
    return L

def parse_givers(text, cid):
    text = text[text.index("*Variants (last move"):]
    m = re.search(r"^- \*\*" + cid + r"(?::\*\*|\*\*[^:\n]*:)\s*(.*)$", text, re.M)
    if not m: raise SystemExit(f"no variants for {cid}")
    body = m.group(1)
    found = re.findall(r"\*([abc])\* (.*?) · (.*?)(?= · \*[abc]\* |$)", body)
    out = []
    for v, text, giver in found:
        giver = giver.strip().rstrip(".")
        cond = {}
        mc = re.match(r"^(.*?) \((on|hearts) ([^)]*)\)$", giver)                 # v6d: a giver's tie to the story
        if mc:
            giver = mc.group(1)
            cond["_on" if mc.group(2) == "on" else "_hearts"] = set(re.findall(r"[SH]\d+", mc.group(3)))
        mm = re.match(r"^(.*?) \((.*) under (\w+)\)$", giver)
        d = {"*": mm.group(1), mm.group(3): mm.group(2)} if mm else {"*": giver}
        d.update(cond); d["_text"] = text
        out.append(d)
    if len(out) != 3: raise SystemExit(f"{cid}: {len(out)} givers parsed from {body[:80]}")
    return out

def parse_hearts_cell(cell):
    """v6d `heart` tag: None (any) or {heart id: set of variant letters, empty = every variant}; and *herself*."""
    herself = "herself" in cell
    c = cell.split(";")[0].strip()
    if c.lower().startswith("any"): return None, herself
    fit = defaultdict(set)
    for h, v in re.findall(r"\b(H\d+)([abc]?)\b", c):
        fit[h]  # present
        if v: fit[h].add(v)
    return dict(fit), herself

def parse_readings(kit):
    """v6d *Readings by enemy and variant*: {"W1c": {"scheme": ..., "C3 on S7": ...}, "OH9": {"M8": ...}}."""
    out = {}
    head = "*Readings by enemy and variant* *(Claude, v6d)*:"
    if head not in kit: return out
    blk = kit[kit.index(head):].split("\n\n")[0]
    for line in blk.split("\n")[1:]:
        m = re.match(r"^- \*\*([A-Z]+\d+[abc]?)\*\*(?: \([^)]*\))?:\s*(.*)$", line)
        if not m: raise SystemExit(f"readings: can't read {line[:60]}")
        out[m.group(1)] = {k: t.strip() for k, t in re.findall(r"(scheme|[MC]\d+(?: on S\d+)?) · \*([^*]+)\*", m.group(2))}
    return out

def section(doc, g):
    i = doc.index(HEADS[g]); nxt = [doc.index(h) for h in HEADS.values() if doc.index(h) > i] + [doc.index("## 7. Notes")]
    return doc[i:min(nxt)]

def parse_kit(doc, g):
    sec = section(doc, g); kit = sec[sec.index("### Story kit"):]
    w = re.search(r"\*\*Weights\*\* \| ([^\n]*)", sec).group(1)
    never = set(re.findall(r"never: (\w+)", w))
    K = dict(g=g, never_fam=never, enemies={}, stakes={}, hearts={}, heart_member={}, moves={}, odd={}, crises={}, endings=[], seat=None,
             blows={}, per_enemy={}, readings=parse_readings(kit))
    for r in tables_after(kit, "**Enemies:**"):
        i = eid(r["#"]); who = col(r, "Who")
        side = col(r, "Side").strip()
        side = None if side.lower().startswith("none") else ("LORD" if "Great House lord" in side else side)
        K["enemies"][i] = dict(id=i, shape=col(r, "Shape").strip(), who=who, leader="**" in who, want=col(r, "Want").strip(),
                               means=frozenset(ticks(col(r, "Means"))), reach=col(r, "Reach").strip(),
                               reaches=sorted(re.findall(r"S\d+", col(r, "Reaches"))), scheme=col(r, "Scheme"),
                               kind=col(r, "Kind").strip(), side=side, paired="yes" in col(r, "Paired").lower())
    for r in tables_after(kit, "**Stakes:**"):
        i = eid(r["#"]); K["stakes"][i] = dict(hit=ticks(r["Hit"])[0], needs=frozenset(ticks(r["Needs"])),
                                               reputation="reputation" in r["Entry"], name=re.sub(r"\*", "", r["Stake"]))
    hs = kit[kit.index("**Hearts:**"):]
    for m in re.finditer(r"^- \*\*(H\d+), ([^:*]+?):?\*\*(.*)$", hs, re.M):
        K["hearts"][m.group(1)] = m.group(2).strip()
        tag = re.search(r"`(member|outside)`", m.group(3))
        if not tag: raise SystemExit(f"{g} {m.group(1)}: no member/outside tag")
        K["heart_member"][m.group(1)] = tag.group(1) == "member"
    for r in tables_after(kit, "**Chapter-1 moves**"):
        i = eid(r["#"]); a = r["Aims"].strip()
        K["moves"][i] = dict(id=i, needs=parse_needs(r["Needs"]), hits=set(ticks(r["Hits"])),
                             aims="any" if a == "any" else set(re.findall(r"S\d+", a)), shapes=parse_shapes(r["Shapes"]),
                             cost=r["Cost"].strip() == "yes", writ=r["Writ"].strip().startswith("yes"), limits=parse_limits(r["Limits"]),
                             act=re.sub(r"\*\*([^*]*):\*\* ", "", r["Public act"]).replace("**", ""))
    for r in tables_after(kit, "**Odd jobs:**"):
        i = eid(r["#"]); prem = re.search(r"\*([^*]+)\*", r["Premise"]).group(1)
        K["odd"][i] = dict(premise=r["Family"].strip() + "/" + prem, thread=(re.findall(r"`([a-z-]+)`", r["Thread"]) or [""])[0],
                           needs=parse_needs(r["Needs"]), fits=ids(r["Fits"]))
    cr = kit[kit.index("**Crises:**"):]
    for r in tables_after(kit, "**Crises:**"):
        i = eid(r["#"]); c = r["Crisis"]; d = r["Decides"].strip()
        k = r["Kills"].lower()
        hfit, herself = parse_hearts_cell(r["Heart"])
        K["crises"][i] = dict(id=i, family=c.split()[0], premise=re.search(r"\*([^*]+)\*", c).group(1), needs=parse_needs(r["Needs"]),
                              hearts=hfit, herself=herself,
                              hits=set(ticks(r["Hits"])), decides="any" if d == "any" else set(re.findall(r"S\d+", d)),
                              shapes=parse_shapes(r["Shapes"]), limits=parse_limits(r["Limits"]), writ=r["Writ"].strip() == "yes",
                              kills="shared" if "shared" in k else ("yes" if "yes" in k else "no"),
                              givers=parse_givers(cr, i), last=r["Last move"])
    for r in tables_after(kit, "**Faction endings:**"):
        e = r["Ending"]; m = re.search(r"\(seat ending: (\w+) only", e)
        label = re.split(r" \*\(Claude\)\*| \(|:", e)[0].strip()
        if m: K["seat"] = (m.group(1), label); continue
        f = r["Fits"]
        K["endings"].append(dict(label=label, hits=set(ticks(f)), fams=set(re.findall(r"\b(Acquire|Contest|Dispute|Investigate|Passage|Reckoning|Rescue|Restore|Threat)\b", f)),
                                 crises=set(re.findall(r"\bC\d+\b", f)), any="any" in f, paired="paired" in f, never=ids(f.split("never")[1]) if "never" in f else set()))
    for r in tables_after(kit, "**Chapter-2 blows"):
        t = r["Target"].replace("the ", "").strip(); ev = r["Event"].strip("* ")
        K["blows"][(t, ev)] = dict(needs=parse_needs(r["Needs"]), here=r["Here it is"])
    if "*Per-enemy blows*" in kit:
        pe = kit[kit.index("*Per-enemy blows*"):].split("\n\n")[0]
        for m in re.finditer(r"^- \*\*(\w+)(?::\*\*|\*\* [^:]*:) (.*)$", pe, re.M):
            bl = {}
            for item, rest in re.findall(r"\*([a-z ]+)\*( \(`[a-z]+`)?", m.group(2)):
                w = item.split(" ", 1)
                key = (w[0], w[1]) if w[0] in ("heart", "player") else ("guild", item)
                bl[key] = set(ticks(rest)) or None                                  # v6d: a means the blow is carried by
            K["per_enemy"][m.group(1)] = bl
    if "**The paired feud**" in kit:
        pf = kit[kit.index("**The paired feud**"):]
        ft = tables_after(pf, "2. **Over what.**")
        K["feud"] = {eid(r["#"]): ids(r["First striker may be"]) for r in ft}
        K["feud_ans"] = {eid(r["#"]): ids(col(r, "Answered by")) for r in ft}           # v6d: the feud fits both schemes
        K["truce"] = {m.group(1): frozenset(ticks(m.group(2))) for m in re.finditer(r"\*([abc])\* [^(·]*\(([^)]*)\)", pf[pf.index("4. **How it ended**"):].split("\n")[0])}
        K["callers"] = re.findall(r"\*(T\d)\*", pf[pf.index("3. **Who called the truce**"):].split("\n")[0])
    return K

def load_hows(path=ASM):
    """assembly §4, *Doors by how the blow fell*: {(target, event): [row]}, each row its how, means, doors and fills."""
    doc = open(path, encoding="utf-8").read()
    out = defaultdict(list)
    for r in tables_after(doc, "**Doors by how the blow fell**"):
        doors = []
        for d in top_split(r["Doors (and what fills them)"], [";"]):
            m = re.match(r"\s*(\w+) \*([^*]+)\*(?: \((.*)\))?\s*$", d)
            fill, only = (m.group(3) or ""), None
            mo = re.match(r"([A-Z][\w ]+?) only;\s*(.*)$", fill)
            if mo: only, fill = GUILD_OF[mo.group(1)], mo.group(2)
            doors.append(dict(family=m.group(1), premise=m.group(2), fill=fill, only=only))
        out[(r["Target"].strip(), r["Event"].strip("* "))].append(
            dict(key=r["Key"].strip(), how=r["How it fell"].strip(), needs=parse_needs(r["Means"]), doors=doors))
    return dict(out)

HOWS = load_hows()

def load(path=DOC):
    doc = open(path, encoding="utf-8").read()
    return {g: parse_kit(doc, g) for g in ORDER}

# ================================================================== the throughline at the kit roll (§11a points 2, 3, 5)
def writ_uses(alts, writ):
    """`uses`: the means a part is carried out with (a keyed roll among them). On a writ part, a choice that takes
    `paper` (or the house's `faith`) is used only when no other choice fits, and then the part is a writ."""
    if writ:
        free = [a for a in alts if not a["means"] & {"paper", "faith"}]
        if free: return sorted({m for a in free for m in a["means"]}), False
        return sorted({m for a in alts for m in a["means"] & {"paper", "faith"}}), bool(alts)
    return sorted({m for a in alts for m in a["means"]}), False

def move_ok(K, e, sid, m):
    if e["shape"] not in m["shapes"]: return None
    L = m["limits"]
    if e["id"] in L["never"] or (L["only"] and e["id"] not in L["only"]): return None
    if L["wants"] and e["want"] not in L["wants"]: return None
    if sid in L["stake_wants"] and e["want"] not in L["stake_wants"][sid]: return None
    if e["kind"] == "the cost" and not m["cost"]: return None
    if m["aims"] == "any":
        if "people" not in m["hits"]: return None
    elif sid not in m["aims"]: return None
    if K["stakes"][sid]["hit"] not in m["hits"] and "people" not in m["hits"]: return None   # point 3
    alts = met(m["needs"], e)
    if not alts: return None                                                                 # point 2
    return writ_uses(alts, m["writ"])

def crisis_ok(K, e, sid, c, paired):
    L = c["limits"]
    if paired and L["mode"] == "unpaired": return None
    if not paired and L["mode"] == "paired": return None
    if e["shape"] not in c["shapes"]: return None
    if e["id"] in L["never"] or (L["only"] and e["id"] not in L["only"]): return None
    if L["wants"] and e["want"] not in L["wants"]: return None
    if L["nocost"] and e["kind"] == "the cost": return None
    dec = set(e["reaches"]) if c["decides"] == "any" else c["decides"]
    if sid not in dec or K["stakes"][sid]["hit"] not in c["hits"]: return None              # point 4: decides the stake
    if paired: return ["truce"], False
    alts = met(c["needs"], e)
    if not alts: return None
    return writ_uses(alts, c["writ"])

def stake_ok(K, e, sid):
    return sid in e["reaches"] and bool(K["stakes"][sid]["needs"] & e["means"])

def endings_fit(K, eid_, sid, c, paired):
    hit = K["stakes"][sid]["hit"]
    out = []
    for en in K["endings"]:
        if eid_ in en["never"]: continue
        if en["paired"] and not paired: continue
        if en["any"] or hit in en["hits"] or c["family"] in en["fams"] or c["id"] in en["crises"] or (en["paired"] and paired):
            out.append(en["label"])
    return out

def blow_list(K, e, sid, crisis, gvr, writ_left, mid=None, heart=None):
    """Every legal chapter-2 blow for this story: (target, event, how, uses, is_writ, detail, door, fill).
    v6d: the blow is a row of assembly §4's *Doors by how the blow fell*; its `uses` is a means of that row and of
    the kit's cell; the heart must fit (a member for *disgraced*; a captive heart neither gives the crisis nor fills
    a *herself* one); no hall is seized after a move that shut one; the door's premise isn't the crisis's."""
    g = K["g"]; out = []; c = K["crises"][crisis]
    pe = K["per_enemy"].get(e["id"])
    for (t, ev), b in K["blows"].items():
        if pe is not None and (t, ev) not in pe: continue
        hits = BLOW_HITS[(t, ev)]
        if K["stakes"][sid]["hit"] not in hits and "people" not in hits: continue            # point 3
        cell = {m for a in met(b["needs"], e) for m in a["means"]}
        if pe and pe[(t, ev)]: cell &= pe[(t, ev)]
        if not cell: continue
        if (t, ev) == ("heart", "captive") and (gvr == "the heart" or c["herself"]): continue   # heart state
        if (t, ev) == ("heart", "disgraced") and heart and not K["heart_member"][heart]: continue
        if ev == "hall seized" and mid and K["moves"][mid]["limits"]["shuts"]: continue     # no step repeats
        counts = t == "guild"
        cands = [(row, u) for row in HOWS[(t, ev)] for u in sorted({m for a in met(row["needs"], e) for m in a["means"]} & cell)]
        if not cands: continue
        free = [(r, u, False) for r, u in cands if not (counts and u in ("paper", "faith"))]
        if free: cands = free
        elif writ_left < 1: continue
        else: cands = [(r, u, True) for r, u in cands]
        for row, u, iw in cands:
            details = [{}]
            if ev == "leader struck down":
                details = [dict(leader=l, how=h, successor=s) for l, h, s in strike_options(g, e["id"], crisis, gvr) if h == STRIKE_ROW[row["key"]]]
            elif ev == "hall seized":
                halls, bar = HALLS[g]
                halls = halls[:-1] if bar(crisis, sid) or (row["key"] == "seized-order" and not e["leader"]) else halls  # v6d: by order, his own hall
                details = [dict(site=s) for s in halls]
            elif ev in ("framed", "disgraced"):
                details = [dict(code_break=cb) for cb in CODE[g] if ev == "disgraced" or cb in row["how"]]
            elif ev == "members taken":
                details = [dict(how=row["how"])]
            for d in details:
                for dr in row["doors"]:
                    if dr["family"] in K["never_fam"] or (dr["only"] and dr["only"] != g): continue
                    if (dr["family"], dr["premise"]) == (c["family"], c["premise"]): continue
                    out.append(dict(target=t, event=ev, how=row["key"], uses=u, writ=iw, detail=dict(d),
                                    door=dr["family"] + "/" + dr["premise"], fill=dr["fill"]))
    return out

# ================================================================== the world (§9) and the roll (§12 step 1)
def key_rng(*keys):
    return random.Random(int(hashlib.sha256("|".join(map(str, keys)).encode()).hexdigest()[:16], 16))

class World:
    def __init__(self, KITS, ledger):
        self.K = KITS; self.st = {}; self.ledger = ledger
        self.crisis_ledger = [tuple(x) for x in ledger.get("crisis_premises", [])]
    # counts over rolled stories
    def n(self, f): return sum(1 for p in self.st.values() if f(p))
    def side_free(self, side):
        """One enemy side per faction. N4's toll lord ("LORD") takes whichever lord group no other story uses."""
        if side is None: return True
        sides = [p["side"] for p in self.st.values() if p["side"]] + [side]
        named = [x for x in sides if x != "LORD"]
        if len(named) != len(set(named)) or sides.count("LORD") > 1: return False
        return "LORD" not in sides or bool({"Free Lords", "Banner Lords"} - set(named))
    def enemy_ok(self, g, e, pair_hh=False):
        side = SIDE_OF[g] if e["shape"] == "Within" else e["side"]
        if not self.side_free(side): return False
        if e["leader"] and self.n(lambda p: p["leader_enemy"]) >= 1: return False
        if e["shape"] == "Within" and self.n(lambda p: p["shape"] == "Within") >= 2: return False
        if e["kind"] == "the cost" and self.n(lambda p: p["kind"] == "the cost") >= 2: return False
        if e["kind"] == "hidden hand" and not pair_hh and self.hidden_hands() >= 2: return False
        return True
    def hidden_hands(self):
        h = self.n(lambda p: p["kind"] == "hidden hand")
        pr = [self.st[g] for g in ("old-hands", "night-market") if g in self.st]
        if len(pr) == 2 and pr[0].get("paired") and all(p["kind"] == "hidden hand" for p in pr): h -= 1
        return h
    def crisis_room(self, fam, prem, kills_new):
        allc = [(p["crisis_family"], p["crisis_premise"]) for p in self.st.values()] + self.crisis_ledger
        if fam == "Reckoning" and sum(f == "Reckoning" for f, _ in allc) >= 3: return False
        if Counter(allc)[(fam, prem)] >= 2: return False
        if kills_new and len({p["kill"] for p in self.st.values() if p["kill"]}) >= 2: return False
        return True
    def premise_room(self, prem):
        prems = [p["odd_job_premise"] for p in self.st.values()] + [p["crisis_family"] + "/" + p["crisis_premise"] for p in self.st.values()]
        return Counter(prems)[prem] < 2 and prem not in [p["odd_job_premise"] for p in self.st.values()]
    def heart_room(self, htype): return self.n(lambda p: p["heart_type"] == htype) < 2
    def rep_room(self): return self.n(lambda p: p["reputation"]) < 2
    def writs(self): return sum(p["writs"] for p in self.st.values())
    def leader_givers(self): return self.n(lambda p: p["giver_is_leader"])

def recent_tier(recent, g, kind, x):
    """0: fresh; 1: used by the older world only (comes back first); 2: used by the last world. `recent` is oldest first."""
    if not recent: return 0
    if (recent[-1].get(g) or {}).get(kind) == x: return 2
    return 1 if any((w.get(g) or {}).get(kind) == x for w in recent[:-1]) else 0

def best_tier(items, tier):
    if not items: return items
    lo = min(tier(x) for x in items); return [x for x in items if tier(x) == lo]

def spines(W, g, paired_story=None):
    """Every legal (enemy, crisis, stake, move) for guild g under the world so far: §11a points 2, 3 and 5,
    the kit's tags, the world spread (§9), writ fatigue, at least 3 fitting endings, a heart, an odd job and a blow."""
    K = W.K[g]; out = []
    rep_ok = W.rep_room(); writ_left = 2 - W.writs()
    for e in K["enemies"].values():
        if paired_story is None and e["paired"]: continue
        if paired_story is not None and (not e["paired"] or e["id"] != paired_story["enemy"]): continue
        if not W.enemy_ok(g, e, pair_hh=paired_story is not None and paired_story.get("pair_hh")): continue
        for c in K["crises"].values():
            if paired_story is not None and c["id"] != paired_story["crisis"]: continue
            if not W.crisis_room(c["family"], c["premise"], c["kills"] == "yes"): continue
            gv = [v for v in range(3) if not (giver(c, v, e["id"]) in LEADERS[g] and W.leader_givers() >= 2)
                  and LEADER_ENEMY.get((g, e["id"])) != giver(c, v, e["id"])]
            if paired_story is not None and paired_story.get("variant") in list(V): gv = [v for v in gv if V[v] == paired_story["variant"]]
            if not gv: continue
            hearts = [h for h, ht in K["hearts"].items() if W.heart_room(ht) and (c["hearts"] is None or h in c["hearts"])]
            if not hearts: continue
            for sid in K["stakes"]:
                if not stake_ok(K, e, sid): continue
                if K["stakes"][sid]["reputation"] and not rep_ok: continue
                cu = crisis_ok(K, e, sid, c, paired_story is not None)
                if not cu: continue
                hv = [(h, v) for h in hearts for v in gv if giver_ok(c, v, sid, h)]                 # v6d: the giver's tie
                if not hv: continue
                ends = endings_fit(K, e["id"], sid, c, paired_story is not None)
                if len(ends) < 3: continue
                odd = [o for o, x in K["odd"].items() if W.premise_room(x["premise"]) and e["id"] in x["fits"]
                       and (x["needs"] is None or met(x["needs"], e))]                                # v6d: the odd job serves the scheme
                if not odd: continue
                for m in K["moves"].values():
                    mu = move_ok(K, e, sid, m)
                    if not mu: continue
                    wr = int(mu[1]) + int(cu[1])
                    if wr > 1 or wr > writ_left: continue                                        # one writ per story; two per world
                    if not any(blow_list(K, e, sid, c["id"], giver(c, v, e["id"]), min(1 - wr, writ_left - wr), m["id"]) for v in {v for _, v in hv}): continue
                    out.append(dict(e=e["id"], c=c["id"], s=sid, m=m["id"], mu=mu[0], cu=cu[0], wr=wr, ends=ends, hv=hv, odd=odd))
    return out

def giver(c, v, e):
    gv = c["givers"][v]; return gv.get(e, gv["*"])

def giver_ok(c, v, sid, h):
    """v6d: a crisis variant whose giver (or last move) is tied to a stake or to some hearts rolls only with them."""
    gv = c["givers"][v]
    return ("_on" not in gv or sid in gv["_on"]) and ("_hearts" not in gv or h in gv["_hearts"])

def roll_story(W, g, R, recent, sp_all, paired_story=None):
    """Enemy first, then a crisis among its legal ones, then the stake, the move, the heart, the odd job, the endings."""
    K = W.K[g]
    def pick(pool, key, kind=None):
        vals = sorted({x[key] for x in pool})
        if kind: vals = best_tier(vals, lambda v: recent_tier(recent, g, kind, v))
        v = R.choice(vals); return [x for x in pool if x[key] == v], v
    # v6d: the revelation kind first, then an enemy of that kind, so an enemy legal in every world (W7, W8) doesn't
    # take the leftovers of the capped kinds (faction-stories §12 step 1)
    kinds = best_tier(sorted({K["enemies"][x["e"]]["kind"] for x in sp_all}),
                      lambda k: min(recent_tier(recent, g, "enemy", x["e"]) for x in sp_all if K["enemies"][x["e"]]["kind"] == k))
    kd = R.choice(kinds)
    pool, e = pick([x for x in sp_all if K["enemies"][x["e"]]["kind"] == kd], "e", "enemy")
    pool, c = pick(pool, "c", "crisis"); pool, s = pick(pool, "s"); pool, m = pick(pool, "m")
    x = R.choice(pool); en = K["enemies"][e]; cr = K["crises"][c]
    h = R.choice(sorted({h for h, _ in x["hv"]})); o = R.choice(best_tier(x["odd"], lambda v: recent_tier(recent, g, "odd_job", v)))
    v = R.choice([v for hh, v in x["hv"] if hh == h])
    hvar = R.choice(sorted((cr["hearts"] or {}).get(h) or V))                       # v6d: a variant-level heart fit
    gvr = giver(cr, v, e)
    ends = list(x["ends"]); R.shuffle(ends); ends = sorted(ends[:3], key=x["ends"].index)
    if K["seat"] and K["seat"][0] == e: ends.append(K["seat"][1])
    mu = R.choice(x["mu"]); cu = R.choice(x["cu"])
    p = dict(guild=g, enemy=e + R.choice(V), shape=en["shape"], want=en["want"], kind=en["kind"], leader_enemy=en["leader"],
             side=SIDE_OF[g] if en["shape"] == "Within" else en["side"], stake=s + R.choice(V), stake_hit=K["stakes"][s]["hit"],
             reputation=K["stakes"][s]["reputation"], heart=h + hvar, heart_type=K["hearts"][h],
             odd_job=o + R.choice(V), odd_job_premise=K["odd"][o]["premise"], odd_job_thread=K["odd"][o]["thread"],
             move=m, crisis=c + V[v], crisis_family=cr["family"], crisis_premise=cr["premise"],
             crisis_giver=gvr, giver_is_leader=gvr in LEADERS[g], kill=("truce" if cr["kills"] == "shared" and paired_story else (g + c if cr["kills"] == "yes" else 0)),
             means_used=dict(move=mu, crisis=cu), writs=x["wr"], faction_endings=ends)
    if paired_story: p.update(paired=True, feud=paired_story["feud"])
    return p

def legal_pairs(sp): return len({(x["e"], x["c"]) for x in sp})

# ---- the paired syndicates (§10): one unit, rolled together -------------------------------------------------------
def paired_options(W):
    KO, KN = W.K["old-hands"], W.K["night-market"]
    out = []
    for oe in [x for x in KO["enemies"].values() if x["paired"]]:
        for ne in [x for x in KN["enemies"].values() if x["paired"]]:
            hh = oe["kind"] == "hidden hand" and ne["kind"] == "hidden hand"
            for blood in ("old-hands", "night-market"):
                oc = "C2" if blood == "old-hands" else "C5"; nc = "C1" if blood == "night-market" else "C2"
                for tv, tn in KO["truce"].items():
                    if not (tn & oe["means"] and tn & ne["means"]): continue
                    so = spines(W, "old-hands", dict(enemy=oe["id"], crisis=oc, pair_hh=False, variant=tv))
                    if not so: continue
                    W2 = World(W.K, W.ledger); W2.st = dict(W.st)
                    W2.st["old-hands"] = dict(side=oe["side"], leader_enemy=False, shape="Beside", kind=oe["kind"], crisis_family=KO["crises"][oc]["family"],
                                              crisis_premise=KO["crises"][oc]["premise"], kill="truce", odd_job_premise="", heart_type="", reputation=False,
                                              writs=0, giver_is_leader=False, paired=True)
                    sn = spines(W2, "night-market", dict(enemy=ne["id"], crisis=nc, pair_hh=hh, variant=tv if nc == "C1" else None))
                    if not sn: continue
                    feuds = [(f, first) for f, st in KO["feud"].items() for first in ("Market side", "Old Hands side")
                             if (oe["id"] if first == "Market side" else ne["id"]) in st
                             and (ne["id"] if first == "Market side" else oe["id"]) in KO["feud_ans"][f]]   # v6d: fits both schemes
                    if feuds: out.append(dict(oe=oe["id"], ne=ne["id"], blood=blood, oc=oc, nc=nc, truce=tv, feuds=feuds, hh=hh))
    return out

def roll_paired(W, R, recent, opts):
    pr = best_tier(sorted({(o["oe"], o["ne"]) for o in opts}),
                   lambda x: recent_tier(recent, "old-hands", "enemy", x[0]) + recent_tier(recent, "night-market", "enemy", x[1]))
    oe, ne = R.choice(pr)
    opts = [o for o in opts if (o["oe"], o["ne"]) == (oe, ne)]
    blood = R.choice(best_tier(sorted({o["blood"] for o in opts}), lambda b: recent_tier(recent, "old-hands", "crisis", "C2" if b == "old-hands" else "C5")
                               + recent_tier(recent, "night-market", "crisis", "C1" if b == "night-market" else "C2")))
    opts = [o for o in opts if o["blood"] == blood]
    o = R.choice(opts); feud, first = R.choice(o["feuds"]); caller = R.choice(W.K["old-hands"]["callers"])
    fd = dict(enemies=[oe, ne], blood_debt=blood, truce_letter=o["truce"], over_what=feud, first_striker=first, truce_called_by=caller)
    so = spines(W, "old-hands", dict(enemy=oe, crisis=o["oc"], variant=o["truce"]))
    p1 = roll_story(W, "old-hands", R, recent, so, dict(enemy=oe, crisis=o["oc"], variant=o["truce"], feud=fd))
    W.st["old-hands"] = p1
    sn = spines(W, "night-market", dict(enemy=ne, crisis=o["nc"], pair_hh=o["hh"], variant=o["truce"] if o["nc"] == "C1" else None))
    if not sn: return False
    p2 = roll_story(W, "night-market", R, recent, sn, dict(enemy=ne, crisis=o["nc"], variant=o["truce"] if o["nc"] == "C1" else None, feud=fd))
    W.st["night-market"] = p2
    for p, c in ((p1, o["oc"]), (p2, o["nc"])): p["means_used"]["crisis"] = sorted(W.K["old-hands"]["truce"][o["truce"]] & W.K[p["guild"]]["enemies"][p["enemy"][:-1]]["means"])[0]
    return True

# ---- the placement plan's blow deal (§12 step 2) -------------------------------------------------------------------
BLOW_KEYS = ("blow_target", "blow_event", "blow_how", "door", "door_fill", "blow_detail", "blow_writ")

def deal(W, R, log, main_ordeal):
    order = sorted(W.st); R.shuffle(order)
    opts = {}
    for g in order:
        p = W.st[g]; K = W.K[g]
        opts[g] = [b for b in blow_list(K, K["enemies"][p["enemy"][:-1]], p["stake"][:-1], p["crisis"][:-1], p["crisis_giver"], 1 - p["writs"],
                                        p["move"], p["heart"][:-1])
                   if (b["target"], b["event"]) != ("heart", main_ordeal)]
    tries = [0]
    def bad():
        w = list(W.st.values()); b = []
        blows = [(p["blow_target"], p["blow_event"]) for p in w if "blow_target" in p]
        if any(v > 1 for v in Counter(blows + [("heart", main_ordeal)]).values()): b.append("target+event used twice")
        if any(v > TARGET_CAP[1] for v in Counter(t for t, _ in blows).values()): b.append("target in >3 stories")
        doors = [p["door"] for p in w if "door" in p]
        if any(v > 1 for v in Counter(doors).values()): b.append("two doors share a family+premise")
        prem = [p["odd_job_premise"] for p in w] + [p["crisis_family"] + "/" + p["crisis_premise"] for p in w] + doors
        if any(v > 2 for v in Counter(prem).values()): b.append("family+premise >2 across guild stories")
        if sum(p["writs"] + int(p.get("blow_writ", False)) for p in w) > 2: b.append("writ fatigue: more than 2 in the world")
        return b
    def go(i):
        if i == len(order):
            c = Counter(W.st[g]["blow_target"] for g in order)
            return all(c[t] >= TARGET_CAP[0] for t in ("heart", "player", "guild"))
        g = order[i]; p = W.st[g]
        combos = sorted({(o["target"], o["event"]) for o in opts[g]}); R.shuffle(combos)
        for te in combos:
            cand = [o for o in opts[g] if (o["target"], o["event"]) == te]; R.shuffle(cand)
            for o in cand[:6]:
                tries[0] += 1
                p.update(blow_target=o["target"], blow_event=o["event"], blow_how=o["how"], door=o["door"], door_fill=o["fill"],
                         blow_detail=o["detail"], blow_writ=o["writ"])
                p["means_used"]["blow"] = o["uses"]
                bd = bad()
                if not bd and go(i + 1): return True
                for k in BLOW_KEYS: p.pop(k, None)
                if tries[0] > 4000: return False
        return False
    ok = go(0)
    log.append(f"  deal: {'done' if ok else 'failed'} after {tries[0]} tries")
    return ok, tries[0]

# ================================================================== the throughline sentence (§11a point 6)
WANT = {"greed": "money", "revenge": "revenge", "preservation": "things kept as they are", "desperation": "a way out of ruin",
        "cover-up": "an old wrong kept buried", "ambition": "more power", "faith": "what he believes the gods demand"}

def who_short(e):
    w = re.sub(r" \((?:High|Middle|Low|Top)[^)]*\)", "", e["who"].replace("**", ""))
    return re.split(r" \*\(|, with |; |, a rival", w)[0].strip()

def pron(e):
    s = e["scheme"].split()[0].lower()
    return {"he": ("he", "his", "him"), "she": ("she", "her", "her"), "they": ("they", "their", "them")}.get(s, ("he", "his", "him"))

def fill(t, e, extra=None):
    p, pos, obj = pron(e)
    d = dict(p=p, pos=pos, obj=obj, **(extra or {}))
    return re.sub(r"\{(\w+)\}", lambda m: str(d.get(m.group(1), m.group(0))), t)

def phrase_for(table, pid, sid, default):
    x = table.get(pid)
    if x is None: return default
    if isinstance(x, str): return x
    return x.get(sid, x.get("*", default))

# the blow's words, by how it fell (assembly §4, *Doors by how the blow fell*; the row's key)
BLOW_TEXT = {"held-by-men": "{pos} men take the heart and hold her", "held-for-debt": "{p} buys the heart's debts and has her held for them",
             "held-on-charge": {"paper": "the heart is held under a charge {p} laid", "faith": "the heart is held on a heresy charge {p} laid"},
             "cried-down": "the heart is cried down for {code}", "kin-turned": "{p} turns the heart's kin against her",
             "framed-damage": "you are framed for {code}", "framed-theft": "you are framed for {code}", "framed-shame": "you are framed for {code}",
             "framed-contract": "you are framed for {code}", "framed-betrayal": "you are framed for {code}",
             "stripped": "you are put down a rank on {pos} word",
             "arrested-charge": {"paper": "you are arrested on a charge {p} laid", "faith": "you are arrested on the house's complaint, at {pos} word"},
             "arrested-men": "{pos} men hold you in a lord's cells",
             "seized-lord": "{pos} men take {site}", "seized-occupied": "{pos} men occupy {site}",
             "seized-seal": "{site} is shut under a court's seal, at {pos} suit", "seized-order": "{site} is shut on {pos} order",
             "taken-bonded": "members are bonded for debts {p} bought", "taken-arrested": "members are arrested on {pos} charge",
             "taken-band": "members are marched off by {pos} men", "taken-fight": "members are taken in a fight by {pos} men",
             "outlawed": {"paper": "a lord's writ {p} obtained bars the guild in one lordship", "faith": "the house's ban, at {pos} complaint, bars the guild in one lordship"},
             "killed": "{leader} is killed by {pos} hand", "wounded-men": "{pos} men wound {leader} on the road",
             "wounded-rigged": "{leader} is hurt through something {pos} people inside rigged", "disgraced": "{leader} is disgraced"}

def they_verbs(t):
    """Pronoun agreement for an enemy who is "they" (the Night Market's leader)."""
    def conj(w):
        return {"has": "have", "is": "are"}.get(w, w[:-2] if w.endswith(("shes", "ches", "sses")) else w[:-1] if w.endswith("s") and not w.endswith("ss") else w)
    return re.sub(r"\bthey (\w+)", lambda m: "they " + conj(m.group(1)), t)

def reading(K, p, part, sid):
    """v6d: a kit reading by variant (W1c) or by enemy (OH9) for a part ("scheme", "M8", "C3"), the stake's first."""
    for key in (p["enemy"], p["enemy"][:-1]):
        r = K["readings"].get(key, {})
        for k in (f"{part} on {sid}", part):
            if k in r: return r[k], key + ":" + k
    return None

def throughline(W, p):
    g = p["guild"]; K = W.K[g]; e = K["enemies"][p["enemy"][:-1]]; sid = p["stake"][:-1]; ph = PHRASE.get(g, {})
    pr, pos, obj = pron(e); used = []
    def kit_or(part, default):
        r = reading(K, p, part, sid)
        if r: used.append(r[1]); return r[0]
        return default
    scheme = kit_or("scheme", e["scheme"])
    mv = K["moves"][p["move"]]
    mtxt = kit_or(mv["id"], fill(phrase_for(ph.get("moves", {}), mv["id"], sid, mv["act"]), e, dict(stake=K["stakes"][sid]["name"])))
    c = K["crises"][p["crisis"][:-1]]
    ctxt = kit_or(c["id"], fill(phrase_for(ph.get("crises", {}), c["id"], sid, c["last"]), e, dict(stake=K["stakes"][sid]["name"], **ph.get("extra", {}).get(e["id"], {}))))
    if p.get("paired"):
        fd = p["feud"]; letter = ph.get("truce", {}).get(fd["truce_letter"], "")
        ctxt = fill(phrase_for(ph.get("paired", {}), c["id"], sid, ctxt), e, dict(truce=letter))
    d = p.get("blow_detail", {}); how = p.get("blow_how")
    bt = ph.get("blows", {}).get((how, e["id"])) or BLOW_TEXT.get(how, "")
    if isinstance(bt, dict): bt = bt.get(p["means_used"].get("blow")) or next(iter(bt.values()))
    btxt = fill(bt, e, dict(code=d.get("code_break", "a code-break"), site=d.get("site", "a hall"), leader=d.get("leader", "the leader")))
    want = ph.get("wants", {}).get(e["id"]) or WANT[e["want"]]
    feud = ""
    if p.get("paired"):                                   # v6d: the feud grows from the scheme, never beside it
        fd = p["feud"]; mine = (fd["first_striker"] == "Market side") == (g == "old-hands")
        over = ph.get("feud", {}).get(fd["over_what"], fd["over_what"])
        feud = (f"it is how the feud with the {SIDE_OF[g]} began, over {over}: " if mine
                else f"when the {SIDE_OF[g]} struck first, over {over}, it was {pos} answer: ")
    p["readings_used"] = used
    who = ph.get('who', {}).get(e['id'], who_short(e))
    if ", who " in who: who += ","
    if pr == "they": mtxt = re.sub(r"\band (\w+)s\b", lambda m: "and " + m.group(1), mtxt)
    out = (f"Because {who} wants {want}, {scheme}; "
           f"{feud}{pr} {mtxt}, then {btxt}, and it comes to a head when {ctxt}.")
    return they_verbs(out) if pr == "they" else out

def door_fill(K, p):
    """The door's slots, filled from the story (assembly §4, *Doors by how the blow fell*)."""
    e = K["enemies"][p["enemy"][:-1]]; d = p.get("blow_detail", {})
    vals = dict(enemy=who_short(e), code=d.get("code_break", ""), site=d.get("site", ""), leader=d.get("leader", ""),
                shield="*Bought*" if "coin" in e["means"] else "*Protected*")
    return re.sub(r"\{(\w+)\}", lambda m: vals.get(m.group(1)) or m.group(0), p.get("door_fill", ""))

# sentence wording, from the kits' readings; {p} {pos} {obj} are the enemy's pronouns
PHRASE = {
 "writers": dict(
  moves={"M1": {"S1": "opens a copy house across the street from the Wayhouse at half the halls' price and takes its clients",
                "S4": "opens a copy house beside the reading room at half its price and takes its clients",
                "S7": "underbids the halls for the lord's court rolls with a copy house of {pos} own"},
         "M2": {"S2": "has a court seize one of the Lending Wall's letters as evidence, under seal",
                "S3": "has a magistrate rule in open court that a Writers' copy doesn't stand for the original",
                "S5": "has a lord's court rule that rolls bought at a dig belong to the buyer, not the halls",
                "S6": "has a lord's court call one camp's edge map as evidence and rule it unproven",
                "S7": "has the lord's court refuse a Writers' copy of its own rolls"},
         "M3": {"S1": "openly takes the Wayhouse strongroom's keys into {pos} own keeping, over the halls' objection",
                "S2": "openly takes the sorting of the Lending Wall into {pos} own hands",
                "S4": "openly takes the reading room's master copies into {pos} own keeping",
                "S7": "openly takes the lord's court rolls into {pos} own keeping, so no other copyist touches them"},
         "M4": {"S4": "buys the reading room's best copyist away from the hall in public, copying book and all",
                "S7": "buys the copyist who keeps the lord's court rolls away from the hall in public"},
         "M5": {"S3": "has it cried in a market square that a Writers' copy was forged",
                "S6": "has it cried in the edge towns that the Writers' maps put the green too far off",
                "S7": "has it cried at the lord's court that the Writers' copies of its rolls are false"},
         "M6": {"S5": "closes the dig the rolls came from and turns its diggers off", "S6": "closes a survey camp at the green's edge and keeps its journals"},
         "M7": {"S2": "has a letter from the Lending Wall read aloud in a market square, so everyone knows the Wall is open"},
         "M8": {"S1": "has the Wayhouse searched for forbidden texts and its doors sealed for a week",
                "S2": "has the Lending Wall searched and its pigeonholes sealed", "S4": "has the reading room searched and sealed"},
         "M9": {"S1": "dismisses, before the whole hall, the copyist who asked where the strongroom's copies went",
                "S2": "dismisses, before the whole hall, the clerk who asked who reads the Wall's letters",
                "S4": "dismisses, before the whole hall, the copyist who asked where the reading room's master copies went",
                "S7": "dismisses, before the whole hall, the copyist who kept the lord's court rolls"}},
  crises={"C1": {"S1": "the Wayhouse strongroom's copies are burned and the heart is named as the one who let the burners in",
                 "S2": "the Lending Wall's letters are burned and the heart is named as the one who let the burners in",
                 "S4": "the reading room's master copies are burned and the heart is named as the one who let the burners in",
                 "S7": "the halls' copies of the lord's court rolls are burned and the heart is named as the one who let the burners in"},
          "C2": {"S2": "a magistrate's writ, on a charge {p} laid, puts the Lending Wall's letters in {pos} hands, to burn at dawn",
                 "S6": "a magistrate's writ, on a charge {p} laid, puts the edge maps in {pos} hands, to burn at dawn"},
          "C3": {"S2": "a letter let out through the Lending Wall at the wrong moment brings armed men to the heart's village, and it burns",
                 "S4": "what leaks out of the reading room brings armed men to the heart's village, and it burns",
                 "S7": "a debt read out of the lord's court rolls turns his quarrel into a feud, and the heart's village burns"},
          "C4": {"S2": "{p} sells the heart's own letters to the man they ruin and is gone, and the heart loses her post",
                 "S3": "{p} sells the heart's own file, in a copy no hall can tell from its own, to the man it ruins, and is gone",
                 "S7": "{p} sells the heart's copy of the court rolls to the lord's rival and is gone, and the heart loses her place at court"},
          "C5": {"S1": "on a charge {p} laid, the authority's men hold the Wayhouse's copyists in its own reading hall, the heart's kin among them",
                 "S2": "on a charge {p} laid, the Lending Wall's clerks are held at the Wayhouse until its letters are handed over, the heart's kin among them",
                 "S5": "on a charge {p} laid, the Wayhouse's copyists are held until the rolls lodged there are handed over, the heart's kin among them"},
          "C6": {"S2": "the heart must carry the one letter that proves it out of Highcrossing, with {pos} men on every road",
                 "S5": "the heart must carry the rolls lodged at the Wayhouse out of Highcrossing, with {pos} men on every road",
                 "S6": "the heart must carry the camp's true readings out of Highcrossing, with {pos} men on every road",
                 "S7": "the heart must carry the halls' true copy of the court rolls out of Highcrossing, with {pos} men on every road"},
          "C7": {"S3": "{p} accuses the heart before a lord's court of a false copy, and the guild's name rides on the hearing",
                 "S6": "{p} accuses the heart before a lord's court of drawing the edge maps false",
                 "S7": "{p} accuses the heart before the lord's court of a false copy of its rolls, and the halls' commission rides on the hearing"},
          "C8": {"S1": "{p} has bought the debt on the Wayhouse fair and square, means to close it at the season's end, and won't sell it back",
                 "S4": "{p} has bought the reading room's lease fair and square, means to turn the copyists out, and won't sell it back",
                 "S7": "{p} has bought the lord's court commission fair and square and won't sell it back"}}),
 "lantern": dict(
  moves={"M1": {"S6": "puts {pos} own blades on the caravan road at half the Company's price", "S5": "puts {pos} own blades on the chapter house's contracts at half its price"},
         "M2": "has it cried in every market that the Company has taken a side",
         "M3": {"S6": "closes the caravan road to Lantern escorts with {pos} men", "S5": "closes the roads round the chapter house to its riders with {pos} men"},
         "M4": {"S4": "calls in the old contract before the contract board, in public", "S3": "calls in, before the contract board, a contract the Company can't refuse, binding it to one side"},
         "M5": {"S3": "buys a well-known chapter captain into {pos} own service in front of the contract board", "S5": "buys the chapter house's captain into {pos} own service in front of its board"},
         "M6": {"S1": "openly takes Lanternhall's contract board into {pos} own hands", "S2": "openly takes the lamps of one chapter's dead into {pos} own keeping and lets no one trim them",
                "S3": "openly posts only one side's contracts on the board, against the quartermaster's word", "S5": "openly shuts the chapter house's books to the quartermaster's auditors",
                "S6": "openly gives the road contract to the side {p} favours, against the quartermaster's word"},
         "M7": {"S2": "has the shrines preach that the Lamp Room's lamps are pagan rites", "S3": "has the shrines preach that a Lantern who guards a heretic's cart shares the heresy"},
         "M8": {"S1": "buys up the last of the Company's debts and posts {pos} terms on Lanternhall's own contract board", "S5": "buys up the chapter house's debts and says so in front of its board",
                "S6": "buys up the debts the road contract was pledged against and says so at the contract board"},
         "M9": {"S1": "throws the clerk who asked who signs Lanternhall's contracts out before the board", "S2": "throws out, before the board, the lamp-keeper who asked whose lamps those are",
                "S3": "throws out, before the board, the captain who spoke for the Company's neutrality", "S5": "throws out, before the board, the rider who asked where the chapter house's old dead lie",
                "S6": "takes the road contract from its captain before the board, for asking whose side it serves"}},
  extra={"L1": dict(writ="his seizure order under the old contract", offense="fighting for his rival"), "L3": dict(writ="a lord's writ she steered for"),
         "L5": dict(writ="the march magistrate's levy writ"), "L6": dict(offense="the killing at his village"),
         "L7": dict(writ="a heresy writ from the house's court", offense="selling its swords to heretics"), "L8": dict(offense="breaking its contracts with him")},
  crises={"C1": {"*": "{writ} names one chapter; it refuses, and is held in Lanternhall's yard, the heart among those pleading for them"},
          "C2": {"S2": "on {pos} orders a chapter is led into an ambush and dies holding a road, the heart's kin among them, and their lamps join the ones {p} kept lit",
                 "S5": "on {pos} orders the chapter house's riders who know the old story are led into an ambush and die holding a road, the heart's kin among them",
                 "S3": "on {pos} orders a chapter is led into an ambush and dies holding a road for the side {p} chose, the heart's kin among them",
                 "S6": "on {pos} orders a chapter is led into an ambush and dies holding the caravan road, the heart's kin among them"},
          "C3": "{p} wins the Company's road contract with the Company's own stolen route maps, and the caravan the heart rides with is ruined",
          "C4": {"S3": "{pos} steering puts two chapters on opposite sides of one fight, and the heart is disowned by her chapter",
                 "S6": "{pos} steering puts two chapters on opposite sides of the caravan road, and the heart is disowned by her chapter",
                 "S1": "two chapters stand at Lanternhall's gate, one for {obj} and one against, and the heart is disowned by her chapter",
                 "S2": "the kin {p} paid come to Lanternhall for their men, {pos} chapter stands by {obj}, and the heart is disowned by her chapter",
                 "S5": "the kin {p} paid come to the chapter house for their men, {pos} chapter stands by {obj}, and the heart is disowned by her chapter"},
          "C5": "{pos} men take to the caravan road out of Carter's Rest as a band, and the caravan the heart rides with is the first they strike",
          "C6": {"S1": "{p} calls in the Company's debts, and the chapter that can't pay is held at Lanternhall to work them off, the heart's kin among them",
                 "S2": "{p} calls in the debts the founding lamp was pledged against, and the chapter that can't pay is held at Lanternhall, the heart's kin among them",
                 "S5": "{p} calls in the chapter house's debts, and its chapter is held at Lanternhall to work them off, the heart's kin among them",
                 "S6": "{p} calls in the debts the road contract was pledged against, and the chapter that rode it is held at Lanternhall, the heart's kin among them"},
          "C7": {"S3": "{p} sets a Lantern chapter against the Company's own escort leaving Lanternhall, the caravan the heart rides with in its charge",
                 "S6": "{p} sets a Lantern chapter against the Company's escort on the caravan road, the caravan the heart rides with in its charge"},
          "C8": {"S3": "{p} accuses the Company before Carter's Rest's judge of {offense}, naming the heart's chapter as the ones who did it",
                 "S2": "{p} accuses the Company before Carter's Rest's judge of keeping pagan lamps for its dead, naming the heart's chapter as their keepers"}}),
 "old-hands": dict(
  moves={"M1": "buys a house's steward away from the Old Hands, and the steward says so at his house's table", "M2": "shuts a quiet crossing to the Old Hands",
         "M3": {"S3": "has a house call the Old Hands thieves before witnesses and send its work elsewhere", "S6": "has the oldest house send its standing work elsewhere and say why before witnesses"},
         "M4": "has a lord's court call in a lineage roll the Old Hands made", "M5": "opens a back room across the lane from one of the Old Hands', selling the houses the same work",
         "M6": "has an old favour the Old Hands did a house named before the house's guests", "M7": "has a steward who asked questions put out of a back room before the others",
         "M8": {"S1": "has the Old Gate House's quarter barred to the Old Hands for a night", "S4": "has a safehouse estate barred to the Old Hands for a night"},
         "M9": "has a scandal the Old Hands were paid to bury cried at a house's feast"},
  crises={"C1": "letters to three houses' rivals tell what the Old Hands carried for whom, naming the heart as the one who carried them",
          "C3": "the house disowns the Old Hands in public and hands the heart to the watch as their go-between",
          "C4": "{pos} letters are sealed at the Old Gate House, to go out at the new moon",
          "C5": "{pos} men wait at the gate that has never closed for the heart's kin, who saw the scheme at work, and the Old Hands must get them out",
          "C6": {"S1": "the head of the house that lends the Old Gate House's quarter dies, and {pos} claim, which would shut it to them, meets the heart's in its inner court",
                 "S4": "the head of the house that lends a safehouse estate dies, and {pos} claim, which would shut it to them, meets the heart's in the Old Gate House's inner court"},
          "C7": "{p} buys the crossing-keepers' debts, and the heart's kin is held for one",
          "C8": "{p} has the heart named before the back rooms as the one who did {pos} work, and the old don calls the Old Hands to judge by their own law"},
  paired={"C2": "the truce meeting ends {truce}, one dead on each side, and the Market's leader shields the killer",
          "C5": "the truce meeting ends {truce}, one dead on each side; the Market blames the heart's kin, and they must be got out of the Old Gate House before the reprisal"},
  truce={"a": "with a knife at the truce table", "b": "with a fall from the crossing's bridge", "c": "with a poisoned cup"},
  feud={"F1": "a contested tavern both syndicates use", "F2": "a quiet crossing the Market's crews began using without leave",
        "F3": "an old house that began buying from the Market", "F4": "a forger who changed sides"}),
 "night-market": dict(
  moves={"M1": "opens another market the same night under another road, with the Market's own buyers", "M2": "shuts a toll-dodge trail",
         "M3": "buys a fence at a card table, in front of the crews", "M4": {"S1": "shows the toll men the plain door into Under the Market Road", "S4": "shows the toll men a smugglers' cove's landing"},
         "M5": "has an exchange refuse the Market's notes", "M6": "has a page of the fences' list read out in a dockside tavern",
         "M7": "calls the Market out before the crews at a dockside card game", "M8": {"S1": "buys the shop over Under the Market Road out from under the Market", "S4": "buys a cove's landing out from under the Market"},
         "M9": "has a crew paid short in front of the others", "M10": "names the big score openly at the market and sends crews after it"},
  crises={"C2": "a raid comes for Under the Market Road tonight with {obj} riding along, and the whole market must move by dawn",
          "C3": "the crews learn their shares have been short for a year, and the heart's share paid for someone else's way out",
          "C4": "the Market's notes are cried down and the heart is named as the forger",
          "C5": "{p} holds the fences' list behind {pos} walls as surety for the crews' debt, and the heart's crew is paid to bring it home",
          "C6": "{p} swears a race for the trails under the Market's oldest rule, and the heart's crew must win it",
          "C7": {"S1": "{pos} men take the crews at the plain door, the heart's kin among them, to be judged at dawn",
                 "S4": "{pos} men take the crews at the cove's landing as the boats come in, the heart's kin among them, to be judged at dawn",
                 "S5": "{pos} men take the crews on a trail, the carts stopped, the heart's kin among them, to be judged at dawn"},
          "C8": "{p} lays the short shares on the heart before the crews, and the leader calls the Market to judge by its own rule"},
  paired={"C1": "the truce meeting ends {truce}, one dead on each side, and the old don shields the killer",
          "C2": "the truce meeting ends {truce}, one dead on each side, and the reprisal comes as a raid on Under the Market Road tonight"},
  truce={"a": "with a knife at the truce table", "b": "with a fall from the crossing's bridge", "c": "with a poisoned cup"},
  feud={"F1": "a contested tavern both syndicates use", "F2": "a quiet crossing the Market's crews began using without leave",
        "F3": "an old house that began buying from the Market", "F4": "a forger who changed sides"}),
 "dueling": dict(
  blows={("taken-bonded", "D3"): "his fighters are sold to the Ring's creditors to work off its debts",
         ("taken-fight", "D7"): "fighters are pressed into his lord's guard by his men", ("taken-fight", "D2"): "fighters are pressed into his guard by his men"},
  moves={"M1": "puts real-steel bouts on at the next circuit stop and draws off the Society's crowd",
         "M2": "calls the Society's way cowardice at a challenge stone and beats the fighter who answers",
         "M3": "takes the ladder's top fighter off it in front of the crowd, into a paid post",
         "M4": {"S1": "gets the Ring shut pending an inquiry", "S2": "gets the tournament's licence suspended pending an inquiry", "S4": "gets the ladder's results held back pending an inquiry"},
         "M5": "cries the Ring down in the taverns as a killers' yard", "M6": "posts men at the Ring's gate on bout nights to turn the crowd away",
         "M7": "overrules the yield rule in public and puts bouts on the card at first blood",
         "M8": "opens a book on the ladder in every tavern, at odds that tell the town who will win"},
  crises={"C1": "a fighter dies after a bout {p} had a hand in, and {pos} champion lays a blood challenge over the Ring's honour",
          "C2": "the tournament licence is revoked after a riot {p} paid for, and the heart is fined and barred",
          "C3": "the heart is sold into a lord's private match and badly hurt",
          "C4": "the sister's old duel is cried abroad and {p} calls her out on blood terms in the Ring",
          "C5": {"S1": "the fighters who follow {obj} take the Ring and beat the heart for standing against {obj}",
                 "S4": "the fighters who follow {obj} take the ladder board and beat the heart for standing against {obj}"},
          "C6": "{p} buys up the ladder's entry debts, and {pos} men take the heart and the top rungs to work them off",
          "C7": {"S1": "the Ring is put under {pos} seal on the grand bout's eve, and the heart is barred from the sand",
                 "S2": "the feast-day licence is put under {pos} shrine's seal on the grand bout's eve, and the heart is barred from the sand"},
          "C8": "{pos} village comes to the Ring for the old winners' blood, the heart's kin among them",
          "C9": "the ladder is cried down as a book at the season's draw, and the heart, whose wins were bought without her knowing, is named the fixer"}),
 "holy-house": dict(
  moves={"M1": {"S1": "lays his house's claim to the Mount before his house's court", "S5": "lays his house's claim to the old grant's last term before his house's court"},
         "M2": {"S2": "has a man taken from the Low Shrine's line", "S3": "has a man taken from a shrine's steps, where he was sheltering"},
         "M3": "has a priest of the priestess's way named a heretic from the Mount's steps", "M4": "raises one valley's tithe and calls in its arrears at once",
         "M5": "buys a tithe-house's debts at its door, in front of the town", "M6": "cries in the markets that the house's shelter is worth nothing",
         "M7": "feeds the outer ring from a tithe-house's door and calls it the house's duty",
         "M8": "puts a 'recovered piece' of the procession relic on show at the stalls, a keeper swearing to it"},
  crises={"C1": "{p} has the riders shut the Low Shrine's doors on its line, and the heart is named before the parish as the one who stirred it",
          "C2": {"S2": "the inquisitors take the heart's kin from the Low Shrine's line on a charge {p} laid, to be judged at the Mount on the feast day",
                 "S3": "the inquisitors take the heart's kin from a shrine's steps on a charge {p} laid, to be judged at the Mount on the feast day"},
          "C3": "the penance debts are sold, the heart's family's among them, and the debtors are penned in the Mount's yard to march at dawn",
          "C4": "by his house's court his men take the Mount's gate under the old grant, and the heart's company of riders is the first pledged",
          "C5": "on the feast day the procession relic is shown to be his copy, he is gone to a lord's town, and the heart, who swore to it, is cast out",
          "C6": "his raiders camp a night from the Mount, the tithe-towns' protection fails, and the heart's village is next",
          "C7": "his false receipts are found, the riders come to the Low Shrine for the grain, and the heart, who carried it, is named the thief",
          "C8": "at the feast-day judgment his bought inquisitors find the heart's kin heretics, and their fields pass to his house"}),
}

# ================================================================== one world
def run(seed, mode, ledger_doc, KITS=None, verbose=True):
    KITS = KITS or load()
    R = random.Random(seed); log = []
    recent = ledger_doc.get("recent") or []
    if isinstance(recent, dict): recent = [recent]
    main_ordeal = ledger_doc.get("main_ordeal", "estranged")
    paired = {"paired": True, "unpaired": False, "auto": key_rng(seed, "syndicates", "paired").random() < 1 / 3}[mode]
    log.append(f"seed {seed}; syndicate pairing: {'PAIRED' if paired else 'unpaired'}; main Ordeal: (heart, {main_ordeal})"
               + (f"; recent worlds: {len(recent)}" if recent else ""))
    stats = dict(restarts=0, rerolls=0, deal_tries=0, fallback_recent=0)
    for restart in range(60):
        W = World(KITS, ledger_doc)
        units = (["syndicates"] if paired else ["old-hands", "night-market"]) + ["writers", "lantern", "dueling", "holy-house"]
        ok = True; order = []
        while units:
            counts = {}
            for u in units:
                if u == "syndicates":
                    po = paired_options(W); counts[u] = (len({(o["oe"], o["ne"], o["blood"]) for o in po}), po)
                else:
                    sp = spines(W, u); fresh = [x for x in sp if recent_tier(recent, u, "enemy", x["e"]) == 0 and recent_tier(recent, u, "crisis", x["c"]) == 0]
                    counts[u] = (legal_pairs(fresh) or legal_pairs(sp), sp)
            tie = key_rng(seed, restart, len(units), "order")
            u = min(units, key=lambda x: (counts[x][0], tie.random()))
            order.append(f"{u}({counts[u][0]})")
            if counts[u][0] == 0: ok = False; log.append(f"  restart {restart + 1}: {u} has no legal enemy + crisis pair"); break
            if u == "syndicates":
                if not roll_paired(W, R, recent, counts[u][1]): ok = False; log.append(f"  restart {restart + 1}: the pair didn't fit"); break
            else:
                p = roll_story(W, u, R, recent, counts[u][1])
                if recent and any(recent_tier(recent, u, k, p[f][:-1]) for k, f in (("enemy", "enemy"), ("crisis", "crisis"), ("odd_job", "odd_job"))):
                    stats["fallback_recent"] += 1
                W.st[u] = p
            units.remove(u)
        def reroll(g, why, exclude=None):
            """§9 / §12: reroll only that story's enemy (and every part after it), keeping the world's Above and Beside."""
            old = W.st.pop(g); sp = spines(W, g)
            if exclude: sp = [x for x in sp if x["e"] != exclude]
            need = {"Above", "Beside"} - {q["shape"] for q in W.st.values()}
            if need: sp = [x for x in sp if W.K[g]["enemies"][x["e"]]["shape"] in need]
            if not sp or len(need) > 1: W.st[g] = old; return False
            W.st[g] = roll_story(W, g, R, recent, sp); stats["rerolls"] += 1
            log.append(f"  reroll {g}: {why} ({old['enemy']} -> {W.st[g]['enemy']})"); return True
        if ok:
            shapes = {p["shape"] for p in W.st.values()}
            if not {"Above", "Beside"} <= shapes:
                last = [x.split("(")[0] for x in order if not x.startswith("syndicates")][-1]
                if not reroll(last, "no Above or no Beside enemy"): ok = False; log.append(f"  restart {restart + 1}: no Above or no Beside enemy")
        if ok:
            log.append("  roll order (legal enemy + crisis pairs): " + ", ".join(order))
            for attempt in range(4):
                dok, tries = deal(W, R, log, main_ordeal); stats["deal_tries"] += tries
                if dok or attempt == 3: break
                nb = {g: len(blow_list(W.K[g], W.K[g]["enemies"][q["enemy"][:-1]], q["stake"][:-1], q["crisis"][:-1], q["crisis_giver"], 1 - q["writs"],
                                          q["move"], q["heart"][:-1]))
                      for g, q in W.st.items() if not q.get("paired")}
                g = min(nb, key=lambda x: (nb[x], x))
                for q in W.st.values():
                    for k in BLOW_KEYS: q.pop(k, None)
                if not reroll(g, "no legal blow deal", exclude=W.st[g]["enemy"][:-1]): break
            if dok: break
            log.append(f"  restart {restart + 1}: no legal blow deal")
        stats["restarts"] += 1
    else:
        return dict(seed=seed, paired=paired, rolls=None, log=log, stats=stats)
    for p in W.st.values():
        if p["side"] == "LORD":
            used = {q["side"] for q in W.st.values()}
            p["side"] = "Free Lords" if "Free Lords" not in used else "Banner Lords"
            p["cast_note"] = f"N4's toll lord is cast from the {p['side']}"
        p["throughline"] = throughline(W, p)
        p["door_fill"] = door_fill(W.K[p["guild"]], p)
        p["last_move_kills"] = bool(p.pop("kill"))
        p["means_used"]["blow"] = p["means_used"].get("blow")
        p["writs"] = p["writs"] + int(bool(p.get("blow_writ")))           # v6d: counted before the field is dropped
        for k in ("reputation", "stake_hit", "blow_writ"): p.pop(k, None)
    return dict(seed=seed, paired=paired, main_ordeal=["heart", main_ordeal], rolls=W.st, log=log, stats=stats)

# ================================================================== checks and stats
def check(KITS):
    for g, K in KITS.items():
        print(f"== {g}: {len(K['enemies'])} enemies, {len(K['stakes'])} stakes, {len(K['hearts'])} hearts, {len(K['moves'])} moves, "
              f"{len(K['odd'])} odd jobs, {len(K['crises'])} crises, {len(K['endings'])} endings + seat {K['seat']}; per-enemy blows {sorted(K['per_enemy'])}")
        W = World(KITS, {})
        sp = spines(W, g)
        by = defaultdict(set)
        for x in sp: by[x["e"]].add(x["c"])
        for e in K["enemies"]:
            if K["enemies"][e]["paired"]: continue
            ms = sorted({x["m"] for x in sp if x["e"] == e}); ss = sorted({x["s"] for x in sp if x["e"] == e})
            print(f"   {e:4} stakes {ss} moves {ms} crises {sorted(by[e])}")
        print(f"   legal enemy + crisis pairs: {legal_pairs(sp)}; spines (enemy, crisis, stake, move): {len(sp)}")
    W = World(KITS, {}); po = paired_options(W)
    print(f"== paired couplings: {len(po)}: " + ", ".join(sorted({f'{o['oe']}+{o['ne']} blood={o['blood'][:3]} truce {o['truce']}' for o in po})))

def stats(a, b):
    KITS = load(); en = defaultdict(Counter); cr = defaultdict(Counter); pr = defaultdict(Counter)
    rer = Counter(); restarts = Counter(); rerolls = Counter(); dealt = []; worlds = 0; paired = 0; fails = 0; texts = []
    for s in range(a, b + 1):
        r = run(s, "auto", {}, KITS)
        worlds += 1; paired += r["paired"]
        if not r["rolls"]: fails += 1; continue
        restarts[r["stats"]["restarts"]] += 1; rerolls[r["stats"]["rerolls"]] += 1; dealt.append(r["stats"]["deal_tries"])
        for l in r["log"]:
            m = re.match(r"\s*reroll ([\w-]+):", l)
            if m: rer[m.group(1)] += 1
        for g, p in r["rolls"].items():
            en[g][p["enemy"][:-1]] += 1; cr[g][p["crisis"][:-1]] += 1; pr[g][(p["enemy"][:-1], p["crisis"][:-1])] += 1
            texts.append(p["throughline"])
    n = worlds - fails
    print(f"{worlds} worlds, {paired} paired ({paired / worlds:.0%}); failed {fails}; restarts per world {dict(sorted(restarts.items()))}; "
          f"worlds needing a restart {sum(v for k, v in restarts.items() if k) / n:.1%}; rerolls per world {dict(sorted(rerolls.items()))}, "
          f"worlds needing a reroll {sum(v for k, v in rerolls.items() if k) / n:.1%}; mean deal tries {sum(dealt) / len(dealt):.1f}")
    for g in ORDER:
        e = ", ".join(f"{k} {v / n:.0%}" for k, v in sorted(en[g].items(), key=lambda kv: (len(kv[0]), kv[0])))
        c = ", ".join(f"{k} {v / n:.0%}" for k, v in sorted(cr[g].items(), key=lambda kv: (len(kv[0]), kv[0])))
        (pe, pc), pv = pr[g].most_common(1)[0]
        print(f"{g}\n  enemies: {e}\n  crises:  {c}\n  top pair: {pe}+{pc} {pv / n:.0%}; rerolled in {rer[g] / n:.1%} of worlds")
    return texts

# ================================================================== the joins check (v6d)
JOIN_RULES = {1: "door fits how the blow fell (and its slots are filled)", 2: "heart fits the crisis and the blow",
              3: "heart state; no step repeats the one before", 4: "odd job serves the scheme",
              5: "no 'and also' (feud fits both schemes; cost protects what pays; last move fits the want)",
              6: "the variant's own reading is used", 7: "blow means read from how it fell; no means the enemy lacks",
              8: "crisis giver has a reason", 9: "bookkeeping (move ids, writs)"}

def story_joins(KITS, w, p):
    """Every v6d join one rolled story breaks: [(rule, note)]. Reads the output and the docs' data only."""
    g = p["guild"]; K = KITS[g]; e = K["enemies"][p["enemy"][:-1]]; ev_ = p["enemy"][-1]
    sid = p["stake"][:-1]; c = K["crises"][p["crisis"][:-1]]; v = V.index(p["crisis"][-1]); h, hv = p["heart"][:-1], p["heart"][-1]
    te = (p["blow_target"], p["blow_event"]); bad = []
    def no(r, why): bad.append((r, f"{g} {p['enemy']}: {why}"))
    # 1. the door is one of the row's, for this how (and code-break); its slots are filled
    rows = [r for r in HOWS[te] if r["key"] == p["blow_how"]]
    if not rows: no(1, f"no row {p['blow_how']} for {te}")
    else:
        row = rows[0]
        if p["door"] not in [d["family"] + "/" + d["premise"] for d in row["doors"] if not d["only"] or d["only"] == g]: no(1, f"door {p['door']} not by how {row['key']}")
        cb = p["blow_detail"].get("code_break")
        if te[1] == "framed" and cb not in row["how"]: no(1, f"framed for {cb}, door row {row['key']}")
        if p["door"].split("/")[0] in K["never_fam"]: no(1, f"door family {p['door']} is never for the guild")
        if not p.get("door_fill") or "{" in p["door_fill"]: no(1, f"door slots unfilled: {p.get('door_fill')!r}")
        if row["key"] == "seized-order" and p["blow_detail"].get("site") == HALLS[g][0][-1] and not e["leader"]: no(1, "headquarters shut by order of an enemy who doesn't lead")
    # 2. heart fitness
    if c["hearts"] is not None and (h not in c["hearts"] or (c["hearts"][h] and hv not in c["hearts"][h])): no(2, f"heart {p['heart']} unfit for {p['crisis']}")
    if te == ("heart", "disgraced") and not K["heart_member"][h]: no(2, f"heart {p['heart']} ({p['heart_type']}) disgraced under a code she isn't bound by")
    # 3. heart state; no step repeats
    if te == ("heart", "captive"):
        if p["crisis_giver"] == "the heart": no(3, "captive heart gives the crisis")
        if c["herself"]: no(3, f"captive heart in {p['crisis']}, which needs her herself")
    if p["door"] == p["crisis_family"] + "/" + p["crisis_premise"]: no(3, f"door and crisis share {p['door']}")
    if te[1] == "hall seized" and K["moves"][p["move"]]["limits"]["shuts"]: no(3, f"hall seized after {p['move']}, which shut one")
    # 4. odd job
    o = K["odd"][p["odd_job"][:-1]]
    if e["id"] not in o["fits"] or (o["needs"] is not None and not met(o["needs"], e)): no(4, f"odd job {p['odd_job']} doesn't serve {e['id']}")
    # 5. no "and also"
    if p.get("paired"):
        fd = p["feud"]; KO = KITS["old-hands"]; oe, ne = fd["enemies"]
        first, other = (oe, ne) if fd["first_striker"] == "Market side" else (ne, oe)
        if first not in KO["feud"][fd["over_what"]] or other not in KO["feud_ans"][fd["over_what"]]: no(5, f"feud {fd['over_what']} doesn't fit both {first} and {other}")
        if not any(k.endswith(":" + p["move"]) or k.endswith(f":{p['move']} on {sid}") for k in p.get("readings_used", [])): no(5, f"paired move {p['move']} not read as {e['id']}'s scheme")
    if e["kind"] == "the cost":
        pe = K["per_enemy"].get(e["id"])
        if pe is None or te not in pe: no(5, f"cost enemy takes {te}")
    L = c["limits"]
    if (L["wants"] and e["want"] not in L["wants"]) or (L["nocost"] and e["kind"] == "the cost") or e["id"] in L["never"] or (L["only"] and e["id"] not in L["only"]):
        no(5, f"crisis {p['crisis']} against its limits for {e['id']}")
    # 6. variant readings
    for key in (p["enemy"], e["id"]):
        for part, text in K["readings"].get(key, {}).items():
            pid, _, on = part.partition(" on ")
            hit = pid == "scheme" or pid == p["move"] or pid == c["id"]
            if hit and (not on or on == sid) and text not in p["throughline"] and not (p.get("paired") and pid == c["id"]):
                if not (on == "" and f"{pid} on {sid}" in K["readings"].get(key, {})): no(6, f"{key} reading for {part} not used")
    # 7. blow means from the sub-event; no means the enemy lacks
    if rows:
        u = p["means_used"]["blow"]
        if u not in {m for a in met(rows[0]["needs"], e) for m in a["means"]} or u not in {m for a in met(K["blows"][te]["needs"], e) for m in a["means"]}:
            no(7, f"blow by `{u}` but {rows[0]['key']} needs {[sorted(a['means']) for a in rows[0]['needs']]}")
    pr, pos, _ = pron(e)
    if "men" not in e["means"] and re.search(rf"\b{pos} men\b", p["throughline"]): no(7, f"'{pos} men' without `men`")
    if "coin" not in e["means"] and re.search(rf"\b{pr} buys?\b|debts {pr} bought", p["throughline"]): no(7, "buys without `coin`")
    vt = c["givers"][v]["_text"]
    if "men" not in e["means"] and re.search(r"\b(his|her|their|the enemy's) men\b", vt): no(7, f"crisis variant {p['crisis']} names the enemy's men")
    # 8. giver
    gv = c["givers"][v]; gname = gv.get(e["id"], gv["*"])
    if p["crisis_giver"] != gname: no(8, f"giver {p['crisis_giver']!r} isn't {p['crisis']}'s")
    if "_on" in gv and sid not in gv["_on"]: no(8, f"giver {gname!r} tied to {sorted(gv['_on'])}, stake {sid}")
    if "_hearts" in gv and h not in gv["_hearts"]: no(8, f"giver/variant {p['crisis']} needs hearts {sorted(gv['_hearts'])}, heart {h}")
    if LEADER_ENEMY.get((g, e["id"])) == p["crisis_giver"]: no(8, "the enemy gives his own crisis")
    if te[1] == "leader struck down" and p["blow_detail"].get("how") == "killed" and p["blow_detail"].get("leader") == p["crisis_giver"]: no(8, "a killed leader gives the crisis")
    # bookkeeping
    if not re.fullmatch(r"M\d+", p["move"]): no(9, f"move {p['move']} carries a variant letter")
    return bad

def joins(a, b):
    KITS = load(); counts = Counter(); ex = defaultdict(list); worlds = 0; stories = 0
    for s in range(a, b + 1):
        r = run(s, "auto", {}, KITS)
        if not r["rolls"]: counts["no world"] += 1; continue
        worlds += 1; w = r["rolls"]
        lg = sum(p["giver_is_leader"] for p in w.values())
        if lg > 2: counts[8] += 1; ex[8].append(f"seed {s}: {lg} leader-given crises")
        wr = sum(p["writs"] for p in w.values())
        if wr > 2: counts[9] += 1; ex[9].append(f"seed {s}: {wr} writs")
        for p in w.values():
            stories += 1
            K = KITS[p["guild"]]; mu = p["means_used"]; pw = ("paper", "faith")
            exp = (int(K["moves"][p["move"]]["writ"] and mu["move"] in pw) + int(not p.get("paired") and K["crises"][p["crisis"][:-1]]["writ"] and mu["crisis"] in pw)
                   + int(p["blow_target"] == "guild" and mu["blow"] in pw))
            if exp != p["writs"]: counts[9] += 1; ex[9].append(f"seed {s}: {p['guild']} writs {p['writs']}, its parts give {exp}")
            for rule, note in story_joins(KITS, w, p):
                counts[rule] += 1
                if len(ex[rule]) < 6: ex[rule].append(f"seed {s}: {note}")
    print(f"{worlds} worlds, {stories} stories; joins broken: {sum(v for k, v in counts.items() if k != 'no world')}")
    for k in sorted(JOIN_RULES):
        print(f"  {k}. {JOIN_RULES[k]}: {counts[k]}")
        for x in ex[k]: print(f"       {x}")
    if counts["no world"]: print(f"  worlds that failed to roll: {counts['no world']}")
    return counts

if __name__ == "__main__":
    if sys.argv[1] == "--joins": c = joins(int(sys.argv[2]), int(sys.argv[3])); sys.exit(1 if sum(c.values()) else 0)
    if sys.argv[1] == "--check": check(load()); sys.exit()
    if sys.argv[1] == "--stats": stats(int(sys.argv[2]), int(sys.argv[3])); sys.exit()
    led = json.load(open(sys.argv[3])) if len(sys.argv) > 3 else {}
    print(json.dumps(run(int(sys.argv[1]), sys.argv[2], led), indent=1, ensure_ascii=False))
