"""Draft 5 of the Writers' spines: read each spine's branches, endings, cards
and flavour out of `spines.md`, check them pairwise, and render them.

Draft 5 (review-v7d, Laz's decisions): one heart per spine; at most two blow
branches, each written out whole (door, code-bending way, standing, how the
crisis reads it, closing accounts); exactly three endings, plus the seat
ending where the spine has one; flavour is description only. The roller
(`roll_samples.py`) takes blows, endings and flavour from here, so the spine
text is the only source.

Inline tags ⟨key: a, b; key: c; flag⟩ ride on the lines and are stripped when
rendered. Keys:
  makes      states a card or branch brings about (heart-cried-down, player-stripped, ...)
  needs      states a closing account or cost presumes; each must be made by
             the shared cards or by that branch
  marks      words only that branch may use; nothing outside it may say them
  harms      who the blow falls on (player, heart, guild, leader)
  captor     the slot whose lord or holder holds the player after the blow
  at-captor  (flag, on STANDING) the standing answers a crisis at the captor's court
  proof      id@holder: a proof or warning in someone's hands before the payload;
             every one must be covered by a HELD line
  covers     on HELD: the proofs it says are not enough, and why
  info       something the player is told; told once, or it makes a later card redundant
  object     id@slot: where a thing is; later cards must agree, or say `moved`
  presumes   on the heart's want: a fact the want takes for granted
  shows      on TELL: a fact the tell shows (!fact denies one)
  in-hands   on TELL: whose hands the tell is in
  loses      a person a card takes off the board; a later card or every
             ending must bring them back (`appears`)
  appears    see loses
"""
import re

TAG = re.compile(r"\s*⟨([^⟩]*)⟩")
FIELD_RE = r"^\*\*{n} · (.*?)(?=^\*\*\d+ · |\Z)"

# --- flavour: description only (Laz, draft 5) --------------------------------
PERSON_WORDS = r"""man men woman women boy boys girl girls son sons daughter daughters father mother parent
wife husband brother sister uncle aunt niece nephew cousin child children widow widower grandfather grandmother
lord lady steward clerk copyist priest priestess master mistress captain sergeant keeper scholar caretaker
apprentice heir heirs kin friend ferryman carter herder miller digger foreman reader tutor leader headman
headwoman guard watchman cook merchant trader buyer seller smith mason surveyor scribe novice inquisitor
archivist courier runner porter forger thief lender moneylender creditor debtor tenant holder baker weaver
nobody somebody someone anyone everyone""".split()
TIME_WORDS = r"""day days night nights nightly daily week weeks weekly month months year years season seasons
dawn dusk morning evening noon midnight ago since before after when once then yesterday tomorrow today
spring summer autumn winter harvest sowing feast fair""".split()
EVENT_WORDS = r"""died dies dead death buried burial born killed kills burned burnt sold sells bought buys
paid pays lost loses won wins fled flees moved moves married marries stole steals struck strikes broke breaks
fought fights drowned fell falls sent sends gave gives took takes came comes went goes left leaves arrived
quarrel quarrelled murder murdered fire war battle raid storm flood""".split()
WHY_WORDS = r"""because lest wants wanted hopes hoped fears feared loves loved hates hated reason why""".split()
WHY_PHRASES = ("so that", "for fear", "to keep", "in order to", "for the sake")
DISTANCE_WORDS = r"""passes passed past every each walks walked rides rode travels travelled visits visited
across along road ford""".split()


def flavour_fault(option, place_words=()):
    """Why a flavour option is not description only (None if it is)."""
    t = option.lower()
    words = {re.sub(r"'s?$", "", w) for w in re.findall(r"[a-z']+", t)}
    for kind, lst in (("a person", PERSON_WORDS), ("a time", TIME_WORDS), ("an event", EVENT_WORDS),
                      ("a reason", WHY_WORDS), ("a distance or habit of going", DISTANCE_WORDS)):
        hit = sorted(words & set(lst))
        if hit:
            return f"names {kind} ({', '.join(hit)})"
    for ph in WHY_PHRASES:
        if ph in t:
            return f"names a reason ({ph})"
    for w in place_words:
        if re.search(rf"\b{re.escape(w)}\b", option):
            return f"names a place ({w})"
    if re.search(r"\b[A-Z][a-z]+", option[1:]):
        return "names someone or something by a proper name"
    return None


# --- parsing -----------------------------------------------------------------
def field(text, n):
    m = re.search(FIELD_RE.format(n=n), text, re.M | re.S)
    return m.group(1) if m else ""


def tags_of(line):
    out = {}
    for m in TAG.finditer(line):
        for part in m.group(1).split(";"):
            part = part.strip()
            if not part:
                continue
            if ":" in part:
                k, v = part.split(":", 1)
                vals = [x.strip() for x in v.split(",") if x.strip() and x.strip() != "—"]
                out.setdefault(k.strip(), []).extend(vals)
            else:
                out.setdefault(part, []).append(True)
    return out


def plain(line):
    return TAG.sub("", line).strip()


LINE = re.compile(r"^- \*\*([A-Z0-9][A-Za-z0-9 \-]*?)(?::\*\*| \*\* ·|\*\* ·)\s*(.*)$")


def card_lines(block):
    """[(key, text, tags)] for '- **KEY** · text' or '- **KEY:** text' lines."""
    out = []
    for raw in block.splitlines():
        m = LINE.match(raw.strip())
        if m:
            out.append((m.group(1).strip(), plain(m.group(2)), tags_of(m.group(2))))
    return out


def parse(sid, text):
    sp = {"sid": sid}
    f4 = field(text, 4)
    m = re.search(r"^- \*\*Want:\*\* (.*)$", f4, re.M)
    sp["want"] = (plain(m.group(1)), tags_of(m.group(1))) if m else ("", {})
    sp["heart_line"] = plain(f4.splitlines()[0]) if f4 else ""
    # field 18: the shared cards
    sp["cards"] = card_lines(field(text, 18))
    # field 9: the opening and the branches
    f9 = field(text, 9)
    sp["branches"] = {}
    for m in re.finditer(r"^\*Branch ([a-c])\* · (.*?)$\n(.*?)(?=^\*Branch [a-c]\* · |\Z)", f9, re.M | re.S):
        k, head, body = m.group(1), m.group(2), m.group(3)
        bits = [b.strip() for b in plain(head).split(" · ")]
        door_m = re.search(r"door (\w+) \*([^*]+)\*(?: \(([^)]*)\))? at `\{(\w+)\}`", head)
        lines = {key: (txt, tg) for key, txt, tg in card_lines(body)}
        ev = re.match(r"\*([^*]+)\*\s*(.*)", bits[1])
        event, rest = (ev.group(1), ev.group(2).strip()) if ev else (bits[1], "")
        how = bits[2] if len(bits) > 2 and not bits[2].startswith("uses") else rest
        sp["branches"][k] = dict(
            head=plain(head), tags=tags_of(head), target=bits[0], event=event, how=how,
            door=(f"{door_m.group(1)} {door_m.group(2)}" if door_m else "?"),
            door_place=door_m.group(4) if door_m else None, lines=lines)
    # field 10: the crisis lines
    sp["crisis"] = {key: (txt, tg) for key, txt, tg in card_lines(field(text, 10))}
    # field 11: the endings
    f11 = field(text, 11)
    sp["endings"], sp["seat"] = [], None
    for m in re.finditer(r"^- \*\*(E\d|SEAT) · ([^*]+)\*\* (.*?)$\n((?:^  .*$\n?)*)", f11, re.M):
        eid, nm, rest, subs = m.group(1), m.group(2).strip(), m.group(3), m.group(4)
        cm = re.search(r"\*costs:\* (.*?) · \*won by:\* (\w+)", rest)
        what = re.match(r"\(([^)]*(?:\([^)]*\)[^)]*)*)\)", rest)
        e = dict(id=eid, name=nm, what=what.group(1) if what else "", costs=plain(cm.group(1)) if cm else "",
                 costs_tags=tags_of(cm.group(1)) if cm else {}, won=cm.group(2) if cm else "?", accounts={},
                 branch_costs={})
        for sm in re.finditer(r"^  - \*\*(all|[a-c]|costs [a-c]):\*\* (.*)$", subs, re.M):
            key, val = sm.group(1), sm.group(2)
            if key.startswith("costs "):
                e["branch_costs"][key[-1]] = (plain(val), tags_of(val))
            else:
                e["accounts"][key] = (plain(val), tags_of(val))
        if eid == "SEAT":
            sp["seat"] = e
        else:
            sp["endings"].append(e)
    # field 14: flavour lists
    sp["flavour"] = {}
    for m in re.finditer(r"^- `\{(\w+)\}` \(([^)]*), on ([^)]+)\): (.*)$", field(text, 14), re.M):
        voice = m.group(3).strip()
        voice = {"the crisis": "CR", "the reveal": "RV", "the door": "DOOR", "the move": "MOVE"}.get(voice, voice)
        sp["flavour"][m.group(1)] = (voice, [o.strip() for o in m.group(4).split(" · ")], m.group(2))
    return sp


def blows(sp):
    """{k: (blow text for the roller's memory, door)} from the branches."""
    return {k: (f"{b['target']} {b['event']}, {b['how']}", b["door"]) for k, b in sp["branches"].items()}


def account(e, k):
    return e["accounts"].get(k) or e["accounts"].get("all")


def costs(e, k):
    return e["branch_costs"].get(k, (e["costs"], e["costs_tags"]))


# --- pairwise checks (draft 5) -------------------------------------------------
SENTENCE_BREAK = re.compile(r"[.!?][\"”’')]*\s+[\"“(]?[A-Z]")


def one_sentence(t):
    t2 = re.sub(r"\"[^\"]*\"", "\"…\"", t)  # quoted speech may hold its own stops
    return not SENTENCE_BREAK.search(t2)


def shared_lines(sp):
    out = [(f"card {k}", t, g) for k, t, g in sp["cards"]]
    out += [(f"crisis {k}", t, g) for k, (t, g) in sp["crisis"].items() if k in ("CR", "HELD", "TELL", "WAYS")]
    return out


def branch_lines(sp, k):
    b = sp["branches"][k]
    return [(f"branch {k} header", b["head"], b["tags"])] + \
        [(f"branch {k} {key}", t, g) for key, (t, g) in b["lines"].items()]


def path_tags(lines, key):
    out = []
    for _w, _t, g in lines:
        out += g.get(key, [])
    return out


def check(sp, known_seat=False, place_words=()):
    """Every draft-5 check on one spine; returns a list of fails."""
    sid, f = sp["sid"], []
    br = sp["branches"]
    cards = {k: (t, g) for k, t, g in sp["cards"]}
    # structure
    if not 1 <= len(br) <= 2:
        f.append(f"{sid}: {len(br)} blow branches; draft 5 allows one or two")
    if len(sp["endings"]) != 3:
        f.append(f"{sid}: {len(sp['endings'])} endings; draft 5 has exactly three")
    if known_seat and not sp["seat"]:
        f.append(f"{sid}: the spine has a seat ending and field 11 doesn't write it")
    for need in ("R1", "R2", "R3", "RV", "MOVE", "C1-1", "C1-2", "C1-3", "C2-0"):
        if need not in cards:
            f.append(f"{sid}: field 18 has no {need} line")
    for need in ("CR", "HELD", "TELL", "WAYS"):
        if need not in sp["crisis"]:
            f.append(f"{sid}: field 10 has no {need} line")
    for k, b in br.items():
        for need in ("DOOR", "BEND", "STANDING", "READING"):
            if need not in b["lines"]:
                f.append(f"{sid}: branch {k} has no {need} line")
        if not b["door_place"]:
            f.append(f"{sid}: branch {k}'s header names no door and place")
    # one sentence per line
    alll = shared_lines(sp) + [x for k in br for x in branch_lines(sp, k)]
    for where, t, _g in alll:
        if "header" in where or t == "—":
            continue
        if not one_sentence(t):
            f.append(f"{sid}: {where} is more than one sentence")
    # pattern 1a: nothing outside a branch says what only that branch makes
    for k in br:
        marks = br[k]["tags"].get("marks", [])
        own = " ".join([t for _w, t, _g in branch_lines(sp, k)]
                       + [e["accounts"][k][0] for e in sp["endings"] + ([sp["seat"]] if sp["seat"] else [])
                          if k in e["accounts"]]).lower()
        for mk in marks:
            if mk.lower() not in own:
                f.append(f"{sid}: branch {k}'s mark '{mk}' is never said in the branch")
        outside = [(w, t) for w, t, _g in shared_lines(sp)]
        for k2 in br:
            if k2 != k:
                outside += [(w, t) for w, t, _g in branch_lines(sp, k2)]
        for e in sp["endings"] + ([sp["seat"]] if sp["seat"] else []):
            for k2 in list(br) + ["all"]:
                if k2 == k:
                    continue
                a = e["accounts"].get(k2)
                if a:
                    outside.append((f"ending {e['name']} ({k2})", a[0]))
                c = e["branch_costs"].get(k2)
                if c:
                    outside.append((f"ending {e['name']} costs ({k2})", c[0]))
            if k not in e["branch_costs"]:
                outside.append((f"ending {e['name']} costs", e["costs"]))
            outside.append((f"ending {e['name']}", e["what"]))
        for mk in marks:
            for w, t in outside:
                if re.search(rf"\b{re.escape(mk.lower())}\b", t.lower()):
                    f.append(f"{sid}: '{mk}' (branch {k} only) is said in {w}")
    # pattern 1b: an ending against the blow dealt
    made = set(path_tags(shared_lines(sp), "makes"))
    for e in sp["endings"] + ([sp["seat"]] if sp["seat"] else []):
        for k in br:
            a = account(e, k)
            if not a:
                f.append(f"{sid}: ending '{e['name']}' has no account for branch {k}")
                continue
            have = made | set(br[k]["tags"].get("makes", [])) | set(path_tags(branch_lines(sp, k), "makes"))
            for n in a[1].get("needs", []) + costs(e, k)[1].get("needs", []):
                if n not in have:
                    f.append(f"{sid}: ending '{e['name']}' ({k}) presumes '{n}', which branch {k} never makes")
        for k in e["accounts"]:
            if k != "all" and k not in br:
                f.append(f"{sid}: ending '{e['name']}' has an account for branch {k}, which doesn't exist")
    # pattern 1c: standing against the blow
    cr_place = sp.get("_cr_place")
    for k, b in br.items():
        st = b["lines"].get("STANDING", ("—", {}))
        if "player" in b["tags"].get("harms", []) and st[0] == "—":
            f.append(f"{sid}: branch {k} harms the player and has no STANDING line")
        cap = b["tags"].get("captor", [])
        if cap and cr_place and cap[0] in cr_place and "at-captor" not in st[1]:
            f.append(f"{sid}: branch {k}'s captor ({cap[0]}) holds the crisis's place, and STANDING doesn't say on what footing the player is there")
    # pattern 2: the knower against Held
    covered = set(sp["crisis"].get("HELD", ("", {}))[1].get("covers", []))
    for k in br:
        kc = covered | set(br[k]["lines"].get("READING", ("", {}))[1].get("covers", []))
        for p in path_tags(shared_lines(sp) + branch_lines(sp, k), "proof"):
            pid = p.split("@")[0]
            if pid not in kc:
                f.append(f"{sid}: {p} is held before the payload (branch {k}), and no HELD line says why it isn't enough")
    allproofs = {p.split("@")[0] for k in br for p in path_tags(shared_lines(sp) + branch_lines(sp, k), "proof")}
    allproofs |= {p.split("@")[0] for p in path_tags(shared_lines(sp), "proof")}
    for c in covered:
        if c not in allproofs and c not in path_tags(shared_lines(sp), "info"):
            f.append(f"{sid}: HELD covers '{c}', which no card hands anyone")
    # pattern 1d: the odd job's object against where later cards say it is
    for k in br:
        where = {}
        for w, _t, g in shared_lines(sp) + branch_lines(sp, k):
            for o in g.get("object", []):
                oid, _, at = o.partition("@")
                if oid in where and where[oid][1] != at and "moved" not in g:
                    f.append(f"{sid}: '{oid}' is at {where[oid][1]} on {where[oid][0]} and at {at} on {w} (branch {k})")
                where[oid] = (w, at)
        told = {}
        for w, _t, g in shared_lines(sp) + branch_lines(sp, k):
            for i in g.get("info", []):
                if i in told and i not in g.get("again", []):
                    f.append(f"{sid}: '{i}' is told on {told[i]} and again on {w} (branch {k}); the later card is redundant")
                told.setdefault(i, w)
    # pattern 1e: the want against the tell
    pres = sp["want"][1].get("presumes", [])
    tell = sp["crisis"].get("TELL", ("", {}))[1]
    for p in pres:
        for s in tell.get("shows", []):
            if s == "!" + p and "heart" in tell.get("in-hands", []):
                f.append(f"{sid}: the tell, in the heart's own hands, shows not-{p}, which the heart's want takes for granted")
        for w, _t, g in alll:
            if "!" + p in g.get("heart-knows", []):
                f.append(f"{sid}: {w} has the heart know not-{p}, against the want")
    if not pres:
        f.append(f"{sid}: the heart's want carries no ⟨presumes: …⟩ tag")
    # pattern 4: a person a card loses comes back
    order = [w for w, _t, _g in shared_lines(sp)]
    for i, (w, _t, g) in enumerate(shared_lines(sp)):
        for p in g.get("loses", []):
            later = any(p in g2.get("appears", []) for _w2, _t2, g2 in shared_lines(sp)[i + 1:])
            every_branch = all(p in path_tags(branch_lines(sp, k), "appears") for k in br)
            every_end = all(p in account(e, k)[1].get("appears", []) for e in sp["endings"] for k in br if account(e, k))
            if not (later or every_branch or every_end):
                f.append(f"{sid}: {w} loses {p}, and no later card or ending brings them back")
    # hedges left unresolved
    for w, t, _g in alll:
        if re.search(r"\b\w+ or gone\b|\bone or the other\b", t):
            f.append(f"{sid}: {w} hedges ('… or gone'); resolve it in the branch")
    # flavour: description only, said on its card
    texts = {k: t for k, t, _g in sp["cards"]}
    texts.update({k: t for k, (t, _g) in sp["crisis"].items()})
    texts["CR"] = " ".join(t for k, (t, _g) in sp["crisis"].items())
    for slot, (voice, opts, what) in sp["flavour"].items():
        if len(opts) < 2:
            f.append(f"{sid}: flavour {{{slot}}} has one option")
        for o in opts:
            why = flavour_fault(o, place_words)
            if why:
                f.append(f"{sid}: flavour {{{slot}}} option '{o}' {why}; flavour is description only")
        body = texts.get(voice, "") if voice != "DOOR" else " ".join(
            b["lines"].get("DOOR", ("", {}))[0] for b in br.values())
        if f"{{{slot}}}" not in body:
            f.append(f"{sid}: flavour {{{slot}}} is voiced on {voice}, and that line never says it")
    return f


# --- rendering ---------------------------------------------------------------
def render_combo(sp, title, k, e, seat=None, places=None, cr_place="", door_label=""):
    """One branch x ending as a plain story: one sentence per line."""
    b = sp["branches"][k]
    cards = {key: t for key, t, _g in sp["cards"]}
    cr = sp["crisis"]
    o = []
    o.append(f"### Branch {k} × {e['id']} · {e['name']}")
    o.append("")
    o.append(f"*Blow {k}: {b['target']} · {b['event']} · {b['how']} → door {b['door']}. Ending shown: {e['name']}.*")
    o.append("")
    lab = places or {}
    for key, label in (("R1", "R1"), ("R2", "R2"), ("R3", "R3"), ("RV", "Reveal"), ("RV close", None),
                       ("MOVE", "Chapter 1's move"), ("C1-1", "C1-1"), ("C1-2", "C1-2"), ("C1-3", "C1-3"),
                       ("C2-0", "C2-0")):
        if key not in cards:
            continue
        where = lab.get(key, "")
        o.append(f"- **{label}{' · ' + where if where else ''}.** {cards[key]}" if label else f"  {cards[key]}")
    o.append(f"- **Blow and door ({k}) · `{{{b['door_place']}}}`.** {b['lines']['DOOR'][0]}")
    o.append(f"- **Code-bending way.** {b['lines']['BEND'][0]}")
    o.append(f"- **Crisis{' · ' + cr_place if cr_place else ''}.** {cr['CR'][0]}")
    o.append(f"  - *How it reads blow {k}:* {b['lines']['READING'][0]}")
    st = b["lines"]["STANDING"][0]
    if st != "—":
        o.append(f"  - *Standing:* {st}")
    o.append(f"  - *What the player holds:* {cr['HELD'][0]}")
    o.append(f"  - *The tell:* {cr['TELL'][0]}")
    o.append(f"  - *The ways:* {cr['WAYS'][0]}")
    o.append("- **Endings.**")
    for x in sp["endings"] + ([sp["seat"]] if sp["seat"] else []):
        c = costs(x, k)[0]
        mark = " ← shown" if x is e else ""
        won = "won by nobody" if x["won"] == "none" else f"won by {x['won']}"
        o.append(f"  - *{x['name']}* ({x['what']}): {won}; who pays: {c}.{mark}")
    a = account(e, k)[0]
    o.append(f"- **A month on ({e['name']}).** {a[:1].upper() + a[1:]}.")
    o.append("")
    return o


def render_spine(sp, title, places=None, cr_place=""):
    o = [f"# {sp['sid']} · {title}", "",
         "Every branch × ending combination of `spines.md` draft 5, rendered by `roll_samples.py --render-all`. "
         "Slots are shown as `{slot}`, flavour as its own slot; one sentence per line. The routine's swaps are not "
         "multiplied out (each is one card's other premise, printed in the spine's field 18).", ""]
    n = 0
    ends = sp["endings"] + ([sp["seat"]] if sp["seat"] else [])
    for k in sorted(sp["branches"]):
        for e in ends:
            o += render_combo(sp, title, k, e, places=places, cr_place=cr_place)
            n += 1
    o.insert(4, f"**{n} combinations:** {len(sp['branches'])} branch(es) × {len(ends)} endings"
             + (" (three, and the seat ending)" if sp["seat"] else "") + ".\n")
    return "\n".join(o) + "\n", n
