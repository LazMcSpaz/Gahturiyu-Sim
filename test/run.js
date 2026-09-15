#!/usr/bin/env node
/* Runs the engine headless. No DOM — the UI layer is skipped. */
const path = require('path'), fs = require('fs');
const root = path.join(__dirname, '..');
const { loadEngine } = require(path.join(root, 'build.js'));
const { newWorld, advance, LEVERS, renderTurn } = loadEngine(root);

let failures = 0;
const ok = (name, cond, note = '') => {
  console.log(`${cond ? '  ok  ' : ' FAIL '} ${name}${note ? ' — ' + note : ''}`);
  if (!cond) failures++;
};

const run = (n, seed = 4242, pop = 200, inputs = []) => {
  let s = newWorld({ seed, population: pop, startYear: 812 });
  for (let i = 0; i < n; i++) s = advance(s, { lever: inputs[i] || 'none' });
  return s;
};

console.log('\ndeterminism');
ok('same seed, same history', JSON.stringify(run(80).stats) === JSON.stringify(run(80).stats));
const inp = Array.from({ length: 60 }, (_, i) => i === 20 ? 'storm' : i === 40 ? 'strangers' : 'none');
ok('replay from inputs reproduces the run',
   JSON.stringify(run(60, 4242, 200, inp).stats) === JSON.stringify(run(60, 4242, 200, inp).stats));

console.log('\nlevers');
let mid = run(40, 99, 150);
for (const k of Object.keys(LEVERS)) {
  let threw = null;
  try { advance(mid, { lever: k }); } catch (e) { threw = e.message; }
  ok(`lever: ${k}`, !threw, threw || '');
}

console.log('\nhundred-year runs');
for (const seed of [1, 7, 20260910, 555555, 31337]) {
  const s = run(400, seed, 150);
  const st = s.stats[s.stats.length - 1];
  ok(`seed ${seed} survives`, st.pop > 20 && st.households > 3,
     `${st.pop} living, ${st.households} households, ${st.mature} homes, ${st.tenders} tenders`);
}

console.log('\nchronicle');
const s = run(120, 20260910, 150);
const lines = s.chronicle.reduce((a, e) => a + renderTurn(e, {}).length, 0);
const dupes = s.chronicle.filter(e => {
  const t = e.events.map(x => x.text); return new Set(t).size !== t.length;
}).length;
ok('every season produces prose', s.chronicle.every(e => renderTurn(e, {}).length >= 2));
ok('no repeated line within a season', dupes === 0, `${dupes} seasons with a duplicate`);
ok('chronicle is not bloated', lines / s.chronicle.length < 12,
   `${(lines / s.chronicle.length).toFixed(1)} lines per season`);

console.log(failures ? `\n${failures} FAILED\n` : '\nall passed\n');
process.exit(failures ? 1 : 0);
