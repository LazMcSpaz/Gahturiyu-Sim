//! The 3D view: an orbiting camera over the ground around the squad.
//!
//! Everything here is built fresh each frame from the simulation, so the view
//! can never disagree with it. Buildings are drawn at true size; people are
//! true size up close and scaled up as the camera pulls back, so they stay
//! visible the way units do in a strategy game.

use macroquad::prelude::*;

use gahturiyu_sim::sim::{
    bands::{BAND1_RADIUS, BAND2_RADIUS},
    geo::{self, V2, WORLD_SIZE},
    group::Kind,
    person::PersonId,
    race::Race,
    rng,
    settlement::{Building, BuildingKind},
    World,
};

use super::mesh::Builder;
use super::ui::{race_color, Hover, Picker, Ui, TEXT};

const SKY: Color = Color::new(0.63, 0.69, 0.72, 1.0);
const GRASS: Color = Color::new(0.30, 0.40, 0.25, 1.0);
const GRASS_DRY: Color = Color::new(0.42, 0.45, 0.30, 1.0);
const SHINGLE: Color = Color::new(0.46, 0.46, 0.43, 1.0);
const SEA: Color = Color::new(0.22, 0.32, 0.34, 1.0);
const SEA_DEEP: Color = Color::new(0.16, 0.24, 0.28, 1.0);
const STONE: Color = Color::new(0.38, 0.39, 0.41, 1.0);
const SANDSTONE: Color = Color::new(0.78, 0.63, 0.42, 1.0);
const WEAVE: Color = Color::new(0.24, 0.21, 0.16, 1.0);
const TIMBER: Color = Color::new(0.36, 0.27, 0.18, 1.0);
const GOLD: Color = Color::new(0.95, 0.76, 0.28, 1.0);
const EMBER: Color = Color::new(1.0, 0.62, 0.26, 1.0);
const TENT: Color = Color::new(0.62, 0.64, 0.74, 1.0);

/// Sea surface height, just below the land.
const SEA_LEVEL: f32 = -0.35;
/// Height of a Horaro stilt-home deck above the water.
const DECK: f32 = 2.4;

pub struct OrbitCam {
    /// The point the camera circles, on the ground (sim x, sim y).
    pub target: V2,
    pub yaw: f32,
    pub pitch: f32,
    /// Metres from target.
    pub dist: f32,
}

impl OrbitCam {
    pub fn new(target: V2) -> OrbitCam {
        OrbitCam { target, yaw: 0.35, pitch: 0.48, dist: 110.0 }
    }

    pub fn camera(&self) -> Camera3D {
        let t = vec3(self.target.x, 0.0, self.target.y);
        let off = vec3(self.pitch.cos() * self.yaw.cos(), self.pitch.sin(), self.pitch.cos() * self.yaw.sin()) * self.dist;
        Camera3D {
            position: t + off,
            target: t,
            up: Vec3::Y,
            fovy: 50f32.to_radians(),
            z_near: (self.dist * 0.01).clamp(0.2, 5.0),
            z_far: self.draw_radius() * 1.6 + self.dist,
            ..Default::default()
        }
    }

    /// How far out from the target anything is drawn.
    pub fn draw_radius(&self) -> f32 {
        (self.dist * 5.0).clamp(900.0, 5500.0)
    }

    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * 0.006;
        self.pitch = (self.pitch + dy * 0.004).clamp(0.12, 1.50);
    }

    pub fn zoom(&mut self, wheel: f32) {
        self.dist = (self.dist * if wheel > 0.0 { 0.87 } else { 1.0 / 0.87 }).clamp(12.0, 4500.0);
    }

    /// Slide the target across the ground, relative to where the camera faces.
    pub fn pan(&mut self, right: f32, forward: f32) {
        let fwd = V2::new(-self.yaw.cos(), -self.yaw.sin());
        let side = V2::new(-fwd.y, fwd.x);
        let s = self.dist * 0.9;
        self.target = geo_clamp(self.target.add(fwd.scale(forward * s)).add(side.scale(-right * s)));
    }
}

fn geo_clamp(p: V2) -> V2 {
    V2::new(p.x.clamp(0.0, WORLD_SIZE), p.y.clamp(0.0, WORLD_SIZE))
}

fn to3(p: V2, h: f32) -> Vec3 {
    vec3(p.x, h, p.y)
}

/// Screen position of a world point, or None if it is behind the camera.
pub fn project(cam: &Camera3D, p: Vec3) -> Option<Vec2> {
    let c = cam.matrix() * p.extend(1.0);
    if c.w <= 0.0 {
        return None;
    }
    let n = c.truncate() / c.w;
    if n.z > 1.0 {
        return None;
    }
    Some(vec2((n.x + 1.0) * 0.5 * screen_width(), (1.0 - n.y) * 0.5 * screen_height()))
}

/// The ground point under a screen position.
pub fn ground_at(cam: &Camera3D, s: Vec2) -> Option<V2> {
    let inv = cam.matrix().inverse();
    let nx = s.x / screen_width() * 2.0 - 1.0;
    let ny = 1.0 - s.y / screen_height() * 2.0;
    let a = inv * vec4(nx, ny, -1.0, 1.0);
    let b = inv * vec4(nx, ny, 1.0, 1.0);
    let (a, b) = (a.truncate() / a.w, b.truncate() / b.w);
    let d = b - a;
    if d.y >= -1e-6 {
        return None;
    }
    let p = a + d * (-a.y / d.y);
    Some(V2::new(p.x, p.z))
}

/// Smooth value noise in 0..1, for patchy ground.
fn noise(x: f32, y: f32, scale: f32, seed: u64) -> f32 {
    let (fx, fy) = (x / scale, y / scale);
    let (ix, iy) = (fx.floor(), fy.floor());
    let (tx, ty) = (fx - ix, fy - iy);
    let h = |a: f32, b: f32| (rng::key(&[seed, a as i64 as u64, b as i64 as u64]) >> 40) as f32 / (1u64 << 24) as f32;
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let top = h(ix, iy) + (h(ix + 1.0, iy) - h(ix, iy)) * s(tx);
    let bot = h(ix, iy + 1.0) + (h(ix + 1.0, iy + 1.0) - h(ix, iy + 1.0)) * s(tx);
    top + (bot - top) * s(ty)
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, 1.0)
}

fn scale_color(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}

/// Things kept between frames. The ground only changes when the camera
/// moves a whole grid cell or zooms, so it is rebuilt only then.
#[derive(Default)]
pub struct SceneCache {
    ground: Option<((i64, i64, u32, u32), Builder)>,
}

const GROUND_CELLS: usize = 90;

pub fn draw(ui: &Ui, oc: &OrbitCam, w: &World, rings: bool, pick: &mut Picker, cache: &mut SceneCache) {
    clear_background(SKY);
    let cam = oc.camera();
    set_camera(&cam);
    let radius = oc.draw_radius();

    // Snap the ground grid to the world so cells don't swim as the camera moves.
    let step = (radius * 2.0 / GROUND_CELLS as f32).max(1.0);
    let key = (
        (oc.target.x / step).floor() as i64,
        (oc.target.y / step).floor() as i64,
        radius.to_bits(),
        (oc.dist * oc.pitch.sin()).round() as u32,
    );
    if cache.ground.as_ref().map(|(k, _)| *k != key).unwrap_or(true) {
        // Ground fog is measured from above the target, not from the camera,
        // so turning the camera doesn't force a rebuild.
        let eye = vec3(oc.target.x, oc.dist * oc.pitch.sin(), oc.target.y);
        let mut g = Builder::new(eye, radius * 0.35, radius * 1.05, SKY);
        let snapped = V2::new(key.0 as f32 * step, key.1 as f32 * step);
        ground(&mut g, snapped, radius, step);
        cache.ground = Some((key, g.finish()));
    }
    if let Some((_, g)) = &cache.ground {
        g.draw();
    }
    let mut b = Builder::new(cam.position, radius * 0.35, radius * 1.05, SKY);

    // Towns.
    let mut labels: Vec<(Vec3, String)> = Vec::new();
    for s in &w.settlements {
        if s.pos.dist(oc.target) > radius + s.reach {
            continue;
        }
        for bd in &s.buildings {
            if bd.pos.dist(oc.target) < radius {
                building(&mut b, bd);
            }
        }
        labels.push((to3(s.pos, 28.0 + s.radius() * 0.08), s.name.clone()));
    }

    // People and groups.
    let k = (oc.dist / 220.0).max(1.0);
    let mut heads: Vec<(Vec3, PersonId)> = Vec::new();
    for s in &w.settlements {
        for pid in w.residents_in_band1(s.id) {
            heads.push(person(&mut b, w, pid, k));
        }
    }
    let mut markers: Vec<(Vec3, u32)> = Vec::new();
    for g in &w.groups {
        if g.pos.dist(oc.target) > radius {
            continue;
        }
        if g.band == 1 {
            if matches!(g.kind, Kind::Wanderer { .. }) && !g.is_moving(w.time) {
                let r = w.people[g.members[0] as usize].seed;
                let at = g.pos.add(V2::new(3.5, 2.0));
                tent(&mut b, to3(at, 0.0), k, r);
            }
            for &m in &g.members {
                heads.push(person(&mut b, w, m, k));
            }
        } else {
            let lead = w.people[g.members[0] as usize].race;
            let mk = (oc.dist / 60.0).max(2.0);
            let base = to3(g.pos, 0.0);
            let n = g.members.len() as f32;
            b.column(base, 0.25 * mk, 0.25 * mk, 3.0 * mk, 4, scale_color(race_color(lead), 0.6));
            b.column(base + vec3(0.0, 3.0 * mk, 0.0), (0.6 + 0.12 * n) * mk, 0.0, 1.3 * mk, 4, race_color(lead));
            markers.push((base + vec3(0.0, 3.6 * mk, 0.0), g.id));
        }
    }
    for &m in &w.squad.members {
        heads.push(person(&mut b, w, m, k));
    }

    // Rings and the order line as flat ribbons on the ground. (macroquad's
    // 3D line call is one draw per segment, which is slow.)
    let sq = to3(w.squad.pos, 0.25);
    let lw = (oc.dist / 260.0).max(0.12);
    ribbon_ring(&mut b, sq, 6.0 * k, lw, 32, WHITE);
    if w.squad.pos.dist(w.squad.target) > 1.0 {
        let tg = to3(w.squad.target, 0.25);
        ribbon(&mut b, sq, tg, lw * 0.7, Color::new(0.95, 0.95, 0.95, 1.0));
        ribbon_ring(&mut b, tg, 2.5 * k, lw, 16, WHITE);
    }
    if rings {
        ribbon_ring(&mut b, sq, BAND1_RADIUS, lw * 2.0, 128, Color::new(0.92, 0.94, 0.95, 1.0));
        ribbon_ring(&mut b, sq, BAND2_RADIUS, lw * 3.0, 192, Color::new(0.80, 0.84, 0.86, 1.0));
    }
    b.finish().draw();

    // Back to flat screen space for text and picking.
    set_default_camera();
    for (p, id) in heads {
        if let Some(s) = project(&cam, p) {
            pick.offer(s, 2.0, Hover::Person(id));
        }
    }
    for (p, id) in markers {
        if let Some(s) = project(&cam, p) {
            pick.offer(s, 2.0, Hover::Group(id));
        }
    }
    for (s, (p, name)) in w.settlements.iter().filter(|s| s.pos.dist(oc.target) <= radius + s.reach).zip(labels) {
        if let Some(q) = project(&cam, p) {
            if p.distance(cam.position) < radius * 1.1 {
                ui.centred(&name, q.x, q.y, 17, TEXT);
                pick.offer(q, 20.0, Hover::Town(s.id));
            }
        }
    }
}

/// A flat line on the ground from `a` to `c`.
fn ribbon(b: &mut Builder, a: Vec3, c: Vec3, width: f32, col: Color) {
    let d = (c - a).normalize_or_zero();
    let side = vec3(-d.z, 0.0, d.x) * (width * 0.5);
    let k = b.fogged(col, a);
    b.quad_raw([a - side, a + side, c + side, c - side], [k; 4]);
}

fn ribbon_ring(b: &mut Builder, c: Vec3, r: f32, width: f32, n: usize, col: Color) {
    for i in 0..n {
        let a0 = i as f32 / n as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / n as f32 * std::f32::consts::TAU;
        let (p0, p1) = (vec3(a0.cos(), 0.0, a0.sin()), vec3(a1.cos(), 0.0, a1.sin()));
        let (i0, o0) = (c + p0 * (r - width * 0.5), c + p0 * (r + width * 0.5));
        let (i1, o1) = (c + p1 * (r - width * 0.5), c + p1 * (r + width * 0.5));
        let k = b.fogged(col, o0);
        b.quad_raw([i0, o0, o1, i1], [k; 4]);
    }
}

/// Land and sea, as a grid of cells around the target. Cells the shoreline
/// runs through are split along it, so the coast is a clean line.
fn ground(b: &mut Builder, centre: V2, radius: f32, step: f32) {
    let cells = GROUND_CELLS;
    let x0 = (centre.x - radius).max(-500.0);
    let y0 = centre.y - radius;
    let land_col = |x: f32, y: f32| -> Color {
        let inland = geo::inland(V2::new(x, y));
        let base = mix(GRASS, GRASS_DRY, noise(x, y, 220.0, 7) * 0.8 + noise(x, y, 45.0, 9) * 0.3);
        let base = scale_color(base, 0.88 + noise(x, y, 18.0, 11) * 0.2);
        if inland < 40.0 {
            mix(SHINGLE, base, (inland / 40.0).clamp(0.0, 1.0))
        } else {
            base
        }
    };
    let sea_col = |x: f32, y: f32| -> Color {
        let out = -geo::inland(V2::new(x, y));
        mix(SEA, SEA_DEEP, (out / 600.0).clamp(0.0, 1.0))
    };
    let lit = |b: &Builder, c: Color, x: f32, h: f32, y: f32| b.shade(c, Vec3::Y, vec3(x, h, y));
    for j in 0..cells {
        let (ya, yb) = (y0 + j as f32 * step, y0 + (j + 1) as f32 * step);
        let (ca, cb) = (geo::coast_x(ya), geo::coast_x(yb));
        // Grid columns that lie wholly on one side of the shore stay square;
        // the strip the shore runs through is one exact sea piece and one
        // exact land piece, split along the coastline itself.
        let first_land = (((ca.max(cb) - x0) / step).ceil().max(0.0) as usize).min(cells);
        let last_sea = (((ca.min(cb) - x0) / step).floor().max(0.0) as usize).min(cells);
        for i in 0..cells {
            let (xa, xb) = (x0 + i as f32 * step, x0 + (i + 1) as f32 * step);
            if i >= first_land {
                let p = [vec3(xa, 0.0, ya), vec3(xb, 0.0, ya), vec3(xb, 0.0, yb), vec3(xa, 0.0, yb)];
                let c = p.map(|q| lit(b, land_col(q.x, q.z), q.x, 0.0, q.z));
                b.quad_raw(p, c);
            } else if i < last_sea {
                let p = [vec3(xa, SEA_LEVEL, ya), vec3(xb, SEA_LEVEL, ya), vec3(xb, SEA_LEVEL, yb), vec3(xa, SEA_LEVEL, yb)];
                let c = p.map(|q| lit(b, sea_col(q.x, q.z), q.x, SEA_LEVEL, q.z));
                b.quad_raw(p, c);
            }
        }
        let xs = x0 + last_sea.min(cells) as f32 * step;
        let xl = x0 + first_land.min(cells) as f32 * step;
        if xl > xs {
            let sea = [vec3(xs, SEA_LEVEL, ya), vec3(ca.max(xs), SEA_LEVEL, ya), vec3(cb.max(xs), SEA_LEVEL, yb), vec3(xs, SEA_LEVEL, yb)];
            let c = sea.map(|q| lit(b, sea_col(q.x, q.z), q.x, SEA_LEVEL, q.z));
            b.quad_raw(sea, c);
            let land = [vec3(ca.min(xl), 0.0, ya), vec3(xl, 0.0, ya), vec3(xl, 0.0, yb), vec3(cb.min(xl), 0.0, yb)];
            let c = land.map(|q| lit(b, land_col(q.x, q.z), q.x, 0.0, q.z));
            b.quad_raw(land, c);
        }
    }
}

fn building(b: &mut Builder, bd: &Building) {
    let base = to3(bd.pos, 0.0);
    let r = bd.size * 0.5;
    let s = bd.seed;
    let unit = |k: u64| (rng::key(&[s, k]) >> 40) as f32 / (1u64 << 24) as f32;
    match bd.kind {
        BuildingKind::RoduroHome => {
            // Grown stone: a lumpy banded mound, sometimes with an upper
            // storey budding out of it, and a warm window facing the hearth.
            let h = bd.size * (0.42 + unit(1) * 0.18);
            b.dome(base, r * (0.95 + unit(2) * 0.2), r * (0.85 + unit(3) * 0.2), h, 0.10, 0.10, s, STONE);
            if bd.size > 9.5 {
                let off = vec3(unit(4) - 0.5, 0.0, unit(5) - 0.5) * r * 0.6;
                b.dome(base + off + vec3(0.0, h * 0.62, 0.0), r * 0.55, r * 0.5, h * 0.55, 0.12, 0.12, s ^ 7, STONE);
            }
            let (sn, cs) = bd.rot.sin_cos();
            let face = base + vec3(cs, 0.0, sn) * (r * 0.97);
            b.glow(face + vec3(0.0, h * 0.35, 0.0), 1.1, 0.8, bd.rot, EMBER);
            b.glow(face + vec3(0.0, 1.0, 0.0) + vec3(-sn, 0.0, cs) * 1.4, 1.0, 2.0, bd.rot, Color::new(0.08, 0.07, 0.06, 1.0));
        }
        BuildingKind::HoraroStilt => {
            // Narrow grown-stone pillars out of the water, a timber deck, and a
            // dark woven dome that bulges more on one side.
            let pillars = 5;
            for i in 0..pillars {
                let a = i as f32 / pillars as f32 * std::f32::consts::TAU + bd.rot;
                let p = base + vec3(a.cos(), 0.0, a.sin()) * (r * 0.55) + vec3(0.0, -2.0, 0.0);
                b.column(p, 0.45, 0.35, DECK + 2.0, 6, STONE);
            }
            b.column(base + vec3(0.0, DECK, 0.0), r * 0.95, r * 0.95, 0.3, 12, TIMBER);
            let off = vec3(bd.rot.cos(), 0.0, bd.rot.sin()) * (r * 0.12);
            b.dome(base + off + vec3(0.0, DECK + 0.3, 0.0), r * 0.62, r * 0.5, r * 0.62, 0.04, 0.0, s, WEAVE);
        }
        BuildingKind::QotiroBlock => {
            // Quarried and stepped: dense living quarters in sandstone.
            let tiers = 2 + (unit(1) * 1.5) as usize;
            let mut y = 0.0;
            let mut sz = bd.size;
            for t in 0..tiers {
                let h = 4.0 + unit(10 + t as u64) * 1.5;
                b.block(base + vec3(0.0, y, 0.0), sz, sz * 0.8, h, bd.rot, SANDSTONE);
                y += h;
                sz *= 0.72;
            }
        }
        BuildingKind::QotiroTemple => {
            // Three tiers: the wide market base, the living middle, and an
            // open summit with a temple and its gold sun disc.
            let mut y = 0.0;
            for (f, h) in [(1.0, 7.0), (0.68, 8.0), (0.40, 3.0)] {
                b.block(base + vec3(0.0, y, 0.0), bd.size * f, bd.size * f, h, bd.rot, SANDSTONE);
                y += h;
            }
            b.block(base + vec3(0.0, y, 0.0), bd.size * 0.16, bd.size * 0.2, 6.0, bd.rot, SANDSTONE);
            b.column(base + vec3(0.0, y + 6.0, 0.0), bd.size * 0.06, 0.0, 3.0, 8, GOLD);
            b.glow(base + vec3(0.0, y + 9.5, 0.0), 2.6, 2.6, bd.rot, GOLD);
        }
        BuildingKind::QotiroHall => {
            // Away from home: two tiers in local dark stone, the gold crown kept.
            b.block(base, bd.size, bd.size * 0.8, 4.5, bd.rot, STONE);
            b.block(base + vec3(0.0, 4.5, 0.0), bd.size * 0.5, bd.size * 0.45, 3.0, bd.rot, STONE);
            b.column(base + vec3(0.0, 7.5, 0.0), 0.9, 0.0, 2.2, 8, GOLD);
        }
        BuildingKind::Hearth => {
            for i in 0..9 {
                let a = i as f32 / 9.0 * std::f32::consts::TAU;
                b.block(base + vec3(a.cos(), 0.0, a.sin()) * 2.4, 0.9, 0.7, 0.55, a, STONE);
            }
            b.column(base, 1.4, 0.3, 1.2, 8, EMBER);
        }
    }
}

/// Draw one person; returns where their head is, for hover.
fn person(b: &mut Builder, w: &World, pid: PersonId, k: f32) -> (Vec3, PersonId) {
    let p = &w.people[pid as usize];
    let at = w.person_pos(pid);
    // Horaro at home on a stilt deck stand above the water.
    let floor = if geo::inland(at) < 0.0 { DECK + 0.3 } else { 0.0 };
    let (h, r, skin) = match p.race {
        Race::Roduro => (1.35, 0.34, Color::new(0.58, 0.47, 0.39, 1.0)), // short and stocky
        Race::Qotiro => (1.80, 0.34, Color::new(0.56, 0.57, 0.59, 1.0)), // grey-skinned, broad
        Race::Horaro => (1.65, 0.27, Color::new(0.36, 0.52, 0.78, 1.0)), // blue-skinned, hairless
        Race::Tadoro => (1.95, 0.22, Color::new(0.80, 0.75, 0.72, 1.0)), // tall and lean
    };
    let (h, r) = (h * k, r * k);
    let base = to3(at, floor);
    b.column(base, r, r * 0.8, h * 0.8, 6, race_color(p.race));
    let head = base + vec3(0.0, h * 0.8, 0.0);
    b.block(head, r * 1.1, r * 1.1, h * 0.2, 0.0, skin);
    (head + vec3(0.0, h * 0.2, 0.0), pid)
}

/// A Ṭaḍoro travelling tent: a taut cone of pale cloth around one pole.
fn tent(b: &mut Builder, base: Vec3, k: f32, seed: u64) {
    let r = 2.2 * k.min(4.0);
    b.column(base, r, 0.05, r * 1.5, 10, TENT);
    let tilt = (seed % 7) as f32 * 0.1;
    b.column(base + vec3(0.0, r * 1.5, 0.0), 0.06 * k, 0.04 * k, 0.6 * k + tilt, 4, TIMBER);
}
