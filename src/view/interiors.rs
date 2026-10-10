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
    let front = if v.shape == Shape::Round { (1.0 - across * across).max(0.0).sqrt() } else { 1.0 };
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
    let col = palette::scale(palette::STONE, 0.92 + unit(9) * 0.14);
    let wall_h = 2.3 + sink;
    let dome_h = size * (0.26 + unit(1) * 0.06);
    // The drum of grown wall, then the dome over it.
    drum(b, f.at(0.0, 0.0, 0.0), f.hx, f.hy, wall_h, 0.97, f.rot, 28, col);
    dome(b, f.at(0.0, 0.0, wall_h), f.hx * 0.97, f.hy * 0.97, dome_h, f.rot, 0.08, 0.10, seed, col);
    let top = wall_h + dome_h;
    let (dx, dy) = door_spot(v);
    let side_free = if v.door > 0.0 { -1.0 } else { 1.0 };
    // An upper storey budding out of the back of the dome.
    if v.storeys >= 2 {
        let (bx, by) = (-0.25, -0.2 * side_free);
        let bud = f.at(bx, by, wall_h + dome_h * 0.45);
        drum(b, bud, f.hx * 0.5, f.hy * 0.46, dome_h * 0.5, 0.95, f.rot, 20, col);
        dome(b, bud + Vec3::Y * dome_h * 0.5, f.hx * 0.48, f.hy * 0.44, dome_h * 0.75, f.rot, 0.1, 0.12, seed ^ 7, col);
        // Its window, looking out to the front.
        let w = f.at(bx, by, 0.0) + f.fwd() * (f.hx * 0.49);
        gl.patch(vec3(w.x, bud.y + dome_h * 0.3, w.z), 0.8, 0.6, f.rot, palette::WINDOW);
    }
    if v.key == "roduro_great" {
        // A second bud on the other side, lower.
        let bud = f.at(-0.35, 0.45 * side_free, wall_h + dome_h * 0.25);
        dome(b, bud, f.hx * 0.38, f.hy * 0.34, dome_h * 0.8, f.rot, 0.1, 0.12, seed ^ 11, col);
        // A stone lantern by the door.
        let lp = f.at(dx, dy, 0.0) + f.fwd() * 1.6 + f.side() * 1.4;
        b.column(lp + Vec3::Y * sink, 0.18, 0.14, 1.5, 6, palette::STONE);
        gl.column(lp + Vec3::Y * (sink + 1.5), 0.16, 0.10, 0.3, 6, palette::WINDOW);
    }
    // The door, dark in the wall, and a warm window on the free side.
    let door = f.at(dx, dy, sink + 1.0) + vec3(f.normal_at(dx, dy, true).cos(), 0.0, f.normal_at(dx, dy, true).sin()) * 0.06;
    b.patch(door, 1.1, 2.0, f.normal_at(dx, dy, true), DOOR_DARK);
    b.block(f.at(dx, dy, 0.0) + f.fwd() * 0.45, 1.5, 0.9, sink + 0.15, f.rot, palette::scale(palette::STONE, 1.15));
    let wy = 0.45 * side_free;
    let wx = (1.0 - wy * wy).sqrt();
    let wn = f.normal_at(wx, wy, true);
    gl.patch(f.at(wx, wy, sink + 1.6) + vec3(wn.cos(), 0.0, wn.sin()) * 0.06, 0.9, 0.7, wn, palette::WINDOW);
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
            // A chimney stack through the dome, glowing at the top, and the
            // forge's mouth glowing through the wall beside it.
            let c = f.at(0.1, -0.62, 0.0);
            b.column(c, 0.62, 0.46, top + 1.4, 8, palette::scale(palette::STONE, 0.8));
            gl.column(c + Vec3::Y * (top + 1.4), 0.4, 0.2, 0.35, 8, palette::EMBER);
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

fn qotiro(b: &mut Builder, gl: &mut Builder, v: &Variant, f: &Frame, size: f32, sink: f32, sign: Option<signs::Picto>) {
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
            // A chimney stack at the back, smoking hot, and a kiln in the yard.
            let c = f.at(-0.75, 0.7, 0.0);
            b.block(c, 1.1, 1.1, roof + 2.8, f.rot, palette::scale(col, 0.85));
            gl.column(c + Vec3::Y * (roof + 2.8), 0.45, 0.2, 0.35, 6, palette::EMBER);
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
    if d.round {
        drum(b, to3(d.centre, low - 0.3), f.hx, f.hy, deep, 1.0, f.rot, 30, floor_col);
    } else {
        b.block(to3(d.centre, low - 0.3), f.hx * 2.0, f.hy * 2.0, deep, f.rot, floor_col);
    }
    // Outer walls, cut away, with a gap where the door is.
    let face = d.outside.sub(f.dir().scale(1.2));
    let gap = 0.8;
    let mut ring: Vec<V2> = Vec::new();
    if d.round {
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
        gl.column(at + Vec3::Y * (0.04 + h), 0.025, 0.0, 0.07, 5, palette::WINDOW);
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
            gl.column(q.p(0.0, 0.0, 0.0), w * 0.28, w * 0.08, 0.32, 6, palette::EMBER);
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
                gl.column(q.p(0.0, s * w * 0.38, h), 0.05, 0.05, 0.18, 5, palette::WINDOW);
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

/// The caches out in the wild (ruins and lairs) near the camera, on the
/// ground: drawn every frame like the ones indoors, with a hover point each.
pub fn wild_containers(b: &mut Builder, gl: &mut Builder, w: &World, on_ground: &dyn Fn(V2) -> f32, near: V2, radius: f32, picks: &mut Vec<(Vec3, f32, Hover)>) {
    use gahturiyu_sim::sim::containers::WILD;
    let open: Vec<_> = w.squad.members.iter().filter_map(|&m| w.searching_now(m)).collect();
    for ru in w.ruins.iter().filter(|r| r.pos.dist(near) <= radius) {
        for c in w.containers.range((WILD, ru.id as u16, 0)..=(WILD, ru.id as u16, u8::MAX)).map(|(_, c)| c) {
            let base = to3(c.pos, on_ground(c.pos));
            holder(b, gl, c.what, base, c.rot, ru.pos.sub(c.pos), w.container_locked(c.id), open.contains(&c.id), !c.items.is_empty());
            picks.push((base + Vec3::Y * (c.what.size().2 + 0.15), 2.0, Hover::Container(c.id)));
        }
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
