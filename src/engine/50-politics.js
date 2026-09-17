/* memory ---------------------------------------------------------------------
   The settlement remembers. A deed done in a famine is still attached to the
   household two generations later, and it changes who is believed, who is
   ruled against, and who will marry into them. This is the layer that turns
   one-off events into causes.
   -------------------------------------------------------------------------- */

const MEMORY_HALFLIFE = 260;   // turns — about sixty-five years

function remember(s, householdId, favour, phrase) {
  const hh = s.households[householdId];
  if (!hh) return;
  s.memory.push({ household: householdId, house: hh.name, favour, phrase, turn: s.turn });
  if (s.memory.length > 240) s.memory.shift();
}

// What a household's past is worth to it now, faded by how long ago it was.
function standingOf(s, hh) {
  let v = hh.standing;
  for (const m of s.memory) {
    if (m.household !== hh.id) continue;
    v += m.favour * Math.pow(0.5, (s.turn - m.turn) / MEMORY_HALFLIFE);
  }
  return v;
}

function reputeOf(s, hh) {
  let v = 0;
  for (const m of s.memory) {
    if (m.household !== hh.id) continue;
    v += m.favour * Math.pow(0.5, (s.turn - m.turn) / MEMORY_HALFLIFE);
  }
  return v;
}

// Occasionally the old thing gets said out loud again.
function sysGrudges(s, r) {
  const sh = s.shrine;
  for (const p of Object.values(s.people)) {
    if (!p.alive || !(p.ties || []).length) continue;   // ties decay in decayTies
    const mine = s.households[p.householdId];
    const hot = p.ties.filter(g => g.value < -40 && s.households[g.target] && !s.households[g.target].extinct);
    if (!hot.length) continue;

    // put it down — but only somewhere that means something, and only if you
    // are the sort of person who would
    if (sh && sh.devotion >= 58 && p.traits.piety >= 50 && chance(r, 0.035)) {
      const g = pick(r, hot);
      const other = s.households[g.target];
      g.heat = 4;
      remember(s, p.householdId, 4, `gave up a grudge they were entitled to keep`);
      ev(s, 'shrine', 5, `${nameOf(s, p.id)} gave up their grudge against the ${other.name} at the ${sh.god} stone.`, { person: p.id });
      continue;
    }

    // or use it. A grudge looks for an opening rather than waiting for one to
    // coincide with it — which is why almost none of them ever fired before.
    if (!mine || !chance(r, 0.13)) continue;
    const g = hot.sort((a, b) => b.heat - a.heat)[0];
    const them = s.households[g.target];

    // 1. they are hungry and you are not
    if ((them.shortSeasons || 0) >= 3 && mine.stores > 10) {
      g.heat += 4;
      remember(s, mine.id, -5, `made a show of their stores while the ${them.name} went short`);
      ev(s, 'spite', 6, `The ${mine.name} made a show of their stores while the ${them.name} were short. ${p.name} did it deliberately.`, { person: p.id, household: mine.id });
      const th = s.people[them.headId];
      if (th) shiftTie(s, th, mine.id, -55, `made a display of their stores while this household went short`);
      continue;
    }

    // 2. they are in a quarrel and you can weigh in against them
    const theirs = Object.values(s.disputes).find(d => d.open && (d.a === them.id || d.b === them.id));
    if (theirs) {
      theirs.heat += 9;
      const foe = s.households[theirs.a === them.id ? theirs.b : theirs.a];
      ev(s, 'spite', 5, `${nameOf(s, p.id)} backed the ${foe ? foe.name : 'other side'} against the ${them.name}, out of a grudge.`, { person: p.id });
      continue;
    }

    // 3. they need a hand with something and yours is the hand
    if (p.tender) {
      const theirStone = Object.values(s.buildings).find(b => b.state === 'growing' && b.householdId === them.id);
      if (theirStone && theirStone.tenderId !== p.id) {
        theirStone.stalled++;
        ev(s, 'spite', 5, `${nameOf(s, p.id)} refused to work the ${them.name} stone, claiming to be busy.`, { person: p.id });
        continue;
      }
    }

    // 4. otherwise it is carried in public, which is its own kind of work
    if (chance(r, 0.5)) {
      them.standing -= 1.2;
      ev(s, 'spite', 2, `${nameOf(s, p.id)} snubbed the ${them.name} in public. The grudge is ${Math.round((s.turn - g.turn) / 4)} years old.`, { person: p.id });
    }
  }
}

function sysMemory(s, r) {
  if (!chance(r, 0.07)) return;
  const old = s.memory.filter(m => s.turn - m.turn > 40 && Math.abs(m.favour) >= 6
    && s.households[m.household] && !s.households[m.household].extinct);
  if (!old.length) return;
  const m = pick(r, old);
  const years = Math.round((s.turn - m.turn) / 4);
  ev(s, 'memory', 3, `The ${m.house} ${m.phrase} — ${years} years ago, and still held against them.`, { household: m.household });
}

/* the shrine -----------------------------------------------------------------
   Every Roduro settlement grows one stone that belongs to nobody. What it is
   worth is not devotion in the abstract — it is whether people will accept a
   ruling they dislike. A settlement that keeps its shrine settles quarrels;
   one that lets it go hard-cases them into feuds.
   -------------------------------------------------------------------------- */

const SHRINE_GODS = [
  { name: 'Horahìda', of: 'the Ocean' },
  { name: 'Dodìṭo', of: 'Stone and Mountains' },
  { name: 'Quyìturo', of: 'Time and Decay' },
  { name: 'Guʻehiqì', of: 'Trees and Growing Things' },
  { name: 'Hiyaḍote', of: 'Death and Night' },
  { name: 'Gìhuqìdu', of: 'Law and Tradition' }
];

function sysShrine(s, r) {
  const sh = s.shrine;
  if (!sh) return;

  // a keeper: the most devout adult in the settlement, if anyone is devout enough
  const keeper = s.people[sh.keeperId];
  if (!keeper || !keeper.alive) {
    if (sh.keeperId) {
      ev(s, 'shrine', 5, `${sh.keeperName} died after ${Math.round((s.turn - sh.keeperSince) / 4)} years keeping the ${sh.god} stone. It has no keeper.`, {});
      remember(s, keeper ? keeper.householdId : null, 3, `kept the ${sh.god} stone`);
    }
    sh.keeperId = null;
    const devout = livingPeople(s).filter(p => p.traits.piety > 62
      && ageOf(s, p) >= 20 && ageOf(s, p) <= 74);
    if (devout.length && chance(r, 0.4)) {
      const k = devout.sort((a, b) => b.traits.piety - a.traits.piety)[0];
      sh.keeperId = k.id; sh.keeperName = nameOf(s, k.id); sh.keeperSince = s.turn;
      ev(s, 'shrine', 4, `${nameOf(s, k.id)} took over keeping the ${sh.god} stone, untended for three seasons.`, { person: k.id });
    }
  }

  // keeping it is regard with the devout, earned by the year
  if (sh.keeperId && s.turn % 4 === 0) {
    const k = s.people[sh.keeperId];
    if (k && k.alive) shiftStanding(k, 'shrine', 7);
  }

  // what the settlement puts into it
  const heads = Object.values(s.households).filter(h => !h.extinct)
    .map(h => s.people[h.headId]).filter(p => p && p.alive);
  const piety = heads.length ? heads.reduce((a, p) => a + p.traits.piety, 0) / heads.length : 30;
  const fed = Object.values(s.households).filter(h => !h.extinct && !h.lodgedWith && h.stores > 4).length;
  const total = Object.values(s.households).filter(h => !h.extinct && !h.lodgedWith).length || 1;
  const target = clamp(piety * 0.75 + (fed / total) * 30 + (sh.keeperId ? 18 : -12), 0, 100);
  const was = sh.devotion;
  sh.devotion += (target - sh.devotion) * 0.12;

  for (const [lo, line] of [[35, 'gone'], [60, 'held']]) {
    if (was >= lo && sh.devotion < lo && lo === 35)
      ev(s, 'shrine', 5, `The ${sh.god} stone is untended. Rulings sworn there no longer bind, and quarrels harden into feuds.`, {});
    if (was < lo && sh.devotion >= lo && lo === 60)
      ev(s, 'shrine', 4, `The ${sh.god} stone is kept again. Rulings sworn there bind.`, {});
  }

  // a hard season brings people back to it
  if (s.weather.storm && sh.devotion > 45 && chance(r, 0.25)) {
    ev(s, 'shrine', 3, `After the storm the settlement made offerings at the ${sh.god} stone. Every household gave up some of its stores.`, {});
    for (const h of Object.values(s.households)) if (!h.extinct) h.stores = Math.max(0, h.stores - 0.6);
    sh.devotion = Math.min(100, sh.devotion + 4);
  }
}

/* claims: the scarce thing --------------------------------------------------- */

// Households with neither a standing house nor stone in the ground. Three
// separate systems were reading a looser count than this — which throttled the
// split rate and the gods' attention against a number four times too large.
function landless(s) {
  return Object.values(s.households).filter(hh => !hh.extinct && !hh.buildingId
    && !Object.values(s.buildings).some(b => b.householdId === hh.id && b.state === 'growing'));
}

function sysClaims(s, r) {
  const homeless = landless(s);
  if (!homeless.length) return;

  // a grown house needs room around it; you cannot pack them shoulder to shoulder
  const taken = Object.values(s.buildings).filter(b => b.state !== 'derelict').map(b => b.tileId);
  const free = s.tiles.filter(t =>
      !t.owner && siteValue(s.tiles, t.id) > 0 && !taken.some(x => tileDist(x, t.id) < 2.15))
    .sort((a, b) => siteValue(s.tiles, b.id) - siteValue(s.tiles, a.id));

  for (const hh of homeless) {
    const head = s.people[hh.headId];
    if (!head || !head.alive) continue;
    hh.waiting = (hh.waiting || 0) + 1;

    if (free.length) {
      // take the best site not already grabbed this turn
      const site = free.shift();
      s.tiles[site.id].owner = hh.id;
      hh.claims.push(site.id);
      startGrowing(s, r, hh, site.id);
      const scarce = free.length <= 2;
      ev(s, 'claim', scarce ? 5 : 2,
        `The ${hh.name} claimed ground on the ${TERRAIN_NAME[site.t]} and started a stone. ${free.length ? free.length + ' building site' + (free.length === 1 ? '' : 's') + ' left' : 'That was the last building site on the coast'}.`,
        { household: hh.id, tile: site.id });
      hh.waiting = 0;
    } else if (hh.waiting > ri(r, 3, 10)) {
      // no free ground. press a neighbour, or leave.
      const targets = Object.values(s.households).filter(o => o.id !== hh.id && !o.extinct && o.claims.length > 1);
      if (targets.length && (head.traits.ambition + head.traits.courage > 62 || hh.waiting > 14)) {
        const target = targets.sort((a, b) => b.claims.length - a.claims.length)[0];
        openDispute(s, r, hh, target, 'ground to grow a house on');
        hh.waiting = 0;
      } else if (chance(r, 0.08)) {
        for (const mid of hh.members) { const m = s.people[mid]; m.alive = false; m.deathTurn = s.turn; m.cause = 'departure'; }
        touchPeople(s);
        hh.extinct = s.turn;
        ev(s, 'leave', 5, `The ${hh.name} gave up waiting for ground and left the coast.`, { household: hh.id });
      }
    }
  }
}

function startGrowing(s, r, hh, tid) {
  const bid = 'b' + s.nextId++;
  const load = {};
  for (const b of Object.values(s.buildings)) if (b.state === 'growing' && b.tenderId) load[b.tenderId] = (load[b.tenderId] || 0) + 1;
  const { tender, refusers } = chooseTender(s, r, hh, tid, load);
  s.buildings[bid] = {
    id: bid, tileId: tid, householdId: hh.id, tenderId: tender ? tender.id : null,
    startTurn: s.turn, maturity: 0.02, capacity: ri(r, 4, 7),
    state: 'growing', stalled: 0, condition: 1, refused: refusers.length
  };
  if (!tender) {
    const anyone = livingPeople(s).some(p => p.tender && ageOf(s, p) >= 16);
    ev(s, 'notender', 6, anyone
      ? `Every free tender refused to start the ${hh.name} stone. It sits unworked.`
      : `No living tender can start the ${hh.name} stone. It sits unworked.`,
      { household: hh.id });
    if (anyone) {
      remember(s, hh.id, -5, `could not get any tender to work their stone`);
      const head = s.people[hh.headId];
      for (const rf of refusers.slice(0, 2)) {
        if (head && head.traits.grudge > 35 && rf.householdId !== hh.id)
          shiftTie(s, head, rf.householdId, -55, `refused to work this household's stone`);
      }
    }
  }
}

/* disputes ------------------------------------------------------------------- */

function openDispute(s, r, claimant, holder, over) {
  const existing = Object.values(s.disputes).find(d => d.open && d.a === claimant.id && d.b === holder.id);
  if (existing) { existing.heat += 12; return existing; }
  const did = 'd' + s.nextId++;
  const d = {
    id: did, a: claimant.id, b: holder.id, over, heat: 20 + r() * 20,
    open: true, turn: s.turn, arbitrated: 0
  };
  s.disputes[did] = d;
  const ha = s.people[claimant.headId];
  const against = Object.values(s.disputes).filter(x => x.open && x.b === holder.id).length;
  const who = ha ? ha.name : 'The ' + claimant.name;
  ev(s, 'dispute', 5,
    `${who} raised a claim against the ${holder.name} over ${over}.` +
    (against >= 2 ? ` That is ${against} open claims against that household.` : ''),
    { dispute: did });
  return d;
}

function sysDisputes(s, r) {
  for (const d of Object.values(s.disputes)) {
    if (!d.open) continue;
    const A = s.households[d.a], B = s.households[d.b];
    if (!A || !B || A.extinct || B.extinct) { d.open = false; continue; }
    const ha = s.people[A.headId], hb = s.people[B.headId];
    const heatGain = 1.3 + ((ha?.traits.grudge || 0) + (hb?.traits.grudge || 0)) / 95;
    d.heat += heatGain * (0.6 + r() * 0.8);
    d.age = s.turn - d.turn;

    // Roduro conflict is slow and structural. It escalates in years, not seasons.
    const dev = s.shrine ? s.shrine.devotion : 50;
    const hasArbiter = officeHolders(s, 'arbiter').length > 0;
    if (d.heat > 75 && chance(r, (hasArbiter ? 0.12 : 0.05) + dev / 900)) {
      // an appointed arbiter rules if there is one; otherwise it falls back to
      // whichever respected house will take it, which is how it used to work
      const seated = officeHolders(s, 'arbiter')[0];
      const bench = Object.values(s.households)
        .filter(h => !h.extinct && h.id !== A.id && h.id !== B.id && h.buildingId)
        .sort((x, y) => standingOf(s, y) - standingOf(s, x)).slice(0, 5);
      let arbHead = null, arb = null;
      if (seated && seated.householdId !== A.id && seated.householdId !== B.id) {
        arbHead = seated; arb = s.households[seated.householdId];
      } else if (bench.length) {
        arb = bench[Math.floor(Math.pow(r(), 1.6) * bench.length)];
        arbHead = s.people[arb.headId];
      }
      if (arb) {
        const spiteA = holdsAgainst(s, arbHead, A.id), spiteB = holdsAgainst(s, arbHead, B.id);
        const scoreA = standingOf(s, A) + (ha?.traits.loyalty || 0) - spiteA;
        const scoreB = standingOf(s, B) + (hb?.traits.loyalty || 0) - spiteB;
        const forA = scoreA > scoreB ? true : chance(r, 0.35);
        const crooked = Math.max(spiteA, spiteB) > 18 && (spiteA > spiteB) !== forA;
        if (crooked) {
          const wronged = spiteA > spiteB ? A : B;
          remember(s, arb.id, -6, `ruled on a quarrel they had a stake in`);
          const wh = s.people[wronged.headId];
          if (wh) shiftTie(s, wh, arb.id, -65, `ruled against this household while holding a grudge against it`);
          ev(s, 'arbitration', 6, `The ${arb.name} ruled between the ${A.name} and the ${B.name} while holding a grudge against the ${wronged.name}. The ruling is not trusted.`, { dispute: d.id });
        }
        const loser = forA ? B : A, winner = forA ? A : B;
        if (forA && B.claims.length > 1) {
          // prefer to hand over ground the winner could actually build on
          const spare = B.claims.filter(c => c !== (s.buildings[B.buildingId] || {}).tileId);
          const buildable = spare.filter(c => siteValue(s.tiles, c) > 0);
          const give = (A.buildingId ? spare : buildable.concat(spare))[0];
          if (give !== undefined) {
            B.claims = B.claims.filter(c => c !== give);
            s.tiles[give].owner = A.id; A.claims.push(give);
            const already = Object.values(s.buildings).some(b => b.householdId === A.id && b.state === 'growing');
            if (!A.buildingId && !already && siteValue(s.tiles, give) > 0) startGrowing(s, r, A, give);
          }
        }
        d.open = false; d.resolved = s.turn; d.winner = winner.id;
        arb.standing += 4;
        for (const id of loser.members) {
          const lp = s.people[id];
          if (lp && lp.alive) shiftStanding(lp, 'government', -5);
        }
        const lh = s.people[loser.headId];
        if (lh && lh.traits.grudge > 40) {
          shiftTie(s, lh, winner.id, -d.heat * 0.6, `the ${arb.name} ruled against them over ${d.over}`);
        }
        if (dev > 60 && s.shrine) {
          ev(s, 'arbitration', 6, `The ${arb.name} ruled for the ${winner.name} over the ${loser.name}, sworn at the ${s.shrine.god} stone. The quarrel is settled.`, { dispute: d.id });
          remember(s, loser.id, -4, `were ruled against at the ${s.shrine.god} stone over ${d.over}`);
          const wh2 = s.people[winner.headId];
          if (wh2) shiftTie(s, wh2, arb.id, 25, `ruled for this household over ${d.over}`);
        } else {
          ev(s, 'arbitration', 6, `${seated && arbHead === seated ? nameOf(s, seated.id) + ', arbiter,' : 'The ' + arb.name} ruled for the ${winner.name} over the ${loser.name}. The ${loser.name} accepted it. The quarrel is settled.`, { dispute: d.id });
        }
        remember(s, arb.id, 5, `ruled between two households in a quarrel`);
      }
    } else if (d.heat > 110 + dev * 0.9) {
      d.open = false; d.resolved = s.turn; d.feud = true;
      for (const side of [[A, B], [B, A]]) {
        const h = s.people[side[0].headId];
        if (h) shiftTie(s, h, side[1].id, -70, `the unsettled quarrel over ${d.over}`);
      }
      remember(s, A.id, -6, `let a quarrel with the ${B.name} harden into a feud`);
      remember(s, B.id, -6, `let a quarrel with the ${A.name} harden into a feud`);
      const q = closePath(s, A, B);
      if (q) {
        A.standing -= 3; B.standing -= 3;
        ev(s, 'feud', 6, `The ground between the ${A.name} and the ${B.name} at ${q.name} is closed. Everyone else now walks further.`, {});
      }
      ev(s, 'feud', 7, `The quarrel between the ${A.name} and the ${B.name} went unruled and hardened into a feud. It will not now be settled.`, { dispute: d.id });
    }
  }
}

function revealGrudge(s, r) {
  const holders = livingPeople(s).filter(p => (p.ties || []).some(x => x.value < -20));
  if (!holders.length) { ev(s, 'divine', 3, 'No hidden grudge was found. Nobody here is holding one.'); return; }
  const p = pick(r, holders);
  const g = pick(r, p.ties.filter(x => x.value < -20));
  shiftTie(s, p, g.target, -30, g.cause);
  const target = s.households[g.target];
  ev(s, 'reveal', 6, `${nameOf(s, p.id)}'s grudge against ${target ? 'the ' + target.name : 'an old injury'} became public: ${g.cause}. It is now an open quarrel.`, { person: p.id });
  if (target && p.householdId !== target.id) {
    const mine = s.households[p.householdId];
    if (mine) openDispute(s, r, mine, target, 'an old injury made public');
  }
}

function arriveStrangers(s, r) {
  const n = ri(r, 3, 7);
  const hid = 'h' + s.nextId++;
  const hh = {
    id: hid, name: lineageName(r), headId: null, members: [], buildingId: null,
    claims: [], stores: 2, standing: 5, lodgedWith: null, founded: s.turn, incomer: true, goods: blankGoods()
  };
  s.households[hid] = hh;
  let brought = null;
  for (let i = 0; i < n; i++) {
    const p = makePerson(s, r, hid, i === 0 ? ri(r, 26, 48) : ri(r, 1, 40));
    if (i === 0) {
      hh.headId = p.id; p.role = 'head';
      // people come from somewhere, and they come knowing something
      brought = tradeForIncomer(s, r);
      if (brought) setTrade(s, p, brought);
    }
  }
  ev(s, 'arrival', 6, `${n} newcomers arrived by boat as the ${hh.name} household. They are asking for ground.`, { household: hid });
  if (brought) {
    const had = livingPeople(s).filter(p => p.trade === brought).length;
    ev(s, 'arrival', had <= 1 ? 7 : 4,
      had <= 1 ? `Their head can work as a ${brought}. Nobody else on this coast can.`
               : `Their head can work as a ${brought}.`, { household: hid });
  }
}
