/* ===========================================================================
   GAHTURIYU — settlement engine
   Pure logic. No DOM access anywhere in this file.
   Contract:  advance(state, input, ) -> new state
   All state is plain JSON. Snapshots are structuredClone. Same state + same
   input + same seed must always produce the same next state.
   =========================================================================== */

const ENGINE_VERSION = '0.1.0';

/* The whole state is plain JSON, so a JSON round-trip is a valid deep copy.
   structuredClone is faster where it exists; older mobile browsers lack it. */
/* A hand-rolled deep copy for plain JSON, which is all the state ever is. It
   beats structuredClone on this shape by a wide margin: structuredClone pays
   a fixed cost per call and a serialisation on every object, and the state is
   thousands of small ones. */
function clone(o) {
  if (o === null || typeof o !== 'object') return o;
  if (Array.isArray(o)) {
    const a = new Array(o.length);
    for (let i = 0; i < o.length; i++) a[i] = clone(o[i]);
    return a;
  }
  const c = {};
  for (const k in o) c[k] = clone(o[k]);
  return c;
}

/* --- seeded randomness ---------------------------------------------------- */

function mulberry32(a) {
  return function () {
    a |= 0; a = a + 0x6D2B79F5 | 0;
    let t = Math.imul(a ^ a >>> 15, 1 | a);
    t = t + Math.imul(t ^ t >>> 7, 61 | t) ^ t;
    return ((t ^ t >>> 14) >>> 0) / 4294967296;
  };
}
// A turn's stream is derived from the run seed and the turn number, so turn N
// always rolls the same numbers regardless of how you arrived at it.
function turnRng(seed, turn) { return mulberry32((seed * 2654435761 + turn * 40503) | 0); }

function pick(r, arr) { return arr[Math.floor(r() * arr.length)]; }
function chance(r, p) { return r() < p; }
function ri(r, lo, hi) { return lo + Math.floor(r() * (hi - lo + 1)); }
function clamp(v, lo, hi) { return v < lo ? lo : v > hi ? hi : v; }
// most people are unremarkable; a few are not
function traitRoll(r) { return clamp(Math.floor(Math.pow(r(), 2.1) * 100), 0, 99); }

/* --- Gogìḍu, the Earth-tongue --------------------------------------------- */
/* Open (C)V syllables, no clusters, retroflex and uvular consonants, six
   vowels, penultimate stress. Names are built to the rules in languages.md. */

const ONSET = ['g','g','d','d','ḍ','ṭ','q','q','r','r','l','th','h','y','sh','t','m','n','ʻ',''];
const VOWEL = ['a','a','o','o','u','e','i','ì'];
const LINEAGE_TAIL = ['du','ro','ḍa','ṭo','qu','lu','ya','hìda','ʻo','ḍu'];

function syl(r) { return pick(r, ONSET) + pick(r, VOWEL); }

function givenName(r) {
  let n = '';
  const count = chance(r, 0.55) ? 3 : chance(r, 0.75) ? 2 : 4;
  for (let i = 0; i < count; i++) n += syl(r);
  return n.charAt(0).toUpperCase() + n.slice(1);
}

function lineageName(r) {
  let n = syl(r) + (chance(r, 0.5) ? syl(r) : '') + pick(r, LINEAGE_TAIL);
  return n.charAt(0).toUpperCase() + n.slice(1);
}

/* --- terrain --------------------------------------------------------------- */

const W = 22, H = 15;
const SEA = 0, SHORE = 1, SLOPE = 2, CRAG = 3, MOOR = 4;
const TERRAIN_NAME = ['open water', 'shore', 'hillside', 'crag', 'moor'];

function tileId(x, y) { return y * W + x; }
function tileXY(id) { return [id % W, Math.floor(id / W)]; }
/* Looked up, not computed. tileDist was eight per cent of a turn on its own,
   because every "who will walk to whom" decision in the engine asks it for
   every pair it considers. The table is a hundred thousand floats and is
   built once. */
let DIST = null;
function tileDist(a, b) {
  if (!DIST) {
    const n = W * H;
    DIST = new Float64Array(n * n);   // float32 rounding flipped a threshold at turn 61 of seed 991
    for (let i = 0; i < n; i++) {
      const ax = i % W, ay = Math.floor(i / W);
      for (let j = 0; j < n; j++) DIST[i * n + j] = Math.hypot(ax - (j % W), ay - Math.floor(j / W));
    }
  }
  return DIST[a * W * H + b];
}

/* --- season-scoped caches ---------------------------------------------------
   Half the engine is "for every person, for every household, for every
   building". The living are a third of the people on record and the set of
   households with a workshop changes a few times a century, so both are
   worked out once a season and thrown away before the state is kept. Every
   site that can change either answer mid-season clears it, so nothing reads
   a stale one — the run has to be bit-identical to the uncached engine, and
   test/run.js checks that it is. */
function livingPeople(s) {
  if (!s._alive) s._alive = Object.values(s.people).filter(p => p.alive);
  return s._alive;
}
function touchPeople(s) { s._alive = null; s._eh = null; }
function touchTrades(s) { s._eh = null; }
function touchBuildings(s) { s._ws = null; }
function dropCaches(s) { s._alive = null; s._ws = null; s._eh = null; s._scarce = null; s._rank = null; }

function buildTerrain(r) {
  // A coastline running roughly north-south down the left, land rising east.
  const wob = [];
  for (let y = 0; y < H; y++) wob.push(Math.sin(y * 0.75) * 1.6 + Math.sin(y * 0.31) * 2.2 + r() * 1.2);
  const tiles = [];
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      const coast = 4 + wob[y];
      const d = x - coast;                       // metres inland, roughly
      let t;
      if (d < -0.6) t = SEA;
      else if (d < 1.2) t = SHORE;
      else if (d < 7 + r() * 2) t = chance(r, 0.22) ? CRAG : SLOPE;
      else t = chance(r, 0.25) ? CRAG : MOOR;
      tiles.push({
        id: tileId(x, y), x, y, t,
        // stone quality governs how fast a home grows here
        stone: t === CRAG ? 0.75 + r() * 0.35 : t === SLOPE ? 0.55 + r() * 0.4 : 0.2 + r() * 0.3,
        // fishing depends on being near the water
        fish: t === SHORE ? 0.6 + r() * 0.5 : t === SEA ? 0 : 0,
        graze: t === MOOR ? 0.5 + r() * 0.5 : t === SLOPE ? 0.25 + r() * 0.3 : 0.05,
        /* Copses in the hollows, and not everywhere — which is what makes a
           wooded claim worth holding. Timber was in the goods list and wanted
           by a joiner from the start, but no tile ever carried any, so the
           settlement held exactly zero of it for a hundred and twenty years
           and nobody noticed until the stone needed mending. */
        wood: (t === SLOPE || t === MOOR) && chance(r, 0.35) ? 0.45 + r() * 0.45 : 0,
        owner: null
      });
    }
  }
  // shore tiles inherit fishing richness from adjacent deep water
  for (const tl of tiles) {
    if (tl.t !== SHORE) continue;
    let deep = 0;
    for (let dy = -1; dy <= 1; dy++) for (let dx = -2; dx <= 0; dx++) {
      const nx = tl.x + dx, ny = tl.y + dy;
      if (nx < 0 || ny < 0 || nx >= W || ny >= H) continue;
      if (tiles[tileId(nx, ny)].t === SEA) deep++;
    }
    tl.fish *= 0.5 + deep * 0.14;
  }
  return tiles;
}

// How good a tile is to grow a home on. Most hillside will not take a house:
// the stone has to be willing. This is the scarcity the whole settlement turns
// on, so the threshold is deliberately high.
const STONE_FLOOR = 0.735;

function siteValue(tiles, id) {
  const tl = tiles[id];
  if (tl.t !== SLOPE && tl.t !== CRAG) return 0;
  if (tl.stone < STONE_FLOOR) return 0;
  let v = tl.stone;
  const [x, y] = tileXY(id);
  for (let dy = -2; dy <= 2; dy++) for (let dx = -2; dx <= 2; dx++) {
    const nx = x + dx, ny = y + dy;
    if (nx < 0 || ny < 0 || nx >= W || ny >= H) continue;
    const n = tiles[tileId(nx, ny)];
    if (n.t === SHORE) v += 0.09;      // close to the boats
    if (n.t === MOOR) v += 0.04;       // close to the grazing
  }
  return v;
}

/* --- geography ---------------------------------------------------------------
   The map carried coordinates but almost nothing read them. Two things now do:
   the settlement is divided into named quarters, and a feud between neighbours
   physically closes the ground between them, which lengthens the walk for
   everyone else. Distance then decides who tends your stone, who marries in,
   who fosters your children, and who is close enough to notice you at all.
   -------------------------------------------------------------------------- */

function quarterOf(s, tid) {
  if (!s.quarters || !s.quarters.length) return null;
  const [x, y] = tileXY(tid);
  return s.quarters.reduce((best, q) =>
    Math.hypot(q.cx - x, q.cy - y) < Math.hypot(best.cx - x, best.cy - y) ? q : best);
}

function makeQuarters(state, r) {
  const homes = Object.values(state.buildings).map(b => b.tileId);
  if (!homes.length) { state.quarters = []; return; }
  const k = clamp(Math.round(homes.length / 7), 2, 5);
  // seed on the most separated homes, then relax
  const cents = [tileXY(homes[0])];
  while (cents.length < k) {
    let far = cents[0], fd = -1;
    for (const h of homes) {
      const [x, y] = tileXY(h);
      const d = Math.min(...cents.map(c => Math.hypot(c[0] - x, c[1] - y)));
      if (d > fd) { fd = d; far = [x, y]; }
    }
    cents.push(far);
  }
  let cur = cents;
  for (let pass = 0; pass < 8; pass++) {
    const buckets = cur.map(() => []);
    for (const h of homes) {
      const [x, y] = tileXY(h);
      let bi = 0, bd = Infinity;
      cur.forEach((c, i) => { const d = Math.hypot(c[0] - x, c[1] - y); if (d < bd) { bd = d; bi = i; } });
      buckets[bi].push([x, y]);
    }
    cur = cur.map((c, i) => buckets[i].length
      ? [buckets[i].reduce((a, p) => a + p[0], 0) / buckets[i].length,
         buckets[i].reduce((a, p) => a + p[1], 0) / buckets[i].length]
      : c);
  }
  state.quarters = cur.map((c) => c).sort((a, b) => a[1] - b[1]).map((c, rank) => ({
    id: 'q' + rank, cx: c[0], cy: c[1],
    name: lineageName(r) + pick(r, ['ʻo', 'du', 'ḍa'])
  }));
}

// Straight-line distance, plus the cost of going round a closed feud line.
function walkDist(s, a, b) {
  let d = tileDist(a, b);
  if (!s.closed || !s.closed.length) return d;
  const [ax, ay] = tileXY(a), [bx, by] = tileXY(b);
  const len2 = (bx - ax) * (bx - ax) + (by - ay) * (by - ay) || 1;
  for (const c of s.closed) {
    const t = clamp(((c.x - ax) * (bx - ax) + (c.y - ay) * (by - ay)) / len2, 0, 1);
    const px = ax + t * (bx - ax), py = ay + t * (by - ay);
    if (Math.hypot(px - c.x, py - c.y) < 1.4) d += 3.2;
  }
  return d;
}

function closePath(s, A, B) {
  const ta = (s.buildings[A.buildingId] || {}).tileId ?? A.claims[0];
  const tb = (s.buildings[B.buildingId] || {}).tileId ?? B.claims[0];
  if (ta === undefined || tb === undefined) return null;
  if (tileDist(ta, tb) > 4.5) return null;      // only neighbours can shut a path
  const [ax, ay] = tileXY(ta), [bx, by] = tileXY(tb);
  s.closed = s.closed || [];
  s.closed.push({ x: (ax + bx) / 2, y: (ay + by) / 2, a: A.id, b: B.id, turn: s.turn });
  return quarterOf(s, ta);
}

/* --- calendar -------------------------------------------------------------- */

const SEASONS = ['spring', 'summer', 'harvest', 'winter'];
function seasonOf(turn) { return SEASONS[turn % 4]; }
function yearOf(state, turn) { return state.startYear + Math.floor(turn / 4); }
