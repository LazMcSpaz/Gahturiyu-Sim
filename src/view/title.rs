//! The title screen (Laz, 13 Oct): what the app opens on. Continue (the
//! newest save), New game (the opening on Salt Isle), Load game (the saves
//! folder), the Testing arena (a fight in the test quarter's yard, a demo
//! outpost under way, or the test quarter itself), Settings, Quit. The
//! world behind it stands still until a choice is made. The Escape menu's
//! "Title screen" brings it back.
//!
//! Drawing only: the choices come back as `TitleAct` and `app.rs` acts.

use bevy::math::Vec2;
use std::path::PathBuf;

use super::hud::{Canvas, Face};
use super::kit::{self, Kind};
use super::palette::{eg, ega, BRASS, DIM, GOLD, TEXT};
use super::squadui::{Bx, Click};

/// Which page is up.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Main,
    Arena,
    Load,
}

/// What the testing arena offers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arena {
    /// A fight in the test quarter's yard.
    Combat,
    /// A demo outpost under way, with a base to run.
    Building,
    /// The test quarter itself (the dev panel on Insert).
    TestArea,
}

impl Arena {
    pub fn label(self) -> &'static str {
        match self {
            Arena::Combat => "Test combat",
            Arena::Building => "Test building",
            Arena::TestArea => "The test area",
        }
    }

    pub fn about(self) -> &'static str {
        match self {
            Arena::Combat => "The squad of four in the test quarter's yard; pick foes in the dev panel",
            Arena::Building => "An outpost under way, its store stocked: huts up, a palisade going",
            Arena::TestArea => "The walled test quarter: the yard, the culture row; Insert for the dev panel",
        }
    }
}

/// What the player chose.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TitleAct {
    /// Back to the game under way (the title came from the game menu).
    Resume,
    Continue(PathBuf),
    NewGame,
    Load(PathBuf),
    Arena(Arena),
    Settings,
    Quit,
}

/// A save on disk.
#[derive(Clone, Debug)]
pub struct SaveFile {
    pub path: PathBuf,
    pub name: String,
    pub modified: std::time::SystemTime,
    /// Made by this version of the game (an older format won't load).
    pub loads: bool,
}

/// The title screen's state.
pub struct Title {
    pub page: Page,
    pub saves: Vec<SaveFile>,
    /// The rules as they stood when the title came up: switches flipped in
    /// the settings from here carry into whatever world comes next.
    pub rules_seen: Option<gahturiyu_sim::sim::rules::Rules>,
    /// Brought up from the game menu: a game is under way to go back to.
    pub from_game: bool,
}

/// Where saves live: `saves/` beside `assets/`.
pub fn saves_dir() -> PathBuf {
    let assets = super::models::assets_dir();
    assets.parent().map(|p| p.to_path_buf()).unwrap_or_default().join("saves")
}

/// Every `.sav` in the saves folder, newest first.
pub fn list_saves() -> Vec<SaveFile> {
    let mut out = Vec::new();
    if let Ok(dir) = std::fs::read_dir(saves_dir()) {
        for e in dir.flatten() {
            let path = e.path();
            if path.extension().and_then(|x| x.to_str()) != Some("sav") {
                continue;
            }
            let Ok(meta) = e.metadata() else { continue };
            let modified = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
            let name = path.file_stem().and_then(|x| x.to_str()).unwrap_or("save").to_string();
            let loads = gahturiyu_sim::sim::save::loadable(&path);
            out.push(SaveFile { path, name, modified, loads });
        }
    }
    out.sort_by(|a, b| b.modified.cmp(&a.modified).then(a.name.cmp(&b.name)));
    out
}

/// "3 minutes ago", "2 hours ago", "4 days ago".
fn ago(t: std::time::SystemTime) -> String {
    let secs = std::time::SystemTime::now().duration_since(t).map(|d| d.as_secs()).unwrap_or(0);
    let (n, unit) = if secs < 90 {
        return "just now".into();
    } else if secs < 90 * 60 {
        (secs / 60, "minute")
    } else if secs < 36 * 3600 {
        (secs / 3600, "hour")
    } else {
        (secs / 86400, "day")
    };
    format!("{n} {unit}{} ago", if n == 1 { "" } else { "s" })
}

impl Title {
    pub fn new() -> Title {
        Title { page: Page::Main, saves: list_saves(), rules_seen: None, from_game: false }
    }

    pub fn on(page: Page) -> Title {
        Title { page, saves: list_saves(), rules_seen: None, from_game: false }
    }

    /// From the game menu: the same, with the game under way to go back to.
    pub fn from_game() -> Title {
        Title { from_game: true, ..Title::new() }
    }

    /// Escape: back a page, or nothing on the main page (the title stays
    /// until a choice is made, unless there's a game to go back to: the
    /// caller closes it then).
    pub fn back(&mut self) -> bool {
        if self.page == Page::Main {
            return false;
        }
        self.page = Page::Main;
        true
    }
}

impl Default for Title {
    fn default() -> Self {
        Title::new()
    }
}

const W: f32 = 380.0;
const ROW: f32 = 46.0;
/// Saves listed on the Load page at most.
const LIST: usize = 10;

/// Only the title's dimmed backdrop, under the settings opened from it (the
/// title's buttons would show through the panel). The whole screen's box.
pub fn backdrop(c: &Canvas, size: Vec2) -> Bx {
    c.rect(0.0, 0.0, size.x, size.y, ega([0.0, 0.0, 0.0], 0.55));
    Bx::new(0.0, 0.0, size.x, size.y)
}

/// Draw the title screen over everything. Returns the choice made, and the
/// panel's box (the whole screen: nothing underneath takes a click).
pub fn draw(c: &Canvas, t: &mut Title, click: Option<Click>, size: Vec2) -> (Option<TitleAct>, Bx) {
    c.rect(0.0, 0.0, size.x, size.y, ega([0.0, 0.0, 0.0], 0.55));
    let rows: Vec<(Option<TitleAct>, String, Kind, bool, Option<String>)> = match t.page {
        Page::Main => {
            // The newest save this version can load.
            let newest = t.saves.iter().find(|s| s.loads).cloned();
            let mut rows = Vec::new();
            if t.from_game {
                rows.push((Some(TitleAct::Resume), "Back to the game".to_string(), Kind::Primary, true, Some("Anything else leaves it: save first (F8)".into())));
            }
            rows.push((
                newest.as_ref().map(|s| TitleAct::Continue(s.path.clone())),
                "Continue".to_string(),
                if t.from_game { Kind::Plain } else { Kind::Primary },
                newest.is_some(),
                newest.as_ref().map(|s| format!("{}, {}", s.name, ago(s.modified))),
            ));
            rows.push((Some(TitleAct::NewGame), "New game".into(), if newest.is_some() || t.from_game { Kind::Plain } else { Kind::Primary }, true, Some("Wrecked on Salt Isle's shore".into())));
            rows.push((Some(TitleAct::Load(PathBuf::new())), "Load game".into(), Kind::Plain, newest.is_some(), None));
            rows.push((Some(TitleAct::Arena(Arena::TestArea)), "Testing arena".into(), Kind::Plain, true, None));
            rows.push((Some(TitleAct::Settings), "Settings".into(), Kind::Plain, true, None));
            rows.push((Some(TitleAct::Quit), "Quit".into(), Kind::Plain, true, None));
            rows
        }
        Page::Arena => {
            let mut rows: Vec<_> = [Arena::Combat, Arena::Building, Arena::TestArea].into_iter().map(|a| (Some(TitleAct::Arena(a)), a.label().to_string(), Kind::Plain, true, Some(a.about().to_string()))).collect();
            rows.push((None, "Back".into(), Kind::Plain, true, None));
            rows
        }
        Page::Load => {
            // Saves from an older version are shown, dimmed, so they aren't
            // a mystery; they can't be loaded.
            let mut rows: Vec<_> = t.saves.iter().take(LIST).map(|s| (Some(TitleAct::Load(s.path.clone())), s.name.clone(), Kind::Plain, s.loads, Some(if s.loads { ago(s.modified) } else { format!("{}, from an older version", ago(s.modified)) }))).collect();
            if rows.is_empty() {
                rows.push((None, "No saves yet".into(), Kind::Plain, false, Some("F8 or the menu's Save makes one".into())));
            }
            rows.push((None, "Back".into(), Kind::Plain, true, None));
            rows
        }
    };
    let subs = rows.iter().filter(|r| r.4.is_some()).count() as f32;
    let h = 110.0 + rows.len() as f32 * ROW + subs * 18.0;
    let (x, y) = ((size.x - W) / 2.0, (size.y - h) / 2.0);
    c.frame_box(x, y, W, h);
    let title = "Gahturiyu";
    let tw = c.styled_width(title, 34.0, Face::Title, 4.0);
    c.styled(title, x + (W - tw) / 2.0, y + 52.0, 34.0, eg(GOLD), Face::Title, 4.0);
    let sub = match t.page {
        Page::Main => "a world that runs whether you watch or not",
        Page::Arena => "testing arena",
        Page::Load => "load a game",
    };
    let sw = c.width(sub, 14.0);
    c.text(sub, x + (W - sw) / 2.0, y + 74.0, 14.0, DIM);
    c.rule_in(x + 30.0, x + W - 30.0, y + 86.0, 1.0, ega(BRASS, 0.6), true);
    let mut by = y + 100.0;
    let mut picked = None;
    for (act, label, kind, enabled, under) in rows {
        let b = Bx::new(x + 40.0, by, W - 80.0, ROW - 10.0);
        kit::button(c, &b, &label, kind, enabled);
        if enabled && click.is_some_and(|k| b.contains(k.at)) {
            match (&act, t.page) {
                // Pages, not choices.
                (Some(TitleAct::Load(p)), Page::Main) if p.as_os_str().is_empty() => t.page = Page::Load,
                (Some(TitleAct::Arena(_)), Page::Main) => t.page = Page::Arena,
                (None, _) => t.page = Page::Main,
                (Some(a), _) => picked = Some(a.clone()),
            }
        }
        by += ROW;
        if let Some(u) = under {
            // (Under the button, not on its edge.)
            c.text(&u, x + 44.0, by + 2.0, 12.0, if enabled { TEXT } else { DIM });
            by += 18.0;
        }
    }
    c.text("Esc goes back", x + 14.0, y + h - 12.0, 12.0, DIM);
    (picked, Bx::new(0.0, 0.0, size.x, size.y))
}

/// A name for a save made now: the world's day and hour and its seed, so
/// the list reads as the game's own time.
pub fn dated_name(day: i64, hour: u32, seed: u64) -> String {
    format!("day{day}-{hour:02}h-seed{seed}")
}
