//! Placeholder signs: what a building or a workplace is for, at a glance.
//!
//! - **Pictograms** (`Picto`): a few flat blocks per trade (an anvil, a
//!   sprouting stone, scales, a coin, a loom frame, a hammer, a bowl, a sun
//!   disc, a tankard, a shield, a scroll, a herb bundle, a flask), each its
//!   own outline and colour, drawn on the glow material so they still read
//!   after dark.
//! - **Roduro** (`roduro_board`): a timber board hung by two short iron
//!   brackets off the grown-stone wall beside the door.
//! - **Qotiro** (`qotiro_plaque`): a darker stone plaque set flush in the
//!   island-stone wall, a gold rim and a gold symbol.
//! - **Workplaces** (`signpost`): a post and crossbar by the place with a
//!   board hung from it, the symbol on both faces.
//! - Who keeps a trade building (`keeper`), for the door's hover line.
//!
//! Only service places hang a building sign (`layout::serves`: the temple
//! and the island hall); trade homes don't, since their trades are sold at
//! the town's workplaces, whose signposts players look for.
//!
//! Drawing only: a building's sign is the sim's `layout::sign_of` (the
//! variant alone) and its keeper the sim's `World::keeper`; nothing here
//! changes the world.

use bevy::math::{vec3, Vec3};

use gahturiyu_sim::sim::{
    buildings::DoorId,
    jobs::{Job, PlaceKind},
    layout::{self, Sign, Variant},
    person::PersonId,
    World,
};

use super::interiors::{disc, obox};
use super::mesh::Builder;
use super::palette::{self, Rgb};

/// A trade's symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Picto {
    Anvil,
    Sprout,
    Scales,
    Coin,
    Loom,
    Hammer,
    Bowl,
    Sun,
    Tankard,
    Shield,
    Scroll,
    Herb,
    Flask,
}

const TIMBER_DARK: Rgb = [0.24, 0.17, 0.11];
const IRON: Rgb = [0.16, 0.16, 0.18];
/// The plaque's stone: the island's dark stone, a shade darker than the wall.
const PLAQUE: Rgb = [0.13, 0.13, 0.15];

impl Picto {
    /// Its glow colour (the Qotiro plaques are always gold).
    fn colour(self) -> Rgb {
        match self {
            Picto::Anvil => [1.0, 0.52, 0.20],
            Picto::Sprout => [0.70, 0.95, 0.80],
            Picto::Scales => [1.0, 0.82, 0.30],
            Picto::Coin => [1.0, 0.86, 0.36],
            Picto::Loom => [0.72, 0.70, 1.0],
            Picto::Hammer => [0.95, 0.78, 0.55],
            Picto::Bowl => [1.0, 0.92, 0.74],
            Picto::Sun => [1.0, 0.80, 0.24],
            Picto::Tankard => [1.0, 0.66, 0.24],
            Picto::Shield => [0.60, 0.78, 1.0],
            Picto::Scroll => [0.98, 0.94, 0.78],
            Picto::Herb => [0.45, 1.0, 0.45],
            Picto::Flask => [0.50, 0.95, 0.95],
        }
    }

    /// Its blocks, as rectangles (x0, y0, x1, y1) in -1..1 of the symbol's
    /// box, and round parts as discs (x, y, r).
    fn shape(self) -> (&'static [(f32, f32, f32, f32)], &'static [(f32, f32, f32)]) {
        match self {
            // Face and horn, waist, foot.
            Picto::Anvil => (
                &[(-0.7, 0.15, 0.45, 0.62), (0.45, 0.3, 0.75, 0.62), (0.75, 0.45, 1.0, 0.62), (-0.85, 0.35, -0.7, 0.62), (-0.25, -0.35, 0.15, 0.15), (-0.5, -0.6, 0.4, -0.35), (-0.7, -0.8, -0.25, -0.6), (0.15, -0.8, 0.6, -0.6)],
                &[],
            ),
            // A stone with a shoot and two leaves.
            Picto::Sprout => (&[(-0.08, -0.4, 0.08, 0.5), (-0.75, -0.05, -0.08, 0.3), (-0.6, 0.3, -0.25, 0.42), (0.08, 0.3, 0.75, 0.65), (0.25, 0.65, 0.6, 0.77), (-0.2, 0.5, 0.2, 0.8)], &[(0.0, -0.62, 0.38)]),
            // Post, beam, two pans on strings, foot.
            Picto::Scales => (
                &[(-0.06, -0.7, 0.06, 0.62), (-0.85, 0.52, 0.85, 0.64), (-0.82, 0.0, -0.76, 0.52), (0.76, 0.0, 0.82, 0.52), (-1.0, -0.12, -0.58, 0.0), (0.58, -0.12, 1.0, 0.0), (-0.45, -0.85, 0.45, -0.7)],
                &[],
            ),
            Picto::Coin => (&[], &[(0.0, 0.0, 0.8)]),
            // A frame, its beams, and the warp.
            Picto::Loom => (
                &[(-0.8, -0.85, -0.62, 0.85), (0.62, -0.85, 0.8, 0.85), (-0.95, 0.62, 0.95, 0.8), (-0.62, -0.5, 0.62, -0.36), (-0.4, -0.36, -0.32, 0.62), (-0.12, -0.36, -0.04, 0.62), (0.16, -0.36, 0.24, 0.62), (0.44, -0.36, 0.52, 0.62)],
                &[],
            ),
            // Handle, head, its striking face and a forked claw (drawn tilted,
            // so it never reads as a "T").
            Picto::Hammer => (
                &[(-0.08, -0.95, 0.08, 0.3), (-0.45, 0.3, 0.5, 0.72), (0.5, 0.36, 0.78, 0.66), (-0.85, 0.58, -0.45, 0.72), (-0.85, 0.3, -0.6, 0.44), (-0.98, 0.62, -0.85, 0.72)],
                &[],
            ),
            // A bowl and its steam.
            Picto::Bowl => (
                &[(-0.9, -0.1, 0.9, 0.05), (-0.72, -0.32, 0.72, -0.1), (-0.45, -0.5, 0.45, -0.32), (-0.22, -0.68, 0.22, -0.5), (-0.4, 0.2, -0.3, 0.55), (-0.05, 0.25, 0.05, 0.75), (0.3, 0.2, 0.4, 0.55)],
                &[],
            ),
            // A disc and eight rays (rays drawn separately).
            Picto::Sun => (&[], &[(0.0, 0.0, 0.48)]),
            // The mug, its handle and the froth.
            Picto::Tankard => (&[(-0.5, -0.7, 0.3, 0.35), (0.3, 0.12, 0.65, 0.24), (0.53, -0.4, 0.65, 0.24), (0.3, -0.4, 0.65, -0.28), (-0.6, 0.35, 0.4, 0.62)], &[]),
            // A shield tapering to a point, and its boss.
            Picto::Shield => (&[(-0.62, -0.1, 0.62, 0.75), (-0.48, -0.42, 0.48, -0.1), (-0.3, -0.65, 0.3, -0.42), (-0.12, -0.85, 0.12, -0.65)], &[]),
            // A sheet between two rolls.
            Picto::Scroll => (&[(-0.55, -0.6, 0.55, 0.5), (-0.8, 0.45, 0.8, 0.72)], &[(-0.8, 0.585, 0.17), (0.8, 0.585, 0.17), (-0.55, -0.6, 0.15)]),
            // Three stems fanned out, leaves, and the tie.
            Picto::Herb => (
                &[(-0.06, -0.85, 0.06, 0.6), (-0.42, -0.2, -0.3, 0.45), (0.3, -0.2, 0.42, 0.45), (-0.72, 0.35, -0.3, 0.6), (0.3, 0.35, 0.72, 0.6), (-0.3, 0.6, 0.3, 0.85), (-0.3, -0.4, 0.3, -0.28)],
                &[],
            ),
            // A round-bellied flask with a neck.
            Picto::Flask => (&[(-0.14, 0.2, 0.14, 0.7), (-0.25, 0.7, 0.25, 0.82)], &[(0.0, -0.3, 0.6)]),
        }
    }
}

// ---- Which sign ----------------------------------------------------------------

/// Whether a building of this variant hangs a sign (the sim's own rule,
/// `layout::sign_of`).
pub fn signed(v: &Variant) -> bool {
    layout::serves(v) && layout::sign_of(v).is_some()
}

/// A building's symbol: the sim's `layout::sign_of`, decided by the variant
/// alone, drawn as its pictogram. None: no sign.
pub fn for_building(v: &Variant) -> Option<Picto> {
    // Only real service places hang a sign (`layout::serves`); trade homes don't.
    if !layout::serves(v) {
        return None;
    }
    Some(match layout::sign_of(v)? {
        Sign::Anvil => Picto::Anvil,
        Sign::Sprout => Picto::Sprout,
        Sign::Coin => Picto::Coin,
        Sign::Scales => Picto::Scales,
        Sign::Loom => Picto::Loom,
        Sign::Hammer => Picto::Hammer,
        Sign::Bowl => Picto::Bowl,
        Sign::Sun => Picto::Sun,
        Sign::Flask => Picto::Flask,
        Sign::Scroll => Picto::Scroll,
    })
}

/// A workplace's symbol (None for those out of town, which need no sign).
pub fn for_place(k: PlaceKind) -> Option<Picto> {
    Some(match k {
        PlaceKind::Market | PlaceKind::Shop(_) => Picto::Scales,
        PlaceKind::Inn => Picto::Tankard,
        PlaceKind::Shrine => Picto::Sun,
        PlaceKind::GuardPost => Picto::Shield,
        PlaceKind::Hall | PlaceKind::TeachingHouse | PlaceKind::LettersHouse | PlaceKind::Desk => Picto::Scroll,
        PlaceKind::ExchangeHouse => Picto::Coin,
        PlaceKind::HealingHouse | PlaceKind::HealersHouse => Picto::Herb,
        PlaceKind::Kitchen | PlaceKind::MessHall => Picto::Bowl,
        PlaceKind::Workyard | PlaceKind::Forge => Picto::Anvil,
        PlaceKind::Bench | PlaceKind::Workshop => Picto::Hammer,
        PlaceKind::AlchemyTable => Picto::Flask,
        PlaceKind::WeaversShed => Picto::Loom,
        PlaceKind::TendersYard => Picto::Sprout,
        _ => return None,
    })
}

/// Who keeps a trade building (the sim's `World::keeper`), and their job.
pub fn keeper(w: &World, door: DoorId) -> Option<(PersonId, Job)> {
    w.keeper(door).map(|p| (p, w.life(p).job))
}

/// What the door's hover calls a building: "Maker's forge · kept by Ana,
/// smith", or just its variant's name.
pub fn building_title(w: &World, door: DoorId) -> Option<String> {
    let s = w.settlements.get(door.0 as usize)?;
    let v = layout::variant_in(s, door.1)?;
    Some(match keeper(w, door) {
        Some((p, j)) => format!("{}  ·  kept by {}, {}", v.name, w.name_of(p), j.name().to_lowercase()),
        None => v.name.to_string(),
    })
}

// ---- Drawing -------------------------------------------------------------------

/// The symbol on a face whose centre is `c`, facing `facing` radians, in a
/// box `sx` by `sy` half-size.
fn symbol(gl: &mut Builder, p: Picto, c: Vec3, facing: f32, sx: f32, sy: f32, col: Rgb) {
    let n = vec3(facing.cos(), 0.0, facing.sin());
    let right = vec3(-facing.sin(), 0.0, facing.cos());
    let (rects, discs) = p.shape();
    let front = c + n * 0.03;
    // The hammer leans over, about 30 degrees.
    let tilt: f32 = if p == Picto::Hammer { -0.52 } else { 0.0 };
    let (ts, tc) = tilt.sin_cos();
    let (rx, ry) = (right * tc + Vec3::Y * ts, Vec3::Y * tc - right * ts);
    let scale = if tilt != 0.0 { 0.85 } else { 1.0 };
    let (sx, sy) = (sx * scale, sy * scale);
    for &(x0, y0, x1, y1) in rects {
        let at = front + rx * ((x0 + x1) * 0.5 * sx) + ry * ((y0 + y1) * 0.5 * sy);
        obox(gl, at, rx * ((x1 - x0) * 0.5 * sx), ry * ((y1 - y0) * 0.5 * sy), n * 0.025, col);
    }
    for &(x, y, r) in discs {
        disc(gl, front + n * 0.03 + right * (x * sx) + Vec3::Y * (y * sy), r * sx.min(sy), facing, col);
    }
    match p {
        Picto::Sun => {
            for k in 0..8 {
                let a = k as f32 / 8.0 * std::f32::consts::TAU;
                let d = right * (a.cos() * sx) + Vec3::Y * (a.sin() * sy);
                let along = d.normalize_or_zero();
                let side = n.cross(along).normalize_or_zero();
                obox(gl, front + d * 0.78, along * (0.17 * sx.min(sy)), side * (0.06 * sx.min(sy)), n * 0.025, col);
            }
        }
        Picto::Scroll => {
            // Lines of writing, dark on the sheet.
            let dark = palette::scale(col, 0.3);
            for (k, len) in [0.8f32, 0.6, 0.75, 0.45].iter().enumerate() {
                let y = 0.25 - k as f32 * 0.22;
                obox(gl, front + n * 0.06 + right * ((-0.4 + len * 0.5 - 0.0) * sx * 0.95) + Vec3::Y * (y * sy), right * (len * 0.5 * sx * 0.95), Vec3::Y * (0.04 * sy), n * 0.02, dark);
            }
        }
        Picto::Coin => {
            // The press's square hole, dark, and a rim.
            let dark = palette::scale(col, 0.35);
            obox(gl, front + n * 0.07, right * (0.18 * sx), Vec3::Y * (0.18 * sy), n * 0.02, dark);
        }
        _ => {}
    }
}

/// How bright a building sign's symbol glows, of its colour: enough to make
/// out at night beside its lantern, not a neon sign.
const SIGN_GLOW: f32 = 0.55;
/// A board's and a plaque's face lit by the lantern beside it.
const BOARD_LIT: Rgb = [0.26, 0.17, 0.10];
const PLAQUE_LIT: Rgb = [0.15, 0.13, 0.12];

/// An iron lantern on a bracket `out` from the wall at `at` (facing
/// `facing`), so a sign beside it is lit (canon: iron lanterns beside
/// doorways).
fn lantern(b: &mut Builder, gl: &mut Builder, at: Vec3, facing: f32, out: f32) {
    let n = vec3(facing.cos(), 0.0, facing.sin());
    let right = vec3(-facing.sin(), 0.0, facing.cos());
    // The bracket, its brace, and a short hook down.
    b.stick(at - n * 0.4, at + n * (out + 0.04), 0.06, IRON);
    b.stick(at - Vec3::Y * 0.35 - n * 0.15, at + n * (out * 0.65), 0.045, IRON);
    let hook = at + n * out;
    b.stick(hook, hook - Vec3::Y * 0.2, 0.03, IRON);
    // The cage: a cap, a base, four corner bars, the flame's glow inside.
    let c = hook - Vec3::Y * 0.55;
    b.column(c + Vec3::Y * 0.3, 0.26, 0.06, 0.16, 4, IRON);
    b.block(c - Vec3::Y * 0.32, 0.38, 0.38, 0.07, facing, IRON);
    for (sx, sz) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let k = c + right * (sx * 0.17) + n * (sz * 0.17);
        b.stick(k - Vec3::Y * 0.26, k + Vec3::Y * 0.31, 0.035, IRON);
    }
    gl.block(c - Vec3::Y * 0.25, 0.27, 0.27, 0.54, facing, palette::WINDOW);
}

/// Which way along the wall (+1 or -1 on its `right`) is away from `door`.
fn away(wall: Vec3, facing: f32, door: Vec3) -> f32 {
    let right = vec3(-facing.sin(), 0.0, facing.cos());
    if right.dot(door - wall) > 0.0 {
        -1.0
    } else {
        1.0
    }
}

/// A Roduro sign: a timber board hung by two short iron brackets from the
/// wall at `wall` (a point on its face), facing `facing`, with an iron
/// lantern hung beside it on the side away from `door`.
pub fn roduro_board(b: &mut Builder, gl: &mut Builder, p: Picto, wall: Vec3, facing: f32, door: Vec3) {
    let n = vec3(facing.cos(), 0.0, facing.sin());
    let right = vec3(-facing.sin(), 0.0, facing.cos());
    let (w, h, out) = (2.0f32, 1.5f32, 0.6f32);
    // The brackets, out from the wall, each with a brace under it.
    for s in [-1.0f32, 1.0] {
        let root = wall + right * (s * w * 0.4);
        b.stick(root - n * 0.3, root + n * (out + 0.06), 0.07, IRON);
        b.stick(root - Vec3::Y * 0.45 - n * 0.2, root + n * (out * 0.7), 0.05, IRON);
        // A short chain down to the board.
        b.stick(root + n * out, root + n * out - Vec3::Y * 0.18, 0.04, IRON);
    }
    let c = wall + n * out - Vec3::Y * (0.18 + h * 0.5);
    obox(b, c, right * (w * 0.5), Vec3::Y * (h * 0.5), n * 0.05, palette::TIMBER);
    // A darker frame round the face.
    for (dx, dy, hx, hy) in [(0.0, h * 0.5 - 0.05, w * 0.5, 0.05), (0.0, -h * 0.5 + 0.05, w * 0.5, 0.05), (w * 0.5 - 0.05, 0.0, 0.05, h * 0.5), (-w * 0.5 + 0.05, 0.0, 0.05, h * 0.5)] {
        obox(b, c + n * 0.03 + right * dx + Vec3::Y * dy, right * hx, Vec3::Y * hy, n * 0.03, TIMBER_DARK);
    }
    // The board's face as its lantern lights it (on the glow material, so
    // it's lit at night and plain timber by day).
    gl.patch(c + n * 0.052, w - 0.16, h - 0.16, facing, BOARD_LIT);
    symbol(gl, p, c + n * 0.05, facing, w * 0.36, h * 0.36, palette::scale(p.colour(), SIGN_GLOW));
    let s = away(wall, facing, door);
    lantern(b, gl, wall + right * (s * (w * 0.5 + 0.45)) + Vec3::Y * 0.1, facing, 0.55);
}

/// A Qotiro sign: a dark stone plaque set into the wall at `wall`, facing
/// `facing`, a gold rim and a gold symbol, an iron lantern beside it on the
/// side away from `door`.
pub fn qotiro_plaque(b: &mut Builder, gl: &mut Builder, p: Picto, wall: Vec3, facing: f32, big: f32, door: Vec3) {
    let n = vec3(facing.cos(), 0.0, facing.sin());
    let right = vec3(-facing.sin(), 0.0, facing.cos());
    let (w, h) = (1.8 * big, 1.8 * big);
    obox(b, wall + n * 0.04, right * (w * 0.5), Vec3::Y * (h * 0.5), n * 0.08, PLAQUE);
    gl.patch(wall + n * 0.122, w - 0.1 * big, h - 0.1 * big, facing, PLAQUE_LIT);
    // The gold rim.
    let rim = palette::scale(palette::METAL_GOLD, SIGN_GLOW);
    let t = 0.05 * big;
    for (dx, dy, hx, hy) in [(0.0, h * 0.5 - t, w * 0.5, t), (0.0, -h * 0.5 + t, w * 0.5, t), (w * 0.5 - t, 0.0, t, h * 0.5), (-w * 0.5 + t, 0.0, t, h * 0.5)] {
        obox(gl, wall + n * 0.13 + right * dx + Vec3::Y * dy, right * hx, Vec3::Y * hy, n * 0.02, palette::scale(rim, 0.8));
    }
    symbol(gl, p, wall + n * 0.12, facing, w * 0.36, h * 0.36, rim);
    let s = away(wall, facing, door);
    lantern(b, gl, wall + right * (s * (w * 0.5 + 0.4)) + Vec3::Y * (h * 0.5 + 0.1), facing, 0.5);
}

/// A workplace's signpost at `foot` (on the ground), its board facing
/// `facing`, the symbol on both faces.
pub fn signpost(b: &mut Builder, gl: &mut Builder, p: Picto, foot: Vec3, facing: f32) {
    let n = vec3(facing.cos(), 0.0, facing.sin());
    let right = vec3(-facing.sin(), 0.0, facing.cos());
    let (w, h) = (2.0f32, 1.5f32);
    let top = 3.6;
    b.stick(foot - Vec3::Y * 0.3, foot + Vec3::Y * top, 0.16, palette::TIMBER);
    // The crossbar out to one side, and its brace.
    let bar = foot + Vec3::Y * (top - 0.15);
    b.stick(bar - right * 0.1, bar + right * (w + 0.35), 0.1, palette::TIMBER);
    b.stick(foot + Vec3::Y * (top - 0.85), bar + right * 0.65, 0.07, palette::TIMBER);
    let c = bar + right * (w * 0.5 + 0.25) - Vec3::Y * (0.2 + h * 0.5);
    for s in [-1.0f32, 1.0] {
        let hook = bar + right * (w * 0.5 + 0.25 + s * w * 0.38);
        b.stick(hook, hook - Vec3::Y * 0.2, 0.04, IRON);
    }
    obox(b, c, right * (w * 0.5), Vec3::Y * (h * 0.5), n * 0.05, palette::TIMBER);
    for f in [facing, facing + std::f32::consts::PI] {
        let fn_ = vec3(f.cos(), 0.0, f.sin());
        symbol(gl, p, c + fn_ * 0.05, f, w * 0.36, h * 0.36, p.colour());
    }
}
