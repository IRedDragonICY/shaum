//! Extension trait for `NaiveDate`.

use chrono::NaiveDate;
use crate::rules::{check, RuleContext};
use crate::types::{FastingAnalysis, FastingStatus};

/// Extends `NaiveDate` with fasting analysis methods.
pub trait ShaumDateExt {
    /// Returns fasting status (default context). Infallible.
    ///
    /// Replaces `fasting_status()`.
    fn status(&self) -> FastingStatus;
    
    /// **Deprecated**: Use `status()` instead.
    #[deprecated(since = "0.3.0", note = "Use `status()` instead")]
    fn fasting_status(&self) -> FastingStatus;
    
    /// Returns full analysis (default context). Infallible.
    fn fasting_analysis(&self) -> FastingAnalysis;
    
    /// Returns full analysis with custom context. Infallible.
    fn analyze_with(&self, ctx: &RuleContext) -> FastingAnalysis;
    
    /// Returns true if Wajib. Infallible.
    fn is_wajib(&self) -> bool;
    
    /// Returns true if Haram. Infallible.
    fn is_haram(&self) -> bool;
    
    /// Returns true if Sunnah. Infallible.
    fn is_sunnah(&self) -> bool;
    
    /// Returns true if Makruh. Infallible.
    fn is_makruh(&self) -> bool;
    
    /// Returns true if Mubah. Infallible.
    fn is_mubah(&self) -> bool;

    /// Finds the next Sunnah fasting day (up to 400 days ahead).
    fn next_sunnah(&self) -> Option<NaiveDate>;

    /// Finds the next Wajib fasting day (up to 400 days ahead).
    fn next_wajib(&self) -> Option<NaiveDate>;
}

impl ShaumDateExt for NaiveDate {
    fn status(&self) -> FastingStatus {
        check(*self, &RuleContext::default()).primary_status
    }

    fn fasting_status(&self) -> FastingStatus {
        self.status()
    }

    fn fasting_analysis(&self) -> FastingAnalysis {
        check(*self, &RuleContext::default())
    }

    fn analyze_with(&self, ctx: &RuleContext) -> FastingAnalysis {
        check(*self, ctx)
    }

    fn is_wajib(&self) -> bool { self.status().is_wajib() }
    fn is_haram(&self) -> bool { self.status().is_haram() }
    fn is_sunnah(&self) -> bool { self.status().is_sunnah() }
    fn is_makruh(&self) -> bool { self.status().is_makruh() }
    fn is_mubah(&self) -> bool { self.status().is_mubah() }

    fn next_sunnah(&self) -> Option<NaiveDate> {
        let mut d = *self;
        for _ in 0..400 {
            d = d.succ_opt()?;
            if d.is_sunnah() { return Some(d); }
        }
        None
    }

    fn next_wajib(&self) -> Option<NaiveDate> {
        let mut d = *self;
        for _ in 0..400 {
            d = d.succ_opt()?;
            if d.is_wajib() { return Some(d); }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_trait() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
        let _status = date.status();
        let _analysis = date.fasting_analysis();
    }

    #[test]
    fn test_analyze_with_custom_context() {
        use crate::Madhab;
        let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
        let ctx = RuleContext::new().madhab(Madhab::Hanafi);
        let analysis = date.analyze_with(&ctx);
        assert!(analysis.primary_status >= FastingStatus::Mubah);
    }

    #[test]
    fn test_jump_algorithms() {
        // Start from a random day
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        
        let sunnah = date.next_sunnah();
        assert!(sunnah.is_some());
        assert!(sunnah.unwrap().is_sunnah());
        assert!(sunnah.unwrap() > date);

        let wajib = date.next_wajib();
        assert!(wajib.is_some());
        assert!(wajib.unwrap().is_wajib());
        assert!(wajib.unwrap() > date);
    }
}
