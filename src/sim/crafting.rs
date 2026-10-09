//! Making things, the squad's way: at a station, from what's in the pack.
//!
//! Seven crafts, each a skill with its own station:
//!
//! | Craft | Makes | Station |
//! |---|---|---|
//! | Handcraft | leather, cloth and wood: the shared basics | workbench |
//! | Smithing | ingots, fire-metal weapons, glass, gold | forge |
//! | Armoring | fire-metal armour and shields | forge, armourer's bench |
//! | Weaving | reed, shell, fishskin and tentsilk; sealing with pitch | weaver's frame |
//! | Tending | grown stone (days to months in the bed) | grower's bed |
//! | Inscription | paper, ink, scrolls, manuals | scribe's desk |
//! | Alchemy | potions | alchemy table, or a mortar and pestle anywhere |
//!
//! A squad member can only work a craft they've taken up (from a crafter's
//! lessons or a manual, `making.rs`); practice raises it from there. The
//! same recipes are what the towns' crafters work from. A recipe takes its
//! materials up front and some game time at the station; then a keyed roll
//! against skill and difficulty decides whether it worked (a botched job
//! gives half the materials back), and another sets the grade of a piece.
//! Pieces carry the maker's mark. Stone grows only while its grower comes
//! by the bed: each dawn they're away adds a day.
//!
//! Stations are set up in the towns' workplaces and free to use. Materials
//! come from merchants, and from the land — kelp on the coast, emberroot and
//! salt on the Qotiro plateau, ore and storm glass in the mountains, deadwood
//! anywhere. A picked spot grows back after a day.

use serde::{Deserialize, Serialize};

use super::geo::{self, V2};
use super::items::{self, item, ItemId, Kind};
use super::materials::{Craft, Grade, Material, Mark, Piece, MARK_SKILL};
use super::person::PersonId;
use super::rng::Rng;
use super::stats::Skill;
use super::world::{World, DAY, HOUR};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Station {
    Forge,
    Bench,
    Desk,
    AlchemyTable,
    /// Weaver's frame and sealing pit.
    Loom,
    /// Leather, cloth and wood.
    Workbench,
    /// Where stone is grown.
    GrowerBed,
}

impl Station {
    pub fn name(self) -> &'static str {
        match self {
            Station::Forge => "Forge",
            Station::Bench => "Armourer's bench",
            Station::Desk => "Scribe's desk",
            Station::AlchemyTable => "Alchemy table",
            Station::Loom => "Weaver's frame",
            Station::Workbench => "Workbench",
            Station::GrowerBed => "Grower's bed",
        }
    }
}

pub const STATIONS: [Station; 7] = [Station::Forge, Station::Bench, Station::Desk, Station::AlchemyTable, Station::Loom, Station::Workbench, Station::GrowerBed];

/// How close to a station you must stand to use it, metres.
pub const AT_STATION: f32 = 3.0;
/// A grower must be this near their bed at dawn for the stone to have grown
/// that day (they've been round to tend it).
pub const TENDING_REACH: f32 = 200.0;

pub struct Recipe {
    /// An item key; for a piece (a weapon, armour), its form.
    pub output: &'static str,
    pub makes: u16,
    pub inputs: &'static [(&'static str, u16)],
    pub skill: Skill,
    /// Skill at which it works about four times in five.
    pub difficulty: f32,
    pub station: Station,
    /// Game seconds at the station (for the squad; a town's crafters work
    /// in hours, see `npc_hours`).
    pub time: f64,
    /// For a piece: what it's made of (`None`: the form's usual material).
    pub main: Material,
    pub second: Material,
}

const fn r(output: &'static str, makes: u16, inputs: &'static [(&'static str, u16)], skill: Skill, difficulty: f32, station: Station, time: f64) -> Recipe {
    Recipe { output, makes, inputs, skill, difficulty, station, time, main: M::None, second: M::None }
}

/// A piece in these materials.
const fn p(form: &'static str, main: Material, second: Material, inputs: &'static [(&'static str, u16)], skill: Skill, difficulty: f32, station: Station, time: f64) -> Recipe {
    Recipe { output: form, makes: 1, inputs, skill, difficulty, station, time, main, second }
}

use super::materials::Material as M;
use Skill as K;
use Station as S;

/// Grown things take days at the bed.
const D: f64 = DAY;

/// Every recipe, for your squad and the towns' crafters alike. Grown stone
/// is grown from rock feedstock and ash (and a seed crystal for Edgeglass);
/// its times are growing times.
pub static RECIPES: &[Recipe] = &[
    // Smithing. (The knife stays first: tests look it up by name.)
    p("knife", M::PlainIron, M::None, &[("iron_ingot", 1), ("leather", 1)], K::Smithing, 10.0, S::Forge, 90.0),
    r("iron_ingot", 1, &[("iron_ore", 2), ("charcoal", 2)], K::Smithing, 5.0, S::Forge, 60.0),
    r("bronze_ingot", 1, &[("iron_ore", 1), ("charcoal", 1)], K::Smithing, 5.0, S::Forge, 45.0),
    p("hatchet", M::PlainIron, M::None, &[("iron_ingot", 1), ("timber", 1)], K::Smithing, 10.0, S::Forge, 90.0),
    p("spear", M::Bronze, M::None, &[("bronze_ingot", 1), ("timber", 2)], K::Smithing, 15.0, S::Forge, 100.0),
    p("short_sword", M::Bronze, M::None, &[("bronze_ingot", 2), ("leather", 1)], K::Smithing, 20.0, S::Forge, 120.0),
    p("spear", M::Forgeiron, M::None, &[("iron_ingot", 1), ("timber", 2)], K::Smithing, 20.0, S::Forge, 120.0),
    p("short_sword", M::Forgeiron, M::None, &[("iron_ingot", 2), ("leather", 1)], K::Smithing, 30.0, S::Forge, 150.0),
    p("war_pick", M::Forgeiron, M::None, &[("iron_ingot", 3), ("timber", 1)], K::Smithing, 40.0, S::Forge, 180.0),
    p("longsword", M::Forgeiron, M::None, &[("iron_ingot", 3), ("leather", 1)], K::Smithing, 50.0, S::Forge, 210.0),
    p("crossbow", M::Forgeiron, M::None, &[("iron_ingot", 2), ("timber", 2)], K::Smithing, 45.0, S::Forge, 200.0),
    // Two traditions: an Edgeglass edge (from a Tender) on a Forgeiron spine.
    p("longsword", M::Edgeglass, M::Forgeiron, &[("edgeglass", 1), ("iron_ingot", 2), ("leather", 1)], K::Smithing, 65.0, S::Forge, 240.0),
    r("sandglass_flask", 1, &[("sand", 2), ("charcoal", 1)], K::Smithing, 15.0, S::Forge, 60.0),
    r("gold_ring", 1, &[("gold_nugget", 2)], K::Smithing, 30.0, S::Forge, 90.0),
    // Armouring.
    p("iron_helm", M::Bronze, M::None, &[("bronze_ingot", 2), ("leather", 1)], K::Armoring, 20.0, S::Forge, 120.0),
    p("iron_helm", M::Forgeiron, M::None, &[("iron_ingot", 2), ("leather", 1)], K::Armoring, 35.0, S::Forge, 150.0),
    p("sandstone_lamellar", M::Bronze, M::None, &[("bronze_ingot", 4), ("leather", 2)], K::Armoring, 30.0, S::Forge, 240.0),
    p("scale_hauberk", M::Forgeiron, M::None, &[("iron_ingot", 6), ("leather", 2)], K::Armoring, 55.0, S::Forge, 300.0),
    p("scale_greaves", M::Forgeiron, M::None, &[("iron_ingot", 3), ("leather", 1)], K::Armoring, 40.0, S::Forge, 180.0),
    p("buckler", M::Bronze, M::None, &[("bronze_ingot", 1), ("timber", 2)], K::Armoring, 20.0, S::Bench, 120.0),
    p("kite_shield", M::Forgeiron, M::None, &[("iron_ingot", 3), ("timber", 2)], K::Armoring, 40.0, S::Bench, 200.0),
    // Leather, cloth and wood: the shared basics.
    r("leather", 1, &[("hide", 2)], K::Handcraft, 5.0, S::Workbench, 60.0),
    r("cloth", 1, &[("fibre", 2)], K::Handcraft, 5.0, S::Workbench, 60.0),
    p("leather_cap", M::Leather, M::None, &[("leather", 1)], K::Handcraft, 8.0, S::Workbench, 60.0),
    p("leather_gloves", M::Leather, M::None, &[("leather", 1)], K::Handcraft, 10.0, S::Workbench, 60.0),
    p("boots", M::Leather, M::None, &[("leather", 2)], K::Handcraft, 10.0, S::Workbench, 90.0),
    p("hide_coat", M::Leather, M::None, &[("leather", 3)], K::Handcraft, 20.0, S::Workbench, 120.0),
    p("hide_leggings", M::Leather, M::None, &[("leather", 2)], K::Handcraft, 15.0, S::Workbench, 100.0),
    p("cloth_shirt", M::Cloth, M::None, &[("cloth", 2)], K::Handcraft, 5.0, S::Workbench, 60.0),
    p("trousers", M::Cloth, M::None, &[("cloth", 2)], K::Handcraft, 5.0, S::Workbench, 60.0),
    p("padded_jacket", M::Cloth, M::None, &[("cloth", 4)], K::Handcraft, 15.0, S::Workbench, 120.0),
    p("small_pack", M::Leather, M::None, &[("leather", 2)], K::Handcraft, 10.0, S::Workbench, 90.0),
    p("large_pack", M::Leather, M::None, &[("leather", 4)], K::Handcraft, 25.0, S::Workbench, 150.0),
    p("club", M::Wood, M::None, &[("timber", 1)], K::Handcraft, 3.0, S::Workbench, 40.0),
    p("staff", M::Wood, M::None, &[("timber", 2)], K::Handcraft, 5.0, S::Workbench, 60.0),
    p("spear", M::Wood, M::None, &[("timber", 2)], K::Handcraft, 8.0, S::Workbench, 60.0),
    p("short_bow", M::Wood, M::None, &[("timber", 2), ("fibre", 1)], K::Handcraft, 20.0, S::Workbench, 120.0),
    p("sling", M::Leather, M::None, &[("leather", 1)], K::Handcraft, 5.0, S::Workbench, 40.0),
    p("buckler", M::Wood, M::None, &[("timber", 3)], K::Handcraft, 10.0, S::Workbench, 90.0),
    r("torch", 2, &[("timber", 1), ("fibre", 1)], K::Handcraft, 2.0, S::Workbench, 30.0),
    r("arrows", 10, &[("timber", 1)], K::Handcraft, 10.0, S::Workbench, 60.0),
    r("sling_stones", 10, &[("rock", 1)], K::Handcraft, 2.0, S::Workbench, 30.0),
    // Weaving and sealing: reed, shell, fishskin, tentsilk.
    p("padded_jacket", M::Seareed, M::None, &[("seareed", 4), ("pitch", 1)], K::Weaving, 15.0, S::Loom, 120.0),
    p("small_pack", M::Seareed, M::None, &[("seareed", 3), ("pitch", 1)], K::Weaving, 10.0, S::Loom, 90.0),
    p("net", M::Seareed, M::None, &[("seareed", 4)], K::Weaving, 15.0, S::Loom, 120.0),
    p("harpoon", M::Nacre, M::None, &[("nacre", 2), ("timber", 2)], K::Weaving, 25.0, S::Loom, 150.0),
    p("trident", M::Nacre, M::None, &[("nacre", 3), ("timber", 2)], K::Weaving, 30.0, S::Loom, 150.0),
    p("spear", M::Nacre, M::None, &[("nacre", 2), ("timber", 2)], K::Weaving, 25.0, S::Loom, 120.0),
    p("knife", M::Nacre, M::None, &[("nacre", 1), ("fishskin", 1)], K::Weaving, 20.0, S::Loom, 90.0),
    p("scale_hauberk", M::Nacre, M::Seareed, &[("nacre", 6), ("seareed", 3), ("pitch", 1)], K::Weaving, 45.0, S::Loom, 300.0),
    // Two traditions: Nacre scales on a Slatewing frame.
    p("scale_hauberk", M::Nacre, M::Slatewing, &[("nacre", 6), ("slatewing", 2)], K::Weaving, 60.0, S::Loom, 320.0),
    p("hide_coat", M::Fishskin, M::None, &[("fishskin", 3)], K::Weaving, 20.0, S::Loom, 120.0),
    p("boots", M::Fishskin, M::None, &[("fishskin", 2)], K::Weaving, 12.0, S::Loom, 90.0),
    p("leather_cap", M::Fishskin, M::None, &[("fishskin", 1)], K::Weaving, 8.0, S::Loom, 60.0),
    p("wraps", M::Tentsilk, M::None, &[("tentsilk", 2)], K::Weaving, 15.0, S::Loom, 90.0),
    p("cloth_shirt", M::Tentsilk, M::None, &[("tentsilk", 2)], K::Weaving, 15.0, S::Loom, 90.0),
    r("tent", 1, &[("tentsilk", 4)], K::Weaving, 20.0, S::Loom, 180.0),
    r("pearl_necklace", 1, &[("pearl", 3), ("seareed", 1)], K::Weaving, 30.0, S::Loom, 120.0),
    // Stone-tending: stock grown for the town (blanks), and pieces grown to
    // shape, which are ordered.
    r("hearthclay", 2, &[("clay", 2), ("ash", 1)], K::Tending, 5.0, S::GrowerBed, 4.0 * D),
    r("ringstone", 1, &[("rock", 3), ("ash", 1)], K::Tending, 15.0, S::GrowerBed, 30.0 * D),
    r("slatewing", 1, &[("rock", 2), ("ash", 1)], K::Tending, 25.0, S::GrowerBed, 21.0 * D),
    r("edgeglass", 1, &[("edge_seed", 1), ("rock", 1), ("ash", 2)], K::Tending, 40.0, S::GrowerBed, 90.0 * D),
    r("hearthclay_pot", 1, &[("clay", 1), ("ash", 1)], K::Tending, 3.0, S::GrowerBed, 4.0 * D),
    p("stone_maul", M::Ringstone, M::None, &[("rock", 6), ("ash", 2), ("timber", 1)], K::Tending, 30.0, S::GrowerBed, 30.0 * D),
    p("kite_shield", M::Ringstone, M::None, &[("rock", 8), ("ash", 3)], K::Tending, 35.0, S::GrowerBed, 30.0 * D),
    p("iron_helm", M::Slatewing, M::None, &[("rock", 2), ("ash", 1), ("leather", 1)], K::Tending, 30.0, S::GrowerBed, 21.0 * D),
    p("sandstone_lamellar", M::Slatewing, M::None, &[("rock", 6), ("ash", 3), ("leather", 2)], K::Tending, 45.0, S::GrowerBed, 21.0 * D),
    p("scale_greaves", M::Slatewing, M::None, &[("rock", 4), ("ash", 2), ("leather", 1)], K::Tending, 35.0, S::GrowerBed, 21.0 * D),
    p("knife", M::Edgeglass, M::None, &[("edge_seed", 1), ("ash", 2), ("leather", 1)], K::Tending, 40.0, S::GrowerBed, 90.0 * D),
    p("short_sword", M::Edgeglass, M::None, &[("edge_seed", 2), ("ash", 3), ("leather", 1)], K::Tending, 50.0, S::GrowerBed, 90.0 * D),
    p("spear", M::Edgeglass, M::None, &[("edge_seed", 1), ("ash", 2), ("timber", 2)], K::Tending, 45.0, S::GrowerBed, 90.0 * D),
    // Paper, ink, scrolls and manuals.
    r("reed_paper", 2, &[("seareed", 1)], K::Inscription, 5.0, S::Desk, 40.0),
    r("squid_ink", 1, &[("ash", 1), ("ash_moss", 1)], K::Inscription, 8.0, S::Desk, 40.0),
    r("scroll_heal", 1, &[("reed_paper", 1), ("squid_ink", 1), ("kelp_frond", 1)], K::Inscription, 20.0, S::Desk, 60.0),
    r("scroll_lightning", 1, &[("reed_paper", 1), ("squid_ink", 1), ("storm_glass", 1)], K::Inscription, 35.0, S::Desk, 60.0),
    r("scroll_paralyze", 1, &[("reed_paper", 1), ("squid_ink", 1), ("ghostcap", 2)], K::Inscription, 40.0, S::Desk, 60.0),
    r("scroll_fireball", 1, &[("reed_paper", 1), ("squid_ink", 1), ("emberroot", 2)], K::Inscription, 45.0, S::Desk, 60.0),
    r("manual_handcraft", 1, &[("reed_paper", 4), ("squid_ink", 2)], K::Inscription, 30.0, S::Desk, 240.0),
    r("manual_smithing", 1, &[("reed_paper", 4), ("squid_ink", 2)], K::Inscription, 30.0, S::Desk, 240.0),
    r("manual_armoring", 1, &[("reed_paper", 4), ("squid_ink", 2)], K::Inscription, 30.0, S::Desk, 240.0),
    r("manual_tending", 1, &[("reed_paper", 4), ("squid_ink", 2)], K::Inscription, 30.0, S::Desk, 240.0),
    r("manual_weaving", 1, &[("reed_paper", 4), ("squid_ink", 2)], K::Inscription, 30.0, S::Desk, 240.0),
    r("manual_inscription", 1, &[("reed_paper", 4), ("squid_ink", 2)], K::Inscription, 30.0, S::Desk, 240.0),
    r("manual_alchemy", 1, &[("reed_paper", 4), ("squid_ink", 2)], K::Inscription, 30.0, S::Desk, 240.0),
    r("manual_carpentry", 1, &[("reed_paper", 4), ("squid_ink", 2)], K::Inscription, 30.0, S::Desk, 240.0),
    r("manual_masonry", 1, &[("reed_paper", 4), ("squid_ink", 2)], K::Inscription, 30.0, S::Desk, 240.0),
    // Alchemy: a mortar and pestle in your pack does as well as the table.
    r("healing_draught", 1, &[("kelp_frond", 2), ("ash_moss", 1)], K::Alchemy, 15.0, S::AlchemyTable, 40.0),
    r("mana_tonic", 1, &[("ghostcap", 2), ("salt_crystal", 1)], K::Alchemy, 25.0, S::AlchemyTable, 40.0),
    r("greater_healing", 1, &[("kelp_frond", 2), ("ghostcap", 1), ("salt_crystal", 1)], K::Alchemy, 45.0, S::AlchemyTable, 60.0),
];

/// How many beds one Tender keeps growing at once (a town's Tenders grow
/// stock in parallel; your squad grows one thing at a time).
pub const TENDER_BEDS: f32 = 40.0;

impl Recipe {
    pub fn craft(&self) -> Craft {
        Craft::of_skill(self.skill).unwrap_or(Craft::Handcraft)
    }

    /// Does it make a piece (a weapon or armour that wears and carries a mark)?
    pub fn is_piece(&self) -> bool {
        items::FORMS.iter().any(|f| f.key == self.output)
    }

    /// Grown to shape: only ever made to order.
    pub fn is_grown_piece(&self) -> bool {
        self.skill == Skill::Tending && self.is_piece()
    }

    /// The item it makes at this grade.
    pub fn item(&self, grade: Grade) -> ItemId {
        let id = items::id(self.output);
        if !self.is_piece() {
            return id;
        }
        let main = if self.main == M::None { items::info(id).main } else { self.main };
        items::variant(id, main, self.second, grade).unwrap_or(id)
    }

    /// Hours a town's crafter spends on it.
    pub fn npc_hours(&self) -> f32 {
        if self.skill == Skill::Tending {
            (self.time / HOUR) as f32 / TENDER_BEDS
        } else {
            (self.time / 60.0) as f32
        }
    }

    /// Does it seal what it makes? (Woven reed made with pitch.)
    pub fn seals(&self) -> bool {
        self.inputs.iter().any(|i| i.0 == "pitch")
    }
}

/// Chance a job comes out right.
pub fn success_chance(skill: f32, difficulty: f32) -> f32 {
    (0.8 + (skill - difficulty) * 0.02).clamp(0.05, 0.98)
}

/// Someone at work at a station.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Job {
    pub who: PersonId,
    pub recipe: usize,
    pub done_at: f64,
    /// Which job this is for them (keys the roll).
    pub n: u64,
    /// Stone being grown: the bed it's in (it only grows while tended).
    #[serde(default)]
    pub bed: Option<V2>,
}

/// A spot where something can be gathered.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Node {
    pub id: u32,
    pub pos: V2,
    pub item: ItemId,
    pub amount: u16,
    /// When it was last picked (it grows back a day later).
    pub picked_at: Option<f64>,
}

impl Node {
    pub fn ready(&self, t: f64) -> bool {
        self.picked_at.map(|p| t - p >= DAY).unwrap_or(true)
    }
}

/// Why a recipe can't be made right now.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cannot {
    Missing(&'static str, u16),
    NoStation(Station),
    Busy,
    /// They haven't taken up the craft (a teacher or a manual first).
    Unknown(Craft),
}

impl World {
    // ---- Setting up ---------------------------------------------------------

    /// Workshops round every town's hearth, and things to gather across the land.
    pub(super) fn place_crafting(&mut self) {
        let mut r = Rng::from_keys(&[self.seed, 0x4E4F_4445]);
        let size = geo::WORLD_SIZE;
        let mut id = 0;
        for _ in 0..9000 {
            if self.nodes.len() >= 700 {
                break;
            }
            let p = V2::new(r.range(300.0, size - 300.0), r.range(300.0, size - 300.0));
            let inland = geo::inland(p);
            let (h, mt, pl) = (self.terrain.height(p), self.terrain.mountains(p), self.terrain.plateau(p));
            let roll = r.f32();
            let key = if (-40.0..15.0).contains(&inland) {
                if roll < 0.5 { "kelp_frond" } else if roll < 0.8 { "mussels" } else { "salt_crystal" }
            } else if inland < 0.0 {
                continue;
            } else if mt > 0.45 && h > 300.0 {
                if roll < 0.3 { "storm_glass" } else { "iron_ore" }
            } else if mt > 0.2 {
                if roll < 0.55 { "iron_ore" } else { "ash_moss" }
            } else if pl > 0.5 {
                if roll < 0.6 { "emberroot" } else { "salt_crystal" }
            } else if roll < 0.3 {
                "timber"
            } else if roll < 0.5 {
                "ghostcap"
            } else if roll < 0.62 {
                "wild_berries"
            } else if roll < 0.75 {
                "ash_moss"
            } else {
                continue;
            };
            // Not in the middle of town.
            if self.settlements.iter().any(|s| s.pos.dist(p) < s.reach) {
                continue;
            }
            let amount = 1 + r.below(3) as u16;
            self.nodes.push(Node { id, pos: p, item: items::id(key), amount, picked_at: None });
            id += 1;
        }
    }

    // ---- Gathering ----------------------------------------------------------

    /// Send someone to gather from a spot.
    pub fn order_gather(&mut self, who: PersonId, node: u32) -> bool {
        let Some(n) = self.nodes.iter().find(|n| n.id == node).copied() else { return false };
        let Some(k) = self.squad.index(who) else { return false };
        let (path, _) = self.route(self.member_pos(k), n.pos);
        self.squad.goal[k] = *path.last().unwrap_or(&n.pos);
        self.squad.route[k] = path;
        self.gathering.retain(|g| g.0 != who);
        self.gathering.push((who, node));
        true
    }

    pub(super) fn do_gathering(&mut self) {
        let mut done = Vec::new();
        for (i, &(who, node)) in self.gathering.iter().enumerate() {
            let (Some(k), Some(ni)) = (self.squad.index(who), self.nodes.iter().position(|n| n.id == node)) else {
                done.push(i);
                continue;
            };
            if self.squad.at[k].dist(self.nodes[ni].pos) > super::squad::REACH {
                continue;
            }
            done.push(i);
            if !self.nodes[ni].ready(self.time) {
                continue;
            }
            let n = self.nodes[ni];
            self.nodes[ni].picked_at = Some(self.time);
            let name = self.people[who as usize].name().unwrap_or("someone").to_string();
            if let Some(d) = self.people[who as usize].detail.as_mut() {
                d.gear.add(n.item, n.amount);
            }
            self.people[who as usize].recompute_might();
            self.log.push_front((self.time, format!("{name} gathers {} × {}.", n.amount, item(n.item).name.to_lowercase())));
            self.log.truncate(14);
        }
        for i in done.into_iter().rev() {
            self.gathering.remove(i);
        }
    }

    // ---- Crafting -----------------------------------------------------------

    /// The nearest station of a kind, if within reach of `p`.
    pub fn station_near(&self, p: V2, kind: Station) -> Option<V2> {
        self.stations.iter().filter(|(_, k)| *k == kind).map(|(s, _)| *s).find(|s| s.dist(p) <= AT_STATION)
    }

    pub fn count_of(&self, who: PersonId, key: &str) -> u16 {
        let id = items::id(key);
        self.people[who as usize].detail.as_ref().map(|d| d.gear.bag.iter().filter(|e| e.0 == id).map(|e| e.1).sum()).unwrap_or(0)
    }

    /// Can this person make recipe `ri` where they stand?
    pub fn can_craft(&self, who: PersonId, ri: usize) -> Result<(), Cannot> {
        let rc = &RECIPES[ri];
        // Stone grows on its own while tended, so it doesn't keep its
        // grower from other work; one bed at a time, though.
        let growing = |j: &Job| RECIPES[j.recipe].skill == Skill::Tending;
        if self.crafting.iter().any(|j| j.who == who && growing(j) == (rc.skill == Skill::Tending)) || self.fighting.contains_key(&who) {
            return Err(Cannot::Busy);
        }
        if !self.knows_craft(who, rc.craft()) {
            return Err(Cannot::Unknown(rc.craft()));
        }
        for &(k, n) in rc.inputs {
            if self.count_of(who, k) < n {
                return Err(Cannot::Missing(k, n));
            }
        }
        let at = self.person_pos(who);
        let portable = rc.station == Station::AlchemyTable && self.count_of(who, "mortar_and_pestle") > 0;
        if !portable && self.station_near(at, rc.station).is_none() {
            return Err(Cannot::NoStation(rc.station));
        }
        Ok(())
    }

    /// Start making something: materials go in now.
    pub fn start_craft(&mut self, who: PersonId, ri: usize) -> Result<(), Cannot> {
        self.can_craft(who, ri)?;
        let rc = &RECIPES[ri];
        if let Some(d) = self.people[who as usize].detail.as_mut() {
            for &(k, n) in rc.inputs {
                for _ in 0..n {
                    d.gear.take(items::id(k));
                }
            }
        }
        self.people[who as usize].recompute_might();
        let bed = (rc.skill == Skill::Tending).then(|| self.station_near(self.person_pos(who), Station::GrowerBed)).flatten();
        let n = self.crafted_count.entry(who).or_insert(0);
        *n += 1;
        let job = Job { who, recipe: ri, done_at: self.time + rc.time, n: *n, bed };
        self.crafting.push(job);
        Ok(())
    }

    pub(super) fn do_crafting(&mut self) {
        let mut done = Vec::new();
        for (i, j) in self.crafting.iter().enumerate() {
            if self.time >= j.done_at {
                done.push((i, *j));
            }
        }
        for &(i, _) in done.iter().rev() {
            self.crafting.remove(i);
        }
        for (_, j) in done {
            let rc = &RECIPES[j.recipe];
            let p = &self.people[j.who as usize];
            let skill = p.effective_stats().skill(rc.skill);
            let mut roll = Rng::from_keys(&[self.seed, j.who as u64, j.n, 0x4352_4146]);
            let name = p.name().unwrap_or("someone").to_string();
            let ok = roll.f32() < success_chance(skill, rc.difficulty);
            let grade = Grade::from(skill, 1.0, roll.f32());
            let at = j.bed.unwrap_or_else(|| self.person_pos(j.who));
            let town = self.town_at(at);
            let p = &mut self.people[j.who as usize];
            p.stats.exercise(rc.skill, if ok { 2.0 } else { 0.8 });
            let line = if ok {
                let id = rc.item(grade);
                if let Some(d) = p.detail.as_mut() {
                    if rc.is_piece() {
                        d.gear.add_piece(id, made_piece(id, j.who, town, skill, rc.seals(), j.done_at));
                    } else {
                        d.gear.add(id, rc.makes);
                    }
                }
                format!("{name} makes {}.", item(id).name.to_lowercase())
            } else {
                if let Some(d) = p.detail.as_mut() {
                    for &(k, n) in rc.inputs {
                        if n / 2 > 0 {
                            d.gear.add(items::id(k), n / 2);
                        }
                    }
                }
                format!("{name} botches the {}.", item(rc.item(Grade::Common)).name.to_lowercase())
            };
            p.recompute_might();
            self.log.push_front((j.done_at, line));
            self.log.truncate(14);
        }
    }

    /// The town a spot is in or beside, if any.
    pub fn town_at(&self, p: V2) -> Option<u16> {
        self.settlements.iter().filter(|s| s.pos.dist(p) <= s.reach + 150.0).min_by(|a, b| a.pos.dist(p).total_cmp(&b.pos.dist(p))).map(|s| s.id)
    }

    /// At dawn: stone that went untended for a day grew no further.
    pub(super) fn tend_beds(&mut self, t: f64) {
        for k in 0..self.crafting.len() {
            let j = self.crafting[k];
            let Some(bed) = j.bed else { continue };
            let here = self.squad.index(j.who).map(|i| self.squad.at[i].dist(bed) <= TENDING_REACH).unwrap_or(false);
            if !here && j.done_at > t {
                self.crafting[k].done_at += DAY;
            }
        }
    }

    /// How far along someone's job is, 0..1.
    pub fn craft_progress(&self, who: PersonId) -> Option<f32> {
        // Work at a station shows before stone growing in a bed.
        let j = self.crafting.iter().filter(|j| j.who == who).min_by_key(|j| j.bed.is_some())?;
        let total = RECIPES[j.recipe].time;
        Some((1.0 - (j.done_at - self.time) / total).clamp(0.0, 1.0) as f32)
    }

    // ---- Using things -------------------------------------------------------

    /// Drink a potion or read a scroll outside a fight. Returns false if it
    /// can't be used now (attack scrolls are for fights).
    pub fn use_item(&mut self, who: PersonId, it: ItemId) -> bool {
        let t = self.time;
        let p = &self.people[who as usize];
        let Some(d) = p.detail.as_ref() else { return false };
        if !d.gear.bag.iter().any(|e| e.0 == it) || self.fighting.contains_key(&who) {
            return false;
        }
        if let Kind::Food(_) = item(it).kind {
            return self.eat(who, it, t);
        }
        if let Kind::StandingTorch(_) = item(it).kind {
            return self.place_torch(who);
        }
        if matches!(item(it).kind, Kind::Notes(_) | Kind::Text(_)) {
            return self.read_lore(who, it);
        }
        if let Kind::Manual(_) = item(it).kind {
            return self.study(who, it);
        }
        // A potion's own effects, or a scroll's spell (no energy, can't fail).
        let effects: &[super::effects::Effect] = match item(it).kind {
            Kind::Potion => item(it).effects,
            Kind::Scroll(key) => {
                let d = super::magic::spell(key).def();
                if !d.works_outside_fights() {
                    return false;
                }
                d.effects
            }
            _ => return false,
        };
        let name = p.name().unwrap_or("someone").to_string();
        self.people[who as usize].detail.as_mut().unwrap().gear.take(it);
        for e in effects {
            self.apply_effect(who, e, 1.0);
        }
        let p = &mut self.people[who as usize];
        p.recompute_might();
        self.log.push_front((t, format!("{name} uses the {}.", item(it).name.to_lowercase())));
        self.log.truncate(14);
        true
    }
}

/// Spread `amount` of healing over a body, worst-hurt vital parts first.
pub fn mend(hp: &mut [f32; 6], stats: &super::stats::Stats, mut amount: f32) {
    use super::body::PARTS;
    while amount > 0.01 {
        // The part furthest below its maximum, vital parts weighted up.
        let Some((i, gap)) = PARTS
            .iter()
            .enumerate()
            .map(|(i, part)| (i, (stats.max_hp(*part) - hp[i]) * if part.vital() { 1.5 } else { 1.0 }))
            .filter(|(_, g)| *g > 0.01)
            .max_by(|a, b| a.1.total_cmp(&b.1))
        else {
            break;
        };
        let room = stats.max_hp(PARTS[i]) - hp[i];
        let give = amount.min(room).min(gap.max(5.0));
        hp[i] += give;
        amount -= give;
    }
}

/// A newly made piece: full durability, the maker's mark (stamped when the
/// work is fine or the maker skilled), sealed if pitch went into it.
pub fn made_piece(id: ItemId, maker: PersonId, town: Option<u16>, skill: f32, sealed: bool, t: f64) -> Piece {
    let grade = items::info(id).grade;
    let mut pc = super::wear::fresh(id, t);
    pc.mark = Some(Mark { maker, town, stamped: grade >= Grade::Fine || skill >= MARK_SKILL });
    pc.sealed = sealed && items::info(id).main.def().rots;
    pc
}
