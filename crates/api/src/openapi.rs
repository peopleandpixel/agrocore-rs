//! Modular OpenAPI documentation.
//!
//! Instead of one monolithic `#[derive(OpenApi)]` struct with all paths and
//! schemas (which slows compilation when adding/modifying a single endpoint),
//! we split documentation into per-module `OpenApi` structs and merge them
//! at startup.

use utoipa::OpenApi;

// ---------------------------------------------------------------------------
// Per-module OpenApi structs
// ---------------------------------------------------------------------------

/// Authentication & authorization endpoints (login, JWT).
#[derive(OpenApi)]
#[openapi(
    paths(crate::handlers::auth::login),
    components(schemas(crate::handlers::auth::LoginRequest, crate::dto::AuthResponseDto,)),
    tags()
)]
struct AuthApiDoc;

/// Site management endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::sites::list_sites,
        crate::handlers::sites::get_site,
        crate::handlers::sites::create_site,
        crate::handlers::sites::update_site,
        crate::handlers::sites::delete_site,
        crate::handlers::sites::import_sites,
        crate::handlers::sites::import_geojson,
        crate::handlers::sites::import_shapefile,
    ),
    components(schemas(
        crate::dto::SiteDto,
        crate::dto::CreateSiteDto,
        crate::dto::UpdateSiteDto,
        crate::dto::PaginatedSiteResponse,
        crate::dto::ImportSitesRequest,
        crate::dto::GeoJsonImportRequest,
        crate::dto::ShapefileImportRequest,
        crate::dto::ImportResult,
        crate::dto::ImportError,
        crate::dto::ImportWarning,
        crate::dto::DuplicateDetectionResult,
        crate::dto::DuplicateMatchType,
        crate::dto::LpisValidationResult,
        crate::dto::GeoJsonFeature,
        crate::dto::GeoJsonGeometry,
    )),
    tags()
)]
struct SitesApiDoc;

/// Order & task management endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::orders::list_orders,
        crate::handlers::orders::get_order,
        crate::handlers::orders::create_order,
        crate::handlers::orders::update_order,
        crate::handlers::orders::delete_order,
        crate::handlers::orders::my_tasks,
    ),
    components(schemas(
        crate::dto::OrderDto,
        crate::dto::CreateOrderDto,
        crate::dto::UpdateOrderDto,
        crate::dto::PaginatedOrderResponse,
        crate::dto::TaskDataDto,
        crate::dto::CreateTaskDataDto,
        agrocore_domain::entities::order::RecurrenceRule,
        agrocore_domain::entities::order::RecurrenceCadence,
        agrocore_domain::entities::order::TaskExecutionMode,
        agrocore_domain::entities::order::TaskExecutionPolicy,
        agrocore_domain::entities::order::TaskAutomationState,
        agrocore_domain::entities::order::MyTask,
    )),
    tags()
)]
struct OrdersApiDoc;

/// Customer management endpoints (Kunden & Verkauf).
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::customers::list_customers,
        crate::handlers::customers::get_customer,
        crate::handlers::customers::create_customer,
        crate::handlers::customers::update_customer,
        crate::handlers::customers::delete_customer,
        crate::handlers::customers::search_customers,
        crate::handlers::customers::get_customer_by_number,
    ),
    components(schemas(
        crate::dto::customer::CustomerDto,
        crate::dto::customer::CreateCustomerDto,
        crate::dto::customer::UpdateCustomerDto,
    )),
    tags()
)]
struct CustomersApiDoc;

/// User management endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::users::list_users,
        crate::handlers::users::get_user,
        crate::handlers::users::create_user,
        crate::handlers::users::update_user,
        crate::handlers::users::delete_user,
    ),
    components(schemas(
        crate::dto::UserDto,
        crate::dto::CreateUserDto,
        crate::dto::UpdateUserDto,
        crate::dto::PaginatedUserResponse,
        agrocore_domain::entities::user::UserRole,
    )),
    tags()
)]
struct UsersApiDoc;

/// Task management endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::tasks::list_tasks,
        crate::handlers::tasks::get_task,
        crate::handlers::tasks::create_task,
        crate::handlers::tasks::update_task,
        crate::handlers::tasks::delete_task,
    ),
    components(schemas(
        crate::dto::TaskDataDto,
        crate::dto::CreateTaskDataDto,
        crate::dto::PaginatedTaskResponse,
    )),
    tags()
)]
struct TasksApiDoc;

/// Weather & phenology endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::weather::list_stations,
        crate::handlers::weather::get_station,
        crate::handlers::weather::create_station,
        crate::handlers::weather::list_weather_data,
        crate::handlers::weather::create_weather_data,
        crate::handlers::weather::list_phenology,
        crate::handlers::weather::create_phenology,
    ),
    components(schemas(
        crate::dto::PaginatedWeatherStationResponse,
        crate::dto::PaginatedWeatherDataResponse,
        crate::dto::PaginatedPhenologyResponse,
    )),
    tags()
)]
struct WeatherApiDoc;

/// Finance & cost-center endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::finance::list_pac_applications,
        crate::handlers::finance::create_pac_application,
        crate::handlers::finance::get_pac_application,
        crate::handlers::finance::list_cost_centers,
        crate::handlers::finance::create_cost_center,
        crate::handlers::finance::get_cost_center,
        crate::handlers::finance::list_financial_records,
        crate::handlers::finance::create_financial_record,
        crate::handlers::finance::get_financial_record,
    ),
    components(schemas(
        agrocore_domain::entities::finance::PACApplication,
        agrocore_domain::entities::finance::PACStatus,
        agrocore_domain::entities::finance::EcoSchemeParticipation,
        agrocore_domain::entities::finance::CostCenter,
        agrocore_domain::entities::finance::CostCenterType,
        agrocore_domain::entities::finance::FinancialRecord,
        agrocore_domain::entities::finance::FinancialRecordType,
        agrocore_domain::entities::finance::CreatePACApplicationDto,
        agrocore_domain::entities::finance::CreateCostCenterDto,
        agrocore_domain::entities::finance::CreateFinancialRecordDto,
        crate::dto::PaginatedPACApplicationResponse,
        crate::dto::PaginatedCostCenterResponse,
        crate::dto::PaginatedFinancialRecordResponse,
    )),
    tags()
)]
struct FinanceApiDoc;

/// Reporting / export endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::reporting::export_orders_excel,
        crate::handlers::reporting::export_sites_geojson,
        crate::handlers::reporting::export_pac_sip,
    ),
    tags()
)]
struct ReportingApiDoc;

/// Livestock management endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::livestock::list_animals,
        crate::handlers::livestock::get_animal,
        crate::handlers::livestock::create_animal,
        crate::handlers::livestock::update_animal,
        crate::handlers::livestock::delete_animal,
        crate::handlers::livestock::add_treatment,
        crate::handlers::livestock::add_grazing,
    ),
    components(schemas(
        agrocore_domain::entities::livestock::Animal,
        agrocore_domain::entities::livestock::AnimalSpecies,
        agrocore_domain::entities::livestock::AnimalStatus,
        agrocore_domain::entities::livestock::TreatmentRecord,
        agrocore_domain::entities::livestock::GrazingRecord,
        agrocore_domain::entities::livestock::CreateAnimalDto,
        agrocore_domain::entities::livestock::UpdateAnimalDto,
        crate::dto::PaginatedAnimalResponse,
    )),
    tags()
)]
struct LivestockApiDoc;

/// SIGPAC (Spanish agricultural parcels) endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::sigpac::list_sigpac_parcels,
        crate::handlers::sigpac::get_sigpac_parcel,
        crate::handlers::sigpac::search_parcels_near_point,
    ),
    components(schemas(
        crate::handlers::sigpac::SigpacParcelDto,
        crate::handlers::sigpac::PaginatedSigpacParcelResponse,
        crate::handlers::sigpac::SigpacParcelQuery,
        crate::handlers::sigpac::NearPointQuery,
    )),
    tags()
)]
struct SigpacApiDoc;

/// LPIS settings & provider endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::settings::get_lpis_settings,
        crate::handlers::settings::update_lpis_settings,
        crate::handlers::settings::list_lpis_providers,
    ),
    components(schemas(
        crate::dto::LpisSettingsResponse,
        crate::dto::UpdateLpisSettingsRequest,
        crate::dto::LpisProviderConfigList,
        crate::dto::LpisProviderConfig,
    )),
    tags()
)]
struct SettingsApiDoc;

/// IoT device registry endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::iot::list_devices,
        crate::handlers::iot::get_device,
        crate::handlers::iot::create_device,
        crate::handlers::iot::update_device,
        crate::handlers::iot::delete_device,
        crate::handlers::iot::get_device_telemetry,
        crate::handlers::iot::send_command,
        crate::handlers::iot::get_ha_discovery,
    ),
    components(schemas(
        crate::dto::CreateIoTDeviceDto,
        crate::dto::UpdateIoTDeviceDto,
        crate::dto::IoTDeviceResponse,
        crate::dto::IoTDeviceListResponse,
        crate::dto::IoTDeviceTelemetryResponse,
        crate::dto::IoTCommandRequestDto,
        crate::dto::IoTCommandResponseDto,
        crate::dto::HaDiscoveryConfigResponse,
        crate::dto::IoTCapabilityDto,
        crate::dto::IoTCapabilityType,
        crate::dto::DeviceStatusDto,
        crate::dto::CommandStatus,
    )),
    tags()
)]
struct IotApiDoc;

/// Error response schema.
#[derive(OpenApi)]
#[openapi(components(schemas(crate::dto::ErrorResponse,)), tags())]
struct ErrorApiDoc;

// ---------------------------------------------------------------------------
// Combined API doc — merges all module docs at startup
// ---------------------------------------------------------------------------

/// Combined OpenAPI document served at `/api-docs/openapi.json`.
pub struct ApiDoc;

impl ApiDoc {
    /// Merge all per-module `OpenApi` structs into one combined document.
    pub fn openapi() -> utoipa::openapi::OpenApi {
        let info = utoipa::openapi::Info::new("AgroCore API", env!("CARGO_PKG_VERSION"));
        let mut doc = utoipa::openapi::OpenApi::new(info, utoipa::openapi::Paths::new());
        doc.merge(AuthApiDoc::openapi());
        doc.merge(SitesApiDoc::openapi());
        doc.merge(OrdersApiDoc::openapi());
        doc.merge(CustomersApiDoc::openapi());
        doc.merge(UsersApiDoc::openapi());
        doc.merge(TasksApiDoc::openapi());
        doc.merge(WeatherApiDoc::openapi());
        doc.merge(FinanceApiDoc::openapi());
        doc.merge(ReportingApiDoc::openapi());
        doc.merge(LivestockApiDoc::openapi());
        doc.merge(SigpacApiDoc::openapi());
        doc.merge(SettingsApiDoc::openapi());
        doc.merge(IotApiDoc::openapi());
        doc.merge(ErrorApiDoc::openapi());
        doc
    }

    /// Build the combined OpenAPI document with bearer-JWT security scheme.
    pub fn openapi_with_security() -> utoipa::openapi::OpenApi {
        let mut doc = Self::openapi();
        if let Some(components) = doc.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
        doc
    }
}
