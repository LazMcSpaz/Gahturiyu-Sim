/* The standard reading of a settlement. Everything the balancing work kept
   measuring by hand, in one object, so a sweep across seeds or a comparison
   against another build asks the same questions every time.

   measure(E, s, s0, track) — E is a loaded engine, s the state to read, s0 the
   founding state of the same run, track the per-turn track (see sweep-worker).
   Every number is a plain JSON scalar so it survives a thread boundary. */

function measure(E, s, s0, track) {
  const people = Object.values(s.people);
  const alive = people.filter(p => p.alive);
  const adults = alive.filter(p => E.ageOf(s, p) >= 18);
  const houses = Object.values(s.households).filter(h => !h.extinct);
  const built = Object.values(s.buildings).filter(b => b.state === 'mature');
  const stages = { 1: 0, 2: 0, 3: 0, 4: 0 };
  for (const b of built) stages[Math.min(4, E.stageOf(b))]++;
  const evs = s.chronicle.flatMap(t => t.events);
  const byKind = {};
  for (const e of evs) byKind[e.kind] = (byKind[e.kind] || 0) + 1;
  const topKind = Object.entries(byKind).sort((a, b) => b[1] - a[1])[0] || ['-', 0];
  const trades = {};
  for (const p of adults) if (p.trade) trades[p.trade] = (trades[p.trade] || 0) + 1;
  const founded = E.TEACHABLE.filter(k => Object.values(s0.people).some(p => p.alive && p.trade === k));
  const held = new Set(Object.values(s.people).filter(p => p.alive && p.trade).map(p => p.trade));
  const lost = founded.filter(k => !held.has(k));
  const mint = E.mintOf(s);
  const n = t => evs.filter(e => t.test(e.text)).length;
  const store = E.commonStore(s);

  return {
    turn: s.turn,
    government: s.government,
    pop: alive.length,
    adults: adults.length,
    households: houses.length,
    lodged: houses.filter(h => h.lodgedWith).length,
    mature: built.length,
    derelict: Object.values(s.buildings).filter(b => b.state === 'derelict').length,
    stage1: stages[1], stage2: stages[2], stage3: stages[3], landmark: stages[4],
    workshops: houses.filter(h => E.hasWorkshop(s, h)).length,
    disrepair: built.filter(b => E.inDisrepair(b)).length,
    condition: built.length ? +(built.reduce((a, b) => a + E.conditionOf(b), 0) / built.length).toFixed(2) : 0,
    carved: built.filter(b => E.beautyOf(b) > 0.1).length,
    craftsHeld: held.size,
    craftsLost: lost.length,
    lost: lost.join(','),
    biggestTrade: Math.max(0, ...Object.values(trades)),
    biggestTradeName: (Object.entries(trades).sort((a, b) => b[1] - a[1])[0] || ['-'])[0],
    biggestCraft: Math.max(0, ...E.TEACHABLE.map(k => trades[k] || 0)),
    tenders: trades.tender || 0,
    masons: trades.mason || 0,
    lowestTenders: track ? Math.min(...track.map(t => t.tenders)) : null,
    lowestStone: track ? Math.min(...track.map(t => t.stone)) : null,       // holders and learners together
    shownPerSeason: track ? +(track.reduce((a, t) => a + t.shown, 0) / track.length).toFixed(1) : null,
    quarters: (s.quarters || []).length,
    legitimacy: Math.round(E.legitimacy(s)),
    coinBatches: mint.batches || 0,
    coinBelieved: !!mint.believed,
    storeFood: Math.round(store.food),
    storeGiven: Math.round(store.given),
    debts: E.debtsOf(s).length,
    creditTightPeak: track ? +Math.max(...track.map(t => t.creditTight)).toFixed(2) : null,
    linesPerSeason: +(evs.length / Math.max(1, s.chronicle.length)).toFixed(1),
    topKind: `${topKind[0]} ${Math.round(topKind[1] / Math.max(1, evs.length) * 100)}%`,
    levies: n(/levied|refused the levy/),
    blocs: n(/stopped working and will not be moved|will not work for anyone outside/),
    fellBack: n(/is not any more\. Too much/),
    rulings: n(/off what they could not pay|for a debt|shut up over|put to death over/),
    moved: n(/moved into the old/),
    leftCoast: n(/left the coast/),
    boatDenials: n(/refused the .* a boat|priced the .* off the water/),
    hungerLines: evs.filter(e => e.kind === 'hunger').length,
    span: (() => { const xs = built.map(b => E.tileXY(b.tileId)[0]); return xs.length ? Math.max(...xs) - Math.min(...xs) : 0; })(),
    gaveUpCraft: n(/nowhere to work as a/)
  };
}

/* Things that have always meant something was wrong when they showed up. A
   flag is a reason to look, not a failure. */
function flags(m, years) {
  const out = [];
  if (m.pop < 50) out.push(`pop ${m.pop}`);
  if (m.craftsLost > 1) out.push(`lost ${m.lost}`);
  if (m.lowestStone !== null && m.lowestStone === 0) out.push('the stone was lost');
  else if (m.lowestTenders !== null && m.lowestTenders === 0) out.push('no working tender for a while');
  if (m.shownPerSeason !== null && m.shownPerSeason > 6) out.push(`${m.shownPerSeason} shown lines/season`);
  if (years >= 100 && m.stage3 === 0) out.push('no third growth');
  if (m.mature && m.disrepair / m.mature > 0.4) out.push(`${m.disrepair}/${m.mature} in disrepair`);
  if (parseInt(m.topKind.split(' ')[1]) > 30) out.push(`chronicle is ${m.topKind}`);
  if (m.biggestCraft > m.adults * 0.25) out.push(`one craft holds ${m.biggestCraft}/${m.adults}`);
  if (m.biggestTrade > m.adults * 0.4) out.push(`${m.biggestTradeName} is ${m.biggestTrade}/${m.adults} of adults`);
  if (m.workshops === 0 && years >= 50) out.push('no workshops');
  return out;
}

module.exports = { measure, flags };
