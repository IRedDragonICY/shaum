//! IP-based Geolocation Module.
//!
//! Provides automatic location detection via IP address lookup.

use crate::calendar::ShaumError;
use crate::types::GeoCoordinate;
use serde::Deserialize;

/// Response from ip-api.com
#[derive(Debug, Deserialize)]
struct IpApiResponse {
    status: String,
    lat: Option<f64>,
    lon: Option<f64>,
    city: Option<String>,
    #[serde(rename = "regionName")]
    region_name: Option<String>,
    country: Option<String>,
    message: Option<String>,
}

/// Location information with coordinates and place name.
#[derive(Debug, Clone)]
pub struct LocationInfo {
    /// Geographic coordinates.
    pub coords: GeoCoordinate,
    /// City name (if available).
    pub city: Option<String>,
    /// Region/Province name (if available).
    pub region: Option<String>,
    /// Country name (if available).
    pub country: Option<String>,
}

impl LocationInfo {
    /// Returns formatted location string (e.g., "Yogyakarta, Daerah Istimewa Yogyakarta, Indonesia").
    pub fn display_name(&self) -> String {
        let parts: Vec<&str> = [
            self.city.as_deref(),
            self.region.as_deref(),
            self.country.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect();
        
        if parts.is_empty() {
            format!("{:.4}°, {:.4}°", self.coords.lat, self.coords.lng)
        } else {
            parts.join(", ")
        }
    }
}

/// Fetches geographic coordinates based on the caller's IP address.
///
/// Uses the free ip-api.com service to determine location.
///
/// # Returns
/// `Result<GeoCoordinate, ShaumError>` with the detected coordinates.
pub async fn get_location_from_ip() -> Result<GeoCoordinate, ShaumError> {
    let info = get_location_info_from_ip().await?;
    Ok(info.coords)
}

/// Fetches full location info (coordinates + place name) based on IP address.
///
/// # Example
/// ```rust,no_run
/// use shaum::network::geo::get_location_info_from_ip;
///
/// #[tokio::main]
/// async fn main() {
///     match get_location_info_from_ip().await {
///         Ok(info) => {
///             println!("📍 {}", info.display_name());
///             println!("   Lat: {}, Lng: {}", info.coords.lat, info.coords.lng);
///         }
///         Err(e) => eprintln!("Failed: {}", e),
///     }
/// }
/// ```
pub async fn get_location_info_from_ip() -> Result<LocationInfo, ShaumError> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("http://ip-api.com/json/")
        .send()
        .await
        .map_err(|e| ShaumError::NetworkError(format!("Failed to reach ip-api.com: {}", e)))?;

    let data: IpApiResponse = response
        .json()
        .await
        .map_err(|e| ShaumError::NetworkError(format!("Failed to parse response: {}", e)))?;

    if data.status != "success" {
        return Err(ShaumError::NetworkError(
            data.message.unwrap_or_else(|| "Unknown error from ip-api.com".to_string())
        ));
    }

    let lat = data.lat.ok_or_else(|| 
        ShaumError::NetworkError("Missing latitude in response".to_string())
    )?;
    let lon = data.lon.ok_or_else(|| 
        ShaumError::NetworkError("Missing longitude in response".to_string())
    )?;

    Ok(LocationInfo {
        coords: GeoCoordinate::new(lat, lon),
        city: data.city,
        region: data.region_name,
        country: data.country,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_get_location_info() {
        let result = get_location_info_from_ip().await;
        assert!(result.is_ok());
        
        let info = result.unwrap();
        println!("Location: {}", info.display_name());
        assert!(info.coords.lat >= -90.0 && info.coords.lat <= 90.0);
    }
}
