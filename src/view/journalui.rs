//! The journal (#127 UI-Journal; the feel plan's 5.6): a ledger of what the
//! squad owes, the work it has taken, and the jobs it has on, in that
//! order. What's owed is on top, in red: where the squad is wanted and who
//! of it is held to work (NM-28, NM-36). Each job is a row; a click on one
//! takes the view to where it points (its giver, the camp, the one the
//! letter is for). Finished jobs fold into one line at the foot.
//!
//! Drawing only: everything is read from the world, and the only thing it
//! hands back is a spot to look at.

use gahturiyu_sim::sim::{
    geo::V2,
    items,
    quests::{Quest, QuestKind, Stage},
    world::{DAY, HOUR},
    World,
};

use super::hud::{Canvas, Face};
use super::kit;
use super::palette::{eg, ega, Rgb, BRASS, BRASS_LIGHT, TEXT};
use super::squadui::{Bx, Click};

const HEAD: Rgb = [0.89, 0.76, 0.48];
const SUB: Rgb = [0.66, 0.60, 0.50];
const GROUP: Rgb = [0.66, 0.57, 0.42];
const OWED: Rgb = [0.90, 0.48, 0.38];
/// The most factions listed under Standing.
const MOST_STANDING: usize = 6;
const FAINT: Rgb = [0.55, 0.50, 0.42];

const W: f32 = 560.0;
const ROW_H: f32 = 44.0;
const LINE_H: f32 = 24.0;
const GROUP_H: f32 = 30.0;
/// The most jobs listed before the rest are counted.
const MOST_JOBS: usize = 8;

/// What a click in the journal asks for.
pub enum JournalAct {
    Close,
    /// Look at this spot.
    Look(V2),
}

/// A job's short title, what to do, and where it points.
pub fn job_card(w: &World, q: &Quest) -> (String, String, Option<V2>) {
    let giver = w.name_of(q.giver);
    let at_giver = Some(w.person_pos(q.giver));
    let line = w.quest_line(q).trim_end_matches('.').to_string();
    match (&q.kind, q.stage) {
        (_, Stage::Report) => ("Report back".into(), line, at_giver),
        (_, Stage::Done) => (format!("Done for {giver}"), line, None),
        (QuestKind::ClearCamp { at, .. }, _) => ("The bandit camp".into(), line, Some(*at)),
        (QuestKind::Fetch { item, count }, _) => (format!("{count} × {}", items::item(*item).name.to_lowercase()), line, at_giver),
        (QuestKind::Deliver { to }, _) => ("A sealed letter".into(), line, Some(w.person_pos(*to))),
        (QuestKind::Job { .. }, _) => (format!("A job for {giver}"), line, at_giver),
    }
}

/// What's owed: where the squad is wanted, and who of it is held to work.
pub fn owed_lines(w: &World) -> Vec<String> {
    let mut out = w.wanted_lines();
    for &m in &w.squad.members {
        let Some(b) = w.bond_of(m) else { continue };
        let town = &w.settlements[b.town as usize].name;
        let name = w.name_of(m);
        out.push(match w.bond_days_left(m) {
            Some(1) => format!("{name} works off a bond in {town}: a day left."),
            Some(n) => format!("{name} works off a bond in {town}: {n} days left."),
            None => format!("{name} is held in {town}."),
        });
    }
    out
}

/// A squad member's town work: who and where, then the hours, the pay and
/// how it stands now.
/// Who thinks what of the squad: every faction (and lord) whose regard has
/// moved off nothing, in words, most moved first.
pub fn standing_lines(w: &World) -> Vec<(String, &'static str, f32)> {
    use gahturiyu_sim::sim::factions::{standing_word, FACTIONS};
    let mut out: Vec<(String, &'static str, f32)> = FACTIONS.iter().enumerate().map(|(k, f)| (f.name.to_string(), w.standing_with(k as u8))).chain(w.factions.lords.iter().map(|l| (format!("Lord {}", l.name), l.standing))).filter(|x| x.1.abs() >= 1.0).map(|(n, s)| (n, standing_word(s), s)).collect();
    out.sort_by(|a, b| b.2.abs().total_cmp(&a.2.abs()).then(a.0.cmp(&b.0)));
    out
}

fn work_cards(w: &World) -> Vec<(String, String)> {
    let day = World::day_of(w.time);
    let h = (w.time.rem_euclid(DAY) / HOUR) as f32;
    w.society
        .contracts
        .iter()
        .filter(|c| w.squad.index(c.member).is_some())
        .map(|c| {
            let what = c.post.map(|j| j.name().to_string()).unwrap_or_else(|| "Guard".into());
            let place = w.society.towns[c.town as usize].places.get(c.place as usize).map(|p| p.kind.name().to_lowercase()).unwrap_or_else(|| "town".into());
            let now = match w.on_shift(c.member) {
                Some(true) => "At work now.",
                Some(false) => "Should be there now.",
                None if day < c.first_day => "Starts tomorrow.",
                None if h < c.hours.0 => "Later today.",
                None => "Done for today.",
            };
            (
                format!("{}: {what} at the {place}, {}", w.name_of(c.member), w.settlements[c.town as usize].name),
                format!("{:02.0}:00 to {:02.0}:00, {:.0} coin a day. {now}", c.hours.0, c.hours.1, c.pay),
            )
        })
        .collect()
}

/// Draw the journal; what was clicked, and the box it covers.
pub fn journal(c: &Canvas, w: &World, click: Option<Click>) -> (Option<JournalAct>, Bx) {
    let clicked = |bx: &Bx| click.map(|k| bx.contains(k.at) && !k.right).unwrap_or(false);
    let owed = owed_lines(w);
    let work = work_cards(w);
    let standing = standing_lines(w);
    let standing_shown = standing.len().min(MOST_STANDING);
    let mut open: Vec<&Quest> = w.quests.iter().filter(|q| q.stage != Stage::Done).collect();
    // Ones to report on first: they're one walk from paid.
    open.sort_by_key(|q| q.stage != Stage::Report);
    let done = w.quests.iter().filter(|q| q.stage == Stage::Done).count();
    let shown = open.len().min(MOST_JOBS);
    let empty = owed.is_empty() && work.is_empty() && open.is_empty() && standing.is_empty();

    // Its height, from what's in it.
    let mut h = 74.0 + 14.0;
    if !owed.is_empty() {
        h += GROUP_H + owed.len() as f32 * LINE_H;
    }
    if !work.is_empty() {
        h += GROUP_H + work.len() as f32 * ROW_H;
    }
    if !standing.is_empty() {
        h += GROUP_H + standing_shown as f32 * LINE_H;
    }
    if !open.is_empty() {
        h += GROUP_H + shown as f32 * ROW_H;
    }
    if open.len() > shown || done > 0 {
        h += LINE_H;
    }
    if empty {
        h += LINE_H;
    }
    let wd = W.min(c.w - 24.0);
    let bottom = c.h - super::frame::BOTTOM_CLEAR - 12.0;
    let h = h.min(bottom - 60.0);
    let r = Bx::new(12.0, bottom - h, wd, h);
    c.frame_box(r.x, r.y, r.w, r.h);
    c.styled("JOURNAL", r.x + 18.0, r.y + 36.0, 22.0, eg(HEAD), Face::Title, 5.0);
    c.styled(&format!("Day {}", World::day_of(w.time) + 1), r.x + 18.0, r.y + 56.0, 14.0, eg(SUB), Face::Italic, 0.0);
    let mut act = None;
    let close = Bx::new(r.x + r.w - 32.0, r.y + 8.0, 26.0, 26.0);
    let cl = kit::look(&close);
    c.styled("×", close.x + 7.0, close.y + 21.0 + kit::nudge(cl), 22.0, eg(if cl.hot { BRASS_LIGHT } else { GROUP }), Face::Body, 0.0);
    if clicked(&close) {
        act = Some(JournalAct::Close);
    }
    let x = r.x + 18.0;
    let mut y = r.y + 74.0;
    let fits = |y: f32, need: f32| y + need <= r.y + r.h - 6.0;
    let group = |y: &mut f32, name: &str, col: Rgb| {
        c.styled(name, x, *y + 18.0, 13.0, eg(col), Face::Title, 3.0);
        c.rule(x, r.x + r.w - 18.0, *y + 24.0, 0.8, ega(col, 0.35));
        *y += GROUP_H;
    };
    if empty {
        c.styled("Nothing taken on. People with work will say so if you ask.", x, y + 16.0, 15.0, eg(FAINT), Face::Italic, 0.0);
        return (act, r);
    }
    if !owed.is_empty() {
        group(&mut y, "OWED", OWED);
        for l in &owed {
            if !fits(y, LINE_H) {
                break;
            }
            c.styled(l, x + 4.0, y + 17.0, 15.0, eg(OWED), Face::Body, 0.0);
            y += LINE_H;
        }
    }
    if !standing.is_empty() && fits(y, GROUP_H + LINE_H) {
        group(&mut y, "STANDING", GROUP);
        for (name, word, s) in standing.iter().take(standing_shown) {
            if !fits(y, LINE_H) {
                break;
            }
            c.styled(name, x + 4.0, y + 17.0, 15.0, eg(TEXT), Face::Body, 0.0);
            let col = match (*word, *s < 0.0) {
                ("Neutral", _) => SUB,
                (_, true) => OWED,
                _ => [0.61, 0.76, 0.48],
            };
            c.styled_right(word, r.x + r.w - 22.0, y + 17.0, 15.0, eg(col), Face::Italic, 0.0);
            y += LINE_H;
        }
    }
    if !work.is_empty() && fits(y, GROUP_H + ROW_H) {
        group(&mut y, "WORK", GROUP);
        for (a, b) in &work {
            if !fits(y, ROW_H) {
                break;
            }
            c.styled(a, x + 4.0, y + 18.0, 15.0, eg(TEXT), Face::Body, 0.0);
            c.styled(b, x + 4.0, y + 36.0, 13.0, eg(SUB), Face::Body, 0.0);
            y += ROW_H;
        }
    }
    if !open.is_empty() && fits(y, GROUP_H + ROW_H) {
        group(&mut y, "JOBS", GROUP);
        for q in open.iter().take(shown) {
            if !fits(y, ROW_H) {
                break;
            }
            let (title, line, at) = job_card(w, q);
            let row = Bx::new(r.x + 8.0, y + 1.0, r.w - 16.0, ROW_H - 2.0);
            let l = kit::row(c, &row, false);
            let dy = kit::nudge(l);
            let report = q.stage == Stage::Report;
            c.diamond(x + 4.0, y + 13.0 + dy, 3.5, ega(if report { BRASS_LIGHT } else { BRASS }, if report { 1.0 } else { 0.6 }));
            c.styled(&title, x + 14.0, y + 18.0 + dy, 15.0, eg(if report { BRASS_LIGHT } else { TEXT }), Face::Caps, 0.0);
            let room = row.w - 30.0;
            let line = fit(c, &line, 13.0, room);
            c.styled(&line, x + 14.0, y + 36.0 + dy, 13.0, eg(SUB), Face::Body, 0.0);
            if let (Some(p), true) = (at, clicked(&row)) {
                act = Some(JournalAct::Look(p));
            }
            y += ROW_H;
        }
    }
    let more = open.len() - shown;
    let foot = match (more, done) {
        (0, 0) => None,
        (0, 1) => Some("One job done.".to_string()),
        (0, d) => Some(format!("{d} jobs done.")),
        (m, 0) => Some(format!("And {m} more.")),
        (m, d) => Some(format!("And {m} more; {d} done.")),
    };
    if let Some(f) = foot {
        if fits(y, LINE_H) {
            c.styled(&f, x + 4.0, y + 17.0, 13.0, eg(FAINT), Face::Italic, 0.0);
        }
    }
    (act, r)
}

/// A line cut to fit `room` pixels, with an ellipsis if it had to be.
fn fit(c: &Canvas, s: &str, size: f32, room: f32) -> String {
    if c.styled_width(s, size, Face::Body, 0.0) <= room {
        return s.to_string();
    }
    let mut out: String = s.to_string();
    while !out.is_empty() && c.styled_width(&format!("{out}…"), size, Face::Body, 0.0) > room {
        out.pop();
    }
    format!("{}…", out.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gahturiyu_sim::sim::worldgen;

    /// NM-28 and NM-36: what the squad owes is in the journal, the bounty
    /// and the bond both, with the days left.
    #[test]
    fn the_journal_says_what_is_owed() {
        let mut w = worldgen::generate(1);
        assert!(owed_lines(&w).is_empty());
        let town = w.settlements[0].id;
        let name = w.settlements[0].name.clone();
        w.bounty.insert(town, 40.0);
        let m = w.squad.members[1];
        let t = w.time;
        w.bond(m, town, None, t, t + 3.5 * DAY);
        let lines = owed_lines(&w);
        assert!(lines.iter().any(|l| l == &format!("Wanted in {name}: 40 coin owed.")), "{lines:?}");
        assert!(lines.iter().any(|l| l == &format!("{} works off a bond in {name}: 4 days left.", w.name_of(m))), "{lines:?}");
    }
}
