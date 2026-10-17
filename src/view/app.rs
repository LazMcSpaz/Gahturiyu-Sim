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

/// How quickly a second click must follow the first to count as a double
/// click, in seconds.
const DOUBLE_CLICK: f32 = 0.4;

/// Every key, by view (the Keys button shows them).
pub const HELP_3D: &str = "Click: move / attack / pick up / select   Right-click: everything you can do   Drag: box-select   Alt: show names   F1–F4: select (Shift adds)   `: all   Z: sneak   N: get up   T: torch   X: put down   I: pack   K: craft   M: spells   G: scout   J: journal   P: town   O: graphics   F8 / F9: save / load   Right-drag / Q E: turn   Middle / WASD: pan   Wheel: zoom   C: follow (one picked: lock on)   Double-click a portrait: go to them   Space: pause   1–5: speed   Enter: deal / take all / make   Esc: close / game menu   V: map   L: detail   B: build   F10: edit the land";
pub const HELP_MAP: &str = "Click: move / attack / pick up / select   Right-click: everything you can do   Drag: box-select   Alt: show names   F1–F4: select (Shift adds)   `: all   Z: sneak   N: get up   T: torch   X: put down   I: pack   K: craft   M: spells   G: scout   J: journal   P: town   O: graphics   F8 / F9: save / load   Right-drag / WASD: pan   Wheel: zoom   C: follow (one picked: lock on)   Double-click a portrait: go to them   Space: pause   1–5: speed   Enter: deal / take all / make   Esc: close / game menu   V: 3D   B: bandits";

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
    /// A ruin or lair.
    Ruin(u32),
    /// A building (anywhere on it, not just its door).
    Building((u16, u16)),
    /// A piece of furniture in a building: (building, its place in the layout).
    Furniture((u16, u16), u8),
    /// A town's workplace: (town, its place in the town's list).
    Place(u16, u16),
    /// A chest, crate, cupboard or barrel in a building.
    Container(gahturiyu_sim::sim::containers::ContainerId),
    /// A bedroll laid out or a tent pitched by the squad (`Bedding::id`).
    Bedding(u32),
    /// A campfire the squad laid.
    Campfire(u32),
}

/// Everything the window keeps between frames.
#[derive(Resource)]
pub struct Game {
    pub world: World,
    pub view: View,
    pub orbit: OrbitCam,
    pub map_cam: MapCam,
    pub follow: bool,
    /// While following: the squad member the camera is locked on to, or the
    /// squad as a whole (None). Laz's P7 and P8.
    pub follow_who: Option<PersonId>,
    /// The last portrait clicked and when (twice quickly centres on them).
    pub last_pick: Option<(PersonId, std::time::Instant)>,
    pub speed_i: usize,
    pub paused: bool,
    pub rings: bool,
    /// The detail-level readout (L).
    pub debug: bool,
    /// The test quarter's dev spawner, while it's open (Home, with L on).
    pub dev: Option<super::testquarter::DevPanel>,
    pub sel: Selection,
    pub inv: Option<PersonId>,
    pub craft: Option<PersonId>,
    /// The making screen, while whoever `craft` follows stands at a bench
    /// (`makeui.rs`): what's picked on it.
    pub making: Option<super::makeui::Making>,
    pub journal: bool,
    /// The town panel (P, or click a town's name).
    pub town: Option<u16>,
    /// The graphics settings panel (O).
    pub options: bool,
    /// Whose spell book is open (M).
    pub book: Option<PersonId>,
    /// A spell waiting for its target: the next click in the world aims it.
    pub aim: Option<(PersonId, gahturiyu_sim::sim::magic::Spell)>,
    /// The scroll being aimed, when `aim` is a scroll's spell rather than a cast.
    pub aim_scroll: Option<items::ItemId>,
    /// The trade screen, while a merchant's wares are out (`tradeui.rs`):
    /// what's on the table; when its sums were last worked out; and the
    /// last line clicked (twice quickly trades that one thing at once).
    pub trading: Option<super::tradeui::Trading>,
    pub trade_quoted: Option<std::time::Instant>,
    pub trade_last: Option<(std::time::Instant, bool, items::ItemId)>,
    /// Space was pressed at the table (Deal).
    pub space_deal: bool,
    /// Space with a loot box open: Take all.
    pub space_take: bool,
    /// The loot box shows the pack, to put things in (a chest).
    pub loot_put: bool,
    /// The pack screen's own memory (sort, scroll, a drag, its menu).
    pub pack: super::packui::PackState,
    /// The left button is down this frame.
    pub left_held: bool,
    /// The talk screen (`talkui.rs`): what's been asked in this talk; their
    /// parting words, held a moment after it; a number key just pressed;
    /// and where the camera was before it came in close.
    pub talking: Option<super::talkui::Talking>,
    pub parting: Option<super::talkui::Parting>,
    pub talk_key: Option<usize>,
    pub talk_cam: Option<super::talkui::TalkCam>,
    /// The small answers to a click (`feel.rs`): marks on the ground, nods,
    /// the rings' breath; and where the camera is gliding to.
    pub feel: super::feel::Feel,
    /// Panels arriving and going (`slide.rs`).
    pub slides: super::slide::Slides,
    pub glide: Option<super::feel::Glide>,
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
    /// The squad list folded to its portraits.
    pub squad_collapsed: bool,
    /// The keys panel open.
    pub keys: bool,
    /// Big news shown across the top: (what, since).
    pub banners: Vec<(String, std::time::Instant)>,
    /// The right-click menu, while open.
    pub menu: Option<super::interact::Menu>,
    /// A thing pinned with Examine: its long description stays up.
    pub examine: Option<Hover>,
    /// Where the right button went down (a click, not a drag, opens the menu).
    pub rpress_at: Option<Vec2>,
    /// Words floating over heads (picked up, hurt, spotted, better at).
    pub floaters: super::floaters::Floaters,
    /// Where each squad member was on screen last frame.
    pub squad_screen: Vec<(PersonId, Vec2)>,
    /// A box being dragged out with the left button, from here to the mouse.
    pub drag: Option<Vec2>,
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
    /// Filled by the 3D scene each frame: the floor height (top) of each
    /// building the squad is in, so a click indoors lands on the floor.
    pub floors: Vec<(gahturiyu_sim::sim::buildings::DoorId, f32)>,
    /// How each figure drawn is walking and facing (UI-9; drawing only).
    pub gaits: super::gait::Gaits,
    /// What the mouse rests on and since when (the long tooltip waits, UI-4).
    /// (And where the mouse was: moving it more than `pointer::STILL` px
    /// starts the wait again, P5.)
    pub resting: Option<(Hover, std::time::Instant, Vec2)>,
    pub labels: Vec<(Vec3, String)>,
    pub bars: Vec<(Vec3, f32, Option<f32>, bool)>,
    /// Remarks townsfolk make as the squad goes by: (who, what, until frame),
    /// and when each may speak up again.
    pub barks: Vec<(PersonId, String, u32)>,
    /// What the squad has noticed and said (`remarks.rs`).
    pub remarks: super::remarks::Remarks,
    /// The last walk that can't get all the way (UI-7): the spot clicked,
    /// who was sent, and when. A grey cross marks the spot while they go.
    pub short: Option<(V2, Vec<PersonId>, std::time::Instant)>,
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
    /// Draw every building near the camera cut open (screenshots only:
    /// `GAHT_CUTAWAY`).
    pub cutaway: bool,
    /// Sounds asked for this frame (`sound.rs` plays them).
    pub sounds: super::sound::Queue,
    /// Whose stats sheet is open.
    pub stats: Option<PersonId>,
    /// The game menu (Escape with nothing open); the world waits while it's up.
    pub game_menu: Option<super::screens::GameMenu>,
    /// The title screen (`title.rs`): up at launch, and from the game menu.
    /// The world stands still under it.
    pub title: Option<super::title::Title>,
    /// Which screen is open (`screens.rs`: one at a time).
    pub screens: super::screens::Screens,
    /// Spells cast outside fights, being shown (`castfx.rs`).
    pub castfx: super::castfx::CastFx,
    /// Quit chosen in the game menu (the window closes at the end of the frame).
    pub quit: bool,
}

/// A new world for `seed`: the forged town (assets/towns/demo) and the
/// seed's map from the land editor, unless `own_land` asks for the seed's
/// own land (screenshots).
/// The title screen at launch: up for a real run, down for screenshots and
/// a `GAHT_LOAD` (which start straight in), unless `GAHT_TITLE=1|arena|load`
/// asks for a page of it (`game`: as from the game menu, a game under way).
fn title_at_launch(shot: bool) -> Option<super::title::Title> {
    use super::title::{Page, Title};
    match std::env::var("GAHT_TITLE").as_deref() {
        Ok("arena") => return Some(Title::on(Page::Arena)),
        Ok("load") => return Some(Title::on(Page::Load)),
        Ok("game") => return Some(Title::from_game()),
        Ok(_) => return Some(Title::new()),
        Err(_) => {}
    }
    if shot || std::env::var("GAHT_LOAD").is_ok() {
        None
    } else {
        Some(Title::new())
    }
}

/// Seconds since the epoch: a seed for a new world.
fn seed_now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(1)
}

/// Set a fresh town-start world up for one of the testing arena's scenes.
fn arena_setup(w: &mut World, a: super::title::Arena) {
    use super::title::Arena;
    match a {
        Arena::TestArea => {
            w.enter_test_quarter();
        }
        // The squad on the arena's west side; foes are picked in the dev
        // panel (opened on People) and come in on the east.
        Arena::Combat => {
            w.dev_to_arena();
        }
        Arena::Building => {
            super::shot::demo_base(w, true);
            if let Some(bid) = w.bases.first().map(|b| b.id) {
                w.dev_stock_base(bid);
            }
        }
    }
    w.step(0.001);
}

/// How a new game begins: `GAHT_START=town|island`, else the opening on
/// Salt Isle, except for screenshots (their scenes want the squad of four).
fn start_kind(shot: bool) -> worldgen::Start {
    match std::env::var("GAHT_START").as_deref() {
        Ok("town") => worldgen::Start::Town,
        Ok("island") => worldgen::Start::Island,
        _ if shot => worldgen::Start::Town,
        _ => worldgen::Start::Island,
    }
}

fn fresh_world(seed: u64, own_land: bool, start: worldgen::Start) -> World {
    let forge = match gahturiyu_sim::sim::forge::load(&forge_dir(), seed) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("couldn't read the forged town: {e}");
            None
        }
    };
    let path = map_path(seed);
    let edits = if own_land {
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
    match start {
        worldgen::Start::Town => worldgen::generate_authored(seed, forge, edits),
        worldgen::Start::Island => worldgen::generate_opening(seed, forge, edits),
    }
}

pub fn run() {
    super::prof::init();
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let shot = Shot::from_env();
    // GAHT_MAPRENDER=dir: the map site's close-up pictures (`maprender.rs`).
    let maprender = super::maprender::MapRender::from_env();
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
        // The map render shows the land with no places laid on it (`grounds::world_for`).
        None if maprender.is_some() => gahturiyu_sim::sim::grounds::world_for(seed),
        None => fresh_world(seed, shot.is_some() && std::env::var("GAHT_MAP").is_err(), start_kind(shot.is_some())),
    };
    if let Some(s) = &shot {
        s.prepare(&mut world);
    }
    let mut game = Game {
        view: View::Scene,
        map_cam: MapCam { centre: world.squad.pos, zoom: 1600.0 / 7000.0 },
        orbit: OrbitCam::new(world.squad.pos),
        follow: true,
        follow_who: None,
        last_pick: None,
        speed_i: 0,
        paused: false,
        rings: true,
        debug: false,
        sel: Selection::default(),
        inv: None,
        craft: None,
        making: None,
        journal: false,
        town: None,
        options: std::env::var("GAHT_SETTINGS").is_ok(),
        book: None,
        aim: None,
        aim_scroll: None,
        trading: None,
        trade_quoted: None,
        trade_last: None,
        space_deal: false,
        space_take: false,
        loot_put: false,
        pack: Default::default(),
        left_held: false,
        talking: None,
        parting: None,
        talk_key: None,
        talk_cam: None,
        feel: Default::default(),
        slides: Default::default(),
        glide: None,
        loads: 0,
        notice: None,
        wild: None,
        squad_collapsed: false,
        keys: false,
        banners: Vec::new(),
        menu: None,
        examine: None,
        rpress_at: None,
        floaters: Default::default(),
        squad_screen: Vec::new(),
        drag: None,
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
        floors: Vec::new(),
        gaits: Default::default(),
        resting: None,
        labels: Vec::new(),
        bars: Vec::new(),
        barks: Vec::new(),
        remarks: Default::default(),
        short: None,
        town_all: false,
        editor: Default::default(),
        dev: None,
        barked: std::collections::HashMap::new(),
        build: false,
        base_tab: false,
        placing: None,
        cutaway: false,
        sounds: Default::default(),
        stats: None,
        game_menu: None,
        title: title_at_launch(shot.is_some() || maprender.is_some()),
        screens: Default::default(),
        quit: false,
        castfx: Default::default(),
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
        if s.make.as_deref().is_some_and(|m| !m.ends_with(",closed")) {
            game.craft = game.world.squad.members.first().copied();
        }
        if s.trade_table {
            // Two of the first thing the merchant would take, and one each
            // of their first two wares.
            let w = &game.world;
            if let Some(cv) = w.talk.as_ref() {
                let mut table = gahturiyu_sim::sim::economy::Table::default();
                if let Some(e) = w.people[cv.with as usize].detail.as_ref().and_then(|d| d.gear.bag.iter().find(|e| w.offer(cv.npc, e.0, e.2.as_ref()).is_some())) {
                    table.give.push((cv.with, e.0, e.1.min(2), e.2.and_then(|p| p.mix)));
                }
                for l in w.stall(cv.npc).iter().take(2) {
                    table.get.push((l.item, 1, l.mix));
                }
                game.trading = Some(super::tradeui::Trading { table, ..Default::default() });
            }
        }
        // `GAHT_QUARTER=<tab>`: the test quarter's spawner on that tab.
        if let Some(tab) = s.quarter.as_deref().filter(|t| *t != "1") {
            use super::testquarter::{DevPanel, Tab};
            let mut dp = DevPanel { tab: match tab {
                "creatures" | "animals" => Tab::Creatures,
                "people" | "arena" => Tab::People,
                "world" | "time" => Tab::World,
                "weather" => Tab::Weather,
                "fire" => Tab::Fire,
                "squad" => Tab::Squad,
                "rules" => Tab::Rules,
                "go" => Tab::Go,
                "yard" => Tab::Yard,
                _ => Tab::Items,
            }, ..Default::default() };
            if dp.tab == Tab::Yard {
                dp.row = gahturiyu_sim::sim::duelgrid::weapons().first().map(|w| gahturiyu_sim::sim::duelgrid::row(w));
            }
            game.dev = Some(dp);
        }
        if s.trade_bottles {
            // One of the bottles on the table, and the stall scrolled down to them.
            let w = &game.world;
            if let Some(cv) = w.talk.as_ref() {
                let mut table = gahturiyu_sim::sim::economy::Table::default();
                if let Some(l) = w.stall(cv.npc).into_iter().find(|l| l.mix.is_some()) {
                    table.get.push((l.item, 1, l.mix));
                }
                game.trading = Some(super::tradeui::Trading { table, scroll: [0.0, 10_000.0], ..Default::default() });
            }
        }
        game.book = s.book.and_then(|k| game.world.squad.members.get(k).copied());
        // `GAHT_STATS=k`: member k's sheet. `GAHT_GAMEMENU=1`: the game menu
        // (`quit` or `new` for its question).
        game.stats = std::env::var("GAHT_STATS").ok().and_then(|v| v.parse::<usize>().ok()).and_then(|k| game.world.squad.members.get(k).copied());
        if let Ok(v) = std::env::var("GAHT_GAMEMENU") {
            use super::screens::{GameMenu, MenuAct};
            let confirm = match v.as_str() {
                "quit" => Some(MenuAct::Quit),
                "new" => Some(MenuAct::NewGame),
                "skip" => Some(MenuAct::SkipIsland),
                _ => None,
            };
            game.game_menu = Some(GameMenu { confirm });
        }
        if s.feud {
            game.town_all = true;
        }
        if s.guard || s.factions {
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
        // `GAHT_EDITBUILD=1`: the editor's Build tool open at the nearest mark,
        // a few things put down there and a weaver's house in hand (`build::demo`).
        if std::env::var("GAHT_EDITBUILD").is_ok() {
            game.editor.on = true;
            game.paused = true;
            let c = super::build::demo(&mut game);
            game.follow = false;
            game.orbit.target = c;
            game.orbit.ground = game.world.terrain.surface(c);
            game.orbit.dist = 75.0;
            game.orbit.pitch = 0.75;
        }
        if let Some(what) = &s.build {
            game.build = what != "store";
            if what == "base" || what == "store" {
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
        if s.bye {
            // (Held: a screenshot takes longer to draw than the words stay up.)
            if let Some((with, npc)) = game.world.talk.as_ref().map(|c| (c.with, c.npc)) {
                if let Some(line) = game.world.ask(gahturiyu_sim::sim::dialogue::Topic::Goodbye) {
                    game.parting = Some(super::talkui::Parting { with, npc, line, since: std::time::Instant::now() + std::time::Duration::from_secs(3600) });
                }
            }
        }
        if let Some(t) = std::env::var("GAHT_SLIDE").ok().and_then(|v| v.parse().ok()) {
            game.slides.hold(t);
        }
        if std::env::var("GAHT_MARK").is_ok() {
            let at = game.world.squad.pos;
            game.feel.show(at);
        }
        if let Some(p) = *super::shot::WAY_SHORT.lock().unwrap() {
            let who: Vec<PersonId> = game.world.squad.members.iter().copied().take(2).collect();
            note_short(&mut game, &who, p);
        }
        // Interiors and the variants row frame themselves.
        game.cutaway = s.cutaway;
        if let Some((at, dist, pitch, yaw)) = *s.framing.lock().unwrap() {
            game.follow = false;
            game.orbit.target = at;
            game.orbit.ground = game.world.terrain.surface(at);
            if s.zoom.is_none() {
                game.orbit.dist = dist;
            }
            if s.pitch.is_none() {
                game.orbit.pitch = pitch;
            }
            if let (None, Some(y)) = (s.yaw, yaw) {
                game.orbit.yaw = y;
            }
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
    let synchronous = shot.is_some() || maprender.is_some();
    game.shot = shot;
    let plugins = DefaultPlugins
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
        .set(bevy::render::RenderPlugin { synchronous_pipeline_compilation: synchronous, ..default() });
    // Screenshots are silent: no sound device is even opened.
    let plugins = if synchronous { plugins.disable::<bevy::audio::AudioPlugin>() } else { plugins };

    let mut app = App::new();
    if let Some(m) = maprender {
        app.insert_resource(m);
    }
    app.add_plugins(plugins)
        .insert_resource(EguiGlobalSettings { auto_create_primary_context: false, ..default() })
        .add_plugins(EguiPlugin::default())
        // New letters reach the screen the frame they're first drawn: the
        // panels' letter sheet is updated before Bevy gathers the frame's
        // changed pictures, not after (or a letter never drawn before is a
        // gap for one frame: "06:0", "s r ng").
        .configure_sets(PostUpdate, bevy_egui::EguiPostUpdateSet::PostProcessOutput.before(bevy::asset::AssetEventSystems))
        .add_plugins(super::foliage::FoliagePlugin)
        .add_plugins(super::weather::WeatherPlugin)
        .add_plugins(super::outline::OutlinePlugin)
        .insert_resource(game)
        .insert_resource(super::settings::Settings::load())
        .init_resource::<scene::Scene3d>()
        .init_resource::<models::Models>()
        .init_resource::<super::foliage::Foliage>()
        .init_resource::<super::sound::Sfx>()
        .insert_resource(ClearColor(Color::srgb(0.63, 0.69, 0.72)))
        .add_systems(Startup, (setup, scene::setup, light::setup, models::start, super::foliage::setup))
        .add_systems(Update, (input, simulate, camera.run_if(super::maprender::off), super::maprender::step, light::update.run_if(super::maprender::off), models::finish, scene::update.run_if(super::maprender::off), super::foliage::update.run_if(super::maprender::off), super::sound::update.run_if(super::maprender::off), screenshot).chain())
        .add_systems(EguiPrimaryContextPass, ui.run_if(super::maprender::off))
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
        // The scene's depth, for the outlines to tell what's behind a wall
        // (`outline.rs`).
        bevy::core_pipeline::prepass::DepthPrepass,
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
    // The authored world's land, the same for every seed (Laz, 14 Oct); a
    // seed's own map file only if the world folder isn't there.
    if let Some(p) = gahturiyu_sim::sim::mapedit::MapEdits::world_path() {
        return p;
    }
    let assets = models::assets_dir();
    assets.parent().map(|p| p.to_path_buf()).unwrap_or_default().join(gahturiyu_sim::sim::mapedit::MapEdits::path_for(seed))
}

pub fn quick_save() -> std::path::PathBuf {
    let assets = models::assets_dir();
    assets.parent().map(|p| p.to_path_buf()).unwrap_or_default().join("saves").join("quick.sav")
}

/// The town panel: open for the town the squad is in or beside, or say
/// there's none. Out in the wilds it doesn't show a town far away (Laz's
/// P15). (A town far off can still be opened from the right-click menu or
/// the map.)
fn toggle_town(game: &mut Game) {
    if game.town.is_some() {
        game.town = None;
        return;
    }
    game.town = game.world.town_at(game.world.squad.pos);
    if game.town.is_none() {
        game.notice = Some(("No town here: you're out in the wilds.".to_string(), std::time::Instant::now()));
    }
}

fn quick_save_now(game: &mut Game) {
    let msg = match game.world.save_to(&quick_save()) {
        Ok(()) => {
            // And a copy named for the world's day and hour, for the title
            // screen's Load list (`title.rs`).
            let w = &game.world;
            let hour = (w.time / gahturiyu_sim::sim::world::HOUR).rem_euclid(24.0) as u32;
            let name = super::title::dated_name(World::day_of(w.time), hour, w.seed);
            let _ = w.save_to(&super::title::saves_dir().join(format!("{name}.sav")));
            format!("Saved ({name}).")
        }
        Err(e) => format!("Couldn't save: {e}"),
    };
    game.notice = Some((msg, std::time::Instant::now()));
}

fn quick_load_now(game: &mut Game) {
    let msg = match World::load_from(&quick_save()) {
        Ok(loaded) => {
            swap_world(game, loaded);
            "Loaded.".to_string()
        }
        Err(gahturiyu_sim::sim::save::LoadError::Io(_)) => "No save yet (F8 saves).".to_string(),
        Err(e) => format!("Couldn't load: {e}"),
    };
    game.notice = Some((msg, std::time::Instant::now()));
}

/// Put a loaded world in place of the current one, and forget anything the
/// window was holding about the old one.
fn swap_world(game: &mut Game, w: World) {
    game.world = w;
    game.loads += 1;
    game.sel = Selection::default();
    game.follow_who = None;
    game.inv = None;
    game.craft = None;
    game.book = None;
    game.stats = None;
    game.town = None;
    game.journal = false;
    game.talking = None;
    game.parting = None;
    game.trading = None;
    game.aim = None;
    game.menu = None;
    game.examine = None;
    game.hover = None;
    game.build = false;
    game.placing = None;
    game.follow = true;
    game.orbit.target = game.world.squad.pos;
    game.orbit.ground = game.world.terrain.surface(game.world.squad.pos);
    game.map_cam.centre = game.world.squad.pos;
}

/// The editor's "Rebuild the world": a new world from the saved places and
/// the land as edited, the camera left where it was.
pub(super) fn rebuild_world(game: &mut Game) {
    let (target, yaw, pitch, dist) = (game.orbit.target, game.orbit.yaw, game.orbit.pitch, game.orbit.dist);
    let w = fresh_world(game.world.seed, false, worldgen::Start::Town);
    swap_world(game, w);
    game.follow = false;
    (game.orbit.target, game.orbit.yaw, game.orbit.pitch, game.orbit.dist) = (target, yaw, pitch, dist);
    game.orbit.ground = game.world.terrain.surface(target);
    // (Reopened from their files, against the new world's towns.)
    game.editor.build.places.clear();
}

/// Keys and mouse: camera moves, and orders.
/// The scripted brush stroke for measuring the editor (`prof.rs`, `GAHT_STROKE`).
#[derive(Default)]
pub(super) struct StrokeScript {
    start: Option<f32>,
    held: ButtonInput<MouseButton>,
    /// When the wheel last turned, in the camera part.
    wheel: f32,
}

#[allow(clippy::too_many_arguments)]
fn input(mut game: ResMut<Game>, keys: Res<ButtonInput<KeyCode>>, buttons: Res<ButtonInput<MouseButton>>, scroll: Res<AccumulatedMouseScroll>, window: Single<&Window, With<PrimaryWindow>>, time: Res<Time>, mut stroke: Local<StrokeScript>) {
    let _prof = super::prof::span("input");
    let game = &mut *game;
    game.screen = Vec2::new(window.width(), window.height());
    let mut mouse = window.cursor_position().unwrap_or(game.last_mouse);
    // `GAHT_STROKE`: a few seconds of play, one with the editor open, four
    // moving the camera about as one does to look at the land (tilting,
    // turning, zooming in and out), then a Raise stroke held for that long,
    // the brush circling the screen's middle.
    let real_buttons = &*buttons;
    let buttons: &ButtonInput<MouseButton> = match super::prof::stroke_secs() {
        Some(secs) => {
            let t = time.elapsed_secs();
            let start = *stroke.start.get_or_insert(t + 3.0);
            if t >= start && !game.editor.on {
                super::editor::toggle(game);
                game.editor.tool = super::editor::Tool::Raise;
            }
            stroke.held.clear();
            let (cam, brush) = (start + 1.0, start + 5.0);
            let done = brush + secs;
            super::prof::phase(if t < start { "play" } else if t < cam { "editor" } else if t < brush { "camera" } else if t < done { "stroke" } else { "after" });
            if t >= cam && t < brush {
                // Tilt up and down, turning, and the wheel in, then out.
                let dt = time.delta_secs();
                let u = t - cam;
                game.orbit.orbit(120.0 * dt, (u * 2.5).cos() * 220.0 * dt);
                if t - stroke.wheel > 0.15 {
                    stroke.wheel = t;
                    game.glide.get_or_insert(super::feel::Glide::of(&game.orbit)).zoom(if u < 2.0 { 1.0 } else { -1.0 });
                }
            }
            if t >= brush && t < brush + secs {
                let a = (t - start) * 1.5;
                mouse = game.screen * 0.5 + Vec2::new(a.cos(), a.sin()) * 120.0;
                stroke.held.press(MouseButton::Left);
            } else if t >= done {
                stroke.held.release(MouseButton::Left);
                game.quit = t >= done + 2.0;
            }
            &stroke.held
        }
        None => real_buttons,
    };
    if let Some(h) = game.shot.as_ref().and_then(|s| s.hover) {
        mouse = h;
    }
    if let Some(d) = game.shot.as_ref().and_then(|s| s.drag) {
        game.drag = Some(d);
    }
    game.mouse = mouse;
    super::kit::input(mouse, buttons.just_pressed(MouseButton::Left), buttons.just_released(MouseButton::Left), time.elapsed_secs());
    let dt = time.delta_secs();
    // (Screenshots step at a fixed pace, so a mark or a nod is the same in each.)
    game.feel.tick(if game.shot.is_some() { 1.0 / 30.0 } else { dt.min(0.1) });
    // (Escape in the Build tool lets go of what's in hand before it leaves.)
    let esc_leaves = game.editor.on && keys.just_pressed(KeyCode::Escape) && !game.editor.build.escape();
    if (keys.just_pressed(KeyCode::F10) && game.title.is_none()) || esc_leaves {
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
    // Under the title screen the world's keys wait (nothing is saved, opened
    // or spawned in the world behind it): Escape goes back a page or closes
    // the settings, O opens them, and a click is the title's.
    if game.title.is_some() {
        if keys.just_pressed(KeyCode::Escape) {
            super::screens::escape(game);
        }
        if keys.just_pressed(KeyCode::KeyO) {
            game.options = !game.options;
        }
        let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        game.ui_click = None;
        game.left_held = buttons.pressed(MouseButton::Left);
        if buttons.just_pressed(MouseButton::Left) {
            game.press_at = Some(mouse);
        }
        if buttons.just_released(MouseButton::Left) && game.press_at.take().is_some_and(|p| (p - mouse).length() < 6.0) {
            game.ui_click = Some(Click { at: mouse, right: false, shift });
        }
        game.last_mouse = mouse;
        return;
    }
    let w = &mut game.world;

    // In a talk the number keys answer (time stands still there anyway);
    // otherwise 1 to 5 set the pace.
    let answering = w.talk.as_ref().is_some_and(|c| !c.trading || c.refused);
    for (i, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9].iter().enumerate() {
        if keys.just_pressed(*key) {
            if answering {
                game.talk_key = Some(i + 1);
            } else if i < 5 {
                game.speed_i = i;
                game.paused = false;
            }
        }
    }
    // Space only ever controls time (Laz's P11). (Not while the game menu
    // is up: the world waits for it anyway.)
    if keys.just_pressed(KeyCode::Space) && game.game_menu.is_none() && game.title.is_none() {
        game.paused = !game.paused;
    }
    // Enter does what the open screen is for: Deal at the trade table (taken
    // up where the screen is drawn), Take all when going through something,
    // Make at a bench (below).
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        if game.trading.is_some() {
            game.space_deal = true;
        } else if super::lootui::open_loot(w).is_some() {
            game.space_take = true;
        }
    }
    // C: follow. With one member picked out, the camera locks on to them;
    // otherwise it follows the squad.
    if keys.just_pressed(KeyCode::KeyC) || keys.just_pressed(KeyCode::KeyF) {
        game.follow = true;
        game.follow_who = match game.sel.0.as_slice() {
            [one] if w.squad.index(*one).is_some() => Some(*one),
            _ => None,
        };
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
    if keys.just_pressed(KeyCode::Escape) && (game.menu.is_some() || game.examine.is_some()) {
        game.menu = None;
        game.examine = None;
    } else if keys.just_pressed(KeyCode::Escape) && game.placing.is_some() {
        // Finish a wall being drawn, or stop placing.
        match game.placing.as_mut() {
            Some(p) if !p.chain.is_empty() => p.chain.clear(),
            _ => game.placing = None,
        }
    } else if keys.just_pressed(KeyCode::Escape) && game.aim.is_some() {
        game.aim = None;
        game.aim_scroll = None;
    } else if keys.just_pressed(KeyCode::Escape) && w.talk.as_ref().is_some_and(|c| c.trading) {
        // Away from the table, back to the talk.
        w.stop_trading();
    } else if keys.just_pressed(KeyCode::Escape) && w.talk.is_some() {
        w.end_talk();
    } else if keys.just_pressed(KeyCode::Escape) && game.parting.is_some() {
        game.parting = None;
    } else if keys.just_pressed(KeyCode::Escape) {
        // Close whatever screen is open, or bring up the game menu.
        super::screens::escape(game);
    }
    let w = &mut game.world;
    // `: everyone.
    if keys.just_pressed(KeyCode::Backquote) {
        game.sel = Selection::default();
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
    // N: get up (any of the selected lying down). Lying down is done by
    // clicking a bed, a bedroll or a tent (Laz, 11 Oct): there's no rest key.
    if keys.just_pressed(KeyCode::KeyN) {
        let who = game.sel.who(w);
        if w.any_resting(&who) {
            w.order_wake(&who);
        }
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
    // P: the town panel for the town the squad is in.
    if keys.just_pressed(KeyCode::KeyP) {
        toggle_town(game);
    }
    // Home, with the detail switch on: the hidden test quarter (dev only,
    // `testquarter.rs`). Outside it: open it and go in; inside: the spawner.
    if keys.just_pressed(KeyCode::Home) && game.debug {
        let inside = game.world.test_quarter().is_some_and(|q| q.open && q.contains(game.world.squad.pos));
        if inside {
            game.dev = if game.dev.is_some() { None } else { Some(Default::default()) };
        } else if game.world.enter_test_quarter() {
            game.notice = Some(("The test quarter (dev only). Home again for the spawner.".to_string(), std::time::Instant::now()));
        } else {
            game.notice = Some(("No flat ground for the test quarter near here.".to_string(), std::time::Instant::now()));
        }
    }
    // Insert, anywhere: the dev panel, open while the game goes on (Laz, 13 Oct).
    if keys.just_pressed(KeyCode::Insert) {
        game.dev = if game.dev.is_some() { None } else { Some(Default::default()) };
    }
    let w = &mut game.world;
    if keys.just_pressed(KeyCode::KeyO) {
        game.options = !game.options;
    }
    if keys.just_pressed(KeyCode::F8) {
        quick_save_now(game);
        return;
    }
    if keys.just_pressed(KeyCode::F9) {
        quick_load_now(game);
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
    // At a bench's screen: Enter makes what's shown, by whoever is shown.
    if (keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter)) && game.trading.is_none() {
        if let Some(m) = game.making.clone().filter(|m| m.bench.station == gahturiyu_sim::sim::crafting::Station::AlchemyTable) {
            // At an alchemy table: Enter brews the mix shown.
            match super::brewui::brewer(w, &m).map(|p| w.brew(p, &m.mix, m.n.clamp(1, super::makeui::MOST))) {
                Some(Ok(())) => game.sounds.ui("ui_press"),
                _ => game.sounds.ui("ui_cant"),
            }
        } else if let Some(m) = game.making.clone() {
            let rows = w.making_list(&m.bench);
            if let Some(ri) = super::makeui::shown(&rows, m.pick) {
                let who = super::makeui::maker(&w.makers(&m.bench, ri), m.who);
                match who.map(|p| w.make(p, ri, m.n.clamp(1, super::makeui::MOST))) {
                    Some(Ok(())) => game.sounds.ui("ui_press"),
                    _ => game.sounds.ui("ui_cant"),
                }
            }
        }
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
    // The wheel over the making screen runs its list up and down.
    if on_panels && scroll.delta.y != 0.0 {
        if let Some(m) = game.making.as_mut() {
            m.scroll = (m.scroll - scroll.delta.y * 46.0).max(0.0);
        }
    }
    // The wheel at the trade table runs the list under the mouse (the pack
    // on the left, the stall on the right); in a talk, the replies.
    // The wheel over the pack scrolls the column under the mouse.
    if on_panels && scroll.delta.y != 0.0 && game.inv.is_some() {
        game.pack.wheel += scroll.delta.y.signum() * 3.0;
    }
    if scroll.delta.y != 0.0 {
        if let Some(t) = game.trading.as_mut() {
            let k = if mouse.x < window.width() / 2.0 { 0 } else { 1 };
            t.scroll[k] = (t.scroll[k] - scroll.delta.y * 32.0).max(0.0);
        } else if let Some(t) = game.talking.as_mut() {
            t.scroll = (t.scroll - scroll.delta.y * 34.0).max(0.0);
        }
    }
    // Clicks on the panels are handled when they're drawn; a short click
    // anywhere else is an order.
    game.ui_click = None;
    game.left_held = buttons.pressed(MouseButton::Left);
    if buttons.just_pressed(MouseButton::Left) {
        game.press_at = Some(mouse);
    }
    if buttons.just_pressed(MouseButton::Right) && on_panels {
        game.ui_click = Some(Click { at: mouse, right: true, shift });
    }
    // Right-click anywhere drops a spell that's waiting to be aimed.
    if buttons.just_pressed(MouseButton::Right) && game.aim.is_some() {
        game.aim = None;
        game.aim_scroll = None;
    } else if buttons.just_pressed(MouseButton::Right) && !on_panels {
        game.rpress_at = Some(mouse);
    }
    // A right-click (not a drag, which turns the camera) opens the menu of
    // everything that can be done with what's under the mouse.
    if buttons.just_released(MouseButton::Right) {
        if let Some(p) = game.rpress_at.take() {
            if (p - mouse).length() < 6.0 && !on_panels {
                let ground = point_under(game, mouse);
                game.menu = Some(super::interact::Menu { at: mouse, hover: game.hover, wild: if game.hover.is_none() { game.wild } else { None }, ground });
            }
        }
    }
    // A left drag that began off the panels draws a box to select the squad
    // members inside it (Shift adds them).
    if buttons.pressed(MouseButton::Left) && game.drag.is_none() && game.aim.is_none() && game.placing.is_none() && game.menu.is_none() {
        if let Some(p) = game.press_at {
            if (p - mouse).length() >= 6.0 && !game.panels.iter().any(|b| b.contains(p)) {
                game.drag = Some(p);
            }
        }
    }
    if buttons.just_released(MouseButton::Left) {
        if let Some(p) = game.drag.take() {
            game.press_at = None;
            let (a, b) = (p.min(mouse), p.max(mouse));
            let inside: Vec<PersonId> = game.squad_screen.iter().filter(|(_, s)| s.x >= a.x && s.x <= b.x && s.y >= a.y && s.y <= b.y).map(|x| x.0).collect();
            if !inside.is_empty() {
                if !shift {
                    game.sel = Selection::default();
                }
                for m in inside {
                    if !game.sel.0.contains(&m) {
                        game.sel.0.push(m);
                    }
                }
            }
        }
        if let Some(p) = game.press_at.take() {
            if (p - mouse).length() < 6.0 {
                if game.menu.is_some() && !on_panels {
                    // A click away from an open menu just closes it.
                    game.menu = None;
                } else if on_panels {
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
                // Nearer or further, gliding there (and tilting with it).
                game.glide.get_or_insert(super::feel::Glide::of(&game.orbit)).zoom(wheel);
            }
            if buttons.pressed(MouseButton::Right) && d.length() > 0.0 {
                game.orbit.orbit(d.x, d.y);
            }
            if buttons.pressed(MouseButton::Middle) && d.length() > 0.0 {
                game.orbit.pan(-d.x / game.screen.y * 1.6, d.y / game.screen.y * 1.6);
                game.follow = false;
            }
            if pan != (0.0, 0.0) {
                // (`pan` takes "right" the way a drag moves the ground, so
                // the keys go the other way: A pans left, D right. P9.)
                game.orbit.pan(-pan.0 * dt, pan.1 * dt);
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
    let _prof = super::prof::span("simulate");
    let game = &mut *game;
    let dt = time.delta_secs();
    game.frame_ms = game.frame_ms * 0.9 + dt as f64 * 1000.0 * 0.1;
    // A fight breaking out near the squad drops the game to real time.
    if !game.world.alerts.is_empty() {
        // Big news gets a banner across the top for a while.
        for a in game.world.alerts.drain(..) {
            game.banners.push((a, std::time::Instant::now()));
        }
        game.banners.retain(|b| b.1.elapsed().as_secs_f32() < 8.0);
        let n = game.banners.len();
        if n > 3 {
            game.banners.drain(..n - 3);
        }
        game.speed_i = 0;
        game.paused = false;
    }
    // Time stands still while you talk, as in Morrowind (and once the
    // squad is gone).
    if !game.paused && game.game_menu.is_none() && game.title.is_none() && game.world.talk.is_none() && !game.editor.on && !game.world.squad.members.is_empty() {
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
    // Locked on to someone who's left the squad: back to the squad.
    if game.follow_who.is_some_and(|p| game.world.squad.index(p).is_none()) {
        game.follow_who = None;
    }
    let followed = match game.follow_who {
        Some(p) => game.world.fighter(p).map(|f| f.pos).unwrap_or_else(|| game.world.person_pos(p)),
        None => game.world.squad.pos,
    };
    if game.follow {
        game.map_cam.centre = followed;
    }
    // The 3D camera glides: after the squad (or the member it's locked on
    // to) when following, and to the distance and tilt it's been sent to
    // (UI-11).
    {
        let snap = game.shot.is_some() || game.frame < 2;
        let to = game.follow.then_some(followed);
        let mut g = game.glide.unwrap_or(super::feel::Glide::of(&game.orbit));
        g.step(&mut game.orbit, to, dt, snap);
        game.glide = Some(g);
    }
    // Talking: the camera comes in low beside the two, and goes back after.
    if game.view == View::Scene {
        let w = &game.world;
        let pair = w.talk.as_ref().map(|c| (c.with, c.npc)).or(game.parting.as_ref().map(|p| (p.with, p.npc)));
        let spots = pair.map(|(a, b)| (w.person_pos(a), w.person_pos(b)));
        let snap = game.shot.is_some();
        let cost = |eye: V2, mid: V2| pair.map(|p| super::talkui::sight_cost(w, p, eye, mid)).unwrap_or(0.0);
        super::talkui::aim(&mut game.orbit, &mut game.talk_cam, spots, &cost, game.follow, dt, snap);
    }
    // Keep the camera's pivot on the ground, easing so it doesn't jolt.
    let g = game.world.terrain.surface(game.orbit.target);
    game.orbit.ground += (g - game.orbit.ground) * (1.0 - (-8.0 * dt).exp());
    if (g - game.orbit.ground).abs() > 60.0 || game.frame < 2 {
        game.orbit.ground = g;
    }
}

/// Aim the 3D camera; switch it off on the map.
pub fn camera(game: Res<Game>, mut cam: Query<(&mut Camera, &mut Transform, &mut Projection), With<MainCamera>>) {
    let _prof = super::prof::span("camera");
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
/// The mouse has rested on `h` long enough for its long tooltip (or Alt is
/// held, or it's a screenshot); restarts the wait when `h` changes (UI-4).
fn tip_due(game: &mut Game, h: Hover, alt: bool) -> bool {
    if game.resting.is_none_or(|r| r.0 != h || (r.2 - game.mouse).length() > super::pointer::STILL) {
        game.resting = Some((h, std::time::Instant::now(), game.mouse));
    }
    // (Screenshots show the tooltip, or with `GAHT_PLATE=1` the name plate.)
    if game.shot.is_some() {
        return alt || std::env::var("GAHT_PLATE").is_err();
    }
    alt || game.resting.is_some_and(|r| r.1.elapsed().as_secs_f32() >= super::pointer::TIP_AFTER)
}

/// Just the name, over the thing under the mouse (UI-4): over its head if
/// it's a person, else just above the pointer.
fn name_plate(c: &hud::Canvas, game: &Game, h: Hover, size: Vec2) {
    let Some(name) = super::interact::short_name(&game.world, h) else { return };
    let vp = game.orbit.view_proj(size);
    let over = match game.view {
        View::Scene => game.picks.iter().find(|p| p.2 == h).and_then(|p| game.orbit.project(&vp, size, p.0)).map(|q| q - Vec2::new(0.0, 14.0)),
        View::Map => None,
    };
    super::pointer::plate(c, over.unwrap_or(game.mouse - Vec2::new(0.0, 16.0)), &name);
}

/// Before a walk is ordered: if it can't get all the way (the water's in
/// the way), mark the spot and have the lead say so (UI-7).
fn note_short(game: &mut Game, who: &[PersonId], p: V2) {
    let w = &game.world;
    let short = who.iter().filter_map(|&m| w.squad.index(m)).any(|k| w.walk_stops_short(w.member_pos(k), p).is_some());
    game.short = None;
    if short {
        game.short = Some((p, who.to_vec(), std::time::Instant::now()));
        // The lead says so, as a squad remark (`about=water` in squad.txt).
        if let Some(&lead) = who.first() {
            super::remarks::now(game, lead, "water");
        }
    }
}

/// The spot under the mouse: on the map, or in the 3D view the floor of a
/// building the squad is in (UI-10), else the land.
fn point_under(game: &Game, mouse: Vec2) -> Option<V2> {
    match game.view {
        View::Map => Some(game.map_cam.to_world(game.screen, mouse)),
        View::Scene => {
            let floor = |d: &gahturiyu_sim::sim::buildings::Door| game.floors.iter().find(|f| f.0 == d.id).map(|f| f.1).unwrap_or(f32::NAN);
            super::interact::floor_under(&game.world, game.orbit.ray(game.screen, mouse), &floor).or_else(|| game.orbit.ground_at(game.screen, mouse, &game.world.terrain))
        }
    }
}

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
            (None, Some(Hover::Container(id))) => game.world.container(id).map(|c| c.pos),
            _ => point,
        };
        let res = match game.aim_scroll.take() {
            Some(it) => game.world.order_read(who, it, target, point),
            None => game.world.order_cast(who, s, target, point),
        };
        if let Err(e) = res {
            game.sounds.ui("ui_cant");
            game.notice = Some((format!("{}: {}", s.def().name, e.0), std::time::Instant::now()));
        }
        return;
    }
    // What a left-click does to the thing under the mouse: the first of its
    // choices (`interact.rs`, which also names it in the tooltip).
    let world = &mut game.world;
    let who = game.sel.who(world);
    let all = game.sel.is_all(world);
    let first = match hover {
        // A workplace or a camp is only named; clicking there walks there.
        Some(Hover::Place(..)) => None,
        Some(h) => super::interact::choices(world, h, &who, shift).into_iter().find(|c| c.act != super::interact::Act::Examine),
        None => game.wild.and_then(|s| super::interact::wild_choices(world, &s).into_iter().next()),
    };
    if let Some(c) = first {
        use super::interact::Act;
        // A locked door or chest with no lockpick among the selected: they
        // only walk up to it.
        let locked = match hover {
            Some(Hover::Door(id)) => world.is_locked(id),
            Some(Hover::Container(id)) => world.container_locked(id),
            _ => false,
        };
        match c.act {
            Act::Select(pid) => game.sel.pick(pid, shift),
            Act::TownPanel(t) => game.town = if game.town == Some(t) { None } else { Some(t) },
            // To a station: they go, and its screen opens once someone is there.
            Act::Make(p) => {
                let _ = super::interact::perform(world, &who, all, Act::Make(p));
                game.sounds.ui("ui_order");
                game.inv = None;
                game.book = None;
                game.craft = game.sel.lead(world);
            }
            act => {
                // Where the click landed in the world, for its mark.
                let spot = match act {
                    Act::Walk(p) | Act::Sleep(p) | Act::SneakTo(p) => Some(p),
                    Act::Attack(p) | Act::Talk(p) | Act::Loot(p) | Act::Carry(p) | Act::Dose(p) | Act::Bind(p) => Some(world.person_pos(p)),
                    Act::Enter(d) | Act::PickLock(d) => world.door(d).map(|d| d.outside),
                    Act::Search(c) | Act::PickChest(c) => world.container(c).map(|c| c.pos),
                    _ => None,
                };
                use super::feel::MarkKind;
                let mut answered = false;
                if let Some(msg) = super::interact::perform(world, &who, all, act) {
                    if !matches!(act, Act::Dose(_) | Act::Bind(_)) {
                        game.sounds.ui("ui_cant");
                        if let Some(p) = spot {
                            game.feel.mark(p, MarkKind::Cant);
                        }
                    }
                    game.notice = Some((msg, std::time::Instant::now()));
                } else if locked && matches!(act, Act::Walk(_)) {
                    game.sounds.ui("ui_cant");
                    if let Some(p) = spot {
                        game.feel.mark(p, MarkKind::Cant);
                    }
                    // They walk up to it, and say so.
                    if let Some(&m) = who.first() {
                        super::remarks::now(game, m, "locked");
                    }
                } else {
                    if matches!(act, Act::Walk(_)) {
                        game.sounds.ui("ui_order");
                    }
                    if let Some(p) = spot {
                        game.feel.mark(p, if matches!(act, Act::Attack(_)) { MarkKind::Attack } else { MarkKind::Walk });
                    }
                    answered = true;
                }
                if answered {
                    answer_order(game, &who);
                }
            }
        }
        return;
    }
    let target = point_under(game, mouse);
    if let Some(t) = target {
        note_short(game, &who, t);
    }
    let world = &mut game.world;
    if let Some(t) = target {
        if game.sel.is_all(world) {
            world.order_squad(t);
        } else {
            world.order_members(&who, t);
        }
        game.sounds.ui("ui_order");
        // (Grey where they can't get all the way: the cross is drawn there.)
        game.feel.mark(t, if game.short.is_some() { super::feel::MarkKind::Cant } else { super::feel::MarkKind::Walk });
        answer_order(game, &who);
    }
}

/// Those just given an order nod, and now and then one says a word (UI-8):
/// a line of their own from `data/lines/squad.txt`, never over something
/// they're already saying.
fn answer_order(game: &mut Game, who: &[PersonId]) {
    if let Some((pid, nth)) = game.feel.ordered(&game.world, who) {
        if game.barks.iter().any(|b| b.0 == pid) {
            return;
        }
        if let Some(line) = game.world.squad_remark(pid, "order", &[], nth) {
            game.barks.push((pid, gahturiyu_sim::sim::speech::plain(&line), game.frame + super::feel::WORD_FRAMES));
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
                game.barks.push((p, gahturiyu_sim::sim::speech::plain(&line), f + BARK_FRAMES));
            }
        }
    }
}

/// How far from a thing's mark on screen the mouse still takes it, pixels
/// (plus the thing's own slack). Laz, 13 Oct: finding the sweet spot made
/// the game annoying to play.
const PICK_BAND: f32 = 18.0;

/// Collects things under the mouse and keeps the closest.
struct Picker {
    mouse: Vec2,
    best: Option<(f32, Hover)>,
}

impl Picker {
    fn offer(&mut self, at: Vec2, slack: f32, h: Hover) {
        let d = (at - self.mouse).length() - slack;
        self.take(d, h);
    }

    /// A thing with some height on screen (a person, head to feet): the
    /// mouse anywhere along it takes it.
    fn offer_span(&mut self, a: Vec2, b: Vec2, slack: f32, h: Hover) {
        let ab = b - a;
        let len2 = ab.length_squared();
        let t = if len2 < 1e-3 { 0.0 } else { ((self.mouse - a).dot(ab) / len2).clamp(0.0, 1.0) };
        let d = (a + ab * t - self.mouse).length() - slack;
        self.take(d, h);
    }

    fn take(&mut self, d: f32, h: Hover) {
        if d < PICK_BAND && self.best.as_ref().map(|(b, _)| d < *b).unwrap_or(true) {
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
    let _prof = super::prof::span("ui");
    let ctx = contexts.ctx_mut()?;
    if !st.fonts {
        let mut fonts = egui::FontDefinitions::default();
        let mut add = |name: &str, bytes: &'static [u8]| {
            fonts.font_data.insert(name.into(), std::sync::Arc::new(egui::FontData::from_static(bytes)));
        };
        add("dejavu", include_bytes!("../../assets/DejaVuSans.ttf"));
        // The HUD's serif faces (SIL Open Font License, `assets/fonts`).
        add("alegreya", include_bytes!("../../assets/fonts/Alegreya.ttf"));
        add("alegreya_italic", include_bytes!("../../assets/fonts/Alegreya-Italic.ttf"));
        add("alegreya_sc", include_bytes!("../../assets/fonts/AlegreyaSC-Bold.ttf"));
        add("cinzel", include_bytes!("../../assets/fonts/Cinzel.ttf"));
        let prop = fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap();
        prop.insert(0, "dejavu".into());
        prop.insert(0, "alegreya".into());
        // Each face falls back to Alegreya, then DejaVu, for anything it lacks.
        for (family, first) in [("title", "cinzel"), ("caps", "alegreya_sc"), ("italic", "alegreya_italic")] {
            fonts.families.insert(egui::FontFamily::Name(family.into()), vec![first.into(), "alegreya".into(), "dejavu".into()]);
        }
        ctx.set_fonts(fonts);
        st.fonts = true;
        // (The new faces only exist from the next frame.)
        return Ok(());
    }
    let game = &mut *game;
    super::lexicon::set_native(settings.native_names);
    super::lexicon::refresh(&game.world, game.loads);
    if st.loads != game.loads {
        st.loads = game.loads;
        st.relief = None;
    }
    let size = game.screen;
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("canvas")));
    let c = Canvas { p: painter, w: size.x, h: size.y };
    game.slides.tick(if game.shot.is_some() { 1.0 / 30.0 } else { ctx.input(|i| i.stable_dt).min(0.1) }, game.shot.is_some());
    let mut pick = Picker { mouse: game.mouse, best: None };
    let mut panels: Vec<Bx> = Vec::new();

    // ---- The view itself ----------------------------------------------------
    let view_name = match game.view {
        View::Map => {
            if st.relief.is_none() {
                st.relief = Some(map::load_relief(ctx, &game.world));
            }
            for (at, slack, h) in map::draw(&c, &game.map_cam, &game.world, game.rings, st.relief.as_ref().unwrap(), &game.sel, game.debug) {
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
                    // (A person is picked by their whole figure, not just the head.)
                    let feet = match h {
                        Hover::Person(pid) => {
                            let at = game.world.person_pos(*pid);
                            game.orbit.project(&vp, size, to3(at, scene.grid.height(&game.world.terrain, at)))
                        }
                        _ => None,
                    };
                    match feet {
                        Some(f) => pick.offer_span(s, f, *slack, *h),
                        None => pick.offer(s, *slack, *h),
                    }
                }
            }
            for (p, vit, mana, down) in &game.bars {
                if let Some(s) = game.orbit.project(&vp, size, *p) {
                    hud::draw_bar(&c, s.x, s.y - 12.0, *vit, *mana, *down);
                }
            }
            // Badges for lasting magic over heads (P20), and what the one
            // under the mouse is.
            let mut fx_tip = None;
            for (p, _, h) in &game.picks {
                if let Hover::Person(pid) = h {
                    if p.distance(eye) < 400.0 {
                        if let Some(s) = game.orbit.project(&vp, size, *p) {
                            fx_tip = super::effecticons::draw(&c, &game.world, *pid, s, game.mouse).or(fx_tip);
                        }
                    }
                }
            }
            if let Some(t) = fx_tip {
                c.panel(&t, game.mouse.x + 18.0, game.mouse.y + 12.0, 15.0);
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
            // And the squad says what it notices.
            super::remarks::update(game);
            for (pid, line, _) in &game.barks {
                let at = game.world.person_pos(*pid);
                let g = scene.grid.height(&game.world.terrain, at);
                if let Some(q) = game.orbit.project(&vp, size, to3(at, g + 2.4)) {
                    let wd = c.width(line, 15.0) + 12.0;
                    c.rect(q.x - wd / 2.0, q.y - 18.0, wd, 22.0, hud::shadow(0.6));
                    // One of ours: a brass edge.
                    if game.world.people[*pid as usize].in_squad {
                        c.rect_lines(q.x - wd / 2.0, q.y - 18.0, wd, 22.0, 1.0, super::palette::ega(super::palette::BRASS, 0.8));
                    }
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
            // Bedrolls and tents the squad has laid out.
            for bd in &game.world.beds {
                let g = scene.grid.height(&game.world.terrain, bd.pos);
                let up = if bd.kind == gahturiyu_sim::sim::bedding::BedKind::Tent { 1.6 } else { 0.3 };
                if let Some(q) = game.orbit.project(&vp, size, to3(bd.pos, g + up)) {
                    pick.offer(q, if bd.kind == gahturiyu_sim::sim::bedding::BedKind::Tent { 6.0 } else { 3.0 }, Hover::Bedding(bd.id));
                }
            }
            // The squad's campfires.
            for f in &game.world.campfires {
                let g = scene.grid.height(&game.world.terrain, f.pos);
                if let Some(q) = game.orbit.project(&vp, size, to3(f.pos, g + 0.3)) {
                    pick.offer(q, 2.5, Hover::Campfire(f.id));
                }
            }
            "3D"
        }
    };

    // ---- Over heads: floating words, and where each squad member is on screen
    // (for drag-selecting) ------------------------------------------------------
    {
        let vp = game.orbit.view_proj(size);
        let grid = scene.grid;
        let (view, orbit, map_cam) = (game.view, &game.orbit, &game.map_cam);
        let terrain = &game.world.terrain;
        let place = |at: V2, up: f32| -> Option<Vec2> {
            match view {
                View::Map => Some(map_cam.to_screen(size, at) - Vec2::new(0.0, 8.0 + up * 4.0)),
                View::Scene => orbit.project(&vp, size, to3(at, grid.height(terrain, at) + up)),
            }
        };
        let mut on_screen = Vec::new();
        for &m in &game.world.squad.members {
            let at = game.world.fighter(m).map(|f| f.pos).unwrap_or_else(|| game.world.person_pos(m));
            if let Some(s) = place(at, 1.0) {
                on_screen.push((m, s));
            }
        }
        // Screenshots step at a fixed pace, so the words do too.
        let dt = if game.shot.is_some() { 1.0 / 30.0 } else { ctx.input(|i| i.stable_dt).min(0.1) };
        game.floaters.update(&game.world, dt, game.loads);
        game.floaters.draw(&c, &game.world, &|at| place(at, 2.6));
        game.squad_screen = on_screen;
        // The box being dragged out to select.
        if let Some(p) = game.drag {
            let (a, b) = (p.min(game.mouse), p.max(game.mouse));
            c.rect(a.x, a.y, b.x - a.x, b.y - a.y, super::palette::ega(super::palette::GOLD, 0.08));
            c.rect_lines(a.x, a.y, b.x - a.x, b.y - a.y, 1.5, super::palette::ega(super::palette::GOLD, 0.8));
        }
    }

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
    // One screen at a time (what the keys opened this frame wins).
    super::screens::settle(game);
    // L also shows everything about strangers on hover (for testing).
    hud::set_see_all(game.debug);
    // The old side panel (counts, timings, races, the full log): with L.
    if game.debug {
        panels.push(Bx::from(hud::draw_hud(&c, &game.world, game.speed_i, game.paused, game.sim_ms, game.frame_ms, view_name)));
    }
    let w = &mut game.world;
    if game.inv.map(|p| w.squad.index(p).is_none()).unwrap_or(false) {
        game.inv = None;
    }
    if game.craft.map(|p| w.squad.index(p).is_none()).unwrap_or(false) {
        game.craft = None;
    }
    let click = game.ui_click.take();
    let mut actions = Vec::new();
    // The frame: the squad, the tracked job, the place, the bottom band.
    let mut open = Vec::new();
    use super::frame::{Button, FrameAct};
    for (on, b) in [(game.inv.is_some(), Button::Pack), (game.craft.is_some(), Button::Craft), (game.book.is_some(), Button::Spells), (game.journal, Button::Journal), (game.town.is_some(), Button::Town), (game.build, Button::Build), (game.view == View::Map, Button::Map), (game.stats.is_some(), Button::Stats)] {
        if on {
            open.push(b);
        }
    }
    let fs = super::frame::FrameState { sel: &game.sel, speed_i: game.speed_i, paused: game.paused, collapsed: game.squad_collapsed, open };
    // Close in on a talk, the frame folds away (the talk screen is the
    // whole window); at the trade table too, and under the title screen.
    let close_up = w.talk.is_some() || game.parting.is_some() || game.title.is_some();
    if game.view == View::Scene && !close_up {
        super::frame::banner(&c, w);
    }
    let (mut list_act, mut bottom_act) = (None, None);
    let mut town_button = false;
    if !close_up {
        let (act, boxes) = super::frame::squad_list(&c, w, &fs, click);
        list_act = act;
        panels.extend(boxes);
        if !fs.right_busy() {
            panels.extend(super::frame::tracked(&c, w));
        }
        let (act, boxes) = super::frame::bottom(&c, w, &fs, click);
        bottom_act = act;
        panels.extend(boxes);
    }
    if game.keys {
        let at = game.slides.begin(&c);
        panels.push(super::frame::keys_panel(&c, game.view == View::Map));
        game.slides.end(&c, "keys", at);
    }
    for fa in [list_act, bottom_act].into_iter().flatten() {
        // (A pick in the squad list sounds as the selection changes.)
        if !matches!(fa, FrameAct::Select(..)) {
            game.sounds.ui("ui_press");
        }
        match fa {
            FrameAct::Select(pid, add) => {
                game.sel.pick(pid, add);
                // Twice quickly on a portrait: the camera goes to them and
                // stays with them (P8).
                let again = game.last_pick.is_some_and(|(p, t)| p == pid && t.elapsed().as_secs_f32() < DOUBLE_CLICK);
                if again && !add {
                    game.follow = true;
                    game.follow_who = Some(pid);
                    game.last_pick = None;
                } else {
                    game.last_pick = Some((pid, std::time::Instant::now()));
                }
            }
            FrameAct::Pack(pid) => {
                game.book = None;
                game.craft = None;
                game.inv = if game.inv == Some(pid) { None } else { Some(pid) };
            }
            FrameAct::Collapse => game.squad_collapsed = !game.squad_collapsed,
            FrameAct::Pause => game.paused = !game.paused,
            FrameAct::Speed(i) => {
                game.speed_i = i;
                game.paused = false;
            }
            FrameAct::Sneak => {
                let who = game.sel.who(w);
                let on = !who.iter().all(|&m| w.is_sneaking(m));
                for m in who {
                    w.set_sneaking(m, on);
                }
            }
            FrameAct::Rest => {
                let who = game.sel.who(w);
                w.order_wake(&who);
            }
            FrameAct::Toggle(b) => {
                let lead = game.sel.lead(w);
                match b {
                    Button::Pack => {
                        game.craft = None;
                        game.book = None;
                        game.inv = if game.inv.is_some() { None } else { lead };
                    }
                    Button::Craft => {
                        game.inv = None;
                        game.book = None;
                        game.craft = if game.craft.is_some() { None } else { lead };
                    }
                    Button::Spells => {
                        game.inv = None;
                        game.craft = None;
                        game.book = if game.book.is_some() { None } else { lead };
                    }
                    Button::Journal => game.journal = !game.journal,
                    Button::Town => town_button = true,
                    Button::Build => {
                        game.build = !game.build;
                        if !game.build {
                            game.placing = None;
                        }
                    }
                    Button::Map => {
                        game.view = if game.view == View::Scene { View::Map } else { View::Scene };
                        if game.view == View::Map {
                            game.map_cam.centre = game.orbit.target;
                        } else {
                            game.orbit.target = game.map_cam.centre;
                        }
                    }
                    Button::Stats => game.stats = if game.stats.is_some() { None } else { lead },
                }
            }
        }
    }
    if town_button {
        toggle_town(game);
    }
    // The dev panel (kept, not drawn, under the title screen).
    if let Some(mut dp) = if game.title.is_none() { game.dev.take() } else { None } {
        let (a, bx) = super::testquarter::panel(&c, &game.world, &dp, click, click.is_some_and(|k| k.shift));
        panels.push(bx);
        let lead = game.sel.lead(&game.world).or_else(|| game.world.squad.members.first().copied());
        let close = matches!(a, Some(super::testquarter::DevAct::Close));
        if let Some(a) = a {
            if let Some(msg) = super::testquarter::apply(&mut game.world, &mut dp, a, lead, &mut weather.force) {
                game.notice = Some((msg, std::time::Instant::now()));
            }
            game.world.rules_step();
        }
        if !close {
            game.dev = Some(dp);
        }
    }
    let w = &mut game.world;
    let mut item_tip = None;
    if let Some(pid) = game.stats.filter(|&p| w.squad.index(p).is_some()) {
        let at = game.slides.begin(&c);
        let (a, bx) = super::statsui::sheet(&c, w, pid, click);
        game.slides.end(&c, "stats", at);
        panels.push(bx);
        if let Some(super::statsui::StatsAct::Close) = a {
            game.stats = None;
        }
    }
    if let Some(pid) = game.inv.filter(|&p| w.squad.index(p).is_some()) {
        let at = game.slides.begin(&c);
        let (a, bx) = super::packui::pack(&c, w, &mut game.pack, pid, game.mouse, game.left_held, click);
        game.slides.end(&c, "pack", at);
        actions.extend(a);
        panels.push(bx);
    }
    // Making: at a bench it's the bench's own screen (at an alchemy table,
    // brewing); anywhere else, what that member knows how to make and where.
    let mut make_act = None;
    match game.craft.and_then(|pid| w.bench_for(pid).map(|b| (pid, b))) {
        Some((_, bench)) => {
            let same = game.making.as_ref().is_some_and(|m| m.bench.station == bench.station && m.bench.carried_by == bench.carried_by && (bench.carried_by.is_some() || m.bench.at.dist(bench.at) < 1.0));
            if same {
                // (A carried mortar moves with its carrier.)
                game.making.as_mut().unwrap().bench = bench;
            } else {
                game.making = Some(super::makeui::Making::at(bench));
            }
            let at = game.slides.begin(&c);
            let (a, bx, scroll) = super::makeui::making(&c, w, game.making.as_ref().unwrap(), game.mouse, click);
            game.slides.end(&c, "making", at);
            game.making.as_mut().unwrap().scroll = scroll;
            make_act = a;
            panels.push(bx);
        }
        None => {
            game.making = None;
            if let Some(pid) = game.craft {
                let at = game.slides.begin(&c);
                let (a, h, bx) = squadui::crafting(&c, w, pid, game.mouse, click);
                game.slides.end(&c, "craft", at);
                actions.extend(a);
                item_tip = item_tip.or(h);
                panels.push(bx);
            }
        }
    }
    if let Some(a) = make_act {
        use super::makeui::MakeAct;
        match a {
            MakeAct::Close => {
                game.craft = None;
                game.making = None;
            }
            MakeAct::Pick(ri) => {
                if let Some(m) = game.making.as_mut() {
                    m.pick = Some(ri);
                    m.n = 1;
                }
                game.sounds.ui("ui_press");
            }
            MakeAct::Who(p) => {
                if let Some(m) = game.making.as_mut() {
                    m.who = Some(p);
                }
                game.sounds.ui("ui_press");
            }
            MakeAct::Count(n) => {
                if let Some(m) = game.making.as_mut() {
                    m.n = n;
                }
                game.sounds.ui("ui_press");
            }
            MakeAct::Make(p, ri, n) => match w.make(p, ri, n) {
                Ok(()) => game.sounds.ui("ui_press"),
                Err(why) => {
                    game.sounds.ui("ui_cant");
                    game.notice = Some((format!("Can't: {}.", why.say()), std::time::Instant::now()));
                }
            },
            MakeAct::Add(id) => {
                if let Some(m) = game.making.as_mut() {
                    if m.mix.len() < gahturiyu_sim::sim::items::herbs::MIX_INGREDIENTS {
                        m.mix.push(id);
                    }
                }
                game.sounds.ui("ui_press");
            }
            MakeAct::Remove(k) => {
                if let Some(m) = game.making.as_mut() {
                    if k < m.mix.len() {
                        m.mix.remove(k);
                    }
                }
                game.sounds.ui("ui_press");
            }
            MakeAct::Recall(ings) => {
                if let Some(m) = game.making.as_mut() {
                    m.mix = ings;
                }
                game.sounds.ui("ui_press");
            }
            MakeAct::Write(p, k) => match w.write_sheet(p, k) {
                Ok(()) => game.sounds.ui("ui_press"),
                Err(why) => {
                    game.sounds.ui("ui_cant");
                    game.notice = Some((format!("Can't: {why}."), std::time::Instant::now()));
                }
            },
            MakeAct::Teach(p) => {
                let others: Vec<_> = game.making.as_ref().map(|m| w.bench_hands(&m.bench)).unwrap_or_default().into_iter().filter(|&o| o != p).collect();
                let learned: usize = others.iter().map(|&o| w.share_lore(p, o)).sum();
                game.sounds.ui("ui_press");
                game.notice = Some((if learned > 0 { format!("{} teaches the others what they know ({learned} effects).", w.name_of(p)) } else { "They know nothing the others don't.".to_string() }, std::time::Instant::now()));
            }
            MakeAct::Brew(p, ings, n) => match w.brew(p, &ings, n) {
                Ok(()) => game.sounds.ui("ui_press"),
                Err(why) => {
                    game.sounds.ui("ui_cant");
                    game.notice = Some((format!("Can't: {}.", why.say()), std::time::Instant::now()));
                }
            },
        }
    }
    let mut journal_act = None;
    if game.journal {
        let at = game.slides.begin(&c);
        let (a, bx) = super::journalui::journal(&c, w, click);
        game.slides.end(&c, "journal", at);
        journal_act = a;
        panels.push(bx);
    }
    let mut build_act = None;
    if game.build {
        let here = match game.view {
            View::Scene => game.orbit.target,
            View::Map => game.map_cam.centre,
        };
        let at = game.slides.begin(&c);
        let (a, bx) = super::baseui::build_panel(&c, w, here, game.placing.as_ref().map(|p| p.def), game.base_tab, game.mouse, click);
        game.slides.end(&c, "build", at);
        build_act = a;
        panels.push(bx);
    }
    let mut town_closed = false;
    if let Some(t) = game.town {
        let at = game.slides.begin(&c);
        if game.debug || game.town_all {
            // The testing readout (L): everything, numbers and all.
            panels.push(super::townui::readout_panel(&c, w, t));
            panels.push(super::townui::life_panel(&c, w, t, true));
        } else {
            let (bx, closed) = super::townui::town_panel(&c, w, t, click);
            panels.push(bx);
            town_closed = closed;
        }
        game.slides.end(&c, "town", at);
    }
    if town_closed {
        game.town = None;
    }
    if game.book.map(|p| w.squad.index(p).is_none()).unwrap_or(false) {
        game.book = None;
    }
    let mut spell_tip = None;
    let mut book_left = game.mouse.x;
    if let Some(pid) = game.book {
        let at = game.slides.begin(&c);
        let (a, h, bx) = squadui::spell_book(&c, w, pid, game.mouse, click);
        game.slides.end(&c, "spells", at);
        actions.extend(a);
        spell_tip = h;
        book_left = bx.x;
        panels.push(bx);
    }
    panels.extend(super::hints::show(&c, &mut game.hints, w, click));
    // The loot box stands beside the body or chest on screen.
    let loot_at = super::lootui::open_loot(w).and_then(|(_, src)| {
        let p = super::lootui::loot_spot(w, src);
        match game.view {
            View::Scene => {
                let g = scene.grid.height(&w.terrain, p);
                game.orbit.project(&game.orbit.view_proj(size), size, to3(p, g + 0.8))
            }
            View::Map => Some(game.map_cam.to_screen(game.screen, p)),
        }
    });
    if super::lootui::open_loot(w).is_none() {
        game.loot_put = false;
    }
    let take_key = std::mem::take(&mut game.space_take);
    let at = game.slides.begin(&c);
    let (loot_act, loot_box) = super::lootui::loot_panel(&c, w, loot_at, game.loot_put, take_key, click);
    game.slides.end(&c, "loot", at);
    panels.extend(loot_box);
    match loot_act {
        Some(super::lootui::LootAct::Take(m, src, what)) => {
            w.take_from(m, src, what);
            game.sounds.ui("take_item");
        }
        Some(super::lootui::LootAct::TakeAll(m, src)) => {
            w.take_all_from(m, src);
            game.sounds.ui("take_item");
        }
        Some(super::lootui::LootAct::Put(m, k)) => {
            w.put_in(m, k);
        }
        Some(super::lootui::LootAct::Close(m)) => w.stop_looting(m),
        Some(super::lootui::LootAct::Tab(put)) => {
            game.loot_put = put;
            game.sounds.ui("ui_press");
        }
        None => {}
    }
    // A merchant's wares out: the trade table takes the talk's place.
    let at_table = w.talk.as_ref().filter(|c| c.trading && !c.refused).map(|c| c.npc);
    if let Some(npc) = at_table {
        use super::tradeui::TradeAct;
        let now = std::time::Instant::now();
        let st = game.trading.get_or_insert_with(Default::default);
        // The sums, worked out afresh when the table changes and now and then
        // (prices move with the hour).
        if st.quote.is_none() || game.trade_quoted.is_none_or(|t| now.duration_since(t).as_secs_f32() > 2.0) {
            st.quote = Some(w.deal_quote(npc, &st.table));
            game.trade_quoted = Some(now);
        }
        let at = game.slides.begin(&c);
        let (a, bx, scrolls) = super::tradeui::trade(&c, w, st, game.mouse, click);
        game.slides.end(&c, "trade", at);
        st.scroll = scrolls;
        panels.push(bx);
        let twice = |last: &Option<(std::time::Instant, bool, items::ItemId)>, giving: bool, it: items::ItemId| last.is_some_and(|l| l.1 == giving && l.2 == it && now.duration_since(l.0).as_secs_f32() < 0.35);
        let deal_key = game.space_deal;
        game.space_deal = false;
        match a.or(if deal_key { Some(TradeAct::Deal) } else { None }) {
            Some(TradeAct::Close) => {
                w.stop_trading();
                game.trading = None;
                game.sounds.ui("ui_close");
            }
            Some(TradeAct::Member(m)) => {
                st.member = Some(m);
                game.sounds.ui("ui_press");
            }
            Some(TradeAct::Give(who, it, mix)) => {
                if twice(&game.trade_last, true, it) {
                    // The one just laid comes back off, and is sold on the spot.
                    if let Some(k) = st.table.give.iter().position(|g| g.0 == who && g.1 == it && g.3 == mix) {
                        st.table.give[k].2 -= 1;
                        if st.table.give[k].2 == 0 {
                            st.table.give.remove(k);
                        }
                    }
                    game.sounds.ui(if w.sell_from_mix(npc, who, it, mix).is_some() { "coins" } else { "ui_cant" });
                    game.trade_last = None;
                } else {
                    // (A brewed bottle: only those of this mix.)
                    let have = match mix {
                        Some(m) => w.people[who as usize].detail.as_ref().map(|d| d.gear.bag.iter().filter(|e| e.0 == it && e.2.and_then(|p| p.mix) == Some(m)).map(|e| e.1).sum()).unwrap_or(0),
                        None => w.count_of(who, items::item(it).key),
                    };
                    st.table.add_give_mix(who, it, mix, have);
                    game.trade_last = Some((now, true, it));
                    game.sounds.ui("ui_press");
                }
                st.quote = None;
            }
            Some(TradeAct::Get(it, mix)) => {
                if twice(&game.trade_last, false, it) {
                    if let Some(k) = st.table.get.iter().position(|g| g.0 == it && g.2 == mix) {
                        st.table.get[k].1 -= 1;
                        if st.table.get[k].1 == 0 {
                            st.table.get.remove(k);
                        }
                    }
                    let lot = w.stall(npc).into_iter().find(|l| l.item == it && l.mix == mix);
                    game.sounds.ui(if lot.is_some_and(|l| w.buy_lot(npc, it, mix, l.price)) { "coins" } else { "ui_cant" });
                    game.trade_last = None;
                } else {
                    let stock = w.stall(npc).iter().find(|l| l.item == it && l.mix == mix).map(|l| l.n).unwrap_or(0);
                    st.table.add_get_mix(it, mix, stock);
                    game.trade_last = Some((now, false, it));
                    game.sounds.ui("ui_press");
                }
                st.quote = None;
            }
            Some(TradeAct::Back(giving, k)) => {
                if giving {
                    if let Some(g) = st.table.give.get_mut(k) {
                        g.2 -= 1;
                        if g.2 == 0 {
                            st.table.give.remove(k);
                        }
                    }
                } else if let Some(g) = st.table.get.get_mut(k) {
                    g.1 -= 1;
                    if g.1 == 0 {
                        st.table.get.remove(k);
                    }
                }
                st.quote = None;
                game.sounds.ui("ui_press");
            }
            Some(TradeAct::Clear) => {
                st.table = Default::default();
                st.quote = None;
                game.sounds.ui("ui_press");
            }
            Some(TradeAct::Deal) => {
                let done = w.deal_table(npc, &st.table);
                if done.stuck.is_none() {
                    st.table = Default::default();
                    game.sounds.ui("coins");
                } else {
                    game.sounds.ui("ui_cant");
                }
                st.quote = None;
            }
            None => {}
        }
    } else {
        game.trading = None;
        game.space_deal = false;
        // Parting words stay up a moment, then the screen lets go.
        if game.parting.as_ref().is_some_and(|p| w.talk.is_some() || p.since.elapsed().as_secs_f32() > super::talkui::PARTING_SECS) {
            game.parting = None;
        }
        let key = game.talk_key.take();
        if let Some(pair) = w.talk.as_ref().map(|cv| (cv.with, cv.npc)) {
            // A new talk starts with nothing asked.
            if game.talking.as_ref().is_none_or(|t| t.pair != pair) {
                game.talking = Some(super::talkui::Talking { pair, ..Default::default() });
            }
        } else {
            game.talking = None;
        }
        if w.talk.is_some() || game.parting.is_some() {
            use super::talkui::TalkAct;
            use gahturiyu_sim::sim::dialogue::Topic;
            let st = game.talking.clone().unwrap_or_default();
            let at = game.slides.begin(&c);
            let (a, bx, scroll) = super::talkui::talk(&c, w, &st, game.parting.as_ref(), game.mouse, click, key);
            game.slides.end(&c, "talk", at);
            if let Some(t) = game.talking.as_mut() {
                t.scroll = scroll;
            }
            panels.push(bx);
            match a {
                Some(TalkAct::Ask(t)) => {
                    let coins = matches!(t, Topic::Trade | Topic::Buy(..) | Topic::Sell(..) | Topic::SellAll(..));
                    game.sounds.ui(if coins { "coins" } else { "ui_press" });
                    let (with, npc) = st.pair;
                    let said = w.ask(t);
                    if let Some(tk) = game.talking.as_mut() {
                        tk.asked.push(t);
                        tk.scroll = 0.0;
                    }
                    // A goodbye ends the talk there and then: their last
                    // words are held on the screen for a moment.
                    if let (None, Some(line)) = (w.talk.as_ref(), said) {
                        game.parting = Some(super::talkui::Parting { with, npc, line, since: std::time::Instant::now() });
                    }
                }
                Some(TalkAct::Done) => game.parting = None,
                None => {}
            }
        }
    }
    match journal_act {
        Some(super::journalui::JournalAct::Close) => game.journal = false,
        // A job clicked: the view goes to where it points.
        Some(super::journalui::JournalAct::Look(p)) => {
            game.follow = false;
            match game.view {
                View::Scene => {
                    game.orbit.target = p;
                    game.orbit.ground = w.terrain.surface(p);
                }
                View::Map => game.map_cam.centre = p,
            }
            game.sounds.ui("ui_press");
        }
        None => {}
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
        Some(super::baseui::BuildAction::OpenStore(b)) => {
            // Whoever is selected and here, else whoever is here.
            let here: Vec<_> = w.base(b).map(|x| x.present.clone()).unwrap_or_default();
            let sel = game.sel.who(w);
            let who = sel.iter().copied().find(|m| here.contains(m)).or_else(|| here.first().copied());
            if who.is_some_and(|m| w.order_store(m, b)) {
                // The store takes the panel's place (B brings it back).
                game.build = false;
                game.placing = None;
            } else {
                game.notice = Some(("Nobody here can see to the store.".to_string(), std::time::Instant::now()));
            }
        }
        None => {}
    }
    for a in actions {
        match a {
            Action::CloseInventory => {
                game.inv = None;
                game.craft = None;
            }
            // A scroll of a harmful spell is aimed like the spell itself.
            Action::Use(pid, it) if matches!(items::item(it).kind, items::Kind::Scroll(k) if !gahturiyu_sim::sim::magic::spell(k).def().works_outside_fights() || w.fighting.contains_key(&pid)) => {
                if let items::Kind::Scroll(k) = items::item(it).kind {
                    game.aim = Some((pid, gahturiyu_sim::sim::magic::spell(k)));
                    game.aim_scroll = Some(it);
                }
            }
            Action::Use(pid, it) => {
                let why = w.why_cant_use(pid, it);
                if !w.use_item(pid, it) {
                    game.sounds.ui("ui_cant");
                    game.notice = Some((why.unwrap_or_else(|| "That can't be used just now.".into()), std::time::Instant::now()));
                }
            }
            Action::Craft(pid, r) => {
                let _ = w.start_craft(pid, r);
            }
            Action::UseEntry(pid, k) => {
                if !w.use_entry(pid, k) {
                    game.sounds.ui("ui_cant");
                    game.notice = Some(("That can't be used just now.".into(), std::time::Instant::now()));
                }
            }
            Action::CoatEntry(pid, k) => {
                if !w.coat_weapon(pid, k) {
                    game.sounds.ui("ui_cant");
                    game.notice = Some((format!("Can't: {}.", w.why_cant_coat(pid, k).unwrap_or("not now")), std::time::Instant::now()));
                }
            }
            Action::Taste(pid, it) => {
                if !w.taste(pid, it) {
                    game.sounds.ui("ui_cant");
                    game.notice = Some(("That can't be tasted just now.".into(), std::time::Instant::now()));
                }
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
            Action::Place(pid, which, g, x, y, turned) => {
                if !w.place_in_pack(pid, which, g, x, y, turned) {
                    game.notice = Some(("It won't fit there.".into(), std::time::Instant::now()));
                }
            }
            Action::Turn(pid, k) => {
                if !w.turn_in_pack(pid, k) {
                    game.notice = Some(("No room to turn it there.".into(), std::time::Instant::now()));
                }
            }
            Action::Tidy(pid) => w.tidy_pack(pid),
            Action::GiveEntry(pid, k, to) => {
                let msg = match w.order_give(pid, k, to) {
                    Ok(m) | Err(m) => m,
                };
                game.notice = Some((msg, std::time::Instant::now()));
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
                    game.aim_scroll = None;
                } else if let Err(e) = w.use_spell(pid, s, None, None) {
                    game.sounds.ui("ui_cant");
                    game.notice = Some((format!("{}: {}", d.name, e.0), std::time::Instant::now()));
                }
            }
        }
    }
    if game.debug {
        panels.push(Bx::from(super::foliage::draw_readout(&c, &game.orbit, &scene, &foliage, &models)));
    }
    if let Some(mut t) = game.title.take() {
        use super::title::TitleAct;
        // (With the settings open from it, only its backdrop: the click is the panel's.)
        let (act, bx) = if game.options { (None, super::title::backdrop(&c, size)) } else { super::title::draw(&c, &mut t, click, size) };
        panels.push(bx);
        let mut keep = true;
        // The rule switches live in the world: those flipped here go with the
        // player into the next one (a new game always takes them).
        let seen = *t.rules_seen.get_or_insert(game.world.rules);
        let rules = game.world.rules;
        let carry = |game: &mut Game, always: bool| {
            if always || rules != seen {
                game.world.rules = rules;
            }
        };
        if let Some(act) = act {
            game.sounds.ui("ui_press");
            match act {
                TitleAct::Resume => keep = false,
                TitleAct::Continue(p) | TitleAct::Load(p) => match World::load_from(&p) {
                    Ok(w) => {
                        swap_world(game, w);
                        carry(game, false);
                        game.notice = Some(("Loaded.".to_string(), std::time::Instant::now()));
                        keep = false;
                    }
                    Err(e) => game.notice = Some((format!("Couldn't load: {e}"), std::time::Instant::now())),
                },
                TitleAct::NewGame => {
                    let seed = seed_now();
                    swap_world(game, fresh_world(seed, true, start_kind(false)));
                    carry(game, true);
                    game.notice = Some((format!("A new world (seed {seed})."), std::time::Instant::now()));
                    keep = false;
                }
                TitleAct::Arena(a) => {
                    let seed = seed_now();
                    let mut w = fresh_world(seed, true, worldgen::Start::Town);
                    w.rules = rules;
                    arena_setup(&mut w, a);
                    swap_world(game, w);
                    if a == super::title::Arena::Combat {
                        game.dev = Some(super::testquarter::DevPanel { tab: super::testquarter::Tab::People, ..Default::default() });
                    }
                    let tip = if a == super::title::Arena::Combat { " Pick foes in the dev panel (Insert)." } else { "" };
                    game.notice = Some((format!("{} (seed {seed}).{tip}", a.label()), std::time::Instant::now()));
                    keep = false;
                }
                TitleAct::Settings => game.options = true,
                TitleAct::Quit => game.quit = true,
            }
        }
        if keep {
            game.title = Some(t);
        }
    }
    if game.options {
        let mut s = settings.clone();
        let at = game.slides.begin(&c);
        panels.push(super::settings::panel(&c, &mut s, &mut game.world.rules, game.mouse, click, game.frame_ms));
        // A rule flipped shows at once, paused or not.
        game.world.rules_step();
        game.slides.end(&c, "settings", at);
        if s != *settings {
            *settings = s;
            game.sounds.ui("ui_press");
        }
    }
    let at = game.slides.begin(&c);
    panels.extend(super::weather::panel(&c, game, &mut weather, click));
    game.slides.end(&c, "weather", at);

    // (Screenshots: `GAHT_MENU=1` right-clicks at the fake mouse.)
    if game.shot.is_some() && game.frame == 20 && std::env::var("GAHT_MENU").is_ok() {
        let ground = point_under(game, game.mouse);
        game.menu = Some(super::interact::Menu { at: game.mouse, hover: game.hover, wild: game.wild, ground });
    }
    // The right-click menu, and anything pinned with Examine.
    if let Some(m) = game.menu {
        let who = game.sel.who(&game.world);
        let all = game.sel.is_all(&game.world);
        let at = game.slides.begin(&c);
        let (picked, bx) = super::interact::draw_menu(&c, &game.world, &m, &who, game.mouse, click);
        game.slides.end(&c, "menu", at);
        panels.push(bx);
        if let Some(act) = picked {
            use super::interact::Act;
            game.menu = None;
            match act {
                Act::Examine => game.examine = m.hover,
                Act::Select(pid) => game.sel.pick(pid, false),
                Act::TownPanel(t) => game.town = Some(t),
                Act::Make(p) => {
                    let _ = super::interact::perform(&mut game.world, &who, all, Act::Make(p));
                    game.sounds.ui("ui_order");
                    game.inv = None;
                    game.book = None;
                    game.craft = game.sel.lead(&game.world);
                }
                Act::SneakTo(p) => {
                    for &x in &who {
                        game.world.set_sneaking(x, true);
                    }
                    note_short(game, &who, p);
                    let _ = super::interact::perform(&mut game.world, &who, all, Act::Walk(p));
                    game.sounds.ui("ui_order");
                }
                act => {
                    if let Act::Walk(p) = act {
                        note_short(game, &who, p);
                    }
                    if let Some(msg) = super::interact::perform(&mut game.world, &who, all, act) {
                        // (A draught handed over says so too; that isn't a failure.)
                        if !matches!(act, Act::Dose(_) | Act::Bind(_)) {
                            game.sounds.ui("ui_cant");
                        }
                        game.notice = Some((msg, std::time::Instant::now()));
                    } else if matches!(act, Act::Walk(_)) {
                        game.sounds.ui("ui_order");
                    }
                }
            }
        }
    }
    if let Some(h) = game.examine {
        let mut lines = hud::describe(&game.world, h);
        lines.retain(|(l, _)| !l.to_lowercase().contains("click"));
        if let Some(f) = lines.first_mut() {
            f.1 = super::palette::GOLD;
        }
        lines.push(("Esc or click × to close".into(), super::palette::DIM));
        let r = Bx::from(c.panel(&lines, size.x - 520.0, 200.0, 15.0));
        let close = Bx::new(r.x + r.w - 24.0, r.y + 6.0, 18.0, 18.0);
        c.text("×", close.x + 3.0, close.y + 15.0, 18.0, super::palette::DIM);
        if click.is_some_and(|k| close.contains(k.at)) {
            game.examine = None;
        }
        panels.push(r);
    }

    // The game menu, over everything else.
    if let Some(mut m) = game.game_menu {
        let on_island = game.world.on_island();
        let (picked, bx) = super::screens::draw_menu(&c, &mut m, click, size, on_island);
        game.game_menu = Some(m);
        panels.push(bx);
        if let Some(act) = picked {
            use super::screens::MenuAct;
            game.sounds.ui("ui_press");
            game.game_menu = None;
            match act {
                MenuAct::Resume => {}
                MenuAct::Settings => game.options = true,
                MenuAct::Keys => game.keys = true,
                MenuAct::Save => quick_save_now(game),
                MenuAct::Load => quick_load_now(game),
                MenuAct::NewGame => {
                    let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(1);
                    swap_world(game, fresh_world(seed, true, start_kind(false)));
                    game.notice = Some((format!("A new world (seed {seed})."), std::time::Instant::now()));
                }
                MenuAct::SkipIsland => {
                    game.world.skip_island();
                    game.notice = Some(("Ashore on the mainland.".to_string(), std::time::Instant::now()));
                }
                MenuAct::Title => game.title = Some(super::title::Title::from_game()),
                MenuAct::Quit => game.quit = true,
            }
        }
    }
    // A screen opened from a panel this frame closes the others.
    super::screens::settle(game);
    // Panels just closed fade where they were (under the tooltips).
    game.slides.finish(&c);

    // ---- Hover --------------------------------------------------------------
    let on_panels = panels.iter().any(|b| b.contains(game.mouse));
    game.hover = if on_panels { None } else { pick.best.map(|(_, h)| h) };
    // Under the mouse when nothing smaller is: a building, a piece of
    // furniture, a camp, a workplace (shown if no animal is there either).
    let ground_hover = if on_panels || game.hover.is_some() {
        None
    } else {
        let gp = point_under(game, game.mouse);
        let on_ground = gp.and_then(|p| super::interact::ground_hover(&game.world, p));
        // A building under the mouse, by its outline on screen (the ground
        // behind a roof isn't the building): the one nearest its middle.
        let mut building = None;
        if game.view == View::Scene && !matches!(on_ground, Some(Hover::Furniture(..))) {
            let vp = game.orbit.view_proj(size);
            let mut best = f32::MAX;
            for d in game.world.doors_near(game.orbit.target, game.orbit.draw_radius().min(400.0)) {
                // (One the squad is in is open: its floor is ground, UI-10.)
                if super::interact::squad_inside(&game.world, d.id) {
                    continue;
                }
                let g = scene.grid.height(&game.world.terrain, d.centre);
                let mid = to3(d.centre, g + d.radius * 0.35);
                let (Some(a), Some(b)) = (game.orbit.project(&vp, size, mid), game.orbit.project(&vp, size, mid + Vec3::new(d.radius * 0.8, 0.0, 0.0))) else { continue };
                let r = (b - a).length().max(6.0);
                let k = (game.mouse - a).length() / r;
                if k < 1.0 && k < best {
                    best = k;
                    building = Some(Hover::Building(d.id));
                }
            }
        }
        match on_ground {
            Some(Hover::Furniture(..)) => on_ground,
            _ => building.or(on_ground),
        }
    };
    let alt = ctx.input(|i| i.modifiers.alt) || (game.shot.is_some() && std::env::var("GAHT_ALT").is_ok());
    let shift_held = ctx.input(|i| i.modifiers.shift);
    let who_sel = game.sel.who(&game.world);
    if let (Some(p), false) = (&game.placing, on_panels) {
        super::baseui::placing_hint(&c, p, game.mouse);
    }
    if let Some(s) = spell_tip {
        let lines = squadui::spell_lines(s);
        let wd = lines.iter().map(|(l, _)| c.width(l, 15.0)).fold(0.0, f32::max) + 24.0;
        // Beside the book, not over it.
        c.panel(&lines, book_left - wd - 10.0, game.mouse.y, 15.0);
    } else if let Some((who, s)) = game.aim {
        // Aiming: what the click would do, and whether it can.
        let ground = point_under(game, game.mouse);
        let (line, ok) = super::interact::aim_label(&game.world, who, s, game.hover, game.aim_scroll.is_some(), ground);
        let lines = vec![(s.def().name.to_string(), super::palette::GOLD), (line, if ok { super::palette::BRASS_LIGHT } else { [0.95, 0.38, 0.30] }), ("Right-click or Esc to cancel".to_string(), super::palette::DIM)];
        c.panel(&lines, game.mouse.x + 18.0, game.mouse.y + 12.0, 15.0);
    }
    if let Some(it) = item_tip {
        let mut lines = squadui::item_lines(it);
        // What a merchant here gives for it, beside the round "worth".
        if let Some((p, town)) = game.inv.map_or_else(|| game.world.sells_for(it, None), |m| game.world.sells_for_held(m, it, None)) {
            let name = &game.world.settlements[town as usize].name;
            lines.push((if p > 0 { format!("Sells for about {p} in {name}") } else { format!("Nobody in {name} pays for it") }, super::palette::DIM));
        }
        let wd = lines.iter().map(|(l, _)| c.width(l, 15.0)).fold(0.0, f32::max) + 24.0;
        c.panel(&lines, game.mouse.x - wd - 18.0, game.mouse.y, 15.0);
    } else if let Some(h) = game.hover {
        if game.aim.is_none() {
            if tip_due(game, h, alt) {
                c.panel(&super::interact::tooltip(&game.world, h, &who_sel, shift_held, alt), game.mouse.x + 18.0, game.mouse.y + 12.0, 16.0);
            } else {
                name_plate(&c, game, h, size);
            }
        }
    }
    super::animals::overlay(&c, game, &scene, &mut panels);
    if game.hover.is_none() && game.wild.is_none() && game.aim.is_none() {
        if let Some(h) = ground_hover {
            game.hover = Some(h);
            if tip_due(game, h, alt) {
                c.panel(&super::interact::tooltip(&game.world, h, &who_sel, shift_held, alt), game.mouse.x + 18.0, game.mouse.y + 12.0, 16.0);
            } else {
                name_plate(&c, game, h, size);
            }
        }
    }
    if game.hover.is_none() {
        game.resting = None;
    }
    // Beside the pointer: what a left-click will do (UI-5). (Screenshots
    // show it only with a mouse set, `GAHT_HOVER`.)
    let real_mouse = game.shot.is_none() || std::env::var("GAHT_HOVER").is_ok();
    if real_mouse && !on_panels && game.aim.is_none() && game.placing.is_none() && game.menu.is_none() && game.drag.is_none() && !game.editor.on {
        let ground = if game.hover.is_none() && game.wild.is_none() { point_under(game, game.mouse) } else { None };
        if let Some((m, crime)) = super::pointer::mark_for(&game.world, game.hover, game.wild.as_ref(), ground, &who_sel, shift_held) {
            super::pointer::draw_mark(&c, game.mouse, m, crime);
        }
    }
    // Alt: a label on everything that can be hovered, as in Baldur's Gate 3.
    if alt && game.view == View::Scene && game.aim.is_none() {
        let vp = game.orbit.view_proj(size);
        let mut n = 0;
        for (p, _, h) in &game.picks {
            if n > 90 || matches!(h, Hover::Group(_)) {
                continue;
            }
            let Some(q) = game.orbit.project(&vp, size, *p) else { continue };
            if q.x < 0.0 || q.y < 0.0 || q.x > size.x || q.y > size.y - super::frame::BOTTOM_CLEAR || panels.iter().any(|b| b.contains(q)) {
                continue;
            }
            if let Some(name) = super::interact::short_name(&game.world, *h) {
                let wd = c.width(&name, 13.0) + 10.0;
                c.rect(q.x - wd / 2.0, q.y - 30.0, wd, 18.0, hud::shadow(0.6));
                c.centred(&name, q.x, q.y - 16.0, 13.0, super::palette::TEXT);
                n += 1;
            }
        }
    }
    // Banners: the latest big news, fading after a few seconds.
    let mut by = 168.0;
    for (msg, at) in game.banners.iter().filter(|b| b.1.elapsed().as_secs_f32() < 8.0 || game.shot.is_some()) {
        let age = at.elapsed().as_secs_f32();
        let fade = if game.shot.is_some() { 1.0 } else { (1.0 - (age - 6.0) / 2.0).clamp(0.0, 1.0) };
        let wd = (c.styled_width(msg, 17.0, hud::Face::Body, 0.0) + 48.0).min(size.x - 80.0);
        let x = (size.x - wd) / 2.0;
        c.frame_box(x, by, wd, 36.0);
        c.diamond(x + 14.0, by + 18.0, 4.0, super::palette::ega(super::palette::BRASS_LIGHT, fade));
        c.styled(msg, x + 28.0, by + 24.0, 17.0, super::palette::ega(super::palette::TEXT, fade), hud::Face::Body, 0.0);
        by += 44.0;
    }
    // Everyone dead: the end of this game (B2).
    if game.world.squad.members.is_empty() {
        let (a, b) = ("Your squad is gone.", "F9 loads your last save.");
        let wd = c.width(a, 22.0).max(c.width(b, 17.0)) + 64.0;
        let x = (size.x - wd) / 2.0;
        let y = size.y / 2.0 - 50.0;
        c.frame_box(x, y, wd, 90.0);
        c.centred(a, size.x / 2.0, y + 40.0, 22.0, super::palette::GOLD);
        c.centred(b, size.x / 2.0, y + 70.0, 17.0, super::palette::TEXT);
    }
    if let Some((msg, at)) = &game.notice {
        if at.elapsed().as_secs_f32() < 3.0 || game.shot.is_some() {
            let wd = c.width(msg, 17.0) + 32.0;
            c.rect((size.x - wd) / 2.0, 130.0, wd, 34.0, hud::shadow(0.7));
            c.centred(msg, size.x / 2.0, 153.0, 17.0, super::palette::GOLD);
        }
    }
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
    super::prof::frame();
    game.frame += 1;
    if game.quit {
        exit.write(AppExit::Success);
        return;
    }
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
