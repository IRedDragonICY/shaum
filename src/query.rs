//! Fluent query engine for finding fasting dates.
 
use chrono::NaiveDate;
use crate::rules::{check, RuleContext};
use crate::types::{FastingAnalysis, FastingType};
use crate::calendar::ShaumError;

/// Query filter mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    All,
    Wajib,
    Sunnah,
    Haram,
    Makruh,
    Mubah,
}

/// Fluent query builder for fasting dates.
#[derive(Debug, Clone)]
pub struct FastingQuery {
    current: NaiveDate,
    end: Option<NaiveDate>,
    context: RuleContext,
    filter: FilterMode,
    exclude_haram: bool,
    exclude_makruh: bool,
    require_type: Option<FastingType>,
}

impl FastingQuery {
    /// Creates query starting from `date`.
    pub fn starting_from(date: NaiveDate) -> Self {
        Self {
            current: date,
            end: None,
            context: RuleContext::default(),
            filter: FilterMode::All,
            exclude_haram: false,
            exclude_makruh: false,
            require_type: None,
        }
    }

    /// Sets end date (inclusive).
    pub fn until(mut self, date: NaiveDate) -> Self { self.end = Some(date); self }
    
    /// Sets custom context.
    pub fn with_context(mut self, ctx: RuleContext) -> Self { self.context = ctx; self }
    
    /// Filters to Wajib only.
    pub fn wajib(mut self) -> Self { self.filter = FilterMode::Wajib; self }
    
    /// Filters to Sunnah only.
    pub fn sunnah(mut self) -> Self { self.filter = FilterMode::Sunnah; self }
    
    /// Filters to Haram only.
    pub fn haram(mut self) -> Self { self.filter = FilterMode::Haram; self }
    
    /// Filters to Makruh only.
    pub fn makruh(mut self) -> Self { self.filter = FilterMode::Makruh; self }
    
    /// Excludes Haram days.
    pub fn exclude_haram(mut self) -> Self { self.exclude_haram = true; self }
    
    /// Excludes Makruh days.
    pub fn exclude_makruh(mut self) -> Self { self.exclude_makruh = true; self }
    
    /// Requires specific fasting type.
    pub fn with_type(mut self, ftype: FastingType) -> Self { self.require_type = Some(ftype); self }

    fn matches(&self, analysis: &FastingAnalysis) -> bool {
        if self.exclude_haram && analysis.primary_status.is_haram() { return false; }
        if self.exclude_makruh && analysis.primary_status.is_makruh() { return false; }
        if let Some(ref t) = self.require_type { if !analysis.has_reason(t) { return false; } }

        match self.filter {
            FilterMode::All => true,
            FilterMode::Wajib => analysis.primary_status.is_wajib(),
            FilterMode::Sunnah => analysis.primary_status.is_sunnah(),
            FilterMode::Haram => analysis.primary_status.is_haram(),
            FilterMode::Makruh => analysis.primary_status.is_makruh(),
            FilterMode::Mubah => analysis.primary_status.is_mubah(),
        }
    }
}

impl Iterator for FastingQuery {
    type Item = Result<FastingAnalysis, ShaumError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(end) = self.end { if self.current > end { return None; } }
            let date = self.current;
            self.current = self.current.succ_opt()?;

            // Propagate errors from check
            let analysis = match check(date, &self.context) {
                Ok(a) => a,
                Err(e) => return Some(Err(e)),
            };

            if self.matches(&analysis) {
                return Some(Ok(analysis));
            }
        }
    }
}

/// Extension for query creation.
pub trait QueryExt {
    /// Creates query for upcoming fasts.
    fn upcoming_fasts(&self) -> FastingQuery;
}

impl QueryExt for NaiveDate {
    fn upcoming_fasts(&self) -> FastingQuery { FastingQuery::starting_from(*self) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_query() {
        let start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let results: Vec<_> = FastingQuery::starting_from(start).take(5).collect();
        assert_eq!(results.len(), 5);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_sunnah_filter() {
        let start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let results: Vec<_> = FastingQuery::starting_from(start).sunnah().take(3).collect();
        for r in &results { 
            assert!(r.as_ref().unwrap().primary_status.is_sunnah()); 
        }
    }

    #[test]
    fn test_until_bound() {
        let start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 5).unwrap();
        let results: Vec<_> = FastingQuery::starting_from(start).until(end).collect();
        assert!(results.len() <= 5);
    }

    #[test]
    fn test_query_ext_trait() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let results: Vec<_> = date.upcoming_fasts().take(3).collect();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_error_propagation() {
        // Year 3000 should fail
        let start = NaiveDate::from_ymd_opt(2077, 1, 1).unwrap();
        let mut query = FastingQuery::starting_from(start);
        let result = query.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_err());
    }
}
