//! Geographic and visibility types.

use serde::{Serialize, Deserialize};

/// Geographic coordinates (Latitude, Longitude) with optional Altitude.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoCoordinate {
    pub lat: f64,
    pub lng: f64,
    /// Altitude above sea level in meters. Default: 0.0
    pub altitude: f64,
}

impl GeoCoordinate {
    /// Creates a new validated coordinate (altitude defaults to 0).
    ///
    /// Returns `Err(ShaumError::ValidationError)` if coordinates are out of range.
    pub fn new(lat: f64, lng: f64) -> Result<Self, crate::ShaumError> {
        if !(-90.0..=90.0).contains(&lat) {
            return Err(crate::ShaumError::ValidationError(
                format!("Latitude {} out of range [-90, 90]", lat)
            ));
        }
        if !(-180.0..=180.0).contains(&lng) {
            return Err(crate::ShaumError::ValidationError(
                format!("Longitude {} out of range [-180, 180]", lng)
            ));
        }
        Ok(Self { lat, lng, altitude: 0.0 })
    }

    /// Creates a coordinate without validation. Use with trusted inputs only.
    #[inline]
    pub const fn new_unchecked(lat: f64, lng: f64) -> Self {
        Self { lat, lng, altitude: 0.0 }
    }
    
    /// Sets the altitude (meters above sea level).
    pub fn with_altitude(mut self, altitude: f64) -> Self {
        self.altitude = altitude;
        self
    }
}

/// Configurable moon visibility criteria for hilal observation.
///
/// Controls the thresholds used when determining if the crescent moon
/// is visible. Default values match MABIMS (Indonesia/Malaysia/Brunei/Singapore).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VisibilityCriteria {
    /// Minimum moon altitude above horizon (degrees). Default: 3.0
    pub min_altitude: f64,
    /// Minimum elongation between sun and moon (degrees). Default: 6.4
    pub min_elongation: f64,
}

impl Default for VisibilityCriteria {
    fn default() -> Self {
        Self { min_altitude: 3.0, min_elongation: 6.4 }
    }
}

impl VisibilityCriteria {
    /// Creates new visibility criteria with custom thresholds.
    pub fn new(min_altitude: f64, min_elongation: f64) -> Self {
        Self { min_altitude, min_elongation }
    }

    /// MABIMS criteria (default for Southeast Asia).
    pub fn mabims() -> Self { Self::default() }

    /// Istanbul 1978 criteria (more conservative).
    pub fn istanbul_1978() -> Self {
        Self { min_altitude: 5.0, min_elongation: 8.0 }
    }
}
