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
thread_local! {
    static HIJRI_CACHE: RefCell<Option<(NaiveDate, i64, usize, usize, usize)>> = const { RefCell::new(None) };
}

/// Converts Gregorian to Hijri with adjustment, clamping if out of range.
///
/// This function never fails. If the date is outside the supported Hijri range (1938-2076),
/// it is clamped to the nearest valid date (Standard Min: 1938-01-01, Max: 2076-12-31).
///
/// # Arguments
/// * `date` - Gregorian date
/// * `adjustment` - Day offset for moon sighting (positive = Hijri ahead)
pub fn to_hijri(date: NaiveDate, adjustment: i64) -> HijriDate {
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
        // Safe to unwrap here because cache is populated from valid HijriDate
        return HijriDate::from_hijri(y, m, d).unwrap_or_else(|_| 
             HijriDate::from_hijri(1356, 10, 29).unwrap() // Fallback literal
        );
    }
    
    let adjusted_date = date + Duration::days(adjustment);
    
    // Clamp year
    let year = adjusted_date.year();
    let clamped_date = if year < HIJRI_MIN_YEAR {
        NaiveDate::from_ymd_opt(HIJRI_MIN_YEAR, 1, 1).unwrap()
    } else if year > HIJRI_MAX_YEAR {
        NaiveDate::from_ymd_opt(HIJRI_MAX_YEAR, 12, 31).unwrap()
    } else {
        adjusted_date
    };

    // HijriDate::from_gr is technically fallible but largely safe within the range.
    // If it fails for some edge case even after clamping, we fall back safely.
    let hijri = HijriDate::from_gr(
        clamped_date.year() as usize, 
        clamped_date.month() as usize, 
        clamped_date.day() as usize
    ).unwrap_or_else(|_| {
        // Extreme fallback (1 Muharram 1357 approx 1938)
        HijriDate::from_hijri(1357, 1, 1).unwrap()
    });
    
    // Update cache
    HIJRI_CACHE.with(|cache| {
        *cache.borrow_mut() = Some((date, adjustment, hijri.year(), hijri.month(), hijri.day()));
    });
    
    hijri
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
        let h1 = to_hijri(date, 0);
        let h2 = to_hijri(date, 0);
        assert_eq!(h1.day(), h2.day());
        assert_eq!(h1.month(), h2.month());
        assert_eq!(h1.year(), h2.year());
    }
    
    #[test]
    fn test_clamping() {
        // BEFORE min year
        let old_date = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        let h_old = to_hijri(old_date, 0);
        // Should clamp to ~1356/1357 AH (around 1938)
        assert!(h_old.year() >= 1356);

        // AFTER max year
        let future_date = NaiveDate::from_ymd_opt(2100, 1, 1).unwrap();
        let h_fut = to_hijri(future_date, 0);
        // Should clamp to ~1499/1500 AH (around 2076)
        assert!(h_fut.year() <= 1500);
    }
}
