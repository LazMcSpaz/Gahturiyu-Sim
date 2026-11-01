//! Gahturiyu: an always-alive world, simulated in bands around your squad.
//!
//! `sim` is the whole simulation and has no graphics dependencies.
//! The playtest window lives in `main.rs` and only ever reads from it.
//! `names` is the naming system: words and names in the four tongues.
//! `questgen` is the quest generator (no graphics): a world's quests as data.

pub mod names;
pub mod questgen;
pub mod sim;
