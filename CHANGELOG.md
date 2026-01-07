# Changelog

All notable changes to this project will be documented in this file.

## [0.8.0] - 2026-01-07

### Added
- **Cargo Workspace**: Refactored into multi-crate workspace architecture:
  - `shaum-types` - Zero-dependency types (FastingStatus, GeoCoordinate, etc.)
  - `shaum-calendar` - Hijri calendar conversion
  - `shaum-astronomy` - VSOP87/ELP2000 astronomical calculations
  - `shaum-rules` - Fasting rules engine
  - `shaum-network` - Optional async network features
  - `shaum-core` - Facade re-exporting all crates
- **WASM Bindings**: `bindings/shaum_wasm` with `wasm-pack` support
  - `analyze(date)` function for JavaScript
  - `Shaum` class-based API
  - TypeScript definitions auto-generated via `tsify`
- **Python Bindings**: `bindings/shaum_py` with `pyo3` + `maturin`
- **XTask Automation**: `xtask` crate for build automation
  - `cargo xtask dist-web` - Build WASM + JSR/NPM packages
  - `cargo xtask dist-python` - Build Python wheel
  - `cargo xtask publish-all --dry-run` - Validate all registries
  - `cargo xtask sync-versions` - Sync versions across manifests
- **JSR/NPM Support**: Ready for publishing to JSR.io and NPM

### Changed
- **Dependencies Updated to Latest**:
  - `thiserror` 1.0 → 2.0 (major)
  - `pyo3` 0.23 → 0.27 (major)
  - `getrandom` 0.2 → 0.3 (major)
  - `criterion` 0.5 → 0.8 (major)
  - `wasm-bindgen` 0.2.100 → 0.2.106
  - `smallvec` 1.11 → 1.15
  - `proptest` 1.4 → 1.9
  - `reqwest` 0.11 → 0.12
  - `maxminddb` 0.24 → 0.27
- **Migrated**: `tsify-next` → `tsify` (deprecated crate)
- **Edition**: Upgraded to Rust 2024 edition

### Fixed
- **WASM Build**: Disabled `wasm-opt` for Rust 1.82+ compatibility (bulk memory operations)

## [0.7.1] - 2026-01-07

### Removed
- **Privacy**: Completely removed deprecated `ip-api.com` integration. Users must now use `LocalGeoProvider` (offline MaxMind) or `reverse_geocode` (Nominatim) for location services.

## [0.7.0] - 2026-01-07

### Added
- **High Precision Mode**: Integrated Altitude/Elevation into prayer time calculations (`GeoCoordinate::altitude`).
- **Reverse Geocoding**: Added `reverse_geocode()` using OpenStreetMap Nominatim for detailed address info.
- **Global Validation**: Verified accuracy for 8 major global cities and 10 ASEAN capitals.
- **Real-Time Validation**: Added `examples/check_accuracy_today.rs` to verify accuracy against live API data.
- **Configuration**: Added `ihtiyat_minutes` (safety margin) and `rounding_granularity_seconds` to `PrayerParams`.
- **Regional Presets**: Added `ISNA` and `UmmAlQura` presets to `PrayerParams`.

### Fixed
- **Altitude Correction**: Fixed logic to correctly account for dip of horizon at high altitudes (e.g. Bandung, Mecca).
- **Timezone Handling**: Improved robust timezone offset handling in validation tests.

## [0.6.0] - 2026-01-06

### Added
- **Prayer Times Engine**: New `astronomy::prayer` module with `calculate_prayer_times()` for Fajr/Imsak/Maghrib calculation
- **IP Geolocation**: New `network::geo` module (async feature) with `get_location_from_ip()` and `get_location_info_from_ip()`
- `PrayerParams` struct with presets (MABIMS, Egyptian, MWL)
- `VisibilityCriteria` struct for configurable moon visibility thresholds
- `LocationInfo` struct with city/region/country names from IP lookup

### Changed
- **BREAKING**: `analyze_date()` now returns `Result<FastingAnalysis, ShaumError>` instead of panicking
- **BREAKING**: `DaudIterator::Item` changed from `NaiveDate` to `Result<NaiveDate, ShaumError>`
- **BREAKING**: `calculate_visibility()` now requires `&VisibilityCriteria` parameter
- `DefaultSunsetProvider` upgraded to use VSOP87 astronomy engine (more accurate)
- Removed hardcoded `MABIMS_MIN_ALTITUDE` and `MABIMS_MIN_ELONGATION` constants

### Deprecated
- `analyze_date_unchecked()` - use `analyze_date()` which returns Result

## [0.5.0] - 2026-01-06

### Added
- Initial production-grade refactor
- `SunsetProvider` trait abstraction
- `MoonProvider` trait for custom moon data sources
- `RuleContext` fluent API with builder pattern
- Unified error handling with `ShaumError`
- Astronomy engine (VSOP87, ELP2000) for accurate calculations
