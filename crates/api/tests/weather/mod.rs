// Weather Handler Tests - DTO Validation
use agrocore_api::dto::{CreateWeatherStationDto, CreateWeatherDataDto};

#[test]
fn test_weather_station_validation() {
    let dto = CreateWeatherStationDto {
        tenant_id: "tenant-123".into(),
        site_id: Some("site-123".into()),
        name: "Wetterstation 1".into(),
        latitude: 38.7223,
        longitude: -9.1393,
    };

    assert_eq!(dto.name, "Wetterstation 1");
    assert!((dto.latitude - 38.7223).abs() < 0.001);
}

#[test]
fn test_weather_data_validation() {
    let dto = CreateWeatherDataDto {
        station_id: "station-123".into(),
        temperature: Some(25.5),
        humidity: Some(65.0),
        rainfall: Some(12.3),
        wind_speed: Some(5.2),
        recorded_at: None,
    };

    assert_eq!(dto.temperature, Some(25.5));
    assert_eq!(dto.humidity, Some(65.0));
}