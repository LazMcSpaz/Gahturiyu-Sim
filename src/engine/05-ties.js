/* ===========================================================================
   Standing, held per faction — and ties, which are grudges with the sign
   allowed to go either way.

   Two things the rest of the settlement reads constantly. Both live here
   because everything downstream depends on them and nothing here depends on
   anything downstream.
   =========================================================================== */

/* --- standing ----------------------------------------------------------------
   A person holds a standing with each faction, −100 to +100, starting at 0.
   Decisions read the faction that is relevant to them: an arbiter weighs
   `government`, a neighbour weighs `quarter`, a master weighs `trade`. The
   composite exists to be displayed and must not be read by anything deciding
   anything — that would put the single number back.
   -------------------------------------------------------------------------- */

const FACTIONS = ['kin', 'quarter', 'government', 'guard', 'shrine', 'trade'];

const STANDING_DECAY = 0.985;   // per season toward 0 — half-life ~46 turns, 11½ years
const SLOW_DECAY = 0.99625;     // a quarter of that rate: what follows you for life

function blankStanding() {
  const o = {};
  for (const f of FACTIONS) o[f] = 0;
  return o;
}

function standingWith(p, faction) {
  return (p.standing && p.standing[faction]) || 0;
}

/* `slow` marks a mark that should not fade at the ordinary rate — a refused
   term is the case it exists for. */
function shiftStanding(p, faction, delta, slow) {
  if (!p.standing) p.standing = blankStanding();
  p.standing[faction] = clamp(p.standing[faction] + delta, -100, 100);
  if (slow) (p.slowStanding = p.slowStanding || {})[faction] = true;
}

function decayStanding(s) {
  for (const p of Object.values(s.people)) {
    if (!p.alive || !p.standing) continue;
    for (const f of FACTIONS) {
      const rate = (p.slowStanding && p.slowStanding[f]) ? SLOW_DECAY : STANDING_DECAY;
      p.standing[f] *= rate;
      if (Math.abs(p.standing[f]) < 0.05) p.standing[f] = 0;
    }
  }
}

/* A household's standing in a faction: its living adults, weighted by age.
   Age weighting is why a house of children carries less than a house of elders.

   The household's own deeds — what `remember` records — count toward `quarter`
   and nothing else. They are what the settlement at large holds against a
   house, which is the neighbours' view, not the guard's or the shrine's. The
   first version added them to all six, which made every faction read the same
   number and hid the only thing worth seeing. */
function houseStanding(s, hh, faction) {
  let sum = 0, weight = 0;
  for (const id of hh.members) {
    const p = s.people[id];
    if (!p || !p.alive) continue;
    const a = ageOf(s, p);
    if (a < 16) continue;
    const w = Math.min(a, 60) / 60;
    sum += standingWith(p, faction) * w;
    weight += w;
  }
  const own = weight ? sum / weight : 0;
  return faction === 'quarter' ? own + reputeOf(s, hh) : own;
}

/* Display only. Nothing that decides anything may call this. */
function compositeOf(s, hh) {
  return FACTIONS.reduce((a, f) => a + houseStanding(s, hh, f), 0) / FACTIONS.length;
}

/* --- ties --------------------------------------------------------------------
   One signed number from a person toward a household: −100 an enemy, +100
   someone they would put themselves out for. Zero is no tie and is not stored.

   Two hundred and fifty people is thirty thousand possible pairs, so ties are
   sparse and capped. An export has to stay a few kilobytes — rollback,
   branching and the save format all depend on it.
   -------------------------------------------------------------------------- */

const TIE_CAP = 12;            // per person; the weakest goes when a thirteenth forms
const TIE_DECAY = 0.99;        // per season toward 0 — half-life ~69 turns, 17 years
const TIE_CLOSE = 40;          // above this, close
const TIE_ENEMY = -40;         // below this, an enemy

function tieTo(p, householdId) {
  const t = (p.ties || []).find(x => x.target === householdId);
  return t ? t.value : 0;
}

/* The only way a tie changes. Returns the tie, or null if it was dropped. */
function shiftTie(s, p, householdId, delta, cause) {
  if (!householdId || !delta) return null;
  p.ties = p.ties || [];
  let t = p.ties.find(x => x.target === householdId);
  if (!t) {
    t = { target: householdId, value: 0, cause, turn: s.turn };
    p.ties.push(t);
  }
  const before = t.value;
  t.value = clamp(t.value + delta, -100, 100);
  // the reason on record is the one that moved it most, not the most recent
  if (Math.abs(delta) >= Math.abs(before) * 0.5 && cause) { t.cause = cause; t.turn = s.turn; }

  if (p.ties.length > TIE_CAP) {
    p.ties.sort((a, b) => Math.abs(b.value) - Math.abs(a.value));
    p.ties.length = TIE_CAP;
  }
  return t;
}

/* Kept for every caller that asks the old question. A positive tie holds
   nothing against anyone, so it answers zero. */
function holdsAgainst(s, person, householdId) {
  if (!person) return 0;
  return Math.max(0, -tieTo(person, householdId));
}

function bondWith(person, householdId) {
  if (!person) return 0;
  return Math.max(0, tieTo(person, householdId));
}

function decayTies(s) {
  for (const p of Object.values(s.people)) {
    if (!p.alive || !p.ties || !p.ties.length) continue;
    for (const t of p.ties) t.value *= TIE_DECAY;
    p.ties = p.ties.filter(t => Math.abs(t.value) >= 1
      && s.households[t.target] && !s.households[t.target].extinct);
  }
}

/* --- contact -----------------------------------------------------------------
   Relationships do not change because a system decided to change them. They
   change because people spend time in the same place. Each context samples a
   few pairs a season and drifts them by how well the two get on.

   Work crews and taverns arrive with later slices; household, quarter and the
   shrine term are enough to start.
   -------------------------------------------------------------------------- */

const CONTEXT_WEIGHT = { household: 1.5, service: 2.0, tavern: 1.2, quarter: 0.4 };
const TAVERN_REACH = 6;   // tiles' walk — a feud that shuts a path cuts it
const PAIRS_PER_CONTEXT = 3;

/* `traitRoll` is pow(r, 2.1) * 100, so traits are not centred on 50 — they
   average about 31. Every term below is measured against that, and the base is
   set so that time spent together brings a shade more people together than it
   drives apart: about 57% drift up, 37% down, the rest too slight to store.
   Written as constants because the first version assumed a midpoint of 50 and
   silently made every relationship in the settlement worse. */
const TRAIT_MID = 31;
const AFFINITY_BASE = 68;

function affinity(a, b) {
  return AFFINITY_BASE
    + (a.traits.loyalty + b.traits.loyalty - 2 * TRAIT_MID) / 3
    - Math.abs(a.traits.ambition - b.traits.ambition) / 3
    - (a.traits.grudge + b.traits.grudge - 2 * TRAIT_MID) / 5;
}

/* Two loyal people on the same boat drift together; two ambitious ones grind.
   Nothing is scripted — put people in a room often enough and the room does it. */
function drift(s, r, a, b, weight, cause) {
  if (!a || !b || a.id === b.id) return;
  if (a.householdId === b.householdId) return;   // ties point at other houses
  const d = clamp((affinity(a, b) - 50) / 10, -5, 5) * weight;
  if (Math.abs(d) < 0.2) return;
  shiftTie(s, a, b.householdId, d, cause);
  shiftTie(s, b, a.householdId, d, cause);
}

/* The first version sampled three random pairs out of a whole quarter. In a
   quarter of forty that is one chance in two hundred and fifty seasons that any
   particular pair meets, so nothing ever accumulated — while a single quarrel
   lands sixty points of ill will at once. Relationships have to deepen with the
   same people repeatedly to get anywhere, which is also how they work.

   So: each person, each season, sees somebody. Usually somebody they already
   know. */
const SEE_SOMEONE = 0.25;      // per person per context per season
const SEE_SOMEONE_KNOWN = 0.6; // and this often it is a face they know

function partnerFor(s, r, p, group) {
  if (group.length < 2) return null;
  const known = group.filter(q => q.id !== p.id && tieTo(p, q.householdId) !== 0);
  const pool = (known.length && chance(r, SEE_SOMEONE_KNOWN)) ? known : group;
  const q = pool[Math.floor(r() * pool.length)];
  return (q && q.id !== p.id) ? q : null;
}

function mingle(s, r, group, weight, cause) {
  for (const p of group) {
    if (!chance(r, SEE_SOMEONE * weight)) continue;
    const q = partnerFor(s, r, p, group);
    if (q) drift(s, r, p, q, weight, cause);
  }
}

function sysContact(s, r) {
  const adults = livingPeople(s).filter(p => ageOf(s, p) >= 12);

  // the quarter you live in — weak, constant, and the only one everyone has
  const byQuarter = {};
  for (const p of adults) {
    const q = quarterOf(s, homeTile(s, p));
    if (q) (byQuarter[q.id] = byQuarter[q.id] || []).push(p);
  }
  for (const group of Object.values(byQuarter)) {
    mingle(s, r, group, CONTEXT_WEIGHT.quarter, 'neighbours in the same quarter');
  }

  // households that share a roof, or lodge under one
  const byHouse = {};
  for (const p of adults) {
    const hh = s.households[p.householdId];
    if (!hh || hh.extinct) continue;
    const host = economicHost(s, hh);
    (byHouse[host.id] = byHouse[host.id] || []).push(p);
  }
  for (const group of Object.values(byHouse)) {
    mingle(s, r, group, CONTEXT_WEIGHT.household, 'lodged under one roof');
  }

  // the shrine term — the only context that crosses class, because everyone owes it
  mingle(s, r, adults.filter(p => p.term && !p.term.done),
         CONTEXT_WEIGHT.service, 'served their term together');

  // a tavern: the one place people choose each other rather than being thrown
  // together. Its reach is a walk, so a feud that shuts a path shrinks it.
  for (const b of tavernsOf(s)) {
    const near = adults.filter(p => walkDist(s, homeTile(s, p), b.tileId) <= TAVERN_REACH);
    mingle(s, r, near, CONTEXT_WEIGHT.tavern, 'drank at the same tavern');
  }
}

/* --- the term of service -----------------------------------------------------
   Everyone owes the shrine a year between 16 and 20. It is unpaid, and it is
   the one ladder wealth does not gate — which makes it the settlement's only
   leveller, and the only institution that puts a rich child and a poor one in
   the same year of work.

   Refusing is permitted and expensive. It should still be among the first
   things said about a household forty years later.
   -------------------------------------------------------------------------- */

const TERM_SEASONS = 4;

function sysService(s, r) {
  const sh = s.shrine;
  for (const p of Object.values(s.people)) {
    if (!p.alive) continue;
    const a = ageOf(s, p);

    // finishing
    if (p.term && !p.term.done) {
      if (s.turn - p.term.since >= TERM_SEASONS) {
        p.term.done = s.turn;
        shiftStanding(p, 'shrine', 8);
        shiftStanding(p, 'quarter', 3);
        // a few stay on, and those are who the shrine draws its keepers from
        if (chance(r, 0.06)) {
          p.shrineSworn = true;
          shiftStanding(p, 'shrine', 10);
          ev(s, 'shrine', 3, `${nameOf(s, p.id)} stayed on at the ${sh ? sh.god : 'communal'} stone after their term.`, { person: p.id });
        }
      }
      continue;
    }

    if (p.term || p.refusedTerm) continue;        // done, or refused, once only
    if (a < 16 || a > 20) continue;
    if (!chance(r, 0.34)) continue;               // spread across the four years

    // who refuses: those with no use for the stone and too much use for themselves
    if (p.traits.piety < 30 && p.traits.ambition > 70) {
      p.refusedTerm = s.turn;
      shiftStanding(p, 'shrine', -25, true);      // slow decay: this one follows you
      shiftStanding(p, 'quarter', -10);
      remember(s, p.householdId, -6, `had one of their own refuse the term at the stone`);
      ev(s, 'shrine', 5, `${nameOf(s, p.id)} refused the term at the ${sh ? sh.god : 'communal'} stone.`, { person: p.id, household: p.householdId });
      continue;
    }
    p.term = { since: s.turn, done: 0 };
  }
}
