//! Drawing animals, and the debug aids for them: stand-in shapes in the 3D
//! view, marks on the map, a hover description, a wildlife panel and
//! screenshot scenes. Like the rest of the window, nothing here changes what
//! happens: it reads the simulation, and its debug keys only call the same
//! plain functions anything else would.
//!
//! The shapes are deliberately plain — a few boxes and domes sized and
//! coloured per species — until there is real art.
//!
//! Debug keys (shown under the wildlife panel): F7 the panel; H the squad
//! sets about the nearest wild herd; Y tame the nearest Ridgehound that is
//! down or young; U take cocoons from the nearest silk colony.

use std::sync::atomic::{AtomicBool, Ordering};

use bevy::math::Vec2;
use bevy::prelude::{vec3, Vec3};
use bevy_egui::egui;

use gahturiyu_sim::sim::{
    animals::{region_centre, region_of, Build, Class, Looks, Phase, Sp, Take, ACROSS, ALL_SPECIES, BRIAR_THRESHOLD, FISH, HABITATS, REGION},
    geo::{self, V2},
    tide,
    world::{DAY, HOUR},
    World,
};

use super::app::{Game, View};
use super::cam::{to3, MapCam};
use super::hud::Canvas;
use super::mesh::Builder;
use super::palette::{self, eg, ega, Rgb, DIM, GOLD, TEXT, WARN};
use super::squadui::Bx;

/// The wildlife panel is showing.
static PANEL: AtomicBool = AtomicBool::new(false);
/// One of every animal is stood in a row by the squad, to look the
/// stand-in shapes over (`GAHT_ANIMALS=parade`). Drawing only.
static PARADE: AtomicBool = AtomicBool::new(false);

/// Within this of the camera's target, animals are drawn in full.
const FULL: f32 = 420.0;
/// Within this, as one plain block each; beyond, one marker per herd.
const PLAIN: f32 = 1500.0;
/// The most animals of one herd or pen drawn one by one.
const MOST: usize = 30;

/// Something of the animals' that can be seen (and hovered).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Thing {
    /// One wild animal: herd and its place in it.
    Animal(u32, usize),
    /// A whole herd, too far off to pick out its animals.
    Herd(u32),
    /// One kept animal: pen and place.
    Kept(u32, usize),
    Led(usize),
    Hound(u32),
    /// An animal in a fight (drawn from the fight).
    Fighting(usize),
    Colony(u32),
    Carcass(u32),
    Flock,
}

#[derive(Clone, Copy, Debug)]
pub struct Seen {
    pub at: V2,
    pub thing: Thing,
    pub sp: Sp,
    pub size: f32,
    pub down: bool,
    /// Which way it faces, radians.
    pub rot: f32,
    /// 1 = full shape, 2 = a plain block, 3 = a marker for the herd.
    pub detail: u8,
    pub count: usize,
}

fn spin(seed: u64) -> f32 {
    (seed % 6283) as f32 / 1000.0
}

/// Everything of the animals' within `radius` of `centre` right now.
pub fn sightings(w: &World, centre: V2, radius: f32) -> Vec<Seen> {
    let t = w.time;
    let a = &w.animals;
    let mut out = Vec::new();
    let detail_at = |p: V2| {
        let d = p.dist(centre);
        if d < FULL || w.bands.band_at(p) == 1 {
            1
        } else if d < PLAIN {
            2
        } else {
            3
        }
    };
    for h in &a.herds {
        let d = h.def();
        if h.home.dist(centre) > radius + d.range + 600.0 {
            continue;
        }
        let alive = h.alive(t);
        if alive == 0 {
            continue;
        }
        // Under the rocks until the water drops.
        if h.sp == Sp::Tidepicker && !h.active(t) {
            continue;
        }
        let pos = w.herd_pos(h.id, t);
        if pos.dist(centre) > radius {
            continue;
        }
        let ahead = w.herd_pos(h.id, t + 4.0).sub(pos);
        let heading = (ahead.len() > 0.3).then(|| ahead.y.atan2(ahead.x));
        let detail = detail_at(pos);
        if detail == 3 {
            out.push(Seen { at: pos, thing: Thing::Herd(h.id), sp: h.sp, size: 1.0, down: false, rot: 0.0, detail, count: alive });
            continue;
        }
        for j in 0..alive.min(MOST) {
            let ident = h.ident(j);
            if w.animal_fighting(h.id, ident) {
                continue;
            }
            let m = h.member(j, t);
            let rot = heading.unwrap_or_else(|| spin(h.seed ^ (ident as u64 * 7919) ^ ((t / 400.0) as u64)));
            out.push(Seen { at: h.member_at(pos, j, t), thing: Thing::Animal(h.id, j), sp: h.sp, size: m.size, down: m.down, rot, detail, count: 1 });
        }
    }
    for (k, f) in w.animals_fighting().into_iter().enumerate() {
        if f.pos.dist(centre) <= radius {
            out.push(Seen { at: f.pos, thing: Thing::Fighting(k), sp: f.sp, size: f.size, down: f.down, rot: spin(k as u64 * 977 + (t * 2.0) as u64 % 3), detail: 1, count: 1 });
        }
    }
    for p in &a.pens {
        if p.home.dist(centre) > radius {
            continue;
        }
        let led = a.led.iter().filter(|l| l.pen == p.id).count();
        let head = (p.now(t).count.floor() as usize).saturating_sub(led);
        let detail = detail_at(p.home).min(2);
        for k in 0..head.min(MOST) {
            let rot = spin(p.seed ^ (k as u64 * 104_729) ^ ((t / 500.0) as u64));
            out.push(Seen { at: p.animal_at(k, t), thing: Thing::Kept(p.id, k), sp: p.sp, size: 1.0, down: false, rot, detail, count: 1 });
        }
    }
    for k in 0..a.led.len() {
        if let Some(at) = w.led_pos(k) {
            if at.dist(centre) <= radius {
                out.push(Seen { at, thing: Thing::Led(k), sp: Sp::Plodder, size: 1.0, down: false, rot: 0.0, detail: 1, count: 1 });
            }
        }
    }
    for h in w.tamed() {
        if a.frays.iter().any(|f| f.parts.iter().any(|p| p.who == gahturiyu_sim::sim::animals::Who::Hound(h.id))) {
            continue;
        }
        if let Some(at) = w.hound_pos(h.id) {
            if at.dist(centre) <= radius {
                out.push(Seen { at, thing: Thing::Hound(h.id), sp: Sp::Ridgehound, size: h.size, down: false, rot: spin(h.seed), detail: 1, count: 1 });
            }
        }
    }
    for c in &a.colonies {
        if c.pos.dist(centre) <= radius {
            out.push(Seen { at: c.pos, thing: Thing::Colony(c.id), sp: Sp::Silkcrawler, size: 1.0, down: false, rot: 0.0, detail: 1, count: w.cocoons(c.id) as usize });
        }
    }
    for c in &a.carcasses {
        if c.gone_at > t && c.pos.dist(centre) <= radius {
            out.push(Seen { at: c.pos, thing: Thing::Carcass(c.id), sp: c.sp, size: c.size, down: true, rot: spin(c.id as u64 * 31), detail: 1, count: 1 });
        }
    }
    for (pos, n) in w.flocks() {
        if pos.dist(centre) <= radius {
            out.push(Seen { at: pos, thing: Thing::Flock, sp: Sp::Bonepicker, size: 1.0, down: false, rot: 0.0, detail: 1, count: n });
        }
    }
    out
}

// ---- Shapes -------------------------------------------------------------------

/// One animal as a few plain solids. `s` scales it (its own size times the
/// zoomed-out enlargement people get too).
fn shape(b: &mut Builder, base: Vec3, lk: &Looks, s: f32, rot: f32, seed: u64) {
    let (len, tall, wide) = (lk.len * s, lk.tall * s, lk.wide * s);
    let fwd = vec3(rot.cos(), 0.0, rot.sin());
    let side = vec3(-rot.sin(), 0.0, rot.cos());
    let up = |h: f32| vec3(0.0, h, 0.0);
    let legs = |b: &mut Builder, at: f32, span: f32, h: f32, thick: f32, col: Rgb| {
        for (fx, sx) in [(at, span), (at, -span), (-at, span), (-at, -span)] {
            let foot = base + fwd * (len * fx) + side * (wide * sx);
            b.stick(foot, foot + up(h), thick, col);
        }
    };
    let dark = palette::scale(lk.coat, 0.7);
    match lk.build {
        Build::Barrel => {
            let leg = tall * 0.28;
            legs(b, 0.3, 0.32, leg, 0.09 * s, dark);
            b.block(base + up(leg), len * 0.8, wide, tall * 0.7, rot, lk.coat);
            // Ring bands, and the broad flat snout.
            for f in [-0.18, 0.12] {
                b.block(base + up(leg) + fwd * (len * f), len * 0.07, wide * 1.04, tall * 0.72, rot, lk.mark);
            }
            b.block(base + up(leg * 0.9) + fwd * (len * 0.48), len * 0.26, wide * 0.8, tall * 0.42, rot, palette::scale(lk.coat, 1.12));
        }
        Build::Dome => {
            b.dome(base + up(tall * 0.2), len * 0.5, wide * 0.5, tall * 0.8, 0.0, 0.0, seed, lk.coat);
            b.dome(base + up(tall * 0.45) + side * (wide * 0.2), len * 0.16, wide * 0.16, tall * 0.4, 0.0, 0.0, seed + 1, lk.mark);
            b.block(base + up(tall * 0.15) + fwd * (len * 0.45), len * 0.22, wide * 0.4, tall * 0.4, rot, [0.52, 0.52, 0.54]);
        }
        Build::Grazer => {
            let leg = tall * 0.48;
            legs(b, 0.3, 0.3, leg, 0.16 * s, dark);
            b.block(base + up(leg), len * 0.78, wide, tall * 0.42, rot, lk.coat);
            // High shoulders, a low-hung head, and whatever grows on its back.
            b.block(base + up(leg + tall * 0.42) + fwd * (len * 0.14), len * 0.4, wide * 0.85, tall * 0.14, rot, lk.coat);
            b.block(base + up(leg * 0.62) + fwd * (len * 0.5), len * 0.24, wide * 0.5, tall * 0.26, rot, dark);
            b.block(base + up(leg + tall * 0.42) + fwd * (-len * 0.16), len * 0.42, wide * 0.6, tall * 0.05, rot, lk.mark);
        }
        Build::Raft => {
            b.dome(base - up(tall * 0.3), len * 0.5, wide * 0.5, tall, 0.12, 0.0, seed, lk.coat);
            for k in 0..7u64 {
                let a = spin(seed.wrapping_mul(k + 3) + k * 911);
                let r = 0.15 + 0.3 * ((seed >> (k * 5)) & 15) as f32 / 15.0;
                let at = base + vec3(a.cos() * len * r, tall * 0.55, a.sin() * wide * r);
                b.dome(at, len * 0.05, len * 0.05, tall * 0.25, 0.3, 0.0, seed + k, lk.mark);
            }
        }
        Build::Runner => {
            let leg = tall * 0.58;
            legs(b, 0.28, 0.3, leg, 0.07 * s, dark);
            b.block(base + up(leg), len * 0.66, wide, tall * 0.36, rot, lk.coat);
            b.stick(base + up(leg + tall * 0.2) + fwd * (len * 0.3), base + up(leg + tall * 0.42) + fwd * (len * 0.46), wide * 0.5, lk.coat);
            b.block(base + up(leg + tall * 0.34) + fwd * (len * 0.5), len * 0.24, wide * 0.6, tall * 0.2, rot, lk.mark);
            b.stick(base + up(leg + tall * 0.25) - fwd * (len * 0.33), base + up(leg) - fwd * (len * 0.52), 0.05 * s, dark);
        }
        Build::Bird => {
            let leg = tall * 0.45;
            for sx in [0.25, -0.25] {
                let foot = base + side * (wide * sx);
                b.stick(foot, foot + up(leg), 0.05 * s, lk.mark);
            }
            b.block(base + up(leg), len * 0.6, wide, tall * 0.3, rot, lk.coat);
            b.stick(base + up(leg + tall * 0.2) + fwd * (len * 0.22), base + up(tall * 0.94) + fwd * (len * 0.34), 0.07 * s, lk.coat);
            b.block(base + up(tall * 0.9) + fwd * (len * 0.4), len * 0.2, wide * 0.4, tall * 0.1, rot, lk.mark);
        }
        Build::Lurker => {
            b.block(base, len * 0.6, wide, tall, rot, lk.coat);
            b.block(base + fwd * (len * 0.42), len * 0.3, wide * 0.7, tall * 0.7, rot, lk.mark);
            b.block(base - fwd * (len * 0.42), len * 0.3, wide * 0.4, tall * 0.5, rot, dark);
            legs(b, 0.22, 0.6, tall * 0.5, 0.1 * s, dark);
        }
        Build::Hulk => {
            let leg = tall * 0.35;
            for (fx, sx) in [(0.28, 0.32), (0.28, -0.32), (-0.28, 0.32), (-0.28, -0.32)] {
                b.column(base + fwd * (len * fx) + side * (wide * sx), wide * 0.16, wide * 0.13, leg, 6, dark);
            }
            b.block(base + up(leg), len * 0.8, wide, tall * 0.45, rot, lk.coat);
            b.block(base + up(leg + tall * 0.45) + fwd * (len * 0.12), len * 0.5, wide * 0.8, tall * 0.2, rot, palette::scale(lk.coat, 0.85));
            b.block(base + up(leg * 0.8) + fwd * (len * 0.5), len * 0.24, wide * 0.6, tall * 0.42, rot, dark);
            // Plates along the back.
            for k in 0..4 {
                b.block(base + up(leg + tall * 0.65) + fwd * (len * (0.24 - k as f32 * 0.16)), len * 0.08, wide * 0.5, tall * 0.1, rot, lk.mark);
            }
        }
        Build::Crawler => {
            if lk.len < 0.5 {
                b.block(base, len, wide, tall, rot, lk.coat);
                return;
            }
            let lift = tall * 0.35;
            b.dome(base + up(lift), len * 0.3, wide * 0.24, tall * 0.65, 0.0, 0.25, seed, lk.coat);
            b.dome(base + up(lift) + fwd * (len * 0.36), len * 0.16, wide * 0.14, tall * 0.4, 0.0, 0.0, seed + 1, lk.mark);
            for k in 0..8 {
                let a = rot + (k as f32 + 0.5) / 8.0 * std::f32::consts::TAU;
                let knee = base + up(tall * 0.8) + vec3(a.cos(), 0.0, a.sin()) * (wide * 0.32);
                let foot = base + vec3(a.cos(), 0.0, a.sin()) * (wide * 0.55);
                b.stick(base + up(lift + tall * 0.2), knee, 0.06 * s, lk.mark);
                b.stick(knee, foot, 0.05 * s, lk.mark);
            }
        }
        Build::Thorn => {
            let leg = tall * 0.25;
            legs(b, 0.26, 0.3, leg, 0.14 * s, dark);
            b.dome(base + up(leg), len * 0.5, wide * 0.5, tall * 0.75, 0.35, 0.0, seed, lk.coat);
            for k in 0..9u64 {
                let a = spin(seed.wrapping_mul(k + 5) + k * 409);
                let root = base + up(leg + tall * 0.45) + vec3(a.cos() * len * 0.28, 0.0, a.sin() * wide * 0.28);
                b.stick(root, root + vec3(a.cos() * len * 0.2, tall * 0.4, a.sin() * wide * 0.2), 0.05 * s, lk.mark);
            }
        }
    }
}

/// Lying down: knocked out, or dead.
fn lying(b: &mut Builder, base: Vec3, lk: &Looks, s: f32, rot: f32, dead: bool) {
    let col = if dead { palette::mix(lk.coat, [0.35, 0.10, 0.08], 0.55) } else { palette::scale(lk.coat, 0.75) };
    b.block(base - vec3(0.0, 0.05, 0.0), lk.len * s * 0.9, lk.wide.max(lk.tall * 0.6) * s, (lk.tall * 0.3 * s).max(0.08), rot, col);
}

/// A fence round a pen: posts and one rail.
fn fence(b: &mut Builder, on_ground: &dyn Fn(V2) -> f32, centre: V2, r: f32, height: f32) {
    let n = ((r * std::f32::consts::TAU / 2.6).round() as usize).clamp(8, 40);
    let post = |k: usize| {
        let a = k as f32 / n as f32 * std::f32::consts::TAU;
        let p = centre.add(V2::new(a.cos(), a.sin()).scale(r));
        to3(p, on_ground(p) - 0.05)
    };
    for k in 0..n {
        let (p, q) = (post(k), post(k + 1));
        b.column(p, 0.07, 0.06, height + 0.1, 4, palette::TIMBER);
        // (A gap for the gate.)
        if k != 0 {
            b.stick(p + vec3(0.0, height * 0.75, 0.0), q + vec3(0.0, height * 0.75, 0.0), 0.06, palette::TIMBER);
        }
    }
}

/// Draw the animals into the 3D view's every-frame meshes. Called once from
/// `scene::update`.
#[allow(clippy::too_many_arguments)]
pub fn draw(w: &World, b: &mut Builder, gl: &mut Builder, on_ground: &dyn Fn(V2) -> f32, centre: V2, radius: f32, k: f32, eye: Vec3) {
    if w.animals.regions.is_empty() {
        return;
    }
    let t = w.time;
    let kk = k.min(4.0);
    // What stands still: pens and colonies.
    for p in &w.animals.pens {
        if p.home.dist(centre) > radius.min(1800.0) {
            continue;
        }
        match p.sp {
            Sp::Raftback => {}
            Sp::Shellhen => {
                fence(b, on_ground, p.home, p.radius, 0.5);
                let at = p.home.add(V2::new(p.radius * 0.45, 0.0));
                b.block(to3(at, on_ground(at) - 0.1), 2.4, 1.8, 1.3, spin(p.seed), palette::TIMBER);
                b.block(to3(at, on_ground(at) + 1.2), 2.8, 2.2, 0.2, spin(p.seed), palette::WEAVE);
            }
            Sp::Plodder => {
                fence(b, on_ground, p.home, p.radius, 1.2);
                // An open shed.
                let at = p.home.add(V2::new(0.0, p.radius * 0.5));
                for (dx, dy) in [(-2.0, -1.4), (2.0, -1.4), (-2.0, 1.4), (2.0, 1.4)] {
                    let q = at.add(V2::new(dx, dy));
                    b.column(to3(q, on_ground(q) - 0.1), 0.1, 0.1, 2.5, 4, palette::TIMBER);
                }
                b.block(to3(at, on_ground(at) + 2.4), 4.8, 3.6, 0.2, 0.0, palette::WEAVE);
            }
            _ => fence(b, on_ground, p.home, p.radius, 1.0),
        }
    }
    for c in &w.animals.colonies {
        if c.pos.dist(centre) > radius.min(1800.0) {
            continue;
        }
        let n = (w.cocoons(c.id) as usize).min(30);
        for j in 0..n {
            let a = spin(c.seed.wrapping_mul(j as u64 + 3));
            let r = 1.0 + 6.0 * (((c.seed >> (j % 40)) & 63) as f32 / 63.0);
            let q = c.pos.add(V2::new(a.cos(), a.sin()).scale(r));
            let base = to3(q, on_ground(q));
            b.dome(base, 0.22 * kk, 0.22 * kk, 0.55 * kk, 0.15, 0.0, c.seed + j as u64, [0.93, 0.92, 0.86]);
        }
        // Strands of web between a few stakes of rock.
        for j in 0..5 {
            let a = j as f32 / 5.0 * std::f32::consts::TAU + spin(c.seed);
            let q = c.pos.add(V2::new(a.cos(), a.sin()).scale(7.0));
            let (p0, p1) = (to3(c.pos, on_ground(c.pos) + 2.2 * kk), to3(q, on_ground(q) + 0.2));
            b.stick(p0, p1, 0.04 * kk, [0.9, 0.9, 0.88]);
        }
    }
    if PARADE.load(Ordering::Relaxed) {
        let mut x = -30.0;
        for sp in ALL_SPECIES {
            let lk = &sp.def().looks;
            if matches!(sp.def().class, Class::Fish | Class::Sea) || sp == Sp::Raftback {
                continue;
            }
            let grow = (0.45 / lk.len).max(1.0);
            x += lk.len * grow * 0.5 + 0.8;
            let at = w.squad.pos.add(V2::new(x, 7.0 + lk.wide * grow * 0.5));
            shape(b, to3(at, on_ground(at) - 0.05), lk, grow, 0.0, sp as u64 * 7 + 3);
            x += lk.len * grow * 0.5;
        }
    }
    // What moves.
    for s in sightings(w, centre, radius) {
        let lk = &s.sp.def().looks;
        let afloat = geo::inland(s.at) < 0.0;
        let ground = if afloat { 0.0 } else { on_ground(s.at) };
        let base = to3(s.at, ground - 0.05);
        // Small things get the same enlargement people do when zoomed out,
        // or they vanish; big things need less of it. The very smallest are
        // drawn a good deal over life size, or they'd be specks.
        let grow = if lk.len > 3.0 { k.min(1.6) } else { kk } * (0.45 / lk.len).max(1.0);
        let seed = match s.thing {
            Thing::Animal(h, j) => (h as u64) << 16 | j as u64,
            Thing::Kept(p, j) => 0x9000_0000 | (p as u64) << 16 | j as u64,
            _ => 7,
        };
        match s.thing {
            Thing::Herd(_) => {
                // Far off: one marker for the lot, as travellers get.
                let mk = (base.distance(eye) / 140.0).max(2.0);
                let n = s.count as f32;
                b.block(base, (1.2 + 0.25 * n.sqrt()) * mk, (0.8 + 0.2 * n.sqrt()) * mk, 0.7 * mk, 0.4, lk.coat);
            }
            Thing::Colony(_) => {}
            Thing::Carcass(_) => lying(b, base, lk, s.size * grow, s.rot, true),
            Thing::Flock => {
                // Birds on the body, and a couple wheeling over it.
                let bl = &Sp::Bonepicker.def().looks;
                for j in 0..s.count {
                    let a = j as f32 * 2.4 + (t / 7.0) as f32 * if j % 2 == 0 { 0.2 } else { -0.15 };
                    let wheeling = j % 4 == 3;
                    let r = if wheeling { 7.0 } else { 0.8 + (j % 3) as f32 * 0.5 };
                    let spot = if wheeling { a + (t / 2.5) as f32 } else { a };
                    let q = s.at.add(V2::new(spot.cos(), spot.sin()).scale(r * kk.min(2.0)));
                    let lift = if wheeling { 9.0 + (j % 3) as f32 * 2.0 } else { 0.0 };
                    shape(b, to3(q, on_ground(q) + lift), bl, kk, spot + 1.6, j as u64);
                }
            }
            _ if s.detail >= 2 => {
                b.block(base, lk.len * s.size * grow, lk.wide * s.size * grow, lk.tall * s.size * grow, s.rot, lk.coat);
            }
            _ if s.down => lying(b, base, lk, s.size * grow, s.rot, false),
            Thing::Hound(_) | Thing::Led(_) => {
                shape(b, base, lk, s.size * grow, s.rot, seed);
                // A band of colour to show it's somebody's.
                gl.block(base + vec3(0.0, lk.tall * s.size * grow * 0.95, 0.0), lk.len * 0.2 * grow, lk.wide * 1.1 * grow, 0.07 * grow, s.rot, GOLD);
            }
            _ => shape(b, base, lk, s.size * grow, s.rot, seed),
        }
    }
}

// ---- The map -------------------------------------------------------------------

fn map_colour(sp: Sp) -> Rgb {
    match sp.def().class {
        Class::Predator => [0.95, 0.38, 0.25],
        Class::Domestic => [0.92, 0.9, 0.82],
        _ => [0.66, 0.86, 0.5],
    }
}

/// Mark the animals on the top-down map. Called once from `map::draw`.
pub fn draw_map(c: &Canvas, cam: &MapCam, w: &World) {
    if w.animals.regions.is_empty() {
        return;
    }
    let t = w.time;
    let size = Vec2::new(c.w, c.h);
    let on_screen = |q: Vec2| q.x > -20.0 && q.y > -20.0 && q.x < c.w + 20.0 && q.y < c.h + 20.0;
    let dot = (cam.zoom * 1.0).clamp(1.5, 4.0);
    for h in &w.animals.herds {
        let alive = h.alive(t);
        if alive == 0 || (h.sp == Sp::Tidepicker && !h.active(t)) {
            continue;
        }
        let pos = w.herd_pos(h.id, t);
        let q = cam.to_screen(size, pos);
        if !on_screen(q) {
            continue;
        }
        let col = map_colour(h.sp);
        let danger = h.def().toward.dangerous();
        if cam.zoom > 1.2 && w.bands.band_at(pos) <= 2 {
            for j in 0..alive.min(MOST) {
                let p = cam.to_screen(size, h.member_at(pos, j, t));
                c.circle(p.x, p.y, (dot * 0.7).max(1.5), eg(col));
            }
        } else if danger {
            let k = dot + 2.0;
            c.triangle(Vec2::new(q.x, q.y - k), Vec2::new(q.x - k, q.y + k * 0.8), Vec2::new(q.x + k, q.y + k * 0.8), ega(col, 0.9));
        } else {
            c.circle(q.x, q.y, dot + (alive as f32).sqrt() * 0.35, ega(col, 0.75));
        }
        // A pack shadowing travellers: a line to its quarry.
        if let Some(hu) = h.hunt {
            if let Some(g) = w.group(hu.victim) {
                c.line(q, cam.to_screen(size, g.position_at(t)), 1.5, ega([1.0, 0.3, 0.2], 0.8));
            }
        }
    }
    if cam.zoom > 0.25 {
        for p in &w.animals.pens {
            let q = cam.to_screen(size, p.home);
            if on_screen(q) {
                let r = (p.radius * cam.zoom).max(2.5);
                c.circle_lines(q.x, q.y, r, 1.0, ega(map_colour(p.sp), 0.8));
            }
        }
    }
    for col in &w.animals.colonies {
        let q = cam.to_screen(size, col.pos);
        if on_screen(q) {
            let k = dot + 1.5;
            c.rect(q.x - k, q.y - k, k * 2.0, k * 2.0, ega([0.93, 0.92, 0.86], 0.85));
        }
    }
    for car in w.animals.carcasses.iter().filter(|x| x.gone_at > t) {
        let q = cam.to_screen(size, car.pos);
        let k = dot;
        c.line(Vec2::new(q.x - k, q.y - k), Vec2::new(q.x + k, q.y + k), 1.5, eg([0.7, 0.4, 0.3]));
        c.line(Vec2::new(q.x - k, q.y + k), Vec2::new(q.x + k, q.y - k), 1.5, eg([0.7, 0.4, 0.3]));
    }
    for (pos, _) in w.flocks() {
        let q = cam.to_screen(size, pos);
        c.circle_lines(q.x, q.y, dot + 5.0, 1.0, eg([0.2, 0.2, 0.22]));
    }
    for h in w.tamed() {
        if let Some(p) = w.hound_pos(h.id) {
            let q = cam.to_screen(size, p);
            c.circle(q.x, q.y, dot, eg(GOLD));
        }
    }
    if PANEL.load(Ordering::Relaxed) {
        // The region grid, and where the Overgrowth stands.
        for r in 0..(ACROSS * ACROSS) as u16 {
            let ctr = region_centre(r);
            let tl = cam.to_screen(size, V2::new(ctr.x - REGION / 2.0, ctr.y - REGION / 2.0));
            let s = REGION * cam.zoom;
            let og = w.overgrowth(r);
            if og > 0.0 {
                c.rect(tl.x, tl.y, s, s, ega([0.2, 0.5, 0.15], 0.1 + 0.3 * og));
            }
            c.rect_lines(tl.x, tl.y, s, s, 1.0, ega([1.0, 1.0, 1.0], 0.12));
        }
    }
}

// ---- Hover, the wildlife panel, debug keys ----------------------------------------

fn amounts(list: &[(&'static str, f32)]) -> String {
    list.iter().map(|(k, a)| if *a == a.round() { format!("{} {:.0}", k.replace('_', " "), a) } else { format!("{} {:.1}", k.replace('_', " "), a) }).collect::<Vec<_>>().join(", ")
}

/// What the hover box says about something.
pub fn describe(w: &World, s: &Seen) -> Vec<(String, Rgb)> {
    let t = w.time;
    let d = s.sp.def();
    let mut out: Vec<(String, Rgb)> = Vec::new();
    let wild_lines = |out: &mut Vec<(String, Rgb)>, herd: u32| {
        let h = &w.animals.herds[herd as usize];
        out.push((format!("Wild  ·  {} of {} in this {}", h.alive(t), h.cap, if d.toward.dangerous() && h.cap > 1 { "pack" } else if h.cap > 1 { "herd" } else { "lair" }), TEXT));
        out.push((format!("Now: {}", w.herd_doing(herd)), TEXT));
        out.push((format!("Up and about {}  ·  {}", d.active.words(), d.toward.words()), DIM));
        if d.protected {
            out.push(("Protected".to_string(), GOLD));
        }
    };
    match s.thing {
        Thing::Animal(herd, slot) => {
            let h = &w.animals.herds[herd as usize];
            let m = h.member(slot, t);
            let mut name = d.name.to_string();
            if m.young {
                name = format!("Young {}", name.to_lowercase());
            }
            out.push((name, GOLD));
            wild_lines(&mut out, herd);
            if m.down {
                out.push(("Knocked out".to_string(), WARN));
            } else if m.hurt {
                out.push(("Hurt".to_string(), WARN));
            }
            if w.tameable(herd, slot) {
                out.push(("Could be tamed (Y)".to_string(), GOLD));
            }
        }
        Thing::Herd(herd) => {
            out.push((format!("{}s", d.name), GOLD));
            wild_lines(&mut out, herd);
        }
        Thing::Kept(pen, _) => {
            let p = &w.animals.pens[pen as usize];
            let now = p.now(t);
            out.push((d.name.to_string(), GOLD));
            let by = p.town.map(|s| format!(" by {}", w.settlements[s as usize].name)).unwrap_or_default();
            let whose = if p.owner == 0 { "no owner set".to_string() } else { format!("owner {}", p.owner) };
            out.push((format!("Kept{by}  ·  {whose}"), TEXT));
            out.push((format!("{} of {:.0} in the pen", now.count.floor(), p.limit), TEXT));
            let hunger = if now.hunger < 0.15 { "well fed" } else if now.hunger < 0.5 { "peckish" } else if now.hunger < 0.85 { "hungry" } else { "starving" };
            out.push((format!("{hunger}  ·  feed in store {:.0}", now.feed), if now.hunger > 0.5 { WARN } else { TEXT }));
            let made = p.yield_between(t - DAY, t);
            if !made.is_empty() {
                out.push((format!("Last day: {}", amounts(&made)), DIM));
            }
        }
        Thing::Led(k) => {
            out.push((d.name.to_string(), GOLD));
            let l = &w.animals.led[k];
            let who = w.people[l.leader as usize].name().unwrap_or("someone");
            out.push((format!("Kept  ·  following {who}  ·  carries up to {:.0} kg", w.plodder_capacity()), TEXT));
        }
        Thing::Hound(id) => {
            out.push(("Ridgehound".to_string(), GOLD));
            if let Some(h) = w.tamed().into_iter().find(|h| h.id == id) {
                let who = w.people[h.owner as usize].name().unwrap_or("someone");
                out.push((format!("Tamed  ·  follows {who}"), TEXT));
                let hunger = h.hunger_at(t);
                out.push((format!("Hunger {:.0}%", hunger * 100.0), if hunger > 0.7 { WARN } else { TEXT }));
            }
        }
        Thing::Fighting(k) => {
            out.push((d.name.to_string(), GOLD));
            if let Some(f) = w.animals_fighting().get(k) {
                let what = if f.down { "down" } else if f.fleeing { "running" } else { "fighting" };
                out.push((format!("{}  ·  {what}", if f.tame { "Tamed" } else { "Wild" }), TEXT));
            }
        }
        Thing::Colony(id) => {
            out.push(("Silk colony".to_string(), GOLD));
            out.push((format!("{} cocoons hanging", w.cocoons(id)), TEXT));
            let c = &w.animals.colonies[id as usize];
            let m = &w.animals.herds[c.mother as usize];
            let mother = if m.alive(t) == 0 {
                "The Silk Mother is dead"
            } else if w.mother_guarding(id) {
                "The Silk Mother is here"
            } else {
                "The Silk Mother is away"
            };
            out.push((mother.to_string(), if w.mother_guarding(id) { WARN } else { TEXT }));
            out.push(("Take cocoons: U".to_string(), DIM));
        }
        Thing::Carcass(_) => {
            out.push((format!("{} carcass", d.name), GOLD));
            if !d.yields.is_empty() {
                out.push((format!("Gives: {}", amounts(&gahturiyu_sim::sim::animals::h_yields(s.sp, s.size))), DIM));
            }
        }
        Thing::Flock => {
            out.push((format!("{} Bonepickers", s.count), GOLD));
            out.push(("Wild  ·  picking a body clean".to_string(), TEXT));
        }
    }
    if matches!(s.thing, Thing::Animal(..) | Thing::Herd(_)) && !d.yields.is_empty() {
        out.push((format!("Gives: {}", amounts(d.yields)), DIM));
    }
    out
}

/// The wildlife panel: one region's numbers, and the world's.
fn panel(c: &Canvas, w: &World, at: V2) -> egui::Rect {
    let t = w.time;
    let r = region_of(at);
    let reg = &w.animals.regions[r as usize];
    let mut lines: Vec<(String, Rgb)> = Vec::new();
    lines.push((format!("Wildlife  ·  region {},{}", r as usize % ACROSS, r as usize / ACROSS), GOLD));
    let country: Vec<String> = HABITATS.iter().filter(|h| reg.habitat[**h as usize] > 0.02).map(|h| format!("{} {:.1}", h.name(), reg.habitat[*h as usize])).collect();
    lines.push((format!("Country (km²): {}", if country.is_empty() { "sea".to_string() } else { country.join(", ") }), DIM));
    let og = w.overgrowth(r);
    lines.push((format!("Overgrowth {:.2}{}  ·  grazing pressure {:.2}", og, if og > BRIAR_THRESHOLD { " (breeding Briarbacks)" } else { "" }, w.grazing_pressure(r).max(0.0) + 0.0), if og > BRIAR_THRESHOLD { WARN } else { TEXT }));
    lines.push((String::new(), TEXT));
    let mut any = false;
    for sp in ALL_SPECIES {
        let (n, cap) = (w.population_at(r, sp, t), w.capacity(r, sp));
        if cap > 0.0 && !FISH.contains(&sp) {
            lines.push((format!("{:<14} {:>4.0} of {:.0}", sp.name(), n, cap), if sp.def().class == Class::Predator { [0.95, 0.6, 0.5] } else { TEXT }));
            any = true;
        }
    }
    if !any {
        lines.push(("Nothing wild lives here".to_string(), DIM));
    }
    let fish: Vec<String> = FISH.iter().filter(|sp| w.capacity(r, **sp) > 0.0).map(|sp| format!("{} {:.0}", sp.name(), w.fish_stock(r, *sp, t))).collect();
    if !fish.is_empty() {
        lines.push((format!("Fish: {}", fish.join(", ")), [0.6, 0.78, 0.95]));
    }
    let kept: f32 = w.animals.pens.iter().filter(|p| p.region == r).map(|p| p.now(t).count.floor()).sum();
    if kept > 0.0 {
        lines.push((format!("Kept animals: {kept:.0}"), TEXT));
    }
    lines.push((String::new(), TEXT));
    let (wild, tame) = w.animal_count();
    let st = &w.animals.stats;
    lines.push((format!("World: {wild} wild in {} herds, {tame} kept in {} pens", w.animals.herds.len(), w.animals.pens.len()), TEXT));
    lines.push((format!("Attacks on people {}  ·  thought better of {}  ·  bodies cleared {}", st.attacks, st.left_alone, st.bodies_cleared), TEXT));
    lines.push((format!("Seen up close so far: {}  ·  killed {}", st.met, st.animals_killed), TEXT));
    let part = match gahturiyu_sim::sim::animals::phase(t) {
        Phase::Night => "night",
        Phase::Dawn => "dawn",
        Phase::Day => "day",
        Phase::Dusk => "dusk",
    };
    lines.push((format!("It is {part}  ·  {}", tide::tide_word(t)), DIM));
    for a in w.animals.attacks.iter().rev().take(4) {
        let line = if a.by_squad {
            let how = if !a.over { "fighting".to_string() } else { format!("{} killed", a.animals_lost) };
            format!("{}  your squad hunted {} ×{}: {how}", super::hud::hhmm(a.t), a.sp.name(), a.pack)
        } else {
            let who = if a.victim.is_some() { "travellers" } else { "your squad" };
            let how = if !a.over { "fighting" } else if a.animals_won { "drove them off" } else { "driven off" };
            format!("{}  {} ×{} on {} {who}: {how}", super::hud::hhmm(a.t), a.sp.name(), a.pack, a.people)
        };
        lines.push((line, DIM));
    }
    lines.push((String::new(), TEXT));
    lines.push(("F7 this panel  ·  H hunt  ·  Y tame  ·  U take cocoons".to_string(), DIM));
    c.panel(&lines, c.w - 470.0, 12.0, 14.0)
}

/// Hover, the wildlife panel and the debug keys. Called once a frame from
/// the window's panel pass, after everything else has had its turn at the
/// mouse.
pub fn overlay(c: &Canvas, game: &mut Game, scene: &super::scene::Scene3d, panels: &mut Vec<Bx>) {
    if game.world.animals.regions.is_empty() {
        return;
    }
    let ctx = c.p.ctx().clone();
    let pressed = |k: egui::Key| ctx.input(|i| i.key_pressed(k));
    if pressed(egui::Key::F7) {
        PANEL.fetch_xor(true, Ordering::Relaxed);
    }
    let w = &mut game.world;
    let here = w.squad.pos;
    if pressed(egui::Key::H) {
        let who = game.sel.who(w);
        let said = match w.herd_near(here, 90.0) {
            Some(h) if w.hunt(&who, h) => None,
            Some(_) => Some("Too far off to set about them (get within 40 m)."),
            None => Some("No wild animals close by."),
        };
        if let Some(s) = said {
            game.notice = Some((s.to_string(), std::time::Instant::now()));
        }
    }
    if pressed(egui::Key::Y) {
        let t = w.time;
        let mut best: Option<(f32, u32, usize, u32)> = None;
        for h in &w.animals.herds {
            if !h.def().tameable || h.home.dist(here) > 3000.0 {
                continue;
            }
            for j in 0..h.alive(t) {
                if !w.tameable(h.id, j) {
                    continue;
                }
                let at = w.animal_pos(h.id, j, t);
                for &m in &w.squad.members {
                    let d = w.person_pos(m).dist(at);
                    if best.map(|b| d < b.0).unwrap_or(true) {
                        best = Some((d, h.id, j, m));
                    }
                }
            }
        }
        let said = match best {
            Some((_, h, j, m)) => match w.tame(m, h, j) {
                Ok(_) => "Tamed.".to_string(),
                Err(e) => format!("Taming: {e}."),
            },
            None => "No Ridgehound down or young near enough to tame.".to_string(),
        };
        game.notice = Some((said, std::time::Instant::now()));
    }
    if pressed(egui::Key::U) {
        let near = w.animals.colonies.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).map(|c| (c.id, c.pos));
        let said = match near {
            Some((id, pos)) => {
                let who = w.squad.members.iter().copied().min_by(|&a, &b| w.person_pos(a).dist(pos).total_cmp(&w.person_pos(b).dist(pos)));
                match who.map(|m| w.take_cocoons(m, id)) {
                    Some(Take::Taken(n)) => format!("Took {n} cocoons."),
                    Some(Take::Fight(_)) => "The Silk Mother is on you!".to_string(),
                    Some(Take::Guarded) => "The Silk Mother is watching.".to_string(),
                    Some(Take::Empty) => "No cocoons hanging yet.".to_string(),
                    Some(Take::TooFar) => "Too far from a colony (stand right at it).".to_string(),
                    _ => "No colony here.".to_string(),
                }
            }
            None => "No silk colonies in this world.".to_string(),
        };
        game.notice = Some((said, std::time::Instant::now()));
    }

    let w = &game.world;
    let size = game.screen;
    let on_panels = panels.iter().any(|b| b.contains(game.mouse));
    // What's under the mouse, if the window's own hover found nothing.
    if game.hover.is_none() && !on_panels {
        let mut best: Option<(f32, Seen)> = None;
        match game.view {
            View::Map => {
                let at = game.map_cam.to_world(size, game.mouse);
                for s in sightings(w, at, 60.0 / game.map_cam.zoom.max(0.01)) {
                    let d = (game.map_cam.to_screen(size, s.at) - game.mouse).length();
                    if d < 12.0 && best.map(|b| d < b.0).unwrap_or(true) {
                        best = Some((d, s));
                    }
                }
            }
            View::Scene => {
                let vp = game.orbit.view_proj(size);
                let k = (game.orbit.dist / 220.0).max(1.0).min(4.0);
                for s in sightings(w, game.orbit.target, game.orbit.draw_radius()) {
                    let lk = &s.sp.def().looks;
                    let ground = if geo::inland(s.at) < 0.0 { 0.0 } else { scene.grid.height(&w.terrain, s.at) };
                    let tall = if s.down { 0.2 } else { lk.tall * s.size * k * 0.6 };
                    let Some(q) = game.orbit.project(&vp, size, to3(s.at, ground + tall)) else { continue };
                    let slack = (lk.len * k * 0.5 * size.y / game.orbit.dist.max(1.0)).clamp(0.0, 40.0);
                    let d = (q - game.mouse).length() - slack;
                    if d < 12.0 && best.map(|b| d < b.0).unwrap_or(true) {
                        best = Some((d, s));
                    }
                }
            }
        }
        if let Some((_, s)) = best {
            c.panel(&describe(w, &s), game.mouse.x + 18.0, game.mouse.y + 12.0, 16.0);
        }
    }
    if PANEL.load(Ordering::Relaxed) {
        let at = match game.view {
            View::Map => game.map_cam.to_world(size, game.mouse),
            View::Scene => game.orbit.target,
        };
        let clamp = V2::new(at.x.clamp(0.0, geo::WORLD_SIZE - 1.0), at.y.clamp(0.0, geo::WORLD_SIZE - 1.0));
        panels.push(Bx::from(panel(c, w, clamp)));
    }
}

// ---- Screenshot scenes ----------------------------------------------------------

/// `GAHT_ANIMALS=…` sets a scene up for a screenshot (or for a look):
///
/// - `hounds`: a Ridgehound pack shadowing a traveller, at dusk if it can
///   find one;
/// - `tide`: Tidepickers out over the rock pools at low water;
/// - `silk`: a Silk Mother at home on her colony;
/// - `bones`: Bonepickers come down where the squad has made a kill;
/// - `see:<species key>` (e.g. `see:wallowback`): stand by the nearest of
///   them, at an hour they're about;
/// - `pens`: stand by the nearest town's livestock;
/// - `parade`: one of every animal stood in a row by the squad (shapes only);
/// - add `,panel` to any of them (or use `panel` alone) for the wildlife panel.
///
/// Called once from `Shot::prepare`.
pub fn prepare(world: &mut World) {
    let Ok(what) = std::env::var("GAHT_ANIMALS") else { return };
    if world.animals.regions.is_empty() {
        return;
    }
    for part in what.split(',') {
        scene(world, part.trim());
    }
}

fn step_to(world: &mut World, t: f64) {
    while world.time < t {
        world.step(60.0f64.min(t - world.time).max(0.01));
    }
}

fn scene(world: &mut World, what: &str) {
    use gahturiyu_sim::sim::animals::{phase, species};
    match what {
        "panel" => PANEL.store(true, Ordering::Relaxed),
        "parade" => {
            // Out of town, on open ground.
            world.teleport_squad(world.squad.pos.add(V2::new(260.0, 40.0)));
            world.step(0.1);
            PARADE.store(true, Ordering::Relaxed);
        }
        "hounds" => {
            // Run on until a pack has picked travellers up (at dusk for
            // choice), then stand off to one side of them.
            let end = world.time + 8.0 * DAY;
            let mut found = None;
            let mut fallback = None;
            while world.time < end && found.is_none() {
                world.step(20.0);
                let t = world.time;
                for h in world.animals.herds.iter().filter(|h| h.sp == Sp::Ridgehound) {
                    let Some(hu) = h.hunt else { continue };
                    // Well into the stalk, with the pack closed up, but not
                    // so late that it's over before the picture is taken.
                    let f = (t - hu.seen) / (hu.strike - hu.seen).max(1.0);
                    if hu.strike - t < 40.0 || f < 0.8 {
                        continue;
                    }
                    if phase(t) == Phase::Dusk {
                        found = Some((h.id, hu.victim));
                    } else if fallback.is_none() && world.time > end - 2.0 * DAY {
                        fallback = Some((h.id, hu.victim));
                        found = fallback;
                    }
                }
            }
            if let Some((herd, victim)) = found {
                let t = world.time;
                if let Some(g) = world.group(victim) {
                    // Beside the pack, looking past it at its quarry.
                    let (a, b) = (g.position_at(t), world.herd_pos(herd, t));
                    let off = b.sub(a);
                    let len = off.len().max(1.0);
                    let side = V2::new(-off.y, off.x).scale(12.0 / len);
                    world.teleport_squad(b.add(off.scale(6.0 / len)).add(side));
                    world.step(0.1);
                }
            }
        }
        "tide" => {
            let Some(herd) = world.animals.herds.iter().filter(|h| h.sp == Sp::Tidepicker).min_by(|a, b| a.home.dist(world.squad.pos).total_cmp(&b.home.dist(world.squad.pos))).map(|h| h.id) else { return };
            // The next low water that falls in daylight.
            let mut low = tide::low_near(world.time + 6.0 * HOUR);
            for _ in 0..30 {
                if low > world.time && phase(low) == Phase::Day {
                    break;
                }
                low += tide::TIDE_PERIOD;
            }
            step_to(world, low);
            let at = world.herd_pos(herd, world.time);
            world.teleport_squad(at.add(V2::new(22.0, 6.0)));
            world.step(0.1);
        }
        "silk" => {
            let here = world.squad.pos;
            let Some((id, pos)) = world.animals.colonies.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).map(|c| (c.id, c.pos)) else { return };
            let end = world.time + 3.0 * DAY;
            // Around midday, so the cliff isn't in shadow.
            let midday = |t: f64| (10.5..14.0).contains(&(t.rem_euclid(DAY) / HOUR));
            while world.time < end && !(world.mother_guarding(id) && midday(world.time)) {
                world.step(300.0);
            }
            // Just outside her ground.
            world.teleport_squad(pos.add(V2::new(34.0, 10.0)));
            world.step(0.1);
        }
        "bones" => {
            // A fight by day, out of town, in which a bandit dies (one is
            // made to, if the fight is kinder); then wait for the birds.
            let mut t = world.time;
            while phase(t) != Phase::Day || phase(t + 4.0 * HOUR) != Phase::Day {
                t += 600.0;
            }
            step_to(world, t);
            world.teleport_squad(world.squad.pos.add(V2::new(260.0, 40.0)));
            world.step(0.1);
            let at = world.squad.pos.add(V2::new(14.0, 5.0));
            world.spawn_bandits(at, 3, false);
            for _ in 0..400 {
                world.step(0.25);
                if world.squad_battle().is_some() {
                    break;
                }
            }
            if let Some(b) = world.battles.iter_mut().find(|b| b.fighters.iter().any(|f| f.side == 0)) {
                if let Some(f) = b.fighters.iter_mut().find(|f| f.side != 0 && f.is_person()) {
                    f.dead = true;
                    f.ko = true;
                }
            }
            for _ in 0..4000 {
                world.step(0.25);
                if world.squad_battle().is_none() {
                    break;
                }
            }
            let Some(body) = world.corpses.iter().map(|c| c.0).min_by(|a, b| a.dist(world.squad.pos).total_cmp(&b.dist(world.squad.pos))) else { return };
            let end = world.time + 100.0 * 60.0;
            while world.time < end && !world.flocks().iter().any(|f| f.0.dist(body) < 5.0) {
                world.step(20.0);
            }
            step_to(world, world.time + 200.0);
            world.teleport_squad(body.add(V2::new(15.0, 7.0)));
            world.step(0.1);
        }
        "pens" => {
            let here = world.squad.pos;
            if let Some(p) = world.animals.pens.iter().filter(|p| p.sp != Sp::Raftback).min_by(|a, b| a.home.dist(here).total_cmp(&b.home.dist(here))) {
                let at = p.home.add(V2::new(p.radius + 10.0, 4.0));
                world.teleport_squad(at);
                world.step(0.1);
            }
        }
        _ => {
            let Some(sp) = what.strip_prefix("see:").and_then(species) else { return };
            let here = world.squad.pos;
            if sp.def().class == Class::Domestic {
                if let Some(p) = world.animals.pens.iter().filter(|p| p.sp == sp).min_by(|a, b| a.home.dist(here).total_cmp(&b.home.dist(here))) {
                    let at = p.home.add(V2::new(p.radius + 9.0, 3.0));
                    world.teleport_squad(at);
                    world.step(0.1);
                }
                return;
            }
            let herd = world.animals.herds.iter().filter(|h| h.sp == sp && h.alive(world.time) > 0).min_by(|a, b| a.home.dist(here).total_cmp(&b.home.dist(here))).map(|h| h.id);
            let Some(herd) = herd else { return };
            // An hour when they're up and about.
            let mut t = world.time;
            for _ in 0..200 {
                if world.animals.herds[herd as usize].active(t + 900.0) && !world.animals.herds[herd as usize].mother_out(t + 900.0) {
                    break;
                }
                t += 900.0;
            }
            step_to(world, t + 900.0);
            let at = world.herd_pos(herd, world.time);
            let d = sp.def();
            // Far enough off not to start anything or scare them.
            let stand = (d.flight.max(if d.toward.dangerous() { d.sense } else { 0.0 }) + 14.0).min(190.0);
            world.teleport_squad(at.add(V2::new(stand, stand * 0.2)));
            world.step(0.1);
        }
    }
}
