/* ===========================================================================
   A house is not finished when it is finished.

   Living stone keeps growing for as long as somebody attends it, and what a
   building *is* changes as it ages. Age is the settlement's most honest
   measure of status, because it cannot be bought or hurried — only kept.

     stage 1  home                       15–20 years
     stage 2  home and workshop          about 45
     stage 3  great house, tavern, or
              something held in common   about 95
     stage 4  a landmark                 rarely, and long after

   The consequence that matters most: tending never ends. A tender is no
   longer precious for twenty years and then idle. They are the reason any
   house ever becomes anything, for as long as it stands.
   =========================================================================== */

const STAGE_TURNS = { 1: 120, 2: 280 };   // turns of tended growth: 30 years, then 70
const CHECKIN_LAPSE = 20;                 // 5 years untended and growth halts
const CHECKIN_DUE = 12;                   // a tender starts thinking about it at 3
const VISITS_PER_TENDER = 2;

const STAGE_WORD = {
  1: 'a home',
  2: 'a home and workshop',
  3: 'a great house',
  4: 'a landmark'
};

function stageOf(b) { return b.stage || 1; }

/* What a stage-3 building is used for. Below stage 3 the question does not
   arise — it is somebody's house. */
function roleOf(b) { return stageOf(b) >= 3 ? (b.role || 'great') : 'home'; }

function roleWord(b) {
  return { great: 'a great house', tavern: 'a tavern', common: 'a common hall',
           house: 'a large house' }[roleOf(b)] || STAGE_WORD[stageOf(b)];
}

/* A workshop is a stage-2 building. The finer trades need one; most trades do
   not. This is the quiet gate on the class system, and it is made of nothing
   but time. */
function hasWorkshop(s, hh) {
  return Object.values(s.buildings).some(b =>
    b.householdId === hh.id && b.state === 'mature' && stageOf(b) >= 2);
}

function tavernsOf(s) {
  return Object.values(s.buildings).filter(b => b.state === 'mature' && roleOf(b) === 'tavern');
}

/* --- tenders keep coming back ---------------------------------------------- */

/* A tender chooses whose stone to attend exactly as they choose whose to
   start: the walk, kinship, what the house is known for, and what they hold
   against it. A house whose tenders will not visit simply stops growing, and
   nobody has to say why. */
function sysCheckIn(s, r) {
  const tenders = Object.values(s.people).filter(p => p.alive && p.tender
    && ageOf(s, p) >= 16 && ageOf(s, p) <= 74);
  if (!tenders.length) return;
  const visits = {};

  const due = Object.values(s.buildings)
    .filter(b => b.state === 'mature' && stageOf(b) < 4
      && s.turn - (b.lastTended || b.startTurn) >= CHECKIN_DUE)
    .sort((a, b) => (a.lastTended || a.startTurn) - (b.lastTended || b.startTurn));

  for (const b of due) {
    const hh = s.households[b.householdId];
    if (!hh || hh.extinct) continue;
    const willing = tenders
      .filter(t => (visits[t.id] || 0) < VISITS_PER_TENDER)
      .map(t => ({ t, v: tenderWill(s, t, hh, b.tileId) + (r() - 0.5) * 6 }))
      .filter(x => x.v > 0)
      .sort((a, c) => c.v - a.v)[0];
    if (!willing) continue;
    visits[willing.t.id] = (visits[willing.t.id] || 0) + 1;
    b.lastTended = s.turn;
    b.lastTenderId = willing.t.id;
  }
}

/* --- growing on ------------------------------------------------------------- */

function sysStages(s, r) {
  for (const b of Object.values(s.buildings)) {
    if (b.state !== 'mature') continue;
    if (!b.stage) { b.stage = 1; b.stageSince = b.startTurn; b.growth = 0; }
    if (b.stage >= 4) continue;

    const hh = s.households[b.householdId];
    if (!hh || hh.extinct) continue;

    // untended stone does not shrink. It simply stops, where it stands.
    const lapsed = s.turn - (b.lastTended || b.startTurn) > CHECKIN_LAPSE;
    if (lapsed) {
      if (!b.stalledStage) {
        b.stalledStage = s.turn;
        if (b.growth > 0.25) {
          ev(s, 'stall', 3, `The ${hh.name} house stopped growing. No tender has been up to it in ${Math.round((s.turn - (b.lastTended || b.startTurn)) / 4)} years.`,
             { building: b.id, household: hh.id });
        }
      }
      continue;
    }
    b.stalledStage = 0;

    const need = STAGE_TURNS[b.stage];
    if (!need) continue;
    b.growth = (b.growth || 0) + (0.7 + s.tiles[b.tileId].stone * 0.5) / need;
    if (b.growth < 1) continue;

    b.growth = 0;
    b.stage += 1;
    b.stageSince = s.turn;
    const years = Math.round((s.turn - b.startTurn) / 4);

    if (b.stage === 2) {
      ev(s, 'stage', 5, `The ${hh.name} house came on to its second growth after ${years} years — it has a workshop in it now.`,
         { building: b.id, household: hh.id });
    } else if (b.stage === 3) {
      b.role = chooseRole(s, r, b, hh);
      ev(s, 'stage', b.role === 'house' ? 5 : 7,
         `The ${hh.name} house reached its third growth after ${years} years. It is ${roleWord(b)} now.`,
         { building: b.id, household: hh.id });
      if (b.role === 'common') {
        for (const id of hh.members) {
          const p = s.people[id];
          if (p && p.alive) shiftStanding(p, 'quarter', 15, true);
        }
        remember(s, hh.id, 9, `gave up their third growth to the quarter`);
      }
    }
  }

  // what a great house or a tavern is worth to the people in it, by the year
  if (s.turn % 4 === 0) {
    for (const b of Object.values(s.buildings)) {
      if (b.state !== 'mature' || stageOf(b) < 3) continue;
      const hh = s.households[b.householdId];
      if (!hh || hh.extinct) continue;
      for (const id of hh.members) {
        const p = s.people[id];
        if (!p || !p.alive) continue;
        if (roleOf(b) === 'great') { shiftStanding(p, 'quarter', 10); shiftStanding(p, 'government', 6); }
        else if (roleOf(b) === 'tavern') shiftStanding(p, 'quarter', 6);
      }
    }
  }
}

/* A household decides what its third growth becomes, by where it stands when
   the stone gets there. A century of family work stopping being family
   property is the interesting case, so it wants a reason. */
/* Reaching a third growth is a century of somebody's attention. It does not by
   itself make a household eminent — if `great` is the fallback, most houses end
   up with one and the word stops meaning anything. Standing comes from being
   eminent already, from keeping a tavern, or from giving the growth away. Every
   other stage-3 building is simply a large house. */
function chooseRole(s, r, b, hh) {
  const houses = Object.values(s.households).filter(h => !h.extinct);
  const ranked = houses.slice().sort((a, c) => compositeOf(s, c) - compositeOf(s, a));
  const top = ranked.slice(0, Math.max(1, Math.round(ranked.length / 4)));
  if (top.some(h => h.id === hh.id)) return 'great';

  // a quarter supports one tavern. A second would have nobody new to draw.
  const q = quarterOf(s, b.tileId);
  const sameQuarter = x => {
    const xq = quarterOf(s, x.tileId);
    return xq && q && xq.id === q.id;
  };
  if (!tavernsOf(s).some(sameQuarter) && chance(r, 0.55)) return 'tavern';

  // and the settlement supports very few common halls — one per two quarters.
  // Without this the devout years produce a dozen, which makes the gift of a
  // third growth to the quarter mean nothing at all.
  const halls = Object.values(s.buildings)
    .filter(x => x.state === 'mature' && roleOf(x) === 'common').length;
  const room = Math.max(1, Math.ceil((s.quarters || []).length / 2));
  if (halls < room && s.shrine && s.shrine.devotion > 60 && chance(r, 0.5)) return 'common';

  return 'house';
}
