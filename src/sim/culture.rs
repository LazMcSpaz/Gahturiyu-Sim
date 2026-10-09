//! Cultures are tendencies, not scripts.
//!
//! Each people has a **profile**: weighted leanings for every custom (who
//! cooks, the rhythm of the day, who belongs together) and for how its
//! people live one by one (lodging, keeping their own hours, where they
//! spend the evening, what work they're drawn to). A community's culture is
//! the **blend** of its residents' profiles, weighted by how many of each
//! live there, plus a small seeded nudge so two towns with the same mix can
//! still differ.
//!
//! Every custom is then **chosen from the blend with one fixed roll** per
//! community and custom. The roll never changes; the blend does. So a town
//! keeps its ways while its people stay much the same, and drifts into new
//! ones when the mix shifts enough to carry the blend past its roll.
//!
//! Nothing outside this file asks what race a town is: systems read the
//! community's chosen customs, and a person's own rolled habits.

use serde::{Deserialize, Serialize};

use super::jobs::Job;
use super::race::{Race, ALL_RACES};
use super::rng::{self, Rng};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cooking {
    /// Each household cooks its own, from its garden and the market.
    Household,
    /// A hearth kitchen feeds the workers; runners carry pots out at midday.
    Hearth,
    /// Everyone eats together, cooked on a shared deck.
    Deck,
}
pub const COOKINGS: [Cooking; 3] = [Cooking::Household, Cooking::Hearth, Cooking::Deck];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rhythm {
    /// Steady hours that slide with the season.
    Seasonal,
    /// Fixed shifts rung by bells.
    Bells,
    /// Work follows the low tide, a little later each day.
    Tides,
    /// Everyone keeps their own hours.
    Irregular,
}
pub const RHYTHMS: [Rhythm; 4] = [Rhythm::Seasonal, Rhythm::Bells, Rhythm::Tides, Rhythm::Irregular];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Belonging {
    /// A household per home, by family line.
    Lineage,
    /// Homes grouped into tier blocks, two homes to a household.
    TierBlock,
    /// The whole community is one household.
    Village,
    /// Everyone is their own household, lodging where they sleep.
    Lodging,
}
pub const BELONGINGS: [Belonging; 4] = [Belonging::Lineage, Belonging::TierBlock, Belonging::Village, Belonging::Lodging];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Layout {
    /// One counter, many hands: one building, separate specialists inside.
    Combined,
    /// Separate shops and houses for each service.
    Split,
}

/// What a strong minority sets up for itself.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Institution {
    TendersYard,
    MessHall,
    Deck,
    LettersHouse,
}

impl Institution {
    pub fn name(self) -> &'static str {
        match self {
            Institution::TendersYard => "a Tenders' yard",
            Institution::MessHall => "a mess hall of their own",
            Institution::Deck => "a cooking deck of their own",
            Institution::LettersHouse => "a letters house",
        }
    }
    /// The way of cooking it brings for the people who use it, if any.
    pub fn cooking(self) -> Option<Cooking> {
        match self {
            Institution::MessHall => Some(Cooking::Hearth),
            Institution::Deck => Some(Cooking::Deck),
            _ => None,
        }
    }
}

impl Cooking {
    pub fn name(self) -> &'static str {
        match self {
            Cooking::Household => "Each household cooks its own",
            Cooking::Hearth => "Hearth kitchen and meal runners",
            Cooking::Deck => "Shared deck cooking",
        }
    }
    /// What the middle food path is called under this custom.
    pub fn kitchen_word(self) -> &'static str {
        match self {
            Cooking::Household => "Home pots",
            Cooking::Hearth => "Hearth kitchen",
            Cooking::Deck => "Deck cooking",
        }
    }
}

impl Rhythm {
    pub fn name(self) -> &'static str {
        match self {
            Rhythm::Seasonal => "Seasonal hours",
            Rhythm::Bells => "Bells and fixed shifts",
            Rhythm::Tides => "Tide hours",
            Rhythm::Irregular => "Irregular hours",
        }
    }
}

impl Belonging {
    pub fn name(self) -> &'static str {
        match self {
            Belonging::Lineage => "Households by family line",
            Belonging::TierBlock => "Tier-block households",
            Belonging::Village => "One household for all",
            Belonging::Lodging => "Everyone lodges",
        }
    }
}

impl Layout {
    pub fn name(self) -> &'static str {
        match self {
            Layout::Combined => "Combined services (one counter, many hands)",
            Layout::Split => "Separate shops and houses",
        }
    }
}

/// A part of a town's government: who sits in it and how they're replaced.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rule {
    /// The heads of the oldest households; a seat passes at death.
    Elders,
    /// Priestesses at the summit, men administering below; a failed rite
    /// brings the high priestess down.
    Priestesses,
    /// A villager speaks for a season, then the next; a poor one is skipped.
    Speaker,
}
pub const RULES: [Rule; 3] = [Rule::Elders, Rule::Priestesses, Rule::Speaker];

impl Rule {
    pub fn name(self) -> &'static str {
        match self {
            Rule::Elders => "Elder circle",
            Rule::Priestesses => "Priestesses",
            Rule::Speaker => "Rotating speaker",
        }
    }
}

/// How a wrong is settled.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Justice {
    /// Elders judge: pay in coin or labour; serious cases, temporary bondage.
    #[default]
    Elders,
    /// The parties fight it out (to knockout); the loser pays.
    Duel,
    /// The village withdraws from the offender.
    Shunning,
    /// It's written down, and follows them wherever arbiters reach.
    Record,
}
pub const JUSTICES: [Justice; 4] = [Justice::Elders, Justice::Duel, Justice::Shunning, Justice::Record];

impl Justice {
    pub fn name(self) -> &'static str {
        match self {
            Justice::Elders => "Elder judgement",
            Justice::Duel => "Trial by duel",
            Justice::Shunning => "Shunning",
            Justice::Record => "Public record",
        }
    }
}

/// What bondage may be.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Slavery {
    /// Slavery for life is allowed.
    Allowed,
    /// Only bondage for a set term, to pay off a debt or settle a dispute.
    #[default]
    Temporary,
    /// Any bondage is written down and term-limited.
    Recorded,
}
pub const SLAVERIES: [Slavery; 3] = [Slavery::Allowed, Slavery::Temporary, Slavery::Recorded];

impl Slavery {
    pub fn name(self) -> &'static str {
        match self {
            Slavery::Allowed => "Slavery allowed",
            Slavery::Temporary => "Temporary bondage only",
            Slavery::Recorded => "Bondage recorded and term-limited",
        }
    }
}

/// Where someone spends the evening, by preference.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Evening {
    Hearth,
    Inn,
    Deck,
    Home,
}
pub const EVENINGS: [Evening; 4] = [Evening::Hearth, Evening::Inn, Evening::Deck, Evening::Home];

/// One people's leanings. Starting weights to tune, not rules.
pub struct Profile {
    /// Household, hearth, deck. All zero means "eat what the hosts cook":
    /// these people add nothing to the blend for this custom.
    pub cooking: [f32; 3],
    /// Seasonal, bells, tides, irregular.
    pub rhythm: [f32; 4],
    /// Lineage, tier block, village, lodging.
    pub belonging: [f32; 4],
    /// Evening at the hearth, inn, deck or home.
    pub evening: [f32; 4],
    /// Chance one of them keeps their own people's hours instead of the town's.
    pub own_ways: f32,
    /// Chance one of them lodges in another household's home.
    pub lodger: f32,
    /// What a strong minority of them sets up.
    pub institution: Institution,
    /// Leanings toward particular work (multipliers; anything not listed is 1).
    pub jobs: &'static [(Job, f32)],
    /// How much they take to each craft (by `Craft::index`: handcraft,
    /// smithing, armouring, tending, weaving, inscription, alchemy).
    pub crafts: [f32; 7],
    /// How their teachers lean to teach a craft: deep and slow, or quick
    /// and drilled (weights).
    pub teaching: [f32; 2],
    /// Which part of a government they lean to set up (elders, priestesses,
    /// speaker). All zero: they set up none of their own.
    pub ruling: [f32; 3],
    /// How they settle wrongs (elders, duel, shunning, record).
    pub justice: [f32; 4],
    /// What bondage may be (allowed, temporary, recorded).
    pub slavery: [f32; 3],
}

/// Indexed by `Race::index()`: Roduro, Qotiro, Horaro, Ṭaḍoro.
pub const PROFILES: [Profile; 4] = [
    // Roduro: households by lineage, seasons and generations.
    Profile {
        cooking: [0.80, 0.12, 0.08],
        rhythm: [0.80, 0.08, 0.04, 0.08],
        belonging: [0.82, 0.08, 0.06, 0.04],
        evening: [0.30, 0.15, 0.05, 0.50],
        own_ways: 0.10,
        lodger: 0.02,
        institution: Institution::TendersYard,
        jobs: &[(Job::StoneTender, 8.0), (Job::Farmer, 1.6), (Job::Woodcutter, 1.4), (Job::Fisher, 0.7), (Job::Exchanger, 0.4), (Job::Arbiter, 0.4)],
        crafts: [1.0, 0.15, 0.10, 1.0, 0.10, 0.20, 0.4],
        teaching: [0.8, 0.2],
        ruling: [1.0, 0.0, 0.0],
        justice: [0.80, 0.10, 0.05, 0.05],
        slavery: [0.05, 0.85, 0.10],
    },
    // Qotiro: mess halls and hearth kitchens, bells and fixed shifts, tier blocks.
    Profile {
        cooking: [0.10, 0.82, 0.08],
        rhythm: [0.10, 0.78, 0.04, 0.08],
        belonging: [0.08, 0.80, 0.06, 0.06],
        evening: [0.45, 0.30, 0.05, 0.20],
        own_ways: 0.15,
        lodger: 0.02,
        institution: Institution::MessHall,
        jobs: &[(Job::Guard, 1.8), (Job::Cook, 1.6), (Job::Runner, 2.2), (Job::Smith, 2.2), (Job::Armourer, 2.2), (Job::Priest, 1.6), (Job::Official, 1.6), (Job::Healer, 1.3), (Job::StoneTender, 0.3), (Job::Exchanger, 0.4)],
        crafts: [1.0, 1.0, 0.8, 0.10, 0.10, 0.3, 0.4],
        teaching: [0.15, 0.85],
        ruling: [0.0, 1.0, 0.0],
        justice: [0.10, 0.80, 0.05, 0.05],
        slavery: [0.80, 0.15, 0.05],
    },
    // Horaro: shared decks, the tide, the whole village one family.
    Profile {
        cooking: [0.10, 0.08, 0.82],
        rhythm: [0.08, 0.04, 0.80, 0.08],
        belonging: [0.08, 0.04, 0.82, 0.06],
        evening: [0.20, 0.10, 0.60, 0.10],
        own_ways: 0.20,
        lodger: 0.04,
        institution: Institution::Deck,
        jobs: &[(Job::Fisher, 3.5), (Job::KelpGatherer, 3.5), (Job::Boatwright, 3.5), (Job::Healer, 1.3), (Job::StoneTender, 0.3), (Job::Guard, 0.7)],
        crafts: [1.0, 0.10, 0.10, 0.10, 1.0, 0.2, 0.4],
        teaching: [0.5, 0.5],
        ruling: [0.0, 0.0, 1.0],
        justice: [0.05, 0.05, 0.85, 0.05],
        slavery: [0.05, 0.85, 0.10],
    },
    // Ṭaḍoro: fed by their hosts (no cooking leaning of their own), the
    // stars, and lodging in others' homes.
    Profile {
        cooking: [0.0, 0.0, 0.0],
        rhythm: [0.05, 0.05, 0.05, 0.85],
        belonging: [0.04, 0.04, 0.04, 0.88],
        evening: [0.35, 0.40, 0.15, 0.10],
        own_ways: 0.60,
        lodger: 0.85,
        institution: Institution::LettersHouse,
        jobs: &[(Job::Exchanger, 8.0), (Job::Arbiter, 8.0), (Job::Teacher, 3.0), (Job::Scribe, 3.5), (Job::Merchant, 1.6), (Job::Caravaner, 1.8), (Job::Farmer, 0.5), (Job::StoneTender, 0.2), (Job::Guard, 0.7)],
        crafts: [1.0, 0.2, 0.1, 0.1, 0.4, 1.0, 0.6],
        teaching: [0.6, 0.4],
        // No towns of their own: they serve as arbiters wherever peoples meet.
        ruling: [0.0, 0.0, 0.0],
        justice: [0.10, 0.05, 0.05, 0.80],
        slavery: [0.0, 0.20, 0.80],
    },
];

pub fn profile(r: Race) -> &'static Profile {
    &PROFILES[r.index()]
}

/// How much a town's own seeded nudge moves its leanings (a spread on a
/// multiplier, so 0.35 is roughly ±35%).
pub const NUDGE: f32 = 0.35;
/// How much the nudge moves the split-or-combined leaning.
pub const SPLIT_NUDGE: f32 = 0.12;
/// A people this large a share (and not the biggest) leans toward its own
/// institution: not at all below the first, almost certainly above the second.
pub const MINORITY: (f32, f32) = (0.12, 0.35);
/// Re-blend once this share of the people a blend was made from has changed.
pub const BLEND_SHIFT: f32 = 0.10;

/// A community's culture: what its people lean toward, together.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Blend {
    /// Share of residents of each people.
    pub share: [f32; 4],
    pub cooking: [f32; 3],
    pub rhythm: [f32; 4],
    pub belonging: [f32; 4],
    /// Leaning toward separate shops over combined ones, 0..1.
    pub split: f32,
    /// Each people's leaning toward its own institution here, 0..1.
    pub minority: [f32; 4],
    /// How much the community takes to each craft (by `Craft::index`).
    #[serde(default)]
    pub crafts: [f32; 7],
    /// Leaning toward each part of government (not normalised: a people with
    /// no leaning adds nothing), toward each way of justice, and each rule
    /// on bondage.
    #[serde(default)]
    pub ruling: [f32; 3],
    #[serde(default)]
    pub justice: [f32; 4],
    #[serde(default)]
    pub slavery: [f32; 3],
}

/// What a community has settled on.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Customs {
    pub cooking: Cooking,
    pub rhythm: Rhythm,
    pub belonging: Belonging,
    pub layout: Layout,
    /// Which peoples have set up their own institution.
    pub own: [bool; 4],
    #[serde(default)]
    pub justice: Justice,
    #[serde(default)]
    pub slavery: Slavery,
}

impl Customs {
    pub fn institutions(&self) -> impl Iterator<Item = (Race, Institution)> + '_ {
        ALL_RACES.iter().filter(|r| self.own[r.index()]).map(|&r| (r, profile(r).institution))
    }
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix<const N: usize>(share: &[f32; 4], pick: impl Fn(&Profile) -> [f32; N], seed: u64, key: u64, tag: u64) -> [f32; N] {
    let mut w = [0.0f32; N];
    for (r, s) in share.iter().enumerate() {
        let p = pick(&PROFILES[r]);
        for o in 0..N {
            w[o] += s * p[o];
        }
    }
    if w.iter().sum::<f32>() <= 0.0 {
        w = [1.0; N];
    }
    for (o, x) in w.iter_mut().enumerate() {
        *x *= (NUDGE * Rng::from_keys(&[seed, key, tag, o as u64, 0x4E55_4447]).normal()).exp();
    }
    let total: f32 = w.iter().sum();
    w.map(|x| x / total)
}

impl Blend {
    /// The blend for these head-counts (by `Race::index()`); `key` names the
    /// community, so its nudge is its own.
    pub fn of(counts: [u32; 4], seed: u64, key: u64) -> Blend {
        let total = counts.iter().sum::<u32>().max(1) as f32;
        let share = counts.map(|c| c as f32 / total);
        let top = (0..4).max_by(|&a, &b| share[a].total_cmp(&share[b])).unwrap_or(0);
        let dominance = ((share[top] - 0.25) / 0.5).clamp(0.0, 1.0);
        let split = (smooth(0.0, 1.0, dominance) + SPLIT_NUDGE * Rng::from_keys(&[seed, key, 0x5350_4C54]).normal()).clamp(0.0, 1.0);
        let mut minority = [0.0; 4];
        for r in 0..4 {
            if r != top {
                minority[r] = smooth(MINORITY.0, MINORITY.1, share[r]);
            }
        }
        Blend {
            share,
            cooking: mix(&share, |p| p.cooking, seed, key, 1),
            rhythm: mix(&share, |p| p.rhythm, seed, key, 2),
            belonging: mix(&share, |p| p.belonging, seed, key, 3),
            split,
            minority,
            crafts: {
                let mut c = [0.0; 7];
                for (r, sh) in share.iter().enumerate() {
                    for (k, x) in c.iter_mut().enumerate() {
                        *x += sh * PROFILES[r].crafts[k];
                    }
                }
                c
            },
            ruling: {
                // Share-weighted, with the town's own nudge on each.
                let mut w = [0.0f32; 3];
                for (r, sh) in share.iter().enumerate() {
                    for (k, x) in w.iter_mut().enumerate() {
                        *x += sh * PROFILES[r].ruling[k];
                    }
                }
                for (k, x) in w.iter_mut().enumerate() {
                    *x *= (NUDGE * 0.5 * Rng::from_keys(&[seed, key, 7, k as u64, 0x4E55_4447]).normal()).exp();
                }
                w
            },
            justice: mix(&share, |p| p.justice, seed, key, 5),
            slavery: mix(&share, |p| p.slavery, seed, key, 6),
        }
    }

    /// Choose every custom: one fixed roll per community and custom, against
    /// the weights as they stand now.
    pub fn choose(&self, seed: u64, key: u64) -> Customs {
        let roll = |tag: u64| Rng::from_keys(&[seed, key, tag, 0x524F_4C4C]).f32();
        let pick = |w: &[f32], u: f32| -> usize {
            let mut acc = 0.0;
            for (i, x) in w.iter().enumerate() {
                acc += x;
                if u < acc {
                    return i;
                }
            }
            w.len() - 1
        };
        let mut own = [false; 4];
        for (r, o) in own.iter_mut().enumerate() {
            *o = roll(10 + r as u64) < self.minority[r];
        }
        Customs {
            cooking: COOKINGS[pick(&self.cooking, roll(1))],
            rhythm: RHYTHMS[pick(&self.rhythm, roll(2))],
            belonging: BELONGINGS[pick(&self.belonging, roll(3))],
            layout: if roll(4) < self.split { Layout::Split } else { Layout::Combined },
            own,
            justice: JUSTICES[pick(&self.justice, roll(5))],
            slavery: SLAVERIES[pick(&self.slavery, roll(6))],
        }
    }
}

/// A person's own habits, rolled once from their people's leanings.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Habits {
    /// Keeps these hours whatever the town keeps.
    pub own_rhythm: Option<Rhythm>,
    pub lodger: bool,
    pub evening: Evening,
    /// How they teach their trade, if they have one to teach.
    #[serde(default)]
    pub teaching: Teaching,
}

/// How a crafter teaches: deep and slow (more skill a lesson, longer and
/// dearer), or drilled (quick and cheap, less each time).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Teaching {
    #[default]
    Deep,
    Drilled,
}

impl Teaching {
    /// Skill a lesson gives, hours it takes, and its price (×).
    pub fn lesson(self) -> (f32, f64, f32) {
        match self {
            Teaching::Deep => (22.0, 4.0, 1.4),
            Teaching::Drilled => (14.0, 1.5, 0.8),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Teaching::Deep => "slow and deep",
            Teaching::Drilled => "quick drill",
        }
    }
}

impl Habits {
    pub fn roll(race: Race, seed: u64) -> Habits {
        let p = profile(race);
        let mut r = Rng::from_keys(&[seed, 0x4841_4249]);
        let own_rhythm = if r.chance(p.own_ways) { Some(RHYTHMS[r.weighted(&p.rhythm).unwrap_or(0)]) } else { None };
        let lodger = r.chance(p.lodger);
        let evening = EVENINGS[r.weighted(&p.evening).unwrap_or(3)];
        // Its own roll, so the habits above are what they always were.
        let teaching = if Rng::from_keys(&[seed, 0x5445_4143]).f32() < p.teaching[0] / (p.teaching[0] + p.teaching[1]) { Teaching::Deep } else { Teaching::Drilled };
        Habits { own_rhythm, lodger, evening, teaching }
    }
}

/// A community's key for its rolls: one for the land, one for the stilts.
pub fn community_key(town: u16, stilts: bool) -> u64 {
    rng::key(&[town as u64, stilts as u64, 0x434F_4D4D])
}

/// How far a population has moved from the one a blend was made from, as a
/// share of the old one.
pub fn shift(was: [u32; 4], now: [u32; 4]) -> f32 {
    let moved: u32 = (0..4).map(|r| was[r].abs_diff(now[r])).sum();
    moved as f32 / was.iter().sum::<u32>().max(1) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_people_alone_mostly_keeps_its_own_ways_but_not_always() {
        // Many single-people towns: the leaning shows, but not every time.
        let mut hearth = 0;
        for k in 0..400u64 {
            let b = Blend::of([0, 100, 0, 0], 7, k);
            if b.choose(7, k).cooking == Cooking::Hearth {
                hearth += 1;
            }
        }
        assert!(hearth > 260 && hearth < 390, "{hearth} of 400 all-Qotiro towns cook at a hearth");
    }

    #[test]
    fn mixed_towns_combine_and_dominated_towns_split() {
        let (mut mixed, mut dominated) = (0, 0);
        for k in 0..300u64 {
            if Blend::of([25, 25, 25, 25], 3, k).choose(3, k).layout == Layout::Split {
                mixed += 1;
            }
            if Blend::of([85, 5, 5, 5], 3, k).choose(3, k).layout == Layout::Split {
                dominated += 1;
            }
        }
        assert!(mixed < 40 && dominated > 220, "split: mixed {mixed}/300, dominated {dominated}/300");
    }

    #[test]
    fn the_roll_stays_put_while_the_blend_moves() {
        // A town's customs only change when its blend moves past its roll:
        // the same counts always give the same customs.
        let a = Blend::of([60, 20, 10, 10], 9, 4).choose(9, 4);
        let b = Blend::of([60, 20, 10, 10], 9, 4).choose(9, 4);
        assert_eq!(a, b);
        // Flooded with a different people, at least something changes.
        let c = Blend::of([60, 900, 10, 10], 9, 4).choose(9, 4);
        assert_ne!(a, c);
    }

    #[test]
    fn hosts_cook_for_their_lodgers() {
        // Ṭaḍoro add nothing to how a town cooks.
        let a = Blend::of([50, 10, 0, 0], 1, 1);
        let b = Blend::of([50, 10, 0, 300], 1, 1);
        for o in 0..3 {
            assert!((a.cooking[o] - b.cooking[o]).abs() < 1e-5);
        }
    }
}
