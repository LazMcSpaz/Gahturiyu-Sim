//! Names in the game: a thin layer onto the naming system (`crate::names`,
//! explained in `docs/naming.md`).
//!
//! - A person's **given name** is a native word with a meaning, made from
//!   their people and their seed alone, so the same unborn person always
//!   gets the same name, whenever and wherever the game first needs it.
//! - A person's **whole name** (the byname, its native form, what the name
//!   hints at) is worked out from where they live and what they do, when it
//!   is asked for: `who`.
//! - A **town** is named for the land it stands on. The name kept is the
//!   English one ("Stonebrow"); its own name in its founders' tongue and
//!   what the other peoples call it are worked out again when asked for:
//!   `town`.
//!
//! Nothing here is stored beyond the two strings the game already kept
//! (a person's name, a town's name).

use super::geo::{self, V2};
use super::person::PersonId;
use super::race::Race;
use super::rng;
use super::settlement::{Settlement, SettlementId};
use super::terrain::{self, Terrain};
use super::world::World;
use crate::names::{self, Context, Feature, Gender, PersonName, PlaceName, Tongue};

/// Whether someone's name is a man's, a woman's or one for either. (The
/// game has no other notion of this yet: it is read off the seed.)
pub fn gender(seed: u64) -> Gender {
    match rng::key(&[seed, 0x4745_4E44]) % 25 {
        0..=11 => Gender::Female,
        12..=23 => Gender::Male,
        _ => Gender::Either,
    }
}

/// A person's given name, off the hand-kept lists (`assets/lang/names/`).
pub fn person_name(race: Race, seed: u64) -> String {
    names::listed_given(race.into(), gender(seed), seed).name
}

/// What is known of a person that a byname can be made from.
fn context(w: &World, id: PersonId) -> Context {
    let p = &w.people[id as usize];
    // A household is one home and one family.
    let household = p.home.map(|h| rng::key(&[w.seed, h as u64, p.dwelling.map(|d| d as u64 + 1).unwrap_or(0), 0x484F_4D45]));
    let job = w.life(id).job;
    Context {
        lineage: household,
        home: household,
        job: if job == super::jobs::Job::None { None } else { Some(job.name().to_string()) },
        birthplace: p.home.map(|h| rng::key(&[w.seed, h as u64, 0x4249_5254])),
        // Nothing counts deeds or teachers yet: a third are still unproven.
        turns: (rng::key(&[p.seed, 0x5455_524E]) % 3) as u32,
    }
}

/// A person's whole name: the given name the game shows for them, their
/// byname in English and in their own tongue, a job byname where they have
/// one, what the given name means and what the whole hints at.
pub fn who(w: &World, id: PersonId) -> PersonName {
    let p = &w.people[id as usize];
    let tongue: Tongue = p.race.into();
    let mut given = names::listed_given(tongue, gender(p.seed), p.seed);
    // A name the game already holds wins (an old save, a renamed squad member).
    if let Some(kept) = p.name().filter(|n| *n != given.name) {
        given = names::Given { name: kept.to_string(), meaning: String::new(), made: String::new(), listed: false };
    }
    names::person_with_given(tongue, gender(p.seed), p.seed, &context(w, id), given)
}

// ---- Places -----------------------------------------------------------------------

/// Height as the seed made it (hand edits to the map never rename a town).
fn rise(t: &Terrain, p: V2) -> f32 {
    let e = terrain::CELL;
    let dx = (t.base_height(p.add(V2::new(e, 0.0))) - t.base_height(p.sub(V2::new(e, 0.0)))) / (2.0 * e);
    let dy = (t.base_height(p.add(V2::new(0.0, e))) - t.base_height(p.sub(V2::new(0.0, e)))) / (2.0 * e);
    (dx * dx + dy * dy).sqrt()
}

/// What the land is like at a spot, strongest first: what a place there
/// would be named for. Read off the land as the seed made it.
pub fn site_features(t: &Terrain, pos: V2, coastal: bool) -> Vec<Feature> {
    let seed = t.seed();
    let h = t.base_height(pos);
    let slope = rise(t, pos);
    let mut out = Vec::new();
    if coastal {
        // The shape of the shore: where it bends inland the sea makes a cove;
        // where it stands out, a headland.
        let d = 350.0;
        let bend = geo::coast_x(pos.y) - (geo::coast_x(pos.y - d) + geo::coast_x(pos.y + d)) / 2.0;
        if terrain::cliff_mask(seed, pos.y) > 0.5 {
            out.push(Feature::Cliff);
        }
        out.push(if bend > 45.0 {
            Feature::Bay
        } else if bend > 15.0 {
            Feature::Cove
        } else if bend < -15.0 {
            Feature::Headland
        } else {
            Feature::Shore
        });
    }
    if t.mountains(pos) > 0.5 {
        out.push(Feature::Mountain);
    }
    if t.plateau(pos) > 0.5 {
        out.push(Feature::Plateau);
    }
    // Higher or lower than the ground 300 m round about.
    let mut round = 0.0;
    for k in 0..8 {
        let a = k as f32 / 8.0 * std::f32::consts::TAU;
        round += t.base_height(pos.add(V2::new(a.cos(), a.sin()).scale(300.0)));
    }
    let sunk = round / 8.0 - h;
    if sunk > 25.0 {
        out.push(Feature::Valley);
    } else if sunk > 10.0 {
        out.push(Feature::Hollow);
    } else if sunk < -10.0 {
        out.push(Feature::Hill);
    }
    if slope > 0.12 {
        out.push(Feature::Hillside);
    }
    if super::animals::wooded(seed, pos) > 0.5 {
        out.push(Feature::Wood);
    }
    // Wet ground: the lowest, flattest shore.
    if geo::inland(pos) < 700.0 && h < 2.5 && slope < 0.03 && terrain::cliff_mask(seed, pos.y) < 0.3 {
        out.push(Feature::Marsh);
    }
    if t.plateau(pos) <= 0.5 && terrain::noise(seed ^ 0x5C2B, pos.x, pos.y, 450.0) > 0.7 {
        out.push(Feature::Heath);
    }
    if out.is_empty() {
        out.push(Feature::Plain);
    }
    out
}

const TOWN: u64 = 0x544F_574E;

/// The whole names of a run of towns, in the order they were founded: each
/// is named knowing the ones before it, so no two share a name in English
/// or in any tongue. Stops at the first town whose kept name is not the one
/// this system gives it (an older save, a town renamed by hand).
fn town_names(world_seed: u64, t: &Terrain, towns: &[Settlement]) -> Vec<PlaceName> {
    let mut out: Vec<PlaceName> = Vec::with_capacity(towns.len() + 1);
    for s in towns {
        let p = names::generate_place_with(&site_features(t, s.pos, s.coastal), None, true, s.founders.into(), rng::key(&[world_seed, s.id as u64, TOWN]), &out);
        if p.english != s.name {
            break;
        }
        out.push(p);
    }
    out
}

/// The name of a new town: the English name the game shows. `earlier` is
/// the towns already founded, in order; the new one is the next.
pub fn town_name(founders: Race, world_seed: u64, t: &Terrain, pos: V2, coastal: bool, earlier: &[Settlement]) -> String {
    let taken = town_names(world_seed, t, earlier);
    names::generate_place_with(&site_features(t, pos, coastal), None, true, founders.into(), rng::key(&[world_seed, earlier.len() as u64, TOWN]), &taken).english
}

/// A town's whole name: its own name in its founders' tongue, what each of
/// the other peoples calls it, what it means and how to say it. `None` if
/// the town's name is not one this system gave it (an older save, or a town
/// renamed by hand): then there is only the name it has.
pub fn town(w: &World, id: SettlementId) -> Option<PlaceName> {
    town_names(w.seed, &w.terrain, w.settlements.get(..=id as usize)?).into_iter().nth(id as usize)
}

/// Every town's whole name at once (cheaper than asking one by one).
pub fn towns(w: &World) -> Vec<PlaceName> {
    town_names(w.seed, &w.terrain, &w.settlements)
}
