/* One trajectory, run in a worker thread. The main thread hands over a seed,
   a population, a government and the turns it will want snapshots at; this
   runs the whole thing and posts back the states at those turns plus the
   per-turn track. The state is plain JSON, so it crosses the thread boundary
   as-is. */
const { parentPort, workerData } = require('worker_threads');
const path = require('path');
const root = path.join(__dirname, '..');
const E = require(path.join(root, 'build.js')).loadEngine(root);

function tracked(s) {
  return {
    tenders: Object.values(s.people).filter(p => p.alive && p.trade === 'tender').length,
    creditTight: s.creditTight || 0,
    landmark: Object.values(s.buildings).some(b => E.stageOf(b) >= 4)
  };
}

const { seed, pop, government, turns } = workerData;
const cfg = { seed, population: pop, startYear: 812 };
if (government) cfg.government = government;
let s = E.newWorld(cfg);
const want = new Set(turns);
const last = Math.max(...turns);
const snaps = { 0: s };
const track = [tracked(s)];
for (let i = 1; i <= last; i++) {
  s = E.advance(s, { lever: 'none' });
  track.push(tracked(s));
  if (want.has(i)) snaps[i] = s;
}
parentPort.postMessage({ seed, pop, government, snaps, track });
