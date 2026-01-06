pub mod calendar;
pub mod rules;
pub mod types;

pub use types::{FastingStatus, FastingType, FastingAnalysis, Madhab, DaudStrategy};
pub use rules::check as analyze;
pub use calendar::{to_hijri, ShaumError};
pub use rules::{RuleContext, MoonProvider};

pub mod prelude {
    pub use crate::types::*;
    pub use crate::analyze;
    pub use crate::to_hijri;
    pub use crate::{RuleContext, ShaumError};
}

use chrono::NaiveDate;

/// Iterator for generating Daud fasting schedule lazily.
pub struct DaudIterator {
    current: NaiveDate,
    end: NaiveDate,
    should_fast: bool,
    context: RuleContext,
}

impl Iterator for DaudIterator {
    type Item = Result<NaiveDate, ShaumError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current <= self.end {
            // Check status
            let analysis_res = analyze(self.current, &self.context);
            
            match analysis_res {
                Ok(analysis) => {
                    let is_haram = analysis.primary_status.is_haram();
                    let date_to_emit = self.current;
                    
                    // Advance date for next iteration
                    self.current = self.current.succ_opt()?; 

                    if is_haram {
                        match self.context.daud_strategy {
                            DaudStrategy::Skip => {
                                self.should_fast = !self.should_fast;
                            },
                            DaudStrategy::Postpone => {
                                // Do not toggle, try again tomorrow
                            }
                        }
                        continue;
                    } else if self.should_fast {
                        self.should_fast = !self.should_fast;
                        return Some(Ok(date_to_emit));
                    } else {
                        self.should_fast = !self.should_fast;
                        continue;
                    }
                },
                Err(e) => return Some(Err(e)),
            }
        }
        None
    }
}

/// Generates a Daud fasting schedule (skip one day) excluding Haram days.
/// Returns an iterator.
pub fn generate_daud_schedule(start: NaiveDate, end: NaiveDate, context: RuleContext) -> DaudIterator {
    DaudIterator {
        current: start,
        end,
        should_fast: true, // Start with fasting unless Haram
        context,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, NaiveDate};

    // Helper to find a specific hijri date in a range (brute force for testing stability)
    fn find_hijri_date(year: usize, month: usize, day: usize) -> NaiveDate {
        let mut d = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        // Limit search to avoiding infinite loop
        for _ in 0..2000 {
            let h = to_hijri(d, 0).expect("Hijri conversion failed");
            if h.year() == year && h.month() == month && h.day() == day {
                return d;
            }
            d = d.succ_opt().unwrap();
        }
        panic!("Date not found for {}/{}/{}", year, month, day);
    }

    #[test]
    fn test_eid_al_fitr_is_haram() {
        // Find 1 Shawwal 1445 (approx April 2024)
        let eid = find_hijri_date(1445, 10, 1);
        let ctx = RuleContext::default();
        let analysis = analyze(eid, &ctx).unwrap();
        assert!(analysis.primary_status.is_haram());
        assert!(analysis.types.contains(&FastingType::EidAlFitr));
    }

    #[test]
    fn test_eid_al_adha_is_haram() {
        // Find 10 Dhul Hijjah 1445
        let eid = find_hijri_date(1445, 12, 10);
        let ctx = RuleContext::default();
        let analysis = analyze(eid, &ctx).unwrap();
        assert!(analysis.primary_status.is_haram());
        assert!(analysis.types.contains(&FastingType::EidAlAdha));
    }

    #[test]
    fn test_tashriq_haram() {
        // 11 Dhul Hijjah
        let tashriq = find_hijri_date(1445, 12, 11);
        let ctx = RuleContext::default();
        let analysis = analyze(tashriq, &ctx).unwrap();
        assert!(analysis.primary_status.is_haram());
        assert!(analysis.types.contains(&FastingType::Tashriq));
    }

    #[test]
    fn test_ramadhan_wajib() {
        // 1 Ramadhan 1445
        let ramadhan = find_hijri_date(1445, 9, 1);
        let ctx = RuleContext::default();
        let analysis = analyze(ramadhan, &ctx).unwrap();
        assert!(analysis.primary_status.is_wajib());
        assert!(analysis.types.contains(&FastingType::Ramadhan));
    }

    #[test]
    fn test_arafah_sunnah() {
        // 9 Dhul Hijjah 1445
        let arafah = find_hijri_date(1445, 12, 9);
        let ctx = RuleContext::default();
        let analysis = analyze(arafah, &ctx).unwrap();
        // Arafah is Sunnah Muakkadah
        assert_eq!(analysis.primary_status, FastingStatus::SunnahMuakkadah);
        assert!(analysis.types.contains(&FastingType::Arafah));
    }

    #[test]
    fn test_friday_makruh_vs_sunnah() {
        // We need 9 Dhul Hijjah to be a Friday.
        let mut d = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let mut found = false;
        
        for _ in 0..5000 {
            if let Ok(h) = to_hijri(d, 0) {
                if h.month() == 12 && h.day() == 9 && d.weekday() == chrono::Weekday::Fri {
                    let ctx = RuleContext::default();
                    let analysis = analyze(d, &ctx).unwrap();
                    // Should be Sunnah, NOT Makruh.
                    assert_eq!(analysis.primary_status, FastingStatus::SunnahMuakkadah);
                    assert!(!analysis.primary_status.is_haram());
                    assert_ne!(analysis.primary_status, FastingStatus::Makruh);
                    found = true;
                    break;
                }
            }
            d = d.succ_opt().unwrap();
        }
        assert!(found, "Could not find an Arafah on Friday for testing");
    }

    #[test]
    fn test_adjustment_shifts_date() {
        // 1st Ramadhan Unadjusted
        let d = find_hijri_date(1445, 9, 1);
        
        // If we adjust -1
        let ctx = RuleContext { adjustment: -1, ..Default::default() };
        let analysis = analyze(d, &ctx).unwrap();
        
        assert_ne!(analysis.primary_status, FastingStatus::Wajib); // Should not be Wajib/Ramadhan
    }

    #[test]
    fn test_daud_schedule() {
         let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
         let end = start + chrono::Duration::days(10);
         let schedule_iter = generate_daud_schedule(start, end, RuleContext::default());
         let schedule: Vec<NaiveDate> = schedule_iter.filter_map(|r| r.ok()).collect();
         
         // Just check it's not empty and skips days.
         assert!(schedule.len() > 0);
         // Ensure no consecutive days (unless skipped due to Haram?)
         for w in schedule.windows(2) {
             let diff = w[1] - w[0];
             assert!(diff.num_days() >= 2, "Daud schedule should skip at least one day");
         }
    }
}
