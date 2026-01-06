use serde::{Serialize, Deserialize};
use smallvec::SmallVec;
use std::fmt;
use std::borrow::Cow;

/// Geographic coordinate for sunset calculation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoCoordinate {
    pub lat: f64,
    pub lng: f64,
}

impl GeoCoordinate {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }
}

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

/// Extensible fasting type/reason.
/// Wraps a string to allow both standard and custom fasting types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FastingType(pub Cow<'static, str>);

impl FastingType {
    /// Creates a new custom fasting type.
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self(name.into())
    }

    pub fn custom(name: &str) -> Self {
        Self(Cow::Owned(name.to_string()))
    }

    pub const RAMADHAN: Self = Self(Cow::Borrowed("Ramadhan"));
    pub const ARAFAH: Self = Self(Cow::Borrowed("Arafah"));
    pub const TASUA: Self = Self(Cow::Borrowed("Tasua"));
    pub const ASHURA: Self = Self(Cow::Borrowed("Ashura"));
    pub const AYYAMUL_BIDH: Self = Self(Cow::Borrowed("AyyamulBidh"));
    pub const MONDAY: Self = Self(Cow::Borrowed("Monday"));
    pub const THURSDAY: Self = Self(Cow::Borrowed("Thursday"));
    pub const SHAWWAL: Self = Self(Cow::Borrowed("Shawwal"));
    pub const DAUD: Self = Self(Cow::Borrowed("Daud"));
    pub const EID_AL_FITR: Self = Self(Cow::Borrowed("EidAlFitr"));
    pub const EID_AL_ADHA: Self = Self(Cow::Borrowed("EidAlAdha"));
    pub const TASHRIQ: Self = Self(Cow::Borrowed("Tashriq"));
    pub const FRIDAY_EXCLUSIVE: Self = Self(Cow::Borrowed("FridayExclusive"));
    pub const SATURDAY_EXCLUSIVE: Self = Self(Cow::Borrowed("SaturdayExclusive"));

    // Legacy-like constructors for backward compat (where possible) or ease of use
    #[allow(non_snake_case)] pub fn Ramadhan() -> Self { Self::RAMADHAN }
    #[allow(non_snake_case)] pub fn Arafah() -> Self { Self::ARAFAH }
    #[allow(non_snake_case)] pub fn Tasua() -> Self { Self::TASUA }
    #[allow(non_snake_case)] pub fn Ashura() -> Self { Self::ASHURA }
    #[allow(non_snake_case)] pub fn AyyamulBidh() -> Self { Self::AYYAMUL_BIDH }
    #[allow(non_snake_case)] pub fn Monday() -> Self { Self::MONDAY }
    #[allow(non_snake_case)] pub fn Thursday() -> Self { Self::THURSDAY }
    #[allow(non_snake_case)] pub fn Shawwal() -> Self { Self::SHAWWAL }
    #[allow(non_snake_case)] pub fn Daud() -> Self { Self::DAUD }
    #[allow(non_snake_case)] pub fn EidAlFitr() -> Self { Self::EID_AL_FITR }
    #[allow(non_snake_case)] pub fn EidAlAdha() -> Self { Self::EID_AL_ADHA }
    #[allow(non_snake_case)] pub fn Tashriq() -> Self { Self::TASHRIQ }
    #[allow(non_snake_case)] pub fn FridayExclusive() -> Self { Self::FRIDAY_EXCLUSIVE }
    #[allow(non_snake_case)] pub fn SaturdayExclusive() -> Self { Self::SATURDAY_EXCLUSIVE }

    pub fn is_haram_type(&self) -> bool {
        matches!(self.0.as_ref(), "EidAlFitr" | "EidAlAdha" | "Tashriq")
    }
    
    pub fn is_sunnah_type(&self) -> bool {
        matches!(self.0.as_ref(), "Arafah" | "Tasua" | "Ashura" | "AyyamulBidh" | 
                 "Monday" | "Thursday" | "Shawwal" | "Daud")
    }
}

impl fmt::Display for FastingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

/// Machine-readable trace codes for rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraceCode {
    // Harams
    EidAlFitr,
    EidAlAdha,
    Tashriq,
    FridaySingledOut,
    SaturdaySingledOut,
    // Wajibs
    Ramadhan,
    // Sunnahs
    Arafah,
    Tasua,
    Ashura,
    AyyamulBidh,
    Monday,
    Thursday,
    Shawwal,
    Daud,
    // Generic
    Custom,
    Debug,
}

impl fmt::Display for TraceCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Rule trace event for explainability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTrace {
    pub code: TraceCode,
    pub details: Option<String>,
}

impl RuleTrace {
    pub fn new(code: TraceCode, details: impl Into<Option<String>>) -> Self {
        Self { code, details: details.into() }
    }
}

/// Fasting analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastingAnalysis {
    pub date: chrono::DateTime<chrono::Utc>,
    pub primary_status: FastingStatus,
    pub hijri_year: usize,
    pub hijri_month: usize,
    pub hijri_day: usize,
    reasons: SmallVec<[FastingType; 2]>,
    traces: SmallVec<[RuleTrace; 2]>,
}

impl FastingAnalysis {
    pub fn new(
        date: chrono::DateTime<chrono::Utc>,
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
        date: chrono::DateTime<chrono::Utc>,
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
    pub fn has_reason(&self, ftype: &FastingType) -> bool { self.reasons.contains(ftype) }

    /// Returns reason count.
    pub fn reason_count(&self) -> usize { self.reasons.len() }

    pub fn is_ramadhan(&self) -> bool { self.has_reason(&FastingType::RAMADHAN) }
    pub fn is_white_day(&self) -> bool { self.has_reason(&FastingType::AYYAMUL_BIDH) }
    pub fn is_eid(&self) -> bool { self.has_reason(&FastingType::EID_AL_FITR) || self.has_reason(&FastingType::EID_AL_ADHA) }
    pub fn is_tashriq(&self) -> bool { self.has_reason(&FastingType::TASHRIQ) }
    pub fn is_arafah(&self) -> bool { self.has_reason(&FastingType::ARAFAH) }
    pub fn is_ashura(&self) -> bool { self.has_reason(&FastingType::ASHURA) }

    /// Returns human-readable explanation.
    pub fn explain(&self) -> String {
        if self.traces.is_empty() {
            self.generate_explanation()
        } else {
            self.traces.iter()
                .map(|t| if let Some(d) = &t.details {
                    format!("{}: {}", t.code, d)
                } else {
                    t.code.to_string()
                })
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
}

impl fmt::Display for FastingAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.explain())
    }
}
