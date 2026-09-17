/* One seed, in a worker. Loads the engine from whatever root it is given —
   the working tree, or a checkout of some other commit — runs it, and posts
   back readings rather than states, so a sweep of twenty seeds costs a few
   kilobytes of messages and not a few hundred megabytes. */
const { parentPort, workerData } = require('worker_threads');
const path = require('path');
const { measure } = require(path.join(__dirname, 'measure.js'));

const { root, seed, pop, government, turns, every, levers } = workerData;
const E = require(path.join(root, 'build.js')).loadEngine(root);

const cfg = { seed, population: pop, startYear: 812 };
if (government) cfg.government = government;
const s0 = E.newWorld(cfg);
let s = s0;
const track = [];
const series = [];
const t0 = Date.now();
for (let i = 1; i <= turns; i++) {
  s = E.advance(s, { lever: (levers && levers[i - 1]) || 'none' });
  track.push({
    tenders: Object.values(s.people).filter(p => p.alive && p.trade === 'tender').length,
    stone: Object.values(s.people).filter(p => p.alive && (p.trade === 'tender' || (p.learning && p.learning.trade === 'tender'))).length,
    creditTight: s.creditTight || 0,
    shown: E.renderTurn(s.chronicle[s.chronicle.length - 1], {}).filter(x => x.type === 'event').length
  });
  if (every && i % every === 0 && i < turns) series.push(measure(E, s, s0, track));
}
const final = measure(E, s, s0, track);
final.msPerTurn = +((Date.now() - t0) / turns).toFixed(1);
parentPort.postMessage({ seed, government: s.government, final, series });
