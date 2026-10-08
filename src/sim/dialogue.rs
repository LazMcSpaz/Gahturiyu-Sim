//! Talking to people, Morrowind-style: a greeting, then topics to ask about.
//!
//! What someone says is built from who they are (people, calling, temper)
//! and what's true in the world right now (their town, the nearest bandit
//! camp, recent ambushes, your bounty). How willing they are to talk is
//! their **disposition** toward you: friendlier if they're sociable and
//! patient, if you're kin, or if you've done them a good turn; colder if
//! your squad has a bounty in their town. Below 20 they won't talk at all.
//!
//! Lines are picked with rolls keyed to the person and topic, so asking the
//! same person the same thing gets the same answer.

use serde::{Deserialize, Serialize};

use super::items;
use super::person::PersonId;
use super::quests::{compass, QuestKind, Stage};
use super::race::Race;
use super::rng::Rng;
use super::stats::Calling;
use super::stealth;
use super::world::{World, HOUR};

/// How close you must be to talk, metres.
pub const TALK_RANGE: f32 = 3.5;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Topic {
    Background,
    ThisTown,
    Advice,
    Rumours,
    Bandits,
    /// They have work to offer.
    Work,
    /// Accept the work just described.
    Accept,
    /// Hand over / report on a job (index into `World::quests`).
    Report(usize),
    /// You carry a letter for them.
    Letter(usize),
    PayBounty,
    /// Ask what they could teach.
    Lessons,
    /// Pay to be taught this spell.
    Learn(super::magic::Spell),
    Goodbye,
}

impl Topic {
    pub fn label(self) -> &'static str {
        match self {
            Topic::Background => "Background",
            Topic::ThisTown => "This town",
            Topic::Advice => "A little advice",
            Topic::Rumours => "Latest rumours",
            Topic::Bandits => "Bandits",
            Topic::Work => "Any work?",
            Topic::Accept => "I'll do it.",
            Topic::Report(_) => "About that job...",
            Topic::Letter(_) => "A letter for you",
            Topic::PayBounty => "Pay my bounty",
            Topic::Lessons => "Could you teach me?",
            Topic::Learn(_) => "Teach me",
            Topic::Goodbye => "Goodbye",
        }
    }

    /// What the button says.
    pub fn text(self) -> String {
        match self {
            Topic::Learn(s) => format!("Teach me {} ({} coin)", s.def().name.to_lowercase(), World::lesson_price(s)),
            t => t.label().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Conversation {
    /// Who's talking for the squad.
    pub with: PersonId,
    pub npc: PersonId,
    /// What's been said, newest last: (spoken by the npc?, text).
    pub lines: Vec<(bool, String)>,
    /// Whether work has been described (so "I'll do it" shows).
    pub offered: bool,
    /// Whether they've said what they could teach (so the lessons show).
    #[serde(default)]
    pub lessons: bool,
}

impl World {
    /// How they feel about the squad, 0..100.
    pub fn disposition(&self, npc: PersonId, with: PersonId) -> f32 {
        let p = &self.people[npc as usize];
        let you = &self.people[with as usize];
        let mut d = 35.0 + p.traits.sociability * 30.0 + p.traits.patience * 10.0;
        if p.race == you.race {
            d += 10.0;
        } else if matches!((p.race, you.race), (Race::Roduro, Race::Horaro) | (Race::Horaro, Race::Roduro)) {
            d += 6.0; // the old alliance
        }
        // (Not if they don't know who you are.)
        if let Some(h) = p.home.filter(|_| self.boon(with, super::effects::Does::Disguise) <= 0.0) {
            d -= self.bounty_known_in(h) / 4.0;
        }
        d += self.regard.get(&npc).copied().unwrap_or(0.0);
        // A swaying word.
        d += self.boon(with, super::effects::Does::Sway);
        d.clamp(0.0, 100.0)
    }

    /// Send a squad member over to talk to someone.
    pub fn order_talk(&mut self, who: PersonId, npc: PersonId) -> bool {
        let p = &self.people[npc as usize];
        if p.bandit || p.in_squad || p.dead {
            return false;
        }
        let Some(k) = self.squad.index(who) else { return false };
        let at = self.person_pos(npc);
        let (path, _) = self.route(self.member_pos(k), at);
        self.squad.goal[k] = *path.last().unwrap_or(&at);
        self.squad.route[k] = path;
        self.want_talk = Some((who, npc));
        self.try_open_talk();
        true
    }

    /// Open the conversation once they're close enough.
    pub(super) fn try_open_talk(&mut self) {
        let Some((who, npc)) = self.want_talk else { return };
        let Some(k) = self.squad.index(who) else {
            self.want_talk = None;
            return;
        };
        if self.member_pos(k).dist(self.person_pos(npc)) > TALK_RANGE {
            return;
        }
        self.want_talk = None;
        self.squad.goal[k] = self.squad.at[k];
        self.squad.route[k].clear();
        if self.people[npc as usize].ensure_detail() {
            self.stats.detailed += 1;
        }
        let greeting = self.greeting(npc, who);
        self.talk = Some(Conversation { with: who, npc, lines: vec![(true, greeting)], offered: false, lessons: false });
    }

    pub fn end_talk(&mut self) {
        self.talk = None;
    }

    fn greeting(&self, npc: PersonId, with: PersonId) -> String {
        let p = &self.people[npc as usize];
        let d = self.disposition(npc, with);
        let night = stealth::daylight(self.time) < 0.5;
        let mut r = Rng::from_keys(&[p.seed, (self.time / HOUR) as u64, 0x4752_4545]);
        if d < 20.0 {
            return ["I've nothing to say to the likes of you.", "Move along.", "Not you. Go away."][r.below(3)].to_string();
        }
        let warm = d > 60.0;
        let base = match p.race {
            Race::Roduro => {
                if warm {
                    "Well met. Mind the moss on the path, it's slick."
                } else {
                    "Hm. A traveller."
                }
            }
            Race::Horaro => {
                if warm {
                    "Ho there! Salt and fair weather to you."
                } else {
                    "You're dripping nothing on my deck, so I suppose you can stay."
                }
            }
            Race::Qotiro => {
                if warm {
                    "Speak plainly and I'll answer plainly."
                } else {
                    "State your business."
                }
            }
            Race::Tadoro => {
                if warm {
                    "Oh — a new face. Do you mind if I write you down?"
                } else {
                    "I was in the middle of a thought."
                }
            }
        };
        if night {
            format!("{base} It's late — Hiyaḍote's hours.")
        } else {
            base.to_string()
        }
    }

    /// What can be asked right now.
    pub fn topics(&self) -> Vec<Topic> {
        let Some(c) = &self.talk else { return vec![] };
        let mut t = vec![Topic::Background, Topic::ThisTown, Topic::Advice, Topic::Rumours, Topic::Bandits];
        if self.disposition(c.npc, c.with) < 20.0 {
            return vec![Topic::Goodbye];
        }
        for (i, q) in self.quests.iter().enumerate() {
            if q.giver == c.npc && (q.stage == Stage::Report || matches!(q.kind, QuestKind::Fetch { .. }) && q.stage == Stage::Active) {
                t.push(Topic::Report(i));
            }
            if let QuestKind::Deliver { to } = q.kind {
                if to == c.npc && q.stage == Stage::Active && self.squad_count(items::id("sealed_letter")) > 0 {
                    t.push(Topic::Letter(i));
                }
            }
        }
        if self.quest_offer(c.npc).is_some() {
            t.push(if c.offered { Topic::Accept } else { Topic::Work });
        }
        if let Some(h) = self.people[c.npc as usize].home {
            if self.bounty_known_in(h) > 0.0 {
                t.push(Topic::PayBounty);
            }
        }
        // Mages teach what they know, for coin.
        let lessons = self.lessons(c.npc, c.with);
        if !lessons.is_empty() {
            if c.lessons {
                t.extend(lessons.into_iter().take(6).map(Topic::Learn));
            } else {
                t.push(Topic::Lessons);
            }
        }
        t.push(Topic::Goodbye);
        t
    }

    /// Ask about something; their answer goes into the conversation.
    pub fn ask(&mut self, topic: Topic) {
        let Some(c) = self.talk.clone() else { return };
        if topic == Topic::Goodbye {
            let p = &self.people[c.npc as usize];
            let bye = match p.race {
                Race::Horaro => "May Horahìda carry you.",
                Race::Roduro => "Dodìṭo keep you steady.",
                Race::Qotiro => "Go with Qotisho's fire.",
                Race::Tadoro => "Rìthaduya guide your road.",
            };
            self.push_talk(false, topic.label().to_string());
            self.push_talk(true, bye.to_string());
            self.talk = None;
            return;
        }
        self.push_talk(false, topic.text());
        if topic == Topic::Lessons {
            if let Some(c) = self.talk.as_mut() {
                c.lessons = true;
            }
        }
        let answer = self.answer(&c, topic);
        self.push_talk(true, answer);
    }

    fn push_talk(&mut self, npc: bool, line: String) {
        if let Some(c) = self.talk.as_mut() {
            c.lines.push((npc, line));
            if c.lines.len() > 30 {
                c.lines.remove(0);
            }
        }
    }

    fn answer(&mut self, c: &Conversation, topic: Topic) -> String {
        let p = self.people[c.npc as usize].clone();
        let mut r = Rng::from_keys(&[p.seed, topic_key(topic), 0x5441_4C4B]);
        let town = p.home.map(|h| self.settlements[h as usize].clone());
        match topic {
            Topic::Background => {
                let race = match p.race {
                    Race::Roduro => "A Stone Tender began growing my house when I was small. It's grown up alongside me — see the bands? Same as mine.",
                    Race::Horaro => "Born on the stilts, swimming before walking. The land-folk think we're strange for sleeping over the water. We think they're strange for not.",
                    Race::Qotiro => "My people quarry their homes and forge their lives. Nothing is given; everything is made.",
                    Race::Tadoro => "I lodge here, for now. I keep notes on everything — the tides, the arguments, how long the bread lasts. Someone should.",
                };
                let work = match p.stats.calling {
                    Calling::Warrior => " I've fought for coin, when there was coin to fight for.",
                    Calling::Hunter => " I hunt, mostly. The land feeds those who watch it.",
                    Calling::Mage => " And I study the old arts, when no one's watching too closely.",
                    Calling::Common => ["  I mend nets.", " I keep goats, and the goats keep me.", " I carry stone for the Tenders.", " I trade what I can."][r.below(4)],
                };
                format!("{race}{work}")
            }
            Topic::ThisTown => {
                let Some(t) = town else { return "I don't belong anywhere in particular.".into() };
                let mut counts = [0usize; 4];
                for &m in &t.residents {
                    counts[self.people[m as usize].race.index()] += 1;
                }
                let most = super::race::ALL_RACES.iter().zip(counts).max_by_key(|(_, n)| *n).map(|(r, _)| r.name()).unwrap_or("?");
                let shore = if t.coastal {
                    " The Horaro live on their stilts just off the shore; without them, there'd be no fish and no trade by water."
                } else {
                    ""
                };
                format!("{} — the {} founded it. {} of us live here, {} most of all.{shore}", t.name, t.founders.name(), t.residents.len(), most)
            }
            Topic::Advice => [
                "Travel by day if you can. Bandits by the road see you a long way off in the sun, but at night they mostly have to hear you.",
                "Heavy armour clanks. If you mean to sneak, leave the scale shirt at home.",
                "Keep a healing draught in your pack. Two, if you're the sort who goes looking for trouble.",
                "Lockpicks snap. Carry more than you think you need — and don't let anyone see you use them.",
                "A mortar and pestle weighs less than a dead friend. Learn to brew.",
                "Bandits only jump people they think they can beat. Look strong, travel together.",
                "Doors lock at night. By day, most folk don't mind you stepping in, so long as your hands stay empty.",
            ][r.below(7)]
            .to_string(),
            Topic::Rumours => {
                let near = self.nearest_camp(p.home);
                let mut lines = vec![];
                if self.stats.ambushes > 0 {
                    lines.push(format!("Bandits have fallen on travellers {} times since the season turned. People say the roads aren't what they were.", self.stats.ambushes));
                }
                if let Some((d, dir, _)) = near {
                    lines.push(format!("Folk coming in say there's a camp by the road {:.1} km {dir} of here.", d / 1000.0));
                }
                if let Some(h) = p.home {
                    for (origin, _) in self.bounties_known_in(h) {
                        if origin == h {
                            lines.push("Someone's been at the locks round here. If I find out who...".into());
                        } else {
                            lines.push(format!("Word from {} is there's thieves on the road. Keep your door shut.", self.settlements[origin as usize].name));
                        }
                    }
                }
                lines.push("They say the Ṭaḍoro write down everything you tell them. Mind what you say.".into());
                lines.swap_remove(r.below(lines.len()))
            }
            Topic::Bandits => match self.nearest_camp(p.home) {
                Some((d, dir, n)) if d < 6000.0 => format!(
                    "There's a band of {n} camped about {:.1} km {dir} of here, by the road. They pick off anyone who looks weak. Go in force, or go at night and go quiet.",
                    d / 1000.0
                ),
                _ => "None close, thank the gods. Not that I've heard.".into(),
            },
            Topic::Work => match self.quest_offer(c.npc) {
                Some((kind, coin, bonus)) => {
                    if let Some(t) = self.talk.as_mut() {
                        t.offered = true;
                    }
                    let extra = bonus.map(|b| format!(" — and {} besides", items::item(b).name.to_lowercase())).unwrap_or_default();
                    match kind {
                        QuestKind::ClearCamp { at, .. } => {
                            let v = at.sub(town.map(|t| t.pos).unwrap_or(at));
                            format!(
                                "Those bandits {:.1} km {} of here have had my cousin twice. Break that camp and I'll pay {coin} coin{extra}.",
                                v.len() / 1000.0,
                                compass(v)
                            )
                        }
                        QuestKind::Fetch { item, count } => format!("I need {count} × {}. Bring them and there's {coin} coin in it.", items::item(item).name.to_lowercase()),
                        QuestKind::Deliver { to } => {
                            let q = &self.people[to as usize];
                            let place = q.home.map(|h| self.settlements[h as usize].name.clone()).unwrap_or_default();
                            format!("Would you carry a letter to {} in {place}? Sealed, mind. {coin} coin when it's done — they'll pay you.", super::names::person_name(q.race, q.seed))
                        }
                    }
                }
                None => "Nothing I'd trust a stranger with.".into(),
            },
            Topic::Accept => match self.accept_quest(c.npc, c.with) {
                Some(id) => {
                    if let Some(t) = self.talk.as_mut() {
                        t.offered = false;
                    }
                    let line = self.quest_line(&self.quests[id as usize].clone());
                    self.log.push_front((self.time, format!("New job: {line}")));
                    self.log.truncate(14);
                    "Good. Don't make me regret it.".into()
                }
                None => "Never mind.".into(),
            },
            Topic::Report(i) => {
                let q = self.quests[i].clone();
                match (q.kind, q.stage) {
                    (_, Stage::Report) => {
                        self.reward(i, c.with);
                        format!("You did it? Then here — {} coin, as promised.", q.coin)
                    }
                    (QuestKind::Fetch { item, count }, Stage::Active) => {
                        if self.squad_count(item) >= count {
                            self.take_from_squad(item, count);
                            self.reward(i, c.with);
                            format!("That's all of them. {} coin — fair's fair.", q.coin)
                        } else {
                            format!("That's not {count}. Come back when it is.")
                        }
                    }
                    _ => "It's not done yet.".into(),
                }
            }
            Topic::Letter(i) => {
                self.take_from_squad(items::id("sealed_letter"), 1);
                self.reward(i, c.with);
                let from = self.people[self.quests[i].giver as usize].name().unwrap_or("someone").to_string();
                format!("From {from}? At last. Here, for your trouble — {} coin.", self.quests[i].coin)
            }
            Topic::PayBounty => {
                let Some(h) = p.home else { return String::new() };
                let known = self.bounties_known_in(h);
                let owed = known.iter().map(|b| b.1).sum::<f32>().ceil() as u16;
                let have = self.squad_count(items::id("coin"));
                if have >= owed {
                    self.take_from_squad(items::id("coin"), owed);
                    for (origin, _) in known {
                        self.bounty_settled(origin);
                    }
                    format!("{owed} coin. Consider the matter closed — this time.")
                } else {
                    format!("You owe {owed}. You've got {have}. Come back when you can pay.")
                }
            }
            Topic::Lessons => {
                let what: Vec<String> = self.lessons(c.npc, c.with).iter().take(6).map(|s| format!("{} ({} coin)", s.def().name.to_lowercase(), World::lesson_price(*s))).collect();
                format!("I could show you {}. It takes coin, mind — learning isn't free.", what.join(", "))
            }
            Topic::Learn(s) => {
                let price = World::lesson_price(s);
                if self.learn_from(c.with, s) {
                    let who = self.people[c.with as usize].name().unwrap_or("you").to_string();
                    let t = self.time;
                    self.log.push_front((t, format!("{who} learns {} for {price} coin.", s.def().name.to_lowercase())));
                    self.log.truncate(14);
                    format!("Watch closely, then. ... There — {} is yours now.", s.def().name.to_lowercase())
                } else {
                    format!("That's {price} coin, and you haven't got it.")
                }
            }
            Topic::Goodbye => String::new(),
        }
    }

    /// Distance, direction and size of the bandit camp nearest a town.
    fn nearest_camp(&self, home: Option<u16>) -> Option<(f32, &'static str, usize)> {
        let from = home.map(|h| self.settlements[h as usize].pos).unwrap_or(self.squad.pos);
        let c = self.camps.iter().min_by(|a, b| a.pos.dist(from).total_cmp(&b.pos.dist(from)))?;
        let n = self.group(c.group).map(|g| g.members.iter().filter(|&&m| !self.people[m as usize].dead).count()).unwrap_or(0);
        Some((c.pos.dist(from), compass(c.pos.sub(from)), n))
    }
}

fn topic_key(t: Topic) -> u64 {
    match t {
        Topic::Background => 1,
        Topic::ThisTown => 2,
        Topic::Advice => 3,
        Topic::Rumours => 4,
        Topic::Bandits => 5,
        Topic::Work => 6,
        Topic::Accept => 7,
        Topic::Report(i) => 100 + i as u64,
        Topic::Letter(i) => 10_000 + i as u64,
        Topic::PayBounty => 8,
        Topic::Goodbye => 9,
        Topic::Lessons => 10,
        Topic::Learn(s) => 20_000 + s.0 as u64,
    }
}

