//! Fasting analysis result and related types.

use serde::{Serialize, Deserialize};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt;

use super::status::FastingStatus;

/// Extensible fasting type/reason.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FastingType(pub Cow<'static, str>);

impl FastingType {
    /// Creates a new custom fasting type.
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self { Self(name.into()) }
    pub fn custom(name: &str) -> Self { Self(Cow::Owned(name.to_string())) }

    // Standard fasting types
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

    // Legacy constructors
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}

/// Machine-readable trace codes for rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraceCode {
    EidAlFitr, EidAlAdha, Tashriq, FridaySingledOut, SaturdaySingledOut,
    Ramadhan, Arafah, Tasua, Ashura, AyyamulBidh,
    Monday, Thursday, Shawwal, Daud,
    Custom, Debug,
}

impl fmt::Display for TraceCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self) }
}

/// Payload for deferred trace formatting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TracePayload {
    None,
    PostMaghribOffset,
    CustomReason(String),
}

impl fmt::Display for TracePayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::PostMaghribOffset => write!(f, "Post-Maghrib: Effective date +1"),
            Self::CustomReason(s) => write!(f, "{}", s),
        }
    }
}

/// Rule trace event for explainability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTrace {
    pub code: TraceCode,
    pub payload: TracePayload,
}

impl RuleTrace {
    pub fn new(code: TraceCode, payload: TracePayload) -> Self { Self { code, payload } }
    #[inline] pub fn simple(code: TraceCode) -> Self { Self { code, payload: TracePayload::None } }
}

/// Returns Hijri month name (inline for pure types crate).
fn get_hijri_month_name(month: usize) -> &'static str {
    match month {
        1 => "Muharram", 2 => "Safar", 3 => "Rabi' al-Awwal", 4 => "Rabi' al-Thani",
        5 => "Jumada al-Ula", 6 => "Jumada al-Akhirah", 7 => "Rajab", 8 => "Sha'ban",
        9 => "Ramadhan", 10 => "Shawwal", 11 => "Dhu al-Qi'dah", 12 => "Dhu al-Hijjah",
        _ => "Unknown",
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
            date, primary_status: status, reasons: types,
            hijri_year: hijri.0, hijri_month: hijri.1, hijri_day: hijri.2,
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
            date, primary_status: status, reasons: types,
            hijri_year: hijri.0, hijri_month: hijri.1, hijri_day: hijri.2,
            traces,
        }
    }

    pub fn reasons(&self) -> impl Iterator<Item = &FastingType> { self.reasons.iter() }
    pub fn has_reason(&self, ftype: &FastingType) -> bool { self.reasons.contains(ftype) }
    pub fn reason_count(&self) -> usize { self.reasons.len() }

    pub fn is_ramadhan(&self) -> bool { self.has_reason(&FastingType::RAMADHAN) }
    pub fn is_white_day(&self) -> bool { self.has_reason(&FastingType::AYYAMUL_BIDH) }
    pub fn is_eid(&self) -> bool { self.has_reason(&FastingType::EID_AL_FITR) || self.has_reason(&FastingType::EID_AL_ADHA) }
    pub fn is_tashriq(&self) -> bool { self.has_reason(&FastingType::TASHRIQ) }
    pub fn is_arafah(&self) -> bool { self.has_reason(&FastingType::ARAFAH) }
    pub fn is_ashura(&self) -> bool { self.has_reason(&FastingType::ASHURA) }

    pub fn explain(&self) -> String {
        if self.traces.is_empty() {
            self.generate_explanation()
        } else {
            self.traces.iter()
                .map(|t| match &t.payload {
                    TracePayload::None => t.code.to_string(),
                    payload => format!("{}: {}", t.code, payload),
                })
                .collect::<Vec<_>>()
                .join("; ")
        }
    }

    pub fn traces(&self) -> impl Iterator<Item = &RuleTrace> { self.traces.iter() }

    #[allow(dead_code)]
    pub(crate) fn add_trace(&mut self, trace: RuleTrace) { self.traces.push(trace); }

    fn generate_explanation(&self) -> String {
        let hijri_str = format!(
            "{} {} {}",
            self.hijri_day,
            get_hijri_month_name(self.hijri_month),
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
}

impl fmt::Display for FastingAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.explain()) }
}
