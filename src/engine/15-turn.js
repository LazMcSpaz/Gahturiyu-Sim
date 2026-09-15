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
  sysGrudges(s, r);
  sysPromotion(s, r);
  sysActors(s, r);
  sysMemory(s, r);
  sysOmen(s, r);

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

const LEVERS = {
  none:     { label: 'Nothing', desc: 'Let the season pass unaided.' },
  storm:    { label: 'Send a storm', desc: 'Horahìda stirs the water. Boats stay in; the catch fails.' },
  bounty:   { label: 'Fill the nets', desc: 'A season of impossible fishing.' },
  blight:   { label: 'Sicken the herds', desc: 'Qotihiqì turns the grazing against them.' },
  quicken:  { label: 'Quicken the stone', desc: 'Dodìṭo hurries every growing home toward maturity.' },
  wither:   { label: 'Still the stone', desc: 'Every growing home halts, as if the rock forgot.' },
  fever:    { label: 'Send a fever', desc: 'Hiyaḍote walks the paths. The old and the crowded suffer.' },
  strangers:{ label: 'Bring strangers', desc: 'A boat of newcomers asks for ground.' },
  reveal:   { label: 'Reveal a grudge', desc: 'Shiḍuro makes a hidden injury public.' },
  temper:   { label: 'Cool tempers', desc: 'Gìhuqìdu settles the standing quarrels.' }
};

function applyInput(s, r, input) {
  const lv = input && input.lever;
  if (!lv || lv === 'none') return;
  const L = LEVERS[lv];
  ev(s, 'divine', 5, `${L.desc}`, { lever: lv });
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
  if (w.storm) ev(s, 'weather', 2, pick(r, [
    'Gales came off the water and the boats stayed drawn up on the shingle.',
    'A run of bad weather. What went out came back mostly empty.',
    'The sea was unworkable for weeks and the nets hung in the rafters.',
    'Storms off the headland. Two boats were lost and not replaced.',
    'Wind from the open water all season, and nobody put out past the shelf.'
  ]));
  else if (season === 'summer' && chance(r, 0.2)) { w.fair = 1; ev(s, 'weather', 1, 'The weather held fair for weeks together, which the older heads distrusted.'); }
}
