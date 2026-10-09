//! Gahturiyu: an always-alive world, simulated in bands around your squad.
//!
//! `sim` is the whole simulation and has no graphics dependencies.
//! The playtest window lives in `main.rs` and only ever reads from it.
//! `names` is the naming system: words and names in the four tongues.

pub mod names;
pub mod sim;
