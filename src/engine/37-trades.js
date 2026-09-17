/* ===========================================================================
   Trades, and how they are handed on.

   A person has one trade. It is learned young, usually from kin, and it does
   not change because the larder filled up — shortage never selects a trade, it
   only shapes choices inside one.

   Three tiers. Tier 1 needs nothing and is where anyone lands who was not
   apprenticed by eighteen. Tier 2 needs a master. Tier 3 needs a master and a
   room to work in, which means a stage-2 building — a house about forty-five
   years old. That last is the quiet gate on the whole class system, and it is
   made of nothing but time.
   =========================================================================== */

const TRADES = {
  // tier 1 — no master, no room, and the floor everyone else stands on
  fieldhand:  { tier: 1, verb: 'works the grazing' },
  boathand:   { tier: 1, verb: 'crews a boat' },
  quarrier:   { tier: 1, verb: 'cuts stone' },
  miner:      { tier: 1, verb: 'digs ore' },
  woodcutter: { tier: 1, verb: 'cuts timber' },
  hauler:     { tier: 1, verb: 'hauls' },

  // tier 2 — a master, but the work happens where the work is
  roper:      { tier: 2, years: 3,  verb: 'lays rope', want: 0.03, near: SHORE },
  tanner:     { tier: 2, years: 4,  verb: 'tans hides', want: 0.025 },
  carrier:    { tier: 2, years: 4,  verb: 'carries goods between houses', want: 0.025 },
  mason:      { tier: 2, years: 5,  verb: 'mends stone', want: 0.03, near: CRAG },
  /* The stone is the exception at every edge. It takes ten years, only one in
     eight can learn it at all, and four in five who begin do not finish — so a
     tender teaches until they are old and takes them young, or the craft simply
     runs out and nothing is ever built again. */
  tender:     { tier: 2, years: 10, verb: 'grows houses', aptitude: true, dropout: 0.012,
                want: 0.055, minAge: 11, maxAge: 24, masterMax: 78 },

  // tier 3 — a master and a room. These arrive late, as houses age into workshops.
  weaver:     { tier: 3, years: 4, room: true, want: 0.015, verb: 'weaves' },
  tailor:     { tier: 3, years: 4, room: true, want: 0.015, verb: 'cuts and sews' },
  herbalist:  { tier: 3, years: 5, room: true, want: 0.015, verb: 'keeps the remedies' },
  joiner:     { tier: 3, years: 5, room: true, want: 0.015, verb: 'fits timber' },
  carver:     { tier: 3, years: 6, room: true, want: 0.015, verb: 'carves', near: CRAG },
  smith:      { tier: 3, years: 7, room: true, want: 0.015, verb: 'works metal' }
};

const TIER1 = Object.keys(TRADES).filter(k => TRADES[k].tier === 1);
const TEACHABLE = Object.keys(TRADES).filter(k => TRADES[k].tier > 1);

const APPRENTICE_MIN = 12, APPRENTICE_MAX = 17;
const MASTER_MIN = 25, MASTER_MAX = 60;
const MASTER_REST = 8;          // turns before a master takes another
const TAKE_ON_CHANCE = 0.09;    // per master per season, once free
const DROPOUT = 0.02;           // per season; tender is worse, see TRADES
const REFUSE_AT = -30;          // a tie this bad either way and the offer fails
/* The design put the floor at 60 with a walk penalty of 3 a tile. Measured
   against a real settlement that let through 5% of master-and-child pairs, and
   fifteen of the sixteen were the master's own household — so every craft
   became kin-only and died with any master who had no child of the right age.
   At 40, with distance softened, 13% clear and two thirds of those are from
   another house, which is what the refusal cascade needs to have anything to
   refuse. */
const CLUSTER_PULL = 12;    // for living where the work already is
const SCORE_FLOOR = 40;
const WALK_COST = 1.5;

/* PLACEHOLDER, until the economy exists (slice 5).

   The food model assumes every pair of hands fishes or herds. That was true
   when there were four kinds of work; with sixteen it starves the settlement —
   a run diversified into crafts and dropped from a hundred and thirty adults
   to fifteen, because a quarrier, a smith and a tanner all produced nothing
   anyone could eat.

   Until goods and exchange exist, everybody also feeds themselves: a quarrier
   keeps a plot and a line in the water, a smith rather less. This is a lie the
   shape of the truth, and slice 5 should delete it outright — when a smith's
   metal can be traded for a fieldhand's food, none of this is needed. */
const FOOD_SHARE = {
  fieldhand: 1, boathand: 1,
  quarrier: 0.55, miner: 0.55, woodcutter: 0.55, hauler: 0.55
};

function foodShare(trade) {
  if (!trade) return 0.8;                        // a child or an idler still gathers
  if (FOOD_SHARE[trade] !== undefined) return FOOD_SHARE[trade];
  return tradeTier(trade) === 3 ? 0.3 : 0.4;     // a craft leaves less of the day
}

function tradeInfo(k) { return TRADES[k] || null; }
function tradeTier(k) { return (TRADES[k] || {}).tier || 0; }

/* Can this person work their trade this season? A room-needing trade with no
   room is a trade you hold and cannot practise.

   The room does not have to be theirs. Tying it to the worker's own household
   killed every fine craft within forty years — not because the masters died,
   but because households split and marry and a smith ends up under a roof that
   has no workshop, while three stand empty across the quarter. What is scarce
   is the room, not the deed to it. So: your own house, your kin's, or a house
   that thinks well enough of you to let you in.

   That keeps the gate where the design wants it — the finer trades still need
   a forty-five-year-old house to exist near — while letting a craft outlive one
   family's fortunes. */
const WORKSHOP_WELCOME = 25;   // a tie this warm and they will have you

function workshopFor(s, p) {
  const own = s.households[p.householdId];
  if (own && hasWorkshop(s, own)) return own;
  for (const hh of Object.values(s.households)) {
    if (hh.extinct || hh.id === p.householdId || !hasWorkshop(s, hh)) continue;
    const kin = own && firstSyllable(own.name) === firstSyllable(hh.name);
    if (kin || tieTo(p, hh.id) >= WORKSHOP_WELCOME) return hh;
  }
  return null;
}

function canPractise(s, p) {
  if (!p.trade) return false;
  const info = TRADES[p.trade];
  if (!info || !info.room) return true;
  if (workshopFor(s, p)) return true;
  /* The last herbalist on the coast is not left standing outside for want of a
     room. Somebody clears a corner. */
  return endangered(s, p.trade)
    && Object.values(s.households).some(h => !h.extinct && hasWorkshop(s, h));
}

/* A craft with one master left teaches as hard as it can; a crowded one barely
   bothers. Without this the long trainings simply die: a tender takes ten years
   and loses four in five, so a settlement that dips to two tenders never climbs
   back, and then nothing is ever built again. Scarcity has to push back. */
function eagerness(s, trade, adults) {
  const want = wantOf(s, trade, adults);
  const { holders, learning } = holdersOf(s, trade);
  const held = holders.length;
  /* A settlement of eighty adults does not need twenty-five tanners. The first
     version only pushed upward when a craft was scarce and never pushed back
     when it was not — and because the rate is per master, a crowded craft kept
     recruiting simply by having many masters. Past its share of the settlement
     a craft stops taking anyone, which is what `want` was always meant to say. */
  if (held + learning >= want * 1.3) return 0;
  // measured in working life left, not in heads
  return clamp(want / Math.max(0.5, effectiveHolders(s, trade)), 0.4, 6);
}

/* A settlement one death away from losing a craft behaves differently, and the
   first version did not. Measured at fifty years: tailor held by two with none
   able to teach, herbalist two and none, carver one and none — the holders were
   locked out by age or by having no workshop to teach in, so the craft simply
   expired while everyone watched.

   A town without a smith puts a high price on getting one. When a craft is down
   to its last hands: the old master teaches anyway, somebody finds them a room,
   and they will take an apprentice they would otherwise have turned away. */
/* Counting heads is not counting a craft. Five tenders aged forty-five to
   seventy-four look like plenty and are not: they were all seeded together,
   they stopped teaching because five seemed enough, and then they died within a
   decade of each other and every half-trained apprentice lost their master
   partway through a ten-year training. That is how a settlement loses the
   ability to build without anybody making a mistake.

   So a holder counts for as much working life as they have left. An old smith
   is most of a smith today and almost none of one in ten years, and a town that
   can see that trains a replacement while there is still somebody to do the
   training. */
/* Who holds or is learning a craft, worked out once a season per craft. This
   and the two functions that ask it were a quarter of every turn, because
   canPractise asks it for every roomless holder and the apprentice pass asks
   it for every master. Every site that changes who holds or is learning a
   craft clears the memo; the weighting below is not memoised, because whether
   a holder can reach a room turns on ties that move within a season. */
function holdersOf(s, trade) {
  if (!s._eh) s._eh = {};
  if (s._eh[trade]) return s._eh[trade];
  const holders = [], learners = [];
  for (const p of livingPeople(s)) {
    if (!p.alive) continue;
    if (p.learning && p.learning.trade === trade) { learners.push(p); continue; }
    if (p.trade === trade) holders.push(p);
  }
  return (s._eh[trade] = { holders, learners, learning: learners.length });
}

function effectiveHolders(s, trade) {
  const ceiling = (TRADES[trade] || {}).masterMax || MASTER_MAX;
  const { holders, learners } = holdersOf(s, trade);
  /* A learner is worth what they have learned. Counted flat at a half each,
     three ten-year apprentices in their first year read as a master and a
     half, and a coast with four tenders aged fifty-seven to sixty-seven read
     as safe — then lost all four inside fourteen seasons and had no tender at
     all until the apprentices finished alone. */
  let n = 0;
  for (const q of learners) n += 0.2 + 0.5 * Math.min(1, q.learning.progress || 0);
  for (const p of holders) {
    let w = clamp((ceiling - ageOf(s, p)) / 25, 0.12, 1);
    /* Somebody with no room to work in holds the craft in name. Counting them
       as a whole one lets the settlement read itself as supplied while nothing
       is made. workshopFor rather than canPractise, which asks this back. */
    if ((TRADES[trade] || {}).room && !workshopFor(s, p)) w *= 0.25;
    n += w;
  }
  return n;
}

function wantOf(s, trade, adults) {
  const base = Math.max(2, Math.round(((TRADES[trade] || {}).want || 0.02) * adults));
  /* A mason is the one craft whose work can be counted directly: it is the
     stone that needs mending, not the number of people. At a flat fraction of
     adults a settlement kept two masons over thirty failing buildings and lost
     ground every season for a century, and nothing in it could see that as a
     shortage. A town whose houses are falling in should be training masons for
     the same reason a town without a smith should be training a smith. */
  if (trade !== 'mason') return base;
  const failing = Object.values(s.buildings).filter(b => b.state === 'mature'
    && conditionOf(b) < MEND_BELOW).length;
  /* With a ceiling. Unchecked, a settlement of seventy-eight people answered
     failing stone with twelve masons — a seventh of its adults doing work that
     makes nothing, while the quarry and the copse it needed went uncut and the
     houses went on falling anyway. */
  return clamp(Math.max(base, Math.round(failing / 3)), 2, Math.round(adults * 0.06));
}

function endangered(s, trade) {
  const adults = livingPeople(s).filter(p => ageOf(s, p) >= 18).length;
  return effectiveHolders(s, trade) <= Math.max(1.5, wantOf(s, trade, adults) * 0.45);
}

function mastersOf(s, trade) {
  return livingPeople(s).filter(p => p.trade === trade
    && ageOf(s, p) >= MASTER_MIN && ageOf(s, p) <= MASTER_MAX);
}

/* --- what a master weighs --------------------------------------------------- */

/* Standing is not a fixed bonus — the master decides its sign, and that is the
   whole mechanism for an unlikely apprentice. Most take the well-connected
   child. A compassionate master takes the one who needs it. A proud one will
   not be seen serving the big house. */
function standingWeight(m, theirStanding) {
  if (m.traits.loyalty > 70 && m.traits.avarice < 40) return -0.35;
  if (m.traits.ambition > 70 && m.traits.loyalty < 40) return theirStanding > 30 ? -0.50 : 0.35;
  return 0.35;
}

function apprenticeScore(s, m, p, scarceGift) {
  const own = s.households[m.householdId];
  const theirs = s.households[p.householdId];
  if (!theirs) return -1;
  const info = TRADES[m.trade] || {};
  let v = 50;
  /* The gift counts for the craft that needs it and for nothing else. Left as a
     flat bonus it made aptitude-holders the most attractive apprentice for
     every trade, so ropers and masons took exactly the children the stone
     needed — and the settlement lost the ability to build without one decision
     ever looking wrong. When tenders are thin, other masters leave them alone. */
  if (p.aptitude) v += info.aptitude ? 40 : (scarceGift ? -45 : 0);
  if (own && p.householdId === m.householdId) v += 25;
  else if (own && firstSyllable(own.name) === firstSyllable(theirs.name)) v += 12;
  v -= walkDist(s, homeTile(s, p), homeTile(s, m)) * WALK_COST;
  v += standingWeight(m, houseStanding(s, theirs, 'trade')) * houseStanding(s, theirs, 'trade');
  v -= holdsAgainst(s, m, theirs.id) * 0.8;
  const head = s.people[theirs.headId];
  if (head && own) v -= holdsAgainst(s, head, own.id) * 0.6;
  v += tieTo(m, theirs.id) * 0.5;
  /* Trades cluster where proximity makes sense, and the ground-worked ones
     already do it on their own — a quarrier is measured at nought tiles from
     a crag against a settlement average of half a one. The taught trades do
     not, and the roper was the proof: rope is rigging, and ropers came out
     six to eleven tiles from the water on a coast where the average house is
     six. A master leans toward somebody who already lives where the work is. */
  if (info.near !== undefined) {
    const t = s.tiles[homeTile(s, p)];
    if (t && t.t === info.near) v += CLUSTER_PULL;
  }
  return v;
}

/* --- taking someone on ------------------------------------------------------ */

function sysApprentice(s, r) {
  const adults = livingPeople(s).filter(p => ageOf(s, p) >= 18).length;
  const learning = new Set(Object.values(s.people)
    .filter(p => p.alive && p.learning).map(p => p.learning.from));

  /* Masters are offered the children in order of how close their craft is to
     being lost. A settlement with one smith and eleven ropers does not let the
     ropers pick first — everyone can see which of the two is about to go. */
  const eager = {};
  for (const k of TEACHABLE) eager[k] = eagerness(s, k, adults);
  const masters = Object.values(s.people)
    .filter(m => m.alive && m.trade && tradeTier(m.trade) >= 2)
    .sort((a, b) => eager[b.trade] - eager[a.trade]);

  for (const m of masters) {
    const info0 = TRADES[m.trade];
    const last = endangered(s, m.trade);
    const a = ageOf(s, m);
    const ceiling = last ? 78 : (info0.masterMax || MASTER_MAX);
    if (a < (last ? 20 : MASTER_MIN) || a > ceiling) continue;
    if (learning.has(m.id)) continue;                            // one at a time
    if (s.turn - (m.lastApprentice || -999) < MASTER_REST) continue;
    const info = TRADES[m.trade];
    if (info.room && !canPractise(s, m)) continue;               // no room, nothing to teach in
    if (!last && !chance(r, TAKE_ON_CHANCE * eager[m.trade])) continue;
    if (last && !chance(r, 0.5)) continue;

    // a craft that takes ten years cannot also draw only from five year-groups
    const minAge = info.minAge || APPRENTICE_MIN;
    const maxAge = last ? Math.max(24, info.maxAge || APPRENTICE_MAX) : (info.maxAge || APPRENTICE_MAX);
    const pool = livingPeople(s).filter(p => !p.trade && !p.learning
      && ageOf(s, p) >= minAge && ageOf(s, p) <= maxAge
      && (!info.aptitude || p.aptitude));
    if (!pool.length) continue;

    const tenders = livingPeople(s).filter(p => p.trade === 'tender').length;
    const scarceGift = tenders < Math.max(2, Math.round(TRADES.tender.want * adults));
    const floor = last ? 12 : SCORE_FLOOR;   // the last master takes who they can get
    const ranked = pool.map(p => ({ p, v: apprenticeScore(s, m, p, scarceGift) + (r() - 0.5) * 8 }))
      .filter(x => x.v >= floor)
      .sort((a2, b2) => b2.v - a2.v);
    if (!ranked.length) continue;

    /* Refusal cascades. Better than a randomness dial, because they produce a
       reason the chronicle can print: the unlikely apprentice is the one who
       was left after three houses said no. */
    let refused = 0, taken = null;
    for (const cand of ranked.slice(0, 5)) {
      const theirs = s.households[cand.p.householdId];
      const head = theirs && s.people[theirs.headId];
      const bad = tieTo(m, theirs.id) <= REFUSE_AT
        || (head && s.households[m.householdId] && tieTo(head, m.householdId) <= REFUSE_AT);
      if (bad) { refused++; continue; }
      taken = cand.p; break;
    }
    if (!taken) continue;

    const kin = taken.householdId === m.householdId;
    touchTrades(s);
    taken.learning = { from: m.id, name: nameOf(s, m.id), trade: m.trade,
                       since: s.turn, progress: 0.01, kin };
    m.lastApprentice = s.turn;
    learning.add(m.id);

    /* An apprenticeship starting is the system working, not news: reported
       every time it was one line a season for a century. What the design says
       to print is the unlikely apprentice and why — a master refused first, a
       craft down to its last hands taking whoever it can, or a child from a
       house well below the master's own. The rest is weight one, which is
       below the chronicle's floor and still in the record. */
    const theirs = s.households[taken.householdId], own = s.households[m.householdId];
    const gap = theirs && own ? compositeOf(s, own) - compositeOf(s, theirs) : 0;
    const scarce = endangered(s, m.trade);
    const why = refused ? `having been refused by ${refused} house${refused > 1 ? 's' : ''} first`
      : scarce ? `— the craft is down to its last hands and takes who it can`
      : gap > 25 ? `from a house well below their own`
      : null;
    ev(s, 'teach', why ? (refused || scarce ? 6 : 5) : 1,
      why ? `${nameOf(s, m.id)} took ${nameOf(s, taken.id)} on to learn ${m.trade}, ${why}.`
          : `${nameOf(s, m.id)} took ${nameOf(s, taken.id)} on to learn ${m.trade}${kin ? '. Same household.' : ', from a different household.'}`,
      { person: taken.id, household: taken.householdId });
  }
}

/* --- learning it ------------------------------------------------------------ */

function sysLearn(s, r) {
  for (const p of Object.values(s.people)) {
    if (!p.alive || !p.learning) continue;
    const L = p.learning;
    const info = TRADES[L.trade] || TRADES.tender;
    const master = s.people[L.from];

    /* The master dies and the training is half a thing. This is how a craft
       actually dies: not for want of apprentices but because a ten-year
       training loses its teacher partway and the half-taught give up.

       When the craft is down to its last hands, the settlement has every
       reason to let them try anyway — there is nobody else, and a poor smith
       is a great deal better than none. */
    if (!master || !master.alive) {
      L.orphaned = (L.orphaned || 0) + 1;
      const last = endangered(s, L.trade);
      if (L.orphaned === 1)
        ev(s, 'teach', 5, `${nameOf(s, p.id)} was still learning ${L.trade} when ${L.name} died. The training is unfinished.`, { person: p.id });

      // somebody else who holds the craft may take them on rather than lose it
      const takenOver = Object.values(s.people).find(q => q.alive && q.trade === L.trade
        && q.id !== p.id && ageOf(s, q) >= 20 && canPractise(s, q));
      if (takenOver && (last || chance(r, 0.25))) {
        L.from = takenOver.id; L.name = nameOf(s, takenOver.id); L.orphaned = 0;
        ev(s, 'teach', 4, `${nameOf(s, takenOver.id)} took over teaching ${nameOf(s, p.id)} the ${L.trade}.`, { person: p.id });
        continue;
      }
      if (L.progress > (last ? 0.4 : 0.62) && chance(r, last ? 0.45 : 0.10)) {
        finishTrade(s, p, L, true);
      } else if (chance(r, last ? 0.01 : 0.05)) {
        ev(s, 'teach', 4, `${nameOf(s, p.id)} gave up on ${L.trade} after ${Math.max(1, Math.round((s.turn - L.since) / 4))} years.`, { person: p.id });
        p.learning = null;
  touchTrades(s);
      touchTrades(s);
        touchTrades(s);
      }
      continue;
    }

    if (chance(r, info.dropout || DROPOUT)) {
      ev(s, 'teach', 3, `${nameOf(s, p.id)} stopped being taught ${L.trade} by ${L.name}.`, { person: p.id });
      p.learning = null;
  touchTrades(s);
      touchTrades(s);
      continue;
    }

    L.progress += (1 / ((info.years || 4) * 4)) * (0.8 + p.traits.loyalty / 300) * (L.kin ? 1.15 : 1);
    if (L.progress >= 1) finishTrade(s, p, L, false);
  }
}

function finishTrade(s, p, L, alone) {
  const yrs = Math.max(1, Math.round((s.turn - L.since) / 4));
  setTrade(s, p, L.trade);
  p.learning = null;
  touchTrades(s);
  remember(s, p.householdId, 4, `taught a ${L.trade} of their own`);
  shiftStanding(p, 'trade', 20);
  ev(s, 'teach', 6, alone
    ? `${nameOf(s, p.id)} finished learning ${L.trade} alone, after their teacher died.`
    : `${nameOf(s, p.id)} can work as a ${L.trade} now, taught by ${L.name} over ${yrs} years.`,
    { person: p.id, household: p.householdId });
}

function setTrade(s, p, trade) {
  p.trade = trade;
  touchTrades(s);
  p.tender = trade === 'tender';          // the rest of the engine still asks this
  if (p.role === 'none' && tradeTier(trade) > 1) p.role = trade;
}

/* --- the floor -------------------------------------------------------------- */

/* Anybody not apprenticed by eighteen works. What they work is whatever their
   household's ground gives them, and failing that whatever the settlement is
   short of — which is how a household with no ground ends up hauling. */
function sysUnskilled(s, r) {
  for (const p of Object.values(s.people)) {
    if (!p.alive || p.trade || p.learning) continue;
    const age = ageOf(s, p);
    if (age < 18) continue;
    /* Someone with the gift is not sent to the grazing at eighteen while the
       craft that needs them will still take a twenty-two-year-old. Without
       this the floor swallows every aptitude-holder before a tender can reach
       them, and the settlement quietly loses the ability to build. */
    if (p.aptitude && age <= (TRADES.tender.maxAge || 24)
        && livingPeople(s).some(m => m.trade === 'tender')) continue;
    const hh = s.households[p.householdId];
    if (!hh) continue;
    const claims = hh.claims || [];
    const fish = claims.some(c => s.tiles[c].fish > 0.25);
    const graze = claims.some(c => s.tiles[c].graze > 0.25);
    const wood = claims.some(c => (s.tiles[c].wood || 0) > 0.3);
    const crag = claims.some(c => s.tiles[c].t === CRAG);
    /* A household feeds itself first and works the rest of its ground after.
       Without the second clause every hand on a pastoral coast goes to the
       grazing and the wood is never cut, which is how the settlement came to
       hold no timber at all. */
    const has = (...ts) => hh.members.some(i => {
      const q = s.people[i];
      return q && q.alive && ts.includes(q.trade);
    });
    /* One pair of hands on the wood, not every spare pair: without the second
       clause a pastoral household put everyone after the first into the
       copse, and thirty-seven of a hundred adults came out as woodcutters
       holding nine hundred lots of timber nobody wanted. */
    /* And a household answers what the settlement is short of, once it has
       somebody on food. Without this the ground decided everything: a coast
       whose houses were coming down for want of stone and timber went on
       putting every spare hand onto the grazing, because nothing in the
       assignment could see a shortage. */
    const fed = has('fieldhand', 'boathand');
    const shortOf = s.shortMaterial || {};
    setTrade(s, p,
        wood && !has('woodcutter') && (fed || !(fish || graze)) ? 'woodcutter'
      : crag && shortOf.stone && fed && !has('quarrier') ? 'quarrier'
      : fish ? 'boathand'
      : graze ? 'fieldhand'
      : crag ? (chance(r, 0.5) ? 'quarrier' : 'miner')
      : pick(r, ['woodcutter', 'hauler', 'quarrier']));
  }
}

/* --- a trade can be lost ---------------------------------------------------- */

/* The design is firm that a lost craft cannot be re-invented: no living master,
   no way to learn it. That leaves exactly one honest way back, and it is the one
   the design already names — somebody arrives who knows it. */
function tradeForIncomer(s, r) {
  const missing = TEACHABLE.filter(k =>
    !livingPeople(s).some(p => p.trade === k));
  if (missing.length && chance(r, 0.5)) return pick(r, missing);
  const thin = TEACHABLE.filter(k =>
    livingPeople(s).filter(p => p.trade === k).length <= 1);
  return thin.length ? pick(r, thin) : pick(r, TEACHABLE);
}

/* A craft needs a room, and the room can go: the household that lent it dies
   out, a tie cools, a house is seized. Nothing used to revisit that, so a
   carver with nowhere to work stayed a carver for forty years and made
   nothing at all. After three years of it they go back to whatever their
   household's ground gives them, and the craft reads as thin, which is the
   thing that gets somebody trained. */
const ROOMLESS_SEASONS = 12;

function sysRoomless(s) {
  for (const p of Object.values(s.people)) {
    if (!p.alive || !p.trade) continue;
    if (!(TRADES[p.trade] || {}).room) { if (p.noRoom) p.noRoom = 0; continue; }
    if (canPractise(s, p)) { if (p.noRoom) p.noRoom = 0; continue; }
    p.noRoom = (p.noRoom || 0) + 1;
    if (p.noRoom < ROOMLESS_SEASONS) continue;
    const trade = p.trade;
    ev(s, 'notender', 5, `${nameOf(s, p.id)} has had nowhere to work as a ${trade} for ${Math.round(p.noRoom / 4)} years. They have gone back to common work.`,
       { person: p.id, household: p.householdId });
    p.trade = null;
    touchTrades(s);
    p.tender = false;
    p.noRoom = 0;
    if (p.role === trade) p.role = 'none';
    shiftStanding(p, 'trade', -12);
  }
}

function sysTradeLoss(s, r) {
  s.lostTrades = s.lostTrades || {};
  for (const k of TEACHABLE) {
    const held = livingPeople(s).some(p => p.trade === k);
    if (held) { if (s.lostTrades[k]) delete s.lostTrades[k]; continue; }
    const learners = livingPeople(s).some(p => p.learning && p.learning.trade === k);
    if (learners || s.lostTrades[k]) continue;
    s.lostTrades[k] = s.turn;
    ev(s, 'notender', 6, `There is nobody left on this coast who can work as a ${k}.`, {});
  }
}
