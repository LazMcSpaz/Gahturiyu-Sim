//! The quest generator: builds a world's quests from a seed, as data.
//!
//! It sits beside the simulation and has no graphics dependency. The design
//! lives in the claude.ai Project (`claude/quest-*.md`); the plain-language
//! overview is `docs/quest-generator.md`.
//!
//! - `data`: the contract. Every type the generator writes and the later
//!   layers (mechanics, dialogue, details) read, field for field as
//!   `claude/quest-card-schema.md` names them.
//!
//! Lane A (shape and mechanics) owns this module except `check/`, `digest`,
//! `detail` and `dialogue`, which are Lane B's (`claude/quest-parallel-plan.md`).

pub mod data;
