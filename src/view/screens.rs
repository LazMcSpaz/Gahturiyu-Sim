//! Which screen is open (drawing only). Laz's P10, P11, P13 and P17:
//! opening one screen closes any other, Escape closes the one that's open
//! (or, with nothing open, brings up the game menu), and a screen about one
//! squad member follows the selection.
//!
//! Screens are still opened the way they always were (a key, a button, a
//! click in a panel). `settle` runs after both the keys and the panels each
//! frame, sees what's open now, and if something new has opened it closes
//! the rest. So a new screen only needs a line in `Screen`, `is_open` and
//! `close`, and no opener has to know about the others.

use bevy::math::Vec2;

use super::app::Game;
use super::hud::{Canvas, Face};
use super::kit::{self, Kind};
use super::palette::{ega, eg, BRASS, GOLD, TEXT};
use super::squadui::{Bx, Click};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Pack,
    Craft,
    Spells,
    Stats,
    Journal,
    Town,
    Build,
    Options,
    Keys,
    Menu,
}

impl Screen {
    pub const ALL: [Screen; 10] = [Screen::Pack, Screen::Craft, Screen::Spells, Screen::Stats, Screen::Journal, Screen::Town, Screen::Build, Screen::Options, Screen::Keys, Screen::Menu];
}

/// What `settle` remembers between frames.
#[derive(Default)]
pub struct Screens {
    /// What was open after last frame's settling.
    open: Option<Screen>,
    /// Whom the member screens showed last frame (the selection's lead).
    lead: Option<gahturiyu_sim::sim::person::PersonId>,
}

/// The game menu (Escape with nothing else open), and a choice waiting to
/// be confirmed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct GameMenu {
    pub confirm: Option<MenuAct>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuAct {
    Resume,
    Settings,
    Keys,
    Save,
    Load,
    NewGame,
    Quit,
}

pub fn is_open(game: &Game, s: Screen) -> bool {
    match s {
        Screen::Pack => game.inv.is_some(),
        Screen::Craft => game.craft.is_some(),
        Screen::Spells => game.book.is_some(),
        Screen::Stats => game.stats.is_some(),
        Screen::Journal => game.journal,
        Screen::Town => game.town.is_some(),
        Screen::Build => game.build,
        Screen::Options => game.options,
        Screen::Keys => game.keys,
        Screen::Menu => game.game_menu.is_some(),
    }
}

pub fn close(game: &mut Game, s: Screen) {
    match s {
        Screen::Pack => game.inv = None,
        Screen::Craft => {
            game.craft = None;
            game.making = None;
        }
        Screen::Spells => game.book = None,
        Screen::Stats => game.stats = None,
        Screen::Journal => game.journal = false,
        Screen::Town => game.town = None,
        Screen::Build => {
            game.build = false;
            game.placing = None;
        }
        Screen::Options => game.options = false,
        Screen::Keys => game.keys = false,
        Screen::Menu => game.game_menu = None,
    }
}

/// The screen that's open, if any.
pub fn open_now(game: &Game) -> Option<Screen> {
    // (Several can be open only within a frame, before `settle`.)
    Screen::ALL.into_iter().find(|&s| is_open(game, s))
}

/// Once after the keys and once after the panels, each frame: keep one
/// screen open (the newest), and keep member screens on the selection.
pub fn settle(game: &mut Game) {
    let now: Vec<Screen> = Screen::ALL.into_iter().filter(|&s| is_open(game, s)).collect();
    let newest = now.iter().copied().find(|&s| Some(s) != game.screens.open).or(game.screens.open.filter(|s| now.contains(s)));
    for s in now {
        if Some(s) != newest {
            close(game, s);
        }
    }
    game.screens.open = newest;
    // A different member picked: the member screens turn to them (P17).
    let lead = game.sel.lead(&game.world);
    if lead != game.screens.lead {
        game.screens.lead = lead;
        if let Some(p) = lead {
            for slot in [&mut game.inv, &mut game.craft, &mut game.book, &mut game.stats] {
                if slot.is_some() {
                    *slot = Some(p);
                }
            }
        }
    }
}

/// Escape, after the things that hold the mouse (a right-click menu, a
/// building being placed, a spell being aimed, a talk) have had their turn:
/// close what's open, or open the game menu if nothing is.
pub fn escape(game: &mut Game) {
    match open_now(game) {
        Some(Screen::Menu) if game.game_menu.is_some_and(|m| m.confirm.is_some()) => game.game_menu = Some(GameMenu::default()),
        Some(s) => close(game, s),
        None => game.game_menu = Some(GameMenu::default()),
    }
}

const ITEMS: [(MenuAct, &str); 7] = [(MenuAct::Resume, "Resume"), (MenuAct::Settings, "Settings"), (MenuAct::Keys, "Keys"), (MenuAct::Save, "Save"), (MenuAct::Load, "Load"), (MenuAct::NewGame, "New game"), (MenuAct::Quit, "Quit")];
const W: f32 = 320.0;
const ROW: f32 = 46.0;

/// Draw the game menu in the middle of the screen. Returns what was chosen
/// (confirmed, for New game and Quit) and the panel's box.
pub fn draw_menu(c: &Canvas, menu: &mut GameMenu, click: Option<Click>, size: Vec2) -> (Option<MenuAct>, Bx) {
    // The world behind it dims a little: the game is paused.
    c.rect(0.0, 0.0, size.x, size.y, ega([0.0, 0.0, 0.0], 0.35));
    let ask = menu.confirm.map(|a| match a {
        MenuAct::NewGame => "Start a new world? Anything not saved is lost.",
        _ => "Quit? Anything not saved is lost.",
    });
    let rows = if ask.is_some() { 2 } else { ITEMS.len() };
    let h = 80.0 + rows as f32 * ROW + if ask.is_some() { 40.0 } else { 0.0 };
    let (x, y) = ((size.x - W) / 2.0, (size.y - h) / 2.0);
    c.frame_box(x, y, W, h);
    let title = "Gahturiyu";
    let tw = c.styled_width(title, 26.0, Face::Title, 3.0);
    c.styled(title, x + (W - tw) / 2.0, y + 44.0, 26.0, eg(GOLD), Face::Title, 3.0);
    c.rule_in(x + 30.0, x + W - 30.0, y + 58.0, 1.0, ega(BRASS, 0.6), true);
    let mut by = y + 74.0;
    let mut picked = None;
    if let Some(q) = ask {
        let qw = c.width(q, 15.0);
        c.text(q, x + (W - qw) / 2.0, by + 18.0, 15.0, TEXT);
        by += 40.0;
        let yes = menu.confirm.unwrap();
        for (act, label, kind) in [(Some(yes), if yes == MenuAct::Quit { "Quit" } else { "Start" }, Kind::Primary), (None, "Back", Kind::Plain)] {
            let b = Bx::new(x + 40.0, by, W - 80.0, ROW - 10.0);
            kit::button(c, &b, label, kind, true);
            if click.is_some_and(|k| b.contains(k.at)) {
                match act {
                    Some(a) => picked = Some(a),
                    None => menu.confirm = None,
                }
            }
            by += ROW;
        }
    } else {
        for (act, label) in ITEMS {
            let b = Bx::new(x + 40.0, by, W - 80.0, ROW - 10.0);
            kit::button(c, &b, label, if act == MenuAct::Resume { Kind::Primary } else { Kind::Plain }, true);
            if click.is_some_and(|k| b.contains(k.at)) {
                match act {
                    MenuAct::NewGame | MenuAct::Quit => menu.confirm = Some(act),
                    a => picked = Some(a),
                }
            }
            by += ROW;
        }
    }
    // (The whole screen counts as the panel: clicks never reach the world.)
    (picked, Bx::new(0.0, 0.0, size.x, size.y))
}
