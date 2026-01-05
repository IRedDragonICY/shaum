pub mod calendar;
pub mod rules;
pub mod types;

pub use types::{FastingStatus, FastingType, FastingAnalysis};
pub use rules::check as analyze;
pub use calendar::to_hijri;

pub mod prelude {
    pub use crate::types::*;
    pub use crate::analyze;
    pub use crate::to_hijri;
}

use chrono::NaiveDate;

/// Generates a Daud fasting schedule (skip one day) excluding Haram days.
pub fn generate_daud_schedule(start: NaiveDate, end: NaiveDate, adjustment: i64) -> Vec<NaiveDate> {
    let mut schedule = Vec::new();
    let mut current = start;
    let mut should_fast = true; // Assume start is fasting day unless Haram.

    while current <= end {
        let analysis = analyze(current, adjustment);
        
        if analysis.primary_status.is_haram() {
            // Cannot fast. 
            // Logic for Daud on Haram days:
            // "If your turn to fast falls on a prohibited day, you skip it."
            // The pattern continues? Or does it pause?
            // Usually, you just don't fast. The pattern of 1-on-1-off implies "Intention" is there.
            // But practically, we just skip this day.
            // Does the NEXT day become FAST?
            // "Fast one day, break one day".
            // If Day 1 (Masked Haram) -> Skip. Day 2 -> Break?
            // Or does the cycle continue? Day 1 (Haram/Break), Day 2 (Fast).
            // Usually, specific days (Arafah etc) are prioritized.
            // For this utility, we will simply Check if Mubah/Sunnah/Wajib, and toggle.
            
            // Simpler approach:
            // Just emit "valid" days according to the toggle.
            // However, we must ensure we don't output Haram days.
        } else if should_fast {
            schedule.push(current);
        }

        current = current.succ_opt().unwrap();
        should_fast = !should_fast;
    }

    schedule
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
            let h = to_hijri(d, 0);
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
        let analysis = analyze(eid, 0);
        assert!(analysis.primary_status.is_haram());
        assert!(analysis.types.contains(&FastingType::EidAlFitr));
    }

    #[test]
    fn test_eid_al_adha_is_haram() {
        // Find 10 Dhul Hijjah 1445
        let eid = find_hijri_date(1445, 12, 10);
        let analysis = analyze(eid, 0);
        assert!(analysis.primary_status.is_haram());
        assert!(analysis.types.contains(&FastingType::EidAlAdha));
    }

    #[test]
    fn test_tashriq_haram() {
        // 11 Dhul Hijjah
        let tashriq = find_hijri_date(1445, 12, 11);
        let analysis = analyze(tashriq, 0);
        assert!(analysis.primary_status.is_haram());
        assert!(analysis.types.contains(&FastingType::Tashriq));
    }

    #[test]
    fn test_ramadhan_wajib() {
        // 1 Ramadhan 1445
        let ramadhan = find_hijri_date(1445, 9, 1);
        let analysis = analyze(ramadhan, 0);
        assert!(analysis.primary_status.is_wajib());
        assert!(analysis.types.contains(&FastingType::Ramadhan));
    }

    #[test]
    fn test_arafah_sunnah() {
        // 9 Dhul Hijjah 1445
        let arafah = find_hijri_date(1445, 12, 9);
        let analysis = analyze(arafah, 0);
        // Arafah is Sunnah Muakkadah
        assert!(analysis.primary_status == FastingStatus::SunnahMuakkadah);
        assert!(analysis.types.contains(&FastingType::Arafah));
    }

    #[test]
    fn test_friday_makruh_vs_sunnah() {
        // We need 9 Dhul Hijjah to be a Friday.
        // Let's search for a year where 9 Dhul Hijjah is Friday.
        // 1446 Arafah -> approx June 2025.
        // I'll search for Arafah on Friday.
        let mut d = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let mut found = false;
        
        for _ in 0..5000 {
            let h = to_hijri(d, 0);
            if h.month() == 12 && h.day() == 9 && d.weekday() == chrono::Weekday::Fri {
                let analysis = analyze(d, 0);
                // Should be Sunnah, NOT Makruh.
                assert!(analysis.primary_status == FastingStatus::SunnahMuakkadah);
                assert!(!analysis.primary_status.is_haram());
                // The Type might effectively imply Friday, but Status must not be Makruh.
                // Our logic: if Status is Mubah -> Makruh. But here Status is SunnahMuakkadah.
                assert_ne!(analysis.primary_status, FastingStatus::Makruh);
                found = true;
                break;
            }
            d = d.succ_opt().unwrap();
        }
        assert!(found, "Could not find an Arafah on Friday for testing");
    }

    #[test]
    fn test_adjustment_shifts_date() {
        // 1st Ramadhan Unadjusted
        let d = find_hijri_date(1445, 9, 1);
        
        // If we adjust -1, it becomes 30 Sha'ban (Mubah usually, unless doubt day, but library treats as neutral)
        let analysis = analyze(d, -1);
        // Date input is `d`. Adjusted is `d-1`.
        // `to_hijri(d, -1)` -> `d-1`.
        // If `d` matches 1 Ramadhan, then `d-1` matches 30 Sha'ban (usually).
        
        assert_ne!(analysis.primary_status, FastingStatus::Wajib); // Should not be Wajib/Ramadhan
        
        // If we adjust +1 (maybe not meaningful here, but checks logic)
    }

    #[test]
    fn test_daud_schedule() {
         let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
         let end = start + chrono::Duration::days(10);
         let schedule = generate_daud_schedule(start, end, 0);
         // Just check it's not empty and skips days.
         assert!(schedule.len() > 0);
         // Ensure no consecutive days (unless skipped due to Haram?)
         // Daud: Day 1, Day 3, Day 5...
         for w in schedule.windows(2) {
             let diff = w[1] - w[0];
             assert!(diff.num_days() >= 2, "Daud schedule should skip at least one day");
         }
    }
}
