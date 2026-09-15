#!/usr/bin/env node
/* Concatenates the source into one self-contained page in dist/.
   Order matters — these files share one scope, they are not ES modules.
   That is deliberate: the result runs from file:// with no server. */
const fs = require('fs'), path = require('path');

const MANIFEST = [
  'src/engine/00-core.js',
  'src/engine/05-ties.js',
  'src/engine/10-world.js',
  'src/engine/15-turn.js',
  'src/engine/20-economy.js',
  'src/engine/30-stone.js',
  'src/engine/40-society.js',
  'src/engine/45-offices.js',
  'src/engine/50-politics.js',
  'src/engine/60-figures.js',
  'src/render/chronicle.js',
  'src/render/panels.js',
  'src/ui/app.js'
];

function sources(root = __dirname) {
  return MANIFEST
    .filter(f => fs.existsSync(path.join(root, f)))
    .map(f => `\n/* ===== ${f} ===== */\n` + fs.readFileSync(path.join(root, f), 'utf8'));
}

function build(root = __dirname) {
  const shell = fs.readFileSync(path.join(root, 'src/ui/index.html'), 'utf8');
  const out = shell.replace('/*{{BUNDLE}}*/', () => sources(root).join('\n'));
  fs.mkdirSync(path.join(root, 'dist'), { recursive: true });
  const dest = path.join(root, 'dist/gahturiyu.html');
  fs.writeFileSync(dest, out);
  return { dest, bytes: out.length };
}

// The test harness evaluates the same sources without the page around them.
// The UI layer is excluded — it touches window and has nothing to test headless.
function engineSource(root = __dirname) {
  return MANIFEST.filter(f => !f.startsWith('src/ui/') && fs.existsSync(path.join(root, f)))
    .map(f => fs.readFileSync(path.join(root, f), 'utf8')).join('\n');
}

// Loads the engine in its own scope and hands back the public surface.
function loadEngine(root = __dirname) {
  return new Function(engineSource(root) +
    '\nreturn {newWorld,advance,LEVERS,renderTurn,chronicleHTML,mapHTML,inTheNews,tileFactsHTML,'
    + 'seasonOf,yearOf,ageOf,mulberry32,reputeOf,tileId,tileXY,quarterOf,W,H,'
    + 'FACTIONS,standingWith,houseStanding,compositeOf,tieTo,holdsAgainst,bondWith,TIE_CAP,hasGoal};')();
}

module.exports = { build, engineSource, loadEngine, MANIFEST };
if (require.main === module) {
  const { dest, bytes } = build();
  console.log(`built ${path.relative(process.cwd(), dest)} — ${(bytes / 1024).toFixed(0)} KB`);
}
