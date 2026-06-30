use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, HttpServer};
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use actix_web_prometheus::PrometheusMetricsBuilder;
use std::sync::Arc;

pub mod dto;
pub mod handlers;
pub mod middleware;

#[cfg(test)]
mod dto_validation_tests;

use agrocore_infrastructure::Database;
use agrocore_messaging::MessagingClient;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub messaging: Arc<MessagingClient>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::login,
        handlers::sites::list_sites,
        handlers::sites::get_site,
        handlers::sites::create_site,
        handlers::sites::update_site,
        handlers::sites::delete_site,
        handlers::orders::list_orders,
        handlers::orders::get_order,
        handlers::orders::create_order,
        handlers::orders::update_order,
        handlers::orders::delete_order,
        handlers::orders::my_tasks,
        handlers::users::list_users,
        handlers::users::get_user,
        handlers::users::create_user,
        handlers::users::update_user,
        handlers::users::delete_user,
        handlers::tasks::list_tasks,
        handlers::tasks::get_task,
        handlers::tasks::create_task,
        handlers::tasks::update_task,
        handlers::tasks::delete_task,
        handlers::weather::list_stations,
        handlers::weather::get_station,
        handlers::weather::create_station,
        handlers::weather::list_weather_data,
        handlers::weather::create_weather_data,
        handlers::weather::list_phenology,
        handlers::weather::create_phenology,
        handlers::finance::list_pac_applications,
        handlers::finance::create_pac_application,
        handlers::finance::get_pac_application,
        handlers::finance::list_cost_centers,
        handlers::finance::create_cost_center,
        handlers::finance::get_cost_center,
        handlers::finance::list_financial_records,
        handlers::finance::create_financial_record,
        handlers::finance::get_financial_record,
        handlers::reporting::export_orders_excel,
        handlers::reporting::export_sites_geojson,
        handlers::reporting::export_pac_sip,
        handlers::livestock::list_animals,
        handlers::livestock::get_animal,
        handlers::livestock::create_animal,
        handlers::livestock::update_animal,
        handlers::livestock::delete_animal,
        handlers::livestock::add_treatment,
        handlers::livestock::add_grazing,
    ),
    components(
        schemas(
            handlers::auth::LoginRequest, 
            dto::AuthResponseDto, 
            dto::SiteDto, 
            dto::OrderDto, 
            dto::UserDto,
            dto::TaskDataDto,
            dto::CreateSiteDto, 
            dto::UpdateSiteDto,
            dto::CreateOrderDto,
            dto::UpdateOrderDto,
            dto::CreateUserDto,
            dto::UpdateUserDto,
            dto::CreateTaskDataDto,
            dto::ErrorResponse,
            dto::PaginatedSiteResponse,
            dto::PaginatedOrderResponse,
            dto::PaginatedUserResponse,
            dto::PaginatedTaskResponse,
            dto::PaginatedWeatherStationResponse,
            dto::PaginatedWeatherDataResponse,
            dto::PaginatedPhenologyResponse,
            dto::PaginatedPACApplicationResponse,
            dto::PaginatedCostCenterResponse,
            dto::PaginatedFinancialRecordResponse,
            dto::PaginatedAnimalResponse,
            agrocore_domain::entities::order::MyTask,
            agrocore_domain::entities::user::UserRole,
            agrocore_domain::entities::SiteType,
            agrocore_domain::entities::CropType,
            agrocore_domain::entities::BbchStage,
            agrocore_domain::entities::OrderType,
            agrocore_domain::entities::OrderStatus,
            agrocore_domain::entities::compliance::AuditAction,
            agrocore_domain::entities::compliance::ChecklistType,
            agrocore_domain::entities::compliance::ComplianceStatus,
            agrocore_domain::entities::vineyard::DocArea,
            agrocore_domain::entities::vineyard::QualityGrade,
            agrocore_domain::entities::olive::OilGrade,
            agrocore_domain::entities::water::WaterSourceType,
            agrocore_domain::entities::water::IrrigationMethod,
            agrocore_domain::entities::workforce::ContractType,
            agrocore_domain::entities::finance::PACApplication,
            agrocore_domain::entities::finance::PACStatus,
            agrocore_domain::entities::finance::EcoSchemeParticipation,
            agrocore_domain::entities::finance::CostCenter,
            agrocore_domain::entities::finance::CostCenterType,
            agrocore_domain::entities::finance::FinancialRecord,
            agrocore_domain::entities::finance::FinancialRecordType,
            agrocore_domain::entities::site::SigpacData,
            agrocore_domain::entities::weather::WeatherStation,
            agrocore_domain::entities::weather::WeatherStationType,
            agrocore_domain::entities::weather::WeatherData,
            agrocore_domain::entities::weather::PhenologyRecord,
            agrocore_domain::entities::weather::CreateWeatherStationDto,
            agrocore_domain::entities::weather::CreateWeatherDataDto,
            agrocore_domain::entities::weather::CreatePhenologyRecordDto,
            agrocore_domain::entities::finance::PACApplication,
            agrocore_domain::entities::finance::PACStatus,
            agrocore_domain::entities::finance::CreatePACApplicationDto,
            agrocore_domain::entities::finance::CostCenter,
            agrocore_domain::entities::finance::CostCenterType,
            agrocore_domain::entities::finance::CreateCostCenterDto,
            agrocore_domain::entities::finance::FinancialRecord,
            agrocore_domain::entities::finance::FinancialRecordType,
            agrocore_domain::entities::finance::CreateFinancialRecordDto,
            agrocore_domain::entities::finance::EcoSchemeParticipation,
            agrocore_domain::entities::livestock::Animal,
            agrocore_domain::entities::livestock::AnimalSpecies,
            agrocore_domain::entities::livestock::AnimalStatus,
            agrocore_domain::entities::livestock::TreatmentRecord,
            agrocore_domain::entities::livestock::GrazingRecord,
            agrocore_domain::entities::livestock::CreateAnimalDto,
            agrocore_domain::entities::livestock::UpdateAnimalDto,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "agrocore-rs", description = "AgroCore API")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::HttpBuilder::new()
                    .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
    }
}

pub async fn run_server(db: Database, messaging: MessagingClient, bind_addr: &str) -> std::io::Result<()> {
    let state = web::Data::new(AppState { 
        db: Arc::new(db),
        messaging: Arc::new(messaging),
    });

    let prometheus = PrometheusMetricsBuilder::new("agrocore")
        .endpoint("/metrics")
        .build()
        .unwrap();

    HttpServer::new(move || {
        let cors = Cors::permissive()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(state.clone())
            .wrap(prometheus.clone())
            .wrap(TracingLogger::default())
            .wrap(cors)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .configure(handlers::configure)
            .service(
                fs::Files::new("/admin", "/var/lib/agrocore/admin-ui")
                    .index_file("index.html")
                    .default_handler(web::to(|| async {
                        fs::NamedFile::open("/var/lib/agrocore/admin-ui/index.html")
                    }))
                    .show_files_listing()
            )
    })
    .bind(bind_addr)?
    .run()
    .await
}
