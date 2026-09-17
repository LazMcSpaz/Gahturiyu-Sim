/* --- world creation --------------------------------------------------------- */

const FOUNDING_PULL = 0.22;   // site value lost per tile from the founding nucleus

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

  /* A nucleus, not a scatter. On a map with room in it, taking the best sites
     anywhere founds a settlement strung along the whole coast with nothing
     between the houses — every walk a day and no quarter a neighbourhood. The
     founding gathers round the best stretch of the coast and leaves the rest
     for later. */
  const good = tiles.map(t => t.id).filter(i => siteValue(tiles, i) > 0);
  const shore = tiles.filter(t => t.t === SHORE).map(t => t.id);
  const toShore = i => Math.min(...shore.map(j => tileDist(i, j)));
  /* On the coast — this is a fishing settlement before it is anything — and
     on the best stretch of it: where good stone is thick on the ground and
     the water is close. Scored on nearby site value alone the nucleus landed
     twenty tiles inland on the crag, because that is where the stone is. */
  const nucleus = good.filter(i => toShore(i) <= 5).map(i => {
    const [, y] = tileXY(i);
    let near = 0;
    for (const j of good) if (tileDist(i, j) < 4) near += siteValue(tiles, j);
    return { i, v: near - Math.abs(y - H / 2) * 0.3 - toShore(i) * 0.4 };
  }).sort((a, b) => b.v - a.v)[0].i;
  const sites = good.slice()
    .sort((a, b) => (siteValue(tiles, b) - tileDist(nucleus, b) * FOUNDING_PULL)
                  - (siteValue(tiles, a) - tileDist(nucleus, a) * FOUNDING_PULL));

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
      buildingId: null, claims: [site], stores: 6 + r() * 6, goods: blankGoods(),
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
  seedWorkshops(state, r);
  seedTrades(state, r);
  for (const hid of hids) assignWork(state, r, state.households[hid]);

  if (!state.government) state.government = pick(r, Object.keys(GOVERNMENTS));
  state.govSince = 0;   // the form has held since founding; legitimacy reads it
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
  dropCaches(state);
  return state;
}

function makePerson(state, r, hid, age) {
  touchPeople(state);
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
    role: 'none', trade: null, goal: null, goalAge: 0, matches: 0, lastApprentice: -999,
    standing: blankStanding(), ties: [], term: null, refusedTerm: 0,
    deeds: [], hunger: 0, parents: []
  };
  state.people[pid] = p;
  if (hh) hh.members.push(pid);
  return p;
}

function ageOf(state, p) { return Math.floor((state.turn - p.birthTurn) / 4); }

/* A settlement is not founded blank. Somebody already knows every craft the
   place depends on, or there are no masters, nobody can be apprenticed, and the
   whole trade system never starts. The founding generation carries the crafts;
   everything after is taught. */
/* A settlement is founded with a few workshops in it, or the crafts that need
   a room cannot be seeded and — because a lost craft is never re-invented —
   the place has no smith, no joiner and no herbalist for as long as it stands.
   Stage is drawn from a building's age, and about three in ten come out old
   enough; one seed in ten draws badly and founds twenty-seven households
   around a single workshop. A quarter of the standing stone is old stone. */
function seedWorkshops(state, r) {
  const mature = Object.values(state.buildings).filter(b => b.state === 'mature');
  const want = Math.max(2, Math.round(mature.length / 4));
  let have = mature.filter(b => stageOf(b) >= 2).length;
  const ones = mature.filter(b => stageOf(b) === 1).sort((a, b) => a.startTurn - b.startTurn);
  for (const b of ones) {
    if (have >= want) break;
    const age = 190 + ri(r, 0, 60);
    const grown = age - 68;
    b.startTurn = -age;
    b.stage = 2;
    b.stageSince = -(grown - 120);
    b.growth = clamp((grown - 120) / STAGE_TURNS[2], 0, 0.95);
    have++;
  }
}

function seedTrades(state, r) {
  const grown = Object.values(state.people)
    .filter(p => p.alive && ageOf(state, p) >= 22 && ageOf(state, p) <= 68);
  const free = () => grown.filter(p => !p.trade);

  // the gift first — without it nothing is buildable and the settlement dies
  const wantTenders = Math.max(2, Math.round(grown.length * 0.055));
  for (let i = 0; i < wantTenders; i++) {
    const pool = free();
    if (!pool.length) break;
    setTrade(state, pick(r, pool), 'tender');
  }

  /* The crafts that need a room come next, before the ones that do not. They
     can only be seeded from the handful of households that already have a
     workshop, so if the open crafts take those people first the settlement is
     founded without a smith or a weaver and can never be taught one. Two
     apiece, not one: a single master who dies young takes the craft with them
     before they have ever had a child old enough to teach. */
  for (const k of TEACHABLE) {
    if (!TRADES[k].room) continue;
    for (let i = 0; i < 2; i++) {
      /* A room a household can reach, not only one it owns — the same rule
         the engine enforces every season afterwards. Owning was too narrow at
         founding: a seed with seven workshops between it could seat a weaver
         and a tailor and then ran out of people, so the settlement was founded
         with no smith, no joiner, no carver and no herbalist, and a craft that
         is not founded cannot be taught. */
      const pool = free().filter(p => workshopFor(state, p));
      if (!pool.length) break;
      setTrade(state, pick(r, pool), k);
    }
  }

  // and the crafts that need only a master take whoever is left
  for (const k of TEACHABLE) {
    if (k === 'tender' || TRADES[k].room) continue;
    const want = Math.max(1, Math.round(grown.length * 0.03));
    for (let i = 0; i < want; i++) {
      const pool = free();
      if (!pool.length) break;
      setTrade(state, pick(r, pool), k);
    }
  }
}

function assignWork(state, r, hh) {
  const tiles = state.tiles;
  const shore = hh.claims.filter(c => tiles[c].t === SHORE);
  const moor = hh.claims.filter(c => tiles[c].t === MOOR || tiles[c].t === SLOPE);
  for (const pid of hh.members) {
    const p = state.people[pid];
    if (!p.alive) continue;
    const a = ageOf(state, p);
    if (a < 12) { p.role = 'child'; continue; }
    if (a > 66) { p.role = 'elder'; continue; }
    // A trade is for life and is chosen in 37-trades.js, not reassigned here
    // because the household's ground changed. This only keeps roles current.
    if (p.trade && p.role === 'none') p.role = p.trade;
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
