//! WASM bindings for Shaum - Islamic Fasting Rules Engine
//!
//! Provides WebAssembly bindings for analyzing fasting status based on Islamic jurisprudence.

use wasm_bindgen::prelude::*;
use shaum_core::{analyze_date, FastingAnalysis};
use serde::Serialize;

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Analyzes a date string (YYYY-MM-DD) and returns fasting status as JSON.
///
/// # Example (JavaScript)
/// ```js
/// const result = analyze("2026-03-01");
/// console.log(result.primary_status); // "Wajib" during Ramadan
/// ```
#[wasm_bindgen]
pub fn analyze(date_str: &str) -> Result<JsValue, JsValue> {
    let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| JsValue::from_str(&format!("Invalid date format: {}", e)))?;
    
    let analysis = analyze_date(date)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    let result = WasmFastingAnalysis::from(analysis);
    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Class-based API for Shaum analysis.
///
/// # Example (JavaScript)
/// ```js
/// const shaum = new Shaum("2026-03-01");
/// const analysis = shaum.analyze();
/// console.log(analysis.status);
/// console.log(shaum.explain());
/// ```
#[wasm_bindgen]
pub struct Shaum {
    date: chrono::NaiveDate,
}

#[wasm_bindgen]
impl Shaum {
    /// Creates a new Shaum instance for the given date.
    #[wasm_bindgen(constructor)]
    pub fn new(date_str: &str) -> Result<Shaum, JsValue> {
        console_error_panic_hook::set_once();
        let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|e| JsValue::from_str(&format!("Invalid date format: {}", e)))?;
        Ok(Shaum { date })
    }
    
    /// Returns the fasting analysis for this date.
    pub fn analyze(&self) -> Result<JsValue, JsValue> {
        let analysis = shaum_core::analyze_date(self.date)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let result = WasmFastingAnalysis::from(analysis);
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    
    /// Returns a human-readable explanation of the fasting status.
    pub fn explain(&self) -> Result<String, JsValue> {
        let analysis = shaum_core::analyze_date(self.date)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(analysis.explain())
    }
    
    /// Returns the Hijri date as a string (day-month-year).
    pub fn hijri_date(&self) -> Result<String, JsValue> {
        let analysis = shaum_core::analyze_date(self.date)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(format!("{}-{}-{}", analysis.hijri_day, analysis.hijri_month, analysis.hijri_year))
    }
}

/// WASM-friendly representation of FastingAnalysis for TypeScript generation.
#[derive(Serialize, tsify::Tsify)]
#[tsify(into_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct WasmFastingAnalysis {
    pub primary_status: String,
    pub hijri_year: usize,
    pub hijri_month: usize,
    pub hijri_day: usize,
    pub reasons: Vec<String>,
    pub explanation: String,
}

impl From<FastingAnalysis> for WasmFastingAnalysis {
    fn from(analysis: FastingAnalysis) -> Self {
        Self {
            primary_status: format!("{:?}", analysis.primary_status),
            hijri_year: analysis.hijri_year,
            hijri_month: analysis.hijri_month,
            hijri_day: analysis.hijri_day,
            reasons: analysis.reasons().map(|r| r.to_string()).collect(),
            explanation: analysis.explain(),
        }
    }
}
