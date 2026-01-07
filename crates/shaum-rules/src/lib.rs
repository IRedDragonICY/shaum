//! Fasting rules engine for Shaum.
//!
//! Provides the core analysis engine, rule context, and DaudIterator.

pub mod rules;
pub mod extension;
pub mod query;
pub mod i18n;
pub mod macros;
pub mod constants;
pub mod daud_util;

// Re-export main items from rules module
pub use rules::{analyze, check, RuleContext, MoonProvider, SunsetProvider, DefaultSunsetProvider};
pub use rules::{FixedAdjustment, NoAdjustment};

pub use query::{FastingQuery, QueryExt};
pub use extension::ShaumDateExt;
pub use daud_util::{DaudIterator, generate_daud_schedule, DaudScheduleBuilder};
