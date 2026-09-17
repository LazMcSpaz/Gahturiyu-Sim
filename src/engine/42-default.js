/* ===========================================================================
   What happens to a household that cannot pay.

   This is where the form of government finally matters mechanically. The form
   caps how cruel enforcement may be; the character of whoever holds the seat
   picks within that cap. A sole ruler can do anything they like. A senate has
   to agree with itself first.

   Seizure is the most violent act in this world and is written that way. A
   home takes fifteen to twenty years to grow and a workshop forty-five. Taking
   one is not foreclosure — it is taking a century of somebody's family and
   handing it to a creditor, and it cannot be replaced in a lifetime.
   =========================================================================== */

const RUIN_AT = 20;          // ḍaqu owed with nothing coming in
const RUIN_SEASONS = 8;      // and this long unable to pay any of it

/* How far each form will go. The ladder, mildest first. */
const HARSHNESS = {
  senate:   ['forbear', 'seize'],
  tribunal: ['forbear', 'seize', 'gaol'],
  sole:     ['forbear', 'seize', 'gaol', 'kill']
};

function rulingSeat(s) {
  const order = ['arbiter', 'ruler', 'tribune', 'senator'];
  for (const k of order) {
    const held = officeHolders(s, k);
    if (held.length) return held[0];
  }
  return null;
}

/* Cruelty is the person, bounded by the office. A vengeful, grasping judge
   with a grudge against the house before them reaches for the far end of what
   the form allows; a loyal one stops at the near end. */
function chooseRuling(s, r, judge, debtor) {
  const allowed = HARSHNESS[s.government] || HARSHNESS.senate;
  if (!judge) return 'forbear';
  let cruel = (judge.traits.grudge * 0.4 + judge.traits.avarice * 0.35
             - judge.traits.loyalty * 0.3 + holdsAgainst(s, judge, debtor.id) * 0.6
             - bondWith(judge, debtor.id) * 0.5) / 45;
  cruel += (r() - 0.5) * 0.35;
  const idx = clamp(Math.round(cruel * (allowed.length - 1)), 0, allowed.length - 1);
  return allowed[idx];
}

/* --- a neighbour may pay it off ---------------------------------------------
   The warmest mechanic here, and it spends a tie rather than reading one. It
   costs the rescuer real stores and leaves the rescued owing something that is
   not money. And a proud house would rather lose the roof than be beholden.
   -------------------------------------------------------------------------- */

function findRescuer(s, debtor, amount) {
  let best = null, bestTie = 0;
  for (const h of Object.values(s.households)) {
    if (h.extinct || h.id === debtor.id) continue;
    const head = s.people[h.headId];
    if (!head || !head.alive) continue;
    const tie = tieTo(head, debtor.id);
    if (tie < 35) continue;
    if (h.stores < amount * 0.5 + 8) continue;
    if (tie > bestTie) { bestTie = tie; best = h; }
  }
  return best;
}

function tooProud(s, debtor) {
  const head = s.people[debtor.headId];
  if (!head) return false;
  return head.traits.ambition > 62 && head.traits.loyalty < 45;
}

/* --- the ruling --------------------------------------------------------------- */

function sysDefault(s, r) {
  const list = debtsOf(s);
  s.mercyRun = s.mercyRun || 0;

  for (const hh of Object.values(s.households)) {
    if (hh.extinct || hh.lodgedWith) continue;
    const owe = totalOwed(s, hh.id);
    if (owe < RUIN_AT) { hh.ruinFor = 0; continue; }
    if (hh.stores > 6) { hh.ruinFor = 0; continue; }          // still paying something
    hh.ruinFor = (hh.ruinFor || 0) + 1;
    if (hh.ruinFor < RUIN_SEASONS) continue;
    if (!officeHolders(s, 'arbiter').length && !rulingSeat(s)) continue;
    hh.ruinFor = 0;

    // somebody may step in before it ever reaches a judgement
    const rescuer = findRescuer(s, hh, owe);
    if (rescuer) {
      if (tooProud(s, hh)) {
        ev(s, 'dispute', 6, `The ${rescuer.name} offered to clear the ${hh.name} debts. The ${hh.name} would not have it.`,
           { household: hh.id });
        remember(s, hh.id, 2, `refused to be bought out of their debts`);
      } else {
        const paid = Math.min(owe, rescuer.stores - 8);
        rescuer.stores -= paid;
        for (const d of list) if (d.from === hh.id) {
          const take = Math.min(d.amount, paid);
          d.amount -= take;
        }
        const rh = s.people[hh.headId], gh = s.people[rescuer.headId];
        if (rh) shiftTie(s, rh, rescuer.id, 45, `paid off this household's debts when nobody else would`);
        if (gh) shiftStanding(gh, 'kin', 16);
        remember(s, rescuer.id, 8, `paid another household's debts out of their own store`);
        ev(s, 'hardship', 6, `The ${rescuer.name} paid off what the ${hh.name} owed.`, { household: hh.id });
        continue;
      }
    }

    const judge = rulingSeat(s);
    const ruling = chooseRuling(s, r, judge, hh);
    const who = judge ? nameOf(s, judge.id) : 'the settlement';
    applyRuling(s, r, hh, ruling, who, judge, owe);
  }
}

function applyRuling(s, r, hh, ruling, who, judge, owe) {
  const list = debtsOf(s);
  const creditors = list.filter(d => d.from === hh.id && d.amount > 0);

  if (ruling === 'forbear') {
    /* Mercy is not free. A settlement whose rulings never bite dries up
       lending, because nobody extends what they will not get back — so a
       merciful place is gentler and poorer. */
    for (const d of creditors) d.amount *= 0.45;
    s.mercyRun++;
    s.creditTight = Math.min(1, (s.creditTight || 0) + 0.22);
    /* Being let off is worth something, but not much: at twelve a household it
       paid for itself many times over, and a settlement full of forgiven
       debtors drove legitimacy *up* through a famine. */
    for (const id of hh.members) {
      const p = s.people[id];
      if (p && p.alive) shiftStanding(p, 'government', 5);
    }
    ev(s, 'arbitration', 6, `${who} let the ${hh.name} off what they could not pay.`, { household: hh.id });
    return;
  }

  if (ruling === 'seize') {
    const b = Object.values(s.buildings).find(x => x.householdId === hh.id && x.state === 'mature');
    const winner = creditors.sort((a, c) => c.amount - a.amount)[0];
    const to = winner && s.households[winner.to];
    if (!b || !to) { applyRuling(s, r, hh, 'forbear', who, judge, owe); return; }

    b.householdId = to.id;
    touchBuildings(s);
    hh.buildingId = null;
    hh.lodgedWith = to.id;
    if (!to.buildingId) to.buildingId = b.id;
    for (const d of creditors) d.amount = 0;

    /* A century of somebody's family, handed to a creditor. The settlement
       should answer this closer to how it answers a killing than a debt. */
    const head = s.people[hh.headId];
    if (head) {
      shiftTie(s, head, to.id, -95, `took the roof from over this household for a debt`);
      shiftTie(s, head, (judge && s.people[judge.id]) ? s.people[judge.id].householdId : to.id, -60,
               `gave away this household's house for a debt`);
    }
    for (const id of hh.members) {
      const p = s.people[id];
      if (p && p.alive) { shiftStanding(p, 'government', -34); shiftStanding(p, 'quarter', -12); }
    }
    remember(s, to.id, -9, `took a house off a household that could not pay`);
    remember(s, hh.id, -4, `lost their house for debt`);
    s.creditTight = Math.max(0, (s.creditTight || 0) - 0.25);
    ev(s, 'feud', 7, `${who} gave the ${hh.name} house to the ${to.name} for a debt of ${Math.round(owe)} ḍaqu. It took ${Math.round((s.turn - b.startTurn) / 4)} years to grow.`,
       { household: hh.id, building: b.id });
    return;
  }

  if (ruling === 'gaol') {
    const live = hh.members.map(i => s.people[i]).filter(p => p && p.alive && ageOf(s, p) >= 16);
    const taken = live.sort((a, b2) => ageOf(s, b2) - ageOf(s, a))[0];
    if (!taken) { applyRuling(s, r, hh, 'seize', who, judge, owe); return; }
    taken.gaoledUntil = s.turn + ri(r, 8, 24);
    for (const d of creditors) d.amount *= 0.6;
    const head = s.people[hh.headId];
    if (head && judge) shiftTie(s, head, s.people[judge.id].householdId, -70, `had one of ours shut up for a debt`);
    for (const id of hh.members) {
      const p = s.people[id];
      if (p && p.alive) shiftStanding(p, 'government', -26);
    }
    ev(s, 'feud', 6, `${who} had ${nameOf(s, taken.id)} shut up over the ${hh.name} debts.`, { household: hh.id, person: taken.id });
    return;
  }

  if (ruling === 'kill') {
    const head = s.people[hh.headId];
    if (!head || !head.alive) { applyRuling(s, r, hh, 'gaol', who, judge, owe); return; }
    for (const d of creditors) d.amount = 0;
    kill(s, head, 'the ruling over their debts');
    /* Almost always a mistake, and the settlement says so. A government that
       kills its debtors bleeds the standing that made its money believable. */
    for (const h of Object.values(s.households)) {
      if (h.extinct) continue;
      for (const id of h.members) {
        const p = s.people[id];
        if (p && p.alive) shiftStanding(p, 'government', -18);
      }
    }
    if (judge) { shiftStanding(judge, 'quarter', -40); shiftStanding(judge, 'kin', -30); }
    remember(s, hh.id, -3, `had their head put to death over a debt`);
    ev(s, 'feud', 7, `${who} had ${nameOf(s, head.id)} put to death over the ${hh.name} debts.`, { household: hh.id });
  }
}

/* Somebody shut up over a debt is not working. */
function sysGaol(s) {
  for (const p of Object.values(s.people)) {
    if (!p.alive || !p.gaoledUntil) continue;
    if (s.turn >= p.gaoledUntil) {
      p.gaoledUntil = 0;
      shiftStanding(p, 'government', -10);
    }
  }
}

function gaoled(p) { return !!(p.gaoledUntil && p.gaoledUntil > 0); }
