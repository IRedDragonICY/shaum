//! # Shaum - Islamic Fasting Rules Engine
//!
//! Determines fasting status (Wajib, Sunnah, Makruh, Haram) for any date.
//!
//! ## Quick Start
//!
//! ```rust
//! use chrono::NaiveDate;
//! use shaum::prelude::*;
//!
//! let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
//!
//! // Extension API
//! if date.is_wajib() { println!("Ramadhan!"); }
//!
//! // Full analysis
//! let analysis = date.fasting_analysis();
//! println!("{}", analysis.explain());
//! ```
//!
//! ## Query Engine
//!
//! ```rust
//! use chrono::NaiveDate;
//! use shaum::query::QueryExt;
//!
//! let date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
//! let sunnah: Vec<_> = date.upcoming_fasts()
//!     .sunnah()
//!     .take(5)
//!     .collect();
//! ```
//!
//! ## Status Priority
//!
//! Haram > Wajib > SunnahMuakkadah > Sunnah > Makruh > Mubah

pub mod calendar;
pub mod rules;
pub mod types;
pub mod constants;
pub mod i18n;
pub mod extension;
pub mod query;
pub mod macros;

pub use types::{FastingStatus, FastingType, FastingAnalysis, Madhab, DaudStrategy, GeoCoordinate, TraceCode};
pub use rules::{analyze, check};
pub use calendar::ShaumError; // Keeping ShaumError for now as types might use it, simplified
pub use calendar::to_hijri;
pub use rules::{RuleContext, MoonProvider};

/// Re-exports for convenience.
pub mod prelude {
    pub use crate::types::{FastingStatus, FastingType, FastingAnalysis, Madhab, DaudStrategy, GeoCoordinate, TraceCode};
    pub use crate::analyze;
    pub use crate::check;
    pub use crate::analyze_date;
    pub use crate::to_hijri;
    pub use crate::{RuleContext, ShaumError, MoonProvider};
    pub use crate::extension::ShaumDateExt;
    pub use crate::query::{FastingQuery, QueryExt};
}

use chrono::NaiveDate;

/// Analyzes date with default context. Infallible.
pub fn analyze_date(date: NaiveDate) -> FastingAnalysis {
    check(date, &RuleContext::default())
}



/// Daud fasting iterator.
pub struct DaudIterator {
    current: NaiveDate,
    end: NaiveDate,
    should_fast: bool,
    context: RuleContext,
    debt: u32,
}

impl DaudIterator {
    pub fn new(start: NaiveDate, end: NaiveDate, context: RuleContext) -> Self {
        Self { current: start, end, should_fast: true, context, debt: 0 }
    }

    pub fn starting_from(date: NaiveDate) -> DaudScheduleBuilder {
        DaudScheduleBuilder::new(date)
    }

    pub fn debt(&self) -> u32 { self.debt }
}

impl Iterator for DaudIterator {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current <= self.end {
            let analysis = check(self.current, &self.context);
            let is_haram = analysis.primary_status.is_haram();
            let date_to_emit = self.current;
            self.current = self.current.succ_opt()?;

            if is_haram {
                match self.context.daud_strategy {
                    DaudStrategy::Skip => { self.should_fast = !self.should_fast; },
                    DaudStrategy::Postpone => { /* keep state */ }
                }
                continue;
            } else if self.should_fast {
                self.should_fast = !self.should_fast;
                return Some(date_to_emit);
            } else {
                self.should_fast = !self.should_fast;
                continue;
            }
        }
        None
    }
}

/// Builder for Daud fasting schedule.
pub struct DaudScheduleBuilder {
    start: NaiveDate,
    end: Option<NaiveDate>,
    context: RuleContext,
}

impl DaudScheduleBuilder {
    pub fn new(start: NaiveDate) -> Self {
        Self { start, end: None, context: RuleContext::default() }
    }

    pub fn until(mut self, end: NaiveDate) -> Self { self.end = Some(end); self }
    pub fn with_context(mut self, ctx: RuleContext) -> Self { self.context = ctx; self }
    pub fn skip_haram_days(mut self) -> Self { self.context = self.context.daud_strategy(DaudStrategy::Skip); self }
    pub fn postpone_on_haram(mut self) -> Self { self.context = self.context.daud_strategy(DaudStrategy::Postpone); self }

    pub fn build(self) -> DaudIterator {
        let end = self.end.unwrap_or_else(|| self.start + chrono::Duration::days(365));
        DaudIterator::new(self.start, end, self.context)
    }
}

/// Creates Daud schedule iterator.
pub fn generate_daud_schedule(start: NaiveDate, end: NaiveDate, context: RuleContext) -> DaudIterator {
    DaudIterator::new(start, end, context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    fn find_hijri_date(year: usize, month: usize, day: usize) -> NaiveDate {
        let mut d = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        for _ in 0..2000 {
            let h = to_hijri(d, 0);
            if h.year() == year && h.month() == month && h.day() == day { return d; }
            d = d.succ_opt().unwrap();
        }
        panic!("Date not found for {}/{}/{}", year, month, day);
    }

    #[test]
    fn test_eid_al_fitr_is_haram() {
        let eid = find_hijri_date(1445, 10, 1);
        let analysis = check(eid, &RuleContext::default());
        assert!(analysis.primary_status.is_haram());
        assert!(analysis.has_reason(&FastingType::EID_AL_FITR));
    }

    #[test]
    fn test_eid_al_adha_is_haram() {
        let eid = find_hijri_date(1445, 12, 10);
        let analysis = check(eid, &RuleContext::default());
        assert!(analysis.primary_status.is_haram());
        assert!(analysis.has_reason(&FastingType::EID_AL_ADHA));
    }

    #[test]
    fn test_tashriq_haram() {
        let tashriq = find_hijri_date(1445, 12, 11);
        let analysis = check(tashriq, &RuleContext::default());
        assert!(analysis.primary_status.is_haram());
        assert!(analysis.has_reason(&FastingType::TASHRIQ));
    }

    #[test]
    fn test_ramadhan_wajib() {
        let ramadhan = find_hijri_date(1445, 9, 1);
        let analysis = check(ramadhan, &RuleContext::default());
        assert!(analysis.primary_status.is_wajib());
        assert!(analysis.has_reason(&FastingType::RAMADHAN));
    }

    #[test]
    fn test_arafah_sunnah() {
        let arafah = find_hijri_date(1445, 12, 9);
        let analysis = check(arafah, &RuleContext::default());
        assert_eq!(analysis.primary_status, FastingStatus::SunnahMuakkadah);
        assert!(analysis.has_reason(&FastingType::ARAFAH));
    }

    #[test]
    fn test_friday_makruh_vs_sunnah() {
        let mut d = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        for _ in 0..5000 {
            let h = to_hijri(d, 0);
            if h.month() == 12 && h.day() == 9 && d.weekday() == chrono::Weekday::Fri {
                let analysis = check(d, &RuleContext::default());
                assert_eq!(analysis.primary_status, FastingStatus::SunnahMuakkadah);
                return;
            }
            d = d.succ_opt().unwrap();
        }
        panic!("Could not find Arafah on Friday");
    }

    #[test]
    fn test_adjustment_shifts_date() {
        let d = find_hijri_date(1445, 9, 1);
        let ctx = RuleContext::new().adjustment(-1);
        let analysis = check(d, &ctx);
        assert_ne!(analysis.primary_status, FastingStatus::Wajib);
    }

    #[test]
    fn test_daud_schedule() {
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = start + chrono::Duration::days(10);
        let schedule: Vec<NaiveDate> = generate_daud_schedule(start, end, RuleContext::default())
            .collect();
        assert!(!schedule.is_empty());
        for w in schedule.windows(2) {
            assert!((w[1] - w[0]).num_days() >= 2);
        }
    }

    #[test]
    fn test_explain_output() {
        let ramadhan = find_hijri_date(1445, 9, 15);
        let analysis = check(ramadhan, &RuleContext::default());
        assert!(analysis.explain().contains("Ramadhan"));
    }

    #[test]
    fn test_daud_schedule_builder() {
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();
        let days: Vec<_> = DaudScheduleBuilder::new(start).until(end).skip_haram_days().build()
            .collect();
        assert!(!days.is_empty());
    }
}
