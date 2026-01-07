//! VSOP87 Implementation for Sun's Position.
//!
//! We use the `vsop87` crate to calculate the Heliocentric coordinates of Earth (L, B, R)
//! and then convert to Geocentric coordinates of the Sun.
//!
//! Reference: Jean Meeus, "Astronomical Algorithms", Chapter 25 and 32.

use vsop87::vsop87d;

/// Calculates the Geocentric Ecliptic coordinates of the Sun.
///
/// Returns (Longitude (deg), Latitude (deg), Distance (AU)).
///
/// Steps:
/// 1. Calculate Earth's Heliocentric coordinates (L, B, R) using VSOP87D.
/// 2. Convert to Sun's Geocentric coordinates:
///    Lon_Sun = Lon_Earth + 180
///    Lat_Sun = -Lat_Earth
///    Dist_Sun = Dist_Earth
pub fn calculate(jd: f64) -> (f64, f64, f64) {
    // VSOP87D returns (x, y, z) usually? Or L, B, R?
    // The `vsop87` crate documentation says `vsop87d::earth(t)` returns (l, b, r) in radians and AU.
    // L: Heliocentric longitude (radians)
    // B: Heliocentric latitude (radians)
    // R: Radius vector (AU)
    
    let coords = vsop87d::earth(jd);
    
    let l_rad = coords.longitude();
    let b_rad = coords.latitude();
    let r_au = coords.distance();

    let mut lon_deg = l_rad.to_degrees() + 180.0;
    let lat_deg = -b_rad.to_degrees(); // Negate latitude
    let dist_au = r_au;

    lon_deg = (lon_deg % 360.0 + 360.0) % 360.0; // Normalize 0-360

    (lon_deg, lat_deg, dist_au)
}
