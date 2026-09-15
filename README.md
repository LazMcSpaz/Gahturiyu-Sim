# Gahturiyu — settlement simulation

A seasonal simulation of a Roduro coastal settlement. It runs in a browser with
no server, and produces a chronicle rather than a dashboard.

## Running it

Open `dist/gahturiyu.html`. That is the whole application — one file, no build
needed to *use* it, works from `file://` or from GitHub Pages on a phone.

## Working on it

```
node build.js        # concatenate src/ into dist/gahturiyu.html
node test/run.js     # headless: determinism, levers, hundred-year runs, prose
node test/probe.js "console.log(sim(7, 200).hist.at(-1))"   # scratch
```

There is no bundler and no dependency. `build.js` concatenates the files listed
in `MANIFEST` in order and inlines them into `src/ui/index.html`. The files share
one scope — they are deliberately **not** ES modules, because that would require
a server and lose the open-the-file-on-your-phone property.

Adding a file means adding it to `MANIFEST`.

## The contract

```
advance(state, input) -> new state
```

Pure, given the seed. All state is plain JSON, every turn's randomness derives
from `(seed, turn)`, and nothing is held outside the state object. Three things
depend on this and will break quietly if it is violated:

- **rollback** — go back to any season
- **branching** — change one decision and replay forward
- **export** — a saved run is the seed plus your inputs, a few kilobytes, and it
  rebuilds the world exactly

If you add a system, it must read and write only `state`, and take its randomness
only from the `r` passed to it.

## Layout

```
src/engine/00-core.js       rng, cloning, the Gogìḍu name generator, terrain,
                            quarters and walking distance, calendar
src/engine/10-world.js      world creation, people, the event helper
src/engine/15-turn.js       advance(), the levers, weather — the turn order lives here
src/engine/20-economy.js    food, and what a household does when it has been short for years
src/engine/30-stone.js      who will work your stone, teaching the gift, growth, quiet-season notes
src/engine/40-society.js    birth and death, pairing between households, succession, splitting
src/engine/45-offices.js    forms of government, the standing offices, boats and the guard
src/engine/50-politics.js   memory, the shrine, claims, disputes, grudges
src/engine/60-figures.js    promotion to named figure, what figures do, omens, bookkeeping
src/render/chronicle.js     events -> connected prose
src/render/panels.js        cast and household panels
src/ui/index.html           page shell and stylesheet, with the bundle slot
src/ui/app.js               timeline, branches, sheet, export/import
```

Turn order is in `advance()` and it matters — food runs before hardship, offices
before the shrine, promotion before figures act.

## What is simulated

**Ground is the scarce thing.** A house is grown from living stone over fifteen
to twenty years, only certain hillside will take one, and they cannot be packed
together. The map holds about thirty. Everything else follows from that.

**The gift is taught, not born.** Aptitude is rare and cannot be taught into
someone; the craft takes about a decade and most who begin do not finish. A
tender chooses who to work for — weighing the walk, kinship, what a household is
known for and what they hold against it — so whoever can build houses decides
which households get to exist.

**The settlement remembers.** Deeds attach to a household and fade over about
sixty-five years. Reputation decides who is believed, who is ruled against, and
whether anyone will marry into you.

**Grudges look for an opening** rather than waiting for one. They withhold food,
back the other side in a quarrel, and find reasons to be busy.

**Offices hold chokepoints.** One of three forms of government is in play; every
form has a captain of the guard, an arbiter and a boat-holder. Under a sole ruler
the ruler appoints, and appoints their own. Most households fish from a hull they
do not own.

**The shrine decides whether rulings bind.** Kept, quarrels end in judgement.
Neglected, they harden into feuds — and a feud between neighbours physically
shuts the ground between them, so everyone else walks further.

## Deploying

GitHub Pages is served from Actions, not from a branch. `.github/workflows/pages.yml`
checks out `main`, runs `node build.js`, runs the tests, and publishes the result — so
what is live is always built from `src/`, never from a stale committed `dist/`.

The page is published twice: at the site root, and at `dist/gahturiyu.html` so older
links keep working.

```
https://lazmcspaz.github.io/Gahturiyu-Sim/
```

Pushing to `main` deploys. Run it by hand from the Actions tab (**Pages → Run workflow**)
if you need to redeploy without a commit.

`.github/workflows/ci.yml` runs the same build and tests on every branch and pull
request, and fails if `dist/gahturiyu.html` was not rebuilt after a change to `src/` —
the committed copy is what someone gets when they clone and open the file directly, so
it has to stay current. Rebuild with `node build.js` and commit it alongside the source.
