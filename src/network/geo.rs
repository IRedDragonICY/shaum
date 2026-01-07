//! IP-based Geolocation Module.
//!
//! Provides location detection via local MaxMind database or async HTTP fallback.

use crate::calendar::ShaumError;
use crate::types::GeoCoordinate;
use serde::Deserialize;

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

// =============================================================================
// Local MaxMind Database Lookup (privacy-preserving, offline)
// =============================================================================

/// Local geolocation provider using MaxMind GeoIP database.
///
/// This approach is privacy-preserving as it does not send any data
/// over the network - all lookups are performed locally.
#[cfg(feature = "local-geo")]
pub struct LocalGeoProvider;

#[cfg(feature = "local-geo")]
impl LocalGeoProvider {
    /// Looks up location information for an IP address using a local MaxMind database.
    ///
    /// # Arguments
    /// * `ip` - The IP address to look up
    /// * `db_path` - Path to the MaxMind GeoLite2 City database (.mmdb file)
    ///
    /// # Errors
    /// Returns `ShaumError::DatabaseError` if the database cannot be opened or lookup fails.
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::net::IpAddr;
    /// use std::path::Path;
    /// use shaum::network::geo::LocalGeoProvider;
    ///
    /// let ip: IpAddr = "8.8.8.8".parse().unwrap();
    /// let db_path = Path::new("/path/to/GeoLite2-City.mmdb");
    ///
    /// let info = LocalGeoProvider::lookup(ip, db_path).unwrap();
    /// println!("Location: {}", info.display_name());
    /// ```
    pub fn lookup(
        ip: std::net::IpAddr,
        db_path: &std::path::Path,
    ) -> Result<LocationInfo, ShaumError> {
        use maxminddb::{Reader, geoip2};

        let reader = Reader::open_readfile(db_path).map_err(|e| {
            ShaumError::DatabaseError(format!(
                "Failed to open MaxMind DB at {:?}: {}",
                db_path, e
            ))
        })?;

        let city: geoip2::City = reader.lookup(ip).map_err(|e| {
            ShaumError::DatabaseError(format!("IP lookup failed for {}: {}", ip, e))
        })?;

        let location = city.location.ok_or_else(|| {
            ShaumError::DatabaseError(format!("No location data for IP {}", ip))
        })?;

        let lat = location.latitude.unwrap_or(0.0);
        let lng = location.longitude.unwrap_or(0.0);

        Ok(LocationInfo {
            coords: GeoCoordinate::new_unchecked(lat, lng),
            city: city
                .city
                .and_then(|c| c.names)
                .and_then(|n| n.get("en").map(|s| s.to_string())),
            region: city
                .subdivisions
                .and_then(|s| s.into_iter().next())
                .and_then(|s| s.names)
                .and_then(|n| n.get("en").map(|s| s.to_string())),
            country: city
                .country
                .and_then(|c| c.names)
                .and_then(|n| n.get("en").map(|s| s.to_string())),
        })
    }
}



// =============================================================================
// Nominatim Reverse Geocoding (OpenStreetMap - detailed address lookup)
// =============================================================================

/// Detailed location info with Indonesian administrative divisions.
#[cfg(feature = "async")]
#[derive(Debug, Clone)]
pub struct DetailedLocationInfo {
    /// Geographic coordinates.
    pub coords: GeoCoordinate,
    /// Kelurahan/Desa (village).
    pub kelurahan: Option<String>,
    /// Kecamatan (district).
    pub kecamatan: Option<String>,
    /// Kabupaten/Kota (regency/city).
    pub kabupaten: Option<String>,
    /// Provinsi (province).
    pub provinsi: Option<String>,
    /// Country name.
    pub country: Option<String>,
    /// Full formatted address.
    pub display_name: String,
}

#[cfg(feature = "async")]
impl DetailedLocationInfo {
    /// Returns formatted Indonesian-style address.
    pub fn alamat_lengkap(&self) -> String {
        let parts: Vec<&str> = [
            self.kelurahan.as_deref(),
            self.kecamatan.as_deref().map(|k| format!("Kec. {}", k).leak() as &str),
            self.kabupaten.as_deref(),
            self.provinsi.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect();
        
        if parts.is_empty() {
            self.display_name.clone()
        } else {
            parts.join(", ")
        }
    }
}

/// Nominatim API response structure.
#[cfg(feature = "async")]
#[derive(Debug, Deserialize)]
struct NominatimResponse {
    display_name: String,
    address: NominatimAddress,
}

#[cfg(feature = "async")]
#[derive(Debug, Deserialize)]
struct NominatimAddress {
    // Indonesian administrative levels
    village: Option<String>,      // Kelurahan/Desa
    suburb: Option<String>,       // Alternative for kelurahan
    neighbourhood: Option<String>,
    
    // Kecamatan can be in multiple fields
    county: Option<String>,       // Sometimes kecamatan
    municipality: Option<String>, // Sometimes kecamatan
    city_district: Option<String>,
    
    // Higher levels
    city: Option<String>,
    town: Option<String>,
    
    #[serde(rename = "state")]
    province: Option<String>,
    
    country: Option<String>,
    
    // Fallback fields
    #[serde(rename = "ISO3166-2-lvl4")]
    iso_province: Option<String>,
}

/// Performs reverse geocoding using OpenStreetMap Nominatim API.
///
/// Returns detailed location info including kelurahan, kecamatan, kabupaten.
///
/// # Rate Limiting
/// Nominatim has a 1 request/second limit. Please respect this.
///
/// # Example
/// ```rust,no_run
/// use shaum::network::geo::reverse_geocode;
/// use shaum::types::GeoCoordinate;
///
/// #[tokio::main]
/// async fn main() {
///     let coords = GeoCoordinate::new(-7.8195, 110.3610).unwrap();
///     let info = reverse_geocode(coords).await.unwrap();
///     
///     println!("Kelurahan: {:?}", info.kelurahan);
///     println!("Kecamatan: {:?}", info.kecamatan);
///     println!("Kabupaten: {:?}", info.kabupaten);
/// }
/// ```
#[cfg(feature = "async")]
pub async fn reverse_geocode(coords: GeoCoordinate) -> Result<DetailedLocationInfo, ShaumError> {
    let client = reqwest::Client::builder()
        .user_agent("shaum-lib/0.6.0 (Islamic prayer times library)")
        .build()
        .map_err(|e| ShaumError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;
    
    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=json&addressdetails=1&accept-language=id",
        coords.lat, coords.lng
    );
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ShaumError::NetworkError(format!("Nominatim request failed: {}", e)))?;
    
    let data: NominatimResponse = response
        .json()
        .await
        .map_err(|e| ShaumError::NetworkError(format!("Failed to parse Nominatim response: {}", e)))?;
    
    let addr = &data.address;
    
    // Extract kelurahan (village level)
    let kelurahan = addr.village.clone()
        .or_else(|| addr.suburb.clone())
        .or_else(|| addr.neighbourhood.clone());
    
    // Extract kecamatan (district level)  
    let kecamatan = addr.county.clone()
        .or_else(|| addr.municipality.clone())
        .or_else(|| addr.city_district.clone());
    
    // Extract kabupaten/kota
    let kabupaten = addr.city.clone()
        .or_else(|| addr.town.clone());
    
    Ok(DetailedLocationInfo {
        coords,
        kelurahan,
        kecamatan,
        kabupaten,
        provinsi: addr.province.clone(),
        country: addr.country.clone(),
        display_name: data.display_name,
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_info_display_name() {
        let info = LocationInfo {
            coords: GeoCoordinate::new_unchecked(-6.2088, 106.8456),
            city: Some("Jakarta".to_string()),
            region: Some("DKI Jakarta".to_string()),
            country: Some("Indonesia".to_string()),
        };
        assert_eq!(info.display_name(), "Jakarta, DKI Jakarta, Indonesia");
    }

    #[test]
    fn test_location_info_display_name_coords_only() {
        let info = LocationInfo {
            coords: GeoCoordinate::new_unchecked(-6.2088, 106.8456),
            city: None,
            region: None,
            country: None,
        };
        assert!(info.display_name().contains("-6.2088"));
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    #[ignore]
    async fn test_get_location_info_http() {
        #[allow(deprecated)]
        let result = get_location_info_from_ip().await;
        assert!(result.is_ok());
    }
}
