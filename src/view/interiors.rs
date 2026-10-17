//! Placeholder buildings (playable-MVP step 6): every building variant in
//! `sim/layout.rs` drawn as simple shapes, outside and cut open inside.
//!
//! - **Outside** (`exterior`): Roduro grown stone is a rounded drum under a
//!   lumpy dome, sized by the variant's outline, with an upper bud for two
//!   storeys; Qotiro quarried stone is stepped boxes, one tier a storey
//!   (the island's dark stone, cut into blocks, never sandstone; a gold sun disc for the
//!   island hall); Horaro stilt homes stand on stone pillars with a deck.
//!   Workshops and shops show their trade outside (a forge's chimney, a
//!   tender's beds, a trader's awning and counter, a smiths' yard's
//!   chimney and furnace, market awnings).
//! - **Inside** (`interior`, when a squad member is in): the floor in the
//!   outline, the outer walls cut away low with a gap at the door, the inner
//!   walls (with their doorways) from `Door::wall_pieces`, and every piece
//!   of furniture as a few blocks.
//! - **Beds follow the household** (`sleepers`): more residents than the
//!   variant's beds and bedrolls means extra bedrolls (up to 8, 16 in
//!   quarters), and a Ṭaḍoro lodger gets a corner (`tadoro_corner`): a bed
//!   draped in faded violet and pale grey-blue cloth with a hanging flap, a
//!   rug, parchment pinned on the nearest wall, papers, an ink pot, candles
//!   and a leather satchel. Where every one of those stands is worked out in
//!   the sim (`layout::sleeping_plan`), so nothing overlaps; this only draws.
//! - **The floor is level** (`floor_height`): at the highest drawn ground
//!   under the outline, its slab reaching below the lowest, with steps up
//!   to the door; everything inside stands on it.
//! - **Containers** (`containers`) are drawn every frame, so a lid opens
//!   while someone goes through it and a picked lock loses its plate.
//!
//! Drawing only: where everything stands comes from the sim's layout.

use bevy::math::{vec3, Vec3};

use gahturiyu_sim::sim::{
    buildings::Door,
    containers::Owner,
    geo::V2,
    layout::{self, Furn, Holder, Shape, Variant},
    rng,
    settlement::{BuildingKind, Settlement},
    terrain::Terrain,
    World,
};

use super::app::Hover;
use super::cam::to3;
use super::mesh::Builder;
use super::palette::{self, Rgb};
use super::signs;
use super::stone;

/// Height of a Horaro stilt deck above what it stands on.
const DECK: f32 = 2.4;
/// How high the cut-away walls of a building someone is in stand, metres.
const CUT: f32 = 1.3;
/// How high the cut-away inner walls stand.
const CUT_INNER: f32 = 1.15;

const DOOR_DARK: Rgb = [0.08, 0.07, 0.06];
const WOOD: Rgb = palette::TIMBER;
const WOOD_LIGHT: Rgb = [0.50, 0.37, 0.24];
const WOOD_DARK: Rgb = [0.26, 0.19, 0.13];
const LINEN: Rgb = [0.80, 0.76, 0.66];
const IRON: Rgb = [0.22, 0.22, 0.24];
const STEEL: Rgb = [0.62, 0.64, 0.68];
const BRICK: Rgb = [0.56, 0.33, 0.23];
const SOIL: Rgb = [0.15, 0.11, 0.08];
const CLOTHS: [Rgb; 4] = [[0.58, 0.24, 0.20], [0.26, 0.36, 0.52], [0.44, 0.50, 0.28], [0.70, 0.56, 0.26]];
const DARK_STONE: Rgb = [0.27, 0.27, 0.29];

/// A building's own frame: layout units (`x` back -1 to front +1, `y`
/// across -1..1) scaled by its half depth and half width.
#[derive(Clone, Copy)]
struct Frame {
    c: V2,
    rot: f32,
    hx: f32,
    hy: f32,
    /// The height things are measured from.
    y: f32,
}

impl Frame {
    fn dir(&self) -> V2 {
        V2::new(self.rot.cos(), self.rot.sin())
    }
    fn side_v2(&self) -> V2 {
        V2::new(-self.rot.sin(), self.rot.cos())
    }
    /// A layout point on the map.
    fn p(&self, x: f32, y: f32) -> V2 {
        self.c.add(self.dir().scale(x * self.hx)).add(self.side_v2().scale(y * self.hy))
    }
    /// A layout point, `h` metres up.
    fn at(&self, x: f32, y: f32, h: f32) -> Vec3 {
        to3(self.p(x, y), self.y + h)
    }
    fn fwd(&self) -> Vec3 {
        vec3(self.rot.cos(), 0.0, self.rot.sin())
    }
    fn side(&self) -> Vec3 {
        vec3(-self.rot.sin(), 0.0, self.rot.cos())
    }
    /// The outward facing (radians) of the outline at layout point (x, y).
    fn normal_at(&self, x: f32, y: f32, round: bool) -> f32 {
        let n = if round {
            self.dir().scale(x / self.hx).add(self.side_v2().scale(y / self.hy))
        } else if x.abs() >= y.abs() {
            self.dir().scale(x.signum())
        } else {
            self.side_v2().scale(y.signum())
        };
        n.y.atan2(n.x)
    }
}

/// Where a variant's door is, in layout units, on its outer wall.
fn door_spot(v: &Variant) -> (f32, f32) {
    let across = v.door * 0.8;
    let front = match v.shape {
        Shape::Round => (1.0 - across * across).max(0.0).sqrt(),
        Shape::Poly(pts) => layout::outline_front(pts, 1.0, 1.0, across),
        Shape::Rect => 1.0,
    };
    (front, across)
}

// ---- Shapes the builder doesn't have ------------------------------------------

/// An upright elliptical drum (sides and top) turned `rot`, its top `taper`
/// times as wide as its foot.
#[allow(clippy::too_many_arguments)]
fn drum(b: &mut Builder, base: Vec3, rx: f32, rz: f32, h: f32, taper: f32, rot: f32, sides: usize, col: Rgb) {
    let (fx, fz) = (vec3(rot.cos(), 0.0, rot.sin()), vec3(-rot.sin(), 0.0, rot.cos()));
    let ring = |k: usize, s: f32, y: f32| {
        let a = k as f32 / sides as f32 * std::f32::consts::TAU;
        base + fx * (a.cos() * rx * s) + fz * (a.sin() * rz * s) + Vec3::Y * y
    };
    let c = palette::lin(col);
    for k in 0..sides {
        let mid = (k as f32 + 0.5) / sides as f32 * std::f32::consts::TAU;
        let n = (fx * (mid.cos() / rx) + fz * (mid.sin() / rz)).normalize_or_zero() + Vec3::Y * (1.0 - taper);
        let n = n.normalize_or_zero();
        b.quad_lin([ring(k, 1.0, 0.0), ring(k + 1, 1.0, 0.0), ring(k + 1, taper, h), ring(k, taper, h)], [n; 4], [c; 4]);
    }
    let top = base + Vec3::Y * h;
    for k in 0..sides {
        b.quad_lin([top, ring(k, taper, h), ring(k + 1, taper, h), top], [Vec3::Y; 4], [c; 4]);
    }
}

/// A squashed half-ball turned `rot`: `rx` along the facing, `rz` across.
/// `lump` roughens it (grown stone), `bands` darkens alternate rings.
#[allow(clippy::too_many_arguments)]
fn dome(b: &mut Builder, base: Vec3, rx: f32, rz: f32, h: f32, rot: f32, lump: f32, bands: f32, seed: u64, col: Rgb) {
    let (rings, sides) = (6usize, 18usize);
    let (fx, fz) = (vec3(rot.cos(), 0.0, rot.sin()), vec3(-rot.sin(), 0.0, rot.cos()));
    let noise = |i: usize, k: usize| -> f32 {
        let x = (seed ^ ((i as u64) << 20) ^ ((k % sides) as u64)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        ((x >> 40) as f32 / (1u64 << 24) as f32) - 0.5
    };
    let point = |i: usize, k: usize| -> (Vec3, Vec3) {
        let lat = i as f32 / rings as f32 * std::f32::consts::FRAC_PI_2;
        let lon = k as f32 / sides as f32 * std::f32::consts::TAU;
        let bump = if i == rings || i == 0 { 1.0 } else { 1.0 + noise(i, k) * lump };
        let p = base + fx * (lat.cos() * lon.cos() * rx * bump) + fz * (lat.cos() * lon.sin() * rz * bump) + Vec3::Y * (lat.sin() * h);
        let n = (fx * (lat.cos() * lon.cos() / rx) + fz * (lat.cos() * lon.sin() / rz) + Vec3::Y * (lat.sin() / h)).normalize_or_zero();
        (p, n)
    };
    for i in 0..rings {
        let band = if bands > 0.0 && i % 2 == 1 { 1.0 - bands } else { 1.0 };
        let c = palette::lin(palette::scale(col, band));
        for k in 0..sides {
            let q = [point(i, k), point(i, k + 1), point(i + 1, k + 1), point(i + 1, k)];
            b.quad_lin([q[0].0, q[1].0, q[2].0, q[3].0], [q[0].1, q[1].1, q[2].1, q[3].1], [c; 4]);
        }
    }
}

/// A box of any turn: its centre and three half-extent axes.
pub(super) fn obox(b: &mut Builder, c: Vec3, ax: Vec3, ay: Vec3, az: Vec3, col: Rgb) {
    for (axis, u, v) in [(ax, ay, az), (ay, az, ax), (az, ax, ay)] {
        for s in [-1.0f32, 1.0] {
            let n = (axis * s).normalize_or_zero();
            let f = c + axis * s;
            b.quad([f - u - v, f + u - v, f + u + v, f - u + v], n, col);
        }
    }
}

/// An upright disc (a sun disc) facing `facing` radians.
pub(super) fn disc(b: &mut Builder, c: Vec3, r: f32, facing: f32, col: Rgb) {
    let n = vec3(facing.cos(), 0.0, facing.sin());
    let right = vec3(-facing.sin(), 0.0, facing.cos());
    let k = 14;
    let lc = palette::lin(col);
    for i in 0..k {
        let (a0, a1) = (i as f32 / k as f32 * std::f32::consts::TAU, (i + 1) as f32 / k as f32 * std::f32::consts::TAU);
        let p0 = c + (right * a0.cos() + Vec3::Y * a0.sin()) * r;
        let p1 = c + (right * a1.cos() + Vec3::Y * a1.sin()) * r;
        b.quad_lin([c, p0, p1, c], [n; 4], [lc; 4]);
    }
}

/// A slab from `a` to `b` on the ground plane, `t` thick, standing `h` tall from `y`.
fn wall(b: &mut Builder, a: V2, c: V2, y: f32, t: f32, h: f32, col: Rgb) {
    let len = a.dist(c);
    if len < 0.05 {
        return;
    }
    let d = c.sub(a);
    b.block(to3(a.lerp(c, 0.5), y), len + 0.04, t, h, d.y.atan2(d.x), col);
}

// ---- A building with an outline of its own ---------------------------------------
//
// The sim knows only such a building's ground tier (`layout::Shape::Poly`);
// what stands on it (more tiers, a dome, a stair inside, a second door) is
// drawing only, here.

/// How a building with an outline of its own is drawn: its tiers, each its
/// own outline (layout units) from `z0` to `z1` metres at `at_size`, its
/// wall straight up to `wall_top` and rounding over above (the first is the
/// sim's outline), the top one ending in a roof `dome` of its height; a spiral stair inside (its middle in layout units, its reach in
/// metres) and more doors drawn shut (a spot inside the room, layout units,
/// and the way out from it, radians from the facing).
struct Form {
    key: &'static str,
    at_size: f32,
    tiers: &'static [(&'static [(f32, f32)], f32, f32, f32)],
    roof: Roof,
    dome: f32,
    stair: Option<((f32, f32), f32)>,
    doors: &'static [((f32, f32), f32)],
    /// Lit windows round each tier.
    windows: &'static [usize],
    /// How its work yard (`Variant::yard`) is dressed.
    yard: Option<YardLook>,
    /// A Qotiro building's outside stairs, upper doors and terrace props.
    qotiro: Option<QotiroLook>,
}

/// What stands on and against a Qotiro building (drawing only): each a
/// kind, a spot (layout units), a turn (radians in the building's frame),
/// a size (w × d × h metres) and the height it stands at.
struct QotiroLook {
    props: &'static [(QProp, (f32, f32), f32, (f32, f32, f32), f32)],
    /// Doors on the upper tiers: spot, facing, height.
    doors_up: &'static [((f32, f32), f32, f32)],
    /// Where the stairs come out on a terrace (the parapet is left out).
    landings: &'static [(f32, f32)],
}

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum QProp {
    /// Outside steps: `at` the foot, `rot` the way up, size (run, width, rise).
    Stair,
    /// The gold sun disc on a wall (`rot` the wall's facing).
    Disc,
    /// A cloth on four poles.
    Awning,
    /// A stone herb trough, planted.
    Trough,
    /// A cluster of clay jars.
    Jars,
    /// A bread oven, a small stone dome.
    Oven,
    /// The forge's chimney tower.
    Chimney,
    /// The shrine's open pavilion: columns, a slab, the sun above, a brazier.
    Pavilion,
    /// An arched opening in a wall with a lamp in it.
    Arch,
    /// A practice post.
    Post,
    /// An iron gate in a yard wall.
    Gate,
    /// An iron railing along a wall top.
    Railing,
}

/// A piece of yard dressing: drawn, never solid (`layout::Yard::pieces`
/// are the solid ones).
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum Dress {
    Bellows,
    Trough,
    Grindstone,
    Barrel,
    Block,
    Frame,
    Tub,
    Pile,
    Crate,
    Bucket,
    Sacks,
    Stool,
    Lantern,
    Post,
    Wheel,
    /// A stack of trimmed logs (the woodworker's).
    Logs,
    /// A small stone smoking house, its own smoke out of the top.
    Smokehouse,
    /// A rough block of quarried stone (the mason's).
    Stone,
}

/// How a trade home's yard is dressed: pieces (kind, spot in layout
/// units, turn, size w × d × h metres), awnings (footprint, height, cloth),
/// roof posts (spot, height), the shed's roof and a chimney (spot, radius,
/// height).
struct YardLook {
    dressing: &'static [(Dress, (f32, f32), f32, (f32, f32, f32))],
    shades: &'static [(&'static [(f32, f32)], f32, Rgb)],
    columns: &'static [((f32, f32), f32)],
    /// The shed's roof: its outline (layout units) and its underside height.
    canopy: Option<(&'static [(f32, f32)], f32)>,
    chimney: Option<((f32, f32), f32, f32)>,
}

const OCHRE: Rgb = [0.70, 0.56, 0.26];
const MADDER: Rgb = [0.58, 0.24, 0.20];
const INDIGO: Rgb = [0.26, 0.36, 0.52];

/// What a tier ends in.
#[derive(Clone, Copy, PartialEq)]
enum Roof {
    /// Drawn in toward its middle all round (grown stone).
    Dome,
    /// Drawn in across only: a barrel vault along the building's length.
    Barrel,
    /// A flat slab with a parapet (quarried Qotiro stone).
    Flat,
}

/// The catalogue's buildings (`claude/placeholder-shapes.md`; metres in
/// `assets/buildings/<key>.json`). Written by
/// `tools/buildings/roduro_forms.py rust`; edit the files, not these.
const FORMS: &[Form] = &[
    Form {
        key: "qotiro_drill_yard_01",
        at_size: 14.0,
        tiers: &[(layout::QOTIRO_DRILL_YARD_01_OUTLINE, 0.0, 3.6, 3.3), (QOTIRO_DRILL_YARD_01_TIER_1, 3.6, 7.2, 6.9)],
        roof: Roof::Flat,
        dome: 0.083,
        stair: None,
        doors: &[],
        windows: &[3, 2],
        yard: Some(YardLook { dressing: &[(Dress::Trough, (3.9337, -0.3429), -1.5752, (1.9, 0.65, 0.6)), (Dress::Barrel, (3.9337, 0.2571), -1.5752, (0.73, 0.75, 0.91)), (Dress::Frame, (4.0337, 0.5143), -0.0044, (0.13, 1.6, 2.0)), (Dress::Sacks, (1.3339, -0.8571), 3.1416, (0.9, 0.6, 0.5))], shades: &[], columns: &[((2.1999, 0.2143), 2.7), ((2.1999, 0.598), 2.7), ((2.1999, 0.9816), 2.7), ((1.3476, 1.0), 2.7), ((1.3046, 0.2143), 2.7)], canopy: Some((&[(2.1999, 0.2143), (2.1999, 1.0), (0.8999, 1.0), (0.8999, 0.2143)], 2.7)), chimney: None }),
        qotiro: Some(QotiroLook { props: &[(QProp::Disc, (0.9999, 0.2143), 0.0, (0.9, 0.1, 0.9), 2.3), (QProp::Awning, (0.6666, 0.7143), 0.0, (1.8, 3.4, 2.3), 3.6), (QProp::Post, (2.3332, -0.4286), 0.0, (0.35, 0.35, 2.0), 0.0), (QProp::Post, (3.1665, 0.0714), 0.0, (0.35, 0.35, 2.0), 0.0), (QProp::Post, (2.3332, 0.5), 0.0, (0.35, 0.35, 2.0), 0.0), (QProp::Gate, (4.333, 0.0), 0.0, (3.0, 0.1, 2.0), 0.0), (QProp::Stair, (1.2333, -1.0), 1.5708, (6.0, -1.4, 3.6), 0.0)], doors_up: &[((0.3333, 0.0714), 0.0, 3.6)], landings: &[(1.2333, -0.2429)] }),
    },
    Form {
        key: "qotiro_family_house_01",
        at_size: 14.0,
        tiers: &[(layout::QOTIRO_FAMILY_HOUSE_01_OUTLINE, 0.0, 3.6, 3.3), (QOTIRO_FAMILY_HOUSE_01_TIER_1, 3.6, 7.2, 6.9), (QOTIRO_FAMILY_HOUSE_01_TIER_2, 7.2, 10.4, 10.1)],
        roof: Roof::Flat,
        dome: 0.094,
        stair: None,
        doors: &[],
        windows: &[4, 3, 2],
        yard: None,
        qotiro: Some(QotiroLook { props: &[(QProp::Disc, (1.0, -0.04), 0.0, (0.9, 0.1, 0.9), 2.3), (QProp::Awning, (0.7143, 0.2), 0.0, (3.6, 4.4, 2.4), 3.6), (QProp::Trough, (0.9, -0.8601), 0.0, (1.4, 0.5, 0.5), 3.6), (QProp::Trough, (0.9, 0.8601), 0.0, (1.4, 0.5, 0.5), 3.6), (QProp::Trough, (-0.7857, -0.8601), 0.0, (1.4, 0.5, 0.5), 3.6), (QProp::Jars, (0.2286, 0.8401), 0.0, (0.9, 0.9, 0.8), 3.6), (QProp::Jars, (-0.6857, -0.36), 0.0, (0.9, 0.9, 0.8), 7.2), (QProp::Stair, (1.0, -1.1401), 3.1416, (6.0, 1.4, 3.6), 0.0), (QProp::Stair, (0.4286, -0.7401), 3.1416, (6.0, 1.4, 3.6), 3.6)], doors_up: &[((0.4286, -0.2), 0.0, 3.6), ((-0.4286, 0.2), 0.0, 7.2)], landings: &[(0.2429, -1.1401), (-0.3286, -0.7401)] }),
    },
    Form {
        key: "qotiro_forge_01",
        at_size: 10.0,
        tiers: &[(layout::QOTIRO_FORGE_01_OUTLINE, 0.0, 3.6, 3.3)],
        roof: Roof::Flat,
        dome: 0.083,
        stair: None,
        doors: &[],
        windows: &[2],
        yard: Some(YardLook { dressing: &[(Dress::Bellows, (1.356, -0.42), 1.5646, (0.9, 0.5, 0.9)), (Dress::Trough, (3.601, 0.76), -0.0087, (1.9, 0.65, 0.6)), (Dress::Grindstone, (1.501, -0.8), 3.1416, (0.9, 0.5, 1.0)), (Dress::Barrel, (3.901, 0.4), -1.577, (0.73, 0.75, 0.91)), (Dress::Pile, (2.851, -0.74), 3.1416, (1.3, 1.1, 0.5)), (Dress::Crate, (3.451, -0.2), -1.577, (0.9, 0.9, 0.7))], shades: &[], columns: &[((2.3, -0.64), 2.7), ((2.3, -0.1086), 2.7), ((2.3, 0.4228), 2.7), ((1.5145, 0.64), 2.7)], canopy: Some((&[(2.3, -0.64), (2.3, 0.64), (0.85, 0.64), (0.85, -0.64)], 2.7)), chimney: None }),
        qotiro: Some(QotiroLook { props: &[(QProp::Chimney, (0.0, -0.6), 0.0, (2.2, 2.2, 11.0), 0.0), (QProp::Gate, (4.5, 0.0), 0.0, (2.2, 0.1, 2.2), 0.0), (QProp::Railing, (4.5, -0.72), 1.5708, (2.8, 0.05, 0.8), 2.0), (QProp::Railing, (4.5, 0.72), 1.5708, (2.8, 0.05, 0.8), 2.0), (QProp::Railing, (1.75, -1.0), 0.0, (11.0, 0.05, 0.8), 2.0), (QProp::Railing, (1.75, 1.0), 0.0, (11.0, 0.05, 0.8), 2.0), (QProp::Disc, (1.0, 0.8), 0.0, (0.7, 0.1, 0.7), 2.3)], doors_up: &[], landings: &[] }),
    },
    Form {
        key: "qotiro_home_corner_01",
        at_size: 12.0,
        tiers: &[(layout::QOTIRO_HOME_CORNER_01_OUTLINE, 0.0, 3.6, 3.3), (QOTIRO_HOME_CORNER_01_TIER_1, 3.6, 7.2, 6.9), (QOTIRO_HOME_CORNER_01_TIER_2, 7.2, 10.1, 9.8)],
        roof: Roof::Flat,
        dome: 0.103,
        stair: None,
        doors: &[((-0.5833, 0.9999), 1.5708)],
        windows: &[4, 3, 1],
        yard: None,
        qotiro: Some(QotiroLook { props: &[(QProp::Disc, (1.0, 0.0), 0.0, (0.9, 0.1, 0.9), 2.3), (QProp::Awning, (0.5, 0.6), 0.0, (3.4, 3.4, 2.3), 3.6), (QProp::Awning, (-0.1667, -0.6), 0.0, (1.8, 3.4, 2.3), 7.2), (QProp::Trough, (0.8833, -0.8599), 0.0, (1.4, 0.5, 0.5), 3.6), (QProp::Trough, (0.25, 0.8599), 0.0, (1.4, 0.5, 0.5), 3.6), (QProp::Jars, (0.7667, -0.32), 0.0, (0.9, 0.9, 0.8), 3.6), (QProp::Jars, (-0.1667, 0.04), 0.0, (0.9, 0.9, 0.8), 7.2), (QProp::Stair, (1.0, 1.1399), 3.1416, (6.0, -1.4, 3.6), 0.0), (QProp::Stair, (-1.0, 0.34), 0.0, (6.0, 1.4, 3.6), 3.6)], doors_up: &[((0.0, -0.4), 0.0, 3.6), ((-0.3333, -0.6), 0.0, 7.2)], landings: &[(0.1167, 1.1399), (-0.1167, 0.34)] }),
    },
    Form {
        key: "qotiro_home_long_01",
        at_size: 20.0,
        tiers: &[(layout::QOTIRO_HOME_LONG_01_OUTLINE, 0.0, 3.6, 3.3), (QOTIRO_HOME_LONG_01_TIER_1, 3.6, 7.2, 6.9)],
        roof: Roof::Flat,
        dome: 0.083,
        stair: None,
        doors: &[],
        windows: &[6, 5],
        yard: None,
        qotiro: Some(QotiroLook { props: &[(QProp::Disc, (1.0, -0.16), 0.0, (0.9, 0.1, 0.9), 2.3), (QProp::Awning, (0.5, -0.35), 0.0, (3.2, 4.0, 2.3), 3.6), (QProp::Awning, (0.5, 0.55), 0.0, (3.2, 4.0, 2.3), 3.6), (QProp::Trough, (0.825, -0.85), 0.0, (1.4, 0.5, 0.5), 3.6), (QProp::Trough, (0.825, 0.08), 0.0, (1.4, 0.5, 0.5), 3.6), (QProp::Trough, (0.825, 0.86), 0.0, (1.4, 0.5, 0.5), 3.6), (QProp::Jars, (0.25, 0.92), 0.0, (0.9, 0.9, 0.8), 3.6), (QProp::Jars, (0.25, -0.08), 0.0, (0.9, 0.9, 0.8), 3.6), (QProp::Stair, (-1.0, -1.07), 0.0, (6.0, -1.4, 3.6), 0.0), (QProp::Stair, (-1.0, 1.07), 0.0, (6.0, 1.4, 3.6), 0.0)], doors_up: &[((0.0, 0.2), 0.0, 3.6), ((0.0, -0.6), 0.0, 3.6)], landings: &[(0.325, -1.07), (0.325, 1.07)] }),
    },
    Form {
        key: "qotiro_home_small_01",
        at_size: 6.0,
        tiers: &[(layout::QOTIRO_HOME_SMALL_01_OUTLINE, 0.0, 3.6, 3.3), (QOTIRO_HOME_SMALL_01_TIER_1, 3.6, 6.3, 6.0)],
        roof: Roof::Flat,
        dome: 0.111,
        stair: None,
        doors: &[],
        windows: &[2, 1],
        yard: None,
        qotiro: Some(QotiroLook { props: &[(QProp::Disc, (1.0, -0.4667), 0.0, (0.8, 0.1, 0.8), 2.2), (QProp::Awning, (0.6667, -0.5333), 0.0, (1.8, 2.6, 2.3), 3.6), (QProp::Trough, (0.8, 0.7), 0.0, (1.0, 0.5, 0.5), 3.6), (QProp::Jars, (-0.7333, -0.7333), 0.0, (0.9, 0.9, 0.8), 3.6), (QProp::Stair, (1.0, -1.2333), 3.1416, (6.0, 1.4, 3.6), 0.0)], doors_up: &[((0.3333, 0.3333), 0.0, 3.6)], landings: &[(-0.7667, -1.2333)] }),
    },
    Form {
        key: "qotiro_home_small_end_01",
        at_size: 8.0,
        tiers: &[(layout::QOTIRO_HOME_SMALL_END_01_OUTLINE, 0.0, 3.6, 3.3), (QOTIRO_HOME_SMALL_END_01_TIER_1, 3.6, 6.5, 6.2)],
        roof: Roof::Flat,
        dome: 0.103,
        stair: None,
        doors: &[],
        windows: &[3, 2],
        yard: None,
        qotiro: Some(QotiroLook { props: &[(QProp::Disc, (1.0, -0.5), 0.0, (0.8, 0.1, 0.8), 2.2), (QProp::Awning, (0.5, 0.2), 0.0, (3.2, 4.0, 2.3), 3.6), (QProp::Trough, (0.85, -0.7667), 0.0, (1.4, 0.5, 0.5), 3.6), (QProp::Trough, (0.85, 0.8), 0.0, (1.0, 0.5, 0.5), 3.6), (QProp::Jars, (0.2, -0.8), 0.0, (0.9, 0.9, 0.8), 3.6), (QProp::Stair, (-1.0, -1.2333), 0.0, (6.0, -1.4, 3.6), 0.0)], doors_up: &[((0.0, -0.2667), 0.0, 3.6)], landings: &[(0.325, -1.2333)] }),
    },
    Form {
        key: "qotiro_home_small_low_01",
        at_size: 8.0,
        tiers: &[(layout::QOTIRO_HOME_SMALL_LOW_01_OUTLINE, 0.0, 3.6, 3.3), (QOTIRO_HOME_SMALL_LOW_01_TIER_1, 3.6, 5.5, 5.3)],
        roof: Roof::Flat,
        dome: 0.105,
        stair: None,
        doors: &[],
        windows: &[3, 0],
        yard: None,
        qotiro: Some(QotiroLook { props: &[(QProp::Disc, (1.0, -0.0667), 0.0, (0.8, 0.1, 0.8), 2.2), (QProp::Awning, (0.3, 0.2667), 0.0, (3.6, 3.2, 2.3), 3.6), (QProp::Oven, (-0.65, 0.6667), 0.0, (1.2, 1.2, 1.3), 3.6), (QProp::Trough, (0.825, -0.7667), 0.0, (1.2, 0.5, 0.5), 3.6), (QProp::Trough, (0.825, 0.8), 0.0, (1.2, 0.5, 0.5), 3.6), (QProp::Jars, (-0.15, -0.7667), 0.0, (0.9, 0.9, 0.8), 3.6), (QProp::Stair, (1.0, -1.2333), 3.1416, (6.0, 1.4, 3.6), 0.0)], doors_up: &[((-0.5, -0.6667), 0.0, 3.6)], landings: &[(-0.325, -1.2333)] }),
    },
    Form {
        key: "qotiro_home_small_tall_01",
        at_size: 6.0,
        tiers: &[(layout::QOTIRO_HOME_SMALL_TALL_01_OUTLINE, 0.0, 3.6, 3.3), (QOTIRO_HOME_SMALL_TALL_01_TIER_1, 3.6, 7.2, 6.9), (QOTIRO_HOME_SMALL_TALL_01_TIER_2, 7.2, 10.4, 10.1)],
        roof: Roof::Flat,
        dome: 0.094,
        stair: None,
        doors: &[],
        windows: &[2, 2, 2],
        yard: None,
        qotiro: Some(QotiroLook { props: &[(QProp::Disc, (1.0, -0.4667), 0.0, (0.8, 0.1, 0.8), 2.2), (QProp::Awning, (0.6667, 0.4), 0.0, (1.8, 3.0, 2.3), 3.6), (QProp::Trough, (0.8, -0.8), 0.0, (1.0, 0.5, 0.5), 3.6), (QProp::Jars, (-0.7333, -0.7333), 0.0, (0.9, 0.9, 0.8), 7.2), (QProp::Stair, (-1.0, -1.2333), 0.0, (6.0, -1.4, 3.6), 0.0), (QProp::Stair, (0.5667, 1.0), -1.5708, (6.0, 1.4, 3.6), 3.6)], doors_up: &[((0.3333, 0.5), 0.0, 3.6), ((-0.3333, -0.3333), -1.5708, 7.2)], landings: &[(0.7667, -1.2333), (0.5667, -0.7667)] }),
    },
    Form {
        key: "qotiro_shrine_hall_01",
        at_size: 12.0,
        tiers: &[(layout::QOTIRO_SHRINE_HALL_01_OUTLINE, 0.0, 3.2, 2.9)],
        roof: Roof::Flat,
        dome: 0.094,
        stair: None,
        doors: &[],
        windows: &[0],
        yard: None,
        qotiro: Some(QotiroLook { props: &[(QProp::Arch, (1.0, -0.6667), 0.0, (1.6, 0.1, 2.4), 0.0), (QProp::Arch, (1.0, -0.3333), 0.0, (1.6, 0.1, 2.4), 0.0), (QProp::Arch, (1.0, 0.3333), 0.0, (1.6, 0.1, 2.4), 0.0), (QProp::Arch, (1.0, 0.6667), 0.0, (1.6, 0.1, 2.4), 0.0), (QProp::Pavilion, (-0.25, 0.0), 0.0, (4.0, 4.0, 3.4), 3.2), (QProp::Jars, (0.5, -0.7667), 0.0, (0.9, 0.9, 0.8), 3.2), (QProp::Stair, (1.7667, 0.0), 3.1416, (4.6, 2.4, 3.2), 0.0)], doors_up: &[], landings: &[(1.1167, 0.0)] }),
    },
    Form {
        key: "roduro_binder_home_01",
        at_size: 14.4,
        tiers: &[(layout::RODURO_BINDER_HOME_01_OUTLINE, 0.0, 4.9, 3.2)],
        roof: Roof::Dome,
        dome: 0.347,
        stair: None,
        doors: &[],
        windows: &[4],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_fletcher_home_01",
        at_size: 11.2,
        tiers: &[(layout::RODURO_FLETCHER_HOME_01_OUTLINE, 0.0, 4.9, 3.3)],
        roof: Roof::Dome,
        dome: 0.327,
        stair: None,
        doors: &[],
        windows: &[3],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_hill_est_01",
        at_size: 12.15,
        tiers: &[(layout::RODURO_HILL_EST_01_OUTLINE, 0.0, 4.98, 4.98)],
        roof: Roof::Dome,
        dome: 0.0,
        stair: None,
        doors: &[],
        windows: &[3],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_hill_great_01",
        at_size: 13.58,
        tiers: &[(layout::RODURO_HILL_GREAT_01_OUTLINE, 0.0, 3.6, 3.6), (RODURO_HILL_GREAT_01_TIER_1, 3.6, 6.9, 6.1)],
        roof: Roof::Dome,
        dome: 0.242,
        stair: Some(((-0.4617, 0.1767), 1.0)),
        doors: &[],
        windows: &[4, 3],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_hill_old_01",
        at_size: 15.35,
        tiers: &[(layout::RODURO_HILL_OLD_01_OUTLINE, 0.0, 4.0, 4.0), (RODURO_HILL_OLD_01_TIER_1, 4.0, 6.5, 6.25)],
        roof: Roof::Dome,
        dome: 0.1,
        stair: Some(((-0.262, 0.0349), 1.0)),
        doors: &[],
        windows: &[3, 2],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_hill_young_01",
        at_size: 8.06,
        tiers: &[(layout::RODURO_HILL_YOUNG_01_OUTLINE, 0.0, 4.91, 4.75)],
        roof: Roof::Dome,
        dome: 0.033,
        stair: None,
        doors: &[],
        windows: &[2],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_long_ancient_01",
        at_size: 19.46,
        tiers: &[(layout::RODURO_LONG_ANCIENT_01_OUTLINE, 0.0, 4.25, 3.0), (RODURO_LONG_ANCIENT_01_TIER_1, 4.25, 6.75, 6.75), (RODURO_LONG_ANCIENT_01_TIER_2, 6.75, 11.36, 11.0)],
        roof: Roof::Dome,
        dome: 0.078,
        stair: Some(((-0.1326, 0.1818), 1.0)),
        doors: &[((-0.6866, 0.741), 2.4102)],
        windows: &[5, 4, 3],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_long_est_01",
        at_size: 13.38,
        tiers: &[(layout::RODURO_LONG_EST_01_OUTLINE, 0.0, 4.93, 3.25)],
        roof: Roof::Barrel,
        dome: 0.341,
        stair: None,
        doors: &[((-0.8504, -0.0964), -3.1335)],
        windows: &[2],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_long_est_02",
        at_size: 11.9,
        tiers: &[(layout::RODURO_LONG_EST_02_OUTLINE, 0.0, 4.95, 4.95)],
        roof: Roof::Dome,
        dome: 0.0,
        stair: None,
        doors: &[],
        windows: &[3],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_long_hall_01",
        at_size: 20.0,
        tiers: &[(layout::RODURO_LONG_HALL_01_OUTLINE, 0.0, 6.4, 4.2)],
        roof: Roof::Barrel,
        dome: 0.344,
        stair: None,
        doors: &[((0.0, 1.0), 1.5708)],
        windows: &[5],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_long_old_01",
        at_size: 16.36,
        tiers: &[(layout::RODURO_LONG_OLD_01_OUTLINE, 0.0, 4.0, 3.75), (RODURO_LONG_OLD_01_TIER_1, 4.0, 6.51, 6.0)],
        roof: Roof::Dome,
        dome: 0.203,
        stair: Some(((0.0628, 0.1696), 1.0)),
        doors: &[],
        windows: &[4, 2],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_long_young_01",
        at_size: 7.49,
        tiers: &[(layout::RODURO_LONG_YOUNG_01_OUTLINE, 0.0, 4.78, 3.25)],
        roof: Roof::Barrel,
        dome: 0.32,
        stair: None,
        doors: &[],
        windows: &[1],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_mason_home_01",
        at_size: 17.62,
        tiers: &[(layout::RODURO_MASON_HOME_01_OUTLINE, 0.0, 4.8, 3.3)],
        roof: Roof::Dome,
        dome: 0.312,
        stair: None,
        doors: &[],
        windows: &[4],
        yard: Some(YardLook { dressing: &[(Dress::Block, (-0.2583, 0.4742), 2.18, (0.7, 0.7, 0.7)), (Dress::Stone, (0.4369, 0.6306), -1.5975, (1.6, 1.2, 1.0)), (Dress::Stone, (0.3347, 0.9762), -0.3739, (1.3, 1.0, 0.8)), (Dress::Stone, (-0.4144, 0.8379), 1.2574, (1.1, 1.1, 1.3)), (Dress::Pile, (0.3347, 0.354), -2.3483, (1.4, 1.2, 0.6)), (Dress::Trough, (-0.2442, 0.9416), 1.8169, (1.9, 0.65, 0.6)), (Dress::Barrel, (0.1645, 0.2158), -2.8393, (0.73, 0.75, 0.91)), (Dress::Bucket, (-0.4825, 0.6651), 1.5787, (0.4, 0.4, 0.3)), (Dress::Lantern, (0.1645, 1.0453), -0.1807, (0.3, 0.3, 2.0))], shades: &[], columns: &[((-0.4641, 0.5812), 2.7), ((-0.1689, 0.6048), 2.7), ((0.0209, 0.3903), 2.7), ((-0.1023, 0.1305), 2.7)], canopy: Some((&[(-0.5961, 0.3263), (-0.5865, 0.435), (-0.5369, 0.5247), (-0.4493, 0.5886), (-0.3377, 0.6175), (-0.1994, 0.6122), (-0.0893, 0.569), (-0.0119, 0.49), (0.0212, 0.3867), (0.0121, 0.2724), (-0.0375, 0.1827), (-0.1251, 0.1188), (-0.2367, 0.0899), (-0.3973, 0.1014), (-0.5144, 0.2035)], 2.7)), chimney: None }),
        qotiro: None,
    },
    Form {
        key: "roduro_round_ancient_01",
        at_size: 17.84,
        tiers: &[(layout::RODURO_ROUND_ANCIENT_01_OUTLINE, 0.0, 4.25, 4.0), (RODURO_ROUND_ANCIENT_01_TIER_1, 4.25, 6.75, 6.75), (RODURO_ROUND_ANCIENT_01_TIER_2, 6.75, 11.39, 9.75)],
        roof: Roof::Dome,
        dome: 0.353,
        stair: Some(((-0.0028, 0.036), 1.0)),
        doors: &[((0.2377, 0.6817), 1.1654)],
        windows: &[4, 4, 3],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_round_est_01",
        at_size: 10.3,
        tiers: &[(layout::RODURO_ROUND_EST_01_OUTLINE, 0.0, 4.94, 4.94)],
        roof: Roof::Dome,
        dome: 0.0,
        stair: None,
        doors: &[],
        windows: &[3],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_round_est_02",
        at_size: 9.57,
        tiers: &[(layout::RODURO_ROUND_EST_02_OUTLINE, 0.0, 4.0, 4.0), (RODURO_ROUND_EST_02_TIER_1, 4.0, 6.46, 6.0)],
        roof: Roof::Dome,
        dome: 0.187,
        stair: Some(((0.0083, 0.1034), 1.0)),
        doors: &[],
        windows: &[2, 1],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_round_old_01",
        at_size: 13.81,
        tiers: &[(layout::RODURO_ROUND_OLD_01_OUTLINE, 0.0, 4.25, 4.0), (RODURO_ROUND_OLD_01_TIER_1, 4.25, 6.73, 6.25)],
        roof: Roof::Dome,
        dome: 0.194,
        stair: Some(((0.0188, 0.0571), 1.0)),
        doors: &[((0.2049, 0.7682), 1.4124)],
        windows: &[3, 3],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_round_young_01",
        at_size: 7.04,
        tiers: &[(layout::RODURO_ROUND_YOUNG_01_OUTLINE, 0.0, 4.97, 4.97)],
        roof: Roof::Dome,
        dome: 0.0,
        stair: None,
        doors: &[],
        windows: &[2],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_round_young_02",
        at_size: 7.61,
        tiers: &[(layout::RODURO_ROUND_YOUNG_02_OUTLINE, 0.0, 4.92, 4.92)],
        roof: Roof::Dome,
        dome: 0.0,
        stair: None,
        doors: &[],
        windows: &[1],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_smith_home_01",
        at_size: 11.3,
        tiers: &[(layout::RODURO_SMITH_HOME_01_OUTLINE, 0.0, 5.0, 3.4)],
        roof: Roof::Dome,
        dome: 0.32,
        stair: None,
        doors: &[],
        windows: &[3],
        yard: Some(YardLook { dressing: &[(Dress::Bellows, (-1.4999, -0.0788), -1.4866, (1.9, 0.55, 1.0)), (Dress::Trough, (-1.8213, 0.1336), 0.0, (0.6, 1.3, 0.6)), (Dress::Grindstone, (-1.7663, 0.6434), 2.0179, (0.83, 0.9, 1.17)), (Dress::Barrel, (-1.1235, 0.2717), -1.6667, (0.73, 0.75, 0.91)), (Dress::Crate, (-1.0164, 0.6434), -2.0708, (0.9, 0.9, 0.7)), (Dress::Pile, (-1.2307, -0.7372), 2.9154, (1.4, 1.2, 0.6)), (Dress::Bucket, (-2.1949, 0.3248), 1.0449, (0.4, 0.4, 0.3)), (Dress::Lantern, (-2.2485, -0.2062), 1.8005, (0.3, 0.3, 2.0))], shades: &[], columns: &[((-2.0392, -0.6067), 2.7), ((-2.1242, -0.1589), 2.7), ((-1.9826, 0.2712), 2.7), ((-1.5596, 0.36), 2.7), ((-1.3192, -0.0154), 2.7), ((-1.3355, -0.4727), 2.7), ((-1.6281, -0.8018), 2.7)], canopy: Some((&[(-2.0392, -0.6067), (-2.1015, -0.4395), (-2.1247, -0.2303), (-2.1081, -0.0099), (-2.0481, 0.172), (-1.9615, 0.2936), (-1.8488, 0.3688), (-1.7037, 0.3929), (-1.5603, 0.3604), (-1.4094, 0.2227), (-1.3203, -0.0099), (-1.3092, -0.3315), (-1.3803, -0.5897), (-1.4981, -0.7389), (-1.658, -0.8071), (-1.8688, -0.7177)], 2.7)), chimney: Some(((-1.1428, -0.3858), 0.7, 6.0)) }),
        qotiro: None,
    },
    Form {
        key: "roduro_smith_home_02",
        at_size: 10.6,
        tiers: &[(layout::RODURO_SMITH_HOME_02_OUTLINE, 0.0, 3.5, 3.5), (RODURO_SMITH_HOME_02_TIER_1, 3.5, 6.6, 5.9)],
        roof: Roof::Dome,
        dome: 0.226,
        stair: Some(((0.1887, -0.5283), 0.9)),
        doors: &[],
        windows: &[3, 2],
        yard: Some(YardLook { dressing: &[(Dress::Bellows, (1.2455, -0.2642), -2.6728, (1.9, 0.55, 1.0)), (Dress::Trough, (1.0757, -0.6604), -2.8161, (0.6, 1.3, 0.6)), (Dress::Grindstone, (2.3211, -0.2266), -0.2427, (0.83, 0.9, 1.17)), (Dress::Barrel, (1.6985, 0.9055), -0.3101, (0.73, 0.75, 0.91)), (Dress::Crate, (0.8494, -1.2455), -2.9756, (0.9, 0.9, 0.7)), (Dress::Pile, (0.906, 0.9621), -0.0632, (1.4, 1.2, 0.6)), (Dress::Bucket, (1.1891, 1.0187), -0.0632, (0.4, 0.4, 0.3)), (Dress::Lantern, (1.8683, -0.9058), -2.5021, (0.3, 0.3, 2.0))], shades: &[], columns: &[((1.7736, -0.5362), 2.7), ((1.6445, -1.0338), 2.7), ((1.1849, -1.1853), 2.7), ((0.8919, -0.7798), 2.7), ((1.1998, 0.1338), 2.7), ((1.6534, -0.0358), 2.7)], canopy: Some((&[(1.7736, -0.5362), (1.7257, -0.8681), (1.5877, -1.1009), (1.4926, -1.1696), (1.3798, -1.204), (1.2383, -1.2002), (1.1281, -1.1583), (0.9655, -0.983), (0.8791, -0.7068), (0.8802, -0.3413), (0.9764, -0.0545), (1.0798, 0.0679), (1.2038, 0.1351), (1.3681, 0.1489), (1.5134, 0.1017), (1.6266, 0.0013), (1.7081, -0.1409), (1.7587, -0.3242)], 2.7)), chimney: Some(((1.7736, -1.1132), 0.7, 6.0)) }),
        qotiro: None,
    },
    Form {
        key: "roduro_smoker_home_01",
        at_size: 9.54,
        tiers: &[(layout::RODURO_SMOKER_HOME_01_OUTLINE, 0.0, 3.5, 3.5), (RODURO_SMOKER_HOME_01_TIER_1, 3.5, 6.6, 5.9)],
        roof: Roof::Dome,
        dome: 0.226,
        stair: Some(((0.1887, -0.5283), 0.9)),
        doors: &[],
        windows: &[3, 2],
        yard: Some(YardLook { dressing: &[(Dress::Frame, (1.0987, -0.7883), 0.0, (0.13, 1.6, 2.0)), (Dress::Logs, (0.5461, -1.2182), 3.1339, (2.4, 1.0, 0.9)), (Dress::Tub, (1.9298, 0.0396), -1.6328, (1.3, 1.3, 1.0)), (Dress::Trough, (0.8606, 0.983), -0.0632, (1.9, 0.65, 0.6)), (Dress::Barrel, (2.2442, -0.4006), -0.4807, (0.73, 0.75, 0.91)), (Dress::Barrel, (1.9298, -0.778), -2.5021, (0.73, 0.75, 0.91)), (Dress::Bucket, (1.9927, 0.7943), -0.4957, (0.4, 0.4, 0.3)), (Dress::Lantern, (2.3071, 0.4799), -1.1176, (0.3, 0.3, 2.0))], shades: &[], columns: &[((1.7736, -0.5363), 2.7), ((1.5885, -1.1002), 2.7), ((1.0419, -1.0889), 2.7), ((1.5996, 0.0323), 2.7)], canopy: Some((&[(1.7736, -0.5363), (1.7258, -0.8681), (1.5878, -1.101), (1.3799, -1.204), (1.2382, -1.2002), (1.1281, -1.1583), (0.9654, -0.983), (0.879, -0.7067), (0.8801, -0.3413), (0.9765, -0.0545), (1.0799, 0.0679), (1.2038, 0.135), (1.3681, 0.1488), (1.5134, 0.1017), (1.6266, 0.0013), (1.7082, -0.1409)], 2.7)), chimney: None }),
        qotiro: None,
    },
    Form {
        key: "roduro_tanner_home_01",
        at_size: 16.79,
        tiers: &[(layout::RODURO_TANNER_HOME_01_OUTLINE, 0.0, 4.8, 3.3)],
        roof: Roof::Dome,
        dome: 0.312,
        stair: None,
        doors: &[],
        windows: &[4],
        yard: Some(YardLook { dressing: &[(Dress::Frame, (-0.1346, 0.384), 4.36, (0.13, 1.3, 1.5)), (Dress::Tub, (0.0599, 0.8756), -0.0512, (1.4, 1.4, 1.1)), (Dress::Tub, (0.1671, 0.5129), 0.0, (1.4, 1.4, 1.1)), (Dress::Tub, (0.3101, 0.7668), -1.3286, (1.4, 1.4, 1.1)), (Dress::Frame, (-0.4046, 0.8756), 3.1394, (0.87, 1.7, 2.1)), (Dress::Frame, (0.4173, 0.4766), -0.298, (0.87, 1.7, 2.1)), (Dress::Trough, (0.2029, 0.259), -2.7799, (1.9, 0.65, 0.6)), (Dress::Barrel, (-0.226, 1.0207), 0.2461, (0.73, 0.75, 0.91)), (Dress::Bucket, (0.3101, 1.0207), -0.3331, (0.4, 0.4, 0.3)), (Dress::Lantern, (0.4887, 0.658), -1.5975, (0.3, 0.3, 2.0))], shades: &[], columns: &[((-0.4119, 0.6019), 2.7), ((-0.0649, 0.5509), 2.7), ((-0.012, 0.2158), 2.7)], canopy: Some((&[(-0.5948, 0.3202), (-0.587, 0.4291), (-0.5396, 0.5199), (-0.4575, 0.5835), (-0.3463, 0.6158), (-0.2035, 0.6123), (-0.0932, 0.571), (-0.0142, 0.4939), (0.0212, 0.3914), (0.0155, 0.2823), (-0.0299, 0.1903), (-0.1154, 0.1226), (-0.2267, 0.0903), (-0.4044, 0.102), (-0.5172, 0.2017)], 2.7)), chimney: None }),
        qotiro: None,
    },
    Form {
        key: "roduro_tanner_home_02",
        at_size: 10.8,
        tiers: &[(layout::RODURO_TANNER_HOME_02_OUTLINE, 0.0, 5.0, 3.4)],
        roof: Roof::Dome,
        dome: 0.32,
        stair: None,
        doors: &[],
        windows: &[3],
        yard: Some(YardLook { dressing: &[(Dress::Frame, (0.6346, -1.407), 4.4315, (0.13, 1.3, 1.5)), (Dress::Tub, (1.1855, -0.7937), 1.0151, (1.4, 1.4, 1.1)), (Dress::Tub, (1.6471, -1.1826), -2.0273, (1.4, 1.4, 1.1)), (Dress::Tub, (1.7624, -0.7381), -1.0157, (1.4, 1.4, 1.1)), (Dress::Frame, (1.2432, -0.2937), 1.3323, (0.87, 1.7, 2.1)), (Dress::Frame, (1.5317, -1.627), -1.3106, (0.87, 1.7, 2.1)), (Dress::Trough, (1.9355, -1.3493), -2.0571, (1.9, 0.65, 0.6)), (Dress::Barrel, (0.0317, -1.1826), 1.856, (0.73, 0.75, 0.91)), (Dress::Bucket, (1.6471, -0.3493), -0.5141, (0.4, 0.4, 0.3)), (Dress::Lantern, (0.9548, -0.627), 1.0772, (0.3, 0.3, 2.0))], shades: &[], columns: &[((0.1965, -1.612), 2.7), ((0.291, -1.1478), 2.7), ((0.7248, -0.967), 2.7), ((1.078, -1.2709), 2.7), ((1.0846, -1.7467), 2.7), ((0.6477, -1.8022), 2.7)], canopy: Some((&[(0.1965, -1.612), (0.1983, -1.4091), (0.2411, -1.2404), (0.3273, -1.1013), (0.45, -1.0069), (0.6136, -0.9641), (0.8025, -0.9843), (0.9394, -1.0609), (1.0436, -1.1922), (1.0971, -1.3374), (1.1153, -1.503), (1.1005, -1.6837), (1.0496, -1.8333), (0.5596, -1.783), (0.3446, -1.7065)], 2.7)), chimney: None }),
        qotiro: None,
    },
    Form {
        key: "roduro_tender_home_01",
        at_size: 10.8,
        tiers: &[(layout::RODURO_TENDER_HOME_01_OUTLINE, 0.0, 4.9, 3.3)],
        roof: Roof::Dome,
        dome: 0.327,
        stair: None,
        doors: &[],
        windows: &[3],
        yard: Some(YardLook { dressing: &[(Dress::Crate, (-0.0952, -1.2692), 0.0, (0.9, 0.9, 0.7)), (Dress::Sacks, (0.627, -1.4423), -2.4625, (0.9, 0.7, 0.4)), (Dress::Barrel, (1.2228, 0.0235), 1.4089, (0.73, 0.75, 0.91)), (Dress::Trough, (1.3339, -1.3034), -3.0455, (1.9, 0.65, 0.6)), (Dress::Pile, (1.5561, -0.0919), -0.4227, (1.2, 1.0, 0.5)), (Dress::Bucket, (1.6672, -1.2457), -2.6037, (0.4, 0.4, 0.3)), (Dress::Lantern, (1.8894, -1.0727), -2.3394, (0.3, 0.3, 2.0))], shades: &[], columns: &[((-0.3107, -1.0346), 2.7), ((0.8985, -1.1501), 2.7), ((0.7217, -1.5988), 2.7), ((0.2483, -1.7688), 2.7), ((-0.2024, -1.5505), 2.7)], canopy: Some((&[(-0.3107, -1.0346), (-0.1852, -0.8513), (0.0144, -0.7325), (0.2874, -0.6923), (0.5507, -0.7359), (0.7407, -0.8513), (0.8619, -1.0242), (0.9072, -1.2463), (0.8743, -1.4824), (0.6324, -1.6761), (0.4843, -1.7436), (0.2346, -1.7684), (0.0226, -1.7272), (-0.128, -1.6369), (-0.2302, -1.5017)], 2.7)), chimney: None }),
        qotiro: None,
    },
    Form {
        key: "roduro_tower_cottage_01",
        at_size: 7.8,
        tiers: &[(layout::RODURO_TOWER_COTTAGE_01_OUTLINE, 0.0, 3.5, 3.5), (RODURO_TOWER_COTTAGE_01_TIER_1, 3.5, 6.6, 5.9)],
        roof: Roof::Dome,
        dome: 0.226,
        stair: Some(((0.2051, 0.6216), 0.9)),
        doors: &[],
        windows: &[2, 2],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_tower_house_01",
        at_size: 13.29,
        tiers: &[(layout::RODURO_TOWER_HOUSE_01_OUTLINE, 0.0, 3.9, 3.9), (RODURO_TOWER_HOUSE_01_TIER_1, 3.9, 7.0, 6.4), (RODURO_TOWER_HOUSE_01_TIER_2, 7.0, 10.3, 9.0)],
        roof: Roof::Dome,
        dome: 0.394,
        stair: Some(((-0.0309, 0.1183), 1.0)),
        doors: &[((0.2099, 0.9849), 1.16)],
        windows: &[4, 4, 3],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_twin_home_01",
        at_size: 14.38,
        tiers: &[(layout::RODURO_TWIN_HOME_01_OUTLINE, 0.0, 5.2, 3.3)],
        roof: Roof::Dome,
        dome: 0.365,
        stair: None,
        doors: &[],
        windows: &[3],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_vault_tall_01",
        at_size: 11.6,
        tiers: &[(layout::RODURO_VAULT_TALL_01_OUTLINE, 0.0, 3.4, 3.4), (RODURO_VAULT_TALL_01_TIER_1, 3.4, 6.9, 5.4)],
        roof: Roof::Barrel,
        dome: 0.429,
        stair: Some(((0.5172, -0.5814), 0.9)),
        doors: &[],
        windows: &[2, 2],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_weaver_home_01",
        at_size: 14.78,
        tiers: &[(layout::RODURO_WEAVER_HOME_01_OUTLINE, 0.0, 5.2, 3.3)],
        roof: Roof::Dome,
        dome: 0.365,
        stair: None,
        doors: &[],
        windows: &[3],
        yard: None,
        qotiro: None,
    },
    Form {
        key: "roduro_woodworker_home_01",
        at_size: 12.21,
        tiers: &[(layout::RODURO_WOODWORKER_HOME_01_OUTLINE, 0.0, 5.0, 3.4)],
        roof: Roof::Dome,
        dome: 0.32,
        stair: None,
        doors: &[],
        windows: &[3],
        yard: Some(YardLook { dressing: &[(Dress::Block, (-1.9432, -0.0362), 1.6065, (0.7, 0.7, 0.7)), (Dress::Crate, (-1.5958, -0.3802), 2.7128, (0.9, 0.9, 0.7)), (Dress::Logs, (-1.0711, -0.5612), -1.2191, (3.0, 1.2, 1.1)), (Dress::Logs, (-1.9148, 0.5199), 0.6094, (2.6, 1.1, 0.9)), (Dress::Pile, (-1.0215, 0.6673), -2.0903, (1.3, 1.1, 0.5)), (Dress::Trough, (-2.2125, 0.1268), 1.4347, (1.9, 0.65, 0.6)), (Dress::Barrel, (-1.3192, -0.7577), 2.9528, (0.73, 0.75, 0.91)), (Dress::Bucket, (-0.8726, 0.8639), 0.0176, (0.4, 0.4, 0.3)), (Dress::Lantern, (-2.2125, -0.3155), 1.9948, (0.3, 0.3, 2.0))], shades: &[], columns: &[((-2.0297, -0.6228), 2.7), ((-2.1248, -0.1771), 2.7), ((-1.9943, 0.2575), 2.7), ((-1.5756, 0.3676), 2.7), ((-1.3222, 0.0016), 2.7), ((-1.3311, -0.4563), 2.7), ((-1.6132, -0.7969), 2.7)], canopy: Some((&[(-2.0297, -0.6228), (-2.0979, -0.4549), (-2.1247, -0.2509), (-2.1113, -0.0249), (-2.0527, 0.164), (-1.9616, 0.294), (-1.8489, 0.3692), (-1.7037, 0.3931), (-1.5603, 0.3607), (-1.4092, 0.2231), (-1.3217, -0.0015), (-1.3083, -0.3235), (-1.3758, -0.5803), (-1.4981, -0.7381), (-1.6729, -0.8079)], 2.7)), chimney: None }),
        qotiro: None,
    },
];
// ---- end generated forms ----

// ---- generated tiers (tools/buildings/roduro_forms.py rust) ----
/// `qotiro_drill_yard_01`'s tier B (drawing only).
pub const QOTIRO_DRILL_YARD_01_TIER_1: &[(f32, f32)] = &[
    (-0.9999, -1.0), (0.3333, -1.0), (0.3333, 0.4286), (-0.9999, 0.4286),
];

/// `qotiro_family_house_01`'s tier B (drawing only).
pub const QOTIRO_FAMILY_HOUSE_01_TIER_1: &[(f32, f32)] = &[
    (-1.0, -0.6001), (0.4286, -0.6001), (0.4286, 0.6001), (-1.0, 0.6001),
];

/// `qotiro_family_house_01`'s tier C (drawing only).
pub const QOTIRO_FAMILY_HOUSE_01_TIER_2: &[(f32, f32)] = &[
    (-1.0, -0.2), (-0.4286, -0.2), (-0.4286, 0.6001), (-1.0, 0.6001),
];

/// `qotiro_home_corner_01`'s tier B (drawing only).
pub const QOTIRO_HOME_CORNER_01_TIER_1: &[(f32, f32)] = &[
    (-1.0, -0.9999), (0.0, -0.9999), (0.0, 0.2), (-1.0, 0.2),
];

/// `qotiro_home_corner_01`'s tier C (drawing only).
pub const QOTIRO_HOME_CORNER_01_TIER_2: &[(f32, f32)] = &[
    (-1.0, -0.9999), (-0.3333, -0.9999), (-0.3333, -0.2), (-1.0, -0.2),
];

/// `qotiro_home_long_01`'s tier B (drawing only).
pub const QOTIRO_HOME_LONG_01_TIER_1: &[(f32, f32)] = &[
    (-1.0, -1.0), (0.0, -1.0), (0.0, 1.0), (-1.0, 1.0),
];

/// `qotiro_home_small_01`'s tier B (drawing only).
pub const QOTIRO_HOME_SMALL_01_TIER_1: &[(f32, f32)] = &[
    (-1.0, -0.3333), (0.3333, -0.3333), (0.3333, 1.0), (-1.0, 1.0),
];

/// `qotiro_home_small_end_01`'s tier B (drawing only).
pub const QOTIRO_HOME_SMALL_END_01_TIER_1: &[(f32, f32)] = &[
    (-1.0, -1.0), (0.0, -1.0), (0.0, 1.0), (-1.0, 1.0),
];

/// `qotiro_home_small_low_01`'s tier B (drawing only).
pub const QOTIRO_HOME_SMALL_LOW_01_TIER_1: &[(f32, f32)] = &[
    (-1.0, -1.0), (-0.5, -1.0), (-0.5, -0.3333), (-1.0, -0.3333),
];

/// `qotiro_home_small_tall_01`'s tier B (drawing only).
pub const QOTIRO_HOME_SMALL_TALL_01_TIER_1: &[(f32, f32)] = &[
    (-1.0, -1.0), (0.3333, -1.0), (0.3333, 1.0), (-1.0, 1.0),
];

/// `qotiro_home_small_tall_01`'s tier C (drawing only).
pub const QOTIRO_HOME_SMALL_TALL_01_TIER_2: &[(f32, f32)] = &[
    (-1.0, -0.3333), (0.3333, -0.3333), (0.3333, 1.0), (-1.0, 1.0),
];

/// `roduro_hill_great_01`'s tier B (drawing only).
pub const RODURO_HILL_GREAT_01_TIER_1: &[(f32, f32)] = &[
    (-1.0258, 0.6716), (-1.0275, 0.539), (-1.0275, 0.405), (-1.0275, 0.2695), (-1.0275, 0.1355), (-1.0275, 0.0015),
    (-1.0275, -0.134), (-1.0275, -0.268), (-1.0275, -0.4021), (-1.0275, -0.5376), (-1.0275, -0.6716), (-0.8796, -0.7158),
    (-0.7249, -0.7408), (-0.5684, -0.7511), (-0.4119, -0.7496), (-0.2554, -0.7393), (-0.1006, -0.7128), (0.0456, -0.6672),
    (0.1797, -0.5965), (0.2932, -0.5052), (0.3809, -0.3932), (0.4359, -0.268), (0.4652, -0.1355), (0.4703, -0.0015),
    (0.4652, 0.134), (0.4377, 0.2651), (0.3809, 0.3918), (0.2949, 0.5037), (0.1814, 0.5965), (0.0473, 0.6657),
    (-0.0989, 0.7128), (-0.2537, 0.7393), (-0.4101, 0.7496), (-0.5666, 0.7511), (-0.7231, 0.7408), (-0.8779, 0.7158),
];

/// `roduro_hill_old_01`'s tier B (drawing only).
pub const RODURO_HILL_OLD_01_TIER_1: &[(f32, f32)] = &[
    (-0.9493, -0.0632), (-0.9337, -0.1752), (-0.9142, -0.2938), (-0.9259, -0.4436), (-0.8402, -0.5739), (-0.6803, -0.6638),
    (-0.462, -0.6782), (-0.2554, -0.6625), (-0.0741, -0.6274), (0.0858, -0.6039), (0.2164, -0.5283), (0.2612, -0.372),
    (0.306, -0.2925), (0.3197, -0.2169), (0.3216, -0.1544), (0.3509, -0.114), (0.3957, -0.0762), (0.4483, -0.0332),
    (0.4815, 0.0202), (0.4971, 0.0814), (0.4912, 0.144), (0.4561, 0.2026), (0.6764, 0.4397), (0.6862, 0.6417),
    (0.5224, 0.7316), (0.3216, 0.7798), (0.1033, 0.729), (-0.0741, 0.6039), (-0.2281, 0.5779), (-0.4016, 0.5765),
    (-0.653, 0.6221), (-0.8714, 0.5805), (-0.9825, 0.4554), (-0.8986, 0.2756), (-0.8675, 0.1492), (-0.924, 0.0476),
];

/// `roduro_long_ancient_01`'s tier B (drawing only).
pub const RODURO_LONG_ANCIENT_01_TIER_1: &[(f32, f32)] = &[
    (-0.7092, -0.0504), (-0.6683, -0.1367), (-0.6164, -0.2169), (-0.5536, -0.2898), (-0.4539, -0.3371), (-0.3666, -0.3854),
    (-0.2792, -0.4358), (-0.1823, -0.4841), (-0.0703, -0.5087), (0.0457, -0.5077), (0.1618, -0.4933), (0.2724, -0.4645),
    (0.3693, -0.4152), (0.4553, -0.3566), (0.4949, -0.2713), (0.5263, -0.1932), (0.5645, -0.1202), (0.6028, -0.0452),
    (0.6123, 0.037), (0.5932, 0.1172), (0.5522, 0.1932), (0.5304, 0.2816), (0.5004, 0.3813), (0.4171, 0.4573),
    (0.3283, 0.5457), (0.2137, 0.631), (0.0689, 0.6876), (-0.0949, 0.703), (-0.2642, 0.705), (-0.4403, 0.6876),
    (-0.5905, 0.6177), (-0.6874, 0.5036), (-0.7284, 0.3731), (-0.7379, 0.2497), (-0.7652, 0.147), (-0.7502, 0.0442),
];

/// `roduro_long_ancient_01`'s tier C (drawing only).
pub const RODURO_LONG_ANCIENT_01_TIER_2: &[(f32, f32)] = &[
    (-0.5331, -0.0391), (-0.5195, -0.1069), (-0.469, -0.1644), (-0.3679, -0.1922), (-0.2997, -0.221), (-0.2314, -0.2405),
    (-0.1713, -0.261), (-0.1058, -0.2693), (-0.043, -0.2734), (0.0184, -0.2713), (0.0785, -0.259), (0.1399, -0.2497),
    (0.1945, -0.2271), (0.2396, -0.1942), (0.2778, -0.1572), (0.3092, -0.1172), (0.3434, -0.0761), (0.3789, -0.0298),
    (0.3898, 0.0216), (0.3829, 0.075), (0.3939, 0.1377), (0.3925, 0.2086), (0.3693, 0.2826), (0.3201, 0.3535),
    (0.256, 0.4286), (0.1672, 0.4985), (0.0526, 0.557), (-0.0812, 0.5879), (-0.2287, 0.6053), (-0.372, 0.5766),
    (-0.4949, 0.5149), (-0.5973, 0.4358), (-0.671, 0.3422), (-0.6888, 0.2333), (-0.6437, 0.1233), (-0.5932, 0.0339),
];

/// `roduro_long_old_01`'s tier B (drawing only).
pub const RODURO_LONG_OLD_01_TIER_1: &[(f32, f32)] = &[
    (-0.5422, -0.0367), (-0.5354, -0.1064), (-0.4797, -0.165), (-0.4239, -0.2188), (-0.3733, -0.2751), (-0.3074, -0.3227),
    (-0.2314, -0.3667), (-0.2432, -0.6699), (-0.0963, -0.8264), (0.0676, -0.5416), (0.1402, -0.3716), (0.2196, -0.3386),
    (0.3057, -0.3154), (0.4037, -0.2922), (0.478, -0.2421), (0.527, -0.1797), (0.5371, -0.1064), (0.5844, -0.0391),
    (0.6824, 0.0403), (0.728, 0.1381), (0.7465, 0.2482), (0.7769, 0.39), (0.6874, 0.4927), (0.5675, 0.5807),
    (0.4222, 0.6491), (0.2652, 0.7029), (0.0929, 0.7335), (-0.0861, 0.7335), (-0.2601, 0.7103), (-0.4003, 0.6235),
    (-0.4256, 0.4413), (-0.4577, 0.3313), (-0.4594, 0.2323), (-0.4577, 0.1528), (-0.478, 0.0917), (-0.4881, 0.0293),
];

/// `roduro_round_ancient_01`'s tier B (drawing only).
pub const RODURO_ROUND_ANCIENT_01_TIER_1: &[(f32, f32)] = &[
    (-0.741, -0.0471), (-0.7436, -0.1614), (-0.7013, -0.2713), (-0.6339, -0.37), (-0.5387, -0.4518), (-0.4316, -0.5191),
    (-0.2836, -0.5146), (-0.1752, -0.5583), (-0.0641, -0.6401), (0.0747, -0.6861), (0.2175, -0.6682), (0.3523, -0.6256),
    (0.47, -0.5572), (0.5586, -0.463), (0.6114, -0.3531), (0.6617, -0.2522), (0.696, -0.1491), (0.7331, -0.046),
    (0.7555, 0.0639), (0.7344, 0.1738), (0.6696, 0.2713), (0.5625, 0.3397), (0.4832, 0.4148), (0.3801, 0.4641),
    (0.2836, 0.5179), (0.1805, 0.5684), (0.0668, 0.6166), (-0.0602, 0.6289), (-0.1871, 0.6099), (-0.3074, 0.574),
    (-0.4303, 0.5325), (-0.5295, 0.4596), (-0.5797, 0.3543), (-0.5691, 0.2343), (-0.622, 0.1502), (-0.6802, 0.0583),
];

/// `roduro_round_ancient_01`'s tier C (drawing only).
pub const RODURO_ROUND_ANCIENT_01_TIER_2: &[(f32, f32)] = &[
    (-0.544, -0.0325), (-0.5242, -0.1121), (-0.4832, -0.185), (-0.4541, -0.2646), (-0.425, -0.3554), (-0.3351, -0.4013),
    (-0.232, -0.4193), (-0.1382, -0.4406), (-0.043, -0.4383), (0.0483, -0.4283), (0.1368, -0.4148), (0.2228, -0.3901),
    (0.2915, -0.3408), (0.3603, -0.2948), (0.425, -0.2422), (0.4792, -0.1805), (0.5374, -0.1132), (0.5572, -0.0336),
    (0.548, 0.0482), (0.5163, 0.1244), (0.4951, 0.2029), (0.4435, 0.2691), (0.3814, 0.3285), (0.3127, 0.3823),
    (0.2347, 0.4294), (0.1487, 0.4675), (0.0522, 0.4865), (-0.0456, 0.4877), (-0.1421, 0.4675), (-0.232, 0.4361),
    (-0.3166, 0.3957), (-0.392, 0.343), (-0.4462, 0.2747), (-0.4938, 0.204), (-0.5295, 0.1289), (-0.5453, 0.0482),
];

/// `roduro_round_est_02`'s tier B (drawing only).
pub const RODURO_ROUND_EST_02_TIER_1: &[(f32, f32)] = &[
    (-0.6454, -0.0324), (-0.5937, -0.1285), (-0.5466, -0.2205), (-0.4635, -0.2853), (-0.4029, -0.3605), (-0.3356, -0.4357),
    (-0.2323, -0.4566), (-0.1268, -0.4483), (-0.0415, -0.5026), (0.0662, -0.5967), (0.1874, -0.6029), (0.2974, -0.5549),
    (0.3625, -0.4504), (0.4254, -0.3668), (0.5915, -0.3584), (0.688, -0.2748), (0.7262, -0.1578), (0.7464, -0.0387),
    (0.7598, 0.0825), (0.7576, 0.2079), (0.7082, 0.325), (0.6274, 0.4253), (0.5062, 0.4838), (0.4209, 0.5695),
    (0.3221, 0.6489), (0.2054, 0.7032), (0.073, 0.7137), (-0.0505, 0.6364), (-0.1717, 0.6489), (-0.2997, 0.6364),
    (-0.4052, 0.5695), (-0.4815, 0.4775), (-0.495, 0.3501), (-0.5286, 0.2539), (-0.6207, 0.1766), (-0.6633, 0.0763),
];

/// `roduro_round_old_01`'s tier B (drawing only).
pub const RODURO_ROUND_OLD_01_TIER_1: &[(f32, f32)] = &[
    (-0.4931, -0.0593), (-0.525, -0.1602), (-0.5438, -0.2758), (-0.4931, -0.3694), (-0.4091, -0.4368), (-0.2904, -0.4485),
    (-0.1977, -0.4646), (-0.1195, -0.4997), (-0.0384, -0.5699), (0.0615, -0.6123), (0.1644, -0.6035), (0.2643, -0.567),
    (0.3497, -0.5055), (0.4265, -0.4353), (0.5033, -0.3651), (0.5988, -0.2934), (0.6857, -0.1983), (0.7089, -0.0768),
    (0.7118, 0.0476), (0.7292, 0.1792), (0.7147, 0.3182), (0.6539, 0.4411), (0.567, 0.5494), (0.4482, 0.6196),
    (0.2904, 0.5933), (0.16, 0.5538), (0.0688, 0.6694), (-0.0558, 0.7279), (-0.1818, 0.703), (-0.315, 0.6869),
    (-0.4381, 0.6299), (-0.5177, 0.5172), (-0.5395, 0.3724), (-0.5844, 0.2641), (-0.6133, 0.1544), (-0.5496, 0.0344),
];

/// `roduro_smith_home_02`'s tier B (drawing only).
pub const RODURO_SMITH_HOME_02_TIER_1: &[(f32, f32)] = &[
    (0.8981, -0.0151), (0.8906, 0.1283), (0.866, 0.2679), (0.8151, 0.4019), (0.7377, 0.5226), (0.6358, 0.6245),
    (0.5151, 0.7019), (0.3811, 0.7528), (0.2415, 0.7774), (0.0981, 0.7849), (-0.0453, 0.7774), (-0.1849, 0.7528),
    (-0.3189, 0.7019), (-0.4396, 0.6245), (-0.5415, 0.5226), (-0.6189, 0.4019), (-0.6698, 0.2679), (-0.6943, 0.1283),
    (-0.7019, -0.0151), (-0.6943, -0.1585), (-0.6698, -0.2981), (-0.6189, -0.4321), (-0.5415, -0.5528), (-0.4396, -0.6547),
    (-0.3189, -0.7321), (-0.1849, -0.783), (-0.0453, -0.8075), (0.0981, -0.8151), (0.2415, -0.8075), (0.3811, -0.783),
    (0.5151, -0.7321), (0.6358, -0.6547), (0.7377, -0.5528), (0.8151, -0.4321), (0.866, -0.2981), (0.8906, -0.1585),
];

/// `roduro_smoker_home_01`'s tier B (drawing only).
pub const RODURO_SMOKER_HOME_01_TIER_1: &[(f32, f32)] = &[
    (0.8931, -0.0147), (0.8868, 0.1279), (0.8616, 0.2683), (0.8092, 0.4025), (0.7338, 0.5241), (0.6331, 0.6247),
    (0.5115, 0.7002), (0.3774, 0.7526), (0.2369, 0.7778), (0.0943, 0.7841), (-0.0482, 0.7778), (-0.1908, 0.7526),
    (-0.3229, 0.7002), (-0.4444, 0.6247), (-0.5451, 0.5241), (-0.6226, 0.4025), (-0.673, 0.2683), (-0.7002, 0.1279),
    (-0.7065, -0.0147), (-0.7002, -0.1572), (-0.673, -0.2998), (-0.6226, -0.4319), (-0.5451, -0.5535), (-0.4444, -0.6541),
    (-0.3229, -0.7317), (-0.1908, -0.782), (-0.0482, -0.8092), (0.0943, -0.8155), (0.2369, -0.8092), (0.3774, -0.782),
    (0.5115, -0.7317), (0.6331, -0.6541), (0.7338, -0.5535), (0.8092, -0.4319), (0.8616, -0.2998), (0.8868, -0.1572),
];

/// `roduro_tower_cottage_01`'s tier B (drawing only).
pub const RODURO_TOWER_COTTAGE_01_TIER_1: &[(f32, f32)] = &[
    (0.8205, 0.0), (0.8128, 0.1513), (0.7846, 0.2973), (0.7282, 0.4378), (0.6487, 0.5621), (0.5436, 0.6621),
    (0.4179, 0.7378), (0.2846, 0.7891), (0.1436, 0.8134), (0.0, 0.8188), (-0.1436, 0.8134), (-0.2846, 0.7891),
    (-0.4179, 0.7378), (-0.5436, 0.6621), (-0.6487, 0.5621), (-0.7282, 0.4378), (-0.7846, 0.2973), (-0.8128, 0.1513),
    (-0.8205, 0.0), (-0.8128, -0.1513), (-0.7846, -0.2973), (-0.7282, -0.4378), (-0.6487, -0.5621), (-0.5436, -0.6621),
    (-0.4179, -0.7378), (-0.2846, -0.7891), (-0.1436, -0.8134), (-0.0, -0.8188), (0.1436, -0.8134), (0.2846, -0.7891),
    (0.4179, -0.7378), (0.5436, -0.6621), (0.6487, -0.5621), (0.7282, -0.4378), (0.7846, -0.2973), (0.8128, -0.1513),
];

/// `roduro_tower_house_01`'s tier B (drawing only).
pub const RODURO_TOWER_HOUSE_01_TIER_1: &[(f32, f32)] = &[
    (0.6915, 0.0683), (0.7096, 0.2167), (0.699, 0.365), (0.6524, 0.5066), (0.5726, 0.6266), (0.4658, 0.7166),
    (0.3424, 0.7749), (0.2099, 0.8049), (0.0745, 0.8133), (-0.0609, 0.8133), (-0.1964, 0.8133), (-0.3318, 0.8099),
    (-0.4658, 0.7916), (-0.5937, 0.7466), (-0.7096, 0.67), (-0.8029, 0.5633), (-0.8646, 0.43), (-0.8901, 0.2833),
    (-0.8841, 0.1333), (-0.8555, -0.0117), (-0.8104, -0.1533), (-0.7517, -0.2866), (-0.678, -0.4133), (-0.5937, -0.53),
    (-0.4944, -0.6316), (-0.3815, -0.7133), (-0.2551, -0.7649), (-0.1211, -0.7866), (0.0143, -0.7783), (0.1437, -0.7399),
    (0.2641, -0.6716), (0.3695, -0.5783), (0.4597, -0.4666), (0.538, -0.345), (0.6042, -0.2133), (0.6554, -0.0767),
];

/// `roduro_tower_house_01`'s tier C (drawing only).
pub const RODURO_TOWER_HOUSE_01_TIER_2: &[(f32, f32)] = &[
    (0.4507, 0.1183), (0.4643, 0.23), (0.4552, 0.3416), (0.4221, 0.4466), (0.3619, 0.5366), (0.2822, 0.605),
    (0.1889, 0.6483), (0.0895, 0.67), (-0.0128, 0.6766), (-0.1136, 0.6766), (-0.2144, 0.6766), (-0.3168, 0.675),
    (-0.4161, 0.66), (-0.5139, 0.6266), (-0.5997, 0.57), (-0.6704, 0.4883), (-0.7156, 0.39), (-0.7351, 0.28),
    (-0.7306, 0.1683), (-0.7096, 0.0583), (-0.6764, -0.0483), (-0.6313, -0.1483), (-0.5771, -0.2433), (-0.5139, -0.33),
    (-0.4387, -0.4066), (-0.3529, -0.4666), (-0.2596, -0.5066), (-0.1588, -0.5233), (-0.0579, -0.5166), (0.0399, -0.4883),
    (0.1302, -0.4366), (0.2084, -0.3666), (0.2777, -0.2833), (0.3363, -0.1917), (0.3845, -0.0933), (0.4236, 0.01),
];

/// `roduro_vault_tall_01`'s tier B (drawing only).
pub const RODURO_VAULT_TALL_01_TIER_1: &[(f32, f32)] = &[
    (0.8603, 0.0), (0.8552, 0.1814), (0.8241, 0.3558), (0.7552, 0.5093), (0.6517, 0.6256), (0.531, 0.7),
    (0.4017, 0.7465), (0.2672, 0.7697), (0.1345, 0.779), (-0.0, 0.779), (-0.1345, 0.779), (-0.2672, 0.7697),
    (-0.4017, 0.7465), (-0.531, 0.7), (-0.6517, 0.6256), (-0.7552, 0.5093), (-0.8241, 0.3558), (-0.8552, 0.1814),
    (-0.8603, 0.0), (-0.8552, -0.1814), (-0.8241, -0.3558), (-0.7552, -0.5093), (-0.6517, -0.6256), (-0.531, -0.7),
    (-0.4017, -0.7465), (-0.2672, -0.7697), (-0.1345, -0.779), (-0.0, -0.779), (0.1345, -0.779), (0.2672, -0.7697),
    (0.4017, -0.7465), (0.531, -0.7), (0.6517, -0.6256), (0.7552, -0.5093), (0.8241, -0.3558), (0.8552, -0.1814),
];
// ---- end generated tiers ----

fn form_of(v: &Variant) -> Option<&'static Form> {
    FORMS.iter().find(|f| f.key == v.key)
}

/// How far a soft shoulder rounds a tier's top edge in, and down, metres.
const SHOULDER: f32 = 0.6;

/// An outline in the building's frame (metres), moved in `m` metres all
/// round (each corner along the middle of its two edges' ways in).
fn inset(f: &Frame, pts: &[(f32, f32)], m: f32) -> Vec<V2> {
    let n = pts.len();
    let q: Vec<V2> = pts.iter().map(|&(x, y)| V2::new(x * f.hx, y * f.hy)).collect();
    let inward = |k: usize| {
        let e = q[(k + 1) % n].sub(q[k]);
        let l = e.len().max(1e-6);
        V2::new(-e.y / l, e.x / l)
    };
    (0..n)
        .map(|k| {
            let (a, b) = (inward((k + n - 1) % n), inward(k));
            let s = a.add(b);
            let l2 = (s.x * s.x + s.y * s.y).max(0.3);
            q[k].add(s.scale(2.0 * m / l2))
        })
        .collect()
}

/// Does an outline turn in anywhere by more than a little (a crescent, a
/// deep nook)?
fn concave(pts: &[(f32, f32)]) -> bool {
    let n = pts.len();
    let mut worst = 0.0f32;
    for k in 0..n {
        let (a, b, c) = (pts[(k + n - 1) % n], pts[k], pts[(k + 1) % n]);
        let cross = (b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0);
        let l = ((b.0 - a.0).hypot(b.1 - a.1) * (c.0 - b.0).hypot(c.1 - b.1)).max(1e-6);
        worst = worst.min(cross / l);
    }
    worst < -0.25
}

/// Frame metres on the map.
fn fm(f: &Frame, q: V2) -> V2 {
    f.c.add(f.dir().scale(q.x)).add(f.side_v2().scale(q.y))
}

/// The way out from each corner of a ring (frame metres), as a direction
/// on the map.
fn outward(f: &Frame, ring: &[V2]) -> Vec<Vec3> {
    let n = ring.len();
    (0..n)
        .map(|k| {
            let (a, b, c) = (ring[(k + n - 1) % n], ring[k], ring[(k + 1) % n]);
            let e = c.sub(a);
            let o = fm(f, b.add(V2::new(e.y, -e.x))).sub(fm(f, b));
            vec3(o.x, 0.0, o.y).normalize_or_zero()
        })
        .collect()
}

/// The band of wall between two rings (frame metres) at heights `y0` and
/// `y1` over the frame's base: upright when the rings match, sloping in
/// when the upper one is smaller. `up` tips the lighting toward the sky.
#[allow(clippy::too_many_arguments)]
fn band(b: &mut Builder, f: &Frame, lo: &[V2], hi: &[V2], y0: f32, y1: f32, up: f32, col: Rgb) {
    let n = lo.len().min(hi.len());
    let (no, nh) = (outward(f, lo), outward(f, hi));
    let c = palette::lin(col);
    for k in 0..n {
        let j = (k + 1) % n;
        let p = [to3(fm(f, lo[k]), f.y + y0), to3(fm(f, lo[j]), f.y + y0), to3(fm(f, hi[j]), f.y + y1), to3(fm(f, hi[k]), f.y + y1)];
        let nn = |o: Vec3| (o * (1.0 - up) + Vec3::Y * up).normalize_or_zero();
        b.quad_lin(p, [nn(no[k]), nn(no[j]), nn(nh[j]), nn(nh[k])], [c; 4]);
    }
}

/// A `band` that samples the stone texture: `off` is where in the strata
/// the building starts (metres), as its walls' `strata` used.
#[allow(clippy::too_many_arguments)]
fn band_tex(b: &mut Builder, f: &Frame, lo: &[V2], hi: &[V2], y0: f32, y1: f32, up: f32, col: Rgb, off: f32) {
    let n = lo.len().min(hi.len());
    let (no, nh) = (outward(f, lo), outward(f, hi));
    let fr = ring_fracs(lo);
    let c = palette::lin(col);
    for k in 0..n {
        let j = (k + 1) % n;
        let uj = if j == 0 { 1.0 } else { fr[j] };
        let p = [to3(fm(f, lo[k]), f.y + y0), to3(fm(f, lo[j]), f.y + y0), to3(fm(f, hi[j]), f.y + y1), to3(fm(f, hi[k]), f.y + y1)];
        let nn = |o: Vec3| (o * (1.0 - up) + Vec3::Y * up).normalize_or_zero();
        let v0 = (y0 + off) / stone::TEX_H;
        let v1 = (y1 + off) / stone::TEX_H;
        b.quad_tex(p, [nn(no[k]), nn(no[j]), nn(nh[j]), nn(nh[k])], [c; 4], [[stone::u_of(fr[k]), v0], [stone::u_of(uj), v0], [stone::u_of(uj), v1], [stone::u_of(fr[k]), v1]]);
    }
}

/// A `band` of grass (the turf material, the grass tile by place).
#[allow(clippy::too_many_arguments)]
fn band_turf(b: &mut Builder, f: &Frame, lo: &[V2], hi: &[V2], y0: f32, y1: f32, up: f32, col: Rgb) {
    let n = lo.len().min(hi.len());
    let (no, nh) = (outward(f, lo), outward(f, hi));
    let c = palette::lin(col);
    for k in 0..n {
        let j = (k + 1) % n;
        let p = [to3(fm(f, lo[k]), f.y + y0), to3(fm(f, lo[j]), f.y + y0), to3(fm(f, hi[j]), f.y + y1), to3(fm(f, hi[k]), f.y + y1)];
        let nn = |o: Vec3| (o * (1.0 - up) + Vec3::Y * up).normalize_or_zero();
        b.quad_turf(p, [nn(no[k]), nn(no[j]), nn(nh[j]), nn(nh[k])], [c; 4]);
    }
}

/// A flat `cap` of grass.
fn cap_turf(b: &mut Builder, f: &Frame, ring: &[V2], y: f32, col: Rgb) {
    let c = palette::lin(col);
    for [i, j, k] in ears(ring) {
        let (p, q, r) = (to3(fm(f, ring[i]), f.y + y), to3(fm(f, ring[j]), f.y + y), to3(fm(f, ring[k]), f.y + y));
        b.quad_turf([p, q, r, r], [Vec3::Y; 4], [c; 4]);
    }
}

/// Where in the strata a building starts (metres), by its seed.
fn strata_off(seed: u64) -> f32 {
    ((seed >> 20) % 1000) as f32 / 1000.0 * stone::TEX_H
}

/// Triangles filling an outline that may curve in (ear clipping), as
/// corner numbers.
fn ears(pts: &[V2]) -> Vec<[usize; 3]> {
    let mut left: Vec<usize> = (0..pts.len()).collect();
    let mut out = Vec::new();
    let cross = |a: V2, b: V2, c: V2| (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    let mut guard = 0;
    while left.len() > 3 && guard < 10_000 {
        guard += 1;
        let n = left.len();
        let mut cut = None;
        for k in 0..n {
            let (i, j, l) = (left[(k + n - 1) % n], left[k], left[(k + 1) % n]);
            let (a, b, c) = (pts[i], pts[j], pts[l]);
            if cross(a, b, c) <= 1e-6 {
                continue;
            }
            let clear = left.iter().all(|&m| m == i || m == j || m == l || {
                let p = pts[m];
                !(cross(a, b, p) >= 0.0 && cross(b, c, p) >= 0.0 && cross(c, a, p) >= 0.0)
            });
            if clear {
                cut = Some(k);
                break;
            }
        }
        // (A sliver the clipping can't settle: cut the first corner.)
        let k = cut.unwrap_or(0);
        out.push([left[(k + n - 1) % n], left[k], left[(k + 1) % n]]);
        left.remove(k);
    }
    if left.len() == 3 {
        out.push([left[0], left[1], left[2]]);
    }
    out
}

/// A flat cap over a ring (frame metres) at height `y`, facing up.
fn cap(b: &mut Builder, f: &Frame, ring: &[V2], y: f32, col: Rgb) {
    for [i, j, k] in ears(ring) {
        let (p, q, r) = (to3(fm(f, ring[i]), f.y + y), to3(fm(f, ring[j]), f.y + y), to3(fm(f, ring[k]), f.y + y));
        b.quad([p, q, r, r], Vec3::Y, col);
    }
}

/// The middle of a ring (the mean of its corners).
fn middle(ring: &[V2]) -> V2 {
    ring.iter().fold(V2::new(0.0, 0.0), |a, p| a.add(*p)).scale(1.0 / ring.len().max(1) as f32)
}

/// Spots `n` evenly round a ring by length (frame metres), each with the way
/// out there (radians on the map), starting `from` of the way round.
fn round_ring(f: &Frame, ring: &[V2], n: usize, from: f32) -> Vec<(V2, f32)> {
    let len: Vec<f32> = (0..ring.len()).map(|k| ring[k].dist(ring[(k + 1) % ring.len()])).collect();
    let total: f32 = len.iter().sum();
    (0..n)
        .map(|i| {
            let mut s = ((i as f32 + from) / n as f32).fract() * total;
            let mut k = 0;
            while s > len[k] && k + 1 < len.len() {
                s -= len[k];
                k += 1;
            }
            let (a, b) = (ring[k], ring[(k + 1) % ring.len()]);
            let p = a.lerp(b, s / len[k].max(1e-6));
            let e = b.sub(a);
            let o = fm(f, p.add(V2::new(e.y, -e.x))).sub(fm(f, p));
            (p, o.y.atan2(o.x))
        })
        .collect()
}

/// Grown stone, as Laz wants the placeholders to show it (15 Oct, a
/// happy medium between these and the final look): the stone darker,
/// its growth in clear strata bands that climb round the building (one
/// band a turn, so the seams spiral), each band a ledge a little in or
/// out of the last; the roof stepped the same way with grass over its
/// crown, a ridge of stone spiralling in to the top, and tufts of grass.
const GRASS: Rgb = [0.27, 0.36, 0.15];
const TUFT: Rgb = [0.35, 0.47, 0.17];

/// How far round the ring (0..1 by length) each corner is.
fn ring_fracs(ring: &[V2]) -> Vec<f32> {
    let n = ring.len();
    let mut out = Vec::with_capacity(n);
    let mut s = 0.0;
    for k in 0..n {
        out.push(s);
        s += ring[k].dist(ring[(k + 1) % n]);
    }
    let total = s.max(1e-6);
    out.iter().map(|v| v / total).collect()
}

/// The ledge of stratum `i` (metres in or out), by the seed.
fn ledge(seed: u64, i: i64) -> f32 {
    let h = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add((i as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9));
    ((h >> 40) % 9) as f32 * 0.015 - 0.06
}

/// The wall of a grown building between heights `z0` and `z1` (over the
/// frame's base): the strata of `stone.rs`, each a ledge a little in or
/// out of the last, their seams waving round the building, and the
/// texture laid on so its bands fall on the same seams. `pts` is the
/// outline in the unit frame; `off` is where in the strata this building
/// starts (metres, by its seed), so no two walls band alike.
fn strata(b: &mut Builder, f: &Frame, pts: &[(f32, f32)], z0: f32, z1: f32, seed: u64, stone: Rgb) {
    if z1 <= z0 {
        return;
    }
    let table = stone::strata();
    let off = strata_off(seed);
    let n = pts.len();
    let fr = ring_fracs(&inset(f, pts, 0.0));
    let col = palette::lin(stone);
    let lip = palette::lin(palette::scale(stone, 1.12));
    // Each stratum's top at each corner, in the building's own heights.
    let idx = stone::between(&table, z0 + off, z1 + off);
    let tops: Vec<Vec<f32>> = idx.iter().map(|&i| (0..=n).map(|k| stone::top_of(&table, i, if k == n { 1.0 } else { fr[k] }) - off).collect()).collect();
    let rings: Vec<Vec<V2>> = idx.iter().map(|&i| inset(f, pts, ledge(seed, i))).collect();
    let uv = |k: usize, y: f32| [stone::u_of(if k == n { 1.0 } else { fr[k] }), (y + off) / stone::TEX_H];
    let c = |y: f32| y.clamp(z0, z1);
    for s in 0..idx.len() {
        // The band from the stratum below's top (or the floor) to this one's.
        let this = &rings[s];
        let (no, nn) = (outward(f, this), if s + 1 < idx.len() { outward(f, &rings[s + 1]) } else { outward(f, this) });
        for k in 0..n {
            let j = k + 1;
            let jj = j % n;
            let (lo_k, lo_j) = if s == 0 { (z0, z0) } else { (tops[s - 1][k], tops[s - 1][j]) };
            let (hi_k, hi_j) = (tops[s][k], tops[s][j]);
            if (hi_k <= z0 && hi_j <= z0) || (lo_k >= z1 && lo_j >= z1) {
                continue;
            }
            let p = [to3(fm(f, this[k]), f.y + c(lo_k)), to3(fm(f, this[jj]), f.y + c(lo_j)), to3(fm(f, this[jj]), f.y + c(hi_j)), to3(fm(f, this[k]), f.y + c(hi_k))];
            b.quad_tex(p, [no[k], no[jj], no[jj], no[k]], [col; 4], [uv(k, c(lo_k)), uv(j, c(lo_j)), uv(j, c(hi_j)), uv(k, c(hi_k))]);
            // The ledge up to the next stratum, where its ring differs.
            if s + 1 < idx.len() && (hi_k < z1 || hi_j < z1) {
                let next = &rings[s + 1];
                let q = [to3(fm(f, this[k]), f.y + c(hi_k)), to3(fm(f, this[jj]), f.y + c(hi_j)), to3(fm(f, next[jj]), f.y + c(hi_j + 0.025)), to3(fm(f, next[k]), f.y + c(hi_k + 0.025))];
                let up = |o: Vec3| (o * 0.35 + Vec3::Y * 0.65).normalize_or_zero();
                b.quad_lin(q, [up(no[k]), up(no[jj]), up(nn[jj]), up(nn[k])], [lip; 4]);
            }
        }
    }
}

/// A tuft of grass: three thin blades crossing at `at`.
fn tuft(b: &mut Builder, at: Vec3, h: f32, seed: u64) {
    let col = palette::scale(TUFT, 0.9 + (seed % 5) as f32 * 0.05);
    for k in 0..3 {
        let a = k as f32 * 1.05 + (seed % 7) as f32 * 0.15;
        let d = vec3(a.cos(), 0.0, a.sin()) * 0.11;
        let lean = vec3(a.sin(), 0.0, -a.cos()) * (h * 0.25);
        b.quad([at - d, at + d, at + lean + Vec3::Y * h, at + lean + Vec3::Y * h], Vec3::Y, col);
    }
}

/// The roof of a grown building over the outline `pts` (unit frame) from
/// the wall's top to the crown at `z1`: stepped rings of stone, grass over
/// the crown with a stone ridge spiralling in to the top (domes only), and
/// tufts of grass. The rings close toward the outline's middle (a dome) or
/// across only (a barrel vault). Returns the crown.
#[allow(clippy::too_many_arguments)]
fn grown_roof(b: &mut Builder, f: &Frame, pts: &[(f32, f32)], top_wall: f32, z1: f32, roof: Roof, long: bool, seed: u64, stone: Rgb) -> V2 {
    let base = inset(f, pts, 0.0);
    let c = middle(&base);
    let dh = (z1 - top_wall).max(0.05);
    let shrink = |p: V2, sh: f32| {
        let d = p.sub(c);
        match roof {
            Roof::Dome | Roof::Flat => c.add(d.scale(sh)),
            Roof::Barrel if long => c.add(V2::new(d.x * (0.75 + 0.25 * sh), d.y * sh)),
            Roof::Barrel => c.add(V2::new(d.x * sh, d.y * (0.75 + 0.25 * sh))),
        }
    };
    let height = |sh: f32| top_wall + dh * (1.0 - sh * sh).max(0.0).sqrt();
    let steps = 8;
    let grass_from = 4;
    let mut prev = base.clone();
    for s in 1..=steps {
        let lat = s as f32 / steps as f32 * std::f32::consts::FRAC_PI_2;
        let sh = lat.cos().max(0.02);
        let ring: Vec<V2> = base.iter().map(|&p| shrink(p, sh)).collect();
        let sh0 = ((s - 1) as f32 / steps as f32 * std::f32::consts::FRAC_PI_2).cos();
        let (ya, yb) = (height(sh0), height(sh));
        let col = if s > grass_from { palette::scale(GRASS, 0.9 + s as f32 * 0.02) } else { palette::scale(stone, [0.9, 0.97, 0.86][s % 3]) };
        // The riser of each ring, then the flat step out to the next (a ledge).
        if s > grass_from {
            band_turf(b, f, &prev, &prev, ya, yb - 0.04, 0.0, palette::scale(col, 0.85));
            band_turf(b, f, &prev, &ring, yb - 0.04, yb, 0.9, col);
        } else {
            band_tex(b, f, &prev, &prev, ya, yb - 0.04, 0.0, palette::scale(col, 0.95), strata_off(seed));
            band(b, f, &prev, &ring, yb - 0.04, yb, 0.9, col);
        }
        prev = ring;
    }
    cap_turf(b, f, &prev, z1, GRASS);
    // The ridge of stone spiralling in over the grass to the crown.
    if roof == Roof::Dome {
        let turns = 2.5;
        let n = (turns * 28.0) as usize;
        let spots = round_ring(f, &base, 28, 0.0);
        let at = |i: usize| {
            let t = i as f32 / n as f32;
            let sh = 0.72 * (1.0 - t) + 0.04;
            let (p, _) = spots[(i % 28) % spots.len()];
            let q = shrink(p, sh);
            to3(fm(f, q), f.y + height(sh) + 0.07)
        };
        let ridge = palette::scale(stone, 1.0);
        for i in 0..n {
            let (p, q) = (at(i), at(i + 1));
            let d = (q - p).normalize_or_zero();
            let w = Vec3::Y.cross(d) * 0.09;
            b.quad([p - w, p + w, q + w, q - w], Vec3::Y, ridge);
        }
    }
    // Tufts of grass over the crown, more on a bigger roof.
    let area = f.hx * f.hy;
    let count = ((area * 0.35) as usize).clamp(6, 36);
    let spots = round_ring(f, &base, count, 0.37);
    for (i, &(p, _)) in spots.iter().enumerate() {
        let h = seed.wrapping_add(i as u64 * 0x2545_F491_4F6C_DD1D).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let sh = 0.08 + ((h >> 40) % 60) as f32 * 0.01;
        let q = shrink(p, sh);
        tuft(b, to3(fm(f, q), f.y + height(sh) - 0.02), 0.22 + ((h >> 50) % 4) as f32 * 0.05, h);
    }
    c
}

/// A Roduro building with an outline of its own, from outside: each tier
/// its outline grown up from the ground (or the tier under it) with a soft
/// shoulder at the top, a flat terrace on it, the top tier domed; ancient
/// stone (darker, in growth-ring bands) with moss on every top; its doors
/// dark in the wall and lit windows round each tier.
#[allow(clippy::too_many_arguments)]
fn roduro_form(b: &mut Builder, gl: &mut Builder, v: &Variant, form: &Form, f: &Frame, size: f32, seed: u64, sink: f32, unit: &dyn Fn(u64) -> f32) {
    let k = size / form.at_size;
    let stone = palette::scale(palette::STONE, 0.64 + unit(9) * 0.08);
    let moss = [0.27, 0.31, 0.22];
    let last = form.tiers.len() - 1;
    let mut doors: Vec<V2> = Vec::new();
    for (i, &(pts, z0, z1, wall_top)) in form.tiers.iter().enumerate() {
        let (z0, z1, wall_top) = (if i == 0 { 0.0 } else { z0 * k + sink - SHOULDER * 0.5 }, z1 * k + sink, wall_top * k + sink);
        let outer = inset(f, pts, 0.0);
        let top_wall = if i == last { wall_top.min(z0 + (z1 - z0) * (1.0 - form.dome)) } else { (z1 - SHOULDER).min(wall_top) };
        // The wall, in strata that climb round it.
        strata(b, f, pts, z0, top_wall, seed ^ (i as u64 * 0x9F), stone);
        if i == 0 {
            // Moss creeping up from the ground.
            let skirt = inset(f, pts, -0.04);
            band(b, f, &skirt, &skirt, -0.1, sink + 0.45, 0.0, moss);
        }
        if i < last {
            // The shoulder rounding over, and the terrace on top.
            let mid = inset(f, pts, SHOULDER * 0.3);
            let ins = inset(f, pts, SHOULDER);
            band(b, f, &outer, &mid, top_wall, z1 - SHOULDER * 0.25, 0.45, palette::scale(stone, 0.9));
            band_turf(b, f, &mid, &ins, z1 - SHOULDER * 0.25, z1, 0.85, moss);
            cap_turf(b, f, &ins, z1, GRASS);
            // Grass along the terrace's edge.
            for (j, &(p, _)) in round_ring(f, &inset(f, pts, SHOULDER + 0.35), 10, 0.2).iter().enumerate() {
                tuft(b, to3(fm(f, p), f.y + z1 - 0.02), 0.22, seed.wrapping_add(j as u64 * 0x1F3));
            }
        } else if concave(pts) {
            // An outline that curls (a crescent) can't be drawn in toward
            // one middle: a low rounded roof, inset from the edge, mossed.
            let base = inset(f, pts, 0.0);
            let mid = inset(f, pts, 0.5);
            let ins = inset(f, pts, 1.1);
            let rise = (z1 - top_wall).min(1.2);
            band(b, f, &base, &mid, top_wall, top_wall + rise * 0.6, 0.45, palette::scale(stone, 0.9));
            band_turf(b, f, &mid, &ins, top_wall + rise * 0.6, top_wall + rise, 0.85, moss);
            cap_turf(b, f, &ins, top_wall + rise, GRASS);
            for (j, &(p, _)) in round_ring(f, &inset(f, pts, 1.6), 14, 0.2).iter().enumerate() {
                tuft(b, to3(fm(f, p), f.y + top_wall + rise - 0.02), 0.22, seed.wrapping_add(j as u64 * 0x1F3));
            }
        } else {
            // The roof: the outline drawn in as it rises, toward its middle
            // (a dome) or across only (a barrel vault along the length).
            let c = grown_roof(b, f, pts, top_wall, z1, form.roof, f.hx >= f.hy, seed, stone);
            // A little stone bud at the crown of a dome.
            if form.roof == Roof::Dome {
                let crown = fm(f, c);
                b.dome(to3(crown, f.y + z1 - 0.1), 0.6, 0.5, 0.6, 0.25, 0.0, seed ^ 5, palette::scale(stone, 1.1));
            }
        }
        // Lit windows round the tier, clear of the doors.
        let n = form.windows.get(i).copied().unwrap_or(0);
        let wy = if i == last { z0 + (top_wall - z0) * 0.55 } else { z0 + (top_wall - z0) * if i == 0 { 0.5 } else { 0.55 } };
        let (dx, dy) = door_spot(v);
        let main = V2::new(dx * f.hx, dy * f.hy);
        for (p, out) in round_ring(f, &outer, n, 0.13 + i as f32 * 0.21) {
            if i == 0 && (p.dist(main) < 3.0 || doors.iter().any(|d| d.dist(p) < 3.0)) {
                continue;
            }
            let at = to3(fm(f, p), f.y + wy) + vec3(out.cos(), 0.0, out.sin()) * 0.15;
            gl.patch(at, 0.8, if i == 0 { 1.1 } else { 0.9 }, out, palette::WINDOW);
        }
        if i == 0 {
            // The other doors, drawn shut, where a line out from inside meets the wall.
            for &((x, y), turn) in form.doors {
                let from = V2::new(x * f.hx, y * f.hy);
                let dir = V2::new(turn.cos(), turn.sin());
                let mut t = 0.0;
                while t < 20.0 && layout::inside_outline(pts, f.hx, f.hy, from.add(dir.scale(t))) {
                    t += 0.05;
                }
                let at = from.add(dir.scale(t));
                doors.push(at);
                let o = fm(f, at.add(dir)).sub(fm(f, at));
                let face = o.y.atan2(o.x);
                b.patch(to3(fm(f, at), f.y + sink + 1.2) + vec3(face.cos(), 0.0, face.sin()) * 0.15, 1.3, 2.4, face, DOOR_DARK);
                b.block(to3(fm(f, at.add(dir.scale(0.4))), f.y), 1.7, 0.9, sink + 0.15, face, palette::scale(stone, 1.15));
            }
        }
    }
    // The way in: dark in the wall, a step before it.
    let (dx, dy) = door_spot(v);
    let at = V2::new(dx * f.hx, dy * f.hy);
    let (_, _, inward) = layout::outline_nearest(form.tiers[0].0, f.hx, f.hy, at.sub(V2::new(0.05, 0.0)));
    let o = fm(f, at.sub(inward)).sub(fm(f, at));
    let face = o.y.atan2(o.x);
    b.patch(to3(fm(f, at), f.y + sink + 1.22) + vec3(face.cos(), 0.0, face.sin()) * 0.15, 1.3, 2.45, face, DOOR_DARK);
    b.block(f.at(dx, dy, 0.0) + f.fwd() * 0.45, 1.6, 1.0, sink + 0.15, f.rot, palette::scale(stone, 1.15));
    if let (Some(y), Some(look)) = (v.yard, form.yard.as_ref()) {
        yard(b, gl, v, y, look, f, form.tiers[0].0, stone, seed, form.qotiro.is_some());
    }
}

/// A trade home's work yard: the fence where it isn't the house's own
/// wall, the gate posts, the roofed shed on its posts, the chimney, the
/// awnings, the stations (the sim's own pieces) and the dressing.
#[allow(clippy::too_many_arguments)]
fn yard(b: &mut Builder, gl: &mut Builder, v: &Variant, y: &layout::Yard, look: &YardLook, f: &Frame, house: &[(f32, f32)], stone: Rgb, seed: u64, quarried: bool) {
    // The yard's wall is the house's own stone, lower: the same growth-ring
    // bands, moss along its top, and it runs on from the house's wall (or,
    // quarried, the same blocks with a dressed coping).
    let n = y.outline.len();
    let pts: Vec<V2> = y.outline.iter().map(|&(x, yy)| V2::new(x * f.hx, yy * f.hy)).collect();
    let gate = V2::new(y.gate.0 * f.hx, y.gate.1 * f.hy);
    let moss = [0.27, 0.31, 0.22];
    let h = y.fence;
    let outer = inset(f, y.outline, 0.0);
    let inner = inset(f, y.outline, 0.7);
    let cap_out = inset(f, y.outline, -0.05);
    let cap_in = inset(f, y.outline, 0.75);
    let drawn = |k: usize| {
        let mid = pts[k].lerp(pts[(k + 1) % n], 0.5);
        layout::outline_nearest(house, f.hx, f.hy, mid).0 <= -0.45 && mid.dist(gate) >= y.gate_width * 0.5 + 0.1
    };
    let (c1, c2, c3) = (palette::lin(stone), palette::lin(palette::scale(stone, 0.88)), palette::lin(if quarried { palette::scale(stone, 1.3) } else { moss }));
    let fr = ring_fracs(&outer);
    let ring_len: f32 = (0..n).map(|k| outer[k].dist(outer[(k + 1) % n])).sum();
    let off = strata_off(seed);
    for k in 0..n {
        if !drawn(k) {
            continue;
        }
        let j = (k + 1) % n;
        let quad = |lo: &[V2], hi: &[V2], y0: f32, y1: f32, c: [f32; 4], flip: bool| {
            let p = [to3(fm(f, lo[k]), f.y + y0), to3(fm(f, lo[j]), f.y + y0), to3(fm(f, hi[j]), f.y + y1), to3(fm(f, hi[k]), f.y + y1)];
            let e = fm(f, lo[j]).sub(fm(f, lo[k]));
            let nn = vec3(e.y, 0.0, -e.x).normalize_or_zero() * if flip { -1.0 } else { 1.0 };
            (p, nn)
        };
        // Outside and inside faces in the house's stone, and the mossed top.
        let _ = c2;
        let uv = |k: usize, y: f32| {
            let t = if k >= n { 1.0 } else { fr[k] };
            [if quarried { stone::u_ashlar((t * ring_len) % stone::ASHLAR_W) } else { stone::u_of(t) }, (y + off) / stone::TEX_H]
        };
        for (lo, flip) in [(&outer, false), (&inner, true)] {
            let (p, nn) = quad(lo, lo, -0.3, h, c1, flip);
            b.quad_tex(p, [nn; 4], [c1; 4], [uv(k, -0.3), uv(k + 1, -0.3), uv(k + 1, h), uv(k, h)]);
        }
        let (p, _) = quad(&cap_out, &cap_out, h, h + 0.16, c3, false);
        b.quad_lin(p, [Vec3::Y; 4], [c3; 4]);
        let (p, _) = quad(&cap_in, &cap_in, h, h + 0.16, c3, true);
        b.quad_lin(p, [Vec3::Y; 4], [c3; 4]);
        let top = [to3(fm(f, cap_out[k]), f.y + h + 0.16), to3(fm(f, cap_out[j]), f.y + h + 0.16), to3(fm(f, cap_in[j]), f.y + h + 0.16), to3(fm(f, cap_in[k]), f.y + h + 0.16)];
        b.quad_lin(top, [Vec3::Y; 4], [c3; 4]);
        // Ends where a run stops (at the gate, or at the house).
        for (e, show) in [(k, !drawn((k + n - 1) % n)), (j, !drawn(j))] {
            if show {
                let end = [to3(fm(f, outer[e]), f.y - 0.3), to3(fm(f, inner[e]), f.y - 0.3), to3(fm(f, inner[e]), f.y + h + 0.16), to3(fm(f, outer[e]), f.y + h + 0.16)];
                b.quad_lin(end, [Vec3::Y; 4], [c2; 4]);
            }
        }
    }
    // The gate: two posts.
    let (_, q, inward) = layout::outline_nearest(y.outline, f.hx, f.hy, gate);
    let along = V2::new(-inward.y, inward.x);
    for s in [-1.0f32, 1.0] {
        let p = q.add(along.scale(s * (y.gate_width * 0.5 + 0.15)));
        b.column(to3(fm(f, p), f.y - 0.1), 0.26, 0.22, y.fence + 0.6, 6, palette::scale(stone, 0.85));
    }
    // The shed: posts and a flat roof slab over them, leaning on the house.
    for &((x, yy), h) in look.columns {
        if quarried {
            b.stick(to3(f.p(x, yy), f.y - 0.1), to3(f.p(x, yy), f.y + h + 0.1), 0.07, WOOD_DARK);
        } else {
            b.column(to3(f.p(x, yy), f.y - 0.1), 0.24, 0.2, h + 0.1, 6, palette::scale(stone, 0.9));
        }
    }
    if let Some((outline, h)) = look.canopy {
        if quarried {
            // A cloth awning on its posts.
            let pts: Vec<Vec3> = outline.iter().map(|&(x, yy)| f.at(x, yy, h + 0.05)).collect();
            let c = pts.iter().fold(Vec3::ZERO, |a, p| a + *p) / pts.len() as f32;
            let cloth = CLOTHS[3];
            for k in 0..pts.len() {
                let j = (k + 1) % pts.len();
                b.quad([c + Vec3::Y * 0.12, pts[k], pts[j], c + Vec3::Y * 0.12], Vec3::Y, cloth);
                b.quad([c + Vec3::Y * 0.1, pts[j], pts[k], c + Vec3::Y * 0.1], -Vec3::Y, palette::scale(cloth, 0.8));
            }
        } else {
        // A grown stone slab over the shed, mossed on top, drawn in a touch
        // as it rises like the house's own roofs.
        let ring = inset(f, outline, -0.2);
        let top = inset(f, outline, 0.5);
        let lo = Frame { y: f.y + h, ..*f };
        band(b, &lo, &ring, &ring, 0.0, 0.3, 0.0, palette::scale(stone, 0.8));
        band(b, &lo, &ring, &top, 0.3, 0.7, 0.6, palette::scale(stone, 0.9));
        cap(b, &lo, &top, 0.7, moss);
        }
    }
    if let Some(((x, yy), r, h)) = look.chimney {
        let c = f.at(x, yy, -0.1);
        b.column(c, r, r * 0.8, h + 0.1, 8, palette::scale(stone, 0.8));
        gl.smoke(c + Vec3::Y * (h + 0.1), r * 0.5);
    }
    // Awnings: a cloth on its corners' posts.
    for &(foot, h, cloth) in look.shades {
        if foot.len() == 4 {
            let p: Vec<Vec3> = foot.iter().map(|&(x, yy)| f.at(x, yy, h + 0.1)).collect();
            b.quad([p[0], p[1], p[2], p[3]], Vec3::Y, cloth);
            b.quad([p[3], p[2], p[1], p[0]], -Vec3::Y, palette::scale(cloth, 0.8));
        }
    }
    // The stations (the same spots the sim has them at), then the dressing.
    for (k, p) in y.pieces.iter().enumerate() {
        furniture(b, gl, p.what, f.at(p.at.0, p.at.1, 0.02), f.rot + p.rot, 64 + k);
    }
    for &(kind, (x, yy), rot, (w, d, h)) in look.dressing {
        dressing(b, gl, kind, f.at(x, yy, 0.0), f.rot + rot, (w, d, h), seed ^ (x.to_bits() as u64));
    }
    let _ = v;
}

/// One piece of yard dressing as a few blocks.
fn dressing(b: &mut Builder, gl: &mut Builder, kind: Dress, base: Vec3, rot: f32, (w, d, h): (f32, f32, f32), seed: u64) {
    let (s, c) = rot.sin_cos();
    let fwd = vec3(c, 0.0, s);
    let side = vec3(-s, 0.0, c);
    match kind {
        Dress::Bellows => {
            b.block(base + Vec3::Y * 0.4, w, d, h * 0.5, rot, LEATHER);
            b.stick(base + Vec3::Y * 0.4, base + Vec3::Y * 0.4 + fwd * (w * 0.6), 0.08, WOOD);
            for sgn in [-1.0f32, 1.0] {
                b.stick(base + side * (sgn * d * 0.4), base + side * (sgn * d * 0.4) + Vec3::Y * 0.4, 0.07, WOOD_DARK);
            }
        }
        Dress::Trough | Dress::Tub => {
            let r = (w.min(d) * 0.5).max(0.3);
            if kind == Dress::Tub {
                b.column(base, r, r * 0.9, h * 0.6, 10, WOOD_DARK);
                b.column(base + Vec3::Y * (h * 0.6 - 0.02), r * 0.85, r * 0.85, 0.02, 10, [0.22, 0.17, 0.11]);
            } else {
                b.block(base, w, d, h, rot, WOOD_DARK);
                b.block(base + Vec3::Y * (h - 0.03), w - 0.12, d - 0.12, 0.03, rot, [0.20, 0.28, 0.34]);
            }
        }
        Dress::Grindstone => {
            b.block(base, w * 0.8, d * 0.8, 0.5, rot, WOOD);
            b.column(base + Vec3::Y * 0.5, 0.05, 0.05, 0.02, 4, WOOD);
            disc(b, base + Vec3::Y * (0.5 + w * 0.35), w * 0.35, rot + std::f32::consts::FRAC_PI_2, palette::scale(palette::STONE, 0.85));
        }
        Dress::Barrel => b.column(base, w * 0.5, w * 0.45, h, 10, WOOD),
        Dress::Block => b.block(base, w, d, h, rot, IRON),
        Dress::Frame => {
            // Two posts and a crossbar; a tall frame gets a cloth or hides on it.
            for sgn in [-1.0f32, 1.0] {
                let p = base + side * (sgn * d * 0.5);
                b.stick(p, p + Vec3::Y * h, 0.07, WOOD_DARK);
            }
            b.stick(base + side * (-d * 0.5) + Vec3::Y * (h - 0.05), base + side * (d * 0.5) + Vec3::Y * (h - 0.05), 0.06, WOOD_DARK);
            if h > 1.5 {
                b.block(base + Vec3::Y * (h * 0.35), w.max(0.08), d * 0.8, h * 0.5, rot, LEATHER);
            }
        }
        Dress::Pile => b.dome(base, w.max(0.9) * 0.6, d.max(0.9) * 0.6, h.max(0.4), 0.3, 0.0, seed, [0.36, 0.28, 0.18]),
        Dress::Crate => b.block(base, w, d, h, rot, WOOD_LIGHT),
        Dress::Bucket => b.column(base, 0.18, 0.15, 0.3, 8, WOOD_DARK),
        Dress::Sacks => {
            b.dome(base, 0.4, 0.3, 0.35, 0.1, 0.0, seed, LINEN);
            b.dome(base + fwd * 0.5, 0.35, 0.3, 0.3, 0.1, 0.0, seed ^ 3, palette::scale(LINEN, 0.9));
        }
        Dress::Stool => b.block(base + Vec3::Y * 0.3, 0.35, 0.35, 0.08, rot, WOOD),
        Dress::Lantern => {
            b.stick(base, base + Vec3::Y * 2.0, 0.06, IRON);
            gl.patch(base + Vec3::Y * 1.85, 0.2, 0.25, rot, palette::WINDOW);
        }
        Dress::Post => b.stick(base, base + Vec3::Y * 2.3, 0.08, WOOD),
        Dress::Wheel => disc(b, base + Vec3::Y * 0.5, 0.5, rot, WOOD_DARK),
        Dress::Logs => {
            // Rows of logs lying along the piece, each row one fewer.
            let r = (d * 0.16).clamp(0.12, 0.2);
            let across = ((d / (r * 2.0)).floor() as usize).clamp(2, 5);
            let mut n = across;
            let mut y = r;
            let mut row = 0;
            while n >= 1 && y < h {
                for k in 0..n {
                    let off = (k as f32 - (n as f32 - 1.0) * 0.5) * r * 2.0;
                    let p = base + side * off + Vec3::Y * y;
                    let jolt = ((seed.wrapping_mul(31).wrapping_add((row * 7 + k) as u64)) % 7) as f32 * 0.04;
                    let col = palette::scale(WOOD, 0.75 + jolt);
                    b.stick(p - fwd * (w * 0.5 - jolt), p + fwd * (w * 0.5), r, col);
                }
                n -= 1;
                y += r * 1.75;
                row += 1;
            }
        }
        Dress::Smokehouse => {
            let r = w.min(d) * 0.5;
            let stone = palette::scale(palette::STONE, 0.75);
            b.column(base, r, r * 0.92, h * 0.7, 10, stone);
            b.dome(base + Vec3::Y * (h * 0.7 - 0.02), r * 0.95, r * 0.95, h * 0.3, 0.1, 0.0, seed, palette::scale(stone, 0.85));
            b.column(base + Vec3::Y * (h * 0.9), r * 0.25, r * 0.2, h * 0.2, 6, palette::scale(stone, 0.7));
            // A low door on the front.
            b.block(base + fwd * (r * 0.9) + Vec3::Y * 0.6, 0.08, 0.6, 1.2, rot, WOOD_DARK);
            gl.smoke(base + Vec3::Y * (h * 1.1), r * 0.35);
        }
        Dress::Stone => {
            let stone = palette::scale(palette::STONE, 0.9 + ((seed % 5) as f32) * 0.03);
            b.block(base, w, d, h, rot + ((seed % 3) as f32) * 0.15, stone);
        }
    }
    let _ = h;
}

/// A spiral stair inside, cut off where the walls are cut away: a stone
/// post and wedge steps winding up round it.
fn stair(b: &mut Builder, f: &Frame, form: &Form) {
    let Some(((x, y), r)) = form.stair else { return };
    let c = f.p(x, y);
    let rise = 0.19;
    let steps = 12;
    b.column(to3(c, f.y), 0.16, 0.16, rise * steps as f32 + 0.3, 8, palette::scale(palette::STONE, 0.8));
    for k in 0..steps {
        let a = f.rot + std::f32::consts::PI + k as f32 * 0.42;
        let at = c.add(V2::new(a.cos(), a.sin()).scale(r * 0.55));
        b.block(to3(at, f.y), r * 0.9, 0.42, rise * (k + 1) as f32, a, palette::scale(palette::STONE, 0.95 - (k % 2) as f32 * 0.06));
    }
}

// ---- Outside -------------------------------------------------------------------

/// Draw building `i` of town `s` from outside as its variant's placeholder
/// (the style stored for it, `layout::variant_in`). False if it has none
/// (the hearth), so the caller draws it its own way.
pub fn exterior(b: &mut Builder, gl: &mut Builder, t: &Terrain, s: &Settlement, i: u16, on_ground: &dyn Fn(V2) -> f32) -> bool {
    let Some(bd) = s.buildings.get(i as usize) else { return false };
    if bd.kind == BuildingKind::Hearth {
        return false;
    }
    let Some(v) = layout::variant_in(s, i) else { return false };
    let ground = on_ground(bd.pos);
    let sink = if bd.kind == BuildingKind::HoraroStilt { 0.0 } else { (t.slope(bd.pos) * bd.size * 0.6).min(4.0) };
    variant(b, gl, v, bd.pos, bd.rot, bd.size, bd.seed, ground, sink);
    true
}

/// A home of the forged town (scenery until it becomes a settlement), when
/// the final models are off: a Roduro variant picked from where it stands.
pub fn forge_home(b: &mut Builder, gl: &mut Builder, t: &Terrain, at: V2, rot: f32, eldest: bool, on_ground: &dyn Fn(V2) -> f32) {
    let seed = rng::key(&[at.x.to_bits() as u64, at.y.to_bits() as u64, 0x464F_5247]);
    let size = if eldest { 12.0 } else { 8.5 + (seed >> 50) as f32 / (1u64 << 14) as f32 * 3.0 };
    let Some(v) = layout::variant_for(BuildingKind::RoduroHome, size, seed) else { return };
    let sink = (t.slope(at) * 6.0).min(2.0);
    variant(b, gl, v, at, rot, size, seed, on_ground(at), sink);
}

/// One variant from outside, standing on `ground` (sunk `sink` into a slope).
#[allow(clippy::too_many_arguments)]
pub fn variant(b: &mut Builder, gl: &mut Builder, v: &Variant, pos: V2, rot: f32, size: f32, seed: u64, ground: f32, sink: f32) {
    let f = Frame { c: pos, rot, hx: v.half.0 * size, hy: v.half.1 * size, y: ground - sink };
    let unit = |k: u64| (rng::key(&[seed, k]) >> 40) as f32 / (1u64 << 24) as f32;
    let sign = signs::for_building(v);
    match v.kind {
        BuildingKind::RoduroHome => roduro(b, gl, v, &f, size, seed, sink, &unit, sign),
        BuildingKind::QotiroBlock | BuildingKind::QotiroHall | BuildingKind::QotiroTemple => qotiro(b, gl, v, &f, size, sink, sign),
        BuildingKind::HoraroStilt => horaro(b, gl, v, &f, seed),
        BuildingKind::Hearth => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn roduro(b: &mut Builder, gl: &mut Builder, v: &Variant, f: &Frame, size: f32, seed: u64, sink: f32, unit: &dyn Fn(u64) -> f32, sign: Option<signs::Picto>) {
    if let Some(form) = form_of(v) {
        return roduro_form(b, gl, v, form, f, size, seed, sink, unit);
    }
    let col = palette::scale(palette::STONE, 0.64 + unit(9) * 0.08);
    let wall_h = 2.3 + sink;
    let dome_h = size * (0.26 + unit(1) * 0.06);
    // The ring of grown wall in its strata, then the roof over it.
    let ring = |cx: f32, cy: f32, rx: f32, ry: f32| -> Vec<(f32, f32)> { (0..28).map(|k| { let a = k as f32 / 28.0 * std::f32::consts::TAU; (cx + rx * a.cos(), cy + ry * a.sin()) }).collect() };
    let outline = ring(0.0, 0.0, 1.0, 1.0);
    strata(b, f, &outline, 0.0, wall_h, seed, col);
    grown_roof(b, f, &ring(0.0, 0.0, 0.97, 0.97), wall_h, wall_h + dome_h, Roof::Dome, f.hx >= f.hy, seed, col);
    let top = wall_h + dome_h;
    let (dx, dy) = door_spot(v);
    let side_free = if v.door > 0.0 { -1.0 } else { 1.0 };
    // An upper storey budding out of the back of the dome.
    if v.storeys >= 2 {
        let (bx, by) = (-0.25, -0.2 * side_free);
        let bud = f.at(bx, by, wall_h + dome_h * 0.45);
        let z = wall_h + dome_h * 0.45;
        strata(b, f, &ring(bx, by, 0.5, 0.46), z - 0.3, z + dome_h * 0.5, seed ^ 7, col);
        grown_roof(b, f, &ring(bx, by, 0.48, 0.44), z + dome_h * 0.5, z + dome_h * 1.25, Roof::Dome, true, seed ^ 7, col);
        // Its window, looking out to the front.
        let w = f.at(bx, by, 0.0) + f.fwd() * (f.hx * 0.49);
        gl.patch(vec3(w.x, bud.y + dome_h * 0.3, w.z), 0.8, 0.6, f.rot, palette::WINDOW);
    }
    if v.key == "roduro_great" {
        // A second bud on the other side, lower.
        let z = wall_h + dome_h * 0.25;
        grown_roof(b, f, &ring(-0.35, 0.45 * side_free, 0.38, 0.34), z, z + dome_h * 0.8, Roof::Dome, true, seed ^ 11, col);
        // A stone lantern by the door.
        let lp = f.at(dx, dy, 0.0) + f.fwd() * 1.6 + f.side() * 1.4;
        b.column(lp + Vec3::Y * sink, 0.18, 0.14, 1.5, 6, palette::STONE);
        gl.flame(lp + Vec3::Y * (sink + 1.5), 0.16, 0.3);
    }
    // The door, dark in the wall, and a warm window on the free side.
    let door = f.at(dx, dy, sink + 1.0) + vec3(f.normal_at(dx, dy, true).cos(), 0.0, f.normal_at(dx, dy, true).sin()) * 0.15;
    b.patch(door, 1.1, 2.0, f.normal_at(dx, dy, true), DOOR_DARK);
    b.block(f.at(dx, dy, 0.0) + f.fwd() * 0.45, 1.5, 0.9, sink + 0.15, f.rot, palette::scale(palette::STONE, 1.15));
    let wy = 0.45 * side_free;
    let wx = (1.0 - wy * wy).sqrt();
    let wn = f.normal_at(wx, wy, true);
    gl.patch(f.at(wx, wy, sink + 1.6) + vec3(wn.cos(), 0.0, wn.sin()) * 0.15, 0.9, 0.7, wn, palette::WINDOW);
    // A trade's sign: a board on brackets beside the door, on the side away
    // from the window (past a trader's awning).
    if let Some(p) = sign {
        let off = if v.key == "roduro_trader" { 2.8 } else { 1.75 };
        let sy = (dy - side_free * off / f.hy).clamp(-0.85, 0.85);
        let sx = (1.0 - sy * sy).sqrt();
        let n = f.normal_at(sx, sy, true);
        signs::roduro_board(b, gl, p, f.at(sx, sy, sink + 2.6), n, door);
    }
    // What the place is for, seen from outside.
    match v.key {
        "roduro_forge" => {
            // A chimney stack through the dome, a wisp of smoke at its top,
            // and the forge's mouth glowing through the wall beside it.
            let c = f.at(0.1, -0.62, 0.0);
            b.column(c, 0.62, 0.46, top + 1.4, 8, palette::scale(palette::STONE, 0.8));
            gl.smoke(c + Vec3::Y * (top + 1.4), 0.35);
            let mn = f.normal_at(0.15, -0.99, true);
            gl.patch(f.at(0.15, -0.99, sink + 0.9) + vec3(mn.cos(), 0.0, mn.sin()) * 0.08, 0.9, 0.6, mn, palette::EMBER);
            // Charcoal heaped outside.
            b.dome(f.at(0.7, -1.25, sink), 0.9, 0.8, 0.6, 0.2, 0.0, seed, [0.12, 0.11, 0.10]);
        }
        "roduro_tender" => {
            // Grower's beds either side of the door, stone buds coming up.
            for s in [-1.0f32, 1.0] {
                let c = f.at(1.0, 0.0, sink) + f.fwd() * 1.5 + f.side() * (s * 2.3);
                b.block(c, 1.1, 1.8, 0.45, f.rot, palette::STONE);
                b.block(c + Vec3::Y * 0.45, 0.9, 1.6, 0.03, f.rot, SOIL);
                for k in 0..3 {
                    let at = c + f.side() * ((k as f32 - 1.0) * 0.5) + Vec3::Y * 0.47;
                    b.dome(at, 0.2, 0.18, 0.25 + 0.1 * k as f32, 0.2, 0.0, seed ^ k, [0.60, 0.63, 0.66]);
                }
            }
            // A pale stone bud growing on the roof.
            b.dome(f.at(0.1, 0.2, top - 0.3), 0.7, 0.6, 0.9, 0.25, 0.0, seed ^ 3, [0.62, 0.66, 0.70]);
        }
        "roduro_trader" => {
            // An awning over the door, a counter beside it, goods stacked out.
            let front = f.at(dx, dy, sink) + f.fwd() * 0.1;
            for (k, col) in CLOTHS.iter().enumerate().take(3) {
                let off = f.side() * ((k as f32 - 1.0) * 1.1);
                let c = front + off + f.fwd() * 0.8 + Vec3::Y * 2.5;
                let down = (f.fwd() * 1.6 - Vec3::Y * 0.5).normalize();
                obox(b, c, f.side() * 0.55, down.cross(f.side()).normalize() * 0.03, down * 0.85, *col);
            }
            for s in [-1.0f32, 1.0] {
                let post = front + f.fwd() * 1.55 + f.side() * (s * 1.6);
                b.stick(post, post + Vec3::Y * 2.15, 0.08, WOOD);
            }
            let ct = front + f.fwd() * 1.3 + f.side() * (2.6 * side_free);
            b.block(ct, 0.7, 1.8, 1.0, f.rot, WOOD_DARK);
            b.block(ct + Vec3::Y * 1.0, 0.8, 1.9, 0.06, f.rot, WOOD_LIGHT);
            for k in 0..3 {
                b.block(ct + Vec3::Y * 1.06 + f.side() * ((k as f32 - 1.0) * 0.55), 0.3, 0.3, 0.2, f.rot, CLOTHS[k]);
            }
            let cr = front + f.fwd() * 1.2 - f.side() * (2.4 * side_free);
            b.block(cr, 0.8, 0.8, 0.7, f.rot + 0.3, WOOD_LIGHT);
            b.column(cr + f.side() * (0.9 * side_free) + f.fwd() * 0.3, 0.32, 0.35, 0.9, 10, [0.50, 0.34, 0.20]);
        }
        "roduro_longhouse" => {
            // A smoke hole over the hearth.
            let c = f.at(0.0, -0.45, top - dome_h * 0.25);
            b.column(c, 0.4, 0.35, dome_h * 0.3 + 0.5, 8, palette::scale(palette::STONE, 0.8));
        }
        _ => {}
    }
}

/// A quarried wall round an outline (layout units) between heights `z0`
/// and `z1`: each face its own run of blocks, the texture laid along it
/// from the face's start.
fn ashlar_walls(b: &mut Builder, f: &Frame, pts: &[(f32, f32)], z0: f32, z1: f32, off: f32, col: Rgb) {
    if z1 <= z0 {
        return;
    }
    let ring = inset(f, pts, 0.0);
    let n = ring.len();
    let c = palette::lin(col);
    for k in 0..n {
        let j = (k + 1) % n;
        let (a, bb) = (ring[k], ring[j]);
        let len = a.dist(bb);
        let e = fm(f, bb).sub(fm(f, a));
        let nn = vec3(e.y, 0.0, -e.x).normalize_or_zero();
        // In runs of at most the texture's width.
        let mut s0 = 0.0;
        while s0 < len - 0.01 {
            let s1 = (s0 + stone::ASHLAR_W).min(len);
            let (pa, pb) = (a.lerp(bb, s0 / len), a.lerp(bb, s1 / len));
            let p = [to3(fm(f, pa), f.y + z0), to3(fm(f, pb), f.y + z0), to3(fm(f, pb), f.y + z1), to3(fm(f, pa), f.y + z1)];
            let (u0, u1) = (stone::u_ashlar(0.0), stone::u_ashlar(s1 - s0));
            let (v0, v1) = ((z0 + off) / stone::TEX_H, (z1 + off) / stone::TEX_H);
            b.quad_tex(p, [nn; 4], [c; 4], [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]);
            s0 = s1;
        }
    }
}

/// A parapet along an outline's edge at height `z`: `h` high, `t` thick,
/// left out within `clear` metres of the `skip` spots (frame metres) and
/// wherever an outline in `over` stands on it.
#[allow(clippy::too_many_arguments)]
fn parapet(b: &mut Builder, f: &Frame, pts: &[(f32, f32)], z: f32, h: f32, t: f32, skip: &[V2], clear: f32, over: &[&[(f32, f32)]], col: Rgb) {
    let outer = inset(f, pts, 0.0);
    let inner = inset(f, pts, t);
    let n = outer.len();
    let c = palette::lin(col);
    let top = palette::lin(palette::scale(col, 1.2));
    for k in 0..n {
        let j = (k + 1) % n;
        let len = outer[k].dist(outer[j]);
        let pieces = (len / 1.0).ceil().max(1.0) as usize;
        for q in 0..pieces {
            let (t0, t1) = (q as f32 / pieces as f32, (q + 1) as f32 / pieces as f32);
            let (oa, ob) = (outer[k].lerp(outer[j], t0), outer[k].lerp(outer[j], t1));
            let (ia, ib) = (inner[k].lerp(inner[j], t0), inner[k].lerp(inner[j], t1));
            let mid = oa.lerp(ob, 0.5);
            if skip.iter().any(|s| s.dist(mid) < clear) {
                continue;
            }
            let m = V2::new(mid.x / f.hx.max(1e-6), mid.y / f.hy.max(1e-6));
            if over.iter().any(|o| layout::inside_outline(o, 1.0, 1.0, V2::new(m.x, m.y)) || layout::outline_nearest(o, f.hx, f.hy, mid).0.abs() < 0.2) {
                continue;
            }
            let e = fm(f, ob).sub(fm(f, oa));
            let nn = vec3(e.y, 0.0, -e.x).normalize_or_zero();
            let (y0, y1) = (f.y + z, f.y + z + h);
            b.quad_lin([to3(fm(f, oa), y0), to3(fm(f, ob), y0), to3(fm(f, ob), y1), to3(fm(f, oa), y1)], [nn; 4], [c; 4]);
            b.quad_lin([to3(fm(f, ib), y0), to3(fm(f, ia), y0), to3(fm(f, ia), y1), to3(fm(f, ib), y1)], [-nn; 4], [c; 4]);
            b.quad_lin([to3(fm(f, oa), y1), to3(fm(f, ob), y1), to3(fm(f, ib), y1), to3(fm(f, ia), y1)], [Vec3::Y; 4], [top; 4]);
            // The ends of a run.
            for (p0, p1, flip) in [(oa, ia, q == 0), (ob, ib, q + 1 == pieces)] {
                if flip {
                    b.quad_lin([to3(fm(f, p0), y0), to3(fm(f, p1), y0), to3(fm(f, p1), y1), to3(fm(f, p0), y1)], [Vec3::Y; 4], [c; 4]);
                }
            }
        }
    }
}

/// A Qotiro doorway on a wall: the opening dark, tapered in toward the
/// head, dressed jambs, a lintel and a threshold.
fn qotiro_door(b: &mut Builder, c: Vec3, facing: f32, w: f32, h: f32, dressed: Rgb) {
    let out = vec3(facing.cos(), 0.0, facing.sin());
    let right = vec3(-facing.sin(), 0.0, facing.cos());
    let base = c + out * 0.04;
    // The opening, narrowing from foot to head.
    let p = [base - right * (w * 0.5), base + right * (w * 0.5), base + right * (w * 0.4) + Vec3::Y * h, base - right * (w * 0.4) + Vec3::Y * h];
    b.quad(p, out, DOOR_DARK);
    // Jambs leaning in with it, the lintel over, the step before it.
    for sgn in [-1.0f32, 1.0] {
        let foot = base + right * (sgn * (w * 0.5 + 0.12));
        let head = base + right * (sgn * (w * 0.4 + 0.12)) + Vec3::Y * h;
        b.stick(foot, head, 0.13, dressed);
    }
    b.block(base + Vec3::Y * h - out * 0.1, 0.28, w + 0.7, 0.32, facing, dressed);
    b.block(c + out * 0.35, 0.7, w + 0.5, 0.14, facing, dressed);
}

/// A lit Qotiro window: a slit with a sill and a lintel.
fn qotiro_window(b: &mut Builder, gl: &mut Builder, c: Vec3, facing: f32, dressed: Rgb) {
    let out = vec3(facing.cos(), 0.0, facing.sin());
    gl.patch(c + out * 0.05, 0.46, 0.95, facing, palette::WINDOW);
    b.block(c - Vec3::Y * 0.5 - out * 0.05, 0.2, 0.7, 0.08, facing, dressed);
    b.block(c + Vec3::Y * 0.5 - out * 0.05, 0.2, 0.7, 0.1, facing, dressed);
}

/// A Qotiro building with a form of its own: tiers of quarried stone,
/// each its slab and parapet, doors, slit windows, the outside stairs and
/// what stands on the terraces.
fn qotiro_form(b: &mut Builder, gl: &mut Builder, v: &Variant, form: &Form, f: &Frame, size: f32, sink: f32) {
    let k = size / form.at_size;
    let stone = palette::scale(palette::QUARRIED, 1.0);
    let dressed = palette::scale(palette::QUARRIED, 1.45);
    let paving = palette::scale(palette::QUARRIED, 1.35);
    let off = strata_off(v.key.len() as u64 * 0x9E37);
    let look = form.qotiro.as_ref();
    let landings: Vec<V2> = look.map(|q| q.landings.iter().map(|&(x, y)| V2::new(x * f.hx, y * f.hy)).collect()).unwrap_or_default();
    let last = form.tiers.len() - 1;
    let (dx, dy) = door_spot(v);
    let main_door = V2::new(dx * f.hx, dy * f.hy);
    for (i, &(pts, z0, z1, wall_top)) in form.tiers.iter().enumerate() {
        let (z0, z1, wall_top) = (if i == 0 { 0.0 } else { z0 * k + sink }, z1 * k + sink, wall_top * k + sink);
        // The walls, the slab's band, the paved top.
        ashlar_walls(b, f, pts, z0, wall_top, off, stone);
        let outer = inset(f, pts, 0.0);
        let proud = inset(f, pts, -0.04);
        band(b, f, &proud, &proud, wall_top, z1, 0.0, dressed);
        cap(b, f, &proud, z1, paving);
        // The parapet, left out where a stair lands and where the next tier
        // stands.
        let over: Vec<&[(f32, f32)]> = form.tiers.iter().skip(i + 1).map(|t| t.0).collect();
        parapet(b, f, pts, z1, 0.8, 0.45, &landings, 1.0, &over, dressed);
        // Windows along the faces, clear of the doors and the corners.
        let n = form.windows.get(i).copied().unwrap_or(0);
        if n > 0 {
            let wz = z0 + (wall_top - z0) * 0.6;
            let doors: Vec<V2> = if i == 0 {
                let mut d = vec![main_door];
                d.extend(form.doors.iter().map(|&((x, y), _)| V2::new(x * f.hx, y * f.hy)));
                d
            } else {
                look.map(|q| q.doors_up.iter().filter(|d| (d.2 * k + sink - z0).abs() < 0.5).map(|&((x, y), _, _)| V2::new(x * f.hx, y * f.hy)).collect()).unwrap_or_default()
            };
            for (p, out) in round_ring(f, &outer, n * 2, 0.07 + i as f32 * 0.13).into_iter().take(n * 2) {
                if doors.iter().any(|d| d.dist(p) < 1.7) || outer.iter().any(|c| c.dist(p) < 0.7) {
                    continue;
                }
                qotiro_window(b, gl, to3(fm(f, p), f.y + wz), out, dressed);
            }
        }
        let _ = last;
    }
    // The doors: the main one on the ground tier, the rest on the terraces.
    let face = f.normal_at(dx, dy, false);
    qotiro_door(b, f.at(dx, dy, sink + 0.0) + vec3(face.cos(), 0.0, face.sin()) * 0.02, face, 1.3, 2.4, dressed);
    for &((x, y), yaw) in form.doors {
        let face = f.rot + yaw;
        qotiro_door(b, f.at(x, y, sink + 0.0), face, 1.3, 2.4, dressed);
    }
    if let Some(q) = look {
        for &((x, y), yaw, z) in q.doors_up {
            let face = f.rot + yaw;
            qotiro_door(b, f.at(x, y, z * k + sink), face, 1.3, 2.4, dressed);
        }
        for &(kind, (x, y), rot, (w, d, h), z) in q.props {
            let base = f.at(x, y, z * k + sink);
            let rot = f.rot + rot;
            qotiro_prop(b, gl, kind, base, rot, (w * k, d * k, h * k), stone, dressed);
        }
    }
    if let (Some(y), Some(look)) = (v.yard, form.yard.as_ref()) {
        yard(b, gl, v, y, look, f, form.tiers[0].0, stone, v.key.len() as u64, true);
    }
}

/// One Qotiro prop as a few blocks.
#[allow(clippy::too_many_arguments)]
fn qotiro_prop(b: &mut Builder, gl: &mut Builder, kind: QProp, base: Vec3, rot: f32, (w, d, h): (f32, f32, f32), stone: Rgb, dressed: Rgb) {
    let (sn, cs) = rot.sin_cos();
    let fwd = vec3(cs, 0.0, sn);
    let side = vec3(-sn, 0.0, cs);
    let iron = IRON;
    match kind {
        QProp::Stair => {
            // Steps up the way `rot` points: each a block from the ground
            // to its tread, then the landing; the side wall on the side
            // away from the building (a negative width says the left).
            let wall = if d < 0.0 { -1.0 } else { 1.0 };
            let d = d.abs();
            let side_w = side * wall;
            let steps = (h / 0.2).round().max(2.0) as usize;
            let depth = w / steps as f32;
            for i in 0..steps {
                let tread = h * (i + 1) as f32 / steps as f32;
                let c = base + fwd * ((i as f32 + 0.5) * depth);
                b.block(c, depth + 0.01, d, tread, rot, if i % 2 == 0 { stone } else { palette::scale(stone, 1.08) });
            }
            // The outer side wall, a low parapet sloping with the steps.
            let lo = base + side_w * (d * 0.5 + 0.12);
            let li = lo - side_w * 0.24;
            let (ya, yb) = (0.0, h);
            let rise = |t: f32| Vec3::Y * (ya + (yb - ya) * t);
            let top = 0.55;
            for (o, i_) in [(lo, li)] {
                let pts = [o + rise(0.0), o + fwd * w + rise(1.0), o + fwd * w + rise(1.0)];
                // Outer face, inner face, the top.
                b.quad([pts[0], pts[1], pts[1] + Vec3::Y * top, pts[0] + Vec3::Y * top], side_w, dressed);
                b.quad([pts[1], pts[2], pts[2] + Vec3::Y * top, pts[1] + Vec3::Y * top], side_w, dressed);
                let ipts = [i_ + rise(0.0), i_ + fwd * w + rise(1.0), i_ + fwd * (w + 2.0) + rise(1.0)];
                b.quad([ipts[1], ipts[0], ipts[0] + Vec3::Y * top, ipts[1] + Vec3::Y * top], -side_w, dressed);
                b.quad([ipts[2], ipts[1], ipts[1] + Vec3::Y * top, ipts[2] + Vec3::Y * top], -side_w, dressed);
                b.quad([pts[0] + Vec3::Y * top, pts[1] + Vec3::Y * top, ipts[1] + Vec3::Y * top, ipts[0] + Vec3::Y * top], Vec3::Y, palette::scale(dressed, 1.1));
                b.quad([pts[1] + Vec3::Y * top, pts[2] + Vec3::Y * top, ipts[2] + Vec3::Y * top, ipts[1] + Vec3::Y * top], Vec3::Y, palette::scale(dressed, 1.1));
                // Under the parapet, solid down to the ground.
                b.quad([pts[0], pts[1], pts[1] - rise(1.0), pts[0]], side_w, stone);
                b.quad([pts[1] - rise(1.0), pts[1], pts[2], pts[2] - rise(1.0)], side_w, stone);
            }
        }
        QProp::Disc => {
            let c = base + fwd * 0.08;
            disc(b, c, w * 0.5, rot, palette::METAL_GOLD);
            for i in 0..8 {
                let a = i as f32 / 8.0 * std::f32::consts::TAU;
                let dir = (side * a.cos() + Vec3::Y * a.sin()).normalize();
                b.stick(c + dir * (w * 0.5), c + dir * (w * 0.5 + 0.18), 0.03, palette::METAL_GOLD);
            }
        }
        QProp::Awning => {
            let cloth = CLOTHS[3];
            let corners = [base + fwd * (w * 0.5) + side * (d * 0.5), base + fwd * (w * 0.5) - side * (d * 0.5), base - fwd * (w * 0.5) - side * (d * 0.5), base - fwd * (w * 0.5) + side * (d * 0.5)];
            for (i, c) in corners.iter().enumerate() {
                let hh = if i < 2 { h } else { h + 0.25 };
                b.stick(*c, *c + Vec3::Y * hh, 0.06, WOOD_DARK);
            }
            let top: Vec<Vec3> = corners.iter().enumerate().map(|(i, c)| *c + Vec3::Y * if i < 2 { h } else { h + 0.25 }).collect();
            let mid = (top[0] + top[1] + top[2] + top[3]) * 0.25 - Vec3::Y * 0.12;
            for i in 0..4 {
                let j = (i + 1) % 4;
                b.quad([mid, top[i], top[j], mid], Vec3::Y, cloth);
                b.quad([mid, top[j], top[i], mid], -Vec3::Y, palette::scale(cloth, 0.8));
            }
        }
        QProp::Trough => {
            b.block(base, w, d, h, rot, dressed);
            b.block(base + Vec3::Y * (h - 0.04), w - 0.16, d - 0.16, 0.04, rot, SOIL);
            let n = (w / 0.35).max(1.0) as usize;
            for i in 0..n {
                let c = base + fwd * ((i as f32 - (n as f32 - 1.0) * 0.5) * 0.35) + Vec3::Y * h;
                b.dome(c, 0.16, 0.14, 0.22, 0.25, 0.0, i as u64 * 7, [0.30, 0.42, 0.20]);
            }
        }
        QProp::Jars => {
            let clay = [0.50, 0.34, 0.22];
            for (k, (ox, oz, r, hh)) in [(0.0f32, 0.0f32, 0.26f32, 0.8f32), (0.45, 0.2, 0.2, 0.6), (-0.1, 0.42, 0.17, 0.5)].iter().enumerate() {
                let c = base + fwd * (ox * w) + side * (oz * d);
                b.column(c, r * 0.7, *r, hh * 0.55, 10, palette::scale(clay, 0.9 + k as f32 * 0.05));
                b.column(c + Vec3::Y * (hh * 0.55), *r, r * 0.5, hh * 0.45, 10, palette::scale(clay, 0.9 + k as f32 * 0.05));
            }
        }
        QProp::Oven => {
            b.dome(base, w * 0.5, d * 0.5, h * 0.85, 0.1, 0.0, 3, stone);
            b.block(base + fwd * (w * 0.35) + Vec3::Y * 0.28, 0.2, 0.45, 0.4, rot, DOOR_DARK);
            b.column(base - fwd * (w * 0.2) + Vec3::Y * (h * 0.7), 0.12, 0.1, h * 0.5, 6, dressed);
            gl.smoke(base - fwd * (w * 0.2) + Vec3::Y * (h * 1.2), 0.15);
        }
        QProp::Chimney => {
            // A tapering tower, a flue mouth glowing near the top, smoke out of it.
            b.column(base, w * 0.72, w * 0.42, h, 4, stone);
            b.block(base + Vec3::Y * h, w * 0.75, d * 0.75, 0.35, rot, dressed);
            gl.patch(base + fwd * (w * 0.45) + Vec3::Y * (h * 0.7), 0.5, 1.2, rot, palette::EMBER);
            gl.smoke(base + Vec3::Y * (h + 0.4), 0.5);
        }
        QProp::Pavilion => {
            for (sx, sz) in [(1.0f32, 1.0f32), (1.0, -1.0), (-1.0, -1.0), (-1.0, 1.0)] {
                let c = base + fwd * (sx * w * 0.4) + side * (sz * d * 0.4);
                b.column(c, 0.28, 0.24, h, 8, dressed);
            }
            b.block(base + Vec3::Y * h, w + 0.4, d + 0.4, 0.35, rot, dressed);
            b.stick(base + Vec3::Y * (h + 0.35), base + Vec3::Y * (h + 1.0), 0.12, palette::METAL_GOLD);
            disc(b, base + Vec3::Y * (h + 1.8), 0.9, rot, palette::METAL_GOLD);
            disc(b, base + Vec3::Y * (h + 1.8) - fwd * 0.02, 0.9, rot + std::f32::consts::PI, palette::METAL_GOLD);
            for i in 0..8 {
                let a = i as f32 / 8.0 * std::f32::consts::TAU;
                let dir = (side * a.cos() + Vec3::Y * a.sin()).normalize();
                b.stick(base + Vec3::Y * (h + 1.8) + dir * 0.9, base + Vec3::Y * (h + 1.8) + dir * 1.25, 0.06, palette::METAL_GOLD);
            }
            // The brazier: a stone bowl on a stem, its fire kept.
            b.column(base, 0.2, 0.16, 0.7, 6, dressed);
            b.column(base + Vec3::Y * 0.7, 0.3, 0.42, 0.35, 10, palette::scale(dressed, 0.9));
            gl.flame(base + Vec3::Y * 1.05, 0.22, 0.45);
        }
        QProp::Arch => {
            // A rounded-headed opening, a lamp glowing inside it.
            let c = base + fwd * 0.05;
            let r = w * 0.5;
            b.block(c + Vec3::Y * ((h - r) * 0.5), 0.02, w, h - r, rot, DOOR_DARK);
            disc(b, c + Vec3::Y * (h - r), r, rot, DOOR_DARK);
            gl.patch(c - fwd * 0.3 + Vec3::Y * (h * 0.45), 0.5, 0.6, rot, palette::WINDOW);
        }
        QProp::Post => {
            b.stick(base, base + Vec3::Y * h, w * 0.5, WOOD_DARK);
            b.block(base + Vec3::Y * (h * 0.7), 1.1, 0.12, 0.12, rot, WOOD);
        }
        QProp::Gate => {
            // Two square stone posts and iron bars between them.
            for sgn in [-1.0f32, 1.0] {
                b.block(base + side * (sgn * (w * 0.5 + 0.25)), 0.5, 0.5, h + 0.5, rot, dressed);
            }
            let bars = (w / 0.18).max(3.0) as usize;
            for i in 0..=bars {
                let c = base + side * ((i as f32 / bars as f32 - 0.5) * w);
                b.stick(c, c + Vec3::Y * h, 0.025, iron);
            }
            for yy in [0.3, h * 0.55, h - 0.15] {
                b.stick(base - side * (w * 0.5) + Vec3::Y * yy, base + side * (w * 0.5) + Vec3::Y * yy, 0.03, iron);
            }
        }
        QProp::Railing => {
            // Along the length `w` (centred on `base`), posts a metre apart.
            let n = (w / 1.0).round().max(1.0) as usize;
            for i in 0..=n {
                let c = base + fwd * ((i as f32 / n as f32 - 0.5) * w);
                b.stick(c, c + Vec3::Y * h, 0.03, iron);
            }
            b.stick(base - fwd * (w * 0.5) + Vec3::Y * h, base + fwd * (w * 0.5) + Vec3::Y * h, 0.03, iron);
            b.stick(base - fwd * (w * 0.5) + Vec3::Y * (h * 0.5), base + fwd * (w * 0.5) + Vec3::Y * (h * 0.5), 0.02, iron);
        }
    }
    let _ = d;
}


fn qotiro(b: &mut Builder, gl: &mut Builder, v: &Variant, f: &Frame, size: f32, sink: f32, sign: Option<signs::Picto>) {
    if let Some(form) = form_of(v) {
        return qotiro_form(b, gl, v, form, f, size, sink);
    }
    // Island stone for every Qotiro building, not sandstone (Laz).
    let col = palette::QUARRIED;
    let cap = palette::scale(palette::QUARRIED, 1.25);
    let (dx, dy) = door_spot(v);
    if v.kind == BuildingKind::QotiroTemple {
        // The temple-fortress: three great steps, a tower and a gold crown.
        let mut y = 0.0;
        for (s, h) in [(1.0, 7.0 + sink), (0.68, 8.0), (0.40, 3.0)] {
            b.block(f.at(0.0, 0.0, y), f.hx * 2.0 * s, f.hy * 2.0 * s, h, f.rot, col);
            y += h;
        }
        b.block(f.at(0.0, 0.0, y), size * 0.16, size * 0.2, 6.0, f.rot, col);
        b.column(f.at(0.0, 0.0, y + 6.0), size * 0.06, 0.0, 3.0, 8, palette::METAL_GOLD);
        gl.patch(f.at(0.0, 0.0, y + 9.5), 2.6, 2.6, f.rot, palette::METAL_GOLD);
        let door = f.at(1.0, dy, sink + 1.8) + f.fwd() * 0.05;
        b.patch(door, 3.0, 3.6, f.rot, DOOR_DARK);
        if let Some(p) = sign {
            let side = if dy < -0.05 { -1.0 } else { 1.0 };
            let sy = (dy + side * (1.5 + 0.35 + 1.3) / f.hy).clamp(-0.85, 0.85);
            signs::qotiro_plaque(b, gl, p, f.at(1.0, sy, sink + 2.4), f.rot, 1.4, door);
        }
        for s in [-1.0f32, 1.0] {
            gl.patch(f.at(1.0, s * 0.55, sink + 4.0) + f.fwd() * 0.05, 0.6, 1.4, f.rot, palette::WINDOW);
        }
        return;
    }
    // A trade's plaque beside the door, toward the nearer corner; the slit
    // windows keep clear of it.
    let wide = if v.key == "qotiro_market" { 2.6 } else { 1.4 };
    let plaque_side = if dy < -0.05 { -1.0 } else { 1.0 };
    let plaque_off = wide * 0.5 + 0.35 + 0.9;
    let by_plaque = |wy: f32| sign.is_some() && ((wy - dy) * plaque_side * f.hy) > 0.0 && ((wy - dy) * plaque_side * f.hy) < plaque_off + 1.1;
    // Tier heights (the first takes up the slope's sink) and how much each
    // steps in.
    let heights: Vec<f32> = match v.key {
        "qotiro_mess" => vec![3.0, 1.6],
        _ => (0..v.storeys).map(|_| 3.5).collect(),
    };
    let shrink = |i: usize| -> f32 {
        match v.key {
            "qotiro_mess" => [1.0, 0.82][i.min(1)],
            _ => [1.0, 0.72, 0.52][i.min(2)],
        }
    };
    let mut y = 0.0;
    for (i, &h0) in heights.iter().enumerate() {
        let h = h0 + if i == 0 { sink } else { 0.0 };
        let s = shrink(i);
        // Each tier steps back from the front, leaving a terrace.
        let cx = -(1.0 - s) * 0.6;
        if v.key == "qotiro_court" && i == 0 {
            // Rooms round an open court: a ring of four ranges.
            let inner = 0.45;
            let band_x = (1.0 - inner) * f.hx;
            for sx in [-1.0f32, 1.0] {
                b.block(f.at(sx * (1.0 + inner) * 0.5, 0.0, y), band_x, f.hy * 2.0, h, f.rot, col);
            }
            for sy in [-1.0f32, 1.0] {
                b.block(f.at(0.0, sy * (1.0 + inner) * 0.5, y), f.hx * 2.0 * inner, (1.0 - inner) * f.hy, h, f.rot, col);
            }
            b.block(f.at(0.0, 0.0, y - 0.05), f.hx * 2.0 * inner, f.hy * 2.0 * inner, 0.1, f.rot, palette::scale(palette::QUARRIED, 1.4));
            y += h;
            continue;
        }
        if v.key == "qotiro_court" && i > 0 {
            // The upper floor only over the back range.
            b.block(f.at(-0.72, 0.0, y), f.hx * 0.56, f.hy * 2.0 * 0.9, h, f.rot, col);
            b.block(f.at(-0.72, 0.0, y + h), f.hx * 0.6, f.hy * 2.0 * 0.94, 0.2, f.rot, cap);
            for k in 0..3 {
                gl.patch(f.at(-0.44, (k as f32 - 1.0) * 0.5, y + h * 0.55) + f.fwd() * 0.03, 0.4, 0.9, f.rot, palette::WINDOW);
            }
            y += h;
            continue;
        }
        b.block(f.at(cx, 0.0, y), f.hx * 2.0 * s, f.hy * 2.0 * s, h, f.rot, col);
        b.block(f.at(cx, 0.0, y + h), f.hx * 2.0 * s + 0.3, f.hy * 2.0 * s + 0.3, 0.22, f.rot, cap);
        // Slit windows along each tier's front.
        let across = f.hy * 2.0 * s;
        let per = if v.key == "qotiro_quarters" { 1.8 } else { 3.2 };
        let n = ((across / per) as usize).max(1);
        let front_x = cx + s;
        for k in 0..n {
            let wy = ((k as f32 + 0.5) / n as f32 * 2.0 - 1.0) * s * 0.85;
            if i == 0 && ((wy - dy).abs() * f.hy < 1.3 || by_plaque(wy)) {
                continue;
            }
            gl.patch(f.at(front_x, wy, y + h * 0.58) + f.fwd() * 0.03, 0.4, 1.0, f.rot, palette::WINDOW);
        }
        y += h;
    }
    let roof = y;
    // The door: a tall dark opening on the ground floor's front.
    b.patch(f.at(dx, dy, sink + 1.2) + f.fwd() * 0.04, wide, 2.4, f.rot, DOOR_DARK);
    b.block(f.at(dx, dy, sink - 0.2) + f.fwd() * 0.5, 1.0, wide + 0.6, 0.35, f.rot, cap);
    if let Some(p) = sign {
        let sy = (dy + plaque_side * plaque_off / f.hy).clamp(-0.85, 0.85);
        signs::qotiro_plaque(b, gl, p, f.at(1.0, sy, sink + 1.85), f.rot, 1.0, f.at(dx, dy, sink + 1.2));
    }
    match v.key {
        "qotiro_workyard" => {
            // A chimney stack at the back, smoking, and a kiln in the yard.
            let c = f.at(-0.75, 0.7, 0.0);
            b.block(c, 1.1, 1.1, roof + 2.8, f.rot, palette::scale(col, 0.85));
            gl.smoke(c + Vec3::Y * (roof + 2.8), 0.4);
            let k = f.at(0.4, 1.0, sink) + f.side() * 2.2;
            b.column(k, 1.3, 0.45, 2.1, 12, BRICK);
            gl.patch(k + f.fwd() * 1.12 + Vec3::Y * 0.5, 0.6, 0.5, f.rot, palette::EMBER);
            // Firewood stacked by the wall.
            for j in 0..4 {
                let a = f.at(0.3, -1.0, sink + 0.15 + j as f32 * 0.22) - f.side() * 0.6;
                b.stick(a - f.fwd() * 1.2, a + f.fwd() * 1.2, 0.2, WOOD);
            }
        }
        "qotiro_market" => {
            // Striped awnings along both long sides over stalls.
            for s in [-1.0f32, 1.0] {
                let n = ((f.hx * 1.6) / 2.4) as usize;
                for k in 0..n.max(1) {
                    let x = ((k as f32 + 0.5) / n.max(1) as f32 * 2.0 - 1.0) * 0.8;
                    let edge = f.at(x, s, sink + 2.8);
                    let out = f.side() * s;
                    let down = (out * 1.8 - Vec3::Y * 0.6).normalize();
                    let c = edge + down * 0.95;
                    obox(b, c, f.fwd() * (f.hx * 0.8 / n.max(1) as f32 - 0.05), down.cross(f.fwd()).normalize() * 0.03, down * 0.95, CLOTHS[k % 4]);
                    let post = edge + out * 1.75;
                    b.stick(vec3(post.x, f.y + sink, post.z), vec3(post.x, post.y - 0.55, post.z), 0.08, WOOD);
                    b.block(f.at(x, s, sink) + out * 0.9, 1.6, 0.7, 0.9, f.rot, WOOD_LIGHT);
                }
            }
        }
        "qotiro_mess" => {
            // A smoke vent on the roof over the kitchen.
            let c = f.at(-0.6, -0.3, roof);
            b.block(c, 1.3, 1.3, 0.9, f.rot, palette::scale(col, 0.8));
            b.block(c + Vec3::Y * 0.9, 1.7, 1.7, 0.15, f.rot, cap);
            gl.patch(c + Vec3::Y * 0.5 + f.fwd() * 0.66, 0.6, 0.3, f.rot, palette::EMBER);
            // Benches out front.
            for s in [-1.0f32, 1.0] {
                b.block(f.at(1.0, s * 0.55, sink) + f.fwd() * 1.0, 0.4, 2.4, 0.45, f.rot, WOOD);
            }
        }
        "qotiro_island_hall" => {
            // A small gold sun disc crowning the hall.
            let c = f.at(-0.1, 0.0, roof + 0.2);
            b.block(c, 0.8, 0.8, 0.9, f.rot, cap);
            b.stick(c + Vec3::Y * 0.9, c + Vec3::Y * 1.5, 0.12, palette::METAL_GOLD);
            disc(b, c + Vec3::Y * 2.3, 0.85, f.rot, palette::METAL_GOLD);
            disc(b, c + Vec3::Y * 2.3 - f.fwd() * 0.02, 0.85, f.rot + std::f32::consts::PI, palette::METAL_GOLD);
            for k in 0..8 {
                let a = k as f32 / 8.0 * std::f32::consts::TAU;
                let d = f.side() * a.cos() + Vec3::Y * a.sin();
                b.stick(c + Vec3::Y * 2.3 + d * 0.95, c + Vec3::Y * 2.3 + d * 1.35, 0.08, palette::METAL_GOLD);
            }
        }
        _ => {}
    }
}

fn horaro(b: &mut Builder, gl: &mut Builder, v: &Variant, f: &Frame, seed: u64) {
    // Stone pillars down into the water (or the ground), and the deck.
    let round = v.shape == Shape::Round;
    let n = 5;
    for i in 0..n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        let p = f.at(a.cos() * 0.55, a.sin() * 0.55, -2.0);
        b.column(p, 0.45, 0.35, DECK + 2.0, 6, palette::STONE);
    }
    let deck = f.at(0.0, 0.0, DECK);
    if round {
        drum(b, deck, f.hx * 0.95, f.hy * 0.95, 0.3, 1.0, f.rot, 18, palette::TIMBER);
    } else {
        b.block(deck, f.hx * 1.9, f.hy * 1.9, 0.3, f.rot, palette::TIMBER);
    }
    let floor = DECK + 0.3;
    match v.key {
        "horaro_twin" => {
            for s in [-1.0f32, 1.0] {
                dome(b, f.at(0.0, s * 0.45, floor), f.hx * 0.6, f.hy * 0.42, f.hy * 0.55, f.rot, 0.04, 0.0, seed ^ s.to_bits() as u64, palette::WEAVE);
            }
            b.block(f.at(0.0, 0.0, floor), 1.4, f.hy * 0.5, 1.6, f.rot, palette::scale(palette::WEAVE, 1.2));
        }
        "horaro_hull" => {
            // An upturned boat's hull, planked, with its keel along the top.
            dome(b, f.at(0.0, 0.0, floor), f.hx * 0.88, f.hy * 0.85, f.hy * 1.1, f.rot, 0.0, 0.14, seed, [0.32, 0.24, 0.17]);
            let top = f.y + floor + f.hy * 1.1;
            b.stick(f.at(-0.95, 0.0, floor + f.hy * 0.4).with_y(top - 0.4), f.at(0.95, 0.0, floor + f.hy * 0.4).with_y(top - 0.4), 0.22, WOOD_DARK);
            b.stick(f.at(0.95, 0.0, floor), f.at(1.0, 0.0, floor).with_y(top - 0.3), 0.22, WOOD_DARK);
        }
        "horaro_net_loft" => {
            // A timber loft with a pitched roof, nets drying on a frame.
            b.block(f.at(-0.15, 0.0, floor), f.hx * 1.2, f.hy * 1.5, 2.2, f.rot, palette::scale(palette::TIMBER, 1.15));
            for s in [-1.0f32, 1.0] {
                let edge = f.at(-0.15, s * 0.8, floor + 2.2);
                let ridge = f.at(-0.15, 0.0, floor + 3.4);
                let c = (edge + ridge) * 0.5;
                let up = (ridge - edge) * 0.5;
                obox(b, c + f.side() * (s * 0.1), f.fwd() * (f.hx * 0.66), up.cross(f.fwd()).normalize() * 0.06, up * 1.12, palette::WEAVE);
            }
            let net = f.at(0.72, 0.0, floor);
            for k in 0..2 {
                let s = if k == 0 { -1.0 } else { 1.0 };
                b.stick(net + f.side() * (s * f.hy * 0.8), net + f.side() * (s * f.hy * 0.8) + Vec3::Y * 2.0, 0.09, WOOD);
            }
            b.stick(net - f.side() * (f.hy * 0.8) + Vec3::Y * 2.0, net + f.side() * (f.hy * 0.8) + Vec3::Y * 2.0, 0.08, WOOD);
            for k in 0..9 {
                let x = (k as f32 / 8.0 * 2.0 - 1.0) * f.hy * 0.75;
                b.stick(net + f.side() * x + Vec3::Y * 2.0, net + f.side() * x + Vec3::Y * 0.4, 0.03, [0.30, 0.34, 0.26]);
            }
            for k in 0..4 {
                let y = 0.6 + k as f32 * 0.4;
                b.stick(net - f.side() * (f.hy * 0.75) + Vec3::Y * y, net + f.side() * (f.hy * 0.75) + Vec3::Y * y, 0.03, [0.30, 0.34, 0.26]);
            }
        }
        _ => {
            dome(b, f.at(0.12, 0.0, floor), f.hx * 0.62, f.hy * 0.55, f.hx * 0.62, f.rot, 0.04, 0.0, seed, palette::WEAVE);
        }
    }
    // A lamp in the doorway.
    gl.patch(f.at(0.62, 0.0, DECK + 1.2) + f.fwd() * 0.1, 0.6, 0.6, f.rot, palette::WINDOW);
}

// ---- Inside --------------------------------------------------------------------

/// A building seen from inside: floor, outer walls cut away low (a gap at the
/// door), inner walls with their doorways, and the furniture.
pub fn interior(b: &mut Builder, gl: &mut Builder, w: &World, d: &Door, on_ground: &dyn Fn(V2) -> f32) {
    let v = d.variant();
    let (floor, low) = floor_height(d, on_ground);
    let f = Frame { c: d.centre, rot: d.rot, hx: d.half.x, hy: d.half.y, y: floor };
    let (wall_col, floor_col, inner_col) = match v.kind {
        BuildingKind::RoduroHome => (palette::STONE, [0.30, 0.28, 0.26], [0.50, 0.50, 0.50]),
        _ => (palette::QUARRIED, [0.30, 0.28, 0.26], [0.45, 0.44, 0.43]),
    };
    // The floor: level at the highest ground under the outline, a slab
    // reaching down below the lowest, so no ground shows through on a slope.
    let deep = floor + 0.05 - (low - 0.3);
    if let Some(pts) = d.outline() {
        let lo = Frame { y: low - 0.3, ..f };
        let ring = inset(&lo, pts, 0.0);
        band(b, &lo, &ring, &ring, 0.0, deep, 0.0, floor_col);
        cap(b, &lo, &ring, deep, floor_col);
    } else if d.round {
        drum(b, to3(d.centre, low - 0.3), f.hx, f.hy, deep, 1.0, f.rot, 30, floor_col);
    } else {
        b.block(to3(d.centre, low - 0.3), f.hx * 2.0, f.hy * 2.0, deep, f.rot, floor_col);
    }
    // Outer walls, cut away, with a gap where the door is.
    let face = d.outside.sub(f.dir().scale(1.2));
    let gap = 0.8;
    let mut ring: Vec<V2> = Vec::new();
    if let Some(pts) = d.outline() {
        let corners: Vec<V2> = inset(&f, pts, 0.17).into_iter().map(|q| fm(&f, q)).collect();
        for k in 0..corners.len() {
            let (a, z) = (corners[k], corners[(k + 1) % corners.len()]);
            let n = ((a.dist(z) / 0.8) as usize).max(1);
            for j in 0..n {
                ring.push(a.lerp(z, j as f32 / n as f32));
            }
        }
        ring.push(ring[0]);
    } else if d.round {
        let n = (((f.hx + f.hy) * std::f32::consts::PI / 0.8) as usize).clamp(24, 72);
        let inset = |r: f32| 1.0 - 0.17 / r;
        for k in 0..=n {
            let a = k as f32 / n as f32 * std::f32::consts::TAU;
            ring.push(f.p(a.cos() * inset(f.hx), a.sin() * inset(f.hy)));
        }
    } else {
        let (ix, iy) = (1.0 - 0.17 / f.hx, 1.0 - 0.17 / f.hy);
        let corners = [(ix, iy), (ix, -iy), (-ix, -iy), (-ix, iy), (ix, iy)];
        for c in corners.windows(2) {
            let (a, z) = (f.p(c[0].0, c[0].1), f.p(c[1].0, c[1].1));
            let n = ((a.dist(z) / 0.8) as usize).max(1);
            for k in 0..n {
                ring.push(a.lerp(z, k as f32 / n as f32));
            }
        }
        ring.push(ring[0]);
    }
    // (From below the lowest ground, so they meet it all the way round.)
    for seg in ring.windows(2) {
        if seg[0].lerp(seg[1], 0.5).dist(face) < gap {
            continue;
        }
        wall(b, seg[0], seg[1], low - 0.3, 0.34, floor + CUT - (low - 0.3), wall_col);
    }
    // Steps up to the door where the floor stands above the ground outside.
    let rise = floor - on_ground(d.outside);
    if rise > 0.12 {
        let steps = ((rise / 0.3).ceil() as usize).clamp(1, 6);
        for k in 0..steps {
            let top = floor - rise * k as f32 / steps as f32;
            let at = face.add(f.dir().scale(0.35 + 0.4 * k as f32));
            b.block(to3(at, low - 0.3), 0.4, gap * 2.0 + 0.5, top - (low - 0.3), f.rot, palette::scale(wall_col, 0.9));
        }
    }
    // The door frame: two posts and a lintel, so the way in reads.
    let across = f.side_v2();
    for s in [-1.0f32, 1.0] {
        let p = face.add(across.scale(s * (gap + 0.05)));
        b.block(to3(p, floor - 0.1), 0.36, 0.28, 2.3, f.rot, palette::scale(wall_col, 0.85));
    }
    b.stick(to3(face.add(across.scale(-gap - 0.2)), floor + 2.2), to3(face.add(across.scale(gap + 0.2)), floor + 2.2), 0.3, palette::scale(wall_col, 0.85));
    // Inner walls, lower and paler, with posts either side of each doorway.
    let pieces = d.wall_pieces();
    for &(a, z) in &pieces {
        wall(b, a, z, floor - 0.1, 0.2, CUT_INNER + 0.1, inner_col);
    }
    if let Some(form) = form_of(v) {
        stair(b, &f, form);
        // The yard stays as it is outside when the house is cut open.
        if let (Some(y), Some(look)) = (v.yard, form.yard.as_ref()) {
            let ground = Frame { y: on_ground(d.centre), ..f };
            yard(b, gl, v, y, look, &ground, form.tiers[0].0, if form.qotiro.is_some() { palette::scale(palette::QUARRIED, 1.15) } else { palette::scale(palette::STONE, 0.78) }, d.id.1 as u64, form.qotiro.is_some());
        }
    }
    for g in d.doorways() {
        for &(a, z) in &pieces {
            for e in [a, z] {
                if e.dist(g) < layout::DOORWAY * 0.75 {
                    let dd = z.sub(a);
                    b.block(to3(e, floor), 0.24, 0.26, 1.9, dd.y.atan2(dd.x), palette::scale(inner_col, 0.8));
                }
            }
        }
    }
    // Furniture.
    for (k, p) in v.furniture.iter().enumerate() {
        let at = f.at(p.at.0, p.at.1, 0.05);
        furniture(b, gl, p.what, at, d.rot + p.rot, k);
    }
    sleepers(b, gl, w, d, &f);
}

// ---- The floor's height ---------------------------------------------------------

/// How far the floor's drawn surface stands over `floor_height` (the slab's
/// top, where furniture and containers stand): people and things lying
/// about indoors stand on it.
pub const FLOOR_TOP: f32 = 0.05;

/// The level a building's floor is drawn at, and the lowest drawn ground
/// under its outline: the highest of the ground at its middle, round its
/// outline and halfway out, so the floor never dips under the ground.
/// Furniture, extra bedrolls, a lodger's corner and the containers all
/// stand on it.
pub fn floor_height(d: &Door, on_ground: &dyn Fn(V2) -> f32) -> (f32, f32) {
    let mut pts = vec![d.centre];
    if let Some(outline) = d.outline() {
        // Its own outline: every other corner, and halfway in to each.
        for &p in outline.iter().step_by(2) {
            pts.push(d.to_world(p));
            pts.push(d.to_world((p.0 * 0.5, p.1 * 0.5)));
        }
        let hs: Vec<f32> = pts.into_iter().map(on_ground).collect();
        return (hs.iter().copied().fold(f32::MIN, f32::max), hs.iter().copied().fold(f32::MAX, f32::min));
    }
    let n = 16;
    for k in 0..n {
        let a = k as f32 / n as f32 * std::f32::consts::TAU;
        let (x, y) = if d.round {
            (a.cos(), a.sin())
        } else {
            // Round the box's edge: corners and points between.
            let (c, s) = (a.cos(), a.sin());
            let m = c.abs().max(s.abs());
            (c / m, s / m)
        };
        pts.push(d.to_world((x, y)));
        pts.push(d.to_world((x * 0.5, y * 0.5)));
    }
    let hs: Vec<f32> = pts.into_iter().map(on_ground).collect();
    let hi = hs.iter().copied().fold(f32::MIN, f32::max);
    let lo = hs.iter().copied().fold(f32::MAX, f32::min);
    (hi, lo)
}

// ---- Beds for the household, and a lodger's corner ---------------------------

const PARCHMENT: [Rgb; 3] = [[0.88, 0.83, 0.68], [0.82, 0.78, 0.64], [0.90, 0.86, 0.74]];
const VIOLET: Rgb = [0.55, 0.50, 0.66];
const LEATHER: Rgb = [0.42, 0.27, 0.15];

/// A building's extra bedrolls and lodger's corner, as the sim lays them out
/// (`World::sleeping_plan`: clear of belongings already lying there).
fn plan(w: &World, d: &Door) -> (Vec<(V2, f32)>, Option<layout::Corner>) {
    w.sleeping_plan(d)
}

/// Where a Ṭaḍoro lodger's corner is in this building, if one lodges here
/// (for framing screenshots).
pub fn lodger_corner(w: &World, d: &Door) -> Option<V2> {
    plan(w, d).1.map(|c| c.at)
}

/// How many sleeping places are drawn in this building (for screenshots).
pub fn sleeping_places(w: &World, d: &Door) -> usize {
    let (extras, corner) = plan(w, d);
    let own = d.variant().furniture.iter().filter(|p| matches!(p.what, Furn::Bed | Furn::Bedroll)).count();
    own + extras.len() + corner.is_some_and(|c| c.on.is_none()) as usize
}

/// The extra bedrolls for residents beyond the variant's beds, and a Ṭaḍoro
/// lodger's corner if one lives here (`layout::sleeping_plan`).
fn sleepers(b: &mut Builder, gl: &mut Builder, w: &World, d: &Door, f: &Frame) {
    let k0 = d.variant().furniture.len();
    let (extras, corner) = plan(w, d);
    for (i, &(at, rot)) in extras.iter().enumerate() {
        furniture(b, gl, Furn::Bedroll, to3(at, f.y + 0.05), rot, k0 + i);
    }
    if let Some(c) = corner {
        if c.on.is_none() {
            furniture(b, gl, Furn::Bedroll, to3(c.at, f.y + 0.05), c.rot, k0 + extras.len());
        }
        tadoro_corner(b, gl, f, &c);
    }
}

/// A Ṭaḍoro lodger's corner round one sleeping place, every piece where the
/// layout put it: a rug, violet and grey-blue cloth draped over the bed with
/// a flap down the room side, papers, ink and candles, a satchel, a cushion
/// if there's room, parchment pinned on the wall and a lantern over it.
fn tadoro_corner(b: &mut Builder, gl: &mut Builder, f: &Frame, c: &layout::Corner) {
    let (bw, bd, _) = if c.bed { Furn::Bed.size() } else { Furn::Bedroll.size() };
    let y0 = f.y + 0.05;
    let q = Put::new(to3(c.at, y0), c.rot);
    let flat = |b: &mut Builder, r: &layout::Rect, inset: f32, y: f32, h: f32, col: Rgb| {
        if r.hw > inset && r.hd > inset {
            b.block(to3(r.c, y0 + y), (r.hw - inset) * 2.0, (r.hd - inset) * 2.0, h, r.rot, col);
        }
    };
    // A rug under it all: a violet border round a grey-blue field, a pale
    // stripe down the middle.
    flat(b, &c.rug, 0.0, 0.0, 0.02, palette::scale(VIOLET, 0.72));
    flat(b, &c.rug, 0.12, 0.0, 0.026, palette::scale(palette::TENT, 0.8));
    if c.rug.hw > 0.3 {
        b.block(to3(c.rug.c, y0), (c.rug.hw - 0.25) * 2.0, 0.12, 0.032, c.rug.rot, palette::scale(LINEN, 0.9));
    }
    // The drape over the body (the pillow left clear), its flap down the
    // room side to the floor, and a folded grey-blue blanket at the foot.
    let (top, thick) = if c.bed { (0.46, 0.06) } else { (0.06, 0.1) };
    let [u, wv] = c.drape.axes();
    let rel = c.drape.c.sub(c.at);
    let (mx, my) = (rel.x * u.x + rel.y * u.y, rel.x * wv.x + rel.y * wv.y);
    let len = c.drape.hw * 2.0;
    q.bx(b, mx, my - c.side * 0.02, top, len, c.drape.hd * 2.0 - 0.04, thick, VIOLET);
    let flap_h = top + thick - 0.02;
    let edge = my + c.side * (c.drape.hd - 0.02);
    q.bx(b, mx, edge, 0.0, len * 0.95, 0.04, flap_h, palette::scale(VIOLET, 0.88));
    q.bx(b, mx + len * 0.18, edge + c.side * 0.01, 0.0, len * 0.22, 0.04, flap_h * 0.85, palette::TENT);
    q.bx(b, -bw * 0.5 + 0.28, 0.0, top + thick, 0.46, bd * 0.85, 0.1, palette::TENT);
    // A bolster across the head.
    let bolster = top + thick + 0.08;
    b.stick(q.p(bw * 0.5 - 0.16, -bd * 0.45, bolster), q.p(bw * 0.5 - 0.16, bd * 0.45, bolster), 0.1, palette::scale(VIOLET, 1.2));
    // A floor cushion.
    if let Some(r) = c.cushion {
        b.block(to3(r.c, y0), 0.5, 0.5, 0.14, r.rot, palette::scale(palette::TENT, 1.1));
        b.block(to3(r.c, y0 + 0.14), 0.4, 0.4, 0.03, r.rot, palette::scale(VIOLET, 0.9));
    }
    // Papers in a little stack, an ink pot and quill, a dish of candles.
    let [pu, pw] = c.papers.axes();
    for (k, (dx, dz, rz)) in [(-0.03f32, -0.02f32, 0.0f32), (0.02, 0.02, 0.25), (-0.01, 0.0, -0.15)].into_iter().enumerate() {
        let at = c.papers.c.add(pu.scale(dx)).add(pw.scale(dz));
        b.block(to3(at, y0 + 0.03 * k as f32), 0.32, 0.24, 0.03, c.papers.rot + std::f32::consts::FRAC_PI_2 * (c.papers.hw < c.papers.hd) as u8 as f32 + rz, PARCHMENT[k]);
    }
    let pot = to3(c.ink.c, y0);
    b.column(pot, 0.06, 0.045, 0.09, 8, [0.10, 0.10, 0.14]);
    b.stick(pot + Vec3::Y * 0.06, pot + vec3(0.06, 0.3, 0.04), 0.02, [0.86, 0.84, 0.80]);
    let candle = to3(c.candles.c, y0);
    b.column(candle, 0.13, 0.13, 0.03, 10, palette::scale(LEATHER, 0.8));
    for (dx, dz, h) in [(0.0f32, 0.0f32, 0.16f32), (0.07, 0.05, 0.11), (-0.05, 0.06, 0.08)] {
        let at = candle + vec3(dx, 0.0, dz);
        b.column(at + Vec3::Y * 0.03, 0.03, 0.03, h, 6, LINEN);
        gl.flame(at + Vec3::Y * (0.04 + h), 0.025, 0.07);
    }
    // A leather satchel, its strap looped up.
    let sat = to3(c.satchel.c, y0);
    let srot = c.satchel.rot + if c.satchel.hw < c.satchel.hd { std::f32::consts::FRAC_PI_2 } else { 0.0 };
    b.block(sat, 0.42, 0.18, 0.28, srot, LEATHER);
    b.block(sat + Vec3::Y * 0.2, 0.44, 0.2, 0.09, srot, palette::scale(LEATHER, 0.82));
    let ax = vec3(srot.cos(), 0.0, srot.sin()) * 0.17;
    b.stick(sat - ax + Vec3::Y * 0.28, sat + Vec3::Y * 0.5, 0.03, palette::scale(LEATHER, 0.7));
    b.stick(sat + Vec3::Y * 0.5, sat + ax + Vec3::Y * 0.28, 0.03, palette::scale(LEATHER, 0.7));
    // Parchment pinned on the wall, each sheet on the wall's face.
    for (k, ((p, inward), (dy, sw, sh))) in c.sheets.iter().zip([(0.02f32, 0.25f32, 0.35f32), (-0.04, 0.3, 0.24), (0.05, 0.22, 0.32)]).enumerate() {
        let face = inward.y.atan2(inward.x);
        let n3 = vec3(inward.x, 0.0, inward.y);
        let at = to3(p.add(inward.scale(0.02 + 0.005 * k as f32)), f.y + c.up + dy);
        b.patch(at, sw, sh, face, PARCHMENT[k]);
        // A dark pin at the top.
        b.block(at + Vec3::Y * (sh * 0.5 - 0.04) + n3 * 0.01, 0.025, 0.025, 0.025, face, [0.15, 0.12, 0.10]);
        // Each sheet's a diagram: a dark circle (one with a cross through
        // it and a dot), or lines of notes.
        let ink = [0.12, 0.10, 0.12];
        let r = sw.min(sh) * 0.36;
        let mid = at - Vec3::Y * 0.01 + n3 * 0.006;
        let right = vec3(-face.sin(), 0.0, face.cos());
        if k != 2 {
            disc(b, mid, r, face, ink);
            disc(b, mid + n3 * 0.003, r * 0.8, face, PARCHMENT[k]);
            if k == 1 {
                obox(b, mid + n3 * 0.006, right * 0.006, Vec3::Y * r, n3 * 0.003, ink);
                obox(b, mid + n3 * 0.006, right * r, Vec3::Y * 0.006, n3 * 0.003, ink);
                disc(b, mid + n3 * 0.01 + right * (r * 0.45) + Vec3::Y * (r * 0.45), r * 0.16, face, ink);
            }
        } else {
            for j in 0..4 {
                let y = sh * 0.25 - j as f32 * sh * 0.15;
                obox(b, mid + n3 * 0.003 + Vec3::Y * y, right * (sw * 0.32), Vec3::Y * 0.007, n3 * 0.003, ink);
            }
        }
    }
    // A small iron lantern hung from a bracket over the middle sheet: a
    // second warm light over the corner.
    let (hang, inward) = c.sheets[1];
    let face = inward.y.atan2(inward.x);
    let n3 = vec3(inward.x, 0.0, inward.y);
    let root = to3(hang, f.y + c.up + 0.75);
    let lamp = root + n3 * 0.3;
    b.stick(root, lamp + n3 * 0.04, 0.025, IRON);
    b.stick(lamp, lamp - Vec3::Y * 0.12, 0.012, IRON);
    let body = lamp - Vec3::Y * 0.32;
    b.block(body + Vec3::Y * 0.2, 0.2, 0.2, 0.04, face, IRON);
    b.block(body - Vec3::Y * 0.02, 0.2, 0.2, 0.04, face, IRON);
    gl.block(body + Vec3::Y * 0.02, 0.15, 0.15, 0.18, face, palette::WINDOW);
}

/// A placement: a spot on the floor and a turn.
struct Put {
    base: Vec3,
    rot: f32,
    fx: Vec3,
    fz: Vec3,
}

impl Put {
    fn new(base: Vec3, rot: f32) -> Put {
        Put { base, rot, fx: vec3(rot.cos(), 0.0, rot.sin()), fz: vec3(-rot.sin(), 0.0, rot.cos()) }
    }
    fn p(&self, x: f32, z: f32, y: f32) -> Vec3 {
        self.base + self.fx * x + self.fz * z + Vec3::Y * y
    }
    /// A block on (x, z), its foot `y` up; `w` along the piece's own x.
    #[allow(clippy::too_many_arguments)]
    fn bx(&self, b: &mut Builder, x: f32, z: f32, y: f32, w: f32, d: f32, h: f32, col: Rgb) {
        b.block(self.p(x, z, y), w, d, h, self.rot, col);
    }
    /// Four legs under a top of `w` by `d`, `h` tall.
    fn legs(&self, b: &mut Builder, w: f32, d: f32, h: f32, col: Rgb) {
        for (sx, sz) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            self.bx(b, sx * (w * 0.5 - 0.07), sz * (d * 0.5 - 0.07), 0.0, 0.08, 0.08, h, col);
        }
    }
}

/// One piece of furniture as a few blocks, standing at `base`, turned `rot`.
fn furniture(b: &mut Builder, gl: &mut Builder, what: Furn, base: Vec3, rot: f32, k: usize) {
    let (w, d, h) = what.size();
    let q = Put::new(base, rot);
    let cloth = CLOTHS[k % CLOTHS.len()];
    match what {
        Furn::Bed => {
            q.bx(b, 0.0, 0.0, 0.0, w, d, h * 0.55, WOOD);
            q.bx(b, 0.0, 0.0, h * 0.55, w * 0.94, d * 0.88, 0.14, LINEN);
            q.bx(b, -w * 0.14, 0.0, h * 0.55 + 0.03, w * 0.62, d * 0.94, 0.15, cloth);
            q.bx(b, w * 0.34, 0.0, h * 0.55 + 0.14, 0.34, d * 0.62, 0.11, [0.93, 0.91, 0.85]);
            q.bx(b, w * 0.5 - 0.04, 0.0, 0.0, 0.08, d, h * 1.7, WOOD_DARK);
        }
        Furn::Bedroll => {
            q.bx(b, -0.1, 0.0, 0.0, w * 0.85, d, 0.07, palette::scale(cloth, 0.85));
            b.stick(q.p(w * 0.42, -d * 0.45, 0.13), q.p(w * 0.42, d * 0.45, 0.13), 0.26, LINEN);
        }
        Furn::Table | Furn::LongTable => {
            q.bx(b, 0.0, 0.0, h - 0.07, w, d, 0.07, WOOD_LIGHT);
            q.legs(b, w, d, h - 0.07, WOOD);
            // Bowls and cups.
            let n = if what == Furn::LongTable { 4 } else { 2 };
            for i in 0..n {
                let x = ((i as f32 + 0.5) / n as f32 * 2.0 - 1.0) * w * 0.35;
                b.column(q.p(x, if i % 2 == 0 { 0.12 } else { -0.15 }, h), 0.12, 0.16, 0.08, 8, [0.62, 0.44, 0.28]);
            }
        }
        Furn::Bench => {
            q.bx(b, 0.0, 0.0, h - 0.06, w, d, 0.06, WOOD_LIGHT);
            for s in [-1.0f32, 1.0] {
                q.bx(b, s * (w * 0.5 - 0.12), 0.0, 0.0, 0.07, d * 0.9, h - 0.06, WOOD);
            }
        }
        Furn::Shelf => {
            for s in [-1.0f32, 1.0] {
                q.bx(b, s * (w * 0.5 - 0.04), 0.0, 0.0, 0.07, d, h, WOOD_DARK);
            }
            let jars = [[0.62, 0.40, 0.22], [0.30, 0.42, 0.50], [0.78, 0.72, 0.60], [0.45, 0.52, 0.30]];
            for (j, y) in [0.05f32, 0.62, 1.2, h - 0.05].into_iter().enumerate() {
                q.bx(b, 0.0, 0.0, y, w, d, 0.05, WOOD_LIGHT);
                if j < 3 {
                    for i in 0..3 {
                        let x = (i as f32 - 1.0) * w * 0.28;
                        b.column(q.p(x, 0.0, y + 0.05), 0.08, 0.07, 0.22 + 0.06 * ((i + j) % 2) as f32, 6, jars[(i + j + k) % 4]);
                    }
                }
            }
        }
        Furn::Hearth => {
            for i in 0..8 {
                let a = i as f32 / 8.0 * std::f32::consts::TAU;
                b.block(q.p(a.cos() * w * 0.42, a.sin() * w * 0.42, 0.0), 0.3, 0.24, 0.24, rot + a, palette::scale(palette::STONE, 0.8));
            }
            b.stick(q.p(-0.3, -0.1, 0.1), q.p(0.3, 0.1, 0.1), 0.1, WOOD_DARK);
            b.stick(q.p(-0.1, 0.3, 0.1), q.p(0.1, -0.3, 0.1), 0.1, WOOD_DARK);
            gl.flame(q.p(0.0, 0.0, 0.0), w * 0.28, 0.32);
        }
        Furn::Workbench => {
            q.bx(b, 0.0, 0.0, h - 0.1, w, d, 0.1, WOOD_LIGHT);
            q.legs(b, w, d, h - 0.1, WOOD);
            q.bx(b, 0.0, 0.0, 0.22, w * 0.9, d * 0.85, 0.05, WOOD);
            q.bx(b, w * 0.2, 0.0, h, 0.5, 0.16, 0.06, STEEL);
            q.bx(b, w * 0.2 + 0.32, 0.0, h, 0.14, 0.14, 0.1, WOOD_DARK);
            q.bx(b, -w * 0.4, d * 0.28, h, 0.22, 0.22, 0.22, IRON);
            b.stick(q.p(-w * 0.1, -d * 0.2, h + 0.03), q.p(-w * 0.1 + 0.4, -d * 0.1, h + 0.03), 0.05, STEEL);
        }
        Furn::Anvil => {
            b.column(q.p(0.0, 0.0, 0.0), 0.24, 0.21, h * 0.55, 8, WOOD);
            q.bx(b, 0.0, 0.0, h * 0.55, w * 0.6, d * 0.55, h * 0.22, IRON);
            q.bx(b, -0.05, 0.0, h * 0.77, w * 0.85, d * 0.7, h * 0.23, IRON);
            b.stick(q.p(w * 0.36, 0.0, h * 0.88), q.p(w * 0.36 + 0.28, 0.0, h * 0.9), 0.1, IRON);
        }
        Furn::Forge => {
            q.bx(b, 0.0, 0.0, 0.0, w, d, h * 0.75, DARK_STONE);
            gl.block(q.p(0.0, 0.0, h * 0.75), w * 0.6, d * 0.55, 0.07, rot, palette::EMBER);
            // A hood on posts, and its flue.
            for s in [-1.0f32, 1.0] {
                b.stick(q.p(-w * 0.4, s * d * 0.4, h * 0.75), q.p(-w * 0.4, s * d * 0.4, h * 0.75 + 0.8), 0.1, IRON);
            }
            b.column(q.p(0.0, 0.0, h * 0.75 + 0.8), w * 0.5, 0.25, 0.6, 4, palette::scale(DARK_STONE, 1.2));
            q.bx(b, 0.0, 0.0, h * 0.75 + 1.35, 0.4, 0.4, 0.8, palette::scale(DARK_STONE, 1.2));
            // Bellows.
            q.bx(b, -w * 0.5 - 0.3, 0.0, 0.3, 0.55, 0.45, 0.24, [0.40, 0.26, 0.16]);
        }
        Furn::Kiln => {
            b.column(q.p(0.0, 0.0, 0.0), w * 0.5, w * 0.24, h, 10, BRICK);
            b.column(q.p(0.0, 0.0, h), 0.17, 0.13, 0.45, 6, palette::scale(BRICK, 0.7));
            gl.patch(q.p(w * 0.47, 0.0, 0.38), 0.45, 0.4, rot, palette::EMBER);
        }
        Furn::Loom => {
            for s in [-1.0f32, 1.0] {
                b.stick(q.p(s * w * 0.5, 0.0, 0.0), q.p(s * w * 0.5, 0.0, h), 0.09, WOOD);
            }
            for y in [0.45f32, h - 0.05] {
                b.stick(q.p(-w * 0.5, 0.0, y), q.p(w * 0.5, 0.0, y), 0.08, WOOD_DARK);
            }
            q.bx(b, 0.0, 0.0, 0.5, w * 0.85, 0.04, h * 0.6, cloth);
            q.bx(b, 0.0, d * 0.6, 0.0, w * 0.6, 0.35, 0.45, WOOD);
        }
        Furn::Counter => {
            q.bx(b, 0.0, 0.0, 0.0, w, d, h - 0.06, WOOD_DARK);
            q.bx(b, 0.0, 0.0, h - 0.06, w * 1.04, d * 1.2, 0.06, WOOD_LIGHT);
            for i in 0..3 {
                let x = (i as f32 - 1.0) * w * 0.3;
                q.bx(b, x, 0.0, h, 0.32, 0.26, 0.16 + 0.06 * (i % 2) as f32, CLOTHS[(i + k) % 4]);
            }
        }
        Furn::Rack => {
            for s in [-1.0f32, 1.0] {
                q.bx(b, s * (w * 0.5 - 0.04), 0.0, 0.0, 0.08, d, h, WOOD);
            }
            for y in [h - 0.1, 0.35] {
                b.stick(q.p(-w * 0.5, 0.0, y), q.p(w * 0.5, 0.0, y), 0.06, WOOD_DARK);
            }
            for i in 0..4 {
                let x = ((i as f32 + 0.5) / 4.0 * 2.0 - 1.0) * w * 0.38;
                b.stick(q.p(x, 0.0, h - 0.1), q.p(x, 0.0, h - 0.75), 0.05, WOOD_DARK);
                q.bx(b, x, 0.0, 0.4, 0.06, 0.18, 0.55, STEEL);
            }
        }
        Furn::GrowerBed => {
            q.bx(b, 0.0, 0.0, 0.0, w, d, h, palette::STONE);
            q.bx(b, 0.0, 0.0, h, w * 0.86, d * 0.8, 0.03, SOIL);
            for i in 0..3 {
                let x = (i as f32 - 1.0) * w * 0.3;
                b.dome(q.p(x, if i == 1 { 0.15 } else { -0.15 }, h + 0.02), 0.2, 0.17, 0.22 + 0.08 * i as f32, 0.2, 0.0, k as u64 * 7 + i as u64, [0.60, 0.63, 0.66]);
            }
        }
        Furn::Altar => {
            q.bx(b, 0.0, 0.0, 0.0, w * 0.88, d * 0.82, h * 0.85, [0.55, 0.54, 0.52]);
            q.bx(b, 0.0, 0.0, h * 0.85, w, d, h * 0.15, palette::METAL_GOLD);
            for s in [-1.0f32, 1.0] {
                gl.flame(q.p(0.0, s * w * 0.38, h), 0.05, 0.18);
            }
            let q2 = std::f32::consts::FRAC_PI_2;
            b.stick(q.p(0.0, d * 0.25, h), q.p(0.0, d * 0.25, h + 0.3), 0.05, palette::METAL_GOLD);
            disc(b, q.p(0.0, d * 0.25, h + 0.6), 0.28, rot + q2, palette::METAL_GOLD);
            disc(b, q.p(0.0, d * 0.25 + 0.01, h + 0.6), 0.28, rot - q2, palette::METAL_GOLD);
        }
        Furn::Screen => {
            for s in [-1.0f32, 1.0] {
                b.stick(q.p(s * w * 0.5, 0.0, 0.0), q.p(s * w * 0.5, 0.0, h), 0.07, WOOD_DARK);
            }
            q.bx(b, 0.0, 0.0, 0.12, w * 0.95, 0.05, h - 0.2, [0.58, 0.50, 0.36]);
        }
    }
}

// ---- Containers -------------------------------------------------------------

/// The containers in a building someone is in, drawn every frame (lids open
/// while being gone through), with a hover point for each.
pub fn containers(b: &mut Builder, gl: &mut Builder, w: &World, d: &Door, on_ground: &dyn Fn(V2) -> f32, picks: &mut Vec<(Vec3, f32, Hover)>) {
    let floor = floor_height(d, on_ground).0 + 0.05;
    let open: Vec<_> = w.squad.members.iter().filter_map(|&m| w.searching_now(m)).collect();
    let mut any = false;
    for c in w.containers_in(d.id) {
        any = true;
        let locked = w.container_locked(c.id);
        let is_open = open.contains(&c.id);
        holder(b, gl, c.what, to3(c.pos, floor), c.rot, d.centre.sub(c.pos), locked, is_open, !c.items.is_empty());
        picks.push((to3(c.pos, floor + c.what.size().2 + 0.15), 2.0, Hover::Container(c.id)));
    }
    // Not laid out yet (nobody has stepped in): drawn shut where they'll be.
    if !any {
        for (_, what, pos, rot) in w.container_spots(d) {
            holder(b, gl, what, to3(pos, floor), rot, d.centre.sub(pos), false, false, false);
        }
    }
}

/// The containers out in the wild (ruins' and lairs' caches, bandit camps'
/// stashes) near the camera, on the ground: drawn every frame like the ones
/// indoors, with a hover point each.
pub fn wild_containers(b: &mut Builder, gl: &mut Builder, w: &World, on_ground: &dyn Fn(V2) -> f32, near: V2, radius: f32, picks: &mut Vec<(Vec3, f32, Hover)>) {
    use gahturiyu_sim::sim::containers::WILD;
    let open: Vec<_> = w.squad.members.iter().filter_map(|&m| w.searching_now(m)).collect();
    for c in w.containers.range((WILD, 0, 0)..=(WILD, u16::MAX, u8::MAX)).map(|(_, c)| c).filter(|c| c.pos.dist(near) <= radius) {
        let base = to3(c.pos, on_ground(c.pos));
        let front = V2::new(-c.rot.sin(), c.rot.cos());
        holder(b, gl, c.what, base, c.rot, front, w.container_locked(c.id), open.contains(&c.id), !c.items.is_empty());
        picks.push((base + Vec3::Y * (c.what.size().2 + 0.15), 2.0, Hover::Container(c.id)));
    }
}

/// One container. Its front is the long side toward `toward` (the room).
#[allow(clippy::too_many_arguments)]
fn holder(b: &mut Builder, gl: &mut Builder, what: Holder, base: Vec3, rot: f32, toward: V2, locked: bool, open: bool, full: bool) {
    let (sw, sd, sh) = what.size();
    let fx = vec3(rot.cos(), 0.0, rot.sin());
    let side = vec3(-rot.sin(), 0.0, rot.cos());
    let front = if side.x * toward.x + side.z * toward.y >= 0.0 { side } else { -side };
    let (hx, hz) = (fx * (sw * 0.5), front * (sd * 0.5));
    let inside = [0.08, 0.06, 0.05];
    match what {
        Holder::Chest => {
            let body = sh * 0.68;
            let wood = [0.46, 0.30, 0.16];
            obox(b, base + Vec3::Y * (body * 0.5), hx, Vec3::Y * (body * 0.5), hz, wood);
            for s in [-1.0f32, 1.0] {
                obox(b, base + fx * (s * sw * 0.3) + Vec3::Y * (body * 0.5), fx * 0.035, Vec3::Y * (body * 0.51), hz * 1.06, [0.34, 0.34, 0.36]);
            }
            let lid_h = (sh - body) * 0.5;
            if open {
                // The lid swung up and back on its hinge, the dark inside showing.
                let hinge = base + Vec3::Y * body - hz;
                let th = 1.85f32;
                let up = front.normalize() * th.cos() + Vec3::Y * th.sin();
                let thick = (front.normalize() * (-th.sin()) + Vec3::Y * th.cos()) * lid_h;
                obox(b, hinge + up * (sd * 0.52) - thick, hx * 1.02, thick, up * (sd * 0.52), [0.33, 0.21, 0.11]);
                b.quad([base + Vec3::Y * (body + 0.01) - hx * 0.92 - hz * 0.88, base + Vec3::Y * (body + 0.01) + hx * 0.92 - hz * 0.88, base + Vec3::Y * (body + 0.01) + hx * 0.92 + hz * 0.88, base + Vec3::Y * (body + 0.01) - hx * 0.92 + hz * 0.88], Vec3::Y, inside);
                if full {
                    gl.block(base + Vec3::Y * (body - 0.02) + fx * 0.15, 0.3, 0.2, 0.06, rot, palette::METAL_GOLD);
                    b.block(base + Vec3::Y * (body - 0.03) - fx * 0.2, 0.3, 0.25, 0.07, rot + 0.4, CLOTHS[1]);
                }
            } else {
                obox(b, base + Vec3::Y * (body + lid_h), hx * 1.03, Vec3::Y * lid_h, hz * 1.05, [0.33, 0.21, 0.11]);
            }
            if locked {
                let plate = base + front * (sd * 0.5 + 0.025) + Vec3::Y * (body - 0.08);
                obox(b, plate, fx * 0.08, Vec3::Y * 0.09, front * 0.02, palette::METAL_GOLD);
                obox(b, plate + front * 0.022, fx * 0.018, Vec3::Y * 0.04, front * 0.004, inside);
            }
        }
        Holder::Crate => {
            let wood = [0.60, 0.47, 0.31];
            let slat = [0.40, 0.29, 0.18];
            let c = base + Vec3::Y * (sh * 0.5);
            obox(b, c, hx, Vec3::Y * (sh * 0.5), hz, wood);
            // Cross braces on each side, and a frame round the edges.
            let y = Vec3::Y * (sh * 0.5);
            for (n, u) in [(hx, hz), (-hx, hz), (hz, hx), (-hz, hx)] {
                let f = c + n * 1.04;
                b.stick(f - u * 0.92 - y * 0.92, f + u * 0.92 + y * 0.92, 0.05, slat);
                b.stick(f - u * 0.92 + y * 0.92, f + u * 0.92 - y * 0.92, 0.05, slat);
                b.stick(f - u - y * 0.95, f + u - y * 0.95, 0.07, slat);
                b.stick(f - u + y * 0.95, f + u + y * 0.95, 0.07, slat);
            }
            if open {
                b.quad([c + y * 1.01 - hx * 0.9 - hz * 0.9, c + y * 1.01 + hx * 0.9 - hz * 0.9, c + y * 1.01 + hx * 0.9 + hz * 0.9, c + y * 1.01 - hx * 0.9 + hz * 0.9], Vec3::Y, inside);
                // The lid leaning against its side.
                let lean = (front * 0.3 + Vec3::Y).normalize();
                obox(b, base + front * (sd * 0.5 + 0.25) + lean * (sh * 0.5), hx, lean * (sh * 0.5), lean.cross(fx).normalize() * 0.03, slat);
            }
        }
        Holder::Cupboard => {
            let wood = [0.40, 0.28, 0.18];
            obox(b, base + Vec3::Y * (sh * 0.5), hx, Vec3::Y * (sh * 0.5), hz, wood);
            obox(b, base + Vec3::Y * (sh + 0.04), hx * 1.06, Vec3::Y * 0.04, hz * 1.12, [0.30, 0.21, 0.13]);
            let face = base + front * (sd * 0.5 + 0.01);
            if open {
                // Both doors swung out, the shelves inside showing.
                b.patch(face + Vec3::Y * (sh * 0.5), sw * 0.9, sh * 0.9, front.z.atan2(front.x), inside);
                for i in 0..3 {
                    b.block(face - front * 0.2 + Vec3::Y * (0.3 + i as f32 * 0.45) + fx * ((i as f32 - 1.0) * 0.2), 0.3, 0.2, 0.14, rot, CLOTHS[i % 4]);
                }
                for s in [-1.0f32, 1.0] {
                    let hinge = face + fx * (s * sw * 0.5);
                    let a = 1.9f32;
                    let out = (fx * (-s) * a.cos() + front.normalize() * a.sin()).normalize();
                    obox(b, hinge + out * (sw * 0.25) + Vec3::Y * (sh * 0.5), out * (sw * 0.25), Vec3::Y * (sh * 0.46), out.cross(Vec3::Y).normalize() * 0.025, wood);
                }
            } else {
                b.stick(face + Vec3::Y * 0.08, face + Vec3::Y * (sh - 0.08), 0.03, inside);
                for s in [-1.0f32, 1.0] {
                    obox(b, face + fx * (s * 0.1) + Vec3::Y * (sh * 0.55), fx * 0.025, Vec3::Y * 0.05, front * 0.03, palette::METAL_GOLD);
                }
            }
            if locked {
                obox(b, face + front * 0.03 + Vec3::Y * (sh * 0.45), fx * 0.06, Vec3::Y * 0.08, front * 0.02, palette::METAL_GOLD);
            }
        }
        Holder::Barrel => {
            let wood = [0.50, 0.34, 0.20];
            let (r0, r1) = (sw * 0.42, sw * 0.5);
            b.column(base, r0, r1, sh * 0.5, 12, wood);
            b.column(base + Vec3::Y * (sh * 0.5), r1, r0, sh * 0.5, 12, wood);
            for y in [0.12f32, 0.88] {
                b.column(base + Vec3::Y * (sh * y - 0.03), r0 + (r1 - r0) * 0.5 + 0.02, r0 + (r1 - r0) * 0.5 + 0.02, 0.06, 12, IRON);
            }
            if open {
                b.column(base + Vec3::Y * (sh - 0.01), r0 * 0.95, r0 * 0.95, 0.015, 12, inside);
                let lean = (front * 0.35 + Vec3::Y).normalize();
                obox(b, base + front * (r1 + 0.06) + lean * (r0 * 1.0), front.cross(Vec3::Y).normalize() * r0, lean * r0, front * 0.025, palette::scale(wood, 0.8));
            } else {
                b.column(base + Vec3::Y * (sh - 0.01), r0 * 0.95, r0 * 0.95, 0.03, 12, palette::scale(wood, 0.75));
            }
            if locked {
                obox(b, base + front * (r1 + 0.02) + Vec3::Y * (sh * 0.7), front.cross(Vec3::Y).normalize() * 0.07, Vec3::Y * 0.08, front * 0.02, palette::METAL_GOLD);
            }
        }
    }
}

// ---- Words for the panels ------------------------------------------------------

/// Whose a container is, for the hover and the loot panel.
pub fn owner_text(w: &World, owner: Owner) -> String {
    match owner {
        Owner::Household(i) => {
            let named = w.society.households.get(i as usize).and_then(|h| h.members.iter().find_map(|&m| w.people[m as usize].name().map(|n| n.to_string())));
            match named {
                Some(n) => format!("{n}'s household"),
                None => "the household here".to_string(),
            }
        }
        Owner::Town(t) => format!("the town of {}", w.settlements.get(t as usize).map(|s| s.name.as_str()).unwrap_or("?")),
        Owner::Nobody => "nobody now".to_string(),
    }
}
