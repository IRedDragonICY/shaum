use chrono::{Datelike, NaiveDate, Weekday};
use crate::calendar::{to_hijri, ShaumError};
use crate::types::{FastingAnalysis, FastingStatus, FastingType};

pub trait MoonProvider {
    fn get_adjustment(&self, date: NaiveDate) -> i64;
}

pub struct RuleContext {
    pub adjustment: i64,
    // Placeholders for future Madhab config
}

pub fn check(g_date: NaiveDate, context: &RuleContext) -> Result<FastingAnalysis, ShaumError> {
    let h_date = to_hijri(g_date, context.adjustment)?;
    let h_month = h_date.month();
    let h_day = h_date.day();
    let weekday = g_date.weekday();

    let mut types = Vec::new();
    let mut status = FastingStatus::Mubah;

    // --- 1. Haram Checks (Absolute Priority) ---
    // Eid al-Fitr
    if h_month == 10 && h_day == 1 {
        types.push(FastingType::EidAlFitr);
        return Ok(FastingAnalysis::new(g_date, FastingStatus::Haram, types, (h_date.year() as usize, h_month, h_day)));
    }

    // Eid al-Adha
    if h_month == 12 && h_day == 10 {
        types.push(FastingType::EidAlAdha);
        return Ok(FastingAnalysis::new(g_date, FastingStatus::Haram, types, (h_date.year() as usize, h_month, h_day)));
    }

    // Tashriq Days
    if h_month == 12 && (11..=13).contains(&h_day) {
        types.push(FastingType::Tashriq);
        return Ok(FastingAnalysis::new(g_date, FastingStatus::Haram, types, (h_date.year() as usize, h_month, h_day)));
    }

    // --- 2. Wajib Checks ---
    if h_month == 9 {
        types.push(FastingType::Ramadhan);
        status = FastingStatus::Wajib;
    }

    // --- 3. Sunnah Checks ---
    
    // Arafah (9 Dhu al-Hijjah)
    if h_month == 12 && h_day == 9 {
        types.push(FastingType::Arafah);
        if !status.is_wajib() { status = FastingStatus::SunnahMuakkadah; }
    }

    // Ashura (10 Muharram)
    if h_month == 1 && h_day == 10 {
        types.push(FastingType::Ashura);
        if !status.is_wajib() { status = FastingStatus::SunnahMuakkadah; }
    }

    // Tasu'a (9 Muharram)
    if h_month == 1 && h_day == 9 {
        types.push(FastingType::Tasua);
        if !status.is_wajib() && status != FastingStatus::SunnahMuakkadah { 
            status = FastingStatus::Sunnah; 
        }
    }

    // Ayyamul Bidh (13, 14, 15) - EXCLUDING 13 Dhu al-Hijjah (Handled by Haram above)
    if (13..=15).contains(&h_day) {
        types.push(FastingType::AyyamulBidh);
        if !status.is_wajib() && status < FastingStatus::Sunnah {
            status = FastingStatus::Sunnah;
        }
    }

    // Monday / Thursday
    match weekday {
        Weekday::Mon => {
            types.push(FastingType::Monday);
            if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
        },
        Weekday::Thu => {
            types.push(FastingType::Thursday);
            if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
        },
        _ => {}
    }

    // Shawwal (Month 10, excluding Day 1)
    if h_month == 10 && h_day > 1 {
        types.push(FastingType::Shawwal);
        if !status.is_wajib() && status < FastingStatus::Sunnah { status = FastingStatus::Sunnah; }
    }

    // --- 4. Makruh Checks (Friday/Saturday) ---
    if status == FastingStatus::Mubah {
        if weekday == Weekday::Fri {
            types.push(FastingType::FridayExclusive);
            status = FastingStatus::Makruh;
        } else if weekday == Weekday::Sat {
            types.push(FastingType::SaturdayExclusive);
            status = FastingStatus::Makruh;
        }
    }

    Ok(FastingAnalysis::new(g_date, status, types, (h_date.year() as usize, h_month, h_day)))
}
