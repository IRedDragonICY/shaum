use shaum::{to_hijri, check, RuleContext, Madhab, DaudStrategy, FastingStatus, generate_daud_schedule};
#[cfg(feature = "async")]
use shaum::rules::{RemoteMoonProvider, MoonProvider};
use shaum::extension::ShaumDateExt;
use chrono::{NaiveDate, Datelike};

#[test]
fn test_to_hijri_out_of_range() {
    // Negative year - should error
    let bad_date = NaiveDate::from_ymd_opt(1800, 1, 1).unwrap();
    // In strict mode, this returns Error
    let res = to_hijri(bad_date, 0);
    assert!(res.is_err());
}

#[test]
fn test_builder_defaults() {
    let ctx = RuleContext::new()
        .adjustment(2)
        .madhab(Madhab::Hanafi);
    
    assert_eq!(ctx.adjustment, 2);
    assert_eq!(ctx.madhab, Madhab::Hanafi);
    assert_eq!(ctx.daud_strategy, DaudStrategy::Skip); // Default
}

#[test]
fn test_arafah_friday_not_makruh() {
    // Find an Arafah that falls on Friday
    let mut d = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
    let mut found = false;
    
    for _ in 0..5000 {
        match to_hijri(d, 0) {
            Ok(h) => {
                // 9 Dhul Hijjah
                if h.month() == 12 && h.day() == 9 {
                    if d.weekday() == chrono::Weekday::Fri {
                        let ctx = RuleContext::new().madhab(Madhab::Shafi);
                        let analysis = check(d, &ctx).unwrap(); 
                        
                        // Should be Sunnah, NOT Makruh
                        assert!(!analysis.primary_status.is_haram());
                        assert_ne!(analysis.primary_status, FastingStatus::Makruh);
                        // Depending on impl, might be SunnahMuakkadah
                        println!("Date: {:?}, Status: {:?}", d, analysis.primary_status);
                        
                        found = true;
                        break;
                    }
                }
            }
            Err(_) => {
                // Skip invalid dates
            }
        }
        d = d.succ_opt().unwrap();
    }
    assert!(found, "Did not find Arafah on Friday to test");
}

#[test]
fn test_daud_skip_strategy() {
    // Find Eid al-Fitr (1 Shawwal)
    let mut eid_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    loop {
        if let Ok(h) = to_hijri(eid_date, 0) {
            if h.month() == 10 && h.day() == 1 { break; }
        }
        eid_date = eid_date.succ_opt().unwrap();
    }
    
    // Use Builder with Skip strategy
    let ctx = RuleContext::new().daud_strategy(DaudStrategy::Skip);
    
    // Start exactly on Eid.
    // Expectation: Eid is skipped (Haram). Turn was "Fast", so we toggle to "Eat".
    // Next day (Eid+1): My turn is "Eat". So result: Not in list.
    // Next day (Eid+2): My turn is "Fast". Result: In list.
    
    let iter = generate_daud_schedule(eid_date, eid_date + chrono::Duration::days(5), ctx);
    let days: Vec<NaiveDate> = iter.filter_map(|r| r.ok()).collect();
    
    assert!(!days.contains(&eid_date), "Should not fast on Eid");
    assert!(!days.contains(&(eid_date + chrono::Duration::days(1))), "Should eat on Eid+1 (Skip strategy)");
    assert!(days.contains(&(eid_date + chrono::Duration::days(2))), "Should fast on Eid+2");
}

#[test]
fn test_daud_postpone_strategy() {
    // Find Eid al-Fitr
    let mut eid_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    loop {
        if let Ok(h) = to_hijri(eid_date, 0) {
            if h.month() == 10 && h.day() == 1 { break; }
        }
        eid_date = eid_date.succ_opt().unwrap();
    }
    
    // Use Builder with Postpone strategy
    let ctx = RuleContext::new().daud_strategy(DaudStrategy::Postpone);
    
    // Start strictly on Eid.
    // Expectation: Eid is skipped (Haram). Turn was "Fast". Do NOT toggle.
    // Next day (Eid+1): My turn implies "Fast" (debt). Result: In list.
    // Next day (Eid+2): Toggle to "Eat". Result: Not in list.
    
    let iter = generate_daud_schedule(eid_date, eid_date + chrono::Duration::days(5), ctx);
    let days: Vec<NaiveDate> = iter.filter_map(|r| r.ok()).collect();
    
    assert!(!days.contains(&eid_date), "Should not fast on Eid");
    assert!(days.contains(&(eid_date + chrono::Duration::days(1))), "Should fast on Eid+1 (Postpone strategy)");
    assert!(!days.contains(&(eid_date + chrono::Duration::days(2))), "Should eat on Eid+2");
}

#[test]
fn test_try_status_invalid() {
    let date = NaiveDate::from_ymd_opt(3000, 1, 1).unwrap();
    assert!(date.try_status().is_err());
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_remote_moon_provider() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/adjustment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "adjustment": 1 })))
        .mount(&mock_server)
        .await;

    let provider = RemoteMoonProvider::new(format!("{}/adjustment", mock_server.uri()));
    // Date doesn't matter for this mock
    let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
    
    let adj = provider.get_adjustment(date, None).await;
    assert!(adj.is_ok());
    assert_eq!(adj.unwrap(), 1);
}
