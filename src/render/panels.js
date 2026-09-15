/* ===========================================================================
   Panel rendering: the cast, the households. Reads state, returns HTML.
   =========================================================================== */

/* --- the standing cast -------------------------------------------------------- */

function castHTML(s) {
  const actors = Object.values(s.people).filter(p => p.alive && hasGoal(p))
    .sort((a, b) => b.goalAge - a.goalAge).slice(0, 20);
  if (!actors.length) return `<p class="empty">No named figures. Nobody is pursuing a goal.</p>`;
  return actors.map(p => {
    const hh = s.households[p.householdId];
    return `<div class="figure">
      <div class="fname">${p.name} <span class="flin">${p.lineage}</span></div>
      <div class="fmeta">${ageOf(s, p)} years · ${describeTraits(p)}</div>
      <div class="fgoal">${p.goal ? 'Goal: ' + p.goal.label + '.' : 'No goal.'}</div>
      ${p.deeds.length ? `<div class="fdeeds">Known for: ${p.deeds.slice(-3).join('; ')}.</div>` : ''}
      ${tieSummary(p)}
    </div>`;
  }).join('');
}

/* What a person holds, in both directions. A tie is one number, so this is one
   question asked twice. */
function tieSummary(p) {
  const ties = p.ties || [];
  const foes = ties.filter(t => t.value <= -40).length;
  const friends = ties.filter(t => t.value >= 40).length;
  const bits = [];
  if (friends) bits.push(`<span class="fbond">Close to ${friends} household${friends > 1 ? 's' : ''}.</span>`);
  if (foes) bits.push(`<span class="fgrudge">At odds with ${foes} household${foes > 1 ? 's' : ''}.</span>`);
  return bits.length ? `<div class="fties">${bits.join(' ')}</div>` : '';
}

function householdsHTML(s) {
  const hs = Object.values(s.households).filter(h => !h.extinct)
    .sort((a, b) => b.standing - a.standing);
  return hs.map(h => {
    const live = h.members.filter(i => s.people[i].alive).length;
    const b = s.buildings[h.buildingId];
    const growing = Object.values(s.buildings).find(x => x.householdId === h.id && x.state === 'growing');
    const home = b && b.state === 'mature' ? 'housed'
      : growing ? `stone at ${Math.round(growing.maturity * 100)}%`
      : 'no ground';
    const rep = reputeOf(s, h);
    const said = rep > 6 ? 'good standing' : rep < -6 ? 'bad standing' : '';
    const mine = s.memory.filter(m => m.household === h.id && (rep < 0 ? m.favour < 0 : m.favour > 0))
      .sort((a, b) => Math.abs(b.favour) - Math.abs(a.favour))[0];
    return `<div class="hrow ${home === 'no ground' ? 'nohome' : ''}">
      <div class="hline"><span class="hname">${h.name}</span>
      <span class="hbits">${live} living · ${h.claims.length} claim${h.claims.length === 1 ? '' : 's'} · ${home} · stores ${Math.round(h.stores)}</span></div>
      ${said ? `<div class="hrep ${rep < 0 ? 'bad' : 'good'}">${said}${mine ? ' — ' + mine.phrase : ''}</div>` : ''}
    </div>`;
  }).join('');
}

/* --- the map inspector --------------------------------------------------------
   The chronicle names households and quarters constantly; the map draws every
   house with the same glyph. This is what closes that loop: tap a tile and it
   says whose it is, in the same terms the chronicle uses.
   -------------------------------------------------------------------------- */

function tileFactsHTML(s, id) {
  const t = s.tiles[id];
  if (!t) return '';
  const q = quarterOf(s, id);
  const where = !q ? TERRAIN_NAME[t.t]
    : t.t === SEA ? `open water off ${q.name}`
    : `the ${TERRAIN_NAME[t.t]} at ${q.name}`;

  const rows = [];
  const row = (k, v, cls) => rows.push(
    `<div class="irow${cls ? ' ' + cls : ''}"><span class="ik">${k}</span><span class="iv">${v}</span></div>`);

  // the shrine belongs to nobody, and outranks whatever else is on the tile
  if (s.shrine && s.shrine.tileId === id) {
    const d = Math.round(s.shrine.devotion);
    row('Shrine', `the ${s.shrine.god} stone`);
    row('Kept', `${d}/100 — ${d < 30 ? 'untended' : d < 55 ? 'barely kept' : d < 78 ? 'kept' : 'well kept'}`);
    row('Keeper', s.shrine.keeperId ? s.shrine.keeperName : 'nobody', s.shrine.keeperId ? '' : 'bad');
    row('Rulings', d > 60 ? 'sworn here, and they bind'
      : d > 35 ? 'some quarrels come here, some do not'
      : 'nobody brings quarrels here');
    return inspectorHTML(where, rows);
  }

  const b = Object.values(s.buildings).find(x => x.tileId === id);
  const hh = b ? s.households[b.householdId] : (t.owner ? s.households[t.owner] : null);

  if (!hh) {
    // open ground: the only question that matters is whether a house can grow here
    if (t.t === SEA) {
      row('Water', t.fish > 0.45 ? 'good fishing' : t.fish > 0.15 ? 'fishable' : 'poor fishing');
    } else {
      const can = siteValue(s.tiles, id) > 0;
      const homes = Object.values(s.buildings).filter(x => x.state !== 'derelict').map(x => x.tileId);
      const tooClose = can && homes.some(x => tileDist(x, id) < 2.15);
      row('Ground', 'unclaimed');
      row('Will it take a house?', !can ? 'no — the stone is too thin here'
        : tooClose ? 'no — too close to a standing house' : 'yes', (can && !tooClose) ? 'good' : 'bad');
      if (t.fish > 0.15) row('Fishing', t.fish > 0.45 ? 'good' : 'fair');
      if (t.graze > 0.2) row('Grazing', t.graze > 0.45 ? 'good' : 'fair');
    }
    return inspectorHTML(where, rows);
  }

  // a household holds this tile
  const live = hh.members.map(i => s.people[i]).filter(p => p && p.alive);
  const head = s.people[hh.headId];
  row('Household', `the ${hh.name}${hh.extinct ? ' — ended' : ''}`);
  if (b) {
    row('House', b.state === 'mature' ? 'standing'
      : b.state === 'derelict' ? 'derelict'
      : `${Math.round(b.maturity * 100)}% grown${b.stalled > 0 ? `, stalled ${Math.round(b.stalled / 4)} years` : ''}`,
      b.state === 'derelict' ? 'bad' : b.state === 'growing' ? 'growing' : '');
    if (b.state === 'growing') {
      const tn = b.tenderId && s.people[b.tenderId];
      row('Tender', tn && tn.alive ? nameOf(s, tn.id) : 'nobody is working it', tn && tn.alive ? '' : 'bad');
    }
  } else {
    row('House', 'none — this is claimed ground', 'bad');
  }
  if (hh.extinct) return inspectorHTML(where, rows);

  row('Living here', `${live.length}${head && head.alive ? `, under ${nameOf(s, head.id)}` : ''}`);
  row('Stores', `${Math.round(hh.stores)}${(hh.shortSeasons || 0) >= 3 ? ` — short ${hh.shortSeasons} seasons` : ''}`,
    (hh.shortSeasons || 0) >= 3 ? 'bad' : '');

  const rep = reputeOf(s, hh);
  const others = Object.values(s.households).filter(h => !h.extinct)
    .sort((a, c) => standingOf(s, c) - standingOf(s, a));
  const rank = others.findIndex(h => h.id === hh.id) + 1;
  row('Standing', `${rank ? `${ordinal(rank)} of ${others.length}` : '—'}` +
    `${rep > 6 ? ', well regarded' : rep < -6 ? ', badly regarded' : ''}`,
    rep > 6 ? 'good' : rep < -6 ? 'bad' : '');

  // regard, faction by faction — the composite above is a summary, these are
  // what decisions actually read
  const regard = FACTIONS
    .map(f => ({ f, v: houseStanding(s, hh, f) }))
    .filter(x => Math.abs(x.v) >= 3)
    .sort((a, c) => Math.abs(c.v) - Math.abs(a.v));
  if (regard.length) {
    row('Regard', regard.map(x =>
      `<span class="reg ${x.v < 0 ? 'bad' : 'good'}">${x.f} ${x.v > 0 ? '+' : ''}${Math.round(x.v)}</span>`).join(' '));
  }

  const ties = hh.members.flatMap(i => (s.people[i] && s.people[i].alive) ? (s.people[i].ties || []) : []);
  const close = ties.filter(x => x.value >= 40).length;
  const foes = ties.filter(x => x.value <= -40).length;
  if (close || foes) {
    row('Ties', [close ? `${close} close` : '', foes ? `${foes} at odds` : ''].filter(Boolean).join(', '),
      foes > close ? 'bad' : 'good');
  }

  const served = hh.members.map(i => s.people[i]).filter(p => p && p.alive && p.term && p.term.done).length;
  const refused = hh.members.map(i => s.people[i]).filter(p => p && p.alive && p.refusedTerm).length;
  if (refused) row('The term', `${refused} refused it${served ? `, ${served} served` : ''}`, 'bad');

  const tenders = live.filter(p => p.tender);
  if (tenders.length) row('Can work stone', tenders.map(p => nameOf(s, p.id)).join(', '), 'good');

  const figures = live.filter(hasGoal);
  for (const p of figures) row('Figure', `${nameOf(s, p.id)} — ${p.goal ? p.goal.label : 'no goal'}`);

  const open = Object.values(s.disputes).filter(d => d.open && (d.a === hh.id || d.b === hh.id));
  for (const d of open) {
    const other = s.households[d.a === hh.id ? d.b : d.a];
    row('Quarrel', `${d.a === hh.id ? 'against' : 'with'} the ${other ? other.name : 'another household'} over ${d.over}`, 'bad');
  }

  const held = s.memory.filter(m => m.household === hh.id)
    .sort((a, c) => Math.abs(c.favour) - Math.abs(a.favour))[0];
  if (held) row('Remembered', `${held.phrase} — ${Math.round((s.turn - held.turn) / 4)} years ago`,
    held.favour < 0 ? 'bad' : 'good');

  return inspectorHTML(where, rows);
}

/* 1st, 2nd, 3rd, 4th — the engine got this wrong once and printed "3th". */
function ordinal(n) {
  const rem100 = n % 100;
  if (rem100 >= 11 && rem100 <= 13) return n + 'th';
  return n + ({ 1: 'st', 2: 'nd', 3: 'rd' }[n % 10] || 'th');
}

function inspectorHTML(where, rows) {
  return `<div class="ihead">${where}</div>` + rows.join('');
}
