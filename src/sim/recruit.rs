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
        if self.holds_office(npc) {
            return Some("The town needs me where I am. No.");
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
        let skills: Vec<String> = best.iter().take(3).map(|(k, v)| format!("{} {:.0}", k.name(), v)).collect();
        // Their kit as it is (or as they'd turn up with it, if never met).
        let kit = match &p.detail {
            Some(d) => d.gear.clone(),
            None => super::inventory::starting_kit(p.race, &p.kit_stats, p.budget, p.seed),
        };
        use super::items::{item, Slot, SLOTS};
        let name = |s: Slot| kit.in_slot(s).map(|it| item(it).name.to_lowercase());
        let arms: Vec<String> = [Slot::MainHand, Slot::OffHand].into_iter().filter_map(name).collect();
        let worn: Vec<String> = SLOTS.into_iter().filter(|s| !matches!(s, Slot::MainHand | Slot::OffHand)).filter_map(name).collect();
        let arms = if arms.is_empty() { "no weapon".to_string() } else { arms.join(" and ") };
        let worn = if worn.is_empty() { "nothing much".to_string() } else { worn.join(", ") };
        format!("best at {} · {arms} · wears {worn}", skills.join(", "))
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
        // They come with a couple of days' bread of their own.
        if let Some(d) = self.people[npc as usize].detail.as_mut() {
            d.gear.add(items::id("flatbread"), RECRUIT_BREAD);
        }
        self.settle_condition(npc, t);
        let name = self.people[npc as usize].name().unwrap_or("someone").to_string();
        let tname = self.settlements[town as usize].name.clone();
        self.log.push_front((t, format!("{name} of {tname} joins the squad.")));
        self.log.truncate(14);
        Ok(fee)
    }
}
