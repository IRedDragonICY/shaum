//! Python bindings for Shaum - Islamic Fasting Rules Engine
//!
//! Provides a Pythonic interface to the Shaum fasting rules engine.
//!
//! # Example
//! ```python
//! import shaum
//!
//! analysis = shaum.analyze("2026-03-01")
//! print(f"Status: {analysis.status}")
//! print(f"Hijri Date: {analysis.hijri_date}")
//! print(analysis.explain())
//! ```

use pyo3::prelude::*;
use shaum_core::{FastingAnalysis as CoreAnalysis, FastingStatus as CoreStatus};

/// Fasting status according to Islamic jurisprudence.
#[pyclass(name = "FastingStatus", eq)]
#[derive(Clone, PartialEq)]
pub enum FastingStatus {
    /// Permissible - no special ruling
    Mubah,
    /// Disliked - better to avoid
    Makruh,
    /// Recommended - rewarded but not obligatory
    Sunnah,
    /// Highly recommended - strongly encouraged
    SunnahMuakkadah,
    /// Obligatory - required (e.g., Ramadan)
    Wajib,
    /// Forbidden - sinful to fast (e.g., Eid)
    Haram,
}

impl From<CoreStatus> for FastingStatus {
    fn from(s: CoreStatus) -> Self {
        match s {
            CoreStatus::Mubah => FastingStatus::Mubah,
            CoreStatus::Makruh => FastingStatus::Makruh,
            CoreStatus::Sunnah => FastingStatus::Sunnah,
            CoreStatus::SunnahMuakkadah => FastingStatus::SunnahMuakkadah,
            CoreStatus::Wajib => FastingStatus::Wajib,
            CoreStatus::Haram => FastingStatus::Haram,
        }
    }
}

#[pymethods]
impl FastingStatus {
    fn __repr__(&self) -> String {
        match self {
            FastingStatus::Mubah => "FastingStatus.Mubah".to_string(),
            FastingStatus::Makruh => "FastingStatus.Makruh".to_string(),
            FastingStatus::Sunnah => "FastingStatus.Sunnah".to_string(),
            FastingStatus::SunnahMuakkadah => "FastingStatus.SunnahMuakkadah".to_string(),
            FastingStatus::Wajib => "FastingStatus.Wajib".to_string(),
            FastingStatus::Haram => "FastingStatus.Haram".to_string(),
        }
    }
    
    fn __str__(&self) -> String {
        match self {
            FastingStatus::Mubah => "Mubah (Permissible)".to_string(),
            FastingStatus::Makruh => "Makruh (Disliked)".to_string(),
            FastingStatus::Sunnah => "Sunnah (Recommended)".to_string(),
            FastingStatus::SunnahMuakkadah => "Sunnah Muakkadah (Highly Recommended)".to_string(),
            FastingStatus::Wajib => "Wajib (Obligatory)".to_string(),
            FastingStatus::Haram => "Haram (Forbidden)".to_string(),
        }
    }
}

/// Analysis result for a specific date's fasting status.
#[pyclass(name = "FastingAnalysis")]
pub struct FastingAnalysis {
    inner: CoreAnalysis,
}

#[pymethods]
impl FastingAnalysis {
    /// The primary fasting status for this date.
    #[getter]
    fn status(&self) -> FastingStatus {
        FastingStatus::from(self.inner.primary_status)
    }
    
    /// Hijri date in "day-month-year" format.
    #[getter]
    fn hijri_date(&self) -> String {
        format!("{}-{}-{}", self.inner.hijri_day, self.inner.hijri_month, self.inner.hijri_year)
    }
    
    /// Hijri year (e.g., 1447).
    #[getter]
    fn hijri_year(&self) -> usize {
        self.inner.hijri_year
    }
    
    /// Hijri month (1-12).
    #[getter]
    fn hijri_month(&self) -> usize {
        self.inner.hijri_month
    }
    
    /// Hijri day (1-30).
    #[getter]
    fn hijri_day(&self) -> usize {
        self.inner.hijri_day
    }
    
    /// List of fasting type reasons (e.g., ["Ramadhan", "Monday"]).
    #[getter]
    fn reasons(&self) -> Vec<String> {
        self.inner.reasons().map(|r| r.to_string()).collect()
    }
    
    /// Returns a human-readable explanation of the fasting ruling.
    fn explain(&self) -> String {
        self.inner.explain()
    }
    
    /// Returns True if this is a Ramadan day.
    fn is_ramadan(&self) -> bool {
        self.inner.is_ramadhan()
    }
    
    /// Returns True if this is an Eid day (Fitr or Adha).
    fn is_eid(&self) -> bool {
        self.inner.is_eid()
    }
    
    /// Returns True if this is a white day (Ayyamul Bidh).
    fn is_white_day(&self) -> bool {
        self.inner.is_white_day()
    }
    
    fn __repr__(&self) -> String {
        format!(
            "FastingAnalysis(hijri={}, status={:?})",
            self.hijri_date(),
            self.inner.primary_status
        )
    }
    
    fn __str__(&self) -> String {
        self.explain()
    }
}

/// Analyze a date and return its fasting status.
///
/// Args:
///     date_str: Date in YYYY-MM-DD format (Gregorian)
///
/// Returns:
///     FastingAnalysis with status, Hijri date, and explanation
///
/// Raises:
///     ValueError: If date format is invalid or date is out of range
///
/// Example:
///     >>> analysis = shaum.analyze("2026-03-01")
///     >>> print(analysis.status)
///     FastingStatus.Wajib
#[pyfunction]
fn analyze(date_str: &str) -> PyResult<FastingAnalysis> {
    let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(
            format!("Invalid date format '{}': {}. Expected YYYY-MM-DD", date_str, e)
        ))?;
    
    let analysis = shaum_core::analyze_date(date)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    
    Ok(FastingAnalysis { inner: analysis })
}

/// Shaum - Islamic Fasting Rules Engine
///
/// A Fiqh-compliant engine for determining Islamic fasting status.
/// Accurately identifies Wajib, Sunnah, Makruh, and Haram fasting days.
///
/// Example:
///     >>> import shaum
///     >>> result = shaum.analyze("2026-03-01")
///     >>> print(result.status)
///     FastingStatus.Wajib
#[pymodule]
fn shaum(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FastingStatus>()?;
    m.add_class::<FastingAnalysis>()?;
    m.add_function(wrap_pyfunction!(analyze, m)?)?;
    Ok(())
}
