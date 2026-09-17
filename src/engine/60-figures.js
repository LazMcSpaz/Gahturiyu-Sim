/* who is pursuing something --------------------------------------------------
   There is no cap on how many people may matter. There used to be one — eight —
   and it was the wrong dial: it limited how many people the simulation would
   let be interesting, when the real question is how much of it reaches the
   page. That belongs to event weight, which the chronicle already filters.

   The real limit is the work available. An opportunity can only be taken once,
   so the number of people with goals is whatever the settlement currently
   needs doing, which is the right thing for it to be.
   -------------------------------------------------------------------------- */

/* Anyone currently pursuing something. `prominence` is gone — having a goal
   said everything the field said. */
function hasGoal(p) { return !!p.goal; }

/* Worth naming when they die, or when they take a seat: someone at it now, or
   someone who was, or someone holding an office. */
function notable(s, p) {
  return hasGoal(p) || (p.deeds && p.deeds.length > 0) || p.tender
    || Object.values(s.offices || {}).some(ids => ids.includes(p.id));
}

/* The same deed done twice is one thing a person is known for, not two. */
function credit(p, deed) {
  if (!p.deeds.includes(deed)) p.deeds.push(deed);
}

function sysPromotion(s, r) {
  // a goal that has gone nowhere for long enough is given up
  for (const p of Object.values(s.people)) {
    if (!p.alive || !p.goal) continue;
    p.goalAge++;
    if (p.goalAge > 30) {
      p.goal = null;
      ev(s, 'settle', 1, `${nameOf(s, p.id)} gave up on their goal.`, { person: p.id });
    }
  }

  const opportunities = findOpportunities(s);
  if (!opportunities.length) return;

  const candidates = Object.values(s.people).filter(p => {
    const a = ageOf(s, p);
    return p.alive && !p.goal && a >= 17 && a <= 68;
  });

  const scored = [];
  for (const p of candidates) {
    for (const o of opportunities) {
      // a goal aimed at your own house is not a goal; you would grind at it
      // forever, because you can never stand above yourself
      if (o.target && o.target === p.householdId) continue;
      const reach = 1 - clamp(walkDist(s, homeTile(s, p), o.tile) / 14, 0, 0.95);
      const fit = o.wants.reduce((acc, t) => acc + p.traits[t], 0) / o.wants.length;
      const score = fit * 0.55 + reach * 40 + ((p.ties || []).some(x => x.value < -20) ? 12 : 0) + (p.tender && o.kind === 'stone' ? 40 : 0);
      if (score > 76) scored.push({ p, o, score: score + r() * 8 });
    }
  }
  scored.sort((a, b) => b.score - a.score);
  /* Without the cap, every opportunity would be filled the season it appears,
     and the chronicle would be a third "took up a goal". People do not work
     like that: a thing sits undone for years and then somebody has had enough.
     An opportunity is taken up in roughly one season in four. */
  const taken = new Set();
  for (const c of scored) {
    if (taken.has(c.o.id) || c.p.goal) continue;
    taken.add(c.o.id);
    if (!chance(r, 0.25)) continue;
    c.p.goal = { kind: c.o.kind, tile: c.o.tile, target: c.o.target, since: s.turn, label: c.o.label };
    c.p.goalAge = 0;
    ev(s, 'rise', 6, `${nameOf(s, c.p.id)} (${describeTraits(c.p)}) took up a goal: ${c.o.label}.`, { person: c.p.id });
  }
}

function findOpportunities(s) {
  const out = [];
  for (const b of Object.values(s.buildings)) {
    if (b.state === 'growing' && (!b.tenderId || !s.people[b.tenderId]?.alive))
      out.push({ id: 'o_' + b.id, kind: 'stone', tile: b.tileId, target: b.id, wants: ['loyalty', 'piety'], label: 'finishing the abandoned stone' });
  }
  for (const d of Object.values(s.disputes)) {
    if (d.open && d.heat > 40)
      out.push({ id: 'o_' + d.id, kind: 'champion', tile: (s.households[d.a]?.claims[0] ?? 0), target: d.id, wants: ['courage', 'grudge'], label: 'the quarrel over ' + d.over });
  }
  const homeless = landless(s);
  if (homeless.length > 1)
    out.push({ id: 'o_ground', kind: 'ground', tile: (homeless[0].claims[0] ?? 0), target: null, wants: ['ambition', 'avarice'], label: 'getting ground for the landless households' });
  const hungry = Object.values(s.households).filter(h => !h.extinct && !h.lodgedWith && h.stores < 1);
  if (hungry.length > 2)
    out.push({ id: 'o_hunger', kind: 'relief', tile: (hungry[0].claims[0] ?? 0), target: null, wants: ['loyalty', 'courage'], label: 'feeding the households with no stores' });
  const rich = Object.values(s.households).filter(h => !h.extinct).sort((a, b) => b.claims.length - a.claims.length)[0];
  if (rich && rich.claims.length >= 3)
    out.push({ id: 'o_seat', kind: 'standing', tile: rich.claims[0], target: rich.id, wants: ['ambition', 'avarice'], label: 'outranking the ' + rich.name + ' household' });
  if (s.shrine && (s.shrine.devotion < 42 || !s.shrine.keeperId))
    out.push({ id: 'o_shrine', kind: 'shrine', tile: s.shrine.tileId, target: null, wants: ['piety', 'loyalty'], label: 'restoring the ' + s.shrine.god + ' stone' });
  const shamed = Object.values(s.households).filter(h => !h.extinct && reputeOf(s, h) < -6)[0];
  if (shamed)
    out.push({ id: 'o_name', kind: 'name', tile: (shamed.claims[0] ?? 0), target: shamed.id, wants: ['loyalty', 'courage'], label: 'clearing the ' + shamed.name + ' name' });
  return out;
}

function describeTraits(p) {
  const t = p.traits;
  const words = [];
  if (t.ambition > 70) words.push('ambitious');
  if (t.grudge > 70) words.push('vengeful');
  if (t.piety > 70) words.push('devout');
  if (t.avarice > 70) words.push('greedy');
  if (t.loyalty > 70) words.push('loyal');
  if (t.courage > 70) words.push('brave');
  if (p.tender) words.push('a stone-tender');
  return words.length ? words.slice(0, 2).join(', ') : 'unremarkable';
}

/* what the named cast actually does ------------------------------------------ */

function sysActors(s, r) {
  s.pursuitLines = 0;
  for (const p of Object.values(s.people)) {
    if (!p.alive || !p.goal) continue;
    const g = p.goal;
    const hh = s.households[p.householdId];
    if (!hh) continue;
    const before = s.log.length;

    if (g.kind === 'stone') {
      const b = s.buildings[g.target];
      if (!b || b.state !== 'growing') { p.goal = null; continue; }
      if (!b.tenderId && p.tender) {
        b.tenderId = p.id;
        ev(s, 'act', 5, `${nameOf(s, p.id)} took over the stalled stone and started work.`, { person: p.id });
        p.goal = null; p.goalAge = 0; credit(p, 'took up abandoned stone');
      } else if (!p.tender && chance(r, 0.3)) {
        ev(s, 'act', 3, `${nameOf(s, p.id)} worked at the stalled stone without the gift. Nothing moved.`, { person: p.id });
      }
    }

    else if (g.kind === 'champion') {
      const d = s.disputes[g.target];
      if (!d || !d.open) { p.goal = null; continue; }
      d.heat += p.traits.courage / 100 * 9;
      if (chance(r, 0.08)) {   // pushing too hard can cost you
        hh.standing -= 2;
        remember(s, hh.id, -3, `pushed a quarrel too far`);
        ev(s, 'act', 5, `${nameOf(s, p.id)} pushed the quarrel too far and lost standing for it.`, { person: p.id });
      }
      if (chance(r, 0.16)) ev(s, 'act', 4, `${nameOf(s, p.id)} pressed the quarrel over ${d.over} again. It got hotter.`, { person: p.id });
    }

    else if (g.kind === 'ground') {
      if (chance(r, 0.22)) {
        const holder = Object.values(s.households).filter(h => !h.extinct && h.claims.length > 1 && h.id !== hh.id)
          .sort((a, b) => b.claims.length - a.claims.length)[0];
        const already = Object.values(s.disputes).some(x => x.open && x.a === hh.id && x.b === holder.id);
        if (holder && !already) {
          openDispute(s, r, hh, holder, 'ground standing idle while others wait');
          ev(s, 'act', 5, `${nameOf(s, p.id)} accused the ${holder.name} of holding ${holder.claims.length} claims they cannot build on, and raised a claim against them.`, { person: p.id });
          credit(p, 'picked a fight over idle ground');
        }
      }
    }

    else if (g.kind === 'relief') {
      if (hh.stores > 6 && chance(r, 0.35)) {
        const needy = Object.values(s.households).filter(h => !h.extinct && h.stores < 1);
        const give = Math.min(hh.stores - 3, needy.length * 1.5);
        if (give > 0) {
          hh.stores -= give;
          for (const n of needy) n.stores += give / needy.length;
          hh.standing += 3; p.goal = null; p.goalAge = 0;
          remember(s, hh.id, 8, `fed households not their own from short stores`);
          shiftStanding(p, 'kin', 14);
          shiftStanding(p, 'quarter', 6);
          for (const n of needy) {
            const nh = s.people[n.headId];
            if (nh && n.id !== hh.id) shiftTie(s, nh, hh.id, 30, `opened their stores to this household`);
          }
          ev(s, 'act', 5, `${nameOf(s, p.id)} opened the ${hh.name} stores to households not their own.`, { person: p.id });
          credit(p, 'fed households not their own');
        }
      } else if (chance(r, 0.18)) {
        ev(s, 'act', 3, `${nameOf(s, p.id)} went round the settlement asking what each household could spare.`, { person: p.id });
      } else if (hh.stores < 2 && p.traits.avarice > 60 && chance(r, 0.25)) {
        const victim = Object.values(s.households).filter(h => !h.extinct && h.stores > 8)[0];
        if (victim) {
          victim.stores -= 4; hh.stores += 3;
          const vh = s.people[victim.headId];
          if (vh) shiftTie(s, vh, hh.id, -55, 'stores taken from them');
          remember(s, hh.id, -7, `suspected of taking stores from the ${victim.name}`);
          ev(s, 'act', 6, `${nameOf(s, p.id)} took stores from the ${victim.name}. It could not be proved.`, { person: p.id });
          credit(p, 'took what was not theirs');
        }
      }
    }

    else if (g.kind === 'shrine') {
      const sh = s.shrine;
      if (!sh) { p.goal = null; continue; }
      if (!sh.keeperId && p.traits.piety > 55) {
        sh.keeperId = p.id; sh.keeperName = nameOf(s, p.id); sh.keeperSince = s.turn;
        sh.devotion = Math.min(100, sh.devotion + 6);
        p.goal = null; p.goalAge = 0;
        credit(p, 'took the keeping of the ' + sh.god + ' stone');
        remember(s, hh.id, 6, `gave the ${sh.god} stone a keeper when it had none`);
        shiftStanding(p, 'shrine', 18);
        ev(s, 'act', 6, `${nameOf(s, p.id)} became keeper of the ${sh.god} stone.`, { person: p.id });
      } else if (chance(r, 0.25)) {
        sh.devotion = Math.min(100, sh.devotion + 2.5);
        ev(s, 'act', 3, `${nameOf(s, p.id)} brought households back to the ${sh.god} stone. It is better kept for it.`, { person: p.id });
      }
    }

    else if (g.kind === 'name') {
      const target = s.households[g.target];
      if (!target || target.extinct || reputeOf(s, target) > -2) {
        if (target && !target.extinct) {
          p.goal = null; p.goalAge = 0;
          credit(p, 'got the ' + target.name + ' name spoken plainly again');
          ev(s, 'act', 6, `${p.name} cleared the ${target.name} name. Nothing is held against that household now.`, { person: p.id });
        } else p.goal = null;
        continue;
      }
      if (chance(r, 0.3)) {
        remember(s, target.id, 2, `had someone speak for them publicly`);
        const th2 = s.people[target.headId];
        if (th2 && hh) shiftTie(s, th2, hh.id, 45, `spoke for this household when nobody else would`);
        ev(s, 'act', 4, `${nameOf(s, p.id)} spoke publicly for the ${target.name}, at cost to their own standing.`, { person: p.id });
      }
    }

    else if (g.kind === 'standing') {
      const target = s.households[g.target];
      if (!target || target.extinct) { p.goal = null; continue; }
      hh.standing += 0.8;
      if (chance(r, 0.2)) ev(s, 'act', 3, `${nameOf(s, p.id)} worked to raise the ${hh.name} standing. It went up a little.`, { person: p.id });
      if (standingOf(s, hh) > standingOf(s, target) && chance(r, 0.3)) {
        p.goal = null; p.goalAge = 0;
        ev(s, 'act', 6, `The ${hh.name} now outrank the ${target.name}. ${p.name} did it.`, { person: p.id });
        credit(p, 'raised their house above another');
      }
    }

    // A figure who got nowhere this season is still visibly at it. Without
    // this they vanish from the chronicle for years between outcomes.
    const acted = s.log.slice(before).some(e => e.person === p.id);
    s.pursuitLines = (s.pursuitLines || 0);
    if (!acted && p.goal && s.pursuitLines < 2 && chance(r, 0.34)) {
      s.pursuitLines++;
      const yrs = Math.max(1, Math.round((s.turn - p.goal.since) / 4));
      ev(s, 'act', 2, `${nameOf(s, p.id)} is still at ${p.goal.label} — ${yrs} year${yrs === 1 ? '' : 's'} now, no result.`, { person: p.id });
    }
  }
}

/* the gods speak, sparingly --------------------------------------------------- */

function sysOmen(s, r) {
  s.omen = null;
  s.godSilence++;
  if (s.godSilence < 3) return;

  const st = s.stats[s.stats.length - 1] || {};
  const homeless = landless(s).length;
  const hungry = Object.values(s.households).filter(h => !h.extinct && !h.lodgedWith && h.stores < 1).length;
  const openD = Object.values(s.disputes).filter(d => d.open).length;
  const hotD = Object.values(s.disputes).filter(d => d.open && d.heat > 60).length;
  const tenders = livingPeople(s).filter(p => p.tender && ageOf(s, p) >= 16).length;
  const growing = Object.values(s.buildings).filter(b => b.state === 'growing').length;

  const candidates = [];
  if (hungry >= 3) candidates.push({ god: 'Horahìda', domain: 'Ocean & Seas', saw: 'households going hungry beside good fishing water', lever: 'bounty' });
  if (hungry >= 4) candidates.push({ god: 'Hiyaḍote', domain: 'Death & Night', saw: 'more people than this ground can feed', lever: 'fever' });
  if (homeless >= 2 && growing > 0) candidates.push({ god: 'Dodìṭo', domain: 'Stone & Mountains', saw: 'households waiting on stone that is growing too slowly', lever: 'quicken' });
  if (tenders <= 1) candidates.push({ god: 'Hiṭogiʻa', domain: 'Creation', saw: 'too few stone-tenders left', lever: 'strangers' });
  if (hotD >= 1) candidates.push({ god: 'Gìhuqìdu', domain: 'Governance & Order', saw: 'a hot quarrel that nobody will rule on', lever: 'temper' });
  if (openD === 0 && s.turn > 12) candidates.push({ god: 'Yohyeʻ', domain: 'Confusion & Delusion', saw: 'no open quarrels, and grudges kept hidden', lever: 'reveal' });
  if (growing >= 3 && hungry === 0) candidates.push({ god: 'Quyìturo', domain: 'Time & Decay', saw: 'stone growing well and nobody going hungry', lever: 'storm' });

  if (!candidates.length) return;
  if (!chance(r, 0.35)) return;

  const fresh = candidates.filter(x => x.lever !== s.lastOmenLever);
  const c = pick(r, fresh.length ? fresh : candidates);
  s.omen = c; s.lastOmenLever = c.lever;
  s.godSilence = 0;
}

/* bookkeeping ---------------------------------------------------------------- */

function recount(s) {
  const live = Object.values(s.people).filter(p => p.alive);
  const hh = Object.values(s.households).filter(h => !h.extinct);
  s.stats.push({
    turn: s.turn,
    pop: live.length,
    households: hh.length,
    waiting: landless(s).length,
    growing: Object.values(s.buildings).filter(b => b.state === 'growing').length,
    mature: Object.values(s.buildings).filter(b => b.state === 'mature').length,
    tenders: live.filter(p => p.tender && ageOf(s, p) >= 16).length,
    stores: Math.round(hh.reduce((a, h) => a + h.stores, 0)),
    hungry: hh.filter(h => !h.lodgedWith && h.stores < 1).length,
    disputes: Object.values(s.disputes).filter(d => d.open).length,
    actors: live.filter(hasGoal).length
  });
  if (s.stats.length > 400) s.stats.shift();
}
