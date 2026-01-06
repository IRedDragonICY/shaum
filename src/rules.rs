use chrono::{Datelike, NaiveDate, Weekday};
use crate::calendar::{to_hijri, ShaumError};
use crate::types::{FastingAnalysis, FastingStatus, FastingType, Madhab, DaudStrategy, RuleTrace};
use crate::constants::*;
use serde::{Serialize, Deserialize};
use smallvec::SmallVec;

/// Moon sighting adjustment provider.
pub trait MoonProvider: std::fmt::Debug + Send + Sync {
    fn get_adjustment(&self, date: NaiveDate) -> i64;
}

/// Fixed day offset for all dates.
#[derive(Debug, Clone, Copy, Default)]
pub struct FixedAdjustment(i64);

impl FixedAdjustment {
    pub fn new(offset: i64) -> Self { Self(offset.clamp(-30, 30)) }
}

impl MoonProvider for FixedAdjustment {
    fn get_adjustment(&self, _date: NaiveDate) -> i64 { self.0 }
}

/// No adjustment (use astronomical calculation).
#[derive(Debug, Clone, Copy, Default)]
pub struct NoAdjustment;

impl MoonProvider for NoAdjustment {
    fn get_adjustment(&self, _date: NaiveDate) -> i64 { 0 }
}

/// Custom rule trait.
pub trait CustomFastingRule: std::fmt::Debug + Send + Sync {
    fn evaluate(&self, date: NaiveDate, hijri_year: usize, hijri_month: usize, hijri_day: usize) 
        -> Option<(FastingStatus, FastingType)>;
}

/// Rule engine configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct RuleContext {
    /// Hijri day offset. Clamped to [-30, 30].
    pub adjustment: i64,
    pub madhab: Madhab,
    pub daud_strategy: DaudStrategy,
    #[serde(skip)]
    pub custom_rules: Vec<Box<dyn CustomFastingRule>>,
}

impl Clone for RuleContext {
    fn clone(&self) -> Self {
        Self {
            adjustment: self.adjustment,
            madhab: self.madhab,
            daud_strategy: self.daud_strategy,
            custom_rules: Vec::new(),
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
    pub fn new() -> Self { Self::default() }

    pub fn adjustment(mut self, adjustment: i64) -> Self {
        self.adjustment = adjustment.clamp(-30, 30);
        self
    }

    pub fn madhab(mut self, madhab: Madhab) -> Self {
        self.madhab = madhab;
        self
    }

    pub fn daud_strategy(mut self, strategy: DaudStrategy) -> Self {
        self.daud_strategy = strategy;
        self
    }

    pub fn add_custom_rule(mut self, rule: Box<dyn CustomFastingRule>) -> Self {
        self.custom_rules.push(rule);
        self
    }

    pub fn with_moon_provider<M: MoonProvider>(mut self, provider: &M, reference_date: NaiveDate) -> Self {
        self.adjustment = provider.get_adjustment(reference_date);
        self
    }
}

/// Builder with validation for `RuleContext`.
#[derive(Debug, Default)]
pub struct RuleContextBuilder {
    adjustment: Option<i64>,
    madhab: Option<Madhab>,
    daud_strategy: Option<DaudStrategy>,
    custom_rules: Vec<Box<dyn CustomFastingRule>>,
    strict_adjustment: bool,
}

impl RuleContextBuilder {
    pub fn new() -> Self { Self::default() }
    
    pub fn adjustment(mut self, adjustment: i64) -> Self { self.adjustment = Some(adjustment); self }
    pub fn madhab(mut self, madhab: Madhab) -> Self { self.madhab = Some(madhab); self }
    pub fn daud_strategy(mut self, strategy: DaudStrategy) -> Self { self.daud_strategy = Some(strategy); self }
    pub fn add_custom_rule(mut self, rule: Box<dyn CustomFastingRule>) -> Self { self.custom_rules.push(rule); self }
    
    /// Enables strict adjustment bounds [-2, 2].
    pub fn strict_adjustment(mut self, strict: bool) -> Self { self.strict_adjustment = strict; self }

    /// Builds and validates.
    pub fn build(self) -> Result<RuleContext, ShaumError> {
        let adjustment = self.adjustment.unwrap_or(0);
        
        if self.strict_adjustment && (adjustment < -2 || adjustment > 2) {
            return Err(ShaumError::invalid_config(format!(
                "Adjustment {} outside strict bounds [-2, 2]", adjustment
            )));
        }

        Ok(RuleContext {
            adjustment: adjustment.clamp(-30, 30),
            madhab: self.madhab.unwrap_or_default(),
            daud_strategy: self.daud_strategy.unwrap_or_default(),
            custom_rules: self.custom_rules,
        })
    }
}

/// Analyzes fasting status for a date.
pub fn check(g_date: NaiveDate, context: &RuleContext) -> Result<FastingAnalysis, ShaumError> {
    let h_date = to_hijri(g_date, context.adjustment)?;
    let h_month = h_date.month();
    let h_day = h_date.day();
    let h_year = h_date.year() as usize;
    let weekday = g_date.weekday();

    let mut types: SmallVec<[FastingType; 4]> = SmallVec::new();
    let mut traces: SmallVec<[RuleTrace; 4]> = SmallVec::new();
    let mut status = FastingStatus::Mubah;

    // Haram checks
    if h_month == MONTH_SHAWWAL && h_day == 1 {
        types.push(FastingType::EidAlFitr);
        traces.push(RuleTrace::new("Eid al-Fitr", format!("1 Shawwal {}", h_year)));
        return Ok(FastingAnalysis::with_traces(g_date, FastingStatus::Haram, types, (h_year, h_month, h_day), traces));
    }

    if h_month == MONTH_DHUL_HIJJAH && h_day == 10 {
        types.push(FastingType::EidAlAdha);
        traces.push(RuleTrace::new("Eid al-Adha", format!("10 Dhul Hijjah {}", h_year)));
        return Ok(FastingAnalysis::with_traces(g_date, FastingStatus::Haram, types, (h_year, h_month, h_day), traces));
    }

    if h_month == MONTH_DHUL_HIJJAH && (11..=13).contains(&h_day) {
        types.push(FastingType::Tashriq);
        traces.push(RuleTrace::new("Tashriq", format!("{} Dhul Hijjah {}", h_day, h_year)));
        return Ok(FastingAnalysis::with_traces(g_date, FastingStatus::Haram, types, (h_year, h_month, h_day), traces));
    }

    // Wajib
    if h_month == MONTH_RAMADHAN {
        types.push(FastingType::Ramadhan);
        traces.push(RuleTrace::new("Ramadhan", format!("{} Ramadhan {}", h_day, h_year)));
        status = FastingStatus::Wajib;
    }

    // Sunnah Muakkadah
    if h_month == MONTH_DHUL_HIJJAH && h_day == DAY_ARAFAH {
        types.push(FastingType::Arafah);
        traces.push(RuleTrace::new("Arafah", "9 Dhul Hijjah"));
        if !status.is_wajib() { status = FastingStatus::SunnahMuakkadah; }
    }

    if h_month == MONTH_MUHARRAM && h_day == DAY_ASHURA {
        types.push(FastingType::Ashura);
        traces.push(RuleTrace::new("Ashura", "10 Muharram"));
        if !status.is_wajib() { status = FastingStatus::SunnahMuakkadah; }
    }

    // Sunnah
    if h_month == MONTH_MUHARRAM && h_day == DAY_TASUA {
        types.push(FastingType::Tasua);
        traces.push(RuleTrace::new("Tasu'a", "9 Muharram"));
        if !status.is_wajib() && status != FastingStatus::SunnahMuakkadah { 
            status = FastingStatus::Sunnah; 
        }
    }

    if (13..=15).contains(&h_day) {
        types.push(FastingType::AyyamulBidh);
        traces.push(RuleTrace::new("Ayyamul Bidh", format!("{} of {}", h_day, crate::calendar::get_hijri_month_name(h_month))));
        if !status.is_wajib() && status < FastingStatus::Sunnah {
            status = FastingStatus::Sunnah;
        }
    }

    match weekday {
        Weekday::Mon => {
            types.push(FastingType::Monday);
            traces.push(RuleTrace::new("Monday", "Weekly sunnah"));
            if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
        },
        Weekday::Thu => {
            types.push(FastingType::Thursday);
            traces.push(RuleTrace::new("Thursday", "Weekly sunnah"));
            if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
        },
        _ => {}
    }

    if h_month == MONTH_SHAWWAL && h_day > 1 {
        types.push(FastingType::Shawwal);
        traces.push(RuleTrace::new("Shawwal", "Six days of Shawwal"));
        if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
    }

    // Makruh
    match context.madhab {
        Madhab::Shafi | Madhab::Hanafi | Madhab::Maliki | Madhab::Hanbali => {
             if status == FastingStatus::Mubah {
                let wd = g_date.weekday();
                if wd == Weekday::Fri {
                    types.push(FastingType::FridayExclusive);
                    traces.push(RuleTrace::new("Friday alone", "Makruh without adjacent day"));
                    status = FastingStatus::Makruh;
                } else if wd == Weekday::Sat {
                    types.push(FastingType::SaturdayExclusive);
                    traces.push(RuleTrace::new("Saturday alone", "Makruh without valid reason"));
                    status = FastingStatus::Makruh;
                }
            }
        }
    }

    // Custom rules
    for rule in &context.custom_rules {
        if let Some((custom_status, custom_type)) = rule.evaluate(g_date, h_year, h_month, h_day) {
            types.push(custom_type);
            traces.push(RuleTrace::new(format!("{:?}", custom_type), format!("{:?}", custom_status)));
            if custom_status > status { status = custom_status; }
        }
    }

    Ok(FastingAnalysis::with_traces(g_date, status, types, (h_year, h_month, h_day), traces))
}
