//! Talking, close up (the feel plan's 5.1; mockup 1). The camera comes in
//! low beside the two of them and the rest of the screen falls away: the
//! town and the place along the top, the talk so far in a narrow column at
//! the side, and at the foot of the screen who is speaking, what they just
//! said in large type, and the squad member's replies, numbered.
//!
//! Replies that do something (take a job, pay, press someone) are gold;
//! ones already asked are dim; leaving is last. A reply that can't be asked
//! just now stays in the list, dim, with the plain reason beside it
//! (`World::locked_topics`). A small mark shows a reply that opens another
//! screen or leads to a list: coins for trade, an anvil for lessons, a
//! scroll for work.
//!
//! Nothing here decides anything: the replies are `World::topics`, the
//! words are the conversation's own lines, and a click or a number key is
//! reported back as the topic to ask. How they feel shows in what they say;
//! there is no number for it (RG-9). The column drops whole lines off its
//! top, never part of one (RG-19).

use bevy::math::Vec2;

use gahturiyu_sim::sim::{
    chances::Press,
    dialogue::Topic,
    geo::V2,
    jobs::Job,
    person::PersonId,
    speech,
    talk::Opt,
    World,
};

use super::cam::OrbitCam;
use super::hud::{Canvas, Face};
use super::palette::{eg, ega, Rgb, BRASS};
use super::squadui::{Bx, Click, NATIVE};

/// The talk screen while a talk is open.
#[derive(Clone, Debug, Default)]
pub struct Talking {
    /// Who it is between: (the squad's one, the other).
    pub pair: (PersonId, PersonId),
    /// What has been asked in this talk (drawn dim).
    pub asked: Vec<Topic>,
    /// How far down the replies are scrolled, in pixels.
    pub scroll: f32,
}

/// Their parting words, held on the screen for a moment after the talk is
/// over (a goodbye ends the talk in the world at once).
#[derive(Clone, Debug)]
pub struct Parting {
    pub with: PersonId,
    pub npc: PersonId,
    pub line: String,
    pub since: std::time::Instant,
}

/// How long parting words stay up, seconds.
pub const PARTING_SECS: f32 = 1.8;

pub enum TalkAct {
    Ask(Topic),
    /// A click while their parting words are up: enough, close.
    Done,
}

const DARK: Rgb = [0.043, 0.031, 0.024];
const NAME: Rgb = [0.94, 0.84, 0.60];
const SUB: Rgb = [0.66, 0.60, 0.50];
const LINE: Rgb = [0.95, 0.91, 0.82];
const PLAIN: Rgb = [0.91, 0.86, 0.77];
const DOES: Rgb = [0.94, 0.78, 0.44];
const ASKED: Rgb = [0.48, 0.44, 0.37];
const LEAVE: Rgb = [0.73, 0.67, 0.57];
const LOCKED: Rgb = [0.42, 0.38, 0.33];
const HOT: Rgb = [1.0, 0.95, 0.85];
const FAR: Rgb = [0.61, 0.56, 0.47];
const NEAR: Rgb = [0.73, 0.67, 0.57];
const HEAD: Rgb = [0.56, 0.48, 0.35];
const HINT: Rgb = [0.44, 0.39, 0.32];
const BAND: Rgb = [0.66, 0.57, 0.42];

/// The widest the words run, and a reply's row.
const COLUMN: f32 = 1100.0;
const ROW: f32 = 34.0;
/// The share of the screen's height kept clear for the two of them.
const SCENE_SHARE: f32 = 0.40;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Does,
    Ask,
    Asked,
    Leave,
    Locked,
}

#[derive(Clone, Copy, PartialEq)]
enum Mark {
    None,
    Coins,
    Anvil,
    Scroll,
    Lock,
}

struct Reply {
    topic: Option<Topic>,
    text: String,
    /// What follows in a quieter voice: a price, or why not.
    note: String,
    kind: Kind,
    mark: Mark,
}

fn kind_of(t: Topic) -> Kind {
    match t {
        Topic::Goodbye => Kind::Leave,
        Topic::Accept | Topic::Report(_) | Topic::Letter(_) | Topic::PayBounty | Topic::Learn(_) | Topic::Mend(..) | Topic::CraftLesson(_) | Topic::Order(..) | Topic::BuyOut(..) | Topic::TakePost(_) | Topic::TakeJob(_) | Topic::PostWork(..) | Topic::QuitWork | Topic::Hire(_) | Topic::Join(_) | Topic::Treat(_) | Topic::RentBeds(_) | Topic::ToNote | Topic::ToCoin | Topic::Buy(..) | Topic::Sell(..) | Topic::SellAll(..) => Kind::Does,
        Topic::Press(_, Press::Pay | Press::Threaten) => Kind::Does,
        Topic::Say(Opt::Help | Opt::Report | Opt::Persuade | Opt::Bribe | Opt::Threaten) => Kind::Does,
        _ => Kind::Ask,
    }
}

fn mark_of(t: Topic) -> Mark {
    match t {
        Topic::Trade => Mark::Coins,
        Topic::Lessons | Topic::CraftLesson(_) | Topic::Learn(_) => Mark::Anvil,
        Topic::Work | Topic::Board | Topic::Orders => Mark::Scroll,
        _ => Mark::None,
    }
}

/// "Teach me snare (30 coin)" as the words and the quieter part.
fn split_note(s: &str) -> (String, String) {
    if let (Some(a), true) = (s.rfind(" ("), s.ends_with(')')) {
        return (s[..a].to_string(), s[a + 2..s.len() - 1].to_string());
    }
    if let Some(a) = s.find(" — ") {
        return (s[..a].to_string(), s[a + " — ".len()..].to_string());
    }
    (s.to_string(), String::new())
}

/// A word, and its meaning if it is one of the speaker's own.
type Word = (String, Option<String>);

/// A line cut into words; a native word keeps its meaning, and punctuation
/// straight after one sticks to it.
fn words(line: &str) -> Vec<Word> {
    let mut out: Vec<Word> = Vec::new();
    for (text, meaning) in speech::spans(line) {
        match meaning {
            Some(m) => out.push((text, Some(m))),
            None => {
                let mut first = true;
                for piece in text.split(' ') {
                    if first && !piece.is_empty() && !text.starts_with(' ') {
                        if let Some(last) = out.last_mut() {
                            last.0.push_str(piece);
                            first = false;
                            continue;
                        }
                    }
                    first = false;
                    if !piece.is_empty() {
                        out.push((piece.to_string(), None));
                    }
                }
            }
        }
    }
    out
}

/// Words laid into rows no wider than `width`; the first row starts `lead`
/// in (after a name).
fn wrap(c: &Canvas, ws: Vec<Word>, size: f32, width: f32, lead: f32) -> Vec<Vec<Word>> {
    let space = size * 0.27;
    let mut rows = Vec::new();
    let mut row: Vec<Word> = Vec::new();
    let mut used = lead;
    for wd in ws {
        let ww = c.styled_width(&wd.0, size, Face::Body, 0.0);
        if used + ww > width && !row.is_empty() {
            rows.push(std::mem::take(&mut row));
            used = 0.0;
        }
        used += ww + space;
        row.push(wd);
    }
    rows.push(row);
    rows
}

/// One row of words; a native word is tinted and dotted under, and its
/// meaning is handed back when the mouse is on it.
fn draw_row(c: &Canvas, row: &[Word], x: f32, y: f32, size: f32, col: Rgb, alpha: f32, mouse: Vec2) -> Option<(String, String)> {
    let space = size * 0.27;
    let mut wx = x;
    let mut gloss = None;
    for (word, meaning) in row {
        let ww = match meaning {
            Some(m) => {
                let ww = c.styled(word, wx, y, size, ega(NATIVE, alpha), Face::Body, 0.0);
                let mut ux = wx;
                while ux < wx + ww {
                    c.rect(ux, y + size * 0.2, 2.0, 1.0, ega(NATIVE, 0.7 * alpha));
                    ux += 4.0;
                }
                if Bx::new(wx, y - size, ww, size * 1.3).contains(mouse) {
                    gloss = Some((word.trim_end_matches(|ch: char| !ch.is_alphanumeric() && ch != '\u{2bb}').to_string(), m.clone()));
                }
                ww
            }
            None => c.styled(word, wx, y, size, ega(col, alpha), Face::Body, 0.0),
        };
        wx += ww + space;
    }
    gloss
}

fn first_name(w: &World, p: PersonId) -> &str {
    let n = w.people[p as usize].name().unwrap_or("?");
    n.split(' ').next().unwrap_or(n)
}

/// How near a town's place someone must stand for the talk to be "at" it.
const AT_PLACE: f32 = 9.0;

/// The town and the spot within it, for the band along the top.
fn where_words(w: &World, at: V2) -> String {
    let Some(town) = w.town_at(at) else { return String::new() };
    let name = super::lexicon::town(w, town).to_uppercase();
    let place = w.society.towns.get(town as usize).and_then(|t| t.places.iter().filter(|p| p.pos.dist(at) < AT_PLACE).min_by(|a, b| a.pos.dist(at).total_cmp(&b.pos.dist(at))));
    match place {
        Some(p) => format!("{name}  ·  THE {}", super::lexicon::thing(w, p.kind.name()).to_uppercase()),
        None => name,
    }
}

fn draw_mark(c: &Canvas, m: Mark, cx: f32, cy: f32, col: bevy_egui::egui::Color32) {
    match m {
        Mark::None => {}
        Mark::Coins => {
            for dy in [4.0, 0.0, -4.0] {
                c.ellipse_lines(cx, cy + dy, 9.0, 3.5, 1.5, col);
            }
        }
        Mark::Anvil => {
            let p = |x: f32, y: f32| Vec2::new(cx + x, cy + y);
            c.poly_lines(&[p(-10.0, -6.0), p(9.0, -6.0), p(6.0, -1.5), p(2.5, -1.5), p(2.5, 3.0), p(6.0, 6.5), p(-6.0, 6.5), p(-2.5, 3.0), p(-2.5, -1.5), p(-6.0, -1.5)], 1.5, col, true);
        }
        Mark::Scroll => {
            c.rect_lines(cx - 6.0, cy - 8.0, 12.0, 16.0, 1.5, col);
            for dy in [-3.5, 0.0, 3.5] {
                c.line(Vec2::new(cx - 3.0, cy + dy), Vec2::new(cx + 3.0, cy + dy), 1.0, col);
            }
        }
        Mark::Lock => {
            c.rect_lines(cx - 6.0, cy - 1.0, 12.0, 9.0, 1.5, col);
            c.arc(cx, cy - 1.0, 3.5, std::f32::consts::PI, std::f32::consts::TAU, 1.5, col);
        }
    }
}

/// Draw the screen for the talk that is open (or the parting words just
/// after one): what was chosen, the box it covers (all of the window:
/// nothing behind it takes a click), and where the replies are really
/// scrolled to. `key` is a number key just pressed, 1 for the first reply
/// showing.
pub fn talk(c: &Canvas, w: &World, st: &Talking, parting: Option<&Parting>, mouse: Vec2, click: Option<Click>, key: Option<usize>) -> (Option<TalkAct>, Bx, f32) {
    let all = Bx::new(0.0, 0.0, c.w, c.h);
    // Who, what has been said, and what can be said back.
    let (with, npc, lines, replies): (PersonId, PersonId, Vec<(bool, String)>, Vec<Reply>) = match (w.talk.as_ref(), parting) {
        (Some(cv), _) => {
            let mut replies: Vec<Reply> = Vec::new();
            let mut leave = None;
            for t in w.topics() {
                let (text, note) = split_note(&w.topic_text(t));
                let kind = match kind_of(t) {
                    Kind::Ask if st.asked.contains(&t) => Kind::Asked,
                    k => k,
                };
                let r = Reply { topic: Some(t), text, note, kind, mark: mark_of(t) };
                if kind == Kind::Leave {
                    leave = Some(r);
                } else {
                    replies.push(r);
                }
            }
            for (t, why) in w.locked_topics() {
                replies.push(Reply { topic: None, text: t.label().to_string(), note: why, kind: Kind::Locked, mark: Mark::Lock });
            }
            // Leaving is always last.
            replies.extend(leave);
            (cv.with, cv.npc, cv.lines.clone(), replies)
        }
        (None, Some(p)) => (p.with, p.npc, vec![(true, p.line.clone())], Vec::new()),
        (None, None) => return (None, all, 0.0),
    };
    let them = &w.people[npc as usize];
    let me = &w.people[with as usize];
    let click = click.filter(|k| !k.right);

    // What they said last is the large line; everything before it is "so far".
    let last_theirs = lines.iter().rposition(|l| l.0);
    let (so_far, saying): (&[(bool, String)], &str) = match last_theirs {
        Some(k) => (&lines[..k], lines[k].1.as_str()),
        None => (&lines[..], ""),
    };

    // The foot of the screen, measured from the bottom up.
    let col_w = COLUMN.min(c.w - 120.0);
    let x0 = (c.w - col_w) / 2.0;
    let quoted = if saying.is_empty() { String::new() } else { format!("\u{201c}{saying}\u{201d}") };
    let mut size = 31.0;
    let mut rows = wrap(c, words(&quoted), size, col_w, 0.0);
    for smaller in [26.0, 22.0] {
        if rows.len() > 3 {
            size = smaller;
            rows = wrap(c, words(&quoted), size, col_w, 0.0);
        }
    }
    let lh = size * 1.32;
    let said_h = rows.len() as f32 * lh;
    let foot = c.h - 44.0;
    let fixed = 92.0 + 22.0 + said_h + 18.0 + 20.0;
    let room = ((foot - c.h * SCENE_SHARE - fixed) / ROW).floor().max(3.0) as usize;
    let showing = replies.len().min(room);
    let list_h = showing as f32 * ROW;
    let top = foot - fixed - list_h;

    // The world steps back: darker toward the edges and under the words.
    c.rect(0.0, 0.0, c.w, c.h, ega(DARK, 0.16));
    let clear = ega(DARK, 0.0);
    c.grad(0.0, 56.0, c.w * 0.2, c.h - 56.0, ega(DARK, 0.6), clear, true);
    c.grad(c.w * 0.8, 56.0, c.w * 0.2, c.h - 56.0, clear, ega(DARK, 0.6), true);
    c.rect(0.0, 0.0, c.w, 56.0, eg(DARK));
    c.grad(0.0, 56.0, c.w, 40.0, ega(DARK, 0.7), clear, false);
    let g0 = top - 70.0;
    c.grad(0.0, g0, c.w, 110.0, clear, ega(DARK, 0.82), false);
    c.grad(0.0, g0 + 110.0, c.w, 110.0, ega(DARK, 0.82), ega(DARK, 0.96), false);
    c.rect(0.0, g0 + 220.0, c.w, c.h - g0 - 220.0, ega(DARK, 0.96));

    // Where this is.
    let place = where_words(w, w.person_pos(npc));
    if !place.is_empty() {
        let pw = c.styled_width(&place, 15.0, Face::Title, 6.0);
        c.styled(&place, (c.w - pw) / 2.0, 34.0, 15.0, eg(BAND), Face::Title, 6.0);
    }

    // The talk so far: newest at the bottom, whole lines only.
    let mut gloss: Option<(String, String)> = None;
    let side_w = 320.0f32.min(c.w * 0.24);
    let side_x = c.w - side_w - 60.0;
    let side_top = 92.0;
    let side_bot = (top - 40.0).max(side_top + 120.0);
    if !so_far.is_empty() && c.w >= 1100.0 {
        let inner = side_w - 40.0;
        let lh2 = 21.5;
        let mut laid: Vec<(bool, Vec<Vec<Word>>, f32)> = Vec::new();
        let mut used = 34.0;
        for (theirs, line) in so_far.iter().rev() {
            let who = first_name(w, if *theirs { npc } else { with });
            let lead = c.styled_width(who, 16.0, Face::Caps, 0.0) + 10.0;
            let r = wrap(c, words(line), 16.0, inner, lead);
            let h = r.len() as f32 * lh2 + 12.0;
            if used + h > side_bot - side_top - 16.0 {
                break;
            }
            used += h;
            laid.push((*theirs, r, lead));
        }
        if !laid.is_empty() {
            let y_top = side_bot - used - 18.0;
            c.grad(side_x, y_top - 30.0, side_w, 60.0, clear, ega(DARK, 0.72), false);
            c.rect(side_x, y_top + 30.0, side_w, side_bot - y_top - 30.0, ega(DARK, 0.72));
            c.rule_in(side_x, side_x + side_w, side_bot, 1.0, ega(BRASS, 0.45), false);
            c.styled("SO FAR", side_x + 20.0, y_top + 24.0, 12.0, eg(HEAD), Face::Title, 4.0);
            let mut y = y_top + 34.0;
            let n = laid.len();
            for (i, (theirs, r, lead)) in laid.iter().rev().enumerate() {
                let who = first_name(w, if *theirs { npc } else { with });
                y += 12.0;
                // Older lines sit further back.
                let col = if i + 2 >= n { NEAR } else { FAR };
                c.styled(who, side_x + 20.0, y + 16.0, 16.0, eg(if *theirs { super::palette::mix(super::palette::race_color(them.race), NAME, 0.35) } else { BRASS }), Face::Caps, 0.0);
                for (k, row) in r.iter().enumerate() {
                    let rx = side_x + 20.0 + if k == 0 { *lead } else { 0.0 };
                    gloss = draw_row(c, row, rx, y + 16.0, 16.0, col, 1.0, mouse).or(gloss);
                    y += lh2;
                }
            }
        }
    }

    // Who is speaking.
    super::frame::portrait(c, x0 + 46.0, top + 46.0, 41.0, them.race, them.seed, false, 3.0);
    c.styled(first_name(w, npc), x0 + 114.0, top + 48.0, 32.0, eg(NAME), Face::Caps, 1.0);
    let job = w.life(npc).job;
    let what = if job == Job::None { them.stats.calling.name().to_lowercase() } else if super::lexicon::native() { super::lexicon::thing(w, job.name()).to_lowercase() } else { job.title(them.seed).to_lowercase() };
    c.styled(&format!("{} · {what}", them.race.name()), x0 + 114.0, top + 76.0, 18.0, eg(SUB), Face::Italic, 0.0);

    // What they just said.
    let mut y = top + 92.0 + 22.0 + size * 0.86;
    for row in &rows {
        // A soft shadow keeps it readable over a bright scene.
        draw_row(c, row, x0 + 1.0, y + 2.0, size, [0.0, 0.0, 0.0], 0.55, Vec2::new(-1.0, -1.0));
        gloss = draw_row(c, row, x0, y, size, LINE, 1.0, mouse).or(gloss);
        y += lh;
    }
    let rule_y = top + 92.0 + 22.0 + said_h + 18.0;
    c.rule_in(x0, x0 + col_w, rule_y, 1.0, ega(BRASS, 0.7), false);

    // The replies.
    let list_y = rule_y + 20.0;
    let max_scroll = (replies.len() as f32 * ROW - list_h).max(0.0);
    let scroll = st.scroll.clamp(0.0, max_scroll);
    let first = (scroll / ROW).round() as usize;
    let row_w = (col_w - 110.0).min(1000.0);
    let mut act = None;
    for (i, r) in replies.iter().enumerate().skip(first).take(showing) {
        let n = i - first + 1;
        let ry = list_y + (i - first) as f32 * ROW;
        let bx = Bx::new(x0 - 14.0, ry, row_w, ROW - 2.0);
        let open = r.kind != Kind::Locked;
        let hot = open && bx.contains(mouse);
        let col = match r.kind {
            _ if hot => HOT,
            Kind::Does => DOES,
            Kind::Ask => PLAIN,
            Kind::Asked => ASKED,
            Kind::Leave => LEAVE,
            Kind::Locked => LOCKED,
        };
        if hot {
            c.grad(bx.x, bx.y, bx.w * 0.6, bx.h, ega(BRASS, 0.26), ega(BRASS, 0.06), true);
            c.grad(bx.x + bx.w * 0.6, bx.y, bx.w * 0.4, bx.h, ega(BRASS, 0.06), ega(BRASS, 0.0), true);
            c.rule_in(bx.x, bx.x + bx.w * 0.7, bx.y + 0.5, 1.0, ega(NAME, 0.35), false);
            c.rule_in(bx.x, bx.x + bx.w * 0.7, bx.y + bx.h - 0.5, 1.0, ega(NAME, 0.2), false);
        }
        // The number to press (the first nine showing).
        let nb = Bx::new(x0, ry + 4.0, 24.0, 24.0);
        if n <= 9 {
            if hot {
                c.rect(nb.x, nb.y, nb.w, nb.h, eg(NAME));
            } else {
                c.rect_lines(nb.x, nb.y, nb.w, nb.h, 1.0, ega(col, if r.kind == Kind::Does { 0.55 } else { 0.42 }));
            }
            let d = n.to_string();
            let dw = c.styled_width(&d, 14.0, Face::Body, 0.0);
            c.styled(&d, nb.x + (24.0 - dw) / 2.0, nb.y + 17.0, 14.0, if hot { eg(DARK) } else { eg(col) }, Face::Body, 0.0);
        }
        // What kind of reply it is.
        let (mx, my) = (x0 + 49.0, ry + 16.0);
        match r.mark {
            Mark::None if hot => c.diamond(mx, my, 4.5, eg(NAME)),
            m => draw_mark(c, m, mx, my, eg(col)),
        }
        let tx = x0 + 74.0;
        let tw = c.styled(&r.text, tx, ry + 24.0, 22.0, eg(col), Face::Body, 0.0);
        if !r.note.is_empty() {
            let nc = if r.kind == Kind::Locked { [0.54, 0.49, 0.40] } else { super::palette::mix(col, SUB, 0.6) };
            c.styled(&r.note, tx + tw + 14.0, ry + 24.0, 18.0, eg(nc), Face::Italic, 0.0);
        }
        let picked = open && (click.is_some_and(|k| bx.contains(k.at)) || key == Some(n));
        if let (true, Some(t)) = (picked, r.topic) {
            act = Some(TalkAct::Ask(t));
        }
    }
    // More above or below than fits.
    if first > 0 {
        c.triangle(Vec2::new(x0 + row_w - 8.0, list_y + 10.0), Vec2::new(x0 + row_w + 8.0, list_y + 10.0), Vec2::new(x0 + row_w, list_y), ega(BRASS, 0.8));
    }
    if first + showing < replies.len() {
        let by = list_y + list_h - 4.0;
        c.triangle(Vec2::new(x0 + row_w - 8.0, by - 10.0), Vec2::new(x0 + row_w + 8.0, by - 10.0), Vec2::new(x0 + row_w, by), ega(BRASS, 0.8));
    }

    // Who is answering for the squad.
    if showing > 0 {
        let (px, py) = (x0 + col_w - 50.0, list_y + 36.0);
        super::frame::portrait(c, px, py, 29.0, me.race, me.seed, false, 2.0);
        let mine = first_name(w, with);
        let mw = c.styled_width(mine, 18.0, Face::Caps, 0.0);
        c.styled(mine, px - mw / 2.0, py + 56.0, 18.0, eg(BRASS), Face::Caps, 0.0);
    }

    // The meaning of one of their own words under the mouse, else the keys.
    let hint = match (&gloss, showing) {
        (Some((word, meaning)), _) => format!("{word}: \u{201c}{meaning}\u{201d} in {}", gahturiyu_sim::names::Tongue::from(them.race).name()),
        (None, 0) => String::new(),
        (None, 1) => "1 to answer  ·  Q E to look round  ·  Esc to leave".to_string(),
        (None, n) => format!("1\u{2013}{} to answer  ·  Q E to look round  ·  Esc to leave", n.min(9)),
    };
    if !hint.is_empty() {
        let hw = c.styled_width(&hint, 14.0, Face::Body, 1.0);
        c.styled(&hint, (c.w - hw) / 2.0, c.h - 16.0, 14.0, if gloss.is_some() { eg(NATIVE) } else { eg(HINT) }, Face::Body, 1.0);
    }
    if replies.is_empty() && click.is_some() {
        act = Some(TalkAct::Done);
    }
    (act, all, scroll)
}

/// Where the 3D camera was before a talk drew it in, and where it is now.
#[derive(Clone, Copy, Debug)]
pub struct TalkCam {
    /// Target, yaw, pitch and distance to go back to.
    pub was: (V2, f32, f32, f32),
    /// The point being looked at (eased by itself: following the squad
    /// would otherwise pull it back every frame).
    pub at: V2,
    /// Which side it settled on (a yaw), chosen once as the talk opens.
    pub side: f32,
    /// Seconds left of being drawn into place; after that the turn, tilt
    /// and distance are the player's again (Q and E look round).
    pub settling: f32,
    /// Seconds left of easing back out, once the talk is over.
    pub leaving: Option<f32>,
}

/// How close and how low the camera sits beside two people talking, and
/// how far round toward the squad member's shoulder (so the other's face
/// is the one in view).
const CAM_DIST: f32 = 6.5;
const CAM_PITCH: f32 = 0.28;
const CAM_ROUND: f32 = 0.38;
const CAM_SETTLE: f32 = 0.9;
const CAM_BACK: f32 = 0.45;

fn turn_toward(from: f32, to: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let d = (to - from).rem_euclid(tau);
    if d > tau / 2.0 { d - tau } else { d }
}

fn near_line(p: V2, a: V2, b: V2) -> f32 {
    let ab = b.sub(a);
    let t = if ab.len() < 1e-3 { 0.0 } else { (p.sub(a).x * ab.x + p.sub(a).y * ab.y) / (ab.len() * ab.len()) };
    p.dist(a.add(ab.scale(t.clamp(0.0, 1.0))))
}

/// How much stands between a camera at `eye` (on the ground plan) and two
/// people talking round `mid`: the rest of the squad standing in the way,
/// and walls (the camera outside the building they are in, or the other
/// way about). Lower is clearer. Drawing only.
pub fn sight_cost(w: &World, pair: (PersonId, PersonId), eye: V2, mid: V2) -> f32 {
    let mut cost = 0.0;
    for (k, &m) in w.squad.members.iter().enumerate() {
        // (Someone beside the line of sight looms as large as someone on it.)
        let d = near_line(w.member_pos(k), eye, mid);
        if m != pair.0 && d < 2.6 {
            cost += 2.6 - d;
        }
    }
    let inside = w.building_at(mid);
    if (1..=8).any(|i| w.building_at(mid.add(eye.sub(mid).scale(i as f32 / 8.0))) != inside) {
        cost += 3.0;
    }
    cost
}

/// Ease the camera in beside the two talking (`pair`: the squad member's
/// spot, the other's), and back out to where it was when they've done.
/// `cost` says how blocked the view from a spot would be (`sight_cost`);
/// `following` says the camera's target is being kept on the squad anyway;
/// `snap` is for screenshots (no easing). Drawing only.
pub fn aim(orbit: &mut OrbitCam, cam: &mut Option<TalkCam>, pair: Option<(V2, V2)>, cost: &dyn Fn(V2, V2) -> f32, following: bool, dt: f32, snap: bool) {
    let k = if snap { 1.0 } else { 1.0 - (-7.0 * dt).exp() };
    match pair {
        Some((a, b)) => {
            let mid = a.add(b).scale(0.5);
            let sep = a.dist(b);
            let dist = (CAM_DIST + sep * 0.8).clamp(CAM_DIST, 10.0);
            if cam.is_none() {
                // Side on to the two, a little round behind the squad
                // member, from whichever side is clearer; if both are
                // crowded, from further round, wherever the view is clear.
                let back = (a.y - b.y).atan2(a.x - b.x);
                let side = std::f32::consts::FRAC_PI_2 - CAM_ROUND;
                let score = |yaw: f32| {
                    let off = turn_toward(back + side, yaw).abs().min(turn_toward(back - side, yaw).abs());
                    cost(mid.add(V2::new(yaw.cos(), yaw.sin()).scale(dist * CAM_PITCH.cos())), mid) + off * 1.2 + turn_toward(orbit.yaw, yaw).abs() * 0.05
                };
                let mut want = back + side;
                for i in 0..16 {
                    for yaw in [back + side + i as f32 * 0.2, back - side - i as f32 * 0.2] {
                        if score(yaw) < score(want) - 1e-4 {
                            want = yaw;
                        }
                    }
                }
                *cam = Some(TalkCam { was: (orbit.target, orbit.yaw, orbit.pitch, orbit.dist), at: orbit.target, side: want, settling: CAM_SETTLE, leaving: None });
            }
            let Some(tc) = cam.as_mut() else { return };
            tc.leaving = None;
            tc.at = tc.at.add(mid.sub(tc.at).scale(k));
            orbit.target = tc.at;
            if tc.settling > 0.0 || snap {
                tc.settling -= dt;
                orbit.yaw += turn_toward(orbit.yaw, tc.side) * k;
                orbit.pitch += (CAM_PITCH - orbit.pitch) * k;
                orbit.dist += (dist - orbit.dist) * k;
            }
        }
        None => {
            let Some(tc) = cam.as_mut() else { return };
            let left = tc.leaving.unwrap_or(CAM_BACK) - dt;
            tc.leaving = Some(left);
            let (target, yaw, pitch, dist) = tc.was;
            if left <= 0.0 || snap {
                if !following {
                    orbit.target = target;
                }
                orbit.yaw = yaw;
                orbit.pitch = pitch;
                orbit.dist = dist;
                *cam = None;
                return;
            }
            if !following {
                tc.at = tc.at.add(target.sub(tc.at).scale(k));
                orbit.target = tc.at;
            }
            orbit.yaw += turn_toward(orbit.yaw, yaw) * k;
            orbit.pitch += (pitch - orbit.pitch) * k;
            orbit.dist += (dist - orbit.dist) * k;
        }
    }
}
