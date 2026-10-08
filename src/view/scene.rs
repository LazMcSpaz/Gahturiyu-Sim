//! The 3D view: an orbiting camera over the land around the squad.
//!
//! Everything here is built from the simulation each frame (the ground is
//! cached until the camera moves), so the view can never disagree with it.
//! Buildings are drawn at true size; people are true size up close and scaled
//! up as the camera pulls back, so they stay visible the way units do in a
//! strategy game.

use macroquad::prelude::*;

use gahturiyu_sim::sim::{
    bands::{BAND1_RADIUS, BAND2_RADIUS},
    geo::{self, V2, WORLD_SIZE},
    group::Kind,
    combat::{FxKind, SQUAD_SIDE},
    magic::StatusKind,
    person::PersonId,
    race::Race,
    rng,
    buildings::{door_of, Door, DoorId},
    settlement::{Building, BuildingKind},
    stealth,
    terrain::Terrain,
    World,
};

use super::mesh::Builder;
use super::palette;
use super::squadui::{ground_color, Selection, GOLD as PICKED};
use super::ui::{race_color, Hover, Picker, Ui, TEXT};

const SKY: Color = Color::new(0.63, 0.69, 0.72, 1.0);
const STONE: Color = Color::new(0.38, 0.39, 0.41, 1.0);
const SANDSTONE: Color = Color::new(0.78, 0.63, 0.42, 1.0);
const WEAVE: Color = Color::new(0.24, 0.21, 0.16, 1.0);
const TIMBER: Color = Color::new(0.36, 0.27, 0.18, 1.0);
const GOLD: Color = Color::new(0.95, 0.76, 0.28, 1.0);
const EMBER: Color = Color::new(1.0, 0.62, 0.26, 1.0);
const TENT: Color = Color::new(0.62, 0.64, 0.74, 1.0);
const NIGHT_SKY: Color = Color::new(0.04, 0.05, 0.10, 1.0);
const CAMP_HIDE: Color = Color::new(0.42, 0.24, 0.18, 1.0);

/// Height of a Horaro stilt-home deck above the water.
const DECK: f32 = 2.4;
/// Cells across the fine ground patch around the camera target.
const GROUND_CELLS: usize = 90;
/// Cells across the coarse ring that carries the far land to the horizon.
const FAR_CELLS: usize = 72;

pub struct OrbitCam {
    /// The point the camera circles, on the ground (sim x, sim y).
    pub target: V2,
    /// Ground height under the target; kept up to date by the caller.
    pub ground: f32,
    pub yaw: f32,
    pub pitch: f32,
    /// Metres from target.
    pub dist: f32,
}

impl OrbitCam {
    pub fn new(target: V2) -> OrbitCam {
        OrbitCam { target, ground: 0.0, yaw: 0.35, pitch: 0.48, dist: 110.0 }
    }

    pub fn camera(&self) -> Camera3D {
        let t = vec3(self.target.x, self.ground, self.target.y);
        let off = vec3(self.pitch.cos() * self.yaw.cos(), self.pitch.sin(), self.pitch.cos() * self.yaw.sin()) * self.dist;
        Camera3D {
            position: t + off,
            target: t,
            up: Vec3::Y,
            fovy: 50f32.to_radians(),
            z_near: (self.dist * 0.01).clamp(0.2, 5.0),
            z_far: self.far_radius() * 1.3 + self.dist,
            ..Default::default()
        }
    }

    /// How far out the detailed world (buildings, people) is drawn.
    pub fn draw_radius(&self) -> f32 {
        (self.dist * 5.0).clamp(900.0, 5500.0)
    }

    /// How far out land of any kind is drawn.
    pub fn far_radius(&self) -> f32 {
        (self.draw_radius() * 3.0).max(9000.0)
    }

    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * 0.006;
        self.pitch = (self.pitch + dy * 0.004).clamp(0.08, 1.50);
    }

    pub fn zoom(&mut self, wheel: f32) {
        self.dist = (self.dist * if wheel > 0.0 { 0.87 } else { 1.0 / 0.87 }).clamp(12.0, 4500.0);
    }

    /// Slide the target across the ground, relative to where the camera faces.
    pub fn pan(&mut self, right: f32, forward: f32) {
        let fwd = V2::new(-self.yaw.cos(), -self.yaw.sin());
        let side = V2::new(-fwd.y, fwd.x);
        let s = self.dist * 0.9;
        let p = self.target.add(fwd.scale(forward * s)).add(side.scale(-right * s));
        self.target = V2::new(p.x.clamp(0.0, WORLD_SIZE), p.y.clamp(0.0, WORLD_SIZE));
    }
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

/// The point on the land (or sea) under a screen position: march along the
/// view ray until it dips below the surface, then home in.
pub fn ground_at(cam: &Camera3D, s: Vec2, t: &Terrain) -> Option<V2> {
    let inv = cam.matrix().inverse();
    let nx = s.x / screen_width() * 2.0 - 1.0;
    let ny = 1.0 - s.y / screen_height() * 2.0;
    let a = inv * vec4(nx, ny, -1.0, 1.0);
    let b = inv * vec4(nx, ny, 1.0, 1.0);
    let (a, b) = (a.truncate() / a.w, b.truncate() / b.w);
    let dir = (b - a).normalize();
    let above = |p: Vec3| p.y - t.surface(V2::new(p.x, p.z));
    let (mut lo, mut step) = (0.0f32, 2.0f32);
    let mut prev = a;
    for _ in 0..600 {
        let hi = lo + step;
        let p = a + dir * hi;
        if above(p) < 0.0 {
            // Bisect between the last point above and this one below.
            let (mut l, mut h) = (lo, hi);
            for _ in 0..20 {
                let m = (l + h) * 0.5;
                if above(a + dir * m) < 0.0 {
                    h = m;
                } else {
                    l = m;
                }
            }
            let q = a + dir * h;
            return Some(V2::new(q.x, q.z));
        }
        prev = p;
        lo = hi;
        step *= 1.04;
        if lo > 30_000.0 {
            break;
        }
    }
    let _ = prev;
    None
}

/// Things kept between frames. The ground only changes when the camera
/// moves a whole grid cell or zooms, so it is rebuilt only then.
#[derive(Default)]
pub struct SceneCache {
    ground: Option<((i64, i64, u32, u32, u32), Builder)>,
    grid: Grid,
}

/// Where the ground mesh's vertices are, so things laid on the ground can
/// follow the drawn surface exactly rather than the finer true terrain (which
/// would leave them buried between vertices on steep slopes).
#[derive(Default, Clone, Copy)]
struct Grid {
    centre: V2,
    half_fine: f32,
    fine: f32,
    coarse: f32,
    far: f32,
}

impl Grid {
    /// Height of the drawn ground at `p`, matching how each cell is split
    /// into two triangles.
    fn height(&self, t: &Terrain, p: V2) -> f32 {
        if self.fine <= 0.0 {
            return t.surface(p);
        }
        let inside = (p.x - self.centre.x).abs() < self.half_fine && (p.y - self.centre.y).abs() < self.half_fine;
        let (half, step) = if inside { (self.half_fine, self.fine) } else { (self.far, self.coarse) };
        let (x0, y0) = (self.centre.x - half, self.centre.y - half);
        let (fx, fy) = ((p.x - x0) / step, (p.y - y0) / step);
        let (i, j) = (fx.floor(), fy.floor());
        let (u, v) = (fx - i, fy - j);
        let corner = |di: f32, dj: f32| {
            let q = V2::new(x0 + (i + di) * step, y0 + (j + dj) * step);
            if geo::inland(q) >= 0.0 { t.height(q).max(0.3) } else { 0.0 }
        };
        let (a, b, c, d) = (corner(0.0, 0.0), corner(1.0, 0.0), corner(1.0, 1.0), corner(0.0, 1.0));
        // Triangles (a, b, c) and (a, c, d), as the quads are built.
        let h = if u >= v { a + (b - a) * u + (c - b) * v } else { a + (c - d) * u + (d - a) * v };
        if geo::inland(p) >= 0.0 { h } else { 0.0 }
    }
}

pub fn draw(ui: &Ui, oc: &OrbitCam, w: &World, rings: bool, pick: &mut Picker, cache: &mut SceneCache, sel: &Selection) {
    // Time of day: the light level is rounded so the cached ground is only
    // rebuilt a few dozen times through a dusk.
    let day = (stealth::daylight(w.time) * 24.0).round() / 24.0;
    let sky = Color::new(NIGHT_SKY.r + (SKY.r - NIGHT_SKY.r) * day, NIGHT_SKY.g + (SKY.g - NIGHT_SKY.g) * day, NIGHT_SKY.b + (SKY.b - NIGHT_SKY.b) * day, 1.0);
    clear_background(sky);
    let cam = oc.camera();
    set_camera(&cam);
    let radius = oc.draw_radius();
    let far = oc.far_radius();
    let t = &w.terrain;

    // The land, cached. Snap to a coarse grid so cells don't swim as the
    // camera moves, and so the fine patch lines up with the far ring.
    let coarse = far * 2.0 / FAR_CELLS as f32;
    let key = (
        (oc.target.x / coarse).round() as i64,
        (oc.target.y / coarse).round() as i64,
        radius.to_bits(),
        (oc.dist * oc.pitch.sin() / 25.0).round() as u32,
        (day * 24.0) as u32,
    );
    if cache.ground.as_ref().map(|(k, _)| *k != key).unwrap_or(true) {
        // Ground fog is measured from above the target, not from the camera,
        // so turning the camera doesn't force a rebuild.
        let eye = vec3(oc.target.x, oc.ground + oc.dist * oc.pitch.sin(), oc.target.y);
        let mut g = Builder::new(eye, radius * 0.45, far * 0.95, sky).lit(day, camp_lamps(w, &|p| t.surface(p)));
        let centre = V2::new(key.0 as f32 * coarse, key.1 as f32 * coarse);
        // Fine patch: a whole number of coarse cells, so its edge meets the ring.
        let half_fine = ((radius / coarse).ceil().max(1.0)) * coarse;
        let fine = half_fine * 2.0 / GROUND_CELLS as f32;
        ground_patch(&mut g, t, centre, half_fine, fine, None);
        ground_patch(&mut g, t, centre, far, coarse, Some(half_fine));
        cache.ground = Some((key, g.finish()));
        cache.grid = Grid { centre, half_fine, fine, coarse, far };
    }
    let grid = cache.grid;
    let on_ground = |p: V2| grid.height(t, p);
    if let Some((_, g)) = &cache.ground {
        g.draw();
    }

    let mut b = Builder::new(cam.position, radius * 0.45, far * 0.95, sky).lit(day, camp_lamps(w, &on_ground));

    // Roads.
    let lw = (oc.dist / 90.0).max(3.0);
    for road in &w.routes.roads {
        for seg in road.windows(2) {
            if seg[0].dist(oc.target) > radius {
                continue;
            }
            draped_ribbon(&mut b, &on_ground, seg[0], seg[1], lw, 0.3, palette::ROAD, (radius / 45.0).max(12.0));
        }
    }

    // Towns.
    let open = w.occupied();
    let mut doors: Vec<(Vec3, DoorId)> = Vec::new();
    let mut labels: Vec<(Vec3, String, u16)> = Vec::new();
    for s in &w.settlements {
        if s.pos.dist(oc.target) > radius + s.reach {
            continue;
        }
        for (i, bd) in s.buildings.iter().enumerate() {
            if bd.pos.dist(oc.target) < radius {
                let id = (s.id, i as u16);
                match door_of(s, i as u16) {
                    // Someone's inside: cut the walls and roof away.
                    Some(d) if open.contains(&id) => interior(&mut b, bd, &d, &on_ground),
                    _ => building(&mut b, t, bd, &on_ground),
                }
                if let Some(d) = door_of(s, i as u16) {
                    if d.outside.dist(oc.target) < 160.0 {
                        doors.push((to3(d.outside, on_ground(d.outside) + 1.2), id));
                    }
                }
            }
        }
        labels.push((to3(s.pos, on_ground(s.pos) + 28.0 + s.radius() * 0.08), s.name.clone(), s.id));
    }

    // People and groups.
    let k = (oc.dist / 220.0).max(1.0);
    let mut heads: Vec<(Vec3, PersonId)> = Vec::new();
    for s in &w.settlements {
        for pid in w.residents_in_band1(s.id) {
            heads.push(person(&mut b, w, pid, k, &on_ground));
        }
    }
    let mut markers: Vec<(Vec3, u32)> = Vec::new();
    for g in &w.groups {
        if g.pos.dist(oc.target) > radius {
            continue;
        }
        if g.band == 1 {
            if matches!(g.kind, Kind::Wanderer { .. }) && !g.is_moving(w.time) && !g.hostile {
                let r = w.people[g.members[0] as usize].seed;
                let at = g.pos.add(V2::new(3.5, 2.0));
                tent(&mut b, to3(at, on_ground(at)), k, r);
            }
            for &m in &g.members {
                heads.push(person(&mut b, w, m, k, &on_ground));
            }
        } else {
            let lead = w.people[g.members[0] as usize].race;
            let mk = (oc.dist / 120.0).max(2.0);
            let base = to3(g.pos, on_ground(g.pos));
            let n = g.members.len() as f32;
            b.column(base, 0.25 * mk, 0.25 * mk, 3.0 * mk, 4, palette::scale(race_color(lead), 0.6));
            b.column(base + vec3(0.0, 3.0 * mk, 0.0), (0.6 + 0.12 * n) * mk, 0.0, 1.3 * mk, 4, race_color(lead));
            markers.push((base + vec3(0.0, 3.6 * mk, 0.0), g.id));
        }
    }
    for &m in &w.squad.members {
        heads.push(person(&mut b, w, m, k, &on_ground));
    }

    // The fallen.
    for &(at, race, _, _) in &w.corpses {
        if at.dist(oc.target) < radius {
            let base = to3(at, on_ground(at) - 0.1);
            b.block(base, 1.6 * k, 0.6 * k, 0.35 * k, at.x * 0.37, Color::new(0.35, 0.12, 0.10, 1.0));
            b.block(base + vec3(0.0, 0.3 * k, 0.0), 1.2 * k, 0.4 * k, 0.15 * k, at.x * 0.37, palette::scale(race_color(race), 0.5));
        }
    }
    // Bandit camps: rough hide lean-tos round a fire.
    for c in &w.camps {
        if c.pos.dist(oc.target) > radius {
            continue;
        }
        let kk = k.min(4.0);
        let fire = to3(c.pos, on_ground(c.pos));
        b.column(fire - vec3(0.0, 0.1, 0.0), 0.9 * kk, 0.2 * kk, 0.7 * kk, 6, EMBER);
        for j in 0..3 {
            let a = j as f32 * 2.1 + c.group as f32;
            let at = c.pos.add(V2::new(a.cos(), a.sin()).scale(7.0 * kk));
            b.block(to3(at, on_ground(at) - 0.1), 3.2 * kk, 2.4 * kk, 1.6 * kk, a, CAMP_HIDE);
        }
    }
    // Things lying about.
    let mut things: Vec<(Vec3, u32)> = Vec::new();
    for g in &w.ground {
        if g.pos.dist(oc.target) < radius.min(600.0) {
            let base = to3(g.pos, on_ground(g.pos));
            let kk = k.min(6.0);
            b.block(base, 0.55 * kk, 0.35 * kk, 0.22 * kk, g.id as f32 * 1.7, ground_color(g.item));
            things.push((base + vec3(0.0, 0.3 * kk, 0.0), g.id));
        }
    }
    // Spell effects, briefly.
    for battle in &w.battles {
        for fx in &battle.fx {
            let age = (w.time - fx.at) as f32;
            if !(0.0..0.8).contains(&age) {
                continue;
            }
            match fx.kind {
                FxKind::Fireball { at, radius } => {
                    let r = radius * (0.4 + age * 1.2);
                    b.dome(to3(at, on_ground(at)), r, r, r * 0.8, 0.2, 0.0, 7, Color::new(1.0, 0.55 - age * 0.4, 0.15, 1.0));
                }
                FxKind::Bolt { from, to } => {
                    let (a, c) = (to3(from, on_ground(from) + 1.5), to3(to, on_ground(to) + 1.2));
                    let side = vec3(-(c.z - a.z), 0.0, c.x - a.x).normalize_or_zero() * 0.25;
                    let col = b.fogged(Color::new(0.85, 0.9, 1.0, 1.0), a);
                    b.quad_raw([a - side, a + side, c + side, c - side], [col; 4]);
                    b.quad_raw([a - vec3(0.0, 0.25, 0.0), a + vec3(0.0, 0.25, 0.0), c + vec3(0.0, 0.25, 0.0), c - vec3(0.0, 0.25, 0.0)], [col; 4]);
                }
                FxKind::Fizzle { at } => {
                    b.glow(to3(at, on_ground(at) + 2.2), 0.8, 0.8, 0.0, Color::new(0.6, 0.6, 0.7, 1.0));
                }
            }
        }
    }

    // Rings and the order line, draped over the land.
    let rw = (oc.dist / 260.0).max(0.12);
    let sq = w.squad.pos;
    for (i, &m) in w.squad.members.iter().enumerate() {
        let at = w.member_pos(i);
        let picked = sel.shows(w, m);
        if picked {
            draped_ring(&mut b, &on_ground, at, 1.2 * k, rw * 0.8, 18, PICKED);
        }
        // Where each member is headed.
        let goal = w.squad.goal[i];
        if w.fighter(m).is_none() && at.dist(goal) > 1.5 {
            let col = if picked { PICKED } else { Color::new(0.95, 0.95, 0.95, 1.0) };
            draped_ribbon(&mut b, &on_ground, at, goal, rw * 0.5, 0.4, col, 10.0);
            draped_ring(&mut b, &on_ground, goal, 0.8 * k, rw * 0.6, 12, col);
        }
    }
    if rings {
        draped_ring(&mut b, &on_ground, sq, BAND1_RADIUS, rw * 2.0, 160, Color::new(0.92, 0.94, 0.95, 1.0));
        draped_ring(&mut b, &on_ground, sq, BAND2_RADIUS, rw * 3.0, 320, Color::new(0.80, 0.84, 0.86, 1.0));
    }
    b.finish().draw();

    // Back to flat screen space for text and picking.
    set_default_camera();
    for (p, id) in heads {
        if let Some(s) = project(&cam, p) {
            pick.offer(s, 2.0, Hover::Person(id));
            if let Some((vit, mana, down)) = super::ui::bar_for(w, id) {
                if p.distance(cam.position) < 400.0 {
                    super::ui::draw_bar(s.x, s.y - 12.0, vit, mana, down);
                }
            }
        }
    }
    for (p, id) in doors {
        if let Some(s) = project(&cam, p) {
            pick.offer(s, 0.0, Hover::Door(id));
        }
    }
    for (p, id) in things {
        if let Some(s) = project(&cam, p) {
            pick.offer(s, 4.0, Hover::Item(id));
        }
    }
    for (p, id) in markers {
        if let Some(s) = project(&cam, p) {
            pick.offer(s, 2.0, Hover::Group(id));
        }
    }
    for (p, name, id) in labels {
        if let Some(q) = project(&cam, p) {
            if p.distance(cam.position) < radius * 1.1 {
                ui.centred(&name, q.x, q.y, 17, TEXT);
                pick.offer(q, 20.0, Hover::Town(id));
            }
        }
    }
}

/// A building seen from inside: floor, the stubs of its walls (with a gap at
/// the door), and what's in it.
fn interior(b: &mut Builder, bd: &Building, d: &Door, on_ground: &dyn Fn(V2) -> f32) {
    let floor_h = on_ground(bd.pos);
    let base = to3(bd.pos, floor_h);
    let (wall, floor) = match bd.kind {
        BuildingKind::RoduroHome | BuildingKind::QotiroHall => (STONE, Color::new(0.30, 0.28, 0.26, 1.0)),
        _ => (SANDSTONE, Color::new(0.62, 0.52, 0.38, 1.0)),
    };
    let r = d.radius;
    b.column(base - vec3(0.0, 0.25, 0.0), r, r, 0.3, 20, floor);
    // A rug in the middle, warm against the stone.
    b.column(base + vec3(0.0, 0.06, 0.0), r * 0.45, r * 0.45, 0.02, 14, Color::new(0.55, 0.22, 0.16, 1.0));
    // Wall stubs round the edge, leaving the doorway.
    let door_dir = d.outside.sub(d.centre);
    let door_a = door_dir.y.atan2(door_dir.x);
    let n = 22;
    for k in 0..n {
        let a = k as f32 / n as f32 * std::f32::consts::TAU;
        let gap = ((a - door_a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI).abs();
        if gap < 0.32 {
            continue;
        }
        let p = bd.pos.add(V2::new(a.cos(), a.sin()).scale(r - 0.3));
        let seg = r * std::f32::consts::TAU / n as f32 + 0.2;
        b.block(to3(p, floor_h - 0.2), 0.6, seg, 1.3, a, wall);
    }
    // Furniture, laid out from the building's seed: bed, table, chest, hearth.
    let mut r2 = rng::Rng::from_keys(&[bd.seed, 0x4655_524E]);
    let back = d.centre.sub(d.inside);
    let back_a = back.y.atan2(back.x);
    let spot = |a: f32, f: f32| bd.pos.add(V2::new((back_a + a).cos(), (back_a + a).sin()).scale(r * f));
    let bed = spot(1.4 + r2.f32() * 0.3, 0.55);
    b.block(to3(bed, floor_h), 2.0, 1.0, 0.5, back_a + 1.4, TIMBER);
    b.block(to3(bed, floor_h + 0.5), 1.8, 0.9, 0.12, back_a + 1.4, Color::new(0.72, 0.68, 0.58, 1.0));
    let table = spot(-1.2 - r2.f32() * 0.3, 0.45);
    b.block(to3(table, floor_h), 1.4, 0.9, 0.8, back_a, TIMBER);
    let chest = spot(0.25, 0.68);
    b.block(to3(chest, floor_h), 1.1, 0.6, 0.6, back_a, Color::new(0.45, 0.30, 0.16, 1.0));
    let fire = spot(-0.35, 0.62);
    b.column(to3(fire, floor_h), 0.5, 0.2, 0.4, 6, EMBER);
}

/// Campfires as lights, at night.
fn camp_lamps(w: &World, ground: &dyn Fn(V2) -> f32) -> Vec<(Vec3, f32, f32)> {
    if stealth::daylight(w.time) > 0.95 {
        return Vec::new();
    }
    w.camps.iter().map(|c| (to3(c.pos, ground(c.pos) + 1.5), 22.0, 0.9)).collect()
}

/// A flat strip laid over the land from `a` to `c`, cut into pieces no longer
/// than `piece` metres so it follows the ground instead of cutting through it.
#[allow(clippy::too_many_arguments)]
fn draped_ribbon(b: &mut Builder, ground: &dyn Fn(V2) -> f32, a: V2, c: V2, width: f32, lift: f32, col: Color, piece: f32) {
    let len = a.dist(c);
    if len < 1e-3 {
        return;
    }
    let d = c.sub(a).scale(1.0 / len);
    let away = to3(a, ground(a)).distance(b.eye());
    let side = V2::new(-d.y, d.x).scale(width.max(away * 0.005) * 0.5);
    let n = ((len / piece).ceil() as usize).max(1);
    for i in 0..n {
        let p0 = a.lerp(c, i as f32 / n as f32);
        let p1 = a.lerp(c, (i + 1) as f32 / n as f32);
        let q = [p0.sub(side), p0.add(side), p1.add(side), p1.sub(side)];
        let v = q.map(|p| to3(p, ground(p) + lift));
        let k = b.shade(col, Vec3::Y, v[0]);
        b.quad_raw(v, [k; 4]);
    }
}

fn draped_ring(b: &mut Builder, ground: &dyn Fn(V2) -> f32, c: V2, r: f32, width: f32, n: usize, col: Color) {
    // Enough pieces that each is about one ground cell, so the ring hugs the land.
    let n = n.max((r * std::f32::consts::TAU / 18.0) as usize).min(2400);
    let lift = 0.5 + r * 0.0008;
    for i in 0..n {
        let a0 = i as f32 / n as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / n as f32 * std::f32::consts::TAU;
        let (u0, u1) = (V2::new(a0.cos(), a0.sin()), V2::new(a1.cos(), a1.sin()));
        // Far pieces are widened so they stay at least a pixel or two thick
        // when seen edge-on; otherwise the ring breaks into dashes.
        let mid = c.add(u0.scale(r));
        let away = to3(mid, ground(mid)).distance(b.eye());
        let width = width.max(away * 0.011);
        let q = [c.add(u0.scale(r - width * 0.5)), c.add(u0.scale(r + width * 0.5)), c.add(u1.scale(r + width * 0.5)), c.add(u1.scale(r - width * 0.5))];
        let v = q.map(|p| to3(p, ground(p) + lift));
        let k = b.fogged(col, v[1]);
        b.quad_raw(v, [k; 4]);
    }
}

/// A square patch of land and sea centred on `centre`, `half` metres each
/// way, in cells `step` wide. With `hole`, cells inside that half-width are
/// skipped (the fine patch covers them).
fn ground_patch(b: &mut Builder, t: &Terrain, centre: V2, half: f32, step: f32, hole: Option<f32>) {
    let cells = (half * 2.0 / step).round() as i64;
    let (x0, y0) = (centre.x - half, centre.y - half);
    let vert = |b: &Builder, p: V2| -> (Vec3, Color) {
        if geo::is_land(p) || geo::inland(p) >= 0.0 {
            let h = t.height(p).max(0.3);
            let (nx, ny, nz) = t.normal(p, step.max(15.0));
            let n = vec3(nx, ny, nz);
            let v = to3(p, h);
            (v, b.shade(palette::ground(t, p, h, ny), n, v))
        } else {
            let v = to3(p, 0.0);
            (v, b.shade(palette::sea(p), Vec3::Y, v))
        }
    };
    let skip = |xa: f32, xb: f32, ya: f32, yb: f32| -> bool {
        match hole {
            Some(hh) => xa >= centre.x - hh - 0.5 && xb <= centre.x + hh + 0.5 && ya >= centre.y - hh - 0.5 && yb <= centre.y + hh + 0.5,
            None => false,
        }
    };
    for j in 0..cells {
        let (ya, yb) = (y0 + j as f32 * step, y0 + (j + 1) as f32 * step);
        let (ca, cb) = (geo::coast_x(ya), geo::coast_x(yb));
        for i in 0..cells {
            let (xa, xb) = (x0 + i as f32 * step, x0 + (i + 1) as f32 * step);
            if skip(xa, xb, ya, yb) {
                continue;
            }
            if xa >= ca.max(cb) || xb <= ca.min(cb) {
                // Wholly land or wholly sea.
                let ps = [V2::new(xa, ya), V2::new(xb, ya), V2::new(xb, yb), V2::new(xa, yb)];
                let vs = ps.map(|p| vert(b, p));
                b.quad_raw(vs.map(|v| v.0), vs.map(|v| v.1));
            } else {
                // The shore runs through: cut the cell along the coastline
                // (straight across this row) into its sea part and land part.
                let rect = [V2::new(xa, ya), V2::new(xb, ya), V2::new(xb, yb), V2::new(xa, yb)];
                let side = |p: V2| p.x - (ca + (cb - ca) * (p.y - ya) / (yb - ya));
                let land = clip(&rect, |p| side(p));
                let sea = clip(&rect, |p| -side(p));
                let vs: Vec<(Vec3, Color)> = sea.iter().map(|&p| (to3(p, 0.0), b.shade(palette::sea(p), Vec3::Y, to3(p, 0.0)))).collect();
                b.poly_raw(&vs.iter().map(|v| v.0).collect::<Vec<_>>(), &vs.iter().map(|v| v.1).collect::<Vec<_>>());
                let vs: Vec<(Vec3, Color)> = land.iter().map(|&p| vert(b, V2::new(p.x + 0.01, p.y))).collect();
                b.poly_raw(&vs.iter().map(|v| v.0).collect::<Vec<_>>(), &vs.iter().map(|v| v.1).collect::<Vec<_>>());
            }
        }
    }
}

/// The part of a convex polygon where `f` is not negative (one pass of
/// Sutherland–Hodgman clipping against a straight line).
fn clip(poly: &[V2], f: impl Fn(V2) -> f32) -> Vec<V2> {
    let mut out = Vec::with_capacity(poly.len() + 1);
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let (fa, fb) = (f(a), f(b));
        if fa >= 0.0 {
            out.push(a);
        }
        if (fa >= 0.0) != (fb >= 0.0) {
            out.push(a.lerp(b, fa / (fa - fb)));
        }
    }
    out
}

fn building(b: &mut Builder, t: &Terrain, bd: &Building, on_ground: &dyn Fn(V2) -> f32) {
    let ground = if bd.kind == BuildingKind::HoraroStilt { 0.0 } else { on_ground(bd.pos) };
    // Sink buildings a little so they sit into slopes rather than float.
    let sink = (t.slope(bd.pos) * bd.size * 0.6).min(4.0);
    let base = to3(bd.pos, ground - sink);
    let r = bd.size * 0.5;
    let s = bd.seed;
    let unit = |k: u64| (rng::key(&[s, k]) >> 40) as f32 / (1u64 << 24) as f32;
    match bd.kind {
        BuildingKind::RoduroHome => {
            // Grown stone: a lumpy banded mound, sometimes with an upper
            // storey budding out of it, and a warm window facing the hearth.
            let h = bd.size * (0.42 + unit(1) * 0.18) + sink;
            b.dome(base, r * (0.95 + unit(2) * 0.2), r * (0.85 + unit(3) * 0.2), h, 0.10, 0.10, s, STONE);
            if bd.size > 9.5 {
                let off = vec3(unit(4) - 0.5, 0.0, unit(5) - 0.5) * r * 0.6;
                b.dome(base + off + vec3(0.0, h * 0.62, 0.0), r * 0.55, r * 0.5, h * 0.55, 0.12, 0.12, s ^ 7, STONE);
            }
            let (sn, cs) = bd.rot.sin_cos();
            let face = vec3(base.x, ground, base.z) + vec3(cs, 0.0, sn) * (r * 0.97);
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
            for tier in 0..tiers {
                let h = 4.0 + unit(10 + tier as u64) * 1.5 + if tier == 0 { sink } else { 0.0 };
                b.block(base + vec3(0.0, y, 0.0), sz, sz * 0.8, h, bd.rot, SANDSTONE);
                y += h;
                sz *= 0.72;
            }
        }
        BuildingKind::QotiroTemple => {
            // Three tiers: the wide market base, the living middle, and an
            // open summit with a temple and its gold sun disc.
            let mut y = 0.0;
            for (f, h) in [(1.0, 7.0 + sink), (0.68, 8.0), (0.40, 3.0)] {
                b.block(base + vec3(0.0, y, 0.0), bd.size * f, bd.size * f, h, bd.rot, SANDSTONE);
                y += h;
            }
            b.block(base + vec3(0.0, y, 0.0), bd.size * 0.16, bd.size * 0.2, 6.0, bd.rot, SANDSTONE);
            b.column(base + vec3(0.0, y + 6.0, 0.0), bd.size * 0.06, 0.0, 3.0, 8, GOLD);
            b.glow(base + vec3(0.0, y + 9.5, 0.0), 2.6, 2.6, bd.rot, GOLD);
        }
        BuildingKind::QotiroHall => {
            // Away from home: two tiers in local dark stone, the gold crown kept.
            b.block(base, bd.size, bd.size * 0.8, 4.5 + sink, bd.rot, STONE);
            b.block(base + vec3(0.0, 4.5 + sink, 0.0), bd.size * 0.5, bd.size * 0.45, 3.0, bd.rot, STONE);
            b.column(base + vec3(0.0, 7.5 + sink, 0.0), 0.9, 0.0, 2.2, 8, GOLD);
        }
        BuildingKind::Hearth => {
            let base = to3(bd.pos, ground);
            for i in 0..9 {
                let a = i as f32 / 9.0 * std::f32::consts::TAU;
                b.block(base + vec3(a.cos(), -0.2, a.sin()) * 2.4, 0.9, 0.7, 0.75, a, STONE);
            }
            b.column(base + vec3(0.0, -0.2, 0.0), 1.4, 0.3, 1.4, 8, EMBER);
        }
    }
}

/// Draw one person; returns where their head is, for hover.
fn person(b: &mut Builder, w: &World, pid: PersonId, k: f32, on_ground: &dyn Fn(V2) -> f32) -> (Vec3, PersonId) {
    let p = &w.people[pid as usize];
    let at = w.person_pos(pid);
    // Horaro at home on a stilt deck stand above the water.
    let floor = if geo::inland(at) < 0.0 { DECK + 0.3 } else { on_ground(at) };
    let (h, r, skin) = match p.race {
        Race::Roduro => (1.35, 0.34, Color::new(0.58, 0.47, 0.39, 1.0)), // short and stocky
        Race::Qotiro => (1.80, 0.34, Color::new(0.56, 0.57, 0.59, 1.0)), // grey-skinned, broad
        Race::Horaro => (1.65, 0.27, Color::new(0.36, 0.52, 0.78, 1.0)), // blue-skinned, hairless
        Race::Tadoro => (1.95, 0.22, Color::new(0.80, 0.75, 0.72, 1.0)), // tall and lean
    };
    let (h, r) = (h * k, r * k);
    let base = to3(at, floor - 0.1);
    // Bandits and anyone you're fighting get a red mark at their feet.
    let foe = p.bandit || w.fighter(pid).map(|f| f.side != SQUAD_SIDE).unwrap_or(false);
    if foe {
        draped_ring(b, on_ground, at, 0.9 * k, 0.18 * k, 14, Color::new(0.9, 0.2, 0.15, 1.0));
    }
    let down = w.fighter(pid).map(|f| f.ko || f.dead).unwrap_or(false) || p.dead;
    if down {
        // Lying where they fell.
        let rot = (p.seed % 628) as f32 / 100.0;
        b.block(base, h, r * 2.0, r * 1.2, rot, palette::scale(race_color(p.race), 0.7));
        return (base + vec3(0.0, r * 1.5, 0.0), pid);
    }
    b.column(base, r, r * 0.8, h * 0.8, 6, race_color(p.race));
    let head = base + vec3(0.0, h * 0.8, 0.0);
    b.block(head, r * 1.1, r * 1.1, h * 0.2, 0.0, skin);
    // A paralyzed or blinded fighter shows it.
    if let Some(f) = w.fighter(pid) {
        if f.paralyzed() {
            b.glow(head + vec3(0.0, h * 0.35, 0.0), r * 2.5, r * 0.6, 0.0, Color::new(0.7, 0.4, 1.0, 1.0));
        }
        if f.has(StatusKind::MageArmor).is_some() {
            draped_ring(b, on_ground, at, 1.3 * k, 0.12 * k, 16, Color::new(0.5, 0.75, 1.0, 1.0));
        }
    }
    (head + vec3(0.0, h * 0.2, 0.0), pid)
}

/// A Ṭaḍoro travelling tent: a taut cone of pale cloth around one pole.
fn tent(b: &mut Builder, base: Vec3, k: f32, seed: u64) {
    let r = 2.2 * k.min(4.0);
    b.column(base - vec3(0.0, 0.2, 0.0), r, 0.05, r * 1.5, 10, TENT);
    let tilt = (seed % 7) as f32 * 0.1;
    b.column(base + vec3(0.0, r * 1.5 - 0.2, 0.0), 0.06 * k, 0.04 * k, 0.6 * k + tilt, 4, TIMBER);
}
