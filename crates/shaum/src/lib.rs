//! # Shaum
//!
//! A production-grade Rust library for Islamic fasting (Shaum) jurisprudence 
//! with high-precision astronomical calculations for Hilal visibility.
//!
//! This crate is a facade that re-exports functionality from the `shaum` ecosystem.
//!
//! ## Modules
//! 
//! - `types`: Core types (FastingStatus, GeoCoordinate, etc.)
//! - `calendar`: Hijri calendar conversion
//! - `astronomy`: Astronomical calculations (Sun/Moon position, visibility)
//! - `rules`: Jurisprudence rules engine (optional)
//! - `network`: Network capabilities (optional)
//!
//! ## Usage
//!
//! ```rust
//! use shaum::prelude::*;
//! use chrono::NaiveDate;
//! 
//! let date = NaiveDate::from_ymd_opt(2025, 6, 5).unwrap();
//! let analysis = shaum::analyze_date(date); // Result<FastingAnalysis, ShaumError>
//! ```

pub use shaum_core::*;
