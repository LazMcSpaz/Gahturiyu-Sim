//! Who belongs to which faction, and the squad's guilds (`docs/factions.md`,
//! F4-F5; Laz, 2026-10-12).
//!
//! **A person's factions** are never stored: they're a pure function of what
//! the world already knows (`factions_of`). Their trade's group comes first
//! (the civic groups that belong to a faction, `disguise::CIVIC`, and the
//! guilds' trades, `GUILD_TRADES`), then whoever holds their town, then that
//! holder's parent. A syndicate is kept quiet (`syndicate_of`): low honour and
//! one keyed roll, the Old Hands for a Roduro, the Night Market for anyone
//! else; it only comes out to someone they trust.
//!
//! **The squad's guilds** (`World::guilds`, saved) are joined in talk with a
//! member who doesn't think badly of them (`can_join`). Joining one syndicate
//! sets the other against them (`deed`); joining the Rooted ends every other
//! membership and closes the door on new ones.

use super::factions::{self, FactionId, Size};
use super::jobs::Job;
use super::person::PersonId;
use super::rng::Rng;
use super::world::World;

/// Guilds by trade (beyond the civic groups that already name one).
pub static GUILD_TRADES: &[(Job, &str)] = &[(Job::Scribe, "writers"), (Job::Teacher, "writers"), (Job::Caravaner, "lantern_company")];

/// Below this honour someone may be in a syndicate; this share of them are.
pub const SYNDICATE_HONOUR: f32 = 0.35;
pub const SYNDICATE_SHARE: f32 = 0.4;
/// What joining does to standing with the guild, and with a rival syndicate.
pub const JOIN_DEED: f32 = 15.0;
pub const RIVAL_DEED: f32 = 60.0;

impl World {
    /// The factions someone belongs to, most particular first: their
    /// trade's group, their town's holder, the holder's parent.
    pub fn factions_of(&self, pid: PersonId) -> Vec<FactionId> {
        let mut out: Vec<FactionId> = Vec::new();
        let mut add = |f: FactionId| {
            if !out.contains(&f) {
                out.push(f);
            }
        };
        let p = &self.people[pid as usize];
        if p.in_squad {
            return out;
        }
        // A leader belongs to their faction before anything else.
        if let Some(l) = self.leader_of(pid) {
            let f = factions::id(l.faction);
            add(f);
            add(factions::parent(f));
        }
        if let Some(life) = self.society.lives.get(pid as usize) {
            let job = life.job;
            if let Some(f) = super::disguise::CIVIC.iter().filter(|c| !c.elite && c.knows.contains(&job)).find_map(|c| c.faction) {
                add(factions::id(f));
            }
            if let Some(&(_, f)) = GUILD_TRADES.iter().find(|g| g.0 == job) {
                add(factions::id(f));
            }
        }
        if let Some(h) = p.home.and_then(|t| self.holder_of(t)) {
            add(h);
            add(factions::parent(h));
        }
        out
    }

    /// The syndicate someone is quietly part of, if any.
    pub fn syndicate_of(&self, pid: PersonId) -> Option<FactionId> {
        let p = &self.people[pid as usize];
        if p.in_squad {
            return None;
        }
        let life = self.society.lives.get(pid as usize)?;
        if life.habits.honour >= SYNDICATE_HONOUR {
            return None;
        }
        let roll = Rng::from_keys(&[self.seed, pid as u64, 0x5359_4E44]).f32();
        (roll < SYNDICATE_SHARE).then(|| factions::id(if p.race == super::race::Race::Roduro { "old_hands" } else { "night_market" }))
    }

    /// The guild someone could take the squad into: one they belong to
    /// (their syndicate only once they trust the asker).
    pub fn guild_of(&self, pid: PersonId, trusted: bool) -> Option<FactionId> {
        let open = self.factions_of(pid).into_iter().find(|&f| factions::def(f).size == Size::Guild);
        open.or_else(|| self.syndicate_of(pid).filter(|_| trusted))
    }

    /// Is the squad in this guild (or faction)?
    pub fn in_guild(&self, f: FactionId) -> bool {
        self.guilds.contains(&f)
    }

    /// Why the squad can't join `f`, if it can't.
    pub fn can_join(&self, f: FactionId) -> Result<(), &'static str> {
        let d = factions::def(f);
        if d.size != Size::Guild && !d.one_way {
            return Err("that isn't something one joins");
        }
        if self.guilds.iter().any(|&g| factions::def(g).one_way) {
            return Err("the Rooted let no one go");
        }
        if self.in_guild(f) {
            return Err("you're one of them already");
        }
        if let Some(r) = rival_of(f) {
            if self.in_guild(r) {
                return Err("you're with the other lot");
            }
        }
        if self.standing_with(f) <= factions::UNWELCOME {
            return Err("they won't have you");
        }
        Ok(())
    }

    /// The squad joins `f`. A syndicate sets its rival against them; the
    /// Rooted take them from everything else.
    pub fn join_guild(&mut self, f: FactionId) -> Result<(), &'static str> {
        self.can_join(f)?;
        if factions::def(f).one_way {
            self.guilds.clear();
        }
        self.guilds.push(f);
        self.deed(f, JOIN_DEED);
        if let Some(r) = rival_of(f) {
            self.deed(r, -RIVAL_DEED);
        }
        let t = self.time;
        self.say(t, format!("You are counted among {} now.", factions::def(f).name));
        Ok(())
    }

    /// The squad leaves `f` (never the Rooted).
    pub fn leave_guild(&mut self, f: FactionId) -> bool {
        if factions::def(f).one_way || !self.in_guild(f) {
            return false;
        }
        self.guilds.retain(|&g| g != f);
        true
    }
}

/// The other syndicate, for one of the two.
pub fn rival_of(f: FactionId) -> Option<FactionId> {
    match factions::def(f).key {
        "old_hands" => Some(factions::id("night_market")),
        "night_market" => Some(factions::id("old_hands")),
        _ => None,
    }
}
