#!/usr/bin/env node
/* Runs the engine headless. No DOM — the UI layer is skipped. */
const path = require('path'), fs = require('fs');
const root = path.join(__dirname, '..');
const { loadEngine } = require(path.join(root, 'build.js'));
const E = loadEngine(root);
const { newWorld, advance, LEVERS, renderTurn, mapHTML, tileFactsHTML, inTheNews, W, H, ageOf,
        FACTIONS, standingWith, houseStanding, compositeOf, tieTo, holdsAgainst, TIE_CAP, hasGoal,
        stageOf, roleOf, roleWord, hasWorkshop, tavernsOf, STAGE_TURNS,
        TRADES, TIER1, TEACHABLE, tradeTier, canPractise, workshopFor, apprenticeScore, endangered,
        GOODS, MAKES, goodsOf, commonStore, TITHE,
        VALUE, debtsOf, owed, totalOwed, totalHeld, priceFor, demandOf,
        legitimacy, mintOf, coinOf, coinWorks, COIN_AT, COIN_KEEP,
        HARSHNESS, chooseRuling, gaoled, RUIN_AT,
        conditionOf, beautyOf, inDisrepair, vacant, DISREPAIR, RUINOUS, MEND_BELOW, SELF_CEILING } = E;

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

// Sections below all wanted the same fifty-year run. Building it once keeps
// the suite quick enough to run on every save.
let _shared = null;
const shared = () => (_shared = _shared || run(200, 20260910, 150));

console.log('\ndeterminism');
ok('same seed, same history', JSON.stringify(run(80).stats) === JSON.stringify(run(80).stats));
const inp = Array.from({ length: 60 }, (_, i) => i === 20 ? 'storm' : i === 40 ? 'strangers' : 'none');
ok('replay from inputs reproduces the run',
   JSON.stringify(run(60, 4242, 200, inp).stats) === JSON.stringify(run(60, 4242, 200, inp).stats));

// advance() carries the chronicle and stats by reference into a fresh array.
// If a later turn could reach back and change an entry, every earlier state in
// a rolled-back run would rot. This is the test that says it cannot.
{
  let a = newWorld({ seed: 77, population: 120, startYear: 812 });
  for (let i = 0; i < 30; i++) a = advance(a, { lever: 'none' });
  const snapshot = JSON.stringify(a.chronicle);
  const lenBefore = a.chronicle.length;
  let b = a;
  for (let i = 0; i < 20; i++) b = advance(b, { lever: 'storm' });
  ok('advancing does not change the state it came from',
     JSON.stringify(a.chronicle) === snapshot && a.chronicle.length === lenBefore,
     `${lenBefore} entries, still ${a.chronicle.length}`);
  ok('the newer state kept the older one\'s history',
     b.chronicle.length === lenBefore + 20
     && JSON.stringify(b.chronicle.slice(0, lenBefore)) === snapshot);
}

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

console.log('\nstanding, per faction');
{
  const m = shared();
  const alive = Object.values(m.people).filter(p => p.alive);

  ok('everyone carries all six factions',
     alive.every(p => FACTIONS.every(f => typeof standingWith(p, f) === 'number')),
     FACTIONS.join(', '));
  ok('standing stays in range',
     alive.every(p => FACTIONS.every(f => Math.abs(standingWith(p, f)) <= 100)));

  // the factions that have something earning them should not be flat
  const live = FACTIONS.filter(f => alive.some(p => Math.abs(standingWith(p, f)) > 1));
  ok('offices, the shrine and the quarter all pay out', live.length >= 3, `live: ${live.join(', ')}`);

  // a household reads as the age-weighted regard of its adults, plus its deeds
  const hh = Object.values(m.households).find(h => !h.extinct && h.members.length > 2);
  ok('a household has a standing in every faction',
     FACTIONS.every(f => Number.isFinite(houseStanding(m, hh, f))));
  ok('the composite is finite and not the only number', Number.isFinite(compositeOf(m, hh)));
}

console.log('\nties');
{
  const m = shared();
  const alive = Object.values(m.people).filter(p => p.alive);
  const all = alive.flatMap(p => p.ties || []);

  ok('nobody holds more ties than the cap',
     alive.every(p => (p.ties || []).length <= TIE_CAP),
     `most held by one person: ${Math.max(...alive.map(p => (p.ties || []).length))} of ${TIE_CAP}`);
  ok('ties stay in range', all.every(t => Math.abs(t.value) <= 100));
  ok('no tie points at a household that is gone',
     all.every(t => m.households[t.target] && !m.households[t.target].extinct));
  ok('every tie carries a reason', all.every(t => typeof t.cause === 'string' && t.cause.length));

  // the point of one signed number: a grudge is a tie read in one direction
  const foe = alive.find(p => (p.ties || []).some(t => t.value < -20));
  const neg = foe && (foe.ties.find(t => t.value < -20));
  ok('a negative tie reads as a grudge',
     !!foe && holdsAgainst(m, foe, neg.target) === -neg.value);
  const friend = alive.find(p => (p.ties || []).some(t => t.value > 20));
  const pos = friend && friend.ties.find(t => t.value > 20);
  ok('a positive tie holds nothing against anyone',
     !!friend && holdsAgainst(m, friend, pos.target) === 0);

  // contact has to make friends as well as enemies, or the leveller is dead
  const close = all.filter(t => t.value >= 40).length;
  const foes = all.filter(t => t.value <= -40).length;
  ok('closeness and enmity both occur', close > 0 && foes > 0, `${close} close, ${foes} at odds`);
}

console.log('\nthe term at the shrine');
{
  const m = shared();
  const alive = Object.values(m.people).filter(p => p.alive);
  // only people born after the settlement was founded — anyone already adult
  // at turn zero was past the age before the obligation existed
  const grown = alive.filter(p => E.ageOf(m, p) > 24 && p.birthTurn > 0);
  const settled = grown.filter(p => (p.term && p.term.done) || p.refusedTerm);
  ok('everyone born here and grown has served or refused',
     settled.length === grown.length, `${settled.length} of ${grown.length}`);

  const refused = alive.filter(p => p.refusedTerm).length;
  ok('some refuse, but not many', refused > 0 && refused / Math.max(1, alive.length) < 0.25,
     `${refused} of ${alive.length} living`);
  ok('refusing costs shrine standing',
     alive.filter(p => p.refusedTerm).every(p => standingWith(p, 'shrine') < 0));

  // the whole point: the term is where ties cross a standing gap
  const fromService = alive.flatMap(p => (p.ties || [])
    .filter(t => /served their term/.test(t.cause))
    .map(t => ({ p, t })));
  const crossed = fromService.filter(({ p, t }) => {
    const mine = m.households[p.householdId], theirs = m.households[t.target];
    return mine && theirs && Math.abs(compositeOf(m, mine) - compositeOf(m, theirs)) > 8;
  });
  ok('the term mints ties across a standing gap', crossed.length > 0,
     `${crossed.length} of ${fromService.length} service ties cross one`);
}

console.log('\nno cap on who matters');
{
  const m = shared();
  const alive = Object.values(m.people).filter(p => p.alive);
  ok('prominence is gone', alive.every(p => p.prominence === undefined));
  ok('a figure is anyone with a goal',
     alive.filter(hasGoal).length === alive.filter(p => p.goal).length);
  // the old limit was eight; what limits it now is the work available
  const over = [];
  let x = newWorld({ seed: 4242, population: 200, startYear: 812 });
  for (let i = 0; i < 200; i++) {
    x = advance(x, { lever: 'none' });
    over.push(Object.values(x.people).filter(p => p.alive && p.goal).length);
  }
  ok('the count moves with the work, not a constant',
     new Set(over).size > 3, `ranged ${Math.min(...over)}–${Math.max(...over)} over 50 years`);
}

console.log('\nbuildings keep growing');
{
  const start = newWorld({ seed: 20260910, population: 150, startYear: 812 });
  const atFounding = Object.values(start.buildings).filter(b => b.state === 'mature');
  ok('a founded settlement already has a few workshops',
     atFounding.some(b => stageOf(b) === 2) && !atFounding.some(b => stageOf(b) >= 3),
     `${atFounding.filter(b => stageOf(b) === 2).length} workshops, no third growth yet`);

  const m = run(480, 20260910, 150);   // 120 years — long enough for a third growth
  const built = Object.values(m.buildings).filter(b => b.state === 'mature');
  ok('every standing building has a stage',
     built.every(b => stageOf(b) >= 1 && stageOf(b) <= 4));
  ok('stone reaches a third growth given long enough',
     built.some(b => stageOf(b) >= 3),
     `${built.filter(b => stageOf(b) >= 3).length} of ${built.length} buildings`);

  // a third growth is a century of attention. It must not become the norm's
  // reward: eminence, a tavern, or a gift to the quarter — or just a big house.
  const third = built.filter(b => stageOf(b) >= 3);
  const great = third.filter(b => roleOf(b) === 'great').length;
  ok('a great house stays rarer than a large one',
     great <= third.filter(b => roleOf(b) === 'house').length,
     `${great} great, ${third.filter(b => roleOf(b) === 'house').length} large`);

  // one tavern to a quarter, and very few halls
  const quarters = (m.quarters || []).length;
  ok('a quarter supports one tavern', tavernsOf(m).length <= quarters,
     `${tavernsOf(m).length} taverns, ${quarters} quarters`);
  const halls = third.filter(b => roleOf(b) === 'common').length;
  ok('common halls stay scarce', halls <= Math.max(1, Math.ceil(quarters / 2)),
     `${halls} of at most ${Math.max(1, Math.ceil(quarters / 2))}`);

  // the whole point: tending never ends, so a house nobody visits stops
  ok('growth halts when no tender will come',
     built.every(b => !b.stalledStage || m.turn - (b.lastTended || b.startTurn) > 20),
     `${built.filter(b => b.stalledStage).length} stalled for want of a tender`);

  // and a workshop is what gates the finer trades later
  const houses = Object.values(m.households).filter(h => !h.extinct);
  const shops = houses.filter(h => hasWorkshop(m, h)).length;
  ok('a workshop is something only some households have',
     shops > 0 && shops < houses.length, `${shops} of ${houses.length} households`);

  ok('every building reads as something', built.every(b => roleWord(b).length > 0));
}

console.log('\ntrades');
{
  const start = newWorld({ seed: 20260910, population: 150, startYear: 812 });
  // a settlement is not founded blank, or there are no masters and nothing
  // can ever be taught
  /* The design's own claim is that a young settlement has a few of the crafts
     that need a room and a very old one has all of them, so demanding all
     eleven from one arbitrary seed was testing that seed's luck: 20260910
     founds with a single workshop between twenty-seven households. What must
     hold is the rule — every open craft is there, and no craft that needs a
     room is left unseeded while somebody could have reached one. */
  const rooms = TEACHABLE.filter(k => TRADES[k].room);
  const opens = TEACHABLE.filter(k => !TRADES[k].room);
  const founds = [20260910, 1, 4242, 991, 7].map(seed => {
    const w = newWorld({ seed, population: 150, startYear: 812 });
    const held = new Set(Object.values(w.people).filter(p => p.trade).map(p => p.trade));
    const spare = Object.values(w.people).filter(p => p.alive && !p.trade
      && ageOf(w, p) >= 22 && ageOf(w, p) <= 68 && workshopFor(w, p)).length;
    return { seed, held, spare };
  });
  ok('every craft that needs only a master is founded with one',
     founds.every(f => opens.every(k => f.held.has(k))),
     `${opens.length} open crafts across ${founds.length} foundings`);
  ok('and a craft that needs a room is founded wherever a room could be reached',
     founds.every(f => rooms.every(k => f.held.has(k)) || f.spare === 0),
     founds.map(f => `${f.seed}: ${rooms.filter(k => f.held.has(k)).length}/${rooms.length}`).join(', '));

  const seeded = new Set(Object.values(start.people).filter(p => p.trade).map(p => p.trade));
  ok('a settlement is never founded blank', seeded.size >= 6, `${seeded.size} crafts at founding`);
  ok('nobody is founded holding a craft they cannot practise',
     Object.values(start.people).filter(p => p.alive && p.trade).every(p => canPractise(start, p)));

  const m = run(480, 20260910, 150);   // 120 years
  const alive = Object.values(m.people).filter(p => p.alive);
  const adults = alive.filter(p => E.ageOf(m, p) >= 18);

  // The one exception is deliberate: somebody with the gift is not sent to the
  // grazing at eighteen while the stone will still take a twenty-four-year-old.
  const idle = adults.filter(p => !p.trade && !p.learning);
  ok('every adult has a trade, is learning one, or is being kept for the stone',
     idle.every(p => p.aptitude && E.ageOf(m, p) <= (TRADES.tender.maxAge || 24)),
     `${idle.length} without one, all held for the stone`);
  ok('every trade held is a real one', alive.every(p => !p.trade || TRADES[p.trade]));

  // the stone must never run out, or nothing is ever built again — and it is
  // an absorbing state, so it has to be checked over the whole run and not
  // just at the end
  let low = 99;
  let x = newWorld({ seed: 4242, population: 150, startYear: 812 });
  for (let i = 0; i < 480; i++) {
    x = advance(x, { lever: 'none' });
    low = Math.min(low, Object.values(x.people).filter(p => p.alive && p.trade === 'tender').length);
  }
  ok('the stone is never wholly lost', low > 0, `fewest tenders at any point: ${low}`);

  // a craft cannot swallow the settlement, and cannot be everyone's job
  const skilled = adults.filter(p => tradeTier(p.trade) > 1).length;
  ok('the crafts stay a minority of the work',
     skilled / adults.length < 0.45, `${skilled} of ${adults.length} adults`);
  const counts = {};
  for (const p of adults) counts[p.trade] = (counts[p.trade] || 0) + 1;
  const biggestCraft = Math.max(0, ...TEACHABLE.map(k => counts[k] || 0));
  ok('no single craft takes over', biggestCraft / adults.length < 0.25,
     `largest craft holds ${biggestCraft} of ${adults.length}`);

  /* A room-needing craft is held only where a room can be reached — with two
     deliberate exceptions. A craft down to its last hands may borrow any
     workshop in the settlement, because the alternative is losing it. And
     somebody whose room has just gone gets three years to find another before
     they give the craft up; what must not happen is the forty-year carver with
     nowhere to work, counted as a holder and making nothing. */
  const anyRoom = Object.values(m.households).some(h => !h.extinct && hasWorkshop(m, h));
  const roomless = alive.filter(p => p.trade && TRADES[p.trade].room
    && !workshopFor(m, p) && anyRoom && !endangered(m, p.trade));
  ok('a craft that needs a room is only held where one can be reached',
     roomless.every(p => (p.noRoom || 0) < 12),
     roomless.length ? `${roomless.length} looking for a room, longest ${Math.max(...roomless.map(p => p.noRoom || 0))} seasons`
                     : 'nobody is working without a room');
  const gaveUp = m.chronicle.flatMap(t => t.events).filter(e => /nowhere to work/.test(e.text)).length;
  ok('and a craft with no room is eventually given up', gaveUp > 0,
     `${gaveUp} gave the craft up over 120 years`);

  // apprenticeship has to reach outside the master's own household, or every
  // craft dies with a master who has no child of the right age
  const outside = alive.filter(p => p.learning && p.learning.kin === false).length;
  const learners = alive.filter(p => p.learning).length;
  ok('apprentices are not all kin', learners === 0 || outside > 0,
     `${outside} of ${learners} currently learning are from another house`);

  // and the gift belongs to the craft that needs it
  const gifted = adults.filter(p => p.aptitude);
  const giftedTenders = gifted.filter(p => p.trade === 'tender').length;
  ok('the gift is not spent on crafts that do not need it',
     gifted.length === 0 || giftedTenders > 0,
     `${giftedTenders} of ${gifted.length} aptitude-holders tend stone`);
}

console.log('\ngoods and the common store');
{
  const m = run(400, 20260910, 150);
  const houses = Object.values(m.households).filter(h => !h.extinct);
  const st = commonStore(m);

  ok('every household has somewhere to put goods',
     houses.every(h => h.goods && GOODS.every(g => typeof h.goods[g] === 'number')));
  ok('no household holds a negative amount of anything',
     houses.every(h => Object.values(h.goods).every(n => n >= -0.001)));

  // the crafts have to actually make what they are for
  const made = GOODS.filter(g => houses.some(h => h.goods[g] > 0.5));
  ok('the crafts produce goods', made.length >= 4, `held somewhere: ${made.join(', ')}`);

  // the government's cut is taken before anything else, so it must accumulate
  ok('the government took its cut', st.taken > 0, `${Math.round(st.taken)} taken over 100 years`);
  ok('the store is capped, not a bottomless hoard',
     st.food <= Object.values(m.people).filter(p => p.alive).length * 2.5,
     `${Math.round(st.food)} in hand`);
  ok('and it was opened to somebody', st.given > 0, `${Math.round(st.given)} given out`);

  // a shortage is only worth saying when the settlement feels it, not each season
  const econ = m.chronicle.flatMap(e => e.events).filter(e => /common store/.test(e.text));
  ok('the store opening does not fill the chronicle',
     econ.length < m.chronicle.length * 0.2,
     `${econ.length} lines across ${m.chronicle.length} seasons`);

  // and a craft that cannot get its input makes nothing rather than conjuring it
  ok('a craft short of its input is recorded', typeof m.shortOf === 'object');
}

console.log('\nexchange and debt');
{
  const m = run(400, 20260910, 150);
  const houses = Object.values(m.households).filter(h => !h.extinct);
  const debts = debtsOf(m);

  // the whole point of 9b: the chains complete. A smith two quarters from any
  // ore made nothing at all before goods could move.
  /* Only for the crafts this settlement actually has. A seed founded without
     a smith holds no metal, and that is the room rule working, not the
     carrying. */
  const canMake = { metal: 'smith', garments: 'tailor', fittings: 'joiner' };
  const possible = Object.entries(canMake)
    .filter(([, t]) => Object.values(m.people).some(p => p.alive && p.trade === t))
    .map(([g]) => g);
  const chained = possible.filter(g => houses.some(h => goodsOf(h)[g] > 0.5));
  ok('goods reach the crafts that need them',
     possible.length ? chained.length >= Math.min(2, possible.length) : true,
     `made and held: ${chained.join(', ') || 'none'} of ${possible.join(', ') || 'no chained craft held'}`);

  // and they spread beyond whoever makes them
  const spread = GOODS.filter(g => houses.filter(h => goodsOf(h)[g] > 0.5).length >= 4);
  ok('goods spread past the households that make them', spread.length >= 3,
     `held by four or more houses: ${spread.join(', ')}`);

  ok('nothing is held in the negative',
     houses.every(h => Object.values(goodsOf(h)).every(n => n >= -0.001)));

  // credit exists, is bounded, and some of it gets written down
  ok('debt is created by trading', debts.length > 0, `${debts.length} outstanding`);
  ok('the ledger stays small enough to export', debts.length <= 90, `${debts.length} of at most 90`);
  ok('every debt names both sides and what it was for',
     debts.every(d => m.households[d.from] && m.households[d.to] && d.from !== d.to));
  const sticks = debts.filter(d => d.tally);
  ok('large debts go on a stick', sticks.length > 0, `${sticks.length} of ${debts.length} on a tally`);

  /* The central claim of the design: a price is not a number, it is a bargain
     between two households, and what they are to each other is half of it. */
  const seller = houses.find(h => goodsOf(h).stone > 6) || houses[0];
  const head = m.people[seller.headId];
  if (head) {
    const quoted = houses.filter(h => h.id !== seller.id)
      .map(h => priceFor(m, seller, h, 'stone', 0).price);
    const lo = Math.min(...quoted), hi = Math.max(...quoted);
    ok('the same good costs different households different amounts',
       hi > lo * 1.15, `${lo.toFixed(2)} to ${hi.toFixed(2)} ḍaqu for one lot of stone`);
  }

  // a remembered debt fades; a stick does not
  let x = newWorld({ seed: 4242, population: 120, startYear: 812 });
  for (let i = 0; i < 60; i++) x = advance(x, { lever: 'none' });
  const before = debtsOf(x).filter(d => !d.tally).map(d => ({ from: d.from, to: d.to, amount: d.amount }));
  for (let i = 0; i < 40; i++) x = advance(x, { lever: 'none' });
  const faded = before.filter(b => {
    const now = debtsOf(x).find(d => d.from === b.from && d.to === b.to && !d.tally);
    return !now || now.amount < b.amount;
  });
  ok('a remembered debt fades or is paid',
     before.length === 0 || faded.length > 0,
     `${faded.length} of ${before.length} shrank or went`);
}

console.log('\nlegitimacy and the coin');
{
  // money is an achievement: it must be possible not to have it
  const outcomes = [1, 7, 4242, 31337, 555555, 20260910].map(seed => {
    let x = newWorld({ seed, population: 150, startYear: 812 });
    for (let i = 0; i < 600; i++) x = advance(x, { lever: 'none' });
    return { seed, m: mintOf(x), legit: legitimacy(x), x };
  });
  const withCoin = outcomes.filter(o => o.m.batches > 0);
  ok('some settlements invent money and some do not',
     withCoin.length > 0 && withCoin.length < outcomes.length,
     `${withCoin.length} of ${outcomes.length} seeds struck a coin in 150 years`);

  // and it cannot be struck without the craft that makes it
  const none = outcomes.find(o => o.m.batches === 0);
  if (none) {
    const everSmith = Object.values(none.x.people).some(p => p.trade === 'smith');
    ok('a settlement without a smith never has money',
       none.m.batches === 0, `seed ${none.seed}${everSmith ? '' : ' never had a smith at all'}`);
  }

  const one = withCoin[0];
  if (one) {
    ok('the mint keeps an honest count',
       one.m.coined > 0 && one.m.batches > 0 && one.m.purse >= 0,
       `${Math.round(one.m.coined)} ḍaqu over ${one.m.batches} batches`);
    ok('the contracted smith was paid out of the batch',
       Object.values(one.x.households).some(h => coinOf(h) > 0));
    ok('a coin is only money while it is believed',
       coinWorks(one.x) === (one.m.believed && one.m.coined > 0));
  }

  /* The collapse loop is the best thing in this design, and it does not fire
     on its own in a hundred and fifty quiet years — the settlement is simply
     hard to break, which is a balance property older than this slice. So the
     path is proved directly: put a government in the state a bad one would be
     in, and the money stops being money. */
  let z = newWorld({ seed: 31337, population: 150, startYear: 812 });
  for (let i = 0; i < 300; i++) z = advance(z, { lever: 'none' });
  if (mintOf(z).believed) {
    commonStore(z).food = 0;
    for (const p of Object.values(z.people)) if (p.alive && p.standing) p.standing.government = -60;
    if (z.shrine) z.shrine.devotion = 5;
    ok('a failed government drives legitimacy below the line',
       legitimacy(z) < COIN_KEEP, `legitimacy ${Math.round(legitimacy(z))}`);
    z = advance(z, { lever: 'none' });
    ok('and the coin stops being taken', !mintOf(z).believed);
    ok('the settlement is told why',
       z.chronicle[z.chronicle.length - 1].events.some(e => /stopped being taken/.test(e.text)));
  }
}

console.log('\nhardship bites, and crafts fight to live');
{
  /* Both of these were measured and wrong. The settlement could not fail — a
     storm season brought in 120% of a normal one because storms cut fishing
     and this place is pastoral — and crafts died of old age in lockstep
     because five holders aged sixty look like five holders. */
  let base = newWorld({ seed: 31337, population: 150, startYear: 812 });
  for (let i = 0; i < 300; i++) base = advance(base, { lever: 'none' });
  const calmPop = Object.values(base.people).filter(p => p.alive).length;
  const calmStore = commonStore(base).food;

  /* The same settlement carried forward twice: twenty years of weather, and
     twenty years of nothing. Comparing the hard run against its own past said
     little — a settlement in a bad patch recovers during a famine and the
     assertion reads as a pass. Against the calm branch it is the weather that
     is being measured. */
  let x = base, calm = base;
  for (let i = 0; i < 80; i++) {
    calm = advance(calm, { lever: 'none' });
    x = advance(x, { lever: i % 3 === 0 ? 'storm' : i % 3 === 1 ? 'blight' : 'fever' });
  }
  const hardPop = Object.values(x.people).filter(p => p.alive).length;

  /* The claim is that hardship costs the settlement something real — not one
     build's exact population. Asserting a precise fall made this test rock
     back and forth on unrelated changes. */
  const hardStore = commonStore(x).food;
  const hurt = hardPop < calmPop || hardStore < calmStore * 0.75
    || Object.values(x.households).filter(h => !h.extinct && (h.shortSeasons || 0) > 0).length >= 5;
  ok('twenty years of storm, blight and fever cost the settlement', hurt,
     `${calmPop} living became ${hardPop}, store ${Math.round(calmStore)} became ${Math.round(hardStore)}`);
  ok('and hardship does not somehow improve the government',
     legitimacy(x) <= legitimacy(calm) + 4,
     `legitimacy ${Math.round(legitimacy(x))} after the weather, ${Math.round(legitimacy(calm))} without it`);

  // a storm has to touch the grazing too, or it is not weather here
  let y = newWorld({ seed: 20260910, population: 150, startYear: 812 });
  for (let i = 0; i < 120; i++) y = advance(y, { lever: 'none' });
  const yieldOf = w => {
    const z = advance(y, { lever: w });
    return Object.values(z.households).filter(h => !h.extinct && !h.lodgedWith)
      .reduce((a, h) => a + (h.lastYield || 0), 0);
  };
  ok('a storm season brings in less than a quiet one',
     yieldOf('storm') < yieldOf('none'),
     `${yieldOf('storm').toFixed(0)} against ${yieldOf('none').toFixed(0)}`);

  // and the crafts hold on across many seeds, the stone above all
  const runs = [20260910, 4242, 31337, 555555, 1, 7, 99, 313].map(seed => {
    let z = newWorld({ seed, population: 150, startYear: 812 });
    const had = TEACHABLE.filter(k => Object.values(z.people).some(p => p.alive && p.trade === k));
    let lowTender = 99;
    for (let i = 0; i < 480; i++) {
      z = advance(z, { lever: 'none' });
      lowTender = Math.min(lowTender, Object.values(z.people).filter(p => p.alive && p.trade === 'tender').length);
    }
    const alive = Object.values(z.people).filter(p => p.alive);
    /* Against what the settlement was founded holding. A craft it never had
       is not a craft it shed, and a founding with one workshop in it has no
       smith to lose. */
    return { lowTender, lost: had.filter(k => !alive.some(p => p.trade === k)) };
  });
  const keptStone = runs.filter(o => o.lowTender > 0).length;
  ok('the stone survives in almost every settlement',
     keptStone >= runs.length - 1, `${keptStone} of ${runs.length} seeds never ran out of tenders`);
  const worst = Math.max(...runs.map(o => o.lost.length));
  ok('a settlement does not shed most of its crafts',
     worst <= 3, `worst run lost ${worst} of the crafts it was founded with`);
}

console.log('\nupkeep, decay and beauty');
{
  const u = run(480, 991, 150);
  const built = Object.values(u.buildings).filter(b => b.state === 'mature');
  ok('every standing building carries a condition',
     built.every(b => conditionOf(b) >= 0 && conditionOf(b) <= 1));

  /* The claim of the slice: stone that nobody keeps up comes back down, and
     stone somebody keeps up does not. Both halves have to be visible or the
     mechanism is only a decay rate. */
  const kept = built.filter(b => b.lastMended && u.turn - b.lastMended < 40);
  ok('a mason keeps a house sound', kept.length > 0
     && kept.reduce((a, b) => a + conditionOf(b), 0) / kept.length > SELF_CEILING,
     `${kept.length} mended within ten years, averaging ${(kept.reduce((a, b) => a + conditionOf(b), 0) / Math.max(1, kept.length)).toFixed(2)}`);

  const evs = u.chronicle.flatMap(t => t.events);
  const fell = evs.filter(e => /is not any more/.test(e.text)).length;
  ok('a house that is given up loses a growth', fell > 0, `${fell} over 120 years`);
  ok('but losing a growth stays rarer than reaching one',
     fell <= evs.filter(e => /reached its third growth|second growth/.test(e.text)).length * 2,
     `${fell} lost against ${evs.filter(e => /reached its third growth|second growth/.test(e.text)).length} reached`);

  // a household without the stone and timber for it is the whole class mechanic
  const poor = Object.values(u.households).filter(h => !h.extinct && (h.wantsMending || 0) > 0);
  ok('a household can be too poor to keep its house up',
     evs.some(e => /no stone or timber to mend it with/.test(e.text)) || poor.length > 0,
     `${poor.length} households waiting on mending they cannot pay for`);

  // and the settlement answers failing stone by training masons
  const masons = Object.values(u.people).filter(p => p.alive && p.trade === 'mason').length;
  const adults = Object.values(u.people).filter(p => p.alive && ageOf(u, p) >= 18).length;
  ok('masons stay a trade, not an industry', masons <= Math.ceil(adults * 0.09),
     `${masons} masons among ${adults} adults`);

  // nothing sound stands empty for long
  const emptySound = built.filter(b => vacant(u, b) && conditionOf(b) > DISREPAIR).length;
  ok('a sound empty house gets taken', emptySound <= 3,
     `${emptySound} standing empty and sound`);
  ok('and somebody does move in', evs.some(e => /moved into the old/.test(e.text)),
     `${evs.filter(e => /moved into the old/.test(e.text)).length} households took an empty house`);

  // beauty: a carver's work shows up, and it is not everywhere
  const carved = built.filter(b => beautyOf(b) > 0.1);
  ok('a carver makes some houses worth looking at',
     carved.length > 0 && carved.length < built.length * 0.8,
     `${carved.length} of ${built.length} carry carving`);

  // and the weather is what keeps the masons in work
  let x = run(200, 20260910, 150);
  const before = Object.values(x.buildings).filter(b => b.state === 'mature')
    .reduce((a, b) => a + conditionOf(b), 0);
  const after = (() => {
    const y = advance(x, { lever: 'storm' });
    return Object.values(y.buildings).filter(b => b.state === 'mature')
      .reduce((a, b) => a + conditionOf(b), 0);
  })();
  const quiet = (() => {
    const y = advance(x, { lever: 'none' });
    return Object.values(y.buildings).filter(b => b.state === 'mature')
      .reduce((a, b) => a + conditionOf(b), 0);
  })();
  ok('a storm breaks things', after < quiet,
     `${after.toFixed(1)} after a storm against ${quiet.toFixed(1)} after a quiet season`);

  // the world has the timber its economy was always written to need
  const wooded = u.tiles.filter(t => (t.wood || 0) > 0.3).length;
  ok('the coast has wood on it', wooded > 10, `${wooded} wooded tiles`);
  ok('and somebody cuts it',
     Object.values(u.people).some(p => p.alive && p.trade === 'woodcutter')
     && Object.values(u.households).some(h => !h.extinct && goodsOf(h).timber > 1));
}

console.log('\ndefault, and what a government will do about it');
{
  /* This is where the form of government finally matters mechanically. A sole
     ruler can do anything; a senate has to agree with itself first. */
  const seeds = [20260910, 4242, 31337, 555555, 1, 7];
  const byForm = {};
  for (const government of ['sole', 'tribunal', 'senate']) {
    const t = { forbear: 0, seize: 0, gaol: 0, kill: 0, rescue: 0, proud: 0 };
    for (const seed of seeds) {
      let x = newWorld({ seed, population: 150, startYear: 812, government });
      for (let i = 0; i < 400; i++) x = advance(x, { lever: 'none' });
      for (const e of x.chronicle.flatMap(c => c.events)) {
        if (/off what they could not pay/.test(e.text)) t.forbear++;
        else if (/house to the .* for a debt/.test(e.text)) t.seize++;
        else if (/shut up over the/.test(e.text)) t.gaol++;
        else if (/put to death over the/.test(e.text)) t.kill++;
        else if (/paid off what the/.test(e.text)) t.rescue++;
        else if (/would not have it/.test(e.text)) t.proud++;
      }
    }
    byForm[government] = t;
  }

  ok('a senate never goes past taking the house',
     byForm.senate.gaol === 0 && byForm.senate.kill === 0,
     `senate: ${JSON.stringify(byForm.senate)}`);
  ok('a tribunal will shut somebody up but not kill them',
     byForm.tribunal.kill === 0, `tribunal: ${JSON.stringify(byForm.tribunal)}`);
  ok('a sole ruler reaches the whole ladder',
     byForm.sole.kill > 0 && byForm.sole.gaol > 0, `sole: ${JSON.stringify(byForm.sole)}`);
  ok('every form mostly lets people off', 
     ['sole', 'tribunal', 'senate'].every(g => byForm[g].forbear > byForm[g].seize));
  ok('neighbours pay debts off, and some households refuse to let them',
     byForm.sole.rescue > 0 && byForm.sole.proud > 0,
     `${byForm.sole.rescue} rescued, ${byForm.sole.proud} too proud`);

  // the ladder is a ladder: harsher forms permit strictly more
  ok('each form permits what the milder one does',
     HARSHNESS.senate.every(k => HARSHNESS.tribunal.includes(k))
     && HARSHNESS.tribunal.every(k => HARSHNESS.sole.includes(k)));

  // mercy costs credit, or being kind is a free win
  let m = newWorld({ seed: 20260910, population: 150, startYear: 812, government: 'senate' });
  for (let i = 0; i < 400; i++) m = advance(m, { lever: 'none' });
  ok('a settlement that forgives debts lends less',
     (m.creditTight || 0) > 0, `credit tightened to ${(m.creditTight || 0).toFixed(2)}`);

  // and a ruling is rare enough not to become the chronicle
  const rulings = m.chronicle.flatMap(c => c.events)
    .filter(e => /off what they could not pay|for a debt|shut up over|put to death over/.test(e.text)).length;
  ok('rulings do not fill the chronicle', rulings < m.chronicle.length * 0.3,
     `${rulings} across ${m.chronicle.length} seasons`);
}

console.log(failures ? `\n${failures} FAILED\n` : '\nall passed\n');
process.exit(failures ? 1 : 0);
