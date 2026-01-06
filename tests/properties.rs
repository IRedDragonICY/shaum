use proptest::prelude::*;
use chrono::NaiveDate;
use shaum::prelude::*;

proptest! {
    /// Invariant: `analyze` never panics for any date between 1900 and 2100.
    #[test]
    fn no_panic_analyze_invariant(days in 0i32..73000) {
        // Base date 1900-01-01
        let base = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        let date = base.checked_add_signed(chrono::Duration::days(days as i64)).unwrap();
        
        // Should not panic
        let _ = analyze_date(date);
    }
    
    /// Invariant: Status Hierarchy (Haram trumps all).
    #[test]
    fn haram_trumps_all(days in 0i32..73000) {
        let base = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        let date = base.checked_add_signed(chrono::Duration::days(days as i64)).unwrap();
        
        let analysis = analyze_date(date);
        
        if analysis.has_reason(FastingType::EidAlFitr) || 
           analysis.has_reason(FastingType::EidAlAdha) || 
           analysis.has_reason(FastingType::Tashriq) {
            assert!(analysis.primary_status.is_haram(), "Date {:?} has Haram reason but status is {:?}", date, analysis.primary_status);
        }
    }
    
    /// Invariant: Ramadhan is always Wajib (unless Travel/Sick - not implemented yet, so Wajib).
    #[test]
    fn ramadhan_is_wajib(days in 0i32..73000) {
        let base = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        let date = base.checked_add_signed(chrono::Duration::days(days as i64)).unwrap();
        
        let analysis = analyze_date(date);
        
        if analysis.has_reason(FastingType::Ramadhan) {
            // Ramadhan is Wajib. Exceptions (Eid?) No, Ramadhan ends before Eid.
            assert!(analysis.primary_status.is_wajib(), "Date {:?} is Ramadhan but not Wajib: {:?}", date, analysis.primary_status);
        }
    }
    
    /// Invariant: Daud never recommends fasting on Haram days.
    #[test]
    fn daud_skips_haram(days in 0i32..1000) {
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = start.checked_add_signed(chrono::Duration::days(days as i64)).unwrap();
        
        let ctx = RuleContext::new().daud_strategy(DaudStrategy::Skip);
        let daud_days = shaum::generate_daud_schedule(start, end, ctx);
        
        for date in daud_days {
            let analysis = shaum::analyze_date(date);
            assert!(!analysis.primary_status.is_haram(), "Daud recommended Haram day: {:?}", date);
        }
    }
}
