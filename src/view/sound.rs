//! Sound: short placeholder effects for what the player does and for fights
//! near the camera. Drawing only, like the floating words: nothing here
//! reaches the world, and the dice that pick a take or nudge the pitch are
//! the window's own (they never touch the sim's randomness).
//!
//! The sounds are ours (`tools/sounds/make_sounds.py` makes them) and live in
//! `assets/sounds`, listed in its `manifest.json` with their group. A name
//! with takes (`ui_press_1..3`) is played by its base name (`ui_press`), one
//! take at random.
//!
//! Anything with `&mut Game` asks for a sound with `game.sounds.ui(name)` or
//! `game.sounds.at(name, pos)`; `update` plays the queue once a frame and
//! also notices, by comparing with last frame, panels opening and closing,
//! the selection changing and the blows, shots, casts and falls of fights
//! near the camera. Silent in screenshots (`GAHT_SHOT`) and when the
//! machine has no sound device (Bevy says so once and carries on).

use std::collections::HashMap;
use std::time::{Duration, Instant};

use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;

use gahturiyu_sim::sim::{combat::Act, geo::V2, loot::Source};

use super::app::{Game, View};
use super::cam::to3;
use super::settings::Settings;

/// The same sound at most this often (a burst of world steps in one frame
/// doesn't stack a dozen copies).
const AGAIN: Duration = Duration::from_millis(50);
/// World sounds: full volume this near the listener, silent (and skipped)
/// beyond `FAR`, metres.
const NEAR: f32 = 20.0;
const FAR: f32 = 120.0;
/// Where the listener sits: this share of the way from the point the camera
/// looks at up to the camera, so zooming out makes the world quieter.
const LISTENER: f32 = 0.3;
/// World sounds vary a little each time: pitch ±, volume −.
const PITCH_JITTER: f32 = 0.06;
const VOLUME_JITTER: f32 = 0.15;

/// Group volumes on top of the master setting (the files are already mixed
/// against each other: interface −18 dBFS, screens −14, world −10).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Ui,
    Screen,
    World,
}

impl Group {
    fn from_manifest(s: &str) -> Group {
        match s {
            "interface" => Group::Ui,
            "screens" => Group::Screen,
            _ => Group::World,
        }
    }
    fn volume(self) -> f32 {
        match self {
            Group::Ui => 1.0,
            Group::Screen => 1.0,
            Group::World => 1.0,
        }
    }
}

/// The name a take is played by: `ui_press_2` → `ui_press`; a name without
/// a take number is its own.
pub fn base_name(file_name: &str) -> &str {
    match file_name.rsplit_once('_') {
        Some((base, n)) if !base.is_empty() && !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) => base,
        _ => file_name,
    }
}

/// Sounds asked for this frame: (name, where in the world, if anywhere).
#[derive(Default)]
pub struct Queue(Vec<(&'static str, Option<V2>)>);

impl Queue {
    /// An interface or screen sound, the same wherever the camera is.
    pub fn ui(&mut self, name: &'static str) {
        self.0.push((name, None));
    }
    /// A sound from a place in the world: quieter with distance. (Nothing
    /// asks yet besides the fights `update` hears itself; footsteps, doors
    /// and fires will.)
    #[allow(dead_code)]
    pub fn at(&mut self, name: &'static str, pos: V2) {
        self.0.push((name, Some(pos)));
    }
}

struct Sound {
    takes: Vec<Handle<AudioSource>>,
    group: Group,
}

/// Every sound in the manifest, loaded once.
#[derive(Resource)]
pub struct Sfx {
    sounds: HashMap<String, Sound>,
    last: HashMap<&'static str, Instant>,
    rng: u64,
    silent: bool,
    seen: Seen,
}

#[derive(serde::Deserialize)]
struct Entry {
    name: String,
    file: String,
    group: String,
}

impl FromWorld for Sfx {
    fn from_world(world: &mut bevy::ecs::world::World) -> Sfx {
        let silent = std::env::var("GAHT_SHOT").is_ok();
        let mut sounds: HashMap<String, Sound> = HashMap::new();
        if !silent {
            let path = super::models::assets_dir().join("sounds").join("manifest.json");
            let list: Vec<Entry> = match std::fs::read_to_string(&path).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string())) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("no sounds ({}: {e})", path.display());
                    Vec::new()
                }
            };
            let server = world.resource::<AssetServer>();
            // Takes in name order, so `_1` is first.
            let mut list = list;
            list.sort_by(|a, b| a.name.cmp(&b.name));
            for e in list {
                let handle: Handle<AudioSource> = server.load(format!("sounds/{}", e.file));
                sounds.entry(base_name(&e.name).to_string()).or_insert_with(|| Sound { takes: Vec::new(), group: Group::from_manifest(&e.group) }).takes.push(handle);
            }
        }
        Sfx { sounds, last: HashMap::new(), rng: 0x9E37_79B9_7F4A_7C15, silent, seen: Seen::default() }
    }
}

impl Sfx {
    /// A number in 0..1 (xorshift; presentation only).
    fn roll(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        (x >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Play `name` (one of its takes) at `gain` (master × distance), unless
    /// it played just now or the sound is off.
    pub fn play(&mut self, commands: &mut Commands, name: &'static str, gain: f32) {
        if self.silent || gain <= 0.0 {
            return;
        }
        let now = Instant::now();
        if self.last.get(name).is_some_and(|t| now.duration_since(*t) < AGAIN) {
            return;
        }
        let Some(group) = self.sounds.get(name).filter(|s| !s.takes.is_empty()).map(|s| s.group) else { return };
        self.last.insert(name, now);
        let n = self.sounds[name].takes.len();
        let k = ((self.roll() * n as f32) as usize).min(n - 1);
        let (mut volume, mut speed) = (gain * group.volume(), 1.0);
        if group == Group::World {
            speed += (self.roll() * 2.0 - 1.0) * PITCH_JITTER;
            volume *= 1.0 - self.roll() * VOLUME_JITTER;
        }
        let handle = self.sounds[name].takes[k].clone();
        commands.spawn((AudioPlayer::new(handle), PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)).with_speed(speed)));
    }
}

/// How loud a world sound is from where the camera listens: 1 near, falling
/// to 0 at `FAR`.
fn falloff(d: f32) -> f32 {
    let k = ((FAR - d) / (FAR - NEAR)).clamp(0.0, 1.0);
    k * k
}

/// One fighter as last seen.
#[derive(Clone, Copy)]
struct Fought {
    hp: f32,
    down: bool,
    shots: u16,
    /// The blow (its landing time) and the cast (its finish) already heard.
    swing: f64,
    cast: f64,
}

/// What the window showed last frame, for the sounds that come from a change.
#[derive(Default)]
struct Seen {
    first: bool,
    loads: u32,
    panels: [bool; 9],
    loot: Option<bool>,
    sel: Vec<gahturiyu_sim::sim::person::PersonId>,
    fights: HashMap<(u32, usize), Fought>,
}

/// Play what was asked for this frame, and what changed since the last.
pub fn update(mut commands: Commands, mut game: ResMut<Game>, mut sfx: ResMut<Sfx>, settings: Res<Settings>) {
    let game = &mut *game;
    let master = settings.sound.gain();
    let mut queue = std::mem::take(&mut game.sounds.0);
    let w = &game.world;

    // Panels: (open now, the sound it opens with). Closing any is `ui_close`.
    let panels = [
        (game.inv.is_some(), "pack_open"),
        (game.craft.is_some(), "craft_open"),
        (game.book.is_some(), "page_turn"),
        (game.journal, "page_turn"),
        (game.town.is_some(), "ui_open"),
        (game.options, "ui_open"),
        (game.build, "ui_open"),
        (game.keys, "ui_open"),
        (w.talk.is_some(), "talk_open"),
    ];
    // The loot window: a chest's lid, or a body's things.
    let loot = w.squad.members.iter().find_map(|&m| w.source_now(m)).map(|s| matches!(s, Source::Chest(_)));
    let sel = game.sel.who(w);
    let seen = &mut sfx.seen;
    let fresh = !seen.first || seen.loads != game.loads;
    if !fresh {
        for (k, &(on, open)) in panels.iter().enumerate() {
            if on && !seen.panels[k] {
                queue.push((open, None));
            } else if !on && seen.panels[k] {
                queue.push(("ui_close", None));
            }
        }
        match (seen.loot, loot) {
            (None, Some(true)) => queue.push(("lid_open", None)),
            (None, Some(false)) => queue.push(("ui_open", None)),
            (Some(true), None) => queue.push(("lid_close", None)),
            (Some(false), None) => queue.push(("ui_close", None)),
            _ => {}
        }
        if sel != seen.sel {
            queue.push(("ui_select", None));
        }
    }
    for (k, (on, _)) in panels.iter().enumerate() {
        seen.panels[k] = *on;
    }
    seen.loot = loot;
    seen.sel = sel;

    // Fights near the camera: a blow's arc starting, an arrow loosed, a cast
    // begun, hurt, someone falling. (The same moments `cues.rs` draws.)
    let ear = game.orbit.look_at().lerp(game.orbit.eye(), LISTENER);
    let near = |p: V2| game.view == View::Scene && to3(p, w.terrain.surface(p)).distance(ear) < FAR;
    let mut live = Vec::new();
    for b in w.battles.iter().filter(|b| !b.over) {
        for (i, f) in b.fighters.iter().enumerate() {
            let key = (b.id, i);
            live.push(key);
            let down = f.ko || f.dead;
            let hp: f32 = f.hp.iter().sum();
            // A hand weapon's blow, once its arc starts; a spell, once begun.
            let swing = match f.act {
                Act::Swing { lands, .. } if f.weapon.range <= 0.0 && f.is_person() && b.time >= lands - f.weapon.windup.max(0.2) as f64 * super::cues::ARC_SHARE => Some(lands),
                _ => None,
            };
            let cast = match f.act {
                Act::Cast { done, .. } => Some(done),
                _ => None,
            };
            let first = !seen.fights.contains_key(&key);
            let e = seen.fights.entry(key).or_insert(Fought { hp, down, shots: f.shots, swing: f64::NAN, cast: f64::NAN });
            let mut heard: Vec<&'static str> = Vec::new();
            if let Some(l) = swing.filter(|&l| l != e.swing) {
                e.swing = l;
                heard.push("swing");
            }
            if let Some(d) = cast.filter(|&d| d != e.cast) {
                e.cast = d;
                heard.push("cast");
            }
            if f.shots > e.shots {
                heard.push("bow_release");
            }
            if hp <= e.hp - 0.5 {
                heard.push("hit_flesh");
            }
            if down && !e.down {
                heard.push("knock_down");
            }
            (e.hp, e.down, e.shots) = (hp, down, f.shots);
            if !first && !fresh && near(f.pos) {
                queue.extend(heard.into_iter().map(|n| (n, Some(f.pos))));
            }
        }
    }
    seen.fights.retain(|k, _| live.contains(k));
    seen.first = true;
    seen.loads = game.loads;

    for (name, at) in queue {
        let gain = match at {
            None => master,
            Some(p) if game.view == View::Scene => master * falloff(to3(p, w.terrain.surface(p)).distance(ear)),
            Some(_) => 0.0,
        };
        sfx.play(&mut commands, name, gain);
    }
}
