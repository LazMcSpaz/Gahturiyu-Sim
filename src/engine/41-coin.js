/* ===========================================================================
   Money, which is an achievement and not a given.

   The settlement starts without it. Coin appears only when the government is
   strong and believed enough to guarantee a standard — so legitimacy is
   mechanically consequential rather than decorative, and a long run lets you
   watch money being invented.

   The unit never changes. A ḍaqu is a measure of grain from the first day; a
   coin is the weight of metal that buys one, and takes the same name. Common
   people go on meaning grain by it while the government means metal.
   =========================================================================== */

/* Measured first: legitimacy runs about 35 to 95 over a long run. At 52 the
   coin appeared inside forty years on every seed and never once failed, which
   is not an achievement. At 72 it takes a settlement that is genuinely well
   governed, and 52 to keep means an ordinary bad stretch can kill it. */
const COIN_AT = 72;          // legitimacy at which somebody strikes the first batch
const COIN_KEEP = 52;        // and below which the settlement stops believing in it
const BATCH_EVERY = 36;      // turns — nine years between batches at the soonest
const BATCH_METAL = 9;       // metal a batch consumes
const BATCH_COIN = 60;       // ḍaqu struck from it
const SMITH_CUT = 0.12;      // what the contracted smith keeps, as payment
const COIN_TAX = 0.05;       // of a household's coin, per year, once there is coin

function mintOf(s) {
  if (!s.mint) s.mint = {
    coined: 0, purse: 0, batches: 0, lastBatch: -999,
    believed: false, since: 0, confidence: 1, lastSmith: null, run: []
  };
  return s.mint;
}

function coinOf(hh) { return hh.coin || 0; }

/* --- legitimacy --------------------------------------------------------------
   Read from things that already exist: what the settlement thinks of whoever
   governs, whether the seats are filled at all, whether the store was there
   when it was needed, and how long the present form has held without falling
   over.
   -------------------------------------------------------------------------- */

function legitimacy(s) {
  const houses = Object.values(s.households).filter(h => !h.extinct);
  if (!houses.length) return 0;
  const gov = houses.reduce((a, h) => a + houseStanding(s, h, 'government'), 0) / houses.length;

  const spec = officeSpec(s);
  const want = spec.reduce((a, sp) => a + sp.n, 0) || 1;
  const held = spec.reduce((a, sp) => a + officeHolders(s, sp.key).length, 0);
  const filled = clamp(held / want, 0, 1);

  const store = commonStore(s);
  const living = Object.values(s.people).filter(p => p.alive).length || 1;
  const fed = clamp(store.food / (living * 0.8), 0, 1);

  const stable = clamp((s.turn - (s.govSince || 0)) / 160, 0, 1);
  const shrine = s.shrine ? clamp(s.shrine.devotion / 100, 0, 1) : 0.4;

  return gov * 1.4 + filled * 22 + fed * 18 + stable * 12 + shrine * 14
       - (store.emptyFor ? 34 : 0)
       - (held < want ? 12 : 0);      // a vacant seat is a government not doing its job
}

/* --- striking a batch --------------------------------------------------------
   Minting is a contract, granted one batch at a time. The smith keeps a
   portion as payment, which makes the contract the most valuable thing a
   government owns — and the reason an office is worth wanting. Per batch, so
   there is no minting dynasty, and the favour is granted fresh every time.
   -------------------------------------------------------------------------- */

function pickSmith(s, r) {
  const smiths = Object.values(s.people).filter(p => p.alive && p.trade === 'smith'
    && ageOf(s, p) >= 18 && canPractise(s, p));
  if (!smiths.length) return null;
  const rulers = officeSpec(s).flatMap(sp => officeHolders(s, sp.key));
  return smiths.map(p => {
    const hh = s.households[p.householdId];
    let v = houseStanding(s, hh, 'government') * 0.5 + houseStanding(s, hh, 'trade') * 0.3;
    for (const o of rulers) {
      const own = s.households[o.householdId];
      if (own && own.id === p.householdId) v += 40;               // one's own house first
      v += tieTo(o, p.householdId) * 0.4;
    }
    return { p, v: v + (r() - 0.5) * 12 };
  }).sort((a, b) => b.v - a.v)[0].p;
}

function strike(s, r) {
  const m = mintOf(s);
  const smith = pickSmith(s, r);
  if (!smith) return false;
  const hh = s.households[smith.householdId];
  if (!hh || !hasGood(hh, 'metal', BATCH_METAL)) return false;

  takeGood(hh, 'metal', BATCH_METAL);
  const struck = BATCH_COIN * (0.85 + r() * 0.3);
  const cut = struck * SMITH_CUT;
  hh.coin = coinOf(hh) + cut;
  m.purse += struck - cut;
  m.coined += struck;
  m.batches++;
  m.lastBatch = s.turn;
  m.run = (m.lastSmith === hh.id ? m.run : []).concat(s.turn).slice(-6);
  const running = m.lastSmith === hh.id ? m.run.length : 1;
  m.lastSmith = hh.id;

  shiftStanding(smith, 'trade', 14);
  remember(s, hh.id, 5, `were given the striking of the coin`);

  if (m.batches === 1) {
    m.believed = true; m.since = s.turn;
    ev(s, 'office', 7, `${nameOf(s, smith.id)} struck the first coin on this coast. A ḍaqu of metal is a ḍaqu of grain.`,
       { person: smith.id, household: hh.id });
  } else if (running >= 3) {
    ev(s, 'office', 6, `The ${hh.name} have had the striking of the coin ${running} batches running.`,
       { household: hh.id });
  }
  return true;
}

/* --- money, and losing faith in it ------------------------------------------ */

function sysCoin(s, r) {
  const m = mintOf(s);
  const legit = s.legitimacy = legitimacy(s);
  const living = Object.values(s.people).filter(p => p.alive).length || 1;

  // the first batch, when the settlement is finally believed enough for one
  if (!m.believed && !m.batches && legit >= COIN_AT && s.turn - m.lastBatch > BATCH_EVERY) {
    if (!strike(s, r)) m.lastBatch = s.turn - BATCH_EVERY + 8;   // no smith, no metal, try again
    return;
  }
  if (!m.batches) return;

  /* A government short of coin orders another batch. Too many and the coin is
     believed less; being believed less is legitimacy falling; and below the
     threshold the coin stops being money at all. The government that abused it
     destroyed it, and no new machinery was needed to say so. */
  if (m.believed && s.turn - m.lastBatch >= BATCH_EVERY) {
    const hungry = m.purse < living * 0.25;
    if (hungry && chance(r, 0.5)) {
      if (strike(s, r)) {
        const perHead = m.coined / living;
        if (perHead > 1.6) m.confidence = Math.max(0, m.confidence - 0.16);
      }
    }
  }
  m.confidence = Math.min(1, m.confidence + 0.004);

  // tax, in coin, once a year
  if (m.believed && s.turn % 4 === 0) {
    for (const hh of Object.values(s.households)) {
      if (hh.extinct || coinOf(hh) < 1) continue;
      const due = coinOf(hh) * COIN_TAX;
      hh.coin = coinOf(hh) - due;
      m.purse += due;
    }
  }

  // and the offices are paid from the purse
  if (m.believed && s.turn % 4 === 0) {
    for (const sp of officeSpec(s)) for (const p of officeHolders(s, sp.key)) {
      const hh = s.households[p.householdId];
      if (!hh || m.purse < 2) continue;
      m.purse -= 2; hh.coin = coinOf(hh) + 2;
    }
  }

  const sound = legit >= COIN_KEEP && m.confidence > 0.35;
  if (m.believed && !sound) {
    m.believed = false; m.since = s.turn;
    ev(s, 'office', 7, `The coin stopped being taken on this coast. What people hold of it is metal and nothing more.`, {});
    for (const sp of officeSpec(s)) for (const p of officeHolders(s, sp.key)) shiftStanding(p, 'government', -18);
  } else if (!m.believed && m.batches && legit >= COIN_AT && m.confidence > 0.6) {
    m.believed = true; m.since = s.turn;
    ev(s, 'office', 6, `The coin is taken again.`, {});
  }
}

/* True when a price can be named in coin rather than haggled over. */
function coinWorks(s) {
  const m = mintOf(s);
  return !!(m.believed && m.coined > 0);
}
