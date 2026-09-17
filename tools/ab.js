#!/usr/bin/env node
/* Before and after. The same sweep against another commit and against the
   working tree, side by side, with the deltas that matter.

     node tools/ab.js                 # HEAD against the working tree
     node tools/ab.js HEAD~3          # three commits back
     node tools/ab.js main --years 250 --seeds 7,991

   Reads with a delta beyond the noise are marked; anything measure.js would
   flag on one side and not the other is listed. */
const path = require('path');
const { sweep, parseArgs } = require('./sweep');
const { flags } = require('./measure');
const checkout = require('./checkout');

const COLS = ['pop', 'mature', 'stage3', 'workshops', 'disrepair', 'craftsLost', 'lowestTenders',
  'legitimacy', 'coinBatches', 'storeFood', 'linesPerSeason', 'levies', 'blocs', 'fellBack', 'rulings'];

(async () => {
  const o = parseArgs(process.argv.slice(2));
  const ref = (o._ && o._[0]) || 'HEAD';
  const before = checkout(ref), after = path.join(__dirname, '..');
  const t0 = Date.now();
  const [A, B] = await Promise.all([sweep(before, o), sweep(after, o)]);
  const years = (o.turns || 480) / 4;
  const cols = o.cols || COLS;

  console.log(`\n${ref} → working tree, ${A.length} seeds, ${years} years — ${((Date.now() - t0) / 1000).toFixed(0)}s`);
  console.log('seed'.padEnd(9) + cols.map(c => c.padStart(Math.max(13, c.length + 1))).join(''));
  for (let i = 0; i < A.length; i++) {
    const a = A[i].final, b = B[i].final;
    console.log(String(A[i].seed).padEnd(9) + cols.map(c => {
      const x = a[c], y = b[c];
      const cell = typeof x === 'number' && typeof y === 'number' && x !== y
        ? `${x}→${y}` : String(y === undefined ? '-' : y);
      return cell.padStart(Math.max(13, c.length + 1));
    }).join(''));
  }

  // means, and which way they moved
  console.log('\nmean'.padEnd(9) + cols.map(c => {
    const ma = A.reduce((s, r) => s + (+r.final[c] || 0), 0) / A.length;
    const mb = B.reduce((s, r) => s + (+r.final[c] || 0), 0) / B.length;
    const d = mb - ma;
    const mark = Math.abs(d) > Math.max(1, Math.abs(ma) * 0.15) ? (d > 0 ? ' ▲' : ' ▼') : '';
    return `${ma.toFixed(1)}→${mb.toFixed(1)}${mark}`.padStart(Math.max(13, c.length + 1));
  }).join(''));

  const changed = [];
  for (let i = 0; i < A.length; i++) {
    const fa = flags(A[i].final, years), fb = flags(B[i].final, years);
    const gone = fa.filter(f => !fb.includes(f)), fresh = fb.filter(f => !fa.includes(f));
    if (gone.length || fresh.length) changed.push({ seed: A[i].seed, gone, fresh });
  }
  console.log(changed.length ? '\nflags that changed:' : '\nno flag changed.');
  for (const c of changed) {
    if (c.fresh.length) console.log(`  ${String(c.seed).padEnd(9)} new:   ${c.fresh.join('; ')}`);
    if (c.gone.length) console.log(`  ${String(c.seed).padEnd(9)} fixed: ${c.gone.join('; ')}`);
  }
})();
