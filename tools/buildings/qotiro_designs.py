"""The Qotiro buildings (Laz, 15 Oct: the same level of detail as the Roduro
batch), designed from the concept paintings in
`E:\\Gahturiyu 3d Assets\\Concept art\\Buildings\\qotiro_*.jpg` and the kit's
grid (`claude/qotiro-kit.md`): quarried stone in stepped boxes on a 2 m
grid, a storey 3.6 m (walls 3.3 m, a 0.3 m slab), parapets 0.8 m round
every roof and terrace, outside stairs against the walls (6 m of steps for
a storey, 1.4 m wide, a landing where the parapet is left out), awnings
on poles, herb troughs and jars on the terraces, a gold sun disc by the
door.

Metres, the building's own frame: x along its facing (the main door on
+x), y across, z up. A tier is an axis-aligned box; the ground tier is
centred on the origin.

Each building: tiers, the ground floor's walls and rooms (as the Roduro
designs), windows per tier, and `qotiro`: the outside stairs, the doors of
the upper tiers, and props (kind, at, rot, size, z). The forge and the
drill yard have a walled yard (`yard`, `shed`, `gate`, as the Roduro trade
homes).
"""
import math

from shapely.geometry import Polygon, box
from shapely.ops import unary_union

from roduro_designs import level_of

STOREY = 3.6
WALL_TOP = 3.3
STAIR_RUN = 6.0     # metres of steps for one storey
STAIR_W = 1.4
# A stair up a 2 m terrace strip is a little wider, so its parapet's outer
# face comes to the terrace's edge and grows out of the terrace's parapet
# (a hair inside it, so the faces don't fight).
TERRACE_W = 1.53
Q = math.pi / 2


def rect(x0, y0, x1, y1):
    return box(x0, y0, x1, y1)


def corners(poly):
    """A box's four corners, counter-clockwise, as the tier outline."""
    minx, miny, maxx, maxy = poly.bounds
    return [(minx, miny), (maxx, miny), (maxx, maxy), (minx, maxy)]


def tier(poly, z0, height, name, wall_top=None):
    return dict(name=name, z0=z0, z1=z0 + height, wall_top=z0 + (WALL_TOP if wall_top is None else wall_top), outline=[(round(x, 3), round(y, 3)) for x, y in corners(poly)])


LANDING = 2.0


def stair(foot, yaw, z0, rise=STOREY, width=STAIR_W, run=None, landing=LANDING):
    """Steps from `foot` running the way `yaw` points, rising `rise` over
    `run` metres (6 m a storey unless said), then a landing `landing` long
    at the top, level with the terrace it serves and open to it along its
    inner side; the landing has its own parapet round the outside, and the
    terrace's parapet is left out along the landing. `head` is the
    landing's middle (where that gap is centred). A stair square to a face
    (the shrine's) has no landing: it meets the terrace head on."""
    length = run if run is not None else STAIR_RUN * rise / STOREY
    mid = length + landing * 0.5 if landing > 0 else length - 0.7
    head = (round(foot[0] + math.cos(yaw) * mid, 3), round(foot[1] + math.sin(yaw) * mid, 3))
    return dict(kind="stair" if landing > 0 else "steps", at=[round(foot[0], 3), round(foot[1], 3)], rot=round(yaw, 4), size=[round(length, 3), width, rise], z=z0, head=list(head), landing=landing)


def check(out):
    """Every building must make sense as a thing to walk on (Laz, 17 Oct:
    "the stairs are not coherent, some of the landings end up outside of
    the walking platform"). So: a stair stands wholly on the ground or on
    a terrace, runs beside walls (never through a tier that rises past
    its foot), and its top step has that terrace's floor right beside it;
    an upper door opens onto a terrace; a prop stands on one."""
    from shapely.geometry import Point
    from shapely import affinity
    problems = []
    for key, d in out.items():
        tiers = [(Polygon(t["outline"]), t["z0"], t["z1"]) for t in d["tiers"]]

        def floor_at(z):
            """The walkable floor at height z: the ground, or the tops of the
            tiers that end there, less whatever stands on them."""
            if z < 0.01:
                base = box(-200, -200, 200, 200).difference(unary_union([p for p, z0, _ in tiers if z0 < 0.01]))
            else:
                tops = [p for p, _, z1 in tiers if abs(z1 - z) < 0.05]
                if not tops:
                    return None
                base = unary_union(tops)
            above = [p for p, z0, z1 in tiers if z0 < z + 0.05 and z1 > z + 0.05]
            return base.difference(unary_union(above)) if above else base

        q = d["qotiro"]
        for st in q["stairs"]:
            x, y = st["at"]; yaw = st["rot"]; run, w, rise = st["size"][0], abs(st["size"][1]), st["size"][2]
            z = st["z"]; landing = st["landing"]
            whole = affinity.rotate(box(0, -w / 2, run + landing, w / 2), yaw, origin=(0, 0), use_radians=True)
            whole = affinity.translate(whole, x, y)
            fl = floor_at(z)
            if fl is None or not fl.buffer(0.05).contains(whole):
                problems.append(f"{key}: stair at {st['at']} (with its landing) doesn't stand on the floor at {z} m")
            blocking = [p for p, z0, z1 in tiers if z0 < z + rise - 0.05 and z1 > z + 0.05 and p.buffer(-0.05).intersects(whole)]
            if blocking:
                problems.append(f"{key}: stair at {st['at']} runs through a tier")
            top = floor_at(z + rise)
            n = (-math.sin(yaw), math.cos(yaw))
            side = -1 if st["size"][1] < 0 else 1   # the wall side; the terrace is the other
            if landing > 0:
                # The terrace must run the whole length of the landing's inner side.
                probes = [Point(x + math.cos(yaw) * (run + t) - n[0] * side * (w / 2 + 0.5), y + math.sin(yaw) * (run + t) - n[1] * side * (w / 2 + 0.5)) for t in (0.3, landing - 0.3)]
            else:
                probes = [Point(x + math.cos(yaw) * (run + 0.5), y + math.sin(yaw) * (run + 0.5))]
            bad = [pr for pr in probes if top is None or not top.buffer(0.05).contains(pr)]
            if bad:
                problems.append(f"{key}: stair at {st['at']} tops out at {z + rise} m with no terrace beside its landing ({bad[0].x:.1f}, {bad[0].y:.1f})")
        for door in q["doors_up"]:
            x, y = door["xy"]; yaw = door["yaw"]; z = door["z"]
            fl = floor_at(z)
            probe = Point(x + math.cos(yaw) * 0.8, y + math.sin(yaw) * 0.8)
            if fl is None or not fl.buffer(0.05).contains(probe):
                problems.append(f"{key}: door at {door['xy']} ({z} m) opens onto nothing")
            on_wall = any(abs(z0 - z) < 0.05 and p.exterior.distance(Point(x, y)) < 0.05 for p, z0, _ in tiers)
            if not on_wall:
                problems.append(f"{key}: door at {door['xy']} ({z} m) isn't on a wall of that storey")
        for pr in q["props"]:
            if pr["kind"] in ("disc", "arch", "gate", "railing", "chimney", "stair"):
                continue
            x, y = pr["at"]; z = pr["z"]; w, dd = pr["size"][0], pr["size"][1]
            foot = affinity.rotate(box(x - w / 2, y - dd / 2, x + w / 2, y + dd / 2), pr["rot"], origin=(x, y), use_radians=True)
            fl = floor_at(z)
            if fl is None or not fl.buffer(0.1).contains(foot):
                problems.append(f"{key}: {pr['kind']} at {pr['at']} ({z} m) isn't on a terrace")
    for key, d in out.items():
        problems += levels(d)[1]
    if problems and RAISE:
        raise ValueError("\n".join(problems))
    return out


RAISE = True   # the walk tool below turns this off to draw what's wrong


BODY = 0.3   # half a person's width: a way narrower than 0.6 m is no way


def levels(d):
    """The walk-through of one building, storey by storey: for each floor
    height, the floor (the ground, or the terraces at that height plus the
    landings of the stairs that come up to it), what stands on it (tiers
    rising from it, stairs that start on it with their landings, props
    with a footprint), the free floor a person can stand on (the floor
    shrunk by `BODY`, less the obstacles grown by it), its connected
    pieces, where a person arrives (the ground: from outside; a terrace:
    off each landing) and where they must be able to get to (every door at
    that height, the foot of every stair that starts there, and a step
    off each landing onto the terrace). Returns a list of level dicts and
    the problems found."""
    from shapely.geometry import Point
    from shapely import affinity
    tiers = [(Polygon(t["outline"]), t["z0"], t["z1"]) for t in d["tiers"]]
    q = d["qotiro"]

    def footprint(st, with_landing=True):
        run, w = st["size"][0], abs(st["size"][1])
        length = run + (st["landing"] if with_landing else 0.0)
        r = affinity.rotate(box(0, -w / 2, length, w / 2), st["rot"], origin=(0, 0), use_radians=True)
        return affinity.translate(r, *st["at"])

    def landing_rect(st):
        run, w = st["size"][0], abs(st["size"][1])
        r = affinity.rotate(box(run, -w / 2, run + st["landing"], w / 2), st["rot"], origin=(0, 0), use_radians=True)
        return affinity.translate(r, *st["at"])

    heights = sorted({0.0} | {round(t["z1"], 2) for t in d["tiers"]} | {round(st["z"] + st["size"][2], 2) for st in q["stairs"]})
    out = []
    problems = []
    for z in heights:
        here = lambda v: abs(v - z) < 0.05
        if z < 0.01:
            floor = box(-40, -40, 40, 40).difference(unary_union([p for p, z0, _ in tiers if z0 < 0.01]))
        else:
            tops = [p for p, _, z1 in tiers if here(z1)]
            floor = unary_union(tops) if tops else Polygon()
        arrivals = [st for st in q["stairs"] if here(st["z"] + st["size"][2]) and st["landing"] > 0]
        if arrivals:
            # (Grown a hair: a stair's rotation is rounded to four places,
            # so its landing may stand a fraction of a millimetre off the
            # terrace it meets.)
            floor = unary_union([floor] + [landing_rect(st).buffer(0.01, join_style=2) for st in arrivals])
        if floor.is_empty:
            continue
        obstacles = [p for p, z0, z1 in tiers if z0 < z + 0.05 and z1 > z + 0.05]
        departures = [st for st in q["stairs"] if here(st["z"])]
        obstacles += [footprint(st) for st in departures]
        for pr in q["props"]:
            if pr["kind"] in ("trough", "jars", "oven", "post", "chimney", "pavilion") and here(pr["z"]):
                x, y = pr["at"]; w, dd = pr["size"][0], pr["size"][1]
                if pr["kind"] == "pavilion":
                    continue   # open between its columns
                obstacles.append(affinity.rotate(box(x - w / 2, y - dd / 2, x + w / 2, y + dd / 2), pr["rot"], origin=(x, y), use_radians=True))
        blocked = unary_union(obstacles) if obstacles else Polygon()
        free = floor.buffer(-BODY).difference(blocked.buffer(BODY)) if not blocked.is_empty else floor.buffer(-BODY)
        pieces = list(free.geoms) if free.geom_type == "MultiPolygon" else ([free] if not free.is_empty else [])

        def piece_of(pt):
            for i, pc in enumerate(pieces):
                if pc.buffer(0.02).contains(pt):
                    return i
            return None

        seeds = []
        if z < 0.01:
            seeds.append(("outside", Point(39, 39)))
        for st in arrivals:
            lr = landing_rect(st)
            seeds.append((f"landing of stair from {st['at']}", lr.centroid))
        targets = []
        for st in departures:
            x, y = st["at"]; yaw = st["rot"]
            targets.append((f"foot of stair at {st['at']}", Point(x - math.cos(yaw) * (BODY + 0.35), y - math.sin(yaw) * (BODY + 0.35))))
        for door in q["doors_up"]:
            if here(door["z"]):
                x, y = door["xy"]; yaw = door["yaw"]
                targets.append((f"door at {door['xy']}", Point(x + math.cos(yaw) * (BODY + 0.35), y + math.sin(yaw) * (BODY + 0.35))))
        for st in arrivals:
            # A step off the landing onto the terrace, at its middle.
            run, w = st["size"][0], abs(st["size"][1]); yaw = st["rot"]
            side = -1 if st["size"][1] < 0 else 1
            n = (-math.sin(yaw), math.cos(yaw))
            mid = run + st["landing"] / 2
            x, y = st["at"]
            targets.append((f"off the landing of stair from {st['at']}", Point(x + math.cos(yaw) * mid - n[0] * side * (w / 2 + BODY + 0.35), y + math.sin(yaw) * mid - n[1] * side * (w / 2 + BODY + 0.35))))
        seed_pieces = {name: piece_of(pt) for name, pt in seeds}
        for name, pc in seed_pieces.items():
            if pc is None:
                problems.append(f"{d['label']}: at {z} m, nowhere to stand at the {name}")
        for name, pt in targets:
            pc = piece_of(pt)
            if pc is None:
                problems.append(f"{d['label']}: at {z} m, no room to stand at the {name} ({pt.x:.1f}, {pt.y:.1f})")
            elif not any(sp == pc for sp in seed_pieces.values()):
                problems.append(f"{d['label']}: at {z} m, no way to the {name} from " + " or ".join(seed_pieces) + f" ({pt.x:.1f}, {pt.y:.1f})")
        out.append(dict(z=z, floor=floor, blocked=blocked, free=free, pieces=pieces, seeds=seeds, targets=targets, stairs=[(st, footprint(st, False), landing_rect(st)) for st in departures + arrivals]))
    return out, problems


def draw_levels(d, path):
    """The walk-through as a picture: one plan per storey, side by side.
    Floor grey, what stands on it dark, the free floor green (each
    connected piece its own green), arrivals as blue dots, the places a
    person must reach as rings (red where they can't)."""
    from PIL import Image, ImageDraw
    lv, problems = levels(d)
    S = 24  # px per metre
    pad = 3.0
    xs = [x for t in d["tiers"] for x, _ in t["outline"]] + [x for st in d["qotiro"]["stairs"] for x in (st["at"][0] - 8, st["at"][0] + 8)]
    ys = [y for t in d["tiers"] for _, y in t["outline"]] + [y for st in d["qotiro"]["stairs"] for y in (st["at"][1] - 8, st["at"][1] + 8)]
    x0, x1, y0, y1 = min(xs) - pad, max(xs) + pad, min(ys) - pad, max(ys) + pad
    W, H = int((x1 - x0) * S), int((y1 - y0) * S)
    img = Image.new("RGB", (W * len(lv) + 10 * (len(lv) - 1), H + 24), (30, 30, 30))
    dr = ImageDraw.Draw(img)
    bad = set()
    for pr in problems:
        bad.add(pr)

    def P(x, y, k):
        return (k * (W + 10) + (x - x0) * S, 24 + (y1 - y) * S)   # north up

    def poly(g, fill, k, outline=None):
        geoms = g.geoms if hasattr(g, "geoms") else [g]
        for gg in geoms:
            if gg.is_empty or gg.geom_type != "Polygon":
                continue
            dr.polygon([P(x, y, k) for x, y in gg.exterior.coords], fill=fill, outline=outline)
            for ring in gg.interiors:
                dr.polygon([P(x, y, k) for x, y in ring.coords], fill=(30, 30, 30) if fill != (120, 120, 120) else (60, 60, 60))
    for k, L in enumerate(lv):
        dr.text((k * (W + 10) + 4, 4), f"{d['label']}  floor at {L['z']} m", fill=(230, 230, 230))
        floor = L["floor"].intersection(box(x0, y0, x1, y1))
        poly(floor, (120, 120, 120), k)
        poly(L["blocked"].intersection(box(x0, y0, x1, y1)), (60, 60, 60), k)
        greens = [(70, 160, 70), (120, 190, 90), (60, 140, 120), (150, 180, 60)]
        for i, pc in enumerate(L["pieces"]):
            poly(pc.intersection(box(x0, y0, x1, y1)), greens[i % len(greens)], k)
        for st, steps, land in L["stairs"]:
            poly(steps, (170, 140, 90), k, outline=(40, 30, 20))
            if st["landing"] > 0:
                poly(land, (200, 180, 120), k, outline=(40, 30, 20))
            x, y = st["at"]; yaw = st["rot"]; run = st["size"][0]
            dr.line([P(x, y, k), P(x + math.cos(yaw) * run, y + math.sin(yaw) * run, k)], fill=(40, 30, 20), width=2)
        for name, pt in L["seeds"]:
            cx, cy = P(pt.x, pt.y, k)
            dr.ellipse([cx - 5, cy - 5, cx + 5, cy + 5], fill=(70, 110, 230))
        for name, pt in L["targets"]:
            cx, cy = P(pt.x, pt.y, k)
            wrong = any(name in pr and f"at {L['z']} m" in pr for pr in problems)
            dr.ellipse([cx - 6, cy - 6, cx + 6, cy + 6], outline=(230, 60, 60) if wrong else (250, 250, 250), width=3)
    img.save(path)
    return problems


def prop(kind, at, rot=0.0, size=(1.0, 1.0, 1.0), z=0.0):
    return dict(kind=kind, at=[round(at[0], 3), round(at[1], 3)], rot=round(rot, 4), size=[round(v, 3) for v in size], z=z)


def door_up(xy, yaw, z):
    return dict(xy=[round(xy[0], 3), round(xy[1], 3)], yaw=round(yaw, 4), z=z)


def wall_sides(out):
    """Each stair's side wall goes on the side away from the building: the
    width is made negative when the wall belongs on the stair's left
    (its -side), as `interiors::qotiro_prop` reads it."""
    from shapely.geometry import Point
    for d in out.values():
        for st in d["qotiro"]["stairs"]:
            yaw = st["rot"]
            side = (-math.sin(yaw), math.cos(yaw))
            probe = Point(st["at"][0] + side[0] * 1.2, st["at"][1] + side[1] * 1.2)
            # The tiers that stand beside the stair: those rising above its foot.
            beside = [Polygon(t["outline"]) for t in d["tiers"] if t["z1"] > st["z"] + 0.5]
            if any(t.buffer(0.2).contains(probe) for t in beside):
                st["size"][1] = -abs(st["size"][1])
            else:
                st["size"][1] = abs(st["size"][1])
    return out


def designs():
    out = {}

    # 1. A small home: one storey, a room on the roof at the back, the
    #    awning terrace in front of it.
    g = rect(-3, -3, 3, 3)
    walls = [((-0.6, 3.0), (-0.6, -3.0), 0.5)]
    out["qotiro_home_small_01"] = dict(
        label="Small home, a room on the roof", kit="home",
        tiers=[tier(g, 0.0, STOREY, "A"), tier(rect(-3, -1, 1, 3), STOREY, 2.7, "B", 2.4)],
        level=level_of(g, walls, dict(main=(1.5, 0.0), bedroom=(-2.0, 0.0)), [((3.0, 0.5), 0.0)]),
        windows=[2, 1],
        qotiro=dict(
            stairs=[stair((3.0, -3.7), math.pi, 0.0, run=4.0)],
            doors_up=[door_up((1.0, 1.0), 0.0, STOREY)],
            props=[prop("disc", (3.0, -1.4), 0.0, (0.8, 0.1, 0.8), 2.2), prop("awning", (2.0, -1.6), 0.0, (1.8, 2.6, 2.3), STOREY),
                   prop("trough", (2.4, 2.1), 0.0, (1.0, 0.5, 0.5), STOREY), prop("jars", (2.5, -2.5), 0.0, (0.9, 0.9, 0.8), STOREY)],
        ),
    )

    # 2. A small home with its roof room at one end.
    g = rect(-4, -3, 4, 3)
    walls = [((-0.5, 3.0), (-0.5, -3.0), 0.5), ((-0.5, 0.3), (-4.0, 0.3), 0.5)]
    out["qotiro_home_small_end_01"] = dict(
        label="Small home, the roof room at the end", kit="home",
        tiers=[tier(g, 0.0, STOREY, "A"), tier(rect(-4, -3, 0, 3), STOREY, 2.9, "B", 2.6)],
        level=level_of(g, walls, dict(main=(1.8, 0.0), bedroom=(-2.3, 1.7), store=(-2.3, -1.5)), [((4.0, 0.3), 0.0)]),
        windows=[3, 2],
        qotiro=dict(
            stairs=[stair((-4.0, -3.7), 0.0, 0.0)],
            doors_up=[door_up((0.0, -0.8), 0.0, STOREY)],
            props=[prop("disc", (4.0, -1.5), 0.0, (0.8, 0.1, 0.8), 2.2), prop("awning", (2.0, 0.6), 0.0, (3.2, 4.0, 2.3), STOREY),
                   prop("trough", (3.4, 0.6), 0.0, (1.4, 0.5, 0.5), STOREY), prop("trough", (3.4, 2.4), 0.0, (1.0, 0.5, 0.5), STOREY),
                   prop("jars", (0.8, -2.4), 0.0, (0.9, 0.9, 0.8), STOREY)],
        ),
    )

    # 3. A low small home: one storey, the stair's hut and a bread oven on
    #    the roof, the awning over the rest.
    g = rect(-4, -3, 4, 3)
    walls = [((-0.4, 3.0), (-0.4, -3.0), 0.5), ((4.0, -0.6), (-0.4, -0.6), 0.5)]
    out["qotiro_home_small_low_01"] = dict(
        label="Low small home, an oven on the roof", kit="home",
        tiers=[tier(g, 0.0, STOREY, "A"), tier(rect(-4, -3, -2, -1), STOREY, 1.9, "B", 1.7)],
        level=level_of(g, walls, dict(main=(1.8, 1.2), kitchen=(1.8, -1.9), bedroom=(-2.2, 0.0)), [((4.0, 1.4), 0.0)]),
        windows=[3, 0],
        qotiro=dict(
            stairs=[stair((-4.0, -3.7), 0.0, 0.0)],
            doors_up=[door_up((-2.0, -2.0), 0.0, STOREY)],
            props=[prop("disc", (4.0, -0.2), 0.0, (0.8, 0.1, 0.8), 2.2), prop("awning", (1.2, 0.8), 0.0, (3.6, 3.2, 2.3), STOREY),
                   prop("oven", (-2.6, 2.0), 0.0, (1.2, 1.2, 1.3), STOREY), prop("trough", (3.3, 0.0), 0.0, (1.2, 0.5, 0.5), STOREY),
                   prop("trough", (3.3, 2.4), 0.0, (1.2, 0.5, 0.5), STOREY), prop("jars", (-0.6, -2.3), 0.0, (0.9, 0.9, 0.8), STOREY)],
        ),
    )

    # 4. A tall small home: three storeys up an 8 x 6 plot, each stepped
    #    back. The first flight climbs the south face to the east end of
    #    the terrace; the second starts in that corner and runs along the
    #    roof room's south face, landing beside the top room. (A 6 x 6 plot
    #    can't hold two flights and their landings: Laz, 17 Oct, the
    #    walk-through check.)
    g = rect(-4, -3, 4, 3)
    walls = [((-0.4, 3.0), (-0.4, -3.0), 0.5)]
    out["qotiro_home_small_tall_01"] = dict(
        label="Tall small home, three storeys", kit="home",
        tiers=[tier(g, 0.0, STOREY, "A"), tier(rect(-4, -1, 0, 3), STOREY, STOREY, "B"), tier(rect(-2, -1, 0, 3), 2 * STOREY, 3.2, "C", 2.9)],
        level=level_of(g, walls, dict(main=(1.8, 0.0), store=(-2.2, 0.0)), [((4.0, 0.3), 0.0)]),
        windows=[2, 2, 2],
        qotiro=dict(
            stairs=[stair((-4.0, -3.7), 0.0, 0.0), stair((2.0, -1.765), math.pi, STOREY, run=4.0, width=TERRACE_W)],
            doors_up=[door_up((0.0, 1.0), 0.0, STOREY), door_up((-2.0, 1.0), math.pi, 2 * STOREY)],
            props=[prop("disc", (4.0, -1.4), 0.0, (0.8, 0.1, 0.8), 2.2), prop("awning", (2.0, 1.2), 0.0, (1.8, 3.0, 2.3), STOREY),
                   prop("trough", (3.4, 2.4), 0.0, (1.0, 0.5, 0.5), STOREY), prop("jars", (-3.4, 2.4), 0.0, (0.9, 0.9, 0.8), 2 * STOREY)],
        ),
    )

    # 5. The family house: three stepped storeys, the first flight up the
    #    south flank, the second along the north face (so it never lies
    #    across the way off the first landing), the awning on the second
    #    terrace.
    g = rect(-7, -5, 7, 5)
    walls = [((-1.0, 5.0), (-1.0, -5.0), 0.55), ((7.0, -1.5), (-1.0, -1.5), 0.5), ((-1.0, 0.0), (-7.0, 0.0), 0.5)]
    out["qotiro_family_house_01"] = dict(
        label="Family house, three storeys stepped", kit="home",
        tiers=[tier(g, 0.0, STOREY, "A"), tier(rect(-7, -3, 3, 3), STOREY, STOREY, "B"), tier(rect(-7, -1, -3, 3), 2 * STOREY, 3.2, "C", 2.9)],
        level=level_of(g, walls, dict(hall=(3.0, 2.0), kitchen=(3.0, -3.3), bedroom=(-4.0, 2.5), back_room=(-4.0, -2.5)), [((7.0, 1.6), 0.0)]),
        windows=[4, 3, 2],
        qotiro=dict(
            stairs=[stair((7.0, -5.7), math.pi, 0.0), stair((3.0, 3.765), math.pi, STOREY, run=4.0, width=TERRACE_W)],
            doors_up=[door_up((3.0, -1.0), 0.0, STOREY), door_up((-3.0, 1.0), 0.0, 2 * STOREY)],
            props=[prop("disc", (7.0, -0.2), 0.0, (0.9, 0.1, 0.9), 2.3), prop("awning", (5.0, 1.0), 0.0, (3.6, 4.4, 2.4), STOREY),
                   prop("trough", (6.3, -4.3), 0.0, (1.4, 0.5, 0.5), STOREY), prop("trough", (6.3, 4.3), 0.0, (1.4, 0.5, 0.5), STOREY),
                   prop("trough", (-5.5, -4.3), 0.0, (1.4, 0.5, 0.5), STOREY), prop("jars", (-4.6, 4.2), 0.0, (0.9, 0.9, 0.8), STOREY),
                   prop("jars", (-4.8, -1.8), 0.0, (0.9, 0.9, 0.8), 2 * STOREY)],
        ),
    )

    # 6. The corner house: doors on two streets, the upper floors to the
    #    back corner, terraces on two sides.
    g = rect(-6, -5, 6, 5)
    walls = [((-1.5, 5.0), (-1.5, -5.0), 0.6), ((6.0, -1.0), (-1.5, -1.0), 0.5), ((-1.5, 1.0), (-6.0, 1.0), 0.5)]
    out["qotiro_home_corner_01"] = dict(
        label="Corner house, two storeys and a top room", kit="home",
        tiers=[tier(g, 0.0, STOREY, "A"), tier(rect(-6, -5, 0, 1), STOREY, STOREY, "B"), tier(rect(-6, -5, -2, -1), 2 * STOREY, 2.9, "C", 2.6)],
        level=level_of(g, walls, dict(hall=(2.5, 2.0), kitchen=(2.5, -3.0), bedroom=(-4.0, 3.0), store=(-4.0, -2.0)), [((6.0, 1.8), 0.0), ((-3.5, 5.0), Q)]),
        windows=[4, 3, 1],
        qotiro=dict(
            stairs=[stair((6.0, 5.7), math.pi, 0.0), stair((0.0, 1.7), math.pi, STOREY, run=4.0)],
            doors_up=[door_up((0.0, -2.0), 0.0, STOREY), door_up((-2.0, -3.0), 0.0, 2 * STOREY)],
            props=[prop("disc", (6.0, 0.0), 0.0, (0.9, 0.1, 0.9), 2.3), prop("awning", (3.0, 3.0), 0.0, (3.4, 3.4, 2.3), STOREY),
                   prop("awning", (-1.0, -3.0), 0.0, (1.8, 3.4, 2.3), 2 * STOREY),
                   prop("trough", (5.3, -4.3), 0.0, (1.4, 0.5, 0.5), STOREY), prop("trough", (1.5, 4.3), 0.0, (1.4, 0.5, 0.5), STOREY),
                   prop("jars", (4.6, -1.6), 0.0, (0.9, 0.9, 0.8), STOREY), prop("jars", (-1.0, -4.2), 0.0, (0.9, 0.9, 0.8), 2 * STOREY)],
        ),
    )

    # 7. The long house: a long front terrace at the first floor with
    #    awnings along it, stairs at both ends.
    g = rect(-4, -10, 4, 10)
    walls = [((0.0, -10.0), (0.0, -2.0), 0.5), ((0.0, -2.0), (0.0, 10.0), 0.4), ((4.0, -5.0), (0.0, -5.0), 0.5),
             ((0.0, -3.0), (-4.0, -3.0), 0.5), ((0.0, 4.0), (-4.0, 4.0), 0.5)]
    out["qotiro_home_long_01"] = dict(
        label="Long house, a terrace along the front", kit="home",
        tiers=[tier(g, 0.0, STOREY, "A"), tier(rect(-4, -10, 0, 10), STOREY, STOREY, "B")],
        level=level_of(g, walls, dict(hall=(2.0, 2.0), kitchen=(2.0, -7.5), bedroom=(-2.0, -6.5), back_room=(-2.0, 0.5), store=(-2.0, 7.0)), [((4.0, 0.0), 0.0)]),
        windows=[6, 5],
        qotiro=dict(
            stairs=[stair((-4.0, -10.7), 0.0, 0.0), stair((-4.0, 10.7), 0.0, 0.0)],
            doors_up=[door_up((0.0, 2.0), 0.0, STOREY), door_up((0.0, -6.0), 0.0, STOREY)],
            props=[prop("disc", (4.0, -1.6), 0.0, (0.9, 0.1, 0.9), 2.3), prop("awning", (2.0, -3.5), 0.0, (3.2, 4.0, 2.3), STOREY),
                   prop("awning", (2.0, 5.5), 0.0, (3.2, 4.0, 2.3), STOREY),
                   prop("trough", (3.3, -8.5), 0.0, (1.4, 0.5, 0.5), STOREY), prop("trough", (3.3, 0.8), 0.0, (1.4, 0.5, 0.5), STOREY),
                   prop("trough", (3.3, 8.6), 0.0, (1.4, 0.5, 0.5), STOREY), prop("jars", (1.0, 9.2), 0.0, (0.9, 0.9, 0.8), STOREY),
                   prop("jars", (1.0, -0.8), 0.0, (0.9, 0.9, 0.8), STOREY)],
        ),
    )

    # 10. The shrine hall: a platform with an arcade along its front and
    #     the hall inside, a stair up the middle to the sun's pavilion.
    g = rect(-6, -6, 6, 6)
    out["qotiro_shrine_hall_01"] = dict(
        label="Shrine hall, the sun's pavilion on its platform", kit="hall",
        tiers=[tier(g, 0.0, 3.2, "A", 2.9)],
        level=level_of(g, [], dict(hall=(0.0, 0.0)), [((6.0, 0.0), 0.0)]),
        windows=[0],
        qotiro=dict(
            stairs=[stair((10.6, 0.0), math.pi, 0.0, rise=3.2, width=2.4, run=4.6, landing=0.0)],
            doors_up=[],
            props=[prop("arch", (6.0, -4.0), 0.0, (1.6, 0.1, 2.4), 0.0), prop("arch", (6.0, -2.0), 0.0, (1.6, 0.1, 2.4), 0.0),
                   prop("arch", (6.0, 2.0), 0.0, (1.6, 0.1, 2.4), 0.0), prop("arch", (6.0, 4.0), 0.0, (1.6, 0.1, 2.4), 0.0),
                   prop("pavilion", (-1.5, 0.0), 0.0, (4.0, 4.0, 3.4), 3.2), prop("jars", (3.0, -4.6), 0.0, (0.9, 0.9, 0.8), 3.2)],
        ),
        use="hall",
    )
    return check(wall_sides(out))


def yard_designs():
    """The forge and the drill yard: a walled, gated yard before a block."""
    out = {}

    # 8. The forge: the smith's block at the back with its chimney tower,
    #    the hearth under a canopy before it, the yard walled with an
    #    iron gate.
    house = rect(-5, -5, -1, 5)
    walls = [((-3.0, 5.0), (-3.0, -5.0), 0.5)]
    out["qotiro_forge_01"] = dict(
        label="Forge, the yard walled, the chimney tower behind", kit="home", trade="smith",
        house=house,
        tiers=[tier(house, 0.0, STOREY, "A")],
        level=level_of(house, walls, dict(main=(-2.0, 2.0), bedroom=(-2.0, -2.5)), [((-1.0, 2.0), 0.0)]),
        windows=[2],
        yard=rect(-5, -5, 6, 5).difference(rect(-5.5, -4.7, -1.3, 4.7)),
        shed=rect(-1.3, -3.2, 1.6, 3.2),
        gate=(6.0, 0.0),
        chimney=None,
        base=2.0,
        qotiro=dict(
            stairs=[],
            doors_up=[],
            props=[prop("chimney", (-3.0, -3.0), 0.0, (2.2, 2.2, 11.0), 0.0), prop("gate", (6.0, 0.0), 0.0, (2.2, 0.1, 2.2), 0.0),
                   prop("railing", (6.0, -3.6), Q, (2.8, 0.05, 0.8), 2.0), prop("railing", (6.0, 3.6), Q, (2.8, 0.05, 0.8), 2.0),
                   prop("railing", (0.5, -5.0), 0.0, (11.0, 0.05, 0.8), 2.0), prop("railing", (0.5, 5.0), 0.0, (11.0, 0.05, 0.8), 2.0),
                   prop("disc", (-1.0, 4.0), 0.0, (0.7, 0.1, 0.7), 2.3)],
        ),
    )

    # 9. The drill yard: a paved yard with practice posts and racks before
    #    the guard hall, two storeys with its stair up the front.
    house = rect(-8, -7, -2, 7)
    walls = [((-8.0, -2.0), (-2.0, -2.0), 0.3), ((-5.0, 7.0), (-5.0, -2.0), 0.5)]
    out["qotiro_drill_yard_01"] = dict(
        label="Drill yard, the guard hall at its back", kit="hall", trade="drill",
        house=house,
        tiers=[tier(house, 0.0, STOREY, "A"), tier(rect(-8, -7, -4, 3), STOREY, STOREY, "B")],
        level=level_of(house, walls, dict(hall=(-3.5, 3.0), store=(-6.5, 3.0), back_room=(-6.5, -4.5), bedroom=(-3.5, -4.5)), [((-2.0, 3.5), 0.0)]),
        windows=[3, 2],
        yard=rect(-8, -7, 8, 7).difference(rect(-8.5, -6.7, -2.3, 6.7)),
        shed=rect(-2.3, 1.5, 1.6, 7.0),
        gate=(8.0, 0.0),
        chimney=None,
        base=1.6,
        qotiro=dict(
            stairs=[stair((-1.3, -7.0), Q, 0.0)],
            doors_up=[door_up((-4.0, 0.5), 0.0, STOREY)],
            props=[prop("disc", (-2.0, 1.5), 0.0, (0.9, 0.1, 0.9), 2.3), prop("awning", (-3.0, 5.0), 0.0, (1.8, 3.4, 2.3), STOREY),
                   prop("post", (2.0, -3.0), 0.0, (0.35, 0.35, 2.0), 0.0), prop("post", (4.5, 0.5), 0.0, (0.35, 0.35, 2.0), 0.0),
                   prop("post", (2.0, 3.5), 0.0, (0.35, 0.35, 2.0), 0.0), prop("gate", (8.0, 0.0), 0.0, (3.0, 0.1, 2.0), 0.0)],
        ),
    )
    return check(wall_sides(out))


# What the two yards hold (the Roduro `TRADE_YARDS` shape): stations under
# the canopy first, then the open ground, then the dressing.
TRADE_YARDS = {
    "smith": dict(
        shed=[("Forge", "wall"), ("Anvil", "middle")],
        open=[("Workbench", "wall"), ("Rack", "wall")],
        dress_shed=[("bellows", (0.9, 0.5, 0.9)), ("bucket", (0.4, 0.4, 0.3))],
        dress_open=[("trough", (1.9, 0.65, 0.6)), ("grindstone", (0.9, 0.5, 1.0)), ("barrel", (0.73, 0.75, 0.91)), ("pile", (1.3, 1.1, 0.5)), ("crate", (0.9, 0.9, 0.7))],
        chimney=False,
    ),
    "drill": dict(
        shed=[("Rack", "wall"), ("Rack", "wall")],
        open=[("Workbench", "wall")],
        dress_shed=[("crate", (0.9, 0.9, 0.7))],
        dress_open=[("trough", (1.9, 0.65, 0.6)), ("barrel", (0.73, 0.75, 0.91)), ("frame", (0.13, 1.6, 2.0)), ("sacks", (0.9, 0.6, 0.5))],
        chimney=False,
    ),
}


if __name__ == "__main__":
    # `python3 qotiro_designs.py walk [key …] [--out dir]`: draw the
    # walk-through plans and print the problems.
    import sys, os
    args = [a for a in sys.argv[2:] if not a.startswith("--")]
    outdir = next((a.split("=", 1)[1] for a in sys.argv if a.startswith("--out=")), ".")
    if len(sys.argv) > 1 and sys.argv[1] == "walk":
        RAISE = False
        all_ = {}
        all_.update(designs())
        all_.update(yard_designs())
        for key, d in all_.items():
            if args and key not in args:
                continue
            path = os.path.join(outdir, f"walk_{key}.png")
            probs = draw_levels(d, path)
            print(key, "ok" if not probs else "\n  " + "\n  ".join(probs))
