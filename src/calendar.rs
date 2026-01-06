use chrono::{Duration, Datelike, NaiveDate};
use hijri_date::HijriDate;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use std::cell::RefCell;

/// Minimum Gregorian year for Hijri conversion.
pub const HIJRI_MIN_YEAR: i32 = 1938;
/// Maximum Gregorian year for Hijri conversion.
pub const HIJRI_MAX_YEAR: i32 = 2076;

/// Errors from shaum operations.
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ShaumError {
    /// Date outside supported range (1938-2076).
    #[error("Date {date} is out of supported range ({min} to {max})")]
    DateOutOfRange {
        date: NaiveDate,
        min: NaiveDate,
        max: NaiveDate,
    },
    
    /// Invalid configuration.
    #[error("Invalid configuration: {reason}")]
    InvalidConfiguration { reason: String },
    
    /// Analysis failure.
    #[error("Analysis failed: {0}")]
    AnalysisError(String),

    /// Hijri conversion error.
    #[error("Hijri conversion failed: {0}")]
    HijriConversionError(String),

    /// Sunset calculation error.
    #[error("Sunset calculation failed: {0}")]
    SunsetCalculationError(String),

    /// Moon provider error.
    #[error("Moon provider error: {0}")]
    MoonProviderError(String),

    /// Network error (async/remote operations).
    #[cfg(feature = "async")]
    #[error("Network error: {0}")]
    NetworkError(String),
}

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

// Thread-local cache: (gregorian, adjustment) -> (hijri_year, month, day)
// Only stores successful conversions.
thread_local! {
    static HIJRI_CACHE: RefCell<Option<(NaiveDate, i64, usize, usize, usize)>> = const { RefCell::new(None) };
}

/// Converts Gregorian to Hijri with adjustment, clamping if out of range.
///
/// Returns `Result<HijriDate, ShaumError>` instead of unwrapping.
///
/// # Arguments
/// * `date` - Gregorian date
/// * `adjustment` - Day offset for moon sighting (positive = Hijri ahead)
pub fn to_hijri(date: NaiveDate, adjustment: i64) -> Result<HijriDate, ShaumError> {
    // Check cache
    let cached = HIJRI_CACHE.with(|cache| {
        cache.borrow().as_ref().and_then(|(d, adj, y, m, day)| {
            if *d == date && *adj == adjustment {
                Some((*y, *m, *day))
            } else {
                None
            }
        })
    });
    
    if let Some((y, m, d)) = cached {
        // We assume cached values are valid. If not, we have a bigger problem (memory corruption or logic bug).
        // But since we need to return Result, we just wrap it.
        return HijriDate::from_hijri(y, m, d).map_err(|e| ShaumError::HijriConversionError(e.to_string()));
    }
    
    let adjusted_date = date + Duration::days(adjustment);
    
    // Check bounds strictly.
    let year = adjusted_date.year();
    if year < HIJRI_MIN_YEAR || year > HIJRI_MAX_YEAR {
       return Err(ShaumError::date_out_of_range(adjusted_date));
    }

    // HijriDate::from_gr is fallible.
    let hijri = HijriDate::from_gr(
        adjusted_date.year() as usize, 
        adjusted_date.month() as usize, 
        adjusted_date.day() as usize
    ).map_err(|e| ShaumError::HijriConversionError(e.to_string()))?;
    
    // Update cache
    HIJRI_CACHE.with(|cache| {
        *cache.borrow_mut() = Some((date, adjustment, hijri.year(), hijri.month(), hijri.day()));
    });
    
    Ok(hijri)
}

/// Returns Hijri month name.
pub fn get_hijri_month_name(month: usize) -> &'static str {
    match month {
        1 => "Muharram",
        2 => "Safar",
        3 => "Rabi' al-Awwal",
        4 => "Rabi' al-Thani",
        5 => "Jumada al-Ula",
        6 => "Jumada al-Akhirah",
        7 => "Rajab",
        8 => "Sha'ban",
        9 => "Ramadhan",
        10 => "Shawwal",
        11 => "Dhu al-Qi'dah",
        12 => "Dhu al-Hijjah",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
        let h1 = to_hijri(date, 0).unwrap();
        let h2 = to_hijri(date, 0).unwrap();
        assert_eq!(h1.day(), h2.day());
        assert_eq!(h1.month(), h2.month());
        assert_eq!(h1.year(), h2.year());
    }
    
    #[test]
    fn test_out_of_range() {
        // BEFORE min year
        let old_date = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        let err_old = to_hijri(old_date, 0);
        assert!(err_old.is_err());

        // AFTER max year
        let future_date = NaiveDate::from_ymd_opt(2100, 1, 1).unwrap();
        let err_fut = to_hijri(future_date, 0);
        assert!(err_fut.is_err());
    }
}
