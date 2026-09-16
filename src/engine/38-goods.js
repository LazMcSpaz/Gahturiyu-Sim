/* ===========================================================================
   What is made, what is eaten, and what the government takes first.

   Ten goods. Each is made by a trade and wanted by somebody who cannot make
   it, which is what forces exchange rather than designing it.

   Food is stored twice: a household fills its own larder, and the government
   takes its cut off the top before any larder is filled. That cut is the
   common store, and it does three things only — it feeds the offices and the
   guard, it opens in an emergency, and it is what a government is judged on
   when the emergency comes.
   =========================================================================== */

const GOODS = ['food', 'stone', 'ore', 'timber', 'cordage', 'leather', 'metal', 'cloth', 'garments', 'remedies'];

/* Per worker, per season, before season and site are applied. Food is the
   exception: it scales with the ground a household actually holds, which is
   why the fishing and grazing numbers stay where they were. */
const MAKES = {
  fieldhand: null, boathand: null,            // food, handled by the old yield model
  quarrier:   { stone: 2.0 },
  miner:      { ore: 1.5 },
  woodcutter: { timber: 2.2 },
  hauler:     null,                            // moves things; makes nothing
  roper:      { cordage: 2.5, needs: { cordage: 0 } },
  tanner:     { leather: 1.6 },
  carrier:    null,
  mason:      null,                            // work, not goods
  tender:     null,
  weaver:     { cloth: 1.4 },
  tailor:     { garments: 0.9, needs: { cloth: 1.2 } },
  herbalist:  { remedies: 1.1 },
  joiner:     { fittings: 0.8, needs: { timber: 1.0 } },
  carver:     null,
  smith:      { metal: 0.7, needs: { ore: 1.6 } }
};

/* What a household wants each season, beyond food. Small, steady, and only
   worth noticing when it cannot be had. */
const WANTS = { garments: 0.06, remedies: 0.04, cordage: 0.05 };

function blankGoods() {
  const o = {};
  for (const g of GOODS) o[g] = 0;
  o.fittings = 0;
  return o;
}

function goodsOf(hh) {
  if (!hh.goods) hh.goods = blankGoods();
  return hh.goods;
}

function hasGood(hh, g, n) { return goodsOf(hh)[g] >= n; }

function takeGood(hh, g, n) {
  const store = goodsOf(hh);
  const got = Math.min(store[g], n);
  store[g] -= got;
  return got;
}

function giveGood(hh, g, n) { goodsOf(hh)[g] += n; }

/* --- making things ---------------------------------------------------------- */

/* Everything but food. A craft that needs an input and cannot get it makes
   nothing, which is the first place the chronicle should feel a shortage. */
function sysMake(s, r) {
  s.shortOf = {};
  for (const p of Object.values(s.people)) {
    if (!p.alive || !p.trade) continue;
    const a = ageOf(s, p);
    if (a < 14 || a > 70 || gaoled(p)) continue;
    const recipe = MAKES[p.trade];
    if (!recipe) continue;
    if (!canPractise(s, p)) continue;          // a craft with no room makes nothing
    const hh = s.households[p.householdId];
    if (!hh || hh.extinct) continue;
    const host = economicHost(s, hh);

    // inputs first
    let ok = true;
    if (recipe.needs) {
      for (const [g, n] of Object.entries(recipe.needs)) {
        if (!n) continue;
        if (!hasGood(host, g, n)) {
          ok = false;
          s.shortOf[p.trade] = (s.shortOf[p.trade] || 0) + 1;
          break;
        }
      }
      if (ok) for (const [g, n] of Object.entries(recipe.needs)) takeGood(host, g, n);
    }
    if (!ok) continue;

    const skill = 0.8 + p.traits.loyalty / 400;
    for (const [g, n] of Object.entries(recipe)) {
      if (g === 'needs') continue;
      giveGood(host, g, n * skill * (0.85 + r() * 0.3));
    }
  }

  // nothing keeps forever, and a settlement that hoards everything is not one
  for (const hh of Object.values(s.households)) {
    if (hh.extinct) continue;
    const store = goodsOf(hh);
    for (const g of Object.keys(store)) store[g] = Math.min(store[g] * 0.985, 60);
  }
}

/* --- the government's cut --------------------------------------------------- */

const TITHE = 0.12;            // off the top, before a larder is filled
const GUARD_KEEP = 0.9;        // what an office-holder or guard eats from the store
/* The buffer has to be small enough to actually run out. At 2.4 a head the
   store carried a settlement through twenty-five years of storm, blight and
   fever without once being empty, so feeding people only ever added
   legitimacy and a government could not fail. */
const STORE_CAP_PER_HEAD = 1.5;
const RELIEF_PER_HOUSE = 2.2;      // what a short household actually needs

function commonStore(s) {
  if (!s.store) s.store = { food: 0, taken: 0, given: 0, emptyFor: 0 };
  return s.store;
}

/* Called from the food system with what a household brought in this season.
   Returns what is left for the larder. */
function tithe(s, hh, yield_) {
  if (yield_ <= 0) return yield_;
  const store = commonStore(s);
  const rate = TITHE * (s.titheRate || 1);
  const cut = yield_ * rate;
  store.food += cut;
  store.taken += cut;
  return yield_ - cut;
}

/* What the store is for. Offices and the guard eat from it; in a hard season it
   is opened to whoever is starving; and if it is empty when that happens, the
   government wears it. */
function sysStore(s, r) {
  const store = commonStore(s);
  const living = Object.values(s.people).filter(p => p.alive).length;
  store.food = Math.min(store.food, living * STORE_CAP_PER_HEAD);

  // the offices and the guard are fed by the settlement, not by their own work
  const kept = new Set();
  for (const sp of officeSpec(s)) for (const p of officeHolders(s, sp.key)) kept.add(p.id);
  const cost = kept.size * GUARD_KEEP;
  store.food = Math.max(0, store.food - cost);

  /* Three consecutive short seasons was too high a bar: households dip and
     recover inside a season or two, so only one to four ever qualified and the
     store trickled out where it should have poured. */
  const short = Object.values(s.households).filter(h => !h.extinct && !h.lodgedWith
    && (h.shortSeasons || 0) >= 2);
  if (!short.length) { store.emptyFor = 0; return; }

  /* Going hungry is held against whoever runs the settlement, fed or not.
     Without this the store was a legitimacy engine: opening it paid regard,
     famine was what opened it, and twenty years of storm, blight and fever
     left a government of ninety-five survivors better thought of than the
     same government over a calm run of a hundred and twenty. Relief now
     softens the blow instead of turning it into a profit. */
  for (const h of short) for (const id of h.members) {
    const p = s.people[id];
    if (p && p.alive) shiftStanding(p, 'government', -4);
  }

  if (store.food < short.length * RELIEF_PER_HOUSE) {
    // the emergency came and the store was not there for it
    store.emptyFor++;
    if (store.emptyFor === 2 || store.emptyFor % 8 === 0) {
      for (const sp of officeSpec(s)) for (const p of officeHolders(s, sp.key)) {
        shiftStanding(p, 'government', -14);
        shiftStanding(p, 'quarter', -10);
      }
      // and the households left to starve think a great deal less of it
      for (const h of short) for (const id of h.members) {
        const p = s.people[id];
        if (p && p.alive) shiftStanding(p, 'government', -22);
      }
      ev(s, 'hardship', 7, `${short.length} households are short and the common store is empty.`, {});
    }
    return;
  }

  store.emptyFor = 0;
  const each = Math.min(RELIEF_PER_HOUSE, store.food / short.length);
  const enough = each >= RELIEF_PER_HOUSE * 0.9;    // a ration, or a gesture
  let anyFresh = false;
  for (const h of short) {
    h.stores += each; store.food -= each; store.given += each;
    /* Being fed once is gratitude. Being fed every season is dependency, and it
       buys a government nothing. The small repeat award looked harmless and
       was not: a steady point a season against a decay of 0.985 settles at
       sixty-odd, so twenty years of storm, blight and fever raised legitimacy
       from fifty-one to seventy-eight while the population fell and the store
       drained. Feeding people who are still hungry next season is not a
       government succeeding. */
    const fresh = s.turn - (h.lastRelief || -99) > 10;
    if (fresh) anyFresh = true;
    h.lastRelief = s.turn;
    for (const id of h.members) {
      const p = s.people[id];
      if (!p || !p.alive) continue;
      if (!enough) shiftStanding(p, 'government', -3);    // turned away with a handful
      else if (fresh) shiftStanding(p, 'government', 6);
    }
  }
  if (anyFresh && enough)
    for (const sp of officeSpec(s)) for (const p of officeHolders(s, sp.key)) shiftStanding(p, 'government', 6);

  /* The store opening is only news when it is not the ordinary run of things.
     Reported every time it happens, it fired most seasons and turned an
     emergency into routine — three lines a season, all the same line. */
  const big = short.length >= 4;
  if (big || s.turn - (store.lastTold || -99) >= 24) {
    store.lastTold = s.turn;
    ev(s, 'hardship', big ? 6 : 4,
      `The common store was opened to ${short.length} household${short.length > 1 ? 's' : ''} that had run out.`, {});
  }
}

/* --- wanting things --------------------------------------------------------- */

/* A household consumes a little of what it does not make. This exists so that
   shortage is felt before exchange is built — the want goes unmet, and the
   chronicle can say so. */
function sysWant(s, r) {
  s.unmet = {};
  for (const hh of Object.values(s.households)) {
    if (hh.extinct || hh.lodgedWith) continue;
    const live = hh.members.filter(i => s.people[i] && s.people[i].alive).length;
    if (!live) continue;
    for (const [g, per] of Object.entries(WANTS)) {
      const need = per * live;
      const got = takeGood(hh, g, need);
      if (got < need * 0.5) s.unmet[g] = (s.unmet[g] || 0) + 1;
    }
  }
  // report a settlement-wide shortage, not a household one
  const houses = Object.values(s.households).filter(h => !h.extinct && !h.lodgedWith).length || 1;
  for (const [g, n] of Object.entries(s.unmet)) {
    if (n < houses * 0.6) continue;
    s.shortSince = s.shortSince || {};
    if (s.turn - (s.shortSince[g] || -99) < 40) continue;
    s.shortSince[g] = s.turn;
    const maker = Object.keys(MAKES).find(k => MAKES[k] && MAKES[k][g] !== undefined);
    const makers = maker ? Object.values(s.people).filter(p => p.alive && p.trade === maker).length : 0;
    ev(s, 'note', 5, `There is no ${g} to be had on this coast${makers ? '' : ` — nobody works as a ${maker}`}.`, {});
  }
}
