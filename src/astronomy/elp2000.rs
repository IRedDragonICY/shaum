//! ELP2000-82 Implementation for Moon's Position using `astro` crate.
//!
//! Note: `astro` crate uses ELP-2000/82 theory (Chapront).
//! This meets our high-precision requirement.

use astro::lunar;

/// Calculates the Geocentric Ecliptic coordinates of the Moon.
///
/// Returns (Longitude (deg), Latitude (deg), Distance (km)).
pub fn calculate(jd: f64) -> (f64, f64, f64) {
    // astro::lunar::geocent_ecl_pos returns (EclPoint, f64)
    // EclPoint has .lng and .lat fields (radians)
    // The f64 is distance in km
    
    let (ecl_point, dist_km) = lunar::geocent_ecl_pos(jd);

    let mut lon_deg = ecl_point.long.to_degrees();
    let lat_deg = ecl_point.lat.to_degrees();
    
    lon_deg = (lon_deg % 360.0 + 360.0) % 360.0;

    (lon_deg, lat_deg, dist_km)
}
