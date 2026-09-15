/* who will work your stone ----------------------------------------------------
   A tender was previously assigned to whatever stone was nearest. But the gift
   is the scarcest thing in the settlement, and the person holding it decides
   which households get to exist. A tender who will not work for a house is
   handing down a sentence, slowly, over twenty years.
   -------------------------------------------------------------------------- */

function holdsAgainst(s, person, householdId) {
  if (!person) return 0;
  return person.grudges
    .filter(g => g.target === householdId)
    .reduce((a, g) => a + g.heat * Math.pow(0.5, (s.turn - g.turn) / 200), 0);
}

// What a tender thinks of a job. Below zero and they will not take it.
function tenderWill(s, t, hh, tid) {
  const own = s.households[t.householdId];
  let v = 20;
  v -= walkDist(s, homeTile(s, t), tid) * 3.4;             // it is a walk, every day, for years
  if (own && own.id === hh.id) v += 55;                    // your own house first
  else if (own && firstSyllable(own.name) === firstSyllable(hh.name)) v += 26;  // kin
  v += reputeOf(s, hh) * 2.2;                              // what the house is known for
  v -= holdsAgainst(s, t, hh.id) * 0.9;                    // and what you hold against it
  if (t.traits.avarice > 55) v += clamp(hh.stores - 6, -8, 14) * (t.traits.avarice / 90);
  if (t.traits.piety > 60 && s.shrine && s.shrine.devotion > 55) v += 14;  // devout tenders refuse less
  if (t.traits.loyalty > 65) v += 10;
  return v;
}

function chooseTender(s, r, hh, tid, load, exclude) {
  const cands = Object.values(s.people).filter(p => p.alive && p.tender
    && ageOf(s, p) >= 16 && ageOf(s, p) <= 74 && (load[p.id] || 0) < 3 && p.id !== exclude);
  if (!cands.length) return { tender: null, refusers: [] };
  const scored = cands.map(p => ({ p, v: tenderWill(s, p, hh, tid) + (r() - 0.5) * 8 }))
    .sort((a, b) => b.v - a.v);
  const willing = scored.filter(x => x.v > 0);
  return { tender: willing.length ? willing[0].p : null, refusers: scored.filter(x => x.v <= 0).map(x => x.p) };
}

/* the growing of homes ------------------------------------------------------ */

function sysGrowth(s, r) {
  const tenders = Object.values(s.people).filter(p => p.alive && p.tender && ageOf(s, p) >= 16 && ageOf(s, p) <= 74);
  const load = {};
  for (const b of Object.values(s.buildings)) {
    if (b.state !== 'growing') continue;
    if (b.tenderId && !s.people[b.tenderId]?.alive) {
      ev(s, 'stall', 4, `The stone at ${siteWord(s, b.tileId)} stopped answering — the hand that shaped it is gone.`, { building: b.id });
      b.tenderId = null;
    }
    if (!b.tenderId) {
      const hhb = s.households[b.householdId];
      const { tender: took, refusers } = hhb
        ? chooseTender(s, r, hhb, b.tileId, load, b.lastTenderId)
        : { tender: null, refusers: [] };
      if (took && chance(r, 0.7)) {
        b.tenderId = took.id;
        ev(s, 'tend', 4, `${nameOf(s, took.id)} took up the unfinished stone at ${siteWord(s, b.tileId)}.`, { building: b.id, person: took.id });
      } else {
        b.stalled++; b.maturity = Math.max(0, b.maturity - 0.0015);
        if (b.stalled === 12 && hhb && refusers.length) {
          ev(s, 'notender', 6, `Three years now and nobody with the gift will go near the ${hhb.name} stone. ${refusers.length === 1 ? nameOf(s, refusers[0].id) + ' has' : 'They have'} not given a reason and are not being asked for one.`, { household: hhb.id });
          remember(s, hhb.id, -4, `had their stone left standing half-grown because no tender would touch it`);
        }
        continue;
      }
    }
    const t = s.people[b.tenderId];
    const hhb = s.households[b.householdId];
    if (hhb && t && tenderWill(s, t, hhb, b.tileId) < -12 && chance(r, 0.3)) {
      ev(s, 'tend', 6, `${nameOf(s, t.id)} stopped work on the ${hhb.name} stone and did not go back. It stands at ${Math.round(b.maturity * 100)} parts in a hundred.`, { building: b.id, person: t.id });
      remember(s, hhb.id, -3, `were walked out on by the tender who was growing their house`);
      b.lastTenderId = t.id; b.tenderId = null; b.stalled++;
      continue;
    }
    load[b.tenderId] = (load[b.tenderId] || 0) + 1;
    const skill = 0.55 + t.traits.loyalty / 300 + (ageOf(s, t) > 34 ? 0.25 : 0);
    const rate = 0.030 * skill * s.tiles[b.tileId].stone * (1 - (load[b.tenderId] - 1) * 0.18);
    b.maturity = Math.min(1, b.maturity + rate);
    if (b.maturity >= 1) {
      b.state = 'mature';
      const hh = s.households[b.householdId];
      if (hh) { hh.buildingId = b.id; hh.lodgedWith = null; }
      ev(s, 'mature', 5, `After ${Math.round((s.turn - b.startTurn) / 4)} years the ${hh ? hh.name : 'new'} house came to its full shape and was opened.`, { building: b.id, household: b.householdId });
    }
  }
  // slow decay of empty or neglected homes
  for (const b of Object.values(s.buildings)) {
    if (b.state !== 'mature') continue;
    const hh = s.households[b.householdId];
    const occupied = hh && hh.members.some(p => s.people[p].alive);
    if (!occupied) {
      b.condition -= 0.012;
      if (b.condition <= 0 && b.state !== 'derelict') {
        b.state = 'derelict';
        ev(s, 'derelict', 3, `The old ${hh ? hh.name : ''} house at ${siteWord(s, b.tileId)} finally split along its rings. Nobody had tended it in years.`, { building: b.id });
      }
    }
  }
}

function homeTile(s, p) {
  const hh = s.households[p.householdId];
  if (!hh) return 0;
  const b = s.buildings[hh.buildingId];
  return b ? b.tileId : (hh.claims[0] ?? 0);
}

function siteWord(s, tid) {
  const q = quarterOf(s, tid);
  const t = s.tiles[tid];
  return q ? `the ${TERRAIN_NAME[t.t]} at ${q.name}` : TERRAIN_NAME[t.t];
}

/* pairing ------------------------------------------------------------------- */
/* Households do not marry within themselves. Without a pass that moves adults
   between houses, every household that splits off is a dead end and the
   settlement bleeds out over two generations while every other number looks
   healthy. */

function adultsOf(s, hh, lo = 17, hi = 45) {
  return hh.members.map(i => s.people[i]).filter(p => p && p.alive && ageOf(s, p) >= lo && ageOf(s, p) <= hi);
}

function sysPairing(s, r) {
  const live = Object.values(s.households).filter(h => !h.extinct);
  const seeking = live.filter(h => {
    const a = adultsOf(s, h);
    if (!a.length) return false;
    return !(a.some(p => p.sex === 'm') && a.some(p => p.sex === 'f'));
  });
  if (!seeking.length) return;

  let moves = 0;
  const moved = new Set();
  for (const hh of seeking) {
    if (moves >= 3) break;
    const have = adultsOf(s, hh);        // recomputed: an earlier match this
    if (!have.length) continue;          // turn may have taken their last adult
    const want = have[0].sex === 'm' ? 'f' : 'm';
    if (!chance(r, 0.34)) continue;

    // Who is actually available: an adult of the right sex whose household is
    // not already a pair, or who is one of two or more of that sex in a house
    // that can spare one. A head can only leave a household with nothing in it.
    const pool = [];
    for (const other of live) {
      if (other.id === hh.id) continue;
      const all = adultsOf(s, other);
      const paired = all.some(q => q.sex === 'm') && all.some(q => q.sex === 'f');
      const sameSex = all.filter(q => q.sex === want).length;
      for (const c of adultsOf(s, other, 17, 40)) {
        if (c.sex !== want || moved.has(c.id)) continue;
        const spareable = !paired || sameSex >= 2;
        if (reputeOf(s, hh) < -7 && !chance(r, 0.35)) continue;   // few will marry into a house like that
        const alone = other.members.filter(i => s.people[i].alive).length <= 1;
        const canLeave = !alone && (c.role !== 'head' || (!other.buildingId && !other.claims.length));
        if (spareable && canLeave) pool.push({ p: c, from: other });
      }
    }

    // An empty pool only means nobody could be *spared* — that is an allocation
    // problem, not a shortage. A second attachment needs the settlement to be
    // genuinely thin of that sex, which happens when a bad season or a run of
    // births falls the wrong way. Nothing arranges for it.
    let second = false;
    if (!pool.length) {
      const wanted = Object.values(s.people).filter(q => q.alive && q.sex === want
        && ageOf(s, q) >= 18 && ageOf(s, q) <= 40 && q.householdId !== hh.id && !moved.has(q.id)
        && (q.traits.ambition > 62 || q.traits.courage > 62 || q.tender || q.prominence > 0));
      if (!wanted.length || !chance(r, 0.16)) continue;
      const q = pick(r, wanted);
      pool.push({ p: q, from: s.households[q.householdId] });
      second = true;
    }

    pool.sort((a, b) => walkDist(s, homeTile(s, a.p), homeTile(s, have[0])) - walkDist(s, homeTile(s, b.p), homeTile(s, have[0])));
    const chosen = pool[Math.min(pool.length - 1, Math.floor(Math.pow(r(), 2) * 4))];
    const p = chosen.p;

    moved.add(p.id);
    if (chosen.from) chosen.from.members = chosen.from.members.filter(i => i !== p.id);
    p.householdId = hh.id;
    p.role = 'none';
    p.matches = (p.matches || 0) + 1;
    hh.members.push(p.id);
    moves++;

    if (second && p.matches > 1) {
      ev(s, 'match', 6, `There was nobody left unspoken-for on this coast, so the ${hh.name} sent for ${p.name} ${p.lineage}, who was already reckoned to the ${chosen.from ? chosen.from.name : 'another'} — the ${p.matches}${p.matches === 2 ? 'nd' : p.matches === 3 ? 'rd' : 'th'} house to claim them. Opinion was divided and stayed divided.`, { person: p.id, household: hh.id });
      if (chosen.from) {
        const jilted = s.people[chosen.from.headId];
        if (jilted && jilted.alive && jilted.traits.grudge > 45)
          jilted.grudges.push({ target: hh.id, cause: `they sent for ${p.name}, who was already reckoned to this house`, turn: s.turn, heat: 45 });
      }
    } else {
      // routine. recorded, not announced.
      ev(s, 'match', 1, `${p.name} ${p.lineage} went to the ${hh.name} household, and the two houses were reckoned joined.`, { person: p.id, household: hh.id });
    }
  }
}

/* quiet-season texture ------------------------------------------------------- */
/* Reports state the simulation already holds. Nothing here changes anything. */

function sysNotes(s, r) {
  // stone passing a visible stage
  for (const b of Object.values(s.buildings)) {
    if (b.state !== 'growing' || !b.tenderId) continue;
    const hh = s.households[b.householdId];
    if (!hh) continue;
    const was = b.lastMark || 0;
    const mark = b.maturity >= 0.9 ? 3 : b.maturity >= 0.6 ? 2 : b.maturity >= 0.3 ? 1 : 0;
    if (mark > was) {
      b.lastMark = mark;
      const line = mark === 1 ? pick(r, [
          `The ${hh.name} stone has taken a shape you can walk into, though it has no roof to speak of yet.`,
          `You can see what the ${hh.name} stone means to be now. Walls to the waist, no more.`,
          `The ${hh.name} stone came up past knee height this year, which is further than it looks.`
        ]) : mark === 2 ? pick(r, [
          `Rings are showing plainly on the ${hh.name} stone now. Children have started using it as a landmark.`,
          `The ${hh.name} stone has its doorway. No roof over it, but a doorway.`,
          `Banding on the ${hh.name} stone has gone dense enough to read the years off it.`
        ]) : pick(r, [
          `The ${hh.name} stone is close. ${nameOf(s, b.tenderId)} has been up there most days.`,
          `The ${hh.name} household has begun moving things up to the new stone, roof or no roof.`,
          `Only the cap of the ${hh.name} stone is still open. ${nameOf(s, b.tenderId)} says two more seasons.`
        ]);
      const t = s.people[b.tenderId];
      const worthSaying = mark === 3 && (b.stalled > 0 || (t && t.prominence > 0));
      ev(s, 'note', worthSaying ? 3 : 1, line, { building: b.id });
    }
  }

  // the season's catch against what the settlement is used to
  const total = Object.values(s.households).reduce((a, h) => a + (h.lastYield || 0), 0);
  s.yieldHist = (s.yieldHist || []).concat(total).slice(-16);
  if (s.yieldHist.length >= 8) {
    const avg = s.yieldHist.slice(0, -1).reduce((a, b) => a + b, 0) / (s.yieldHist.length - 1);
    if (total > avg * 1.35) ev(s, 'note', 2, pick(r, [
      'The best season anyone could remember for some years. Stores went in full.',
      'Everything came in heavy — nets, herds, the lot. Nobody quite trusted it.',
      'A season of plenty, and the older houses quietly put some aside.'
    ]));
    else if (total < avg * 0.68) ev(s, 'note', 2, pick(r, [
      'A thin season. What came in was counted twice.',
      'Little came in from anywhere, and it was noticed at every table.',
      'The worst season in years, though nobody went short who had stores.'
    ]));
  }

  // the oldest living person, when that changes hands
  const live = Object.values(s.people).filter(p => p.alive);
  if (live.length) {
    const eldest = live.sort((a, b) => a.birthTurn - b.birthTurn)[0];
    if (s.eldestId !== eldest.id && ageOf(s, eldest) > 68) {
      s.eldestId = eldest.id;
      ev(s, 'note', 2, `${nameOf(s, eldest.id)} is now the oldest living soul here, at ${ageOf(s, eldest)}.`, { person: eldest.id });
    }
  }
}

/* teaching the gift -----------------------------------------------------------
   The gift was a coin flip at birth, so a tender's death put it back into the
   lottery and nothing about who held it could be inherited or withheld. It is
   taught now: a tender takes an apprentice, usually kin, over about a decade.
   That makes the question of who can build houses something families hold.
   -------------------------------------------------------------------------- */

function best0(p) { return p.name + ' ' + p.lineage; }

function sysTeaching(s, r) {
  const tenders = Object.values(s.people).filter(p => p.alive && p.tender
    && ageOf(s, p) >= 26 && ageOf(s, p) <= 78);

  // finish what is already being taught
  for (const p of Object.values(s.people)) {
    if (!p.alive || !p.learning) continue;
    const master = s.people[p.learning.from];
    if (!master || !master.alive) {
      p.learning.orphaned = (p.learning.orphaned || 0) + 1;
      if (p.learning.orphaned === 1)
        ev(s, 'teach', 5, `${nameOf(s, p.id)} was half-taught when ${p.learning.name} died. Whether half is enough is not a thing anyone can tell by looking.`, { person: p.id });
      if (p.learning.progress > 0.62 && chance(r, 0.10)) {
        p.tender = true; p.learning = null;
        ev(s, 'teach', 6, `${nameOf(s, p.id)} got the stone to answer, alone, years after the person teaching them died. Nobody is certain they were taught the whole of it.`, { person: p.id });
      } else if (chance(r, 0.05)) {
        ev(s, 'teach', 4, `${nameOf(s, p.id)} gave up on the stone. They had been at it ${Math.max(1, Math.round(p.learning.progress * 12))} years.`, { person: p.id });
        p.learning = null;
      }
      continue;
    }
    // most who start do not finish
    if (chance(r, 0.009)) {
      ev(s, 'teach', 3, `${best0(p)} stopped going up to the stone with ${p.learning.name}. Neither of them said why.`, { person: p.id });
      p.learning = null; continue;
    }
    p.learning.progress += 0.022 * (0.7 + p.traits.loyalty / 250) * (p.learning.kin ? 1.25 : 1);
    if (p.learning.progress >= 1) {
      p.tender = true;
      const yrs = Math.max(1, Math.round((s.turn - p.learning.since) / 4));
      p.learning = null;
      remember(s, p.householdId, 4, `have the gift in the house, taught and not merely born to`);
      ev(s, 'teach', 6, `${nameOf(s, p.id)} can work the stone now. It took ${yrs} years and ${nameOf(s, master.id)} says it took longer than it should have.`, { person: p.id });
    }
  }

  // and take someone new on
  for (const t of tenders) {
    const already = Object.values(s.people).some(p => p.alive && p.learning && p.learning.from === t.id);
    if (already || !chance(r, 0.075)) continue;

    const own = s.households[t.householdId];
    const pool = Object.values(s.people).filter(p => p.alive && !p.tender && !p.learning
      && p.aptitude && ageOf(s, p) >= 11 && ageOf(s, p) <= 24);
    if (!pool.length) continue;

    // kin first, then whoever is near and willing — and a tender will not teach
    // into a house they hold something against
    const scored = pool.map(p => {
      const hh = s.households[p.householdId];
      let v = p.traits.loyalty * 0.5 + p.traits.piety * 0.3 - walkDist(s, homeTile(s, p), homeTile(s, t)) * 2;
      const kin = !!(own && p.householdId === t.householdId);
      if (kin) v += 45;
      else if (own && hh && firstSyllable(own.name) === firstSyllable(hh.name)) v += 22;
      if (hh) v -= holdsAgainst(s, t, hh.id) * 0.8;
      return { p, v, kin };
    }).sort((a, b) => b.v - a.v);

    const best = scored[0];
    if (!best || best.v < 10) continue;
    best.p.learning = { from: t.id, name: nameOf(s, t.id), since: s.turn, progress: 0.02, kin: best.kin };
    ev(s, 'teach', 4, best.kin
      ? `${nameOf(s, t.id)} started teaching ${best.p.name} the stone. It stays in the house, which is how it usually goes.`
      : `${nameOf(s, t.id)} took ${nameOf(s, best.p.id)} on to teach, out of another house entirely. That was remarked on.`,
      { person: best.p.id });
  }
}
