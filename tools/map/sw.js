// The map page offline: on first visit everything the page reads is put in the
// cache (the pages, Laz's map, the catalogue, the land under every mark, the data
// files), and after that the page is served from the cache first, so it opens with
// no signal at all. A new site version (version.txt changes) refreshes the cache
// in the background.
const CACHE = "gaht-map-v2";
const CORE = ["./", "index.html", "build.html", "map.png", "roads.png", "draft.png", "catalogue.json", "grounds/index.json", "data/marks-export.json", "data/places.txt", "version.txt", "manifest.json"];

self.addEventListener("install", (e) => {
  e.waitUntil((async () => {
    const c = await caches.open(CACHE);
    await c.addAll(CORE.map((p) => new Request(p, { cache: "reload" })));
    // Every mark's land, and every place file, from the lists.
    try {
      const idx = await (await c.match("grounds/index.json")).json();
      const keys = new Set(Object.values(idx).map((v) => (v && v.key) || v).filter(Boolean));
      await Promise.all([...keys].map((k) => c.add("grounds/" + k + ".json").catch(() => {})));
    } catch (e) {}
    try {
      const list = (await (await c.match("data/places.txt")).text()).split("\n").map((s) => s.trim()).filter(Boolean);
      await Promise.all(list.map((f) => c.add("data/places/" + f).catch(() => {})));
    } catch (e) {}
    self.skipWaiting();
  })());
});
self.addEventListener("activate", (e) => { e.waitUntil((async () => { for (const k of await caches.keys()) if (k !== CACHE) await caches.delete(k); await self.clients.claim(); })()); });
// Pages and data: the network first (a new version shows at once), the cache when there's no
// signal. The heavy, slow-changing files (the map, the land, the catalogue): the cache first,
// refreshed in the background.
const heavy = (url) => /\.(png|jpg|webp|svg)$/.test(url.pathname) || /\/grounds\//.test(url.pathname) || /catalogue\.json$/.test(url.pathname);
self.addEventListener("fetch", (e) => {
  const url = new URL(e.request.url);
  if (e.request.method !== "GET" || url.origin !== location.origin) return;
  e.respondWith((async () => {
    const c = await caches.open(CACHE);
    const fetching = fetch(e.request).then((r) => { if (r && r.ok) c.put(e.request, r.clone()); return r; }).catch(() => null);
    if (heavy(url)) {
      const hit = await c.match(e.request, { ignoreSearch: true });
      if (hit) { e.waitUntil(fetching); return hit; }
      const r = await fetching; return r || new Response("Offline, and not cached yet.", { status: 503, headers: { "Content-Type": "text/plain" } });
    }
    const r = await fetching;
    if (r) return r;
    const hit = await c.match(e.request, { ignoreSearch: true });
    return hit || new Response("Offline, and not cached yet.", { status: 503, headers: { "Content-Type": "text/plain" } });
  })());
});
