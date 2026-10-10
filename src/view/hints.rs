//! First-hour hints (the playable-MVP list): short tips that turn up when
//! they're useful (the first fight, the first beaten foe, a full pack,
//! nightfall), one at a time, each once. Click a tip to put it away; it
//! goes on its own after a while. Which have been seen is kept in
//! `hints.txt` beside `settings.txt` (delete it to see them all again).
//!
//! Drawing only: the conditions read the world, and nothing here changes it.

use std::time::{Duration, Instant};

use gahturiyu_sim::sim::{condition, stealth, World};

use super::hud::Canvas;
use super::palette::{DIM, GOLD, TEXT};
use super::squadui::{Bx, Click};

/// How long a tip stays up if it's not clicked away.
const SHOW_FOR: Duration = Duration::from_secs(45);
/// Wrap tips at about this many characters.
const WRAP: usize = 74;

/// The tips, in the order they're considered.
const HINTS: &[(&str, &str)] = &[
    ("welcome", "Welcome. Left-click the ground to walk. Click a card along the bottom (or F1–F4) to choose who takes orders; ` picks everyone again. Hover over anything to see what it is."),
    ("town", "A town. Click a townsperson to talk: merchants trade, some have work, and a few restless ones will join the squad if asked."),
    ("work", "Short of coin? Every town has a woodlot (and some a mine) nearby, marked by a post. Click it and the selected work until their packs are full."),
    ("fight", "A fight! Click an enemy to set the selected on them. Z sneaks; T lights a torch. The ring under each fighter points the way they face."),
    ("loot", "A beaten foe: click them and someone goes through their things. Bandits carry coin."),
    ("full", "A full pack slows you down. Talk to a merchant (\"What have you got?\") and use Sell all."),
    ("hungry", "Someone's hungry. They eat from their pack when they need to: buy food from a merchant, or hunt (click a wild animal) and cut up what you kill."),
    ("night", "Night's coming. Press N to rest (a tent in someone's pack makes it a better sleep), or T for torches if you'd rather keep going."),
    ("beaten", "Beaten and robbed. Rest until you can stand (N), get some gear, and go and take it back from their camp."),
];

pub struct Hints {
    seen: Vec<String>,
    showing: Option<(usize, Instant)>,
    /// Kept in `hints.txt`, and shown as they fall due (both off for
    /// screenshots, which only show a tip they ask for).
    persist: bool,
    /// Checked every so many frames.
    tick: u32,
}

fn path() -> std::path::PathBuf {
    let assets = super::models::assets_dir();
    assets.parent().map(|p| p.join("hints.txt")).unwrap_or_else(|| "hints.txt".into())
}

impl Hints {
    pub fn load(persist: bool) -> Hints {
        let seen = if persist { std::fs::read_to_string(path()).map(|t| t.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()).unwrap_or_default() } else { Vec::new() };
        Hints { seen, showing: None, persist, tick: 0 }
    }

    /// Show this tip now (screenshots).
    pub fn force(&mut self, id: &str) {
        if let Some(i) = HINTS.iter().position(|h| h.0 == id) {
            self.showing = Some((i, Instant::now()));
        }
    }

    fn done(&mut self, i: usize) {
        self.showing = None;
        let id = HINTS[i].0.to_string();
        if !self.seen.contains(&id) {
            self.seen.push(id);
            if self.persist {
                let _ = std::fs::write(path(), self.seen.join("\n"));
            }
        }
    }
}

/// Is this tip's moment here?
fn due(w: &World, id: &str) -> bool {
    let near = |p: gahturiyu_sim::sim::geo::V2, r: f32| w.squad.pos.dist(p) <= r;
    match id {
        "welcome" => true,
        "town" => w.settlements.iter().any(|s| near(s.pos, s.reach + 60.0)),
        "work" => w.deposits.iter().any(|d| near(d.pos, 400.0)),
        "fight" => w.squad_battle().is_some(),
        "loot" => w.squad_battle().is_none() && w.groups.iter().filter(|g| g.band <= 1).flat_map(|g| g.members.iter()).any(|&m| w.can_loot(m) && near(w.person_pos(m), 80.0)),
        "full" => w.squad.members.iter().any(|&m| w.load_of(m) > 0.95),
        "hungry" => w.squad.members.iter().any(|&m| w.hunger_of(m).is_some_and(|h| condition::stage_of(h) != condition::HungerStage::Fed)),
        "night" => stealth::daylight(w.time) < 0.35,
        "beaten" => w.log.front().is_some_and(|l| l.1.starts_with("Beaten.") && w.time - l.0 < 3600.0),
        _ => false,
    }
}

fn wrap(text: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    for word in text.split(' ') {
        let line = out.last_mut().unwrap();
        if !line.is_empty() && line.chars().count() + word.chars().count() + 1 > WRAP {
            out.push(word.to_string());
        } else {
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
    }
    out
}

/// Pick the tip to show, and draw it. Returns its box (so clicks on it
/// don't reach the world).
pub fn show(c: &Canvas, h: &mut Hints, w: &World, click: Option<Click>) -> Option<Bx> {
    h.tick = h.tick.wrapping_add(1);
    if let Some((i, since)) = h.showing {
        if since.elapsed() > SHOW_FOR {
            h.done(i);
        }
    }
    if h.showing.is_none() && h.persist && h.tick % 30 == 0 {
        let next = HINTS.iter().position(|(id, _)| !h.seen.iter().any(|s| s == id) && due(w, id));
        if let Some(i) = next {
            // The welcome goes first, before anything else.
            if i == 0 || h.seen.iter().any(|s| s == "welcome") {
                h.showing = Some((i, Instant::now()));
            }
        }
    }
    let (i, _) = h.showing?;
    let mut lines: Vec<(String, super::palette::Rgb)> = vec![("Tip".to_string(), GOLD)];
    lines.extend(wrap(HINTS[i].1).into_iter().map(|l| (l, TEXT)));
    lines.push(("Click to put this away.".to_string(), DIM));
    let size = 15.0;
    let width = lines.iter().map(|(l, _)| c.width(l, size)).fold(0.0, f32::max) + 24.0;
    let r = Bx::from(c.panel(&lines, (c.w - width) / 2.0, 60.0, size));
    if let Some(ck) = click {
        if r.contains(ck.at) {
            h.done(i);
        }
    }
    Some(r)
}
