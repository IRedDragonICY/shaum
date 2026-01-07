//! Prayer time calculation parameters.

use serde::{Serialize, Deserialize};

/// Prayer time calculation parameters.
///
/// Controls angles and buffers used for prayer time calculations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PrayerParams {
    /// Sun altitude angle for Fajr (degrees below horizon). Default: -20.0 (MABIMS/Indonesia)
    pub fajr_angle: f64,
    /// Minutes to subtract from Fajr for Imsak. Default: 10
    pub imsak_buffer_minutes: i64,
    /// Safety margin (Ihtiyat) added to all prayer times. Default: 2 minutes
    pub ihtiyat_minutes: i64,
    /// Seconds to round prayer times to. Default: 60 (round to next minute)
    pub rounding_granularity_seconds: i64,
}

impl Default for PrayerParams {
    fn default() -> Self {
        Self {
            fajr_angle: -20.0,
            imsak_buffer_minutes: 10,
            ihtiyat_minutes: 2,
            rounding_granularity_seconds: 60,
        }
    }
}

impl PrayerParams {
    /// Creates new prayer parameters with defaults for Ihtiyat (2m) and rounding (60s).
    pub fn new(fajr_angle: f64, imsak_buffer_minutes: i64) -> Self {
        Self { 
            fajr_angle, 
            imsak_buffer_minutes,
            ihtiyat_minutes: 2,
            rounding_granularity_seconds: 60,
        }
    }
    
    /// Set Ihtiyat (safety margin) in minutes.
    pub fn with_ihtiyat(mut self, minutes: i64) -> Self {
        self.ihtiyat_minutes = minutes;
        self
    }
    
    /// Set rounding granularity in seconds.
    pub fn with_rounding(mut self, seconds: i64) -> Self {
        self.rounding_granularity_seconds = seconds;
        self
    }

    /// MABIMS/Indonesia standard (-20°, 10 min, +2 min Ihtiyat).
    pub fn mabims() -> Self { Self::default() }

    /// Egyptian General Authority (-19.5°, 10 min).
    pub fn egyptian() -> Self {
        Self { fajr_angle: -19.5, imsak_buffer_minutes: 10, ihtiyat_minutes: 2, rounding_granularity_seconds: 60 }
    }

    /// Muslim World League (-18°, 10 min).
    pub fn mwl() -> Self {
        Self { fajr_angle: -18.0, imsak_buffer_minutes: 10, ihtiyat_minutes: 2, rounding_granularity_seconds: 60 }
    }

    /// ISNA (North America) standard (-15°, 10 min).
    pub fn isna() -> Self {
        Self { fajr_angle: -15.0, imsak_buffer_minutes: 10, ihtiyat_minutes: 2, rounding_granularity_seconds: 60 }
    }

    /// Umm Al-Qura (Saudi Arabia) standard (-18.5°, 10 min).
    pub fn umm_al_qura() -> Self {
        Self { fajr_angle: -18.5, imsak_buffer_minutes: 10, ihtiyat_minutes: 2, rounding_granularity_seconds: 60 }
    }
}
