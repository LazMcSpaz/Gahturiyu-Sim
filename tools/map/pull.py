"""After a save from the map site lands: turn it into what the game reads.

    python tools/map/pull.py                 # after `git pull`: towns, landmarks, the site's data
    python tools/map/pull.py EXPORT.txt      # first split a page export (the offline days) into its files
    python tools/map/pull.py --no-site       # skip refreshing tools/map (no Rust build)

The map site saves Laz's files straight into the repo, one commit a save:
`assets/world/marks-export.json` (marks and roads), `assets/world/places/<key>.json`
(a place a file) and `assets/world/people/<key>.ron` (a roster a place). The game
reads the places and rosters as they are; the marks go through
`marks_to_towns.py` into `towns.ron` and `landmarks.ron`. Then `headless site`
refreshes what the site serves beside its page (the catalogue, every mark's
land, copies of the marks and places) and reads every roster, so a bad one is
said here.

An export file from the page (Export everything, for a day with no signal) holds
the rosters as sections headed `// ---- assets/world/people/<key>.ron (...)`;
given one, each section is written to its file first. (Its marks section is
the page's own RON, not the export JSON: marks come in through
marks-export.json.)

Exits 1 if any step fails.
"""

import os
import re
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
PEOPLE = os.path.join(ROOT, "assets", "world", "people")
HEADER = re.compile(r"^// ---- (assets/world/people/([a-z0-9_]+)\.ron)\b.*$", re.M)


def split_export(path):
    """Write each roster section of a page export to its file. Returns the paths."""
    text = open(path, encoding="utf-8").read().replace("\r\n", "\n")
    heads = list(HEADER.finditer(text))
    written = []
    for n, h in enumerate(heads):
        end = heads[n + 1].start() if n + 1 < len(heads) else len(text)
        # A section ends where the next "// ----" header starts (rosters, or anything after them).
        body = text[h.end():end]
        nxt = re.search(r"^// ---- ", body, re.M)
        if nxt:
            body = body[: nxt.start()]
        body = body.strip("\n") + "\n"
        out = os.path.join(ROOT, *h.group(1).split("/"))
        os.makedirs(os.path.dirname(out), exist_ok=True)
        with open(out, "w", encoding="utf-8", newline="\n") as f:
            f.write(body)
        written.append(os.path.relpath(out, ROOT))
    return written


def run(what, cmd):
    print("==", what, flush=True)
    r = subprocess.run(cmd, cwd=ROOT)
    if r.returncode != 0:
        print("FAILED:", what)
        sys.exit(1)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if args:
        for p in split_export(args[0]):
            print("wrote", p, flush=True)
    run("marks -> towns.ron, landmarks.ron", [sys.executable, os.path.join(ROOT, "tools", "map", "marks_to_towns.py")])
    if "--no-site" not in sys.argv:
        run("the site's data (headless site)", ["cargo", "run", "--release", "--quiet", "--bin", "headless", "--", "site"])
    print("done: commit towns.ron, landmarks.ron, any rosters written and tools/map/ (grounds, data, catalogue)")


if __name__ == "__main__":
    main()
