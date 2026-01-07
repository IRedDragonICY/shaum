//! Fasting status enum (Hukum).

use serde::{Serialize, Deserialize};
use std::fmt;

/// Fasting status (Hukum). Ordered by priority: Haram > Wajib > SunnahMuakkadah > Sunnah > Makruh > Mubah.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FastingStatus {
    Mubah,
    Makruh,
    Sunnah,
    SunnahMuakkadah,
    Wajib,
    Haram,
}

impl FastingStatus {
    #[inline] pub fn is_haram(&self) -> bool { matches!(self, Self::Haram) }
    #[inline] pub fn is_wajib(&self) -> bool { matches!(self, Self::Wajib) }
    #[inline] pub fn is_sunnah(&self) -> bool { matches!(self, Self::Sunnah | Self::SunnahMuakkadah) }
    #[inline] pub fn is_makruh(&self) -> bool { matches!(self, Self::Makruh) }
    #[inline] pub fn is_mubah(&self) -> bool { matches!(self, Self::Mubah) }
}

impl fmt::Display for FastingStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Mubah => "Mubah (Permissible)",
            Self::Makruh => "Makruh (Disliked)",
            Self::Sunnah => "Sunnah (Recommended)",
            Self::SunnahMuakkadah => "Sunnah Muakkadah (Highly Recommended)",
            Self::Wajib => "Wajib (Obligatory)",
            Self::Haram => "Haram (Forbidden)",
        };
        write!(f, "{}", s)
    }
}
