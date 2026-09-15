/* --- world creation --------------------------------------------------------- */

function newWorld(cfg) {
  const seed = cfg.seed >>> 0;
  const r = mulberry32(seed);
  const tiles = buildTerrain(r);

  const state = {
    version: ENGINE_VERSION,
    seed, turn: 0, startYear: cfg.startYear ?? 812,
    name: cfg.name || lineageName(r) + 'ʻa',
    tiles,
    people: {}, households: {}, buildings: {}, disputes: {},
    nextId: 1,
    weather: { storm: 0, cold: 0 },
    memory: [],        // what the settlement has not forgotten
    government: cfg.government || null,
    offices: {}, lastOffices: {}, boatsDenied: [],
    quarters: [], closed: [],
    shrine: null,
    log: [],           // events for the current turn only
    chronicle: [],     // one entry per elapsed turn
    stats: [],
    omen: null,        // a god's standing suggestion, if any
    godSilence: 0
  };

  const id = () => state.nextId++;

  // seed households across the best sites
  const sites = tiles.map(t => t.id)
    .filter(i => siteValue(tiles, i) > 0)
    .sort((a, b) => siteValue(tiles, b) - siteValue(tiles, a));

  let placed = 0, target = Math.max(6, Math.round(cfg.population / 5.5));
  const used = [];
  for (const s of sites) {
    if (placed >= target) break;
    if (used.some(u => tileDist(u, s) < 1.7)) continue;
    used.push(s); placed++;
  }

  for (const site of used) {
    const hid = 'h' + id();
    const hh = {
      id: hid, name: lineageName(r), headId: null, members: [],
      buildingId: null, claims: [site], stores: 6 + r() * 6,
      standing: 20 + traitRoll(r) * 0.4, lodgedWith: null, founded: -ri(r, 8, 240)
    };
    tiles[site].owner = hid;
    // Most houses are long finished; a few are still coming up, at every
    // stage, so the settlement has work in flight from the first season.
    const bid = 'b' + id();
    const unfinished = placed > 3 && chance(r, 0.3);
    if (unfinished) {
      const m = 0.15 + r() * 0.8;
      state.buildings[bid] = {
        id: bid, tileId: site, householdId: hid, tenderId: null,
        startTurn: -Math.round(m * 55), maturity: m, capacity: ri(r, 4, 7),
        state: 'growing', stalled: 0, condition: 1
      };
      hh.buildingId = null;
    } else {
      const age = ri(r, 20, 260);
      // stages are reached by age, so a settlement founded with old houses
      // already has a few workshops in it — and no great house yet
      const grown = age - 68;                       // turns since it became a home
      const stage = grown >= 120 + 200 ? 3 : grown >= 120 ? 2 : 1;
      const spent = stage === 1 ? grown : stage === 2 ? grown - 120 : grown - 320;
      state.buildings[bid] = {
        id: bid, tileId: site, householdId: hid, tenderId: null,
        startTurn: -age, maturity: 1, capacity: ri(r, 4, 7),
        state: 'mature', stalled: 0, condition: 0.6 + r() * 0.4,
        stage, stageSince: -Math.max(0, spent), growth: clamp(spent / (STAGE_TURNS[stage] || 200), 0, 0.95),
        lastTended: -ri(r, 0, 12), role: null
      };
      hh.buildingId = bid;
    }

    // claim a working tile or two nearby
    for (const t of tiles) {
      if (hh.claims.length >= 3) break;
      if (t.owner || tileDist(t.id, site) > 2.6) continue;
      if (t.t === SHORE ? r() < 0.5 : t.t === MOOR ? r() < 0.35 : false) {
        t.owner = hid; hh.claims.push(t.id);
      }
    }
    state.households[hid] = hh;
  }

  // populate them
  const hids = Object.keys(state.households);
  let made = 0;
  while (made < cfg.population) {
    const hid = hids[made % hids.length];
    const hh = state.households[hid];
    const isFirst = hh.members.length === 0;
    const age = isFirst ? ri(r, 28, 62) : ri(r, 0, 58);
    const p = makePerson(state, r, hid, age);
    if (isFirst) { hh.headId = p.id; p.role = 'head'; }
    made++;
  }
  // a founding generation has to already hold the gift, or nothing is buildable
  const grown = Object.values(state.people).filter(p => ageOf(state, p) >= 22 && ageOf(state, p) <= 68);
  const want = Math.max(2, Math.round(grown.length * 0.055));
  for (let i = 0; i < want && grown.length; i++) pick(r, grown).tender = true;

  for (const hid of hids) assignWork(state, r, state.households[hid]);

  if (!state.government) state.government = pick(r, Object.keys(GOVERNMENTS));
  makeQuarters(state, r);

  // the communal stone: sited where the paths already converge
  const homeTiles = Object.values(state.buildings).map(b => b.tileId);
  const cx = homeTiles.reduce((a, t) => a + tileXY(t)[0], 0) / homeTiles.length;
  const cy = homeTiles.reduce((a, t) => a + tileXY(t)[1], 0) / homeTiles.length;
  const centre = tiles.filter(t => (t.t === SLOPE || t.t === CRAG) && !t.owner)
    .sort((a, b) => (Math.hypot(a.x - cx, a.y - cy)) - (Math.hypot(b.x - cx, b.y - cy)))[0];
  const god = pick(r, SHRINE_GODS);
  state.shrine = {
    tileId: centre ? centre.id : homeTiles[0], god: god.name, godOf: god.of,
    devotion: 45 + r() * 30, keeperId: null, keeperName: null, keeperSince: 0
  };

  recount(state);
  return state;
}

function makePerson(state, r, hid, age) {
  const pid = 'p' + state.nextId++;
  const hh = state.households[hid];
  const p = {
    id: pid, name: givenName(r), lineage: hh ? hh.name : lineageName(r),
    birthTurn: state.turn - age * 4, sex: chance(r, 0.5) ? 'm' : 'f',
    householdId: hid, alive: true, deathTurn: null, cause: null,
    traits: {
      ambition: traitRoll(r), grudge: traitRoll(r), piety: traitRoll(r),
      avarice: traitRoll(r), loyalty: traitRoll(r), courage: traitRoll(r)
    },
    tender: false,        // the gift is taught, not born — see sysTeaching
    // Teaching alone compounds: every tender makes more tenders and within
    // fifty years a third of the settlement has the gift. Aptitude is the
    // scarce half — it is rare, and it cannot be taught into someone.
    aptitude: chance(r, 0.13),
    learning: null,
    role: 'none', work: null, goal: null, goalAge: 0, matches: 0,
    standing: blankStanding(), ties: [], term: null, refusedTerm: 0,
    deeds: [], hunger: 0, parents: []
  };
  state.people[pid] = p;
  if (hh) hh.members.push(pid);
  return p;
}

function ageOf(state, p) { return Math.floor((state.turn - p.birthTurn) / 4); }

function assignWork(state, r, hh) {
  const tiles = state.tiles;
  const shore = hh.claims.filter(c => tiles[c].t === SHORE);
  const moor = hh.claims.filter(c => tiles[c].t === MOOR || tiles[c].t === SLOPE);
  for (const pid of hh.members) {
    const p = state.people[pid];
    if (!p.alive) continue;
    const a = ageOf(state, p);
    if (a < 12) { p.role = 'child'; p.work = null; continue; }
    if (a > 66) { p.role = 'elder'; p.work = null; continue; }
    if (p.tender) { p.work = 'tend'; if (p.role === 'none') p.role = 'tender'; continue; }
    if (shore.length && (!p.work || p.work === 'none')) p.work = 'fish';
    else if (moor.length) p.work = 'herd';
    else p.work = 'labour';
  }
}

/* --- events ---------------------------------------------------------------- */
/* Every event carries enough structure for the chronicle to write around it. */

function ev(state, kind, weight, text, extra = {}) {
  if (state.log.some(e => e.text === text)) return;   // never the same line twice in a season
  state.log.push(Object.assign({ kind, weight, text, turn: state.turn }, extra));
}

function nameOf(state, pid) {
  const p = state.people[pid];
  return p ? p.name + ' ' + p.lineage : 'someone';
}
