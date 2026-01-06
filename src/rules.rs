use chrono::{Datelike, NaiveDate, Weekday};
use crate::calendar::{to_hijri, ShaumError};
use crate::types::{FastingAnalysis, FastingStatus, FastingType, Madhab, DaudStrategy};
use crate::constants::*;
use serde::{Serialize, Deserialize};
use smallvec::SmallVec;

pub trait MoonProvider {
    fn get_adjustment(&self, date: NaiveDate) -> i64;
}

/// Trait for defining custom fasting rules.
pub trait CustomFastingRule: std::fmt::Debug + Send + Sync {
    /// Evaluates the rule for a given date.
    /// Returns Some((status, type)) if the rule applies.
    fn evaluate(&self, date: NaiveDate, hijri_year: usize, hijri_month: usize, hijri_day: usize) -> Option<(FastingStatus, FastingType)>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleContext {
    /// Manual day offset for Hijri calculation (e.g., +1, -1). 
    /// Clamped to [-30, 30].
    pub adjustment: i64,
    /// The school of jurisprudence to follow for specific rules (e.g., Friday/Saturday exclusive).
    pub madhab: Madhab,
    /// Strategy to use when a Daud fast coincides with a Haram day.
    pub daud_strategy: DaudStrategy,
    /// Custom rules to be evaluated.
    #[serde(skip)]
    pub custom_rules: Vec<Box<dyn CustomFastingRule>>,
}

impl Clone for RuleContext {
    fn clone(&self) -> Self {
        Self {
            adjustment: self.adjustment,
            madhab: self.madhab,
            daud_strategy: self.daud_strategy,
            custom_rules: Vec::new(), // Custom rules are not easily cloneable without more boilerplate
        }
    }
}

impl Default for RuleContext {
    fn default() -> Self {
        Self {
            adjustment: 0,
            madhab: Madhab::default(),
            daud_strategy: DaudStrategy::default(),
            custom_rules: Vec::new(),
        }
    }
}

impl RuleContext {
    /// Creates a new `RuleContext` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Hijri day adjustment.
    pub fn adjustment(mut self, adjustment: i64) -> Self {
        // Clamp adjustment to safe bounds to avoid Chrono panics
        self.adjustment = adjustment.clamp(-30, 30);
        self
    }

    /// Sets the Madhab (school of jurisprudence).
    pub fn madhab(mut self, madhab: Madhab) -> Self {
        self.madhab = madhab;
        self
    }

    /// Sets the strategy for Daud fasting.
    pub fn daud_strategy(mut self, strategy: DaudStrategy) -> Self {
        self.daud_strategy = strategy;
        self
    }

    /// Adds a custom fasting rule.
    pub fn add_custom_rule(mut self, rule: Box<dyn CustomFastingRule>) -> Self {
        self.custom_rules.push(rule);
        self
    }
}

pub fn check(g_date: NaiveDate, context: &RuleContext) -> Result<FastingAnalysis, ShaumError> {
    let h_date = to_hijri(g_date, context.adjustment)?;
    let h_month = h_date.month();
    let h_day = h_date.day();
    let weekday = g_date.weekday();

    let mut types = SmallVec::new();
    let mut status = FastingStatus::Mubah;

    // --- 1. Haram Checks (Absolute Priority) ---
    // Eid al-Fitr
    if h_month == MONTH_SHAWWAL && h_day == 1 {
        types.push(FastingType::EidAlFitr);
        return Ok(FastingAnalysis::new(g_date, FastingStatus::Haram, types, (h_date.year() as usize, h_month, h_day)));
    }

    // Eid al-Adha
    if h_month == MONTH_DHUL_HIJJAH && h_day == 10 {
        types.push(FastingType::EidAlAdha);
        return Ok(FastingAnalysis::new(g_date, FastingStatus::Haram, types, (h_date.year() as usize, h_month, h_day)));
    }

    // Tashriq Days
    if h_month == MONTH_DHUL_HIJJAH && (11..=13).contains(&h_day) {
        types.push(FastingType::Tashriq);
        return Ok(FastingAnalysis::new(g_date, FastingStatus::Haram, types, (h_date.year() as usize, h_month, h_day)));
    }

    // --- 2. Wajib Checks ---
    if h_month == MONTH_RAMADHAN {
        types.push(FastingType::Ramadhan);
        status = FastingStatus::Wajib;
    }

    // --- 3. Sunnah Checks ---
    
    // Arafah (9 Dhu al-Hijjah)
    if h_month == MONTH_DHUL_HIJJAH && h_day == DAY_ARAFAH {
        types.push(FastingType::Arafah);
        if !status.is_wajib() { status = FastingStatus::SunnahMuakkadah; }
    }

    // Ashura (10 Muharram)
    if h_month == MONTH_MUHARRAM && h_day == DAY_ASHURA {
        types.push(FastingType::Ashura);
        if !status.is_wajib() { status = FastingStatus::SunnahMuakkadah; }
    }

    // Tasu'a (9 Muharram)
    if h_month == MONTH_MUHARRAM && h_day == DAY_TASUA {
        types.push(FastingType::Tasua);
        if !status.is_wajib() && status != FastingStatus::SunnahMuakkadah { 
            status = FastingStatus::Sunnah; 
        }
    }

    // Ayyamul Bidh (13, 14, 15) - EXCLUDING 13 Dhu al-Hijjah (Handled by Haram above)
    if (13..=15).contains(&h_day) {
        types.push(FastingType::AyyamulBidh);
        if !status.is_wajib() && status < FastingStatus::Sunnah {
            status = FastingStatus::Sunnah;
        }
    }

    // Monday / Thursday
    match weekday {
        Weekday::Mon => {
            types.push(FastingType::Monday);
            if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
        },
        Weekday::Thu => {
            types.push(FastingType::Thursday);
            if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
        },
        _ => {}
    }

    // Shawwal (Month 10, excluding Day 1)
    if h_month == MONTH_SHAWWAL && h_day > 1 {
        types.push(FastingType::Shawwal);
        if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
    }

    // --- 4. Makruh Checks (Friday/Saturday) ---
    // General Rule: Singling out Friday or Saturday is Makruh in most Madhabs.
    // Exception: If it coincides with a Sunnah (Arafah, Ashura, etc.) or Wajib, it is not Makruh.
    // We handle this by checking if status is still Mubah (meaning no other reason to fast was found).
    
    match context.madhab {
        Madhab::Shafi | Madhab::Hanafi | Madhab::Maliki | Madhab::Hanbali => {
             if status == FastingStatus::Mubah {
                let weekday = g_date.weekday();
                if weekday == Weekday::Fri {
                    types.push(FastingType::FridayExclusive);
                    status = FastingStatus::Makruh;
                } else if weekday == Weekday::Sat {
                    types.push(FastingType::SaturdayExclusive);
                    status = FastingStatus::Makruh;
                }
            }
        }
    }

    // --- 5. Custom Rules ---
    for rule in &context.custom_rules {
        if let Some((custom_status, custom_type)) = rule.evaluate(g_date, h_date.year() as usize, h_month, h_day) {
            types.push(custom_type);
            if custom_status > status {
                status = custom_status;
            }
        }
    }

    Ok(FastingAnalysis::new(g_date, status, types, (h_date.year() as usize, h_month, h_day)))
}
