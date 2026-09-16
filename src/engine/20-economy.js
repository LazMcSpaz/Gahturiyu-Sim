/* food ---------------------------------------------------------------------- */

/* A household that has split off but has no ground of its own still works the
   ground of the house it lodges under, and eats from the same store. Treating
   it as an independent economy the moment it has a name is what starved the
   settlement in the first build. */
function economicHost(s, hh) {
  let cur = hh, guard = 0;
  while (cur.lodgedWith && s.households[cur.lodgedWith] && !s.households[cur.lodgedWith].extinct && guard++ < 8) {
    cur = s.households[cur.lodgedWith];
  }
  return cur;
}

function sysFood(s, r) {
  const w = s.weather;
  const seasonMul = { spring: 1.0, summer: 1.3, harvest: 1.25, winter: 0.65 }[seasonOf(s.turn)];

  // group households into economic units
  const units = {};
  for (const hh of Object.values(s.households)) {
    if (hh.extinct) continue;
    const host = economicHost(s, hh);
    (units[host.id] = units[host.id] || { host, members: [] }).members.push(hh);
  }

  for (const u of Object.values(units)) {
    const claims = u.members.flatMap(h => h.claims);
    const bestFish = Math.max(0, ...claims.map(c => s.tiles[c].fish), 0);
    const bestGraze = Math.max(0, ...claims.map(c => s.tiles[c].graze), 0);
    // more hands on the same ground help, but with diminishing return
    let hands = 0, yield_ = 0, mouths = 0;
    const people = u.members.flatMap(h => h.members).map(p => s.people[p]).filter(p => p && p.alive);

    for (const p of people) {
      const a = ageOf(s, p);
      mouths += a < 12 ? 0.5 : a > 66 ? 0.75 : 1;
      if (a < 12 || a > 70) continue;
      // Everyone puts some of the day into food; a craft leaves less of it,
      // and buys the difference back with what the craft makes.
      const share = foodShare(p.trade);
      if (share <= 0) continue;
      hands += share;
      const crowd = Math.pow(Math.max(1, hands), -0.22);    // the fifth boat is worth less than the first
      if (bestFish > 0.15 && (p.trade === 'boathand' || !bestGraze)) {
        yield_ += share * bestFish * 4.2 * seasonMul * crowd * (w.storm ? 0.3 : 1) * (w.bounty ? 2.4 : 1) * (0.85 + r() * 0.3);
      } else if (bestGraze > 0.1) {
        yield_ += share * bestGraze * 3.4 * seasonMul * crowd * (w.blight ? 0.25 : 1) * (0.85 + r() * 0.3);
      } else {
        yield_ += share * 1.5 * seasonMul * (0.7 + r() * 0.6);   // gathering the shore, no ground of their own
      }
    }

    // the government's cut comes off the top, before any larder is filled
    const kept = tithe(s, u.host, yield_);
    u.host.stores = clamp(u.host.stores + kept - mouths, -99, 44);
    u.host.lastYield = kept; u.host.mouths = mouths;
    for (const h of u.members) if (h !== u.host) { h.stores = 0; h.lastYield = 0; }

    if (u.host.stores < 0) {
      const deficit = -u.host.stores;
      u.host.stores = 0;
      for (const p of people) p.hunger = clamp(p.hunger + deficit * 0.55, 0, 10);
      u.host.shortSeasons = (u.host.shortSeasons || 0) + 1;
      // report a shortage once it has lasted, not the first season it dips
      if (u.host.shortSeasons === 3 || (u.host.shortSeasons > 3 && u.host.shortSeasons % 6 === 0)) {
        ev(s, 'hunger', 4, `The ${u.host.name} have been short of food for ${u.host.shortSeasons} seasons.`, { household: u.host.id });
      }
    } else {
      for (const p of people) p.hunger = Math.max(0, p.hunger - 1.8);
      u.host.shortSeasons = 0;
    }
  }
}

/* hardship --------------------------------------------------------------------
   A household that has been short for years has to do something about it. It
   was previously just accruing a slightly higher death risk in silence.
   -------------------------------------------------------------------------- */

function sysHardship(s, r) {
  for (const hh of Object.values(s.households)) {
    if (hh.extinct || hh.lodgedWith) continue;
    const n = hh.shortSeasons || 0;
    if (n < 4) continue;
    const head = s.people[hh.headId];
    if (!head || !head.alive) continue;
    const live = hh.members.map(i => s.people[i]).filter(p => p.alive);
    const fat = Object.values(s.households).filter(h => !h.extinct && !h.lodgedWith
      && h.id !== hh.id && h.stores > 12);

    // 1. send a child out to a house that can feed it
    if (n >= 4 && chance(r, 0.22)) {
      const kids = live.filter(p => ageOf(s, p) >= 7 && ageOf(s, p) <= 15);
      const asked = fat.sort((a, b) => standingOf(s, b) - standingOf(s, a));
      let to = null;
      for (const cand of asked) {
        if (holdsAgainst(s, s.people[cand.headId], hh.id) > 18) {
          ev(s, 'hardship', 6, `The ${hh.name} asked the ${cand.name} to take in a child and were refused. The ${cand.name} hold a grudge against them.`, { household: hh.id });
          remember(s, cand.id, -6, `refused to take in a hungry child`);
          continue;   // ask the next house
        }
        to = cand; break;
      }
      if (kids.length && to) {
        const kid = pick(r, kids);
        hh.members = hh.members.filter(i => i !== kid.id);
        kid.householdId = to.id; to.members.push(kid.id);
        remember(s, to.id, 5, `took in another household's child during a shortage`);
        if (head) shiftTie(s, head, to.id, 40, `took in one of ours when we had nothing`);
        const taker = s.people[to.headId];
        if (taker) shiftStanding(taker, 'kin', 9);
        ev(s, 'hardship', 5, `The ${hh.name} sent ${kid.name} to the ${to.name} to be fed.`, { person: kid.id, household: hh.id });
        continue;
      }
    }

    // 2. take what is not offered
    if (n >= 6 && head.traits.avarice + head.traits.courage > 105 && chance(r, 0.18) && fat.length) {
      const victim = pick(r, fat);
      const took = Math.min(6, victim.stores * 0.3);
      victim.stores -= took; hh.stores += took * 0.8;
      const vh = s.people[victim.headId];
      if (vh) shiftTie(s, vh, hh.id, -60, `stores taken from them`);
      remember(s, hh.id, -8, `took stores from the ${victim.name}`);
      shiftStanding(head, 'kin', -12);
      shiftStanding(head, 'quarter', -8);
      ev(s, 'hardship', 6, `The ${hh.name} took stores from the ${victim.name}. The ${victim.name} hold a grudge for it.`, { household: hh.id });
      continue;
    }

    // 3. press for better ground — not a house site, a place to work
    if (n >= 8 && chance(r, 0.2)) {
      const holder = Object.values(s.households).filter(h => !h.extinct && h.id !== hh.id
        && h.claims.filter(c => s.tiles[c].fish > 0.3 || s.tiles[c].graze > 0.4).length > 1)
        .sort((a, b) => b.claims.length - a.claims.length)[0];
      if (holder) { openDispute(s, r, hh, holder, 'water and grazing enough to live off'); continue; }
    }

    // 4. give up on the place
    if (n >= 13 && chance(r, 0.16)) {
      for (const p of live) { p.alive = false; p.deathTurn = s.turn; p.cause = 'departure'; }
      hh.extinct = s.turn;
      for (const c of hh.claims) s.tiles[c].owner = null;
      remember(s, hh.id, -2, `left the coast rather than starve`);
      ev(s, 'hardship', 6, `The ${hh.name} left the coast after ${Math.round(n / 4)} years short of food. The household is gone.`, { household: hh.id });
    }
  }

  // and the other direction: a full house may simply give
  for (const hh of Object.values(s.households)) {
    if (hh.extinct || hh.lodgedWith || hh.stores < 16) continue;
    const head = s.people[hh.headId];
    if (!head || !head.alive || head.traits.avarice > 55 || head.traits.loyalty < 50) continue;
    const needy = Object.values(s.households).filter(h => !h.extinct && !h.lodgedWith && (h.shortSeasons || 0) >= 3);
    if (needy.length < 2 || !chance(r, 0.10)) continue;
    const skipped = needy.filter(n => holdsAgainst(s, head, n.id) > 25);
    const given = needy.filter(n => !skipped.includes(n));
    if (!given.length) continue;
    const give = Math.min(hh.stores - 10, given.length * 2.5);
    hh.stores -= give;
    for (const nd of given) {
      nd.stores += give / given.length;
      const nh = s.people[nd.headId];
      if (nh) shiftTie(s, nh, hh.id, 35, `fed this household in a hard season`);
    }
    remember(s, hh.id, 7, `gave food to households not their own`);
    if (head) shiftStanding(head, 'kin', 8);
    if (skipped.length) {
      const missed = skipped[0];
      remember(s, hh.id, -4, `fed every short household except the ${missed.name}, deliberately`);
      const mh = s.people[missed.headId];
      if (mh) shiftTie(s, mh, hh.id, -70, `passed this household over when feeding every other short house`);
      ev(s, 'hardship', 6, `The ${hh.name} gave food to every short household except the ${missed.name}, who were passed over deliberately. The ${missed.name} hold a grudge for it.`, { household: hh.id });
    } else {
      ev(s, 'hardship', 5, `The ${hh.name} gave food to ${given.length === 1 ? 'the ' + given[0].name : given.length + ' short households'}.`, { household: hh.id });
    }
  }
}
