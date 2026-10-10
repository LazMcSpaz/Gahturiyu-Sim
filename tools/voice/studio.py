"""The panel: a page of knobs and sliders for inventing accents by ear.

    python tools/voice/voice.py studio

It serves one page on this machine only (127.0.0.1) and speaks each change
with the same code that later makes the game's clips, so what is heard here
is what a batch will produce.
"""

from __future__ import annotations

import base64
import io
import json
import re
import threading
import time
import webbrowser
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

import accents
import voice as V

HERE = Path(__file__).resolve().parent
LOCK = threading.Lock()   # one line is spoken at a time
VOICES_HEADER = """# The game's cast of voices. Each [name] is one speaker an NPC can have:
# an accent (accents.toml, or a tongue at a thickness such as "roduro@2"),
# a raw voice or a mix of them, and that speaker's own body.
# The panel (`voice.py studio`) rewrites this file when a speaker is saved;
# notes added by hand are not kept.
#
#   base = "kokoro:bm_george*0.6+hm_omega*0.4"   `voice.py voices` lists them all
#   speaks = "roduro"   optional: only voices lines tagged voice=roduro (and untagged ones)
#   pitch, size, age, rough ...   any dial from dials.toml"""

# How far each slider goes: (lowest, highest, step).
RANGES = {
    "speed": (0.6, 1.5, 0.01), "pause": (0.3, 2.5, 0.05), "range": (0.3, 2.0, 0.05),
    "lilt": (-4.0, 4.0, 0.25), "pitch": (-6.0, 6.0, 0.25), "energy": (-1.0, 1.0, 0.05),
    "breath": (0.0, 1.0, 0.05), "size": (0.8, 1.25, 0.01), "age": (0.0, 1.0, 0.05),
    "rough": (0.0, 1.0, 0.05), "tremor": (0.0, 1.0, 0.05),
}
ACCENT_DIALS = ["speed", "pause", "range", "lilt", "pitch", "energy", "breath"]
BODY_DIALS = ["pitch", "size", "age", "rough"]


def meta() -> dict:
    raw = []
    for name in V.kokoro().get_voices():
        kind = V.KOKORO_ACCENTS.get(name[0], ("?", ""))[0]
        raw.append({"id": name, "sex": "woman" if name[1] == "f" else "man", "kind": kind})
    lines = [l["text"] for l in V.read_lines(V.REPO / "data" / "lines", {"greeting", "farewell", "bark"})]
    return {
        "knobs": accents.knob_meta(),
        "tongues": {k: {"label": t["label"], "voices": t.get("voices", []), "mix": t.get("mix", ""),
                        "reading": t.get("reading", "en-us"),
                        "steps": [x.get("what", "") for x in t.get("step", [])]} for k, t in accents.TONGUES.items()},
        "ranges": RANGES, "accent_dials": ACCENT_DIALS, "body_dials": BODY_DIALS,
        "defaults": V.DEFAULTS, "multiplied": sorted(V.MULTIPLIED),
        "emotions": list(V.DIALS["emotion"]), "voices": raw, "lines": lines,
        "accents": accents.load_accents(), "cast": V.load_profiles(),
        "ears": (V.MODELS / V.EARS_DIR).exists(),
    }


def mix_text(mix: list) -> str:
    parts = [f"{m['voice']}*{round(float(m['weight']), 2)}" for m in mix if m.get("voice") and float(m.get("weight", 0)) > 0]
    if not parts:
        V.die("pick at least one raw voice with some weight")
    return "kokoro:" + "+".join(parts)


def build(body: dict) -> V.Voice:
    """The voice the panel is describing right now."""
    v = V.voice_from({"base": mix_text(body.get("mix", []))})
    acc = body.get("accent") or {}
    if not body.get("plain"):
        v.accent = {"sounds": {k: s for k, s in (acc.get("sounds") or {}).items() if accents.setting(s)[0]},
                    "delivery": {k: float(x) for k, x in (acc.get("delivery") or {}).items() if k in V.DEFAULTS},
                    "reading": acc.get("reading", "en-us")}
        accents.check(v.accent["sounds"])
    else:
        v.reading = acc.get("reading", "en-us")
    for k, x in (body.get("dials") or {}).items():
        if k in V.DEFAULTS:
            v.dials[k] = float(x)
    return v


def say(body: dict) -> dict:
    text = (body.get("text") or "").strip()
    if not text:
        V.die("type a line to say")
    v = build(body)
    started = time.time()
    plain, bent = V.sounds_of(v, text)
    x, sr, _ = V.render(v, text, body.get("emotion") or None)
    import soundfile as sf
    buf = io.BytesIO()
    sf.write(buf, x, sr, format="WAV", subtype="PCM_16")
    out = {"wav": base64.b64encode(buf.getvalue()).decode("ascii"), "plain": plain, "bent": bent,
           "took": round(time.time() - started, 2), **V.measure(x, sr)}
    if body.get("listen") and (V.MODELS / V.EARS_DIR).exists():
        out.update(V.hear(x, sr, text))
    return out


def save_voice(name: str, body: dict) -> None:
    if not re.fullmatch(r"[a-z0-9_]{1,40}", name or ""):
        V.die("a speaker's name is lower-case letters, digits and _ (for example roduro_m1)")
    acc = body.get("accent_name") or ""
    try:
        accents.find(acc)
    except ValueError:
        V.die("save the accent first, then the speaker: a speaker points at an accent by name")
    cast = V.load_profiles()
    entry = {"accent": acc, "base": mix_text(body.get("mix", []))}
    if body.get("speaks"):
        entry["speaks"] = body["speaks"]
    for k, x in (body.get("dials") or {}).items():
        if k in V.DEFAULTS and abs(float(x) - V.DEFAULTS[k]) > 1e-6:
            entry[k] = round(float(x), 3)
    cast[name] = entry
    (HERE / "voices.toml").write_text(accents.dump(cast, VOICES_HEADER), encoding="utf-8")


def delete_voice(name: str) -> None:
    cast = V.load_profiles()
    if cast.pop(name, None) is not None:
        (HERE / "voices.toml").write_text(accents.dump(cast, VOICES_HEADER), encoding="utf-8")


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args) -> None:   # keep the terminal quiet
        pass

    def send(self, code: int, body: bytes, kind: str) -> None:
        self.send_response(code)
        self.send_header("Content-Type", kind)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def answer(self, fn) -> None:
        try:
            with LOCK:
                result = fn()
            self.send(200, json.dumps(result if result is not None else {"ok": True}, ensure_ascii=False).encode("utf-8"),
                      "application/json; charset=utf-8")
        except (V.Problem, ValueError) as e:
            self.send(400, json.dumps({"error": str(e)}, ensure_ascii=False).encode("utf-8"),
                      "application/json; charset=utf-8")
        except Exception as e:   # anything else: say so on the page, don't die
            self.send(500, json.dumps({"error": f"{type(e).__name__}: {e}"}, ensure_ascii=False).encode("utf-8"),
                      "application/json; charset=utf-8")

    def do_GET(self) -> None:
        url = urlparse(self.path)
        if url.path == "/":
            self.send(200, (HERE / "studio.html").read_bytes(), "text/html; charset=utf-8")
        elif url.path == "/api/meta":
            self.answer(meta)
        elif url.path == "/api/recipe":
            q = parse_qs(url.query)
            self.answer(lambda: accents.recipe(q.get("tongue", [""])[0], int(q.get("thickness", ["0"])[0])))
        else:
            self.send(404, b"not found", "text/plain")

    def do_POST(self) -> None:
        length = int(self.headers.get("Content-Length") or 0)
        try:
            body = json.loads(self.rfile.read(length) or b"{}")
        except json.JSONDecodeError:
            return self.send(400, b'{"error":"bad request"}', "application/json")
        routes = {
            "/api/say": lambda: say(body),
            "/api/save_accent": lambda: accents.save_accent(body.get("name", ""), body.get("accent") or {}),
            "/api/delete_accent": lambda: accents.delete_accent(body.get("name", "")),
            "/api/save_voice": lambda: save_voice(body.get("name", ""), body),
            "/api/delete_voice": lambda: delete_voice(body.get("name", "")),
        }
        fn = routes.get(urlparse(self.path).path)
        if fn is None:
            return self.send(404, b"not found", "text/plain")
        self.answer(fn)


def serve(port: int = 8765, open_page: bool = True) -> None:
    V.kokoro()   # load the voice engine before the first knob is turned
    server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    url = f"http://127.0.0.1:{port}/"
    print(f"The panel is at {url}  (Ctrl+C here to stop it)", flush=True)
    if open_page:
        threading.Timer(0.5, lambda: webbrowser.open(url)).start()
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nstopped")
