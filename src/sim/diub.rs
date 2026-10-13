//! Diʻub and Ḍeshu (`docs/tutorial-island.md`, Laz's tutorial island).
//!
//! - **Diʻub**, a Roduro man and an escaped debtor, hides in the shelter hut
//!   above the shore (`island::HUT_BUILDING`). He's the tutorial's guide on
//!   both paths: he explains things in his own voice instead of pop-up text
//!   (`guide_line`), and his words are `data/lines/diub.txt`.
//! - **His four questions** come over the meal at the hut. The player never
//!   sees a background named, only the questions and their own answers;
//!   each answer quietly adds `BOOST` to the skills listed for it
//!   (`QUESTIONS`, from the brief), and he answers it. "I'd rather not say"
//!   is always there: no boost, and he shrugs. The fourth answered, the
//!   "questions" lesson is done (`tutorial.rs`) and he makes his offer.
//! - **Ḍeshu of the Grey House**, the Roduro Free Lord who owns the island,
//!   is in the overseers' hall, visiting when the player is caught.
//!
//! Both are placed last of all when the world is made (`place_the_strangers`),
//! so nobody else's dice move. Their names are kept as Laz chose them
//! (`assets/lang/canon.ron`: Diʻub's closed ending breaks the Roduro sound
//! rules on purpose).

use serde::{Deserialize, Serialize};

use super::island::HUT_BUILDING;
use super::jobs::Job;
use super::person::{Person, PersonId};
use super::race::Race;
use super::rng;
use super::stats::Skill;
use super::world::World;

/// The two strangers' names.
pub const DIUB: &str = "Diʻub";
pub const DESHU: &str = "Ḍeshu";
/// The lord's house, for his full style.
pub const DESHU_HOUSE: &str = "of the Grey House";
/// What one answer adds to each skill it names.
pub const BOOST: f32 = 5.0;
/// "I'd rather not say": the answer after the four.
pub const RATHER_NOT: u8 = 4;
/// The building the lord keeps to: the overseers' hall.
pub const HALL_BUILDING: u16 = 1;

/// The skills each answer tilts, by question and answer (the brief's table).
pub const QUESTIONS: [[&[Skill]; 4]; 4] = {
    use Skill::*;
    [
        // "What were you doing on that ship?"
        [&[Athletics, Weaving], &[Blade, Block], &[Structured, Inscription], &[Sneak, Security]],
        // "What did you do with your hands, before all this?"
        [&[Carpentry, Masonry], &[Smithing, Armoring], &[Alchemy], &[Marksman, Spear]],
        // "Someone comes at you in the dark. What do you do?"
        [&[Unarmed, Blunt], &[Dodge, Athletics], &[Sneak], &[Felt]],
        // "Where were you going, before the storm?"
        [&[Handcraft, Tending], &[Handcraft, Athletics], &[Structured, Ritual], &[Sneak, Dodge]],
    ]
};

/// The strangers in the world, and how the questions went. Saved.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Strangers {
    pub diub: Option<PersonId>,
    pub lord: Option<PersonId>,
    /// The answers given, in order (`RATHER_NOT` for "I'd rather not say").
    pub answers: Vec<u8>,
}

impl Strangers {
    /// The question up now (0..4), if any are left.
    pub fn question(&self) -> Option<u8> {
        (self.answers.len() < QUESTIONS.len()).then_some(self.answers.len() as u8)
    }

    /// Which way the answers lean: fighting answers against thinking ones
    /// (for who buys the caught out: the Lantern Company or the Writers).
    /// Positive leans to fighting.
    pub fn lean(&self) -> i32 {
        let fighting = [(0, 1), (1, 3), (2, 0), (2, 1)];
        let thinking = [(0, 2), (1, 2), (3, 2)];
        self.answers.iter().enumerate().map(|(q, &a)| fighting.contains(&(q, a as usize)) as i32 - thinking.contains(&(q, a as usize)) as i32).sum()
    }
}

/// A Roduro man of the compound, keeping to one building: the strangers.
/// (A seed that gives a man's name, so the name and the seed agree.)
fn stranger(people: &mut Vec<Person>, seed: u64, tag: u64, town: super::settlement::SettlementId) -> Person {
    let mut k = 0;
    let pseed = loop {
        let s = rng::key(&[seed, tag, k]);
        if super::names::gender(s) == crate::names::Gender::Male || k > 64 {
            break s;
        }
        k += 1;
    };
    let pid = people.len() as PersonId;
    let mut p = Person::summary(pid, pseed, Race::Roduro, Some(town));
    p.home = Some(town);
    p
}

impl World {
    /// Diʻub and the lord, last of all at world creation: residents of the
    /// compound keeping to the hut and the hall, each a household of one
    /// with no trade.
    pub(super) fn place_the_strangers(&mut self) {
        let Some(isle) = self.island.clone() else { return };
        let town = isle.settlement;
        let shore = self.society.towns[town as usize].shore;
        let mut ids = Vec::new();
        for (tag, home, name) in [(0xD1_0B, HUT_BUILDING as u16, DIUB), (0xDE_5A, HALL_BUILDING, DESHU)] {
            let mut p = stranger(&mut self.people, self.seed, tag, town);
            let pid = p.id;
            p.dwelling = Some(home);
            p.wounds.at = self.time;
            p.mana_at = self.time;
            p.ensure_detail();
            if let Some(d) = p.detail.as_mut() {
                d.name = name.to_string();
            }
            self.people.push(p);
            self.busy_until.push(0.0);
            self.group_of.push(None);
            self.settlements[town as usize].residents.push(pid);
            let hh = self.society.households.len() as u32;
            let mut l = super::society::Life::new(Race::Roduro, self.people[pid as usize].seed);
            l.community = Some(shore);
            l.household = Some(hh);
            l.job = Job::None;
            l.place = None;
            self.society.lives.push(l);
            self.society.minds.push(Default::default());
            self.society.households.push(super::society::Household { community: shore, members: vec![pid], home: Some(home), purse: Default::default(), feelings: Vec::new() });
            ids.push(pid);
        }
        self.strangers = Strangers { diub: ids.first().copied(), lord: ids.get(1).copied(), answers: Vec::new() };
        self.refresh_container_owners();
        self.solid_changed();
    }

    /// Is this Diʻub?
    pub fn is_diub(&self, pid: PersonId) -> bool {
        self.strangers.diub == Some(pid)
    }

    /// Is this the lord, Ḍeshu?
    pub fn is_lord(&self, pid: PersonId) -> bool {
        self.strangers.lord == Some(pid)
    }

    /// Does this person keep to their home all day (the strangers, the test
    /// quarter's culture row)?
    pub fn keeps_home(&self, pid: PersonId) -> bool {
        self.is_diub(pid) || self.is_lord(pid) || self.in_culture_row(pid)
    }

    /// A question answered by squad member `who`: the boosts, and the answer
    /// remembered. The fourth done, the lesson is too.
    pub(super) fn answer_question(&mut self, who: PersonId, a: u8) {
        let Some(q) = self.strangers.question() else { return };
        if a < RATHER_NOT {
            let p = &mut self.people[who as usize];
            for &s in QUESTIONS[q as usize][a as usize] {
                let now = p.stats.skill(s);
                p.stats.set_skill(s, now + BOOST);
            }
            p.recompute_might();
        }
        self.strangers.answers.push(a.min(RATHER_NOT));
        if self.strangers.question().is_none() {
            self.lesson_done("questions");
        }
    }

    /// What Diʻub says for a lesson, as its guide (`data/lines/diub.txt`,
    /// slot `guide`, `lesson=<key>`), if he has a line for it.
    pub fn guide_line(&self, lesson: &str) -> Option<String> {
        let tag = format!("lesson={lesson}");
        super::talk::pieces().iter().find(|p| p.topic == "diub" && p.slot == "guide" && p.must.iter().any(|m| *m == tag)).map(|p| super::speech::plain(p.text))
    }

    /// One of Diʻub's set lines (`offer`, `night_market`, ...), by slot.
    pub fn diub_line(&self, slot: &str) -> Option<String> {
        super::talk::pieces().iter().find(|p| p.topic == "diub" && p.slot == slot).map(|p| super::speech::plain(p.text))
    }

    /// What the squad member says for an answer (the reply button).
    pub fn answer_text(&self, q: u8, a: u8) -> String {
        let tags = [format!("q={}", q + 1), format!("a={}", a + 1)];
        super::talk::pieces()
            .iter()
            .find(|p| p.topic == "diub" && p.slot == "answer" && tags.iter().all(|t| p.must.iter().any(|m| m == t)))
            .map(|p| p.text.to_string())
            .unwrap_or_else(|| "...".into())
    }
}
