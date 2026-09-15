/* ===========================================================================
   Panel rendering: the cast, the households. Reads state, returns HTML.
   =========================================================================== */

/* --- the standing cast -------------------------------------------------------- */

function castHTML(s) {
  const actors = Object.values(s.people).filter(p => p.alive && p.prominence > 0)
    .sort((a, b) => b.prominence - a.prominence);
  if (!actors.length) return `<p class="empty">No named figures. Nobody is pursuing a goal.</p>`;
  return actors.map(p => {
    const hh = s.households[p.householdId];
    return `<div class="figure">
      <div class="fname">${p.name} <span class="flin">${p.lineage}</span></div>
      <div class="fmeta">${ageOf(s, p)} years · ${describeTraits(p)}</div>
      <div class="fgoal">${p.goal ? 'Goal: ' + p.goal.label + '.' : 'No goal.'}</div>
      ${p.deeds.length ? `<div class="fdeeds">Known for: ${p.deeds.slice(-3).join('; ')}.</div>` : ''}
      ${p.grudges.length ? `<div class="fgrudge">Holds ${p.grudges.length} grudge${p.grudges.length > 1 ? 's' : ''}.</div>` : ''}
    </div>`;
  }).join('');
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
