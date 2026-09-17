/* Runs many seeds in parallel against one engine root and returns their
   readings. Shared by probe.js and ab.js. */
const os = require('os');
const path = require('path');
const { Worker } = require('worker_threads');

const DEFAULT_SEEDS = [1, 2, 3, 7, 99, 313, 4242, 991, 31337, 555555, 20260910];

function sweep(root, opts = {}) {
  const seeds = opts.seeds || DEFAULT_SEEDS;
  const threads = Math.min(opts.threads || os.cpus().length, seeds.length);
  const queue = seeds.slice();
  const out = new Map();
  const lane = async () => {
    while (queue.length) {
      const seed = queue.shift();
      const res = await new Promise((resolve, reject) => {
        const w = new Worker(path.join(__dirname, 'sweep-worker.js'), {
          workerData: { root, seed, pop: opts.pop || 150, government: opts.government || null,
                        turns: opts.turns || 480, every: opts.every || 0, levers: opts.levers || null }
        });
        w.once('message', resolve); w.once('error', reject);
      });
      out.set(seed, res);
    }
  };
  return Promise.all(Array.from({ length: threads }, lane)).then(() => seeds.map(s => out.get(s)));
}

function parseArgs(argv) {
  const o = {};
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--seeds') o.seeds = argv[++i].split(',').map(Number);
    else if (a === '--turns') o.turns = Number(argv[++i]);
    else if (a === '--years') o.turns = Number(argv[++i]) * 4;
    else if (a === '--pop') o.pop = Number(argv[++i]);
    else if (a === '--gov') o.government = argv[++i];
    else if (a === '--every') o.every = Number(argv[++i]);
    else if (a === '--threads') o.threads = Number(argv[++i]);
    else if (a === '--json') o.json = true;
    else if (a === '--storms') { const n = Number(argv[++i]); o.levers = Array.from({ length: o.turns || 480 }, (_, i) => i % n === 0 ? 'storm' : 'none'); }
    else if (a === '--ref') o.ref = argv[++i];
    else if (a === '--cols') o.cols = argv[++i].split(',');
    else o._ = (o._ || []).concat(a);
  }
  return o;
}

module.exports = { sweep, parseArgs, DEFAULT_SEEDS };
