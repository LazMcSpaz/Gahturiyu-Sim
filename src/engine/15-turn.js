/* --- the turn -------------------------------------------------------------- */

function advance(state, input) {
  const s = clone(state);
  s.turn += 1;
  s.log = [];
  const r = turnRng(s.seed, s.turn);

  s.weather = { storm: 0, cold: 0 };   // nothing carries over; a lever sets it below
  applyInput(s, r, input);
  sysWeather(s, r);
  sysFood(s, r);
  sysHardship(s, r);
  sysTeaching(s, r);
  sysGrowth(s, r);
  sysNotes(s, r);
  sysPairing(s, r);
  sysLife(s, r);
  sysHouseholds(s, r);
  sysClaims(s, r);
  sysDisputes(s, r);
  sysOffices(s, r);
  sysShrine(s, r);
  sysService(s, r);     // who owes the stone a year, and who refuses it
  sysContact(s, r);     // who spent the season near whom
  sysGrudges(s, r);
  sysPromotion(s, r);
  sysActors(s, r);
  sysMemory(s, r);
  sysOmen(s, r);

  decayTies(s);         // regard fades before it is counted
  decayStanding(s);

  recount(s);
  s.chronicle.push({
    turn: s.turn, season: seasonOf(s.turn), year: yearOf(s, s.turn),
    weather: clone(s.weather), events: clone(s.log),
    shrine: s.shrine ? { god: s.shrine.god, devotion: Math.round(s.shrine.devotion), keeper: s.shrine.keeperId ? s.shrine.keeperName : null } : null,
    stats: clone(s.stats[s.stats.length - 1]),
    input: input && input.lever ? input.lever : null
  });
  return s;
}

/* levers ------------------------------------------------------------------- */

/* label — the button. desc — what it will do, shown before you choose.
   fact  — the chronicle line, past tense, no god named and no atmosphere. */
const LEVERS = {
  none:     { label: 'Nothing',           desc: 'Let the season pass.',                         fact: '' },
  storm:    { label: 'Send a storm',      desc: 'Boats stay in. The catch fails.',              fact: 'Storm sent. No catch this season.' },
  bounty:   { label: 'Fill the nets',     desc: 'The catch is doubled.',                        fact: 'Catch doubled this season.' },
  blight:   { label: 'Sicken the herds',  desc: 'The grazing fails.',                           fact: 'Blight sent. The herds sickened.' },
  quicken:  { label: 'Quicken the stone', desc: 'Every growing home gains a fifth of its growth.', fact: 'Every growing home gained about a fifth of its growth.' },
  wither:   { label: 'Still the stone',   desc: 'Every growing home stalls.',                   fact: 'Every growing home stalled.' },
  fever:    { label: 'Send a fever',      desc: 'The old and the crowded die.',                 fact: 'Fever sent. The old and the crowded were at risk.' },
  strangers:{ label: 'Bring strangers',   desc: 'A boat of newcomers asks for ground.',         fact: 'A boat of newcomers arrived asking for ground.' },
  reveal:   { label: 'Reveal a grudge',   desc: 'One hidden grudge becomes public.',            fact: 'A hidden grudge was made public.' },
  temper:   { label: 'Cool tempers',      desc: 'Every open quarrel loses most of its heat.',   fact: 'Every open quarrel lost most of its heat.' }
};

function applyInput(s, r, input) {
  const lv = input && input.lever;
  if (!lv || lv === 'none') return;
  const L = LEVERS[lv];
  ev(s, 'divine', 5, L.fact, { lever: lv });
  switch (lv) {
    case 'storm': s.weather.storm = 1; break;
    case 'bounty': s.weather.bounty = 1; break;
    case 'blight': s.weather.blight = 1; break;
    case 'quicken':
      for (const b of Object.values(s.buildings)) if (b.state === 'growing') b.maturity = Math.min(1, b.maturity + 0.22);
      break;
    case 'wither':
      for (const b of Object.values(s.buildings)) if (b.state === 'growing') b.stalled += 6;
      break;
    case 'fever': s.weather.fever = 1; break;
    case 'strangers': arriveStrangers(s, r); break;
    case 'reveal': revealGrudge(s, r); break;
    case 'temper':
      for (const d of Object.values(s.disputes)) if (d.open) d.heat = Math.max(0, d.heat - 45);
      break;
  }
}

/* weather ------------------------------------------------------------------- */

function sysWeather(s, r) {
  const season = seasonOf(s.turn);
  const w = s.weather;
  if (!w.storm) w.storm = (season === 'winter' && chance(r, 0.35)) || (season === 'harvest' && chance(r, 0.18)) ? 1 : 0;
  w.cold = season === 'winter' ? 0.6 + r() * 0.4 : season === 'spring' ? 0.2 : 0;
  if (w.storm) ev(s, 'weather', 2, 'Storms. The boats stayed in and the catch failed.');
  else if (season === 'summer' && chance(r, 0.2)) { w.fair = 1; ev(s, 'weather', 1, 'Fair weather. Fishing was good.'); }
}
