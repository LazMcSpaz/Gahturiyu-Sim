/* ===========================================================================
   UPKEEP, DECAY AND BEAUTY

   Everything in this settlement is made of stone that took a century to grow,
   and stone that nobody keeps up comes back down. Condition falls every
   season and faster in a storm; a mason mends it for a stone and a timber;
   a house left too long falls back a stage, taking its workshop and whatever
   craft needed that workshop with it.

   This is where the weather finally earns its keep. Storms already existed
   and cost a season's grazing; now they break things, which gives the
   settlement a renewable supply of small jobs and a renewable supply of
   households who cannot afford them. Upkeep is the most honest class
   mechanic in the design, because it is not a rule about the poor — it is a
   bill that arrives every season whether or not you can pay it.

   And the other half: a carver makes a house worth looking at. Beauty is
   the one thing here that is purely display, which is exactly why a
   household with means wants it.
   =========================================================================== */

const WEAR = 0.02;              // per season, every standing building
const STORM_WEAR = 0.10;        // and a storm is worth five quiet seasons
/* An empty house has to outlast the wait for somebody to want it. At 0.012 on
   top of the ordinary wear it was past mending in six years, so a settlement
   losing families faster than that simply lost their stone, and the count of
   workshops fell from thirty to eleven over a century — which took the crafts
   that need a room with it. */
const EMPTY_WEAR = 0.008;       // a house with nobody in it goes faster still
const REPAIR = 0.25;            // what one visit from a mason puts back
const REPAIR_COST = { stone: 1, timber: 1 };
const MEND_BELOW = 0.75;        // a mason does not come out for a sound house
const DISREPAIR = 0.5;          // visibly failing, and the quarter notices
const RUINOUS = 0.2;            // past mending: the building loses a stage
const MASON_JOBS = 3;           // houses one mason gets round in a season
const GRAIN_FLOOR = 14;         // what a household keeps back before paying a mason in food
/* People patch their own roofs. Without this a quarter with no mason willing
   to walk to it simply came down — twenty to forty growths lost a century and
   three seeds in nine with no third growth at all after a hundred and twenty
   years. A household can keep its house standing and no better than standing;
   a mason is what takes it past squalid. */
/* A share of the gap, not a flat trickle. At 0.022 a season against wear of
   0.02 the net was 0.002, so a house knocked under by three storms took
   twenty-five years to climb back over the line it fell under — which meant
   every workshop in the settlement was gone by year twenty-four and the
   crafts that need a room with them. People patch the worst of it first. */
const SELF_MEND = 0.12;         // of the way back to what a household can manage alone
const SELF_CEILING = 0.45;
const RUINOUS_GRACE = 8;        // seasons under the line before a growth is lost
const ABANDONED = 40;           // and ten years with nobody mending it at all

const BEAUTY_WORK = 0.15;       // a season of a carver's attention
const BEAUTY_DECAY = 0.01;
const BEAUTY_STANDING = 15;     // at its finest, per year, to the household
const CARVE_COST = { timber: 1 };

function conditionOf(b) { return b.condition === undefined ? 1 : b.condition; }
function beautyOf(b) { return b.beauty || 0; }
function inDisrepair(b) { return b.state === 'mature' && conditionOf(b) < DISREPAIR; }

function buildingOf(s, hh) {
  const b = s.buildings[hh.buildingId];
  return b && b.state !== 'derelict' ? b : null;
}

/* --- wear --------------------------------------------------------------- */

function sysWear(s) {
  const storm = s.weather && s.weather.storm ? STORM_WEAR : 0;
  for (const b of Object.values(s.buildings)) {
    if (b.state !== 'mature') continue;          // stone still being grown is being worked
    const hh = s.households[b.householdId];
    const occupied = hh && !hh.extinct && hh.members.some(i => s.people[i] && s.people[i].alive);
    let c = conditionOf(b) - WEAR - storm - (occupied ? 0 : EMPTY_WEAR);
    if (occupied && c < SELF_CEILING) c = Math.min(SELF_CEILING, c + (SELF_CEILING - c) * SELF_MEND);
    b.condition = clamp(c, 0, 1);
    b.beauty = Math.max(0, beautyOf(b) - BEAUTY_DECAY);

    if (b.condition <= 0 && !occupied) {
      b.state = 'derelict';
      touchBuildings(s);
      ev(s, 'derelict', 4, `The ${hh ? hh.name + ' ' : ''}house at ${siteWord(s, b.tileId)} came down. There had been nobody in it for years.`,
         { building: b.id });
    }
  }
}

/* --- the mason ------------------------------------------------------------
   A mason chooses whose stone to mend the way a tender chooses whose to grow:
   the walk, kinship, what the house is known for, what they hold against it.
   The difference is that mending has a bill, and a household that cannot find
   a stone and a timber does not get mended however willing the mason is.
   -------------------------------------------------------------------------- */

function masonWill(s, m, hh, b) {
  const own = s.households[m.householdId];
  let v = 24;
  v -= walkDist(s, homeTile(s, m), b.tileId) * 2.2;   // a mason travels; it is a day, not a life
  if (own && own.id === hh.id) v += 60;                                          // your own roof first
  else if (own && firstSyllable(own.name) === firstSyllable(hh.name)) v += 24;
  /* A paid job. A tender grows a house for years and cares what the house is
     known for; a mason is there for a season and is paid at the door, so
     repute counts for less than it did. At 1.8 it closed a loop: disrepair
     costs a household quarter standing, standing is repute, and a house of
     low repute was a house no mason would walk to — seven of sixteen failing
     houses a season on a declining coast, and the decline fed itself. */
  v += reputeOf(s, hh) * 0.8;
  v -= holdsAgainst(s, m, hh.id) * 0.9;
  v += (1 - conditionOf(b)) * 30;                                                // the worse it is, the more it calls
  if (m.traits.avarice > 55) v += clamp(hh.stores - 6, -8, 12) * (m.traits.avarice / 90);
  if (m.traits.loyalty > 65) v += 8;
  return v;
}

/* The bill. A household pays for its own mending out of what it holds; a
   household with nothing is carried by the mason, who brings the stone and
   the timber and is owed for them. That second case is the whole class
   mechanic: the poor do not live in worse houses because a rule says so,
   they live in houses they are still paying for. And it feeds the ledger
   that 9d already reads, so a house mended on credit for thirty years is a
   house that can be taken.

   Nobody is carried in a settlement that has been burned often enough —
   `creditTight` is the same dial mercy moves. */
function payForRepair(s, r, hh, m) {
  if (Object.entries(REPAIR_COST).every(([g, n]) => hasGood(hh, g, n))) {
    for (const [g, n] of Object.entries(REPAIR_COST)) takeGood(hh, g, n);
    return 'paid';
  }
  const own = s.households[m.householdId];
  if (!own || own.id === hh.id || own.extinct) return null;
  if (!Object.entries(REPAIR_COST).every(([g, n]) => hasGood(own, g, n))) return null;
  const price = Object.entries(REPAIR_COST).reduce((a, [g, n]) => a + (VALUE[g] || 0.3) * n, 0) * 1.3;

  /* Paid in grain, which is the unit. This is the barter the design promised
     when it said a merciful settlement is thrown back on it: with credit
     tight, masons on one coast were refusing seventeen repairs a season for
     want of payment while the settlement held two hundred and fifty lots of
     stone, and half its houses fell in. A household with food to spare hands
     some over and owes nothing. */
  if (hh.stores - price >= GRAIN_FLOOR) {
    for (const [g, n] of Object.entries(REPAIR_COST)) takeGood(own, g, n);
    hh.stores -= price; own.stores += price;
    return 'grain';
  }
  if (chance(r, s.creditTight || 0)) return null;
  for (const [g, n] of Object.entries(REPAIR_COST)) takeGood(own, g, n);
  addDebt(s, hh.id, own.id, price, 'mending their house');
  return 'credit';
}

function sysRepairs(s, r) {
  const masons = livingPeople(s).filter(p => p.trade === 'mason'
    && ageOf(s, p) >= 16 && ageOf(s, p) <= 70 && !gaoled(p));
  const failing = Object.values(s.buildings).filter(b => b.state === 'mature'
    && conditionOf(b) < MEND_BELOW);
  s.unmended = 0;
  if (!failing.length) { s.shortMaterial = {}; return; }

  /* What the settlement is short of, for the hands that have not been given
     work yet. Held against what the failing stone would need to be sound
     again, not against some flat number. */
  const need = failing.length * 2;
  const total = g => Object.values(s.households)
    .reduce((a, h) => a + (h.extinct ? 0 : (goodsOf(h)[g] || 0)), 0);
  s.shortMaterial = { stone: total('stone') < need, timber: total('timber') < need };

  const done = new Set();
  for (const m of masons) {
    const ranked = failing
      .filter(b => !done.has(b.id) && s.households[b.householdId])
      .map(b => ({ b, hh: s.households[b.householdId] }))
      .filter(x => x.hh && !x.hh.extinct)
      .map(x => ({ x, v: masonWill(s, m, x.hh, x.b) + (r() - 0.5) * 10 }))
      .filter(x => x.v > 0)
      .sort((a, c) => c.v - a.v);

    /* Three jobs done, not three jobs looked at. Taking the top three of the
       list and moving on whether or not they could pay spent a mason's whole
       season on households with nothing, while the ones with a stone and a
       timber waiting on the step went unmended — twelve a season on one
       coast. A household that cannot pay is passed over for one that can. */
    let jobs = 0;
    for (const { x } of ranked) {
      if (jobs >= MASON_JOBS) break;
      const { b, hh } = x;
      const paid = payForRepair(s, r, hh, m);
      if (!paid) {
        hh.wantsMending = (hh.wantsMending || 0) + 1;
        s.unmended++;
        continue;
      }
      jobs++;
      hh.wantsMending = 0;
      b.condition = clamp(conditionOf(b) + REPAIR, 0, 1);
      b.lastMended = s.turn;
      done.add(b.id);
      shiftStanding(m, 'trade', 3);
      if (m.householdId !== hh.id) {
        const head = s.people[hh.headId];
        if (head) shiftTie(s, head, m.householdId, 8, `mended this house`);
      }
    }
  }

  /* A house nobody will come to, or nobody can afford to mend, is worth a
     line — once, and only when it has gone on long enough to be a fact about
     the household rather than a bad season. */
  for (const b of failing) {
    if (conditionOf(b) >= DISREPAIR) continue;
    const hh = s.households[b.householdId];
    if (!hh || hh.extinct) continue;
    const since = s.turn - (b.lastMended || b.startTurn || 0);
    if (since < 40 || s.turn - (b.toldRuin || -99) < 40) continue;
    b.toldRuin = s.turn;
    const broke = !Object.entries(REPAIR_COST).every(([g, n]) => hasGood(hh, g, n));
    ev(s, 'stall', 5, broke
      ? `The ${hh.name} house is falling in and the household has no stone or timber to mend it with.`
      : `No mason has been to the ${hh.name} house in ${Math.round(since / 4)} years. It is falling in.`,
      { building: b.id, household: hh.id });
  }
}

/* --- disrepair, and losing a growth ---------------------------------------
   A century of somebody's family comes back down one stage at a time. This is
   the counterweight the stages needed: growth is slow and one-way, so without
   it every settlement ends up all third growths and the word stops meaning
   anything.
   -------------------------------------------------------------------------- */

function sysDisrepair(s, r) {
  if (s.turn % 4 !== 0) return;
  for (const b of Object.values(s.buildings)) {
    if (b.state !== 'mature') continue;
    const hh = s.households[b.householdId];

    if (conditionOf(b) < DISREPAIR && hh && !hh.extinct) {
      for (const id of hh.members) {
        const p = s.people[id];
        if (p && p.alive) shiftStanding(p, 'quarter', -5);
      }
    }

    /* A growth is not lost the season the number dips. Eighteen occupied
       houses a century came down a stage on a single bad reading, which is
       not a settlement failing to keep its stone — it is a threshold with no
       patience in it. Two years under the line, which is long enough for the
       household to find a stone and a timber and a mason willing to come. */
    /* A landmark does not come down a stage. It can still be abandoned and
       fall in — a ruin of one is a fine thing for a chronicle to carry — but
       once a coast has raised one, it is not quietly demoted back to a great
       house because a mason was slow one decade. */
    if (stageOf(b) >= 4) continue;

    if (conditionOf(b) >= RUINOUS) { b.ruinousSince = 0; continue; }
    if (!b.ruinousSince) { b.ruinousSince = s.turn; continue; }
    if (s.turn - b.ruinousSince < RUINOUS_GRACE) continue;

    /* Poverty and abandonment are different stories and the design wants both.
       A household that cannot afford its mending lives in a squalid house —
       that is the disrepair penalty above, and it is common. The stone only
       comes down when nobody has touched it in ten years, which is a house
       being given up rather than a house being lost. Without this every third
       seed came out with no third growth at all after a century, thirty or
       forty growths lost to households who were simply poor. */
    if (s.turn - (b.lastMended || b.startTurn || 0) < ABANDONED) continue;
    b.ruinousSince = 0;

    if (stageOf(b) > 1) {
      const was = roleWord(b);
      b.stage -= 1;
      touchBuildings(s);
      b.growth = 0;
      b.stageSince = s.turn;
      b.condition = clamp(conditionOf(b) + 0.30, 0, 1);   // what is left of it stands
      if (stageOf(b) < 3) b.role = null;
      ev(s, 'stage', 7, `The ${hh ? hh.name + ' ' : ''}house was ${was} and is not any more. Too much of it came down to hold the growth.`,
         { building: b.id, household: hh ? hh.id : undefined });
      if (hh) remember(s, hh.id, -6, `let their house come down a growth`);
      continue;
    }

    // a stage-1 house past mending is simply a ruin, occupied or not
    if (chance(r, 0.25)) {
      b.state = 'derelict';
      touchBuildings(s);
      ev(s, 'derelict', 6, `The ${hh ? hh.name + ' ' : ''}house at ${siteWord(s, b.tileId)} is a ruin. What was left of it came down.`,
         { building: b.id });
      if (hh && !hh.extinct) hh.buildingId = null;
    }
  }
}

/* --- the carver ------------------------------------------------------------
   The one thing in the settlement that is for looking at. A carver works
   where they are welcome and where there is something worth carving, which
   in practice means a house that has already stood long enough to be worth
   the trouble — so beauty gathers where age and standing already are.
   -------------------------------------------------------------------------- */

function sysBeauty(s, r) {
  const carvers = livingPeople(s).filter(p => p.trade === 'carver'
    && ageOf(s, p) >= 16 && ageOf(s, p) <= 72 && !gaoled(p) && canPractise(s, p));
  if (!carvers.length) return;

  for (const c of carvers) {
    const ranked = Object.values(s.buildings)
      .filter(b => b.state === 'mature' && stageOf(b) >= 2 && beautyOf(b) < 0.97
        && conditionOf(b) >= DISREPAIR)               // nobody decorates a house that is falling in
      .map(b => ({ b, hh: s.households[b.householdId] }))
      .filter(x => x.hh && !x.hh.extinct && Object.entries(CARVE_COST).every(([g, n]) => hasGood(x.hh, g, n)))
      .map(x => ({
        x,
        v: 14 - walkDist(s, homeTile(s, c), x.b.tileId) * 2.6
           + reputeOf(s, x.hh) * 1.6
           + standingOf(s, x.hh) * 0.6
           + (x.hh.id === c.householdId ? 30 : 0)
           - holdsAgainst(s, c, x.hh.id) * 0.8
           + (r() - 0.5) * 12
      }))
      .filter(y => y.v > 0)
      .sort((a, b) => b.v - a.v);
    if (!ranked.length) continue;

    const { b, hh } = ranked[0].x;
    for (const [g, n] of Object.entries(CARVE_COST)) takeGood(hh, g, n);
    const was = beautyOf(b);
    b.beauty = Math.min(1, was + BEAUTY_WORK);
    shiftStanding(c, 'trade', 4);
    if (hh.id !== c.householdId) remember(s, c.householdId, 2, `carved the ${hh.name} house`);

    /* Once for the house, not once per crossing: beauty decays back under the
       line and gets carved over it again, which reported the same house as a
       marvel fifty times in a century. */
    if (was < 0.8 && b.beauty >= 0.8 && !b.toldBeauty) {
      b.toldBeauty = true;
      const q = quarterOf(s, b.tileId);
      ev(s, 'stage', 6, `${nameOf(s, c.id)} finished the carving on the ${hh.name} house. There is nothing else like it${q ? ` in ${q.name}` : ''}.`,
         { building: b.id, household: hh.id, person: c.id });
      remember(s, hh.id, 6, `keeps the finest house in the quarter`);
    }
  }

  // what being worth looking at is worth, by the year
  if (s.turn % 4 === 0) {
    for (const b of Object.values(s.buildings)) {
      if (b.state !== 'mature' || beautyOf(b) < 0.1) continue;
      const hh = s.households[b.householdId];
      if (!hh || hh.extinct) continue;
      for (const id of hh.members) {
        const p = s.people[id];
        if (p && p.alive) shiftStanding(p, 'quarter', beautyOf(b) * BEAUTY_STANDING / 4);
      }
    }
  }
}

/* --- somebody moves in -----------------------------------------------------
   A settlement does not let a sound house stand empty while a household has
   nowhere to live. Before this, a family that died out left its stone to rot
   for twenty years while the household next door spent fifteen growing a new
   one, and the settlement's stock of workshops fell by a third over a century
   — which quietly took the tier-three crafts with it, because a craft with no
   room is a craft given up.
   -------------------------------------------------------------------------- */

function vacant(s, b) {
  if (b.state !== 'mature') return false;
  const hh = s.households[b.householdId];
  return !hh || hh.extinct || !hh.members.some(i => s.people[i] && s.people[i].alive);
}

function sysTakeEmpty(s, r) {
  const empty = Object.values(s.buildings).filter(b => vacant(s, b) && conditionOf(b) > RUINOUS);
  if (!empty.length) return;

  const growing = new Set(Object.values(s.buildings)
    .filter(b => b.state === 'growing').map(b => b.householdId));
  const homeless = Object.values(s.households).filter(h => !h.extinct
    && !growing.has(h.id)
    && h.members.some(i => s.people[i] && s.people[i].alive)
    && !(s.buildings[h.buildingId] && s.buildings[h.buildingId].state === 'mature'
         && s.buildings[h.buildingId].householdId === h.id));
  if (!homeless.length) return;

  const taken = new Set();
  for (const hh of homeless) {
    const from = (hh.claims && hh.claims.length) ? hh.claims[0]
      : (hh.lodgedWith && s.households[hh.lodgedWith] ? homeTileOfHousehold(s, s.households[hh.lodgedWith]) : null);
    const ranked = empty
      .filter(b => !taken.has(b.id))
      .map(b => {
        const old = s.households[b.householdId];
        const kin = old && firstSyllable(old.name) === firstSyllable(hh.name);
        return { b, v: conditionOf(b) * 22 + stageOf(b) * 14 + (kin ? 25 : 0)
          - (from === null ? 0 : walkDist(s, from, b.tileId) * 1.8) + (r() - 0.5) * 8 };
      })
      .sort((a, c) => c.v - a.v);
    if (!ranked.length) break;

    const b = ranked[0].b;
    const old = s.households[b.householdId];
    taken.add(b.id);
    b.householdId = hh.id;
    touchBuildings(s);
    b.ruinousSince = 0;
    hh.buildingId = b.id;
    hh.lodgedWith = null;
    ev(s, 'settle', 5, `The ${hh.name} moved into the old ${old ? old.name + ' ' : ''}house at ${siteWord(s, b.tileId)}. It had been standing empty.`,
       { building: b.id, household: hh.id });
  }
}

function homeTileOfHousehold(s, hh) {
  const b = s.buildings[hh.buildingId];
  if (b) return b.tileId;
  return (hh.claims && hh.claims.length) ? hh.claims[0] : null;
}
