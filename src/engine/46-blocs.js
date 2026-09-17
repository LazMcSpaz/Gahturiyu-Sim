/* ===========================================================================
   THE LEVY, AND ACTING AS ONE

   Two mechanisms, and the first exists mostly to feed the second.

   A settlement whose richest households are merely *expected* to give has no
   politics in it. The ruling body can compel — take a share of a full house's
   stores when people are going short — and that is deliberately unpleasant
   from both ends: it fills the store and it makes enemies, and the office
   holder who orders it pays for it later. That cost is what makes an office
   worth having.

   And it is the grievance a neighbourhood can share. A quarter that acts as
   one is the only place in this design where the unskilled have weight: they
   never rise one at a time, but they count when a whole quarter stops
   working. A trade organising is the same act against a faction that is a
   monopoly rather than a neighbourhood, which makes it the more dangerous of
   the two — four ropers agreeing costs the settlement its rigging, and the
   ropers can still eat.

   Whether either works is a question of means, not a roll. A quarter can only
   refuse what it can afford to refuse.
   =========================================================================== */

const LEVY_FLOOR = 10;          // what a household keeps whatever happens
const LEVY_SHARE = 0.25;        // of everything above it
const LEVY_NEED = 3;            // households short before compulsion is on the table
const LEVY_EVERY = 12;          // and the same house is not squeezed every season

const BLOC_COHESION = 25;       // mean tie among the pairs that know each other
const BLOC_KNOWN = 0.25;        // and this much of the bloc has to know each other at all
/* Eight per cent a season with a ten-year memory gave eleven of these in a
   century, which makes a thing the design wants to be remarkable into a
   feature of ordinary life. A bloc that has acted does not act again for a
   generation, and the settlement as a whole is quiet for ten years after any
   of them — a quarter that loses one is a worse place to live for that long. */
const BLOC_CHANCE = 0.05;
const BLOC_QUIET = 100;         // a bloc that has just acted does not act again
const BLOC_SETTLED = 40;        // nor does any other, for a while
const GRIEVANCE_MEMORY = 24;    // how long a levy is felt as one

/* --- the levy ------------------------------------------------------------- */

function rulingSeats(s) {
  const gov = GOVERNMENTS[s.government];
  if (!gov) return [];
  return gov.seats(s).flatMap(sp => officeHolders(s, sp.key));
}

function topQuartile(s) {
  const houses = Object.values(s.households).filter(h => !h.extinct && !h.lodgedWith);
  const ranked = houses.slice().sort((a, b) => compositeOf(s, b) - compositeOf(s, a));
  return ranked.slice(0, Math.max(1, Math.round(ranked.length / 4)));
}

function sysLevy(s, r) {
  const short = Object.values(s.households).filter(h => !h.extinct && !h.lodgedWith
    && (h.shortSeasons || 0) >= 2).length;
  if (short < LEVY_NEED) return;

  const seats = rulingSeats(s);
  if (!seats.length) return;
  const orderer = seats[0];
  const ruling = s.households[orderer.householdId];
  if (!ruling) return;

  /* Under a tribunal or a senate it takes a majority, and a body that cannot
     agree does nothing — which is the difference between the forms showing up
     somewhere other than the gallows. */
  if (seats.length > 1) {
    const willing = seats.filter(p => p.traits.ambition + p.traits.loyalty > 62).length;
    if (willing * 2 <= seats.length) return;
  }

  const fat = topQuartile(s).filter(h => h.stores > LEVY_FLOOR + 4
    && s.turn - (h.leviedAt || -99) > LEVY_EVERY
    && h.id !== ruling.id);
  if (!fat.length || !chance(r, 0.18)) return;

  // the ruling household squeezes the house it likes least of the ones it can
  const target = fat.map(h => ({ h, v: holdsAgainst(s, orderer, h.id) * 1.4
    + compositeOf(s, h) * 0.4 + (r() - 0.5) * 20 }))
    .sort((a, b) => b.v - a.v)[0].h;
  const head = s.people[target.headId];
  const take = (target.stores - LEVY_FLOOR) * LEVY_SHARE;
  target.leviedAt = s.turn;

  /* Refusing is not a roll on its own: it is a character with the standing to
     survive it. A house nobody thinks much of cannot tell the tribunal no. */
  const nerve = (head ? head.traits.avarice * 0.5 + (100 - head.traits.loyalty) * 0.4
    + head.traits.courage * 0.3 : 0) + standingOf(s, target) * 0.5;
  const refuses = head && nerve > 78 && chance(r, 0.6);

  if (!refuses) {
    target.stores -= take;
    commonStore(s).food += take;
    commonStore(s).taken += take;
    for (const id of target.members) {
      const p = s.people[id];
      if (!p || !p.alive) continue;
      shiftStanding(p, 'government', 6);
      shiftStanding(p, 'quarter', 4);
    }
    if (head) shiftTie(s, head, ruling.id, -30, `took a share of this household's stores`);
    ev(s, 'levy', 6, `${nameOf(s, orderer.id)} levied ${Math.round(take)} from the ${target.name} for the common store. ${short} households are short.`,
       { household: target.id, person: orderer.id });
    remember(s, target.id, 3, `gave up a share of their stores when the settlement was short`);
  } else {
    for (const id of target.members) {
      const p = s.people[id];
      if (!p || !p.alive) continue;
      shiftStanding(p, 'government', -25);
      shiftStanding(p, 'quarter', -15);
    }
    openDispute(s, r, ruling, target, 'a levy refused');
    ev(s, 'levy', 7, `The ${target.name} refused the levy. ${nameOf(s, orderer.id)} ordered it and they would not give.`,
       { household: target.id, person: orderer.id });
    remember(s, target.id, -7, `refused a levy while the settlement went short`);
  }

  // whichever way it went, the quarter it happened in felt it
  const b = buildingOf(s, target);
  const q = b ? quarterOf(s, b.tileId) : null;
  if (q) { s.grievance = s.grievance || {}; s.grievance[q.id] = s.turn; }
  for (const id of target.members) {
    const p = s.people[id];
    if (p && p.alive && p.trade) {
      s.grievance = s.grievance || {};
      s.grievance['t:' + p.trade] = s.turn;
    }
  }
}

/* --- acting as one --------------------------------------------------------- */

/* The regard inside a group, measured over the pairs that have any regard at
   all. Averaging over every pair was the wrong number and made the whole
   mechanism dead: nobody holds a tie to more than twelve households, so most
   pairs in a quarter of eight have never had anything to do with each other,
   and those zeroes drag the mean to nine. In sixteen hundred quarter-seasons
   the cohesion test passed thirty-one times and never once while the quarter
   had a grievance. A neighbourhood is not close because everybody knows
   everybody. It is close when the people who do deal with each other get on,
   and when enough of them deal with each other to speak for the place — which
   is the second number here.  */
function cohesionOf(s, houses) {
  if (houses.length < 2) return { mean: -99, share: 0 };
  let sum = 0, known = 0, pairs = 0;
  for (const h of houses) {
    const head = s.people[h.headId];
    if (!head || !head.alive) continue;
    for (const o of houses) {
      if (o.id === h.id) continue;
      pairs++;
      const t = tieTo(head, o.id);
      if (t !== 0) { sum += t; known++; }
    }
  }
  return { mean: known ? sum / known : -99, share: pairs ? known / pairs : 0 };
}

function holdsTogether(s, houses) {
  const c = cohesionOf(s, houses);
  return c.mean >= BLOC_COHESION && c.share >= BLOC_KNOWN;
}

/* How badly the settlement needs what the bloc is withholding. It cannot be
   read off the goods alone: a tender makes nothing at all and the settlement
   cannot do without one for a decade, while a carver's work is the first thing
   anybody can live without. */
const VITAL = ['tender', 'mason', 'roper', 'tailor', 'herbalist', 'weaver', 'carrier'];

function quarterHouses(s, q) {
  return Object.values(s.households).filter(h => {
    if (h.extinct || h.lodgedWith) return false;
    const b = buildingOf(s, h);
    if (!b) return false;
    const hq = quarterOf(s, b.tileId);
    return hq && hq.id === q.id;
  });
}

function tradeHouses(s, trade) {
  const ids = new Set();
  for (const p of Object.values(s.people)) {
    if (p.alive && p.trade === trade && ageOf(s, p) >= 16) ids.add(p.householdId);
  }
  return [...ids].map(id => s.households[id]).filter(h => h && !h.extinct && !h.lodgedWith);
}

/* Can they outlast the settlement's need of them? Two numbers: how long the
   bloc can feed itself while it holds out, and how badly the rest of the
   settlement depends on what it is withholding. A poor quarter that stops
   working folds. A quarter holding the only rope on the coast does not. */
function blocWins(s, houses, hold) {
  const heads = houses.reduce((a, h) => a + h.members.filter(i => s.people[i] && s.people[i].alive).length, 0);
  const living = Object.values(s.people).filter(p => p.alive).length || 1;
  const stores = houses.reduce((a, h) => a + h.stores, 0) / Math.max(1, houses.length);
  const dependence = clamp(hold !== null ? hold : heads / living, 0, 1);
  return stores >= 8 && dependence > 0.22;
}

/* A quarter's grievance is not only a levy. The design names three and only
   one of them was wired, which left the quarter branch dead: in three runs of
   a hundred and twenty years no neighbourhood ever acted, and the claim that
   this is where the unskilled finally have weight never once came true. The
   other two are a feud line cutting the quarter's paths, and the quarter's
   standing sliding against the rest of the settlement. */
function quarterGrievance(s, q, houses) {
  if (s.turn - ((s.grievance || {})[q.id] || -99) < GRIEVANCE_MEMORY) return true;

  // a feud line drawn across the quarter is everyone's business in it
  if ((s.closed || []).some(c => Math.hypot(c.x - q.cx, c.y - q.cy) < 3.2)) return true;

  // and sliding against the others: measured against where they were, not zero
  const mean = houses.reduce((a, h) => a + compositeOf(s, h), 0) / houses.length;
  s.quarterWas = s.quarterWas || {};
  const was = s.quarterWas[q.id];
  if (s.turn % 20 === 0) s.quarterWas[q.id] = mean;
  return was !== undefined && mean < was - 8;
}

function sysBlocs(s, r) {
  s.blocs = s.blocs || {};
  const candidates = [];

  for (const q of (s.quarters || [])) {
    const houses = quarterHouses(s, q);
    if (houses.length < 3) continue;
    candidates.push({ key: q.id, kind: 'quarter', label: q.name, houses, hold: null,
      faction: 'quarter', sore: quarterGrievance(s, q, houses) });
  }

  for (const k of TEACHABLE) {
    const houses = tradeHouses(s, k);
    if (houses.length < 3) continue;
    /* A trade's dependence is what share of that craft the bloc holds — which
       for a trade is nearly all of it, and is exactly why a trade organising
       bites harder than a neighbourhood doing the same thing. */
    candidates.push({ key: 't:' + k, kind: 'trade', label: `the ${k}s`, houses,
      hold: VITAL.includes(k) ? 0.85 : 0.14, faction: 'trade', trade: k,
      sore: s.turn - ((s.grievance || {})['t:' + k] || -99) < GRIEVANCE_MEMORY });
  }

  if (s.turn - (s.blocs.any || -99) < BLOC_SETTLED) return;

  for (const c of candidates) {
    if (s.turn - (s.blocs[c.key] || -99) < BLOC_QUIET) continue;
    if (!c.sore) continue;
    if (!holdsTogether(s, c.houses)) continue;
    if (!chance(r, BLOC_CHANCE)) continue;

    s.blocs[c.key] = s.turn;
    s.blocs.any = s.turn;
    const won = blocWins(s, c.houses, c.hold);
    const what = c.kind === 'trade'
      ? `will not work for anyone outside their own houses`
      : `stopped working and will not be moved`;

    for (const h of c.houses) {
      for (const id of h.members) {
        const p = s.people[id];
        if (!p || !p.alive) continue;
        if (won) shiftStanding(p, c.faction, 15);
        else shiftStanding(p, 'government', -20);
      }
      // and the bloc's own regard for each other moves with the outcome
      const head = s.people[h.headId];
      if (!head) continue;
      for (const o of c.houses) {
        if (o.id === h.id) continue;
        shiftTie(s, head, o.id, won ? 10 : -15,
          won ? `stood with this household and it worked` : `stood with this household and it failed`);
      }
    }

    ev(s, 'bloc', 8, won
      ? `${c.label} ${what}. They had the stores to outlast it and the settlement gave way.`
      : `${c.label} ${what}. It did not hold — they could not afford it and went back to work.`,
      { quarter: c.kind === 'quarter' ? c.key : undefined });

    if (!won) {
      const cap = officeHolders(s, 'captain')[0];
      if (cap && chance(r, 0.5)) {
        ev(s, 'bloc', 6, `${nameOf(s, cap.id)} set the guard on ${c.label} until they did.`, { person: cap.id });
        for (const h of c.houses) {
          const head = s.people[h.headId];
          if (head) shiftTie(s, head, cap.householdId, -50, `set the guard on us`);
        }
      }
    }
    break;     // one settlement, one of these at a time
  }
}

/* --- the quarters themselves -----------------------------------------------
   The centres are chosen once, at founding, and never redrawn, because the
   names are the valuable part. Two things follow and the design wants both:
   a quarter can empty and keep its name, and growth in a new direction has
   nowhere to belong. The first is a fact the chronicle should be allowed to
   say plainly. The second is a genuine flaw, and the fix is not to move the
   old centres but to let a new one form — so a settlement gains a quarter and
   keeps its ghosts.
   -------------------------------------------------------------------------- */

/* Measured, not guessed: on a coast twenty-two tiles wide with four founding
   centres, the farthest lived-in house in a century sits about five and a half
   tiles from the nearest of them, so a threshold of five and a half meant no
   settlement could ever gain a quarter. Four and a bit is the far edge of this
   map and a real walk. */
const FAR_FROM_ANY = 4.2;       // tiles from every existing centre
const NEW_QUARTER_HOUSES = 4;   // standing, lived-in houses out there together

function sysQuarters(s, r) {
  if (s.turn % 8 !== 0 || !s.quarters || !s.quarters.length) return;

  for (const q of s.quarters) {
    const houses = quarterHouses(s, q);
    if (houses.length) { q.hollow = false; continue; }
    if (q.hollow) continue;
    q.hollow = true;
    ev(s, 'settle', 6, `Nobody lives at ${q.name} any more. The name stays on the hillside.`, {});
  }

  const lived = Object.values(s.buildings).filter(b => b.state === 'mature' && !vacant(s, b));
  const far = lived.map(b => {
    const [x, y] = tileXY(b.tileId);
    return { b, x, y, d: Math.min(...s.quarters.map(q => Math.hypot(q.cx - x, q.cy - y))) };
  }).filter(o => o.d > FAR_FROM_ANY);
  if (far.length < NEW_QUARTER_HOUSES) return;

  // the houses out there have to be near each other, not merely far from here
  const seed = far.sort((a, b) => b.d - a.d)[0];
  const group = far.filter(o => Math.hypot(o.x - seed.x, o.y - seed.y) < 3.4);
  if (group.length < NEW_QUARTER_HOUSES) return;

  const cx = group.reduce((a, o) => a + o.x, 0) / group.length;
  const cy = group.reduce((a, o) => a + o.y, 0) / group.length;
  const name = lineageName(r) + pick(r, ['ʻo', 'du', 'ḍa']);
  s.quarters.push({ id: 'q' + s.nextId++, cx, cy, name });
  ev(s, 'settle', 7, `The houses out past ${nearestQuarterName(s, cx, cy)} are called ${name} now. There are ${group.length} of them and nobody there thinks of themselves as living anywhere else.`, {});
}

function nearestQuarterName(s, cx, cy) {
  let best = null, bd = Infinity;
  for (const q of s.quarters) {
    const d = Math.hypot(q.cx - cx, q.cy - cy);
    if (d < bd) { bd = d; best = q; }
  }
  return best ? best.name : 'the shrine';
}
