# The voice tool

Two jobs, in order:

1. **Invent accents and speakers** in a panel of knobs and sliders, by ear.
2. **Make the spoken clips** for NPC lines with the speakers that were kept.

It runs ahead of time on a developer's machine (or in a Claude session) and
writes sound files; nothing here runs inside the game, and the game doesn't
play voices yet. Free, offline once set up, and usable in a sold game
(licences at the bottom).

## The panel

On Windows, double-click `tools\voice\studio.bat`. The first run sets
itself up (Python 3.11 or newer must be installed; about 350 MB of voice
models are fetched once), then a page opens in the browser. It only serves
this computer. On Mac or Linux: `tools/voice/studio.sh`.

- **The accent** (left) is shared by a people or a region. Start from a
  tongue and a thickness, then turn any knob: which sounds are swapped, how
  words are shaped, the rhythm and tune.
- **The speaker** (right) is one person with that accent: a raw voice or a
  mix of up to three, and their own body (pitch, size, age, gravel).
- Every change is spoken straight away. **Plain** plays the same speaker
  with no accent; **Before** replays the clip from before the last change.
- **Save accent** writes `accents.toml`; **Save speaker** writes
  `voices.toml`. Commit those two files: they are the game's accents and cast.

### Where the accents come from

Each tongue has sounds it does not own. A speaker reaches for the nearest
sound they do own, exactly as the naming system bends a borrowed word
(`assets/lang/sounds.ron`, the `*_borrows` lists). `tongues.toml` holds, for
each tongue:

- **Four steps for plain speech.** Laz's rule: an accent on an English line
  must leave the words recognisable as the words on screen. The steps hold
  only the habits that pass that test, thickest last. They were chosen by
  adding one habit at a time and keeping it only while a speech-to-text
  listener still caught about four words in five across twelve game lines,
  which is the level Laz marked "about right" by ear.
- **The native mouth** (`NAME@native`, the last stop of the panel's
  thickness slider): everything the tongue's rules say. It stops being
  plain speech; keep it for words in that tongue.

| Tongue | Owns | Does not own | Shape and rhythm |
|---|---|---|---|
| Roduro | t, curled-back t and d, throaty k, the catch, d, g, sh, th, h, r, l, y | lip sounds, nose sounds | one consonant and one vowel per beat; stress next to last; slow, low |
| Qotiro | p, t, k, throaty k, d, g, m, n, a rasp, rolled r | l, y, h, s, sh, th, f, w | consonants stacked, hard endings; first beat hit; short vowels |
| Horaro | l, m, n, w, gentle r, h, long vowels | every hard stop and hiss | open, vowel-heavy; soft drawn-out stress |
| Ṭaḍoro | f, h, s, sh, th, w, y, gliding vowels | hard stops, nose sounds, l, r | weak vowels blur; last beat rises; quick, breathy |

`sounds.toml` is the list of knobs (21 of them, each with its settings).
A knob set to "light" spares the first sound of each word, which is what
keeps a word recognisable.

Raw voices are suggested per tongue from languages that already have that
tongue's sounds (a Spanish-trained voice already rolls its r; a
Hindi-trained one already curls its t and d back). That choice was made by
reasoning, not by ear.

## Set up (once per machine or session)

```
pip install -r tools/voice/requirements.txt
python tools/voice/voice.py setup        # about 520 MB down, into tools/voice/models (not in git)
python tools/voice/voice.py selftest     # measures that every dial and knob does what it says
```

## Use

```
python tools/voice/voice.py studio                      # the panel
python tools/voice/voice.py accents                     # tongues and saved accents
python tools/voice/voice.py accents qotiro@2            # what an accent does to a line, as sounds
python tools/voice/voice.py say "Mind the road." --voice kokoro:bm_george --accent roduro@2 --check -o out.ogg
python tools/voice/voice.py say "Mind the road." --voice vctk:p247 --emotion weary --set pitch=-2 -o out.ogg
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
- **Delivery:** `speed`, `pause` (gaps at commas), `stop` (how long a full
  stop lingers), `life` (how much pace and tune change from sentence to
  sentence), `range` (flat to sing-song), `lilt` (sentences end rising or
  falling), `energy` (soft to hard).
- **Phrasing:** a line is spoken one sentence at a time. Each sentence gets
  its own pace, height and ending by what kind it is (statement, yes-or-no
  question, open question, shout, trailing off) and where it falls in the
  line, plus a small fixed roll so no two come out alike, and a real
  silence after it. The numbers are the `[phrasing]` tables in
  `dials.toml`. The engines on their own read a question and a statement
  with the same falling tune and leave almost no gap at a full stop.
- **Emotion:** `angry`, `stern`, `weary`, `warm`, `afraid`, `sly`, each
  with a strength (`--emotion angry:0.5`). These are bundles of the dials
  above. They change how a line is delivered; they are not an actor's
  performance.
- **Accent:** `--accent NAME` (a saved accent) or a tongue at a thickness
  (`roduro@1` to `roduro@4`, or `roduro@native`). Accents bend sounds the clean engine can say;
  they are not meant for the real-accent engine.

## Invented names

The engines guess at names like Horahìda. `lexicon.txt` says how each is
pronounced; add a line whenever one comes out wrong.

## How Claude checks a clip (it can't hear)

- `--check` runs a speech-to-text model over the clip and compares what it
  heard with the text. A mismatch is a flag to look at, not a verdict:
  strong accents and invented names trip it.
- Every clip reports its length, loudness and pitch.
- `selftest` measures each dial against what it claims (pitch in
  semitones, throat size from the voice's resonances, speed from length),
  checks that every setting of every accent knob changes the sounds of a
  line, and that each tongue stays followable through step 4 and stops
  being followable at its native mouth (by the listener's count).
- Whether a voice *sounds right* is Laz's call, by ear.

## Rules this follows

- A clip is the same every time it's made from the same voice, text and
  take; turning a dial changes that one thing and keeps the reading.
  `--take 1` gives a different reading.
- All numbers are data (`dials.toml`); knobs are data (`sounds.toml`); each
  tongue's habits are data (`tongues.toml`); accents and speakers are data
  (`accents.toml`, `voices.toml`).
- A voice is presentation only. Nothing in the sim reads it.

## Licences

- Kokoro v1.0 model and voices: Apache 2.0.
- VCTK accent model (a Piper voice): trained on the CSTR VCTK corpus,
  CC BY 4.0. **Credit needed** in the game: "Speech voices derived from
  the CSTR VCTK Corpus, University of Edinburgh."
- Listener model (Whisper base.en): MIT. Praat (via parselmouth) and
  espeak-ng are GPL tools used while making clips; nothing of them ships
  in the game.
