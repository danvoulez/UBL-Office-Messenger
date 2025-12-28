//! Context Module
//!
//! Manages context frames, narrative generation, and memory strategies.

mod frame;
mod builder;
mod narrator;
mod memory;

pub use frame::{ContextFrame, ContextHash, Affordance, Obligation, ObligationStatus, GuardianInfo, FrameSummary};
pub use builder::ContextFrameBuilder;
pub use narrator::{Narrator, NarrativeConfig};
pub use memory::{Memory, MemoryStrategy, MemoryEntry, Bookmark, MemoryConfig, HistoricalSynthesis};
