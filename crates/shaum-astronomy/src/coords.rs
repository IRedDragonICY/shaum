//! Coordinate Conversions and Corrections.
//!
//! Implements:
//! - Ecliptic → Equatorial conversion
//! - Equatorial → Horizontal (Azimuth/Altitude) conversion
//! - Local Sidereal Time (LST)
//! - Topocentric Parallax Correction
//! - Atmospheric Refraction
//!
//! Reference: Jean Meeus, "Astronomical Algorithms", Chapters 13, 40.

use std::f64::consts::PI;

/// Degrees to radians.
const DEG_TO_RAD: f64 = PI / 180.0;
/// Radians to degrees.
const RAD_TO_DEG: f64 = 180.0 / PI;

/// Mean obliquity of the ecliptic for a given Julian Day (simplified formula).
/// 
/// Reference: Meeus, Eq. 22.2 (simplified)
pub fn mean_obliquity(jd: f64) -> f64 {
    let t = (jd - 2451545.0) / 36525.0;
    // Epsilon_0 in degrees
    23.439291 - 0.0130042 * t - 1.64e-7 * t * t + 5.04e-7 * t * t * t
}

/// Converts Ecliptic coordinates to Equatorial coordinates.
///
/// # Arguments
/// * `lon` - Ecliptic longitude (degrees)
/// * `lat` - Ecliptic latitude (degrees)
/// * `obliquity` - Obliquity of the ecliptic (degrees)
///
/// # Returns
/// (Right Ascension (degrees), Declination (degrees))
pub fn ecliptic_to_equatorial(lon: f64, lat: f64, obliquity: f64) -> (f64, f64) {
    let lon_rad = lon * DEG_TO_RAD;
    let lat_rad = lat * DEG_TO_RAD;
    let eps_rad = obliquity * DEG_TO_RAD;

    let sin_lon = lon_rad.sin();
    let cos_lon = lon_rad.cos();
    let sin_lat = lat_rad.sin();
    let cos_lat = lat_rad.cos();
    let sin_eps = eps_rad.sin();
    let cos_eps = eps_rad.cos();

    // Right Ascension (Meeus Eq. 13.3)
    let ra_rad = (sin_lon * cos_eps - sin_lat / cos_lat * sin_eps).atan2(cos_lon);
    // Declination (Meeus Eq. 13.4)
    let dec_rad = (sin_lat * cos_eps + cos_lat * sin_eps * sin_lon).asin();

    let mut ra_deg = ra_rad * RAD_TO_DEG;
    ra_deg = (ra_deg % 360.0 + 360.0) % 360.0; // Normalize 0-360

    (ra_deg, dec_rad * RAD_TO_DEG)
}

/// Calculates the Local Sidereal Time (LST) for a given JD and longitude.
///
/// # Arguments
/// * `jd` - Julian Day
/// * `longitude` - Observer's longitude (degrees, positive East)
///
/// # Returns
/// Local Sidereal Time in degrees (0-360).
///
/// Reference: Meeus, Chapter 12.
pub fn local_sidereal_time(jd: f64, longitude: f64) -> f64 {
    let t = (jd - 2451545.0) / 36525.0;
    
    // Greenwich Mean Sidereal Time at 0h UT (Meeus Eq. 12.4)
    let mut theta_0 = 280.46061837 
        + 360.98564736629 * (jd - 2451545.0) 
        + 0.000387933 * t * t 
        - t * t * t / 38710000.0;

    theta_0 = (theta_0 % 360.0 + 360.0) % 360.0;

    // Local Sidereal Time
    let lst = theta_0 + longitude;
    (lst % 360.0 + 360.0) % 360.0
}

/// Converts Equatorial coordinates to Horizontal (Azimuth/Altitude).
///
/// # Arguments
/// * `ra` - Right Ascension (degrees)
/// * `dec` - Declination (degrees)
/// * `lst` - Local Sidereal Time (degrees)
/// * `lat` - Observer's latitude (degrees)
///
/// # Returns
/// (Azimuth (degrees, measured from North, clockwise), Altitude (degrees))
///
/// Reference: Meeus, Chapter 13.
pub fn equatorial_to_horizontal(ra: f64, dec: f64, lst: f64, lat: f64) -> (f64, f64) {
    let h_rad = (lst - ra) * DEG_TO_RAD; // Hour Angle in radians
    let dec_rad = dec * DEG_TO_RAD;
    let lat_rad = lat * DEG_TO_RAD;

    let sin_h = h_rad.sin();
    let cos_h = h_rad.cos();
    let sin_dec = dec_rad.sin();
    let cos_dec = dec_rad.cos();
    let sin_lat = lat_rad.sin();
    let cos_lat = lat_rad.cos();

    // Altitude (Meeus Eq. 13.6)
    let alt_rad = (sin_lat * sin_dec + cos_lat * cos_dec * cos_h).asin();

    // Azimuth (Meeus Eq. 13.5) - measured from South, westward
    // We convert to measured from North, clockwise (standard convention)
    let az_rad = sin_h.atan2(cos_h * sin_lat - sin_dec / cos_dec * cos_lat);
    let mut az_deg = az_rad * RAD_TO_DEG + 180.0; // Shift from South to North
    az_deg = (az_deg % 360.0 + 360.0) % 360.0;

    (az_deg, alt_rad * RAD_TO_DEG)
}

/// Applies atmospheric refraction correction to the altitude.
///
/// Uses Bennett's formula (standard atmospheric conditions: 10°C, 1010 hPa).
///
/// # Arguments
/// * `apparent_alt` - Apparent altitude (degrees)
///
/// # Returns
/// Refraction correction in degrees (add to apparent altitude to get true altitude).
///
/// Reference: Meeus, Chapter 16.
pub fn refraction_correction(apparent_alt: f64) -> f64 {
    if apparent_alt < -1.0 {
        return 0.0; // Below horizon, no correction
    }
    
    // Bennett's formula (Meeus Eq. 16.4)
    // R = 1.02 / tan(h + 10.3 / (h + 5.11)) arcminutes
    // where h is in degrees
    let h = apparent_alt.max(0.0);
    let r_arcmin = 1.02 / ((h + 10.3 / (h + 5.11)) * DEG_TO_RAD).tan();
    r_arcmin / 60.0 // Convert arcminutes to degrees
}

/// Earth's equatorial radius in km.
const EARTH_RADIUS_KM: f64 = 6378.14;

/// Applies topocentric parallax correction to the Moon's equatorial coordinates.
///
/// This is significant for the Moon (~1 degree) due to its proximity.
///
/// # Arguments
/// * `ra` - Geocentric Right Ascension (degrees)
/// * `dec` - Geocentric Declination (degrees)
/// * `distance_km` - Distance to the Moon (km)
/// * `observer_lat` - Observer's latitude (degrees)
/// * `observer_elevation_m` - Observer's elevation above sea level (meters)
/// * `lst` - Local Sidereal Time (degrees)
///
/// # Returns
/// (Topocentric RA (degrees), Topocentric Declination (degrees))
///
/// Reference: Meeus, Chapter 40.
pub fn apply_parallax(
    ra: f64,
    dec: f64,
    distance_km: f64,
    observer_lat: f64,
    observer_elevation_m: f64,
    lst: f64,
) -> (f64, f64) {
    let lat_rad = observer_lat * DEG_TO_RAD;
    
    // Calculate geocentric observer's position (Meeus, Eq. 11.4, 11.5)
    // Using simplified spherical Earth model for now
    let _u = lat_rad.atan(); // Reduced latitude (simplified: same as lat for sphere)
    let rho_sin_phi = lat_rad.sin() + observer_elevation_m / 6378140.0 * lat_rad.sin();
    let rho_cos_phi = lat_rad.cos() + observer_elevation_m / 6378140.0 * lat_rad.cos();

    // Equatorial horizontal parallax (Meeus Eq. 40.1)
    let sin_pi = EARTH_RADIUS_KM / distance_km;
    
    let h_rad = (lst - ra) * DEG_TO_RAD; // Hour angle in radians
    let dec_rad = dec * DEG_TO_RAD;

    // Delta RA and Delta Dec (Meeus Eq. 40.2, 40.3)
    let delta_ra_rad = (-rho_cos_phi * sin_pi * h_rad.sin())
        .atan2(dec_rad.cos() - rho_cos_phi * sin_pi * h_rad.cos());
    
    let ra_prime = ra + delta_ra_rad * RAD_TO_DEG;
    
    // Topocentric declination
    let cos_dec = dec_rad.cos();
    let sin_dec = dec_rad.sin();
    let dec_prime_rad = (
        (sin_dec - rho_sin_phi * sin_pi) * delta_ra_rad.cos()
    ).atan2(cos_dec - rho_cos_phi * sin_pi * h_rad.cos());

    let ra_normalized = (ra_prime % 360.0 + 360.0) % 360.0;
    (ra_normalized, dec_prime_rad * RAD_TO_DEG)
}
