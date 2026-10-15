"""Buildings designed here rather than modelled (Laz, 15 Oct): the gaps in
the Roduro catalogue, and trade homes with a work yard grown against one
wall. Each is a few parameters; `roduro_forms.py design` turns them into
the same spec files as the modelled ones.

Metres, the building's own frame: x along its facing (the main door on
+x), y across, z up. The outline is an oval (`squareness` 2 is an ellipse,
higher is boxier) with optional lobes, an indent, a flat side, or the
union of two ovals; upper tiers are the ground outline scaled and shifted.
"""
import json, math, os

from shapely.geometry import Polygon, Point, LineString, box
from shapely.ops import split, unary_union
from shapely import affinity

N_OUT = 36


def oval(rx, ry, squareness=2.0, lobes=None, indent=None, flat=None, n=360):
    """An oval as a polygon, metres."""
    pts = []
    for k in range(n):
        a = k / n * 2 * math.pi
        c, s = math.cos(a), math.sin(a)
        r = (abs(c / rx) ** squareness + abs(s / ry) ** squareness) ** (-1 / squareness)
        if lobes:
            m, amp, phase = lobes
            r *= 1 + amp * math.cos(m * (a - math.radians(phase)))
        pts.append((r * c, r * s))
    poly = Polygon(pts)
    if indent:
        ang, depth, width = indent
        d = (math.cos(math.radians(ang)), math.sin(math.radians(ang)))
        edge = poly.exterior.interpolate(poly.exterior.project(Point(d[0] * 50, d[1] * 50)))
        poly = poly.difference(Point(edge.x + d[0] * (width - depth), edge.y + d[1] * (width - depth)).buffer(width))
    if flat:
        ang, cut = flat
        d = (math.cos(math.radians(ang)), math.sin(math.radians(ang)))
        far = max(rx, ry) * 3
        side = (-d[1], d[0])
        reach = (abs(d[0]) * rx + abs(d[1]) * ry) * cut
        keep = Polygon([(d[0] * reach + side[0] * far, d[1] * reach + side[1] * far), (d[0] * reach - side[0] * far, d[1] * reach - side[1] * far),
                        (-d[0] * far - side[0] * far, -d[1] * far - side[1] * far), (-d[0] * far + side[0] * far, -d[1] * far + side[1] * far)])
        poly = poly.intersection(keep)
    return poly


def resample(poly, n=N_OUT):
    """`n` corners evenly along a polygon's edge, counter-clockwise."""
    ring = poly.exterior if poly.exterior.is_ccw else LineString(list(poly.exterior.coords)[::-1])
    L = ring.length
    pts = [ring.interpolate(k / n * L) for k in range(n)]
    return [(round(p.x, 2), round(p.y, 2)) for p in pts]


def rooms_of(outline, walls, names, inset=0.3):
    """Rooms: the outline split by the walls (carried right through), named
    by which spot falls in each."""
    inner = outline.buffer(-inset, join_style=2)
    parts = [inner]
    for a, b in walls:
        d = (b[0] - a[0], b[1] - a[1])
        l = math.hypot(*d)
        d = (d[0] / l * 60, d[1] / l * 60)
        cutter = LineString([(a[0] - d[0], a[1] - d[1]), (b[0] + d[0], b[1] + d[1])])
        parts = [q for p in parts for q in split(p, cutter).geoms]
    rooms = []
    for name, spot in names.items():
        here = [p for p in parts if p.contains(Point(spot))]
        if not here:
            raise ValueError(f"no room at {spot} for {name}")
        p = max(here, key=lambda q: q.area)
        rooms.append(dict(name=name, area_m2=round(p.area, 1), hub=name in ("hall", "main"), poly=[(round(x, 3), round(y, 3)) for x, y in p.exterior.coords[:-1]]))
    return rooms


def level_of(outline, walls, names, doors):
    """The ground floor as the catalogue's interior files have it."""
    inner = outline.buffer(-0.3, join_style=2)
    door_gaps = []
    wl = []
    for a, b, g in walls:
        wl.append([list(a), list(b)])
        if g is not None:
            p = (a[0] + (b[0] - a[0]) * g, a[1] + (b[1] - a[1]) * g)
            d = (b[0] - a[0], b[1] - a[1])
            l = math.hypot(*d)
            d = (d[0] / l * 0.5, d[1] / l * 0.5)
            door_gaps.append([[round(p[0] + d[0], 3), round(p[1] + d[1], 3)], [round(p[0] - d[0], 3), round(p[1] - d[1], 3)]])
    return dict(
        level=1,
        floor_z=0.1,
        floor_m2=round(inner.area, 1),
        outline=[(round(x, 3), round(y, 3)) for x, y in inner.exterior.coords[:-1]],
        rooms=rooms_of(outline, [(a, b) for a, b, _ in walls], names),
        doors=[],
        outside_doors=[dict(xy=list(xy), z=0.1, yaw=yaw) for xy, yaw in doors],
        walls=wl,
        door_gaps=door_gaps,
    )


def tier(poly, z0, z1, wall_top, scale=1.0, shift=(0.0, 0.0), name="A"):
    p = affinity.scale(poly, scale, scale if not isinstance(scale, tuple) else scale[1], origin=(0, 0)) if not isinstance(scale, tuple) else affinity.scale(poly, scale[0], scale[1], origin=(0, 0))
    p = affinity.translate(p, shift[0], shift[1])
    return dict(name=name, z0=z0, z1=z1, wall_top=wall_top, outline=resample(p))


# ---- The six (Laz, 15 Oct) ------------------------------------------------------

def designs():
    out = {}

    # 1. A tower cottage: a small plot grown upward.
    g = oval(3.9, 3.7, 2.3)
    walls = [((-1.4, 4.0), (-1.4, -4.0), 0.5)]
    out["roduro_tower_cottage_01"] = dict(
        label="Tower cottage, two storeys",
        kit="drum",
        tiers=[tier(g, 0.0, 3.5, 3.5), tier(g, 3.5, 6.6, 5.9, 0.82, (0.0, 0.0), "B")],
        level=level_of(g, walls, dict(main=(1.0, 0.0), store=(-2.6, 0.0)), [((3.9, 0.3), 0.0)]),
        stairs=[dict(xy=[0.8, 2.3], r=0.9, from_level=1, to_level=2, steps=18)],
        windows=[2, 2],
        roof="dome",
    )

    # 2. A tower house: three storeys on a middling plot.
    g = oval(6.5, 6.0, 2.2, lobes=(3, 0.07, 30))
    walls = [((0.3, -2.0), (2.0, -6.2), None), ((6.0, -0.4), (0.3, -2.0), 0.67), ((0.3, -2.0), (-5.0, -4.0), 0.32)]
    out["roduro_tower_house_01"] = dict(
        label="Tower house, three storeys",
        kit="drum",
        tiers=[tier(g, 0.0, 3.9, 3.9), tier(g, 3.9, 7.0, 6.4, 0.8, (-0.6, 0.0), "B"), tier(g, 7.0, 10.3, 9.0, 0.6, (-0.9, 0.3), "C")],
        level=level_of(g, walls, dict(hall=(-1.0, 2.5), kitchen=(3.5, -3.5), store=(-2.5, -4.5)), [((6.4, 1.1), 0.0), ((1.4, 5.5), 1.16)]),
        stairs=[dict(xy=[-0.2, 0.3], r=1.0, from_level=1, to_level=2, steps=20), dict(xy=[-0.2, 0.3], r=1.0, from_level=2, to_level=3, steps=16)],
        windows=[4, 4, 3],
        roof="dome",
    )

    # 3. A two-storey vault: the vault line grown a storey.
    g = oval(5.8, 4.3, 2.8)
    walls = [((-1.5, 4.5), (-1.5, -4.5), 0.5)]
    out["roduro_vault_tall_01"] = dict(
        label="Two-storey vault",
        kit="vault",
        tiers=[tier(g, 0.0, 3.4, 3.4), tier(g, 3.4, 6.9, 5.4, (0.86, 0.78), (0.0, 0.0), "B")],
        level=level_of(g, walls, dict(main=(1.5, 0.0), back_room=(-3.5, 0.0)), [((5.8, 0.4), 0.0)]),
        stairs=[dict(xy=[3.0, -2.5], r=0.9, from_level=1, to_level=2, steps=18)],
        windows=[2, 2],
        roof="barrel",
    )

    # 4. A twin home: two rounds grown into one, a hall and a bedroom.
    big = affinity.translate(oval(5.3, 5.1, 2.1), 0.9, 1.9)
    small = affinity.translate(oval(4.1, 3.9, 2.1), -2.8, -3.5)
    g = unary_union([big, small])
    # The wall across the waist, between the two rounds.
    ring = big.exterior.intersection(small.exterior)
    pts = [p for p in ring.geoms] if hasattr(ring, "geoms") else [ring]
    pts = sorted(pts, key=lambda p: p.x)
    waist = ((round(pts[0].x, 3), round(pts[0].y, 3)), (round(pts[-1].x, 3), round(pts[-1].y, 3)))
    walls = [(waist[0], waist[1], 0.55), ((-3.2, 5.6), (-1.2, 1.4), 0.6)]
    out["roduro_twin_home_01"] = dict(
        label="Twin home",
        kit="home",
        tiers=[tier(g, 0.0, 5.2, 3.3)],
        level=level_of(g, walls, dict(hall=(1.5, 2.5), bedroom=(-3.0, -4.0), store=(-3.0, 3.0)), [((6.2, 1.9), 0.0)]),
        stairs=[],
        windows=[3],
        roof="dome",
    )

    # 5. A long hall: a low vault with a great hearth, a kitchen at the far end.
    g = oval(10.0, 5.2, 3.0)
    walls = [((-6.0, 5.5), (-6.0, -5.5), 0.5)]
    out["roduro_long_hall_01"] = dict(
        label="Long hall",
        kit="vault",
        tiers=[tier(g, 0.0, 6.4, 4.2)],
        level=level_of(g, walls, dict(hall=(1.0, 0.0), kitchen=(-8.0, 0.0)), [((10.0, 0.0), 0.0), ((0.0, 5.2), 1.5708)]),
        stairs=[],
        windows=[5],
        roof="barrel",
    )

    # 6. A hillside great house: two storeys against the slope, the upper
    # floor stepped back toward the flat side.
    g = oval(7.5, 6.8, 2.4, flat=(180, 0.55))
    walls = [((0.5, -1.5), (1.8, -6.5), None), ((6.5, -0.5), (0.5, -1.5), 0.6), ((0.5, -1.5), (-4.1, -3.0), 0.35)]
    out["roduro_hill_great_01"] = dict(
        label="Hillside great house",
        kit="drum",
        tiers=[tier(g, 0.0, 3.6, 3.6), tier(g, 3.6, 6.9, 6.1, 0.75, (-1.2, 0.0), "B")],
        level=level_of(g, walls, dict(hall=(-1.0, 2.0), kitchen=(4.0, -3.5), store=(-2.5, -4.5)), [((7.3, 1.3), 0.0)]),
        stairs=[dict(xy=[-1.0, 1.2], r=1.0, from_level=1, to_level=2, steps=18)],
        windows=[4, 3],
        roof="dome",
    )
    return out


# ---- Trade homes: house and work yard as one grown form (Laz, 15 Oct) ---------
#
# Laz liked all five shapes (the yard wall is the house's own stone, lower):
#   A. one oval, the roofed end the house, the open end the yard;
#   B. a crescent house curled round its court;
#   C. the shed a low bud grown out of the house, the court in the crook;
#   D. one broad low footprint the house rises out of;
#   E. two lobes with the crease between them filled.
# Every upper tier sits well inside the one under it (nothing hangs off an
# edge). The main door faces +x; the gate faces +x too, except B's (side).

def smooth(p, r):
    return p.buffer(r, join_style=1).buffer(-r, join_style=1)


def turn(p, deg):
    return affinity.rotate(p, deg, origin=(0, 0))


# What stands in a trade's yard: the sim's pieces (stations, racks), then
# the dressing the window draws. `shed` pieces go under the roof.
TRADE_YARDS = {
    "smith": dict(
        shed=[("Forge", "wall"), ("Anvil", "middle"), ("Rack", "wall")],
        open=[("Workbench", "wall")],
        dress_shed=[("bellows", (1.9, 0.55, 1.0)), ("trough", (0.6, 1.3, 0.6)), ("block", (0.64, 0.64, 0.8))],
        dress_open=[("grindstone", (0.83, 0.9, 1.17)), ("barrel", (0.73, 0.75, 0.91)), ("crate", (0.9, 0.9, 0.7)), ("pile", (1.4, 1.2, 0.6)), ("bucket", (0.4, 0.4, 0.3)), ("lantern", (0.3, 0.3, 2.0))],
        chimney=True,
    ),
    "tanner": dict(
        shed=[("Workbench", "wall"), ("Rack", "wall")],
        open=[("Rack", "middle")],
        dress_shed=[("pile", (1.25, 1.0, 0.4)), ("frame", (0.13, 1.3, 1.5))],
        dress_open=[("tub", (1.4, 1.4, 1.1)), ("tub", (1.4, 1.4, 1.1)), ("tub", (1.4, 1.4, 1.1)), ("frame", (0.87, 1.7, 2.1)), ("frame", (0.87, 1.7, 2.1)), ("trough", (1.9, 0.65, 0.6)), ("barrel", (0.73, 0.75, 0.91)), ("bucket", (0.4, 0.4, 0.3)), ("lantern", (0.3, 0.3, 2.0))],
        chimney=False,
    ),
    "tender": dict(
        shed=[("Workbench", "wall"), ("Rack", "wall")],
        open=[("GrowerBed", "middle"), ("GrowerBed", "middle"), ("GrowerBed", "middle")],
        dress_shed=[("crate", (0.9, 0.9, 0.7)), ("sacks", (0.9, 0.7, 0.4))],
        dress_open=[("barrel", (0.73, 0.75, 0.91)), ("trough", (1.9, 0.65, 0.6)), ("pile", (1.2, 1.0, 0.5)), ("bucket", (0.4, 0.4, 0.3)), ("lantern", (0.3, 0.3, 2.0))],
        chimney=False,
    ),
}


def yard_designs():
    """The five trade homes: house outline, tiers, ground floor, and the yard
    (its outline round the open ground, the roofed shed, the gate, the chimney)."""
    out = {}

    # A. One oval, one end open: the smith. The house is the street end
    # (its door to the street), the yard behind it, the gate on the side.
    big = oval(9.6, 5.8, 2.3)
    house = affinity.translate(oval(5.6, 5.8, 2.3), 4.0, 0.0).intersection(big)
    yard = big.difference(house.buffer(-0.3))
    shed = affinity.translate(oval(2.3, 3.4, 2.2), -5.6, -1.2).intersection(big.buffer(-0.5))
    walls = [((1.0, 5.8), (1.0, 0.3), 0.5), ((1.0, 0.3), (1.0, -5.8), None)]
    out["roduro_smith_home_01"] = dict(
        label="Smith's home, one oval", kit="home", trade="smith",
        house=house, tiers=[tier(house, 0.0, 5.0, 3.4)],
        level=level_of(house, walls, dict(main=(4.5, 0.0), bedroom=(-0.4, 2.5), store=(-0.4, -2.5)), [((9.6, 0.3), 0.0)]),
        stairs=[], windows=[3], roof="dome",
        yard=yard, shed=shed, gate=(-4.0, 5.9), chimney=(-2.4, -2.2),
    )

    # B. A crescent house curled round its court: the tanner. The court
    # opens to +y (the gate on that side); the door is on the +x arm's
    # outside, to the street. (The house is thick enough at the bottom that
    # its frame's middle is indoors, which the sim counts on.)
    whole = oval(8.4, 9.0, 2.2)
    court_oval = affinity.translate(oval(5.0, 4.8, 2.2), 0.0, 4.6)
    opening = Polygon([(-4.6, 4.6), (4.6, 4.6), (4.6, 20), (-4.6, 20)])
    house = smooth(whole.difference(court_oval.union(opening)), 0.9)
    yard = whole.intersection(court_oval.union(opening)).difference(house.buffer(-0.3))
    shed = affinity.translate(oval(2.6, 2.2, 2.2), -2.4, 2.2).intersection(yard.buffer(0.6))
    # Walls across each arm of the crescent: a bedroom and a store in the
    # arms, the main room round the bottom.
    walls = [((4.9, 1.5), (9.2, 1.5), 0.5), ((-4.9, 1.5), (-9.2, 1.5), 0.5)]
    out["roduro_tanner_home_01"] = dict(
        label="Tanner's home, a crescent round its court", kit="home", trade="tanner",
        house=house, tiers=[tier(house, 0.0, 4.8, 3.3)],
        level=level_of(house, walls, dict(main=(0.0, -7.0), bedroom=(6.5, 3.5), store=(-6.5, 3.5)), [((6.0, -5.5), 0.0)]),
        stairs=[], windows=[4], roof="dome",
        yard=yard, shed=shed, gate=(0.0, 8.8), chimney=None,
    )

    # C. The shed a low bud grown out of the house, the court in the crook:
    # the tender. (The court on the -y side, so the door to the street is
    # clear of it.)
    house = oval(5.4, 5.2, 2.3)
    bud = affinity.translate(oval(3.4, 3.0, 2.2), 1.5, -6.6)
    mass = smooth(unary_union([house, bud]), 1.6)
    court = smooth(unary_union([mass, affinity.translate(oval(5.6, 4.6, 2.2), 6.2, -3.4)]), 2.0)
    yard = court.difference(house.buffer(-0.3))
    walls = [((-5.4, -1.6), (-0.4, -1.6), 0.55), ((-0.4, -1.6), (-0.4, -5.2), None)]
    out["roduro_tender_home_01"] = dict(
        label="Stone-tender's home, the shed a bud", kit="home", trade="tender",
        house=house, tiers=[tier(house, 0.0, 4.9, 3.3)],
        level=level_of(house, walls, dict(main=(1.5, 1.5), bedroom=(-3.0, -3.5), store=(1.5, -4.0)), [((5.4, 2.8), 0.0)]),
        stairs=[], windows=[3], roof="dome",
        yard=yard, shed=bud.intersection(court.buffer(-0.4)), gate=(11.6, -3.0), chimney=None,
    )

    # D. One broad low footprint the house rises out of: the smith, ancient.
    foot = oval(9.8, 7.2, 2.3, lobes=(3, 0.05, 20))
    house = affinity.translate(oval(5.3, 5.3, 2.3), -3.6, 0.4)
    assert foot.buffer(-0.3).contains(house)
    yard = foot.difference(house.buffer(-0.3))
    shed = affinity.translate(oval(2.4, 3.6, 2.2), 3.4, -2.4).intersection(foot.buffer(-0.5))
    walls = [((-3.6, 5.7), (-3.6, 1.0), 0.5), ((-3.6, 1.0), (-3.6, -4.9), 0.5)]
    out["roduro_smith_home_02"] = dict(
        label="Smith's home, risen from one footprint", kit="drum", trade="smith",
        house=house, tiers=[tier(house, 0.0, 3.5, 3.5), tier(house, 3.5, 6.6, 5.9, 0.8, (-0.2, 0.0), "B")],
        level=level_of(house, walls, dict(main=(-1.5, 0.0), bedroom=(-6.5, 3.0), store=(-6.5, -2.0)), [((1.7, 0.8), 0.0)]),
        stairs=[dict(xy=[-2.6, -2.4], r=0.9, from_level=1, to_level=2, steps=18)], windows=[3, 2], roof="dome",
        yard=yard, shed=shed, gate=(9.7, 1.2), chimney=(5.8, -5.5), base=1.3,
    )

    # E. Two lobes, the crease filled in: the tanner, established. The yard
    # lobe is on the -y side, so the door (to the street, on +x) is clear
    # of the yard's wall.
    house = oval(5.2, 5.4, 2.3)
    lobe = affinity.translate(oval(6.4, 5.0, 2.3), 5.4, -5.4)
    court = smooth(unary_union([house, lobe]), 2.0)
    yard = court.difference(house.buffer(-0.3))
    shed = affinity.translate(oval(2.4, 3.0, 2.2), 3.4, -8.2).intersection(court.buffer(-0.5))
    walls = [((-5.2, 2.8), (5.2, 2.8), 0.5), ((0.4, 2.8), (0.4, 5.4), None)]
    out["roduro_tanner_home_02"] = dict(
        label="Tanner's home, two lobes grown together", kit="home", trade="tanner",
        house=house, tiers=[tier(house, 0.0, 5.0, 3.4)],
        level=level_of(house, walls, dict(main=(0.0, -1.0), bedroom=(-2.5, 4.2), store=(3.0, 4.2)), [((5.2, 1.6), 0.0)]),
        stairs=[], windows=[3], roof="dome",
        yard=yard, shed=shed, gate=(11.6, -5.0), chimney=None,
    )
    return out
