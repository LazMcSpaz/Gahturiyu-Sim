#!/usr/bin/env node
/* Runs the engine headless. No DOM — the UI layer is skipped. */
const path = require('path'), fs = require('fs');
const root = path.join(__dirname, '..');
const { loadEngine } = require(path.join(root, 'build.js'));
const { newWorld, advance, LEVERS, renderTurn, mapHTML, tileFactsHTML, inTheNews, W, H } = loadEngine(root);

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
ok('every season produces a line', s.chronicle.every(e => renderTurn(e, {}).length >= 1));
ok('no repeated line within a season', dupes === 0, `${dupes} seasons with a duplicate`);
ok('chronicle is not bloated', lines / s.chronicle.length < 12,
   `${(lines / s.chronicle.length).toFixed(1)} lines per season`);

// The chronicle is facts only: every line is an event, or the explicit
// "nothing happened" line. No openers, no bridges, no connective prose.
const kinds = new Set(s.chronicle.flatMap(e => renderTurn(e, {}).map(p => p.type)));
ok('no connective prose', [...kinds].every(k => k === 'event' || k === 'quiet'),
   `part types: ${[...kinds].sort().join(', ')}`);

// A season with no events says so rather than rendering empty.
const quiet = s.chronicle.filter(e => renderTurn(e, {}).every(p => p.type === 'quiet'));
ok('a quiet season still says something', quiet.every(e => renderTurn(e, {})[0].text.length > 0),
   `${quiet.length} of ${s.chronicle.length} seasons had nothing to report`);

console.log('\nmap');
{
  const m = run(200, 20260910, 150);
  const html = mapHTML(m, null);
  const ids = [...html.matchAll(/data-t="(\d+)"/g)].map(x => Number(x[1]));
  ok('every tile is addressable', ids.length === W * H, `${ids.length} of ${W * H} tiles carry an id`);
  ok('tile ids are the tile ids', ids.every((v, i) => v === i));

  // the inspector must answer for any tile on the map, not just the built ones
  let blank = 0, threw = null;
  for (let i = 0; i < W * H; i++) {
    try { if (!tileFactsHTML(m, i).trim()) blank++; } catch (e) { threw = `tile ${i}: ${e.message}`; }
  }
  ok('the inspector answers for every tile', !threw && blank === 0, threw || `${blank} tiles said nothing`);

  // a house tile must name its household — that is the whole point
  const b = Object.values(m.buildings).find(x => x.state === 'mature' && m.households[x.householdId]);
  const facts = b ? tileFactsHTML(m, b.tileId) : '';
  ok('a house names its household', !!b && facts.includes(m.households[b.householdId].name),
     b ? `tile ${b.tileId}` : 'no mature house in this run');

  // the shrine tile reports the shrine, not whatever ground it sits on
  ok('the shrine tile reports the shrine',
     !m.shrine || tileFactsHTML(m, m.shrine.tileId).includes(m.shrine.god));

  // selection is drawn, and only on the selected tile
  const sel = b ? b.tileId : 0;
  const drawn = mapHTML(m, sel);
  ok('the selected tile is marked once', (drawn.match(/m-sel/g) || []).length === 1);

  // households in the latest season are the ones highlighted
  const news = inTheNews(m);
  const last = m.chronicle[m.chronicle.length - 1];
  const named = new Set(last.events.filter(e => e.household).map(e => e.household));
  ok('households named last season are in the news', [...named].every(h => news.has(h)),
     `${news.size} household${news.size === 1 ? '' : 's'} highlighted`);
}

console.log(failures ? `\n${failures} FAILED\n` : '\nall passed\n');
process.exit(failures ? 1 : 0);
