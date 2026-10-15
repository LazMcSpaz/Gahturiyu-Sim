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


# ---- Work yards grown against a home (Laz, 15 Oct) ----------------------------

# What each of a yard model's stations is to the game: a piece of the sim's
# furniture (a station someone can work at, or a container), or dressing
# the window draws and nothing else reads.
STATION_KINDS = {
    "forge_hearth": ("Forge", None), "anvil": ("Anvil", None), "workbench": ("Workbench", None), "tool_rack": ("Rack", None),
    "charcoal_bin": (None, "crate"), "iron_stock": (None, "crate"), "quench_trough": (None, "trough"), "bellows": (None, "bellows"),
    "grindstone": (None, "grindstone"), "water_barrel": (None, "barrel"), "swage_block": (None, "block"), "weapon_rack": (None, "frame"),
    "cooling_rack": (None, "frame"),
    "fleshing_beam": ("Workbench", None), "drying_rack": ("Rack", None), "bark_pile": (None, "pile"), "hide_stack": (None, "pile"),
    "soak_vat": (None, "tub"), "stretching_frame": (None, "frame"), "water_trough": (None, "trough"), "knife_rack": (None, "frame"),
    "hide_line": (None, "frame"),
}
DRESSING_KINDS = {"bucket": "bucket", "sacks": "sacks", "stool": "stool", "lantern": "lantern", "shade_post": "post", "scrap_pile": "pile",
                  "cart_wheel": "wheel", "ash_patch": None}
HOLDERS = {"Chest", "Crate", "Cupboard", "Barrel"}


def yard_of(model, house, side_y, along_x, fence_h=1.3):
    """A yard model (its JSON) grown against the house's +y wall: its oval
    overlaps the house a little and the house's wall becomes that side of
    the fence. `side_y` is where the yard's middle sits across, `along_x`
    along the house. Returns the yard in the house's frame, metres."""
    oy, ox = side_y, along_x
    at = lambda p: (round(p[0] + ox, 3), round(p[1] + oy, 3))
    outline = affinity.translate(Polygon(model["outline"]), ox, oy)
    # The yard keeps only what's outside the house wall (by a hand).
    yard = outline.difference(house.buffer(-0.25))
    if yard.geom_type != "Polygon":
        yard = max(yard.geoms, key=lambda p: p.area)
    yard = yard.simplify(0.08)
    ring = yard.exterior if yard.exterior.is_ccw else LineString(list(yard.exterior.coords)[::-1])
    pts = [(round(x, 3), round(y, 3)) for x, y in ring.coords[:-1]]
    gate = at(model["gate"]["xy"])
    pieces, dressing = [], []
    for st in model["stations"]:
        furn, dress = STATION_KINDS.get(st["kind"], (None, None))
        p = at(st["xy"])
        if not Point(p).within(yard.buffer(-0.6)):
            continue
        most = 2 if furn == "Rack" else 1
        if furn and sum(1 for q in pieces if q["what"] == furn) < most:
            pieces.append(dict(what=furn, at=list(p), rot=round(st["yaw"], 4)))
        elif dress:
            dressing.append(dict(kind=dress, at=list(p), rot=round(st["yaw"], 4), size=st["size"]))
    for d in model["dressing"]:
        kind = DRESSING_KINDS.get(d["kind"])
        p = at(d["xy"])
        if kind and Point(p).within(yard.buffer(-0.3)):
            dressing.append(dict(kind=kind, at=list(p), rot=0.0, size=[0.5, 0.5, 0.5]))
    shades = [dict(cloth=s["cloth"], footprint=[list(at(p)) for p in s["footprint"]], height=s["lowest_z"]) for s in model["shades"]]
    canopy = model.get("canopy")
    chimney = model.get("chimney")
    # The roofed body (under the canopy) leans on the house: its columns.
    columns = [dict(at=list(at(c["xy"])), height=c["height"]) for c in model["columns"] if Point(at(c["xy"])).within(yard.buffer(0.3))]
    return dict(
        model=model["id"],
        outline=pts,
        gate=list(gate),
        gate_width=model["gate"]["width"],
        fence_h=fence_h,
        pieces=pieces,
        dressing=dressing,
        shades=shades,
        canopy=dict(at=list(at(canopy["xy"])), rx=canopy["rx"], ry=canopy["ry"], yaw=canopy["yaw"], height=canopy["underside_z"]) if canopy else None,
        chimney=dict(at=list(at(chimney["xy"])), r=chimney["r"], height=chimney["height"]) if chimney else None,
        columns=columns,
    )


def yard_homes(cat_dir, specs):
    """Trade homes: a catalogue home with a yard model grown against its
    +y side. `specs` holds the homes' specs (for their outlines)."""
    out = {}
    # (The yard models' plan files, transcribed into the repo: `yards/`.)
    here = os.path.dirname(os.path.abspath(__file__))
    forge = json.load(open(os.path.join(here, "yards", "roduro_forge_yard_01.json")))
    tannery = json.load(open(os.path.join(here, "yards", "roduro_tannery_yard_01.json")))
    for key, label, home_id, model, side_y, along_x in [
        ("roduro_smith_home_01", "Smith's home and yard", "roduro_round_est_01", forge, 9.6, -0.6),
        ("roduro_tanner_home_01", "Tanner's home and yard", "roduro_long_est_02", tannery, 10.6, -1.0),
    ]:
        home = specs[home_id]
        house = Polygon(home["exterior"]["tiers"][0]["outline"])
        yard = yard_of(model, house, side_y, along_x)
        out[key] = dict(home=home_id, label=label, yard=yard)
    return out
