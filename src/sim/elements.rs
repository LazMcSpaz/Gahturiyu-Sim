//! Fire, water and cold as lasting conditions, and how they meet.
//!
//! In a fight, fire sets people **burning** (hurting a little every second,
//! and lighting the ground round them like a torch); water leaves them
//! **wet**; cold builds up as **chill** until they're **frozen** stiff for a
//! moment. They meet:
//!
//! | | on someone wet | on someone burning | on someone chilled |
//! |---|---|---|---|
//! | Fire | steam: half the hurt, both gone | burns on | thaws them |
//! | Cold | chills twice as fast, hurts more | puts the fire out | builds the chill |
//! | Lightning | hurts half again as much | | |
//! | Water | stays wet | puts the fire out | |
//!
//! Frozen people are helpless, and brittle armour (Nacre, Slatewing) cracks
//! under every blow while they are. Pitch-sealed gear catches: fire hurts
//! more and the pitch burns away. Wet people can't light a torch.
//!
//! **What you carry.** For your squad, what burned or soaked in a fight is
//! settled when it ends: paper (scrolls, notes, manuals, banknotes, letters)
//! can burn or be ruined by water, and pitch-sealed pieces scorch. Wetness
//! goes out with them and dries by the clock. (Strangers restock at home, as
//! with everything else a fight uses up.)

use super::items::{item, ItemId, Kind};

/// How long a fire on someone keeps burning, and how much it hurts a second.
pub const BURN_SECS: f64 = 6.0;
pub const BURN_PER_SEC: f32 = 2.5;
/// How long it takes to dry off.
pub const WET_SECS: f64 = 600.0;
/// Lightning on the wet hurts this much more; cold sinks in this much faster.
pub const WET_SHOCK: f32 = 1.5;
pub const WET_CHILL: f32 = 2.0;
pub const WET_COLD: f32 = 1.3;
/// Chill (built from cold damage) that freezes someone, how long chill
/// lingers, and how long they're frozen.
pub const FREEZE_AT: f32 = 14.0;
pub const CHILL_SECS: f64 = 20.0;
pub const FROZEN_SECS: f64 = 3.0;
/// Fire on the wet: this share of the hurt gets through.
pub const STEAM: f32 = 0.5;
/// Fire on someone in pitch-sealed gear hurts this much more.
pub const PITCH_FIRE: f32 = 1.25;
/// Durability a sealed piece loses each second its wearer burns.
pub const PITCH_SCORCH: f32 = 3.0;
/// Chance each paper thing is lost for each second its carrier burned, and
/// once if they were soaked.
pub const PAPER_BURN: f32 = 0.08;
pub const PAPER_SOAK: f32 = 0.5;

/// Paper: it burns, and water ruins it.
pub fn is_paper(id: ItemId) -> bool {
    matches!(item(id).kind, Kind::Scroll(_) | Kind::Notes(_) | Kind::Text(_) | Kind::Manual(_)) || matches!(item(id).key, "reed_paper" | "note" | "sealed_letter")
}

/// Chance a paper thing survives what its carrier went through.
pub fn paper_survives(burned: f32, soaked: bool) -> f32 {
    (1.0 - PAPER_BURN).powf(burned) * if soaked { 1.0 - PAPER_SOAK } else { 1.0 }
}
