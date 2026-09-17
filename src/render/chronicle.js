/* ===========================================================================
   Rendering: the chronicle, and the map plate.
   The chronicle is a record of facts, one per line, in a fixed order.
   No connective prose, no seasonal scene-setting — if a line is there,
   something happened, and the line says what.
   =========================================================================== */

/* Events are ordered by category, not by when in the turn they fired, so the
   same kind of news always turns up in the same place in a season.

   `max` caps a group per season. Removing the cast limit means a busy season
   can produce eleven lines about what named people did, and the settlement has
   other news. The events are all still recorded and the simulation still uses
   them — this decides what reaches the page, which is where volume belongs.
   Showing everything lifts the caps along with the weight floor. */
const GROUPS = [
  { key: 'weather', kinds: ['divine', 'weather'] },
  { key: 'hardship', kinds: ['hunger', 'hardship', 'death', 'extinct', 'derelict', 'leave'], max: 4 },
  { key: 'stone', kinds: ['teach', 'note', 'mature', 'stage', 'stall', 'tend', 'notender', 'claim', 'birth', 'split', 'succession', 'arrival', 'match'], max: 4 },
  { key: 'quarrel', kinds: ['office', 'spite', 'dispute', 'arbitration', 'feud', 'reveal'], max: 4 },
  { key: 'shrine', kinds: ['shrine'] },
  { key: 'people', kinds: ['rise', 'act', 'settle'], max: 2 },
  { key: 'memory', kinds: ['memory'] }
];

function renderTurn(entry, opts = {}) {
  const evs = entry.events.filter(e => e.weight >= (opts.floor ?? 2));
  const uncapped = (opts.floor ?? 2) < 2;
  const parts = [];

  for (const g of GROUPS) {
    let items = evs.filter(e => g.kinds.includes(e.kind));
    items.sort((x, y) => y.weight - x.weight);
    if (g.max && !uncapped) items = items.slice(0, g.max);
    for (const e of items) parts.push({ type: 'event', kind: e.kind, weight: e.weight, text: e.text });
  }

  if (!parts.length) parts.push({ type: 'quiet', text: 'Nothing recorded.' });
  return parts;
}

function chronicleHTML(entry, opts) {
  const parts = renderTurn(entry, opts);
  let html = `<article class="turn" id="turn-${entry.turn}">`;
  html += `<h2 class="season"><span class="s-name">${entry.season}</span><span class="s-year">${entry.year}</span></h2>`;
  if (entry.input && entry.input !== 'none' && typeof LEVERS !== 'undefined' && LEVERS[entry.input])
    html += `<p class="hand">Your doing: ${LEVERS[entry.input].label}</p>`;
  for (const p of parts) {
    if (p.type === 'quiet') html += `<p class="quiet">${p.text}</p>`;
    else html += `<p class="event w${Math.min(7, p.weight)} k-${p.kind}">${p.text}</p>`;
  }
  const st = entry.stats;
  if (entry.govChange) html += `<p class="hand">${entry.govChange}</p>`;
  if (entry.shrine) html += `<p class="tally shrinerow">${entry.shrine.god} stone: ${entry.shrine.devotion}/100 kept${entry.shrine.keeper ? ', keeper ' + entry.shrine.keeper : ', no keeper'}</p>`;
  if (st) html += `<p class="tally">${st.pop} living · ${st.households} households · ${st.mature} homes, ${st.growing} growing${st.waiting ? ' · ' + st.waiting + ' waiting on ground' : ''} · ${st.tenders} tenders${st.disputes ? ' · ' + st.disputes + ' open quarrel' + (st.disputes > 1 ? 's' : '') : ''}</p>`;
  html += `</article>`;
  return html;
}

/* --- the map ----------------------------------------------------------------- */

const GLYPH = {
  sea: '·', shore: '~', slope: '˄', crag: '▲', moor: ',',
  mature: '⌂', growing: '◌', derelict: '×',
  third: '▣', landmark: '✦'      // a building that grew past being a house
};

/* Households named in the season just past, so the map can show you who the
   newest lines are actually about. */
function inTheNews(s) {
  const last = s.chronicle[s.chronicle.length - 1];
  const out = new Set();
  if (!last) return out;
  for (const e of last.events) {
    if (e.household) out.add(e.household);
    if (e.person && s.people[e.person]) out.add(s.people[e.person].householdId);
    if (e.building && s.buildings[e.building]) out.add(s.buildings[e.building].householdId);
  }
  return out;
}

function mapHTML(s, selected) {
  const occupied = {};
  for (const b of Object.values(s.buildings)) {
    const hh = s.households[b.householdId];
    const live = hh && hh.members.some(i => s.people[i].alive);
    occupied[b.tileId] = { b, live };
  }
  const actorTiles = {};
  for (const p of Object.values(s.people)) {
    if (p.alive && hasGoal(p)) actorTiles[homeTile(s, p)] = p;
  }
  const disputed = new Set();
  for (const d of Object.values(s.disputes)) {
    if (!d.open) continue;
    for (const hid of [d.a, d.b]) for (const c of (s.households[hid]?.claims || [])) disputed.add(c);
  }

  const news = inTheNews(s);
  const newsTiles = new Set();
  for (const b of Object.values(s.buildings)) if (news.has(b.householdId)) newsTiles.add(b.tileId);
  for (const hid of news) for (const c of (s.households[hid]?.claims || [])) newsTiles.add(c);

  const shrineTile = s.shrine ? s.shrine.tileId : -1;
  let out = '';
  for (let y = 0; y < H; y++) {
    let row = '';
    for (let x = 0; x < W; x++) {
      const id = tileId(x, y);
      const t = s.tiles[id];
      const o = occupied[id];
      let cls = ['t'], g;
      if (o) {
        if (o.b.state === 'derelict') { g = GLYPH.derelict; cls.push('m-derelict'); }
        else if (o.b.state === 'growing') { g = GLYPH.growing; cls.push('m-growing'); }
        else {
          const st = stageOf(o.b);
          g = st >= 4 ? GLYPH.landmark : st >= 3 ? GLYPH.third : GLYPH.mature;
          if (!o.live) cls.push('m-dark');
          else if (st >= 3) cls.push('m-' + roleOf(o.b));
          else cls.push(st >= 2 ? 'm-workshop' : 'm-lit');
        }
        // a house visibly falling in reads as one, whatever else it is
        if (o.b.state === 'mature' && inDisrepair(o.b)) cls.push('m-worn');
        if (actorTiles[id]) cls.push('m-actor');
      } else {
        g = t.t === SEA ? GLYPH.sea : t.t === SHORE ? GLYPH.shore : t.t === SLOPE ? GLYPH.slope : t.t === CRAG ? GLYPH.crag : GLYPH.moor;
        cls.push('g-' + ['sea', 'shore', 'slope', 'crag', 'moor'][t.t]);
        if (t.owner) cls.push('owned');
      }
      if (id === shrineTile) {
        g = '†';
        cls = ['t', s.shrine.devotion > 55 ? 'm-shrine' : 'm-shrine-cold'];
      }
      if (disputed.has(id)) cls.push('m-disputed');
      if (newsTiles.has(id)) cls.push('m-news');
      if (id === selected) cls.push('m-sel');
      row += `<span class="${cls.join(' ')}" data-t="${id}">${g}</span>`;
    }
    out += `<div class="mrow">${row}</div>`;
  }
  return out;
}
