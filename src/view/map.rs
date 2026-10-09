//! The top-down map, drawn flat over the window.

use bevy::math::{vec2, Vec2, Vec3};
use bevy_egui::egui::{self, Color32, ColorImage, Pos2, TextureHandle, TextureOptions};

use gahturiyu_sim::sim::{
    bands::{BAND1_RADIUS, BAND2_RADIUS},
    combat::{FxKind, SQUAD_SIDE},
    geo::{self, V2, WORLD_SIZE},
    race::Race,
    World,
};

use super::app::Hover;
use super::cam::MapCam;
use super::hud::Canvas;
use super::palette::{self, eg, ega, race_color, GOLD, TEXT, WHITE};
use super::squadui::{ground_color, Selection};

const SHORE: [f32; 3] = [0.30, 0.36, 0.33];
/// Pixels along one side of the relief map picture.
const RELIEF: usize = 1024;

/// A shaded-relief picture of the whole world, made once at start-up.
pub fn relief_image(w: &World) -> ColorImage {
    let t = &w.terrain;
    let px = WORLD_SIZE / RELIEF as f32;
    let sun = Vec3::new(-0.45, 0.80, -0.35).normalize();
    let mut pixels = Vec::with_capacity(RELIEF * RELIEF);
    for j in 0..RELIEF {
        for i in 0..RELIEF {
            let p = V2::new((i as f32 + 0.5) * px, (j as f32 + 0.5) * px);
            let c = if geo::is_land(p) {
                let h = t.height(p);
                let (nx, ny, nz) = t.normal(p, px);
                let n = Vec3::new(nx * 1.6, ny, nz * 1.6).normalize();
                let lit = 0.45 + 0.75 * n.dot(sun).max(0.0);
                palette::scale(palette::ground(t, p, h, ny), lit)
            } else {
                palette::sea(p)
            };
            pixels.push(eg(c));
        }
    }
    ColorImage::new([RELIEF, RELIEF], pixels)
}

pub fn load_relief(ctx: &egui::Context, w: &World) -> TextureHandle {
    ctx.load_texture("relief", relief_image(w), TextureOptions::LINEAR)
}

/// Draw the map; returns things that can be hovered (screen point, slack).
pub fn draw(c: &Canvas, cam: &MapCam, w: &World, rings: bool, relief: &TextureHandle, sel: &Selection) -> Vec<(Vec2, f32, Hover)> {
    let size = vec2(c.w, c.h);
    let mut picks = Vec::new();
    let s = |p: V2| cam.to_screen(size, p);
    c.rect(0.0, 0.0, c.w, c.h, eg([0.03, 0.04, 0.04]));
    let tl = s(V2::new(0.0, 0.0));
    let span = WORLD_SIZE * cam.zoom;
    c.p.image(relief.id(), egui::Rect::from_min_size(Pos2::new(tl.x, tl.y), egui::vec2(span, span)), egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
    // A crisp shoreline over the relief picture, which is blurry up close.
    let top = cam.to_world(size, vec2(0.0, 0.0)).y.max(0.0);
    let bottom = cam.to_world(size, vec2(0.0, c.h)).y.min(WORLD_SIZE);
    let step = (3.0 / cam.zoom).max(10.0);
    let mut y = top - step;
    while y < bottom + step {
        c.line(s(V2::new(geo::coast_x(y), y)), s(V2::new(geo::coast_x(y + step), y + step)), 1.5, eg(SHORE));
        y += step;
    }
    let road_w = (cam.zoom * 6.0).clamp(1.5, 4.0);
    for road in &w.routes.roads {
        for seg in road.windows(2) {
            c.line(s(seg[0]), s(seg[1]), road_w, eg(palette::ROAD));
        }
    }
    if rings {
        let q = s(w.squad.pos);
        c.circle(q.x, q.y, BAND1_RADIUS * cam.zoom, ega(WHITE, 0.045));
        c.circle_lines(q.x, q.y, BAND1_RADIUS * cam.zoom, 1.5, ega(WHITE, 0.45));
        c.circle_lines(q.x, q.y, BAND2_RADIUS * cam.zoom, 1.0, ega(WHITE, 0.25));
    }
    let dot = (cam.zoom * 1.2).clamp(2.0, 5.0);

    for st in &w.settlements {
        let p = s(st.pos);
        let r = (st.radius() * cam.zoom).max(4.0);
        c.circle(p.x, p.y, r, ega(race_color(st.founders), 0.22));
        c.circle_lines(p.x, p.y, r, 1.5, ega(race_color(st.founders), 0.8));
        if let Some(sp) = st.stilts {
            let q = s(sp);
            c.circle(q.x, q.y, (60.0 * cam.zoom).max(2.5), ega(race_color(Race::Horaro), 0.25));
        }
        picks.push((p, r.min(30.0) - 6.0, Hover::Town(st.id)));
        if cam.zoom > 0.06 || w.bands.band_at(st.pos) <= 2 {
            c.centred(&st.name, p.x, p.y - r - 6.0, 15.0, TEXT);
        }
        // Workplaces, close up.
        if cam.zoom > 0.35 {
            if let Some(tl) = w.society.towns.get(st.id as usize) {
                for wp in &tl.places {
                    let q = s(wp.pos);
                    let half = (wp.kind.size() * 0.5 * cam.zoom).max(2.0);
                    let col = if wp.kind == gahturiyu_sim::sim::jobs::PlaceKind::Fields { [0.45, 0.55, 0.25] } else { [0.62, 0.58, 0.50] };
                    c.rect(q.x - half, q.y - half, half * 2.0, half * 2.0, ega(col, 0.45));
                }
            }
        }
        for pid in w.residents_in_band1(st.id) {
            if w.is_indoors_asleep(pid) {
                continue;
            }
            let q = s(w.person_pos(pid));
            c.circle(q.x, q.y, dot, eg(race_color(w.people[pid as usize].race)));
            picks.push((q, 0.0, Hover::Person(pid)));
        }
    }

    for g in &w.groups {
        let lead = w.people[g.members[0] as usize].race;
        if g.band == 1 {
            for &m in &g.members {
                let p = s(w.person_pos(m));
                c.circle(p.x, p.y, dot, eg(race_color(w.people[m as usize].race)));
                c.circle_lines(p.x, p.y, dot + 1.0, 1.0, ega(WHITE, 0.5));
                picks.push((p, 0.0, Hover::Person(m)));
            }
        } else {
            let p = s(g.pos);
            let (r, a) = if g.band == 2 { (3.6, 0.95) } else { (2.4, 0.55) };
            c.circle(p.x, p.y, r + (g.members.len() as f32 - 1.0) * 0.35, ega(race_color(lead), a));
            picks.push((p, 0.0, Hover::Group(g.id)));
        }
    }

    super::animals::draw_map(c, cam, w);
    for cp in &w.camps {
        let q = s(cp.pos);
        let k = (cam.zoom * 8.0).clamp(4.0, 9.0);
        c.triangle(vec2(q.x, q.y - k), vec2(q.x - k, q.y + k * 0.7), vec2(q.x + k, q.y + k * 0.7), ega([0.85, 0.22, 0.18], 0.9));
        if cam.zoom > 0.15 {
            c.centred("bandits", q.x, q.y + k + 13.0, 13.0, [0.95, 0.5, 0.45]);
        }
    }
    for st in &w.standing {
        if st.burning(w.time) {
            let q = s(st.pos);
            c.circle(q.x, q.y, 3.0, eg(palette::EMBER));
        }
    }
    for &(at, _, _, _) in &w.corpses {
        let q = s(at);
        let k = dot + 1.0;
        let red = eg([0.8, 0.2, 0.15]);
        c.line(vec2(q.x - k, q.y - k), vec2(q.x + k, q.y + k), 2.0, red);
        c.line(vec2(q.x - k, q.y + k), vec2(q.x + k, q.y - k), 2.0, red);
    }
    for battle in &w.battles {
        for f in &battle.fighters {
            if f.dead || f.fled {
                continue;
            }
            let q = s(f.pos);
            if f.side != SQUAD_SIDE {
                c.circle_lines(q.x, q.y, dot + 2.5, 1.5, eg([0.95, 0.25, 0.2]));
            }
            if let Some((vit, mana, down)) = super::hud::bar_for(w, f.pid) {
                if cam.zoom > 0.6 {
                    super::hud::draw_bar(c, q.x, q.y - dot - 10.0, vit, mana, down);
                }
            }
        }
        for fx in &battle.fx {
            let age = (w.time - fx.at) as f32;
            match fx.kind {
                FxKind::Fireball { at, radius } if (0.0..0.8).contains(&age) => {
                    let q = s(at);
                    c.circle(q.x, q.y, radius * cam.zoom * (0.5 + age), ega([1.0, 0.5, 0.1], 0.6 - age * 0.6));
                }
                FxKind::Bolt { from, to } if (0.0..0.8).contains(&age) => {
                    c.line(s(from), s(to), 3.0, ega([0.85, 0.9, 1.0], 1.0 - age));
                }
                FxKind::Arrow { from, to, .. } => {
                    let fly = from.dist(to) / super::scene::ARROW_SPEED;
                    if (0.0..fly).contains(&age) {
                        let u = age / fly;
                        c.line(s(from.lerp(to, u)), s(from.lerp(to, (u + 0.08).min(1.0))), 2.0, eg([0.95, 0.9, 0.75]));
                    }
                }
                _ => {}
            }
        }
    }

    if cam.zoom > 0.6 {
        for g in &w.ground {
            let q = s(g.pos);
            let k = (cam.zoom * 0.25).clamp(2.0, 4.0);
            c.rect(q.x - k, q.y - k, k * 2.0, k * 2.0, eg(ground_color(g.item)));
            picks.push((q, 0.0, Hover::Item(g.id)));
        }
    }

    let sq = s(w.squad.pos);
    c.circle_lines(sq.x, sq.y, (14.0 * cam.zoom).max(9.0), 2.0, ega(WHITE, 0.5));
    for (i, &m) in w.squad.members.iter().enumerate() {
        let at = w.member_pos(i);
        let p = s(at);
        let picked = sel.shows(w, m);
        let goal = w.squad.goal[i];
        if w.fighter(m).is_none() && at.dist(goal) > 1.5 {
            let g = s(goal);
            let col = if picked { GOLD } else { WHITE };
            c.line(p, g, 1.0, ega(col, 0.45));
            c.line(vec2(g.x - 4.0, g.y - 4.0), vec2(g.x + 4.0, g.y + 4.0), 1.5, eg(col));
            c.line(vec2(g.x - 4.0, g.y + 4.0), vec2(g.x + 4.0, g.y - 4.0), 1.5, eg(col));
        }
        c.circle(p.x, p.y, dot + 0.5, eg(race_color(w.people[m as usize].race)));
        if w.torch_lit(m) {
            c.circle_lines(p.x, p.y, dot + 5.0, 1.0, ega(palette::EMBER, 0.8));
        }
        if picked {
            c.circle_lines(p.x, p.y, dot + 3.0, 1.5, eg(GOLD));
        }
        picks.push((p, 0.0, Hover::Person(m)));
    }
    picks
}
