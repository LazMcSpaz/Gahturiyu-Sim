//! The window: Bevy runs the frame, the simulation is stepped each frame, and
//! the panels are drawn with egui. Nothing here decides what happens in the
//! world; clicks and keys become orders (`order_squad` and friends).
//!
//! Frame order: read keys and mouse → step the world → aim the camera and
//! light the scene → build the 3D scene → draw the panels → screenshot.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::{PresentMode, PrimaryWindow, WindowResolution};
use bevy_egui::{egui, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};

use gahturiyu_sim::sim::{
    geo::V2,
    items,
    person::PersonId,
    worldgen, World,
};

use super::cam::{to3, MapCam, OrbitCam, FOV_DEG};
use super::hud::{self, Canvas, SPEEDS};
use super::squadui::{self, Action, Bx, Click, Selection};
use super::{light, map, models, scene, shot::Shot};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum View {
    Scene,
    Map,
}

/// Something the mouse can be over.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hover {
    Person(PersonId),
    Group(u32),
    Town(u16),
    /// Something lying on the ground.
    Item(u32),
    /// A building's door.
    Door((u16, u16)),
    /// Something to gather.
    Node(u32),
    /// A workshop (index into `World::stations`).
    Station(usize),
    /// A standing torch (index into `World::standing`).
    Torch(usize),
    /// A woodlot or mine the squad can work.
    Deposit(u32),
}

/// Everything the window keeps between frames.
#[derive(Resource)]
pub struct Game {
    pub world: World,
    pub view: View,
    pub orbit: OrbitCam,
    pub map_cam: MapCam,
    pub follow: bool,
    pub speed_i: usize,
    pub paused: bool,
    pub rings: bool,
    /// The detail-level readout (L).
    pub debug: bool,
    pub sel: Selection,
    pub inv: Option<PersonId>,
    pub craft: Option<PersonId>,
    pub journal: bool,
    /// The town panel (P, or click a town's name).
    pub town: Option<u16>,
    /// The graphics settings panel (O).
    pub options: bool,
    /// Whose spell book is open (M).
    pub book: Option<PersonId>,
    /// A spell waiting for its target: the next click in the world aims it.
    pub aim: Option<(PersonId, gahturiyu_sim::sim::magic::Spell)>,
    /// How many times a save has been loaded (so cached drawing of the old
    /// world is thrown away).
    pub loads: u32,
    /// A short message on screen ("Saved."), and when it appeared.
    pub notice: Option<(String, std::time::Instant)>,
    /// The wild animal or carcass under the mouse last frame (found by
    /// `animals::overlay`), for clicks: hunt it, or cut it up.
    pub wild: Option<super::animals::Seen>,
    /// First-hour tips.
    pub hints: super::hints::Hints,
    pub shot: Option<Shot>,
    pub frame: u32,
    pub shot_at: Option<u32>,
    pub sim_ms: f64,
    pub frame_ms: f64,
    pub screen: Vec2,
    pub mouse: Vec2,
    last_mouse: Vec2,
    press_at: Option<Vec2>,
    /// A click on the panels, for them to handle as they're drawn.
    pub ui_click: Option<Click>,
    /// Where the panels were last frame (a click there isn't an order).
    pub panels: Vec<Bx>,
    /// What the mouse was over last frame.
    pub hover: Option<Hover>,
    /// Filled by the 3D scene each frame: hoverable points, town names and
    /// health bars, in world space.
    pub picks: Vec<(Vec3, f32, Hover)>,
    pub labels: Vec<(Vec3, String)>,
    pub bars: Vec<(Vec3, f32, Option<f32>, bool)>,
    /// Remarks townsfolk make as the squad goes by: (who, what, until frame),
    /// and when each may speak up again.
    pub barks: Vec<(PersonId, String, u32)>,
    /// The town's talk panel shows everything (as with the detail readout).
    pub town_all: bool,
    /// The land editor (F10).
    pub editor: super::editor::Editor,
    pub barked: std::collections::HashMap<PersonId, u32>,
    /// The Build panel (B), and a building waiting to be put down.
    pub build: bool,
    /// The Base tab of the Build panel rather than the list of buildings.
    pub base_tab: bool,
    pub placing: Option<super::baseui::Placing>,
}

pub fn run() {
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let shot = Shot::from_env();
    // GAHT_LOAD=path starts from a save instead of a new world.
    let loaded = std::env::var("GAHT_LOAD").ok().map(|p| World::load_from(std::path::Path::new(&p)));
    let mut world = match loaded {
        Some(Ok(w)) => w,
        Some(Err(e)) => {
            eprintln!("{e}; starting a new world");
            worldgen::generate(seed)
        }
        // A map authored for this seed (the land editor saves it). Screenshots
        // use the seed's own land unless GAHT_MAP is set.
        None => {
            // The forged town (assets/towns/demo), then the map, unless a
            // screenshot asks for the seed's own land.
            let forge = match gahturiyu_sim::sim::forge::load(&forge_dir(), seed) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("couldn't read the forged town: {e}");
                    None
                }
            };
            let path = map_path(seed);
            let edits = if shot.is_some() && std::env::var("GAHT_MAP").is_err() {
                Default::default()
            } else {
                match gahturiyu_sim::sim::mapedit::MapEdits::load_from(&path) {
                    Ok((_, edits)) => edits,
                    Err(e) => {
                        if path.exists() {
                            eprintln!("couldn't read {}: {e}; starting from the seed's own land", path.display());
                        }
                        Default::default()
                    }
                }
            };
            worldgen::generate_authored(seed, forge, edits)
        }
    };
    if let Some(s) = &shot {
        s.prepare(&mut world);
    }
    let mut game = Game {
        view: View::Scene,
        map_cam: MapCam { centre: world.squad.pos, zoom: 1600.0 / 7000.0 },
        orbit: OrbitCam::new(world.squad.pos),
        follow: true,
        speed_i: 2,
        paused: false,
        rings: true,
        debug: false,
        sel: Selection::default(),
        inv: None,
        craft: None,
        journal: false,
        town: None,
        options: std::env::var("GAHT_SETTINGS").is_ok(),
        book: None,
        aim: None,
        loads: 0,
        notice: None,
        wild: None,
        hints: {
            let mut h = super::hints::Hints::load(shot.is_none());
            if let Ok(id) = std::env::var("GAHT_HINT") {
                h.force(&id);
            }
            h
        },
        frame: 0,
        shot_at: None,
        sim_ms: 0.0,
        frame_ms: 16.0,
        screen: Vec2::new(1600.0, 1000.0),
        mouse: Vec2::ZERO,
        last_mouse: Vec2::ZERO,
        press_at: None,
        ui_click: None,
        panels: Vec::new(),
        hover: None,
        picks: Vec::new(),
        labels: Vec::new(),
        bars: Vec::new(),
        barks: Vec::new(),
        town_all: false,
        editor: Default::default(),
        barked: std::collections::HashMap::new(),
        build: false,
        base_tab: false,
        placing: None,
        shot: None,
        world,
    };
    game.orbit.ground = game.world.terrain.surface(game.world.squad.pos);
    if let Some(s) = &shot {
        if s.view.as_deref() == Some("map") {
            game.view = View::Map;
        }
        if let Some(z) = s.zoom {
            if game.view == View::Map {
                game.map_cam.zoom = z;
            } else {
                game.orbit.dist = z;
            }
        }
        if let Some(p) = s.pitch {
            game.orbit.pitch = p;
        }
        if let Some(y) = s.yaw {
            game.orbit.yaw = y;
        }
        if let Some(sp) = s.speed {
            game.speed_i = sp.min(SPEEDS.len() - 1);
        }
        game.debug = s.debug;
        if let Some(k) = s.select {
            if let Some(&m) = game.world.squad.members.get(k) {
                game.sel.pick(m, false);
            }
        }
        game.inv = s.inventory.and_then(|k| game.world.squad.members.get(k).copied());
        game.craft = s.craft.and_then(|k| game.world.squad.members.get(k).copied());
        game.book = s.book.and_then(|k| game.world.squad.members.get(k).copied());
        if s.feud {
            game.town_all = true;
        }
        if s.guard {
            game.journal = true;
        }
        if s.edit {
            game.editor.on = true;
            game.paused = true;
            let at = game.world.squad.pos.add(V2::new(60.0, -70.0));
            game.editor.cursor = Some(at);
            game.editor.tool = super::editor::Tool::Paint(11);
            game.follow = false;
            game.orbit.target = at;
        }
        if let Some(what) = &s.build {
            game.build = true;
            if what == "base" {
                game.base_tab = true;
            } else {
                game.placing = Some(super::baseui::Placing { def: gahturiyu_sim::sim::base::def_index("hut"), ..Default::default() });
            }
        }
        if s.town || s.feud {
            let at = game.world.squad.pos;
            game.town = game.world.settlements.iter().min_by(|a, b| a.pos.dist(at).total_cmp(&b.pos.dist(at))).map(|t| t.id);
        }
        if let Some(p) = s.focus(&game.world) {
            game.follow = false;
            game.orbit.target = p;
            game.orbit.ground = game.world.terrain.surface(p);
        }
        if s.forest {
            if let Some(p) = super::foliage::biggest_wood_near(&game.world, game.world.squad.pos) {
                game.follow = false;
                game.orbit.target = p;
                game.orbit.ground = game.world.terrain.surface(p);
                if s.zoom.is_none() {
                    game.orbit.dist = 900.0;
                }
                if s.pitch.is_none() {
                    game.orbit.pitch = 0.32;
                }
            }
        }
    }
    // Screenshots wait for every shader to be ready, so the first frames
    // aren't missing whatever was still being prepared.
    let synchronous = shot.is_some();
    game.shot = shot;

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Gahturiyu".into(),
                        resolution: WindowResolution::new(1600, 1000),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin { file_path: models::assets_dir().to_string_lossy().into_owned(), ..default() })
                .set(bevy::render::RenderPlugin { synchronous_pipeline_compilation: synchronous, ..default() }),
        )
        .insert_resource(EguiGlobalSettings { auto_create_primary_context: false, ..default() })
        .add_plugins(EguiPlugin::default())
        .add_plugins(super::foliage::FoliagePlugin)
        .add_plugins(super::weather::WeatherPlugin)
        .insert_resource(game)
        .insert_resource(super::settings::Settings::load())
        .init_resource::<scene::Scene3d>()
        .init_resource::<models::Models>()
        .init_resource::<super::foliage::Foliage>()
        .insert_resource(ClearColor(Color::srgb(0.63, 0.69, 0.72)))
        .add_systems(Startup, (setup, scene::setup, light::setup, models::start, super::foliage::setup))
        .add_systems(Update, (input, simulate, camera, light::update, models::finish, scene::update, super::foliage::update, screenshot).chain())
        .add_systems(EguiPrimaryContextPass, ui)
        .run();
}

#[derive(Component)]
pub struct MainCamera;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera { order: 0, ..default() },
        Projection::Perspective(PerspectiveProjection { fov: FOV_DEG.to_radians(), near: 0.2, far: 30_000.0, ..default() }),
        Transform::from_xyz(0.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
        light::camera_bundle(),
    ));
    // The panels are drawn by a second camera on top.
    // It shares the 3D camera's wide-range picture (so it draws on top of it
    // rather than replacing it) and leaves its colours alone.
    commands.spawn((Camera2d, Camera { order: 1, clear_color: ClearColorConfig::None, ..default() }, bevy::camera::Hdr, bevy::core_pipeline::tonemapping::Tonemapping::None, PrimaryEguiContext));
}

/// Where the town forge writes the demo town.
pub fn forge_dir() -> std::path::PathBuf {
    models::assets_dir().join("towns").join("demo")
}

/// Where the map for a seed is kept (beside the saves).
pub fn map_path(seed: u64) -> std::path::PathBuf {
    let assets = models::assets_dir();
    assets.parent().map(|p| p.to_path_buf()).unwrap_or_default().join(gahturiyu_sim::sim::mapedit::MapEdits::path_for(seed))
}

pub fn quick_save() -> std::path::PathBuf {
    let assets = models::assets_dir();
    assets.parent().map(|p| p.to_path_buf()).unwrap_or_default().join("saves").join("quick.sav")
}

/// Put a loaded world in place of the current one, and forget anything the
/// window was holding about the old one.
fn swap_world(game: &mut Game, w: World) {
    game.world = w;
    game.loads += 1;
    game.sel = Selection::default();
    game.inv = None;
    game.craft = None;
    game.hover = None;
    game.build = false;
    game.placing = None;
    game.follow = true;
    game.orbit.target = game.world.squad.pos;
    game.orbit.ground = game.world.terrain.surface(game.world.squad.pos);
    game.map_cam.centre = game.world.squad.pos;
}

/// Keys and mouse: camera moves, and orders.
fn input(mut game: ResMut<Game>, keys: Res<ButtonInput<KeyCode>>, buttons: Res<ButtonInput<MouseButton>>, scroll: Res<AccumulatedMouseScroll>, window: Single<&Window, With<PrimaryWindow>>, time: Res<Time>) {
    let game = &mut *game;
    game.screen = Vec2::new(window.width(), window.height());
    let mut mouse = window.cursor_position().unwrap_or(game.last_mouse);
    if let Some(h) = game.shot.as_ref().and_then(|s| s.hover) {
        mouse = h;
    }
    game.mouse = mouse;
    let dt = time.delta_secs();
    if keys.just_pressed(KeyCode::F10) || (game.editor.on && keys.just_pressed(KeyCode::Escape)) {
        super::editor::toggle(game);
        game.last_mouse = mouse;
        return;
    }
    if game.editor.on {
        // Camera as usual; the brush instead of orders; the world stands still.
        camera_input(game, &keys, &buttons, &scroll, mouse, dt);
        if keys.just_pressed(KeyCode::KeyV) || keys.just_pressed(KeyCode::Tab) {
            game.view = if game.view == View::Scene { View::Map } else { View::Scene };
        }
        let on_panels = game.panels.iter().any(|b| b.contains(mouse));
        game.ui_click = None;
        let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        // A click on the panel is one that began there too (not a stroke
        // let go over it).
        if buttons.just_pressed(MouseButton::Left) {
            game.press_at = Some(mouse);
        }
        if buttons.just_released(MouseButton::Left) {
            let began_on = game.press_at.take().is_some_and(|p| game.panels.iter().any(|b| b.contains(p)));
            if on_panels && began_on {
                game.ui_click = Some(Click { at: mouse, right: false, shift });
            }
        }
        super::editor::input(game, &keys, &buttons, mouse, on_panels, dt, time.elapsed_secs());
        game.last_mouse = mouse;
        return;
    }
    let w = &mut game.world;

    for (i, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5].iter().enumerate() {
        if keys.just_pressed(*key) {
            game.speed_i = i;
            game.paused = false;
        }
    }
    if keys.just_pressed(KeyCode::Space) {
        game.paused = !game.paused;
    }
    if keys.just_pressed(KeyCode::KeyC) || keys.just_pressed(KeyCode::KeyF) {
        game.follow = true;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        game.rings = !game.rings;
    }
    if keys.just_pressed(KeyCode::KeyL) {
        game.debug = !game.debug;
    }
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    for (k, key) in [KeyCode::F1, KeyCode::F2, KeyCode::F3, KeyCode::F4, KeyCode::F5, KeyCode::F6].iter().enumerate() {
        if keys.just_pressed(*key) {
            if let Some(&m) = w.squad.members.get(k) {
                game.sel.pick(m, shift);
                if game.inv.is_some() {
                    game.inv = Some(m);
                }
                if game.craft.is_some() {
                    game.craft = Some(m);
                }
                if game.book.is_some() {
                    game.book = Some(m);
                }
            }
        }
    }
    if keys.just_pressed(KeyCode::Escape) && game.placing.is_some() {
        // Finish a wall being drawn, or stop placing.
        match game.placing.as_mut() {
            Some(p) if !p.chain.is_empty() => p.chain.clear(),
            _ => game.placing = None,
        }
    } else if keys.just_pressed(KeyCode::Escape) && game.aim.is_some() {
        game.aim = None;
    } else if keys.just_pressed(KeyCode::Escape) && w.talk.is_some() {
        w.end_talk();
    } else if keys.just_pressed(KeyCode::Backquote) || keys.just_pressed(KeyCode::Escape) {
        game.sel = Selection::default();
        game.inv = None;
        game.craft = None;
    }
    if keys.just_pressed(KeyCode::KeyZ) {
        let who = game.sel.who(w);
        let on = !who.iter().all(|&m| w.is_sneaking(m));
        for m in who {
            w.set_sneaking(m, on);
        }
    }
    if keys.just_pressed(KeyCode::KeyX) {
        for m in game.sel.who(w) {
            w.put_down(m);
        }
    }
    if keys.just_pressed(KeyCode::KeyN) {
        let who = game.sel.who(w);
        w.order_rest(&who);
    }
    // T: the selected light their torches (or put them out). With everyone
    // selected, only those already holding one, else the first who has one.
    if keys.just_pressed(KeyCode::KeyT) {
        let who = game.sel.who(w);
        let holding: Vec<PersonId> = who.iter().copied().filter(|&m| w.torch_in_hand(m).is_some()).collect();
        let targets = if !holding.is_empty() || !game.sel.is_all(w) {
            if holding.is_empty() { who } else { holding }
        } else {
            who.iter().copied().find(|&m| w.people[m as usize].detail.as_ref().map(|d| d.gear.bag.iter().any(|e| e.0 == items::id("torch"))).unwrap_or(false)).into_iter().collect()
        };
        for m in targets {
            w.toggle_torch(m);
        }
    }
    if keys.just_pressed(KeyCode::KeyI) {
        game.craft = None;
        game.book = None;
        game.inv = match game.inv {
            Some(_) => None,
            None => game.sel.lead(w),
        };
    }
    if keys.just_pressed(KeyCode::KeyJ) {
        game.journal = !game.journal;
    }
    // P: the town panel for the town nearest the camera.
    if keys.just_pressed(KeyCode::KeyP) {
        let at = match game.view {
            View::Scene => game.orbit.target,
            View::Map => w.squad.pos,
        };
        game.town = match game.town {
            Some(_) => None,
            None => w.settlements.iter().min_by(|a, b| a.pos.dist(at).total_cmp(&b.pos.dist(at))).map(|s| s.id),
        };
    }
    if keys.just_pressed(KeyCode::KeyO) {
        game.options = !game.options;
    }
    if keys.just_pressed(KeyCode::F8) {
        let msg = match w.save_to(&quick_save()) {
            Ok(()) => "Saved.".to_string(),
            Err(e) => format!("Couldn't save: {e}"),
        };
        game.notice = Some((msg, std::time::Instant::now()));
        return;
    }
    if keys.just_pressed(KeyCode::F9) {
        let msg = match World::load_from(&quick_save()) {
            Ok(loaded) => {
                swap_world(game, loaded);
                "Loaded.".to_string()
            }
            Err(gahturiyu_sim::sim::save::LoadError::Io(_)) => "No save yet (F8 saves).".to_string(),
            Err(e) => format!("Couldn't load: {e}"),
        };
        game.notice = Some((msg, std::time::Instant::now()));
        return;
    }
    if keys.just_pressed(KeyCode::KeyK) {
        game.inv = None;
        game.book = None;
        game.craft = match game.craft {
            Some(_) => None,
            None => game.sel.lead(w),
        };
    }
    // Look through a scout spirit (G again, or C, to come back).
    if keys.just_pressed(KeyCode::KeyG) {
        let t = w.time;
        let scout = w.wards.iter().filter(|x| x.does == gahturiyu_sim::sim::effects::Does::Scout && x.until > t).map(|x| x.pos).last();
        match scout {
            Some(p) if game.follow || game.orbit.target.dist(p) > 1.0 => {
                game.follow = false;
                game.orbit.target = p;
                game.orbit.ground = w.terrain.surface(p);
                game.orbit.dist = game.orbit.dist.min(60.0);
                game.notice = Some(("Looking through the scout".to_string(), std::time::Instant::now()));
            }
            Some(_) => game.follow = true,
            None => game.notice = Some(("No scout spirit out".to_string(), std::time::Instant::now())),
        }
    }
    if keys.just_pressed(KeyCode::KeyM) {
        game.inv = None;
        game.craft = None;
        game.book = match game.book {
            Some(_) => None,
            None => game.sel.lead(w),
        };
    }
    // B: the Build panel. (Shift+B: bandits, for testing.)
    if keys.just_pressed(KeyCode::KeyB) && !shift {
        game.build = !game.build;
        if !game.build {
            game.placing = None;
        }
    }
    if keys.just_pressed(KeyCode::KeyB) && shift {
        let n = 2 + (w.time as usize % 3);
        let at = w.squad.pos.add(V2::new(26.0, 12.0));
        w.spawn_bandits(at, n, true);
    }
    if keys.just_pressed(KeyCode::KeyV) || keys.just_pressed(KeyCode::Tab) {
        game.view = if game.view == View::Scene { View::Map } else { View::Scene };
        if game.view == View::Map {
            game.map_cam.centre = game.orbit.target;
        } else {
            game.orbit.target = game.map_cam.centre;
        }
    }

    // A building being placed: Shift + wheel turns it; its ghost follows
    // the cursor.
    let turning = game.placing.is_some() && shift && scroll.delta.y != 0.0;
    if turning {
        if let Some(p) = game.placing.as_mut() {
            p.rot += scroll.delta.y.signum() * std::f32::consts::PI / 12.0;
        }
    }
    if game.placing.is_some() {
        let at = match game.view {
            View::Map => Some(game.map_cam.to_world(game.screen, mouse)),
            View::Scene => game.orbit.ground_at(game.screen, mouse, &game.world.terrain),
        };
        let on_panels = game.panels.iter().any(|b| b.contains(mouse));
        let mut p = game.placing.take().unwrap();
        super::baseui::update_ghost(&game.world, &mut p, if on_panels { None } else { at });
        game.placing = Some(p);
    }

    if !turning {
        camera_input(game, &keys, &buttons, &scroll, mouse, dt);
    }
    let on_panels = game.panels.iter().any(|b| b.contains(mouse));
    // Clicks on the panels are handled when they're drawn; a short click
    // anywhere else is an order.
    game.ui_click = None;
    if buttons.just_pressed(MouseButton::Left) {
        game.press_at = Some(mouse);
    }
    if buttons.just_pressed(MouseButton::Right) && on_panels {
        game.ui_click = Some(Click { at: mouse, right: true, shift });
    }
    // Right-click anywhere drops a spell that's waiting to be aimed.
    if buttons.just_pressed(MouseButton::Right) && game.aim.is_some() {
        game.aim = None;
    }
    if buttons.just_released(MouseButton::Left) {
        if let Some(p) = game.press_at.take() {
            if (p - mouse).length() < 6.0 {
                if on_panels {
                    game.ui_click = Some(Click { at: mouse, right: false, shift });
                } else {
                    click_world(game, mouse, shift);
                }
            }
        }
    }
    game.last_mouse = mouse;
}

/// Turning, panning and zooming the camera (the same in the editor).
fn camera_input(game: &mut Game, keys: &ButtonInput<KeyCode>, buttons: &ButtonInput<MouseButton>, scroll: &AccumulatedMouseScroll, mouse: Vec2, dt: f32) {
    let d = mouse - game.last_mouse;
    let wheel = scroll.delta.y;
    let mut pan = (0.0f32, 0.0f32);
    // (Not while Ctrl is down: Ctrl+S saves.)
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    for (k, dx, dy) in [(KeyCode::KeyW, 0.0, 1.0), (KeyCode::KeyS, 0.0, -1.0), (KeyCode::KeyA, -1.0, 0.0), (KeyCode::KeyD, 1.0, 0.0)] {
        if keys.pressed(k) && !ctrl {
            pan.0 += dx;
            pan.1 += dy;
        }
    }
    let on_panels = game.panels.iter().any(|b| b.contains(mouse));
    match game.view {
        View::Map => {
            if wheel != 0.0 && !on_panels {
                game.map_cam.zoom_at(game.screen, mouse, wheel);
            }
            if (buttons.pressed(MouseButton::Right) || buttons.pressed(MouseButton::Middle)) && d.length() > 0.0 {
                game.map_cam.centre = game.map_cam.centre.sub(V2::new(d.x / game.map_cam.zoom, d.y / game.map_cam.zoom));
                game.follow = false;
            }
            if pan != (0.0, 0.0) {
                let s = 900.0 / game.map_cam.zoom * dt;
                game.map_cam.centre = game.map_cam.centre.add(V2::new(pan.0 * s, -pan.1 * s));
                game.follow = false;
            }
        }
        View::Scene => {
            if wheel != 0.0 && !on_panels {
                game.orbit.zoom(wheel);
            }
            if buttons.pressed(MouseButton::Right) && d.length() > 0.0 {
                game.orbit.orbit(d.x, d.y);
            }
            if buttons.pressed(MouseButton::Middle) && d.length() > 0.0 {
                game.orbit.pan(-d.x / game.screen.y * 1.6, d.y / game.screen.y * 1.6);
                game.follow = false;
            }
            if pan != (0.0, 0.0) {
                game.orbit.pan(pan.0 * dt, pan.1 * dt);
                game.follow = false;
            }
            if keys.pressed(KeyCode::KeyQ) {
                game.orbit.orbit(-260.0 * dt, 0.0);
            }
            if keys.pressed(KeyCode::KeyE) {
                game.orbit.orbit(260.0 * dt, 0.0);
            }
        }
    }
}

/// Step the world by this frame's share of game time.
fn simulate(mut game: ResMut<Game>, time: Res<Time>) {
    let game = &mut *game;
    let dt = time.delta_secs();
    game.frame_ms = game.frame_ms * 0.9 + dt as f64 * 1000.0 * 0.1;
    // A fight breaking out near the squad drops the game to real time.
    if !game.world.alerts.is_empty() {
        game.world.alerts.clear();
        game.speed_i = 0;
        game.paused = false;
    }
    // Time stands still while you talk, as in Morrowind.
    if !game.paused && game.world.talk.is_none() && !game.editor.on {
        let t0 = std::time::Instant::now();
        // Screenshots run at a fixed pace so they don't depend on how fast
        // the machine draws.
        let dt = if game.shot.is_some() { 1.0 / 30.0 } else { dt.min(0.1) };
        let mut left = dt as f64 * SPEEDS[game.speed_i].0;
        let sub = (left / 600.0).max(1.0);
        while left > 1e-9 {
            let step = left.min(sub);
            game.world.step(step);
            left -= step;
        }
        game.sim_ms = game.sim_ms * 0.9 + t0.elapsed().as_secs_f64() * 1000.0 * 0.1;
    }
    if game.follow {
        game.map_cam.centre = game.world.squad.pos;
        game.orbit.target = game.world.squad.pos;
    }
    // Keep the camera's pivot on the ground, easing so it doesn't jolt.
    let g = game.world.terrain.surface(game.orbit.target);
    game.orbit.ground += (g - game.orbit.ground) * (1.0 - (-8.0 * dt).exp());
    if (g - game.orbit.ground).abs() > 60.0 || game.frame < 2 {
        game.orbit.ground = g;
    }
}

/// Aim the 3D camera; switch it off on the map.
fn camera(game: Res<Game>, mut cam: Query<(&mut Camera, &mut Transform, &mut Projection), With<MainCamera>>) {
    let Ok((mut c, mut tf, mut proj)) = cam.single_mut() else { return };
    c.is_active = game.view == View::Scene;
    *tf = Transform::from_translation(game.orbit.eye()).looking_at(game.orbit.look_at(), Vec3::Y);
    if let Projection::Perspective(p) = &mut *proj {
        p.near = game.orbit.near();
        p.far = game.orbit.far();
    }
}

/// A click while a building is being placed: put it down (Shift keeps
/// placing more of the same); for a wall, add the next point.
fn place_click(game: &mut Game, shift: bool) {
    use gahturiyu_sim::sim::base::{Kind, BUILDINGS};
    let Some(mut p) = game.placing.take() else { return };
    let d = &BUILDINGS[p.def];
    let Some(&(plan, ok)) = p.ghost.last() else {
        game.placing = Some(p);
        return;
    };
    let w = &mut game.world;
    let mut keep = shift;
    let res = if d.kind == Kind::Wall {
        keep = true;
        match p.chain.last().copied() {
            None => ok.map(|_| p.chain.push(plan.at)),
            Some(last) => {
                let to = p.cursor.unwrap_or(plan.at);
                w.place_wall(p.def, &[last, to]).map(|_| p.chain.push(to))
            }
        }
    } else if d.kind == Kind::Marker {
        keep = false;
        w.found_base(plan.at).map(|_| ())
    } else {
        w.place_building(plan).map(|_| ())
    };
    match res {
        Ok(()) => {}
        Err(e) => {
            keep = true;
            game.notice = Some((format!("{}: {}", d.name, e.why()), std::time::Instant::now()));
        }
    }
    if keep {
        game.placing = Some(p);
    }
}

/// A short left-click in the world: select a squad member, attack an enemy,
/// pick something up, or walk there.
fn click_world(game: &mut Game, mouse: Vec2, shift: bool) {
    if game.placing.is_some() {
        place_click(game, shift);
        return;
    }
    let hover = game.hover;
    // Aiming a spell: whoever is under the mouse, and the spot.
    if let Some((who, s)) = game.aim.take() {
        let target = match hover {
            Some(Hover::Person(pid)) => Some(pid),
            _ => None,
        };
        let point = match game.view {
            View::Map => Some(game.map_cam.to_world(game.screen, mouse)),
            View::Scene => game.orbit.ground_at(game.screen, mouse, &game.world.terrain),
        };
        let point = match (target, hover) {
            (Some(p), _) => Some(game.world.person_pos(p)),
            (None, Some(Hover::Door(id))) => game.world.door(id).map(|d| d.outside),
            _ => point,
        };
        if let Err(e) = game.world.use_spell(who, s, target, point) {
            game.notice = Some((format!("{}: {}", s.def().name, e.0), std::time::Instant::now()));
        }
        return;
    }
    let world = &mut game.world;
    let who = game.sel.who(world);
    match hover {
        // A beaten foe: the nearest selected member goes through their things
        // (Shift-click to carry them off instead).
        Some(Hover::Person(pid)) if world.can_loot(pid) && !shift => {
            let at = world.person_pos(pid);
            let looter = who.iter().copied().min_by(|&a, &b| world.person_pos(a).dist(at).total_cmp(&world.person_pos(b).dist(at)));
            if let Some(m) = looter {
                world.order_loot(m, pid);
                return;
            }
        }
        Some(Hover::Person(pid)) if who.iter().any(|&m| world.can_carry(m, pid)) => {
            let at = world.body_pos(pid);
            let carrier = who.iter().copied().filter(|&m| world.can_carry(m, pid)).min_by(|&a, &b| world.person_pos(a).dist(at).total_cmp(&world.person_pos(b).dist(at)));
            if let Some(c) = carrier {
                world.order_carry(c, pid);
                return;
            }
        }
        Some(Hover::Person(pid)) if world.squad.index(pid).is_some() => {
            game.sel.pick(pid, shift);
            return;
        }
        Some(Hover::Person(pid)) => {
            if world.attack(&who, pid) {
                return;
            }
            if let Some(&lead) = who.first() {
                if world.squad_battle().is_none() && world.order_talk(lead, pid) {
                    return;
                }
            }
        }
        Some(Hover::Door(id)) => {
            if world.is_locked(id) {
                let pick = items::id("lockpick");
                let picker = who
                    .iter()
                    .copied()
                    .filter(|&m| world.people[m as usize].detail.as_ref().map(|d| d.gear.bag.iter().any(|e| e.0 == pick)).unwrap_or(false))
                    .max_by(|&a, &b| world.pick_chance(a, 50.0).total_cmp(&world.pick_chance(b, 50.0)));
                match picker {
                    Some(p) => {
                        world.order_pick(p, id);
                    }
                    None => world.log.push_front((world.time, "Nobody selected has a lockpick.".into())),
                }
                return;
            }
            if let Some(d) = world.door(id) {
                world.order_members(&who, d.centre);
                return;
            }
        }
        Some(Hover::Town(t)) => {
            game.town = if game.town == Some(t) { None } else { Some(t) };
            return;
        }
        Some(Hover::Node(node)) => {
            let pos = world.nodes.iter().find(|n| n.id == node).map(|n| n.pos);
            if let Some(pos) = pos {
                if let Some(f) = who.iter().copied().min_by(|&a, &b| world.person_pos(a).dist(pos).total_cmp(&world.person_pos(b).dist(pos))) {
                    world.order_gather(f, node);
                    return;
                }
            }
        }
        Some(Hover::Deposit(id)) => {
            let pos = world.deposit(id).map(|d| d.pos);
            if let Some(pos) = pos {
                // Everyone selected goes to work it.
                let mut any = false;
                for &m in &who {
                    any |= world.order_labour(m, id);
                }
                if any {
                    let _ = pos;
                    return;
                }
            }
        }
        Some(Hover::Item(thing)) => {
            let pos = world.ground.iter().find(|g| g.id == thing).map(|g| g.pos);
            if let Some(pos) = pos {
                if let Some(f) = who.iter().copied().min_by(|&a, &b| world.person_pos(a).dist(pos).total_cmp(&world.person_pos(b).dist(pos))) {
                    world.order_pickup(f, thing);
                    return;
                }
            }
        }
        None => {
            use super::animals::Thing;
            match game.wild.map(|s| s.thing) {
                Some(Thing::Animal(h, _)) | Some(Thing::Herd(h)) => {
                    if world.order_hunt(&who, h) {
                        return;
                    }
                }
                Some(Thing::Carcass(id)) => {
                    let pos = world.animals.carcasses.iter().find(|c| c.id == id).map(|c| c.pos);
                    if let Some(pos) = pos {
                        if let Some(m) = who.iter().copied().min_by(|&a, &b| world.person_pos(a).dist(pos).total_cmp(&world.person_pos(b).dist(pos))) {
                            if world.order_butcher(m, id) {
                                return;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    let target = match game.view {
        View::Map => Some(game.map_cam.to_world(game.screen, mouse)),
        View::Scene => game.orbit.ground_at(game.screen, mouse, &game.world.terrain),
    };
    let world = &mut game.world;
    if let Some(t) = target {
        if game.sel.is_all(world) {
            world.order_squad(t);
        } else {
            world.order_members(&who, t);
        }
    }
}

/// How near a squad member someone must be to bark, metres; how long a
/// remark stays up and how long before the same person speaks up again,
/// frames.
const BARK_NEAR: f32 = 7.0;
const BARK_FRAMES: u32 = 180;
const BARK_AGAIN: u32 = 3600;

/// Now and then, people near the squad with something strong on their mind
/// say it (drawing only: barks change nothing in the world).
fn update_barks(game: &mut Game) {
    let f = game.frame;
    game.barks.retain(|b| b.2 > f);
    if f % 20 != 0 || game.world.talk.is_some() {
        return;
    }
    let w = &game.world;
    let Some(town) = w.settlements.iter().filter(|s| s.pos.dist(w.squad.pos) < 500.0).min_by(|a, b| a.pos.dist(w.squad.pos).total_cmp(&b.pos.dist(w.squad.pos))) else { return };
    for (k, &m) in w.squad.members.iter().enumerate() {
        let at = w.member_pos(k);
        for &p in &town.residents {
            if game.barks.len() >= 3 {
                return;
            }
            if w.people[p as usize].dead || game.barked.get(&p).is_some_and(|&t| f < t) || !w.is_about(p, w.time) || w.person_pos(p).dist(at) > BARK_NEAR {
                continue;
            }
            game.barked.insert(p, f + BARK_AGAIN);
            if let Some(line) = w.bark(p, m) {
                game.barks.push((p, line, f + BARK_FRAMES));
            }
        }
    }
}

/// Collects things under the mouse and keeps the closest.
struct Picker {
    mouse: Vec2,
    best: Option<(f32, Hover)>,
}

impl Picker {
    fn offer(&mut self, at: Vec2, slack: f32, h: Hover) {
        let d = (at - self.mouse).length() - slack;
        if d < 10.0 && self.best.as_ref().map(|(b, _)| d < *b).unwrap_or(true) {
            self.best = Some((d, h));
        }
    }
}

#[derive(Default)]
struct UiState {
    fonts: bool,
    loads: u32,
    relief: Option<egui::TextureHandle>,
    relief_version: u32,
}

/// The panels, labels, map and tooltips; clicks on panels become actions.
#[allow(clippy::too_many_arguments)]
fn ui(mut contexts: EguiContexts, mut game: ResMut<Game>, mut st: Local<UiState>, scene: Res<scene::Scene3d>, models: Res<models::Models>, foliage: Res<super::foliage::Foliage>, mut settings: ResMut<super::settings::Settings>, mut weather: ResMut<super::weather::WeatherView>) -> Result {
    let ctx = contexts.ctx_mut()?;
    if !st.fonts {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert("dejavu".into(), std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../../assets/DejaVuSans.ttf"))));
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "dejavu".into());
        ctx.set_fonts(fonts);
        st.fonts = true;
    }
    let game = &mut *game;
    if st.loads != game.loads {
        st.loads = game.loads;
        st.relief = None;
    }
    let size = game.screen;
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("canvas")));
    let c = Canvas { p: painter, w: size.x, h: size.y };
    let mut pick = Picker { mouse: game.mouse, best: None };
    let mut panels: Vec<Bx> = Vec::new();

    // ---- The view itself ----------------------------------------------------
    let view_name = match game.view {
        View::Map => {
            if st.relief.is_none() {
                st.relief = Some(map::load_relief(ctx, &game.world));
            }
            for (at, slack, h) in map::draw(&c, &game.map_cam, &game.world, game.rings, st.relief.as_ref().unwrap(), &game.sel) {
                pick.offer(at, slack, h);
            }
            super::weather::map_colours(&c, ctx, game, &mut weather);
            "map"
        }
        View::Scene => {
            let vp = game.orbit.view_proj(size);
            let eye = game.orbit.eye();
            for (p, slack, h) in &game.picks {
                if let Some(s) = game.orbit.project(&vp, size, *p) {
                    pick.offer(s, *slack, *h);
                }
            }
            for (p, vit, mana, down) in &game.bars {
                if let Some(s) = game.orbit.project(&vp, size, *p) {
                    hud::draw_bar(&c, s.x, s.y - 12.0, *vit, *mana, *down);
                }
            }
            for (p, name) in &game.labels {
                if let Some(q) = game.orbit.project(&vp, size, *p) {
                    if p.distance(eye) < game.orbit.draw_radius() * 1.1 {
                        c.centred(name, q.x, q.y, 17.0, super::palette::TEXT);
                    }
                }
            }
            // Townsfolk with something on their mind say so as the squad passes.
            update_barks(game);
            for (pid, line, _) in &game.barks {
                let at = game.world.person_pos(*pid);
                let g = scene.grid.height(&game.world.terrain, at);
                if let Some(q) = game.orbit.project(&vp, size, to3(at, g + 2.4)) {
                    let wd = c.width(line, 15.0) + 12.0;
                    c.rect(q.x - wd / 2.0, q.y - 18.0, wd, 22.0, hud::shadow(0.6));
                    c.centred(&format!("\u{201c}{line}\u{201d}"), q.x, q.y - 2.0, 15.0, super::palette::TEXT);
                }
            }
            if game.editor.on {
                let project = |p: Vec3| game.orbit.project(&vp, size, p);
                let grid = scene.grid;
                let terrain = &game.world.terrain;
                let drawn = |p: V2| grid.height(terrain, p);
                super::editor::draw_cursor(&c, game, &project, &drawn);
            }
            // Standing torches can be hovered too.
            for (i, s) in game.world.standing.iter().enumerate() {
                let g = scene.grid.height(&game.world.terrain, s.pos);
                if let Some(q) = game.orbit.project(&vp, size, to3(s.pos, g + 2.0)) {
                    pick.offer(q, 2.0, Hover::Torch(i));
                }
            }
            "3D"
        }
    };

    // ---- Panels ---------------------------------------------------------------
    if game.editor.on {
        if game.view == View::Map {
            if let Some(p) = game.editor.cursor {
                let s = game.map_cam.to_screen(size, p);
                c.circle_lines(s.x, s.y, game.editor.radius * game.map_cam.zoom, 2.0, super::palette::eg(super::palette::GOLD));
            }
        }
        let click = game.ui_click.take();
        let bx = super::editor::panel(&c, game, click);
        game.panels = vec![bx];
        // The map's picture follows the edits once a stroke is done.
        if !game.editor.stroking && st.relief_version != game.world.terrain.edits.version {
            st.relief_version = game.world.terrain.edits.version;
            st.relief = None;
        }
        return Ok(());
    }
    panels.push(Bx::from(hud::draw_hud(&c, &game.world, game.speed_i, game.paused, game.sim_ms, game.frame_ms, view_name)));
    let w = &mut game.world;
    if game.inv.map(|p| w.squad.index(p).is_none()).unwrap_or(false) {
        game.inv = None;
    }
    if game.craft.map(|p| w.squad.index(p).is_none()).unwrap_or(false) {
        game.craft = None;
    }
    let click = game.ui_click.take();
    let mut actions = Vec::new();
    let (a, boxes) = squadui::squad_bar(&c, w, &game.sel, click);
    actions.extend(a);
    panels.extend(boxes);
    let mut item_tip = None;
    if let Some(pid) = game.inv {
        let (a, h, bx) = squadui::inventory(&c, w, pid, game.mouse, click);
        actions.extend(a);
        item_tip = h;
        panels.extend(bx);
    }
    if let Some(pid) = game.craft {
        let (a, h, bx) = squadui::crafting(&c, w, pid, game.mouse, click);
        actions.extend(a);
        item_tip = item_tip.or(h);
        panels.push(bx);
    }
    if game.journal {
        panels.push(squadui::journal(&c, w));
    }
    let mut build_act = None;
    if game.build {
        let here = match game.view {
            View::Scene => game.orbit.target,
            View::Map => game.map_cam.centre,
        };
        let (a, bx) = super::baseui::build_panel(&c, w, here, game.placing.as_ref().map(|p| p.def), game.base_tab, game.mouse, click);
        build_act = a;
        panels.push(bx);
    }
    if let Some(t) = game.town {
        panels.push(super::townui::town_panel(&c, w, t));
        panels.push(super::townui::life_panel(&c, w, t, game.debug || game.town_all));
    }
    if game.book.map(|p| w.squad.index(p).is_none()).unwrap_or(false) {
        game.book = None;
    }
    let mut spell_tip = None;
    let mut book_left = game.mouse.x;
    if let Some(pid) = game.book {
        let (a, h, bx) = squadui::spell_book(&c, w, pid, game.mouse, click);
        actions.extend(a);
        spell_tip = h;
        book_left = bx.x;
        panels.push(bx);
    }
    panels.extend(super::hints::show(&c, &mut game.hints, w, click));
    let (loot_act, loot_box) = super::lootui::loot_panel(&c, w, game.mouse, click);
    panels.extend(loot_box);
    match loot_act {
        Some(super::lootui::LootAct::Take(m, body, what)) => {
            w.take_loot(m, body, what);
        }
        Some(super::lootui::LootAct::TakeAll(m, body)) => {
            w.take_all_loot(m, body);
        }
        Some(super::lootui::LootAct::Close(m)) => w.stop_looting(m),
        None => {}
    }
    if w.talk.is_some() {
        let (t, bx) = squadui::talk(&c, w, game.mouse, click);
        if let Some(t) = t {
            w.ask(t);
        }
        panels.extend(bx);
    }
    match build_act {
        Some(super::baseui::BuildAction::Pick(def)) => {
            game.placing = Some(super::baseui::Placing { def, ..Default::default() });
        }
        Some(super::baseui::BuildAction::Store(b)) => {
            let n = w.store_materials(b);
            game.notice = Some((if n == 0 { "Nobody here carries building materials.".to_string() } else { format!("{n} building materials stored.") }, std::time::Instant::now()));
        }
        Some(super::baseui::BuildAction::Deconstruct(b, id)) => {
            w.deconstruct(b, id);
        }
        Some(super::baseui::BuildAction::Close) => {
            game.build = false;
            game.placing = None;
        }
        Some(super::baseui::BuildAction::Tab(base)) => {
            game.base_tab = base;
            game.placing = None;
        }
        Some(super::baseui::BuildAction::Leave(m, b)) => {
            if let Err(e) = w.leave_at_base(m, b) {
                game.notice = Some((e.to_string(), std::time::Instant::now()));
            }
        }
        Some(super::baseui::BuildAction::PickUp(m)) => {
            if let Err(e) = w.pick_up(m) {
                game.notice = Some((e.to_string(), std::time::Instant::now()));
            }
        }
        Some(super::baseui::BuildAction::NextJob(m)) => {
            use gahturiyu_sim::sim::baselife::JOBS;
            if let Some((b, k)) = w.resident_of(m) {
                let now = w.base(b).unwrap().residents[k].job;
                let i = JOBS.iter().position(|&j| j == now).unwrap_or(0);
                w.set_base_job(m, JOBS[(i + 1) % JOBS.len()]);
            }
        }
        Some(super::baseui::BuildAction::NextRecipe(m)) => {
            let options = w.base_recipes(m);
            if let Some((b, k)) = w.resident_of(m) {
                let now = w.base(b).unwrap().residents[k].recipe;
                let next = match now.and_then(|r| options.iter().position(|&o| o == r)) {
                    Some(i) if i + 1 < options.len() => Some(options[i + 1]),
                    Some(_) => None,
                    None => options.first().copied(),
                };
                w.set_base_recipe(m, next);
            }
        }
        Some(super::baseui::BuildAction::Seal(b, id)) => {
            if let Err(e) = w.seal_building(b, id) {
                game.notice = Some((e.to_string(), std::time::Instant::now()));
            }
        }
        None => {}
    }
    for a in actions {
        match a {
            Action::Select(pid, add) => game.sel.pick(pid, add),
            Action::OpenInventory(pid) => {
                game.book = None;
                game.inv = if game.inv == Some(pid) { None } else { Some(pid) };
            }
            Action::CloseInventory => {
                game.inv = None;
                game.craft = None;
            }
            Action::Use(pid, it) => {
                w.use_item(pid, it);
            }
            Action::Craft(pid, r) => {
                let _ = w.start_craft(pid, r);
            }
            Action::EquipEntry(pid, k) => {
                w.equip_entry(pid, k);
            }
            Action::Unequip(pid, slot) => {
                w.unequip(pid, slot);
            }
            Action::Care(pid, slot) => {
                w.care_for(pid, slot);
            }
            Action::DropEntry(pid, k) => {
                w.drop_entry(pid, k);
            }
            Action::CloseBook => game.book = None,
            Action::Spell(pid, s) => {
                use gahturiyu_sim::sim::magic::{Aim, Style};
                let d = s.def();
                let held = match w.fighter(pid) {
                    Some(f) => f.held == Some(s),
                    None => w.held_ritual(pid) == Some(s),
                };
                // A ritual is begun where they stand; anything else aimed
                // waits for a click in the world.
                let performing = d.style == Style::Ritual && !held;
                if !performing && d.aim != Aim::Caster {
                    game.aim = Some((pid, s));
                } else if let Err(e) = w.use_spell(pid, s, None, None) {
                    game.notice = Some((format!("{}: {}", d.name, e.0), std::time::Instant::now()));
                }
            }
        }
    }
    if game.debug {
        panels.push(Bx::from(super::foliage::draw_readout(&c, &game.orbit, &scene, &foliage, &models)));
    }
    if game.options {
        let mut s = settings.clone();
        panels.push(super::settings::panel(&c, &mut s, game.mouse, click, game.frame_ms));
        if s != *settings {
            *settings = s;
        }
    }
    panels.extend(super::weather::panel(&c, game, &mut weather, click));

    // ---- Hover --------------------------------------------------------------
    let on_panels = panels.iter().any(|b| b.contains(game.mouse));
    game.hover = if on_panels { None } else { pick.best.map(|(_, h)| h) };
    if let (Some(p), false) = (&game.placing, on_panels) {
        super::baseui::placing_hint(&c, p, game.mouse);
    }
    if let Some(s) = spell_tip {
        let lines = squadui::spell_lines(s);
        let wd = lines.iter().map(|(l, _)| c.width(l, 15.0)).fold(0.0, f32::max) + 24.0;
        // Beside the book, not over it.
        c.panel(&lines, book_left - wd - 10.0, game.mouse.y, 15.0);
    } else if let Some((_, s)) = game.aim {
        use gahturiyu_sim::sim::magic::Aim;
        let what = match s.def().aim {
            Aim::Foe => "click an enemy",
            Aim::Friend => "click a friend (or themselves)",
            Aim::Anyone => "click someone",
            Aim::Door => "click a door",
            Aim::Corpse => "click by a body",
            _ => "click a spot",
        };
        let t = format!("{}: {what}  ·  right-click to cancel", s.def().name);
        c.text(&t, game.mouse.x + 18.0, game.mouse.y - 8.0, 15.0, squadui::RITUAL);
    }
    if let Some(it) = item_tip {
        let lines = squadui::item_lines(it);
        let wd = lines.iter().map(|(l, _)| c.width(l, 15.0)).fold(0.0, f32::max) + 24.0;
        c.panel(&lines, game.mouse.x - wd - 18.0, game.mouse.y, 15.0);
    } else if let Some(h) = game.hover {
        c.panel(&hud::describe(&game.world, h), game.mouse.x + 18.0, game.mouse.y + 12.0, 16.0);
    }
    super::animals::overlay(&c, game, &scene, &mut panels);
    let help = match game.view {
        View::Scene => "Click: move / attack / pick up / select   F1–F4: select (Shift adds)   `: all   Z: sneak   N: rest   T: torch   X: put down   I: pack   K: craft   M: spells   G: scout   J: journal   P: town   O: graphics   F8 / F9: save / load   Right-drag / Q E: turn   Middle / WASD: pan   Wheel: zoom   C: follow   Space: pause   1–5: speed   V: map   L: detail   B: build   F7: weather   F12: wildlife   F10: edit the land",
        View::Map => "Click: move / attack / pick up / select   F1–F4: select (Shift adds)   `: all   Z: sneak   N: rest   T: torch   X: put down   I: pack   K: craft   M: spells   G: scout   J: journal   P: town   O: graphics   F8 / F9: save / load   Right-drag / WASD: pan   Wheel: zoom   C: follow   Space: pause   1–5: speed   V: 3D   B: bandits",
    };
    if let Some((msg, at)) = &game.notice {
        if at.elapsed().as_secs_f32() < 3.0 || game.shot.is_some() {
            let wd = c.width(msg, 17.0) + 32.0;
            c.rect((size.x - wd) / 2.0, 70.0, wd, 34.0, hud::shadow(0.7));
            c.centred(msg, size.x / 2.0, 93.0, 17.0, super::palette::GOLD);
        }
    }
    c.rect(0.0, size.y - 30.0, size.x, 30.0, hud::shadow(0.45));
    // Shrink the help line to fit narrower windows.
    let fit = (15.0 * (size.x - 28.0) / c.width(help, 15.0)).clamp(10.0, 15.0);
    c.text(help, 14.0, size.y - 10.0, fit, super::palette::DIM);
    game.panels = panels;
    Ok(())
}

impl From<egui::Rect> for Bx {
    fn from(r: egui::Rect) -> Bx {
        Bx::new(r.min.x, r.min.y, r.width(), r.height())
    }
}

/// Save a screenshot at the asked-for frame, then quit once it's on disk.
fn screenshot(mut commands: Commands, mut game: ResMut<Game>, mut exit: MessageWriter<AppExit>) {
    game.frame += 1;
    let Some(s) = &game.shot else { return };
    let path = s.path.clone();
    let frames = s.frames;
    if game.frame == frames {
        let _ = std::fs::remove_file(&path);
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path.clone()));
        game.shot_at = Some(game.frame);
    }
    if let Some(at) = game.shot_at {
        let done = std::fs::metadata(&path).map(|m| m.len() > 0).unwrap_or(false);
        if (done && game.frame > at + 2) || game.frame > at + 600 {
            exit.write(AppExit::Success);
        }
    }
}
