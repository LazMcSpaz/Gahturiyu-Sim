//! The simulation. Nothing in here knows a window exists: it can run headless,
//! be tested, and be stepped as fast as the machine allows.

pub mod ai;
pub mod bands;
pub mod body;
pub mod combat;
pub mod fights;
pub mod geo;
pub mod group;
pub mod inventory;
pub mod items;
pub mod magic;
pub mod names;
pub mod person;
pub mod race;
pub mod rng;
pub mod routes;
pub mod settlement;
pub mod squad;
pub mod stats;
pub mod terrain;
pub mod world;
pub mod worldgen;

pub use world::World;
