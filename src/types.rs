


use serde::{Serialize, Deserialize};

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
}

/// The specific reason or type of fasting associated with a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FastingType {
    Ramadhan,
    Arafah,
    Tasua,
    Ashura,
    AyyamulBidh,
    Monday,
    Thursday,
    Shawwal,
    Daud,
    EidAlFitr,
    EidAlAdha,
    Tashriq,
    FridayExclusive,
    SaturdayExclusive,
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
    /// Skip fasting entirely for this turn. Resume on the next calendar day.
    Skip,
    /// Postpone the fast to the next permissible day.
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
    pub types: Vec<FastingType>,
    // Store Hijri date components for on-demand description generation
    pub hijri_year: usize,
    pub hijri_month: usize,
    pub hijri_day: usize,
}

impl FastingAnalysis {
    pub fn new(date: chrono::NaiveDate, status: FastingStatus, types: Vec<FastingType>, hijri: (usize, usize, usize)) -> Self {
        Self {
            date,
            primary_status: status,
            types,
            hijri_year: hijri.0,
            hijri_month: hijri.1,
            hijri_day: hijri.2,
        }
    }
    
    pub fn description(&self, localizer: &impl Localizer) -> String {
        localizer.format_description(self)
    }
}

pub trait Localizer {
    fn month_name(&self, month: usize) -> String;
    fn status_name(&self, status: FastingStatus) -> String;
    fn type_name(&self, f_type: FastingType) -> String;
    fn format_description(&self, analysis: &FastingAnalysis) -> String;
}

pub struct EnglishLocalizer;

impl Localizer for EnglishLocalizer {
    fn month_name(&self, month: usize) -> String {
        crate::calendar::get_hijri_month_name(month).to_string()
    }

    fn status_name(&self, status: FastingStatus) -> String {
        format!("{:?}", status)
    }

    fn type_name(&self, f_type: FastingType) -> String {
        format!("{:?}", f_type)
    }

    fn format_description(&self, analysis: &FastingAnalysis) -> String {
        format!(
            "Hijri Date: {} {} {}", 
            analysis.hijri_day, 
            self.month_name(analysis.hijri_month), 
            analysis.hijri_year
        )
    }
}
