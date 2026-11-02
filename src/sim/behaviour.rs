//! What each squad member does when the player isn't controlling them
//! (Laz, 2 Nov 2026; `docs/third-person.md`): settings the player chooses,
//! read by the squad mates' own decisions. Data only; the rules that read
//! them are where the acting is (following and holding in `dungeon.rs`;
//! fighting, healing and items when the fight rules take them up).

use serde::{Deserialize, Serialize};

use super::geo::V2;
use super::person::PersonId;
use super::world::World;

/// How ready they are to fight.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stance {
    /// Goes looking for a fight.
    Aggressive,
    /// Fights what attacks them or the one controlled.
    Defensive,
    /// Keeps out of fights.
    Passive,
}

/// How they fight.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Close in.
    Melee,
    /// Keep their distance and shoot.
    Ranged,
    /// Stay back: spells and healing.
    Support,
}

/// How close they keep to the one controlled, or a spot they hold.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Follow {
    Close,
    Medium,
    Far,
    /// Stay at `Behaviour::hold`.
    Hold,
}

impl Follow {
    /// How far behind they let the one controlled get before going after
    /// them, metres.
    pub fn gap(self) -> f32 {
        match self {
            Follow::Close => 1.8,
            Follow::Medium => 3.5,
            Follow::Far => 7.0,
            Follow::Hold => f32::INFINITY,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Behaviour {
    pub stance: Stance,
    pub role: Role,
    pub follow: Follow,
    /// Where they hold (with `Follow::Hold`): the site, the spot (world
    /// metres) and its height.
    pub hold: Option<(u32, V2, f32)>,
    /// Drink a healing draught below this share of their health, in
    /// hundredths (0: never).
    pub heal_below: u8,
    /// Bind their own bleeding wounds.
    pub bind: bool,
    /// Use scrolls and bottles of their own accord.
    pub items: bool,
}

impl Default for Behaviour {
    fn default() -> Self {
        Behaviour { stance: Stance::Defensive, role: Role::Melee, follow: Follow::Medium, hold: None, heal_below: 50, bind: true, items: false }
    }
}

/// The choices the settings offer, in order (the panel's rows).
pub const STANCES: [(Stance, &str); 3] = [(Stance::Aggressive, "Aggressive"), (Stance::Defensive, "Defensive"), (Stance::Passive, "Passive")];
pub const ROLES: [(Role, &str); 3] = [(Role::Melee, "Melee"), (Role::Ranged, "Ranged"), (Role::Support, "Support")];
pub const FOLLOWS: [(Follow, &str); 4] = [(Follow::Close, "Close"), (Follow::Medium, "Medium"), (Follow::Far, "Far"), (Follow::Hold, "Hold here")];
pub const HEALS: [(u8, &str); 4] = [(0, "Never"), (25, "Below ¼"), (50, "Below ½"), (75, "Below ¾")];

impl World {
    /// A squad member's settings (the defaults until changed).
    pub fn behaviour(&self, pid: PersonId) -> Behaviour {
        self.behaviours.get(&pid).copied().unwrap_or_default()
    }

    /// Change a squad member's settings. Choosing to hold holds where they
    /// stand now (in a site: its spot and height).
    pub fn set_behaviour(&mut self, pid: PersonId, mut b: Behaviour) {
        if b.follow == Follow::Hold && self.behaviour(pid).follow != Follow::Hold {
            b.hold = None;
            if let (Some(i), Some(k)) = (self.sites.inside.get(&pid), self.squad.index(pid)) {
                b.hold = Some((i.site, self.squad.at[k], i.h));
            }
        }
        if b.follow != Follow::Hold {
            b.hold = None;
        }
        self.behaviours.insert(pid, b);
    }
}
