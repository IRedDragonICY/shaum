use chrono::{Datelike, NaiveDate, Weekday, DateTime, Utc, TimeZone};
use crate::calendar::{ShaumError, to_hijri, HIJRI_MIN_YEAR, HIJRI_MAX_YEAR};
use crate::types::{FastingAnalysis, FastingStatus, FastingType, Madhab, DaudStrategy, RuleTrace, TraceCode, GeoCoordinate, VisibilityCriteria, TracePayload};
use crate::constants::*;
use serde::Serialize;
#[cfg(feature = "async")]
use serde::Deserialize;
use smallvec::SmallVec;

/// Moon sighting adjustment provider.
/// 
/// When the `async` feature is enabled, returns a pinned boxed future.
/// Otherwise, returns a synchronous result.
pub trait MoonProvider: std::fmt::Debug + Send + Sync {
    #[cfg(feature = "async")]
    fn get_adjustment(
        &self,
        date: NaiveDate,
        coords: Option<GeoCoordinate>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<i64, ShaumError>> + Send + '_>>;
    
    #[cfg(not(feature = "async"))]
    fn get_adjustment(&self, date: NaiveDate, coords: Option<GeoCoordinate>) -> Result<i64, ShaumError>;
}

/// Fixed day offset for all dates.
#[derive(Debug, Clone, Copy, Default)]
pub struct FixedAdjustment(pub i64);

impl FixedAdjustment {
    pub fn new(offset: i64) -> Self { Self(offset.clamp(-30, 30)) }
}

impl MoonProvider for FixedAdjustment {
    #[cfg(feature = "async")]
    fn get_adjustment(
        &self,
        _date: NaiveDate,
        _coords: Option<GeoCoordinate>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<i64, ShaumError>> + Send + '_>> {
        let val = self.0;
        Box::pin(async move { Ok(val) })
    }

    #[cfg(not(feature = "async"))]
    fn get_adjustment(&self, _date: NaiveDate, _coords: Option<GeoCoordinate>) -> Result<i64, ShaumError> {
        Ok(self.0)
    }
}

/// No adjustment (use astronomical calculation).
#[derive(Debug, Clone, Copy, Default)]
pub struct NoAdjustment;

impl MoonProvider for NoAdjustment {
    #[cfg(feature = "async")]
    fn get_adjustment(
        &self,
        _date: NaiveDate,
        _coords: Option<GeoCoordinate>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<i64, ShaumError>> + Send + '_>> {
        Box::pin(async move { Ok(0) })
    }

    #[cfg(not(feature = "async"))]
    fn get_adjustment(&self, _date: NaiveDate, _coords: Option<GeoCoordinate>) -> Result<i64, ShaumError> {
        Ok(0)
    }
}

/// Remote moon provider fetching adjustment from an API.
#[cfg(feature = "async")]
#[derive(Debug, Clone)]
pub struct RemoteMoonProvider {
    endpoint: String,
    client: reqwest::Client,
}

#[cfg(feature = "async")]
impl RemoteMoonProvider {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[cfg(feature = "async")]
impl MoonProvider for RemoteMoonProvider {
    fn get_adjustment(
        &self,
        _date: NaiveDate,
        _coords: Option<GeoCoordinate>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<i64, ShaumError>> + Send + '_>> {
        let endpoint = self.endpoint.clone();
        let client = self.client.clone();
        
        Box::pin(async move {
            #[derive(Deserialize)]
            struct AdjustmentResponse {
                adjustment: i64,
            }

            let resp = client.get(&endpoint)
                .send()
                .await
                .map_err(|e| ShaumError::NetworkError(e.to_string()))?;
                
            let data = resp.json::<AdjustmentResponse>()
                .await
                .map_err(|e| ShaumError::NetworkError(e.to_string()))?;
                
            Ok(data.adjustment)
        })
    }
}

/// Interface for calculating sunset time.
pub trait SunsetProvider: std::fmt::Debug + Send + Sync {
    /// Returns the sunset timestamp for a given date and coordinate.
    fn get_sunset(&self, date: NaiveDate, coords: GeoCoordinate) -> Result<DateTime<Utc>, ShaumError>;
}

/// Default sunset calculator using VSOP87 astronomy engine.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultSunsetProvider;

impl SunsetProvider for DefaultSunsetProvider {
    fn get_sunset(&self, date: NaiveDate, coords: GeoCoordinate) -> Result<DateTime<Utc>, ShaumError> {
        // Use the astronomy engine for accurate sunset calculation
        crate::astronomy::visibility::estimate_sunset(date, coords)
    }
}

/// Custom rule trait.
pub trait CustomFastingRule: std::fmt::Debug + Send + Sync {
    fn evaluate(&self, date: NaiveDate, hijri_year: usize, hijri_month: usize, hijri_day: usize) 
        -> Option<(FastingStatus, FastingType)>;
}

/// Rule engine configuration.
#[derive(Debug, Serialize)] // Removing Deserialize because dynamic traits (SunsetProvider) are hard to deserialize without specific logic
pub struct RuleContext {
    /// Hijri day offset. Clamped to [-30, 30].
    pub adjustment: i64,
    pub madhab: Madhab,
    pub daud_strategy: DaudStrategy,
    pub strict: bool,
    /// Moon visibility criteria for hilal observation.
    pub visibility_criteria: VisibilityCriteria,
    #[serde(skip)]
    pub custom_rules: Vec<Box<dyn CustomFastingRule>>,
    #[serde(skip)]
    pub sunset_provider: Box<dyn SunsetProvider>,
}

impl Clone for RuleContext {
    fn clone(&self) -> Self {
        Self {
            adjustment: self.adjustment,
            madhab: self.madhab,
            daud_strategy: self.daud_strategy,
            strict: self.strict,
            visibility_criteria: self.visibility_criteria,
            custom_rules: Vec::new(),
            sunset_provider: Box::new(DefaultSunsetProvider), // Resetting provider on clone as we can't clone trait object easily without `dyn Clone`
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
            visibility_criteria: VisibilityCriteria::default(),
            custom_rules: Vec::new(),
            sunset_provider: Box::new(DefaultSunsetProvider),
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

    pub fn with_sunset_provider<P: SunsetProvider + 'static>(mut self, provider: P) -> Self {
        self.sunset_provider = Box::new(provider);
        self
    }

    /// Sets moon visibility criteria.
    pub fn visibility_criteria(mut self, criteria: VisibilityCriteria) -> Self {
        self.visibility_criteria = criteria;
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
    sunset_provider: Option<Box<dyn SunsetProvider>>,
    visibility_criteria: Option<VisibilityCriteria>,
    strict_adjustment: bool,
    strict_mode: bool,
}

impl RuleContextBuilder {
    pub fn new() -> Self { Self::default() }
    
    pub fn adjustment(mut self, adjustment: i64) -> Self { self.adjustment = Some(adjustment); self }
    pub fn madhab(mut self, madhab: Madhab) -> Self { self.madhab = Some(madhab); self }
    pub fn daud_strategy(mut self, strategy: DaudStrategy) -> Self { self.daud_strategy = Some(strategy); self }
    pub fn add_custom_rule(mut self, rule: Box<dyn CustomFastingRule>) -> Self { self.custom_rules.push(rule); self }
    pub fn with_sunset_provider<P: SunsetProvider + 'static>(mut self, provider: P) -> Self {
        self.sunset_provider = Some(Box::new(provider));
        self
    }
    
    /// Enables strict adjustment bounds [-2, 2].
    pub fn strict_adjustment(mut self, strict: bool) -> Self { self.strict_adjustment = strict; self }

    /// Sets moon visibility criteria.
    pub fn visibility_criteria(mut self, criteria: VisibilityCriteria) -> Self { 
        self.visibility_criteria = Some(criteria); 
        self 
    }

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
            visibility_criteria: self.visibility_criteria.unwrap_or_default(),
            sunset_provider: self.sunset_provider.unwrap_or_else(|| Box::new(DefaultSunsetProvider)),
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
        // Use provider from context
        let sunset = context.sunset_provider.get_sunset(effective_date, c)?;
        if datetime > sunset {
            effective_date = effective_date.succ_opt()
                .ok_or_else(|| ShaumError::date_out_of_range(effective_date))?;
            traces.push(RuleTrace::new(TraceCode::Debug, TracePayload::PostMaghribOffset));
        }
    }

    // 2. Strict Mode Check (handled by to_hijri implicitly returning error if out of range)
    // But we check bounds here too to be nice?
    // Actually to_hijri will error out.
    // If strict is OFF, we might want to handle error "gracefully" if it's purely a range issue?
    // But the prompt says "NO PANICS: Remove unwrap... Use Result propagation".
    // So if to_hijri fails, analyze fails.
    
    let year = effective_date.year();
    if (year < HIJRI_MIN_YEAR || year > HIJRI_MAX_YEAR) && context.strict {
         return Err(ShaumError::date_out_of_range(effective_date));
    }

    // This propagates error.
    let h_date = to_hijri(effective_date, context.adjustment)?;
    
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
        traces.push(RuleTrace::simple(TraceCode::EidAlFitr));
        return Ok(FastingAnalysis::with_traces(datetime, FastingStatus::Haram, types, (h_year, h_month, h_day), traces));
    }

    if h_month == MONTH_DHUL_HIJJAH && h_day == 10 {
        types.push(FastingType::EID_AL_ADHA);
        traces.push(RuleTrace::simple(TraceCode::EidAlAdha));
        return Ok(FastingAnalysis::with_traces(datetime, FastingStatus::Haram, types, (h_year, h_month, h_day), traces));
    }

    if h_month == MONTH_DHUL_HIJJAH && (11..=13).contains(&h_day) {
        types.push(FastingType::TASHRIQ);
        traces.push(RuleTrace::simple(TraceCode::Tashriq));
        return Ok(FastingAnalysis::with_traces(datetime, FastingStatus::Haram, types, (h_year, h_month, h_day), traces));
    }

    // Wajib
    if h_month == MONTH_RAMADHAN {
        types.push(FastingType::RAMADHAN);
        traces.push(RuleTrace::simple(TraceCode::Ramadhan));
        status = FastingStatus::Wajib;
    }

    // Sunnah Muakkadah
    if h_month == MONTH_DHUL_HIJJAH && h_day == DAY_ARAFAH {
        types.push(FastingType::ARAFAH);
        traces.push(RuleTrace::simple(TraceCode::Arafah));
        if !status.is_wajib() { status = FastingStatus::SunnahMuakkadah; }
    }

    if h_month == MONTH_MUHARRAM && h_day == DAY_ASHURA {
        types.push(FastingType::ASHURA);
        traces.push(RuleTrace::simple(TraceCode::Ashura));
        if !status.is_wajib() { status = FastingStatus::SunnahMuakkadah; }
    }

    // Sunnah
    if h_month == MONTH_MUHARRAM && h_day == DAY_TASUA {
        types.push(FastingType::TASUA);
        traces.push(RuleTrace::simple(TraceCode::Tasua));
        if !status.is_wajib() && status != FastingStatus::SunnahMuakkadah { 
            status = FastingStatus::Sunnah; 
        }
    }

    if (13..=15).contains(&h_day) {
        types.push(FastingType::AYYAMUL_BIDH);
        traces.push(RuleTrace::simple(TraceCode::AyyamulBidh));
        if !status.is_wajib() && status < FastingStatus::Sunnah {
            status = FastingStatus::Sunnah;
        }
    }

    match weekday {
        Weekday::Mon => {
            types.push(FastingType::MONDAY);
            traces.push(RuleTrace::simple(TraceCode::Monday));
            if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
        },
        Weekday::Thu => {
            types.push(FastingType::THURSDAY);
            traces.push(RuleTrace::simple(TraceCode::Thursday));
            if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
        },
        _ => {}
    }

    if h_month == MONTH_SHAWWAL && h_day > 1 {
        types.push(FastingType::SHAWWAL);
        traces.push(RuleTrace::simple(TraceCode::Shawwal));
        if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
    }

    // Makruh Checks
    if status == FastingStatus::Mubah {
        match context.madhab {
            Madhab::Shafi | Madhab::Hanafi | Madhab::Maliki | Madhab::Hanbali => {
                if weekday == Weekday::Fri {
                    types.push(FastingType::FRIDAY_EXCLUSIVE);
                    traces.push(RuleTrace::simple(TraceCode::FridaySingledOut));
                    status = FastingStatus::Makruh;
                } else if weekday == Weekday::Sat {
                    types.push(FastingType::SATURDAY_EXCLUSIVE);
                    traces.push(RuleTrace::simple(TraceCode::SaturdaySingledOut));
                    status = FastingStatus::Makruh;
                }
            }
        }
    }

    // Custom rules evaluation
    for rule in &context.custom_rules {
        if let Some((custom_status, custom_type)) = rule.evaluate(effective_date, h_year, h_month, h_day) {
            types.push(custom_type.clone());
            traces.push(RuleTrace::new(TraceCode::Custom, TracePayload::CustomReason(custom_type.to_string())));
            if custom_status > status { status = custom_status; }
        }
    }

    Ok(FastingAnalysis::with_traces(datetime, status, types, (h_year, h_month, h_day), traces))
}

/// Checks fasting status for a given date.
/// Defaults to Noon UTC.
/// 
/// Returns `Result<FastingAnalysis, ShaumError>` (Changed from infallible).
pub fn check(g_date: NaiveDate, context: &RuleContext) -> Result<FastingAnalysis, ShaumError> {
    let dt = Utc.from_utc_datetime(&g_date.and_hms_opt(12, 0, 0).unwrap());
    analyze(dt, context, None)
}

