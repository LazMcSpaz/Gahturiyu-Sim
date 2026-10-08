//! The four races, and what being born into one tilts.
//!
//! A race sets the *average* of each trait, never the value. Every person is
//! drawn around their race's average with a wide spread, so a homebody people
//! still produces adventurers and a restless people still produces homebodies.
//! The four overlap far more than they differ — they live in the same towns.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Race {
    Roduro,
    Qotiro,
    Horaro,
    Tadoro,
}

pub const ALL_RACES: [Race; 4] = [Race::Roduro, Race::Qotiro, Race::Horaro, Race::Tadoro];

/// Per-person temperament, each 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Traits {
    /// Pull toward the road. Drives who leaves home, and how far they go.
    pub wanderlust: f32,
    /// Willingness to take risks. Feeds fighting strength; later, who starts trouble.
    pub boldness: f32,
    /// Liking for company. Decides who tags along when someone sets off.
    pub sociability: f32,
    /// Slowness to anger and to forget. Will feed grudges and feuds later.
    pub patience: f32,
}

/// How far individuals stray from their race's average. Wide on purpose.
pub const TRAIT_SPREAD: f32 = 0.17;

impl Race {
    pub fn name(self) -> &'static str {
        match self {
            Race::Roduro => "Roduro",
            Race::Qotiro => "Qotiro",
            Race::Horaro => "Horaro",
            Race::Tadoro => "Ṭaḍoro",
        }
    }

    pub fn element(self) -> &'static str {
        match self {
            Race::Roduro => "Earth",
            Race::Qotiro => "Fire",
            Race::Horaro => "Water",
            Race::Tadoro => "Air",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    /// The race's average temperament.
    pub fn trait_means(self) -> Traits {
        match self {
            // Melancholic: rooted, patient, slow to move.
            Race::Roduro => Traits { wanderlust: 0.22, boldness: 0.45, sociability: 0.45, patience: 0.75 },
            // Choleric: willful, martial, quick to act.
            Race::Qotiro => Traits { wanderlust: 0.40, boldness: 0.75, sociability: 0.50, patience: 0.30 },
            // Phlegmatic: receptive, communal, coast-bound.
            Race::Horaro => Traits { wanderlust: 0.32, boldness: 0.32, sociability: 0.62, patience: 0.62 },
            // Sanguine: quick, sociable, restless.
            Race::Tadoro => Traits { wanderlust: 0.68, boldness: 0.50, sociability: 0.62, patience: 0.35 },
        }
    }

    /// Body build, as a contribution to fighting strength.
    pub fn build(self) -> f32 {
        match self {
            Race::Roduro => 1.10, // short and stocky
            Race::Qotiro => 1.20, // muscular
            Race::Horaro => 0.95,
            Race::Tadoro => 0.90, // lean
        }
    }

    /// Walking speed in metres per second.
    pub fn walk_speed(self) -> f32 {
        match self {
            Race::Roduro => 1.15,
            Race::Qotiro => 1.35,
            Race::Horaro => 1.20,
            Race::Tadoro => 1.45,
        }
    }
}

/// Plain-words description of a trait value, for the playtest window.
pub fn trait_word(v: f32, low: &'static str, mid: &'static str, high: &'static str) -> &'static str {
    if v < 0.33 {
        low
    } else if v < 0.66 {
        mid
    } else {
        high
    }
}
