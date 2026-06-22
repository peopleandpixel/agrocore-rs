use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware::Logger};
use std::sync::Arc;

pub mod dto;
pub mod handlers;
pub mod middleware;

use agrocore_infrastructure::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
}

pub async fn run_server(db: Database, bind_addr: &str) -> std::io::Result<()> {
    let state = web::Data::new(AppState { db: Arc::new(db) });

    HttpServer::new(move || {
        let cors = Cors::permissive()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(state.clone())
            .wrap(Logger::default())
            .wrap(cors)
            .configure(handlers::configure)
    })
    .bind(bind_addr)?
    .run()
    .await
}
