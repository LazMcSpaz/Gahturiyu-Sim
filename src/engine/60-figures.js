/* promotion: who gets to be a person in the story ---------------------------- */

const MAX_ACTORS = 8;

function sysPromotion(s, r) {
  const actors = Object.values(s.people).filter(p => p.alive && p.prominence > 0);
  // demotion first: an actor with no live goal for a long while sinks back
  for (const p of actors) {
    p.goalAge++;
    const spent = p.goal ? p.goalAge > 30 : p.goalAge > 12;
    if (spent) {
      p.prominence = 0; p.goal = null;
      ev(s, 'settle', 1, `${nameOf(s, p.id)} stopped reaching for anything in particular.`, { person: p.id });
    }
  }
  let live = Object.values(s.people).filter(p => p.alive && p.prominence > 0).length;
  if (live >= MAX_ACTORS) return;

  const opportunities = findOpportunities(s);
  if (!opportunities.length) return;

  const candidates = Object.values(s.people).filter(p => {
    const a = ageOf(s, p);
    return p.alive && p.prominence === 0 && a >= 17 && a <= 68;
  });

  const scored = [];
  for (const p of candidates) {
    for (const o of opportunities) {
      const reach = 1 - clamp(walkDist(s, homeTile(s, p), o.tile) / 14, 0, 0.95);
      const fit = o.wants.reduce((acc, t) => acc + p.traits[t], 0) / o.wants.length;
      const score = fit * 0.55 + reach * 40 + (p.grudges.length ? 12 : 0) + (p.tender && o.kind === 'stone' ? 40 : 0);
      if (score > 76) scored.push({ p, o, score: score + r() * 8 });
    }
  }
  scored.sort((a, b) => b.score - a.score);
  const taken = new Set();
  for (const c of scored) {
    if (live >= MAX_ACTORS) break;
    if (taken.has(c.o.id) || c.p.prominence > 0) continue;
    taken.add(c.o.id);
    c.p.prominence = 1;
    c.p.goal = { kind: c.o.kind, tile: c.o.tile, target: c.o.target, since: s.turn, label: c.o.label };
    c.p.goalAge = 0;
    ev(s, 'rise', 6, `${nameOf(s, c.p.id)} — ${describeTraits(c.p)} — set themselves at ${c.o.label}.`, { person: c.p.id });
    live++;
  }
}

function findOpportunities(s) {
  const out = [];
  for (const b of Object.values(s.buildings)) {
    if (b.state === 'growing' && (!b.tenderId || !s.people[b.tenderId]?.alive))
      out.push({ id: 'o_' + b.id, kind: 'stone', tile: b.tileId, target: b.id, wants: ['loyalty', 'piety'], label: 'the unfinished stone that nobody would take up' });
  }
  for (const d of Object.values(s.disputes)) {
    if (d.open && d.heat > 40)
      out.push({ id: 'o_' + d.id, kind: 'champion', tile: (s.households[d.a]?.claims[0] ?? 0), target: d.id, wants: ['courage', 'grudge'], label: 'the front of the quarrel over ' + d.over });
  }
  const homeless = landless(s);
  if (homeless.length > 1)
    out.push({ id: 'o_ground', kind: 'ground', tile: (homeless[0].claims[0] ?? 0), target: null, wants: ['ambition', 'avarice'], label: 'getting ground for the households with none' });
  const hungry = Object.values(s.households).filter(h => !h.extinct && !h.lodgedWith && h.stores < 1);
  if (hungry.length > 2)
    out.push({ id: 'o_hunger', kind: 'relief', tile: (hungry[0].claims[0] ?? 0), target: null, wants: ['loyalty', 'courage'], label: 'feeding the households that had run out' });
  const rich = Object.values(s.households).filter(h => !h.extinct).sort((a, b) => b.claims.length - a.claims.length)[0];
  if (rich && rich.claims.length >= 3)
    out.push({ id: 'o_seat', kind: 'standing', tile: rich.claims[0], target: rich.id, wants: ['ambition', 'avarice'], label: 'the standing the ' + rich.name + ' household holds' });
  if (s.shrine && (s.shrine.devotion < 42 || !s.shrine.keeperId))
    out.push({ id: 'o_shrine', kind: 'shrine', tile: s.shrine.tileId, target: null, wants: ['piety', 'loyalty'], label: 'the state the ' + s.shrine.god + ' stone has been let fall into' });
  const shamed = Object.values(s.households).filter(h => !h.extinct && reputeOf(s, h) < -6)[0];
  if (shamed)
    out.push({ id: 'o_name', kind: 'name', tile: (shamed.claims[0] ?? 0), target: shamed.id, wants: ['loyalty', 'courage'], label: 'getting the ' + shamed.name + ' name back to something people will say out loud' });
  return out;
}

function describeTraits(p) {
  const t = p.traits;
  const words = [];
  if (t.ambition > 70) words.push('reaching');
  if (t.grudge > 70) words.push('a keeper of injuries');
  if (t.piety > 70) words.push('devout to a fault');
  if (t.avarice > 70) words.push('close with what they have');
  if (t.loyalty > 70) words.push('immovable about their own');
  if (t.courage > 70) words.push('unafraid');
  if (p.tender) words.push('able to work the stone');
  return words.length ? words.slice(0, 2).join(' and ') : 'nobody anyone had noticed';
}

/* what the named cast actually does ------------------------------------------ */

function sysActors(s, r) {
  s.pursuitLines = 0;
  for (const p of Object.values(s.people)) {
    if (!p.alive || p.prominence === 0 || !p.goal) continue;
    const g = p.goal;
    const hh = s.households[p.householdId];
    if (!hh) continue;
    const before = s.log.length;

    if (g.kind === 'stone') {
      const b = s.buildings[g.target];
      if (!b || b.state !== 'growing') { p.goal = null; continue; }
      if (!b.tenderId && p.tender) {
        b.tenderId = p.id;
        ev(s, 'act', 5, `${nameOf(s, p.id)} put their hands to the stalled stone. It answered slowly.`, { person: p.id });
        p.goal = null; p.goalAge = 0; p.prominence = 2; p.deeds.push('took up abandoned stone');
      } else if (!p.tender && chance(r, 0.3)) {
        ev(s, 'act', 3, pick(r, [
          `${nameOf(s, p.id)} spent another season trying to coax a stone they had no gift for. It did not move.`,
          `${nameOf(s, p.id)} has been going up to the stalled stone alone and telling nobody what they do there.`,
          `${nameOf(s, p.id)} went round every house that might have the gift. Two said no and one did not answer the door.`
        ]), { person: p.id });
      }
    }

    else if (g.kind === 'champion') {
      const d = s.disputes[g.target];
      if (!d || !d.open) { p.goal = null; continue; }
      d.heat += p.traits.courage / 100 * 9;
      if (chance(r, 0.08)) {   // pushing too hard can cost you
        p.prominence = Math.max(1, p.prominence - 1);
        hh.standing -= 2;
        remember(s, hh.id, -3, `pushed a quarrel further than the settlement wanted it pushed`);
        ev(s, 'act', 5, `${nameOf(s, p.id)} overplayed it. Two households that had been sympathetic stopped being sympathetic.`, { person: p.id });
      }
      if (chance(r, 0.16)) ev(s, 'act', 4, pick(r, [
        `${nameOf(s, p.id)} would not let the quarrel over ${d.over} rest, and raised it again where it could be heard.`,
        `${nameOf(s, p.id)} walked the boundary again with witnesses, which everyone understood the meaning of.`,
        `${nameOf(s, p.id)} put the matter of ${d.over} to two more households, and neither would say no outright.`,
        `${nameOf(s, p.id)} had the old markers dug up and looked at. They proved nothing either way.`,
        `Nobody could get ${nameOf(s, p.id)} to speak about anything except ${d.over}.`
      ]), { person: p.id });
    }

    else if (g.kind === 'ground') {
      if (chance(r, 0.22)) {
        const holder = Object.values(s.households).filter(h => !h.extinct && h.claims.length > 1 && h.id !== hh.id)
          .sort((a, b) => b.claims.length - a.claims.length)[0];
        const already = Object.values(s.disputes).some(x => x.open && x.a === hh.id && x.b === holder.id);
        if (holder && !already) {
          openDispute(s, r, hh, holder, 'ground standing idle while others wait');
          ev(s, 'act', 5, pick(r, [
            `${nameOf(s, p.id)} began saying openly that the ${holder.name} held more hillside than they could ever grow on.`,
            `${nameOf(s, p.id)} put a number on how much ground the ${holder.name} were sitting on, and repeated the number until others did.`,
            `${nameOf(s, p.id)} stopped being careful about what they said concerning the ${holder.name} and their hillside.`
          ]), { person: p.id });
          p.deeds.push('picked a fight over idle ground');
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
          hh.standing += 3; p.prominence = 2; p.goal = null; p.goalAge = 0;
          remember(s, hh.id, 8, `fed households not their own in a year they had little to spare`);
          ev(s, 'act', 5, `${nameOf(s, p.id)} opened the ${hh.name} stores to households not their own. It was remembered.`, { person: p.id });
          p.deeds.push('fed households not their own');
        }
      } else if (chance(r, 0.18)) {
        ev(s, 'act', 3, pick(r, [
          `${nameOf(s, p.id)} spent the season counting other people's stores out loud, which made them useful and disliked in the same measure.`,
          `${nameOf(s, p.id)} got two households to agree to share a boat. It lasted most of the season.`,
          `${nameOf(s, p.id)} walked the whole settlement asking what each house could spare. Most of them lied.`
        ]), { person: p.id });
      } else if (hh.stores < 2 && p.traits.avarice > 60 && chance(r, 0.25)) {
        const victim = Object.values(s.households).filter(h => !h.extinct && h.stores > 8)[0];
        if (victim) {
          victim.stores -= 4; hh.stores += 3;
          const vh = s.people[victim.headId];
          if (vh) vh.grudges.push({ target: hh.id, cause: 'stores taken from their own in a hard season', turn: s.turn, heat: 55 });
          remember(s, hh.id, -7, `were never quite cleared of what went missing from the ${victim.name} house`);
          ev(s, 'act', 6, `Stores went missing from the ${victim.name} house, and everyone knew who, and nobody could prove it.`, { person: p.id });
          p.deeds.push('took what was not theirs');
        }
      }
    }

    else if (g.kind === 'shrine') {
      const sh = s.shrine;
      if (!sh) { p.goal = null; continue; }
      if (!sh.keeperId && p.traits.piety > 55) {
        sh.keeperId = p.id; sh.keeperName = nameOf(s, p.id); sh.keeperSince = s.turn;
        sh.devotion = Math.min(100, sh.devotion + 6);
        p.prominence = 2; p.goal = null; p.goalAge = 0;
        p.deeds.push('took the keeping of the ' + sh.god + ' stone');
        remember(s, hh.id, 6, `put one of their own to the keeping of the ${sh.god} stone when nobody else would`);
        ev(s, 'act', 6, `${nameOf(s, p.id)} took the keeping of the ${sh.god} stone. It had been let go badly and the first season was mostly clearing.`, { person: p.id });
      } else if (chance(r, 0.25)) {
        sh.devotion = Math.min(100, sh.devotion + 2.5);
        ev(s, 'act', 3, pick(r, [
          `${nameOf(s, p.id)} has been going up to the ${sh.god} stone alone and coming back with opinions about who else does not.`,
          `${nameOf(s, p.id)} got four households to bring something up to the ${sh.god} stone this season. Four is more than last year.`,
          `${nameOf(s, p.id)} would not let the matter of the ${sh.god} stone drop at any table they sat at.`
        ]), { person: p.id });
      }
    }

    else if (g.kind === 'name') {
      const target = s.households[g.target];
      if (!target || target.extinct || reputeOf(s, target) > -2) {
        if (target && !target.extinct) {
          p.prominence = 2; p.goal = null; p.goalAge = 0;
          p.deeds.push('got the ' + target.name + ' name spoken plainly again');
          ev(s, 'act', 6, `Whatever the ${target.name} did, it is no longer the first thing said about them, and ${p.name} is why.`, { person: p.id });
        } else p.goal = null;
        continue;
      }
      if (chance(r, 0.3)) {
        remember(s, target.id, 2, `had someone speak for them when it was not popular to`);
        ev(s, 'act', 4, pick(r, [
          `${nameOf(s, p.id)} spoke for the ${target.name} again, to a room that had heard it before.`,
          `${nameOf(s, p.id)} worked a season on the ${target.name} ground in full view of everyone.`,
          `${nameOf(s, p.id)} put their own name behind the ${target.name} in front of witnesses, which cost them something.`
        ]), { person: p.id });
      }
    }

    else if (g.kind === 'standing') {
      const target = s.households[g.target];
      if (!target || target.extinct) { p.goal = null; continue; }
      hh.standing += 0.8;
      if (chance(r, 0.2)) ev(s, 'act', 3, pick(r, [
        `${nameOf(s, p.id)} has been at every table where anything is decided, and says little at most of them.`,
        `${nameOf(s, p.id)} made sure the ${hh.name} were the ones who lent the boat, again.`,
        `${nameOf(s, p.id)} settled a small thing between two houses that had nothing to do with them.`
      ]), { person: p.id });
      if (standingOf(s, hh) > standingOf(s, target) && chance(r, 0.3)) {
        p.prominence = 2; p.goal = null; p.goalAge = 0;
        ev(s, 'act', 6, `The ${hh.name} household now stands above the ${target.name}, and ${p.name} is the reason. It took years and it was noticed at every step.`, { person: p.id });
        p.deeds.push('raised their house above another');
      }
    }

    // A figure who got nowhere this season is still visibly at it. Without
    // this they vanish from the chronicle for years between outcomes.
    const acted = s.log.slice(before).some(e => e.person === p.id);
    s.pursuitLines = (s.pursuitLines || 0);
    if (!acted && p.goal && s.pursuitLines < 2 && chance(r, 0.34)) {
      s.pursuitLines++;
      const kind = p.goal.kind;
      const pool = {
        stone:    [`${nameOf(s, p.id)} was up at the unfinished stone again, and came down again.`,
                   `Nothing moved on the stone this season. ${p.name} did not stop going.`],
        champion: [`${p.name} raised it once more, and once more it was heard and not answered.`,
                   `The quarrel sat where it was. ${p.name} sat with it.`],
        ground:   [`${p.name} asked three houses about ground this season and got three different kinds of no.`,
                   `${p.name} has started walking the hillside measuring things that are not theirs.`],
        relief:   [`${p.name} kept the count of who had what, which nobody had asked them to do.`,
                   `${p.name} was seen at doors that were not opened.`],
        standing: [`${p.name} did nothing anyone could point at, and was in the room for most of it.`,
                   `The ${hh.name} were a little more central this year than last, by no visible means.`],
        shrine:   [`${p.name} swept the stone alone again. Attendance did not improve.`,
                   `${p.name} kept at the stone through a season when nobody else went near it.`],
        name:     [`${p.name} said the ${(s.households[p.goal.target] || {}).name || 'old'} name plainly in company, and let the silence sit.`,
                   `${p.name} is still at it, and people have started expecting them to be.`]
      }[kind];
      if (pool) ev(s, 'act', 2, pick(r, pool), { person: p.id });
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
  const tenders = Object.values(s.people).filter(p => p.alive && p.tender && ageOf(s, p) >= 16).length;
  const growing = Object.values(s.buildings).filter(b => b.state === 'growing').length;

  const candidates = [];
  if (hungry >= 3) candidates.push({ god: 'Horahìda', domain: 'Ocean & Seas', saw: 'houses going hungry with the water right there', lever: 'bounty' });
  if (hungry >= 4) candidates.push({ god: 'Hiyaḍote', domain: 'Death & Night', saw: 'more mouths than this ground has ever carried', lever: 'fever' });
  if (homeless >= 2 && growing > 0) candidates.push({ god: 'Dodìṭo', domain: 'Stone & Mountains', saw: 'households waiting on stone that will not hurry', lever: 'quicken' });
  if (tenders <= 1) candidates.push({ god: 'Hiṭogiʻa', domain: 'Creation', saw: 'the gift running thin in this settlement', lever: 'strangers' });
  if (hotD >= 1) candidates.push({ god: 'Gìhuqìdu', domain: 'Governance & Order', saw: 'a quarrel that no one will rule on', lever: 'temper' });
  if (openD === 0 && s.turn > 12) candidates.push({ god: 'Yohyeʻ', domain: 'Confusion & Delusion', saw: 'a settlement entirely certain of itself', lever: 'reveal' });
  if (growing >= 3 && hungry === 0) candidates.push({ god: 'Quyìturo', domain: 'Time & Decay', saw: 'a comfortable season, which does not last', lever: 'storm' });

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
    actors: live.filter(p => p.prominence > 0).length
  });
  if (s.stats.length > 400) s.stats.shift();
}
