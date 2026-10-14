//! The crowd filler (authored people, stage 3; `docs/npc-tool-prompt.md`):
//! Laz sets how many of each role live in a town (`Roster::crowd_rows`),
//! and this makes them, person by person, with small differences so they
//! aren't clones: a name in their people's way, a household, one or two
//! traits shifted from their people's average, a wealth step up or down
//! from the role's usual, a clothing variant, and one personal detail from
//! `data/lines/details.txt` their talk can mention.
//!
//! Everything is keyed by the town's key, the row and the number in it
//! (`Rng::from_keys`): the same rows always give the same people, and the
//! world's seed plays no part, so a roster written for one world is the
//! same in every world. The crowd is **stored** in the roster file
//! (`headless crowd <town key>`), never re-rolled at world creation, so
//! Laz can edit any one of them by hand.

use super::jobs::{Job, ALL_JOBS};
use super::race::{Race, ALL_RACES, TRAIT_SPREAD};
use super::rng::{self, Rng};
use super::roster::{CrowdPerson, CrowdRow, Mix, Roster, Sex, Traits, Wealth};
use crate::names::people::{generate_person, Context, Gender};
use crate::names::Tongue;

/// The founders' share of a town's crowd, against the other three peoples
/// (as `worldgen::affinity` tilts the seeded towns).
pub const FOUNDERS_SHARE: f32 = 2.5;
/// How far a shifted trait moves from the people's average, at least and at most.
const SHIFT: (f32, f32) = (0.18, 0.32);
/// The odds a crowd member's wealth steps away from the role's usual.
const WEALTH_STEP: f32 = 0.3;
/// Households run from one to this many.
const HOUSEHOLD_MOST: usize = 5;
/// The odds a household's later members share its first member's people.
const HOUSEHOLD_SAME_PEOPLE: f32 = 0.85;
/// How many clothing variants a people has (`claude/clothing.md`; the window
/// picks the look by this number).
pub const APPEARANCES: u8 = 4;

/// The founders' mix for a town.
pub fn founders_mix(founders: Race) -> Mix {
    let share = |r: Race| if r == founders { FOUNDERS_SHARE } else { 1.0 };
    let total = ALL_RACES.iter().map(|&r| share(r)).sum::<f32>();
    Mix { roduro: share(Race::Roduro) / total, qotiro: share(Race::Qotiro) / total, horaro: share(Race::Horaro) / total, tadoro: share(Race::Tadoro) / total }
}

fn weights(m: &Mix) -> [f32; 4] {
    [m.roduro.max(0.0), m.qotiro.max(0.0), m.horaro.max(0.0), m.tadoro.max(0.0)]
}

/// A town key as a number to key rolls with.
pub fn key_hash(town: &str) -> u64 {
    // FNV-1a, so the key's letters and nothing else decide the rolls.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in town.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// A role's usual wealth: the step its people start from.
pub fn usual_wealth(role: Job) -> Wealth {
    use Job as J;
    match role {
        J::Merchant | J::Innkeeper | J::Exchanger | J::Official | J::Caravaner => Wealth::Rich,
        J::Priest | J::Healer | J::Teacher | J::Arbiter | J::Scribe | J::Smith | J::Armourer | J::Alchemist | J::Boatwright | J::Carpenter | J::Mason => Wealth::Comfortable,
        J::Labourer | J::Runner | J::KelpGatherer | J::CharcoalBurner | J::Tanner | J::Forager => Wealth::Poor,
        J::Drifter | J::None => Wealth::Destitute,
        _ => Wealth::Comfortable,
    }
}

fn step(w: Wealth, up: bool) -> Wealth {
    use Wealth as W;
    match (w, up) {
        (W::Destitute, true) => W::Poor,
        (W::Poor, true) => W::Comfortable,
        (W::Comfortable, true) => W::Rich,
        (W::Rich, true) | (W::VeryRich, true) => W::VeryRich,
        (W::VeryRich, false) => W::Rich,
        (W::Rich, false) => W::Comfortable,
        (W::Comfortable, false) => W::Poor,
        (W::Poor, false) | (W::Destitute, false) => W::Destitute,
    }
}

/// One personal detail: its words and who it fits.
#[derive(Clone, Debug, PartialEq)]
pub struct Detail {
    pub text: String,
    /// `people=` tags it wants (empty: anyone).
    pub peoples: Vec<Race>,
    /// `job=` tags it wants (empty: any trade).
    pub jobs: Vec<Job>,
}

const DETAILS: &str = include_str!("../../data/lines/details.txt");

/// The details list (`data/lines/details.txt`: `detail | tags | text`).
pub fn details() -> &'static [Detail] {
    static D: std::sync::OnceLock<Vec<Detail>> = std::sync::OnceLock::new();
    D.get_or_init(|| parse_details(DETAILS))
}

pub fn parse_details(text: &str) -> Vec<Detail> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, '|').map(str::trim).collect();
        if parts.len() < 3 || parts[0] != "detail" {
            continue;
        }
        let mut d = Detail { text: parts[2].to_string(), peoples: Vec::new(), jobs: Vec::new() };
        for tag in parts[1].split(',').map(str::trim).filter(|t| !t.is_empty()) {
            if let Some(p) = tag.strip_prefix("people=") {
                if let Some(r) = ALL_RACES.iter().find(|r| r.name().eq_ignore_ascii_case(p) || (p.eq_ignore_ascii_case("tadoro") && **r == Race::Tadoro)) {
                    d.peoples.push(*r);
                }
            } else if let Some(j) = tag.strip_prefix("job=") {
                if let Some(job) = ALL_JOBS.iter().find(|x| x.name().eq_ignore_ascii_case(j)) {
                    d.jobs.push(*job);
                }
            }
        }
        out.push(d);
    }
    out
}

fn detail_for(rng: &mut Rng, people: Race, job: Job) -> String {
    let fits: Vec<&Detail> = details().iter().filter(|d| (d.peoples.is_empty() || d.peoples.contains(&people)) && (d.jobs.is_empty() || d.jobs.contains(&job))).collect();
    if fits.is_empty() {
        return String::new();
    }
    // The particular ones (with a tag) a little likelier than the plain.
    let w: Vec<f32> = fits.iter().map(|d| if d.peoples.is_empty() && d.jobs.is_empty() { 1.0 } else { 2.0 }).collect();
    fits[rng.weighted(&w).unwrap_or(0)].text.clone()
}

/// The crowd a town's rows make. `founders` picks the mix of a row with none.
pub fn fill_crowd(town: &str, founders: Race, rows: &[CrowdRow]) -> Vec<CrowdPerson> {
    let th = key_hash(town);
    let base = founders_mix(founders);
    let mut out = Vec::new();
    let mut household: u32 = 0;
    for (r, row) in rows.iter().enumerate() {
        let mix = row.mix.unwrap_or(base);
        let w = weights(&mix);
        // Households run through a row: a few people at a time, mostly of
        // one people.
        let mut left_in_household = 0usize;
        let mut household_people = founders;
        for n in 0..row.count as usize {
            let mut rng = Rng::from_keys(&[th, r as u64, n as u64, 0x4352_4F57]);
            if left_in_household == 0 {
                household += 1;
                left_in_household = 1 + rng.below(HOUSEHOLD_MOST);
                household_people = ALL_RACES[rng.weighted(&w).unwrap_or(0)];
            }
            left_in_household -= 1;
            let people = if rng.chance(HOUSEHOLD_SAME_PEOPLE) { household_people } else { ALL_RACES[rng.weighted(&w).unwrap_or(0)] };
            let sex = if rng.chance(0.5) { Sex::Man } else { Sex::Woman };
            let name_seed = rng::key(&[th, r as u64, n as u64, 0x4E41_4D45]);
            let ctx = Context { lineage: None, home: Some(th), job: Some(row.role.name().to_string()), birthplace: Some(th), turns: 0 };
            let gender = match sex {
                Sex::Man => Gender::Male,
                Sex::Woman => Gender::Female,
            };
            let name = generate_person(Tongue::from(people), gender, name_seed, &ctx).full();
            // One or two traits shifted from the people's average.
            let means = people.trait_means();
            let mut traits = Traits::default();
            let shifts = 1 + rng.below(2);
            let mut picked: Vec<usize> = Vec::new();
            while picked.len() < shifts {
                let k = rng.below(6);
                if !picked.contains(&k) {
                    picked.push(k);
                }
            }
            for k in picked {
                let up = rng.chance(0.5);
                let by = rng.range(SHIFT.0, SHIFT.1) * if up { 1.0 } else { -1.0 };
                let v = |mean: f32| Some((mean + by).clamp(0.05, 0.95));
                match k {
                    0 => traits.wanderlust = v(means.wanderlust),
                    1 => traits.boldness = v(means.boldness),
                    2 => traits.sociability = v(means.sociability),
                    3 => traits.patience = v(means.patience),
                    4 => traits.honour = v(super::culture::profile(people).honour),
                    _ => traits.asks = v(super::culture::profile(people).asks),
                }
            }
            let mut wealth = usual_wealth(row.role);
            if rng.chance(WEALTH_STEP) {
                wealth = step(wealth, rng.chance(0.5));
            }
            let appearance = rng.below(APPEARANCES as usize) as u8;
            let detail = detail_for(&mut rng, people, row.role);
            out.push(CrowdPerson { key: format!("{town}.{r}.{n}"), name, people, sex, profession: row.role, household, home: None, traits, wealth, appearance, detail });
        }
    }
    let _ = TRAIT_SPREAD;
    out
}

impl Roster {
    /// Fill the roster's crowd from its rows. Refuses (Err) when a crowd
    /// is already stored unless `replace`: the stored people may have been
    /// edited by hand.
    pub fn fill_crowd(&mut self, founders: Race, replace: bool) -> Result<usize, String> {
        if !self.crowd.is_empty() && !replace {
            return Err(format!("{}: the crowd is already made ({} people); say replace to make it again", self.town, self.crowd.len()));
        }
        self.crowd = fill_crowd(&self.town, founders, &self.crowd_rows);
        Ok(self.crowd.len())
    }
}
