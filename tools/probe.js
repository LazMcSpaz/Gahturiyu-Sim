#!/usr/bin/env node
/* The dashboard. Runs a sweep of seeds in parallel and prints one line per
   settlement plus anything that looks wrong.

     node tools/probe.js                       # eleven seeds, 120 years
     node tools/probe.js --years 250 --seeds 7,3
     node tools/probe.js --gov sole --every 80  # a time series per seed
     node tools/probe.js --storms 6             # a storm every sixth season
     node tools/probe.js --json > before.json   # for ab.js, or for later

   Every column is a field of tools/measure.js; pass --cols to pick your own. */
const path = require('path');
const { sweep, parseArgs } = require('./sweep');
const { flags } = require('./measure');

const COLS = ['pop', 'households', 'mature', 'stage2', 'stage3', 'landmark', 'workshops', 'disrepair',
  'condition', 'craftsHeld', 'craftsLost', 'lowestTenders', 'legitimacy', 'coinBatches', 'storeFood',
  'linesPerSeason', 'topKind', 'levies', 'blocs', 'fellBack', 'rulings', 'msPerTurn'];

function fmt(v) { return v === null || v === undefined ? '-' : typeof v === 'boolean' ? (v ? 'yes' : 'no') : String(v); }

function table(rows, cols, label) {
  const widths = cols.map(c => Math.max(c.length, ...rows.map(r => fmt(r[c]).length)));
  const line = (r) => cols.map((c, i) => fmt(r[c]).padStart(widths[i])).join('  ');
  console.log(`\n${label}`);
  console.log('seed'.padEnd(9) + 'gov'.padEnd(9) + cols.map((c, i) => c.padStart(widths[i])).join('  '));
  for (const r of rows) console.log(String(r.seed).padEnd(9) + String(r.government).slice(0, 8).padEnd(9) + line(r.final));
}

(async () => {
  const o = parseArgs(process.argv.slice(2));
  const root = o.ref ? require('./checkout')(o.ref) : path.join(__dirname, '..');
  const t0 = Date.now();
  const res = await sweep(root, o);
  const years = (o.turns || 480) / 4;
  if (o.json) { console.log(JSON.stringify({ opts: o, years, results: res }, null, 1)); return; }

  const cols = o.cols || COLS;
  table(res, cols, `${res.length} settlements after ${years} years${o.ref ? ` on ${o.ref}` : ''} — ${((Date.now() - t0) / 1000).toFixed(0)}s`);

  if (o.every) {
    for (const r of res) {
      console.log(`\nseed ${r.seed} over time`);
      const cs = ['turn', 'pop', 'mature', 'stage2', 'stage3', 'workshops', 'disrepair', 'condition', 'craftsHeld', 'legitimacy', 'storeFood'];
      console.log(cs.map(c => c.padStart(11)).join(''));
      for (const m of r.series.concat([r.final])) console.log(cs.map(c => fmt(m[c]).padStart(11)).join(''));
    }
  }

  const worries = res.map(r => ({ seed: r.seed, f: flags(r.final, years) })).filter(x => x.f.length);
  console.log(worries.length ? '\nworth a look:' : '\nnothing flagged.');
  for (const w of worries) console.log(`  ${String(w.seed).padEnd(9)} ${w.f.join('; ')}`);
})();
