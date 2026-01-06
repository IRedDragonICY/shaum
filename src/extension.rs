//! Extension trait for `NaiveDate`.

use chrono::NaiveDate;
use crate::calendar::ShaumError;
use crate::rules::{check, RuleContext};
use crate::types::{FastingAnalysis, FastingStatus};

/// Extends `NaiveDate` with fasting analysis methods.
pub trait ShaumDateExt {
    /// Returns fasting status (default context). Panics on out-of-range.
    fn fasting_status(&self) -> FastingStatus;
    
    /// Returns full analysis (default context).
    fn fasting_analysis(&self) -> Result<FastingAnalysis, ShaumError>;
    
    /// Returns full analysis with custom context.
    fn analyze_with(&self, ctx: &RuleContext) -> Result<FastingAnalysis, ShaumError>;
    
    /// Returns true if Wajib. Panics on out-of-range.
    fn is_wajib(&self) -> bool;
    
    /// Returns true if Haram. Panics on out-of-range.
    fn is_haram(&self) -> bool;
    
    /// Returns true if Sunnah. Panics on out-of-range.
    fn is_sunnah(&self) -> bool;
    
    /// Returns true if Makruh. Panics on out-of-range.
    fn is_makruh(&self) -> bool;
    
    /// Returns true if Mubah. Panics on out-of-range.
    fn is_mubah(&self) -> bool;
}

impl ShaumDateExt for NaiveDate {
    fn fasting_status(&self) -> FastingStatus {
        self.fasting_analysis()
            .map(|a| a.primary_status)
            .unwrap_or_else(|e| panic!("Failed to analyze {}: {}", self, e))
    }

    fn fasting_analysis(&self) -> Result<FastingAnalysis, ShaumError> {
        check(*self, &RuleContext::default())
    }

    fn analyze_with(&self, ctx: &RuleContext) -> Result<FastingAnalysis, ShaumError> {
        check(*self, ctx)
    }

    fn is_wajib(&self) -> bool { self.fasting_status().is_wajib() }
    fn is_haram(&self) -> bool { self.fasting_status().is_haram() }
    fn is_sunnah(&self) -> bool { self.fasting_status().is_sunnah() }
    fn is_makruh(&self) -> bool { self.fasting_status().is_makruh() }
    fn is_mubah(&self) -> bool { self.fasting_status().is_mubah() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_trait() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
        let _status = date.fasting_status();
        let _analysis = date.fasting_analysis().unwrap();
    }

    #[test]
    fn test_analyze_with_custom_context() {
        use crate::Madhab;
        let date = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
        let ctx = RuleContext::new().madhab(Madhab::Hanafi);
        let analysis = date.analyze_with(&ctx).unwrap();
        assert!(analysis.primary_status >= FastingStatus::Mubah);
    }
}
