//! # Shaum - Islamic Fasting Rules Engine
//!
//! Determines fasting status (Wajib, Sunnah, Makruh, Haram) for any date.
//!
//! ## Quick Start
//!
//! ```rust
//! use chrono::NaiveDate;
//! use shaum_core::prelude::*;
//!
//! let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
//!
//! // Full analysis
//! let analysis = date.fasting_analysis();
//! println!("{}", analysis.explain());
//! ```

// Re-export specific items from sub-crates to maintain API
pub use shaum_types::{
    FastingStatus, FastingType, FastingAnalysis, Madhab, DaudStrategy,
    GeoCoordinate, TraceCode, VisibilityCriteria, PrayerParams
};

pub use shaum_calendar::{to_hijri, ShaumError};

pub use shaum_rules::{
    analyze, check, RuleContext, MoonProvider, SunsetProvider, 
    DefaultSunsetProvider, FixedAdjustment, NoAdjustment,
    shaum_context, DaudIterator, generate_daud_schedule, DaudScheduleBuilder
};

// Re-export modules as if they were local (optional, but good for discovery)
pub mod types {
    pub use shaum_types::*;
}

pub mod calendar {
    pub use shaum_calendar::*;
}

pub mod astronomy {
    pub use shaum_astronomy::*;
}

pub mod rules {
    pub use shaum_rules::*;
}

pub mod extension {
    pub use shaum_rules::extension::*;
}

pub mod query {
    pub use shaum_rules::query::*;
}

#[cfg(feature = "shaum-network")]
pub mod network {
    pub use shaum_network::*;
}

/// Re-exports for convenience.
pub mod prelude {
    pub use shaum_types::{
        FastingStatus, FastingType, FastingAnalysis, Madhab, DaudStrategy,
        GeoCoordinate, TraceCode, VisibilityCriteria, PrayerParams
    };
    pub use shaum_calendar::{to_hijri, ShaumError, HijriDate};
    pub use shaum_rules::{
        analyze, check, RuleContext, MoonProvider, SunsetProvider,
        // Extension traits are re-exported by rules prelude or directly?
        // Let's re-export items used in prelude previously.
        FastingQuery, QueryExt, ShaumDateExt,
        shaum_context, DaudIterator, generate_daud_schedule
    };
    // Re-export deprecated or common functions
    pub use crate::analyze_date;
}

// Extension traits need to be accessible for .fasting_analysis() to work
pub use shaum_rules::{ShaumDateExt, FastingQuery, QueryExt};

use chrono::NaiveDate;

/// Analyzes date with default context. Returns Result for safe error handling.
pub fn analyze_date(date: NaiveDate) -> Result<FastingAnalysis, ShaumError> {
    check(date, &RuleContext::default())
}
