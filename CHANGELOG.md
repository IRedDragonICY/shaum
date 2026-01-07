# Changelog

All notable changes to this project will be documented in this file.

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
