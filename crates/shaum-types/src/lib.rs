//! Core types for Shaum - Islamic fasting rules engine.
//!
//! This crate contains pure type definitions with no business logic.

mod geo;
mod prayer;
mod status;
mod madhab;
mod analysis;
mod error;

pub use geo::{GeoCoordinate, VisibilityCriteria};
pub use prayer::PrayerParams;
pub use status::FastingStatus;
pub use madhab::{Madhab, DaudStrategy};
pub use analysis::{FastingType, FastingAnalysis, RuleTrace, TraceCode, TracePayload};
pub use error::ShaumError;
