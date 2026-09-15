/* ===========================================================================
   OFFICES
   A role is only worth wanting if it can be withheld. Each office below holds
   a chokepoint: whether rulings bind, who may quarrel openly, who eats, and
   who may build. They are held by named people, they fall vacant on death,
   and who fills them is decided by the form of government in play.
   =========================================================================== */

const GOVERNMENTS = {
  sole: {
    name: 'a sole ruler',
    describe: 'One house speaks for the settlement, with two advisors who are listened to as far as it suits.',
    seats: () => [
      { key: 'ruler',   title: 'Head of the Settlement', n: 1 },
      { key: 'advisor', title: 'Advisor',                n: 2 }
    ]
  },
  tribunal: {
    name: 'a tribunal',
    describe: 'Three sit together and nothing carries unless two of them agree.',
    seats: () => [{ key: 'tribune', title: 'Tribune', n: 3 }]
  },
  senate: {
    name: 'elected senators',
    describe: 'Each quarter of the settlement sends one voice, and the quarters do not always want the same thing.',
    seats: (s) => [{ key: 'senator', title: 'Senator', n: Math.max(2, Math.ceil(liveHouseholds(s).length / 15)) }]
  }
};

// Held under every form of government.
const STANDING_OFFICES = [
  { key: 'captain', title: 'Captain of the Guard', n: 1,
    wants: ['courage', 'loyalty'], minAge: 22, maxAge: 58 },
  { key: 'arbiter', title: 'Arbiter', n: 1,
    wants: ['loyalty', 'piety'], minAge: 30, maxAge: 76 },
  { key: 'boatholder', title: 'Boat-holder', n: 1,
    wants: ['avarice', 'ambition'], minAge: 24, maxAge: 66 }
];

function liveHouseholds(s) { return Object.values(s.households).filter(h => !h.extinct); }

function officeSpec(s) {
  const gov = GOVERNMENTS[s.government];
  return gov.seats(s).map(x => Object.assign({
    wants: x.key === 'ruler' || x.key === 'tribune' || x.key === 'senator'
      ? ['ambition', 'loyalty'] : ['loyalty', 'ambition'],
    minAge: 26, maxAge: 78
  }, x)).concat(STANDING_OFFICES);
}

function officeHolders(s, key) {
  return (s.offices[key] || []).map(id => s.people[id]).filter(p => p && p.alive);
}

function holdsOffice(s, pid) {
  for (const [key, ids] of Object.entries(s.offices)) if (ids.includes(pid)) return key;
  return null;
}

function officeTitle(s, key) {
  const spec = officeSpec(s).find(x => x.key === key);
  return spec ? spec.title : key;
}

/* --- filling a seat ---------------------------------------------------------- */

// How badly a person wants a seat, and how much anyone else would accept them
// in it. Standing is the house's; the rest is the person's.
function fitness(s, p, spec, quarter) {
  const hh = s.households[p.householdId];
  if (!hh) return -99;
  const a = ageOf(s, p);
  if (a < spec.minAge || a > spec.maxAge) return -99;
  if (holdsOffice(s, p.id)) return -99;
  let v = spec.wants.reduce((acc, t) => acc + p.traits[t], 0) / spec.wants.length;
  v += standingOf(s, hh) * 0.5;
  v += reputeOf(s, hh) * 1.4;
  if (quarter) {
    const q = quarterOf(s, homeTile(s, p));
    if (!q || q.id !== quarter.id) return -99;      // a senator must be of the quarter
  }
  return v;
}

function seatSomeone(s, r, spec, quarter) {
  const pool = Object.values(s.people).filter(p => p.alive)
    .map(p => ({ p, v: fitness(s, p, spec, quarter) + (r() - 0.5) * 10 }))
    .filter(x => x.v > -50)
    .sort((a, b) => b.v - a.v);
  if (!pool.length) return null;

  // Under a sole ruler the ruler picks, and picks people who suit them. Under a
  // tribunal the sitting tribunes do. Elected seats go to whoever the quarter
  // will actually back, which is nearer the top of the list but not always it.
  if (s.government === 'sole' && spec.key !== 'ruler') {
    const ruler = officeHolders(s, 'ruler')[0];
    if (ruler) {
      const rh = s.households[ruler.householdId];
      pool.sort((a, b) => {
        const bias = (x) => (x.p.householdId === ruler.householdId ? 40 : 0)
          + (rh && firstSyllable((s.households[x.p.householdId] || {}).name || '') === firstSyllable(rh.name) ? 18 : 0)
          - holdsAgainst(s, ruler, x.p.householdId) * 0.9
          + x.p.traits.loyalty * 0.35;
        return (b.v + bias(b)) - (a.v + bias(a));
      });
      return pool[0].p;
    }
  }
  const depth = s.government === 'senate' ? 4 : 2;
  return pool[Math.floor(Math.pow(r(), 1.7) * Math.min(depth, pool.length))].p;
}

/* --- the turn ---------------------------------------------------------------- */

function sysOffices(s, r) {
  if (!s.offices) s.offices = {};
  const spec = officeSpec(s);
  const known = new Set(spec.map(x => x.key));
  for (const key of Object.keys(s.offices)) if (!known.has(key)) delete s.offices[key];

  for (const sp of spec) {
    s.offices[sp.key] = (s.offices[sp.key] || []).filter(id => s.people[id] && s.people[id].alive);

    // report the ones who fell out
    const lost = (s.lastOffices || {})[sp.key] || [];
    for (const id of lost) {
      if (s.offices[sp.key].includes(id)) continue;
      const p = s.people[id];
      if (!p || p.alive) continue;
      const yrs = Math.max(1, Math.round((s.turn - (p.officeSince || s.turn)) / 4));
      ev(s, 'office', 6, `${nameOf(s, id)} died after ${yrs} years as ${sp.title.toLowerCase()}. The seat is open.`, { person: id });
      remember(s, p.householdId, 4, `held the ${sp.title.toLowerCase()} until they died`);
    }

    // senators are per quarter; everything else is per settlement
    if (sp.key === 'senator') {
      const quarters = s.quarters.slice(0, sp.n);
      for (const q of quarters) {
        const sitting = officeHolders(s, 'senator').filter(p => {
          const pq = quarterOf(s, homeTile(s, p));
          return pq && pq.id === q.id;
        });
        if (sitting.length) continue;
        const pick_ = seatSomeone(s, r, sp, q);
        if (!pick_) continue;
        s.offices.senator.push(pick_.id);
        pick_.officeSince = s.turn;
        remember(s, pick_.householdId, 3, `elected to speak for ${q.name}`);
        ev(s, 'office', 5, `${q.name} elected ${nameOf(s, pick_.id)} to speak for it.`, { person: pick_.id });
      }
      continue;
    }

    while (s.offices[sp.key].length < sp.n) {
      const chosen = seatSomeone(s, r, sp, null);
      if (!chosen) break;
      s.offices[sp.key].push(chosen.id);
      chosen.officeSince = s.turn;
      const hh = s.households[chosen.householdId];
      const line = {
        ruler: `${nameOf(s, chosen.id)} of the ${hh.name} is now sole ruler of the settlement.`,
        advisor: `${nameOf(s, chosen.id)} of the ${hh.name} was made an advisor to the ruler.`,
        tribune: `${nameOf(s, chosen.id)} took the empty seat on the tribunal.`,
        captain: `${nameOf(s, chosen.id)}, ${ageOf(s, chosen)}, was made captain of the guard.`,
        arbiter: `${nameOf(s, chosen.id)} was made arbiter. Quarrels go to them first.`,
        boatholder: `${nameOf(s, chosen.id)} is now boat-holder. They decide which households may fish.`
      }[sp.key] || `${nameOf(s, chosen.id)} took the seat of ${sp.title.toLowerCase()}.`;
      ev(s, 'office', 5, line, { person: chosen.id });
      remember(s, chosen.householdId, 3, `took the seat of ${sp.title.toLowerCase()}`);
    }
  }

  // a seat held is regard earned, every year, in the faction that grants it
  if (s.turn % 4 === 0) {
    for (const sp of officeSpec(s)) {
      for (const p of officeHolders(s, sp.key)) {
        shiftStanding(p, 'government', sp.key === 'ruler' ? 6 : 4);
        if (sp.key === 'captain') shiftStanding(p, 'guard', 6);
      }
    }
  }

  s.lastOffices = {};
  for (const [k, v] of Object.entries(s.offices)) s.lastOffices[k] = v.slice();

  sysBoats(s, r);
  sysCaptain(s, r);
}

/* --- the boat-holder ---------------------------------------------------------
   Most households fish from a hull they do not own. The holder decides who is
   on the water this season, which is the same as deciding who eats.
   -------------------------------------------------------------------------- */

function sysBoats(s, r) {
  const holder = officeHolders(s, 'boatholder')[0];
  s.boatsDenied = [];
  if (!holder) return;
  const own = s.households[holder.householdId];
  if (!own) return;

  const fishers = liveHouseholds(s).filter(h => h.id !== own.id
    && h.claims.some(c => s.tiles[c].fish > 0.25));
  for (const h of fishers) {
    const spite = holdsAgainst(s, holder, h.id);
    const owed = reputeOf(s, h);
    const price = holder.traits.avarice > 58 && h.stores < 4 ? 14 : 0;   // no credit for the poor
    if (spite > 30 || owed < -8 || price > 0) {
      if (!chance(r, 0.5)) continue;
      s.boatsDenied.push(h.id);
      h.noBoat = (h.noBoat || 0) + 1;
      if (h.noBoat === 1 || h.noBoat % 8 === 0) {
        ev(s, 'office', 6, spite > 30
          ? `${nameOf(s, holder.id)} refused the ${h.name} a boat. They hold a grudge against that household.`
          : `${nameOf(s, holder.id)} priced the ${h.name} off the water. The household cannot afford a hull.`,
          { household: h.id });
        remember(s, own.id, -5, `kept the ${h.name} off the water`);
        const hh = s.people[h.headId];
        if (hh && hh.traits.grudge > 30)
          shiftTie(s, hh, own.id, -60, `kept this household off the water`);
      }
    } else if (h.noBoat) {
      h.noBoat = 0;
    }
  }
}

/* --- the captain -------------------------------------------------------------
   Quarrels that would otherwise be aired in the open get sat on. Whether that
   is order or suppression depends on who the captain's house owes.
   -------------------------------------------------------------------------- */

function sysCaptain(s, r) {
  const cap = officeHolders(s, 'captain')[0];
  if (!cap) return;
  const own = s.households[cap.householdId];
  const open = Object.values(s.disputes).filter(d => d.open && d.heat > 45);
  if (!open.length || !chance(r, 0.3)) return;
  const d = pick(r, open);
  const A = s.households[d.a], B = s.households[d.b];
  if (!A || !B) return;

  // a captain leans toward the house they are of, or owe
  const favourA = own && (own.id === A.id || holdsAgainst(s, cap, B.id) > 20);
  const favourB = own && (own.id === B.id || holdsAgainst(s, cap, A.id) > 20);

  if (favourA || favourB) {
    const put = favourA ? B : A, kept = favourA ? A : B;
    d.heat += 10;
    remember(s, own.id, -5, `used the guard against the ${put.name} in a quarrel their own household was in`);
    const ph = s.people[put.headId];
    if (ph) shiftTie(s, ph, own.id, -70, `set the guard on this household over a private quarrel`);
    ev(s, 'office', 6, `${nameOf(s, cap.id)} set the guard on the ${put.name} and not the ${kept.name}. The captain's own household is on the ${kept.name}'s side.`, { person: cap.id });
  } else {
    d.heat = Math.max(0, d.heat - 22);
    ev(s, 'office', 4, `${nameOf(s, cap.id)} broke up the quarrel between the ${A.name} and the ${B.name}. It cooled.`, { person: cap.id });
  }
}
