//! Factions (Laz, `docs/factions.md` and `docs/leaders.md`, 12 Oct): who
//! the island's powers are, how they regard each other, and how they regard
//! the squad. Steps F1 and F2 of the faction plan:
//!
//! - **The data** (`FACTIONS`): the three majors, each with three
//!   sub-factions; the minor factions; the guilds. Each says its parent, its
//!   people (whose tongue its native name is in, `assets/lang/things.ron`),
//!   and anything special about it (the Rooted are one-way; the two
//!   syndicates are each other's enemy). How they regard each other is one
//!   table (`RELATIONS`).
//! - **Leaders** (`LEADERS`) are data too; they become people with places
//!   once Laz's map gives the factions their towns.
//! - **The Great House's lords** are each their own entity (`Lord`, saved):
//!   their group (Banner or Free Lords) and how they stand with the Throne
//!   drift at dawn, by keyed rolls, so the Empire's politics move on their
//!   own.
//! - **The squad's standing** with each faction (and each lord), from -100
//!   to 100, saved. A deed with a sub-faction counts half again with its
//!   parent; a deed with anyone ripples to those who like or loathe them
//!   (`deed`). Entry to a town reads its holder's parent; quests and
//!   hostility read the sub-faction.
//! - **What standing does in a town** that has a holder (`Settlement::holder`;
//!   none until the map): prices, whether its merchants and inns will deal,
//!   and whether its watch goes for the squad on sight (F2).

use serde::{Deserialize, Serialize};

use super::race::Race;
use super::rng::Rng;
use super::world::World;

/// A faction, by its place in `FACTIONS`.
pub type FactionId = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    Major,
    /// One of a major's three.
    Sub,
    Minor,
    Guild,
}

#[derive(Clone, Copy, Debug)]
pub struct Faction {
    pub key: &'static str,
    pub name: &'static str,
    pub size: Size,
    /// A sub-faction's major.
    pub parent: Option<&'static str>,
    /// Whose tongue its native name is in (the Open Hand is mixed: Ṭaḍoro).
    pub people: Race,
    /// Joining it ends every other membership and bars new ones.
    pub one_way: bool,
    /// Peoples it distrusts (the First Peoples' contempt: worse prices, a
    /// colder welcome for squad members of those peoples).
    pub distrusts: &'static [Race],
    /// One line, for the window.
    pub about: &'static str,
}

const fn f(key: &'static str, name: &'static str, size: Size, parent: Option<&'static str>, people: Race, about: &'static str) -> Faction {
    Faction { key, name, size, parent, people, one_way: false, distrusts: &[], about }
}

use Race::{Horaro as H, Qotiro as Q, Roduro as R, Tadoro as T};
use Size::{Guild, Major, Minor, Sub};

pub static FACTIONS: &[Faction] = &[
    // The majors and their three each.
    Faction { distrusts: &[Q, T], ..f("first_peoples", "The First Peoples", Major, None, R, "The Roduro-Horaro Alliance: proud, closed, ruled by old families") },
    f("seated_houses", "The Seated Houses", Sub, Some("first_peoples"), R, "The Roduro houses listened to now; they run the Alliance"),
    f("true_line", "The True Line", Sub, Some("first_peoples"), R, "Old houses claiming a purer line; they scheme to rule"),
    f("tide_houses", "The Tide Houses", Sub, Some("first_peoples"), H, "The Horaro houses still in the Alliance: honoured, unheard"),
    f("great_house", "The Great House", Major, None, R, "What's left of the old empire: one banner, many rival lords"),
    f("throne", "The Throne", Sub, Some("great_house"), R, "The emperor's court and soldiers, who'd restore the empire"),
    f("banner_lords", "The Banner Lords", Sub, Some("great_house"), R, "Lords who fly the Throne's banner, in public at least"),
    f("free_lords", "The Free Lords", Sub, Some("great_house"), R, "Lords who ignore the Throne and keep their own"),
    f("open_hand", "The Open Hand", Major, None, T, "The people's faction: mixed, commercial, run by a council"),
    f("open_council", "The Open Council", Sub, Some("open_hand"), T, "The council houses and the trading mainstream"),
    f("sharp_hands", "The Sharp Hands", Sub, Some("open_hand"), T, "Cutthroat merchants who'd be the new old money"),
    f("road_crews", "The Road Crews", Sub, Some("open_hand"), T, "The carters and caravan masters who do the hauling"),
    // The minor factions.
    f("holy_house", "The Holy House", Minor, None, R, "The Roduro church: corrupt, armed, losing believers"),
    f("last_wave", "The Last Wave", Minor, None, Q, "The newest Qotiro: warrior enclaves, seen as a threat"),
    f("far_sails", "The Far Sails", Minor, None, H, "Horaro who left the Alliance: ships and sea lanes"),
    f("binders", "The Binders", Minor, None, Q, "Slavers who sell to the lords"),
    f("wardens", "The Wardens", Minor, None, R, "A small order holding the line against the Overgrowth"),
    f("highfolk", "The Highfolk", Minor, None, R, "Mountain clans the Empire claims; they see themselves as free"),
    Faction { one_way: true, ..f("rooted", "The Rooted", Minor, None, R, "Sworn to the green: they'd cover the island with it") },
    // The guilds, in every large town.
    f("writers", "The Writers", Guild, None, T, "Scholars: spells, manuals, everyone's secrets"),
    f("lantern_company", "The Lantern Company", Guild, None, T, "Swords for hire: guards, escorts, bounties"),
    f("old_hands", "The Old Hands", Guild, None, R, "A Roduro syndicate rooted in the old families"),
    f("night_market", "The Night Market", Guild, None, H, "A mixed smuggling and fencing ring"),
    f("dueling_society", "The Dueling Society", Guild, None, T, "Combat as sport: nobody dies"),
];

/// How factions regard each other, -1 (loathe) .. 1 (allied); anything not
/// listed is 0. Both ways.
pub static RELATIONS: &[(&str, &str, f32)] = &[
    ("first_peoples", "great_house", -0.3),
    ("first_peoples", "open_hand", -0.5),
    ("great_house", "open_hand", -0.4),
    ("first_peoples", "last_wave", -0.8),
    ("first_peoples", "holy_house", 0.5),
    ("first_peoples", "far_sails", -0.3),
    ("great_house", "binders", 0.4),
    ("free_lords", "binders", 0.6),
    ("throne", "free_lords", -0.4),
    ("open_hand", "binders", -0.6),
    ("open_hand", "far_sails", 0.3),
    ("true_line", "seated_houses", -0.5),
    ("true_line", "holy_house", 0.4),
    ("sharp_hands", "road_crews", -0.3),
    ("holy_house", "last_wave", -0.6),
    ("far_sails", "night_market", 0.4),
    ("old_hands", "night_market", -1.0),
    ("old_hands", "last_wave", -0.6),
    ("old_hands", "first_peoples", 0.3),
    ("wardens", "rooted", -1.0),
    ("highfolk", "great_house", -0.3),
    ("dueling_society", "lantern_company", 0.2),
];

/// The leaders (Laz's names): whose, what they are. They become people with
/// homes once the map gives the factions their towns.
#[derive(Clone, Copy, Debug)]
pub struct Leader {
    pub name: &'static str,
    pub byname: &'static str,
    pub faction: &'static str,
    pub role: &'static str,
}

const fn l(name: &'static str, byname: &'static str, faction: &'static str, role: &'static str) -> Leader {
    Leader { name, byname, faction, role }
}

pub static LEADERS: &[Leader] = &[
    l("Tuhuḍo", "of the Ninth-Ring House", "seated_houses", "the blind archivist-lord"),
    l("Garoṭaqa", "of the First-Stone House", "true_line", "the mother"),
    l("Ruʻo", "of the First-Stone House", "true_line", "the son"),
    l("Niralea", "daughter of the Reef Line", "tide_houses", "the season's speaker"),
    l("Gahìru", "of the Great House", "great_house", "the emperor"),
    l("Litiṭaʻa", "of the Great House", "throne", "the emperor's sister"),
    l("Haʻahoqo", "the General, of the Long-Wall House", "banner_lords", "the old general"),
    l("Shahayuh", "of the North Road", "banner_lords", "the Ṭaḍoro-born lord"),
    l("Kotqot", "Wall-breaker", "free_lords", "the Qotiro-blooded lord"),
    l("Waarua", "of the Harbour", "open_council", "the freed bondswoman"),
    l("Feyesh", "of the Silver Road", "sharp_hands", "the broker"),
    l("Dorog", "Coin-counter", "sharp_hands", "one of the twins"),
    l("Kodor", "Debt-taker", "sharp_hands", "one of the twins"),
    l("Doruḍoqu", "the Carter", "road_crews", "the elected foreman"),
    l("Hotorìṭo", "the High Priest", "holy_house", "the high priest"),
    l("Aloqi", "the Priestess", "holy_house", "the sincere priestess"),
    l("Textrim", "High Priestess", "last_wave", "the high priestess"),
    l("Totkit", "Sea-crosser", "last_wave", "the war-chief"),
    l("Nemu", "of the Wide Bay", "far_sails", "the speaker"),
    l("Paqax", "Chain-breaker", "binders", "the ex-slave slaver"),
    l("Haʻo", "of the Green Wall", "wardens", "the husband"),
    l("Hoqa", "of the Green Wall", "wardens", "the wife"),
    l("Shihi", "of the High Snows", "highfolk", "the chieftain"),
    l("Fiyeweya", "of No Road", "writers", "the wandering head"),
    l("Lamawina", "of the Lantern", "lantern_company", "the quartermaster"),
    l("Yitolaʻo", "of the Old Gate House", "old_hands", "the old don"),
    l("Yaweth", "of the Market Road", "night_market", "the fence"),
    l("Liʻo", "of the Ring", "dueling_society", "the brother"),
    l("Kitim", "Duel-ender", "dueling_society", "the sister"),
];

// ---- Standing ---------------------------------------------------------------------

/// Standing runs from -100 to 100; these are its steps.
pub const HOSTILE: f32 = -50.0;
pub const UNWELCOME: f32 = -20.0;
pub const FRIENDLY: f32 = 25.0;
pub const HONOURED: f32 = 60.0;
/// A deed with a sub-faction counts this much with its parent too.
pub const PARENT_SHARE: f32 = 0.5;
/// A deed ripples to those who like or loathe the faction, by this much
/// times how strongly they do.
pub const RIPPLE: f32 = 0.3;
/// A lord flies the Throne's banner above this standing with it, and turns
/// Free Lord below its negative (between, they stay as they are).
pub const BANNER_AT: f32 = 15.0;
/// How far a lord's standing with the Throne can drift in a day.
pub const LORD_DRIFT: f32 = 4.0;
/// A job done for someone in a held town, with its holder.
pub const JOB_DEED: f32 = 3.0;

/// A crime told in a held town, with its holder: more for a bigger fine.
pub fn crime_deed(fine: f32) -> f32 {
    (2.0 + fine / 20.0).min(20.0)
}

/// Prices in a held town, by the squad's standing with its holder.
pub const PRICE_HONOURED: f32 = 0.9;
pub const PRICE_FRIENDLY: f32 = 0.95;
pub const PRICE_UNWELCOME: f32 = 1.2;

/// The squad's standing in words.
pub fn standing_word(s: f32) -> &'static str {
    match s {
        s if s <= HOSTILE => "Hostile",
        s if s <= UNWELCOME => "Unwelcome",
        s if s >= HONOURED => "Honoured",
        s if s >= FRIENDLY => "Friendly",
        _ => "Neutral",
    }
}

/// A faction by its key (panics on a key that isn't in `FACTIONS`: data).
pub fn id(key: &str) -> FactionId {
    FACTIONS.iter().position(|f| f.key == key).unwrap_or_else(|| panic!("no faction `{key}`")) as FactionId
}

pub fn def(f: FactionId) -> &'static Faction {
    &FACTIONS[f as usize]
}

/// A sub-faction's major (itself for anything else).
pub fn parent(f: FactionId) -> FactionId {
    def(f).parent.map(id).unwrap_or(f)
}

/// How `a` regards `b`, -1..1 (themselves: 1).
pub fn relation(a: FactionId, b: FactionId) -> f32 {
    if a == b {
        return 1.0;
    }
    // The Rooted are against everyone, and everyone against them.
    if def(a).one_way || def(b).one_way {
        return -1.0;
    }
    let (ka, kb) = (def(a).key, def(b).key);
    RELATIONS.iter().find(|r| (r.0 == ka && r.1 == kb) || (r.0 == kb && r.1 == ka)).map(|r| r.2).unwrap_or(0.0)
}

/// One of the Great House's lords: their own entity, with their own
/// standing with the Throne and with the squad.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Lord {
    pub name: String,
    /// Banner Lords or Free Lords.
    pub group: FactionId,
    /// How they stand with the Throne, -100..100.
    pub throne: f32,
    /// How they regard the squad, -100..100.
    pub standing: f32,
}

/// The squad's standing with the factions and the lords, and the lords'
/// own politics.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Factions {
    /// By `FactionId` (shorter than `FACTIONS` in an older world: the rest 0).
    pub standing: Vec<f32>,
    pub lords: Vec<Lord>,
}

impl Factions {
    /// As the world begins: nobody thinks anything of the squad yet, and the
    /// lords stand with the Throne as Laz wrote them.
    pub fn new() -> Factions {
        let lord = |name: &str, group: &str, throne: f32| Lord { name: name.to_string(), group: id(group), throne, standing: 0.0 };
        Factions { standing: vec![0.0; FACTIONS.len()], lords: vec![lord("Haʻahoqo", "banner_lords", 60.0), lord("Shahayuh", "banner_lords", 35.0), lord("Kotqot", "free_lords", -50.0)] }
    }

    pub fn standing(&self, f: FactionId) -> f32 {
        self.standing.get(f as usize).copied().unwrap_or(0.0)
    }

    fn add(&mut self, f: FactionId, x: f32) {
        if self.standing.len() < FACTIONS.len() {
            self.standing.resize(FACTIONS.len(), 0.0);
        }
        let s = &mut self.standing[f as usize];
        *s = (*s + x).clamp(-100.0, 100.0);
    }
}

impl World {
    /// The squad's standing with a faction.
    pub fn standing_with(&self, f: FactionId) -> f32 {
        self.factions.standing(f)
    }

    /// Something the squad did that a faction cares about (`amount` +
    /// helped, - harmed): it counts in full with them, half again with their
    /// parent, and ripples to those who like or loathe them.
    pub fn deed(&mut self, f: FactionId, amount: f32) {
        let p = parent(f);
        self.factions.add(f, amount);
        if p != f {
            self.factions.add(p, amount * PARENT_SHARE);
        }
        for g in 0..FACTIONS.len() as FactionId {
            if g == f || g == p {
                continue;
            }
            let r = relation(f, g);
            if r != 0.0 {
                self.factions.add(g, amount * RIPPLE * r);
            }
        }
    }

    /// Something the squad did for or against one lord: it counts with them,
    /// and (less) with their group.
    pub fn lord_deed(&mut self, k: usize, amount: f32) {
        let Some(l) = self.factions.lords.get_mut(k) else { return };
        l.standing = (l.standing + amount).clamp(-100.0, 100.0);
        let g = l.group;
        self.deed(g, amount * PARENT_SHARE);
    }

    /// At dawn: each lord's standing with the Throne drifts (one keyed roll
    /// per lord and day), and one who drifts far enough changes sides.
    pub(super) fn dawn_factions(&mut self, t: f64) {
        let day = (t / super::world::DAY).floor() as u64;
        let (banner, free) = (id("banner_lords"), id("free_lords"));
        for k in 0..self.factions.lords.len() {
            let r = Rng::from_keys(&[self.seed, k as u64, day, 0x4C4F_5244]).range(-LORD_DRIFT, LORD_DRIFT);
            let l = &mut self.factions.lords[k];
            l.throne = (l.throne + r).clamp(-100.0, 100.0);
            let was = l.group;
            if l.throne > BANNER_AT {
                l.group = banner;
            } else if l.throne < -BANNER_AT {
                l.group = free;
            }
            if l.group != was {
                let line = if l.group == banner { format!("Lord {} raises the Throne's banner again.", l.name) } else { format!("Lord {} no longer flies the Throne's banner.", l.name) };
                self.log.push_front((t, line));
                self.log.truncate(14);
            }
        }
    }

    /// Who holds a town (none until the map gives the factions their towns).
    pub fn holder_of(&self, town: super::settlement::SettlementId) -> Option<FactionId> {
        self.settlements.get(town as usize).and_then(|s| s.holder)
    }

    /// How a town's holder regards the squad: entry reads the major, so
    /// this is the parent's standing (None: nobody holds it).
    pub fn town_regard(&self, town: super::settlement::SettlementId) -> Option<f32> {
        self.holder_of(town).map(|f| self.standing_with(parent(f)))
    }

    /// Prices in a town by its holder's regard: better when honoured, worse
    /// when unwelcome (1 where nobody holds it).
    pub fn regard_price(&self, town: super::settlement::SettlementId) -> f32 {
        match self.town_regard(town) {
            Some(s) if s >= HONOURED => PRICE_HONOURED,
            Some(s) if s >= FRIENDLY => PRICE_FRIENDLY,
            Some(s) if s <= UNWELCOME => PRICE_UNWELCOME,
            _ => 1.0,
        }
    }

    /// Will a town's merchants and inns deal with the squad? Not where its
    /// holder is hostile to them.
    pub fn deals_with_squad(&self, town: super::settlement::SettlementId) -> bool {
        self.town_regard(town).is_none_or(|s| s > HOSTILE)
    }
}
