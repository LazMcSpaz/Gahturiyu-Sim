"""The Horaro buildings (Laz, 17 Oct: "Horaro next"), designed from the
five paintings cut into the kit (`claude/horaro-kit.md`: the rock home,
the market hall, the stilt home, the egg home, the quay market), five of
the kit's generated houses, and the six work-deck paintings in
`E:\\Gahturiyu 3d Assets\\Concept art\\Buildings\\horaro_*.jpg`: woven lobes
lashed together with rope, on rough rock pillars, stacked stone drums,
brick piers or grown root bases, under a plank deck with a rope rail,
reached by a ladder, a ramp, steps to a quay or a low dock.

Metres, the building's own frame: x along its facing (the way on, +x), y
across, z up from the water. The deck's floor is `FLOOR` over the water
(where the game stands Horaro at home), its slab `SLAB` thick under that;
the supports reach from under the water to the slab. Each building: the
deck outline (the one tier the sim knows), supports, hulls (kind, centre,
turn, length x width x height), the seams between lobes, doors and
portholes on the hulls, the ways on and off (each on the deck's edge),
and the gear on the deck. `check()` makes sure everything stands where
it should and that a person can walk from each way on to every door and
every piece of work; `python3 horaro_designs.py walk` draws the plans.

These are look-only, like the stilt homes before them: the sea is
off-limits to the squad, so nobody walks a deck yet.
"""
import math

from shapely.geometry import Polygon, Point, LineString, box
from shapely import affinity
from shapely.ops import unary_union

FLOOR = 2.7
SLAB = 0.3
Q = math.pi / 2
BODY = 0.3


def ellipse(a, b, n=36):
    """An ellipse `a` by `b` (full lengths), counter-clockwise from +x."""
    return Polygon([(a / 2 * math.cos(2 * math.pi * i / n), b / 2 * math.sin(2 * math.pi * i / n)) for i in range(n)])


def rrect(w, d, r=1.0):
    """A rounded rectangle `w` by `d`, corners of radius `r`, from +x."""
    p = box(-w / 2 + r, -d / 2 + r, w / 2 - r, d / 2 - r).buffer(r, join_style=1, resolution=4)
    pts = list(p.exterior.coords[:-1])
    # Start the ring at the point nearest +x on the y = 0 line.
    k = min(range(len(pts)), key=lambda i: abs(pts[i][1]) + (0 if pts[i][0] > 0 else 100))
    return Polygon(pts[k:] + pts[:k])


def outline(poly):
    pts = [(round(x, 3), round(y, 3)) for x, y in poly.exterior.coords[:-1]]
    if not Polygon(pts).exterior.is_ccw:
        pts = pts[::-1]
    return pts


def support(kind, at, r):
    return dict(kind=kind, at=[round(at[0], 3), round(at[1], 3)], r=r)


def ring_supports(kind, poly, r, spacing, inset=0.9):
    """Supports along the deck's edge, `inset` in, every `spacing` metres."""
    edge = poly.buffer(-inset, join_style=1).exterior
    n = max(3, int(edge.length / spacing))
    out = []
    for i in range(n):
        p = edge.interpolate(i / n, normalized=True)
        out.append(support(kind, (p.x, p.y), r))
    return out


def grid_supports(kind, poly, r, step, inset=0.8):
    inner = poly.buffer(-inset, join_style=2)
    minx, miny, maxx, maxy = inner.bounds
    out = []
    nx = max(2, int(round((maxx - minx) / step)) + 1)
    ny = max(2, int(round((maxy - miny) / step)) + 1)
    for i in range(nx):
        for j in range(ny):
            x = minx + (maxx - minx) * i / (nx - 1)
            y = miny + (maxy - miny) * j / (ny - 1)
            if inner.buffer(0.05).contains(Point(x, y)):
                out.append(support(kind, (x, y), r))
    return out


def hull(kind, at, yaw, size):
    return dict(kind=kind, at=[round(at[0], 3), round(at[1], 3)], yaw=round(yaw, 4), size=[round(s, 3) for s in size])


PLAN_EXP = {"egg": 2.0, "dome": 2.0, "hull": 2.8, "longhouse": 4.0, "lid": 2.0, "arcade": 4.0}
FLOOR_RHO = {"egg": 1.0, "dome": 1.0, "hull": 1.0, "longhouse": 1.0, "lid": 0.74, "arcade": 1.0}


def hull_plan(h, n=48, rho=None):
    """A hull's footprint at the floor (a superellipse)."""
    a, b = h["size"][0] / 2, h["size"][1] / 2
    e = PLAN_EXP[h["kind"]]
    rho = FLOOR_RHO[h["kind"]] if rho is None else rho
    pts = []
    for i in range(n):
        phi = 2 * math.pi * i / n
        c, s = math.cos(phi), math.sin(phi)
        r = (abs(c) ** e / a ** e + abs(s) ** e / b ** e) ** (-1 / e) * rho
        pts.append((c * r, s * r))
    p = Polygon(pts)
    p = affinity.rotate(p, h["yaw"], origin=(0, 0), use_radians=True)
    return affinity.translate(p, h["at"][0], h["at"][1])


def rho_at(kind, s):
    """A hull's plan radius factor at height fraction `s` (as the window
    draws it: `Lobe::profile`)."""
    s = min(max(s, 0.0), 1.0)
    c, e = {"egg": (0.12, 2.0), "dome": (0.35, 2.5), "hull": (0.45, 3.0), "longhouse": (0.0, 2.0), "arcade": (0.0, 2.0), "lid": (0.0, 1.6)}[kind]
    if s < c:
        return 1.0
    return max(0.0, 1.0 - ((s - c) / (1.0 - c)) ** e) ** (1.0 / e)


def on_hull(h, phi, out=0.0, z=0.0):
    """The spot on a hull's wall at angle `phi` (its own frame) and height
    `z`, and the facing there."""
    a, b = h["size"][0] / 2, h["size"][1] / 2
    e = PLAN_EXP[h["kind"]]
    c, s = math.cos(phi), math.sin(phi)
    rho = rho_at(h["kind"], z / h["size"][2])
    r = (abs(c) ** e / a ** e + abs(s) ** e / b ** e) ** (-1 / e) * rho + out
    lx, ly = c * r, s * r
    yaw = h["yaw"]
    x = h["at"][0] + lx * math.cos(yaw) - ly * math.sin(yaw)
    y = h["at"][1] + lx * math.sin(yaw) + ly * math.cos(yaw)
    # The outward facing: the ellipse's normal.
    nx, ny = c / a ** 2, s / b ** 2
    facing = math.atan2(ny, nx) + yaw
    return (round(x, 3), round(y, 3)), round(facing, 4)


def door(h, phi):
    at, facing = on_hull(h, phi)
    return dict(xy=list(at), yaw=facing)


def porthole(h, phi, z=1.6):
    at, facing = on_hull(h, phi, z=z)
    return dict(xy=list(at), yaw=facing, z=z)


def edge_point(poly, yaw):
    """Where a line from the middle out the way `yaw` meets the deck's edge."""
    far = (40 * math.cos(yaw), 40 * math.sin(yaw))
    hit = LineString([(0, 0), far]).intersection(poly.exterior)
    if hit.geom_type == "Point":
        p = hit
    else:
        p = max(hit.geoms, key=lambda g: g.x ** 2 + g.y ** 2)
    return (round(p.x, 3), round(p.y, 3))


ACCESS_GAP = {"ladder": 2.0, "ramp": 2.4, "stair": 2.4, "dock": 3.0, "walkway": 2.2}


def access(kind, poly, yaw, length=4.0):
    return dict(kind=kind, at=list(edge_point(poly, yaw)), yaw=round(yaw, 4), len=length)


def prop(kind, at, rot=0.0, size=(1.0, 1.0, 1.0), z=0.0):
    return dict(kind=kind, at=[round(at[0], 3), round(at[1], 3)], rot=round(rot, 4), size=[round(s, 3) for s in size], z=z)


# The footprint each prop takes on the deck (w, d of its size, or none).
FOOTPRINT = {"stall": 1.0, "frame": 0.25, "table": 1.0, "oven": 1.0, "hearth": 1.0, "jars": 0.6, "pan": 1.0, "sweep": 0.3, "sacks": 1.0,
             "basket": 1.0, "coils": 1.0, "wheel": 1.0, "loom": 0.5, "dyepots": 1.0, "yarnline": 0.2, "treepost": 0.4, "bench": 1.0, "boat": 0.0,
             "awning": 0.0, "lamp": 0.0, "mooring": 0.3, "chimney": 0.0, "roof": 0.0}
# Gear a worker must be able to get to.
WORKED = {"stall", "frame", "table", "oven", "hearth", "pan", "sweep", "wheel", "loom", "dyepots", "coils", "basket", "sacks", "jars"}


def rail_runs(poly, accesses):
    """Rope rail all the way round but for a gap at each way on or off, as
    fractions of the way round from the outline's first corner."""
    ring = LineString(list(poly.exterior.coords))
    L = ring.length
    gaps = []
    for a in accesses:
        t = ring.project(Point(a["at"])) / L
        half = ACCESS_GAP[a["kind"]] / 2 / L
        gaps.append((t - half, t + half))
    gaps.sort()
    runs = []
    t = 0.0
    for g0, g1 in gaps:
        if g0 > t:
            runs.append((round(t, 4), round(g0, 4)))
        t = max(t, g1)
    if t < 1.0:
        runs.append((round(t, 4), 1.0))
    # A gap that wraps past the start cuts the first run too.
    for g0, g1 in gaps:
        if g1 > 1.0 and runs and runs[0][0] < g1 - 1.0:
            runs[0] = (round(g1 - 1.0, 4), runs[0][1])
    return [r for r in runs if r[1] - r[0] > 0.01]


def building(key, label, deck, supports, hulls, seams=(), doors=(), portholes=(), accesses=(), props=(), quay=None, rail=True, use="home"):
    d = dict(label=label, deck=deck, use=use,
             horaro=dict(deck=FLOOR, supports=supports, hulls=hulls, seams=list(seams), doors=list(doors), portholes=list(portholes),
                         access=list(accesses), props=list(props), quay=quay, rail=rail_runs(deck, accesses) if rail else []))
    return key, d


def designs():
    out = {}

    # Lobes are about half as tall as they are wide (Laz, 17 Oct: domes, not
    # eggs, their widest line on the deck).
    # 1. The rock home (as painted): one great woven dome on six lashed rock
    #    pillars, its skirt hiding the small deck; a ladder up the front.
    deck = ellipse(13.2, 10.4)
    h = [hull("dome", (-0.6, 0), 0.0, (9.4, 7.8, 4.4))]
    acc = [access("ladder", deck, 0.0)]
    k, d = building("horaro_rock_home_01", "Rock home, a dome on lashed rock pillars", deck,
                    ring_supports("rock", deck, 0.95, 5.2, inset=1.3), h, doors=[door(h[0], 0.0)],
                    portholes=[porthole(h[0], 0.7), porthole(h[0], -0.7)], accesses=acc, rail=False)
    out[k] = d

    # 2. The market hall (as painted): twin domes and a stall on a wide
    #    deck over a grown root base, a ramp down to a landing.
    deck = ellipse(21.0, 15.0)
    h = [hull("dome", (-3.5, 2.8), 0.0, (8.6, 7.4, 4.1)), hull("dome", (-3.0, -3.4), 0.0, (7.0, 6.2, 3.5))]
    acc = [access("ramp", deck, 0.25, 6.0)]
    props = [prop("stall", (5.5, 2.0), math.pi, (4.0, 3.2, 2.7)), prop("jars", (6.5, -2.5), 0.0, (1.0, 1.0, 1.1)),
             prop("basket", (3.5, -4.5), 0.0, (0.9, 0.9, 0.8)), prop("lamp", (5.5, 3.8), 0.0, (0.3, 0.3, 2.5)),
             prop("coils", (1.5, 6.0), 0.3, (1.8, 0.6, 0.3))]
    k, d = building("horaro_market_hall_01", "Market hall, twin domes and a stall on a root base", deck,
                    [support("root", (-5.0, 3.0), 1.4), support("root", (-4.0, -4.5), 1.3), support("root", (5.0, -0.5), 1.5), support("root", (0.5, 5.0), 1.1)],
                    h, seams=[(0, 1)], doors=[door(h[0], 0.1), door(h[1], -0.2)], portholes=[porthole(h[0], 0.9), porthole(h[1], 0.8)],
                    accesses=acc, props=props, use="hall")
    out[k] = d

    # 3. The stilt home (as painted): a boat-hull house on six lashed stone
    #    drums, a ladder at the bow.
    deck = ellipse(16.6, 10.4)
    h = [hull("hull", (-0.8, 0), 0.0, (13.4, 8.2, 4.1))]
    acc = [access("ladder", deck, 0.0)]
    k, d = building("horaro_stilt_home_01", "Stilt home, a woven hull on stone drums", deck,
                    ring_supports("drum", deck, 0.62, 6.0, inset=1.4), h, doors=[door(h[0], 0.0)],
                    portholes=[porthole(h[0], 0.55), porthole(h[0], -0.55), porthole(h[0], 2.3)], accesses=acc, rail=False)
    out[k] = d

    # 4. The egg home (as painted): an egg on a round deck with a rope rail,
    #    many drum piles under, a ladder.
    deck = ellipse(17.0, 16.0)
    h = [hull("egg", (-0.5, 0), 0.0, (11.0, 9.2, 4.6))]
    acc = [access("ladder", deck, 0.0)]
    props = [prop("basket", (5.5, 4.0), 0.0, (0.9, 0.9, 0.8)), prop("jars", (5.0, -4.5), 0.0, (1.0, 1.0, 1.0)), prop("coils", (-6.5, 3.0), 0.5, (1.8, 0.6, 0.3))]
    k, d = building("horaro_egg_home_01", "Egg home on a round deck", deck,
                    ring_supports("drum", deck, 0.5, 3.4, inset=1.1) + [support("drum", (0, 0), 0.6), support("drum", (-4, 2.5), 0.5), support("drum", (3.5, -2.5), 0.5), support("drum", (-3.5, -3), 0.5), support("drum", (3, 3), 0.5)],
                    h, doors=[door(h[0], 0.0)], portholes=[porthole(h[0], 0.6), porthole(h[0], -0.6)], accesses=acc, props=props)
    out[k] = d

    # 5. The quay market (as painted): a longhouse and a stall on a deck
    #    over brick piers, steps down to a stone quay with mooring posts.
    deck = rrect(20.0, 13.0, 1.2)
    h = [hull("longhouse", (-3.5, 0), 0.0, (13.0, 7.0, 4.3))]
    acc = [access("stair", deck, 0.0, 3.2)]
    quay = [(10.0, -7.5), (15.0, -7.0), (15.5, 0.0), (15.0, 7.0), (10.0, 7.5)]
    qz = 1.3
    props = [prop("stall", (5.5, 2.5), math.pi, (4.0, 3.4, 2.7)), prop("lamp", (5.5, 4.2), 0.0, (0.3, 0.3, 2.5)), prop("jars", (6.0, -3.0), 0.0, (1.0, 1.0, 1.1)),
             prop("basket", (3.5, -5.0), 0.0, (0.9, 0.9, 0.8)), prop("sacks", (-8.5, 4.5), 0.0, (1.4, 0.6, 0.6)),
             prop("mooring", (14.6, -5.0), 0.0, (0.3, 0.3, 1.1), qz - FLOOR), prop("mooring", (15.1, 0.0), 0.0, (0.3, 0.3, 1.1), qz - FLOOR),
             prop("mooring", (14.6, 5.0), 0.0, (0.3, 0.3, 1.1), qz - FLOOR), prop("boat", (13.0, -9.5), 0.3, (3.2, 1.2, 0.5), 0.1 - FLOOR)]
    k, d = building("horaro_quay_market_01", "Quay market, a longhouse over brick piers", deck,
                    grid_supports("pier", deck, 0.42, 3.2, inset=0.9), h, doors=[door(h[0], 0.0)], portholes=[],
                    accesses=acc, props=props, quay=dict(outline=quay, z=qz), use="hall")
    out[k] = d

    # 6. Egg twin (kit): a big egg and a small one lashed together.
    deck = ellipse(16.0, 14.0)
    h = [hull("egg", (-2.0, 1.5), 0.0, (9.4, 7.8, 3.9)), hull("egg", (2.6, -2.2), 0.4, (6.8, 5.8, 2.9))]
    acc = [access("ladder", deck, -0.3)]
    props = [prop("basket", (5.5, 3.5), 0.0, (0.9, 0.9, 0.8)), prop("jars", (-5.5, -3.5), 0.0, (1.0, 1.0, 1.0))]
    k, d = building("horaro_egg_twin_01", "Egg twin, two lobes lashed together", deck,
                    ring_supports("drum", deck, 0.5, 3.6, inset=1.1) + [support("drum", (-2, 1.5), 0.6), support("drum", (3, -2.5), 0.5)],
                    h, seams=[(0, 1)], doors=[door(h[0], 0.3)], portholes=[porthole(h[0], -0.6), porthole(h[1], 0.3)], accesses=acc, props=props)
    out[k] = d

    # 7. Hull bent (kit): two hull lobes at an angle, on drums.
    deck = ellipse(18.0, 13.0)
    h = [hull("hull", (-2.5, 0.5), 0.0, (11.0, 7.0, 3.5)), hull("hull", (4.0, -1.5), -0.55, (7.6, 5.8, 2.9))]
    acc = [access("ladder", deck, 0.4)]
    props = [prop("coils", (5.0, 4.0), 0.2, (1.8, 0.6, 0.3)), prop("basket", (7.0, 2.2), 0.0, (0.9, 0.9, 0.8))]
    k, d = building("horaro_hull_bent_01", "Hull house bent round, two lobes", deck,
                    ring_supports("drum", deck, 0.58, 4.4, inset=1.2) + [support("drum", (-2, 0.5), 0.6), support("drum", (4, -1.5), 0.55)],
                    h, seams=[(0, 1)], doors=[door(h[1], 0.9)], portholes=[porthole(h[0], 1.2), porthole(h[0], -1.3), porthole(h[1], -1.0)], accesses=acc, props=props)
    out[k] = d

    # 8. Rock-hull T (kit): a dome with a smaller dome across its end, on
    #    rock pillars.
    deck = ellipse(16.0, 12.0)
    h = [hull("dome", (-2.5, 0), 0.0, (8.8, 7.0, 3.9)), hull("dome", (3.0, -0.8), Q, (6.4, 5.0, 2.8))]
    acc = [access("ladder", deck, 0.5)]
    props = [prop("jars", (-1.0, 4.8), 0.0, (1.0, 1.0, 1.0))]
    k, d = building("horaro_rockhull_t_01", "Rock-hull T, a dome across a dome's end", deck,
                    ring_supports("rock", deck, 0.85, 5.6, inset=1.3) + [support("rock", (-1, 0), 0.9)],
                    h, seams=[(0, 1)], doors=[door(h[1], 0.0)], portholes=[porthole(h[0], 1.0), porthole(h[0], -1.0), porthole(h[1], 1.4)], accesses=acc, props=props)
    out[k] = d

    # 9. Longhouse bent (kit): two barrel lobes meeting at an angle, over
    #    brick piers, a ramp down.
    deck = rrect(19.0, 13.0, 1.4)
    h = [hull("longhouse", (-3.5, 1.5), 0.3, (10.0, 6.0, 3.7)), hull("longhouse", (3.8, -2.4), -0.5, (8.0, 6.0, 3.7))]
    acc = [access("ramp", deck, 0.35, 6.0)]
    props = [prop("basket", (7.5, 3.5), 0.0, (0.9, 0.9, 0.8)), prop("sacks", (-7.0, -4.5), 0.0, (1.4, 0.6, 0.6)), prop("lamp", (8.0, -1.0), 0.0, (0.3, 0.3, 2.4))]
    k, d = building("horaro_longhouse_bent_01", "Longhouse bent, two vaults meeting", deck,
                    grid_supports("pier", deck, 0.42, 3.4, inset=0.9), h, seams=[(0, 1)],
                    doors=[door(h[1], 0.0), door(h[0], math.pi)], accesses=acc, props=props)
    out[k] = d

    # 10. Domes cluster (kit): three domes grown together on root bases.
    deck = ellipse(15.0, 13.0)
    h = [hull("dome", (-2.5, 0.5), 0.0, (7.2, 6.4, 3.6)), hull("dome", (2.4, -2.6), 0.0, (6.0, 5.4, 3.0)), hull("dome", (2.2, 2.8), 0.0, (5.2, 4.8, 2.7))]
    acc = [access("ladder", deck, 0.0)]
    props = [prop("basket", (6.0, 0.0), 0.0, (0.9, 0.9, 0.8))]
    k, d = building("horaro_domes_cluster_01", "Domes cluster, three lobes on roots", deck,
                    [support("root", (-3.0, 0.5), 1.4), support("root", (3.0, -2.5), 1.2), support("root", (2.5, 3.5), 1.1), support("root", (-1.5, -4.0), 0.9)],
                    h, seams=[(0, 1), (0, 2)], doors=[door(h[1], 0.2), door(h[2], -0.3)], portholes=[porthole(h[0], 0.4), porthole(h[0], -0.4)], accesses=acc, props=props)
    out[k] = d

    # 11. The fish deck (painting): a round deck with a smokehouse dome, a
    #     small smoke oven, drying frames hung with fish, round tables.
    deck = ellipse(16.0, 15.0)
    h = [hull("dome", (-4.0, 0), 0.0, (7.0, 6.2, 3.5))]
    acc = [access("ladder", deck, -0.35)]
    props = [prop("oven", (2.5, 4.2), 0.0, (1.8, 1.8, 1.5)), prop("frame", (0.5, -4.5), 0.3, (3.6, 0.3, 2.4)), prop("frame", (-3.0, -5.5), -0.2, (3.0, 0.3, 2.4)),
             prop("table", (3.5, 0.5), 0.0, (1.3, 1.3, 0.8)), prop("table", (3.0, -1.8), 0.0, (1.3, 1.3, 0.8)),
             prop("basket", (5.5, 2.5), 0.0, (0.8, 0.8, 0.7)), prop("basket", (-1.0, 5.5), 0.0, (0.8, 0.8, 0.7)), prop("coils", (-6.0, 3.5), 0.4, (1.8, 0.6, 0.3))]
    k, d = building("horaro_fish_deck_01", "Fish deck, drying frames and a smokehouse", deck,
                    ring_supports("drum", deck, 0.5, 3.3, inset=1.0) + [support("drum", (-4, 0), 0.6), support("drum", (2, 2), 0.5), support("drum", (2, -2.5), 0.5)],
                    h, doors=[door(h[0], 0.0)], portholes=[porthole(h[0], 1.0)], accesses=acc, props=props, use="workshop")
    out[k] = d

    # 12. The lamp-oil works (painting): a hearth with cauldrons under a
    #     thatch roof, a tall beehive hut, oil jars, a dock with a boat.
    deck = ellipse(16.0, 9.5)
    h = [hull("dome", (-4.5, 0), 0.0, (5.2, 5.2, 5.0))]   # the beehive hut: tall, as painted
    acc = [access("dock", deck, 0.0, 5.0)]
    props = [prop("roof", (2.0, 0.5), 0.0, (4.2, 3.6, 2.7)), prop("hearth", (2.0, 0.5), 0.0, (1.8, 1.8, 1.7)),
             prop("jars", (-1.0, -3.0), 0.0, (1.1, 1.1, 1.1)), prop("jars", (-1.2, 3.0), 0.0, (1.1, 1.1, 1.1)), prop("coils", (4.5, -3.2), 0.0, (1.8, 0.6, 0.3)),
             prop("lamp", (-1.8, 0.0), 0.0, (0.3, 0.3, 2.6)), prop("boat", (7.5, -4.5), 0.2, (3.2, 1.2, 0.5), 0.1 - FLOOR), prop("chimney", (-4.5, 0.0), 0.0, (0.3, 0.3, 0.8), 4.8)]
    k, d = building("horaro_lamp_oil_works_01", "Lamp-oil works, a hearth under thatch", deck,
                    ring_supports("drum", deck, 0.5, 3.6, inset=1.0) + [support("drum", (-4.5, 0), 0.55), support("drum", (2, 0.5), 0.5)],
                    h, doors=[door(h[0], 0.0)], portholes=[porthole(h[0], 1.3, 2.4)], accesses=acc, props=props, use="workshop")
    out[k] = d

    # 13. The rope walk (painting): a long covered walk over the sea, a
    #     hut at its landward end, the spinning wheel and coils beside it.
    deck = rrect(30.0, 6.4, 1.0)
    h = [hull("dome", (-11.0, 0), 0.0, (5.6, 5.0, 2.8)), hull("arcade", (4.5, 0), 0.0, (21.0, 4.4, 3.2))]
    acc = [access("walkway", deck, 0.0, 12.0), access("ladder", deck, Q + 0.8)]
    props = [prop("wheel", (-6.5, 1.8), Q, (1.1, 0.6, 1.2)), prop("coils", (-6.0, -1.9), 0.0, (1.8, 0.6, 0.3)), prop("bench", (-7.5, -2.5), 0.0, (1.6, 0.4, 0.45)),
             prop("yarnline", (10.0, 2.6), 0.0, (4.0, 0.2, 1.9)), prop("coils", (0.0, -2.3), 0.0, (1.8, 0.6, 0.3)), prop("boat", (6.0, -5.2), 0.1, (3.2, 1.2, 0.5), 0.1 - FLOOR)]
    k, d = building("horaro_rope_walk_01", "Rope walk, a covered walk over the sea", deck,
                    grid_supports("post", deck, 0.2, 2.9, inset=0.5), h, doors=[door(h[0], 0.0)], portholes=[porthole(h[0], 2.0)],
                    accesses=acc, props=props, use="workshop")
    out[k] = d

    # 14. The salt deck (painting): round pans of brine drying, a hearth
    #     before the open hut, a well-sweep to draw sea water, sacks.
    deck = ellipse(15.0, 15.0)
    h = [hull("dome", (-4.0, 0), 0.0, (7.0, 6.4, 3.6))]
    acc = [access("ladder", deck, -0.4)]
    pans = [prop("pan", (x, y), 0.0, (1.5, 1.5, 0.3)) for x, y in [(1.0, 3.8), (3.8, 2.6), (4.2, -1.0), (2.2, -3.4), (-0.5, -4.8)]]
    props = pans + [prop("hearth", (1.4, 0.4), 0.0, (1.6, 1.6, 1.6)), prop("sweep", (-1.5, 5.8), Q, (4.5, 0.5, 3.6)), prop("sacks", (-3.5, -4.5), 0.3, (1.4, 0.6, 0.6)),
                    prop("basket", (5.5, 1.0), 0.0, (0.8, 0.8, 0.7))]
    k, d = building("horaro_salt_deck_01", "Salt deck, pans and a well-sweep", deck,
                    ring_supports("drum", deck, 0.5, 3.4, inset=1.0) + [support("drum", (-4, 0), 0.6), support("drum", (2, 1), 0.5), support("drum", (1, -2.5), 0.5)],
                    h, doors=[door(h[0], 0.0)], portholes=[porthole(h[0], 1.1)], accesses=acc, props=props, use="workshop")
    out[k] = d

    # 15. The salt works (painting): a long pier with a dome at its end,
    #     a hearth smoking, long pans down its length, a sweep at the sea
    #     end, a walkway to the shore.
    deck = rrect(24.0, 7.5, 1.0)
    h = [hull("dome", (-8.0, 0), 0.0, (7.4, 6.4, 3.6))]
    acc = [access("walkway", deck, 0.0, 8.0), access("ladder", deck, -Q)]
    pans = [prop("pan", (x, y), 0.0, (1.8, 1.8, 0.3)) for x, y in [(-0.5, 1.8), (2.0, -1.6), (4.5, 1.8), (7.0, -1.6)]]
    props = pans + [prop("hearth", (-1.9, 0.0), 0.0, (1.6, 1.6, 1.6)), prop("sweep", (9.5, 2.6), 0.6, (4.5, 0.5, 3.6)), prop("sacks", (9.0, -2.0), 0.0, (1.4, 0.6, 0.6)),
                    prop("basket", (-1.5, -2.6), 0.0, (0.8, 0.8, 0.7)), prop("chimney", (-8.0, 0.0), 0.0, (0.3, 0.3, 0.8), 3.5)]
    k, d = building("horaro_salt_works_01", "Salt works, a pier of pans", deck,
                    grid_supports("drum", deck, 0.45, 3.2, inset=0.9), h, doors=[door(h[0], 0.0)], portholes=[porthole(h[0], 1.2), porthole(h[0], -1.2)],
                    accesses=acc, props=props, use="workshop")
    out[k] = d

    # 16. The weaving hall (painting): a great woven canopy held up on
    #     tree-like posts over the looms, dye pots and lines of dyed yarn.
    deck = ellipse(19.0, 17.0)
    h = [hull("lid", (0, 0), 0.0, (23.0, 20.0, 7.0))]
    acc = [access("ladder", deck, -0.3), access("dock", deck, Q + 0.5, 4.0)]
    props = [prop("treepost", (-4.0, 2.5), 0.0, (3.0, 3.0, 3.85)), prop("treepost", (3.5, 3.0), 0.0, (3.0, 3.0, 3.85)), prop("treepost", (0.5, -4.0), 0.0, (3.0, 3.0, 3.85)),
             prop("loom", (-2.0, 0.0), -Q, (2.2, 0.5, 2.2)), prop("loom", (1.5, 0.0), -Q, (2.2, 0.5, 2.2)), prop("loom", (-0.2, 5.0), Q, (2.2, 0.5, 2.2)),
             prop("dyepots", (-6.0, -2.5), 0.3, (1.8, 0.6, 0.5)), prop("yarnline", (-6.5, 1.5), Q, (4.0, 0.2, 1.9)), prop("yarnline", (5.5, -3.0), 0.4, (4.0, 0.2, 1.9)),
             prop("bench", (3.5, -1.5), 0.0, (1.6, 0.4, 0.45)), prop("basket", (6.5, 2.0), 0.0, (0.8, 0.8, 0.7))]
    k, d = building("horaro_weaving_hall_01", "Weaving hall under a woven canopy", deck,
                    ring_supports("drum", deck, 0.5, 3.4, inset=1.0) + [support("drum", (-4, 2.5), 0.5), support("drum", (3.5, 3), 0.5), support("drum", (0.5, -4), 0.5), support("drum", (0, 0), 0.5)],
                    h, accesses=acc, props=props, use="workshop")
    out[k] = d

    return check(out)


RAISE = True


def levels(d):
    """One deck's walk-through: the deck less the hulls (a door is a way
    in, not a way across), the gear with a footprint, shrunk by half a
    body; from each way on, a person must reach every door and every
    piece of work."""
    q = d["horaro"]
    deck = d["deck"]
    obstacles = []
    for h in q["hulls"]:
        if h["kind"] in ("lid", "arcade"):
            continue
        obstacles.append(hull_plan(h))
    for p in q["props"]:
        f = FOOTPRINT.get(p["kind"], 1.0)
        if f <= 0.0 or p["z"] < -0.5:
            continue
        x, y = p["at"]; w, dd = p["size"][0] * f, p["size"][1] * f
        obstacles.append(affinity.rotate(box(x - w / 2, y - dd / 2, x + w / 2, y + dd / 2), p["rot"], origin=(x, y), use_radians=True))
    blocked = unary_union(obstacles) if obstacles else Polygon()
    free = deck.buffer(-BODY).difference(blocked.buffer(BODY)) if not blocked.is_empty else deck.buffer(-BODY)
    pieces = list(free.geoms) if free.geom_type == "MultiPolygon" else ([free] if not free.is_empty else [])

    def piece_of(pt):
        for i, pc in enumerate(pieces):
            if pc.buffer(0.02).contains(pt):
                return i
        return None

    seeds = []
    for a in q["access"]:
        x, y = a["at"]; yaw = a["yaw"]
        seeds.append((f"{a['kind']} at {a['at']}", Point(x - math.cos(yaw) * (BODY + 0.4), y - math.sin(yaw) * (BODY + 0.4))))
    targets = []
    for dr in q["doors"]:
        x, y = dr["xy"]; yaw = dr["yaw"]
        targets.append((f"door at {dr['xy']}", Point(x + math.cos(yaw) * (BODY + 0.4), y + math.sin(yaw) * (BODY + 0.4))))
    for p in q["props"]:
        if p["kind"] in WORKED and p["z"] > -0.5:
            x, y = p["at"]
            f = FOOTPRINT.get(p["kind"], 1.0)
            reach = max(p["size"][0], p["size"][1]) * f / 2 + BODY + 0.35
            # Any side of it will do: the nearest free spot within reach.
            pt = Point(x, y)
            best = None
            for pc in pieces:
                if pc.distance(pt) <= reach:
                    best = pc.exterior.interpolate(pc.exterior.project(pt)) if not pc.contains(pt) else pt
                    break
            targets.append((f"{p['kind']} at {p['at']}", best if best is not None else pt))
    problems = []
    seed_pieces = {name: piece_of(pt) for name, pt in seeds}
    for name, pc in seed_pieces.items():
        if pc is None:
            problems.append(f"{d['label']}: nowhere to stand at the top of the {name}")
    for name, pt in targets:
        pc = piece_of(pt)
        if pc is None:
            problems.append(f"{d['label']}: no room to stand at the {name} ({pt.x:.1f}, {pt.y:.1f})")
        elif not any(sp == pc for sp in seed_pieces.values()):
            problems.append(f"{d['label']}: no way to the {name} from the way on ({pt.x:.1f}, {pt.y:.1f})")
    return dict(deck=deck, blocked=blocked, pieces=pieces, seeds=seeds, targets=targets), problems


def check(out):
    problems = []
    for key, d in out.items():
        q = d["horaro"]
        deck = d["deck"]
        for s in q["supports"]:
            if not deck.buffer(0.2).contains(Point(s["at"])):
                problems.append(f"{key}: a {s['kind']} support at {s['at']} isn't under the deck")
        for h in q["hulls"]:
            if h["kind"] in ("lid", "arcade"):
                continue
            if not deck.buffer(0.3).contains(hull_plan(h)):
                problems.append(f"{key}: the {h['kind']} at {h['at']} hangs off the deck")
        for i, j in q["seams"]:
            if not hull_plan(q["hulls"][i], rho=1.0).intersects(hull_plan(q["hulls"][j], rho=1.0)):
                problems.append(f"{key}: hulls {i} and {j} are lashed together but don't touch")
        for p in q["props"]:
            if p["z"] < -0.5:
                continue
            if not deck.buffer(0.4).contains(Point(p["at"])):
                problems.append(f"{key}: {p['kind']} at {p['at']} isn't on the deck")
        for a in q["access"]:
            if deck.exterior.distance(Point(a["at"])) > 0.1:
                problems.append(f"{key}: the {a['kind']} at {a['at']} isn't on the deck's edge")
        problems += levels(d)[1]
    if problems and RAISE:
        raise ValueError("\n".join(problems))
    return out


def draw(d, path):
    from PIL import Image, ImageDraw
    L, problems = levels(d)
    S = 20
    pad = 4.0
    minx, miny, maxx, maxy = L["deck"].bounds
    x0, x1, y0, y1 = minx - pad, maxx + pad + 8, miny - pad, maxy + pad
    W, H = int((x1 - x0) * S), int((y1 - y0) * S)
    img = Image.new("RGB", (W, H + 24), (30, 30, 30))
    dr = ImageDraw.Draw(img)
    P = lambda x, y: ((x - x0) * S, 24 + (y1 - y) * S)

    def poly(g, fill, outline=None):
        for gg in (g.geoms if hasattr(g, "geoms") else [g]):
            if gg.is_empty or gg.geom_type != "Polygon":
                continue
            dr.polygon([P(x, y) for x, y in gg.exterior.coords], fill=fill, outline=outline)
            for ring in gg.interiors:
                dr.polygon([P(x, y) for x, y in ring.coords], fill=(60, 60, 60))
    dr.text((4, 4), d["label"], fill=(230, 230, 230))
    poly(L["deck"], (120, 120, 120))
    poly(L["blocked"], (60, 60, 60))
    greens = [(70, 160, 70), (120, 190, 90), (60, 140, 120), (150, 180, 60)]
    for i, pc in enumerate(L["pieces"]):
        poly(pc, greens[i % len(greens)])
    for h in d["horaro"]["hulls"]:
        poly(hull_plan(h), None, outline=(220, 200, 120))
    if d["horaro"]["quay"]:
        poly(Polygon(d["horaro"]["quay"]["outline"]), (80, 80, 90))
    for name, pt in L["seeds"]:
        cx, cy = P(pt.x, pt.y)
        dr.ellipse([cx - 5, cy - 5, cx + 5, cy + 5], fill=(70, 110, 230))
    for name, pt in L["targets"]:
        cx, cy = P(pt.x, pt.y)
        wrong = any(name in pr for pr in problems)
        dr.ellipse([cx - 6, cy - 6, cx + 6, cy + 6], outline=(230, 60, 60) if wrong else (250, 250, 250), width=3)
    img.save(path)
    return problems


if __name__ == "__main__":
    import sys, os
    args = [a for a in sys.argv[2:] if not a.startswith("--")]
    outdir = next((a.split("=", 1)[1] for a in sys.argv if a.startswith("--out=")), ".")
    if len(sys.argv) > 1 and sys.argv[1] == "walk":
        RAISE = False
        for key, d in designs().items():
            if args and key not in args:
                continue
            probs = draw(d, os.path.join(outdir, f"walk_{key}.png"))
            print(key, "ok" if not probs else "\n  " + "\n  ".join(probs))
