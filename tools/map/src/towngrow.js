  // ---- growing a draft town (Laz, 17 Oct 2026: "Grow a draft" in the builder). A town grows
  //      from what its mark's notes and label say it grew on (a harbour, a ford, a hill, a
  //      crossroads, a road; the land decides when they don't say): a square where it began,
  //      its temple or great house on the high ground near it, streets from there out along the
  //      roads, to the landmark and down to the shore, found by a path search that charges for
  //      climbing (so they bend round hills), lanes off them to good ground, and the homes on
  //      plots fronting the streets, packed near the square and thinning toward the edge, each
  //      people where it leans (Roduro round the core, Qotiro up on the terraces, Horaro on
  //      stilts off the shore, Ṭaḍoro tents at the edges). Towns and bigger are walled; Roduro
  //      in grown stone. The research behind it: docs/towns.md. Everything is a draft: it goes
  //      into the place as ordinary things and paths, to move, delete and save like any other. ----
  const TownGrow = (() => {
    const CELL = 4; // the working grid, metres a cell
    const MAX_GRADE = 0.33; // steeper than this a street won't climb
    // What a mark's words say the town grew on; the first in this order wins ("sea fort" is a fort).
    const PURPOSE_WORDS = [
      ["fort", "fort"], ["castle", "fort"], ["keep", "fort"], ["garrison", "fort"], ["citadel", "fort"],
      ["ford", "ford"], ["river", "ford"], ["bridge", "ford"], ["crossing", "ford"],
      ["harbour", "harbour"], ["harbor", "harbour"], ["port", "harbour"], ["fishing", "harbour"], ["fishers", "harbour"], ["dock", "harbour"], ["docks", "harbour"], ["quay", "harbour"], ["lake", "harbour"],
      ["hill", "hill"], ["hilltop", "hill"], ["ridge", "hill"], ["cliffs", "hill"], ["cliff", "hill"], ["peak", "hill"], ["mountain", "hill"],
      ["crossroads", "crossroads"], ["junction", "crossroads"], ["market", "crossroads"],
      ["road", "street"], ["travel", "street"],
    ];
    // Words that change the size, the spread and the edge.
    const SMALL = ["small", "tiny", "remote"], LOOSE = ["spread", "scattered", "marshy", "sparse"], WALLED = ["walled", "wall", "walls", "palisade", "stockade"], OPEN = ["unwalled", "wallless"];
    // Homes by kind of mark (before the notes' "small").
    const HOMES = { capital: 160, city: 110, town: 50, village: 22, hamlet: 8, outpost: 6, fort: 12, port: 14 };
    const WALLED_KINDS = ["capital", "city", "town", "fort"];
    const PURPOSE_NAME = { harbour: "a harbour town", ford: "a ford town", hill: "a hill town", crossroads: "a crossroads town", street: "a road town", green: "a village round a green", fort: "a fort town" };

    const words = (t) => (t || "").toLowerCase().split(/[^a-z0-9]+/).filter(Boolean);
    const hasAny = (ws, list) => ws.some((w) => list.includes(w));

    // A seeded roll: the same draft for the same place and roll number.
    function rng(seed) {
      let s = seed >>> 0 || 1;
      return () => { s ^= s << 13; s >>>= 0; s ^= s >>> 17; s ^= s << 5; s >>>= 0; return s / 4294967296; };
    }
    const hash = (str) => { let h = 2166136261; for (let i = 0; i < str.length; i++) { h ^= str.charCodeAt(i); h = Math.imul(h, 16777619); } return h >>> 0; };

    // ---- the land round the place, at 4 m a cell: the place's own grid (with Laz's paint) inside
    //      its area, the coarser ring round it beyond, so a sea or a road just outside counts ----
    function grid(pl) {
      const W = pl.GW * pl.CELL, H = pl.GH * pl.CELL, A = pl.LAND.around;
      const aw = A.size[0] * A.cell, ah = A.size[1] * A.cell;
      const EW = Math.min(aw, Math.max(W * 2.4, 520)), EH = Math.min(ah, Math.max(H * 2.4, 420));
      const nx = Math.floor(EW / CELL), ny = Math.floor(EH / CELL);
      const g = { nx, ny, x0: -EW / 2, y0: -EH / 2, W, H, h: new Float32Array(nx * ny), wet: new Uint8Array(nx * ny), road: new Uint8Array(nx * ny) };
      const ringH = A.heightsDecoded || (A.heightsDecoded = (() => { const s = atob(A.height), n = A.size[0] * A.size[1], out = new Float32Array(n); for (let k = 0; k < n && k * 2 + 1 < s.length; k++) { const v = s.charCodeAt(k * 2) | (s.charCodeAt(k * 2 + 1) << 8); out[k] = (v > 32767 ? v - 65536 : v) / 10; } return out; })());
      const sub = CELL / pl.CELL;
      for (let j = 0; j < ny; j++) for (let i = 0; i < nx; i++) {
        const x = g.x0 + (i + 0.5) * CELL, y = g.y0 + (j + 0.5) * CELL, k = j * nx + i;
        const pi = Math.floor((x - CELL / 2) / pl.CELL + pl.P.origin[0]), pj = Math.floor((y - CELL / 2) / pl.CELL + pl.P.origin[1]);
        if (pi >= 0 && pj >= 0 && pi + sub <= pl.GW && pj + sub <= pl.GH) {
          // Inside the place: its own 2 m cells, Laz's paint and heights first.
          let hs = 0, n = 0, sea = 0, river = 0, road = 0;
          for (let b = 0; b < sub; b++) for (let a = 0; a < sub; a++) {
            const q = (pj + b) * pl.GW + (pi + a), painted = pl.height[q];
            hs += Number.isNaN(painted) ? pl.natH[q] : painted; n++;
            const ch = pl.ground[q] !== "." ? pl.ground[q] : (pl.natG ? pl.natG[q] : "g");
            if (ch === "~") sea++; else if (ch === "q") river++; else if (ch === "R") road++;
          }
          g.h[k] = hs / n; g.wet[k] = sea * 2 >= n ? 1 : river * 2 >= n ? 2 : 0; g.road[k] = road ? 1 : 0;
        } else {
          // Beyond it: the ring's 8 m cells.
          const ai = Math.floor(x / A.cell + A.origin[0]), aj = Math.floor(y / A.cell + A.origin[1]);
          if (ai < 0 || aj < 0 || ai >= A.size[0] || aj >= A.size[1]) { g.wet[k] = 1; continue; }
          const q = aj * A.size[0] + ai, ch = A.ground[q];
          g.h[k] = ringH[q]; g.wet[k] = ch === "~" ? 1 : ch === "q" ? 2 : 0; g.road[k] = ch === "R" ? 1 : 0;
        }
      }
      g.at = (x, y) => { const i = Math.floor((x - g.x0) / CELL), j = Math.floor((y - g.y0) / CELL); return i >= 0 && j >= 0 && i < nx && j < ny ? j * nx + i : -1; };
      g.pos = (k) => [g.x0 + ((k % nx) + 0.5) * CELL, g.y0 + (Math.floor(k / nx) + 0.5) * CELL];
      g.slope = (k) => { const i = k % nx, j = Math.floor(k / nx); if (i < 1 || j < 1 || i >= nx - 1 || j >= ny - 1) return 1; const dx = (g.h[k + 1] - g.h[k - 1]) / (2 * CELL), dy = (g.h[k + nx] - g.h[k - nx]) / (2 * CELL); return Math.hypot(dx, dy); };
      g.dry = (x, y) => { const k = g.at(x, y); return k >= 0 && !g.wet[k]; };
      return g;
    }

    function flattest(g, cx, cy, r) {
      let best = Infinity, at = [cx, cy];
      for (let k = 0; k < g.h.length; k++) {
        if (g.wet[k]) continue;
        const [x, y] = g.pos(k), d = Math.hypot(x - cx, y - cy);
        if (d > r) continue;
        const s = g.slope(k) * 40 + d / r;
        if (s < best) { best = s; at = [x, y]; }
      }
      return at;
    }

    // The cheapest way from cell `from` to any goal cell, climbing charged as it rises; streets
    // already laid are cheap to walk along, the land's roads nearly as cheap.
    const STEPS = [[1, 0, 1], [-1, 0, 1], [0, 1, 1], [0, -1, 1], [1, 1, 1.414], [1, -1, 1.414], [-1, 1, 1.414], [-1, -1, 1.414], [2, 1, 2.236], [1, 2, 2.236], [-1, 2, 2.236], [-2, 1, 2.236], [-2, -1, 2.236], [-1, -2, 2.236], [1, -2, 2.236], [2, -1, 2.236]];
    function path(g, from, goal, streets, ford, limit) {
      const N = g.h.length, dist = new Float32Array(N).fill(Infinity), prev = new Int32Array(N).fill(-1);
      // A binary heap of (cost, cell) in typed arrays.
      let hc = new Float32Array(1024), hk = new Int32Array(1024), hn = 0;
      const push = (c, k) => {
        if (hn === hc.length) { const c2 = new Float32Array(hn * 2), k2 = new Int32Array(hn * 2); c2.set(hc); k2.set(hk); hc = c2; hk = k2; }
        let i = hn++;
        while (i > 0) { const p = (i - 1) >> 1; if (hc[p] <= c) break; hc[i] = hc[p]; hk[i] = hk[p]; i = p; }
        hc[i] = c; hk[i] = k;
      };
      const pop = () => {
        const top = [hc[0], hk[0]], c = hc[--hn], k = hk[hn];
        let i = 0;
        for (;;) { const l = 2 * i + 1, r = l + 1; let m = -1, mc = c; if (l < hn && hc[l] < mc) { m = l; mc = hc[l]; } if (r < hn && hc[r] < mc) { m = r; mc = hc[r]; } if (m < 0) break; hc[i] = hc[m]; hk[i] = hk[m]; i = m; }
        if (hn > 0) { hc[i] = c; hk[i] = k; }
        return top;
      };
      dist[from] = 0; push(0, from);
      let end = -1;
      while (hn > 0) {
        const [d, k] = pop();
        if (d > dist[k]) continue;
        if (goal[k] && k !== from) { end = k; break; }
        if (d > limit) break;
        const i = k % g.nx, j = Math.floor(k / g.nx);
        for (const [di, dj, ln] of STEPS) {
          const ii = i + di, jj = j + dj;
          if (ii < 1 || jj < 1 || ii >= g.nx - 1 || jj >= g.ny - 1) continue;
          const m = jj * g.nx + ii;
          if (g.wet[m] === 1 || (g.wet[m] === 2 && !ford)) continue;
          const run = ln * CELL, grade = Math.abs(g.h[m] - g.h[k]) / run;
          if (grade > MAX_GRADE) continue;
          let c = run * (1 + 10 * grade + 160 * grade * grade);
          if (g.wet[m] === 2) c += 60; // a bridge or a ford: once, where it's cheapest
          if (streets[m]) c *= 0.3; else if (g.road[m]) c *= 0.45;
          if (d + c < dist[m]) { dist[m] = d + c; prev[m] = k; push(d + c, m); }
        }
      }
      if (end < 0) return null;
      const out = [end];
      while (out[out.length - 1] !== from) out.push(prev[out[out.length - 1]]);
      return out.reverse();
    }
    // A path's cells as a line, the grid's steps smoothed out.
    function line(g, cells) {
      let pts = cells.map((k) => g.pos(k));
      for (let pass = 0; pass < 3 && pts.length > 2; pass++) {
        const s = [pts[0]];
        for (let i = 1; i < pts.length - 1; i++) s.push([(pts[i - 1][0] + 2 * pts[i][0] + pts[i + 1][0]) / 4, (pts[i - 1][1] + 2 * pts[i][1] + pts[i + 1][1]) / 4]);
        s.push(pts[pts.length - 1]); pts = s;
      }
      // Fewer points: drop those that barely bend the line.
      const out = [pts[0]];
      for (let i = 1; i < pts.length - 1; i++) {
        const a = out[out.length - 1], b = pts[i], c = pts[i + 1];
        const cross = Math.abs((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])) / (Math.hypot(c[0] - a[0], c[1] - a[1]) || 1);
        if (cross > 0.6 || Math.hypot(b[0] - a[0], b[1] - a[1]) > 24) out.push(b);
      }
      out.push(pts[pts.length - 1]);
      return out;
    }
    function stamp(g, pts, width, into) {
      const r = Math.round(width / 2 / CELL);
      for (let s = 0; s + 1 < pts.length; s++) {
        const [ax, ay] = pts[s], [bx, by] = pts[s + 1], n = Math.max(1, Math.ceil(Math.hypot(bx - ax, by - ay) / (CELL / 2)));
        for (let t = 0; t <= n; t++) {
          const x = ax + (bx - ax) * t / n, y = ay + (by - ay) * t / n;
          for (let dj = -r; dj <= r; dj++) for (let di = -r; di <= r; di++) { const k = g.at(x + di * CELL, y + dj * CELL); if (k >= 0) into[k] = 1; }
        }
      }
    }

    // ---- the grower ----
    // o: { pl, mark: { kind, label, notes, founders }, cat (catalogue by key), roll }
    function grow(o) {
      const { pl, mark, cat } = o;
      const g = grid(pl);
      const W = pl.GW * pl.CELL, H = pl.GH * pl.CELL;
      const ws = words((mark.notes || "") + " " + (mark.label || ""));
      const rand = rng(hash(pl.key) ^ Math.imul(o.roll + 1, 2654435761));
      const range = (a, b) => a + (b - a) * rand();
      const kind = String(mark.kind || "village");
      let homes = HOMES[kind] || 10;
      if (hasAny(ws, SMALL)) homes = Math.round(homes * 0.6);
      const loose = hasAny(ws, LOOSE);
      const radius = Math.min(W, H) * 0.46;
      const walled = hasAny(ws, OPEN) ? false : hasAny(ws, WALLED) || WALLED_KINDS.includes(kind);
      const founders = String(mark.founders || "roduro").toLowerCase();

      // What it grew on: the words, if the land can hold them; else the land.
      const sea = [], river = []; let top = [-Infinity, 0, 0];
      for (let k = 0; k < g.h.length; k++) {
        const [x, y] = g.pos(k), d = Math.hypot(x, y);
        if (g.wet[k] === 1) { if (d < Math.max(W, H) * 0.9) sea.push([d, x, y]); }
        else if (g.wet[k] === 2) { if (d < Math.max(W, H) * 0.6) river.push([d, x, y]); }
        else if (d < radius * 0.7 && g.h[k] > top[0]) top = [g.h[k], x, y];
      }
      sea.sort((a, b) => a[0] - b[0]); river.sort((a, b) => a[0] - b[0]);
      // Roads leaving the place: road cells on the rim, grouped by angle.
      const exits = [];
      { let run = null; const rim = Math.min(W, H) / 2 - 8;
        const on = [];
        for (let a = 0; a < 360; a++) { const r = a * Math.PI / 180; const k = g.at(Math.cos(r) * W * 0.5, Math.sin(r) * H * 0.5); on.push(k >= 0 && g.road[k] === 1); }
        for (let a = 0; a < 360; a++) { if (on[a]) { if (!run) run = [a, a]; else run[1] = a; } else if (run) { exits.push(run); run = null; } }
        if (run) { if (exits.length && exits[0][0] === 0) exits[0][0] = run[0] - 360; else exits.push(run); }
      }
      const exitPts = exits.map(([a0, a1]) => { const r = ((a0 + a1) / 2) * Math.PI / 180; return [Math.cos(r) * W * 0.5, Math.sin(r) * H * 0.5]; });
      const rimH = []; for (let a = 0; a < 64; a++) { const k = g.at(Math.cos(a * 0.098) * radius, Math.sin(a * 0.098) * radius); if (k >= 0) rimH.push(g.h[k]); }
      rimH.sort((a, b) => a - b);
      const rise = top[0] - (rimH[rimH.length >> 1] || top[0]);
      const found = sea.length > 30 ? "harbour" : river.length > 10 ? "ford" : rise > 14 ? "hill" : exitPts.length >= 3 ? "crossroads" : exitPts.length === 2 ? "street" : "green";
      let said = null; for (const [w, p] of PURPOSE_WORDS) if (ws.includes(w)) { said = p; break; }
      const purpose = said === "harbour" && !sea.length ? found : said === "ford" && !river.length ? found : said || found;

      // Where it began.
      let heart;
      const why = o.why || {}; const no = (r) => { why[r] = (why[r] || 0) + 1; };
      if (purpose === "harbour" && sea.length) {
        const [, sx, sy] = sea[0]; let best = null;
        for (let k = 0; k < g.h.length; k++) {
          if (g.wet[k]) continue;
          const [x, y] = g.pos(k); const d = Math.hypot(x - sx, y - sy); if (d > 40) continue;
          if ([[-12, 0], [12, 0], [0, -12], [0, 12]].some(([dx, dy]) => { const m = g.at(x + dx, y + dy); return m >= 0 && g.wet[m] === 1; })) continue;
          const s = g.slope(k) * 30 + d / 40; if (!best || s < best[0]) best = [s, x, y];
        }
        heart = best ? [best[1], best[2]] : flattest(g, 0, 0, 60);
      } else if (purpose === "ford" && river.length) {
        const [, rx, ry] = river[0]; let best = null;
        for (let k = 0; k < g.h.length; k++) {
          if (g.wet[k]) continue;
          const [x, y] = g.pos(k), d = Math.hypot(x - rx, y - ry); if (d < 14 || d > 40) continue;
          const s = -g.h[k] * 0.3 + g.slope(k) * 30; if (!best || s < best[0]) best = [s, x, y];
        }
        heart = best ? [best[1], best[2]] : flattest(g, 0, 0, 60);
      } else if (purpose === "hill" || purpose === "fort") {
        heart = flattest(g, top[1] * 0.6, top[2] * 0.6, 50);
      } else if ((purpose === "crossroads" || purpose === "street") && g.road.some((v) => v)) {
        let best = null;
        for (let k = 0; k < g.h.length; k++) if (g.road[k]) { const [x, y] = g.pos(k), d = Math.hypot(x, y); if (!best || d < best[0]) best = [d, x, y]; }
        heart = flattest(g, best[1], best[2], 16);
      } else heart = flattest(g, 0, 0, 50);
      // (Always inside the place: a harbour whose shore lies beyond it starts at its edge nearest the water.)
      if (Math.abs(heart[0]) > W / 2 - 24 || Math.abs(heart[1]) > H / 2 - 24) heart = flattest(g, Math.max(-W / 2 + 30, Math.min(W / 2 - 30, heart[0])), Math.max(-H / 2 + 30, Math.min(H / 2 - 30, heart[1])), 20);
      const [hx, hy] = heart;

      // The landmark: the highest flat ground near the heart (the top, on a hill).
      let mark2;
      if (purpose === "hill" || purpose === "fort") mark2 = flattest(g, top[1], top[2], 25);
      else {
        let best = null;
        for (let k = 0; k < g.h.length; k++) {
          if (g.wet[k] || g.slope(k) > 0.2) continue;
          const [x, y] = g.pos(k), d = Math.hypot(x - hx, y - hy); if (d < 40 || d > radius * 0.6) continue;
          const s = g.h[k] - d * 0.05; if (!best || s > best[0]) best = [s, x, y];
        }
        mark2 = best ? [best[1], best[2]] : [hx + 50, hy];
      }

      // Streets: from each goal back to the heart, or to a street already laid.
      const goals = exitPts.slice(); goals.push(mark2);
      if (purpose === "harbour") for (let i = 0; i < sea.length && i < 400; i += 80) goals.push([sea[i][1], sea[i][2]]);
      // With fewer than two roads in, a few ways out to the edge of the place anyway, toward the easiest ground.
      if (exitPts.length < 2) for (let i = 0; i < 3; i++) { const a = range(0, Math.PI * 2); goals.push([Math.max(-W / 2 + 4, Math.min(W / 2 - 4, hx + Math.cos(a) * W)), Math.max(-H / 2 + 4, Math.min(H / 2 - 4, hy + Math.sin(a) * H))]); }
      const street = new Uint8Array(g.h.length), streets = [];
      const hk = g.at(hx, hy);
      for (const [gx, gy] of goals) {
        const gk = g.at(gx, gy); if (gk < 0) continue;
        const target = street.slice(); target[hk] = 1;
        const cells = path(g, gk, target, street, purpose === "ford", Infinity);
        if (!cells || cells.length < 2) continue;
        const pts = line(g, cells); stamp(g, pts, 6, street); streets.push({ pts, main: true });
      }
      // Lanes off them, to good ground no street serves yet.
      const lanes = Math.max(2, Math.round(homes / 6));
      const cand = [];
      for (let k = 0; k < g.h.length; k += 3) {
        if (g.wet[k] || g.slope(k) > 0.18) continue;
        const [x, y] = g.pos(k), d = Math.hypot(x - hx, y - hy); if (d > radius) continue;
        cand.push([d + range(0, radius * 0.35), x, y]);
      }
      cand.sort((a, b) => a[0] - b[0]);
      let laid = 0, tried = 0;
      for (const [, x, y] of cand) {
        if (laid >= lanes || tried > lanes * 6) break;
        let served = false;
        for (let dj = -3; dj <= 3 && !served; dj++) for (let di = -3; di <= 3; di++) { const k = g.at(x + di * 9, y + dj * 9); if (k >= 0 && street[k]) { served = true; break; } }
        if (served) continue;
        tried++;
        const cells = path(g, g.at(x, y), street, street, false, 900);
        if (cells && cells.length >= 2 && cells.length * CELL < 170) { const pts = line(g, cells); stamp(g, pts, 4, street); streets.push({ pts, main: false }); laid++; }
      }

      // Who builds: the founders count two and a half times, Ṭaḍoro pitch tents.
      const share = { roduro: 1, qotiro: 1, horaro: 1, tadoro: 0.5 }; if (share[founders] !== undefined) share[founders] *= 2.5;
      const coastal = sea.length > 30;
      const total = Object.values(share).reduce((a, b) => a + b, 0);
      const quota = {}; for (const p in share) quota[p] = Math.round(homes * share[p] / total);
      const stiltN = coastal ? quota.horaro : 0; if (coastal) quota.horaro = 0;

      // The catalogue's footprints.
      const dims = (key) => { const d = cat[key]; return d ? [d.depth, d.width] : [8, 8]; };
      const reach = (key) => { const [a, b] = dims(key); return Math.hypot(a, b) / 2; };
      const pickKey = (people, rel, main, n) => {
        if (people === "roduro") { if (main && rel < 0.5 && n % 4 === 1) return ["roduro_forge", "roduro_loomroom", "roduro_benchroom", "roduro_trader", "roduro_tender"][Math.floor(rand() * 5)]; return rel < 0.3 && rand() < 0.35 ? "roduro_great" : rand() < 0.4 ? "roduro_longhouse" : "roduro_cottage"; }
        if (people === "qotiro") { if (main && rel < 0.5 && n % 3 === 1) return ["qotiro_workyard", "qotiro_weavehall", "qotiro_market", "qotiro_benchyard"][Math.floor(rand() * 4)]; return rand() < 0.5 ? "qotiro_court" : "qotiro_quarters"; }
        if (people === "horaro") return rand() < 0.5 ? "roduro_cottage" : "roduro_longhouse"; // a Horaro family ashore lives in a land house
        return "tent";
      };

      // Plots along every street, fronting it.
      const plots = [];
      for (const st of streets) for (let s = 0; s + 1 < st.pts.length; s++) {
        const [ax, ay] = st.pts[s], [bx, by] = st.pts[s + 1], L = Math.hypot(bx - ax, by - ay); if (L < 1e-3) continue;
        const tx = (bx - ax) / L, ty = (by - ay) / L;
        for (let u = 0; u < L; u += 3) {
          const px = ax + tx * u, py = ay + ty * u, d = Math.hypot(px - hx, py - hy);
          for (const sg of [-1, 1]) plots.push({ score: d * (st.main ? 0.7 : 1) + range(-6, 6), x: px, y: py, nx: -ty * sg, ny: tx * sg, main: st.main });
        }
      }
      plots.sort((a, b) => a.score - b.score);
      const fray = loose ? 0.55 : 0.75;
      const usable = plots.filter((p) => { const rel = Math.hypot(p.x - hx, p.y - hy) / radius; return rel <= 1.25 && !(rel > fray && rand() < (rel - fray) * 1.6); });

      const things = [], placed = [];
      const put = (k, x, y, rotRad, note) => { const d = cat[k]; things.push({ k, x: Math.round(x * 10) / 10, y: Math.round(y * 10) / 10, rot: (Math.round(rotRad * 180 / Math.PI) % 360 + 360) % 360, s: 1, label: "", room: d ? d.beds || 0 : 0, note: note || "" }); placed.push([x, y, reach(k)]); };
      const clearOf = (x, y, r, gap) => placed.every(([qx, qy, qr]) => Math.hypot(qx - x, qy - y) > qr + r + gap);
      // Clear of every street and lane, by the lines themselves: a footprint `depth` by `width`
      // turned `rot` keeps its corners and edges a step off each street's edge.
      const segs = [];
      for (const st of streets) for (let i = 0; i + 1 < st.pts.length; i++) { const [ax, ay] = st.pts[i], [bx, by] = st.pts[i + 1]; segs.push([ax, ay, bx, by, (st.main ? 5 : 2.5) / 2 + 0.6, Math.min(ax, bx), Math.max(ax, bx), Math.min(ay, by), Math.max(ay, by)]); }
      const segDist = (x, y, [ax, ay, bx, by]) => { const dx = bx - ax, dy = by - ay, L = dx * dx + dy * dy; const t = L ? Math.max(0, Math.min(1, ((x - ax) * dx + (y - ay) * dy) / L)) : 0; return Math.hypot(x - ax - dx * t, y - ay - dy * t); };
      const offStreets = (x, y, depth, width, rot) => {
        const c = Math.cos(rot), s = Math.sin(rot), pts = [[0, 0]];
        for (const [u, v] of [[1, 1], [1, -1], [-1, 1], [-1, -1], [1, 0], [-1, 0], [0, 1], [0, -1]]) pts.push([u * depth / 2, v * width / 2]);
        const R = Math.hypot(depth, width) / 2 + 4;
        for (const sg of segs) {
          if (x + R < sg[5] || x - R > sg[6] || y + R < sg[7] || y - R > sg[8]) continue;
          for (const [u, v] of pts) if (segDist(x + u * c - v * s, y + u * s + v * c, sg) < sg[4]) return false;
        }
        return true;
      };
      const inside = (x, y, r) => Math.abs(x) + r < W / 2 - 2 && Math.abs(y) + r < H / 2 - 2;

      // The square: a market where it began (a village and bigger), the well beside it.
      if (homes >= 14 && cat.market) put("market", hx, hy, 0, "the square");
      else placed.push([hx, hy, 9]);
      if (cat.well) { const a = range(0, 6.28); put("well", hx + Math.cos(a) * 12, hy + Math.sin(a) * 12, 0); }
      // The landmark.
      const lmKey = founders === "qotiro" && homes >= 14 && cat.qotiro_temple ? "qotiro_temple" : cat.roduro_great ? "roduro_great" : null;
      if (lmKey && clearOf(mark2[0], mark2[1], reach(lmKey), 2) && inside(mark2[0], mark2[1], reach(lmKey))) put(lmKey, mark2[0], mark2[1], Math.atan2(hy - mark2[1], hx - mark2[0]), "the landmark: the high ground near the square");

      // Homes: each plot goes to whoever leans to it most, while they have homes to build.
      const affinity = (p, x, y, rel, k) => {
        if (p === "roduro") return 1 - rel;
        if (p === "qotiro") return 0.6 * (g.h[k] - g.h[hk]) / 20 + 0.6 * g.slope(k) / 0.2 - Math.hypot(x - mark2[0], y - mark2[1]) / radius;
        if (p === "horaro") return sea.length ? -Math.hypot(x - sea[0][1], y - sea[0][2]) / 60 : -rel;
        return rel; // Ṭaḍoro: the edges
      };
      let n = 0;
      for (const p of usable) {
        const left = Object.keys(quota).filter((q) => quota[q] > 0); if (!left.length) break;
        const rel = Math.hypot(p.x - hx, p.y - hy) / radius;
        const k0 = g.at(p.x + p.nx * 8, p.y + p.ny * 8); if (k0 < 0 || g.wet[k0]) { no("wet"); continue; }
        const people = left.reduce((a, b) => (affinity(b, p.x, p.y, rel, k0) + rand() * 0.25 > affinity(a, p.x, p.y, rel, k0) + rand() * 0.25 ? b : a));
        const key = pickKey(people, rel, p.main, n);
        if (!cat[key]) continue;
        const [depth, width] = dims(key);
        const front = (p.main ? 2.5 : 1.25) + 1.2 + depth / 2;
        const x = p.x + p.nx * front, y = p.y + p.ny * front, k = g.at(x, y);
        if (k < 0 || g.wet[k]) { no("wet"); continue; }
        if (g.slope(k) > (people === "qotiro" ? 0.32 : 0.2)) { no("steep"); continue; }
        const r = reach(key);
        if (!inside(x, y, r)) { no("outside"); continue; }
        if (Math.hypot(x - hx, y - hy) < 14 + r) { no("square"); continue; }
        if (![[-1, -1], [1, -1], [1, 1], [-1, 1]].every(([dx, dy]) => g.dry(x + dx * r * 0.7, y + dy * r * 0.7))) { no("wet corner"); continue; }
        if (!offStreets(x, y, depth, width, Math.atan2(-p.ny, -p.nx))) { no("street"); continue; }
        if (!clearOf(x, y, r, 1.5 + 4 * Math.max(0, rel - 0.3))) { no("crowded"); continue; }
        put(key, x, y, Math.atan2(-p.ny, -p.nx), g.slope(k) > 0.16 ? "terraced" : "");
        quota[people]--; n++;
      }

      // Stilt homes off the shore near the quay, on a boardwalk from the square.
      const roads = streets.map((st) => ({ k: st.main ? "road" : "path", w: st.main ? 5 : 2.5, pts: st.pts.map(([x, y]) => [Math.round(x * 10) / 10, Math.round(y * 10) / 10]) }));
      if (stiltN > 0) {
        const spots = [];
        for (let k = 0; k < g.h.length; k++) {
          if (g.wet[k] !== 1) continue;
          const [x, y] = g.pos(k); if (Math.hypot(x - hx, y - hy) > radius) continue;
          let near = 99; for (let dy = -40; dy <= 40; dy += 8) for (let dx = -40; dx <= 40; dx += 8) { const m = g.at(x + dx, y + dy); if (m >= 0 && !g.wet[m]) near = Math.min(near, Math.hypot(dx, dy)); }
          if (near >= 16 && near <= 36) spots.push([Math.hypot(x - hx, y - hy) + range(0, 20), x, y]);
        }
        spots.sort((a, b) => a[0] - b[0]);
        const homesAt = [];
        for (const [, x, y] of spots) {
          if (homesAt.length >= stiltN) break;
          const key = ["horaro_dome", "horaro_twin", "horaro_hull", "horaro_net_loft"][Math.floor(rand() * 4)];
          if (!cat[key] || !inside(x, y, reach(key)) || !clearOf(x, y, reach(key), 3)) continue;
          put(key, x, y, range(0, 6.28)); homesAt.push([x, y]);
        }
        // The boardwalk: from the quay to the nearest home, then home to nearest home.
        if (homesAt.length) {
          const done = [[hx, hy]], rest = homesAt.slice();
          while (rest.length) {
            let best = null;
            for (const a of done) for (let i = 0; i < rest.length; i++) { const d = Math.hypot(a[0] - rest[i][0], a[1] - rest[i][1]); if (!best || d < best[0]) best = [d, a, i]; }
            const b = rest.splice(best[2], 1)[0];
            roads.push({ k: "path", w: 2, pts: [[Math.round(best[1][0] * 10) / 10, Math.round(best[1][1] * 10) / 10], [Math.round(b[0] * 10) / 10, Math.round(b[1] * 10) / 10]] });
            done.push(b);
          }
        }
      }

      // Fields along the roads out, beyond the last house.
      const outer = placed.reduce((m, [x, y]) => Math.max(m, Math.hypot(x - hx, y - hy)), 0);
      let fields = 0;
      if (cat.field_plot) for (const st of streets) {
        if (!st.main) continue;
        for (let s = 0; s + 1 < st.pts.length && fields < (hasAny(ws, ["farming", "farms", "fields"]) ? 14 : 8); s++) {
          const [ax, ay] = st.pts[s], [bx, by] = st.pts[s + 1], L = Math.hypot(bx - ax, by - ay); if (L < 1e-3) continue;
          const mx = (ax + bx) / 2, my = (ay + by) / 2, d = Math.hypot(mx - hx, my - hy);
          if (d < outer + 8 || rand() < 0.4) continue;
          const tx = (bx - ax) / L, ty = (by - ay) / L, [fd, fw] = dims("field_plot");
          for (const sg of [-1, 1]) {
            const nx = -ty * sg, ny = tx * sg, x = mx + nx * (5 + fd / 2), y = my + ny * (5 + fd / 2), k = g.at(x, y);
            if (k < 0 || g.wet[k] || g.slope(k) > 0.12 || !inside(x, y, reach("field_plot")) || !clearOf(x, y, reach("field_plot"), 2) || !offStreets(x, y, fd, fw, Math.atan2(ny, nx))) continue;
            put("field_plot", x, y, Math.atan2(ny, nx)); fields++;
          }
        }
      }

      // The wall: round the built town, with a gate where each main street leaves; grown stone
      // for a Roduro town (the stone wall stands in until the game has one), a palisade else.
      let wallNote = "";
      if (walled && placed.length > 4) {
        const stone = founders === "roduro" || founders === "qotiro";
        const wk = stone ? "rubble_wall" : "palisade";
        // The line: round the built town (fields stay outside), a few metres out, kept inside the place.
        const N = 72, ring = new Array(N).fill(24), homesOnly = things.filter((t) => t.k !== "field_plot" && t.k !== "well");
        for (let a = 0; a < N; a++) {
          const ang = a / N * Math.PI * 2;
          for (const t of homesOnly) {
            const dx = t.x - hx, dy = t.y - hy, d = Math.hypot(dx, dy); if (d < 1) continue;
            let da = Math.abs(Math.atan2(dy, dx) - ang); if (da > Math.PI) da = Math.PI * 2 - da;
            if (da < 0.3) ring[a] = Math.max(ring[a], d + reach(t.k) + 6);
          }
          // (Never past the place's edge.)
          const c = Math.cos(ang), sn = Math.sin(ang);
          const edge = Math.min(Math.abs(c) > 1e-6 ? ((c > 0 ? W / 2 : -W / 2) - hx) / c : Infinity, Math.abs(sn) > 1e-6 ? ((sn > 0 ? H / 2 : -H / 2) - hy) / sn : Infinity) - 4;
          ring[a] = Math.min(ring[a], edge);
        }
        for (let pass = 0; pass < 4; pass++) { const r0 = ring.slice(); for (let a = 0; a < N; a++) ring[a] = Math.max(r0[a], (r0[(a + N - 1) % N] + r0[a] * 2 + r0[(a + 1) % N]) / 4); }
        const pt = (a) => { const ang = (a % N) / N * Math.PI * 2; return [hx + Math.cos(ang) * ring[a % N], hy + Math.sin(ang) * ring[a % N]]; };
        // Where each main street crosses the line: a gate there.
        const cross = (a1, a2, b1, b2) => { const d = (a2[0] - a1[0]) * (b2[1] - b1[1]) - (a2[1] - a1[1]) * (b2[0] - b1[0]); if (Math.abs(d) < 1e-9) return null; const t = ((b1[0] - a1[0]) * (b2[1] - b1[1]) - (b1[1] - a1[1]) * (b2[0] - b1[0])) / d, u = ((b1[0] - a1[0]) * (a2[1] - a1[1]) - (b1[1] - a1[1]) * (a2[0] - a1[0])) / d; return t >= 0 && t <= 1 && u >= 0 && u <= 1 ? [a1[0] + (a2[0] - a1[0]) * t, a1[1] + (a2[1] - a1[1]) * t] : null; };
        const gatesAt = [];
        for (const st of streets) { if (!st.main) continue; for (let i = 0; i + 1 < st.pts.length; i++) for (let a = 0; a < N; a++) { const x = cross(pt(a), pt(a + 1), st.pts[i], st.pts[i + 1]); if (x && gatesAt.every((q) => Math.hypot(q[0] - x[0], q[1] - x[1]) > 10)) gatesAt.push(x); } }
        // (And where a main street runs out to the wall without crossing it.)
        const nearLine = (x, y) => { let best = null; for (let a = 0; a < N; a++) { const [ax, ay] = pt(a), [bx, by] = pt(a + 1), dx = bx - ax, dy = by - ay, L2 = dx * dx + dy * dy, t = L2 ? Math.max(0, Math.min(1, ((x - ax) * dx + (y - ay) * dy) / L2)) : 0, q = [ax + dx * t, ay + dy * t], d = Math.hypot(x - q[0], y - q[1]); if (!best || d < best[0]) best = [d, q]; } return best; };
        for (const st of streets) { if (!st.main) continue; for (const e of [st.pts[0], st.pts[st.pts.length - 1]]) { const nl = nearLine(e[0], e[1]); if (nl && nl[0] < 9 && Math.hypot(e[0] - hx, e[1] - hy) > 20 && gatesAt.every((q) => Math.hypot(q[0] - nl[1][0], q[1] - nl[1][1]) > 10)) gatesAt.push(nl[1]); } }
        const [wd, ww] = dims(wk), long = Math.max(wd, ww), alongDepth = wd >= ww, [gd, gw] = dims("gate");
        let pieces = 0, gates = 0;
        for (const [x, y] of gatesAt) {
          // Turned along the wall there.
          const a = Math.round(((Math.atan2(y - hy, x - hx) + Math.PI * 2) % (Math.PI * 2)) / (Math.PI * 2) * N);
          const [ax, ay] = pt(a + N - 1), [bx, by] = pt(a + 1), dir = Math.atan2(by - ay, bx - ax);
          if (cat.gate) { put("gate", x, y, gw >= gd ? dir : dir + Math.PI / 2); gates++; }
        }
        // The wall itself, a piece at a time along the line, leaving room for the gates, the water and the streets' ends.
        for (let a = 0; a < N; a++) {
          const [ax, ay] = pt(a), [bx, by] = pt(a + 1), L = Math.hypot(bx - ax, by - ay); if (L < 0.5) continue;
          const n = Math.max(1, Math.round(L / long)), dir = Math.atan2(by - ay, bx - ax), rot = alongDepth ? dir : dir + Math.PI / 2;
          for (let i = 0; i < n; i++) {
            const x = ax + (bx - ax) * (i + 0.5) / n, y = ay + (by - ay) * (i + 0.5) / n;
            if (!g.dry(x, y) || gatesAt.some((q) => Math.hypot(q[0] - x, q[1] - y) < Math.max(gd, gw) / 2 + long / 2 + 1)) continue;
            things.push({ k: wk, x: Math.round(x * 10) / 10, y: Math.round(y * 10) / 10, rot: (Math.round(rot * 180 / Math.PI) % 360 + 360) % 360, s: Math.round(L / n / long * 100) / 100, label: "", room: 0, note: stone && founders === "roduro" ? "grown stone wall" : "" });
            pieces++;
          }
        }
        // Watchtowers by the gates of a city or a fort.
        if (cat.watchtower && ["capital", "city", "fort"].includes(kind)) for (const [x, y] of gatesAt) { const a = Math.atan2(y - hy, x - hx); for (const sg of [-1, 1]) { const tx = x + Math.cos(a + sg * Math.PI / 2) * 7, ty = y + Math.sin(a + sg * Math.PI / 2) * 7; if (g.dry(tx, ty) && inside(tx, ty, 2)) put("watchtower", tx, ty, a); } }
        wallNote = ` A ${stone ? (founders === "roduro" ? "grown-stone" : "stone") : "timber"} wall, ${gates} gate${gates === 1 ? "" : "s"}.`;
      }

      const counts = {}; for (const t of things) { const d = cat[t.k]; const p = d && d.group === "building" ? d.people : null; if (p) counts[p] = (counts[p] || 0) + 1; }
      const tents = things.filter((t) => t.k === "tent").length;
      const who = Object.entries(counts).map(([p, c]) => `${c} ${p}`).join(", ") + (tents ? `, ${tents} tents` : "");
      const summary = `Grown: ${PURPOSE_NAME[purpose]}${said ? "" : " (from the land)"}, ${streets.filter((s) => s.main).length} streets and ${streets.filter((s) => !s.main).length} lanes, ${who}.${wallNote}`;
      return { things, roads, purpose, summary, keepFoliage: (f) => placed.every(([x, y, r]) => Math.hypot(f.x - x, f.y - y) > r + 1) && (() => { const k = g.at(f.x, f.y); return k < 0 || !street[k]; })() };
    }
    return { grow, purposeIn: (t) => { const ws = words(t); for (const [w, p] of PURPOSE_WORDS) if (ws.includes(w)) return p; return null; } };
  })();
