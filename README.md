# Shaum

A Rust library for determining the legal status (Hukum) of fasting for any given date according to Islamic Jurisprudence (Fiqh al-Ibadaat).

## Overview

Shaum provides a robust engine to convert Gregorian dates to Hijri and analyze them against standard Fiqh rules to determine whether fasting is Obligatory (Wajib), Recommended (Sunnah), Permissible (Mubah), Disliked (Makruh), or Prohibited (Haram).

## Features

- **Hijri Conversion**: Standard Umm al-Qura algorithm with support for manual moon-sighting adjustment.
- **Fiqh Analysis**: Prioritized rule engine resolving conflicts (e.g., Arafah on Friday).
- **Status Classification**:
    - **Wajib**: Ramadhan.
    - **Haram**: Eid al-Fitr, Eid al-Adha, Days of Tashriq.
    - **Sunnah**: Arafah, Ashura, Tasu'a, Ayyamul Bidh, Mondays, Thursdays, Shawwal.
    - **Makruh**: Singling out Friday or Saturday.
- **Utilities**: Schedule generation for Daud fasting (alternate days).

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
shaum = "0.1.0"
```

### Example

```rust
use shaum::prelude::*;
use chrono::NaiveDate;

fn main() {
    let date = NaiveDate::from_ymd_opt(2025, 6, 5).unwrap();
    // Optional adjustment for moon sighting (+/- days)
    let adjustment = 0; 
    
    let analysis = shaum::analyze(date, adjustment);

    if analysis.primary_status.is_haram() {
        println!("Fasting is prohibited: {:?}", analysis.types);
    } else {
        println!("Status: {:?}", analysis.primary_status);
        // Output: Status: Sunnah (e.g., Monday)
    }
}
```

## License

MIT License. Copyright (c) 2026 Mohammad Farid Hendianto (IRedDragonICY).
