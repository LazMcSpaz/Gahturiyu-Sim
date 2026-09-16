/* ===========================================================================
   How things change hands.

   Before money there is no price, only a bargain between two households — and
   what they are to each other is half of it. A grudge costs you. Regard buys
   you a better deal. The same rope costs two houses different amounts, and the
   chronicle is allowed to say so.

   Nothing is paid for at the time. Neighbours remember, and a remembered debt
   fades. A debt large enough, in a settlement with somebody to enforce it,
   becomes a tally — a split stick that does not fade and does not die with the
   debtor.
   =========================================================================== */

/* In ḍaqu: a standard measure of grain, which is the unit from the first day
   whether or not there is ever a coin. Everything is priced against eating. */
const VALUE = {
  food: 1.0,
  timber: 0.45, stone: 0.5, ore: 0.7,
  cordage: 1.2, leather: 2.0, cloth: 2.6,
  fittings: 4.0, remedies: 3.5, garments: 5.5, metal: 7.0
};

const SELL_KEEP = 4;        // a house does not sell down to nothing
const TRADE_LOT = 1.2;      // how much moves in one bargain
const TALLY_AT = 6;         // a debt this size gets written on a stick
const DEBT_FADE = 0.985;    // a remembered debt fades; a tally does not
const MAX_DEBTS = 90;

function debtsOf(s) { return (s.debts = s.debts || []); }

function owed(s, fromId, toId) {
  const d = debtsOf(s).find(x => x.from === fromId && x.to === toId);
  return d ? d.amount : 0;
}

function totalOwed(s, hid) {
  return debtsOf(s).filter(d => d.from === hid).reduce((a, d) => a + d.amount, 0);
}

function totalHeld(s, hid) {
  return debtsOf(s).filter(d => d.to === hid).reduce((a, d) => a + d.amount, 0);
}

function addDebt(s, fromId, toId, amount, over) {
  if (fromId === toId || amount <= 0) return null;
  const list = debtsOf(s);
  let d = list.find(x => x.from === fromId && x.to === toId);
  if (!d) {
    d = { from: fromId, to: toId, amount: 0, since: s.turn, tally: false, over };
    list.push(d);
  }
  d.amount += amount;
  d.over = over || d.over;
  return d;
}

/* --- what a bargain costs ---------------------------------------------------
   Four things, and the last is the one that matters: what these two households
   are to each other. In barter the spread is wide, which is the whole
   difference between a settlement that has money and one that does not.
   -------------------------------------------------------------------------- */

/* Worked out once a season and read many times. Computed per call it walked
   every household for every buyer and every good — four thousand scans a
   season, and the cost showed. */
function scarcity(s, g) {
  if (!s._scarce) {
    s._scarce = {};
    const houses = Object.values(s.households).filter(h => !h.extinct);
    const n = Math.max(1, houses.length);
    for (const k of GOODS.concat(['fittings'])) {
      const total = houses.reduce((a, h) => a + (goodsOf(h)[k] || 0), 0);
      s._scarce[k] = clamp(1.8 - (total / n) / 3, 0.6, 2.6);
    }
  }
  return s._scarce[g] !== undefined ? s._scarce[g] : 1;
}

/* The price one household will name to another, for one lot. */
function priceFor(s, seller, buyer, g, urgency) {
  const base = (VALUE[g] || 1) * TRADE_LOT;
  const head = s.people[seller.headId];
  let mult = scarcity(s, g) * (1 + (urgency || 0) * 0.5);

  // what lies between them. A grudge is dear; regard is cheap.
  if (head) {
    const tie = tieTo(head, buyer.id);
    mult *= clamp(1 - tie / 220, 0.7, 1.6);
  }
  // and what the settlement thinks of the buyer
  mult *= clamp(1 - houseStanding(s, buyer, 'quarter') / 400, 0.8, 1.3);

  return { price: base * mult, mult };
}

/* --- who wants what ---------------------------------------------------------- */

function demandOf(s, hh) {
  const want = {};
  const live = hh.members.map(i => s.people[i]).filter(p => p && p.alive);
  const heads = live.length;
  if (!heads) return want;

  for (const [g, per] of Object.entries(WANTS)) {
    const need = per * heads * 6 - (goodsOf(hh)[g] || 0);
    if (need > 0) want[g] = need;
  }
  // a craft that cannot get its input is the sharpest demand in the settlement
  for (const p of live) {
    const recipe = MAKES[p.trade];
    if (!recipe || !recipe.needs || !canPractise(s, p)) continue;
    for (const [g, n] of Object.entries(recipe.needs)) {
      if (!n) continue;
      const short = n * 4 - (goodsOf(hh)[g] || 0);
      if (short > 0) want[g] = Math.max(want[g] || 0, short);
    }
  }
  return want;
}

/* --- the season's bargains --------------------------------------------------- */

function sysExchange(s, r) {
  const houses = Object.values(s.households).filter(h => !h.extinct && !h.lodgedWith);
  if (houses.length < 2) return;
  s.refusals = 0;
  s._scarce = null;                     // this season's prices, worked out once

  // who has anything spare, by good — rather than rescanning for every buyer
  const sellersBy = {};
  for (const h of houses) {
    const store = goodsOf(h);
    for (const g of Object.keys(store)) {
      if (store[g] > SELL_KEEP) (sellersBy[g] = sellersBy[g] || []).push(h);
    }
  }

  for (const buyer of houses) {
    const want = demandOf(s, buyer);
    for (const [g, need] of Object.entries(want)) {
      if (need < 0.3) continue;
      const urgency = clamp(need / 3, 0, 1.5) + ((buyer.shortSeasons || 0) > 2 ? 0.6 : 0);

      const sellers = (sellersBy[g] || []).filter(h => h.id !== buyer.id && (goodsOf(h)[g] || 0) > SELL_KEEP);
      if (!sellers.length) continue;

      const offers = sellers.map(sel => ({ sel, ...priceFor(s, sel, buyer, g, urgency) }))
        .sort((a, b) => a.price - b.price);

      /* A seller can simply decline. Holding something hard against a house is
         reason enough, and it is the line the chronicle exists to print. */
      const head = s.people[offers[0].sel.headId];
      const spite = head ? holdsAgainst(s, head, buyer.id) : 0;
      if (spite > 45 && chance(r, 0.6)) {
        s.refusals++;
        if (chance(r, 0.055)) {
          ev(s, 'spite', 5, `The ${offers[0].sel.name} would not sell ${g} to the ${buyer.name} this season.`,
             { household: buyer.id });
        }
        if (offers.length < 2) continue;
        offers.shift();
      }

      const deal = offers[0];
      const lot = Math.min(TRADE_LOT, need, (goodsOf(deal.sel)[g] || 0) - SELL_KEEP);
      if (lot <= 0.05) continue;

      /* Mercy is paid for here. A settlement whose rulings never bite lends
         less, because nobody extends what they will not get back — so being
         kind makes the place gentler and poorer at once. */
      const ceiling = TALLY_AT * 3 * (1 - (s.creditTight || 0) * 0.6);
      if (owed(s, buyer.id, deal.sel.id) > ceiling) {
        s.refusals++;
        buyer.noCredit = (buyer.noCredit || 0) + 1;
        if (buyer.noCredit === 8) {
          ev(s, 'hardship', 6, `Nobody on this coast will extend the ${buyer.name} any more credit.`,
             { household: buyer.id });
        }
        continue;
      }
      buyer.noCredit = 0;

      takeGood(deal.sel, g, lot);
      giveGood(buyer, g, lot);
      const owe = deal.price * (lot / TRADE_LOT);
      const d = addDebt(s, buyer.id, deal.sel.id, owe, g);

      /* Written down once it is worth writing down — but a stick being cut is
         the system working, not news. Reported every time it filled a sixth of
         the chronicle with bookkeeping. Only a debt large enough to be a
         millstone is worth a line. */
      if (d && !d.tally && d.amount >= TALLY_AT && officeHolders(s, 'arbiter').length) {
        d.tally = true; d.struck = s.turn;
        if (d.amount >= TALLY_AT * 2.5) {
          ev(s, 'dispute', 5, `The ${buyer.name} are ${Math.round(d.amount)} ḍaqu deep to the ${deal.sel.name}, and it went on a stick.`,
             { household: buyer.id });
        }
      }
    }
  }
}

/* --- paying, forgetting, and passing the stick on ---------------------------- */

function sysDebts(s, r) {
  const list = debtsOf(s);
  /* Slowly. At 0.012 a season a forgiven debt was forgotten inside four
     years and lending never actually tightened. */
  s.creditTight = Math.max(0, (s.creditTight || 0) - 0.004);

  for (const d of list) {
    const from = s.households[d.from], to = s.households[d.to];
    if (!from || !to || from.extinct || to.extinct) { d.amount = 0; continue; }

    // pay what can be paid, in food, because food is the unit
    if (from.stores > 8 && d.amount > 0) {
      const pay = Math.min(d.amount, (from.stores - 8) * 0.4);
      from.stores -= pay; to.stores += pay; d.amount -= pay;
      if (d.amount < 0.4) {
        d.amount = 0;
        const fh = s.people[from.headId];
        if (fh) shiftTie(s, fh, to.id, 6, `a debt settled between them`);
      }
    }
    // a remembered debt fades. A stick does not.
    if (!d.tally) d.amount *= DEBT_FADE;
  }

  /* A stick can be handed on: a creditor settles their own debt with somebody
     else's. What it is worth depends on whether the settlement believes the
     debtor will pay, which is the settlement putting a number on a household's
     name. */
  if (chance(r, 0.14)) {
    const tallies = list.filter(d => d.tally && d.amount > 2);
    for (const t of tallies) {
      const holder = s.households[t.to];
      if (!holder || holder.extinct) continue;
      const own = list.find(d => d.from === holder.id && d.amount > 2 && d.to !== t.from);
      if (!own) continue;
      const debtor = s.households[t.from];
      const trust = clamp(0.55 + houseStanding(s, debtor, 'quarter') / 120, 0.3, 1);
      const worth = Math.min(t.amount * trust, own.amount);
      if (worth < 1) continue;
      own.amount -= worth;
      t.to = own.to;                                  // the stick changes hands
      if (trust < 0.6 && chance(r, 0.25)) {
        ev(s, 'spite', 4, `The ${debtor.name} stick passed on at a loss — few believe that house will pay.`,
           { household: debtor.id });
      }
      break;
    }
  }

  s.debts = list.filter(d => d.amount > 0.4).slice(-MAX_DEBTS);
}
