  // ---- growing a draft town (Laz, 17 Oct 2026: "Grow a draft" in the builder), after Watabou's
  //      Medieval Fantasy City Generator: the town is wards (Voronoi cells of seeds on a spiral,
  //      dense in the middle) round a square where it began (the quay, the crossing, the road
  //      junction, a hill's shoulder; from the mark's notes and label, else the land). Every edge
  //      between two wards is a street; the main streets run from each gate to the square along
  //      them, by a search that charges for climbing; the roads out cross the land from the gates.
  //      Each ward is a block cut into lots across its longest side, again and again, a house on
  //      each lot fronting its street. Wards have kinds: the square, the temple ground on the high
  //      ground by it, the better houses higher up, merchants round the square, craftsmen by the
  //      gates, a park and a keep in a city, a harbour with piers on a shore. Who builds: the
  //      founders' people, and any the notes name; Qotiro take the high wards; Horaro only ever on
  //      stilts over the water; Ṭaḍoro lodge with others and build nothing. Towns and bigger are
  //      walled along the inner wards' outline (grown stone for Roduro), towers at its corners,
  //      gates where the main streets leave; houses string out along the roads outside, then
  //      fields. Everything is a draft in the place, to move, delete and save like anything else. ----
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
      // (Costs in full precision: rounded to single, a cost could look improvable for ever.)
      const N = g.h.length, dist = new Float64Array(N).fill(Infinity), prev = new Int32Array(N).fill(-1);
      // A binary heap of (cost, cell) in typed arrays.
      let hc = new Float64Array(1024), hk = new Int32Array(1024), hn = 0;
      const push = (c, k) => {
        if (hn === hc.length) { const c2 = new Float64Array(hn * 2), k2 = new Int32Array(hn * 2); c2.set(hc); k2.set(hk); hc = c2; hk = k2; }
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

    // ---- polygons (convex: Voronoi cells and the pieces they're cut into) ----
    const area = (p) => { let a = 0; for (let i = 0; i < p.length; i++) { const [x1, y1] = p[i], [x2, y2] = p[(i + 1) % p.length]; a += x1 * y2 - x2 * y1; } return a / 2; };
    const centroid = (p) => { let x = 0, y = 0, a = 0; for (let i = 0; i < p.length; i++) { const [x1, y1] = p[i], [x2, y2] = p[(i + 1) % p.length], c = x1 * y2 - x2 * y1; a += c; x += (x1 + x2) * c; y += (y1 + y2) * c; } if (Math.abs(a) < 1e-9) return p.reduce((s, q) => [s[0] + q[0] / p.length, s[1] + q[1] / p.length], [0, 0]); return [x / (3 * a), y / (3 * a)]; };
    // Keep the part of `p` where n·q >= c.
    function clipHalf(p, nx, ny, c) {
      const out = [];
      for (let i = 0; i < p.length; i++) {
        const a = p[i], b = p[(i + 1) % p.length], da = a[0] * nx + a[1] * ny - c, db = b[0] * nx + b[1] * ny - c;
        if (da >= 0) out.push(a);
        if ((da >= 0) !== (db >= 0)) { const t = da / (da - db); out.push([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]); }
      }
      return out.length >= 3 ? out : null;
    }
    const clipRect = (p, x0, y0, x1, y1) => { let q = p; for (const [nx, ny, c] of [[1, 0, x0], [-1, 0, -x1], [0, 1, y0], [0, -1, -y1]]) { if (!q) return null; q = clipHalf(q, nx, ny, c); } return q; };
    // Pull each edge of a (counter-clockwise) convex polygon in by its own distance.
    function inset(p, ds) {
      const n = p.length, lines = [];
      for (let i = 0; i < n; i++) {
        const [ax, ay] = p[i], [bx, by] = p[(i + 1) % n], L = Math.hypot(bx - ax, by - ay) || 1, nx = -(by - ay) / L, ny = (bx - ax) / L;
        lines.push([ax + nx * ds[i], ay + ny * ds[i], (bx - ax) / L, (by - ay) / L]);
      }
      const out = [];
      for (let i = 0; i < n; i++) {
        const [px, py, dx, dy] = lines[(i + n - 1) % n], [qx, qy, ex, ey] = lines[i], den = dx * ey - dy * ex;
        if (Math.abs(den) < 1e-9) { out.push([qx, qy]); continue; }
        const t = ((qx - px) * ey - (qy - py) * ex) / den;
        out.push([px + dx * t, py + dy * t]);
      }
      // Gone inside out: too small for streets that wide.
      if (area(out) <= 1 || out.some((q, i) => { const a = out[(i + n - 1) % n], b = out[(i + 1) % n]; return (q[0] - a[0]) * (b[1] - q[1]) - (q[1] - a[1]) * (b[0] - q[0]) < -1e-6; })) return null;
      return out;
    }
    const ccw = (p) => (area(p) < 0 ? p.slice().reverse() : p);
    const segDist = (x, y, ax, ay, bx, by) => { const dx = bx - ax, dy = by - ay, L = dx * dx + dy * dy, t = L ? Math.max(0, Math.min(1, ((x - ax) * dx + (y - ay) * dy) / L)) : 0; return Math.hypot(x - ax - dx * t, y - ay - dy * t); };
    const inPoly = (p, x, y) => { let inside = false; for (let i = 0, j = p.length - 1; i < p.length; j = i++) { const [xi, yi] = p[i], [xj, yj] = p[j]; if ((yi > y) !== (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi) inside = !inside; } return inside; };

    // ---- Voronoi cells, through Bowyer–Watson's triangulation ----
    function voronoi(pts) {
      const P = pts.concat([[-1e5, -1e5], [1e5, -1e5], [0, 1e5]]), n = pts.length;
      const circ = (t) => { const [ax, ay] = P[t[0]], [bx, by] = P[t[1]], [cx, cy] = P[t[2]], d = 2 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by)); if (Math.abs(d) < 1e-12) return { x: 0, y: 0, r2: Infinity }; const ux = ((ax * ax + ay * ay) * (by - cy) + (bx * bx + by * by) * (cy - ay) + (cx * cx + cy * cy) * (ay - by)) / d, uy = ((ax * ax + ay * ay) * (cx - bx) + (bx * bx + by * by) * (ax - cx) + (cx * cx + cy * cy) * (bx - ax)) / d; return { x: ux, y: uy, r2: (ax - ux) ** 2 + (ay - uy) ** 2 }; };
      let tris = [{ v: [n, n + 1, n + 2] }]; tris[0].c = circ(tris[0].v);
      for (let i = 0; i < n; i++) {
        const [px, py] = P[i], keep = [], edges = new Map();
        for (const t of tris) {
          if ((px - t.c.x) ** 2 + (py - t.c.y) ** 2 < t.c.r2) for (const [a, b] of [[t.v[0], t.v[1]], [t.v[1], t.v[2]], [t.v[2], t.v[0]]]) { const k = a < b ? a + "," + b : b + "," + a; edges.set(k, edges.has(k) ? null : [a, b]); }
          else keep.push(t);
        }
        tris = keep;
        for (const e of edges.values()) if (e) { const t = { v: [e[0], e[1], i] }; t.c = circ(t.v); tris.push(t); }
      }
      // Each point's cell: the centres of its triangles, round it in order.
      const around = Array.from({ length: n }, () => []);
      tris.forEach((t, ti) => { for (const v of t.v) if (v < n) around[v].push(ti); });
      const cells = around.map((ts, i) => ts.map((ti) => ({ ti, a: Math.atan2(tris[ti].c.y - P[i][1], tris[ti].c.x - P[i][0]) })).sort((a, b) => a.a - b.a).map((q) => q.ti));
      return { tris, cells, verts: tris.map((t) => [t.c.x, t.c.y]) };
    }

    // ---- the grower: wards round a square, after Watabou's city generator ----
    // o: { pl, mark: { kind, label, notes, founders }, cat (catalogue by key), roll }
    // Wards and size by kind: how many inner wards, and how far they reach (m).
    const SIZE = { capital: [26, 185], city: [18, 145], town: [11, 100], fort: [6, 62], port: [6, 58], village: [7, 66], hamlet: [3, 34], outpost: [3, 28] };
    const PEOPLES = ["roduro", "qotiro", "horaro", "tadoro"];
    function grow(o) {
      const { pl, mark, cat } = o;
      const g = grid(pl), W = g.W, H = g.H;
      const ws = words((mark.notes || "") + " " + (mark.label || ""));
      const rand = rng(hash(pl.key) ^ Math.imul(o.roll + 1, 2654435761));
      const range = (a, b) => a + (b - a) * rand();
      const pickOf = (list) => list[Math.floor(rand() * list.length)];
      const kind = String(mark.kind || "village");
      let [nWards, R] = SIZE[kind] || [4, 40];
      if (hasAny(ws, SMALL)) { nWards = Math.max(2, Math.round(nWards * 0.6)); R *= 0.78; }
      R = Math.min(R, Math.min(W, H) * 0.46);
      const village = !["capital", "city", "town", "fort"].includes(kind);
      const loose = hasAny(ws, LOOSE) || village;
      const walled = hasAny(ws, OPEN) ? false : hasAny(ws, WALLED) || WALLED_KINDS.includes(kind);
      const founders = PEOPLES.includes(String(mark.founders || "").toLowerCase()) ? String(mark.founders).toLowerCase() : "roduro";
      // Who lives here: the founders' people, and any the notes name ("mixed": all three who build).
      const peoples = [founders];
      for (const p of hasAny(ws, ["mixed", "mixing"]) ? ["roduro", "qotiro", "horaro"] : PEOPLES) if (!peoples.includes(p) && ws.some((w) => w.startsWith(p))) peoples.push(p);
      const builders = peoples.filter((p) => p === "roduro" || p === "qotiro"); // Horaro build over the water; Ṭaḍoro lodge with others
      if (!builders.length) builders.push("roduro");
      const why = o.why || {}; const no = (r) => { why[r] = (why[r] || 0) + 1; };

      // ---- what it grew on, and where it began (as before: the words, else the land) ----
      const sea = [], river = []; let top = [-Infinity, 0, 0];
      for (let k = 0; k < g.h.length; k++) {
        const [x, y] = g.pos(k), d = Math.hypot(x, y);
        if (g.wet[k] === 1) { if (d < Math.max(W, H) * 0.9) sea.push([d, x, y]); }
        else if (g.wet[k] === 2) { if (d < Math.max(W, H) * 0.6) river.push([d, x, y]); }
        else if (d < R * 0.8 && g.h[k] > top[0]) top = [g.h[k], x, y];
      }
      sea.sort((a, b) => a[0] - b[0]); river.sort((a, b) => a[0] - b[0]);
      const exits = [];
      { let run = null; const on = [];
        for (let a = 0; a < 360; a++) { const r = a * Math.PI / 180, k = g.at(Math.cos(r) * W * 0.5, Math.sin(r) * H * 0.5); on.push(k >= 0 && g.road[k] === 1); }
        for (let a = 0; a < 360; a++) { if (on[a]) { if (!run) run = [a, a]; else run[1] = a; } else if (run) { exits.push(run); run = null; } }
        if (run) { if (exits.length && exits[0][0] === 0) exits[0][0] = run[0] - 360; else exits.push(run); } }
      const exitPts = exits.map(([a0, a1]) => { const r = ((a0 + a1) / 2) * Math.PI / 180; return [Math.cos(r) * W * 0.5, Math.sin(r) * H * 0.5]; });
      const rimH = []; for (let a = 0; a < 64; a++) { const k = g.at(Math.cos(a * 0.098) * R, Math.sin(a * 0.098) * R); if (k >= 0) rimH.push(g.h[k]); }
      rimH.sort((a, b) => a - b);
      const rise = top[0] - (rimH[rimH.length >> 1] || top[0]);
      const found = sea.length > 30 ? "harbour" : river.length > 10 ? "ford" : rise > 14 ? "hill" : exitPts.length >= 3 ? "crossroads" : exitPts.length === 2 ? "street" : "green";
      let said = null; for (const [w, p] of PURPOSE_WORDS) if (ws.includes(w)) { said = p; break; }
      const purpose = said === "harbour" && !sea.length ? found : said === "ford" && !river.length ? found : said || found;
      let heart;
      if (purpose === "harbour" && sea.length) {
        const [, sx, sy] = sea[0]; let best = null;
        for (let k = 0; k < g.h.length; k++) {
          if (g.wet[k]) continue;
          const [x, y] = g.pos(k), d = Math.hypot(x - sx, y - sy); if (d > 70 || d < 24) continue;
          const s = g.slope(k) * 30 + Math.abs(d - 40) / 30; if (!best || s < best[0]) best = [s, x, y];
        }
        heart = best ? [best[1], best[2]] : flattest(g, 0, 0, 60);
      } else if (purpose === "ford" && river.length) {
        const [, rx, ry] = river[0]; let best = null;
        for (let k = 0; k < g.h.length; k++) { if (g.wet[k]) continue; const [x, y] = g.pos(k), d = Math.hypot(x - rx, y - ry); if (d < 20 || d > 50) continue; const s = -g.h[k] * 0.3 + g.slope(k) * 30; if (!best || s < best[0]) best = [s, x, y]; }
        heart = best ? [best[1], best[2]] : flattest(g, 0, 0, 60);
      } else if (purpose === "hill" || purpose === "fort") heart = flattest(g, top[1] * 0.5, top[2] * 0.5, 40);
      else if ((purpose === "crossroads" || purpose === "street") && g.road.some((v) => v)) {
        let best = null; for (let k = 0; k < g.h.length; k++) if (g.road[k]) { const [x, y] = g.pos(k), d = Math.hypot(x, y); if (!best || d < best[0]) best = [d, x, y]; }
        heart = flattest(g, best[1], best[2], 16);
      } else heart = flattest(g, 0, 0, 40);
      // (Inside the place, far enough in for the town round it.)
      const m = Math.min(R * 0.55, Math.min(W, H) * 0.3);
      if (Math.abs(heart[0]) > W / 2 - m || Math.abs(heart[1]) > H / 2 - m) heart = flattest(g, Math.max(-W / 2 + m, Math.min(W / 2 - m, heart[0])), Math.max(-H / 2 + m, Math.min(H / 2 - m, heart[1])), 15);
      const [hx, hy] = heart;

      // ---- the wards: seeds on a spiral (dense in the middle), their Voronoi cells ----
      const sa = rand() * Math.PI * 2, nSeeds = Math.max(nWards * 5, 24), raw = [];
      for (let i = 0; i < nSeeds; i++) { const a = sa + Math.sqrt(i) * 5, r = i === 0 ? 0 : 10 + i * (2 + rand()); raw.push([Math.cos(a) * r, Math.sin(a) * r]); }
      const rIn = Math.hypot(...raw[Math.min(nWards, raw.length - 1)]) || 1, sc = (R * 0.8) / rIn;
      let seeds = raw.map(([x, y]) => [hx + x * sc, hy + y * sc]);
      const frame = []; const far = Math.max(W, H) * 1.5 + R * 3; for (let a = 0; a < 24; a++) frame.push([hx + Math.cos(a / 24 * Math.PI * 2) * far, hy + Math.sin(a / 24 * Math.PI * 2) * far]);
      let V = voronoi(seeds.concat(frame));
      // The middle wards evened out (three rounds of Lloyd's relaxation).
      for (let it = 0; it < 3; it++) { for (let i = 0; i < 3 && i < nSeeds; i++) seeds[i] = centroid(V.cells[i].map((t) => V.verts[t])); V = voronoi(seeds.concat(frame)); }
      const wetAt = (x, y) => { const k = g.at(x, y); return k < 0 ? 1 : g.wet[k]; };
      const patches = seeds.map((s, i) => {
        const ids = V.cells[i], poly = ids.map((t) => V.verts[t]);
        const c = centroid(poly);
        let wet = 0, n = 0, hs = 0, slope = 0;
        for (const q of [c].concat(poly.map((v) => [c[0] + (v[0] - c[0]) * 0.6, c[1] + (v[1] - c[1]) * 0.6]))) { const k = g.at(q[0], q[1]); n++; if (k < 0 || g.wet[k]) wet++; else { hs += g.h[k]; slope += g.slope(k); } }
        const dry = n - wet;
        return { i, ids, poly, c, wet: wet / n, h: dry ? hs / dry : -99, slope: dry ? slope / dry : 1, d: Math.hypot(s[0] - hx, s[1] - hy), inner: false, ward: "" };
      });
      const byD = patches.slice().sort((a, b) => a.d - b.d);
      let innerN = 0;
      for (const p of byD) { if (innerN >= nWards) break; if (p.wet < 0.5 && Math.abs(p.c[0]) < W / 2 - 6 && Math.abs(p.c[1]) < H / 2 - 6) { p.inner = true; innerN++; } }
      patches[0].inner = true;
      // Edges: each Voronoi edge (between two cells' shared corners) and the wards either side.
      const edges = new Map(), ekey = (a, b) => (a < b ? a + "," + b : b + "," + a);
      patches.forEach((p) => { for (let i = 0; i < p.ids.length; i++) { const a = p.ids[i], b = p.ids[(i + 1) % p.ids.length], k = ekey(a, b); const e = edges.get(k) || { a: Math.min(a, b), b: Math.max(a, b), sides: [] }; e.sides.push(p.i); edges.set(k, e); } });
      const P = (t) => V.verts[t];
      const isInner = (i) => i < patches.length && patches[i].inner;
      for (const e of edges.values()) {
        const ins = e.sides.filter(isInner).length;
        e.interior = ins === 2; e.rim = ins === 1;
        const [ax, ay] = P(e.a), [bx, by] = P(e.b); e.len = Math.hypot(bx - ax, by - ay);
        e.wet = wetAt((ax + bx) / 2, (ay + by) / 2);
      }

      // ---- the rim (the wall's line) and its gates ----
      const rimAdj = new Map();
      for (const e of edges.values()) if (e.rim) { for (const [u, v] of [[e.a, e.b], [e.b, e.a]]) { if (!rimAdj.has(u)) rimAdj.set(u, []); rimAdj.get(u).push(v); } }
      let loop = [];
      { const seen = new Set();
        for (const s of rimAdj.keys()) { if (seen.has(s)) continue; const l = [s]; seen.add(s); let prev = -1, cur = s; for (;;) { const nx = (rimAdj.get(cur) || []).find((v) => v !== prev && !seen.has(v)); if (nx === undefined) break; l.push(nx); seen.add(nx); prev = cur; cur = nx; } if (l.length > loop.length) loop = l; } }
      const onLoop = new Set(loop);
      // Gates: where the land's roads come in (the rim corner nearest each), at least two.
      const gates = [];
      const nearestLoop = (x, y) => loop.reduce((b, t) => { const [px, py] = P(t), d = Math.hypot(px - x, py - y); return !b || d < b[0] ? [d, t] : b; }, null);
      for (const [x, y] of exitPts) { const b = nearestLoop(x, y); if (b && !gates.includes(b[1]) && wetAt(...P(b[1])) === 0) gates.push(b[1]); }
      if (gates.length < 2 && loop.length) {
        // Toward the easiest open ground: the rim corners farthest apart that look out on dry land.
        const dryOut = loop.filter((t) => { const [x, y] = P(t), dx = x - hx, dy = y - hy, L = Math.hypot(dx, dy) || 1; return wetAt(x + dx / L * 20, y + dy / L * 20) === 0 && Math.abs(x) < W / 2 - 4 && Math.abs(y) < H / 2 - 4; });
        while (gates.length < 2 && dryOut.length) {
          const pick = gates.length ? dryOut.reduce((b, t) => { const d = Math.min(...gates.map((q) => Math.hypot(P(q)[0] - P(t)[0], P(q)[1] - P(t)[1]))); return !b || d > b[0] ? [d, t] : b; }, null)[1] : pickOf(dryOut);
          gates.push(pick); dryOut.splice(dryOut.indexOf(pick), 1);
        }
      }
      if (purpose === "harbour" && sea.length) {
        // A sea gate: the rim corner nearest the water.
        const b = loop.filter((t) => wetAt(...P(t)) === 0).reduce((b, t) => { const [x, y] = P(t), d = Math.hypot(x - sea[0][1], y - sea[0][2]); return !b || d < b[0] ? [d, t] : b; }, null);
        if (b && !gates.includes(b[1])) gates.push(b[1]);
      }

      // ---- the main streets: from each gate to the square, along the wards' edges ----
      const adj = new Map();
      for (const e of edges.values()) if ((e.interior || e.rim) && e.wet !== 1) { for (const [u, v] of [[e.a, e.b], [e.b, e.a]]) { if (!adj.has(u)) adj.set(u, []); adj.get(u).push([v, e]); } }
      const hAt = (t) => { const k = g.at(...P(t)); return k < 0 ? 0 : g.h[k]; };
      const plazaV = new Set(patches[0].ids);
      const artery = new Set();
      const arteries = [];
      for (const gt of gates) {
        const dist = new Map([[gt, 0]]), prev = new Map(), open = [[0, gt]]; let end = null;
        const popMin = () => { let bi = 0; for (let i = 1; i < open.length; i++) if (open[i][0] < open[bi][0]) bi = i; const top = open[bi]; open[bi] = open[open.length - 1]; open.pop(); return top; };
        while (open.length) {
          const [d, u] = popMin();
          if (d > (dist.get(u) ?? Infinity)) continue;
          if (plazaV.has(u)) { end = u; break; }
          for (const [v, e] of adj.get(u) || []) {
            if (e.rim && !(onLoop.has(u) && u === gt)) { if (!e.interior && !(u === gt)) continue; }
            const grade = Math.abs(hAt(v) - hAt(u)) / (e.len || 1);
            const c = e.len * (1 + 8 * grade + 90 * grade * grade) + (e.wet === 2 ? 60 : 0) + (artery.has(ekey(e.a, e.b)) ? -e.len * 0.5 : 0);
            const nd = d + Math.max(0.1, c);
            if (nd < (dist.get(v) ?? Infinity)) { dist.set(v, nd); prev.set(v, u); open.push([nd, v]); }
          }
        }
        if (end === null) continue;
        const chain = [end]; while (chain[chain.length - 1] !== gt) chain.push(prev.get(chain[chain.length - 1]));
        for (let i = 0; i + 1 < chain.length; i++) artery.add(ekey(chain[i], chain[i + 1]));
        // On into the square's middle.
        arteries.push(chain.map(P).concat([[hx, hy]]));
      }

      // ---- the roads out: from each gate across the land to the edge of the place ----
      const roads = [];
      const r1 = (q) => [Math.round(q[0] * 10) / 10, Math.round(q[1] * 10) / 10];
      const city = kind === "capital" || kind === "city";
      for (const pts of arteries) roads.push({ k: city ? "cobbles" : "road", w: village ? 4.5 : 6, pts: pts.reverse().map(r1) });
      const noStreet = new Uint8Array(g.h.length);
      const outsideRoads = [];
      for (const gt of gates) {
        const [gx, gy] = P(gt), dx = gx - hx, dy = gy - hy, L = Math.hypot(dx, dy) || 1;
        // Toward the road that comes in here, else straight out.
        const ex = exitPts.reduce((b, q) => { const d = Math.hypot(q[0] - gx, q[1] - gy); return !b || d < b[0] ? [d, q] : b; }, null);
        const goalPt = ex && ex[0] < Math.max(W, H) * 0.6 ? ex[1] : [Math.max(-W / 2, Math.min(W / 2, gx + dx / L * W)), Math.max(-H / 2, Math.min(H / 2, gy + dy / L * H))];
        const goal = new Uint8Array(g.h.length); let any = false;
        for (let k = 0; k < g.h.length; k++) { const [x, y] = g.pos(k); if (Math.hypot(x - goalPt[0], y - goalPt[1]) < 10 && !g.wet[k]) { goal[k] = 1; any = true; } }
        const from = g.at(gx + dx / L * 6, gy + dy / L * 6);
        const cells = any && from >= 0 ? path(g, from, goal, noStreet, purpose === "ford", Infinity) : null;
        if (cells && cells.length > 1) { const pts = [[gx, gy]].concat(line(g, cells)); outsideRoads.push(pts); roads.push({ k: "road", w: village ? 4 : 5, pts: pts.map(r1) }); }
      }

      // ---- the other streets: every edge between two inner wards ----
      const streetEdges = [...edges.values()].filter((e) => e.interior && !artery.has(ekey(e.a, e.b)) && e.wet !== 1);
      { // Joined into runs through corners where only two meet, so they read as streets.
        const sAdj = new Map(); for (const e of streetEdges) for (const [u, v] of [[e.a, e.b], [e.b, e.a]]) { if (!sAdj.has(u)) sAdj.set(u, []); sAdj.get(u).push(v); }
        const used = new Set();
        for (const e of streetEdges) {
          const k0 = ekey(e.a, e.b); if (used.has(k0)) continue; used.add(k0);
          const run = [e.a, e.b];
          for (const dir of [0, 1]) {
            for (;;) {
              const end = dir ? run[run.length - 1] : run[0], prevV = dir ? run[run.length - 2] : run[1];
              const nb = sAdj.get(end) || []; if (nb.length !== 2) break;
              const nx = nb[0] === prevV ? nb[1] : nb[0], k = ekey(end, nx); if (used.has(k)) break; used.add(k);
              if (dir) run.push(nx); else run.unshift(nx);
            }
          }
          roads.push({ k: village ? "path" : "road", w: village ? 3 : 4, pts: run.map((t) => r1(P(t))) });
        }
      }

      // ---- what each ward is ----
      const inner = patches.filter((p) => p.inner);
      const plaza = patches[0]; plaza.ward = "plaza";
      const nextTo = (p, q) => p.ids.some((t) => q.ids.includes(t));
      const free = () => inner.filter((p) => !p.ward);
      const landmark = free().filter((p) => p.d < R * 0.7 && p.slope < 0.25).sort((a, b) => (b.h - b.d * 0.05) - (a.h - a.d * 0.05))[0];
      if (landmark) landmark.ward = "temple";
      if (walled && (kind === "fort" || city || purpose === "fort")) {
        const keep = free().filter((p) => p.ids.some((t) => onLoop.has(t))).sort((a, b) => b.h - a.h)[0];
        if (keep) keep.ward = "keep";
      }
      const shore = (p) => p.poly.some((v) => wetAt(v[0], v[1]) === 1) || patches.some((q) => q.wet >= 0.5 && nextTo(p, q) && q.c && wetAt(q.c[0], q.c[1]) === 1);
      if (sea.length) for (const p of free()) if (shore(p)) p.ward = "harbour";
      if (city) { const pk = free().filter((p) => p.d > R * 0.3).sort(() => rand() - 0.5)[0]; if (pk) pk.ward = "park"; }
      // The rest are homes; the highest one or two the better houses; by the square, the merchants;
      // by the gates, the craftsmen.
      const homesW = free();
      homesW.sort((a, b) => b.h - a.h).slice(0, Math.max(1, Math.round(homesW.length / 6))).forEach((p) => (p.ward = "rich"));
      for (const p of free()) p.ward = nextTo(p, plaza) ? "market" : p.ids.some((t) => gates.includes(t)) ? "craft" : "common";
      // Who builds in each: the founders, unless the notes name others; Qotiro take the high wards.
      const housing = inner.filter((p) => ["rich", "market", "craft", "common", "harbour"].includes(p.ward));
      for (const p of housing) p.people = founders === "qotiro" ? "qotiro" : "roduro";
      if (builders.length > 1) {
        const q = housing.filter((p) => p.ward !== "harbour").sort((a, b) => (b.h + b.slope * 40) - (a.h + a.slope * 40));
        const share = founders === "qotiro" ? 0.7 : 0.3;
        q.slice(0, Math.max(1, Math.round(q.length * share))).forEach((p) => (p.people = "qotiro"));
        if (founders === "qotiro") q.slice(Math.round(q.length * share)).forEach((p) => (p.people = "roduro"));
      }

      // ---- building it ----
      const things = [], foliage = [], paint = [], placed = [];
      const dims = (key) => { const d = cat[key]; return d ? [d.depth, d.width] : [8, 8]; };
      const deg = (r) => (Math.round(r * 180 / Math.PI) % 360 + 360) % 360;
      const insideArea = (x, y, r) => Math.abs(x) + r < W / 2 - 1 && Math.abs(y) + r < H / 2 - 1;
      const put = (k, x, y, rot, s, note) => { const d = cat[k]; if (!d) return false; things.push({ k, x: Math.round(x * 10) / 10, y: Math.round(y * 10) / 10, rot: deg(rot), s: Math.round((s || 1) * 100) / 100, label: "", room: d.beds ? Math.round(d.beds * (s || 1) * (s || 1)) : 0, note: note || "" }); placed.push([x, y, Math.hypot(d.depth, d.width) / 2 * (s || 1)]); return true; };
      // A ward's block: its cell pulled in from each edge by half that street's width (more off the wall).
      const block = (p) => {
        const poly = ccw(p.ids.map(P)), ids = area(p.ids.map(P)) < 0 ? p.ids.slice().reverse() : p.ids;
        const ds = ids.map((a, i) => { const b = ids[(i + 1) % ids.length], e = edges.get(ekey(a, b)); if (!e) return 2; if (artery.has(ekey(a, b))) return village ? 2.6 : 3.4; if (e.rim) return walled ? 3.5 : 1; return village ? 1.7 : 2.2; });
        const b = inset(poly, ds); if (!b) return null;
        return clipRect(b, -W / 2 + 2, -H / 2 + 2, W / 2 - 2, H / 2 - 2);
      };
      // Cut a block into lots: across its longest edge, again and again (Watabou's), an alley now and then.
      // (A cut with a gap is an alley through the block: a path, and houses may front it.)
      const alleys = [];
      const cut = (poly, minSq, chaos, alley, out, depth) => {
        const A = area(poly);
        if (A < minSq * (1.4 + rand() * 0.8) || depth > 9) { out.push(poly); return; }
        // A deep block gets a lane through it, so its middle has a way in.
        const deep = A > minSq * 7;
        let li = 0, ll = 0; for (let i = 0; i < poly.length; i++) { const a = poly[i], b = poly[(i + 1) % poly.length], L = Math.hypot(b[0] - a[0], b[1] - a[1]); if (L > ll) { ll = L; li = i; } }
        const a = poly[li], b = poly[(li + 1) % poly.length], t = 0.5 + (rand() - 0.5) * 0.35 * chaos;
        const px = a[0] + (b[0] - a[0]) * t, py = a[1] + (b[1] - a[1]) * t;
        const ang = Math.atan2(b[1] - a[1], b[0] - a[0]) + (rand() - 0.5) * chaos * Math.PI / 7;
        const nx = Math.cos(ang), ny = Math.sin(ang), c = nx * px + ny * py, gap = deep ? 1.6 : rand() < alley ? 1.1 : 0;
        const h1 = clipHalf(poly, nx, ny, c + gap), h2 = clipHalf(poly, -nx, -ny, -c + gap);
        if (!h1 || !h2) { out.push(poly); return; }
        if (gap) {
          // Where the cut crosses the block: the alley's line.
          const xs = [];
          for (let i = 0; i < poly.length; i++) { const a2 = poly[i], b2 = poly[(i + 1) % poly.length], da = a2[0] * nx + a2[1] * ny - c, db = b2[0] * nx + b2[1] * ny - c; if ((da >= 0) !== (db >= 0)) { const t2 = da / (da - db); xs.push([a2[0] + (b2[0] - a2[0]) * t2, a2[1] + (b2[1] - a2[1]) * t2]); } }
          if (xs.length === 2) alleys.push([xs[0], xs[1], gap]);
        }
        cut(h1, minSq, chaos, alley, out, depth + 1); cut(h2, minSq, chaos, alley, out, depth + 1);
      };
      // A lot's house: on its street side (the edge it shares with the block's outline), door to the street.
      const KEYS = {
        roduro: { common: ["roduro_cottage", "roduro_cottage", "roduro_longhouse"], rich: ["roduro_great", "roduro_longhouse"], market: ["roduro_trader", "roduro_cottage", "roduro_longhouse"], craft: ["roduro_forge", "roduro_loomroom", "roduro_benchroom", "roduro_tender", "roduro_cottage", "roduro_longhouse"], harbour: ["roduro_benchroom", "roduro_cottage", "roduro_longhouse"] },
        qotiro: { common: ["qotiro_quarters", "qotiro_court"], rich: ["qotiro_court"], market: ["qotiro_market", "qotiro_quarters"], craft: ["qotiro_workyard", "qotiro_weavehall", "qotiro_benchyard", "qotiro_quarters"], harbour: ["qotiro_benchyard", "qotiro_quarters"] },
      };
      const lotHouse = (lot, blockPoly, p) => {
        let best = null;
        for (let i = 0; i < lot.length; i++) {
          const a = lot[i], b = lot[(i + 1) % lot.length], mx = (a[0] + b[0]) / 2, my = (a[1] + b[1]) / 2, L = Math.hypot(b[0] - a[0], b[1] - a[1]);
          let onEdge = false; for (let j = 0; j < blockPoly.length; j++) { const c = blockPoly[j], d = blockPoly[(j + 1) % blockPoly.length]; if (segDist(mx, my, c[0], c[1], d[0], d[1]) < 0.3) { onEdge = true; break; } }
          if (!onEdge) for (const [c, d, gp] of alleys) if (segDist(mx, my, c[0], c[1], d[0], d[1]) < gp + 0.3) { onEdge = true; break; }
          if (onEdge && L > 3 && (!best || L > best[2])) best = [a, b, L];
        }
        if (!best) { no("no street side"); return; }
        const [a, b, L] = best, ux = (b[0] - a[0]) / L, uy = (b[1] - a[1]) / L, cen = centroid(lot);
        let nx = -uy, ny = ux; if ((cen[0] - a[0]) * nx + (cen[1] - a[1]) * ny < 0) { nx = -nx; ny = -ny; }
        const depthAvail = Math.max(...lot.map((q) => (q[0] - a[0]) * nx + (q[1] - a[1]) * ny));
        const list = KEYS[p.people][p.ward] || KEYS[p.people].common;
        for (const key of [pickOf(list)].concat(list)) {
          const [dd, dw] = dims(key);
          let s = Math.min(depthAvail * 0.9 / dd, L * 0.92 / dw); if (s < 0.72) continue; s = Math.min(s, p.ward === "rich" ? 1.3 : 1.12);
          const mx = (a[0] + b[0]) / 2, my = (a[1] + b[1]) / 2, x = mx + nx * (dd * s / 2 + 0.4), y = my + ny * (dd * s / 2 + 0.4);
          const k = g.at(x, y);
          if (k < 0 || g.wet[k]) { no("wet"); return; }
          if (g.slope(k) > (p.people === "qotiro" ? 0.34 : 0.24)) { no("steep"); return; }
          if (!insideArea(x, y, Math.hypot(dd, dw) / 2 * s)) { no("outside"); return; }
          put(key, x, y, Math.atan2(-ny, -nx), s, g.slope(k) > 0.16 ? "terraced" : "");
          return;
        }
        no("lot too small");
      };
      const LOT = { roduro: 95, qotiro: 230 }; // a lot's area, m², about a house and its yard

      let homes = 0;
      for (const p of inner) {
        const bl = block(p); if (!bl) continue;
        const cen = centroid(bl);
        if (p.ward === "plaza") {
          // A town's square is paved; a village's green stays grass.
          if (!village) for (let j = 0; j < pl.GH; j++) for (let i = 0; i < pl.GW; i++) { const x = (i - pl.P.origin[0] + 0.5) * pl.CELL, y = (j - pl.P.origin[1] + 0.5) * pl.CELL; if (inPoly(bl, x, y)) paint.push([j * pl.GW + i, "v"]); }
          if (city || kind === "town") put("market", cen[0], cen[1], 0, Math.min(1.3, Math.sqrt(area(bl)) / 16 * 0.6));
          if (cat.well) put("well", cen[0] + 7, cen[1] + 7, 0);
          const nStalls = city ? 8 : kind === "town" ? 5 : 3;
          for (let i = 0; i < nStalls; i++) { const a = i / nStalls * Math.PI * 2 + 0.3, x = cen[0] + Math.cos(a) * 12, y = cen[1] + Math.sin(a) * 12; if (inPoly(bl, x, y) && cat.stall) put("stall", x, y, a + Math.PI); }
          continue;
        }
        if (p.ward === "temple") {
          const key = founders === "qotiro" && cat.qotiro_temple ? "qotiro_temple" : cat.roduro_great ? "roduro_great" : null;
          if (key) { const [dd, dw] = dims(key), s = Math.max(0.6, Math.min(1.4, Math.sqrt(area(bl)) * 0.55 / Math.max(dd, dw))); put(key, cen[0], cen[1], Math.atan2(hy - cen[1], hx - cen[0]), s, "the landmark: the temple ground, highest by the square"); }
          if (founders !== "qotiro" && cat.shrine) put("shrine", cen[0] + 10, cen[1] - 8, Math.atan2(hy - cen[1], hx - cen[0]));
          for (let i = 0; i < 10; i++) { const a = rand() * Math.PI * 2, r = 9 + rand() * 12, x = cen[0] + Math.cos(a) * r, y = cen[1] + Math.sin(a) * r; if (inPoly(bl, x, y) && placed.every(([qx, qy, qr]) => Math.hypot(qx - x, qy - y) > qr + 2)) foliage.push({ k: "tree_broad", x: Math.round(x * 10) / 10, y: Math.round(y * 10) / 10, s: Math.round((0.8 + rand() * 0.5) * 100) / 100 }); }
          continue;
        }
        if (p.ward === "park") {
          for (let i = 0; i < Math.round(area(bl) / 60); i++) { const x = cen[0] + (rand() - 0.5) * 60, y = cen[1] + (rand() - 0.5) * 60; if (inPoly(bl, x, y)) foliage.push({ k: rand() < 0.7 ? "tree_broad" : "bush", x: Math.round(x * 10) / 10, y: Math.round(y * 10) / 10, s: Math.round((0.8 + rand() * 0.5) * 100) / 100 }); }
          continue;
        }
        if (p.ward === "keep") {
          // The keep: its own wall and towers round its block, the hall and the guard in it.
          const k2 = inset(bl, bl.map(() => 2));
          if (k2) {
            for (let i = 0; i < k2.length; i++) { const a = k2[i], b = k2[(i + 1) % k2.length]; wallRun(a, b, "rubble_wall", i === 0 ? 0.5 : -1); if (cat.watchtower) put("watchtower", a[0], a[1], 0); }
            if (cat.hall) put("hall", cen[0], cen[1], Math.atan2(hy - cen[1], hx - cen[0]));
            if (cat.guard_post) put("guard_post", cen[0] + 9, cen[1] + 6, Math.atan2(hy - cen[1], hx - cen[0]));
          }
          continue;
        }
        // Homes (and in the harbour, its stores).
        const lots = [];
        const minSq = (LOT[p.people] || 100) * (p.ward === "rich" ? 1.8 : p.ward === "harbour" ? 1.3 : 1) * (loose ? 1.5 : 1) * (0.85 + p.d / R * 0.5);
        cut(bl, minSq, loose ? 0.8 : 0.45, village ? 0.15 : 0.3, lots, 0);
        for (const lot of lots) {
          // Some lots stay gardens: more toward the edge, more in a village.
          if (rand() < (village ? 0.18 : 0.05) + 0.15 * Math.max(0, p.d / R - 0.6)) { no("garden"); continue; }
          if (p.ward === "harbour" && rand() < 0.35 && cat.storehouse) { const c = centroid(lot); if (g.dry(c[0], c[1])) put("storehouse", c[0], c[1], Math.atan2(hy - c[1], hx - c[0])); continue; }
          const n0 = things.length; lotHouse(lot, bl, p); if (things.length > n0) homes++;
        }
      }

      // The alleys cut through the blocks, as paths.
      for (const [a, b, gp] of alleys) roads.push({ k: "path", w: Math.max(1.6, gp * 2 - 0.4), pts: [r1(a), r1(b)] });

      // ---- the harbour: piers out from the shore by the town, Horaro homes on stilts beyond ----
      let piers = 0, stilts = 0;
      if (sea.length) {
        const harbourW = inner.filter((p) => p.ward === "harbour" || p === plaza);
        const shoreSpots = [];
        for (const p of harbourW) for (const v of p.ids.map(P)) if (wetAt(v[0], v[1]) === 1) shoreSpots.push(v);
        // A pier at a few spots on the town's shore, straight out over the water.
        const fine = (x, y) => { const i = Math.floor(x / pl.CELL + pl.P.origin[0]), j = Math.floor(y / pl.CELL + pl.P.origin[1]); if (i < 0 || j < 0 || i >= pl.GW || j >= pl.GH) return false; const q = j * pl.GW + i, ch = pl.ground[q] !== "." ? pl.ground[q] : pl.natG ? pl.natG[q] : "g"; return ch === "~"; };
        const pierEnds = [];
        for (let k = 0; k < g.h.length && piers < (city ? 5 : 3); k += 1) {
          if (g.wet[k] !== 1) continue;
          const [x, y] = g.pos(k); if (Math.hypot(x - hx, y - hy) > R * 0.9 || !insideArea(x, y, 4)) continue;
          // A shore cell: water here, land beside it.
          const land = [[4, 0], [-4, 0], [0, 4], [0, -4]].find(([dx, dy]) => g.dry(x + dx, y + dy)); if (!land) continue;
          if (pierEnds.some((q) => Math.hypot(q[2] - x, q[3] - y) < 30)) continue;
          const ox = -land[0] / 4, oy = -land[1] / 4; let len = 0;
          while (len < 40 && fine(x + ox * (len + 2), y + oy * (len + 2)) && insideArea(x + ox * (len + 2), y + oy * (len + 2), 1)) len += 2;
          if (len < 12) continue;
          roads.push({ k: "path", w: 2.5, pts: [r1([x - ox * 2, y - oy * 2]), r1([x + ox * len, y + oy * len])] });
          if (cat.dock && piers === 0) put("dock", x + ox * 6, y + oy * 6, Math.atan2(oy, ox));
          pierEnds.push([x + ox * len, y + oy * len, x, y]); piers++;
        }
        if (peoples.includes("horaro")) {
          // Stilt homes on open water near the piers, every corner over the sea, joined by boardwalks.
          const want = Math.max(3, Math.round(nWards * 1.5));
          for (const [ex, ey] of pierEnds) {
            for (let i = 0; i < 40 && stilts < want; i++) {
              const a = rand() * Math.PI * 2, r = 9 + rand() * 18, x = ex + Math.cos(a) * r, y = ey + Math.sin(a) * r;
              const key = pickOf(["horaro_dome", "horaro_twin", "horaro_hull", "horaro_net_loft"]); if (!cat[key]) continue;
              const [dd, dw] = dims(key), rr = Math.hypot(dd, dw) / 2 + 1;
              if (![[0, 0], [rr, 0], [-rr, 0], [0, rr], [0, -rr]].every(([dx, dy]) => fine(x + dx, y + dy))) continue;
              if (!insideArea(x, y, rr) || placed.some(([qx, qy, qr]) => Math.hypot(qx - x, qy - y) < qr + rr + 2)) continue;
              put(key, x, y, Math.atan2(ey - y, ex - x)); stilts++;
              roads.push({ k: "path", w: 2, pts: [r1([ex, ey]), r1([x, y])] });
            }
          }
        }
      }

      // ---- outside the wall: houses along the roads out by the gates, then fields ----
      const outsideLots = [];
      for (const pts of outsideRoads) {
        let along = 0;
        for (let s = 0; s + 1 < pts.length; s++) {
          const [ax, ay] = pts[s], [bx, by] = pts[s + 1], L = Math.hypot(bx - ax, by - ay); if (L < 1e-3) continue;
          const ux = (bx - ax) / L, uy = (by - ay) / L;
          for (let u = 0; u < L; u += 13) {
            const x0 = ax + ux * u, y0 = ay + uy * u, d = along + u;
            for (const sg of [-1, 1]) outsideLots.push([d, x0, y0, -uy * sg, ux * sg]);
          }
          along += L;
        }
      }
      let suburbs = 0, fields = 0;
      for (const [d, x0, y0, nx, ny] of outsideLots) {
        const key = d < 70 ? (founders === "qotiro" ? "qotiro_quarters" : pickOf(["roduro_cottage", "roduro_longhouse"])) : "field_plot";
        if (!cat[key]) continue;
        if (key !== "field_plot" && rand() < 0.35 + d / 140) continue;
        if (key === "field_plot" && (rand() < 0.3 || fields > (hasAny(ws, ["farming", "farms", "fields"]) ? 24 : 14))) continue;
        const [dd, dw] = dims(key), x = x0 + nx * (dd / 2 + 3.5), y = y0 + ny * (dd / 2 + 3.5), k = g.at(x, y), rr = Math.hypot(dd, dw) / 2;
        if (k < 0 || g.wet[k] || g.slope(k) > (key === "field_plot" ? 0.12 : 0.24) || !insideArea(x, y, rr)) continue;
        if (walled && loop.length && inPoly(loop.map(P), x, y)) continue;
        if (placed.some(([qx, qy, qr]) => Math.hypot(qx - x, qy - y) < qr + rr + 1.5)) continue;
        if (segNear(x, y, rr * 0.7)) continue;
        put(key, x, y, Math.atan2(-ny, -nx)); if (key === "field_plot") fields++; else suburbs++;
      }
      function segNear(x, y, r) { for (const rd of roads) for (let i = 0; i + 1 < rd.pts.length; i++) if (segDist(x, y, rd.pts[i][0], rd.pts[i][1], rd.pts[i + 1][0], rd.pts[i + 1][1]) < r + rd.w / 2) return true; return false; }

      // ---- the wall, its towers and gates ----
      function wallRun(a, b, key, gateAt) {
        const [dd, dw] = dims(key), long = Math.max(dd, dw), alongDepth = dd >= dw;
        const L = Math.hypot(b[0] - a[0], b[1] - a[1]), n = Math.max(1, Math.round(L / long)), dir = Math.atan2(b[1] - a[1], b[0] - a[0]);
        for (let i = 0; i < n; i++) {
          const t = (i + 0.5) / n, x = a[0] + (b[0] - a[0]) * t, y = a[1] + (b[1] - a[1]) * t;
          if (gateAt >= 0 && Math.abs(t - gateAt) < 0.5 / n + 0.01) { const [gd, gw] = dims("gate"); put("gate", x, y, gw >= gd ? dir : dir + Math.PI / 2); continue; }
          if (wetAt(x, y) === 1 || !insideArea(x, y, 0.5)) continue;
          things.push({ k: key, x: Math.round(x * 10) / 10, y: Math.round(y * 10) / 10, rot: deg(alongDepth ? dir : dir + Math.PI / 2), s: Math.round(L / n / long * 100) / 100, label: "", room: 0, note: key === "rubble_wall" && founders === "roduro" ? "grown stone wall" : "" });
        }
      }
      let gatesBuilt = 0;
      if (walled && loop.length > 2) {
        const stone = founders === "roduro" || founders === "qotiro", wk = stone ? "rubble_wall" : "palisade";
        for (let i = 0; i < loop.length; i++) {
          const a = P(loop[i]), b = P(loop[(i + 1) % loop.length]);
          wallRun(a, b, wk, -1);
          if (cat.watchtower && wetAt(a[0], a[1]) !== 1 && insideArea(a[0], a[1], 2) && (city || kind === "fort" || i % 2 === 0) && !gates.includes(loop[i])) put("watchtower", a[0], a[1], 0);
        }
        for (const gt of gates) {
          const i = loop.indexOf(gt); if (i < 0) continue;
          const a = P(loop[(i + loop.length - 1) % loop.length]), b = P(loop[(i + 1) % loop.length]), [x, y] = P(gt), dir = Math.atan2(b[1] - a[1], b[0] - a[0]);
          // The gate where the street comes through, and the wall cleared either side of it.
          for (let j = things.length - 1; j >= 0; j--) if ((things[j].k === wk) && Math.hypot(things[j].x - x, things[j].y - y) < 6) things.splice(j, 1);
          const [gd, gw] = dims("gate"); if (put("gate", x, y, gw >= gd ? dir : dir + Math.PI / 2)) gatesBuilt++;
          if (cat.watchtower) for (const sg of [-1, 1]) { const tx = x + Math.cos(dir) * 6 * sg, ty = y + Math.sin(dir) * 6 * sg; if (wetAt(tx, ty) !== 1 && insideArea(tx, ty, 2)) put("watchtower", tx, ty, dir); }
        }
      }

      const counts = {}; for (const t of things) { const d = cat[t.k]; if (d && d.group === "building") counts[d.people] = (counts[d.people] || 0) + 1; }
      const who = Object.entries(counts).map(([p, c]) => `${c} ${p}`).join(", ");
      const wardsSaid = inner.map((p) => p.ward).reduce((m, w) => ((m[w] = (m[w] || 0) + 1), m), {});
      const summary = `Grown: ${PURPOSE_NAME[purpose]}${said ? "" : " (from the land)"} of ${inner.length} wards (${Object.entries(wardsSaid).map(([w, c]) => (c > 1 ? c + " " : "") + w).join(", ")}): ${who || "no buildings"}${piers ? `, ${piers} piers` : ""}${suburbs ? `, ${suburbs} houses outside` : ""}${fields ? `, ${fields} fields` : ""}.${walled ? ` A ${founders === "roduro" ? "grown-stone" : founders === "qotiro" ? "stone" : "timber"} wall, ${gatesBuilt} gate${gatesBuilt === 1 ? "" : "s"}.` : ""}`;
      const roadNear = (x, y) => roads.some((rd) => rd.pts.some((q, i) => i + 1 < rd.pts.length && segDist(x, y, q[0], q[1], rd.pts[i + 1][0], rd.pts[i + 1][1]) < rd.w / 2 + 1));
      return { things, roads, foliage, paint, purpose, summary, keepFoliage: (f) => placed.every(([x, y, r]) => Math.hypot(f.x - x, f.y - y) > r + 1) && !roadNear(f.x, f.y) && !(walled && loop.length && inPoly(loop.map(P), f.x, f.y) && !inner.some((p) => (p.ward === "park" || p.ward === "temple") && inPoly(p.poly, f.x, f.y))) };
    }
    return { grow, purposeIn: (t) => { const ws = words(t); for (const [w, p] of PURPOSE_WORDS) if (ws.includes(w)) return p; return null; } };
  })();
