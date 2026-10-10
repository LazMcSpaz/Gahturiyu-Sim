"""Accents: bending the sounds of a line, knob by knob.

A line is read as plain English and turned into its sounds (voice.py does
that). `bend` then reshapes those sounds by an accent: a setting for each
knob in sounds.toml. The four tongues' starting recipes are in tongues.toml;
accents Laz has made and saved are in accents.toml.

Nothing here makes audio. It only rewrites strings of sounds, so every knob
can be checked by reading what comes out.
"""

from __future__ import annotations

import re
import tomllib
from pathlib import Path

HERE = Path(__file__).resolve().parent


def _toml(name: str) -> dict:
    p = HERE / name
    if not p.exists():
        return {}
    with open(p, "rb") as f:
        return tomllib.load(f)


SPEC = _toml("sounds.toml")
KNOBS: dict = SPEC["knob"]
ORDER: list = SPEC["order"]
TONGUES: dict = _toml("tongues.toml")

VOWELS = set("aeiouæɑɒɔəɚɛɜɪʊʌɐᵻ")
GLIDES = ["eɪ", "aɪ", "ɔɪ", "aʊ", "oʊ", "əʊ", "iə", "ɪə", "eə", "ʊə"]
PAIRS = ["tʃ", "dʒ"]
# A glide split into its two beats (the "split" setting of the gliding-vowels knob).
SPLIT = {"eɪ": "ei", "aɪ": "ai", "ɔɪ": "oi", "aʊ": "au", "oʊ": "ou", "əʊ": "ou",
         "iə": "ia", "ɪə": "ia", "eə": "ea", "ʊə": "ua"}
UNVOICE = {"b": "p", "d": "t", "ɡ": "k", "v": "f", "z": "s", "ʒ": "ʃ", "dʒ": "tʃ", "ɖ": "ʈ", "ɣ": "x"}
SEPARATOR = re.compile(r"(\s+|[,.!?;:—…]+)")
PARTING_VOWEL = "ɛ"   # what goes between parted consonants (the naming rules use e)


class Phone:
    """One sound of a word."""
    __slots__ = ("sym", "vowel", "stress", "long", "first")

    def __init__(self, sym: str, vowel: bool, stress: str = "", long: bool = False):
        self.sym, self.vowel, self.stress, self.long = sym, vowel, stress, long
        self.first = False   # was it the first sound of its word, before any change

    def text(self) -> str:
        return (self.stress if self.vowel else "") + self.sym + ("ː" if self.long else "")


def parse_word(word: str) -> list:
    out, i, stress = [], 0, ""
    while i < len(word):
        two = word[i:i + 2]
        ch = word[i]
        if ch in "ˈˌ":
            stress, i = ch, i + 1
        elif ch == "ː":
            if out:
                out[-1].long = True
            i += 1
        elif two in PAIRS:
            out.append(Phone(two, False))
            i += 2
        elif two in GLIDES:
            out.append(Phone(two, True, stress))
            stress, i = "", i + 2
        elif ch in VOWELS:
            out.append(Phone(ch, True, stress))
            stress, i = "", i + 1
        else:
            out.append(Phone(ch, False))
            i += 1
    if out:
        out[0].first = True
    return out


def setting(value) -> tuple:
    """ "roll:light" -> ("roll", True);  "" or "plain" or None -> (None, False)."""
    if not value or value == "plain":
        return None, False
    name, _, level = str(value).partition(":")
    return name, level == "light"


def check(sounds: dict) -> None:
    """Raise ValueError for a knob or variant that sounds.toml doesn't have."""
    for knob, value in (sounds or {}).items():
        if knob not in KNOBS:
            raise ValueError(f"unknown accent knob {knob!r}; the knobs are {', '.join(ORDER)}")
        name, _ = setting(value)
        if name and name not in KNOBS[knob]["variants"]:
            raise ValueError(f"knob {knob!r} has no setting {name!r}; it has "
                             f"{', '.join(KNOBS[knob]['variants'])}")


def _nuclei(word: list) -> list:
    return [p for p in word if p.vowel]


def _restress(word: list, where: str, light: bool) -> None:
    beats = _nuclei(word)
    if len(beats) < (3 if light else 2) or not any(p.stress == "ˈ" for p in beats):
        return
    for p in beats:
        p.stress = ""
    if where == "even":
        return
    target = {"first": 0, "nextlast": len(beats) - 2, "last": len(beats) - 1}[where]
    beats[target].stress = "ˈ"


def _vowel_set(word: list, variant: dict, light: bool) -> None:
    for k, p in enumerate(_nuclei(word)):
        if light and p.stress:
            continue
        maps = [variant.get("first" if k == 0 else "later", {}), variant.get("any", {})]
        if not p.stress:
            maps.insert(0, variant.get("unstressed", {}))
        for m in maps:
            if p.sym in m:
                new = m[p.sym]
                if new.endswith("ː"):
                    new, p.long = new[:-1], True
                p.sym = new
                break


def _glides(word: list, name: str, variant: dict) -> list:
    out = []
    for p in word:
        if not p.vowel or p.sym not in GLIDES:
            out.append(p)
        elif name == "pure" and p.sym in variant.get("map", {}):
            new = variant["map"][p.sym]
            p.sym, p.long = new.rstrip("ː"), new.endswith("ː")
            out.append(p)
        elif name == "split":
            a, b = SPLIT[p.sym]
            out += [Phone(a, True, p.stress), Phone(b, True)]
            out[-2].first = p.first
        else:
            out.append(p)
    return out


def _length(word: list, name: str) -> list:
    out = []
    for p in word:
        out.append(p)
        if not p.vowel:
            continue
        if name == "clipped":
            p.long = False
        elif p.stress == "ˈ" and p.sym not in GLIDES:
            p.long = True
            if name == "drawn":
                out.append(Phone(p.sym, True))
    return out


def _consonants(word: list, table: dict) -> list:
    out = []
    for p in word:
        if p.vowel or p.sym not in table:
            out.append(p)
            continue
        new, light, then = table[p.sym]
        if light and p.first:
            out.append(p)
            continue
        if then and new in table:          # "l turned to r" follows the R knob
            new = table[new][0]
        if new:
            p.sym = new
            p.vowel = new in VOWELS       # y opened to ee becomes a beat of its own
            out.append(p)
    return out


def _endings(word: list, name: str) -> list:
    if not word or not _nuclei(word):
        return word
    if name == "clipped" and len(_nuclei(word)) >= 2 and word[-1].vowel and not word[-1].stress:
        word = word[:-1]
    last = word[-1]
    if name in ("hard", "clipped") and not last.vowel:
        last.sym = UNVOICE.get(last.sym, last.sym)
    elif name == "echo" and not last.vowel:
        v = _nuclei(word)[-1].sym
        word = word + [Phone(SPLIT.get(v, v)[-1], True)]
    elif name == "open" and not last.vowel and len(word) > 2:
        word = word[:-1]
    return word


def _part(word: list, light: bool) -> list:
    """Part consonants that stand together. Light: only runs of three or more, parted once."""
    out, k = [], 0
    while k < len(word):
        if word[k].vowel:
            out.append(word[k])
            k += 1
            continue
        run = []
        while k < len(word) and not word[k].vowel:
            run.append(word[k])
            k += 1
        if light:
            out += run[:1] + ([Phone(PARTING_VOWEL, True)] if len(run) >= 3 else []) + run[1:]
        else:
            for j, c in enumerate(run):
                if j:
                    out.append(Phone(PARTING_VOWEL, True))
                out.append(c)
    return out


def _catch(word: list, light: bool, after_word: bool) -> list:
    out = []
    for k, p in enumerate(word):
        if p.vowel and ((k > 0 and word[k - 1].vowel) or (k == 0 and after_word and not light)):
            out.append(Phone("ʔ", False))
        out.append(p)
    return out


def _table(sounds: dict) -> dict:
    """sound -> (what it becomes, light only, follows another knob). First knob in the order wins."""
    table = {}
    for knob in ORDER:
        name, light = setting(sounds.get(knob))
        variant = KNOBS[knob]["variants"].get(name) if name else None
        if not variant or KNOBS[knob].get("group") != "Consonants":
            continue
        for src, dst in variant.get("map", {}).items():
            table.setdefault(src, (dst, light, bool(variant.get("then"))))
    return table


def bend(sounds_text: str, sounds: dict, reading: str = "en-us") -> str:
    """One sentence's sounds, reshaped by an accent's knob settings."""
    sounds = sounds or {}
    check(sounds)
    get = lambda k: setting(sounds.get(k))
    variant = lambda k, name: KNOBS[k]["variants"][name]
    table = _table(sounds)
    items = [s for s in SEPARATOR.split(sounds_text) if s != ""]
    out, prev_word = [], False
    for n, item in enumerate(items):
        if SEPARATOR.fullmatch(item):
            out.append(item)
            prev_word = prev_word and item.isspace()
            continue
        word = parse_word(item)
        if reading == "en-us":
            # Put the r back where American spelling hides it, and undo the
            # American habit of flicking t (better -> bedder), so the knobs
            # get a plain t and a plain r to work on.
            fixed = []
            for k, p in enumerate(word):
                nxt = word[k + 1].sym if k + 1 < len(word) else ""
                if p.sym == "ɚ":
                    p.sym = "ə"
                    fixed += [p, Phone("ɹ", False)]
                elif p.sym == "ɜ" and nxt != "ɹ":
                    fixed += [p, Phone("ɹ", False)]
                elif p.sym == "ɾ":
                    p.sym = "t"
                    fixed.append(p)
                else:
                    fixed.append(p)
            word = fixed
        name, light = get("stress")
        if name:
            _restress(word, name, light)
        name, light = get("vowel_set")
        if name:
            _vowel_set(word, variant("vowel_set", name), light)
        name, _ = get("diphthongs")
        if name:
            word = _glides(word, name, variant("diphthongs", name))
        name, _ = get("length")
        if name:
            word = _length(word, name)
        word = _consonants(word, table)
        before_pause = n + 1 >= len(items) or not items[n + 1].isspace()
        name, light = get("endings")
        if name and (before_pause or not light):
            word = _endings(word, name)
        name, light = get("clusters")
        if name:
            word = _part(word, light)
        name, light = get("joins")
        if name:
            word = _catch(word, light, prev_word)
        # The same consonant twice running is said once.
        said = []
        for p in word:
            if said and not p.vowel and not said[-1].vowel and said[-1].sym == p.sym:
                continue
            said.append(p)
        out.append("".join(p.text() for p in said))
        prev_word = True
    return "".join(out)


# ---------------------------------------------------------------------------
# Recipes: a tongue at a thickness, and accents kept on file.

NATIVE = 5   # one past the last step: the tongue's whole mouth, not meant for plain speech


def recipe(tongue: str, thickness: int) -> dict:
    """A tongue's habits laid over each other up to a thickness.

    0 is plain speech; 1 to 4 are the steps that keep a line recognisable;
    5 (NATIVE) is everything the tongue's own rules say, for native words.
    """
    if tongue not in TONGUES:
        raise ValueError(f"no tongue {tongue!r}; have {', '.join(TONGUES)}")
    t = TONGUES[tongue]
    thickness = max(0, min(int(thickness), NATIVE))
    acc = {"label": t["label"], "tongue": tongue, "thickness": thickness,
           "reading": t.get("reading", "en-us"), "sounds": {}, "delivery": {},
           "voices": list(t.get("voices", [])), "mix": t.get("mix", "")}
    steps = t.get("step", [])
    for step in steps[:min(thickness, len(steps))]:
        acc["sounds"].update(step.get("sounds", {}))
        acc["delivery"].update(step.get("delivery", {}))
    if thickness >= NATIVE:
        acc["sounds"] = dict(t.get("native", {}).get("sounds", acc["sounds"]))
    return acc


def knob_meta() -> list:
    """The knobs as the panel shows them."""
    out = []
    for k in ORDER:
        spec = KNOBS[k]
        out.append({"id": k, "label": spec["label"], "group": spec["group"], "help": spec.get("help", ""),
                    "levels": spec.get("levels", True),
                    "variants": [{"id": v, "label": d["label"]} for v, d in spec["variants"].items()]})
    return out


def _value(v) -> str:
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, (int, float)):
        return repr(round(v, 4) if isinstance(v, float) else v)
    if isinstance(v, list):
        return "[" + ", ".join(_value(x) for x in v) + "]"
    return '"' + str(v).replace("\\", "\\\\").replace('"', '\\"') + '"'


def dump(tables: dict, header: str) -> str:
    """Write {name: {key: value or {key: value}}} as TOML."""
    lines = [header.rstrip(), ""]
    for name, table in tables.items():
        lines.append(f"[{name}]")
        subs = []
        for k, v in table.items():
            if isinstance(v, dict):
                subs.append((k, v))
            elif v not in (None, "", []):
                lines.append(f"{k} = {_value(v)}")
        for k, sub in subs:
            if sub:
                lines.append(f"{k} = {{ " + ", ".join(f"{a} = {_value(b)}" for a, b in sub.items()) + " }")
        lines.append("")
    return "\n".join(lines)


ACCENTS_FILE = HERE / "accents.toml"
ACCENTS_HEADER = """# Accents made in the panel (`voice.py studio`). One [name] per accent.
# An accent is shared by many speakers: it holds how sounds are bent
# (sounds.toml lists the knobs) and the rhythm and melody of the speech.
# The panel rewrites this file when an accent is saved; notes added by hand
# are not kept."""


def load_accents() -> dict:
    return _toml("accents.toml")


def save_accent(name: str, accent: dict) -> None:
    if not re.fullmatch(r"[a-z0-9_]{1,40}", name):
        raise ValueError("an accent's name is lower-case letters, digits and _ (for example roduro_coast)")
    check(accent.get("sounds", {}))
    all_ = load_accents()
    all_[name] = {"label": accent.get("label", name), "tongue": accent.get("tongue", ""),
                  "thickness": accent.get("thickness"), "reading": accent.get("reading", "en-us"),
                  "voices": accent.get("voices", []),
                  "sounds": {k: v for k, v in accent.get("sounds", {}).items() if setting(v)[0]},
                  "delivery": dict(accent.get("delivery", {}))}
    ACCENTS_FILE.write_text(dump(all_, ACCENTS_HEADER), encoding="utf-8")


def delete_accent(name: str) -> None:
    all_ = load_accents()
    if all_.pop(name, None) is not None:
        ACCENTS_FILE.write_text(dump(all_, ACCENTS_HEADER), encoding="utf-8")


def find(name: str) -> dict:
    """An accent by name: a saved one, or a tongue at a thickness ("qotiro@2")."""
    saved = load_accents()
    if name in saved:
        return saved[name]
    tongue, _, level = name.partition("@")
    if tongue in TONGUES:
        return recipe(tongue, NATIVE if level == "native" else int(level) if level else 2)
    raise ValueError(f"no accent called {name!r}. Saved: {', '.join(saved) or '(none)'}; "
                     f"tongues: {', '.join(t + '@1..4' for t in TONGUES)} (or @native)")
