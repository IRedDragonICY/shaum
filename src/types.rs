


use serde::{Serialize, Deserialize};
use smallvec::SmallVec;
use std::fmt;

/// The legal status (Hukum) of fasting on a specific day.
/// 
/// Ordered by priority for conflict resolution:
/// Haram > Wajib > SunnahMuakkadah > Sunnah > Makruh > Mubah
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FastingStatus {
    // Priority 0 (Lowest)
    Mubah,
    // Priority 1
    Makruh,
    // Priority 2
    Sunnah,
    // Priority 3
    SunnahMuakkadah,
    // Priority 4
    Wajib,
    // Priority 5 (Highest)
    Haram,
}

impl FastingStatus {
    pub fn is_haram(&self) -> bool {
        matches!(self, FastingStatus::Haram)
    }

    pub fn is_wajib(&self) -> bool {
        matches!(self, FastingStatus::Wajib)
    }

    pub fn is_sunnah(&self) -> bool {
        matches!(self, FastingStatus::Sunnah | FastingStatus::SunnahMuakkadah)
    }

    pub fn is_makruh(&self) -> bool {
        matches!(self, FastingStatus::Makruh)
    }

    pub fn is_mubah(&self) -> bool {
        matches!(self, FastingStatus::Mubah)
    }
}

impl fmt::Display for FastingStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FastingStatus::Mubah => "Mubah (Permissible)",
            FastingStatus::Makruh => "Makruh (Disliked)",
            FastingStatus::Sunnah => "Sunnah (Recommended)",
            FastingStatus::SunnahMuakkadah => "Sunnah Muakkadah (Highly Recommended)",
            FastingStatus::Wajib => "Wajib (Obligatory)",
            FastingStatus::Haram => "Haram (Forbidden)",
        };
        write!(f, "{}", s)
    }
}

/// The specific reason or type of fasting associated with a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FastingType {
    /// The obligatory fasting during the month of Ramadhan.
    Ramadhan,
    /// The recommended fast on the Day of Arafah (9th Dhu al-Hijjah).
    Arafah,
    /// The recommended fast on the Day of Tasu'a (9th Muharram).
    Tasua,
    /// The recommended fast on the Day of Ashura (10th Muharram).
    Ashura,
    /// The "White Days" (13th, 14th, 15th of every Hijri month).
    AyyamulBidh,
    /// Sunnah fast on Mondays.
    Monday,
    /// Sunnah fast on Thursdays.
    Thursday,
    /// Six days of Sunnah fast in the month of Shawwal.
    Shawwal,
    /// Fasting every other day, as practiced by Prophet Daud (A.S).
    Daud,
    /// Forbidden fast on the day of Eid al-Fitr.
    EidAlFitr,
    /// Forbidden fast on the day of Eid al-Adha.
    EidAlAdha,
    /// Forbidden fast during the three days following Eid al-Adha.
    Tashriq,
    /// Disliked to fast on Friday alone without a specific reason/joining.
    FridayExclusive,
    /// Disliked to fast on Saturday alone without a specific reason/joining.
    SaturdayExclusive,
}

impl fmt::Display for FastingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FastingType::Ramadhan => "Ramadhan",
            FastingType::Arafah => "Day of Arafah",
            FastingType::Tasua => "Tasu'a (9th Muharram)",
            FastingType::Ashura => "Ashura (10th Muharram)",
            FastingType::AyyamulBidh => "Ayyamul Bidh (13th, 14th, 15th)",
            FastingType::Monday => "Monday",
            FastingType::Thursday => "Thursday",
            FastingType::Shawwal => "Six Days of Shawwal",
            FastingType::Daud => "Fasting of Prophet Daud (A.S)",
            FastingType::EidAlFitr => "Eid al-Fitr",
            FastingType::EidAlAdha => "Eid al-Adha",
            FastingType::Tashriq => "Days of Tashriq",
            FastingType::FridayExclusive => "Singling out Friday",
            FastingType::SaturdayExclusive => "Singling out Saturday",
        };
        write!(f, "{}", s)
    }
}

/// The four major Sunni schools of jurisprudence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Madhab {
    Shafi,
    Hanafi,
    Maliki,
    Hanbali,
}

impl Default for Madhab {
    fn default() -> Self {
        Self::Shafi
    }
}

/// Strategy for Daud fasting when a turn falls on a Haram day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DaudStrategy {
    /// Skip fasting entirely for this turn if it hits a Haram day. Resume on the next calendar day.
    Skip,
    /// Postpone the fast to the next permissible day if it hits a Haram day.
    Postpone,
}

impl Default for DaudStrategy {
    fn default() -> Self {
        Self::Skip
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastingAnalysis {
    pub date: chrono::NaiveDate,
    pub primary_status: FastingStatus,
    pub types: SmallVec<[FastingType; 4]>,
    // Store Hijri date components for on-demand description generation
    pub hijri_year: usize,
    pub hijri_month: usize,
    pub hijri_day: usize,
}

impl FastingAnalysis {
    pub fn new(date: chrono::NaiveDate, status: FastingStatus, types: SmallVec<[FastingType; 4]>, hijri: (usize, usize, usize)) -> Self {
        Self {
            date,
            primary_status: status,
            types,
            hijri_year: hijri.0,
            hijri_month: hijri.1,
            hijri_day: hijri.2,
        }
    }
    
    pub fn description(&self, localizer: &impl crate::i18n::Localizer) -> String {
        localizer.format_description(self)
    }
}
