//! What people wear, as their figure shows it (Laz, 11 Oct; drawing only).
//!
//! A figure is made of parts that clothes and armour are laid on: the lower
//! half (legs), the upper half (body), the head, the shoulders (shown only
//! with shoulder pieces worn) and the back (a pack). Each part's look comes
//! from what's worn in its slot (`Gear::in_slot`), coloured by the item's
//! main material; with nothing worn there, from the person's work clothes
//! (by their job: a guard's red, an official's blue, a priest's white),
//! else a dye of their own (seeded). Guards, officials and priests also wear
//! a band across the chest in their colour, so they can be told apart even
//! in armour. No rule of the world reads any of this.

use gahturiyu_sim::sim::{
    items::{self, info, item, Kind, Slot},
    jobs::Job,
    materials::Material,
    person::PersonId,
    World,
};

use super::palette::Rgb;

/// What a head wears.
#[derive(Clone, Copy, PartialEq)]
pub enum Headwear {
    /// A close cap over the crown.
    Cap,
    /// A helm down over the brow and the back of the head.
    Helm,
}

/// One person's outfit, part by part.
#[derive(Clone, Copy)]
pub struct Outfit {
    pub upper: Rgb,
    pub lower: Rgb,
    pub head: Option<(Headwear, Rgb)>,
    pub shoulders: Option<Rgb>,
    /// A pack on the back: its colour and how big (0..1).
    pub back: Option<(Rgb, f32)>,
    /// A band across the chest (guards, officials, priests).
    pub band: Option<Rgb>,
}

/// The colour of a material as worn.
pub(super) fn material_colour(m: Material) -> Option<Rgb> {
    Some(match m {
        Material::PlainIron | Material::Forgeiron => [0.48, 0.50, 0.53],
        Material::Bronze => [0.62, 0.45, 0.26],
        Material::Gold => [0.86, 0.68, 0.30],
        Material::Leather => [0.42, 0.28, 0.17],
        Material::Fishskin => [0.45, 0.52, 0.50],
        Material::Wood => [0.45, 0.33, 0.21],
        Material::Ringstone | Material::Slatewing => [0.40, 0.40, 0.42],
        Material::Hearthclay => [0.58, 0.36, 0.24],
        Material::Sandglass | Material::Edgeglass => [0.70, 0.62, 0.45],
        Material::Nacre => [0.82, 0.80, 0.76],
        Material::Seareed => [0.52, 0.55, 0.36],
        Material::Tentsilk => [0.72, 0.70, 0.62],
        // Cloth takes the wearer's dye.
        Material::Cloth | Material::None => return None,
    })
}

/// The colour of a worn item, or None if it's plain cloth (dyed by its wearer).
fn worn_colour(id: items::ItemId) -> Option<Rgb> {
    let key = item(info(id).form).key;
    // Some forms have a look of their own whatever they're made of.
    match key {
        "scale_hauberk" | "scale_greaves" => return Some(material_colour(info(id).main).unwrap_or([0.50, 0.52, 0.55])),
        "plate_cuirass" => return Some([0.74, 0.62, 0.42]),
        "hide_coat" | "hide_leggings" => return Some([0.48, 0.36, 0.24]),
        "padded_jacket" => return Some([0.62, 0.54, 0.38]),
        "wraps" => return Some([0.78, 0.72, 0.62]),
        _ => {}
    }
    material_colour(info(id).main)
}

/// Work clothes by job, and the band some jobs wear.
fn job_colours(job: Job) -> Option<(Rgb, Option<Rgb>)> {
    Some(match job {
        Job::Guard => ([0.50, 0.20, 0.18], Some([0.80, 0.16, 0.14])),
        Job::Official | Job::Arbiter => ([0.20, 0.26, 0.45], Some([0.82, 0.70, 0.32])),
        Job::Priest => ([0.85, 0.82, 0.74], Some([0.86, 0.68, 0.30])),
        Job::Healer => ([0.55, 0.66, 0.52], None),
        Job::Merchant | Job::Exchanger | Job::Caravaner => ([0.70, 0.52, 0.22], None),
        Job::Innkeeper | Job::Cook => ([0.72, 0.66, 0.54], None),
        Job::Smith | Job::Armourer | Job::Tanner | Job::Leatherworker => ([0.32, 0.24, 0.18], None),
        Job::Farmer | Job::Forager | Job::Woodcutter | Job::Labourer => ([0.55, 0.46, 0.32], None),
        Job::Fisher | Job::KelpGatherer | Job::Boatwright => ([0.36, 0.46, 0.50], None),
        Job::Scribe | Job::Teacher | Job::Alchemist => ([0.38, 0.30, 0.42], None),
        _ => return None,
    })
}

/// A dye of their own, from a muted set (their seed picks).
fn own_dye(seed: u64) -> Rgb {
    const DYES: [Rgb; 8] = [[0.46, 0.38, 0.28], [0.36, 0.40, 0.30], [0.44, 0.30, 0.26], [0.32, 0.36, 0.44], [0.52, 0.46, 0.36], [0.40, 0.32, 0.38], [0.30, 0.38, 0.38], [0.56, 0.42, 0.30]];
    DYES[(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 61) as usize]
}

fn darker(c: Rgb, k: f32) -> Rgb {
    [c[0] * k, c[1] * k, c[2] * k]
}

pub fn outfit(w: &World, pid: PersonId) -> Outfit {
    let p = &w.people[pid as usize];
    let job = if p.in_squad || p.bandit { Job::None } else { w.society.lives.get(pid as usize).map(|l| l.job).unwrap_or(Job::None) };
    let (work, band) = job_colours(job).map(|(c, b)| (c, b)).unwrap_or((own_dye(p.seed), None));
    // Bandits: rough, dark cloth.
    let cloth = if p.bandit { darker(own_dye(p.seed), 0.7) } else { work };
    let gear = p.detail.as_ref().map(|d| &d.gear);
    let slot = |s: Slot| gear.and_then(|g| g.in_slot(s));
    let upper = slot(Slot::BodyOver).or_else(|| slot(Slot::BodyUnder)).and_then(worn_colour).unwrap_or(cloth);
    let lower = slot(Slot::LegsOver).or_else(|| slot(Slot::LegsUnder)).and_then(worn_colour).unwrap_or(darker(cloth, 0.72));
    let head = slot(Slot::Head).map(|id| {
        let key = item(info(id).form).key;
        let col = worn_colour(id).unwrap_or(darker(cloth, 0.9));
        (if key.contains("helm") { Headwear::Helm } else { Headwear::Cap }, col)
    });
    // Shoulder pieces: none are made yet, so the part waits for them.
    let shoulders = slot(Slot::BodyOver).filter(|&id| item(info(id).form).key.contains("pauldron")).and_then(worn_colour);
    let back = slot(Slot::Back).map(|id| {
        let size = match item(id).kind {
            Kind::Pack(kg) => (kg / 45.0).clamp(0.4, 1.0),
            _ => 0.6,
        };
        (worn_colour(id).unwrap_or([0.42, 0.30, 0.18]), size)
    });
    Outfit { upper, lower, head, shoulders, back, band }
}
