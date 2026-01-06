//! Quick test for location + prayer times

use shaum::network::geo::get_location_info_from_ip;
use shaum::astronomy::prayer::calculate_prayer_times;
use shaum::types::PrayerParams;
use chrono::{NaiveDate, Duration};

#[tokio::main]
async fn main() {
    // Get full location info from IP
    let location = match get_location_info_from_ip().await {
        Ok(info) => {
            println!("=====================================");
            println!("  📍 LOKASI KAMU");
            println!("=====================================");
            println!("  {}", info.display_name());
            println!("  Lat: {:.4}°, Lng: {:.4}°", info.coords.lat, info.coords.lng);
            info
        }
        Err(e) => {
            eprintln!("❌ Gagal deteksi lokasi: {}", e);
            return;
        }
    };

    // Today's date (2026-01-06)
    let today = NaiveDate::from_ymd_opt(2026, 1, 6).unwrap();
    let params = PrayerParams::default(); // MABIMS: -20°, 10 min buffer
    
    let times = calculate_prayer_times(today, location.coords, &params);
    
    // Convert to WIB (UTC+7)
    let wib_offset = Duration::hours(7);
    let imsak_wib = times.imsak + wib_offset;
    let fajr_wib = times.fajr + wib_offset;
    let maghrib_wib = times.maghrib + wib_offset;
    
    println!();
    println!("=====================================");
    println!("  🕌 WAKTU SHALAT - 6 Januari 2026");
    println!("=====================================");
    println!("  Imsak:   {} WIB", imsak_wib.format("%H:%M:%S"));
    println!("  Subuh:   {} WIB", fajr_wib.format("%H:%M:%S"));
    println!("  Maghrib: {} WIB  ← BUKA PUASA", maghrib_wib.format("%H:%M:%S"));
    println!("=====================================");
}
