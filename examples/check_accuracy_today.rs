use shaum::{
    astronomy::prayer::{calculate_prayer_times, PrayerTimes},
    types::{GeoCoordinate, PrayerParams},
};
use chrono::{Local, Datelike, Timelike, Duration};
use serde::Deserialize;
use std::error::Error;



#[derive(Deserialize, Debug)]
struct AladhanResponse {
    data: AladhanData,
}

#[derive(Deserialize, Debug)]
struct AladhanData {
    timings: Timings,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct Timings {
    Fajr: String,
    Dhuhr: String,
    Asr: String,
    Maghrib: String,
    Isha: String,
    Imsak: String,
}

struct CityConfig {
    name: &'static str,
    lat: f64,
    lng: f64,
    alt: f64,
    method_id: i32, // Aladhan API Method ID
    timezone_offset: i32, // Hours from UTC (approx for display)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("\n==========================================================================");
    println!("   REAL-TIME VALIDATION: SHAUM vs ALADHAN API (Multi-City Edition)");
    println!("     (Comparing calculations for TODAY - Shaum Raw vs API)");
    println!("==========================================================================\n");

    let now = Local::now();
    let date = now.date_naive();
    println!("📅 Date: {}\n", date.format("%A, %d %B %Y"));

    // 1. Define Cities
    let cities = vec![
        // --- 10 Major Indonesian Cities (Method 11: MABIMS, Alt adjusted) ---
        CityConfig { name: "Jakarta",     lat: -6.1754, lng: 106.8272, alt: 8.0,   method_id: 11, timezone_offset: 7 },
        CityConfig { name: "Surabaya",    lat: -7.2575, lng: 112.7521, alt: 5.0,   method_id: 11, timezone_offset: 7 },
        CityConfig { name: "Bandung",     lat: -6.9175, lng: 107.6191, alt: 768.0, method_id: 11, timezone_offset: 7 },
        CityConfig { name: "Medan",       lat: 3.5952,  lng: 98.6722,  alt: 26.0,  method_id: 11, timezone_offset: 7 },
        CityConfig { name: "Semarang",    lat: -6.9667, lng: 110.4167, alt: 4.0,   method_id: 11, timezone_offset: 7 },
        CityConfig { name: "Makassar",    lat: -5.1477, lng: 119.4327, alt: 25.0,  method_id: 11, timezone_offset: 8 },
        CityConfig { name: "Yogyakarta",  lat: -7.7955, lng: 110.3695, alt: 113.0, method_id: 11, timezone_offset: 7 },
        CityConfig { name: "Denpasar",    lat: -8.6705, lng: 115.2126, alt: 4.0,   method_id: 11, timezone_offset: 8 },
        CityConfig { name: "Ambon",       lat: -3.6954, lng: 128.1814, alt: 15.0,  method_id: 11, timezone_offset: 9 },
        CityConfig { name: "Jayapura",    lat: -2.5000, lng: 140.7167, alt: 287.0, method_id: 11, timezone_offset: 9 },

        // --- Global Cities (Various Methods) ---
        // Mecca: Method 4 (Umm Al-Qura)
        CityConfig { name: "Mecca",       lat: 21.3891, lng: 39.8579,  alt: 277.0, method_id: 4,  timezone_offset: 3 },
        // Tokyo: Method 3 (MWL)
        CityConfig { name: "Tokyo",       lat: 35.6895, lng: 139.6917, alt: 40.0,  method_id: 3,  timezone_offset: 9 },
        // London: Method 2 (ISNA) - generic good fit
        CityConfig { name: "London",      lat: 51.5074, lng: -0.1278,  alt: 11.0,  method_id: 2,  timezone_offset: 0 },
        // New York: Method 2 (ISNA)
        CityConfig { name: "New York",    lat: 40.7128, lng: -74.0060, alt: 10.0,  method_id: 2,  timezone_offset: -5 },
        // Cairo: Method 5 (Egyptian)
        CityConfig { name: "Cairo",       lat: 30.0444, lng: 31.2357,  alt: 23.0,  method_id: 5,  timezone_offset: 2 },
        // Sydney: Method 3 (MWL)
        CityConfig { name: "Sydney",      lat: -33.8688, lng: 151.2093, alt: 3.0,  method_id: 3,  timezone_offset: 11 },
    ];

    println!("{:<15} | {:<5} | {:<5} | {:<5} | {:<5} | {:<5} | {:<5}", 
             "CITY", "FAJR", "API", "DIFF", "MAGHRIB", "API", "DIFF");
    println!("{:-<75}", "");

    let client = reqwest::Client::new();

    for city in cities {
        match validate_city(&client, &city, date).await {
            Ok(_) => {},
            Err(e) => println!("{:<15} | ERROR: {}", city.name, e),
        }
        // Small delay to be nice to the API
        std::thread::sleep(std::time::Duration::from_millis(500)); 
    }

    println!("==========================================================================");
    Ok(())
}

async fn validate_city(client: &reqwest::Client, city: &CityConfig, date: chrono::NaiveDate) -> Result<(), Box<dyn Error>> {
    // 1. Calculate Raw Shaum Times (No Ihtiyat)
    // We use Raw to match API purely mathematically.
    // Note: We pick the calculation method matching the API choice.
    let params_raw = match city.method_id {
        4 => { // Umm Al-Qura
            let mut p = PrayerParams::umm_al_qura();
            p.ihtiyat_minutes = 0;
            p.rounding_granularity_seconds = 1;
            p
        },
        2 => { // ISNA
            let mut p = PrayerParams::isna();
            p.ihtiyat_minutes = 0;
            p.rounding_granularity_seconds = 1;
            p
        },
        5 => { // Egyptian
            let mut p = PrayerParams::egyptian();
            p.ihtiyat_minutes = 0;
            p.rounding_granularity_seconds = 1;
            p
        },
        11 | _ => { // MABIMS (11) or MWL (3/others)
            // For MABIMS "raw", we just want the angles -20.
            let mut p = if city.method_id == 11 { PrayerParams::mabims() } else { PrayerParams::mwl() };
            p.ihtiyat_minutes = 0;
            p.rounding_granularity_seconds = 1; // No rounding
            p
        }
    };

    let coords = GeoCoordinate::new_unchecked(city.lat, city.lng).with_altitude(city.alt);
    let times = calculate_prayer_times(date, coords, &params_raw)?;

    // 2. Fetch API
    let url = format!(
        "http://api.aladhan.com/v1/timings/{}?latitude={}&longitude={}&method={}", 
        date.format("%d-%m-%Y"), city.lat, city.lng, city.method_id
    );

    let resp = client.get(&url).send().await?.json::<AladhanResponse>().await?;
    let api_times = resp.data.timings;

    // 3. Display Row
    // We add arbitrary offset just for HH:MM visualization (doesn't affect diff)
    let offset = Duration::hours(city.timezone_offset as i64);
    
    let fajr_s = (times.fajr + offset).format("%H:%M").to_string();
    let magh_s = (times.maghrib + offset).format("%H:%M").to_string();
    
    // API returns e.g. "04:20 (WIB)", we split to get time
    let api_fajr_clean = clean_time(&api_times.Fajr);
    let api_magh_clean = clean_time(&api_times.Maghrib);
    
    let diff_f = calc_diff_minutes(&fajr_s, &api_fajr_clean);
    let diff_m = calc_diff_minutes(&magh_s, &api_magh_clean);
    
    let status_f = if diff_f.abs() <= 2 { "✅" } else { "❌" };
    let status_m = if diff_m.abs() <= 2 { "✅" } else { "❌" };
    
    println!("{:<15} | {:<5} | {:<5} | {:+2} {} | {:<5}   | {:<5} | {:+2} {}", 
             city.name, 
             fajr_s, api_fajr_clean, diff_f, status_f,
             magh_s, api_magh_clean, diff_m, status_m);
             
    Ok(())
}

fn clean_time(s: &str) -> String {
    // Take anything before a space or parenthesis
    let s = s.split_whitespace().next().unwrap_or(s);
    let s = s.split('(').next().unwrap_or(s);
    s.trim().to_string()
}

fn calc_diff_minutes(t1: &str, t2: &str) -> i64 {
    let parse_time = |s: &str| -> Result<i64, Box<dyn Error>> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 2 { return Err("Invalid format".into()); }
        let h: i64 = parts[0].parse()?;
        let m: i64 = parts[1].parse()?;
        Ok(h * 60 + m)
    };

    let m1 = parse_time(t1).unwrap_or(0);
    let m2 = parse_time(t2).unwrap_or_else(|_| {
        eprint!(" [Err: {}] ", t2); // Print error if API time is weird
        0
    });
    
    m1 - m2
}

