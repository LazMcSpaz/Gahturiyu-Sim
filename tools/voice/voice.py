#!/usr/bin/env python3
"""The voice tool: turns a line of text into a spoken clip for an NPC.

Nothing here runs inside the game. It makes sound files ahead of time; the
game would only ever play them. See README.md beside this file.

    python tools/voice/voice.py setup
    python tools/voice/voice.py voices
    python tools/voice/voice.py say "Mind the road." --voice vctk:p247 -o out.ogg
    python tools/voice/voice.py sheet audition.toml -o out/audition
    python tools/voice/voice.py batch --voices a,b -o assets/voice
    python tools/voice/voice.py check out.ogg --text "Mind the road."

Two engines sit behind one set of dials:
  kokoro  the best-sounding one. American and British voices, plus voices
          trained on other languages that colour their English. Voices can
          be mixed ("bm_george*0.6+am_onyx*0.4").
  vctk    109 real speakers recorded with their own regional accents
          (Scottish, Irish, Welsh, Northern English, American, ...).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import re
import sys
import tarfile
import tomllib
import unicodedata
import urllib.request
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import accents  # noqa: E402  (the accent knobs; beside this file)

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
MODELS = Path(os.environ.get("GAHT_VOICE_MODELS", HERE / "models"))

# ---------------------------------------------------------------------------
# Models: where they come from and what they must hash to.

KOKORO_REL = "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/"
SHERPA_REL = "https://github.com/k2-fsa/sherpa-onnx/releases/download/"
DOWNLOADS = [
    # (group, file or archive name, url, sha256, the path that proves it's installed)
    ("voices", "kokoro-v1.0.onnx", KOKORO_REL + "kokoro-v1.0.onnx",
     "7d5df8ecf7d4b1878015a32686053fd0eebe2bc377234608764cc0ef3636a6c5", "kokoro-v1.0.onnx"),
    ("voices", "voices-v1.0.bin", KOKORO_REL + "voices-v1.0.bin",
     "bca610b8308e8d99f32e6fe4197e7ec01679264efed0cac9140fe9c29f1fbf7d", "voices-v1.0.bin"),
    ("voices", "vits-piper-en_GB-vctk-medium.tar.bz2",
     SHERPA_REL + "tts-models/vits-piper-en_GB-vctk-medium.tar.bz2",
     "abafd35bdab0a72a3c6b947228ae3cccdf3624db83313c77d1abb5cbc75e1f64",
     "vits-piper-en_GB-vctk-medium/en_GB-vctk-medium.onnx"),
    ("ears", "sherpa-onnx-whisper-base.en.tar.bz2",
     SHERPA_REL + "asr-models/sherpa-onnx-whisper-base.en.tar.bz2",
     "475bc7052ce299c007f6d5d5407ba8601f819a2867f6eecee510ed17df581542",
     "sherpa-onnx-whisper-base.en/base.en-encoder.int8.onnx"),
]
VCTK_DIR = "vits-piper-en_GB-vctk-medium"
EARS_DIR = "sherpa-onnx-whisper-base.en"

KOKORO_ACCENTS = {
    "a": ("American", "en-us"), "b": ("British", "en-gb"),
    "e": ("Spanish-coloured", "en-us"), "f": ("French-coloured", "en-gb"),
    "h": ("Hindi-coloured", "en-gb"), "i": ("Italian-coloured", "en-gb"),
    "j": ("Japanese-coloured", "en-us"), "p": ("Portuguese-coloured", "en-us"),
    "z": ("Mandarin-coloured", "en-us"),
}


class Problem(Exception):
    """Something the person asking can fix; the message says what."""


def die(msg: str) -> None:
    raise Problem(msg)


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def cmd_setup(args) -> None:
    """Fetch the models (about 520 MB down, 850 MB on disk)."""
    MODELS.mkdir(parents=True, exist_ok=True)
    for group, name, url, want, proof in DOWNLOADS:
        if group == "ears" and (args.no_ears or args.studio):
            continue
        if args.studio and not name.startswith(("kokoro", "voices")):
            continue
        if (MODELS / proof).exists():
            print(f"have   {proof}")
            continue
        dest = MODELS / name
        print(f"fetch  {name} ...", flush=True)
        tmp = dest.with_suffix(dest.suffix + ".part")
        with urllib.request.urlopen(url) as r, open(tmp, "wb") as f:
            while block := r.read(1 << 20):
                f.write(block)
        got = sha256(tmp)
        if got != want:
            tmp.unlink()
            die(f"{name} downloaded but doesn't match its fingerprint (got {got[:12]}, "
                f"want {want[:12]}). Nothing was installed from it.")
        tmp.replace(dest)
        if name.endswith(".tar.bz2"):
            with tarfile.open(dest) as tar:
                tar.extractall(MODELS, filter="data")
            dest.unlink()
        print(f"ok     {proof}")
    print(f"Models are in {MODELS}")


def need(proof: str) -> Path:
    p = MODELS / proof
    if not p.exists():
        die(f"missing {p}. Run:  python tools/voice/voice.py setup")
    return p


# ---------------------------------------------------------------------------
# Data files.

def load_toml(path: Path) -> dict:
    with open(path, "rb") as f:
        return tomllib.load(f)


DIALS = load_toml(HERE / "dials.toml")
DEFAULTS: dict = DIALS["defaults"]
ENG: dict = DIALS["engine"]
MULTIPLIED = {"speed", "pause", "range", "size"}


def load_profiles() -> dict:
    p = Path(os.environ.get("GAHT_VOICE_FILE", HERE / "voices.toml"))
    return load_toml(p) if p.exists() else {}


def load_lexicon() -> dict:
    out = {}
    p = HERE / "lexicon.txt"
    if p.exists():
        for line in p.read_text(encoding="utf-8").splitlines():
            if not line.strip() or line.lstrip().startswith("#") or "|" not in line:
                continue
            word, ipa = (x.strip() for x in line.split("|", 1))
            out[fold(word)] = ipa
    return out


def fold(word: str) -> str:
    return unicodedata.normalize("NFC", word).casefold()


def load_vctk() -> dict:
    out = {}
    for line in (HERE / "vctk_speakers.tsv").read_text(encoding="utf-8").splitlines():
        if line.startswith("#") or not line.strip():
            continue
        f = line.split("\t") + ["", "", "", "", ""]
        out[f[0]] = {"age": f[1], "sex": f[2], "accent": f[3], "region": f[4]}
    return out


# ---------------------------------------------------------------------------
# A voice: which engine and speaker, plus every dial.

@dataclass
class Voice:
    engine: str = "kokoro"
    base: str = "bm_george"
    reading: str = ""         # which English the text is read as first: en-us, en-gb, en-gb-scotland ...
    accent: dict = field(default_factory=dict)   # how the sounds are then bent, and the rhythm (accents.py)
    dials: dict = field(default_factory=lambda: dict(DEFAULTS))
    name: str = ""

    def describe(self) -> str:
        changed = {k: v for k, v in self.dials.items() if v != DEFAULTS[k]}
        bits = [f"{self.engine}:{self.base}"]
        if self.accent.get("name"):
            bits.append(f"accent={self.accent['name']}")
        elif self.accent.get("sounds"):
            bits.append("sounds=" + ",".join(f"{k}:{v}" for k, v in self.accent["sounds"].items()))
        if self.reading:
            bits.append(f"reading={self.reading}")
        bits += [f"{k}={round(v, 3)}" for k, v in changed.items()]
        return " ".join(bits)


def accent_named(name: str) -> dict:
    """A saved accent, or a tongue at a thickness ("qotiro@2")."""
    try:
        acc = dict(accents.find(name))
    except ValueError as e:
        die(str(e))
    acc["name"] = name
    return acc


def voice_from(spec: dict, name: str = "") -> Voice:
    """Build a voice from a table: base = "engine:speaker", an accent, then any dials."""
    v = Voice(name=name)
    spec = dict(spec)
    base = spec.pop("base", None)
    if base:
        if ":" not in base:
            die(f"base {base!r} should look like kokoro:bm_george or vctk:p247")
        v.engine, v.base = base.split(":", 1)
    if v.engine not in ("kokoro", "vctk"):
        die(f"unknown engine {v.engine!r} (kokoro or vctk)")
    v.reading = spec.pop("reading", "")
    if "accent" in spec:
        v.accent = accent_named(spec.pop("accent"))
    if "sounds" in spec or "delivery" in spec:   # an accent written out in place
        v.accent = {**v.accent, "sounds": {**v.accent.get("sounds", {}), **spec.pop("sounds", {})},
                    "delivery": {**v.accent.get("delivery", {}), **spec.pop("delivery", {})}}
        v.accent.pop("name", None)
    try:
        accents.check(v.accent.get("sounds", {}))
    except ValueError as e:
        die(str(e))
    for k in v.accent.get("delivery", {}):
        if k not in DEFAULTS:
            die(f"unknown dial {k!r} in the accent; the dials are {', '.join(DEFAULTS)}")
    for k, val in spec.items():
        if k in DEFAULTS:
            v.dials[k] = float(val)
        elif k not in ("label", "group", "id", "text", "emotion", "note", "speaks"):
            die(f"unknown dial {k!r}; the dials are {', '.join(DEFAULTS)}")
    return v


def resolve_voice(arg: str, sets: list, accent: str | None = None) -> Voice:
    """--voice is a name from voices.toml, or a raw base like vctk:p247."""
    if ":" in arg:
        v = voice_from({"base": arg}, name=arg)
    else:
        profiles = load_profiles()
        if arg not in profiles:
            die(f"no voice called {arg!r} in voices.toml. Have: {', '.join(profiles) or '(none yet)'}")
        v = voice_from(profiles[arg], name=arg)
    if accent:
        v.accent = accent_named(accent)
    for s in sets or []:
        if "=" not in s:
            die(f"--set wants dial=value, got {s!r}")
        k, val = s.split("=", 1)
        if k == "reading":
            v.reading = val
        elif k in DEFAULTS:
            v.dials[k] = float(val)
        else:
            die(f"unknown dial {k!r}; the dials are {', '.join(DEFAULTS)}")
    return v


def with_emotion(dials: dict, emotion: str | None, delivery: dict | None = None) -> dict:
    """Fold an accent's rhythm, an emotion ("angry" or "angry:0.5") and the age dial into plain dials."""
    d = dict(dials)

    def fold_in(bundle: dict, strength: float) -> None:
        for k, val in bundle.items():
            if k in MULTIPLIED:
                d[k] *= float(val) ** strength
            else:
                d[k] += float(val) * strength

    if delivery:
        fold_in(delivery, 1.0)
    if emotion:
        name, _, s = emotion.partition(":")
        if name not in DIALS["emotion"]:
            die(f"unknown emotion {name!r}; have {', '.join(DIALS['emotion'])}")
        fold_in(DIALS["emotion"][name], float(s) if s else 1.0)
    if d["age"] > 0:
        fold_in(DIALS["age"], min(d["age"], 1.0))
    d["speed"] = min(max(d["speed"], 0.5), 2.0)
    for k in ("breath", "rough", "tremor"):
        d[k] = min(max(d[k], 0.0), 1.0)
    d["energy"] = min(max(d["energy"], -1.0), 1.0)
    return d


# ---------------------------------------------------------------------------
# Text to sounds.

_espeak_ready = False
_backends: dict = {}
SENTENCE_END = re.compile(r"(?<=[.!?…])\s+")
WORD = re.compile(r"[^\W\d_]+(?:['’][^\W\d_]+)*", re.UNICODE)


def espeak(lang: str):
    global _espeak_ready
    from phonemizer.backend import EspeakBackend
    from phonemizer.backend.espeak.wrapper import EspeakWrapper
    if not _espeak_ready:
        import espeakng_loader
        EspeakWrapper.set_data_path(espeakng_loader.get_data_path())
        EspeakWrapper.set_library(espeakng_loader.get_library_path())
        _espeak_ready = True
    if lang not in _backends:
        try:
            _backends[lang] = EspeakBackend(lang, preserve_punctuation=True, with_stress=True)
        except Exception as e:
            die(f"accent {lang!r} isn't one the sound-maker knows ({e})")
    return _backends[lang]


def to_sounds(text: str, accent: str, lexicon: dict) -> list:
    """The line as one string of sounds per sentence."""
    sentences = [s for s in SENTENCE_END.split(text.strip()) if s.strip()]
    out = []
    for sentence in sentences:
        parts, pos = [], 0
        for m in WORD.finditer(sentence):
            word = fold(m.group()).replace("’", "'")
            ipa = lexicon.get(word)
            if ipa is None and word.endswith("'s") and word[:-2] in lexicon:
                ipa = lexicon[word[:-2]] + "z"
            if ipa is None:
                continue
            if m.start() > pos:
                parts.append(("text", sentence[pos:m.start()]))
            parts.append(("ipa", ipa))
            pos = m.end()
        if pos < len(sentence):
            parts.append(("text", sentence[pos:]))
        sounds = ""
        for kind, chunk in parts:
            if kind == "ipa":
                piece = chunk
            elif WORD.search(chunk):
                piece = espeak(accent).phonemize([chunk], strip=True)[0]
                # espeak drops the chunk's own edge spaces and edge marks keep theirs
                if chunk[:1].isspace():
                    piece = " " + piece
                if chunk[-1:].isspace():
                    piece = piece + " "
            else:
                piece = chunk  # only spaces and marks
            sounds += piece
        out.append(" ".join(sounds.split()))
    return out


# ---------------------------------------------------------------------------
# The two engines. Each returns (samples, sample rate).

_kokoro = None
_vctk_meta = None


def kokoro():
    global _kokoro
    if _kokoro is None:
        from kokoro_onnx import Kokoro
        _kokoro = Kokoro(str(need("kokoro-v1.0.onnx")), str(need("voices-v1.0.bin")))
    return _kokoro


def kokoro_style(base: str) -> np.ndarray:
    """One voice, or a mix: "bm_george*0.6+am_onyx*0.4"."""
    k = kokoro()
    total, weight = None, 0.0
    for part in base.split("+"):
        name, _, w = part.partition("*")
        name, w = name.strip(), float(w) if w else 1.0
        if name not in k.get_voices():
            die(f"kokoro has no voice {name!r}. Run `voices` to list them.")
        s = k.get_voice_style(name) * w
        total = s if total is None else total + s
        weight += w
    return (total / weight).astype(np.float32)


def speak_kokoro(v: Voice, sounds: list, d: dict, seed: int):
    k = kokoro()
    style = kokoro_style(v.base)
    phonemes = " ".join(sounds)
    known = k.tokenizer.known(phonemes)
    if not known.strip():
        die("nothing in that line the voice can say")
    audio, sr = k.create(known, voice=style, speed=d["speed"], is_phonemes=True, trim=False)
    return np.asarray(audio, dtype=np.float32), sr


def vctk_meta() -> dict:
    global _vctk_meta
    if _vctk_meta is None:
        cfg = need(f"{VCTK_DIR}/en_GB-vctk-medium.onnx.json")
        _vctk_meta = json.loads(cfg.read_text(encoding="utf-8"))
    return _vctk_meta


_probe = None


def vctk_probe():
    """A kept-open copy of the accent model, for plain readings with no randomness."""
    global _probe
    if _probe is None:
        import onnxruntime as ort
        opts = ort.SessionOptions()
        opts.log_severity_level = 3
        _probe = ort.InferenceSession(str(need(f"{VCTK_DIR}/en_GB-vctk-medium.onnx")), opts,
                                      providers=["CPUExecutionProvider"])
    return _probe


def speak_vctk(v: Voice, sounds: list, d: dict, seed: int):
    import onnxruntime as ort
    meta = vctk_meta()
    ids, speakers = meta["phoneme_id_map"], meta["speaker_id_map"]
    if v.base not in speakers:
        die(f"the accent model has no speaker {v.base!r}. Run `voices` to list them.")
    base = meta["inference"]
    # The model's loose reading is random. A fresh session with a fixed seed
    # makes a clip come out the same every time it's made with the same inputs.
    ort.set_seed(seed)
    opts = ort.SessionOptions()
    opts.log_severity_level = 3
    sess = ort.InferenceSession(str(need(f"{VCTK_DIR}/en_GB-vctk-medium.onnx")), opts,
                                providers=["CPUExecutionProvider"])
    sr = meta["audio"]["sample_rate"]
    sid = np.array([speakers[v.base]], dtype=np.int64)
    seqs = []
    for sentence in sounds:
        seq = list(ids["^"]) + list(ids["_"])
        for ch in sentence:
            if ch in ids:
                seq += ids[ch] + ids["_"]
        seqs.append(np.array([seq + list(ids["$"])], dtype=np.int64))

    def run(session, noise, length, noise_w):
        scales = np.array([noise, length, noise_w], dtype=np.float32)
        return [np.asarray(session.run(None, {
            "input": x, "input_lengths": np.array([x.shape[1]], dtype=np.int64),
            "scales": scales, "sid": sid})[0], dtype=np.float32).ravel() for x in seqs]

    length = base["length_scale"] / d["speed"]
    if abs(d["speed"] - 1) > 1e-3:
        # The model gives every sound at least one whole step, so asking for
        # 1.3x gives much less. A few quick plain readings find the setting
        # that really makes this line that much shorter or longer.
        probe = vctk_probe()

        def plain(ls):
            return sum(len(c) for c in run(probe, 0.0, ls, 0.0)) / sr

        l0, t0 = base["length_scale"], plain(base["length_scale"])
        want = t0 / d["speed"]
        l1, t1 = length, plain(length)
        for _ in range(5):
            if abs(t1 - want) < 0.02 * want or abs(t1 - t0) < 1e-6:
                break
            nxt = min(max(l1 + (want - t1) * (l1 - l0) / (t1 - t0), 0.3), 4.0)
            l0, t0, l1 = l1, t1, nxt
            t1 = plain(l1)
        length = l1
    chunks = [trim(c, sr, pad=0.01) for c in
              run(sess, base["noise_scale"] * d["variation"], length, base["noise_w"] * d["variation"])]
    gap = np.zeros(int(ENG["sentence_gap_seconds"] * sr), dtype=np.float32)
    out = []
    for c in chunks:
        if out:
            out.append(gap)
        out.append(c)
    return np.concatenate(out), sr


# ---------------------------------------------------------------------------
# Shaping the sound after it's spoken.

def frame_levels(x: np.ndarray, sr: int, seconds: float = 0.01):
    n = max(1, int(sr * seconds))
    usable = len(x) // n * n
    if usable == 0:
        return np.zeros(0), n
    return np.sqrt((x[:usable].reshape(-1, n) ** 2).mean(1)), n


def quiet_frames(x: np.ndarray, sr: int, db: float = -40.0):
    lev, n = frame_levels(x, sr)
    if len(lev) == 0 or lev.max() <= 0:
        return np.ones(len(lev), dtype=bool), n
    return lev <= lev.max() * 10 ** (db / 20), n


def trim(x: np.ndarray, sr: int, pad: float = 0.02) -> np.ndarray:
    """Cut the silence off both ends, keeping a short pad."""
    quiet, n = quiet_frames(x, sr, -45.0)
    loud = np.flatnonzero(~quiet)
    if len(loud) == 0:
        return x
    a = max(0, loud[0] * n - int(pad * sr))
    b = min(len(x), (loud[-1] + 1) * n + int(pad * sr))
    return x[a:b]


def scale_pauses(x: np.ndarray, sr: int, factor: float) -> np.ndarray:
    """Stretch or shrink the gaps inside a line (never the ends)."""
    if abs(factor - 1.0) < 1e-3:
        return x
    quiet, n = quiet_frames(x, sr)
    least = max(1, round(ENG["pause_min_seconds"] / 0.01))
    out, i, pos = [], 0, 0
    while i < len(quiet):
        if not quiet[i]:
            i += 1
            continue
        j = i
        while j < len(quiet) and quiet[j]:
            j += 1
        if i > 0 and j < len(quiet) and j - i >= least:
            a, b = i * n, j * n
            mid = (a + b) // 2
            want = max(int(0.04 * sr), int((b - a) * factor))
            if want >= b - a:
                out += [x[pos:mid], np.zeros(want - (b - a), dtype=x.dtype)]
                pos = mid
            else:
                cut = (b - a) - want
                out.append(x[pos:mid - cut // 2])
                pos = mid + (cut - cut // 2)
        i = j
    out.append(x[pos:])
    return np.concatenate(out)


def high_shelf(x: np.ndarray, sr: int, gain_db: float, hz: float = 2000.0) -> np.ndarray:
    if abs(gain_db) < 0.05:
        return x
    from scipy.signal import lfilter
    a_ = 10 ** (gain_db / 40)
    w = 2 * math.pi * hz / sr
    alpha = math.sin(w) / 2 * math.sqrt(2)
    c = math.cos(w)
    b = [a_ * ((a_ + 1) + (a_ - 1) * c + 2 * math.sqrt(a_) * alpha),
         -2 * a_ * ((a_ - 1) + (a_ + 1) * c),
         a_ * ((a_ + 1) + (a_ - 1) * c - 2 * math.sqrt(a_) * alpha)]
    a = [(a_ + 1) - (a_ - 1) * c + 2 * math.sqrt(a_) * alpha,
         2 * ((a_ - 1) - (a_ + 1) * c),
         (a_ + 1) - (a_ - 1) * c - 2 * math.sqrt(a_) * alpha]
    return lfilter(np.array(b) / a[0], np.array(a) / a[0], x).astype(np.float32)


def add_breath(x: np.ndarray, sr: int, amount: float, rng) -> np.ndarray:
    """Air: hiss that follows the loudness of the speech."""
    if amount <= 0:
        return x
    from scipy.signal import butter, lfilter
    env = np.abs(x)
    b, a = butter(1, 30 / (sr / 2))
    env = lfilter(b, a, env)
    noise = rng.standard_normal(len(x)).astype(np.float32)
    b, a = butter(2, 1500 / (sr / 2), btype="high")
    noise = lfilter(b, a, noise)
    return (x * (1 - 0.25 * amount) + noise * env * ENG["breath_mix"] * amount).astype(np.float32)


def reshape(x: np.ndarray, sr: int, d: dict, rng) -> np.ndarray:
    """Body size and everything about pitch, done with Praat."""
    pitch, size, rang = d["pitch"], d["size"], d["range"]
    lilt, tremor, rough = d["lilt"], d["tremor"], d["rough"]
    contour = abs(lilt) > 1e-3 or tremor > 1e-3 or rough > 1e-3
    plain = abs(pitch) > 1e-3 or abs(rang - 1) > 1e-3
    resize = abs(size - 1) > 1e-3
    if not (contour or plain or resize):
        return x
    import parselmouth
    from parselmouth.praat import call
    snd = parselmouth.Sound(x.astype(np.float64), sampling_frequency=sr)
    lo, hi = 60.0, 600.0
    if resize:
        # One pass moves the throat size and, while it's there, the plain pitch changes.
        median = call(snd.to_pitch(pitch_floor=lo, pitch_ceiling=hi), "Get quantile", 0, 0, 0.5, "Hertz")
        target = 0.0
        if plain and median == median and median > 0:
            target = median * 2 ** (pitch / 12)
        snd = call(snd, "Change gender", lo, hi, 1.0 / size, target, rang if target else 1.0, 1.0)
        if target:
            pitch, rang, plain = 0.0, 1.0, False
    if contour or plain:
        manip = call(snd, "To Manipulation", 0.01, lo, hi)
        tier = call(manip, "Extract pitch tier")
        count = call(tier, "Get number of points")
        if count >= 2:
            t = np.array([call(tier, "Get time from index", i) for i in range(1, count + 1)])
            f = np.array([call(tier, "Get value at index", i) for i in range(1, count + 1)])
            median = float(np.median(f))
            st = 12 * np.log2(f / median) * rang + pitch
            u = (t - t[0]) / max(t[-1] - t[0], 1e-3)
            st += lilt * np.clip((u - 0.6) / 0.4, 0, 1)
            st += tremor * ENG["tremor_semitones"] * np.sin(2 * math.pi * ENG["tremor_hz"] * t)
            st += rough * ENG["rough_semitones"] * rng.standard_normal(len(t))
            f2 = np.clip(median * 2 ** (st / 12), 50.0, 650.0)
            new = call("Create PitchTier", "p", 0.0, snd.duration)
            for ti, fi in zip(t, f2):
                call(new, "Add point", float(ti), float(fi))
            call([new, manip], "Replace pitch tier")
            snd = call(manip, "Get resynthesis (overlap-add)")
    return np.asarray(snd.values[0], dtype=np.float32)


def finish(x: np.ndarray, sr: int, d: dict) -> np.ndarray:
    """Trim, set how hard and how loud it is, fade the ends."""
    x = trim(x, sr)
    e = d["energy"]
    x = high_shelf(x, sr, ENG["energy_tilt_db"] * e)
    peak = float(np.abs(x).max()) or 1.0
    x = x / peak
    if e > 0:  # press the loud parts down so the whole line sits forward
        g = 1 + 2 * e
        x = np.tanh(g * x) / math.tanh(g)
    rms = float(np.sqrt((x ** 2).mean())) or 1.0
    x = x * (10 ** ((ENG["level_db"] + ENG["energy_level_db"] * e) / 20) / rms)
    peak = float(np.abs(x).max())
    if peak > 0.89:
        x = x * (0.89 / peak)
    n = min(int(0.005 * sr), len(x) // 2)
    if n:
        ramp = np.linspace(0, 1, n, dtype=np.float32)
        x[:n] *= ramp
        x[-n:] *= ramp[::-1]
    return x.astype(np.float32)


def seed_for(*keys) -> int:
    """A clip's randomness is keyed by what it is, not when it's made."""
    h = hashlib.sha256("\x1f".join(str(k) for k in keys).encode("utf-8")).digest()
    return int.from_bytes(h[:4], "little") & 0x7FFFFFFF


def reading_of(v: Voice) -> str:
    if v.reading:
        return v.reading
    if v.accent.get("reading"):
        return v.accent["reading"]
    if v.engine == "kokoro":
        return KOKORO_ACCENTS.get(v.base[:1], ("", "en-us"))[1]
    return vctk_meta()["espeak"]["voice"]


def sounds_of(v: Voice, text: str) -> tuple:
    """The line's sounds as plain English reads them, and as this voice's accent bends them."""
    reading = reading_of(v)
    plain = to_sounds(text, reading, load_lexicon())
    knobs = v.accent.get("sounds") or {}
    bent = [accents.bend(s, knobs, reading) for s in plain] if knobs else list(plain)
    return plain, bent


def render(v: Voice, text: str, emotion: str | None = None, take: int = 0):
    """One line, one voice: (samples, sample rate, the sounds it was said with)."""
    d = with_emotion(v.dials, emotion, v.accent.get("delivery"))
    _, sounds = sounds_of(v, text)
    if not sounds:
        die("there's no text to say")
    # Keyed by who is speaking and what they say, not by the dials: turning a
    # dial changes that one thing and leaves the reading itself alone.
    seed = seed_for(v.engine, v.base, reading_of(v), text, take)
    rng = np.random.default_rng(seed)
    x, sr = (speak_kokoro if v.engine == "kokoro" else speak_vctk)(v, sounds, d, seed)
    x = trim(x, sr)
    x = scale_pauses(x, sr, d["pause"])
    x = reshape(x, sr, d, rng)
    x = add_breath(x, sr, d["breath"], rng)
    x = finish(x, sr, d)
    return x, sr, " | ".join(sounds)


def save(path: Path, x: np.ndarray, sr: int) -> None:
    import soundfile as sf
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.suffix.lower() == ".ogg":
        sf.write(str(path), x, sr, format="OGG", subtype="VORBIS")
    elif path.suffix.lower() == ".wav":
        sf.write(str(path), x, sr, subtype="PCM_16")
    else:
        die(f"{path.name}: clips are .ogg or .wav")


# ---------------------------------------------------------------------------
# Checking a clip without ears: measure it, and have a listener model
# write down what it hears.

_ears = None


def measure(x: np.ndarray, sr: int) -> dict:
    import parselmouth
    snd = parselmouth.Sound(x.astype(np.float64), sampling_frequency=sr)
    f = snd.to_pitch(pitch_floor=60, pitch_ceiling=600).selected_array["frequency"]
    f = f[f > 0]
    out = {"seconds": round(len(x) / sr, 2),
           "level_db": round(20 * math.log10(max(float(np.sqrt((x ** 2).mean())), 1e-9)), 1),
           "peak_db": round(20 * math.log10(max(float(np.abs(x).max()), 1e-9)), 1)}
    if len(f) > 5:
        med = float(np.median(f))
        out["pitch_hz"] = round(med)
        out["pitch_spread_st"] = round(float(np.std(12 * np.log2(f / med))), 2)
    return out


SHORT_FORMS = (("n't", " not"), ("'ve", " have"), ("'re", " are"), ("'ll", " will"), ("'m", " am"), ("'d", " would"))


def words_of(text: str) -> list:
    """The words of a line, with "I've" and "I have" counted as the same thing."""
    out = []
    for w in WORD.findall(text):
        w = fold(w).replace("’", "'")
        for short, full in SHORT_FORMS:
            if w.endswith(short):
                w = w[:-len(short)] + full
        out += w.split()
    return out


def hear(x: np.ndarray, sr: int, text: str | None = None) -> dict:
    """What a speech-to-text model makes of the clip, and how well that matches."""
    global _ears
    import sherpa_onnx
    from scipy.signal import resample_poly
    if _ears is None:
        d = need(f"{EARS_DIR}/base.en-encoder.int8.onnx").parent
        _ears = sherpa_onnx.OfflineRecognizer.from_whisper(
            encoder=str(d / "base.en-encoder.int8.onnx"), decoder=str(d / "base.en-decoder.int8.onnx"),
            tokens=str(d / "base.en-tokens.txt"), language="en", task="transcribe", num_threads=2)
    g = math.gcd(16000, sr)
    y = resample_poly(x, 16000 // g, sr // g).astype(np.float32)
    y = np.concatenate([np.zeros(3200, np.float32), y, np.zeros(3200, np.float32)])
    s = _ears.create_stream()
    s.accept_waveform(16000, y)
    _ears.decode_stream(s)
    heard = s.result.text.strip()
    out = {"heard": heard}
    if text is not None:
        import difflib
        lex = load_lexicon()
        # invented names can't be transcribed, so they aren't asked for
        want = [w for w in words_of(text) if w not in lex and w.removesuffix("'s") not in lex]
        got = words_of(heard)
        # The share of the wanted words that were heard, in order.
        found = sum(b.size for b in difflib.SequenceMatcher(None, want, got, autojunk=False).get_matching_blocks())
        out["match"] = round(found / len(want), 2) if want else 1.0
    return out


# ---------------------------------------------------------------------------
# Commands.

def cmd_voices(args) -> None:
    want = (args.accent or "").casefold()
    rows = []
    for name in kokoro().get_voices():
        acc = KOKORO_ACCENTS.get(name[0], ("?", ""))[0]
        rows.append((f"kokoro:{name}", "F" if name[1] == "f" else "M", acc, ""))
    for sid, m in load_vctk().items():
        rows.append((f"vctk:{sid}", m["sex"], m["accent"], m["region"]))
    for base, sex, acc, region in rows:
        if want and want not in acc.casefold() and want not in region.casefold():
            continue
        if args.sex and sex != args.sex.upper():
            continue
        print(f"{base:22} {sex}  {acc}{'  (' + region + ')' if region else ''}")
    profiles = load_profiles()
    if profiles and not want:
        print("\nNamed voices (voices.toml):")
        for name, spec in profiles.items():
            print(f"  {name:20} {voice_from(spec, name).describe()}")


def report(x, sr, text, check: bool) -> dict:
    m = measure(x, sr)
    if check:
        m.update(hear(x, sr, text))
    return m


def cmd_say(args) -> None:
    v = resolve_voice(args.voice, args.set, args.accent)
    x, sr, sounds = render(v, args.text, args.emotion, args.take)
    out = Path(args.out)
    save(out, x, sr)
    m = report(x, sr, args.text, args.check)
    print(f"wrote  {out}")
    print(f"voice  {v.describe()}" + (f"  emotion={args.emotion}" if args.emotion else ""))
    print(f"sounds {sounds}")
    print("clip   " + "  ".join(f"{k}={val}" for k, val in m.items()))


def cmd_check(args) -> None:
    import soundfile as sf
    x, sr = sf.read(args.file, dtype="float32", always_2d=False)
    if x.ndim > 1:
        x = x.mean(1)
    m = measure(x, sr)
    m.update(hear(x, sr, args.text))
    print("  ".join(f"{k}={val}" for k, val in m.items()))


def cmd_sheet(args) -> None:
    """Render a list of clips to compare by ear: clips, index.json, index.html."""
    spec = load_toml(Path(args.spec))
    outdir = Path(args.out)
    outdir.mkdir(parents=True, exist_ok=True)
    default_text = spec.get("text", "Mind the road.")
    items = []
    for n, item in enumerate(spec.get("item", [])):
        ident = item.get("id") or f"clip{n:03d}"
        text = item.get("text", default_text)
        v = voice_from(item, name=ident)
        x, sr, sounds = render(v, text, item.get("emotion"))
        save(outdir / f"{ident}.ogg", x, sr)
        m = report(x, sr, text, not args.no_check)
        items.append({"id": ident, "group": item.get("group", ""), "label": item.get("label", ident),
                      "note": item.get("note", ""), "text": text, "voice": v.describe(),
                      "emotion": item.get("emotion", ""), "file": f"{ident}.ogg", "sounds": sounds, **m})
        flag = "" if m.get("match", 1) >= 0.8 else "   <-- the listener misheard this one"
        print(f"{ident:28} {m['seconds']:>5}s  {m.get('pitch_hz', '-'):>4} Hz  "
              f"heard: {m.get('heard', '(not checked)')}{flag}", flush=True)
    (outdir / "index.json").write_text(json.dumps(items, indent=1, ensure_ascii=False), encoding="utf-8")
    (outdir / "index.html").write_text(sheet_html(spec.get("title", "Voice sheet"), items), encoding="utf-8")
    print(f"\n{len(items)} clips in {outdir}. Open {outdir / 'index.html'} to listen.")


def sheet_html(title: str, items: list) -> str:
    import html
    rows, group = [], None
    for it in items:
        if it["group"] != group:
            group = it["group"]
            rows.append(f"<h2>{html.escape(group)}</h2>")
        rows.append(
            f"<div class=row><audio controls preload=none src=\"{html.escape(it['file'])}\"></audio>"
            f"<div><b>{html.escape(it['label'])}</b> <code>{html.escape(it['id'])}</code><br>"
            f"<small>{html.escape(it['text'])}<br>{html.escape(it['voice'])}"
            f"{' · ' + html.escape(it['emotion']) if it['emotion'] else ''}</small></div></div>")
    return ("<!doctype html><meta charset=utf-8><title>" + html.escape(title) + "</title>"
            "<style>body{font:15px system-ui;max-width:860px;margin:2em auto;padding:0 1em}"
            ".row{display:flex;gap:1em;align-items:center;margin:.5em 0}audio{flex:none;width:260px}"
            "small{color:#666}code{color:#888}</style><h1>" + html.escape(title) + "</h1>" + "\n".join(rows))


def read_lines(folder: Path, slots: set) -> list:
    """The pieces in data/lines that are plain text (no {slots} to fill)."""
    out = []
    for path in sorted(folder.glob("*.txt")):
        for raw in path.read_text(encoding="utf-8").splitlines():
            if not raw.strip() or raw.lstrip().startswith("#"):
                continue
            p = [x.strip() for x in raw.split("|")]
            if len(p) < 3 or p[0] not in slots or not p[2] or "{" in p[2]:
                continue
            tags = [t.strip() for t in p[1].split(",") if t.strip()]
            out.append({"file": path.stem, "slot": p[0], "tags": tags, "text": p[2]})
    return out


def line_key(text: str) -> str:
    """A clip is found by its voice and by this fingerprint of the exact text."""
    return hashlib.sha1(unicodedata.normalize("NFC", text).encode("utf-8")).hexdigest()[:10]


def cmd_batch(args) -> None:
    """Voice every plain line in data/lines for the named voices."""
    profiles = load_profiles()
    names = [n for n in args.voices.split(",") if n]
    for n in names:
        if n not in profiles:
            die(f"no voice called {n!r} in voices.toml")
    lines = read_lines(Path(args.lines), set(args.slots.split(",")))
    if args.limit:
        lines = lines[:args.limit]
    outdir = Path(args.out)
    rows, bad = [], 0
    for name in names:
        spec = profiles[name]
        v = voice_from(spec, name=name)
        speaks = spec.get("speaks")  # optional: only lines tagged for this way of speaking
        for ln in lines:
            ways = [t.split("=", 1)[1] for t in ln["tags"] if t.startswith("voice=")]
            if ways and speaks and speaks not in ways:
                continue
            emotion = next((DIALS["line_emotion"][t] for t in DIALS["line_emotion"] if t in ln["tags"]), None)
            key = line_key(ln["text"])
            rel = f"{name}/{key}.ogg"
            if (outdir / rel).exists() and not args.redo:
                rows.append((name, key, ln["slot"], rel, ln["text"]))
                continue
            x, sr, _ = render(v, ln["text"], emotion)
            note = ""
            if not args.no_check:
                h = hear(x, sr, ln["text"])
                # The accent model reads a little differently each take. If the
                # listener can't follow this one, try a few more and keep the clearest.
                for take in range(1, args.takes if v.engine == "vctk" else 1):
                    if h["match"] >= 0.8:
                        break
                    x2, _, _ = render(v, ln["text"], emotion, take)
                    h2 = hear(x2, sr, ln["text"])
                    if h2["match"] > h["match"]:
                        x, h, note = x2, h2, f"   (take {take})"
                if h["match"] < 0.8:
                    bad += 1
                    note = f"   <-- heard: {h['heard']!r}"
            save(outdir / rel, x, sr)
            rows.append((name, key, ln["slot"], rel, ln["text"]))
            print(f"{rel:40} {ln['text']}{note}", flush=True)
    manifest = outdir / "manifest.tsv"
    manifest.parent.mkdir(parents=True, exist_ok=True)
    if manifest.exists():  # keep the voices this run didn't touch
        kept = [tuple(l.split("\t")) for l in manifest.read_text(encoding="utf-8").splitlines()
                if l and not l.startswith("#") and l.split("\t")[0] not in names]
        rows = kept + rows
    manifest.write_text("# voice\tline key\tslot\tfile\ttext\n" +
                        "".join("\t".join(r) + "\n" for r in rows), encoding="utf-8")
    print(f"\n{len(rows)} clips listed in {manifest}" + (f"; {bad} the listener misheard" if bad else ""))


def cmd_studio(args) -> None:
    """Open the panel of knobs and sliders."""
    need("kokoro-v1.0.onnx")
    import studio
    studio.serve(args.port, not args.no_browser)


def cmd_accents(args) -> None:
    """List the tongues and saved accents, or show what one does to a line."""
    if args.name:
        acc = accent_named(args.name)
        v = voice_from({"base": "kokoro:bm_george"})
        v.accent = acc
        plain, bent = sounds_of(v, args.text)
        print(f"{args.name}: reads as {reading_of(v)}")
        for k, val in acc.get("sounds", {}).items():
            name, light = accents.setting(val)
            spec = accents.KNOBS[k]
            print(f"  {spec['label']:22} {spec['variants'][name]['label']}{'  (light)' if light else ''}")
        for k, val in acc.get("delivery", {}).items():
            print(f"  {k:22} {val}")
        print("plain  " + " | ".join(plain))
        print("bent   " + " | ".join(bent))
        return
    print("Tongues (use as NAME@1 to NAME@4 for plain speech, NAME@native for words in that tongue):")
    for k, t in accents.TONGUES.items():
        print(f"  {k:10} {t['label']}")
    saved = accents.load_accents()
    print("\nSaved accents (accents.toml):" + ("" if saved else " none yet"))
    for k, a in saved.items():
        print(f"  {k:20} {a.get('label', '')}")


def cmd_selftest(args) -> None:
    """Does each dial do what it says? Measured, not listened to."""
    import parselmouth
    line = "Keep your hand on your purse round here. Mind the road."
    fails = []

    def clip(base, emotion=None, **dials):
        return render(voice_from({"base": base, **dials}), line, emotion)[:2]

    def resonances(x, sr, ceiling):
        """Where the 2nd and 3rd resonances of the throat sit, on average, while voiced."""
        snd = parselmouth.Sound(x.astype(np.float64), sampling_frequency=sr)
        pitch = snd.to_pitch(pitch_floor=60, pitch_ceiling=600)
        f = snd.to_formant_burg(maximum_formant=ceiling)
        ts = [t for t in np.arange(0.05, snd.duration - 0.05, 0.01)
              if not math.isnan(pitch.get_value_at_time(t))]
        return [float(np.nanmedian([f.get_value_at_time(k, t) for t in ts])) for k in (2, 3)]

    def resonance_shift(x0, x1, sr, size):
        """How far x1's resonances sit below x0's, as a ratio (1.1 = 10% lower)."""
        # The tracker looks for five resonances under a ceiling; a bigger throat
        # has them lower, so its ceiling moves with the size being tested.
        a, b = resonances(x0, sr, 5000.0), resonances(x1, sr, 5000.0 / size)
        return float(np.mean([p / q for p, q in zip(a, b)]))

    def expect(name, ok, detail):
        print(f"{'ok  ' if ok else 'FAIL'} {name:34} {detail}")
        if not ok:
            fails.append(name)

    for base in ("kokoro:bm_george", "vctk:p247"):
        x0, sr = clip(base)
        m0 = measure(x0, sr)
        x1, _ = clip(base)
        expect(f"{base} same clip twice", len(x0) == len(x1) and bool(np.array_equal(x0, x1)), "identical samples")
        h = hear(x0, sr, line)
        expect(f"{base} can be understood", h["match"] >= 0.8, f"heard {h['heard']!r}")
        for st in (-3.0, 3.0):
            m = measure(*clip(base, pitch=st))
            got = 12 * math.log2(m["pitch_hz"] / m0["pitch_hz"])
            expect(f"{base} pitch {st:+}", abs(got - st) < 0.8, f"moved {got:+.2f} semitones")
        for size in (0.9, 1.12):
            x, _ = clip(base, size=size)
            got = resonance_shift(x0, x, sr, size)
            expect(f"{base} size {size}", abs(got - size) < 0.05, f"resonances sit x{got:.3f} lower")
            m = measure(x, sr)
            drift = 12 * math.log2(m["pitch_hz"] / m0["pitch_hz"])
            expect(f"{base} size {size} keeps pitch", abs(drift) < 0.8, f"pitch drift {drift:+.2f} semitones")
        m = measure(*clip(base, range=0.5))
        expect(f"{base} range 0.5", m["pitch_spread_st"] < 0.75 * m0["pitch_spread_st"],
               f"spread {m0['pitch_spread_st']} -> {m['pitch_spread_st']} semitones")
        m = measure(*clip(base, range=1.5))
        expect(f"{base} range 1.5", m["pitch_spread_st"] > 1.2 * m0["pitch_spread_st"],
               f"spread {m0['pitch_spread_st']} -> {m['pitch_spread_st']} semitones")
        m = measure(*clip(base, speed=1.3))
        expect(f"{base} speed 1.3", 0.68 < m["seconds"] / m0["seconds"] < 0.88,
               f"{m0['seconds']}s -> {m['seconds']}s")
        m = measure(*clip(base, pause=2.0))
        expect(f"{base} pause 2", m["seconds"] > m0["seconds"] + 0.1, f"{m0['seconds']}s -> {m['seconds']}s")
        m = measure(*clip(base, energy=1.0))
        expect(f"{base} energy 1", m["level_db"] > m0["level_db"] + 1.5, f"{m0['level_db']} -> {m['level_db']} dB")
        for emotion in DIALS["emotion"]:
            x, _ = clip(base, emotion)
            h = hear(x, sr, line)
            m = measure(x, sr)
            expect(f"{base} {emotion} still clear", h["match"] >= 0.8,
                   f"{m['seconds']}s {m.get('pitch_hz')} Hz, heard {h['heard']!r}")
        x, _ = clip(base, age=1.0)
        h = hear(x, sr, line)
        expect(f"{base} age 1 still clear", h["match"] >= 0.8, f"heard {h['heard']!r}")
    # Accent knobs. Every setting of every knob must change the sounds of a
    # line that has its sounds in it; every tongue must get further from plain
    # speech with each step, by the listener's count.
    probe = ("She sells thick leather straps by the judge's church. Your brother will carry "
             "the heavy water over. A young boy thought of pure joy, going home again tonight. "
             "Nothing happens without a reason, usually. Perhaps nobody is always behind, "
             "I remember. Tomorrow is good.")
    for reading in ("en-us", "en-gb"):
        plain = to_sounds(probe, reading, {})
        for k in accents.ORDER:
            for name in accents.KNOBS[k]["variants"]:
                levels = ["", ":light"] if accents.KNOBS[k].get("levels", True) else [""]
                for lvl in levels:
                    bent = [accents.bend(x, {k: name + lvl}, reading) for x in plain]
                    if bent == plain:
                        expect(f"knob {k}={name}{lvl} ({reading})", False, "changed nothing")
    expect("every accent knob changes the sounds", not any(f.startswith("knob ") for f in fails),
           f"{sum(len(accents.KNOBS[k]['variants']) for k in accents.ORDER)} settings of {len(accents.ORDER)} knobs")
    for tongue, t in accents.TONGUES.items():
        scores = []
        for step in range(0, accents.NATIVE + 1):
            v = voice_from({"base": "kokoro:" + t["mix"]})
            if step:
                v.accent = accents.recipe(tongue, step)
            else:
                v.reading = t["reading"]
            x, sr, _ = render(v, line)
            scores.append(hear(x, sr, line)["match"])
        # Steps 1 to 4 must leave the words recognisable; the native mouth must not.
        expect(f"{tongue} stays plain speech to step 4", min(scores[1:5]) >= 0.7 and scores[5] < min(scores[1:5]) - 0.2,
               "listener's match, plain then steps 1 to 4 then native: " + " ".join(f"{m:.2f}" for m in scores))
    print("\n" + ("all passed" if not fails else f"{len(fails)} failed: {', '.join(fails)}"))
    sys.exit(1 if fails else 0)


def main() -> None:
    ap = argparse.ArgumentParser(description="Make spoken clips for NPC lines.")
    sub = ap.add_subparsers(dest="cmd", required=True)

    s = sub.add_parser("setup", help="download the voice models")
    s.add_argument("--no-ears", action="store_true", help="skip the listener model used by --check")
    s.add_argument("--studio", action="store_true", help="only what the panel needs (about 350 MB)")
    s.set_defaults(fn=cmd_setup)

    s = sub.add_parser("voices", help="list every speaker and named voice")
    s.add_argument("--accent", help="only those whose accent or region contains this")
    s.add_argument("--sex", choices=["m", "f", "M", "F"])
    s.set_defaults(fn=cmd_voices)

    s = sub.add_parser("say", help="one line, one voice, one clip")
    s.add_argument("text")
    s.add_argument("--voice", required=True, help="a name from voices.toml, or kokoro:NAME / vctk:pNNN")
    s.add_argument("--accent", help="a saved accent, or a tongue at a thickness: roduro@2, qotiro@3 ...")
    s.add_argument("--emotion", help="angry, stern, weary, warm, afraid, sly; add :0.5 for half strength")
    s.add_argument("--set", nargs="*", metavar="DIAL=VALUE", help="e.g. pitch=-2 size=1.1 speed=0.9")
    s.add_argument("--take", type=int, default=0, help="a different reading of the same line")
    s.add_argument("--check", action="store_true", help="have the listener model write down what it hears")
    s.add_argument("-o", "--out", default="out/say.ogg")
    s.set_defaults(fn=cmd_say)

    s = sub.add_parser("check", help="measure a clip and transcribe it")
    s.add_argument("file")
    s.add_argument("--text", help="what it was meant to say")
    s.set_defaults(fn=cmd_check)

    s = sub.add_parser("sheet", help="render a list of clips to compare by ear")
    s.add_argument("spec", help="a .toml list of clips (see sheets/)")
    s.add_argument("-o", "--out", required=True)
    s.add_argument("--no-check", action="store_true")
    s.set_defaults(fn=cmd_sheet)

    s = sub.add_parser("batch", help="voice the plain lines in data/lines for named voices")
    s.add_argument("--voices", required=True, help="comma-separated names from voices.toml")
    s.add_argument("--lines", default=str(REPO / "data" / "lines"))
    s.add_argument("--slots", default="greeting,farewell,bark")
    s.add_argument("-o", "--out", default=str(REPO / "assets" / "voice"))
    s.add_argument("--limit", type=int, default=0)
    s.add_argument("--redo", action="store_true", help="remake clips that already exist")
    s.add_argument("--takes", type=int, default=4, help="readings to try when the listener can't follow one")
    s.add_argument("--no-check", action="store_true")
    s.set_defaults(fn=cmd_batch)

    s = sub.add_parser("studio", help="open the panel of knobs and sliders for inventing accents")
    s.add_argument("--port", type=int, default=8765)
    s.add_argument("--no-browser", action="store_true", help="don't open the page, just serve it")
    s.set_defaults(fn=cmd_studio)

    s = sub.add_parser("accents", help="list tongues and saved accents, or show what one does to a line")
    s.add_argument("name", nargs="?", help="a saved accent, or a tongue at a thickness such as horaro@2")
    s.add_argument("--text", default="Keep your hand on your purse round here. Mind the road.")
    s.set_defaults(fn=cmd_accents)

    s = sub.add_parser("selftest", help="measure that each dial does what it says")
    s.set_defaults(fn=cmd_selftest)

    args = ap.parse_args()
    try:
        args.fn(args)
    except Problem as e:
        print(f"voice: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
