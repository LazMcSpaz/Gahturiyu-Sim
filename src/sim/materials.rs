//! What things are made of, who makes them, and how well.
//!
//! A **material** carries the traits that matter in use: how keen an edge
//! it takes and how much weight it puts behind a blow, how much armour it
//! gets through, how well it turns cuts and blows, what it weighs, how long
//! it lasts and whether it can be mended. Every weapon and piece of armour is
//! a **form** (sword, spear, scale coat...) in a main material, sometimes
//! backed by a second one (an Edgeglass edge on a Forgeiron spine), at a
//! **grade** from crude to masterwork. The catalogue in `items.rs` holds
//! every such combination, worked out once from these tables.
//!
//! Each material belongs to a making **tradition** (a label, not a rule:
//! anyone can learn any craft) and is worked by one **craft** — the skill
//! that makes and mends it.

use serde::{Deserialize, Serialize};

use super::stats::Skill;

/// The making traditions. A label on materials; never a gate.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Tradition {
    Shared,
    Grown,
    Fire,
    Sea,
    Written,
}

impl Tradition {
    pub fn name(self) -> &'static str {
        match self {
            Tradition::Shared => "shared basics",
            Tradition::Grown => "grown stone",
            Tradition::Fire => "fire-made",
            Tradition::Sea => "sea and shore",
            Tradition::Written => "written and carried",
        }
    }
}

/// The crafts: each is a skill, made and mended at its own station.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Craft {
    /// Leather, cloth and wood: the shared basics.
    Handcraft,
    /// Fire metals and glass: weapons, ingots.
    Smithing,
    /// Fire-metal armour and shields.
    Armoring,
    /// Growing stone.
    Tending,
    /// Reed, shell, fishskin and tentsilk.
    Weaving,
    /// Paper, ink and scrolls.
    Inscription,
    Alchemy,
    /// Timber building (huts, longhouses, palisades): bases, Part 7.
    Carpentry,
    /// Laid stone (rubble walls, wells, kitchens): bases, Part 7.
    Masonry,
}

pub const CRAFTS: [Craft; 9] = [Craft::Handcraft, Craft::Smithing, Craft::Armoring, Craft::Tending, Craft::Weaving, Craft::Inscription, Craft::Alchemy, Craft::Carpentry, Craft::Masonry];
pub const N_CRAFTS: usize = 9;

impl Craft {
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn skill(self) -> Skill {
        match self {
            Craft::Handcraft => Skill::Handcraft,
            Craft::Smithing => Skill::Smithing,
            Craft::Armoring => Skill::Armoring,
            Craft::Tending => Skill::Tending,
            Craft::Weaving => Skill::Weaving,
            Craft::Inscription => Skill::Inscription,
            Craft::Alchemy => Skill::Alchemy,
            Craft::Carpentry => Skill::Carpentry,
            Craft::Masonry => Skill::Masonry,
        }
    }
    pub fn of_skill(s: Skill) -> Option<Craft> {
        CRAFTS.iter().copied().find(|c| c.skill() == s)
    }
    /// Can someone who works this craft mend what `other` makes? (Smiths and
    /// armourers both work fire metals.)
    pub fn mends(self, other: Craft) -> bool {
        self == other || matches!((self, other), (Craft::Smithing, Craft::Armoring) | (Craft::Armoring, Craft::Smithing))
    }

    pub fn name(self) -> &'static str {
        match self {
            Craft::Handcraft => "leather, cloth and woodwork",
            Craft::Smithing => "smithing",
            Craft::Armoring => "armouring",
            Craft::Tending => "stone-tending",
            Craft::Weaving => "weaving and sealing",
            Craft::Inscription => "paper, ink and scrolls",
            Craft::Alchemy => "alchemy",
            Craft::Carpentry => "carpentry",
            Craft::Masonry => "masonry",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Material {
    /// Not a made thing (or not one that wears): rings, tools, food.
    None,
    // Shared basics.
    PlainIron,
    Wood,
    Leather,
    Cloth,
    // Grown.
    Ringstone,
    Hearthclay,
    Slatewing,
    Edgeglass,
    // Fire.
    Bronze,
    Forgeiron,
    Sandglass,
    Gold,
    // Sea and shore.
    Seareed,
    Nacre,
    Fishskin,
    // Written and carried.
    Tentsilk,
}

/// A material's traits. Multipliers are against a plain middling material
/// (1.0); shares are 0..1.
pub struct MatDef {
    pub name: &'static str,
    pub tradition: Tradition,
    /// The craft that makes and mends it.
    pub craft: Craft,
    /// Cut damage, as an edge.
    pub edge: f32,
    /// Blunt damage, as weight behind a blow.
    pub mass: f32,
    /// Share of the target's armour a weapon of it gets through.
    pub pierce: f32,
    /// How well it turns cuts, and blows, as armour.
    pub turn_cut: f32,
    pub turn_blunt: f32,
    /// Weight multiplier.
    pub density: f32,
    /// Durability, in blows.
    pub durability: f32,
    /// How fast it wears per blow (Edgeglass wears slowly).
    pub wear: f32,
    /// Extra wear from heavy blunt blows (cracks and splits).
    pub brittle: f32,
    pub repairable: bool,
    /// Loses durability to damp unless sealed with pitch.
    pub rots: bool,
    /// Days to grow (grown materials), to an ordered piece.
    pub grow_days: f64,
    /// Value multiplier.
    pub worth: f32,
}

const fn m(name: &'static str, tradition: Tradition, craft: Craft, edge: f32, mass: f32, pierce: f32, turn_cut: f32, turn_blunt: f32, density: f32, durability: f32, wear: f32, brittle: f32, repairable: bool, rots: bool, grow_days: f64, worth: f32) -> MatDef {
    MatDef { name, tradition, craft, edge, mass, pierce, turn_cut, turn_blunt, density, durability, wear, brittle, repairable, rots, grow_days, worth }
}

use Craft as C;
use Tradition as T;

/// Indexed by `Material as usize`. Placeholder numbers, to tune.
pub static MATERIALS: [MatDef; 17] = [
    //  name          tradition  craft          edge  mass pierce tcut tblunt dens  dur    wear brittle repair rots  grow  worth
    m("Unmade", T::Shared, C::Handcraft, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, false, false, 0.0, 1.0),
    m("Plain iron", T::Shared, C::Handcraft, 0.9, 1.0, 0.10, 0.9, 0.9, 1.0, 120.0, 1.0, 0.0, true, false, 0.0, 0.8),
    m("Wood", T::Shared, C::Handcraft, 0.7, 0.9, 0.0, 0.7, 0.9, 0.7, 90.0, 1.0, 0.2, true, false, 0.0, 0.6),
    m("Leather", T::Shared, C::Handcraft, 0.8, 0.8, 0.0, 1.0, 1.0, 1.0, 100.0, 1.0, 0.0, true, false, 0.0, 1.0),
    m("Cloth", T::Shared, C::Handcraft, 0.6, 0.6, 0.0, 1.0, 1.0, 1.0, 60.0, 1.0, 0.0, true, false, 0.0, 1.0),
    m("Ringstone", T::Grown, C::Tending, 0.8, 1.3, 0.05, 1.0, 1.35, 1.3, 300.0, 0.6, 0.0, true, false, 30.0, 1.4),
    m("Hearthclay", T::Grown, C::Tending, 0.5, 0.8, 0.0, 0.6, 0.6, 1.0, 25.0, 1.0, 1.0, true, false, 4.0, 0.5),
    m("Slatewing", T::Grown, C::Tending, 1.0, 0.9, 0.05, 1.3, 0.75, 0.65, 160.0, 1.0, 1.4, true, false, 21.0, 1.6),
    m("Edgeglass", T::Grown, C::Tending, 1.4, 0.8, 0.0, 1.1, 0.6, 0.7, 400.0, 0.35, 0.6, false, false, 90.0, 2.6),
    m("Bronze", T::Fire, C::Smithing, 0.9, 1.0, 0.15, 0.95, 0.95, 1.05, 130.0, 1.0, 0.0, true, false, 0.0, 0.7),
    m("Forgeiron", T::Fire, C::Smithing, 1.0, 1.05, 0.35, 1.0, 1.0, 1.0, 220.0, 1.0, 0.0, true, false, 0.0, 1.0),
    m("Sandglass", T::Fire, C::Smithing, 0.9, 0.5, 0.0, 0.3, 0.2, 0.6, 15.0, 1.0, 2.0, false, false, 0.0, 1.2),
    m("Gold", T::Fire, C::Smithing, 0.5, 1.2, 0.0, 0.5, 0.5, 1.9, 60.0, 1.0, 0.0, true, false, 0.0, 6.0),
    m("Seareed", T::Sea, C::Weaving, 0.5, 0.5, 0.0, 0.8, 0.9, 0.5, 80.0, 1.0, 0.0, true, true, 0.0, 0.7),
    m("Nacre", T::Sea, C::Weaving, 1.15, 0.8, 0.10, 1.3, 0.7, 0.7, 150.0, 1.0, 1.3, true, false, 0.0, 1.7),
    m("Fishskin", T::Sea, C::Weaving, 0.6, 0.6, 0.0, 1.05, 0.9, 0.7, 75.0, 1.0, 0.0, true, false, 0.0, 1.1),
    m("Tentsilk", T::Written, C::Weaving, 0.5, 0.5, 0.0, 1.1, 1.0, 0.35, 110.0, 1.0, 0.0, true, false, 0.0, 1.8),
];

impl Material {
    pub fn def(self) -> &'static MatDef {
        &MATERIALS[self as usize]
    }
    pub fn name(self) -> &'static str {
        self.def().name
    }
    pub fn is_grown(self) -> bool {
        self.def().tradition == Tradition::Grown
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Grade {
    Crude,
    Common,
    Fine,
    Masterwork,
}

pub const GRADES: [Grade; 4] = [Grade::Crude, Grade::Common, Grade::Fine, Grade::Masterwork];

impl Grade {
    pub fn name(self) -> &'static str {
        match self {
            Grade::Crude => "Crude",
            Grade::Common => "Common",
            Grade::Fine => "Fine",
            Grade::Masterwork => "Masterwork",
        }
    }
    /// Damage and protection.
    pub fn power(self) -> f32 {
        [0.85, 1.0, 1.1, 1.2][self as usize]
    }
    pub fn durability(self) -> f32 {
        [0.7, 1.0, 1.3, 1.6][self as usize]
    }
    pub fn worth(self) -> f32 {
        [0.6, 1.0, 1.7, 2.8][self as usize]
    }
    /// The grade a piece comes out at: the maker's skill (0–100), how well
    /// set up their station is (0 = makeshift, 1 = a proper one), and a roll.
    pub fn from(skill: f32, station: f32, roll: f32) -> Grade {
        let score = skill / 100.0 + station * 0.1 + (roll - 0.5) * 0.4;
        if score < 0.2 {
            Grade::Crude
        } else if score < 0.7 {
            Grade::Common
        } else if score < 0.95 {
            Grade::Fine
        } else {
            Grade::Masterwork
        }
    }
}

/// A maker's skill at which they stamp their mark whatever the grade.
pub const MARK_SKILL: f32 = 60.0;

/// Who made a piece, and where. Kept on every made piece (later, for law and
/// stolen goods); `stamped` pieces show the maker's mark and sell for more.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    pub maker: u32,
    pub town: Option<u16>,
    pub stamped: bool,
}

/// One made thing's own state: how worn it is, who made it, whether it's sealed.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    /// Durability left, as of `at`.
    pub left: f32,
    pub at: f64,
    pub mark: Option<Mark>,
    /// Sealed with pitch (woven things that would otherwise rot).
    pub sealed: bool,
}

/// Durability lost a day by unsealed reed.
pub const ROT_PER_DAY: f32 = 4.0;

impl Piece {
    pub fn new(max: f32, t: f64) -> Piece {
        Piece { left: max, at: t, mark: None, sealed: false }
    }
    /// Durability left at time `t`, after any rot.
    pub fn left_at(&self, rots: bool, t: f64) -> f32 {
        if rots && !self.sealed {
            self.left - ROT_PER_DAY * ((t - self.at).max(0.0) / super::world::DAY) as f32
        } else {
            self.left
        }
    }
    /// Bring the stored value up to `t`.
    pub fn settle(&mut self, rots: bool, t: f64) {
        self.left = self.left_at(rots, t);
        self.at = t;
    }
}
