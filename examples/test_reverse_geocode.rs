//! Test reverse geocoding with Nominatim

use shaum::network::geo::{reverse_geocode, get_location_info_from_ip};
use shaum::astronomy::prayer::calculate_prayer_times;
use shaum::types::{PrayerParams, GeoCoordinate};
use chrono::{NaiveDate, Duration};

#[tokio::main]
async fn main() {
    println!("=====================================");
    println!("  🗺️  REVERSE GEOCODING TEST");
    println!("=====================================\n");
    
    // Step 1: Get coordinates from IP
    #[allow(deprecated)]
    let ip_location = match get_location_info_from_ip().await {
        Ok(info) => {
            println!("📍 IP Location: {}", info.display_name());
            println!("   Koordinat: {:.4}°, {:.4}°\n", info.coords.lat, info.coords.lng);
            info.coords
        }
        Err(e) => {
            eprintln!("❌ Gagal deteksi IP: {}", e);
            // Fallback to Yogyakarta coords
            GeoCoordinate::new_unchecked(-7.8195, 110.3610)
        }
    };
    
    // Step 2: Reverse geocode for detailed address
    println!("🔍 Mencari detail alamat dari Nominatim...\n");
    
    match reverse_geocode(ip_location).await {
        Ok(detail) => {
            println!("=====================================");
            println!("  📍 DETAIL LOKASI (OpenStreetMap)");
            println!("=====================================");
            println!("  Kelurahan : {:>25}", detail.kelurahan.as_deref().unwrap_or("-"));
            println!("  Kecamatan : {:>25}", detail.kecamatan.as_deref().unwrap_or("-"));
            println!("  Kabupaten : {:>25}", detail.kabupaten.as_deref().unwrap_or("-"));
            println!("  Provinsi  : {:>25}", detail.provinsi.as_deref().unwrap_or("-"));
            println!("  Negara    : {:>25}", detail.country.as_deref().unwrap_or("-"));
            println!("-------------------------------------");
            println!("  Full: {}", detail.display_name);
            println!("=====================================\n");
        }
        Err(e) => {
            eprintln!("❌ Gagal reverse geocode: {}", e);
        }
    }
    
    // Step 3: Calculate prayer times
    let today = NaiveDate::from_ymd_opt(2026, 1, 7).unwrap();
    let params = PrayerParams::default();
    
    if let Ok(times) = calculate_prayer_times(today, ip_location, &params) {
        let wib = Duration::hours(7);
        let fajr_wib = times.fajr + wib;
        let maghrib_wib = times.maghrib + wib;
        
        println!("=====================================");
        println!("  🕌 WAKTU SHALAT - 7 Januari 2026");
        println!("=====================================");
        println!("  Imsak:   {} WIB", (times.imsak + wib).format("%H:%M:%S"));
        println!("  Subuh:   {} WIB", fajr_wib.format("%H:%M:%S"));
        println!("  Maghrib: {} WIB  ← BUKA PUASA", maghrib_wib.format("%H:%M:%S"));
        println!("=====================================");
    }
}
