use agrocore_domain::entities::tenant::GeoCoordinates;
use agrocore_logging::warn;

/// Geocodes an address using the OpenStreetMap Nominatim API.
///
/// Returns coordinates if the address is valid, `None` if the address cannot
/// be geocoded or the API is unavailable.
///
/// Nominatim requires a User-Agent header and enforces a 1 request/second rate
/// limit. For setup flows this is fine — we call it once per setup.
pub async fn geocode_address(address: &str, country: &str) -> Option<GeoCoordinates> {
    let query = format!("{}, {}", address.trim(), country.trim());

    // URL-encode the query manually to avoid an extra dependency
    let encoded = url_encode(&query);
    let url = format!(
        "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1",
        encoded
    );

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to create HTTP client for geocoding: {}", e);
            return None;
        }
    };

    let resp = match client
        .get(&url)
        .header("User-Agent", "AgroCore/1.0 (agrocore@agrocore.com)")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!("Geocoding request failed: {}", e);
            return None;
        }
    };

    if !resp.status().is_success() {
        warn!("Geocoding API returned status {}", resp.status());
        return None;
    }

    let bodies: Vec<NominatimResult> = match resp.json().await {
        Ok(b) => b,
        Err(e) => {
            warn!("Failed to parse geocoding response: {}", e);
            return None;
        }
    };

    bodies.into_iter().next().map(|r| GeoCoordinates {
        lat: r.lat,
        lng: r.lon,
    })
}

/// Simple URL encoder for query parameters.
fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

#[derive(serde::Deserialize)]
struct NominatimResult {
    #[serde(rename = "lat")]
    lat: f64,
    #[serde(rename = "lon")]
    lon: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_encode_encodes_spaces_and_special_chars() {
        assert_eq!(url_encode("Rua Nova 1"), "Rua%20Nova%201");
        assert_eq!(url_encode("Strasse, Nr. 5"), "Strasse,%20Nr.%205");
        assert_eq!(url_encode("simple"), "simple");
    }

    #[test]
    fn url_encode_preserves_unreserved_chars() {
        assert_eq!(url_encode("a-b_c.d~e"), "a-b_c.d~e");
    }
}
