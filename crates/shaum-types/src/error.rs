//! Error types for Shaum.

use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use std::fmt;

/// Minimum Gregorian year for Hijri conversion.
pub const HIJRI_MIN_YEAR: i32 = 1938;
/// Maximum Gregorian year for Hijri conversion.
pub const HIJRI_MAX_YEAR: i32 = 2076;

/// Errors from shaum operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaumError {
    /// Date outside supported range (1938-2076).
    DateOutOfRange {
        date: NaiveDate,
        min: NaiveDate,
        max: NaiveDate,
    },
    
    /// Invalid configuration.
    InvalidConfiguration { reason: String },
    
    /// Analysis failure.
    AnalysisError(String),

    /// Hijri conversion error.
    HijriConversionError(String),

    /// Sunset calculation error.
    SunsetCalculationError(String),

    /// Moon provider error.
    MoonProviderError(String),

    /// Coordinate or input validation error.
    ValidationError(String),

    /// Astronomy calculation error (e.g., polar regions).
    AstronomyError(String),

    /// Database error (e.g., MaxMind GeoIP lookup).
    DatabaseError(String),

    /// Network error (async/remote operations).
    NetworkError(String),
}

impl fmt::Display for ShaumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DateOutOfRange { date, min, max } => {
                write!(f, "Date {} is out of supported range ({} to {})", date, min, max)
            }
            Self::InvalidConfiguration { reason } => write!(f, "Invalid configuration: {}", reason),
            Self::AnalysisError(s) => write!(f, "Analysis failed: {}", s),
            Self::HijriConversionError(s) => write!(f, "Hijri conversion failed: {}", s),
            Self::SunsetCalculationError(s) => write!(f, "Sunset calculation failed: {}", s),
            Self::MoonProviderError(s) => write!(f, "Moon provider error: {}", s),
            Self::ValidationError(s) => write!(f, "Validation error: {}", s),
            Self::AstronomyError(s) => write!(f, "Astronomy error: {}", s),
            Self::DatabaseError(s) => write!(f, "Database error: {}", s),
            Self::NetworkError(s) => write!(f, "Network error: {}", s),
        }
    }
}

impl std::error::Error for ShaumError {}

impl ShaumError {
    /// Creates a `DateOutOfRange` error with standard bounds.
    pub fn date_out_of_range(date: NaiveDate) -> Self {
        Self::DateOutOfRange {
            date,
            min: NaiveDate::from_ymd_opt(HIJRI_MIN_YEAR, 1, 1)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(1938, 1, 1).unwrap()),
            max: NaiveDate::from_ymd_opt(HIJRI_MAX_YEAR, 12, 31)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(2076, 12, 31).unwrap()),
        }
    }
    
    /// Creates an `InvalidConfiguration` error.
    pub fn invalid_config(reason: impl Into<String>) -> Self {
        Self::InvalidConfiguration { reason: reason.into() }
    }
}
