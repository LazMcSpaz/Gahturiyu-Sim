//! The top-down map.

use macroquad::prelude::*;

use gahturiyu_sim::sim::{
    bands::{BAND1_RADIUS, BAND2_RADIUS},
    geo::{self, V2, WORLD_SIZE},
    race::Race,
    World,
};

use super::ui::{race_color, with_alpha, Hover, Picker, Ui, TEXT};

const LAND: Color = Color::new(0.16, 0.21, 0.17, 1.0);
const SEA: Color = Color::new(0.10, 0.15, 0.19, 1.0);
const SHORE: Color = Color::new(0.30, 0.36, 0.33, 1.0);

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

pub fn draw(ui: &Ui, cam: &MapCam, w: &World, rings: bool, pick: &mut Picker) {
    clear_background(LAND);
    draw_sea(cam);
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

    let sq = cam.to_screen(w.squad.pos);
    let tg = cam.to_screen(w.squad.target);
    if w.squad.pos.dist(w.squad.target) > 1.0 {
        draw_line(sq.x, sq.y, tg.x, tg.y, 1.0, with_alpha(WHITE, 0.35));
        draw_line(tg.x - 5.0, tg.y - 5.0, tg.x + 5.0, tg.y + 5.0, 2.0, WHITE);
        draw_line(tg.x - 5.0, tg.y + 5.0, tg.x + 5.0, tg.y - 5.0, 2.0, WHITE);
    }
    draw_circle_lines(sq.x, sq.y, (14.0 * cam.zoom).max(9.0), 2.0, WHITE);
    for &m in &w.squad.members {
        let p = cam.to_screen(w.person_pos(m));
        draw_circle(p.x, p.y, dot + 0.5, race_color(w.people[m as usize].race));
        pick.offer(p, 0.0, Hover::Person(m));
    }
}

fn draw_sea(cam: &MapCam) {
    let top = cam.to_world(vec2(0.0, 0.0)).y.max(0.0);
    let bottom = cam.to_world(vec2(0.0, screen_height())).y.min(WORLD_SIZE);
    let step = (3.0 / cam.zoom).max(10.0);
    let mut y = top - step;
    while y < bottom + step {
        let a = cam.to_screen(V2::new(geo::coast_x(y), y));
        let b = cam.to_screen(V2::new(geo::coast_x(y + step), y + step));
        draw_rectangle(-10.0, a.y, a.x + 10.0, b.y - a.y + 1.0, SEA);
        draw_line(a.x, a.y, b.x, b.y, 2.0, SHORE);
        y += step;
    }
    let tl = cam.to_screen(V2::new(0.0, 0.0));
    let br = cam.to_screen(V2::new(WORLD_SIZE, WORLD_SIZE));
    let edge = Color::new(0.03, 0.04, 0.04, 1.0);
    draw_rectangle(-10.0, -10.0, screen_width() + 20.0, tl.y + 10.0, edge);
    draw_rectangle(-10.0, br.y, screen_width() + 20.0, screen_height(), edge);
    draw_rectangle(br.x, -10.0, screen_width(), screen_height() + 20.0, edge);
}
