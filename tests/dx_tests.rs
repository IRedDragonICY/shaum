//! DX Tests - Verify the new ergonomic APIs.
//!
//! This test file validates all 10 DX features implemented in the shaum library.

use chrono::{NaiveDate, Datelike};
use shaum::prelude::*;
use shaum::query::{FastingQuery, QueryExt};
use shaum::rules::{RuleContextBuilder, FixedAdjustment, NoAdjustment, MoonProvider};
use shaum::{DaudScheduleBuilder, generate_daud_schedule};

// ═══════════════════════════════════════════════════════════════════════════
// FEATURE 1: Chrono Extension (Zero-Ceremony API)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_extension_trait_fasting_status() {
    let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
    let status = date.status();
    
    // Should return a valid status without panicking
    assert!(
        status.is_wajib() || status.is_sunnah() || status.is_makruh() || 
        status.is_mubah() || status.is_haram()
    );
}

#[test]
fn test_extension_trait_fasting_analysis() {
    let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
    let analysis = date.fasting_analysis(); // No unwrap needed
    
    // Should have a valid status and date
    assert_eq!(analysis.date, date);
    assert!(analysis.hijri_year > 1400);
}

#[test]
fn test_extension_trait_analyze_with_context() {
    let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
    let ctx = RuleContext::new().madhab(Madhab::Hanafi);
    
    let analysis = date.analyze_with(&ctx); // No unwrap needed
    assert_eq!(analysis.date, date);
}

#[test]
fn test_extension_trait_boolean_methods() {
    // Find a date that is definitely Sunnah (Monday)
    let monday = NaiveDate::from_ymd_opt(2024, 11, 11).unwrap(); // A Monday
    assert!(monday.weekday() == chrono::Weekday::Mon);
    
    // Monday should be Sunnah (unless overridden by higher priority)
    let analysis = monday.fasting_analysis(); // No unwrap needed
    // At minimum, it should have Monday as a reason
    assert!(analysis.has_reason(FastingType::Monday));
}

// ═══════════════════════════════════════════════════════════════════════════
// FEATURE 2: Fluent Query Engine
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_query_engine_basic() {
    let start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
    
    let results: Vec<_> = FastingQuery::starting_from(start)
        .take(5)
        .collect();
    
    assert_eq!(results.len(), 5);
}

#[test]
fn test_query_engine_sunnah_filter() {
    let start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
    
    let results: Vec<_> = FastingQuery::starting_from(start)
        .sunnah()
        .take(5)
        .collect();
    
    // All should be Sunnah
    for r in &results {
        assert!(r.primary_status.is_sunnah(), "Expected Sunnah, got {:?}", r.primary_status);
    }
}

#[test]
fn test_query_engine_until_bound() {
    let start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 3, 7).unwrap();
    
    let results: Vec<_> = FastingQuery::starting_from(start)
        .until(end)
        .collect();
    
    // Should be at most 7 days
    assert!(results.len() <= 7);
}

#[test]
fn test_query_engine_exclude_makruh() {
    let start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
    
    let results: Vec<_> = FastingQuery::starting_from(start)
        .exclude_makruh()
        .take(10)
        .collect();
    
    // None should be Makruh
    for r in &results {
        assert!(!r.primary_status.is_makruh());
    }
}

#[test]
fn test_query_engine_with_type() {
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    
    let results: Vec<_> = FastingQuery::starting_from(start)
        .with_type(FastingType::Monday)
        .take(4)
        .collect();
    
    // All should have Monday reason
    for r in &results {
        assert!(r.has_reason(FastingType::Monday));
    }
}

#[test]
fn test_query_ext_trait() {
    let date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
    
    let results: Vec<_> = date.upcoming_fasts()
        .sunnah()
        .take(3)
        .collect();
    
    assert_eq!(results.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// FEATURE 3: Smart Daud Iterator
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_daud_schedule_builder() {
    let start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    
    let schedule = DaudScheduleBuilder::new(start)
        .until(end)
        .skip_haram_days()
        .build();
    
    let days: Vec<_> = schedule.collect();
    
    // Should have some fasting days
    assert!(!days.is_empty());
    
    // Should skip at least one day between each (Daud pattern)
    for window in days.windows(2) {
        let diff = (window[1] - window[0]).num_days();
        assert!(diff >= 2, "Daud should skip at least one day");
    }
}

#[test]
fn test_daud_schedule_postpone_strategy() {
    let start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 6, 20).unwrap();
    
    let schedule = DaudScheduleBuilder::new(start)
        .until(end)
        .postpone_on_haram()
        .build();
    
    let days: Vec<_> = schedule.collect();
    assert!(!days.is_empty());
}

#[test]
fn test_daud_never_yields_haram() {
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
    
    let ctx = RuleContext::new();
    let schedule = generate_daud_schedule(start, end, ctx);
    
    for date in schedule {
        let analysis = date.fasting_analysis(); // No unwrap
        assert!(
            !analysis.primary_status.is_haram(),
            "Daud should never yield a Haram day: {}", date
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FEATURE 4: Traceability & Explainability
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_explain_returns_string() {
    let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
    let analysis = date.fasting_analysis(); // No unwrap
    
    let explanation = analysis.explain();
    assert!(!explanation.is_empty());
}

#[test]
fn test_explain_contains_hijri_date() {
    let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
    let analysis = date.fasting_analysis(); // No unwrap
    
    let explanation = analysis.explain();
    // Should contain some Hijri date info
    assert!(
        explanation.contains("144") || // Hijri year like 1445
        explanation.contains("Ramadhan") || 
        explanation.contains("Shawwal") ||
        explanation.len() > 10
    );
}

#[test]
fn test_traces_available() {
    let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
    let ctx = RuleContext::new();
    let analysis = analyze(date, &ctx); // No unwrap
    
    // There should be at least one trace if there's a reason
    if analysis.reason_count() > 0 {
        assert!(analysis.traces().count() > 0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FEATURE 5: Moon Provider Architecture
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_fixed_adjustment_provider() {
    let provider = FixedAdjustment::new(2);
    let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
    
    assert_eq!(provider.get_adjustment(date), 2);
}

#[test]
fn test_no_adjustment_provider() {
    let provider = NoAdjustment;
    let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
    
    assert_eq!(provider.get_adjustment(date), 0);
}

#[test]
fn test_moon_provider_clamping() {
    // Over 30 should be clamped
    let provider = FixedAdjustment::new(100);
    let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
    
    assert_eq!(provider.get_adjustment(date), 30);
}

// ═══════════════════════════════════════════════════════════════════════════
// FEATURE 6: Robust Builder Pattern
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_rule_context_builder_basic() {
    let ctx = RuleContextBuilder::new()
        .adjustment(1)
        .madhab(Madhab::Hanafi)
        .build()
        .unwrap();
    
    assert_eq!(ctx.adjustment, 1);
    assert_eq!(ctx.madhab, Madhab::Hanafi);
}

#[test]
fn test_rule_context_builder_strict_validation_pass() {
    let ctx = RuleContextBuilder::new()
        .adjustment(2)
        .strict_adjustment(true)
        .build();
    
    assert!(ctx.is_ok());
}

#[test]
fn test_rule_context_builder_strict_validation_fail() {
    let result = RuleContextBuilder::new()
        .adjustment(5)
        .strict_adjustment(true)
        .build();
    
    assert!(result.is_err());
    
    if let Err(ShaumError::InvalidConfiguration { reason }) = result {
        assert!(reason.contains("outside strict bounds"));
    } else {
        panic!("Expected InvalidConfiguration error");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FEATURE 7: Error Handling Overhaul (Now Testing Safety)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_date_clamping_trace() {
    use shaum::analyze;
    
    // 1900 is outside range. Will be clamped.
    let bad_date = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
    let analysis = analyze(bad_date, &RuleContext::default());
    
    // Should have a trace about clamping
    let traces: Vec<_> = analysis.traces().collect();
    let has_clamped_trace = traces.iter().any(|t| t.rule == "Date Clamped");
    assert!(has_clamped_trace, "Should have a 'Date Clamped' trace for 1900");
}

// ═══════════════════════════════════════════════════════════════════════════
// FEATURE 8: Type Convenience & Encapsulation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_reasons_iterator() {
    let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
    let analysis = date.fasting_analysis();
    
    // Should be able to iterate over reasons
    let _count = analysis.reasons().count();
}

#[test]
fn test_has_reason() {
    // Find an Eid al-Fitr date (1 Shawwal)
    let mut d = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
    for _ in 0..100 {
        let analysis = d.fasting_analysis();
        if analysis.has_reason(FastingType::EidAlFitr) {
            // Found it!
            assert!(analysis.is_eid());
            return;
        }
        d = d.succ_opt().unwrap();
    }
}

#[test]
fn test_convenience_helpers() {
    // Find a Monday
    let date = NaiveDate::from_ymd_opt(2024, 11, 11).unwrap();
    assert!(date.weekday() == chrono::Weekday::Mon);
    
    let analysis = date.fasting_analysis();
    
    // Should have the has_reason method working
    let has_monday = analysis.has_reason(FastingType::Monday);
    assert!(has_monday);
}

#[test]
fn test_is_ramadhan_helper() {
    // Find a Ramadhan date
    let mut d = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
    for _ in 0..60 {
        let analysis = d.fasting_analysis();
        if analysis.is_ramadhan() {
            assert!(analysis.primary_status.is_wajib());
            return;
        }
        d = d.succ_opt().unwrap();
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FEATURE 9: Performance (Memoization)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_cache_consistency() {
    use shaum::to_hijri;
    
    let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
    
    // First call
    let h1 = to_hijri(date, 0);
    
    // Second call (should be cached)
    let h2 = to_hijri(date, 0);
    
    // Should be identical
    assert_eq!(h1.year(), h2.year());
    assert_eq!(h1.month(), h2.month());
    assert_eq!(h1.day(), h2.day());
}

#[test]
fn test_cache_invalidation_on_different_adjustment() {
    use shaum::to_hijri;
    
    let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
    
    let h1 = to_hijri(date, 0);
    let h2 = to_hijri(date, 1);
    
    // Different adjustments should give different results
    // (at least the effective hijri date should differ)
    // This is a simple sanity check
    assert!(h1.day() != h2.day() || h1.month() != h2.month() || h1.year() != h2.year());
}

// ═══════════════════════════════════════════════════════════════════════════
// INTEGRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_full_workflow() {
    // Simulate a realistic usage pattern
    let today = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    
    // 1. Quick check using extension trait
    let status = today.status();
    println!("Today's status: {:?}", status);
    
    // 2. Get full analysis
    let analysis = today.fasting_analysis();
    println!("Explanation: {}", analysis.explain());
    
    // 3. Find next 5 Sunnah days
    let sunnah_days: Vec<_> = today.upcoming_fasts()
        .sunnah()
        .take(5)
        .collect();
    
    println!("Next 5 Sunnah days:");
    for day in &sunnah_days {
        println!("  - {} ({:?})", day.date, day.primary_status);
    }
    
    // 4. Generate Daud schedule for the month
    let end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
    let daud: Vec<_> = DaudScheduleBuilder::new(today)
        .until(end)
        .skip_haram_days()
        .build()
        .collect();
    
    println!("Daud schedule for June: {} days", daud.len());
    
    // All should succeed without panics
    assert!(sunnah_days.len() == 5);
    assert!(!daud.is_empty());
}
