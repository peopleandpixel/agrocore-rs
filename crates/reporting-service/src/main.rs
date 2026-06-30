use rust_xlsxwriter::*;
use geojson::{Feature, FeatureCollection, Geometry, GeometryValue};
use agrocore_shared::Pagination;
use agrocore_infrastructure::Database;
use agrocore_domain::repositories::AnimalRepository;
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use uuid::Uuid;

pub struct ReportingService {
    db: Database,
}

impl ReportingService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn generate_orders_excel(&self, tenant_id: Uuid) -> anyhow::Result<Vec<u8>> {
        let pagination = Pagination { page: Some(1), per_page: Some(1000) };
        let orders = self.db.order_repo().find_all(tenant_id, pagination).await?.data;

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        let header_format = Format::new().set_bold();

        worksheet.write_with_format(0, 0, "ID", &header_format)?;
        worksheet.write_with_format(0, 1, "Label", &header_format)?;
        worksheet.write_with_format(0, 2, "Type", &header_format)?;
        worksheet.write_with_format(0, 3, "Status", &header_format)?;
        worksheet.write_with_format(0, 4, "Created At", &header_format)?;

        for (i, order) in orders.iter().enumerate() {
            let row = (i + 1) as u32;
            worksheet.write(row, 0, order.id.to_string())?;
            worksheet.write(row, 1, &order.label)?;
            worksheet.write(row, 2, format!("{:?}", order.order_type))?;
            worksheet.write(row, 3, format!("{:?}", order.status))?;
            worksheet.write(row, 4, order.created_at.to_rfc3339())?;
        }

        let buffer = workbook.save_to_buffer()?;
        Ok(buffer)
    }

    pub async fn generate_sites_geojson(&self, tenant_id: Uuid) -> anyhow::Result<FeatureCollection> {
        let pagination = Pagination { page: Some(1), per_page: Some(1000) };
        let sites = self.db.site_repo().find_all(tenant_id, pagination).await?.data;

        let mut features = Vec::new();

        for site in sites {
            if let Some(boundary) = site.boundary {
                let ring = boundary.iter().map(|p| vec![p.lng, p.lat].into()).collect::<Vec<geojson::Position>>();
                let geometry = Geometry::new(GeometryValue::Polygon { coordinates: vec![ring] });
                
                let mut properties = serde_json::Map::new();
                properties.insert("id".to_string(), serde_json::json!(site.id));
                properties.insert("label".to_string(), serde_json::json!(site.label));
                properties.insert("site_type".to_string(), serde_json::json!(site.site_type));
                properties.insert("crop_type".to_string(), serde_json::json!(site.crop_type));
                properties.insert("area".to_string(), serde_json::json!(site.area));

                features.push(Feature {
                    bbox: None,
                    geometry: Some(geometry),
                    id: None,
                    properties: Some(properties),
                    foreign_members: None,
                });
            }
        }

        Ok(FeatureCollection {
            bbox: None,
            features,
            foreign_members: None,
        })
    }

    pub async fn generate_pac_sip_excel(&self, tenant_id: Uuid) -> anyhow::Result<Vec<u8>> {
        let pagination = Pagination { page: Some(1), per_page: Some(1000) };
        let sites = self.db.site_repo().find_all(tenant_id, pagination).await?.data;

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        let header_format = Format::new().set_bold();

        worksheet.write_with_format(0, 0, "Provincia", &header_format)?;
        worksheet.write_with_format(0, 1, "Municipio", &header_format)?;
        worksheet.write_with_format(0, 2, "Agregado", &header_format)?;
        worksheet.write_with_format(0, 3, "Zona", &header_format)?;
        worksheet.write_with_format(0, 4, "Poligono", &header_format)?;
        worksheet.write_with_format(0, 5, "Parcela", &header_format)?;
        worksheet.write_with_format(0, 6, "Recinto", &header_format)?;
        worksheet.write_with_format(0, 7, "Uso", &header_format)?;
        worksheet.write_with_format(0, 8, "Superficie (ha)", &header_format)?;

        let mut row = 1;
        for site in sites {
            if let Some(sigpac) = site.sigpac_data {
                worksheet.write(row, 0, sigpac.province as u32)?;
                worksheet.write(row, 1, sigpac.municipality as u32)?;
                worksheet.write(row, 2, sigpac.aggregate as u32)?;
                worksheet.write(row, 3, sigpac.zone as u32)?;
                worksheet.write(row, 4, sigpac.polygon as u32)?;
                worksheet.write(row, 5, sigpac.parcel as u32)?;
                worksheet.write(row, 6, sigpac.enclosure as u32)?;
                worksheet.write(row, 7, sigpac.usage_code.unwrap_or_default())?;
                worksheet.write(row, 8, site.area)?;
                row += 1;
            }
        }

        let buffer = workbook.save_to_buffer()?;
        Ok(buffer)
    }

    pub async fn generate_veterinary_report(&self, tenant_id: Uuid) -> anyhow::Result<Vec<u8>> {
        let pagination = Pagination { page: Some(1), per_page: Some(1000) };
        let animals = self.db.animal_repo().find_all(tenant_id, pagination).await?.data;

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        let header_format = Format::new().set_bold();

        worksheet.write_with_format(0, 0, "Tier-ID", &header_format)?;
        worksheet.write_with_format(0, 1, "Art", &header_format)?;
        worksheet.write_with_format(0, 2, "Datum", &header_format)?;
        worksheet.write_with_format(0, 3, "Behandlung", &header_format)?;
        worksheet.write_with_format(0, 4, "Medikament", &header_format)?;
        worksheet.write_with_format(0, 5, "Wartezeit (Tage)", &header_format)?;

        let mut row = 1;
        for animal in animals {
            for treatment in animal.treatments {
                worksheet.write(row, 0, &animal.identifier)?;
                worksheet.write(row, 1, format!("{:?}", animal.species))?;
                worksheet.write(row, 2, treatment.date.to_rfc3339())?;
                worksheet.write(row, 3, &treatment.treatment_type)?;
                worksheet.write(row, 4, treatment.medication.as_deref().unwrap_or("-"))?;
                worksheet.write(row, 5, treatment.withdrawal_days.unwrap_or(0))?;
                row += 1;
            }
        }

        let buffer = workbook.save_to_buffer()?;
        Ok(buffer)
    }
}

pub mod worker;

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok", "service": "reporting"}))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    agrocore_shared::telemetry::init_telemetry("agrocore_reporting_service");

    let mongodb_uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://192.168.1.69:27017".to_string());
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://192.168.1.44:4222".to_string());
    let bind_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3002".to_string());

    let db = Database::connect(&mongodb_uri, "agrocore").await?;
    let reporting_service = ReportingService::new(db.clone());
    
    println!("Reporting Service starting on {}...", bind_addr);
    
    // Start NATS workers in background
    let nats_url_clone = nats_url.clone();
    tokio::spawn(async move {
        if let Err(e) = worker::start(reporting_service, nats_url_clone).await {
            eprintln!("Reporting worker error: {}", e);
        }
    });
    
    let db_clone = db.clone();
    tokio::spawn(async move {
        if let Err(e) = worker::start_audit_worker(db_clone, nats_url).await {
            eprintln!("Audit worker error: {}", e);
        }
    });

    // Start HTTP server for health checks
    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health))
    })
    .bind(bind_addr)?
    .run()
    .await?;

    Ok(())
}
