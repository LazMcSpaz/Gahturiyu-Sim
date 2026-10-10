# The voice tool

Turns a line of text into a spoken clip for an NPC. It runs ahead of time
on a developer's machine (or in a Claude session) and writes sound files;
nothing here runs inside the game, and the game doesn't play voices yet.

Free, offline once set up, and the voices may be used in a sold game
(licences at the bottom).

## Set up (once per machine or session)

```
pip install -r tools/voice/requirements.txt
python tools/voice/voice.py setup        # about 520 MB down, into tools/voice/models (not in git)
python tools/voice/voice.py selftest     # measures that every dial does what it says
```

## Use

```
python tools/voice/voice.py voices --accent scottish
python tools/voice/voice.py say "Mind the road." --voice vctk:p247 --emotion weary --set pitch=-2 --check -o out.ogg
python tools/voice/voice.py sheet tools/voice/sheets/audition.toml -o tools/voice/out/audition
python tools/voice/voice.py batch --voices dockhand_m1,fishwife_f1
```

- `say` makes one clip. `--voice` is a name from `voices.toml` or a raw
  speaker (`kokoro:bm_george`, `vctk:p247`).
- `sheet` makes a page of clips to compare by ear (clips, `index.html`,
  `index.json`). A sheet is a short list in a `.toml` file.
- `batch` voices every plain line in `data/lines` (greetings, farewells,
  barks; pieces with `{slots}` are skipped) for the named voices, into
  `assets/voice/<voice>/<line key>.ogg` plus `manifest.tsv`. A line's tags
  choose its emotion (`[line_emotion]` in `dials.toml`).
- `check` measures a clip and has a listener model write down what it hears.

## Two engines, one set of dials

| | What it is | Use it for |
|---|---|---|
| `kokoro` | 54 clean studio-quality voices: American, British, and voices trained on other languages that colour their English. Any of them can be mixed. | The best sound; invented accents by mixing. |
| `vctk` | 109 real people recorded with their own accents: Scottish, Irish, Northern Irish, Welsh, many English regions, American, Canadian, South African, Australian, New Zealand, Indian. A step rougher than kokoro. | Real regional accents; sheer number of different people. |

Dials (all in `dials.toml`, with their numbers):

- **Body:** `pitch`, `size` (bigger or smaller chest and throat), `age`,
  `breath`, `rough`, `tremor`.
- **Delivery:** `speed`, `pause`, `range` (flat to sing-song), `lilt` (the
  line ends rising or falling), `energy` (soft to hard).
- **Emotion:** `angry`, `stern`, `weary`, `warm`, `afraid`, `sly`, each
  with a strength (`--emotion angry:0.5`). These are bundles of the dials
  above. They change how a line is delivered; they are not an actor's
  performance.
- **Sound swaps:** `rules = ["trill"]` rolls every r, and so on.
- **Accent of the reading:** `accent = "en-gb-scotland"` reads the text
  with another English's sounds before the voice says it.

## Invented names

The engines guess at names like Horahìda. `lexicon.txt` says how each is
pronounced; add a line whenever one comes out wrong.

## How Claude checks a clip (it can't hear)

- `--check` runs a speech-to-text model over the clip and compares what it
  heard with the text. A mismatch is a flag to look at, not a verdict:
  strong accents and invented names trip it.
- Every clip reports its length, loudness and pitch.
- `selftest` measures each dial against what it claims (pitch in
  semitones, throat size from the voice's resonances, speed from length).
- Whether a voice *sounds right* is Laz's call, by ear.

## Rules this follows

- A clip is the same every time it's made from the same voice, text and
  take; turning a dial changes that one thing and keeps the reading.
  `--take 1` gives a different reading.
- All numbers are data (`dials.toml`); voices are data (`voices.toml`).
- A voice is presentation only. Nothing in the sim reads it.

## Licences

- Kokoro v1.0 model and voices: Apache 2.0.
- VCTK accent model (a Piper voice): trained on the CSTR VCTK corpus,
  CC BY 4.0. **Credit needed** in the game: "Speech voices derived from
  the CSTR VCTK Corpus, University of Edinburgh."
- Listener model (Whisper base.en): MIT. Praat (via parselmouth) and
  espeak-ng are GPL tools used while making clips; nothing of them ships
  in the game.
