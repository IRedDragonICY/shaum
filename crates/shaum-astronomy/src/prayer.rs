//! Prayer Times Calculation Module.
//!
//! Calculates Fajr (Subuh), Imsak, and Maghrib times using astronomical algorithms.
//! Reuses the existing astronomy infrastructure (VSOP87, coordinate conversions).

use chrono::{DateTime, Duration, NaiveDate, Utc, TimeZone, Datelike};
use shaum_types::{GeoCoordinate, PrayerParams};
use super::{vsop87, coords};
use super::visibility::{datetime_to_jd, estimate_sunset};

/// Prayer times for a specific date and location.
#[derive(Debug, Clone)]
pub struct PrayerTimes {
    /// Imsak time (end of Suhur, start of fasting).
    pub imsak: DateTime<Utc>,
    /// Fajr/Subuh time (beginning of dawn prayer).
    pub fajr: DateTime<Utc>,
    /// Maghrib time (sunset, end of fasting).
    pub maghrib: DateTime<Utc>,
}

/// Finds the time when the sun reaches a specific altitude using binary search.
///
/// # Arguments
/// * `date` - The date to calculate for
/// * `coords` - Observer's geographic coordinates
/// * `target_altitude` - Target sun altitude in degrees (negative for below horizon)
/// * `is_morning` - True to search for morning event, false for evening
///
/// # Returns
/// The UTC time when sun altitude crosses the target value.
fn find_sun_altitude_time(
    date: NaiveDate,
    coords: GeoCoordinate,
    target_altitude: f64,
    is_morning: bool,
) -> Result<DateTime<Utc>, shaum_types::ShaumError> {
    use shaum_types::ShaumError;
    
    // Calculate timezone offset from longitude (approximate: 15° = 1 hour)
    // For Yogyakarta (lng=110.36), offset ≈ +7.36 hours
    // So local midnight = UTC - offset
    let tz_offset_hours = coords.lng / 15.0;
    let tz_offset_minutes = (tz_offset_hours * 60.0).round() as i64;
    
    // Base datetime at local midnight (converted to UTC)
    // Local 00:00 = UTC 00:00 - tz_offset
    let base_utc_midnight = Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
        .single()
        .ok_or_else(|| ShaumError::AstronomyError("Invalid date for prayer time calculation".to_string()))?;
    
    // Shift to local midnight in UTC
    let local_midnight_utc = base_utc_midnight - Duration::minutes(tz_offset_minutes);
    
    let (mut low, mut high) = if is_morning {
        // Search from local midnight to local noon (in UTC)
        // For Yogyakarta: local 00:00-12:00 = UTC 17:00 (prev day) to 05:00
        (local_midnight_utc, local_midnight_utc + Duration::hours(12))
    } else {
        // Search from local noon to local midnight (in UTC)
        (local_midnight_utc + Duration::hours(12), local_midnight_utc + Duration::hours(24))
    };

    // Binary search with 20 iterations (~1 second precision)
    for _ in 0..20 {
        let mid = low + Duration::seconds((high - low).num_seconds() / 2);
        let jd = datetime_to_jd(mid);
        
        let (sun_lon, sun_lat, _) = vsop87::calculate(jd);
        let obliquity = coords::mean_obliquity(jd);
        let (sun_ra, sun_dec) = coords::ecliptic_to_equatorial(sun_lon, sun_lat, obliquity);
        let lst = coords::local_sidereal_time(jd, coords.lng);
        let (_, sun_alt) = coords::equatorial_to_horizontal(sun_ra, sun_dec, lst, coords.lat);

        if is_morning {
            // For morning: sun altitude increases, search for when it crosses from below
            if sun_alt < target_altitude {
                low = mid;
            } else {
                high = mid;
            }
        } else {
            // For evening: sun altitude decreases, search for when it crosses from above
            if sun_alt > target_altitude {
                low = mid;
            } else {
                high = mid;
            }
        }
    }

    // Return midpoint of final range
    Ok(low + Duration::seconds((high - low).num_seconds() / 2))
}

/// Calculates prayer times for a given date and location.
///
/// # Arguments
/// * `date` - The Gregorian date
/// * `coords` - Geographic coordinates (latitude, longitude)
/// * `params` - Prayer calculation parameters (Fajr angle, Imsak buffer)
///
/// # Returns
/// `Result<PrayerTimes, ShaumError>` containing Imsak, Fajr, and Maghrib times in UTC.
///
/// # Errors
/// Returns `ShaumError::AstronomyError` for polar regions (|lat| > 66.5°).
///
/// # Example
/// ```rust
/// use chrono::NaiveDate;
/// use shaum_types::{GeoCoordinate, PrayerParams};
/// use shaum_astronomy::prayer::calculate_prayer_times;
///
/// let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
/// let jakarta = GeoCoordinate::new(-6.2088, 106.8456).unwrap();
/// let params = PrayerParams::default(); // MABIMS: -20°, 10 min
///
/// let times = calculate_prayer_times(date, jakarta, &params).unwrap();
/// println!("Fajr: {}", times.fajr);
/// println!("Maghrib: {}", times.maghrib);
/// ```


pub fn calculate_prayer_times(
    date: NaiveDate,
    coords: GeoCoordinate,
    params: &PrayerParams,
) -> Result<PrayerTimes, shaum_types::ShaumError> {
    use shaum_types::ShaumError;
    
    // Polar region check - prayer times undefined
    if coords.lat.abs() > 66.5 {
        return Err(ShaumError::AstronomyError(
            format!("Polar region latitude {:.2}° not supported for prayer times", coords.lat)
        ));
    }
    
    // Fajr calculation (raw)
    let fajr_raw = find_sun_altitude_time(date, coords, params.fajr_angle, true)?;
    
    // Maghrib calculation (raw)
    // Note: Use 0 altitude here, estimate_sunset handles horizon dip internally if using _with_altitude
    // But since estimate_sunset is hardcoded for -0.833, we use it directly.
    // Ideally update this to use altitude if available in coords (need z-coord support)
    let maghrib_raw = estimate_sunset(date, coords)?;

    // Apply Ihtiyat and Rounding
    let fajr = apply_ihtiyat_and_round(
        fajr_raw, 
        params.ihtiyat_minutes, 
        params.rounding_granularity_seconds
    );
    
    let maghrib = apply_ihtiyat_and_round(
        maghrib_raw, 
        params.ihtiyat_minutes, 
        params.rounding_granularity_seconds
    );
    
    // Imsak: Calculated from RAW Fajr, subtracted buffer, then rounded
    // Why raw? Because buffer is relative to astronomical phenomenon, then we apply rounding/ihtiyat
    // But commonly Imsak matches Fajr logic.
    // Let's follow standard: (Fajr_Raw - Buffer) + Ihtiyat -> Round
    let imsak_raw = fajr_raw - Duration::minutes(params.imsak_buffer_minutes);
    let imsak = apply_ihtiyat_and_round(
        imsak_raw, 
        params.ihtiyat_minutes, 
        params.rounding_granularity_seconds
    );

    Ok(PrayerTimes { imsak, fajr, maghrib })
}

/// Helper to apply Ihtiyat and rounding
fn apply_ihtiyat_and_round(
    dt: DateTime<Utc>, 
    ihtiyat_min: i64, 
    granularity_sec: i64
) -> DateTime<Utc> {
    // 1. Add Ihtiyat (safety margin)
    let dt_with_safety = dt + Duration::minutes(ihtiyat_min);
    
    // 2. Round up to next granularity (ceiling)
    if granularity_sec <= 1 {
        return dt_with_safety;
    }
    
    let timestamp = dt_with_safety.timestamp();
    let remainder = timestamp % granularity_sec;
    
    if remainder == 0 {
        dt_with_safety
    } else {
        // Round up
        dt_with_safety + Duration::seconds(granularity_sec - remainder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_prayer_times_jakarta() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let jakarta = GeoCoordinate::new_unchecked(-6.2088, 106.8456);
        let params = PrayerParams::default();

        let times = calculate_prayer_times(date, jakarta, &params).unwrap();

        // Fajr should be before Maghrib
        assert!(times.fajr < times.maghrib);
        // Imsak should be before Fajr
        assert!(times.imsak < times.fajr);
        // Fajr should be in the morning (before noon UTC)
        assert!(times.fajr.hour() < 12 || times.fajr.hour() > 20); // Jakarta is UTC+7
    }

    #[test]
    fn test_prayer_times_mecca() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let mecca = GeoCoordinate::new_unchecked(21.4225, 39.8262);
        let params = PrayerParams::mwl(); // -18°

        let times = calculate_prayer_times(date, mecca, &params).unwrap();

        assert!(times.imsak < times.fajr);
        assert!(times.fajr < times.maghrib);
    }

    #[test]
    fn test_imsak_buffer() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let coords = GeoCoordinate::new_unchecked(0.0, 106.0);
        
        let params_10 = PrayerParams::new(-20.0, 10);
        let params_15 = PrayerParams::new(-20.0, 15);

        let times_10 = calculate_prayer_times(date, coords, &params_10).unwrap();
        let times_15 = calculate_prayer_times(date, coords, &params_15).unwrap();

        // 15 min buffer should be 5 min earlier than 10 min buffer
        let diff = (times_10.imsak - times_15.imsak).num_minutes();
        assert_eq!(diff, 5);
    }

    #[test]
    fn test_polar_region_returns_error() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let arctic = GeoCoordinate::new_unchecked(70.0, 25.0);
        let params = PrayerParams::default();

        let result = calculate_prayer_times(date, arctic, &params);
        assert!(result.is_err());
    }
}
