/* ===========================================================================
   Rendering: the chronicle, and the map plate.
   The chronicle's job is to read as a record someone kept, not as a log.
   =========================================================================== */

function det(turn, salt) { return mulberry32((turn * 7919 + salt * 104729) | 0)(); }
function detPick(turn, salt, arr) { return arr[Math.floor(det(turn, salt) * arr.length)]; }

/* --- season openers --------------------------------------------------------- */

const OPENERS = {
  spring: [
    'The ice went out of the ground and the paths turned to mud.',
    'Spring came late and grudging, the way it does on this coast.',
    'The first green showed in the grooves of the older houses.',
    'Boats went back in the water and the shore smelled of tar again.'
  ],
  summer: [
    'Long light. The stone held its heat well past dark.',
    'A dry stretch, and the wells on the upper slope ran low.',
    'The settlement worked outdoors from first light and the children ran loose.',
    'Nets out every morning, and the rings on the new houses came up a shade paler.'
  ],
  harvest: [
    'The moor went brown and everything that could be gathered was gathered.',
    'A season of counting: stores, beasts, years on the walls.',
    'The wind turned round to the north and stayed there.',
    'Everyone worked with one eye on the sky.'
  ],
  winter: [
    'Short days. Lamps lit by mid-afternoon and left burning.',
    'The sea came up grey and stayed grey for months.',
    'Doors shut early. What was in the stores in autumn was all there was.',
    'Cold enough that the damp got into the walls and stayed.'
  ]
};

const GROUPS = [
  { key: 'weather', kinds: ['divine', 'weather'] },
  { key: 'shrine', kinds: ['shrine'] },
  { key: 'hardship', kinds: ['hunger', 'hardship', 'death', 'extinct', 'derelict', 'leave'] },
  { key: 'stone', kinds: ['teach', 'note', 'mature', 'stall', 'tend', 'notender', 'claim', 'birth', 'split', 'succession', 'arrival', 'match'] },
  { key: 'quarrel', kinds: ['office', 'spite', 'dispute', 'arbitration', 'feud', 'reveal'] },
  { key: 'people', kinds: ['rise', 'act', 'settle'] },
  { key: 'memory', kinds: ['memory'] }
];

const BRIDGES = {
  'weather>hardship': ['It told on the households before the season was out.', 'The cost of it came due quickly.', 'Not everyone came through it.'],
  'weather>stone': ['Underneath all that, the ordinary business of the place went on.', 'The slower work carried on regardless.', 'The settlement did what it does in any season.'],
  'weather>quarrel': ['Hard weather has a way of shortening tempers.', 'Shut indoors together, old things surfaced.'],
  'weather>people': ['And a few made something of it.'],
  'hardship>stone': ['Against that, there was some ordinary living done.', 'Life went on in the usual way alongside it.', 'Elsewhere the slow work continued.'],
  'hardship>quarrel': ['Losses like that do not stay private long.', 'Grief found somewhere to point itself.', 'It was not long before someone was blamed.'],
  'hardship>people': ['Out of it, someone stepped forward.', 'A gap had opened, and it did not stay open.'],
  'hardship>weather': ['The weather offered no help.'],
  'stone>quarrel': ['Not everything settled so quietly.', 'Ground is the thing this place argues about, and it argued.', 'One matter would not lie down.'],
  'stone>people': ['Somebody was watching all of this with an interest of their own.', 'And one name began coming up more than it had.'],
  'stone>hardship': ['It was not all growth.'],
  'quarrel>people': ['These things need someone to carry them, and someone did.', 'A quarrel is only as long-lived as the person keeping it.'],
  'quarrel>hardship': ['While the argument ran, the season took its own.'],
  'quarrel>stone': ['Meanwhile the stone kept its own pace, indifferent.'],
  'weather>shrine': ['People went looking for a reason.', 'It sent some of them up the hill.'],
  'shrine>hardship': ['Whatever was asked for was not granted.', 'It made no difference to the stores.'],
  'shrine>stone': ['Below all that, the ordinary work.'],
  'shrine>quarrel': ['Not everything could be settled by going up there.'],
  'hardship>shrine': ['A bad enough year sends people up the hill.'],
  'stone>shrine': ['The one stone nobody owns had its own year.'],
  'people>memory': ['None of it happens against a clean slate.'],
  'quarrel>memory': ['These things have long roots here.'],
  'stone>memory': ['Older business surfaced too.'],
  'hardship>memory': ['Hard years bring up old ones.'],
  'weather>memory': ['It put people in mind of other years.']
};

const QUIET = [
  'Nothing worth recording. The stone grew, the boats went out, and the year turned.',
  'A season of no consequence, which the older heads counted as a good one.',
  'Nothing happened that anyone thought to write down.',
  'The settlement simply carried on.',
  'No deaths, no quarrels, no ground changing hands. It happens.'
];

function renderTurn(entry, opts = {}) {
  const t = entry.turn;
  const parts = [];
  const evs = entry.events.filter(e => e.weight >= (opts.floor ?? 2));

  const buckets = GROUPS.map(g => ({ key: g.key, items: evs.filter(e => g.kinds.includes(e.kind)) }))
    .filter(b => b.items.length);

  // opener: weather-led if there was weather, otherwise a season line
  const openerUsed = detPick(t, 1, OPENERS[entry.season]);
  parts.push({ type: 'prose', text: openerUsed });

  if (!buckets.length) {
    parts.push({ type: 'prose', text: detPick(t, 2, QUIET) });
    return parts;
  }

  let prev = null;
  buckets.forEach((b, i) => {
    if (prev) {
      const key = prev + '>' + b.key;
      const pool = BRIDGES[key];
      if (pool) parts.push({ type: 'bridge', text: detPick(t, 10 + i, pool) });
    }
    b.items.sort((x, y) => y.weight - x.weight);
    for (const e of b.items) parts.push({ type: 'event', kind: e.kind, weight: e.weight, text: e.text });
    prev = b.key;
  });

  return parts;
}

function chronicleHTML(entry, opts) {
  const parts = renderTurn(entry, opts);
  let html = `<article class="turn" id="turn-${entry.turn}">`;
  html += `<h2 class="season"><span class="s-name">${entry.season}</span><span class="s-year">${entry.year}</span></h2>`;
  if (entry.input) html += `<p class="hand">A hand moved over the settlement.</p>`;
  for (const p of parts) {
    if (p.type === 'prose') html += `<p class="prose">${p.text}</p>`;
    else if (p.type === 'bridge') html += `<p class="bridge">${p.text}</p>`;
    else html += `<p class="event w${Math.min(7, p.weight)} k-${p.kind}">${p.text}</p>`;
  }
  const st = entry.stats;
  if (entry.govChange) html += `<p class="hand">${entry.govChange}</p>`;
  if (entry.shrine) html += `<p class="tally shrinerow">The ${entry.shrine.god} stone: ${entry.shrine.devotion < 30 ? 'untended' : entry.shrine.devotion < 55 ? 'kept, barely' : entry.shrine.devotion < 78 ? 'kept' : 'well kept'}${entry.shrine.keeper ? ', by ' + entry.shrine.keeper : ', by nobody'}</p>`;
  if (st) html += `<p class="tally">${st.pop} living · ${st.households} households · ${st.mature} homes standing, ${st.growing} still growing${st.waiting ? ', ' + st.waiting + ' household' + (st.waiting > 1 ? 's' : '') + ' waiting on ground' : ''} · ${st.tenders} who can work stone${st.disputes ? ' · ' + st.disputes + ' unsettled quarrel' + (st.disputes > 1 ? 's' : '') : ''}</p>`;
  html += `</article>`;
  return html;
}

/* --- the map ----------------------------------------------------------------- */

const GLYPH = {
  sea: '·', shore: '~', slope: '˄', crag: '▲', moor: ',',
  mature: '⌂', growing: '◌', derelict: '×'
};

function mapHTML(s) {
  const occupied = {};
  for (const b of Object.values(s.buildings)) {
    const hh = s.households[b.householdId];
    const live = hh && hh.members.some(i => s.people[i].alive);
    occupied[b.tileId] = { b, live };
  }
  const actorTiles = {};
  for (const p of Object.values(s.people)) {
    if (p.alive && p.prominence > 0) actorTiles[homeTile(s, p)] = p;
  }
  const disputed = new Set();
  for (const d of Object.values(s.disputes)) {
    if (!d.open) continue;
    for (const hid of [d.a, d.b]) for (const c of (s.households[hid]?.claims || [])) disputed.add(c);
  }

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
        else { g = GLYPH.mature; cls.push(o.live ? 'm-lit' : 'm-dark'); }
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
      row += `<span class="${cls.join(' ')}">${g}</span>`;
    }
    out += `<div class="mrow">${row}</div>`;
  }
  return out;
}
