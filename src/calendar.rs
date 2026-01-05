use chrono::{Duration, Datelike, NaiveDate};
use hijri_date::HijriDate;
use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ShaumError {
    #[error("Date out of supported Hijri range (1938-2076)")]
    HijriConversionError,
}

/// Converts a Gregorian date to Hijri with a manual day adjustment.
/// 
/// # Arguments
/// * `date` - The Gregorian date.
/// * `adjustment` - Day offset (e.g., +1 or -1) to account for moon sighting discrepancies.
///   A positive adjustment means the Hijri calendar is ahead (moon seen earlier).
pub fn to_hijri(date: NaiveDate, adjustment: i64) -> Result<HijriDate, ShaumError> {
    let adjusted_date = date + Duration::days(adjustment);
    HijriDate::from_gr(
        adjusted_date.year() as usize, 
        adjusted_date.month() as usize, 
        adjusted_date.day() as usize
    ).map_err(|_| ShaumError::HijriConversionError)
}

/// Helper to get Hijri month name or index if needed.
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
