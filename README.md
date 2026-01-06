# Shaum

A production-grade Rust library for Islamic fasting (Shaum) jurisprudence with high-precision astronomical calculations for Hilal visibility.

## Features

### Fiqh Engine
- **Hijri Conversion**: Umm al-Qura algorithm with moon-sighting adjustment support.
- **Rule Priority**: Haram > Wajib > Sunnah > Makruh > Mubah.
- **Status Classification**:
  - **Wajib**: Ramadan
  - **Haram**: Eid al-Fitr, Eid al-Adha, Days of Tashriq
  - **Sunnah**: Arafah, Ashura, Tasu'a, Ayyamul Bidh, Mondays, Thursdays, Shawwal
  - **Makruh**: Singling out Friday or Saturday

### Astronomy Engine (v0.5.0)
Native high-precision ephemeris calculations for Hilal visibility determination:
- **Sun Position**: VSOP87 theory (~1 arcsec precision)
- **Moon Position**: ELP2000-82 theory (~1 arcsec precision)
- **Coordinate Systems**: Ecliptic, Equatorial, Horizontal
- **Corrections**: Topocentric parallax, atmospheric refraction (Bennett)
- **Visibility**: MABIMS criteria (altitude ≥ 3°, elongation ≥ 6.4°)

**Validated against 87 years of historical Indonesian Ramadan dates (1938-2024).**

## Usage

```toml
[dependencies]
shaum = "0.5"
```

### Fasting Status
```rust
use shaum::prelude::*;
use chrono::NaiveDate;

let date = NaiveDate::from_ymd_opt(2025, 6, 5).unwrap();
let analysis = shaum::analyze_date(date);

println!("Status: {:?}", analysis.primary_status);
```

### Hilal Visibility
```rust
use shaum::astronomy::visibility::calculate_visibility;
use shaum::types::GeoCoordinate;
use chrono::{Utc, TimeZone};

let jakarta = GeoCoordinate { lat: -6.2088, lng: 106.8456 };
let sunset = Utc.with_ymd_and_hms(2026, 2, 17, 11, 0, 0).unwrap();

let report = calculate_visibility(sunset, jakarta);

println!("Altitude: {:.2}°", report.moon_altitude);
println!("Elongation: {:.2}°", report.elongation);
println!("MABIMS: {}", report.meets_mabims);
```

## Validation

| Component | Method | Precision |
|-----------|--------|-----------|
| Sun (VSOP87) | Meeus Example 25.a | ~0.27 arcsec |
| Moon (ELP2000) | Meeus Example 47.a | ~0.02 arcsec |
| Historical | Indonesian Ramadan 1938-2024 | 87/87 passed |

## References

- Jean Meeus, *Astronomical Algorithms* (2nd ed.)
- MABIMS visibility criteria (Indonesia/Malaysia/Brunei/Singapore)
- VSOP87 (Bretagnon & Francou, 1988)
- ELP2000-82 (Chapront-Touzé & Chapront, 1983)

## License

MIT License. Copyright (c) 2026 Mohammad Farid Hendianto.
