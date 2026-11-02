//! Third-person control (Laz, 2 Nov 2026; `docs/third-person.md`), in
//! dungeon sites for now: the camera behind the controlled character, the
//! pointer captured and a crosshair in the middle, the mouse turning the
//! view; WASD moves them the way the camera faces, Space jumps, Q dodges
//! back, C sneaks, E uses what the crosshair is on, Alt pauses, Tab takes the
//! next squad member in the site. Esc (or any screen opening) frees the
//! pointer; a click on the world takes it again. The sim does the moving
//! (`sim/dungeon.rs`); this is input and camera.

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::person::PersonId;

use super::app::{Game, View};
use super::hud::Canvas;
use super::palette::{ega, DIM, GOLD, TEXT, WARN};

/// Radians of turn per pixel of mouse movement.
const SENSE: f32 = 0.0035;
/// The camera: how far behind, how far to the right of the head, how high
/// the point it looks at stands above the feet.
const BEHIND: f32 = 4.5;
const SHOULDER: f32 = 0.95;
const HEAD: f32 = 1.75;

pub struct Third {
    /// The pointer is captured (the crosshair is up).
    pub locked: bool,
    /// The player wants it captured (until Esc).
    pub want: bool,
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
    /// What the crosshair is on: the thing, what E would do, and whether
    /// it's within reach.
    pub target: Option<(u16, String, bool)>,
    /// Third person was on last frame (the camera starts from the view).
    was: bool,
}

impl Default for Third {
    fn default() -> Self {
        Third { locked: false, want: true, yaw: 0.0, pitch: 0.32, dist: BEHIND, target: None, was: false }
    }
}

/// Who the player controls in a site: the one chosen before, if still in
/// one; else the first selected who is; else the first of the squad who is.
pub fn sync_lead(game: &mut Game) {
    let w = &game.world;
    let inside = |m: PersonId| w.site_of(m).is_some() && w.squad.index(m).is_some();
    let lead = w.sites.lead.filter(|&m| inside(m)).or_else(|| game.sel.who(w).into_iter().find(|&m| inside(m))).or_else(|| w.squad.members.iter().copied().find(|&m| inside(m)));
    if lead != game.world.sites.lead {
        game.world.sites.lead = lead;
        if let Some(m) = lead {
            game.sel.pick(m, false);
        }
    }
}

/// Is third person on: the 3D view, someone controlled in a site, nothing
/// else (the title, the editor) holding the screen.
pub fn active(game: &Game) -> bool {
    game.view == View::Scene && game.title.is_none() && !game.editor.on && game.world.sites.lead.is_some()
}

/// Does something on screen need the pointer free?
fn wants_pointer(game: &Game) -> bool {
    super::screens::open_now(game).is_some() || game.title.is_some() || game.options || game.menu.is_some() || game.examine.is_some() || game.world.talk.is_some() || game.parting.is_some() || game.trading.is_some() || super::lootui::open_loot(&game.world).is_some()
}

/// The keys and mouse in third person. True when they were all taken here
/// (the rest of the input waits); false to let the usual handling run (the
/// pointer is free, for the panels).
pub fn input(game: &mut Game, keys: &ButtonInput<KeyCode>, buttons: &ButtonInput<MouseButton>, motion: &AccumulatedMouseMotion, scroll: &AccumulatedMouseScroll, mouse: Vec2, dt: f32) -> bool {
    let _ = dt;
    if !game.third.was {
        // Coming in: the camera starts where the view looked.
        game.third.yaw = game.orbit.yaw;
        game.third.was = true;
    }
    // (Screenshots never take the pointer: it's Laz's mouse.)
    game.third.locked = game.third.want && !wants_pointer(game) && game.shot.is_none();
    let Some(lead) = game.world.sites.lead else { return false };
    // B: the squad's behaviours (open, the pointer is free for it).
    if keys.just_pressed(KeyCode::KeyB) {
        game.behave = !game.behave;
        game.world.steer_in_site(lead, None);
        return true;
    }
    if !game.third.locked {
        let on_panels = game.panels.iter().any(|b| b.contains(mouse));
        if buttons.just_pressed(MouseButton::Left) && !on_panels && !wants_pointer(game) && game.shot.is_none() {
            game.third.want = true;
            return true;
        }
        return false;
    }
    if keys.just_pressed(KeyCode::Escape) {
        // Free the pointer, and on to the usual Escape (the game menu).
        game.third.want = false;
        game.third.locked = false;
        for m in game.world.squad.members.clone() {
            game.world.steer_in_site(m, None);
        }
        return false;
    }
    // The mouse turns the view; the wheel brings the camera in or out.
    game.third.yaw += motion.delta.x * SENSE;
    game.third.pitch = (game.third.pitch + motion.delta.y * SENSE).clamp(-0.25, 1.25);
    if scroll.delta.y != 0.0 {
        game.third.dist = (game.third.dist - scroll.delta.y.signum() * 0.5).clamp(2.0, 8.0);
    }
    let w = &mut game.world;
    let f = V2::new(-game.third.yaw.cos(), -game.third.yaw.sin());
    let r = V2::new(-f.y, f.x);
    let mut d = V2::default();
    for (k, v) in [(KeyCode::KeyW, f), (KeyCode::KeyS, f.scale(-1.0)), (KeyCode::KeyA, r.scale(-1.0)), (KeyCode::KeyD, r)] {
        if keys.pressed(k) {
            d = d.add(v);
        }
    }
    w.steer_in_site(lead, (d.len() > 1e-3).then_some(d));
    w.sprint_in_site(lead, keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight));
    if keys.just_pressed(KeyCode::Space) {
        w.jump_in_site(lead);
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        w.dodge_in_site(lead, f);
    }
    if keys.just_pressed(KeyCode::KeyC) {
        let on = !w.is_sneaking(lead);
        w.set_sneaking(lead, on);
    }
    if keys.just_pressed(KeyCode::AltLeft) || keys.just_pressed(KeyCode::AltRight) {
        game.paused = !game.paused;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        if let Some((i, _, true)) = game.third.target.clone() {
            if game.world.use_in_reach(lead, i) {
                game.sounds.ui("ui_order");
            }
        } else {
            game.sounds.ui("ui_cant");
        }
    }
    if keys.just_pressed(KeyCode::Tab) {
        // The next of the squad in the same site.
        let w = &game.world;
        let site = w.site_of(lead).map(|i| i.site);
        let here: Vec<PersonId> = w.squad.members.iter().copied().filter(|&m| w.site_of(m).map(|i| i.site) == site).collect();
        if let Some(k) = here.iter().position(|&m| m == lead) {
            let next = here[(k + 1) % here.len()];
            if next != lead {
                game.world.steer_in_site(lead, None);
                game.world.sites.lead = Some(next);
                game.sel.pick(next, false);
            }
        }
    }
    // F1–F6 still pick out a member's portrait (and so whom you control).
    for (k, key) in [KeyCode::F1, KeyCode::F2, KeyCode::F3, KeyCode::F4, KeyCode::F5, KeyCode::F6].iter().enumerate() {
        if keys.just_pressed(*key) {
            if let Some(&m) = game.world.squad.members.get(k) {
                if game.world.site_of(m).is_some() {
                    game.world.steer_in_site(lead, None);
                    game.world.sites.lead = Some(m);
                    game.sel.pick(m, false);
                }
            }
        }
    }
    true
}

/// After the camera's usual follow step: in third person, behind the
/// controlled character, over the shoulder, and what the crosshair is on.
pub fn place_camera(game: &mut Game) {
    if !active(game) {
        game.third.was = false;
        game.third.locked = false;
        game.third.target = None;
        return;
    }
    let Some(lead) = game.world.sites.lead else { return };
    let Some(k) = game.world.squad.index(lead) else { return };
    let Some(h) = game.world.site_height(lead) else { return };
    let at = game.world.squad.at[k];
    let f = V2::new(-game.third.yaw.cos(), -game.third.yaw.sin());
    let r = V2::new(-f.y, f.x);
    // Over the shoulder, but not into a wall beside them; and back from
    // there, but not through a wall or the ceiling behind.
    let solid = |p: Vec3| super::dungeonview::solid_at(&game.world, p).unwrap_or(false);
    let head = Vec3::new(at.x, h + HEAD, at.y);
    let side = Vec3::new(r.x, 0.0, r.y);
    let mut shoulder = 0.0;
    while shoulder < SHOULDER && !solid(head + side * (shoulder + 0.25)) {
        shoulder += 0.05;
    }
    let look = head + side * shoulder;
    let (p, y) = (game.third.pitch, game.third.yaw);
    let back = Vec3::new(p.cos() * y.cos(), p.sin(), p.cos() * y.sin());
    let mut dist = 0.3;
    while dist < game.third.dist && !solid(look + back * (dist + 0.25)) {
        dist += 0.05;
    }
    game.orbit.target = V2::new(look.x, look.z);
    game.orbit.ground = look.y;
    game.orbit.yaw = y;
    game.orbit.pitch = p;
    game.orbit.dist = dist.max(0.4);
    game.follow = false;
    game.glide = None;
    // What the crosshair is on (the screen's middle).
    let ray = game.orbit.ray(game.screen, game.screen * 0.5);
    game.third.target = super::dungeonview::pick(&game.world, ray).and_then(|(site, p, ph)| {
        let i = game.world.site_object_near(site, p, ph)?;
        let label = game.world.site_use_label(site, i)?;
        Some((i, label, game.world.site_in_reach(lead, i)))
    });
}

/// The crosshair and what E would do; or, with the pointer free, how to take
/// it back.
pub fn draw(c: &Canvas, game: &Game) {
    if !active(game) || game.title.is_some() {
        return;
    }
    let mid = game.screen * 0.5;
    if game.third.locked || game.shot.is_some() {
        let col = ega(GOLD, 0.85);
        c.line(mid - Vec2::new(9.0, 0.0), mid - Vec2::new(3.0, 0.0), 2.0, col);
        c.line(mid + Vec2::new(3.0, 0.0), mid + Vec2::new(9.0, 0.0), 2.0, col);
        c.line(mid - Vec2::new(0.0, 9.0), mid - Vec2::new(0.0, 3.0), 2.0, col);
        c.line(mid + Vec2::new(0.0, 3.0), mid + Vec2::new(0.0, 9.0), 2.0, col);
        if let Some((_, label, reach)) = &game.third.target {
            let s = if *reach { format!("E  {label}") } else { format!("{label} (too far)") };
            let tw = c.width(&s, 15.0);
            c.text(&s, mid.x - tw / 2.0, mid.y + 34.0, 15.0, if *reach { TEXT } else { WARN });
        }
        if game.paused {
            let s = "Paused (Alt)";
            let tw = c.width(s, 15.0);
            c.text(s, mid.x - tw / 2.0, mid.y - 30.0, 15.0, GOLD);
        }
    } else if !wants_pointer(game) {
        let s = "Click to take control  ·  Esc frees the pointer";
        let tw = c.width(s, 14.0);
        c.text(s, mid.x - tw / 2.0, mid.y + 34.0, 14.0, DIM);
    }
}

/// The pointer: captured and hidden in third person; otherwise kept inside
/// the window while playing (Laz: "the cursor needs to lock into the window
/// when in play"), free on the title screen and the game menu. Screenshot
/// runs never take it.
pub fn pointer(game: Res<Game>, mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    let playing = game.title.is_none() && game.game_menu.is_none() && game.shot.is_none() && !game.quit;
    let grab = if game.third.locked {
        CursorGrabMode::Locked
    } else if playing {
        CursorGrabMode::Confined
    } else {
        CursorGrabMode::None
    };
    if cursor.grab_mode != grab {
        cursor.grab_mode = grab;
    }
    let visible = !game.third.locked;
    if cursor.visible != visible {
        cursor.visible = visible;
    }
}
