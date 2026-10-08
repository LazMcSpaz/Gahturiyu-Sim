//! The top-down map.

use macroquad::prelude::*;

use gahturiyu_sim::sim::{
    bands::{BAND1_RADIUS, BAND2_RADIUS},
    geo::{self, V2, WORLD_SIZE},
    race::Race,
    World,
};

use super::palette;
use super::squadui::{ground_color, Selection, GOLD};
use super::ui::{race_color, with_alpha, Hover, Picker, Ui, TEXT};

const SHORE: Color = Color::new(0.30, 0.36, 0.33, 1.0);
/// Pixels along one side of the relief map texture.
const RELIEF: usize = 1024;

/// A shaded-relief picture of the whole world, made once at start-up.
pub struct Relief {
    tex: Texture2D,
}

impl Relief {
    pub fn new(w: &World) -> Relief {
        let t = &w.terrain;
        let px = WORLD_SIZE / RELIEF as f32;
        let mut bytes = vec![0u8; RELIEF * RELIEF * 4];
        let sun = vec3(-0.45, 0.80, -0.35).normalize();
        for j in 0..RELIEF {
            for i in 0..RELIEF {
                let p = V2::new((i as f32 + 0.5) * px, (j as f32 + 0.5) * px);
                let c = if geo::is_land(p) {
                    let h = t.height(p);
                    let (nx, ny, nz) = t.normal(p, px);
                    // Exaggerate relief a little so hills read from above.
                    let n = vec3(nx * 1.6, ny, nz * 1.6).normalize();
                    let lit = 0.45 + 0.75 * n.dot(sun).max(0.0);
                    palette::scale(palette::ground(t, p, h, ny), lit)
                } else {
                    palette::sea(p)
                };
                let k = (j * RELIEF + i) * 4;
                bytes[k] = (c.r.clamp(0.0, 1.0) * 255.0) as u8;
                bytes[k + 1] = (c.g.clamp(0.0, 1.0) * 255.0) as u8;
                bytes[k + 2] = (c.b.clamp(0.0, 1.0) * 255.0) as u8;
                bytes[k + 3] = 255;
            }
        }
        let tex = Texture2D::from_rgba8(RELIEF as u16, RELIEF as u16, &bytes);
        tex.set_filter(FilterMode::Linear);
        Relief { tex }
    }
}

pub struct MapCam {
    pub centre: V2,
    /// Pixels per metre.
    pub zoom: f32,
}

impl MapCam {
    pub fn to_screen(&self, p: V2) -> Vec2 {
        vec2((p.x - self.centre.x) * self.zoom + screen_width() / 2.0, (p.y - self.centre.y) * self.zoom + screen_height() / 2.0)
    }
    pub fn to_world(&self, s: Vec2) -> V2 {
        V2::new((s.x - screen_width() / 2.0) / self.zoom + self.centre.x, (s.y - screen_height() / 2.0) / self.zoom + self.centre.y)
    }
    pub fn zoom_at(&mut self, mouse: Vec2, wheel: f32) {
        let before = self.to_world(mouse);
        self.zoom = (self.zoom * if wheel > 0.0 { 1.15 } else { 1.0 / 1.15 }).clamp(screen_width() / WORLD_SIZE / 1.2, 12.0);
        let after = self.to_world(mouse);
        self.centre = self.centre.add(before.sub(after));
    }
}

pub fn draw(ui: &Ui, cam: &MapCam, w: &World, rings: bool, pick: &mut Picker, relief: &Relief, sel: &Selection) {
    clear_background(Color::new(0.03, 0.04, 0.04, 1.0));
    let tl = cam.to_screen(V2::new(0.0, 0.0));
    let size = WORLD_SIZE * cam.zoom;
    draw_texture_ex(&relief.tex, tl.x, tl.y, WHITE, DrawTextureParams { dest_size: Some(vec2(size, size)), ..Default::default() });
    draw_shore(cam);
    let road_w = (cam.zoom * 6.0).clamp(1.5, 4.0);
    for road in &w.routes.roads {
        for seg in road.windows(2) {
            let (a, b) = (cam.to_screen(seg[0]), cam.to_screen(seg[1]));
            draw_line(a.x, a.y, b.x, b.y, road_w, palette::ROAD);
        }
    }
    if rings {
        let c = cam.to_screen(w.squad.pos);
        draw_circle(c.x, c.y, BAND1_RADIUS * cam.zoom, Color::new(1.0, 1.0, 1.0, 0.045));
        draw_circle_lines(c.x, c.y, BAND1_RADIUS * cam.zoom, 1.5, Color::new(1.0, 1.0, 1.0, 0.45));
        draw_circle_lines(c.x, c.y, BAND2_RADIUS * cam.zoom, 1.0, Color::new(1.0, 1.0, 1.0, 0.25));
    }
    let dot = (cam.zoom * 1.2).clamp(2.0, 5.0);

    for s in &w.settlements {
        let p = cam.to_screen(s.pos);
        let r = (s.radius() * cam.zoom).max(4.0);
        draw_circle(p.x, p.y, r, with_alpha(race_color(s.founders), 0.22));
        draw_circle_lines(p.x, p.y, r, 1.5, with_alpha(race_color(s.founders), 0.8));
        if let Some(st) = s.stilts {
            let q = cam.to_screen(st);
            draw_circle(q.x, q.y, (60.0 * cam.zoom).max(2.5), with_alpha(race_color(Race::Horaro), 0.25));
        }
        pick.offer(p, r.min(30.0) - 6.0, Hover::Town(s.id));
        if cam.zoom > 0.06 || w.bands.band_at(s.pos) <= 2 {
            ui.centred(&s.name, p.x, p.y - r - 6.0, 15, TEXT);
        }
        // Townsfolk exist as individuals only where they stand inside band 1.
        for pid in w.residents_in_band1(s.id) {
            let q = cam.to_screen(w.person_pos(pid));
            draw_circle(q.x, q.y, dot, race_color(w.people[pid as usize].race));
            pick.offer(q, 0.0, Hover::Person(pid));
        }
    }

    for g in &w.groups {
        let lead = w.people[g.members[0] as usize].race;
        if g.band == 1 {
            for &m in &g.members {
                let p = cam.to_screen(w.person_pos(m));
                draw_circle(p.x, p.y, dot, race_color(w.people[m as usize].race));
                draw_circle_lines(p.x, p.y, dot + 1.0, 1.0, with_alpha(WHITE, 0.5));
                pick.offer(p, 0.0, Hover::Person(m));
            }
        } else {
            let p = cam.to_screen(g.pos);
            let (r, a) = if g.band == 2 { (3.6, 0.95) } else { (2.4, 0.55) };
            draw_circle(p.x, p.y, r + (g.members.len() as f32 - 1.0) * 0.35, with_alpha(race_color(lead), a));
            pick.offer(p, 0.0, Hover::Group(g.id));
        }
    }

    // Bandit camps.
    for c in &w.camps {
        let q = cam.to_screen(c.pos);
        let k = (cam.zoom * 8.0).clamp(4.0, 9.0);
        draw_triangle(vec2(q.x, q.y - k), vec2(q.x - k, q.y + k * 0.7), vec2(q.x + k, q.y + k * 0.7), Color::new(0.85, 0.22, 0.18, 0.9));
        if cam.zoom > 0.15 {
            ui.centred("bandits", q.x, q.y + k + 13.0, 13, Color::new(0.95, 0.5, 0.45, 1.0));
        }
    }
    for &(at, _, _, _) in &w.corpses {
        let q = cam.to_screen(at);
        let k = dot + 1.0;
        draw_line(q.x - k, q.y - k, q.x + k, q.y + k, 2.0, Color::new(0.8, 0.2, 0.15, 1.0));
        draw_line(q.x - k, q.y + k, q.x + k, q.y - k, 2.0, Color::new(0.8, 0.2, 0.15, 1.0));
    }
    for battle in &w.battles {
        for f in &battle.fighters {
            if f.dead || f.fled {
                continue;
            }
            let q = cam.to_screen(f.pos);
            if f.side != gahturiyu_sim::sim::combat::SQUAD_SIDE {
                draw_circle_lines(q.x, q.y, dot + 2.5, 1.5, Color::new(0.95, 0.25, 0.2, 1.0));
            }
            if let Some((vit, mana, down)) = super::ui::bar_for(w, f.pid) {
                if cam.zoom > 0.6 {
                    super::ui::draw_bar(q.x, q.y - dot - 10.0, vit, mana, down);
                }
            }
        }
        for fx in &battle.fx {
            let age = (w.time - fx.at) as f32;
            if !(0.0..0.8).contains(&age) {
                continue;
            }
            match fx.kind {
                gahturiyu_sim::sim::combat::FxKind::Fireball { at, radius } => {
                    let q = cam.to_screen(at);
                    draw_circle(q.x, q.y, radius * cam.zoom * (0.5 + age), Color::new(1.0, 0.5, 0.1, 0.6 - age * 0.6));
                }
                gahturiyu_sim::sim::combat::FxKind::Bolt { from, to } => {
                    let (a, b) = (cam.to_screen(from), cam.to_screen(to));
                    draw_line(a.x, a.y, b.x, b.y, 3.0, Color::new(0.85, 0.9, 1.0, 1.0 - age));
                }
                gahturiyu_sim::sim::combat::FxKind::Arrow { from, to, .. } => {
                    let fly = from.dist(to) / 45.0;
                    if age < fly {
                        let u = age / fly;
                        let (a, b) = (cam.to_screen(from.lerp(to, u)), cam.to_screen(from.lerp(to, (u + 0.08).min(1.0))));
                        draw_line(a.x, a.y, b.x, b.y, 2.0, Color::new(0.95, 0.9, 0.75, 1.0));
                    }
                }
                _ => {}
            }
        }
    }

    if cam.zoom > 0.6 {
        for g in &w.ground {
            let q = cam.to_screen(g.pos);
            let k = (cam.zoom * 0.25).clamp(2.0, 4.0);
            draw_rectangle(q.x - k, q.y - k, k * 2.0, k * 2.0, ground_color(g.item));
            pick.offer(q, 0.0, Hover::Item(g.id));
        }
    }

    let sq = cam.to_screen(w.squad.pos);
    draw_circle_lines(sq.x, sq.y, (14.0 * cam.zoom).max(9.0), 2.0, with_alpha(WHITE, 0.5));
    for (i, &m) in w.squad.members.iter().enumerate() {
        let at = w.member_pos(i);
        let p = cam.to_screen(at);
        let picked = sel.shows(w, m);
        let goal = w.squad.goal[i];
        if w.fighter(m).is_none() && at.dist(goal) > 1.5 {
            let g = cam.to_screen(goal);
            let col = if picked { GOLD } else { WHITE };
            draw_line(p.x, p.y, g.x, g.y, 1.0, with_alpha(col, 0.45));
            draw_line(g.x - 4.0, g.y - 4.0, g.x + 4.0, g.y + 4.0, 1.5, col);
            draw_line(g.x - 4.0, g.y + 4.0, g.x + 4.0, g.y - 4.0, 1.5, col);
        }
        draw_circle(p.x, p.y, dot + 0.5, race_color(w.people[m as usize].race));
        if picked {
            draw_circle_lines(p.x, p.y, dot + 3.0, 1.5, GOLD);
        }
        pick.offer(p, 0.0, Hover::Person(m));
    }
}

/// A crisp shoreline over the relief picture, which is blurry up close.
fn draw_shore(cam: &MapCam) {
    let top = cam.to_world(vec2(0.0, 0.0)).y.max(0.0);
    let bottom = cam.to_world(vec2(0.0, screen_height())).y.min(WORLD_SIZE);
    let step = (3.0 / cam.zoom).max(10.0);
    let mut y = top - step;
    while y < bottom + step {
        let a = cam.to_screen(V2::new(geo::coast_x(y), y));
        let b = cam.to_screen(V2::new(geo::coast_x(y + step), y + step));
        draw_line(a.x, a.y, b.x, b.y, 1.5, SHORE);
        y += step;
    }
}
