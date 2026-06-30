use actix_web::web;

pub mod auth;
pub mod sites;
pub mod orders;
pub mod users;
pub mod tasks;
pub mod system;
pub mod equipment;
pub mod compliance;
pub mod specialized;
pub mod water;
pub mod workforce;
pub mod weather;
pub mod finance;
pub mod reporting;
pub mod nutrition;
pub mod livestock;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(web::resource("/health").route(web::get().to(health)))
            .service(web::resource("/auth/login").route(web::post().to(auth::login)))
            .service(
                web::resource("/sites")
                    .route(web::get().to(sites::list_sites))
                    .route(web::post().to(sites::create_site))
            )
            .service(
                web::resource("/sites/{id}")
                    .route(web::get().to(sites::get_site))
                    .route(web::put().to(sites::update_site))
                    .route(web::delete().to(sites::delete_site))
            )
            .service(
                web::resource("/orders")
                    .route(web::get().to(orders::list_orders))
                    .route(web::post().to(orders::create_order))
            )
            .service(
                web::resource("/orders/my-tasks").route(web::get().to(orders::my_tasks))
            )
            .service(
                web::resource("/orders/{id}")
                    .route(web::get().to(orders::get_order))
                    .route(web::put().to(orders::update_order))
                    .route(web::delete().to(orders::delete_order))
            )
            .service(
                web::resource("/orders/{id}/complete")
                    .route(web::post().to(orders::complete_order))
            )
            .service(
                web::resource("/orders/{id}/start")
                    .route(web::post().to(orders::start_order))
            )
            .service(
                web::resource("/users")
                    .route(web::get().to(users::list_users))
                    .route(web::post().to(users::create_user))
            )
            .service(
                web::resource("/users/{id}")
                    .route(web::get().to(users::get_user))
                    .route(web::put().to(users::update_user))
                    .route(web::delete().to(users::delete_user))
            )
            .service(
                web::resource("/tasks")
                    .route(web::get().to(tasks::list_tasks))
                    .route(web::post().to(tasks::create_task))
            )
            .service(
                web::resource("/tasks/{id}")
                    .route(web::get().to(tasks::get_task))
                    .route(web::put().to(tasks::update_task))
                    .route(web::delete().to(tasks::delete_task))
            )
            .service(
                web::scope("/system")
                    .route("/status", web::get().to(system::get_status))
                    .route("/setup", web::post().to(system::initial_setup))
            )
            .service(
                web::resource("/equipments")
                    .route(web::get().to(equipment::list_equipments))
                    .route(web::post().to(equipment::create_equipment))
            )
            .service(
                web::resource("/equipments/{id}")
                    .route(web::get().to(equipment::get_equipment))
                    .route(web::put().to(equipment::update_equipment))
                    .route(web::delete().to(equipment::delete_equipment))
            )
            .configure(compliance::configure)
            .configure(specialized::configure)
            .configure(water::configure)
            .configure(workforce::configure)
            .configure(weather::configure)
            .configure(finance::configure)
            .configure(reporting::configure)
            .configure(nutrition::configure)
            .configure(livestock::configure)
    );
}

async fn health() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}
