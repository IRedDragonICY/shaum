//! Hijri calendar conversion for Shaum.
//!
//! Provides Gregorian to Hijri date conversion with caching.

use chrono::{Duration, Datelike, NaiveDate};
use std::cell::RefCell;

pub use shaum_types::ShaumError;

/// Minimum Gregorian year for Hijri conversion.
pub const HIJRI_MIN_YEAR: i32 = 1938;
/// Maximum Gregorian year for Hijri conversion.
pub const HIJRI_MAX_YEAR: i32 = 2076;

// Thread-local cache: (gregorian, adjustment) -> (hijri_year, month, day)
thread_local! {
    static HIJRI_CACHE: RefCell<Option<(NaiveDate, i64, usize, usize, usize)>> = const { RefCell::new(None) };
}

/// Converts Gregorian to Hijri with adjustment.
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
        return HijriDate::from_hijri(y, m, d)
            .map_err(|e| ShaumError::HijriConversionError(e.to_string()));
    }
    
    let adjusted_date = date + Duration::days(adjustment);
    
    // Check bounds
    let year = adjusted_date.year();
    if year < HIJRI_MIN_YEAR || year > HIJRI_MAX_YEAR {
       return Err(ShaumError::date_out_of_range(adjusted_date));
    }

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
        1 => "Muharram", 2 => "Safar", 3 => "Rabi' al-Awwal", 4 => "Rabi' al-Thani",
        5 => "Jumada al-Ula", 6 => "Jumada al-Akhirah", 7 => "Rajab", 8 => "Sha'ban",
        9 => "Ramadhan", 10 => "Shawwal", 11 => "Dhu al-Qi'dah", 12 => "Dhu al-Hijjah",
        _ => "Unknown",
    }
}

// Re-export hijri_date crate and struct
pub use hijri_date;
pub use hijri_date::HijriDate;

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
        let old_date = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        assert!(to_hijri(old_date, 0).is_err());

        let future_date = NaiveDate::from_ymd_opt(2100, 1, 1).unwrap();
        assert!(to_hijri(future_date, 0).is_err());
    }
}
