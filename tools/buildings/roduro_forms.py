"""The Roduro catalogue's buildings as placeholder shapes for the game
(`claude/placeholder-shapes.md`).

    python tools/buildings/roduro_forms.py spec <catalogue dir> [id ...]
        For each catalogue building: read `<id>_profile.json` (the model's
        outer radius in 72 directions per 0.25 m band, from
        `_tools/roduro_shell_profile.py` in Blender) and `<id>_interior.json`
        (the floor plan), work out the tiers, the ground floor's walls and
        doorways, and a furniture layout, and write
        `assets/buildings/<id>.json` plus `assets/buildings/_plans/<id>.png`.
    python tools/buildings/roduro_forms.py rust
        Write the generated blocks in `src/sim/layout.rs` (the variants and
        their outlines) and `src/view/interiors.rs` (the forms: tiers, roof,
        stair, extra doors) from every `assets/buildings/*.json`.

Metres, the building's own frame: x along its facing (the main door is on
+x), y across, z up. Layout units are metres from the outline's middle
divided by the half depth and half width (`sim/layout.rs`).
"""
import json, math, os, sys, glob

from shapely.geometry import Polygon, Point, LineString, box
from shapely import affinity

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
ASSETS = os.path.join(ROOT, "assets", "buildings")

DIRS_OUT = 36          # corners per tier outline (every 10 degrees)
WALL_IN = 0.34         # sim/layout.rs
INNER_WALL = 0.2
DOORWAY = 1.2
Q = math.pi / 2

# Furniture sizes (sim/layout.rs `Furn::size`, `Holder::size`): width along its own x, depth.
SIZES = dict(Bed=(2.0, 1.0), Bedroll=(1.9, 0.8), Table=(1.4, 0.9), LongTable=(3.6, 0.9), Bench=(1.8, 0.4),
             Shelf=(1.6, 0.4), Hearth=(1.0, 1.0), Chest=(1.1, 0.6), Crate=(0.8, 0.8), Cupboard=(1.2, 0.5),
             Barrel=(0.7, 0.7), Workbench=(2.0, 0.8), Anvil=(0.7, 0.4), Forge=(1.4, 1.2), Kiln=(1.3, 1.3), Loom=(1.6, 0.9),
             Counter=(2.4, 0.7), Rack=(1.8, 0.35), GrowerBed=(1.8, 1.1), Altar=(1.6, 0.9), Screen=(1.6, 0.1))
HOLDERS = {"Chest", "Crate", "Cupboard", "Barrel"}


# ---- The outside: tiers from the profile ----------------------------------------

def tiers_of(p, want):
    """Tiers of the model: (z0, z1, wall_top, radii[72]) from the ground up,
    `want` of them (a home or vault has one, a drum two, an inn three). A
    tier ends where the radius drops by a fifth; its wall ends where the
    radius starts to fall away toward the roof."""
    bands = p["bands"]
    step = p["band_m"]
    n = p["dirs"]

    def mean_r(b):
        rs = [r for r in b["r"] if r > 0]
        return sum(rs) / len(rs) if rs else 0.0

    means = [mean_r(b) for b in bands]
    # Tier boundaries: a drop to under 0.8 of the running tier radius, above 2.5 m.
    bounds = [0]
    ref = None
    for i, m in enumerate(means):
        z = bands[i]["z0"]
        if m == 0:
            continue
        if ref is None:
            ref = m
            continue
        if z >= 2.5 and m < 0.8 * ref and i + 1 < len(means) and means[i + 1] < 0.85 * ref and z + 1.0 < p["height"]:
            bounds.append(i)
            ref = None
        elif ref is not None:
            ref = max(ref, m) if i - bounds[-1] <= 4 else ref
    bounds = bounds[:want] + [len(bands)]
    out = []
    for k in range(len(bounds) - 1):
        i0, i1 = bounds[k], bounds[k + 1]
        z0 = bands[i0]["z0"]
        z1 = bands[i1]["z0"] if i1 < len(bands) else p["height"]
        # The tier's radii: the biggest seen in each direction over its lower
        # two thirds (above the ground band, which is sunk and ragged).
        lo = i0 + (1 if i0 == 0 else 0)
        hi = max(lo + 1, i0 + int((i1 - i0) * 0.66))
        radii = [0.0] * n
        for b in bands[lo:hi]:
            for d in range(n):
                radii[d] = max(radii[d], b["r"][d])
        # Fill directions the sparse mesh missed from their neighbours.
        for d in range(n):
            if radii[d] == 0:
                l, r = radii[(d - 1) % n], radii[(d + 1) % n]
                radii[d] = max(l, r)
        # Smoothed (the sparse mesh leaves spikes): a running median of five,
        # then the mean of three, round the circle.
        med = [sorted(radii[(d + j) % n] for j in range(-2, 3))[2] for d in range(n)]
        radii = [(med[(d - 1) % n] + med[d] + med[(d + 1) % n]) / 3 for d in range(n)]
        tier_mean = sum(radii) / n
        # The wall top: where the band mean falls under 0.9 of the tier's.
        wall_top = z1
        for i in range(hi, i1):
            if means[i] and means[i] < 0.9 * tier_mean:
                wall_top = bands[i]["z0"]
                break
        out.append(dict(z0=z0, z1=z1, wall_top=wall_top, radii=radii))
    return out


def outline_of(p, tier):
    """A tier's outline: `DIRS_OUT` corners, the biggest radius of the
    directions each covers, counter-clockwise, metres from the model origin."""
    n = p["dirs"]
    per = n // DIRS_OUT
    c = p["centre_xy"]
    a0 = math.radians(p["angle0_deg"])
    pts = []
    for k in range(DIRS_OUT):
        r = max(tier["radii"][k * per + j] for j in range(per))
        a = a0 + (k * per + per / 2) / n * 2 * math.pi
        pts.append((round(c[0] + r * math.cos(a), 2), round(c[1] + r * math.sin(a), 2)))
    return pts


# ---- The ground floor: walls, doors, furniture ----------------------------------

def carry_to(outline, a, b):
    """The end `b` of the wall a-b carried straight on to the outline."""
    d = (b[0] - a[0], b[1] - a[1])
    l = math.hypot(*d)
    d = (d[0] / l, d[1] / l)
    far = (b[0] + d[0] * 30, b[1] + d[1] * 30)
    hit = LineString([a, far]).intersection(outline.exterior)
    if hit.is_empty:
        return b
    pts = [hit] if hit.geom_type == "Point" else [g for g in hit.geoms if g.geom_type == "Point"]
    # The first crossing past `b` (the end may start outside the outline).
    proj = lambda q: (q.x - a[0]) * d[0] + (q.y - a[1]) * d[1]
    beyond = [q for q in pts if proj(q) > l - 0.05]
    best = min(beyond, key=proj) if beyond else max(pts, key=proj)
    return (round(best.x - d[0] * 0.02, 3), round(best.y - d[1] * 0.02, 3))


def walls_of(level, outline):
    """The level's inner walls carried out to the outline where they end in
    the air, each with its doorway's place along it (None: solid)."""
    walls = level["walls"]
    gaps = [((g[0][0] + g[1][0]) / 2, (g[0][1] + g[1][1]) / 2) for g in level["door_gaps"]]
    def shared(e, own):
        # Does another wall end (or pass) within a hand of this end?
        for k, (a, b) in enumerate(walls):
            if k == own:
                continue
            if LineString([a, b]).distance(Point(e)) < 0.12:
                return True
        return False

    out = []
    for k, (a, b) in enumerate(walls):
        a, b = tuple(a), tuple(b)
        # An end no other wall meets is carried to the outer wall.
        aa = a if shared(a, k) else carry_to(outline, b, a)
        bb = b if shared(b, k) else carry_to(outline, a, b)
        l2 = (bb[0] - aa[0]) ** 2 + (bb[1] - aa[1]) ** 2
        frac = None
        for g in gaps:
            t = ((g[0] - aa[0]) * (bb[0] - aa[0]) + (g[1] - aa[1]) * (bb[1] - aa[1])) / l2
            q = (aa[0] + (bb[0] - aa[0]) * t, aa[1] + (bb[1] - aa[1]) * t)
            if 0.0 < t < 1.0 and math.dist(q, g) < 0.15:
                frac = round(t, 4)
        out.append((aa, bb, frac))
    return out


def foot(name, x, y, rot):
    w, d = SIZES[name]
    return affinity.rotate(box(x - w / 2, y - d / 2, x + w / 2, y + d / 2), rot, origin=(x, y), use_radians=True)


# What goes in a room, by its name: (piece, prefer) where prefer is 'wall',
# 'middle' or 'corner'. The main room gets beds when there's no bedroom.
KITS = {
    "hall": [("Hearth", "middle"), ("LongTable", "middle"), ("Bench", "beside"), ("Bench", "beside"), ("Shelf", "wall"), ("Cupboard", "wall")],
    "main": [("Hearth", "middle"), ("Table", "middle"), ("Bench", "beside"), ("Shelf", "wall"), ("Cupboard", "wall")],
    "kitchen": [("Hearth", "wall"), ("Table", "middle"), ("Shelf", "wall"), ("Barrel", "corner"), ("Barrel", "corner")],
    "store": [("Crate", "corner"), ("Crate", "corner"), ("Chest", "wall"), ("Barrel", "corner")],
    "bedroom": [("Bed", "wall"), ("Chest", "wall")],
    "back_room": [("Bed", "wall"), ("Bed", "wall"), ("Chest", "wall"), ("Shelf", "wall")],
    # Workrooms of the trades that work indoors (Laz, 15 Oct).
    "loomroom": [("Loom", "wall"), ("Loom", "wall"), ("Rack", "wall"), ("Shelf", "wall"), ("Crate", "corner")],
    "fletchery": [("Workbench", "wall"), ("Rack", "wall"), ("Shelf", "wall"), ("Crate", "corner")],
    "bindery": [("Workbench", "wall"), ("Table", "middle"), ("Shelf", "wall"), ("Shelf", "wall"), ("Chest", "corner")],
}
BEDS_IN_MAIN = 2


def kit_for(name, has_bedroom, area):
    base = name.rstrip("_0123456789")
    kit = list(KITS.get(base, KITS["main"]))
    if base in ("main", "hall"):
        beds = 0 if has_bedroom else BEDS_IN_MAIN
        if base == "hall":
            beds += 1 if area > 60 else 0
        # (Beds first: they're the biggest and go against the wall.)
        kit = [("Bed", "wall")] * beds + kit
        if area < 20:
            kit = [k for k in kit if k[0] not in ("Bench", "Cupboard")]
        if base == "hall" and area < 50:
            kit = [("Table", p) if n == "LongTable" else (n, p) for n, p in kit]
    if base == "bedroom" and area < 7:
        kit = [("Bedroll", "wall"), ("Chest", "wall")]
    if base == "store" and area < 5:
        kit = kit[:2]
    return kit


def place(level, outline, walls, stair, door_face, inner):
    """Furniture and containers on the floor, room by room: each piece on
    the free spot it likes best (against a wall, in a corner, in the middle,
    beside the last piece), clear of the way in, the doorways, the stair,
    the walls and each other."""
    rooms = level["rooms"]
    has_bedroom = any(r["name"].startswith("bedroom") or r["name"] == "back_room" for r in rooms)
    # What must stay clear.
    clear = [box(door_face[0] - 2.2, door_face[1] - 0.62, door_face[0] + 0.1, door_face[1] + 0.62)]
    for a, b, g in walls:
        if g is not None:
            p = (a[0] + (b[0] - a[0]) * g, a[1] + (b[1] - a[1]) * g)
            rot = math.atan2(b[1] - a[1], b[0] - a[0])
            clear.append(affinity.rotate(box(p[0] - 0.7, p[1] - 1.1, p[0] + 0.7, p[1] + 1.1), rot, origin=p, use_radians=True))
    if stair:
        clear.append(Point(stair["xy"]).buffer(stair["r"] + 0.5))
    wall_lines = []
    for a, b, g in walls:
        segs = [(a, b)]
        if g is not None:
            l = math.dist(a, b)
            half = min(DOORWAY * 0.5 / l, 0.45)
            lerp = lambda t: (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)
            segs = [(a, lerp(max(g - half, 0))), (lerp(min(g + half, 1)), b)]
        for p, q in segs:
            wall_lines.append(LineString([p, q]).buffer(INNER_WALL / 2, cap_style=2))
    taken = []
    out = []
    # A walker's room to move: the floor left when everything solid is kept
    # a body's width off. Every piece must leave the floor one piece,
    # reachable from the door, with somewhere to stand beside it.
    from shapely.ops import unary_union
    BODY = 0.3
    entry = Point(door_face[0] - 1.6, door_face[1])
    solid = unary_union(wall_lines)

    def still_open(f):
        free = inner.difference(unary_union([solid] + taken + [f]).buffer(BODY + 0.02))
        comps = list(free.geoms) if free.geom_type == "MultiPolygon" else [free]
        home = [c for c in comps if c.contains(entry)]
        if not home:
            return False
        # Every other piece and this one can be stood beside from the way in.
        return all(home[0].distance(t) < BODY + 0.15 for t in taken + [f])

    for room in rooms:
        poly = Polygon(room["poly"]).intersection(inner)
        if poly.is_empty:
            continue
        kit = kit_for(room["name"], has_bedroom, room["area_m2"])
        last = None
        for name, prefer in kit:
            w, d = SIZES[name]
            cands = []
            # Candidate spots on a grid over the room.
            minx, miny, maxx, maxy = poly.bounds
            xs = [minx + 0.25 * i for i in range(int((maxx - minx) / 0.25) + 1)]
            ys = [miny + 0.25 * j for j in range(int((maxy - miny) / 0.25) + 1)]
            edge = poly.exterior
            for x in xs:
                for y in ys:
                    c = Point(x, y)
                    if not poly.contains(c):
                        continue
                    # Turned to lie along the nearest edge of the room.
                    near = edge.interpolate(edge.project(c))
                    nd = c.distance(near)
                    e0 = edge.interpolate(max(edge.project(c) - 0.3, 0))
                    e1 = edge.interpolate(min(edge.project(c) + 0.3, edge.length))
                    along = math.atan2(e1.y - e0.y, e1.x - e0.x)
                    for rot in ([along, along + Q] if prefer != "middle" else [0.0, Q, along]):
                        f = foot(name, x, y, rot)
                        if not inner.contains(f) or not poly.buffer(0.05).contains(f):
                            continue
                        if any(f.intersects(k) for k in clear):
                            continue
                        gap = 0.08 if (name == "Bench" and last is not None and last[0] in ("Table", "LongTable")) else 0.75
                        if any(f.distance(t) < (0.08 if name == "Bench" and t is last[1] else gap) for t in taken):
                            continue
                        if any(f.distance(wl) < 0.5 for wl in wall_lines):
                            continue
                        # Its score: what it prefers.
                        if prefer == "wall":
                            s = nd
                        elif prefer == "corner":
                            s = nd + 0.3 * c.distance(Point(poly.centroid))
                            s -= 0.2 * sum(1 for q in [e0, e1] if abs(math.atan2(e1.y - e0.y, e1.x - e0.x) - along) > 0.3)
                        elif prefer == "beside" and last is not None:
                            s = f.distance(last[1]) + abs(rot - last[2]) * 0.5
                        else:
                            s = c.distance(poly.centroid)
                        if name == "Hearth" and prefer == "middle":
                            s = abs(nd - 1.4)
                        cands.append((s, x, y, rot, f))
            cands.sort(key=lambda c: c[0])
            best = next((c for c in cands[:60] if still_open(c[4])), None)
            if best is None:
                continue
            _, x, y, rot, f = best
            taken.append(f)
            last = (name, f, rot)
            out.append((name, round(x, 3), round(y, 3), round(rot, 4)))
    return out


# ---- The spec ----------------------------------------------------------------------

def spec_of(cat_dir, id, cat):
    """A catalogue model's spec, from its profile and floor plan."""
    p = json.load(open(os.path.join(cat_dir, id + "_profile.json")))
    d = json.load(open(os.path.join(cat_dir, id + "_interior.json")))
    entry = cat[id]
    kit = entry.get("kit", "home")
    want = {"home": 1, "vault": 1, "drum": 2}.get(kit, 1)
    if entry.get("params", {}).get("variant") == "inn":
        want = 3
    tiers = tiers_of(p, want)
    tier_specs = []
    for k, t in enumerate(tiers):
        pts = outline_of(p, t)
        tier_specs.append(dict(name="ABC"[k] if k < 3 else str(k), z0=round(t["z0"], 2), z1=round(t["z1"], 2), wall_top=round(t["wall_top"], 2), outline=pts))
    feats = entry.get("params", {}).get("feats", {})
    windows = [feats.get("winA", feats.get("win", 2)), feats.get("winA2", feats.get("winB", 2)), feats.get("winA3", 2)][: len(tiers)]
    spec = finish_spec(id, entry["label"], kit, tier_specs, d["levels"][0], d.get("stairs", []), windows, p["size"], roof="barrel" if kit == "vault" else "dome")
    spec["age"] = entry.get("age")
    spec["levels"] = d["levels"]
    spec["outside_door_sizes"] = [dict(level=f["level"], width=f["width"], height=f["height"], balcony=f.get("balcony")) for f in d.get("outside_door_fit", [])]
    return spec


def finish_spec(id, label, kit, tier_specs, L1, stairs, windows, size_xyz, roof="dome", yard=None):
    """The spec from the tiers and the ground floor's plan: the game's frame,
    the walls carried to the outline and cut clear of the way in, the door,
    and a furniture layout (or the one done by hand in the file already there)."""
    ground = Polygon(tier_specs[0]["outline"])
    if ground.area < 0 or not ground.exterior.is_ccw:
        tier_specs[0]["outline"] = tier_specs[0]["outline"][::-1]
        ground = Polygon(tier_specs[0]["outline"])
    walls = walls_of(L1, ground)
    inner = ground.buffer(-WALL_IN - 0.12, join_style=2)
    # The game's frame: the outline's bounding box middle; the main door is
    # the one facing +x.
    minx, miny, maxx, maxy = ground.bounds
    cx, cy = (minx + maxx) / 2, (miny + maxy) / 2
    hx, hy = (maxx - minx) / 2, (maxy - miny) / 2
    size = round(max(hx, hy) * 2, 2)
    half = (round(hx / size, 4), round(hy / size, 4))
    doors = sorted(L1["outside_doors"], key=lambda o: abs(o["yaw"]))
    main = doors[0]
    across = main["xy"][1] - cy
    door = round(across / hy / 0.8, 4)
    # Where the sim puts the door face: the outline's front at that y.
    hit = LineString([(cx, cy + across), (cx + 60, cy + across)]).intersection(ground.exterior)
    xs = [hit.x] if hit.geom_type == "Point" else [g.x for g in hit.geoms]
    face = (max(xs), cy + across)
    stair = stairs[0] if stairs else None
    # A wall that runs into the way in from the door (the sim keeps a strip
    # from the door to 2 m inside clear) is cut back out of it.
    way = box(face[0] - 2.4, face[1] - 0.75, face[0] + 0.5, face[1] + 0.75)
    trimmed = []
    for a, b, g in walls:
        line = LineString([a, b])
        if not line.intersects(way):
            trimmed.append((a, b, g))
            continue
        rest = line.difference(way)
        parts = [rest] if rest.geom_type == "LineString" else [x for x in rest.geoms if x.geom_type == "LineString"]
        parts = [x for x in parts if x.length > 0.6]
        if not parts:
            continue
        keep = max(parts, key=lambda x: x.length)
        (ax, ay), (bx, by) = keep.coords[0], keep.coords[-1]
        # The doorway's place along what's left, if it's still on it.
        if g is not None:
            gp = (a[0] + (b[0] - a[0]) * g, a[1] + (b[1] - a[1]) * g)
            t = keep.project(Point(gp)) / keep.length
            g = round(t, 4) if keep.distance(Point(gp)) < 0.05 and 0.1 < t < 0.9 else None
        trimmed.append(((round(ax, 3), round(ay, 3)), (round(bx, 3), round(by, 3)), g))
    # A doorway within a stride of a wall's end: the wall just stops short
    # there instead (an open end), so no sliver of wall is left beside it.
    ended = []
    for a, b, g in trimmed:
        if g is not None:
            l = math.dist(a, b)
            if g * l < 0.9 or (1.0 - g) * l < 0.9:
                lerp = lambda t: (round(a[0] + (b[0] - a[0]) * t, 3), round(a[1] + (b[1] - a[1]) * t, 3))
                hg = min(DOORWAY * 0.5 / l, 0.45)
                a, b, g = (lerp(min(g + hg, 1.0)), b, None) if g * l < 0.9 else (a, lerp(max(g - hg, 0.0)), None)
                if math.dist(a, b) < 0.6:
                    continue
        ended.append((a, b, g))
    walls = ended
    # A layout done by hand in the file already there is kept.
    old = os.path.join(ASSETS, id + ".json")
    hand = None
    if os.path.exists(old):
        og = json.load(open(old)).get("game", {})
        if og.get("hand"):
            hand = og
    furn = [] if hand else place(L1, ground, walls, stair, face, inner)
    top = tier_specs[-1]
    dome = round((top["z1"] - top["wall_top"]) / max(top["z1"] - top["z0"], 0.1), 3)
    game = dict(
        variant=id,
        centre=[round(cx, 3), round(cy, 3)],
        size=size,
        half=list(half),
        sizes=[round(size - 0.25, 2), round(size + 0.25, 2)],
        door=door,
        storeys=len(tier_specs),
        roof=roof,
        dome=dome,
        windows=windows,
        walls=[dict(a=list(a), b=list(b), gap=g) for a, b, g in walls],
        hand=bool(hand),
        furniture=hand["furniture"] if hand else [dict(what=n, at=[x, y], rot=r) for n, x, y, r in furn if n not in HOLDERS],
        holders=hand["holders"] if hand else [dict(what=n, at=[x, y], rot=r) for n, x, y, r in furn if n in HOLDERS],
        stair=dict(xy=stair["xy"], r=stair["r"]) if stair else None,
        other_doors=[dict(xy=o["xy"], yaw=o["yaw"]) for o in doors[1:]],
        yard=yard,
        note="The sim walks the ground tier only: tier A's outline, level 1's walls and doors. The other tiers, the roof, the stair and the other doors are drawn, not walked. Layout units are metres from `centre` divided by half × size.",
    )
    return dict(
        id=id,
        label=label,
        units="metres",
        frame="front (main door) faces +x; z up; origin = model origin; plan coordinates [x, y]",
        kit=kit,
        overall=dict(size_xyz=size_xyz, height=size_xyz[2]),
        game=game,
        exterior=dict(tiers=tier_specs),
        levels=[L1],
        stairs=stairs,
    )


def plan_png(spec, path):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    import matplotlib.patches
    g = spec["game"]
    fig, ax = plt.subplots(figsize=(6, 6))
    for t, col in zip(spec["exterior"]["tiers"], ["k", "0.4", "0.6"]):
        o = Polygon(t["outline"])
        ax.plot(*o.exterior.xy, color=col, lw=1.2 if col == "k" else 0.8)
    for w in g["walls"]:
        a, b = w["a"], w["b"]
        ax.plot([a[0], b[0]], [a[1], b[1]], color="0.3", lw=4)
        if w["gap"] is not None:
            p = (a[0] + (b[0] - a[0]) * w["gap"], a[1] + (b[1] - a[1]) * w["gap"])
            ax.plot(*p, "ws", ms=6)
    if g["stair"]:
        ax.add_patch(plt.Circle(g["stair"]["xy"], g["stair"]["r"], color="orange", alpha=0.5))
    for pc in g["furniture"] + g["holders"]:
        f = foot(pc["what"], pc["at"][0], pc["at"][1], pc["rot"])
        ax.fill(*f.exterior.xy, color="tab:blue" if pc["what"] in HOLDERS else "tab:brown", alpha=0.7)
        ax.text(pc["at"][0], pc["at"][1], pc["what"], fontsize=5, ha="center")
    if g.get("yard"):
        y = g["yard"]
        ax.plot(*Polygon(y["outline"]).exterior.xy, color="tab:green", lw=1.5)
        ax.plot(*y["gate"], "g^", ms=8)
        for pc in y["pieces"]:
            f = foot(pc["what"], pc["at"][0], pc["at"][1], pc["rot"])
            ax.fill(*f.exterior.xy, color="tab:olive", alpha=0.7)
            ax.text(pc["at"][0], pc["at"][1], pc["what"], fontsize=5, ha="center")
        for d in y["dressing"]:
            ax.plot(*d["at"], "k.", ms=3)
            ax.text(d["at"][0], d["at"][1], d["kind"], fontsize=4, ha="center", color="0.4")
        for sh in y["shades"]:
            ax.fill(*Polygon(sh["footprint"]).exterior.xy, color="tab:orange", alpha=0.15)
        if y["canopy"]:
            ax.plot(*Polygon(y["canopy"]["outline"]).exterior.xy, ls="--", color="0.3")
    cx, cy = g["centre"]
    ax.plot(cx + g["half"][0] * g["size"], cy + g["door"] * 0.8 * g["half"][1] * g["size"], "r>", ms=8)
    for o in g["other_doors"]:
        ax.plot(*o["xy"], "r^", ms=6)
    ax.set_aspect("equal")
    ax.set_title(f"{spec['id']}: {spec['label']}", fontsize=8)
    ax.grid(alpha=0.3)
    fig.savefig(path, dpi=100)
    plt.close(fig)


# ---- Rust ----------------------------------------------------------------------------

def f3(v):
    s = f"{v:.4f}".rstrip("0")
    return s + "0" if s.endswith(".") else s


def rot_s(r):
    if abs(r) < 1e-6:
        return "0.0"
    if abs(r - Q) < 1e-4:
        return "Q"
    return f3(r)


def unit_pts(spec, pts):
    g = spec["game"]
    cx, cy = g["centre"]
    hx, hy = g["half"][0] * g["size"], g["half"][1] * g["size"]
    return [((x - cx) / hx, (y - cy) / hy) for x, y in pts]


def const_block(name, pts, doc):
    lines = [f"/// {doc}", f"pub const {name}: &[(f32, f32)] = &["]
    for k in range(0, len(pts), 6):
        lines.append("    " + " ".join(f"({f3(x)}, {f3(y)})," for x, y in pts[k:k + 6]))
    lines.append("];")
    return "\n".join(lines)


def rust_layout(specs):
    """The variants and their outlines for `src/sim/layout.rs`."""
    variants = []
    consts = []
    for spec in specs:
        g = spec["game"]
        id = spec["id"]
        up = id.upper()
        u = lambda p: unit_pts(spec, [p])[0]
        outline = unit_pts(spec, spec["exterior"]["tiers"][0]["outline"])
        consts.append(const_block(f"{up}_OUTLINE", outline, f"`{id}`'s ground tier (`assets/buildings/{id}.json`), layout units, counter-clockwise."))
        walls = ", ".join(
            f"w(({f3(u(w['a'])[0])}, {f3(u(w['a'])[1])}), ({f3(u(w['b'])[0])}, {f3(u(w['b'])[1])}), {'None' if w['gap'] is None else 'Some(' + f3(w['gap']) + ')'})"
            for w in g["walls"])
        furn = ",\n".join(f"            f({p['what']}, {f3(u(p['at'])[0])}, {f3(u(p['at'])[1])}, {rot_s(p['rot'])})" for p in g["furniture"])
        hold = ",\n".join(f"            h({p['what']}, {f3(u(p['at'])[0])}, {f3(u(p['at'])[1])}, {rot_s(p['rot'])})" for p in g["holders"])
        yard = "None"
        if g.get("yard"):
            y = g["yard"]
            consts.append(const_block(f"{up}_YARD_OUTLINE", unit_pts(spec, y["outline"]), f"`{id}`'s work yard, round the house's +y side, layout units (past ±1)."))
            pieces = ", ".join(f"f({p['what']}, {f3(u(p['at'])[0])}, {f3(u(p['at'])[1])}, {rot_s(p['rot'])})" for p in y["pieces"])
            consts.append(f"""/// `{id}`'s work yard: fenced, gated, its stations in the open.
pub static {up}_YARD: Yard = Yard {{ outline: {up}_YARD_OUTLINE, gate: ({f3(u(y['gate'])[0])}, {f3(u(y['gate'])[1])}), gate_width: {f3(y['gate_width'])}, fence: {f3(y['fence_h'])}, pieces: &[{pieces}] }};""")
            yard = f"Some(&{up}_YARD)"
        use = {"workshop": "Use::Workshop", "hall": "Use::Hall", "shrine": "Use::Shrine"}.get(g.get("use"), "Use::Home")
        kind = "K::QotiroBlock" if g.get("kind") == "qotiro" else "K::RoduroHome"
        variants.append(f'''    Variant {{
        key: "{id}",
        name: "{spec['label'].split(',')[0].split(' / ')[0].split(' (')[0]}",
        kind: {kind},
        shape: Shape::Poly({up}_OUTLINE),
        use_: {use},
        half: ({f3(g['half'][0])}, {f3(g['half'][1])}),
        sizes: ({f3(g['sizes'][0])}, {f3(g['sizes'][1])}),
        weight: 0,
        door: {f3(g['door'])},
        storeys: {g['storeys']},
        walls: &[{walls}],
        furniture: &[
{furn}{',' if furn else ''}
        ],
        holders: &[
{hold}{',' if hold else ''}
        ],
        yard: {yard},
    }},''')
    return "\n".join(variants), "\n\n".join(consts)


def rust_forms(specs):
    """The forms for `src/view/interiors.rs`."""
    forms = []
    consts = []
    for spec in specs:
        g = spec["game"]
        id = spec["id"]
        up = id.upper()
        tiers = spec["exterior"]["tiers"]
        names = [f"layout::{up}_OUTLINE"]
        for k, t in enumerate(tiers[1:], 1):
            name = f"{up}_TIER_{k}"
            consts.append(const_block(name, unit_pts(spec, t["outline"]), f"`{id}`'s tier {t['name']} (drawing only)."))
            names.append(name)
        tier_s = ", ".join(f"({n}, {f3(t['z0'])}, {f3(t['z1'])}, {f3(t['wall_top'])})" for n, t in zip(names, tiers))
        u = lambda p: unit_pts(spec, [p])[0]
        stair = f"Some((({f3(u(g['stair']['xy'])[0])}, {f3(u(g['stair']['xy'])[1])}), {f3(g['stair']['r'])}))" if g["stair"] else "None"
        doors = ", ".join(f"(({f3(u(o['xy'])[0])}, {f3(u(o['xy'])[1])}), {f3(o['yaw'])})" for o in g["other_doors"])
        windows = ", ".join(str(n) for n in g["windows"])
        look = "None"
        if g.get("yard"):
            y = g["yard"]
            dress = ", ".join(f"({'Dress::' + d['kind'].capitalize()}, ({f3(u(d['at'])[0])}, {f3(u(d['at'])[1])}), {f3(d['rot'])}, ({f3(d['size'][0])}, {f3(d['size'][1])}, {f3(d['size'][2])}))" for d in y["dressing"])
            shades = ", ".join("(&[" + ", ".join(f"({f3(u(p)[0])}, {f3(u(p)[1])})" for p in sh["footprint"]) + f"], {f3(sh['height'])}, {sh['cloth'].upper()})" for sh in y["shades"])
            cols = ", ".join(f"(({f3(u(c['at'])[0])}, {f3(u(c['at'])[1])}), {f3(c['height'])})" for c in y["columns"])
            canopy = "None"
            if y["canopy"]:
                c = y["canopy"]
                canopy = "Some((&[" + ", ".join(f"({f3(u(p)[0])}, {f3(u(p)[1])})" for p in c["outline"]) + f"], {f3(c['height'])}))"
            chimney = "None"
            if y["chimney"]:
                c = y["chimney"]
                chimney = f"Some((({f3(u(c['at'])[0])}, {f3(u(c['at'])[1])}), {f3(c['r'])}, {f3(c['height'])}))"
            look = f"Some(YardLook {{ dressing: &[{dress}], shades: &[{shades}], columns: &[{cols}], canopy: {canopy}, chimney: {chimney} }})"
        qotiro = "None"
        if g.get("qotiro"):
            q = g["qotiro"]
            props = []
            for p in q["props"] + q["stairs"]:
                props.append(f"(QProp::{p['kind'].capitalize()}, ({f3(u(p['at'])[0])}, {f3(u(p['at'])[1])}), {f3(p['rot'])}, ({f3(p['size'][0])}, {f3(p['size'][1])}, {f3(p['size'][2])}), {f3(p['z'])})")
            ups = ", ".join(f"(({f3(u(o['xy'])[0])}, {f3(u(o['xy'])[1])}), {f3(o['yaw'])}, {f3(o['z'])})" for o in q["doors_up"])
            heads = ", ".join(f"({f3(u(st['head'])[0])}, {f3(u(st['head'])[1])})" for st in q["stairs"])
            qotiro = f"Some(QotiroLook {{ props: &[{', '.join(props)}], doors_up: &[{ups}], landings: &[{heads}] }})"
        roof = {"barrel": "Barrel", "flat": "Flat"}.get(g["roof"], "Dome")
        forms.append(f'''    Form {{
        key: "{id}",
        at_size: {f3(g['size'])},
        tiers: &[{tier_s}],
        roof: Roof::{roof},
        dome: {f3(g['dome'])},
        stair: {stair},
        doors: &[{doors}],
        windows: &[{windows}],
        yard: {look},
        qotiro: {qotiro},
    }},''')
    return "\n".join(forms), "\n\n".join(consts)


def splice(path, start, end, body):
    s = open(path).read()
    i, j = s.index(start), s.index(end)
    s = s[:i + len(start)] + "\n" + body + "\n" + s[j:]
    open(path, "w").write(s)


def design_specs(cat_dir):
    """The designed buildings (`roduro_designs.py`): the six that fill the
    catalogue's gaps, and the trade homes with a yard."""
    import roduro_designs as rd
    out = {}
    for id, d in rd.designs().items():
        tiers = d["tiers"]
        xs = [p[0] for p in tiers[0]["outline"]]
        ys = [p[1] for p in tiers[0]["outline"]]
        size_xyz = [round(max(xs) - min(xs), 2), round(max(ys) - min(ys), 2), tiers[-1]["z1"]]
        spec = finish_spec(id, d["label"], d["kit"], tiers, d["level"], d["stairs"], d["windows"], size_xyz, roof=d["roof"])
        spec["designed"] = True
        out[id] = spec
    for id, d in rd.indoor_designs().items():
        tiers = d["tiers"]
        xs = [p[0] for p in tiers[0]["outline"]]
        ys = [p[1] for p in tiers[0]["outline"]]
        size_xyz = [round(max(xs) - min(xs), 2), round(max(ys) - min(ys), 2), tiers[-1]["z1"]]
        spec = finish_spec(id, d["label"], d["kit"], tiers, d["level"], d["stairs"], d["windows"], size_xyz, roof=d["roof"])
        spec["designed"] = True
        spec["game"]["use"] = "workshop"
        out[id] = spec
    for id, d in rd.yard_designs().items():
        tiers = d["tiers"]
        xs = [p[0] for p in tiers[0]["outline"]]
        ys = [p[1] for p in tiers[0]["outline"]]
        size_xyz = [round(max(xs) - min(xs), 2), round(max(ys) - min(ys), 2), tiers[-1]["z1"]]
        spec = finish_spec(id, d["label"], d["kit"], tiers, d["level"], d["stairs"], d["windows"], size_xyz, roof=d["roof"])
        spec["designed"] = True
        spec["game"]["use"] = "workshop"
        spec["game"]["yard"] = place_yard(d, Polygon(spec["exterior"]["tiers"][0]["outline"]))
        out[id] = spec
    # The Qotiro buildings (`qotiro_designs.py`): quarried boxes, flat roofs,
    # outside stairs and terrace props; two with a walled yard.
    import qotiro_designs as qd
    for id, d in qd.designs().items():
        tiers = d["tiers"]
        xs = [p[0] for p in tiers[0]["outline"]]
        ys = [p[1] for p in tiers[0]["outline"]]
        size_xyz = [round(max(xs) - min(xs), 2), round(max(ys) - min(ys), 2), tiers[-1]["z1"]]
        spec = finish_spec(id, d["label"], d["kit"], tiers, d["level"], [], d["windows"], size_xyz, roof="flat")
        spec["designed"] = True
        spec["game"]["kind"] = "qotiro"
        spec["game"]["qotiro"] = d["qotiro"]
        if d.get("use"):
            spec["game"]["use"] = d["use"]
        out[id] = spec
    for id, d in qd.yard_designs().items():
        tiers = d["tiers"]
        xs = [p[0] for p in tiers[0]["outline"]]
        ys = [p[1] for p in tiers[0]["outline"]]
        size_xyz = [round(max(xs) - min(xs), 2), round(max(ys) - min(ys), 2), tiers[-1]["z1"]]
        spec = finish_spec(id, d["label"], d["kit"], tiers, d["level"], [], d["windows"], size_xyz, roof="flat")
        spec["designed"] = True
        spec["game"]["kind"] = "qotiro"
        spec["game"]["qotiro"] = d["qotiro"]
        spec["game"]["use"] = "workshop"
        spec["game"]["yard"] = place_yard(d, Polygon(spec["exterior"]["tiers"][0]["outline"]), qd.TRADE_YARDS)
        out[id] = spec
    return out


def place_yard(d, house, trades=None):
    """A designed trade home's yard: the fence ring, the gate, the roofed
    shed on its posts, the chimney, and the trade's pieces and dressing
    placed on the open ground (stations under the shed first)."""
    import roduro_designs as rd
    from shapely.ops import unary_union
    trade = (trades or rd.TRADE_YARDS)[d["trade"]]
    # (Opened by a third of a metre: that drops any sliver of ground the
    # yard's outline shares with the house's own wall.)
    yard = d["yard"].buffer(-0.35, join_style=1).buffer(0.35, join_style=1).simplify(0.08)
    if yard.geom_type != "Polygon":
        yard = max(yard.geoms, key=lambda p: p.area)
    ring = yard.exterior if yard.exterior.is_ccw else LineString(list(yard.exterior.coords)[::-1])
    pts = [(round(x, 3), round(y, 3)) for x, y in ring.coords[:-1]]
    shed = d["shed"]
    gate = Point(d["gate"])
    gate = ring.interpolate(ring.project(gate))
    gate = (round(gate.x, 3), round(gate.y, 3))
    # The ground someone can stand on: the yard in from its wall, and the
    # fence's gate kept clear.
    floor = yard.buffer(-0.7, join_style=2).difference(house.buffer(0.4))
    clear = [Point(gate).buffer(1.6)]
    # The house's door may open onto the yard: the way in from it stays clear.
    for o in d["level"]["outside_doors"]:
        x, y = o["xy"]
        clear.append(box(x - 0.5, y - 0.8, x + 2.6, y + 0.8))
    taken = []
    pieces, dressing = [], []
    BODY = 0.3

    def still_open(f):
        free = floor.difference(unary_union(taken + [f]).buffer(BODY + 0.02))
        comps = list(free.geoms) if free.geom_type == "MultiPolygon" else [free]
        home = [c for c in comps if c.distance(Point(gate)) < 2.0]
        return bool(home) and all(home[0].distance(t) < BODY + 0.15 for t in taken + [f])

    def put(name, prefer, size, area, dress):
        w, dd = size[:2]
        minx, miny, maxx, maxy = area.bounds
        edge = yard.exterior
        cands = []
        for i in range(int((maxx - minx) / 0.3) + 1):
            for j in range(int((maxy - miny) / 0.3) + 1):
                x, y = minx + 0.3 * i, miny + 0.3 * j
                c = Point(x, y)
                if not area.contains(c):
                    continue
                near = edge.interpolate(edge.project(c))
                e0 = edge.interpolate(max(edge.project(c) - 0.3, 0))
                e1 = edge.interpolate(min(edge.project(c) + 0.3, edge.length))
                along = math.atan2(e1.y - e0.y, e1.x - e0.x)
                for rot in [along, along + Q, 0.0]:
                    f = affinity.rotate(box(x - w / 2, y - dd / 2, x + w / 2, y + dd / 2), rot, origin=(x, y), use_radians=True)
                    if not area.contains(f) or any(f.intersects(k) for k in clear) or any(f.distance(t) < 0.55 for t in taken):
                        continue
                    sc = c.distance(near) if prefer == "wall" else c.distance(area.centroid)
                    cands.append((sc, x, y, rot, f))
        cands.sort(key=lambda c: c[0])
        best = next((c for c in cands[:40] if still_open(c[4])), None)
        if best is None:
            return
        _, x, y, rot, f = best
        taken.append(f)
        if dress:
            dressing.append(dict(kind=name, at=[round(x, 3), round(y, 3)], rot=round(rot, 4), size=list(size)))
        else:
            pieces.append(dict(what=name, at=[round(x, 3), round(y, 3)], rot=round(rot, 4)))

    under = shed.buffer(-0.5).intersection(floor)
    open_ground = floor.difference(shed.buffer(0.3))
    for name, prefer in trade["shed"]:
        put(name, prefer, SIZES[name], under, False)
    for name, prefer in trade["open"]:
        put(name, prefer, SIZES[name], open_ground, False)
    for kind, size in trade["dress_shed"]:
        put(kind, "wall", size, under, True)
    for kind, size in trade["dress_open"]:
        put(kind, "wall" if kind != "tub" else "middle", size, open_ground, True)
    # Posts round the shed where it isn't against the house.
    posts = []
    sr = shed.exterior
    n = max(4, int(sr.length / 2.6))
    for k in range(n):
        p = sr.interpolate(k / n * sr.length)
        if house.buffer(0.3).contains(p):
            continue
        posts.append(dict(at=[round(p.x, 3), round(p.y, 3)], height=2.7))
    canopy = [(round(x, 3), round(y, 3)) for x, y in shed.simplify(0.1).exterior.coords[:-1]]
    return dict(
        outline=pts,
        gate=list(gate),
        gate_width=2.2,
        fence_h=d.get("base", 1.3),
        pieces=pieces,
        dressing=dressing,
        shades=[],
        canopy=dict(outline=canopy, height=2.7),
        chimney=dict(at=list(d["chimney"]), r=0.7, height=6.0) if d.get("chimney") else None,
        columns=posts,
    )


def main():
    if len(sys.argv) < 2 or sys.argv[1] not in ("spec", "rust", "design"):
        print(__doc__)
        sys.exit(1)
    if sys.argv[1] == "design":
        sys.path.insert(0, HERE)
        os.makedirs(os.path.join(ASSETS, "_plans"), exist_ok=True)
        for id, spec in design_specs(sys.argv[2]).items():
            json.dump(spec, open(os.path.join(ASSETS, id + ".json"), "w"), indent=1)
            plan_png(spec, os.path.join(ASSETS, "_plans", id + ".png"))
            g = spec["game"]
            print(f"{id}: {len(spec['exterior']['tiers'])} tiers, size {g['size']}, {len(g['walls'])} walls, {len(g['furniture'])} pieces, {len(g['holders'])} containers" + (f", yard with {len(g['yard']['pieces'])} pieces" if g.get("yard") else ""))
        return
    if sys.argv[1] == "spec":
        cat_dir = sys.argv[2]
        cat = json.load(open(os.path.join(cat_dir, "catalogue.json")))
        ids = sys.argv[3:] or sorted(k for k in cat if cat[k].get("catalogue") == "v2" and os.path.exists(os.path.join(cat_dir, k + "_profile.json")))
        os.makedirs(os.path.join(ASSETS, "_plans"), exist_ok=True)
        for id in ids:
            spec = spec_of(cat_dir, id, cat)
            json.dump(spec, open(os.path.join(ASSETS, id + ".json"), "w"), indent=1)
            plan_png(spec, os.path.join(ASSETS, "_plans", id + ".png"))
            g = spec["game"]
            print(f"{id}: {len(spec['exterior']['tiers'])} tiers, size {g['size']}, {len(g['walls'])} walls, {len(g['furniture'])} pieces, {len(g['holders'])} containers")
    else:
        specs = [json.load(open(f)) for f in sorted(glob.glob(os.path.join(ASSETS, "*.json")))]
        v, c = rust_layout(specs)
        splice(os.path.join(ROOT, "src/sim/layout.rs"), "    // ---- generated: catalogue buildings (tools/buildings/roduro_forms.py rust) ----", "    // ---- end generated ----", v)
        splice(os.path.join(ROOT, "src/sim/layout.rs"), "// ---- generated outlines (tools/buildings/roduro_forms.py rust) ----", "// ---- end generated outlines ----", c)
        f, c = rust_forms(specs)
        splice(os.path.join(ROOT, "src/view/interiors.rs"), "const FORMS: &[Form] = &[", "];\n// ---- end generated forms ----", f)
        splice(os.path.join(ROOT, "src/view/interiors.rs"), "// ---- generated tiers (tools/buildings/roduro_forms.py rust) ----", "// ---- end generated tiers ----", c)
        print(f"{len(specs)} buildings written into layout.rs and interiors.rs")


if __name__ == "__main__":
    main()
