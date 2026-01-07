//! Extension trait for `NaiveDate`.
 
use chrono::NaiveDate;
use crate::rules::{check, RuleContext};
use shaum_types::{FastingAnalysis, FastingStatus};
use shaum_types::ShaumError;

/// Extends `NaiveDate` with fasting analysis methods.
pub trait ShaumDateExt {
    /// Returns fasting status (default context). Panics on error.
    ///
    /// # Panics
    /// Panics if the date is out of supported Hijri range (1938-2076).
    #[deprecated(since = "0.4.0", note = "Use try_status() for safe error handling")]
    fn status(&self) -> FastingStatus;

    /// Returns fasting status (default context). Safe version.
    fn try_status(&self) -> Result<FastingStatus, ShaumError>;
    
    /// **Deprecated**: Use `status()` instead.
    #[deprecated(since = "0.4.0", note = "Use status() or try_status() instead")]
    fn fasting_status(&self) -> FastingStatus;
    
    /// Returns full analysis (default context). Panics on error.
    fn fasting_analysis(&self) -> FastingAnalysis;

    /// Returns full analysis (default context). Safe version.
    fn try_fasting_analysis(&self) -> Result<FastingAnalysis, ShaumError>;
    
    /// Returns full analysis with custom context. Panics on error.
    fn analyze_with(&self, ctx: &RuleContext) -> FastingAnalysis;
    
    /// Returns true if Wajib. Panics on invalid date.
    fn is_wajib(&self) -> bool;
    
    /// Returns true if Haram. Panics on invalid date.
    fn is_haram(&self) -> bool;
    
    /// Returns true if Sunnah. Panics on invalid date.
    fn is_sunnah(&self) -> bool;
    
    /// Returns true if Makruh. Panics on invalid date.
    fn is_makruh(&self) -> bool;
    
    /// Returns true if Mubah. Panics on invalid date.
    fn is_mubah(&self) -> bool;

    /// Finds the next Sunnah fasting day (up to 400 days ahead).
    fn next_sunnah(&self) -> Option<NaiveDate>;

    /// Finds the next Wajib fasting day (up to 400 days ahead).
    fn next_wajib(&self) -> Option<NaiveDate>;
}

impl ShaumDateExt for NaiveDate {
    fn status(&self) -> FastingStatus {
        check(*self, &RuleContext::default()).unwrap().primary_status
    }

    fn try_status(&self) -> Result<FastingStatus, ShaumError> {
        check(*self, &RuleContext::default()).map(|a| a.primary_status)
    }

    #[allow(deprecated)]
    fn fasting_status(&self) -> FastingStatus {
        self.try_status().expect("Fasting status failed")
    }

    fn fasting_analysis(&self) -> FastingAnalysis {
        check(*self, &RuleContext::default()).expect("Fasting analysis failed")
    }

    fn try_fasting_analysis(&self) -> Result<FastingAnalysis, ShaumError> {
        check(*self, &RuleContext::default())
    }

    fn analyze_with(&self, ctx: &RuleContext) -> FastingAnalysis {
        check(*self, ctx).expect("Fasting analysis failed")
    }

    fn is_wajib(&self) -> bool { self.try_status().map(|s| s.is_wajib()).unwrap_or(false) }
    fn is_haram(&self) -> bool { self.try_status().map(|s| s.is_haram()).unwrap_or(false) }
    fn is_sunnah(&self) -> bool { self.try_status().map(|s| s.is_sunnah()).unwrap_or(false) }
    fn is_makruh(&self) -> bool { self.try_status().map(|s| s.is_makruh()).unwrap_or(false) }
    fn is_mubah(&self) -> bool { self.try_status().map(|s| s.is_mubah()).unwrap_or(false) }

    fn next_sunnah(&self) -> Option<NaiveDate> {
        let mut d = *self;
        for _ in 0..400 {
            d = d.succ_opt()?;
            // We use status() which unwraps. If date goes out of range (2076), it panics.
            // But 400 days from now is unlikely to hit limit unless we are near 2076.
            // PROD: We could use try_status() and treat error as "stop searching".
            // But per spec "unwrap for is_wajib etc", we probably stick to unwrap here for consistency or handle it?
            // "next_sunnah" implies valid search.
            if let Ok(s) = d.try_status() {
                if s.is_sunnah() { return Some(d); }
            } else {
                return None; // Stop if we hit error (out of range)
            }
        }
        None
    }

    fn next_wajib(&self) -> Option<NaiveDate> {
        let mut d = *self;
        for _ in 0..400 {
            d = d.succ_opt()?;
            if let Ok(s) = d.try_status() {
                if s.is_wajib() { return Some(d); }
            } else {
                return None;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_extension_trait() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
        let _status = date.try_status().unwrap();
        let _analysis = date.fasting_analysis();
    }

    #[test]
    fn test_try_status_out_of_range() {
        let bad_date = NaiveDate::from_ymd_opt(3000, 1, 1).unwrap();
        assert!(bad_date.try_status().is_err());
    }

    #[test]
    fn test_analyze_with_custom_context() {
        use shaum_types::Madhab;
        let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
        let ctx = RuleContext::new().madhab(Madhab::Hanafi);
        let analysis = date.analyze_with(&ctx);
        assert!(analysis.primary_status >= FastingStatus::Mubah);
    }
}
