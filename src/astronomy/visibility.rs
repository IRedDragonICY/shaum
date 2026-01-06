//! Moon Visibility Metrics for Hilal Observation.
//!
//! Calculates all parameters needed to determine if the crescent moon is visible
//! according to established visibility criteria (e.g., MABIMS).
//!
//! Reference: Various Islamic astronomy sources, MABIMS criteria documentation.

use crate::astronomy::{vsop87, elp2000, coords};
use crate::types::{GeoCoordinate, VisibilityCriteria};
use chrono::{DateTime, Utc, Duration, Datelike, Timelike, TimeZone};



/// Report containing all visibility metrics for hilal observation.
#[derive(Debug, Clone)]
pub struct MoonVisibilityReport {
    /// Moon's altitude above horizon (degrees, refraction-corrected).
    pub moon_altitude: f64,
    /// Sun's altitude (degrees, typically negative at observation time).
    pub sun_altitude: f64,
    /// Angular separation between Sun and Moon (degrees).
    pub elongation: f64,
    /// Hours since last astronomical new moon (conjunction).
    pub moon_age_hours: f64,
    /// Minutes between sunset and moonset (positive = moon sets after sun).
    pub lag_time_minutes: f64,
    /// Whether MABIMS criteria are satisfied.
    pub meets_mabims: bool,
    /// Observation datetime (sunset time).
    pub observation_time: DateTime<Utc>,
}

/// Converts `DateTime<Utc>` to Julian Day.
pub fn datetime_to_jd(dt: DateTime<Utc>) -> f64 {
    // Algorithm from Meeus, Chapter 7.
    let year = dt.year();
    let month = dt.month() as i32;
    let day = dt.day() as f64 
        + dt.hour() as f64 / 24.0 
        + dt.minute() as f64 / 1440.0 
        + dt.second() as f64 / 86400.0;

    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };

    let a = (y as f64 / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();

    (365.25 * (y as f64 + 4716.0)).floor()
        + (30.6001 * (m as f64 + 1.0)).floor()
        + day
        + b
        - 1524.5
}

/// Converts Julian Day to `DateTime<Utc>` (approximate).
pub fn jd_to_datetime(jd: f64) -> DateTime<Utc> {
    // Inverse algorithm from Meeus, Chapter 7.
    let z = (jd + 0.5).floor();
    let f = jd + 0.5 - z;

    let alpha = ((z - 1867216.25) / 36524.25).floor();
    let a = z + 1.0 + alpha - (alpha / 4.0).floor();
    let b = a + 1524.0;
    let c = ((b - 122.1) / 365.25).floor();
    let d = (365.25 * c).floor();
    let e = ((b - d) / 30.6001).floor();

    let day = b - d - (30.6001 * e).floor() + f;
    let month = if e < 14.0 { e - 1.0 } else { e - 13.0 };
    let year = if month > 2.0 { c - 4716.0 } else { c - 4715.0 };

    let day_int = day.floor() as u32;
    let frac = day - day.floor();
    let hours = (frac * 24.0).floor() as u32;
    let minutes = ((frac * 24.0 - hours as f64) * 60.0).floor() as u32;
    let seconds = ((frac * 24.0 - hours as f64) * 60.0 - minutes as f64) * 60.0;

    chrono::Utc
        .with_ymd_and_hms(year as i32, month as u32, day_int, hours, minutes, seconds as u32)
        .unwrap()
}

/// Estimates the approximate sunset time for a given date and location.
///
/// Uses a simple algorithm to find when the Sun's altitude is approximately -0.833°
/// (accounting for atmospheric refraction and Sun's semi-diameter).
pub fn estimate_sunset(date: chrono::NaiveDate, coords: GeoCoordinate) -> DateTime<Utc> {
    // Start from an initial guess of 18:00 local time
    let initial_hour = 18.0 - coords.lng / 15.0; // Approximate UTC hour of local 18:00
    
    let base_dt = chrono::Utc
        .with_ymd_and_hms(date.year(), date.month(), date.day(), initial_hour as u32, 0, 0)
        .unwrap();
    
    // Iterative refinement (simple Newton-Raphson-like)
    let mut dt = base_dt;
    for _ in 0..5 {
        let jd = datetime_to_jd(dt);
        let (sun_lon, sun_lat, _) = vsop87::calculate(jd);
        let obliquity = coords::mean_obliquity(jd);
        let (sun_ra, sun_dec) = coords::ecliptic_to_equatorial(sun_lon, sun_lat, obliquity);
        let lst = coords::local_sidereal_time(jd, coords.lng);
        let (_, sun_alt) = coords::equatorial_to_horizontal(sun_ra, sun_dec, lst, coords.lat);
        
        // Target altitude for sunset: -0.833° (refraction + semi-diameter)
        let target_alt = -0.833;
        let alt_diff = sun_alt - target_alt;
        
        // Sun moves approximately 15°/hour, altitude change rate depends on latitude
        // Simplified: assume ~4 minutes per 1° altitude near sunset
        let time_correction_minutes = alt_diff * 4.0;
        
        if time_correction_minutes.abs() < 0.1 {
            break;
        }
        
        dt = dt + Duration::seconds((time_correction_minutes * 60.0) as i64);
    }
    
    dt
}

/// Calculates the approximate time of the last new moon (conjunction) before the given date.
///
/// Uses a simplified algorithm based on the Metonic cycle.
fn approximate_last_new_moon(dt: DateTime<Utc>) -> DateTime<Utc> {
    // Mean synodic month: 29.530588853 days
    const SYNODIC_MONTH: f64 = 29.530588853;
    
    // Known new moon reference: January 6, 2000, 18:14 UT (JD 2451550.26)
    const REF_JD: f64 = 2451550.26;
    
    let current_jd = datetime_to_jd(dt);
    let lunations_since_ref = (current_jd - REF_JD) / SYNODIC_MONTH;
    let last_new_moon_lunation = lunations_since_ref.floor();
    let last_new_moon_jd = REF_JD + last_new_moon_lunation * SYNODIC_MONTH;
    
    jd_to_datetime(last_new_moon_jd)
}

/// Calculates the elongation (angular separation) between Sun and Moon.
fn calculate_elongation(sun_lon: f64, sun_lat: f64, moon_lon: f64, moon_lat: f64) -> f64 {
    // Spherical law of cosines for angular distance
    let sun_lon_rad = sun_lon.to_radians();
    let sun_lat_rad = sun_lat.to_radians();
    let moon_lon_rad = moon_lon.to_radians();
    let moon_lat_rad = moon_lat.to_radians();
    
    let cos_elong = sun_lat_rad.sin() * moon_lat_rad.sin()
        + sun_lat_rad.cos() * moon_lat_rad.cos() * (moon_lon_rad - sun_lon_rad).cos();
    
    cos_elong.acos().to_degrees()
}

/// Calculates the visibility report for the Moon at a specific datetime and location.
///
/// This is the main entry point for hilal visibility determination.
///
/// # Arguments
/// * `datetime` - Observation datetime in UTC
/// * `coords` - Observer's geographic coordinates
/// * `criteria` - Visibility criteria thresholds (altitude, elongation)
pub fn calculate_visibility(
    datetime: DateTime<Utc>,
    coords: GeoCoordinate,
    criteria: &VisibilityCriteria,
) -> MoonVisibilityReport {
    let date = datetime.date_naive();
    
    // 1. Find sunset time for observation
    let sunset = estimate_sunset(date, coords);
    let jd = datetime_to_jd(sunset);
    
    // 2. Calculate Sun's position
    let (sun_lon, sun_lat, _sun_dist) = vsop87::calculate(jd);
    let obliquity = coords::mean_obliquity(jd);
    let (sun_ra, sun_dec) = coords::ecliptic_to_equatorial(sun_lon, sun_lat, obliquity);
    let lst = coords::local_sidereal_time(jd, coords.lng);
    let (_, sun_alt) = coords::equatorial_to_horizontal(sun_ra, sun_dec, lst, coords.lat);
    
    // 3. Calculate Moon's geocentric position
    let (moon_lon, moon_lat, moon_dist) = elp2000::calculate(jd);
    let (moon_ra, moon_dec) = coords::ecliptic_to_equatorial(moon_lon, moon_lat, obliquity);
    
    // 4. Apply topocentric parallax correction
    // Note: GeoCoordinate doesn't have elevation field currently, assume sea level
    let elevation_m = 0.0;
    let (moon_ra_topo, moon_dec_topo) = coords::apply_parallax(
        moon_ra, moon_dec, moon_dist, coords.lat, elevation_m, lst
    );
    
    // 5. Convert to horizontal coordinates
    let (_, moon_alt_apparent) = coords::equatorial_to_horizontal(
        moon_ra_topo, moon_dec_topo, lst, coords.lat
    );
    
    // 6. Apply atmospheric refraction
    let refraction = coords::refraction_correction(moon_alt_apparent);
    let moon_alt = moon_alt_apparent + refraction;
    
    // 7. Calculate elongation
    let elongation = calculate_elongation(sun_lon, sun_lat, moon_lon, moon_lat);
    
    // 8. Calculate moon age
    let last_new_moon = approximate_last_new_moon(sunset);
    let moon_age_hours = (sunset - last_new_moon).num_seconds() as f64 / 3600.0;
    
    // 9. Estimate lag time (simplified: difference in set times based on position)
    // For now, approximate using the difference in altitudes and typical motion
    let alt_diff = moon_alt - sun_alt;
    let lag_time_minutes = alt_diff * 4.0; // Rough estimate: 4 min per degree
    
    // 10. Check criteria
    let meets_mabims = moon_alt >= criteria.min_altitude && elongation >= criteria.min_elongation;
    
    MoonVisibilityReport {
        moon_altitude: moon_alt,
        sun_altitude: sun_alt,
        elongation,
        moon_age_hours,
        lag_time_minutes,
        meets_mabims,
        observation_time: sunset,
    }
}
