/* ===========================================================================
   Interface. The only file that touches the DOM.
   The run is stored as: config + the list of inputs. States are cached for
   speed, but the inputs are the source of truth, which is what makes an
   export a few kilobytes instead of a few hundred.
   =========================================================================== */

const Run = {
  cfg: null,
  fullRecord: false,   // weight-1 events are always stored, just not shown
  states: [],     // states[i] is the world after i turns
  inputs: [],     // inputs[i] is what was applied to reach states[i+1]
  cursor: 0,
  branches: [],
  selected: null   // map tile being inspected, or null
};

function state() { return Run.states[Run.cursor]; }
function opts() { return { floor: Run.fullRecord ? 1 : 2 }; }

function toggleRecord() {
  Run.fullRecord = !Run.fullRecord;
  const b = document.getElementById('rec');
  b.textContent = Run.fullRecord ? 'Showing everything' : 'Show everything';
  b.setAttribute('aria-pressed', String(Run.fullRecord));
  renderAll(true);
  closeSheet();
}

function startRun(cfg) {
  Run.cfg = cfg;
  Run.states = [newWorld(cfg)];
  Run.inputs = [];
  Run.cursor = 0;
  Run.selected = null;
  renderAll(true);
}

function step(lever) {
  // advancing from a rolled-back point discards the future
  if (Run.cursor < Run.states.length - 1) {
    Run.states.length = Run.cursor + 1;
    Run.inputs.length = Run.cursor;
  }
  const input = { lever: lever || 'none' };
  const next = advance(state(), input);
  Run.states.push(next);
  Run.inputs.push(input);
  Run.cursor++;
  renderAll(false);
  const el = document.getElementById('turn-' + next.turn);
  const dock = document.getElementById('dock');
  if (el && el.scrollIntoView) {
    // block:'start' would put the season under the dock, so scroll by hand
    const y = el.getBoundingClientRect().top + window.scrollY - dock.offsetHeight - 8;
    window.scrollTo({ top: Math.max(0, y), behavior: prefersMotion() ? 'smooth' : 'auto' });
  }
}

function stepMany(n) {
  for (let i = 0; i < n; i++) step('none');
}

function rollbackTo(turn) {
  Run.cursor = Math.max(0, Math.min(turn, Run.states.length - 1));
  renderAll(true);
  closeSheet();
}

function saveBranch(name) {
  Run.branches.push({
    name: name || `${state().name}, ${seasonOf(state().turn)} ${yearOf(state(), state().turn)}`,
    cfg: clone(Run.cfg),
    inputs: clone(Run.inputs.slice(0, Run.cursor)),
    turn: Run.cursor
  });
  renderPanels();
}

function loadBranch(i) {
  const b = Run.branches[i];
  Run.cfg = clone(b.cfg);
  Run.states = [newWorld(Run.cfg)];
  Run.inputs = [];
  for (const inp of b.inputs) {
    Run.states.push(advance(Run.states[Run.states.length - 1], inp));
    Run.inputs.push(inp);
  }
  Run.cursor = Run.states.length - 1;
  Run.selected = null;
  renderAll(true);
  closeSheet();
}

function prefersMotion() { return !window.matchMedia('(prefers-reduced-motion: reduce)').matches; }

/* --- the map ------------------------------------------------------------------
   The map stays on screen while the chronicle scrolls under it. Hiding it is
   the only thing that takes it away, and that choice is remembered.
   -------------------------------------------------------------------------- */

function setMapShown(shown) {
  document.getElementById('plate').hidden = !shown;
  const b = document.getElementById('mapmin');
  b.textContent = shown ? 'Hide map' : 'Show map';
  b.title = shown ? 'Hide the map' : 'Show the map';
  b.setAttribute('aria-expanded', String(shown));
  try { localStorage.setItem('gahturiyu.map', shown ? '1' : '0'); } catch (e) { /* private window */ }
}

function toggleMap() {
  const showing = document.getElementById('plate').hidden;
  setMapShown(showing);
  if (!showing) closeInspector();   // hiding the map takes its inspector with it
}

/* Tapping a tile answers the question the chronicle keeps raising: whose is
   that? The selection is a tile id, and it survives advancing a season. */
function openInspector(id) {
  Run.selected = id;
  document.getElementById('inspect-body').innerHTML = tileFactsHTML(state(), id);
  document.getElementById('inspect').hidden = false;
  document.getElementById('dock').classList.add('inspecting');
  drawMap();
}

function closeInspector() {
  Run.selected = null;
  document.getElementById('inspect').hidden = true;
  document.getElementById('dock').classList.remove('inspecting');
  drawMap();
}

function drawMap() {
  document.getElementById('map').innerHTML = mapHTML(state(), Run.selected);
}

/* The selected tile keeps its meaning across a season, so refresh its facts
   whenever the world moves under it. */
function refreshInspector() {
  if (Run.selected === null || Run.selected === undefined) return;
  document.getElementById('inspect-body').innerHTML = tileFactsHTML(state(), Run.selected);
}

function mapShownPref() {
  try { return localStorage.getItem('gahturiyu.map') !== '0'; } catch (e) { return true; }
}

/* --- rendering ---------------------------------------------------------------- */

function renderAll(rebuild) {
  const s = state();
  document.getElementById('place').textContent = s.name;
  document.getElementById('when').textContent = s.turn === 0
    ? `founded, ${s.startYear}`
    : `${seasonOf(s.turn)} ${yearOf(s, s.turn)}`;

  const chron = document.getElementById('chronicle');
  if (rebuild || chron.childElementCount === 0) {
    if (!s.chronicle.length) {
      chron.innerHTML = `<article class="turn"><h2 class="season"><span class="s-name">before</span><span class="s-year">${s.startYear}</span></h2>
        <p class="opening">${openingHTML(s)}</p></article>`;
    } else {
      chron.innerHTML = openingCard(s) + s.chronicle.map(e => chronicleHTML(e, opts())).join('');
    }
  } else {
    const last = s.chronicle[s.chronicle.length - 1];
    if (last) chron.insertAdjacentHTML('beforeend', chronicleHTML(last, opts()));
  }

  drawMap();
  refreshInspector();
  renderOmen(s);
  renderPanels();
}

function openingCard(s) {
  const first = Run.states[0];
  return `<article class="turn"><h2 class="season"><span class="s-name">before</span><span class="s-year">${s.startYear}</span></h2>
    <p class="opening">${openingHTML(first)}</p></article>`;
}

function openingHTML(s) {
  const st = s.stats[s.stats.length - 1];
  const oldest = Object.values(s.buildings).sort((a, b) => a.startTurn - b.startTurn)[0];
  return `${st.pop} Roduro in ${st.households} households. ${st.mature} homes standing, the oldest begun ` +
    `${Math.round(-oldest.startTurn / 4)} years ago. ${st.tenders} ${st.tenders === 1 ? 'person' : 'people'} can work the stone.`;
}

function renderOmen(s) {
  const box = document.getElementById('omen');
  if (!s.omen) { box.hidden = true; box.innerHTML = ''; return; }
  const o = s.omen;
  const L = LEVERS[o.lever];
  box.hidden = false;
  box.innerHTML = `<p class="god">${o.god}<span class="dom">, ${o.domain}</span></p>
    <p class="saw">has seen ${o.saw}.</p>
    <p class="odom">If you accept:</p>
    <p class="offer">${L.desc}</p>
    <div class="orow">
      <button class="btn accept" onclick="step('${o.lever}')">Do it</button>
      <button class="btn quiet" onclick="step('none')">Decline</button>
    </div>`;
}

function renderPanels() {
  const s = state();
  document.getElementById('p-cast').innerHTML = castHTML(s);
  document.getElementById('p-houses').innerHTML = householdsHTML(s);
  document.getElementById('p-offices').innerHTML = officesHTML(s);
  document.getElementById('p-trades').innerHTML = tradesHTML(s);
  const strongest = {};
  for (const m of s.memory) {
    const cur = strongest[m.household];
    if (!cur || Math.abs(m.favour) > Math.abs(cur.favour)) strongest[m.household] = m;
  }
  const mem = Object.values(strongest)
    .filter(m => s.households[m.household] && !s.households[m.household].extinct)
    .sort((a, b) => Math.abs(b.favour) - Math.abs(a.favour)).slice(0, 12);
  document.getElementById('p-memory').innerHTML = mem.length
    ? `<p class="hint">What each household is still held to, strongest first. It fades over about sixty-five years, and it decides who is believed and who is ruled against.</p>`
      + mem.map(m => `<div class="memrow ${m.favour < 0 ? 'bad' : 'good'}">
          <span class="memhouse">${m.house}</span> ${m.phrase}
          <span class="memwhen">${Math.round((s.turn - m.turn) / 4)} years ago</span></div>`).join('')
    : `<p class="empty">Nothing is held against any household yet.</p>`;

  const st = s.stats[s.stats.length - 1];
  document.getElementById('p-numbers').innerHTML = `
    <div class="ngrid">
      ${num('Living', st.pop)}${num('Households', st.households)}
      ${num('Waiting on ground', st.waiting)}${num('Stone growing', st.growing)}
      ${num('Homes standing', st.mature)}${num('Stone-tenders', st.tenders)}
      ${num('Stores', st.stores)}${num('Households short', st.hungry)}
      ${num('Open quarrels', st.disputes)}${num('Named figures', st.actors)}
    </div>` + shrineHTML(s) + sparkHTML(s);

  document.getElementById('p-time').innerHTML =
    `<p class="hint">Tap a season to go back to it. Advancing from there replaces everything after — save the line first if you want to keep it.</p>` +
    `<div class="tl">` + Run.states.map((x, i) => {
      const label = i === 0 ? `founded ${x.startYear}` : `${seasonOf(i)} ${yearOf(x, i)}`;
      const lv = Run.inputs[i - 1]?.lever;
      return `<button class="tstep ${i === Run.cursor ? 'here' : ''}" onclick="rollbackTo(${i})">${label}${lv && lv !== 'none' ? `<span class="tlv">${LEVERS[lv].label}</span>` : ''}</button>`;
    }).join('') + `</div>` +
    `<button class="btn wide" onclick="saveBranch()">Save this line</button>` +
    (Run.branches.length ? `<div class="branches">` + Run.branches.map((b, i) =>
      `<button class="tstep" onclick="loadBranch(${i})">${b.name} <span class="tlv">${b.turn} seasons</span></button>`).join('') + `</div>` : '');

  const lv = document.getElementById('p-levers');
  lv.innerHTML = Object.entries(LEVERS).filter(([k]) => k !== 'none').map(([k, L]) =>
    `<button class="lever" onclick="step('${k}'); closeSheet();">
      <span class="lname">${L.label}</span><span class="ldesc">${L.desc}</span></button>`).join('');
}

function officesHTML(s) {
  const gov = GOVERNMENTS[s.government];
  let h = `<p class="hint"><strong>${s.name} is under ${gov.name}.</strong> ${gov.describe}</p>`;
  for (const sp of officeSpec(s)) {
    const held = officeHolders(s, sp.key);
    h += `<div class="orow2"><div class="otitle">${sp.title}${sp.n > 1 ? ` <span class="ocount">${held.length} of ${sp.n}</span>` : ''}</div>`;
    h += held.length
      ? held.map(p => {
          const q = quarterOf(s, homeTile(s, p));
          return `<div class="oholder">${p.name} <span class="flin">${p.lineage}</span>
            <span class="ometa">${ageOf(s, p)} years${q ? ' · ' + q.name : ''} · ${Math.max(1, Math.round((s.turn - (p.officeSince || s.turn)) / 4))} years in the seat</span></div>`;
        }).join('')
      : `<div class="ovacant">vacant</div>`;
    h += `</div>`;
  }
  const denied = (s.boatsDenied || []).map(id => (s.households[id] || {}).name).filter(Boolean);
  if (denied.length) h += `<p class="hint bad">Kept off the water this season: ${denied.join(', ')}.</p>`;
  return h;
}

function shrineHTML(s) {
  const sh = s.shrine;
  if (!sh) return '';
  const d = Math.round(sh.devotion);
  const word = d < 30 ? 'untended' : d < 55 ? 'kept, barely' : d < 78 ? 'kept' : 'well kept';
  return `<div class="shrinebox">
    <div class="shname">The ${sh.god} stone</div>
    <div class="shstate">${word}${sh.keeperId ? ' — ' + sh.keeperName + ' keeps it' : ' — nobody keeps it'}</div>
    <div class="shbar"><span style="width:${d}%"></span></div>
    <div class="shnote">${d > 60
      ? 'Rulings sworn here are obeyed. Quarrels end in judgement rather than in feud.'
      : d > 35
      ? 'Some will take a quarrel up there. Some will not.'
      : 'Nobody takes anything up there. Quarrels harden instead of ending.'}</div>
  </div>`;
}

function num(label, v) { return `<div class="nbox"><span class="nv">${v}</span><span class="nl">${label}</span></div>`; }

function sparkHTML(s) {
  const h = s.stats.slice(-60);
  if (h.length < 3) return '';
  const max = Math.max(...h.map(x => x.pop));
  const min = Math.min(...h.map(x => x.pop));
  const span = Math.max(1, max - min);
  const pts = h.map((x, i) => `${(i / (h.length - 1)) * 100},${30 - ((x.pop - min) / span) * 28}`).join(' ');
  return `<div class="spark"><svg viewBox="0 0 100 30" preserveAspectRatio="none" role="img" aria-label="Population over the last ${h.length} seasons">
    <polyline points="${pts}" fill="none" stroke="currentColor" stroke-width="0.8" vector-effect="non-scaling-stroke"/></svg>
    <span class="sparkl">Living, last ${h.length} seasons — low ${min}, high ${max}</span></div>`;
}

/* --- sheet ------------------------------------------------------------------- */

let sheetOpen = null;
function openSheet(which) {
  sheetOpen = which;
  document.querySelectorAll('.panel').forEach(p => p.hidden = p.dataset.panel !== which);
  document.querySelectorAll('.tab').forEach(b => b.setAttribute('aria-selected', b.dataset.tab === which));
  document.getElementById('sheet').classList.add('up');
  document.getElementById('scrim').hidden = false;
}
function closeSheet() {
  document.getElementById('sheet').classList.remove('up');
  document.getElementById('scrim').hidden = true;
  sheetOpen = null;
}

/* --- export / import ---------------------------------------------------------- */

function exportRun() {
  const blob = new Blob([JSON.stringify({
    engine: ENGINE_VERSION, cfg: Run.cfg,
    inputs: Run.inputs.slice(0, Run.cursor), branches: Run.branches
  }, null, 1)], { type: 'application/json' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = `${state().name.toLowerCase()}-${state().turn}.json`;
  a.click();
  URL.revokeObjectURL(a.href);
}

function importRun(file) {
  const fr = new FileReader();
  fr.onload = () => {
    try {
      const d = JSON.parse(fr.result);
      Run.cfg = d.cfg;
      Run.branches = d.branches || [];
      Run.states = [newWorld(d.cfg)];
      Run.inputs = [];
      for (const inp of d.inputs) {
        Run.states.push(advance(Run.states[Run.states.length - 1], inp));
        Run.inputs.push(inp);
      }
      Run.cursor = Run.states.length - 1;
      Run.selected = null;
      renderAll(true); closeSheet();
    } catch (e) {
      alert('That file could not be read as a run. ' + e.message);
    }
  };
  fr.readAsText(file);
}

function newSettlement() {
  const seed = parseInt(document.getElementById('f-seed').value, 10) || Math.floor(Math.random() * 1e9);
  const pop = clamp(parseInt(document.getElementById('f-pop').value, 10) || 150, 40, 1200);
  const gov = document.getElementById('f-gov').value || null;
  startRun({ seed, population: pop, startYear: 812, government: gov });
  closeSheet();
}

/* --- boot -------------------------------------------------------------------- */

window.addEventListener('DOMContentLoaded', () => {
  document.querySelectorAll('.tab').forEach(b =>
    b.addEventListener('click', () => sheetOpen === b.dataset.tab ? closeSheet() : openSheet(b.dataset.tab)));
  document.getElementById('scrim').addEventListener('click', closeSheet);
  document.getElementById('advance').addEventListener('click', () => step('none'));
  document.getElementById('mapmin').addEventListener('click', toggleMap);
  document.getElementById('map').addEventListener('click', e => {
    const tile = e.target.closest('[data-t]');
    if (!tile) return;
    const id = Number(tile.dataset.t);
    if (id === Run.selected) closeInspector(); else openInspector(id);
  });
  document.getElementById('inspect-x').addEventListener('click', closeInspector);
  setMapShown(mapShownPref());
  document.getElementById('advance-year').addEventListener('click', () => stepMany(4));
  document.getElementById('f-import').addEventListener('change', e => e.target.files[0] && importRun(e.target.files[0]));
  document.addEventListener('keydown', e => {
    if (e.key === 'Escape') { closeSheet(); closeInspector(); }
    if (e.key === ' ' && !sheetOpen && e.target === document.body) { e.preventDefault(); step('none'); }
  });
  startRun({ seed: 20260910, population: 150, startYear: 812 });
});
