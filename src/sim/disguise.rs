//! Disguise (Laz, 12 Oct: a hybrid of New Vegas and Kenshi; the civic
//! groups in `docs/factions.md`). Step F3 of the faction plan.
//!
//! - **The groups** (`CIVIC`) are data: what garment makes the look (its
//!   `items::Role`), who knows the real ones (by job), how close they are
//!   (`Close`: a tight group notices a stranger in its own clothes), whether
//!   impersonating them is a crime (`Crime`), the faction they answer to,
//!   and what the look opens. Groups whose garments aren't made yet are
//!   listed with none: nobody can wear them until they are.
//! - **Wearing a role garment** puts a squad member in that group while it's
//!   worn (`disguise_of`). Townsfolk treat them as the role: a witness is
//!   slower to report one of the watch, and the watch doesn't turn one of
//!   its own out of a house at night.
//! - **The fit check** (Kenshi) is rolled only when someone pays attention:
//!   one of the real ones who is up, near and sees them plainly (the same
//!   sight as the watch knowing a face). One keyed roll per looker, wearer
//!   and hour (`seen_through`), against how close the group is, whether the
//!   looker is one of them, whether the rest of the kit fits (a priest with a
//!   sword), whether a tabard is this town's, the wearer's Sneak, and how
//!   often they've been doubted before.
//! - **What follows** (New Vegas) reads the squad's standing with the
//!   group's faction: honoured or friendly, it's let pass; hostile, they
//!   come after them as the watch does after a wanted face; otherwise
//!   they're questioned first, and a keyed roll on the wearer's wits
//!   (`talks_past`) decides. Talked past, they're doubted more next time;
//!   found out, impersonating the watch is always a crime, a priest's
//!   vestments a crime where the community's custom forbids it (`Garb`), and
//!   always a loss of standing with the Holy House.

use serde::{Deserialize, Serialize};

use super::culture::Garb;
use super::factions::{self, FactionId, FRIENDLY, HOSTILE};
use super::geo::V2;
use super::items::{self, item, Kind, Role, Slot};
use super::jobs::Job;
use super::law::Wrong;
use super::person::PersonId;
use super::pursuit::{Pursuit, KNOW_FACE};
use super::rng::Rng;
use super::settlement::SettlementId;
use super::stats::{Attr, Skill};
use super::world::{World, HOUR};

/// How well a group's members know each other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Close {
    Loose,
    Medium,
    Tight,
    VeryTight,
}

impl Close {
    /// The chance one of them sees through a stranger in their clothes,
    /// before anything else counts.
    pub fn base(self) -> f32 {
        match self {
            Close::Loose => 0.10,
            Close::Medium => 0.25,
            Close::Tight => 0.45,
            Close::VeryTight => 0.65,
        }
    }
}

/// Whether passing as one of them is a crime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crime {
    Always,
    /// Where the community's custom says so (`culture::Garb`).
    ByCustom,
    Never,
}

/// A civic group or elite uniform (`docs/factions.md`).
#[derive(Clone, Copy, Debug)]
pub struct Civic {
    pub key: &'static str,
    pub name: &'static str,
    /// The garment's role that makes the look (none: no garment made yet).
    pub role: Option<Role>,
    /// Who the real ones are, by job (they're the ones who look hard).
    pub knows: &'static [Job],
    pub close: Close,
    pub crime: Crime,
    /// Whose it is (none: the town's holder).
    pub faction: Option<&'static str>,
    /// An elite uniform: everyone inside knows each other, and being caught
    /// in one is worse.
    pub elite: bool,
    /// What the look opens, in words.
    pub opens: &'static str,
}

const fn c(key: &'static str, name: &'static str, role: Option<Role>, knows: &'static [Job], close: Close, crime: Crime, faction: Option<&'static str>, opens: &'static str) -> Civic {
    Civic { key, name, role, knows, close, crime, faction, elite: false, opens }
}

const fn elite(key: &'static str, name: &'static str, knows: &'static [Job], close: Close, faction: Option<&'static str>, opens: &'static str) -> Civic {
    Civic { key, name, role: None, knows, close, crime: Crime::Always, faction, elite: true, opens }
}

use Close::{Loose, Medium, Tight, VeryTight};

pub static CIVIC: &[Civic] = &[
    // Uniformed groups (passes).
    c("watch", "The town watch", Some(Role::Guard), &[Job::Guard], Tight, Crime::Always, None, "gates after dark, the barracks, the jail, questioning people"),
    c("shrine_keepers", "Shrine keepers", Some(Role::Priest), &[Job::Priest], Tight, Crime::ByCustom, Some("holy_house"), "shrines, the back rooms of rites, funerals"),
    c("clerks", "Exchange-house clerks", None, &[Job::Exchanger], Tight, Crime::Never, Some("writers"), "vaults, ledgers, the coin presses"),
    c("couriers", "Couriers", None, &[Job::Runner], Loose, Crime::Never, None, "walking anywhere with a letter"),
    c("harbour_crews", "Harbour crews", None, &[Job::Fisher, Job::Boatwright], Medium, Crime::Never, Some("far_sails"), "docks, holds, cargo"),
    // Work clothes (blending in).
    c("craft_trades", "The craft trades", None, &[Job::Smith, Job::Armourer, Job::Weaver, Job::StoneTender], Medium, Crime::Never, None, "workyards and back doors"),
    c("labourers", "Labourers and porters", Some(Role::Labourer), &[Job::Labourer], Loose, Crime::Never, None, "warehouses, kitchens, building sites"),
    c("kitchen", "Kitchen and mess-hall workers", None, &[Job::Cook], Medium, Crime::Never, None, "kitchens, the meal runners' routes, mess halls"),
    c("healers", "Healers", None, &[Job::Healer], Medium, Crime::Never, None, "sickrooms; the wounded and prisoners"),
    c("merchants", "Merchants", Some(Role::Merchant), &[Job::Merchant], Loose, Crime::Never, None, "market halls and caravans"),
    c("beggars", "Beggars and street folk", None, &[], Loose, Crime::Never, None, "unseen by the rich: good for watching and listening"),
    // Elite uniforms (Laz: in from the start; their garments come later).
    elite("palace_guard", "The palace guard", &[Job::Guard], VeryTight, Some("throne"), "the court and the emperor's halls"),
    elite("retainers", "A lord's retainers", &[Job::Guard], Tight, Some("great_house"), "that lord's hall and lands"),
    elite("armed_order", "The armed order", &[Job::Guard, Job::Priest], Tight, Some("holy_house"), "fortified shrines and inquisitor patrols"),
    elite("house_guard", "House guard", &[Job::Guard], VeryTight, Some("first_peoples"), "the old houses and the Seated Houses' halls"),
    elite("temple_guard", "Temple guard", &[Job::Guard, Job::Priest], VeryTight, Some("last_wave"), "enclave summits and rites"),
];

/// A looker who isn't one of the group sees through it this much less often.
pub const OUTSIDER_EYE: f32 = 0.4;
/// An elite uniform is seen through this much more often.
pub const ELITE_EYE: f32 = 1.3;
/// Each thing in the kit that doesn't fit the role (a priest with a sword).
pub const MISFIT: f32 = 0.2;
/// A watch tabard that isn't this town's.
pub const WRONG_COLOURS: f32 = 0.25;
/// Each time they've been doubted before.
pub const DOUBTED: f32 = 0.15;
/// What Sneak takes off (per point).
pub const SNEAK_EASE: f32 = 1.0 / 250.0;
/// Found out impersonating: the fine, and what impersonating a priest costs
/// with the Holy House (twice that in an elite uniform).
pub const IMPERSONATION_FINE: f32 = 40.0;
pub const PRIEST_DEED: f32 = 6.0;
/// A witness is this much less likely to report one of the watch.
pub const WATCH_UNREPORTED: f32 = 0.5;
const SEEN_TAG: u64 = 0x4449_5347;
const TALK_TAG: u64 = 0x5155_4553;

/// One group by key (panics on a key that isn't there: data).
pub fn civic(key: &str) -> usize {
    CIVIC.iter().position(|c| c.key == key).unwrap_or_else(|| panic!("no civic group `{key}`"))
}

/// Someone doubted in a disguise, and how often (the next look is harder
/// to pass).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Doubt {
    pub who: PersonId,
    pub group: u8,
    pub times: u8,
}

impl World {
    /// The group a squad member passes as, by what they wear (the first
    /// group whose garment's role they have on).
    pub fn disguise_of(&self, pid: PersonId) -> Option<usize> {
        self.squad.index(pid)?;
        let gear = self.people[pid as usize].kit();
        let roles: Vec<Role> = gear.equipped().filter_map(|id| items::look(id).role).collect();
        CIVIC.iter().position(|c| c.role.is_some_and(|r| roles.contains(&r)))
    }

    /// Whose a group is in a town: its own faction, else the town's holder.
    pub fn group_faction(&self, g: usize, town: SettlementId) -> Option<FactionId> {
        CIVIC[g].faction.map(factions::id).or_else(|| self.holder_of(town))
    }

    /// How many things in `pid`'s kit don't fit group `g`'s role.
    pub fn misfits(&self, pid: PersonId, g: usize) -> u32 {
        let gear = self.people[pid as usize].kit();
        let armed = gear.in_slot(Slot::MainHand).is_some_and(|id| matches!(item(id).kind, Kind::Weapon(_)));
        let armoured = [Slot::BodyOver, Slot::Head, Slot::Hands, Slot::LegsOver].iter().any(|&s| gear.in_slot(s).is_some_and(|id| matches!(item(id).kind, Kind::Armor(_))));
        match CIVIC[g].role {
            // One of the watch goes armed.
            Some(Role::Guard) => (!armed) as u32,
            Some(_) => armed as u32 + armoured as u32,
            None => 0,
        }
    }

    /// Is the watch tabard `pid` wears this town's (made here, or under the
    /// same holder)?
    fn tabard_fits(&self, pid: PersonId, town: SettlementId) -> bool {
        let gear = self.people[pid as usize].kit();
        let Some(slot) = [Slot::BodyOver, Slot::Cloak, Slot::BodyUnder].into_iter().find(|&s| gear.in_slot(s).is_some_and(|id| items::look(id).role == Some(Role::Guard))) else { return false };
        let Some(made) = gear.piece(slot).and_then(|p| p.mark).and_then(|m| m.town) else { return false };
        made == town || self.holder_of(made).is_some_and(|h| Some(h) == self.holder_of(town))
    }

    /// The chance `looker` sees through `wearer` passing as group `g` in
    /// `town`.
    pub fn seen_through_chance(&self, looker: PersonId, wearer: PersonId, g: usize, town: SettlementId) -> f32 {
        let c = &CIVIC[g];
        let mut p = c.close.base();
        if !c.knows.contains(&self.life(looker).job) {
            p *= OUTSIDER_EYE;
        }
        if c.elite {
            p *= ELITE_EYE;
        }
        p += MISFIT * self.misfits(wearer, g) as f32;
        if c.role == Some(Role::Guard) && !self.tabard_fits(wearer, town) {
            p += WRONG_COLOURS;
        }
        p -= self.people[wearer as usize].stats.skill(Skill::Sneak) * SNEAK_EASE;
        p += DOUBTED * self.doubted_times(wearer, g) as f32;
        p.clamp(0.02, 0.95)
    }

    /// How often `who` has been doubted passing as group `g`.
    pub fn doubted_times(&self, who: PersonId, g: usize) -> u8 {
        self.doubts.iter().find(|d| d.who == who && d.group as usize == g).map(|d| d.times).unwrap_or(0)
    }

    fn doubt(&mut self, who: PersonId, g: usize, n: u8) {
        match self.doubts.iter_mut().find(|d| d.who == who && d.group as usize == g) {
            Some(d) => d.times = d.times.saturating_add(n),
            None => self.doubts.push(Doubt { who, group: g as u8, times: n }),
        }
    }

    /// Does `looker` see through it this hour? One keyed roll per looker,
    /// wearer and hour.
    pub fn seen_through(&self, looker: PersonId, wearer: PersonId, g: usize, town: SettlementId, t: f64) -> bool {
        let hour = (t / HOUR).floor() as u64;
        Rng::from_keys(&[self.seed, looker as u64, wearer as u64, g as u64, hour, SEEN_TAG]).f32() < self.seen_through_chance(looker, wearer, g, town)
    }

    /// Questioned, does the wearer talk their way past? (Their wits and
    /// their Sneak, against how close the group is.)
    pub fn talks_past(&self, looker: PersonId, wearer: PersonId, g: usize, t: f64) -> bool {
        let p = &self.people[wearer as usize].stats;
        let chance = (0.3 + (p.attr(Attr::Intellect) + p.skill(Skill::Sneak)) / 400.0 - CIVIC[g].close.base() * 0.4).clamp(0.05, 0.9);
        let hour = (t / HOUR).floor() as u64;
        Rng::from_keys(&[self.seed, looker as u64, wearer as u64, g as u64, hour, TALK_TAG]).f32() < chance
    }

    /// Is passing as group `g` a crime where `looker` lives?
    pub fn impersonation_is_crime(&self, g: usize, looker: PersonId) -> bool {
        match CIVIC[g].crime {
            Crime::Always => true,
            Crime::Never => false,
            Crime::ByCustom => self.community_of(looker).is_some_and(|c| c.customs.garb == Garb::Forbidden),
        }
    }

    /// Each step: a squad member in a disguise, in a town, is looked at by
    /// the nearest of the real ones who is up and sees them plainly (once an
    /// hour each). Seen through, what follows reads the squad's standing
    /// with the group's faction: let pass, questioned, or come after.
    pub(super) fn watch_for_disguises(&mut self) {
        let t = self.time;
        let hour = (t / HOUR).floor() as i64;
        self.eyed.retain(|e| e.2 == hour);
        let mut found: Option<(PersonId, PersonId, usize, SettlementId, V2)> = None;
        for m in self.squad.members.clone() {
            let Some(g) = self.disguise_of(m) else { continue };
            if self.is_down(m) || self.fighting.contains_key(&m) || self.pursuits.iter().any(|p| p.culprit == m) {
                continue;
            }
            let k = self.squad.index(m).unwrap();
            let at = self.member_pos(k);
            let Some(town) = self.settlements.iter().find(|s| at.dist(s.pos) <= s.reach).map(|s| s.id) else { continue };
            let near = KNOW_FACE * self.visibility_of(m);
            let room = self.building_at(at).map(|d| d.id);
            let knows = CIVIC[g].knows;
            let s = &self.settlements[town as usize];
            let looker = s
                .residents
                .iter()
                .copied()
                .filter(|&p| !self.people[p as usize].dead && knows.contains(&self.life(p).job))
                .filter(|&p| !self.fighting.contains_key(&p) && !self.is_down(p) && !self.pursuits.iter().any(|x| x.guard == p))
                .filter(|&p| self.doing_now(p).is_some_and(|d| d.0 != super::routine::Doing::Asleep))
                .map(|p| (p, self.person_pos(p)))
                .filter(|&(_, lp)| lp.dist(at) <= near && self.building_at(lp).map(|d| d.id) == room)
                .min_by(|a, b| a.1.dist(at).total_cmp(&b.1.dist(at)).then(a.0.cmp(&b.0)));
            let Some((l, _)) = looker else { continue };
            if self.eyed.iter().any(|e| e.0 == l && e.1 == m) {
                continue;
            }
            self.eyed.push((l, m, hour));
            if self.seen_through(l, m, g, town, t) {
                found = Some((l, m, g, town, at));
                break;
            }
        }
        let Some((l, m, g, town, at)) = found else { return };
        self.found_out(l, m, g, town, at, t);
    }

    /// `looker` has seen through `wearer`: what follows.
    fn found_out(&mut self, looker: PersonId, wearer: PersonId, g: usize, town: SettlementId, at: V2, t: f64) {
        let (ln, wn) = (self.name_of(looker), self.name_of(wearer));
        let group = CIVIC[g].name.to_lowercase();
        let regard = self.group_faction(g, town).map(|f| self.standing_with(f)).unwrap_or(0.0);
        // Passing as a priest always costs with the priests, once found out.
        if CIVIC[g].faction == Some("holy_house") || CIVIC[g].role == Some(Role::Priest) {
            let d = if CIVIC[g].elite { 2.0 * PRIEST_DEED } else { PRIEST_DEED };
            self.deed(factions::id("holy_house"), -d);
        }
        if regard >= FRIENDLY {
            self.doubt(wearer, g, 1);
            self.say(t, format!("{ln} gives {wn} a long look, and lets it go."));
            return;
        }
        let crime = self.impersonation_is_crime(g, looker);
        let fine = if CIVIC[g].elite { 2.0 * IMPERSONATION_FINE } else { IMPERSONATION_FINE };
        if regard <= HOSTILE {
            // They don't ask: they come for them.
            self.doubt(wearer, g, 2);
            let line = format!("{ln} sees through {wn}, passing as {group}, and comes for them.");
            if self.life(looker).job == Job::Guard {
                self.pursuits.push(Pursuit { guard: looker, culprit: wearer, town, wrong: Wrong::Impersonation, fine, witness: None, start: t, pos: None, last_seen: at, seen_t: t, owed: false });
            }
            self.say(t, line.clone());
            self.alerts.push(line);
            return;
        }
        // Questioned first.
        if self.talks_past(looker, wearer, g, t) {
            self.doubt(wearer, g, 1);
            self.say(t, format!("{ln} stops {wn} with questions, and lets them go, for now."));
            return;
        }
        self.doubt(wearer, g, 2);
        let line = format!("{ln} questions {wn} and finds them out: no one of {group}.");
        if crime {
            self.wrong_seen(wearer, town, Wrong::Impersonation, fine, line, Some(looker), None, at);
        } else {
            self.say(t, format!("{line} It's no crime here, but it won't be forgotten."));
        }
    }
}
