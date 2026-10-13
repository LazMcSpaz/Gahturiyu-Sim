//! One squad member's sheet (Laz's P12; drawing only): who they are, their
//! attributes and skills, what kind of person they are, how they are now
//! (body, hunger, tiredness, load, energy) and what's on them. It follows the
//! selection like the other member screens (`screens.rs`); the Stats button
//! opens it.

use gahturiyu_sim::sim::{
    body::PARTS,
    condition::{stage_of, HungerStage, EXHAUSTED},
    person::PersonId,
    stats::{Attr, ATTRS, SKILLS},
    World,
};

use super::hud::{Canvas, Face};
use super::kit;
use super::palette::{ega, eg, Rgb, BAR_HEALTH, BAR_LOW, BAR_STAMINA, BRASS, BRASS_DARK, BRASS_LIGHT, DIM, GOLD, MANA, TEXT};
use super::squadui::{Bx, Click};

const W: f32 = 760.0;
const H: f32 = 560.0;
const LH: f32 = 22.0;
/// Skills below this aren't listed (everyone has a little of everything).
const SKILL_SHOWN: f32 = 1.0;

/// What the sheet asks for.
pub enum StatsAct {
    Close,
}

fn trait_word(v: f32, low: &'static str, mid: &'static str, high: &'static str) -> &'static str {
    if v < 0.4 {
        low
    } else if v > 0.6 {
        high
    } else {
        mid
    }
}

fn cap_first(s: &str) -> String {
    let mut ch = s.chars();
    ch.next().map(|f| f.to_uppercase().chain(ch).collect()).unwrap_or_default()
}

fn attr_name(a: Attr) -> &'static str {
    match a {
        Attr::Strength => "Strength",
        Attr::Agility => "Agility",
        Attr::Toughness => "Toughness",
        Attr::Intellect => "Intellect",
        Attr::Willpower => "Willpower",
        Attr::Personality => "Personality",
    }
}

/// A labelled row with a bar: label, number, and how full (0..1).
fn bar_row(c: &Canvas, x: f32, y: f32, w: f32, label: &str, value: &str, k: f32, col: Rgb) {
    c.text(label, x, y, 16.0, TEXT);
    let vw = c.width(value, 16.0);
    c.text(value, x + w - vw, y, 16.0, DIM);
    let (bx, bw) = (x + 130.0, w - 130.0 - 56.0);
    c.rect(bx, y - 9.0, bw, 6.0, ega([0.0, 0.0, 0.0], 0.5));
    c.rect(bx, y - 9.0, bw * k.clamp(0.0, 1.0), 6.0, eg(col));
}

fn heading(c: &Canvas, x: f32, y: f32, w: f32, s: &str) {
    c.styled(&s.to_uppercase(), x, y, 13.0, eg(GOLD), Face::Title, 1.5);
    c.rule_in(x, x + w, y + 7.0, 1.0, ega(BRASS, 0.5), false);
}

pub fn sheet(c: &Canvas, w: &World, pid: PersonId, click: Option<Click>) -> (Option<StatsAct>, Bx) {
    let p = &w.people[pid as usize];
    let x0 = (c.w - W) / 2.0;
    let y0 = ((c.h - H) / 2.0 - 40.0).max(20.0);
    c.frame_box(x0, y0, W, H);
    let mut act = None;

    // Who.
    super::frame::portrait(c, x0 + 46.0, y0 + 50.0, 32.0, p.race, p.seed, false, 2.0);
    let st = p.effective_stats();
    c.styled(p.name().unwrap_or("(no name)"), x0 + 92.0, y0 + 50.0, 26.0, eg(BRASS_LIGHT), Face::Caps, 1.0);
    c.styled(&format!("{} · {}", p.race.name(), st.calling.name().to_lowercase()), x0 + 92.0, y0 + 74.0, 17.0, eg(DIM), Face::Italic, 0.0);
    let t = p.traits;
    let kind = format!(
        "{}, {}, {}, {}",
        trait_word(t.wanderlust, "a homebody", "settled", "restless"),
        trait_word(t.boldness, "cautious", "steady", "bold"),
        trait_word(t.sociability, "solitary", "easygoing", "gregarious"),
        trait_word(t.patience, "hot-tempered", "even-tempered", "patient"),
    );
    c.styled(&kind, x0 + 92.0, y0 + 96.0, 16.0, eg(TEXT), Face::Body, 0.0);
    // Close.
    let close = Bx::new(x0 + W - 40.0, y0 + 10.0, 30.0, 30.0);
    let cl = kit::look(&close);
    c.styled("×", close.x + 8.0, close.y + 22.0 + kit::nudge(cl), 24.0, eg(if cl.hot { BRASS_LIGHT } else { DIM }), Face::Body, 0.0);
    if click.is_some_and(|k| close.contains(k.at)) {
        act = Some(StatsAct::Close);
    }

    let col_w = (W - 3.0 * 28.0) / 2.0;
    let (lx, rx) = (x0 + 28.0, x0 + 28.0 * 2.0 + col_w);
    let top = y0 + 130.0;

    // Left: attributes, then skills.
    let mut y = top;
    heading(c, lx, y, col_w, "Attributes");
    y += 26.0;
    for a in ATTRS {
        let v = st.attr(a);
        bar_row(c, lx, y, col_w, attr_name(a), &format!("{v:.0}"), v / 100.0, BRASS);
        y += LH;
    }
    y += 14.0;
    heading(c, lx, y, col_w, "Skills");
    y += 26.0;
    let mut skills: Vec<_> = SKILLS.iter().map(|&k| (k, st.skill(k))).filter(|s| s.1 >= SKILL_SHOWN).collect();
    skills.sort_by(|a, b| b.1.total_cmp(&a.1));
    let room = ((y0 + H - 20.0 - y) / LH).floor().max(0.0) as usize;
    for (k, v) in skills.iter().take(room) {
        bar_row(c, lx, y, col_w, k.name(), &format!("{v:.0}"), v / 100.0, BRASS_DARK);
        y += LH;
    }
    if skills.is_empty() {
        c.text("Nothing learned yet", lx, y, 16.0, DIM);
    }

    // Right: how they are, then what's on them.
    let mut y = top;
    heading(c, rx, y, col_w, "Body");
    y += 26.0;
    let (hp, missing) = match w.fighter(pid) {
        Some(f) => (f.hp, f.missing),
        None => (p.wounds.hp_at(&p.stats, w.time), p.wounds.missing),
    };
    for (i, part) in PARTS.iter().enumerate() {
        let max = p.stats.max_hp(*part);
        if missing[i] {
            c.text(&cap_first(part.name()), rx, y, 16.0, TEXT);
            c.text("lost", rx + col_w - c.width("lost", 16.0), y, 16.0, [0.9, 0.35, 0.3]);
        } else {
            let k = hp[i] / max.max(1.0);
            bar_row(c, rx, y, col_w, &cap_first(part.name()), &format!("{:.0}/{:.0}", hp[i], max), k, if k < 0.35 { BAR_LOW } else { BAR_HEALTH });
        }
        y += LH;
    }
    y += 14.0;
    heading(c, rx, y, col_w, "Condition");
    y += 26.0;
    if let Some(h) = w.hunger_of(pid) {
        let word = match stage_of(h) {
            HungerStage::Fed => "Fed",
            HungerStage::Hungry => "Hungry",
            HungerStage::Weak => "Weak from hunger",
            HungerStage::Starving => "Starving",
        };
        bar_row(c, rx, y, col_w, "Hunger", word, h / 100.0, if h >= 40.0 { BAR_LOW } else { BAR_STAMINA });
        y += LH;
    }
    if let Some((t, water)) = w.thirst_of(pid) {
        use gahturiyu_sim::sim::condition::{PARCHED, THIRSTY};
        let word = if t >= PARCHED { "Parched" } else if t >= THIRSTY { "Thirsty" } else { "Slaked" };
        let carried = if water >= 1.0 { format!("{word}, {:.0} drinks carried", water.floor()) } else { format!("{word}, no water") };
        bar_row(c, rx, y, col_w, "Thirst", &carried, t / 100.0, if t >= THIRSTY { BAR_LOW } else { BAR_STAMINA });
        y += LH;
    }
    // Cold (the weather against what they wear), once there's any.
    if let Some(cold) = w.cold_of(pid).filter(|&x| x >= 0.5) {
        use gahturiyu_sim::sim::condition::{CHILLED, COLD, FREEZING};
        let word = if cold >= FREEZING { "Freezing" } else if cold >= COLD { "Cold through" } else if cold >= CHILLED { "Chilled" } else { "Cool" };
        bar_row(c, rx, y, col_w, "Cold", word, cold / 100.0, if cold >= CHILLED { BAR_LOW } else { BAR_STAMINA });
        y += LH;
    }
    // Toxicity (what they've drunk), once there's any.
    if let Some(tox) = w.toxicity_of(pid).filter(|&x| x >= 0.5) {
        use gahturiyu_sim::sim::condition::{QUEASY, SICK, TOX_COLLAPSE};
        let word = if tox >= TOX_COLLAPSE { "Collapsed" } else if tox >= SICK { "Sick" } else if tox >= QUEASY { "Queasy" } else { "Fine" };
        bar_row(c, rx, y, col_w, "Toxicity", word, tox / 100.0, if tox >= QUEASY { BAR_LOW } else { BAR_STAMINA });
        y += LH;
    }
    if let Some(tired) = w.tired_of(pid) {
        let word = if tired >= EXHAUSTED { "Exhausted" } else if tired >= EXHAUSTED * 0.5 { "Tired" } else { "Rested" };
        bar_row(c, rx, y, col_w, "Tiredness", word, tired / 100.0, if tired >= EXHAUSTED { BAR_LOW } else { BAR_STAMINA });
        y += LH;
    }
    let mana = w.fighter(pid).map(|f| f.mana).unwrap_or_else(|| p.mana_at(w.time));
    bar_row(c, rx, y, col_w, "Energy", &format!("{:.0}/{:.0}", mana, p.max_mana()), mana / p.max_mana().max(1.0), MANA);
    y += LH;
    // (`load_of` is the share of what they can carry.)
    let (share, cap) = (w.load_of(pid), w.capacity_at(pid, w.time));
    bar_row(c, rx, y, col_w, "Load", &format!("{:.0}/{cap:.0} kg", share * cap), share, if share > 1.0 { BAR_LOW } else { BRASS });
    y += LH + 14.0;
    heading(c, rx, y, col_w, "On them");
    y += 26.0;
    let boons = w.boons_on(pid);
    if boons.is_empty() {
        c.text("Nothing", rx, y, 16.0, DIM);
    }
    for b in boons.iter().take(((y0 + H - 20.0 - y) / LH).floor().max(0.0) as usize) {
        let mins = ((b.until - w.time) / 60.0).max(0.0);
        let left = if mins >= 90.0 { format!("{:.0} h", mins / 60.0) } else { format!("{mins:.0} min") };
        c.text(&format!("{:?}", b.does), rx, y, 16.0, [0.65, 0.75, 1.0]);
        c.text(&left, rx + col_w - c.width(&left, 16.0), y, 16.0, DIM);
        y += LH;
    }
    (act, Bx::new(x0, y0, W, H))
}
