/* birth, age and death ------------------------------------------------------ */

function sysLife(s, r) {
  const w = s.weather;
  for (const p of Object.values(s.people)) {
    if (!p.alive) continue;
    const a = ageOf(s, p);
    const hh = s.households[p.householdId];
    const crowd = hh ? crowding(s, hh) : 1;

    let risk = 0.0009;
    if (a < 3) risk = 0.016;
    else if (a > 58) risk = 0.004 * Math.pow(1.13, a - 58);
    risk *= 1 + p.hunger * 0.18;
    risk *= 1 + Math.max(0, crowd - 1) * 0.35;
    risk *= 1 + w.cold * 0.4;
    if (w.fever) risk *= a > 60 || crowd > 1.2 ? 3.4 : 1.6;

    if (chance(r, risk)) {
      kill(s, p, p.hunger > 4 ? 'hunger' : w.fever ? 'the fever' : a > 58 ? 'age' : 'illness');
    }
  }

  // births
  for (const hh of Object.values(s.households)) {
    const women = hh.members.map(i => s.people[i]).filter(p => p.alive && p.sex === 'f' && ageOf(s, p) >= 18 && ageOf(s, p) <= 42);
    const men = hh.members.map(i => s.people[i]).filter(p => p.alive && p.sex === 'm' && ageOf(s, p) >= 18);
    if (!women.length || !men.length) continue;
    const fed = hh.stores > 3 ? 1 : 0.35;
    const room = crowding(s, hh) > 1.15 ? 0.45 : 1;
    for (const wman of women) {
      if (chance(r, 0.055 * fed * room)) {
        const child = makePerson(s, r, hh.id, 0);
        child.role = 'child';
        child.parents = [wman.id, pick(r, men).id];
        // the gift runs in families
        if (wman.tender || s.people[child.parents[1]].tender) child.tender = chance(r, 0.34);
        if (child.tender) ev(s, 'birth', 3, `${child.name} was born to the ${hh.name} household, with the aptitude to learn the stone.`, { person: child.id, household: hh.id });
      }
    }
  }
}

function kill(s, p, cause) {
  p.alive = false; p.deathTurn = s.turn; p.cause = cause;
  touchPeople(s);
  const hh = s.households[p.householdId];
  const weight = notable(s, p) ? 6 : p.role === 'head' ? 4 : p.tender ? 5 : 1;
  let text;
  if (p.tender) text = `${nameOf(s, p.id)} died of ${cause}. They were a stone-tender.`;
  else if (notable(s, p)) text = `${nameOf(s, p.id)} died of ${cause}.`;
  else if (p.role === 'head') text = `${nameOf(s, p.id)} died of ${cause}. They were head of the ${hh ? hh.name : ''} household.`;
  else text = `${nameOf(s, p.id)} died of ${cause}.`;
  ev(s, 'death', weight, text, { person: p.id, household: p.householdId, cause });

  // ties pass to the heirs when the dead held them hard — either way
  if (hh) {
    for (const g of p.ties || []) {
      if (Math.abs(g.value) < 30) continue;
      for (const mid of hh.members) {
        const m = s.people[mid];
        if (!m.alive || m.id === p.id) continue;
        if (m.traits.grudge > 45 && chance(mulberry32(s.turn + parseInt(mid.slice(1))), 0.5)) {
          shiftTie(s, m, g.target, g.value * 0.5, g.cause + ', which their kin died still holding');
        }
      }
    }
  }
}

function crowding(s, hh) {
  const b = s.buildings[hh.buildingId];
  const live = hh.members.filter(i => s.people[i].alive).length;
  if (!b || b.state !== 'mature') return live > 0 ? 1.8 : 1;   // lodging or under canvas
  return live / Math.max(1, b.capacity);
}

/* households: succession, splitting, lodging -------------------------------- */

function sysHouseholds(s, r) {
  for (const hh of Object.values(s.households)) {
    const live = hh.members.map(i => s.people[i]).filter(p => p.alive);
    if (!live.length) {
      if (!hh.extinct) {
        hh.extinct = s.turn;
        const yrs = Math.round((s.turn - (hh.founded || 0)) / 4);
        ev(s, 'extinct', 6, `The last of the ${hh.name} died${yrs > 0 && yrs < 300 ? ', ' + yrs + ' years after the household was founded' : ''}. Their ground is now unclaimed.`, { household: hh.id });
        for (const c of hh.claims) s.tiles[c].owner = null;
        hh.claims = [];
      }
      continue;
    }
    // succession
    const head = s.people[hh.headId];
    if (!head || !head.alive) {
      const heirs = live.filter(p => ageOf(s, p) >= 16)
        .sort((a, b) => (b.traits.ambition + b.traits.loyalty * 0.4 + ageOf(s, b) * 0.7) - (a.traits.ambition + a.traits.loyalty * 0.4 + ageOf(s, a) * 0.7));
      if (heirs.length) {
        const next = heirs[0];
        next.role = 'head'; hh.headId = next.id;
        const contested = heirs.length > 1 && heirs[1].traits.ambition > 62 && heirs[1].traits.ambition > next.traits.ambition - 18;
        if (contested) {
          const rival = heirs[1];
          shiftTie(s, rival, hh.id, -(35 + rival.traits.grudge * 0.4), `the household seat passed to ${next.name} instead of them`);
          ev(s, 'succession', 5, `${nameOf(s, next.id)} took the ${hh.name} seat. ${rival.name} was passed over and holds a grudge for it.`, { person: next.id, household: hh.id });
        } else {
          const outsider = next.lineage !== hh.name;
          const odd = outsider || next.tender || hasGoal(next);
          ev(s, 'succession', odd ? 4 : 1, outsider
            ? `${nameOf(s, next.id)} took the ${hh.name} seat. They were born to the ${next.lineage}, not to this household.`
            : `${nameOf(s, next.id)} took the ${hh.name} seat.`, { person: next.id, household: hh.id });
        }
      }
    }
    assignWork(s, r, hh);

    // a grown child with ambition sets up on their own — but only when the old
    // house is both crowded and fed enough to spare them
    const founders = live.filter(p => ageOf(s, p) >= 21 && ageOf(s, p) <= 44 && p.role !== 'head');
    const homes = Object.values(s.buildings).filter(b => b.state !== 'derelict').map(b => b.tileId);
    const ground = s.tiles.some(t => !t.owner && siteValue(s.tiles, t.id) > 0
      && !homes.some(x => tileDist(x, t.id) < 2.15));
    if (founders.length && live.length >= 4 && crowding(s, hh) > 0.9 && hh.stores > 6) {
      const f = founders.sort((a, b) => b.traits.ambition - a.traits.ambition)[0];
      const waiting = landless(s).length;
      const pressure = clamp(1 - waiting / 18, 0.10, 1);   // nobody splits into a queue
      const urge = ((ground ? 0.021 : 0.003) + f.traits.ambition / (ground ? 2800 : 14000)) * pressure;
      if (chance(r, urge)) splitHousehold(s, r, hh, f);
    }
  }
}

const VOWEL_SET = new Set(['a', 'e', 'i', 'ì', 'o', 'u']);

// Gogìḍu is strictly (C)V, so a syllable ends at the first vowel. Onsets can be
// two characters ('th', 'sh'), which is what a plain slice got wrong.
function firstSyllable(name) {
  const lower = name.charAt(0).toLowerCase() + name.slice(1);
  for (let i = 0; i < lower.length; i++) {
    if (VOWEL_SET.has(lower[i])) return lower.slice(0, i + 1);
  }
  return lower.slice(0, 2);
}

function cadetName(s, r, parentName) {
  const stem = firstSyllable(parentName);
  const taken = new Set(Object.values(s.households).map(h => h.name));
  const cap = (n) => n.charAt(0).toUpperCase() + n.slice(1);
  for (let i = 0; i < 30; i++) {
    const n = cap(stem + syl(r) + pick(r, LINEAGE_TAIL));
    if (!taken.has(n)) return n;
  }
  return cap(stem + syl(r) + syl(r));
}

function splitHousehold(s, r, parent, founder) {
  const hid = 'h' + s.nextId++;
  const name = cadetName(s, r, parent.name);
  const hh = {
    id: hid, name, headId: founder.id, members: [founder.id],
    buildingId: null, claims: [], stores: Math.max(0, parent.stores * 0.25),
    standing: parent.standing * 0.5, lodgedWith: parent.id, founded: s.turn, parentId: parent.id,
    goods: blankGoods()
  };
  parent.stores *= 0.75;
  parent.members = parent.members.filter(i => i !== founder.id);
  founder.householdId = hid; founder.role = 'head';
  s.households[hid] = hh;
  const homes = Object.values(s.buildings).filter(b => b.state !== 'derelict').map(b => b.tileId);
  const roomLeft = s.tiles.some(t => !t.owner && siteValue(s.tiles, t.id) > 0
    && !homes.some(x => tileDist(x, t.id) < 2.15));
  const worthTelling = !roomLeft || founder.tender || hasGoal(founder);
  ev(s, 'split', worthTelling ? 4 : 1,
    `${founder.name} split from the ${parent.name} to found the ${name}. ` + (roomLeft
      ? `They lodge with the ${parent.name} until a site is found.`
      : `No building ground is left on the coast; they lodge with the ${parent.name}.`),
    { person: founder.id, household: hid });
  founder.lineage = name;
}
