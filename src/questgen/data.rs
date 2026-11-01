//! The contract: what a generated quest set holds.
//!
//! Field names match `claude/quest-card-schema.md` exactly (v1.8), so a quest
//! set reads and writes as the JSON the design tests used
//! (`tools/quests/samples/`). Every collection is ordered (`Vec` or
//! `BTreeMap`), never a hash map, so nothing depends on iteration order.
//!
//! **Storage:** a quest set is plain data and serializes to JSON. Some fields
//! are open-ended (`details`, `kit_rolls`, a crossing's note), held as
//! `serde_json::Value`; when the sim saves a quest set it stores the JSON text,
//! not bincode.
//!
//! **Ids** are plain strings with a kind prefix by convention: `p-` person,
//! `pl-` place, `t-` thing, `f-` faction, `x-` fact, `c-` card, `s-` spec,
//! `m-` move, `ch-` chapter, `n` node, `fs-` faction story.
//!
//! **Tolerant reading:** every struct takes `#[serde(default)]`, so a field
//! missing from older data reads as empty, and unknown fields are ignored.
//! Closed value sets are enums; open ones (premises, roles, link types,
//! traits) are strings, checked by the rules, not by the type.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub type Id = String;

// ---------------------------------------------------------------------------
// The quest set
// ---------------------------------------------------------------------------

/// One world's quests: the generator's whole output.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct QuestSet {
    /// What the land generator handed over: regions, places, roads. Input,
    /// kept beside the output so checkers can read distances and quotas.
    pub map: MapSummary,
    pub registry: Registry,
    /// Every fact, stored once. Entries and cards refer to facts by id.
    pub facts: Vec<Fact>,
    pub cards: Vec<Card>,
    /// Job descriptions not yet cast, or kept after casting with `filled_by`.
    pub specs: Vec<Spec>,
    /// The one main story. None while only faction stories exist (a test world).
    pub story: Option<Story>,
    pub faction_stories: Vec<FactionStory>,
}

// ---------------------------------------------------------------------------
// Shared value sets
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Standing {
    Low,
    Middle,
    High,
    Top,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Household,
    Community,
    Region,
    Realm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Secret,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Public
    }
}

/// The shared want list (rulebook §7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Want {
    Greed,
    Revenge,
    #[serde(rename = "cover-up")]
    CoverUp,
    Desperation,
    Ambition,
    Preservation,
}

/// The nine families (rulebook §8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Family {
    Rescue,
    Investigate,
    Restore,
    Acquire,
    Passage,
    Threat,
    Dispute,
    Reckoning,
    Contest,
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Registry {
    pub people: Vec<Person>,
    pub places: Vec<Place>,
    pub things: Vec<Thing>,
    pub factions: Vec<Faction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Person {
    pub id: Id,
    pub name: String,
    pub standing: Standing,
    /// The leader archetype this person fills (`quest-casting.md` §4).
    pub archetype: Option<Id>,
    /// The class they were made from, if any (`quest-casting.md` §3).
    pub class: Option<String>,
    pub traits: Vec<String>,
    pub links: Vec<Link>,
    pub faction: Option<Membership>,
    /// Fact ids.
    pub facts: Vec<Id>,
    pub premade: bool,
    pub dead: bool,
    pub home: Option<Id>,
    /// A move id: alive on cards before it, dead after.
    pub dies_at: Option<Id>,
    pub fallback: Option<Id>,
}

impl Default for Person {
    fn default() -> Self {
        Person {
            id: Id::new(),
            name: String::new(),
            standing: Standing::Low,
            archetype: None,
            class: None,
            traits: Vec::new(),
            links: Vec::new(),
            faction: None,
            facts: Vec::new(),
            premade: false,
            dead: false,
            home: None,
            dies_at: None,
            fallback: None,
        }
    }
}

/// A link between two people (rulebook §6). `kind` is the link type: family,
/// kin, spouse/lover, ally, employer/employee, rival, creditor/debtor, grudge,
/// dependent, trade. `as` is which side this person is on, for one-way links.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Link {
    #[serde(rename = "type")]
    pub kind: String,
    pub to: Id,
    #[serde(rename = "as")]
    pub side: Option<String>,
    pub visibility: Visibility,
    /// When the link began (a relative day or a move id), when casting added it.
    pub when: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Membership {
    pub id: Id,
    pub sub: Option<Id>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Access {
    Open,
    Guarded,
    Remote,
    Fortified,
}

impl Default for Access {
    fn default() -> Self {
        Access::Open
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Place {
    pub id: Id,
    pub name: String,
    /// village, keep, ford, mill, toll-house… (routes: road, pass, ford, river, wilds)
    pub kind: String,
    pub owner: Option<Id>,
    pub access: Access,
    pub standing: Standing,
    pub region: Option<Id>,
    /// landmark, foundational
    pub tags: Vec<String>,
    pub facts: Vec<Id>,
    /// Seats only: who holds it now (rulebook, *Seats and offices have one holder*).
    pub holder: Option<Id>,
    /// Offices tied to this place with no building of their own.
    pub titles: Vec<Title>,
    /// Settlements only: public facts the big stories wrote here. Never a gate.
    pub context: Vec<Id>,
    /// Sites only: the settlement it sits in.
    pub at: Option<Id>,
    pub descriptors: Vec<String>,
    pub affordances: Vec<Affordance>,
}

impl Default for Place {
    fn default() -> Self {
        Place {
            id: Id::new(),
            name: String::new(),
            kind: String::new(),
            owner: None,
            access: Access::Open,
            standing: Standing::Low,
            region: None,
            tags: Vec::new(),
            facts: Vec::new(),
            holder: None,
            titles: Vec::new(),
            context: Vec::new(),
            at: None,
            descriptors: Vec::new(),
            affordances: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Title {
    pub title: String,
    pub holder: Option<Id>,
    pub since: Option<String>,
}

/// Something a card's ways need the place to have (a drain nobody guards).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Affordance {
    pub text: String,
    pub from_card: Option<Id>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Thing {
    pub id: Id,
    pub name: String,
    /// heirloom / document / weapon / relic / goods / tool
    pub kind: String,
    pub owner: Option<Id>,
    pub value: Standing,
    pub location: Option<Id>,
    pub held_by: Option<Id>,
    /// The card or move it came to `held_by` at.
    pub since: Option<Id>,
    pub tags: Vec<String>,
    pub facts: Vec<Id>,
}

impl Default for Thing {
    fn default() -> Self {
        Thing {
            id: Id::new(),
            name: String::new(),
            kind: String::new(),
            owner: None,
            value: Standing::Low,
            location: None,
            held_by: None,
            since: None,
            tags: Vec::new(),
            facts: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FactionSize {
    Major,
    /// A sub-faction of a major one.
    Sub,
    Minor,
    Guild,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Reach {
    Community,
    Region,
    Realm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelationKind {
    Ally,
    Rival,
    #[serde(rename = "at war")]
    AtWar,
    Trade,
    Serves,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relation {
    #[serde(rename = "type")]
    pub kind: RelationKind,
    pub to: Id,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Faction {
    pub id: Id,
    pub name: String,
    pub size: FactionSize,
    pub parent: Option<Id>,
    pub leaders: Vec<Id>,
    pub seat: Option<Id>,
    pub reach: Reach,
    pub want: Option<Want>,
    pub virtue: Option<String>,
    pub wound: Option<String>,
    /// A secret fact id.
    pub hidden_side: Option<Id>,
    pub relations: Vec<Relation>,
    pub facts: Vec<Id>,
}

impl Default for Faction {
    fn default() -> Self {
        Faction {
            id: Id::new(),
            name: String::new(),
            size: FactionSize::Minor,
            parent: None,
            leaders: Vec::new(),
            seat: None,
            reach: Reach::Community,
            want: None,
            virtue: None,
            wound: None,
            hidden_side: None,
            relations: Vec::new(),
            facts: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactKind {
    #[serde(rename = "past event")]
    PastEvent,
    #[serde(rename = "state")]
    State,
    #[serde(rename = "trait")]
    Trait,
    #[serde(rename = "plant")]
    Plant,
    #[serde(rename = "move")]
    Move,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Fact {
    pub id: Id,
    pub text: String,
    pub visibility: Visibility,
    /// Revelations only: the cards that make it public.
    pub published_by: Vec<Id>,
    pub kind: FactKind,
    /// Required on past events: "40 days before the story", "after m-2".
    pub when: Option<String>,
    /// A card id, `story`, or `world`.
    pub source: Option<String>,
}

impl Default for Fact {
    fn default() -> Self {
        Fact {
            id: Id::new(),
            text: String::new(),
            visibility: Visibility::Public,
            published_by: Vec::new(),
            kind: FactKind::PastEvent,
            when: None,
            source: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Specs (job descriptions for casting, `quest-casting.md` §2)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    Person,
    Place,
    Thing,
}

impl Default for EntryKind {
    fn default() -> Self {
        EntryKind::Person
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CastHow {
    #[serde(rename = "reused")]
    Reused,
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "looked up")]
    LookedUp,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Spec {
    pub id: Id,
    pub card: Id,
    pub role: String,
    pub kind: EntryKind,
    /// The layer of power whose casting fills it (0–4).
    pub layer: u8,
    pub needs: Vec<Need>,
    /// Soft wishes, in the same vocabulary as `needs`; each met adds a point.
    pub prefers: Vec<Need>,
    pub group: Option<Id>,
    pub filled_by: Option<Id>,
    pub cast_how: Option<CastHow>,
    pub score: Option<f32>,
}

/// One requirement in a spec (`quest-casting.md` §2.1). Written in JSON as
/// `{"need": "standing", "min": "Middle", "max": "High"}` and so on.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "need", rename_all = "snake_case")]
pub enum Need {
    Standing { min: Standing, max: Standing },
    Class { class: String },
    Archetype { archetype: Id },
    /// In this faction (or sub-faction); `not: true` means outside it.
    Faction {
        faction: Id,
        #[serde(default)]
        not: bool,
    },
    People { people: String },
    Sex { sex: String },
    Trait { name: String },
    /// A link to another slot on the card (`to_role`) or another spec (`to_spec`).
    Link {
        #[serde(rename = "type")]
        kind: String,
        #[serde(default, rename = "as")]
        side: Option<String>,
        #[serde(default)]
        to_role: Option<String>,
        #[serde(default)]
        to_spec: Option<Id>,
        #[serde(default)]
        visibility: Visibility,
    },
    /// Where the filler lives or sits, relative to a place or a spec.
    Where {
        rel: Nearness,
        #[serde(default)]
        of_place: Option<Id>,
        #[serde(default)]
        of_spec: Option<Id>,
    },
    /// Holds this seat or office: looked up, never searched.
    Office { seat: Id, #[serde(default)] title: Option<String> },
    /// Alive until this move (`dies_at`).
    AliveUntil { move_id: Id },
    /// Free at this card's time: not held, fled, dead or captive.
    Free,
    AtHome,
    MustReuse,
    /// A place spec's required affordance (a cell, a back door).
    Affords { text: String },
    /// Place specs: access level.
    Access { access: Access },
    /// Place or thing specs: what kind of place or thing.
    OfKind { kind: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Nearness {
    At,
    Near,
    InRegion,
    JourneyFrom,
}

// ---------------------------------------------------------------------------
// Cards
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Relatedness {
    Main,
    Minor,
    Fallout,
    Echo,
    Independent,
    Faction,
}

impl Default for Relatedness {
    fn default() -> Self {
        Relatedness::Independent
    }
}

/// A slot holds one entry id (or fact id, or spec id), or a short list of them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SlotValue {
    One(Id),
    Many(Vec<Id>),
}

impl SlotValue {
    pub fn ids(&self) -> Vec<&Id> {
        match self {
            SlotValue::One(id) => vec![id],
            SlotValue::Many(ids) => ids.iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Clock {
    pub checkpoint: String,
    pub world_ending: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CardFaction {
    pub faction: Id,
    /// member / backer / attacker / representative
    pub role: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Twist {
    pub name: String,
    /// The twist group (rulebook §9).
    pub group: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThreadMark {
    pub id: Id,
    pub opens: bool,
    pub closes: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WaySource {
    #[serde(rename = "premise")]
    Premise,
    #[serde(rename = "family rule")]
    FamilyRule,
    #[serde(rename = "ally")]
    Ally,
    #[serde(rename = "sacrifice")]
    Sacrifice,
    #[serde(rename = "code_break")]
    CodeBreak,
}

impl Default for WaySource {
    fn default() -> Self {
        WaySource::Premise
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Way {
    pub text: String,
    /// take / persuade / examine / question / provoke / … (the family's kinds)
    pub kind: String,
    pub source: WaySource,
    pub requires: Option<String>,
    pub contest: Option<bool>,
    pub names_slot: Option<String>,
    pub price: Option<String>,
    /// The plant fact this way pays (rulebook, *Plant-backed ways roll first*).
    pub pays: Option<Id>,
    pub code_forbidden: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RemovedWay {
    pub way: String,
    pub reason: String,
}

/// An ending: a label, or a label with the state it writes on a person.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Ending {
    Plain(String),
    WithState { text: String, state: EndingState },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EndingState {
    pub person: String,
    pub state: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlantType {
    #[serde(rename = "shared")]
    Shared,
    #[serde(rename = "unique")]
    Unique,
    #[serde(rename = "ally")]
    Ally,
    #[serde(rename = "fault line")]
    FaultLine,
    #[serde(rename = "lead")]
    Lead,
    #[serde(rename = "harm")]
    Harm,
    #[serde(rename = "quiet")]
    Quiet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plant {
    pub fact: Id,
    #[serde(rename = "type")]
    pub kind: PlantType,
    #[serde(default)]
    pub document: Option<Value>,
    /// The slot role who can attest it.
    #[serde(default)]
    pub knower: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Opening,
    Middle,
    Turn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Routine,
    Reveal,
    Story,
    Crisis,
    Epilogue,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GuildMeeting {
    pub guild: Id,
    pub person: Id,
    pub tie: Option<RelationKind>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Advantage {
    pub fact: Id,
    pub for_node: Id,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EchoOf {
    pub person: Id,
    pub from_card: Id,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LinkedPair {
    pub with: Id,
    pub order: u8,
}

/// One quest card. See `claude/quest-card-schema.md` §2 for every field.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Card {
    // Identity and placement
    pub id: Id,
    pub family: Family,
    pub premise: String,
    pub variant: Option<String>,
    pub relatedness: Relatedness,
    pub scope: Scope,
    pub place: Option<Id>,
    pub offered_at: Option<Id>,
    pub giver: Option<Id>,
    pub giver_faction: Option<Id>,
    pub signature: bool,
    pub clock: Option<Clock>,
    /// How the card is tied to its place's trade or hook (locations, place ties).
    pub place_tie: Option<String>,
    pub theme: Option<String>,

    // Who
    pub slots: BTreeMap<String, SlotValue>,
    pub opposing_slot: Option<String>,
    pub fallbacks: BTreeMap<String, Id>,
    pub reasons: BTreeMap<String, String>,
    pub motive: Option<String>,
    pub factions: Vec<CardFaction>,

    // What
    pub truth: Option<Id>,
    pub pressure: Option<String>,
    pub pressure_level: Option<u8>,
    pub separation: Option<String>,
    pub twist: Option<Twist>,
    pub open_threads: Vec<ThreadMark>,

    // Ways through
    pub ways_pool: Vec<Way>,
    pub ways_rolled: Vec<usize>,
    pub ways_removed: Vec<RemovedWay>,

    // Endings and knowledge
    pub possible_endings: Vec<Ending>,
    pub plants: Vec<Plant>,
    /// Family-specific fields (shield, demand, red_herrings, contest terms…).
    pub details: BTreeMap<String, Value>,
    pub depends_on: Vec<Id>,

    // Main-story fields
    pub chapter: Option<Id>,
    pub position: Option<Position>,
    pub cast_present: Vec<Id>,
    pub proxies_present: Vec<Id>,
    #[serde(rename = "move")]
    pub move_id: Option<Id>,
    pub door: Option<String>,
    pub landmark: Option<bool>,

    // Faction-story fields
    pub faction_story: Option<Id>,
    pub phase: Option<Phase>,
    pub odd_job: Option<bool>,
    pub private_side: Option<bool>,
    pub faction_endings: Option<Vec<String>>,
    pub guild_meeting: Option<GuildMeeting>,
    pub shared_town: Option<bool>,

    // Side-quest fields
    pub attached_to: Option<Id>,
    pub advantage: Option<Advantage>,
    pub source_move: Option<Id>,
    pub move_fills: Option<String>,
    pub face: Option<String>,
    pub aftershock_of: Option<Id>,
    pub checkpoint: Option<String>,
    pub echo_of: Option<EchoOf>,
    pub linked_pair: Option<LinkedPair>,
}

impl Default for Card {
    fn default() -> Self {
        Card {
            id: Id::new(),
            family: Family::Rescue,
            premise: String::new(),
            variant: None,
            relatedness: Relatedness::Independent,
            scope: Scope::Community,
            place: None,
            offered_at: None,
            giver: None,
            giver_faction: None,
            signature: false,
            clock: None,
            place_tie: None,
            theme: None,
            slots: BTreeMap::new(),
            opposing_slot: None,
            fallbacks: BTreeMap::new(),
            reasons: BTreeMap::new(),
            motive: None,
            factions: Vec::new(),
            truth: None,
            pressure: None,
            pressure_level: None,
            separation: None,
            twist: None,
            open_threads: Vec::new(),
            ways_pool: Vec::new(),
            ways_rolled: Vec::new(),
            ways_removed: Vec::new(),
            possible_endings: Vec::new(),
            plants: Vec::new(),
            details: BTreeMap::new(),
            depends_on: Vec::new(),
            chapter: None,
            position: None,
            cast_present: Vec::new(),
            proxies_present: Vec::new(),
            move_id: None,
            door: None,
            landmark: None,
            faction_story: None,
            phase: None,
            odd_job: None,
            private_side: None,
            faction_endings: None,
            guild_meeting: None,
            shared_town: None,
            attached_to: None,
            advantage: None,
            source_move: None,
            move_fills: None,
            face: None,
            aftershock_of: None,
            checkpoint: None,
            echo_of: None,
            linked_pair: None,
        }
    }
}

impl Card {
    /// Every entry id in the card's slots, with its role.
    pub fn slot_ids(&self) -> Vec<(&str, &Id)> {
        let mut out = Vec::new();
        for (role, v) in &self.slots {
            for id in v.ids() {
                out.push((role.as_str(), id));
            }
        }
        out
    }

    /// The rolled ways, in pool order.
    pub fn rolled_ways(&self) -> Vec<&Way> {
        self.ways_rolled
            .iter()
            .filter_map(|&i| self.ways_pool.get(i))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// The main story
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StoryArc {
    Rising,
    Ripple,
    Inward,
    Deep,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Shape {
    Above,
    Beside,
    Within,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StoryCast {
    pub antagonist: Option<Id>,
    pub want: Option<Want>,
    pub stake: Option<Id>,
    pub heart: Option<Id>,
    pub fallback: Option<Id>,
    pub hand: Option<Id>,
    pub lieutenant: Option<Id>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FaultLine {
    pub person: Option<Id>,
    pub sub: Option<Id>,
    #[serde(rename = "virtue violated")]
    pub virtue_violated: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StoryFactions {
    pub antagonist_side: Option<Id>,
    pub ally: Option<Id>,
    pub fault_line: Option<FaultLine>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Revelation {
    pub fact: Id,
    /// Faction stories: the hidden hand / the cost / the old debt.
    pub kind: Option<String>,
    pub stored_on: Option<Id>,
}

/// How a move plays out (main-quest, *Wins stick*).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MoveFaces {
    pub plain: Option<String>,
    pub despite_you: Option<String>,
    pub target: Option<Id>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Move {
    pub id: Id,
    pub layer: Option<u32>,
    pub fact: Id,
    pub scope: Option<Scope>,
    /// debt, law, rumour, theft, force, bribery, blackmail, kin, faith, fire
    pub lever: Option<String>,
    pub faces: Option<MoveFaces>,
    /// harm / move / ordeal event / last move
    pub kind: String,
    pub harm_type: Option<String>,
    /// captive / disgraced / estranged
    pub event: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StoryLayer {
    pub layer: u32,
    pub role: String,
    pub scope: Option<Scope>,
    #[serde(rename = "node ids")]
    pub node_ids: Vec<Id>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Node {
    pub id: Id,
    pub layer: u32,
    pub role: String,
    pub chapter: Option<Id>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Edge {
    pub from: Id,
    pub to: Id,
    /// A cast id, proxy id or fact id carried from one node to the next.
    pub thread: Option<Id>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct OpenThread {
    pub id: Id,
    /// A thing, person or fact id.
    pub what: Option<Id>,
    pub opened_by: Option<Id>,
    pub closed_by: Option<Id>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Chapter {
    pub id: Id,
    pub node: Option<Id>,
    pub role: Option<String>,
    /// A fact id.
    pub problem: Option<Id>,
    pub cards: Vec<Id>,
    pub region: Option<Id>,
    pub settlement: Option<Id>,
    /// Ripple trials only: profit / resist / already won.
    pub face: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Story {
    pub arc: StoryArc,
    pub shape: Shape,
    #[serde(rename = "L")]
    pub l: u32,
    #[serde(rename = "W")]
    pub w: u32,
    pub cast: StoryCast,
    pub factions: StoryFactions,
    pub revelation: Option<Revelation>,
    pub moves: Vec<Move>,
    pub layers: Vec<StoryLayer>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub open_threads: Vec<OpenThread>,
    pub chapters: Vec<Chapter>,
    #[serde(rename = "return")]
    pub return_chapter: Option<Id>,
    /// Climax ending group → the Return's problem text.
    pub return_variants: BTreeMap<String, String>,
}

impl Default for Story {
    fn default() -> Self {
        Story {
            arc: StoryArc::Rising,
            shape: Shape::Above,
            l: 0,
            w: 0,
            cast: StoryCast::default(),
            factions: StoryFactions::default(),
            revelation: None,
            moves: Vec::new(),
            layers: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            open_threads: Vec::new(),
            chapters: Vec::new(),
            return_chapter: None,
            return_variants: BTreeMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Faction stories
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FactionCast {
    pub enemy: Option<Id>,
    pub shape: Option<Shape>,
    pub want: Option<Want>,
    pub stake: Option<Id>,
    pub heart: Option<Id>,
    pub heart_type: Option<String>,
    pub fallback: Option<Id>,
    pub hand: Option<Id>,
    pub leader: Option<Id>,
    pub fault_line: Option<Id>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FactionChapter {
    pub id: Id,
    pub cards: Vec<Id>,
    pub problem: Option<Id>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Phases {
    pub routine: Vec<Id>,
    pub reveal: Option<Id>,
    pub chapters: Vec<FactionChapter>,
    pub crisis: Option<Id>,
    pub epilogue: Option<Id>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CodeEnding {
    pub ending: String,
    /// A condition, or "always".
    pub when: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Placement {
    pub region: Option<Id>,
    pub settlement: Option<Id>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Meeting {
    pub card: Id,
    pub guild: Id,
    pub tie: Option<RelationKind>,
    pub person: Option<Id>,
    pub role: Option<String>,
    pub site: Option<Id>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FactionStory {
    pub id: Id,
    pub faction: Id,
    /// Kit entry ids with variants (`L5b`), by pool.
    pub kit_rolls: BTreeMap<String, Value>,
    pub cast: FactionCast,
    pub revelation: Option<Revelation>,
    pub moves: Vec<Move>,
    pub phases: Phases,
    pub faction_endings: Vec<String>,
    pub code_endings: Vec<CodeEnding>,
    /// The one person or public plant shared with the main story.
    pub crossing: Option<Value>,
    pub paired_with: Option<Id>,
    /// Card position (card id or phase name) → region and settlement.
    pub placement: BTreeMap<String, Placement>,
    pub meetings: Vec<Meeting>,
}

// ---------------------------------------------------------------------------
// The map summary (input from the land generator)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MapSummary {
    pub regions: Vec<MapRegion>,
    pub territories: Vec<Territory>,
    pub places: Vec<MapPlace>,
    /// Each road is a list of place ids it joins, in order.
    pub roads: Vec<Vec<Id>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MapRegion {
    pub id: Id,
    pub name: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Territory {
    pub id: Id,
    /// The faction that holds it.
    pub holder: Option<Id>,
    pub sub: Option<Id>,
}

/// A settlement, presence or landmark on the map, with its quest quota.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MapPlace {
    pub id: Id,
    pub name: String,
    pub kind: String,
    /// great / town / village / presence / site …
    pub size: String,
    pub territory: Option<Id>,
    pub region: Option<Id>,
    /// Map position, in days of travel.
    pub x: f32,
    pub y: f32,
    pub standing: Option<Standing>,
    pub canon: bool,
    pub owner_faction: Option<Id>,
    /// Least and most side quests the location pass gives it.
    pub quota: [u32; 2],
    /// Places within about a day.
    pub near: Vec<Id>,
    pub presences_in_town: Vec<String>,
    pub presence_of: Option<Id>,
    pub landmark: Option<String>,
    pub trade: Option<String>,
    pub hook: Option<String>,
    /// Hidden places (the Womb): never on a card.
    pub secret: bool,
}
