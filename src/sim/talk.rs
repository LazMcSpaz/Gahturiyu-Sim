//! Assembled conversation (Part 5, Stage 5).
//!
//! What someone says is put together from small pieces of text in
//! `data/lines/` (format in `data/lines/FORMAT.md`), only when the squad
//! actually talks to them:
//!
//! 1. **What's on their mind**: their concerns ranked from their own
//!    memories, needs, the events they know of, the work they'd give an
//!    outsider, and how they feel about whoever's talking. The top three are
//!    kept. People only talk about what they know.
//! 2. **Facts** for those topics only: their temper, honour, mood and voice,
//!    the hour, the squad member's people and standing, who's involved,
//!    where, whom they blame.
//! 3. **Pieces** are picked by their conditions: the most specific that fits,
//!    ties broken by a roll fixed for that person, skipping what they said to
//!    this squad member lately.
//! 4. Put together: greeting + topic + feeling + hook; farewell at the end.
//!    A refusal ends the talk.
//!
//! The squad's **options** come from the concerns, gated by who's talking:
//! ask more, offer help, report it, persuade, pay, threaten. **Barks** are
//! one-liners for people with something strong on their mind as the squad
//! goes by; they change nothing, so the window asks for them.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::chances::{Chance, OppState};
use super::history::Deed;
use super::items;
use super::lives::{Need, Work};
use super::memory::Who;
use super::person::PersonId;
use super::race::Race;
use super::rng::Rng;
use super::world::{World, DAY, HOUR};

// ---- Dials -----------------------------------------------------------------

/// Concerns kept in mind for a talk.
pub const CONCERNS: usize = 3;
/// A piece said to the same squad member within this long isn't said again
/// (if anything else fits), seconds.
pub const RECENT: f64 = 2.0 * DAY;
/// Pieces remembered as said, world-wide.
pub const SAID_CAP: usize = 300;
/// Disposition below which they won't talk; above which they're warm.
pub const DISTRUST: f32 = 20.0;
pub const WARM: f32 = 65.0;
/// How much what they remember of a squad member moves their regard (× memory).
pub const MEMORY_WEIGHT: f32 = 50.0;
/// A concern this strong gets said aloud as the squad passes.
pub const BARK_AT: f32 = 0.6;

// ---- The pieces ------------------------------------------------------------

/// What someone can have on their mind.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subject {
    Theft,
    Grudge,
    Money,
    Hunger,
    Safety,
    Work,
    Ring,
    Kindness,
    Job,
    News,
    Ambition,
}

impl Subject {
    pub fn key(self) -> &'static str {
        match self {
            Subject::Theft => "theft",
            Subject::Grudge => "grudge",
            Subject::Money => "money",
            Subject::Hunger => "hunger",
            Subject::Safety => "safety",
            Subject::Work => "work",
            Subject::Ring => "ring",
            Subject::Kindness => "kindness",
            Subject::Job => "job",
            Subject::News => "news",
            Subject::Ambition => "ambition",
        }
    }
}

/// The data files, compiled in.
const FILES: [(&str, &str); 18] = [
    ("", include_str!("../../data/lines/greetings.txt")),
    ("", include_str!("../../data/lines/farewells.txt")),
    ("theft", include_str!("../../data/lines/theft.txt")),
    ("grudge", include_str!("../../data/lines/grudge.txt")),
    ("money", include_str!("../../data/lines/money.txt")),
    ("hunger", include_str!("../../data/lines/hunger.txt")),
    ("safety", include_str!("../../data/lines/safety.txt")),
    ("work", include_str!("../../data/lines/work.txt")),
    ("ring", include_str!("../../data/lines/ring.txt")),
    ("kindness", include_str!("../../data/lines/kindness.txt")),
    ("job", include_str!("../../data/lines/job.txt")),
    ("news", include_str!("../../data/lines/news.txt")),
    ("ambition", include_str!("../../data/lines/ambition.txt")),
    ("background", include_str!("../../data/lines/background.txt")),
    ("town", include_str!("../../data/lines/town.txt")),
    ("advice", include_str!("../../data/lines/advice.txt")),
    ("rumours", include_str!("../../data/lines/rumours.txt")),
    ("bandits", include_str!("../../data/lines/bandits.txt")),
];

#[derive(Clone, Debug)]
pub struct Piece {
    pub id: u16,
    /// The topic file it's from ("" for greetings and farewells).
    pub topic: &'static str,
    pub slot: &'static str,
    /// Tags that must hold (and, with `!`, must not).
    pub must: Vec<&'static str>,
    pub not: Vec<&'static str>,
    pub text: &'static str,
    pub refuse: bool,
}

/// Every piece, parsed once.
pub fn pieces() -> &'static [Piece] {
    static P: OnceLock<Vec<Piece>> = OnceLock::new();
    P.get_or_init(|| {
        let mut out = Vec::new();
        for (topic, src) in FILES {
            for line in src.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let f: Vec<&'static str> = line.split('|').map(|x| x.trim()).collect();
                if f.len() < 2 {
                    continue;
                }
                let conds: Vec<&'static str> = f[1].split(',').map(|x| x.trim()).filter(|x| !x.is_empty()).collect();
                out.push(Piece {
                    id: out.len() as u16,
                    topic,
                    slot: f[0],
                    must: conds.iter().copied().filter(|c| !c.starts_with('!')).collect(),
                    not: conds.iter().filter(|c| c.starts_with('!')).map(|c| &c[1..]).collect(),
                    text: f.get(2).copied().unwrap_or(""),
                    refuse: f.get(3).is_some_and(|x| x.contains("refuse")),
                });
            }
        }
        out
    })
}

// ---- What's on their mind --------------------------------------------------

/// Something on someone's mind.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Concern {
    pub subject: Subject,
    pub score: f32,
    /// The event it's about, the work they'd give, and the other person.
    pub event: Option<u32>,
    pub opp: Option<u32>,
    pub about: Option<PersonId>,
}

/// What the squad can say back.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Opt {
    /// Ask about the next thing on their mind.
    More,
    /// Offer to help: they say what they'd want done.
    Help,
    /// Tell the watch who did it.
    Report,
    /// Talk them out of a grudge.
    Persuade,
    /// Pay for what they know.
    Bribe,
    /// Lean on them.
    Threaten,
}

impl Opt {
    pub fn label(self) -> &'static str {
        match self {
            Opt::More => "Go on.",
            Opt::Help => "Maybe I can help.",
            Opt::Report => "I'll tell the watch.",
            Opt::Persuade => "Let it go. It isn't worth it.",
            Opt::Bribe => "Here's something for your trouble (10 coin).",
            Opt::Threaten => "You'll tell me what you know.",
        }
    }
}

/// What was said in a talk, for the conversation.
#[derive(Clone, Debug, Default)]
pub struct Said {
    pub text: String,
    pub pieces: Vec<u16>,
    pub refused: bool,
}

/// Tags and slot values for one talk.
#[derive(Clone, Debug, Default)]
struct Facts {
    tags: BTreeSet<String>,
    slots: BTreeMap<&'static str, String>,
    /// The speaker's tongue, for `{w:root}` words.
    tongue: Option<crate::names::Tongue>,
}

impl Facts {
    fn tag(&mut self, t: impl Into<String>) {
        self.tags.insert(t.into());
    }
    fn set(&mut self, k: &'static str, v: impl Into<String>) {
        self.slots.insert(k, v.into());
    }
    fn fits(&self, p: &Piece) -> bool {
        p.must.iter().all(|c| self.tags.contains(*c)) && !p.not.iter().any(|c| self.tags.contains(*c))
    }
    fn fill(&self, text: &str) -> String {
        let mut out = String::new();
        let mut rest = text;
        while let Some(a) = rest.find('{') {
            out.push_str(&rest[..a]);
            let Some(b) = rest[a..].find('}') else { break };
            let key = &rest[a + 1..a + b];
            // A word in the speaker's own tongue.
            if let (Some(root), Some(t)) = (key.strip_prefix("w:").or_else(|| key.strip_prefix("W:")), self.tongue) {
                out.push_str(&super::speech::native_word(root, t, key.starts_with('W')));
                rest = &rest[a + b + 1..];
                continue;
            }
            out.push_str(self.slots.get(key).map(|s| s.as_str()).unwrap_or(if matches!(key, "item" | "when" | "workplace" | "place" | "why" | "offer" | "span" | "count" | "pay" | "days" | "reward" | "debt" | "job" | "deed" | "there" | "far" | "dir") { "" } else { "someone" }));
            rest = &rest[a + b + 1..];
        }
        out.push_str(rest);
        // Tidy doubled spaces left by empty slots.
        out.split_whitespace().collect::<Vec<_>>().join(" ").replace(" ,", ",").replace(" .", ".")
    }
}

/// Roughly how long a walk a distance is, as someone would say it.
fn walk_words(metres: f32) -> &'static str {
    // An unhurried traveller's pace, and about ten hours on the road a day.
    let hours = metres / (super::squad::SQUAD_SPEED * 3600.0);
    match hours {
        h if h < 1.5 => "an hour's walk",
        h if h < 4.0 => "a few hours' walk",
        h if h < 7.0 => "half a day's walk",
        h if h < 14.0 => "a day's walk",
        h if h < 25.0 => "two days' walk",
        h if h < 35.0 => "three days' walk",
        _ => "many days' walk",
    }
}

fn when(now: f64, t: f64) -> String {
    let days = ((now - t) / DAY).floor();
    let h = (t.rem_euclid(DAY) / HOUR) as i32;
    match days as i64 {
        0 if !(6..20).contains(&h) => "last night".into(),
        0 => "today".into(),
        1 => "yesterday".into(),
        2..=6 => format!("{} days ago", ["", "", "two", "three", "four", "five", "six"][days as usize]),
        _ => "a while back".into(),
    }
}

impl World {
    /// How someone feels about a squad member, 0..100, counting what they
    /// remember of them.
    pub fn regard_of(&self, npc: PersonId, with: PersonId) -> f32 {
        let day = World::day_of(self.time) as i32;
        (self.disposition(npc, with) + self.memory_of(npc, Who::Person(with), day) * MEMORY_WEIGHT).clamp(0.0, 100.0)
    }

    /// What's on someone's mind, most pressing first (at most `CONCERNS`).
    pub fn on_mind(&self, npc: PersonId) -> Vec<Concern> {
        let mut out: Vec<Concern> = Vec::new();
        if npc as usize >= self.society.minds.len() {
            return out;
        }
        let day = World::day_of(self.time) as i32;
        let m = self.mind(npc);
        let pat = self.people[npc as usize].traits.patience;
        let mut c = |subject, score, event, opp, about| out.push(Concern { subject, score, event, opp, about });
        // Work they'd give an outsider.
        if let Some(o) = self.open_offer(npc) {
            let subject = match o.kind {
                Chance::Guard | Chance::Recover | Chance::FindOut if o.event.and_then(|e| self.event(e)).is_some_and(|e| matches!(e.deed, Deed::Theft | Deed::Con)) => Subject::Theft,
                Chance::CollectDebt => Subject::Money,
                Chance::Intimidate | Chance::Kill => Subject::Grudge,
                Chance::ClearCamp { .. } => Subject::Safety,
                _ => Subject::Job,
            };
            c(subject, 0.9, o.event, Some(o.id), o.target);
        }
        // What's been done to them, or for them.
        for mem in m.memories.iter() {
            let s = mem.strength(day, pat);
            let about = match mem.about {
                Who::Person(p) => Some(p),
                _ => None,
            };
            let subject = match mem.deed {
                Deed::Theft | Deed::Con if s < -0.1 && !mem.heard => Some(Subject::Theft),
                Deed::Extortion if s < -0.1 => Some(Subject::Ring),
                Deed::Kindness | Deed::Loan | Deed::JobDone if s > 0.25 => Some(Subject::Kindness),
                _ => None,
            };
            if let Some(sub) = subject {
                let ev = self.mind(npc).knows.iter().rev().filter_map(|k| self.known_event(*k)).find(|e| e.victim == Some(npc) && e.deed == mem.deed).map(|e| e.id);
                c(sub, s.abs() + 0.3, ev, None, about);
            }
        }
        // A grudge between households.
        if let Some(h) = self.society.lives[npc as usize].household {
            if let Some(f) = self.society.households[h as usize].feelings.iter().filter(|f| f.stage >= 1).min_by(|a, b| a.warmth.total_cmp(&b.warmth)) {
                let about = self.society.households.get(f.other as usize).and_then(|x| x.members.first().copied());
                c(Subject::Grudge, -f.warmth + 0.1 * f.stage as f32, None, None, about);
            }
        }
        // Needs.
        let n = m.needs();
        if n[Need::Money as usize] > 0.35 {
            c(Subject::Money, n[Need::Money as usize], None, None, None);
        }
        if n[Need::Hunger as usize] > 0.35 {
            c(Subject::Hunger, n[Need::Hunger as usize], None, None, None);
        }
        if n[Need::Safety as usize] > 0.35 {
            c(Subject::Safety, n[Need::Safety as usize] * 0.8, None, None, None);
        }
        if matches!(m.work, Work::Jobless) && m.lost.is_some() {
            c(Subject::Work, 0.6, None, None, None);
        }
        if n[Need::Ambition as usize] > 0.6 {
            c(Subject::Ambition, 0.3, None, None, None);
        }
        // The best thing they've heard.
        if let Some(k) = m.knows.iter().copied().filter(|k| self.known_event(*k).is_some_and(|e| e.victim != Some(npc) && e.actor != Some(npc) && (e.deed.is_wrong() || e.deed.of_town()))).max_by(|a, b| self.interest(*a, day).total_cmp(&self.interest(*b, day))) {
            let e = self.known_event(k).unwrap();
            let sub = if e.deed == Deed::Theft { Subject::Theft } else { Subject::News };
            c(sub, self.interest(k, day) + 0.2, Some(e.id), None, e.actor);
        }
        out.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.subject.cmp(&b.subject)));
        // One of each subject.
        let mut seen = BTreeSet::new();
        out.retain(|x| seen.insert(x.subject));
        out.truncate(CONCERNS);
        out
    }

    /// The facts for one concern of `npc`'s, talking to `with`.
    fn facts(&self, npc: PersonId, with: PersonId, concern: Option<&Concern>) -> Facts {
        let mut f = Facts::default();
        let p = &self.people[npc as usize];
        let l = self.society.lives[npc as usize];
        let tr = p.traits;
        let voice = |r: Race| match r {
            Race::Roduro => "roduro",
            Race::Qotiro => "qotiro",
            Race::Horaro => "horaro",
            Race::Tadoro => "tadoro",
        };
        f.tag(format!("voice={}", voice(p.race)));
        f.tongue = Some(p.race.into());
        f.tag(format!("you={}", voice(self.people[with as usize].race)));
        f.tag(format!("job={}", l.job.name().to_lowercase()));
        f.set("name", self.name_of(npc));
        f.set("job", l.job.title(p.seed).to_lowercase());
        {
            let j = l.job.title(p.seed).to_lowercase();
            let an = if j.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
            f.set("a_job", format!("{an} {j}"));
        }
        if tr.patience < 0.4 && tr.boldness > 0.5 {
            f.tag("hot");
        }
        if tr.patience > 0.6 {
            f.tag(if tr.sociability > 0.5 { "gentle" } else { "cold" });
            f.tag("patient");
        }
        f.tag(if l.habits.honour < 0.35 { "low_honour" } else if l.habits.honour > 0.7 { "high_honour" } else { "mid_honour" });
        let m = self.mind(npc);
        let n = m.needs();
        f.tag(if n[1] > 0.4 { "hungry" } else if n[0] > 0.4 { "worried" } else if n[2] > 0.4 { "afraid" } else { "content" });
        let h = (self.time.rem_euclid(DAY) / HOUR) as i32;
        f.tag(match h {
            5..=10 => "morning",
            11..=16 => "day",
            17..=20 => "evening",
            _ => "night",
        });
        // The squad member.
        let town = p.home;
        let regard = self.regard_of(npc, with);
        if regard < DISTRUST {
            f.tag("distrusts");
        } else if regard > WARM {
            f.tag("warm");
        }
        match town {
            Some(t) if self.standing(with, t) >= super::law::HEARD => f.tag("known"),
            _ => f.tag("stranger"),
        }
        if let Some(t) = town {
            f.set("town", self.settlements[t as usize].name.clone());
        }
        let hh = l.household;
        let coin = hh.map(|h| self.society.households[h as usize].purse.coin - self.daily_cost(h) * 3.0).unwrap_or(0.0);
        f.tag(if coin > 20.0 { "has_coin" } else { "poor" });
        let Some(c) = concern else { return f };
        f.tag(format!("topic={}", c.subject.key()));
        let ev = c.event.and_then(|e| self.event(e)).cloned();
        if let Some(e) = &ev {
            f.set("when", when(self.time, e.t));
            f.tag(if e.t.rem_euclid(DAY) / HOUR >= 20.0 || e.t.rem_euclid(DAY) / HOUR < 6.0 { "night" } else { "day" });
            f.set("victim", e.victim.map(|v| self.name_of(v)).unwrap_or_default());
            f.set("deed", e.deed.words());
            // Where it happened: so news from somewhere else can be followed
            // there. Named, with roughly how far and which way from here.
            let there = &self.settlements[e.town as usize];
            f.set("there", there.name.clone());
            if let Some(here) = town.filter(|&h| h != e.town) {
                let from = self.settlements[here as usize].pos;
                f.tag("elsewhere");
                f.set("dir", super::quests::compass(there.pos.sub(from)));
                f.set("far", walk_words(there.pos.dist(from)));
            }
            match e.deed {
                Deed::RiteFailed => f.tag("rite"),
                Deed::Rising => f.tag("rising"),
                _ => {}
            }
            if e.deed.of_town() {
                // (The lines for one person's deed to another don't fit.)
                f.tag("of_town");
            }
            if e.deed.of_town() {
                // Nobody's doing, and no secret: neither a culprit nor a mystery.
            } else if let (Some(a), false) = (e.actor, e.hidden) {
                f.set("actor", self.name_of(a));
                f.set("thief", self.name_of(a));
                f.tag("thief_known");
            } else {
                f.tag("thief_unknown");
                f.tag("hidden");
            }
            if e.victim == Some(npc) {
                f.tag("victim");
                let n = self.events(e.town).iter().filter(|x| x.victim == Some(npc) && matches!(x.deed, Deed::Theft | Deed::Con) && x.t > self.time - 7.0 * DAY).count();
                if n >= 2 {
                    f.tag("repeated");
                    f.set("count", ["", "", "Two", "Three", "Four", "Five"][n.min(5)]);
                    f.set("span", "week");
                }
            } else {
                f.tag("heard");
            }
            if let Some(s) = self.society.stolen.iter().find(|s| s.event == e.id) {
                f.set("item", items::item(s.item).name.to_lowercase());
            } else if e.deed == Deed::Theft {
                f.set("item", "coin");
            }
            // Whom they blame.
            if let Some(t) = town {
                if self.society.towns[t as usize].owed > 0.0 {
                    f.tag("blames_watch");
                }
                if e.hidden && self.ring(t).is_some() {
                    f.tag("blames_ring");
                }
                // Trouble that started about when the squad came to town.
                if e.hidden && regard < 45.0 && self.squad.pos.dist(self.settlements[t as usize].pos) < 600.0 {
                    f.tag("suspects_squad");
                }
            }
        }
        if let Some(pl) = l.place {
            if let Some(t) = town {
                f.set("workplace", self.society.towns[t as usize].places[pl as usize].kind.name().to_lowercase());
            }
        } else {
            f.set("workplace", "house");
        }
        if let Some(a) = c.about {
            // Said to their face, it's "you" (NM-56).
            f.set("target", if a == with { "You".to_string() } else { self.name_of(a) });
            let rel = match (self.society.lives.get(a as usize).and_then(|x| x.household), hh) {
                (Some(x), Some(y)) if x == y => "my household",
                _ => "my neighbour",
            };
            f.set("relation", rel);
        }
        match c.subject {
            Subject::Grudge => {
                if let Some(h) = hh {
                    if self.society.households[h as usize].feelings.iter().any(|x| x.stage >= 5) {
                        f.tag("feud");
                    }
                    if l.habits.honour < 0.35 {
                        f.tag("revenge");
                    }
                }
                f.set("relation", "We");
            }
            Subject::Money => {
                if let Some(h) = hh {
                    let d = self.society.households[h as usize].purse.debt();
                    if d > 0.0 {
                        f.tag("in_debt");
                        f.set("debt", format!("{d:.0}"));
                    }
                }
            }
            Subject::Safety => {
                if town.is_some_and(|t| self.camps.iter().any(|cp| cp.pos.dist(self.settlements[t as usize].pos) < 4000.0)) {
                    f.tag("camp");
                }
            }
            Subject::Work => {
                if let Some((j, why)) = m.lost {
                    f.tag("lost_post");
                    f.set("job", j.name().to_lowercase());
                    f.set("why", why.name().trim_start_matches("when ").trim_start_matches("while ").to_string());
                }
            }
            Subject::Ring => f.tag("extorted"),
            _ => {}
        }
        // The work they'd give, as a hook.
        if let Some(o) = c.opp.and_then(|id| self.opportunity(id)) {
            f.tag("has_offer");
            f.tag(format!("offer={}", o.kind.name()));
            f.set("offer", {
                let s = self.opp_line(o);
                let mut ch = s.chars();
                ch.next().map(|x| x.to_uppercase().collect::<String>() + ch.as_str()).unwrap_or_default()
            });
            if !o.legal {
                f.tag("unlawful");
            }
            let reward = if o.favour { "a favour owed".to_string() } else { format!("{} coin", o.reward) };
            f.set("reward", reward);
            f.set("pay", format!("{:.0} coin", o.amount));
            f.set("days", super::chances::GUARD_DAYS.to_string());
            match o.kind {
                Chance::Guard => f.tag("wants_guard"),
                Chance::Recover => f.tag("wants_item"),
                Chance::FindOut => f.tag("wants_find"),
                Chance::CollectDebt => {
                    f.tag("wants_debt");
                    f.set("debt", format!("{:.0} coin", o.amount));
                }
                Chance::ClearCamp { .. } => f.tag("wants_camp"),
                Chance::Intimidate | Chance::Kill => f.tag("revenge"),
                _ => {}
            }
            if o.favour {
                f.tag("wants_help");
            }
        }
        f
    }

    /// Pick the piece for a slot: the most specific that fits, not said to
    /// this squad member lately if anything else fits; a fixed roll breaks ties.
    fn pick(&self, f: &Facts, topic: &str, slot: &str, npc: PersonId, with: PersonId, avoid: &[u16]) -> Option<&'static Piece> {
        let fits: Vec<&'static Piece> = pieces().iter().filter(|p| p.slot == slot && (p.topic.is_empty() || p.topic == topic) && f.fits(p)).collect();
        if fits.is_empty() {
            return None;
        }
        let recent = |p: &Piece| avoid.contains(&p.id) || self.talk_said.iter().any(|s| s.0 == npc && s.1 == with && s.2 == p.id && self.time - s.3 < RECENT);
        let fresh: Vec<&'static Piece> = fits.iter().copied().filter(|p| !recent(p)).collect();
        let pool = if fresh.is_empty() { fits } else { fresh };
        // A refusal always wins when it fits.
        let best = pool.iter().map(|p| p.must.len() + p.not.len() + if p.refuse { 100 } else { 0 }).max().unwrap();
        let top: Vec<&'static Piece> = pool.into_iter().filter(|p| p.must.len() + p.not.len() + if p.refuse { 100 } else { 0 } == best).collect();
        let mut r = Rng::from_keys(&[self.seed, npc as u64, with as u64, slot.len() as u64, topic.len() as u64, 0x5049_434B]);
        Some(top[r.below(top.len())])
    }

    /// Put together what someone says about a concern (greeting first if
    /// `opening`): greeting + topic + feeling + hook.
    pub fn assemble_talk(&self, npc: PersonId, with: PersonId, concern: Option<&Concern>, opening: bool) -> Said {
        let f = self.facts(npc, with, concern);
        let topic = concern.map(|c| c.subject.key()).unwrap_or("");
        let mut said = Said::default();
        let mut parts: Vec<String> = Vec::new();
        let slots: &[&str] = if opening { &["greeting", "topic", "feeling", "hook"] } else { &["topic", "feeling", "hook"] };
        for &slot in slots {
            if slot != "greeting" && concern.is_none() {
                continue;
            }
            let Some(p) = self.pick(&f, topic, slot, npc, with, &said.pieces) else { continue };
            said.pieces.push(p.id);
            let text = f.fill(p.text);
            if !text.is_empty() {
                parts.push(text);
            }
            if p.refuse {
                said.refused = true;
                break;
            }
        }
        said.text = parts.join(" ");
        said
    }

    /// Something said from one of the asked-about files (`background`,
    /// `town`, `advice`, `rumours`, `bandits`): a piece for each slot in
    /// turn, chosen as any other piece is, with the facts the asking code
    /// gathered added as tags and slot values.
    pub fn say_from(&self, npc: PersonId, with: PersonId, topic: &str, slots: &[&str], tags: &[String], values: &[(&'static str, String)]) -> Said {
        let mut f = self.facts(npc, with, None);
        for t in tags {
            f.tag(t.clone());
        }
        for (k, v) in values {
            f.set(k, v.clone());
        }
        let mut said = Said::default();
        let mut parts = Vec::new();
        for &slot in slots {
            let Some(p) = self.pick(&f, topic, slot, npc, with, &said.pieces) else { continue };
            said.pieces.push(p.id);
            let text = f.fill(p.text);
            if !text.is_empty() {
                parts.push(text);
            }
        }
        said.text = parts.join(" ");
        said
    }

    /// Their farewell.
    pub fn farewell(&self, npc: PersonId, with: PersonId) -> Said {
        let f = self.facts(npc, with, None);
        let mut said = Said::default();
        if let Some(p) = self.pick(&f, "", "farewell", npc, with, &[]) {
            said.pieces.push(p.id);
            said.text = f.fill(p.text);
        }
        said
    }

    /// A one-line remark as the squad passes, if they've something strong on
    /// their mind. Changes nothing; the window shows it.
    pub fn bark(&self, npc: PersonId, with: PersonId) -> Option<String> {
        let c = self.on_mind(npc).into_iter().next().filter(|c| c.score >= BARK_AT)?;
        let f = self.facts(npc, with, Some(&c));
        let p = self.pick(&f, c.subject.key(), "bark", npc, with, &[])?;
        let s = f.fill(p.text);
        (!s.is_empty()).then_some(s)
    }

    /// Remember which pieces were said to whom (so they aren't said again soon).
    pub(super) fn note_said(&mut self, npc: PersonId, with: PersonId, pieces: &[u16]) {
        let t = self.time;
        self.talk_said.retain(|s| t - s.3 < RECENT);
        for &id in pieces {
            self.talk_said.push((npc, with, id, t));
        }
        let n = self.talk_said.len();
        if n > SAID_CAP {
            self.talk_said.drain(..n - SAID_CAP);
        }
    }

    /// The options the squad member has on a concern, gated by who they are.
    pub fn options(&self, npc: PersonId, with: PersonId, concerns: &[Concern], k: usize) -> Vec<Opt> {
        let mut out = Vec::new();
        let Some(c) = concerns.get(k) else { return out };
        if k + 1 < concerns.len() {
            out.push(Opt::More);
        }
        let regard = self.regard_of(npc, with);
        let town = self.people[npc as usize].home;
        let standing = town.map(|t| self.standing(with, t)).unwrap_or(0.0);
        let clean = town.is_some_and(|t| self.bounty_known_in(t) <= 0.0) && self.records.get(&with).copied().unwrap_or(0.0) < 1.0;
        let ev = c.event.and_then(|e| self.event(e));
        // Work they'd give: offered to those they don't distrust.
        if c.opp.and_then(|id| self.opportunity(id)).is_some_and(|o| o.state == OppState::Open) && regard >= 35.0 {
            out.push(Opt::Help);
        }
        // A known thief can be reported, by someone the law will hear.
        // (Not twice: once they've been arrested for it, it's done.)
        let answered = |e: &super::history::Event| self.events(e.town).iter().any(|x| x.deed == Deed::Arrest && x.victim == e.actor && x.t >= e.t);
        // (To the watch of the town it happened in: NM-56.)
        if ev.is_some_and(|e| !e.hidden && e.actor.is_some() && e.deed.is_wrong() && !answered(e) && Some(e.town) == town) && clean && standing >= 0.0 {
            out.push(Opt::Report);
        }
        // A grudge can be talked down by someone they'd listen to.
        if c.subject == Subject::Grudge && (regard >= 50.0 || self.people[with as usize].race == self.people[npc as usize].race) {
            out.push(Opt::Persuade);
        }
        // What they know of a hidden deed can be bought or forced.
        if ev.is_some_and(|e| e.hidden) {
            if self.squad_count(items::id("coin")) >= 10 {
                out.push(Opt::Bribe);
            }
            if self.people[with as usize].might >= 40.0 {
                out.push(Opt::Threaten);
            }
        }
        out
    }

    /// The squad member says something; returns their answer.
    pub fn say_opt(&mut self, npc: PersonId, with: PersonId, concerns: &[Concern], k: usize, opt: Opt) -> String {
        let Some(c) = concerns.get(k).copied() else { return "Hm?".into() };
        let t = self.time;
        let day = World::day_of(t) as i32;
        let town = self.people[npc as usize].home.unwrap_or(0);
        let mut r = Rng::from_keys(&[self.seed, npc as u64, with as u64, k as u64, opt as u64, (t / HOUR) as u64, 0x4F50_5453]);
        match opt {
            Opt::More => {
                let s = self.assemble_talk(npc, with, concerns.get(k + 1), false);
                self.note_said(npc, with, &s.pieces);
                s.text
            }
            Opt::Help => {
                let Some(o) = c.opp.and_then(|id| self.opportunity(id)).cloned() else { return "Never mind.".into() };
                if let Some(x) = self.society.opps.iter_mut().find(|x| x.id == o.id) {
                    x.known = true;
                }
                if let Some(talk) = self.talk.as_mut() {
                    talk.offered = true;
                }
                format!("You'd do that? Then: {}.", self.opp_line(&o))
            }
            Opt::Report => {
                let Some(e) = c.event.and_then(|e| self.event(e)).cloned() else { return "Report what?".into() };
                let Some(a) = e.actor else { return "Report who?".into() };
                let guard = self.settlements[town as usize].residents.iter().copied().find(|&g| self.society.lives[g as usize].job == super::jobs::Job::Guard && !self.people[g as usize].dead);
                if guard.is_none() || self.people[a as usize].dead {
                    return "There's no one to tell.".into();
                }
                self.note(Deed::Arrest, guard, Some(a), town, t, false);
                if let Some(v) = e.victim {
                    let mut jr = Rng::from_keys(&[self.seed, a as u64, day as u64, 0x5245_5054]);
                    self.public_dispute(a, v, town, t, &mut jr);
                }
                // Thanks come from the one wronged, or their household; not
                // from whoever passed the story on.
                let wronged = e.victim.is_some_and(|v| v == npc || (self.society.lives[v as usize].household.is_some() && self.society.lives[v as usize].household == self.society.lives[npc as usize].household));
                if wronged {
                    self.remember(npc, Who::Person(with), Deed::Kindness, 0.3, day);
                }
                self.remember(a, Who::Person(with), Deed::TurnedIn, -0.5, day);
                self.add_standing(with, town, 1.0);
                "Good. Let them answer for it.".into()
            }
            Opt::Persuade => {
                let Some(h) = self.society.lives[npc as usize].household else { return "Hm.".into() };
                let Some(o) = c.about.and_then(|a| self.society.lives.get(a as usize)).and_then(|l| l.household) else { return "Hm.".into() };
                if r.chance(0.3 + self.people[npc as usize].traits.patience * 0.5) {
                    self.feel(h, o, 0.3, day);
                    "...Maybe you're right. Maybe it's gone far enough.".into()
                } else {
                    "Easy for you to say. It wasn't done to you.".into()
                }
            }
            Opt::Bribe | Opt::Threaten => {
                if opt == Opt::Bribe {
                    self.take_from_squad(items::id("coin"), 10);
                    if let Some(h) = self.society.lives[npc as usize].household {
                        self.society.households[h as usize].purse.coin += 10.0;
                    }
                } else {
                    self.remember(npc, Who::Person(with), Deed::Threat, -0.4, day);
                }
                let e = c.event.and_then(|e| self.event(e)).cloned();
                let knows = e.as_ref().and_then(|e| e.actor).is_some_and(|a| self.knows_culprit(npc, a));
                let gives = if opt == Opt::Bribe { r.chance(0.6) } else { r.chance((0.4 + self.people[with as usize].might / 200.0 - self.people[npc as usize].traits.boldness * 0.4).clamp(0.05, 0.9)) };
                match (knows && gives, e) {
                    (true, Some(e)) => {
                        let a = e.actor.unwrap();
                        self.expose(e.id, t);
                        // Anyone the squad's working for wanting to know, knows.
                        for o in self.society.opps.clone() {
                            if o.kind == Chance::FindOut && o.event == Some(e.id) && o.state == OppState::Taken {
                                if let Some(q) = self.quests.iter_mut().find(|q| q.opp == Some(o.id)) {
                                    q.stage = super::quests::Stage::Report;
                                }
                                if let Some(x) = self.society.opps.iter_mut().find(|x| x.id == o.id) {
                                    x.done = true;
                                }
                            }
                        }
                        format!("It was {}. You didn't hear it from me.", self.name_of(a))
                    }
                    _ => "I don't know anything about it.".into(),
                }
            }
        }
    }
}
