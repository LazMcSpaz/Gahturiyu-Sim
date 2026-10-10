//! Recruiting (the playable-MVP list): asking a townsperson to join the
//! squad. Kenshi-style, the footloose come for nothing and the restless who
//! have a living want a signing fee; either way they leave their town for
//! good, as a hired hand does, and become a full squad member: their own
//! condition, skills that grow with use, orders like everyone else.
//!
//! Who'll come is a plain function of who they are and how they live now
//! (`join_terms`), the same whether or not they've been met.

use super::items;
use super::person::PersonId;
use super::world::World;

/// The most the travelling squad can be.
pub const MAX_SQUAD: usize = 10;
/// Restlessness (boldness + wanderlust) below this never takes to the road.
pub const RESTLESS_FLOOR: f32 = 0.3;
/// Of the out-of-work and the needy, the share per point of restlessness
/// above the floor who'd come along for nothing.
pub const FREE_SHARE: f32 = 0.5;
/// Of those with a living, the share per point who'd leave it, for a fee.
pub const FEE_SHARE: f32 = 0.05;
/// A signing fee is this many hours of their work's pay.
pub const FEE_HOURS: f32 = 40.0;

/// A squad member's standing in a town below this (caught at something):
/// nobody there will go off with them.
pub const WARY_STANDING: f32 = -2.0;

/// A recruit comes with little of their own (Laz, B3): nothing worth more
/// than this, each, beyond coin. The rest stays with their household.
pub const RECRUIT_KIT_MAX: f32 = 20.0;

/// What a recruit takes on the road: their kit, less anything worth more than
/// `RECRUIT_KIT_MAX` apiece.
pub fn travelling_kit(gear: &super::inventory::Gear) -> super::inventory::Gear {
    use super::items::{item, SLOTS};
    let mut g = gear.clone();
    for s in SLOTS {
        if g.in_slot(s).is_some_and(|it| item(it).value > RECRUIT_KIT_MAX) {
            g.discard(s);
        }
    }
    g.bag.retain(|e| item(e.0).key == "coin" || item(e.0).value <= RECRUIT_KIT_MAX);
    g
}

/// Flatbread a recruit brings with them: about two days' eating.
pub const RECRUIT_BREAD: u16 = 3;

impl World {
    /// Would this person join the squad if asked, and for what fee (0: for
    /// nothing)? `None`: they won't, or can't.
    pub fn join_terms(&self, npc: PersonId) -> Option<u16> {
        let p = &self.people[npc as usize];
        if p.dead || p.in_squad || p.bandit || self.group_of[npc as usize].is_some() || self.resident_of(npc).is_some() {
            return None;
        }
        if self.squad.members.len() >= MAX_SQUAD {
            return None;
        }
        let life = self.society.lives.get(npc as usize)?;
        life.community?;
        p.home?;
        let t = self.time;
        if self.is_bonded(npc, t) || self.holds_office(npc) || self.society.rings.iter().any(|r| r.leader == Some(npc) || r.members.contains(&npc)) {
            return None;
        }
        let m = self.mind(npc);
        use super::lives::Work;
        if matches!(m.work, Work::Bonded | Work::Injured) {
            return None;
        }
        // The watch, the shrine and anyone waiting on the squad to do a job
        // for them stay put; and nobody goes off with people the town has
        // caught at wrongdoing, or who have wronged them.
        if self.wary_of_squad(npc) || matches!(life.job, super::jobs::Job::Guard | super::jobs::Job::Priest) || self.society.opps.iter().any(|o| o.asker == npc && o.state == super::chances::OppState::Taken && !o.done) {
            return None;
        }
        // A fixed roll per person (like a trait), weighed by how restless they
        // are, so every people has some who'd go, the restless more.
        let restless = (p.traits.boldness + p.traits.wanderlust - RESTLESS_FLOOR).clamp(0.0, 1.2);
        let roll = super::rng::Rng::from_keys(&[p.seed, 0x4A4F_494E]).f32();
        let footloose = m.work == Work::Jobless || m.needs()[0] > 0.5 || m.needs()[1] > 0.5;
        if footloose {
            return (roll < FREE_SHARE * restless).then_some(0);
        }
        (roll < FEE_SHARE * restless).then(|| (life.job.pay().max(0.7) * FEE_HOURS).ceil() as u16)
    }

    /// Has the squad a name for trouble in this person's town (a bounty, or
    /// caught at something), or done them a wrong they remember?
    pub fn wary_of_squad(&self, npc: PersonId) -> bool {
        let Some(town) = self.people[npc as usize].home else { return false };
        let day = World::day_of(self.time) as i32;
        self.bounty_known_in(town) > 0.0 || self.squad.members.iter().any(|&m| self.standing(m, town) < WARY_STANDING || self.memory_of(npc, super::memory::Who::Person(m), day) < -0.3)
    }

    /// Why someone won't come, in a line of their own (`None` if they would).
    pub fn why_not_join(&self, npc: PersonId) -> Option<&'static str> {
        if self.join_terms(npc).is_some() {
            return None;
        }
        let p = &self.people[npc as usize];
        if self.squad.members.len() >= MAX_SQUAD {
            return Some("There's no room for another of you, by the look of it.");
        }
        let t = self.time;
        if self.is_bonded(npc, t) {
            return Some("I'm bound here till my debt's worked off. I can't go anywhere.");
        }
        if self.holds_office(npc) || matches!(self.society.lives.get(npc as usize).map(|l| l.job), Some(super::jobs::Job::Guard | super::jobs::Job::Priest)) {
            return Some("The town needs me where I am. No.");
        }
        if self.wary_of_squad(npc) {
            return Some("Go with you lot? After what's been going on? No.");
        }
        if self.society.opps.iter().any(|o| o.asker == npc && o.state == super::chances::OppState::Taken && !o.done) {
            return Some("You've a job to do for me first.");
        }
        let restless = p.traits.boldness + p.traits.wanderlust;
        let jobless = matches!(self.society.lives.get(npc as usize).map(|l| l.job), Some(super::jobs::Job::None | super::jobs::Job::Drifter));
        Some(match (restless < RESTLESS_FLOOR + 0.2, jobless) {
            (true, _) => "Me? On the road? No — I'm not made for it.",
            (false, true) => "Not this time. I've something to see to here first.",
            (false, false) => "I've a living here, and I'm not leaving it.",
        })
    }

    /// Who they are as a hire, before they're hired: their three best
    /// skills and what they'd bring.
    pub fn recruit_card(&self, npc: PersonId) -> String {
        let p = &self.people[npc as usize];
        let mut best: Vec<(super::stats::Skill, f32)> = super::stats::SKILLS.iter().map(|&k| (k, p.stats.skill(k))).collect();
        best.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        // In words, as they'd say it of themselves: no numbers on a stranger.
        let skills: Vec<String> = best.iter().take(3).map(|(k, _)| k.name().to_lowercase()).collect();
        // What they'd bring on the road (their kit, or as they'd turn up
        // with it if never met), less the good things they'd leave at home.
        let kit = travelling_kit(&match &p.detail {
            Some(d) => d.gear.clone(),
            None => super::inventory::starting_kit(p.race, &p.kit_stats, p.budget, p.seed),
        });
        use super::items::{item, Slot, SLOTS};
        let name = |s: Slot| kit.in_slot(s).map(|it| item(it).name.to_lowercase());
        let arms: Vec<String> = [Slot::MainHand, Slot::OffHand].into_iter().filter_map(name).collect();
        let worn: Vec<String> = SLOTS.into_iter().filter(|s| !matches!(s, Slot::MainHand | Slot::OffHand)).filter_map(name).collect();
        let arms = if arms.is_empty() { "no weapon".to_string() } else { arms.join(" and ") };
        let worn = if worn.is_empty() { "nothing much".to_string() } else { worn.join(", ") };
        format!("good at {} · {arms} · wears {worn}", skills.join(", "))
    }

    /// Everyone in a town who'd come if asked: (who, fee), cheapest first.
    pub fn willing_in(&self, town: super::settlement::SettlementId) -> Vec<(PersonId, u16)> {
        let mut v: Vec<(PersonId, u16)> = self.settlements[town as usize].residents.iter().filter_map(|&p| self.join_terms(p).map(|f| (p, f))).collect();
        v.sort_by_key(|&(p, f)| (f, p));
        v
    }

    /// Take someone into the squad: `by` (a squad member beside them) pays
    /// any fee, which goes home to their household; they leave their post,
    /// home and household and stand beside `by`.
    pub fn recruit(&mut self, npc: PersonId, by: PersonId) -> Result<u16, &'static str> {
        let fee = self.join_terms(npc).ok_or("they won't come")?;
        let Some(k) = self.squad.index(by) else { return Err("not in the squad") };
        let coin = items::id("coin");
        if self.squad_count(coin) < fee {
            return Err("you can't pay what they ask");
        }
        let t = self.time;
        let town = self.people[npc as usize].home.unwrap_or(0);
        if self.people[npc as usize].ensure_detail() {
            self.stats.detailed += 1;
        }
        if fee > 0 {
            self.take_from_squad(coin, fee);
        }
        let life = &mut self.society.lives[npc as usize];
        let (trade, household) = (life.job, life.household);
        if trade.is_post() {
            self.society.minds[npc as usize].lost = Some((trade, super::lives::Loss::Away));
        }
        life.job = super::jobs::Job::None;
        life.place = None;
        life.community = None;
        life.household = None;
        if let Some(h) = household {
            let hh = &mut self.society.households[h as usize];
            hh.members.retain(|&m| m != npc);
            hh.purse.coin += fee as f32;
        }
        self.settlements[town as usize].residents.retain(|&m| m != npc);
        // From here they're one of the squad: their own condition timeline.
        let p = &mut self.people[npc as usize];
        p.in_squad = true;
        p.cond = Some(super::condition::Condition::new(t));
        // Their home's chests may change hands.
        self.refresh_container_owners();
        let (here, inside) = (self.squad.at[k], self.squad.inside[k]);
        let off = super::squad::formation(self.squad.members.len()).scale(0.4);
        self.squad.add(npc, here.add(off));
        if let Some(v) = self.squad.inside.last_mut() {
            *v = inside;
        }
        // They come with little: the good things stay at home. And a couple
        // of days' bread of their own.
        if let Some(d) = self.people[npc as usize].detail.as_mut() {
            d.gear = travelling_kit(&d.gear);
            d.gear.add(items::id("flatbread"), RECRUIT_BREAD);
        }
        self.people[npc as usize].recompute_might();
        self.settle_condition(npc, t);
        let name = self.people[npc as usize].name().unwrap_or("someone").to_string();
        let tname = self.settlements[town as usize].name.clone();
        self.log.push_front((t, format!("{name} of {tname} joins the squad.")));
        self.log.truncate(14);
        Ok(fee)
    }
}

impl World {
    /// Send a squad member away: they stay in the town they're standing in,
    /// looking for a living there. Says why not, if not.
    pub fn send_away(&mut self, who: PersonId) -> Result<String, String> {
        let name = self.people[who as usize].name().unwrap_or("someone").to_string();
        let Some(k) = self.squad.index(who) else { return Err(format!("{name} isn't one of you.")) };
        if self.squad.members.len() <= 1 {
            return Err("There'd be nobody left.".into());
        }
        if self.is_down(who) || self.carried_by(who).is_some() || self.carrying(who).is_some() || self.fighting.contains_key(&who) || !self.free_to_order(who) {
            return Err(format!("Not now: {name} can't go anywhere."));
        }
        let at = self.member_pos(k);
        let Some(town) = self.settlements.iter().filter(|s| s.pos.dist(at) < s.radius() + 50.0).min_by(|a, b| a.pos.dist(at).total_cmp(&b.pos.dist(at))).map(|s| s.id) else {
            return Err(format!("Not out here: {name} would part ways in a town."));
        };
        self.quit_work(who);
        self.looting.retain(|l| l.who != who);
        self.picking.retain(|p| p.who != who);
        self.pickups.retain(|p| p.who != who);
        self.giving.retain(|g| g.from != who && g.to != who);
        self.squad.retain(|m| m != who);
        let ci = self.society.towns[town as usize].shore;
        let p = &mut self.people[who as usize];
        p.in_squad = false;
        p.cond = None;
        p.home = Some(town);
        let life = &mut self.society.lives[who as usize];
        life.job = super::jobs::Job::None;
        life.place = None;
        life.household = None;
        life.community = Some(ci);
        self.settlements[town as usize].residents.push(who);
        self.refresh_container_owners();
        let t = self.time;
        let line = format!("{name} leaves the squad and stays in {}.", self.settlements[town as usize].name);
        self.log.push_front((t, line.clone()));
        self.log.truncate(14);
        Ok(line)
    }
}
