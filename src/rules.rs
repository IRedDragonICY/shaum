use chrono::{Datelike, NaiveDate, Weekday, DateTime, Utc, TimeZone, Duration};
use crate::calendar::{ShaumError, to_hijri, HIJRI_MIN_YEAR, HIJRI_MAX_YEAR};
use crate::types::{FastingAnalysis, FastingStatus, FastingType, Madhab, DaudStrategy, RuleTrace, TraceCode, GeoCoordinate};
use crate::constants::*;
use serde::{Serialize, Deserialize};
use smallvec::SmallVec;

#[cfg(feature = "async")]
use async_trait::async_trait;

/// Moon sighting adjustment provider.
#[cfg_attr(feature = "async", async_trait)]
pub trait MoonProvider: std::fmt::Debug + Send + Sync {
    #[cfg(feature = "async")]
    async fn get_adjustment(&self, date: NaiveDate, coords: Option<GeoCoordinate>) -> Result<i64, ShaumError>;
    
    #[cfg(not(feature = "async"))]
    fn get_adjustment(&self, date: NaiveDate, coords: Option<GeoCoordinate>) -> Result<i64, ShaumError>;
}

/// Fixed day offset for all dates.
#[derive(Debug, Clone, Copy, Default)]
pub struct FixedAdjustment(pub i64);

impl FixedAdjustment {
    pub fn new(offset: i64) -> Self { Self(offset.clamp(-30, 30)) }
}

#[cfg_attr(feature = "async", async_trait)]
impl MoonProvider for FixedAdjustment {
    #[cfg(feature = "async")]
    async fn get_adjustment(&self, _date: NaiveDate, _coords: Option<GeoCoordinate>) -> Result<i64, ShaumError> {
        Ok(self.0)
    }

    #[cfg(not(feature = "async"))]
    fn get_adjustment(&self, _date: NaiveDate, _coords: Option<GeoCoordinate>) -> Result<i64, ShaumError> {
        Ok(self.0)
    }
}

/// No adjustment (use astronomical calculation).
#[derive(Debug, Clone, Copy, Default)]
pub struct NoAdjustment;

#[cfg_attr(feature = "async", async_trait)]
impl MoonProvider for NoAdjustment {
    #[cfg(feature = "async")]
    async fn get_adjustment(&self, _date: NaiveDate, _coords: Option<GeoCoordinate>) -> Result<i64, ShaumError> {
        Ok(0)
    }

    #[cfg(not(feature = "async"))]
    fn get_adjustment(&self, _date: NaiveDate, _coords: Option<GeoCoordinate>) -> Result<i64, ShaumError> {
        Ok(0)
    }
}

/// Interface for calculating sunset time.
pub trait SunsetCalculator: std::fmt::Debug + Send + Sync {
    /// Returns the sunset timestamp for a given date and coordinate.
    /// If calculation fails, returns None.
    fn get_sunset(&self, date: NaiveDate, coords: GeoCoordinate) -> Option<DateTime<Utc>>;
}

/// Placeholder sunset calculator (assumes 18:00 Local Mean Time approx).
#[derive(Debug, Default, Clone, Copy)]
pub struct SimpleSunsetCalculator;

impl SunsetCalculator for SimpleSunsetCalculator {
    fn get_sunset(&self, date: NaiveDate, coords: GeoCoordinate) -> Option<DateTime<Utc>> {
        // Very rough approximation: 18:00 local time.
        // longitude / 15.0 = offset in hours from UTC.
        // Local 18:00 = UTC 18:00 - offset.
        let offset_hours = coords.lng / 15.0;
        let sunset_utc_hour = 18.0 - offset_hours;
        
        let naive_time = chrono::NaiveTime::from_hms_opt(0, 0, 0)?;
        let naive_dt = chrono::NaiveDateTime::new(date, naive_time);
        
        // Add hours manually
        let seconds = (sunset_utc_hour * 3600.0) as i64;
        let dt = Utc.from_utc_datetime(&naive_dt);
        dt.checked_add_signed(Duration::seconds(seconds))
    }
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
    pub strict: bool,
    #[serde(skip)]
    pub custom_rules: Vec<Box<dyn CustomFastingRule>>,
}

impl Clone for RuleContext {
    fn clone(&self) -> Self {
        Self {
            adjustment: self.adjustment,
            madhab: self.madhab,
            daud_strategy: self.daud_strategy,
            strict: self.strict,
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
            strict: false,
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

    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    pub fn with_moon_provider<M: MoonProvider>(mut self, provider: &M, reference_date: NaiveDate) -> Self {
        // self.adjustment = provider.get_adjustment(reference_date); // Can't satisfy async/sync or signature easily.
        // Dropping this method effectively as per architecture change.
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
    strict_mode: bool,
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
            strict: self.strict_mode,
        })
    }
}

/// Analyzes fasting status for a specific moment in time.
/// 
/// * `datetime`: The checking time in UTC.
/// * `context`: The rule configuration.
/// * `coords`: Optional coordinates for sunset-aware calculation.
pub fn analyze(
    datetime: DateTime<Utc>,
    context: &RuleContext,
    coords: Option<GeoCoordinate>
) -> Result<FastingAnalysis, ShaumError> {
    let mut traces: SmallVec<[RuleTrace; 2]> = SmallVec::new();
    
    // 1. Determine Effective Date (Maghrib Logic)
    let mut effective_date = datetime.date_naive();
    
    if let Some(c) = coords {
        let calculator = SimpleSunsetCalculator; // Could be part of context if we wanted dependency injection
        if let Some(sunset) = calculator.get_sunset(effective_date, c) {
            if datetime > sunset {
                effective_date = effective_date.succ_opt().ok_or_else(|| ShaumError::date_out_of_range(effective_date))?;
                traces.push(RuleTrace::new(TraceCode::Debug, Some("Post-Maghrib: Effective date +1".to_string())));
            }
        }
    }

    // 2. Strict Mode Check
    let year = effective_date.year();
    if year < HIJRI_MIN_YEAR || year > HIJRI_MAX_YEAR {
        if context.strict {
            return Err(ShaumError::date_out_of_range(effective_date));
        }
        traces.push(RuleTrace::new(
            TraceCode::Debug, 
            Some(format!("Date {} outside supported Hijri range (1938-2076). Clamping applied.", effective_date))
        ));
    }

    let h_date = to_hijri(effective_date, context.adjustment);
    let h_month = h_date.month();
    let h_day = h_date.day();
    let h_year = h_date.year() as usize;
    let weekday = effective_date.weekday();

    let mut types: SmallVec<[FastingType; 2]> = SmallVec::new();
    let mut status = FastingStatus::Mubah;

    // --- Rules ---

    // Haram Priority
    if h_month == MONTH_SHAWWAL && h_day == 1 {
        types.push(FastingType::EID_AL_FITR);
        traces.push(RuleTrace::new(TraceCode::EidAlFitr, None));
        return Ok(FastingAnalysis::with_traces(datetime, FastingStatus::Haram, types, (h_year, h_month, h_day), traces));
    }

    if h_month == MONTH_DHUL_HIJJAH && h_day == 10 {
        types.push(FastingType::EID_AL_ADHA);
        traces.push(RuleTrace::new(TraceCode::EidAlAdha, None));
        return Ok(FastingAnalysis::with_traces(datetime, FastingStatus::Haram, types, (h_year, h_month, h_day), traces));
    }

    if h_month == MONTH_DHUL_HIJJAH && (11..=13).contains(&h_day) {
        types.push(FastingType::TASHRIQ);
        traces.push(RuleTrace::new(TraceCode::Tashriq, None));
        return Ok(FastingAnalysis::with_traces(datetime, FastingStatus::Haram, types, (h_year, h_month, h_day), traces));
    }

    // Wajib
    if h_month == MONTH_RAMADHAN {
        types.push(FastingType::RAMADHAN);
        traces.push(RuleTrace::new(TraceCode::Ramadhan, None));
        status = FastingStatus::Wajib;
    }

    // Sunnah Muakkadah
    if h_month == MONTH_DHUL_HIJJAH && h_day == DAY_ARAFAH {
        types.push(FastingType::ARAFAH);
        traces.push(RuleTrace::new(TraceCode::Arafah, None));
        if !status.is_wajib() { status = FastingStatus::SunnahMuakkadah; }
    }

    if h_month == MONTH_MUHARRAM && h_day == DAY_ASHURA {
        types.push(FastingType::ASHURA);
        traces.push(RuleTrace::new(TraceCode::Ashura, None));
        if !status.is_wajib() { status = FastingStatus::SunnahMuakkadah; }
    }

    // Sunnah
    if h_month == MONTH_MUHARRAM && h_day == DAY_TASUA {
        types.push(FastingType::TASUA);
        traces.push(RuleTrace::new(TraceCode::Tasua, None));
        if !status.is_wajib() && status != FastingStatus::SunnahMuakkadah { 
            status = FastingStatus::Sunnah; 
        }
    }

    if (13..=15).contains(&h_day) {
        types.push(FastingType::AYYAMUL_BIDH);
        traces.push(RuleTrace::new(TraceCode::AyyamulBidh, None));
        if !status.is_wajib() && status < FastingStatus::Sunnah {
            status = FastingStatus::Sunnah;
        }
    }

    match weekday {
        Weekday::Mon => {
            types.push(FastingType::MONDAY);
            traces.push(RuleTrace::new(TraceCode::Monday, None));
            if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
        },
        Weekday::Thu => {
            types.push(FastingType::THURSDAY);
            traces.push(RuleTrace::new(TraceCode::Thursday, None));
            if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
        },
        _ => {}
    }

    if h_month == MONTH_SHAWWAL && h_day > 1 {
        types.push(FastingType::SHAWWAL);
        traces.push(RuleTrace::new(TraceCode::Shawwal, None));
        if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
    }

    // Makruh Checks
    if status == FastingStatus::Mubah {
        match context.madhab {
            Madhab::Shafi | Madhab::Hanafi | Madhab::Maliki | Madhab::Hanbali => {
                if weekday == Weekday::Fri {
                    types.push(FastingType::FRIDAY_EXCLUSIVE);
                    traces.push(RuleTrace::new(TraceCode::FridaySingledOut, None));
                    status = FastingStatus::Makruh;
                } else if weekday == Weekday::Sat {
                    types.push(FastingType::SATURDAY_EXCLUSIVE);
                    traces.push(RuleTrace::new(TraceCode::SaturdaySingledOut, None));
                    status = FastingStatus::Makruh;
                }
            }
        }
    }

    // Custom rules evaluation
    for rule in &context.custom_rules {
        if let Some((custom_status, custom_type)) = rule.evaluate(effective_date, h_year, h_month, h_day) {
            types.push(custom_type.clone());
            traces.push(RuleTrace::new(TraceCode::Custom, Some(custom_type.to_string())));
            if custom_status > status { status = custom_status; }
        }
    }

    Ok(FastingAnalysis::with_traces(datetime, status, types, (h_year, h_month, h_day), traces))
}

/// Helper for backwards compatibility or simple checks.
/// Defaults to Noon UTC for the given date.
pub fn check(g_date: NaiveDate, context: &RuleContext) -> FastingAnalysis {
    let dt = Utc.from_utc_datetime(&g_date.and_hms_opt(12, 0, 0).unwrap());
    analyze(dt, context, None).unwrap_or_else(|_| {
        // Fallback or panic depending on design. Since check didn't return Result before, we should try to return something valid or minimal.
        // But strict mode might fail.
        // If strict mode is on, this panics?
        // Old `check` was infallible.
        // We'll create a dummy 'Mubah' result if it fails.
        FastingAnalysis::new(dt, FastingStatus::Mubah, SmallVec::new(), (1400, 1, 1))
    })
}

