//! Prayer Times Calculation Module.
//!
//! Calculates Fajr (Subuh), Imsak, and Maghrib times using astronomical algorithms.
//! Reuses the existing astronomy infrastructure (VSOP87, coordinate conversions).

use chrono::{DateTime, Duration, NaiveDate, Utc, TimeZone, Datelike, Timelike};
use crate::types::{GeoCoordinate, PrayerParams};
use super::{vsop87, coords};
use super::visibility::{datetime_to_jd, jd_to_datetime, estimate_sunset};

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
) -> DateTime<Utc> {
    // Initial search bounds
    let base_dt = Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0).unwrap();
    
    let (mut low, mut high) = if is_morning {
        // Search from midnight to noon for morning events
        (base_dt, base_dt + Duration::hours(12))
    } else {
        // Search from noon to midnight for evening events
        (base_dt + Duration::hours(12), base_dt + Duration::hours(24))
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
    low + Duration::seconds((high - low).num_seconds() / 2)
}

/// Calculates prayer times for a given date and location.
///
/// # Arguments
/// * `date` - The Gregorian date
/// * `coords` - Geographic coordinates (latitude, longitude)
/// * `params` - Prayer calculation parameters (Fajr angle, Imsak buffer)
///
/// # Returns
/// `PrayerTimes` containing Imsak, Fajr, and Maghrib times in UTC.
///
/// # Example
/// ```rust
/// use chrono::NaiveDate;
/// use shaum::types::{GeoCoordinate, PrayerParams};
/// use shaum::astronomy::prayer::calculate_prayer_times;
///
/// let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
/// let jakarta = GeoCoordinate::new(-6.2088, 106.8456);
/// let params = PrayerParams::default(); // MABIMS: -20°, 10 min
///
/// let times = calculate_prayer_times(date, jakarta, &params);
/// println!("Fajr: {}", times.fajr);
/// println!("Maghrib: {}", times.maghrib);
/// ```
pub fn calculate_prayer_times(
    date: NaiveDate,
    coords: GeoCoordinate,
    params: &PrayerParams,
) -> PrayerTimes {
    // Fajr: when sun altitude equals fajr_angle before sunrise
    let fajr = find_sun_altitude_time(date, coords, params.fajr_angle, true);
    
    // Imsak: fajr minus buffer
    let imsak = fajr - Duration::minutes(params.imsak_buffer_minutes);
    
    // Maghrib: reuse existing estimate_sunset
    let maghrib = estimate_sunset(date, coords);

    PrayerTimes { imsak, fajr, maghrib }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prayer_times_jakarta() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let jakarta = GeoCoordinate::new(-6.2088, 106.8456);
        let params = PrayerParams::default();

        let times = calculate_prayer_times(date, jakarta, &params);

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
        let mecca = GeoCoordinate::new(21.4225, 39.8262);
        let params = PrayerParams::mwl(); // -18°

        let times = calculate_prayer_times(date, mecca, &params);

        assert!(times.imsak < times.fajr);
        assert!(times.fajr < times.maghrib);
    }

    #[test]
    fn test_imsak_buffer() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let coords = GeoCoordinate::new(0.0, 106.0);
        
        let params_10 = PrayerParams::new(-20.0, 10);
        let params_15 = PrayerParams::new(-20.0, 15);

        let times_10 = calculate_prayer_times(date, coords, &params_10);
        let times_15 = calculate_prayer_times(date, coords, &params_15);

        // 15 min buffer should be 5 min earlier than 10 min buffer
        let diff = (times_10.imsak - times_15.imsak).num_minutes();
        assert_eq!(diff, 5);
    }
}
