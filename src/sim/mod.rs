//! The simulation. Nothing in here knows a window exists: it can run headless,
//! be tested, and be stepped as fast as the machine allows.

pub mod ai;
pub mod animals;
pub mod bands;
pub mod body;
pub mod buildings;
pub mod carry;
pub mod casting;
pub mod combat;
pub mod condition;
pub mod crafting;
pub mod culture;
pub mod dialogue;
pub mod economy;
pub mod effects;
pub mod encounters;
pub mod fights;
pub mod geo;
pub mod group;
pub mod inventory;
pub mod jobs;
pub mod items;
pub mod magic;
pub mod names;
pub mod news;
pub mod person;
pub mod quests;
pub mod race;
pub mod rng;
pub mod routine;
pub mod routes;
pub mod save;
pub mod settlement;
pub mod society;
pub mod squad;
pub mod stats;
pub mod torch;
pub mod stealth;
pub mod terrain;
pub mod tide;
pub mod world;
pub mod worldgen;

pub use world::World;
