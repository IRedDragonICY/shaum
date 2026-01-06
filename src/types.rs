use serde::{Serialize, Deserialize};
use smallvec::SmallVec;
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
    pub fn is_haram(&self) -> bool { matches!(self, Self::Haram) }
    pub fn is_wajib(&self) -> bool { matches!(self, Self::Wajib) }
    pub fn is_sunnah(&self) -> bool { matches!(self, Self::Sunnah | Self::SunnahMuakkadah) }
    pub fn is_makruh(&self) -> bool { matches!(self, Self::Makruh) }
    pub fn is_mubah(&self) -> bool { matches!(self, Self::Mubah) }
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

/// Fasting type/reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl FastingType {
    pub fn is_haram_type(&self) -> bool {
        matches!(self, Self::EidAlFitr | Self::EidAlAdha | Self::Tashriq)
    }
    
    pub fn is_sunnah_type(&self) -> bool {
        matches!(self, Self::Arafah | Self::Tasua | Self::Ashura | Self::AyyamulBidh | 
                 Self::Monday | Self::Thursday | Self::Shawwal | Self::Daud)
    }
}

impl fmt::Display for FastingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Ramadhan => "Ramadhan",
            Self::Arafah => "Day of Arafah",
            Self::Tasua => "Tasu'a (9th Muharram)",
            Self::Ashura => "Ashura (10th Muharram)",
            Self::AyyamulBidh => "Ayyamul Bidh (13th, 14th, 15th)",
            Self::Monday => "Monday",
            Self::Thursday => "Thursday",
            Self::Shawwal => "Six Days of Shawwal",
            Self::Daud => "Fasting of Prophet Daud (A.S)",
            Self::EidAlFitr => "Eid al-Fitr",
            Self::EidAlAdha => "Eid al-Adha",
            Self::Tashriq => "Days of Tashriq",
            Self::FridayExclusive => "Singling out Friday",
            Self::SaturdayExclusive => "Singling out Saturday",
        };
        write!(f, "{}", s)
    }
}

/// Sunni schools of jurisprudence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Madhab {
    Shafi,
    Hanafi,
    Maliki,
    Hanbali,
}

impl Default for Madhab {
    fn default() -> Self { Self::Shafi }
}

/// Strategy for Daud fasting on Haram days.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DaudStrategy {
    /// Skip turn, lose the fast.
    Skip,
    /// Postpone to next permissible day.
    Postpone,
}

impl Default for DaudStrategy {
    fn default() -> Self { Self::Skip }
}

/// Rule trace event for explainability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTrace {
    pub rule: String,
    pub reason: String,
}

impl RuleTrace {
    pub fn new(rule: impl Into<String>, reason: impl Into<String>) -> Self {
        Self { rule: rule.into(), reason: reason.into() }
    }
}

/// Fasting analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastingAnalysis {
    pub date: chrono::NaiveDate,
    pub primary_status: FastingStatus,
    pub hijri_year: usize,
    pub hijri_month: usize,
    pub hijri_day: usize,
    reasons: SmallVec<[FastingType; 2]>,
    traces: SmallVec<[RuleTrace; 2]>,
}

impl FastingAnalysis {
    pub fn new(
        date: chrono::NaiveDate,
        status: FastingStatus,
        types: SmallVec<[FastingType; 2]>,
        hijri: (usize, usize, usize),
    ) -> Self {
        Self {
            date,
            primary_status: status,
            reasons: types,
            hijri_year: hijri.0,
            hijri_month: hijri.1,
            hijri_day: hijri.2,
            traces: SmallVec::new(),
        }
    }

    pub fn with_traces(
        date: chrono::NaiveDate,
        status: FastingStatus,
        types: SmallVec<[FastingType; 2]>,
        hijri: (usize, usize, usize),
        traces: SmallVec<[RuleTrace; 2]>,
    ) -> Self {
        Self {
            date,
            primary_status: status,
            reasons: types,
            hijri_year: hijri.0,
            hijri_month: hijri.1,
            hijri_day: hijri.2,
            traces,
        }
    }

    /// Iterates over fasting types.
    pub fn reasons(&self) -> impl Iterator<Item = &FastingType> { self.reasons.iter() }

    /// Checks if `ftype` is among the reasons.
    pub fn has_reason(&self, ftype: FastingType) -> bool { self.reasons.contains(&ftype) }

    /// Returns reason count.
    pub fn reason_count(&self) -> usize { self.reasons.len() }

    pub fn is_ramadhan(&self) -> bool { self.has_reason(FastingType::Ramadhan) }
    pub fn is_white_day(&self) -> bool { self.has_reason(FastingType::AyyamulBidh) }
    pub fn is_eid(&self) -> bool { self.has_reason(FastingType::EidAlFitr) || self.has_reason(FastingType::EidAlAdha) }
    pub fn is_tashriq(&self) -> bool { self.has_reason(FastingType::Tashriq) }
    pub fn is_arafah(&self) -> bool { self.has_reason(FastingType::Arafah) }
    pub fn is_ashura(&self) -> bool { self.has_reason(FastingType::Ashura) }

    /// Returns human-readable explanation.
    pub fn explain(&self) -> String {
        if self.traces.is_empty() {
            self.generate_explanation()
        } else {
            self.traces.iter()
                .map(|t| format!("{}: {}", t.rule, t.reason))
                .collect::<Vec<_>>()
                .join("; ")
        }
    }

    /// Returns trace iterator.
    pub fn traces(&self) -> impl Iterator<Item = &RuleTrace> { self.traces.iter() }

    #[allow(dead_code)]
    pub(crate) fn add_trace(&mut self, trace: RuleTrace) { self.traces.push(trace); }

    fn generate_explanation(&self) -> String {
        let hijri_str = format!(
            "{} {} {}",
            self.hijri_day,
            crate::calendar::get_hijri_month_name(self.hijri_month),
            self.hijri_year
        );

        let status_str = match self.primary_status {
            FastingStatus::Haram => "Haram",
            FastingStatus::Wajib => "Wajib",
            FastingStatus::SunnahMuakkadah => "Sunnah Muakkadah",
            FastingStatus::Sunnah => "Sunnah",
            FastingStatus::Makruh => "Makruh",
            FastingStatus::Mubah => "Mubah",
        };

        if self.reasons.is_empty() {
            format!("{} - {}", hijri_str, status_str)
        } else {
            let reasons: Vec<String> = self.reasons.iter().map(|r| r.to_string()).collect();
            format!("{} - {} because: {}", hijri_str, status_str, reasons.join(", "))
        }
    }

    /// Localized description.
    pub fn description(&self, localizer: &impl crate::i18n::Localizer) -> String {
        localizer.format_description(self)
    }

    /// **Deprecated**: Use `reasons()` instead.
    #[deprecated(since = "0.2.0", note = "Use `reasons()` instead")]
    pub fn types(&self) -> &SmallVec<[FastingType; 2]> { &self.reasons }
}

impl fmt::Display for FastingAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.explain())
    }
}
