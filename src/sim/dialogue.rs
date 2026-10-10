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

use super::crafting::RECIPES;
use super::items::{self, SLOTS};
use super::materials::Grade;
use super::person::PersonId;
use super::quests::{compass, QuestKind, Stage};
use super::race::Race;
use super::rng::Rng;

use super::world::{World, DAY};

/// How close you must be to talk, metres.
pub const TALK_RANGE: f32 = 3.5;
/// Further apart than this, a talk is over.
pub const TALK_PARTED: f32 = 8.0;

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
    /// See what a merchant has.
    Trade,
    /// Buy one of this, at this price.
    Buy(items::ItemId, u16),
    /// Sell one of this, at this price.
    Sell(items::ItemId, u16),
    /// Coin for a note, at the exchange.
    ToNote,
    /// A note for coin.
    ToCoin,
    /// Pay this crafter to mend a squad member's piece: (whose, which slot in
    /// `SLOTS`, price).
    Mend(PersonId, u8, u16),
    /// Pay a crafter to teach their craft, at this price.
    CraftLesson(u16),
    /// Ask a Tender what they'd grow to order.
    Orders,
    /// Order a grown piece: (recipe, price).
    Order(u16, u16),
    /// Collect an order (index into `World::orders`).
    Collect(u16),
    /// Buy a squad member out of their bond, at this price.
    BuyOut(PersonId, u16),
    /// Take up a post in this town's government.
    TakePost(super::law::Post),
    /// Press them about a job: (opportunity, how).
    Press(u32, super::chances::Press),
    /// What's posted at the hall.
    Board,
    /// Take a job posted at the hall.
    TakeJob(u32),
    /// Work a post that's going in town: (job, workplace).
    PostWork(super::jobs::Job, u16),
    /// Say something about what's on their mind.
    Say(super::talk::Opt),
    /// Give up the work you're doing here.
    QuitWork,
    /// Come and work at the squad's outpost, for this many coin a day.
    Hire(u16),
    /// Join the squad, for this signing fee (0: for nothing).
    Join(u16),
    /// Sell every one of these the merchant will take.
    SellAll(items::ItemId),
    /// Something a squad member has on that the merchant would buy: who's
    /// wearing it and about what it would fetch. Asking only gets told to
    /// take it off first.
    SellWorn(items::ItemId, PersonId, u16),
    /// Show everything the merchant would take, not just the dearest few.
    SellRest,
    Goodbye,
    /// Ask someone who won't come to join anyway: they say why not.
    /// (Last, so saves made before it still read.)
    AskJoin,
    /// Have the healer see to the hurt, for this many coin.
    Treat(u16),
    /// Take beds at the inn for the night, for this many coin in all.
    RentBeds(u16),
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
            Topic::Trade => "What have you got?",
            Topic::Buy(..) => "Buy",
            Topic::Sell(..) => "Sell",
            Topic::ToNote => "Change coin for a note",
            Topic::ToCoin => "Change a note for coin",
            Topic::Mend(..) => "Mend this",
            Topic::CraftLesson(_) => "Teach me your trade",
            Topic::Orders => "What could you grow for me?",
            Topic::Order(..) => "Order",
            Topic::Collect(_) => "Is my order ready?",
            Topic::BuyOut(..) => "Buy out a bond",
            Topic::TakePost(_) => "Take up a post",
            Topic::Press(_, super::chances::Press::Ask) => "About that matter...",
            Topic::Press(_, super::chances::Press::Pay) => "I'll make it worth your while.",
            Topic::Press(_, super::chances::Press::Threaten) => "Tell me, or else.",
            Topic::Board => "What's posted here?",
            Topic::TakeJob(_) => "I'll take that job",
            Topic::PostWork(..) => "I'm looking for work",
            Topic::QuitWork => "I'm giving up this work",
            Topic::Hire(_) => "Come and work at my outpost",
            Topic::Join(_) | Topic::AskJoin => "Come with us",
            Topic::Treat(_) => "See to my people's wounds",
            Topic::RentBeds(_) => "Beds for the night",
            Topic::SellAll(..) => "Sell all",
            Topic::SellWorn(..) => "Sell what's being worn",
            Topic::SellRest => "What else would you take?",
            Topic::Say(o) => o.label(),
            Topic::Goodbye => "Goodbye",
        }
    }

    /// What the button says.
    pub fn text(self) -> String {
        match self {
            Topic::Learn(s) => format!("Teach me {} ({} coin)", s.def().name.to_lowercase(), World::lesson_price(s)),
            Topic::Buy(it, p) => format!("Buy {} ({p} coin)", items::item(it).name.to_lowercase()),
            Topic::Sell(it, p) => format!("Sell {} ({p} coin)", items::item(it).name.to_lowercase()),
            Topic::ToNote => format!("Change {} coin for a note", super::economy::NOTE_VALUE + super::economy::EXCHANGE_FEE),
            Topic::ToCoin => format!("Change a note for {} coin", super::economy::NOTE_VALUE - super::economy::EXCHANGE_FEE),
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
    /// Whether their wares are laid out (so buying and selling show).
    #[serde(default)]
    pub trading: bool,
    /// Whether a Tender has said what they'd grow (so the orders show).
    #[serde(default)]
    pub orders: bool,
    /// What's on their mind (`talk.rs`), and which of it is being talked about.
    #[serde(default)]
    pub concerns: Vec<super::talk::Concern>,
    #[serde(default)]
    pub at: usize,
    /// Every piece said in this talk, and whether they refused to talk.
    #[serde(default)]
    pub pieces: Vec<u16>,
    #[serde(default)]
    pub refused: bool,
    /// What's been tried in this talk already: (option, which concern).
    #[serde(default)]
    pub tried: Vec<(super::talk::Opt, usize)>,
    /// Which page of what they'd buy is showing: 0 is their wares and the
    /// few things that fetch most; later pages are the rest of the sell list.
    #[serde(default)]
    pub sell_page: u8,
}

/// How many kinds of thing the sell list shows before "What else would you
/// take?" (what fetches most first, so the best of the loot is never the
/// part cut off).
pub const SELL_SHOWN: usize = 10;
/// How many kinds a later page of the sell list holds (what fits the talk
/// panel with nothing else on it).
pub const SELL_PAGE: usize = 18;
/// How many worn things are pointed out as sellable.
pub const WORN_SHOWN: usize = 3;

impl World {
    /// What a topic's button says, naming what it's about.
    pub fn topic_text(&self, t: Topic) -> String {
        match t {
            Topic::Mend(owner, slot, p) => {
                let what = self.people[owner as usize].detail.as_ref().and_then(|d| d.gear.in_slot(SLOTS[slot as usize])).map(|id| items::item(id).name.to_lowercase()).unwrap_or_default();
                let whose = self.people[owner as usize].name().unwrap_or("?").to_string();
                format!("Mend {whose}'s {what} ({p} coin)")
            }
            Topic::CraftLesson(p) => match self.talk.as_ref().and_then(|c| self.craft_lesson(c.npc, c.with)) {
                Some((k, _, style)) => format!("Teach me {} — {} ({p} coin)", k.name(), style.name()),
                None => format!("Teach me your trade ({p} coin)"),
            },
            Topic::BuyOut(m, p) => format!("Buy {} out of their bond ({p} coin)", self.people[m as usize].name().unwrap_or("?")),
            Topic::TakePost(post) => format!("Take up the post of {}", post.name()),
            Topic::TakeJob(id) => match self.opportunity(id) {
                Some(o) => format!("I'll take it: {}", self.opp_line(o)),
                None => t.text(),
            },
            Topic::PostWork(job, place) => {
                let at = self.talk.as_ref().and_then(|c| self.people[c.npc as usize].home).and_then(|h| self.society.towns[h as usize].places.get(place as usize)).map(|p| format!(" at the {}", p.kind.name().to_lowercase())).unwrap_or_default();
                format!("I'll work as {}{at} ({} coin a day, 8 till 5)", job.name().to_lowercase(), self.post_wage(job))
            }
            Topic::Hire(wage) => format!("Come and work at my outpost ({wage} coin a day)"),
            Topic::SellAll(it) => {
                // The coin on the button is the coin received: worked out
                // sale by sale, since the price of a good drops as they buy.
                let (n, total) = self.talk.as_ref().map(|c| self.sell_all_quote(c.npc, it)).unwrap_or((0, 0));
                let (have, name) = (self.squad_count(it), items::item(it).name.to_lowercase());
                // (Kept short: the window's topic column is narrow.)
                if n >= have {
                    format!("Sell all {have} × {name} — {total} coin")
                } else {
                    format!("Sell {n} of {have} × {name} — {total} coin")
                }
            }
            Topic::SellWorn(it, _, p) => {
                let name = items::item(it).name;
                format!("{name} (worn: take it off to sell, ~{p} coin)")
            }
            Topic::SellRest => {
                let left = self.talk.as_ref().map(|c| self.sell_kinds(c.npc).len().saturating_sub(sell_seen(c.sell_page))).unwrap_or(0);
                format!("What else would you take? ({left} more)")
            }
            Topic::Join(0) => "Come with us — join the squad".into(),
            Topic::Join(fee) => format!("Come with us — join the squad ({fee} coin to sign on)"),
            Topic::Treat(p) => format!("See to my people's wounds ({p} coin)"),
            Topic::RentBeds(p) => format!("Beds for the night ({p} coin)"),
            Topic::Order(ri, p) => format!("Grow me a {} ({p} coin, half now; {:.0} days)", items::item(RECIPES[ri as usize].item(Grade::Common)).name.to_lowercase(), RECIPES[ri as usize].time / DAY),
            Topic::Collect(k) => match self.orders.get(k as usize) {
                Some(o) if self.order_ready(k as usize) => format!("Collect my {} ({} coin owed)", items::item(RECIPES[o.recipe as usize].item(o.grade)).name.to_lowercase(), o.rest),
                Some(o) => format!("How's my {} coming on?", items::item(RECIPES[o.recipe as usize].item(Grade::Common)).name.to_lowercase()),
                None => t.text(),
            },
            t => t.text(),
        }
    }

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
        if !self.valid_person(npc) {
            return false;
        }
        let p = &self.people[npc as usize];
        if p.bandit || p.in_squad || p.dead || self.is_indoors_asleep(npc) || self.is_down(who) {
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
        // What they say first is put together from what's on their mind.
        let concerns = self.on_mind(npc);
        let said = self.assemble_talk(npc, who, concerns.first(), true);
        self.note_said(npc, who, &said.pieces);
        self.talk = Some(Conversation { with: who, npc, lines: vec![(true, said.text)], offered: false, lessons: false, trading: false, orders: false, concerns, at: 0, pieces: said.pieces, refused: said.refused, tried: Vec::new(), sell_page: 0 });
    }

    pub fn end_talk(&mut self) {
        self.talk = None;
    }

    /// A talk ends when the two part, or when either can't go on (NM-54):
    /// nothing is said, sold or agreed at a distance.
    pub(super) fn close_parted_talk(&mut self) {
        let Some((with, npc)) = self.talk.as_ref().map(|c| (c.with, c.npc)) else { return };
        let apart = match self.squad.index(with) {
            Some(k) => self.member_pos(k).dist(self.person_pos(npc)) > TALK_PARTED,
            None => true,
        };
        if apart || self.is_down(with) || self.people[npc as usize].dead || self.is_indoors_asleep(npc) {
            self.talk = None;
        }
    }

    /// What can be asked right now.
    pub fn topics(&self) -> Vec<Topic> {
        let Some(c) = &self.talk else { return vec![] };
        if c.refused || self.people[c.npc as usize].in_squad || self.regard_of(c.npc, c.with) < super::talk::DISTRUST {
            return vec![Topic::Goodbye];
        }
        // What the squad member can say about what's on their mind.
        let mut t: Vec<Topic> = self.options(c.npc, c.with, &c.concerns, c.at).into_iter().filter(|o| *o == super::talk::Opt::More || !c.tried.contains(&(*o, c.at))).map(Topic::Say).collect();
        t.extend([Topic::Background, Topic::ThisTown, Topic::Advice, Topic::Rumours, Topic::Bandits]);
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
        // A bounty is paid to the watch or the town's officials, not to
        // whoever happens to be in the street.
        let collects = matches!(self.life(c.npc).job, super::jobs::Job::Guard | super::jobs::Job::Official | super::jobs::Job::Arbiter);
        if let Some(h) = self.people[c.npc as usize].home.filter(|_| collects) {
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
        // Merchants at their stalls trade; exchangers at work change money.
        if self.is_trading(c.npc) {
            if c.trading {
                // One line per kind of thing (several of it sell together),
                // what fetches most first. The first page is their wares,
                // the best of what you carry and what's being worn; the rest
                // of the sell list comes a page at a time on asking.
                let kinds = self.sell_kinds(c.npc);
                let page = if sell_seen(c.sell_page.saturating_sub(1)) < kinds.len() { c.sell_page } else { 0 };
                let line = |&(it, p, _): &(items::ItemId, u16, u16)| if self.squad_count(it) > 1 { Topic::SellAll(it) } else { Topic::Sell(it, p) };
                if page == 0 {
                    t.extend(self.for_sale(c.npc).into_iter().take(7).map(|(it, _, p)| Topic::Buy(it, p)));
                    t.extend(kinds.iter().take(SELL_SHOWN).map(line));
                    if kinds.len() > SELL_SHOWN {
                        t.push(Topic::SellRest);
                    }
                    // And what they'd buy off someone's back, so nobody
                    // wonders why the helm isn't on the list. (One line per
                    // kind of thing: three hide coats are one hint.)
                    let mut kinds_worn: Vec<items::ItemId> = Vec::new();
                    for (it, m, p) in self.worn_sellable(c.npc) {
                        if kinds_worn.len() < WORN_SHOWN && !kinds_worn.contains(&it) {
                            kinds_worn.push(it);
                            t.push(Topic::SellWorn(it, m, p));
                        }
                    }
                } else {
                    // A later page: nothing but the next of the sell list,
                    // and the way back to their wares.
                    t.clear();
                    t.extend(kinds.iter().skip(sell_seen(page - 1)).take(SELL_PAGE).map(line));
                    if kinds.len() > sell_seen(page) {
                        t.push(Topic::SellRest);
                    }
                    t.push(Topic::Trade);
                }
            } else {
                t.push(Topic::Trade);
            }
        }
        if self.is_exchanging(c.npc) {
            t.push(Topic::ToNote);
            t.push(Topic::ToCoin);
        }
        // Crafters at work mend what they know how to work, and teach their trade.
        if let Some(craft) = self.life(c.npc).job.craft().filter(|_| self.at_work(c.npc, self.time)) {
            let mut mends = Vec::new();
            for &m in &self.squad.members {
                for (k, &slot) in SLOTS.iter().enumerate() {
                    let Some(id) = self.people[m as usize].detail.as_ref().and_then(|d| d.gear.in_slot(slot)) else { continue };
                    if craft.mends(items::craft_of(id)) {
                        if let Some(p) = self.mend_price(m, slot).filter(|_| items::repairable(id)) {
                            mends.push(Topic::Mend(m, k as u8, p));
                        }
                    }
                }
            }
            t.extend(mends.into_iter().take(4));
            if let Some((_, price, _)) = self.craft_lesson(c.npc, c.with) {
                t.push(Topic::CraftLesson(price));
            }
        }
        // A bond is bought out from any official, arbiter or guard of the
        // town that holds it, wherever they are met (NM-37).
        if matches!(self.life(c.npc).job, super::jobs::Job::Official | super::jobs::Job::Arbiter | super::jobs::Job::Guard) {
            if let Some(town) = self.people[c.npc as usize].home {
                for &m in &self.squad.members {
                    if let (Some(b), Some(p)) = (self.bond_of(m), self.buy_out_price(m)) {
                        if b.town == town {
                            t.push(Topic::BuyOut(m, p));
                        }
                    }
                }
            }
        }
        // At the hall: posts taken up.
        if matches!(self.life(c.npc).job, super::jobs::Job::Official | super::jobs::Job::Arbiter) && self.at_work(c.npc, self.time) {
            if let Some(town) = self.people[c.npc as usize].home {
                if self.standing(c.with, town) >= super::law::COUNCIL {
                    use super::law::Post;
                    for post in [Post::Elder, Post::Priestess, Post::Administrator, Post::Speaker, Post::Arbiter] {
                        if self.eligible(c.with, post) {
                            t.push(Topic::TakePost(post));
                        }
                    }
                }
            }
        }
        // Jobs the squad has taken that this person could move on.
        for id in self.pressable(c.npc) {
            use super::chances::Press;
            t.extend([Topic::Press(id, Press::Ask), Topic::Press(id, Press::Pay), Topic::Press(id, Press::Threaten)]);
        }
        // At the hall: what's posted, and posts going.
        if matches!(self.life(c.npc).job, super::jobs::Job::Official | super::jobs::Job::Arbiter) && self.at_work(c.npc, self.time) {
            if let Some(town) = self.people[c.npc as usize].home {
                if !self.hall_board(town).is_empty() {
                    t.push(Topic::Board);
                    t.extend(self.hall_board(town).into_iter().filter(|&id| self.opportunity(id).is_some_and(|o| o.known)).take(4).map(Topic::TakeJob));
                }
                if self.contract_of(c.with).is_some() {
                    t.push(Topic::QuitWork);
                } else {
                    t.extend(self.vacant_posts(town).into_iter().take(2).map(|(j, pl)| Topic::PostWork(j, pl)));
                }
            }
        }
        // Anyone footloose (or a carpenter or mason) may come and work at
        // the squad's outpost, for a wage.
        if let Some(wage) = self.hire_terms(c.npc) {
            t.push(Topic::Hire(wage));
        }
        // The restless may take to the road with the squad.
        // (Anyone else, asked, says why not in a line.)
        match self.join_terms(c.npc) {
            Some(fee) => t.push(Topic::Join(fee)),
            None if self.people[c.npc as usize].home.is_some() && !self.people[c.npc as usize].in_squad => t.push(Topic::AskJoin),
            None => {}
        }
        // A healer at work sees to the hurt; an innkeeper lets beds.
        if let Some((_, price)) = self.treat_terms(c.npc) {
            t.push(Topic::Treat(price));
        }
        if let Some((_, price)) = self.bed_terms(c.npc) {
            t.push(Topic::RentBeds(price));
        }
        // Tenders take orders for grown pieces.
        if !self.order_options(c.npc).is_empty() {
            if c.orders {
                t.extend(self.order_options(c.npc).into_iter().take(8).map(|(ri, p)| Topic::Order(ri as u16, p)));
            } else {
                t.push(Topic::Orders);
            }
        }
        if self.at_work(c.npc, self.time) {
            t.extend(self.orders_with(c.npc).into_iter().map(|k| Topic::Collect(k as u16)));
        }
        t.push(Topic::Goodbye);
        t
    }

    /// Ask about something; their answer goes into the conversation.
    pub fn ask(&mut self, topic: Topic) {
        let Some(c) = self.talk.clone() else { return };
        if topic == Topic::Goodbye {
            let bye = self.farewell(c.npc, c.with);
            self.note_said(c.npc, c.with, &bye.pieces);
            self.push_talk(false, topic.label().to_string());
            self.push_talk(true, bye.text);
            self.talk = None;
            return;
        }
        if let Topic::Say(opt) = topic {
            self.push_talk(false, opt.label().to_string());
            let answer = self.say_opt(c.npc, c.with, &c.concerns, c.at, opt);
            if let Some(t) = self.talk.as_mut() {
                t.tried.push((opt, t.at));
                if opt == super::talk::Opt::More {
                    t.at += 1;
                }
            }
            self.push_talk(true, answer);
            return;
        }
        self.push_talk(false, self.topic_text(topic));
        if topic == Topic::Lessons {
            if let Some(c) = self.talk.as_mut() {
                c.lessons = true;
            }
        }
        if topic == Topic::Trade {
            if let Some(c) = self.talk.as_mut() {
                c.trading = true;
                c.sell_page = 0;
            }
        }
        if topic == Topic::SellRest {
            if let Some(c) = self.talk.as_mut() {
                c.sell_page = c.sell_page.saturating_add(1);
            }
        }
        if topic == Topic::Orders {
            if let Some(c) = self.talk.as_mut() {
                c.orders = true;
            }
        }
        let answer = self.answer(&c, topic);
        self.push_talk(true, answer);
    }

    /// A reply from one of the asked-about line files. Steady: the same
    /// person gives the same answer to the same question (the pick is keyed
    /// to them), and different people answer differently.
    fn spoken(&mut self, c: &Conversation, topic: &str, slots: &[&str], tags: &[String], values: &[(&'static str, String)]) -> String {
        let said = self.say_from(c.npc, c.with, topic, slots, tags, values);
        if said.text.is_empty() {
            "Hm. I've nothing to tell you about that.".into()
        } else {
            said.text
        }
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
                let job = self.life(c.npc).job;
                let mut tags = vec![format!("calling={}", p.stats.calling.name().to_lowercase())];
                let mut values: Vec<(&'static str, String)> = Vec::new();
                if !matches!(job, super::jobs::Job::None | super::jobs::Job::Drifter) {
                    tags.push("has_job".into());
                    if let Some(w) = self.workplace_of(c.npc) {
                        // Where it is from here, so it can be found.
                        let v = w.pos.sub(self.person_pos(c.npc));
                        let d = v.len();
                        let way = if d < 40.0 { String::new() } else { format!(", {} m {} of here", ((d / 10.0).round() * 10.0) as u32, super::quests::compass(v)) };
                        values.push(("place", format!(" at the {}{way}", w.kind.name().to_lowercase())));
                    }
                } else {
                    tags.push("no_job".into());
                }
                self.spoken(c, "background", &["origin", "work"], &tags, &values)
            }
            Topic::ThisTown => {
                let Some(t) = town else { return self.spoken(c, "town", &["about"], &["no_town".into()], &[]) };
                let mut counts = [0usize; 4];
                for &m in &t.residents {
                    counts[self.people[m as usize].race.index()] += 1;
                }
                let most = super::race::ALL_RACES.iter().zip(counts).max_by_key(|(_, n)| *n).map(|(r, _)| r.name()).unwrap_or("?");
                let mut tags = vec![if t.coastal { "coastal".to_string() } else { "inland".to_string() }];
                // The town as the speaker's people say it (its English name on hover).
                let said = super::names::town(self, t.id).map(|n| format!("\u{27e6}{}|{}\u{27e7}", n.in_tongue(p.race.into()), t.name)).unwrap_or_else(|| t.name.clone());
                let mut values: Vec<(&'static str, String)> = vec![
                    ("town", said),
                    ("founders", t.founders.name().to_string()),
                    ("count", t.residents.len().to_string()),
                    ("most", most.to_string()),
                ];
                if let Some(cm) = self.community_of(c.npc) {
                    tags.push("customs".into());
                    use super::culture::{Cooking, Rhythm};
                    values.push((
                        "cooking",
                        match cm.customs.cooking {
                            Cooking::Household => "each household cooks for itself",
                            Cooking::Hearth => "the hearth kitchen feeds everyone at work, and runners carry the pots out at midday",
                            Cooking::Deck => "we all eat together on the deck",
                        }
                        .to_string(),
                    ));
                    values.push((
                        "rhythm",
                        match cm.customs.rhythm {
                            Rhythm::Seasonal => "we work longer days in summer and shorter in winter",
                            Rhythm::Bells => "the bells call the shifts",
                            Rhythm::Tides => "our days follow the tides",
                            Rhythm::Irregular => "everyone keeps their own hours",
                        }
                        .to_string(),
                    ));
                }
                self.spoken(c, "town", &["about", "folk", "ways"], &tags, &values)
            }
            Topic::Advice => self.spoken(c, "advice", &["line"], &[], &[]),
            Topic::Rumours => {
                // Something they've heard, if they've heard anything.
                if let Some(news) = c.concerns.iter().find(|k| matches!(k.subject, super::talk::Subject::News | super::talk::Subject::Theft) && k.event.is_some()).copied() {
                    let said = self.assemble_talk(c.npc, c.with, Some(&news), false);
                    if !said.text.is_empty() {
                        self.note_said(c.npc, c.with, &said.pieces);
                        return said.text;
                    }
                }
                let mut tags = Vec::new();
                let mut values: Vec<(&'static str, String)> = Vec::new();
                if self.stats.ambushes > 0 {
                    tags.push("ambushes".to_string());
                    values.push(("count", self.stats.ambushes.to_string()));
                }
                if let Some((d, dir, _)) = self.nearest_camp(p.home) {
                    tags.push("camp_near".into());
                    values.push(("km", format!("{:.1}", d / 1000.0)));
                    values.push(("dir", dir.to_string()));
                }
                if let Some(h) = p.home {
                    for (origin, _) in self.bounties_known_in(h) {
                        if origin == h {
                            tags.push("bounty_here".into());
                        } else {
                            tags.push("bounty_road".into());
                            values.push(("place", self.settlements[origin as usize].name.clone()));
                        }
                    }
                }
                // One of the things they could say, picked like any other piece.
                let pick = if tags.is_empty() { None } else { Some(tags[r.below(tags.len())].clone()) };
                self.spoken(c, "rumours", &["line"], &pick.into_iter().collect::<Vec<_>>(), &values)
            }
            Topic::Bandits => match self.nearest_camp(p.home) {
                Some((d, dir, n)) if d < 6000.0 => self.spoken(c, "bandits", &["line"], &["camp_near".into()], &[("count", n.to_string()), ("km", format!("{:.1}", d / 1000.0)), ("dir", dir.to_string())]),
                _ => self.spoken(c, "bandits", &["line"], &["no_camp".into()], &[]),
            },
            Topic::Work => match self.quest_offer(c.npc) {
                Some((kind, coin, bonus)) => {
                    if let Some(t) = self.talk.as_mut() {
                        t.offered = true;
                    }
                    let npc = c.npc;
                    if let Some(o) = self.society.opps.iter_mut().find(|o| o.asker == npc && o.state == super::chances::OppState::Open) {
                        o.known = true;
                    }
                    let extra = bonus.map(|b| format!(" — and {} besides", items::item(b).name.to_lowercase())).unwrap_or_default();
                    // A favour: nothing to pay, and said so.
                    let paid = coin > 0;
                    match kind {
                        QuestKind::ClearCamp { at, .. } => {
                            let v = at.sub(town.map(|t| t.pos).unwrap_or(at));
                            format!(
                                "Those bandits {:.1} km {} of here have had my cousin twice. Break that camp and {}{extra}.",
                                v.len() / 1000.0,
                                compass(v),
                                if paid { format!("I'll pay {coin} coin") } else { "I'll owe you".to_string() }
                            )
                        }
                        QuestKind::Fetch { item, count } if paid => format!("I need {count} × {}. Bring them and there's {coin} coin in it.", items::item(item).name.to_lowercase()),
                        QuestKind::Fetch { item, count } => format!("I need {count} × {}. I can't pay, but I'd owe you.", items::item(item).name.to_lowercase()),
                        QuestKind::Deliver { to } => {
                            let q = &self.people[to as usize];
                            let place = q.home.map(|h| self.settlements[h as usize].name.clone()).unwrap_or_default();
                            let pay = if paid { format!("{coin} coin when it's done; they'll pay you.") } else { "I can't pay, but I'd owe you.".to_string() };
                            format!("Would you carry a letter to {} in {place}? Sealed, mind. {pay}", super::names::person_name(q.race, q.seed))
                        }
                        QuestKind::Job { opp } => {
                            let line = self.opportunity(opp).map(|o| self.opp_line(o)).unwrap_or_default();
                            format!("There's something. {}{}.", line[..1].to_uppercase(), &line[1..])
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
            Topic::Press(id, how) => self.press(id, c.with, c.npc, how),
            Topic::Say(_) => String::new(),
            Topic::Board => {
                let town = p.home.unwrap_or(0);
                let ids = self.hall_board(town);
                for &id in &ids {
                    if let Some(o) = self.society.opps.iter_mut().find(|o| o.id == id) {
                        o.known = true;
                    }
                }
                let lines: Vec<String> = ids.iter().filter_map(|&id| self.opportunity(id)).map(|o| format!("{} wants someone to {}", self.name_of(o.asker), self.opp_line(o))).collect();
                format!("Posted here: {}.", lines.join("; "))
            }
            Topic::TakeJob(id) => match self.take_opportunity(id, c.with) {
                Some(q) => {
                    let line = self.quest_line(&self.quests[q as usize].clone());
                    self.log.push_front((self.time, format!("New job: {line}")));
                    self.log.truncate(14);
                    "It's yours. Go and see them.".into()
                }
                None => "That's gone, I'm afraid.".into(),
            },
            Topic::PostWork(job, place) => {
                let town = p.home.unwrap_or(0);
                if self.take_post_work(c.with, town, job, place) {
                    let name = self.people[c.with as usize].name().unwrap_or("you").to_string();
                    let at = self.society.towns[town as usize].places.get(place as usize).map(|p| format!(" at the {}", p.kind.name().to_lowercase())).unwrap_or_default();
                    format!("Good, {name}. You'll work as {}{at} from tomorrow, 8 till 5, for {} coin a day, paid each dawn.", job.name().to_lowercase(), self.post_wage(job))
                } else {
                    "You've work already.".into()
                }
            }
            Topic::QuitWork => {
                self.quit_work(c.with);
                "So be it.".into()
            }
            Topic::Hire(wage) => match self.hire(c.npc) {
                Ok(bid) => {
                    let place = self.base(bid).map(|b| b.name.clone()).unwrap_or_default();
                    format!("{wage} a day, a bed and my meals? Then I'll set out for {place} today. Pay me each dawn.")
                }
                Err(e) => format!("No — {e}."),
            },
            Topic::AskJoin => self.why_not_join(c.npc).unwrap_or("Ask me properly.").to_string(),
            Topic::Treat(price) => match self.treat(c.npc) {
                Ok(who) if who.len() == 1 => format!("Hold still. ... There: cleaned and bound. {price} coin. Go easy on it for a day."),
                Ok(_) => format!("One at a time, then. ... There: all of them cleaned and bound. {price} coin."),
                Err(e) => format!("No — {e}."),
            },
            Topic::RentBeds(price) => match self.rent_beds(c.npc) {
                Ok(who) if who.len() == 1 => format!("A bed till morning, {price} coin. Go on up."),
                Ok(who) => format!("{} beds till morning, {price} coin. Go on up.", who.len()),
                Err(e) => format!("No — {e}."),
            },
            Topic::Join(fee) => match self.recruit(c.npc, c.with) {
                Ok(_) if fee > 0 => format!("{fee} coin to my household, and I'm yours. Where are we going?"),
                Ok(_) => "Nothing keeps me here. I'll get my things — lead on.".into(),
                Err(e) => format!("No — {e}."),
            },
            Topic::Report(i) => {
                let q = self.quests[i].clone();
                match (q.kind, q.stage) {
                    (_, Stage::Report) => {
                        self.reward(i, c.with);
                        if q.coin > 0 { format!("You did it? Then here — {} coin, as promised.", q.coin) } else { "You did it? I won't forget it.".into() }
                    }
                    (QuestKind::Fetch { item, count }, Stage::Active) => {
                        if self.squad_count(item) >= count {
                            self.take_from_squad(item, count);
                            self.reward(i, c.with);
                            if q.coin > 0 { format!("That's all of them. {} coin — fair's fair.", q.coin) } else { "That's all of them. I owe you.".into() }
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
                if self.quests[i].coin > 0 { format!("From {from}? At last. Here, for your trouble — {} coin.", self.quests[i].coin) } else { format!("From {from}? At last. Thank you.") }
            }
            Topic::PayBounty => {
                let Some(h) = p.home else { return String::new() };
                let known = self.bounties_known_in(h);
                let owed = known.iter().map(|b| b.1).sum::<f32>().ceil() as u16;
                let have = self.squad_count(items::id("coin"));
                if have >= owed {
                    self.take_from_squad(items::id("coin"), owed);
                    // It clears the matter; it earns nothing on top (running
                    // and paying later mustn't beat standing to be judged).
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
            Topic::Trade => {
                if self.for_sale(c.npc).is_empty() {
                    "The shelves are bare, I'm afraid. Try again after the next market.".into()
                } else {
                    ["Have a look. Fair prices — the press doesn't lie.", "What'll it be?", "All stamped and counted. Take your time."][r.below(3)].into()
                }
            }
            Topic::SellAll(it) => {
                let before = self.squad_count(items::id("coin"));
                let mut n = 0;
                while self.sell(c.npc, it) {
                    n += 1;
                }
                let got = self.squad_count(items::id("coin")).saturating_sub(before);
                match n {
                    0 => "I can't take any more of that.".into(),
                    _ if self.squad_count(it) > 0 => format!("{n} of them, {got} coin. That's all I can take for now."),
                    _ => format!("{n} of them — {got} coin. Pleasure."),
                }
            }
            Topic::SellWorn(it, m, p) => format!("{} would have to take the {} off first. I'd give about {p} coin for it.", self.people[m as usize].name().unwrap_or("Your friend"), items::item(it).name.to_lowercase()),
            Topic::SellRest => "Anything here, if the price suits you.".into(),
            Topic::Buy(it, price) => {
                if self.buy_at(c.npc, it, price) {
                    format!("{price} coin. There you are.")
                } else {
                    "You haven't the coin for that — or I've none left.".into()
                }
            }
            Topic::Sell(it, price) => {
                if self.sell_at(c.npc, it, price) {
                    format!("I'll give you {price} for the {}.", items::item(it).name.to_lowercase())
                } else {
                    "I can't take that just now.".into()
                }
            }
            Topic::ToNote => {
                if self.exchange(c.npc, true) {
                    "One note, struck and sealed. Don't lose it — paper burns and blows away.".into()
                } else {
                    "That's not enough coin for a note.".into()
                }
            }
            Topic::ToCoin => {
                if self.exchange(c.npc, false) {
                    "Coin for your note, less the house's share.".into()
                } else {
                    "You've no note to change.".into()
                }
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
            Topic::Mend(owner, slot, price) => match self.pay_to_mend(c.npc, owner, SLOTS[slot as usize]) {
                Ok(()) => format!("There — good as it was. {price} coin."),
                Err(e) => format!("I can't: {e}."),
            },
            Topic::CraftLesson(_) => {
                let style = self.life(c.npc).habits.teaching;
                match self.take_craft_lesson(c.npc, c.with) {
                    Ok(k) => match style {
                        super::culture::Teaching::Deep => format!("Sit. We'll go slowly — {} isn't learned in a hurry. A few hours, and you'll have the bones of it.", k.name()),
                        super::culture::Teaching::Drilled => format!("Watch, then do it. Again. Again. That's {} drilled into you — it'll stick.", k.name()),
                    },
                    Err(e) => format!("Not now: {e}."),
                }
            }
            Topic::Orders => "Stone takes its time. Pay half now, and come back when it's grown — I'll need to be here to tend it.".into(),
            Topic::Order(ri, _) => match self.place_order(c.npc, ri as usize) {
                Ok(k) => {
                    let o = self.orders[k];
                    format!("Done. It'll be ready in {:.0} days, if I'm here to tend it. {} coin now, {} when you collect.", (o.ready_at - self.time) / DAY, o.deposit, o.rest)
                }
                Err(e) => format!("I can't take that on: {e}."),
            },
            Topic::Collect(k) => {
                if !self.order_ready(k as usize) {
                    let o = self.orders[k as usize];
                    return format!("Not yet. Give it {:.1} more days.", ((o.ready_at - self.time) / DAY).max(0.1));
                }
                match self.collect_order(k as usize) {
                    Ok(id) => format!("Here it is — your {}. Mind how you treat it.", items::item(id).name.to_lowercase()),
                    Err(e) => format!("Not just now: {e}."),
                }
            }
            Topic::BuyOut(m, _) => match self.buy_out(m) {
                Ok(()) => "Paid in full. They're free to go.".into(),
                Err(e) => format!("Not so fast: {e}."),
            },
            Topic::TakePost(post) => {
                let Some(town) = p.home else { return String::new() };
                match self.take_post(c.with, town, post) {
                    Ok(()) => format!("Then it's settled: you sit as {} from today.", post.name()),
                    Err(e) => format!("That can't be: {e}."),
                }
            }
            Topic::Goodbye => String::new(),
        }
    }

    /// Distance, direction and size of the bandit camp nearest a town.
    fn nearest_camp(&self, home: Option<u16>) -> Option<(f32, &'static str, usize)> {
        let from = home.map(|h| self.settlements[h as usize].pos).unwrap_or(self.squad.pos);
        let c = self.camps.iter().filter(|c| !self.is_warden(c.group)).min_by(|a, b| a.pos.dist(from).total_cmp(&b.pos.dist(from)))?;
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
        Topic::Trade => 11,
        Topic::Buy(it, _) => 30_000 + it as u64,
        Topic::Sell(it, _) => 40_000 + it as u64,
        Topic::ToNote => 12,
        Topic::ToCoin => 13,
        Topic::Mend(m, k, _) => 50_000 + m as u64 * 16 + k as u64,
        Topic::CraftLesson(_) => 14,
        Topic::Orders => 15,
        Topic::Order(ri, _) => 60_000 + ri as u64,
        Topic::Collect(k) => 70_000 + k as u64,
        Topic::BuyOut(m, _) => 80_000 + m as u64,
        Topic::TakePost(p) => 16 + p as u64,
        Topic::Press(id, how) => 90_000 + id as u64 * 4 + how as u64,
        Topic::Board => 30,
        Topic::TakeJob(id) => 1_000_000 + id as u64,
        Topic::PostWork(j, _) => 31 + j as u64,
        Topic::QuitWork => 70,
        Topic::Hire(_) => 71,
        Topic::Join(_) => 72,
        Topic::AskJoin => 73,
        Topic::Treat(_) => 74,
        Topic::RentBeds(_) => 75,
        Topic::SellAll(it) => 2_000_000 + it as u64,
        Topic::SellWorn(it, m, _) => 3_000_000 + ((it as u64) << 32) + m as u64,
        Topic::SellRest => 73,
        Topic::Say(o) => 80 + o as u64,
    }
}


impl World {
    /// The kinds of thing the squad carries that this merchant would buy:
    /// (item, the price of the first of it, what the lot would fetch), with
    /// what fetches most first. By the lot, not the piece, so sixty timber
    /// (the woodcutter's whole morning) doesn't sink under one torch.
    pub fn sell_kinds(&self, npc: PersonId) -> Vec<(items::ItemId, u16, u16)> {
        let mut kinds: Vec<(items::ItemId, u16, u16)> = Vec::new();
        for (it, p) in self.sellable(npc) {
            if !kinds.iter().any(|k| k.0 == it) {
                kinds.push((it, p, self.sell_all_quote(npc, it).1));
            }
        }
        kinds.sort_by(|a, b| b.2.cmp(&a.2).then(b.1.cmp(&a.1)).then(a.0.cmp(&b.0)));
        kinds
    }
}

/// How many kinds of the sell list have been shown by the end of this page.
fn sell_seen(page: u8) -> usize {
    SELL_SHOWN + page as usize * SELL_PAGE
}
