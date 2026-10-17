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
Q = math.pi / 2


def rect(x0, y0, x1, y1):
    return box(x0, y0, x1, y1)


def corners(poly):
    """A box's four corners, counter-clockwise, as the tier outline."""
    minx, miny, maxx, maxy = poly.bounds
    return [(minx, miny), (maxx, miny), (maxx, maxy), (minx, maxy)]


def tier(poly, z0, height, name, wall_top=None):
    return dict(name=name, z0=z0, z1=z0 + height, wall_top=z0 + (WALL_TOP if wall_top is None else wall_top), outline=[(round(x, 3), round(y, 3)) for x, y in corners(poly)])


def stair(foot, yaw, z0, rise=STOREY, width=STAIR_W, run=None):
    """Steps from `foot` running the way `yaw` points, rising `rise` over
    `run` metres (6 m a storey unless said). The top step is at `head`;
    the terrace it serves must lie beside the top of the run, and the
    parapet is left out there (`landing`, the middle of the last stride)."""
    length = run if run is not None else STAIR_RUN * rise / STOREY
    head = (round(foot[0] + math.cos(yaw) * length, 3), round(foot[1] + math.sin(yaw) * length, 3))
    landing = (round(foot[0] + math.cos(yaw) * (length - 0.7), 3), round(foot[1] + math.sin(yaw) * (length - 0.7), 3))
    return dict(kind="stair", at=[round(foot[0], 3), round(foot[1], 3)], rot=round(yaw, 4), size=[round(length, 3), width, rise], z=z0, head=list(landing))


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
            z = st["z"]
            foot = affinity.rotate(box(0, -w / 2, run, w / 2), yaw, origin=(0, 0), use_radians=True)
            foot = affinity.translate(foot, x, y)
            fl = floor_at(z)
            if fl is None or not fl.buffer(0.05).contains(foot):
                problems.append(f"{key}: stair at {st['at']} doesn't stand on the floor at {z} m")
            blocking = [p for p, z0, z1 in tiers if z0 < z + rise - 0.05 and z1 > z + 0.05 and p.buffer(-0.05).intersects(foot)]
            if blocking:
                problems.append(f"{key}: stair at {st['at']} runs through a tier")
            top = floor_at(z + rise)
            n = (-math.sin(yaw), math.cos(yaw))
            side = -1 if st["size"][1] < 0 else 1   # the wall side; the terrace is the other
            probe = Point(x + math.cos(yaw) * (run - 0.7) - n[0] * side * (w / 2 + 0.5), y + math.sin(yaw) * (run - 0.7) - n[1] * side * (w / 2 + 0.5))
            ahead = Point(x + math.cos(yaw) * (run + 0.5), y + math.sin(yaw) * (run + 0.5))
            if top is None or not (top.buffer(0.05).contains(probe) or top.buffer(0.05).contains(ahead)):
                problems.append(f"{key}: stair at {st['at']} tops out at {z + rise} m with no terrace beside its head ({probe.x:.1f}, {probe.y:.1f})")
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
    if problems:
        raise ValueError("\n".join(problems))
    return out


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
            stairs=[stair((3.0, -3.7), math.pi, 0.0)],
            doors_up=[door_up((1.0, 1.0), 0.0, STOREY)],
            props=[prop("disc", (3.0, -1.4), 0.0, (0.8, 0.1, 0.8), 2.2), prop("awning", (2.0, -1.6), 0.0, (1.8, 2.6, 2.3), STOREY),
                   prop("trough", (2.4, 2.1), 0.0, (1.0, 0.5, 0.5), STOREY), prop("jars", (-2.2, -2.2), 0.0, (0.9, 0.9, 0.8), STOREY)],
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
                   prop("trough", (3.4, -2.3), 0.0, (1.4, 0.5, 0.5), STOREY), prop("trough", (3.4, 2.4), 0.0, (1.0, 0.5, 0.5), STOREY),
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
            stairs=[stair((4.0, -3.7), math.pi, 0.0)],
            doors_up=[door_up((-2.0, -2.0), 0.0, STOREY)],
            props=[prop("disc", (4.0, -0.2), 0.0, (0.8, 0.1, 0.8), 2.2), prop("awning", (1.2, 0.8), 0.0, (3.6, 3.2, 2.3), STOREY),
                   prop("oven", (-2.6, 2.0), 0.0, (1.2, 1.2, 1.3), STOREY), prop("trough", (3.3, -2.3), 0.0, (1.2, 0.5, 0.5), STOREY),
                   prop("trough", (3.3, 2.4), 0.0, (1.2, 0.5, 0.5), STOREY), prop("jars", (-0.6, -2.3), 0.0, (0.9, 0.9, 0.8), STOREY)],
        ),
    )

    # 4. A tall small home: three storeys up a small plot, each stepped
    #    back, the stairs winding round.
    g = rect(-3, -3, 3, 3)
    walls = [((-0.8, 3.0), (-0.8, -3.0), 0.5)]
    out["qotiro_home_small_tall_01"] = dict(
        label="Tall small home, three storeys", kit="home",
        tiers=[tier(g, 0.0, STOREY, "A"), tier(rect(-3, -3, 1, 3), STOREY, STOREY, "B"), tier(rect(-3, -1, 1, 3), 2 * STOREY, 3.2, "C", 2.9)],
        level=level_of(g, walls, dict(main=(1.2, 0.0), store=(-2.0, 0.0)), [((3.0, 0.3), 0.0)]),
        windows=[2, 2, 2],
        qotiro=dict(
            stairs=[stair((-3.0, -3.7), 0.0, 0.0), stair((1.7, 3.0), -Q, STOREY)],
            doors_up=[door_up((1.0, 1.5), 0.0, STOREY), door_up((-1.0, -1.0), -Q, 2 * STOREY)],
            props=[prop("disc", (3.0, -1.4), 0.0, (0.8, 0.1, 0.8), 2.2), prop("awning", (2.0, 1.2), 0.0, (1.8, 3.0, 2.3), STOREY),
                   prop("trough", (2.4, -2.4), 0.0, (1.0, 0.5, 0.5), STOREY), prop("jars", (-2.2, -2.2), 0.0, (0.9, 0.9, 0.8), 2 * STOREY)],
        ),
    )

    # 5. The family house: three stepped storeys, stairs up the flank,
    #    the awning on the second terrace.
    g = rect(-7, -5, 7, 5)
    walls = [((-1.0, 5.0), (-1.0, -5.0), 0.55), ((7.0, -1.5), (-1.0, -1.5), 0.5), ((-1.0, 0.0), (-7.0, 0.0), 0.5)]
    out["qotiro_family_house_01"] = dict(
        label="Family house, three storeys stepped", kit="home",
        tiers=[tier(g, 0.0, STOREY, "A"), tier(rect(-7, -3, 3, 3), STOREY, STOREY, "B"), tier(rect(-7, -1, -3, 3), 2 * STOREY, 3.2, "C", 2.9)],
        level=level_of(g, walls, dict(hall=(3.0, 2.0), kitchen=(3.0, -3.3), bedroom=(-4.0, 2.5), back_room=(-4.0, -2.5)), [((7.0, 1.6), 0.0)]),
        windows=[4, 3, 2],
        qotiro=dict(
            stairs=[stair((7.0, -5.7), math.pi, 0.0), stair((3.0, -3.7), math.pi, STOREY)],
            doors_up=[door_up((3.0, -1.0), 0.0, STOREY), door_up((-3.0, 1.0), 0.0, 2 * STOREY)],
            props=[prop("disc", (7.0, -0.2), 0.0, (0.9, 0.1, 0.9), 2.3), prop("awning", (5.0, 1.0), 0.0, (3.6, 4.4, 2.4), STOREY),
                   prop("trough", (6.3, -4.3), 0.0, (1.4, 0.5, 0.5), STOREY), prop("trough", (6.3, 4.3), 0.0, (1.4, 0.5, 0.5), STOREY),
                   prop("trough", (-5.5, -4.3), 0.0, (1.4, 0.5, 0.5), STOREY), prop("jars", (1.6, 4.2), 0.0, (0.9, 0.9, 0.8), STOREY),
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
            stairs=[stair((6.0, 5.7), math.pi, 0.0), stair((-6.0, 1.7), 0.0, STOREY)],
            doors_up=[door_up((0.0, -2.0), 0.0, STOREY), door_up((-2.0, -3.0), 0.0, 2 * STOREY)],
            props=[prop("disc", (6.0, 0.0), 0.0, (0.9, 0.1, 0.9), 2.3), prop("awning", (3.0, 3.0), 0.0, (3.4, 3.4, 2.3), STOREY),
                   prop("awning", (-1.0, -3.0), 0.0, (1.8, 3.4, 2.3), 2 * STOREY),
                   prop("trough", (5.3, -4.3), 0.0, (1.4, 0.5, 0.5), STOREY), prop("trough", (1.5, 4.3), 0.0, (1.4, 0.5, 0.5), STOREY),
                   prop("jars", (4.6, -1.6), 0.0, (0.9, 0.9, 0.8), STOREY), prop("jars", (-1.0, 0.2), 0.0, (0.9, 0.9, 0.8), 2 * STOREY)],
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
            stairs=[stair((10.6, 0.0), math.pi, 0.0, rise=3.2, width=2.4, run=4.6)],
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
