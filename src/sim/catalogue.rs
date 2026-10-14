//! The catalogue: every thing the place builder can put down (Laz's page,
//! `docs/place-builder.md`), with the game's own sizes, so the page and the
//! sim agree on every shape. Written to `assets/world/catalogue.json` by
//! `headless catalogue`; `tests/catalogue.rs` fails if the file is stale.
//!
//! One entry per thing, in groups: `building` (a `layout::VARIANTS` entry
//! at its usual size), `base` (a `base::BUILDINGS` entry), `workplace` (a
//! `jobs::PlaceKind` footprint), `pen` (a kept species' pen at a usual
//! number of beasts), `prop` (things that stand on their own), `foliage`
//! (trees, bushes, rocks as drawn) and `ground` (the paint classes). A thing
//! the page can show but the game can't stand up yet says `missing: true`.
//!
//! Sizes are metres at scale 1: `depth` along the facing (the door end),
//! `width` across; `radius` for round things drawn as circles.

use serde::Serialize;

use super::animals::Sp;
use super::base::BUILDINGS;
use super::crafting::STATIONS;
use super::jobs::{works_at, PlaceKind, Shelf, ALL_JOBS};
use super::layout::{self, Furn, Shape, Use, VARIANTS};
use super::settlement::BuildingKind;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Entry {
    pub key: String,
    pub name: String,
    pub group: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub people: Option<&'static str>,
    pub kind: String,
    pub shape: &'static str,
    pub depth: f32,
    pub width: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_range: Option<[f32; 2]>,
    pub storeys: u8,
    pub beds: u16,
    /// Bedrolls the game lays on the floor when more live there than the
    /// beds hold (a building's crowding room; 0 for anything else).
    #[serde(skip_serializing_if = "is_zero")]
    pub extra_beds: u16,
    pub jobs: Vec<String>,
    pub stations: Vec<String>,
    pub scalable: bool,
    pub rotatable: bool,
    pub glyph: String,
    pub door: f32,
    /// A pen's usual number of beasts (its size is worked out from it).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<f32>,
    /// A ground class's letter in a place file's `ground` raster.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter: Option<char>,
    pub blurb: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub missing: bool,
}

/// Rounded to the centimetre, so the file reads cleanly.
fn cm(x: f32) -> f32 {
    (x * 100.0).round() / 100.0
}

fn entry(key: &str, name: &str, group: &'static str, kind: &str) -> Entry {
    Entry {
        key: key.to_string(),
        name: name.to_string(),
        group,
        people: None,
        kind: kind.to_string(),
        shape: "rect",
        depth: 0.0,
        width: 0.0,
        radius: None,
        size_range: None,
        storeys: 1,
        beds: 0,
        extra_beds: 0,
        jobs: Vec::new(),
        stations: Vec::new(),
        scalable: false,
        rotatable: true,
        glyph: String::new(),
        door: 0.0,
        limit: None,
        letter: None,
        blurb: String::new(),
        missing: false,
    }
}

fn snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// The stations a building's furniture makes.
fn stations_of(furniture: &[layout::Piece]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for st in STATIONS {
        if let Some(f) = st.furniture() {
            if furniture.iter().any(|p| p.what == f) && !out.contains(&format!("{st:?}")) {
                out.push(format!("{st:?}"));
            }
        }
    }
    out
}

/// Every workplace kind, with the key the page uses.
pub const WORKPLACES: &[(PlaceKind, &str)] = &[
    (PlaceKind::Fields, "fields"),
    (PlaceKind::Wilds, "wilds"),
    (PlaceKind::Woodlot, "woodlot"),
    (PlaceKind::Quarry, "quarry"),
    (PlaceKind::CharcoalPit, "charcoal_pit"),
    (PlaceKind::Dock, "dock"),
    (PlaceKind::DivePlatform, "dive_platform"),
    (PlaceKind::KelpBeds, "kelp_beds"),
    (PlaceKind::Boatyard, "boatyard"),
    (PlaceKind::Kitchen, "kitchen"),
    (PlaceKind::MessHall, "mess_hall"),
    (PlaceKind::Deck, "deck"),
    (PlaceKind::Market, "market"),
    (PlaceKind::Shop(Shelf::Goods), "shop"),
    (PlaceKind::Shop(Shelf::Food), "shop_food"),
    (PlaceKind::Shop(Shelf::Materials), "shop_materials"),
    (PlaceKind::Inn, "inn"),
    (PlaceKind::Shrine, "shrine"),
    (PlaceKind::GuardPost, "guard_post"),
    (PlaceKind::Hall, "hall"),
    (PlaceKind::HealingHouse, "healing_house"),
    (PlaceKind::HealersHouse, "healers_house"),
    (PlaceKind::TeachingHouse, "teaching_house"),
    (PlaceKind::ExchangeHouse, "exchange_house"),
    (PlaceKind::LettersHouse, "letters_house"),
    (PlaceKind::TendersYard, "tenders_yard"),
    (PlaceKind::Workyard, "workyard"),
    (PlaceKind::Forge, "smithy"),
    (PlaceKind::Bench, "bench"),
    (PlaceKind::Desk, "desk"),
    (PlaceKind::AlchemyTable, "alchemy_table"),
    (PlaceKind::WeaversShed, "weavers_shed"),
    (PlaceKind::Workshop, "workshop"),
];

/// A workplace's key, by kind.
pub fn workplace_key(k: PlaceKind) -> Option<&'static str> {
    WORKPLACES.iter().find(|w| w.0 == k).map(|w| w.1)
}

/// A workplace kind, by key.
pub fn workplace_of(key: &str) -> Option<PlaceKind> {
    WORKPLACES.iter().find(|w| w.1 == key).map(|w| w.0)
}

/// Out-of-doors workplaces: drawn as open ground with their gear, not a room.
fn open_ground(k: PlaceKind) -> bool {
    matches!(
        k,
        PlaceKind::Fields | PlaceKind::Wilds | PlaceKind::Woodlot | PlaceKind::Quarry | PlaceKind::CharcoalPit | PlaceKind::KelpBeds | PlaceKind::Market | PlaceKind::Workyard | PlaceKind::TendersYard | PlaceKind::Dock | PlaceKind::DivePlatform | PlaceKind::Deck | PlaceKind::Boatyard
    )
}

/// The kept species, with a usual number of beasts in a town's pen (the
/// default livestock's rule, `animals/place.rs`, for a town of about two
/// hundred).
pub const PENS: &[(Sp, f32)] = &[(Sp::Turiyu, 14.0), (Sp::Shellhen, 30.0), (Sp::Mossback, 8.0), (Sp::Plodder, 6.0), (Sp::Raftback, 2.0)];

/// A pen's radius for so many beasts (`animals/place.rs`, `add_pen`).
pub fn pen_radius(sp: Sp, limit: f32) -> f32 {
    match sp {
        Sp::Raftback => 14.0,
        Sp::Shellhen => 5.0,
        _ => (4.0 + limit.sqrt() * sp.def().looks.len * 1.6).min(26.0),
    }
}

/// The ground paint classes and their letters in a place file.
pub const GROUND: &[(char, &str, &str)] = &[
    ('g', "grass", "Grass"),
    ('s', "scrub", "Scrub"),
    ('d', "dirt", "Dirt"),
    ('n', "sand", "Sand"),
    ('m', "mud", "Mud"),
    ('v', "gravel", "Gravel"),
    ('r', "rock", "Rock"),
    ('w', "snow", "Snow"),
    ('~', "water", "Water"),
];

/// The whole catalogue, in a fixed order.
pub fn catalogue() -> Vec<Entry> {
    let mut out = Vec::new();

    // Buildings: every variant at its usual size.
    for v in VARIANTS {
        let (size, _) = layout::example_of(v);
        let mut e = entry(v.key, v.name, "building", &format!("{:?}", v.kind));
        e.people = Some(match v.kind {
            BuildingKind::RoduroHome => "roduro",
            BuildingKind::HoraroStilt => "horaro",
            _ => "qotiro",
        });
        e.shape = if v.shape == Shape::Round { "round" } else { "rect" };
        e.depth = cm(2.0 * v.half.0 * size);
        e.width = cm(2.0 * v.half.1 * size);
        e.size_range = Some([cm(v.sizes.0), cm(v.sizes.1)]);
        e.storeys = v.storeys;
        e.beds = v.furniture.iter().filter(|p| matches!(p.what, Furn::Bed | Furn::Bedroll)).count() as u16;
        e.extra_beds = if v.use_ == Use::Quarters { layout::MAX_EXTRA_QUARTERS } else { layout::MAX_EXTRA } as u16;
        e.jobs = ALL_JOBS.iter().filter(|&&j| layout::keeps(v, j)).map(|j| format!("{j:?}")).collect();
        e.stations = stations_of(v.furniture);
        e.scalable = true;
        e.glyph = snake(&format!("{:?}", v.kind));
        e.door = cm(v.door);
        let what = match v.use_ {
            Use::Home => "a home",
            Use::Workshop => "a workshop with living space",
            Use::Shop => "a shop with living space",
            Use::Quarters => "quarters for several households",
            Use::Hall => "a hall where people eat or gather",
            Use::Shrine => "a shrine",
        };
        let beds = if e.beds == 1 { "1 bed".to_string() } else { format!("{} beds", e.beds) };
        e.blurb = if e.jobs.is_empty() { format!("{}, {beds}.", cap(what)) } else { format!("{}, {beds}; kept by a {}.", cap(what), e.jobs.join(" or ").to_lowercase()) };
        out.push(e);
    }

    // The squad's base buildings.
    for d in BUILDINGS {
        let mut e = entry(d.key, d.name, "base", &format!("{:?}", d.kind));
        e.depth = cm(d.d);
        e.width = cm(d.w);
        e.storeys = 1;
        e.beds = d.beds as u16;
        e.stations = d.station.map(|s| vec![format!("{s:?}")]).unwrap_or_default();
        e.glyph = snake(&format!("{:?}", d.kind));
        e.blurb = d.does.to_string();
        out.push(e);
    }

    // Workplaces: a square of the kind's size.
    for &(k, key) in WORKPLACES {
        let mut e = entry(key, k.name(), "workplace", &format!("{k:?}"));
        e.depth = cm(k.size());
        e.width = cm(k.size());
        e.jobs = ALL_JOBS.iter().filter(|&&j| works_at(j, k, false) || works_at(j, k, true)).map(|j| format!("{j:?}")).collect();
        e.stations = k.stations().iter().map(|s| format!("{s:?}")).collect();
        e.glyph = if open_ground(k) { "open_ground".into() } else { "room".into() };
        e.blurb = if e.jobs.is_empty() { k.name().to_string() } else { format!("{}: {}.", k.name(), e.jobs.join(", ").to_lowercase()) };
        out.push(e);
    }

    // Pens, by kept species.
    for &(sp, limit) in PENS {
        let def = sp.def();
        let r = pen_radius(sp, limit);
        let mut e = entry(&format!("pen_{}", def.key), &format!("{} pen", def.name), "pen", def.key);
        e.shape = "round";
        e.radius = Some(cm(r));
        e.depth = cm(2.0 * r);
        e.width = cm(2.0 * r);
        e.limit = Some(limit);
        e.scalable = true;
        e.rotatable = false;
        e.glyph = if sp == Sp::Raftback { "raft_mooring".into() } else { "pen".into() };
        e.blurb = format!("Room for about {limit} {}; scale it for more or fewer.", def.name.to_lowercase());
        out.push(e);
    }

    // Props: what stands on its own. (Missing: the page can draw it, the
    // game can't stand it up yet.)
    let props: &[(&str, &str, f32, f32, bool, &str)] = &[
        ("rock_boulder", "Boulder", 2.4, 2.0, false, "A rock (the land editor's Boulder)."),
        ("rock_slab", "Slab", 3.0, 2.0, false, "A low flat rock."),
        ("rock_pillar", "Pillar", 1.4, 1.2, false, "A standing rock."),
        ("rock_scree", "Scree", 5.2, 5.2, false, "Loose stones; walkable."),
        ("rock_outcrop", "Outcrop", 3.1, 2.3, false, "A broken rock face."),
        ("tent", "Tent", 2.2, 2.2, false, "A traveller's tent (sleeps three)."),
        ("bedroll", "Bedroll", 1.9, 0.8, false, "One sleeper, out of doors."),
        ("campfire", "Campfire", 1.2, 1.2, false, "A fire to cook at and keep warm by."),
        ("standing_torch", "Standing torch", 0.4, 0.4, false, "Light at night until it burns down."),
        ("timber_pile", "Timber stack", 3.0, 1.6, true, "Only as part of a woodlot so far."),
        ("fence_run", "Fence", 4.0, 0.3, true, "Fences only come with pens, yards and palisades so far."),
        ("town_well", "Well", 2.0, 2.0, true, "A town well: only the squad's base well (`well`) exists so far."),
        ("garden_plot", "Garden plot", 4.0, 3.0, true, "Gardens exist in town life, not as a placed thing yet."),
        ("stall", "Stall", 2.4, 1.2, true, "Only as a market's counters so far."),
        ("cart", "Cart", 3.0, 1.6, true, "No carts yet."),
        ("tanners_vats", "Tanner's vats", 4.0, 3.0, true, "No vats yet: tanning is done in a workshop."),
        ("smokehouse", "Smokehouse", 4.0, 3.0, true, "No smokehouse yet."),
    ];
    for &(key, name, d, w, missing, blurb) in props {
        let mut e = entry(key, name, "prop", key);
        e.depth = d;
        e.width = w;
        e.missing = missing;
        e.glyph = key.to_string();
        e.blurb = blurb.to_string();
        if key.starts_with("rock_") {
            e.shape = "round";
            e.scalable = true;
        }
        out.push(e);
    }

    // Foliage as drawn (`view/foliage.rs`: canopies before the random 0.8-1.25 scale).
    let foliage: &[(&str, &str, f32, bool, &str)] = &[
        ("tree_broad", "Broadleaf tree", 2.6, false, "A broad tree, about nine metres tall."),
        ("tree_conifer", "Conifer", 2.4, false, "A needle tree."),
        ("tree_small", "Young tree", 1.4, true, "The game draws no young trees yet."),
        ("bush", "Bush", 1.0, false, "A low bush."),
        ("rock_small", "Small rock", 0.6, false, "A stone you could sit on."),
        ("rock_large", "Large rock", 1.5, false, "A rock you'd walk round."),
    ];
    for &(key, name, r, missing, blurb) in foliage {
        let mut e = entry(key, name, "foliage", key);
        e.shape = "round";
        e.radius = Some(r);
        e.depth = cm(2.0 * r);
        e.width = cm(2.0 * r);
        e.scalable = true;
        e.rotatable = false;
        e.missing = missing;
        e.glyph = key.to_string();
        e.blurb = blurb.to_string();
        out.push(e);
    }

    // Ground paint.
    for &(letter, key, ground) in GROUND {
        let mut e = entry(key, ground, "ground", ground);
        e.letter = Some(letter);
        e.rotatable = false;
        e.glyph = "ground".into();
        e.blurb = if letter == '~' { "Fresh water: a pond or stream (stops a walker).".into() } else { format!("{ground} underfoot.") };
        out.push(e);
    }
    out
}

fn is_zero(n: &u16) -> bool {
    *n == 0
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// The catalogue as the file holds it.
pub fn catalogue_json() -> String {
    let mut s = serde_json::to_string_pretty(&catalogue()).expect("the catalogue writes");
    s.push('\n');
    s
}

/// An entry by key.
pub fn find(key: &str) -> Option<Entry> {
    catalogue().into_iter().find(|e| e.key == key)
}
