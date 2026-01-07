# Shaum

A production-grade Rust library for Islamic fasting (Shaum) jurisprudence with high-precision astronomical calculations for Hilal visibility.

[![Crates.io](https://img.shields.io/crates/v/shaum)](https://crates.io/crates/shaum)
[![NPM](https://img.shields.io/npm/v/shaum-wasm)](https://www.npmjs.com/package/shaum-wasm)
[![JSR](https://jsr.io/badges/@islam/shaum)](https://jsr.io/@islam/shaum)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

### Fiqh Engine
- **Hijri Conversion**: Umm al-Qura algorithm with moon-sighting adjustment support.
- **Rule Priority**: Haram > Wajib > Sunnah > Makruh > Mubah.
- **Status Classification**:
  - **Wajib**: Ramadan
  - **Haram**: Eid al-Fitr, Eid al-Adha, Days of Tashriq
  - **Sunnah**: Arafah, Ashura, Tasu'a, Ayyamul Bidh, Mondays, Thursdays, Shawwal
  - **Makruh**: Singling out Friday or Saturday

### Astronomy Engine
Native high-precision ephemeris calculations for Hilal visibility determination:
- **Sun Position**: VSOP87 theory (~1 arcsec precision)
- **Moon Position**: ELP2000-82 theory (~1 arcsec precision)
- **Coordinate Systems**: Ecliptic, Equatorial, Horizontal
- **Corrections**: Topocentric parallax, atmospheric refraction (Bennett)
- **Visibility**: MABIMS criteria (altitude ≥ 3°, elongation ≥ 6.4°)

**Validated against 87 years of historical Indonesian Ramadan dates (1938-2024).**

## Installation

### Rust
```toml
[dependencies]
shaum = "0.8"
```

### JavaScript/TypeScript (NPM)
```bash
npm install shaum-wasm
```

### Deno (JSR)
```typescript
import { analyze } from "jsr:@islam/shaum";
```

## Usage

### Rust
```rust
use shaum::prelude::*;
use chrono::NaiveDate;

let date = NaiveDate::from_ymd_opt(2025, 6, 5).unwrap();
let analysis = shaum::analyze_date(date)?;

println!("Status: {:?}", analysis.primary_status);
```

### JavaScript
```javascript
import init, { analyze } from 'shaum-wasm';

await init();

const result = analyze("2025-06-05");
console.log(result.primaryStatus);
```

## Workspace Structure

```
shaum/
├── crates/
│   ├── shaum-types/       # Zero-dependency types
│   ├── shaum-calendar/    # Hijri conversion
│   ├── shaum-astronomy/   # VSOP87/ELP2000
│   ├── shaum-rules/       # Fasting jurisprudence
│   ├── shaum-network/     # Async geolocation
│   └── shaum-core/        # Facade crate
├── bindings/
│   ├── shaum_wasm/        # WebAssembly
│   └── shaum_py/          # Python (pyo3)
└── xtask/                 # Build automation
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
