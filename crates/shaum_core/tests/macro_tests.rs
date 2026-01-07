use shaum_core::shaum_context;
use shaum_core::types::{Madhab, DaudStrategy};

#[test]
fn test_macro_full_usage() {
    let ctx = shaum_context! {
        madhab: Madhab::Hanafi,
        adjustment: 1,
        strategy: DaudStrategy::Postpone
    };
    
    assert_eq!(ctx.madhab, Madhab::Hanafi);
    assert_eq!(ctx.adjustment, 1);
    assert_eq!(ctx.daud_strategy, DaudStrategy::Postpone);
}

#[test]
fn test_macro_partial_usage() {
    let ctx = shaum_context! {
        adjustment: -1
    };
    
    // Others should be default
    assert_eq!(ctx.adjustment, -1);
    assert_eq!(ctx.madhab, Madhab::default());
}

#[test]
fn test_macro_reordered() {
    let ctx = shaum_context! {
        strategy: DaudStrategy::Skip,
        madhab: Madhab::Maliki
    };
    
    assert_eq!(ctx.daud_strategy, DaudStrategy::Skip);
    assert_eq!(ctx.madhab, Madhab::Maliki);
    assert_eq!(ctx.adjustment, 0);
}
