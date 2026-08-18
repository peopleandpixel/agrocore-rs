use actix_web::web;

pub mod agriculture;
pub mod auth;
pub mod compliance;
pub mod customers;
pub mod equipment;
pub mod finance;
pub mod harvest;
pub mod inventory;
pub mod iot;
pub mod livestock;
pub mod nutrition;
pub mod orders;
pub mod reporting;
pub mod settings;
pub mod sigpac;
pub mod sites;
pub mod specialized;
pub mod system;
pub mod tasks;
pub mod users;
pub mod water;
pub mod weather;
pub mod workforce;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Auth endpoints with stricter rate limiting (10 req/min per IP)
    let auth_gov_conf = actix_governor::GovernorConfigBuilder::default()
        .seconds_per_request(60)
        .burst_size(10)
        .key_extractor(crate::middleware::TestableIpKeyExtractor)
        .finish()
        .unwrap();
    cfg.service(
        web::scope("/api/v1")
            .service(web::resource("/health").route(web::get().to(health)))
            .service(
                web::resource("/auth/login")
                    .wrap(actix_governor::Governor::new(&auth_gov_conf))
                    .route(web::post().to(auth::login)),
            )
            .service(
                web::resource("/auth/refresh")
                    .wrap(actix_governor::Governor::new(&auth_gov_conf))
                    .route(web::post().to(auth::refresh_token)),
            )
            .service(
                web::resource("/auth/logout")
                    .wrap(actix_governor::Governor::new(&auth_gov_conf))
                    .route(web::post().to(auth::logout)),
            )
            .service(
                web::resource("/auth/impersonate/stop")
                    .wrap(actix_governor::Governor::new(&auth_gov_conf))
                    .route(web::post().to(auth::stop_impersonation)),
            )
            .service(
                web::resource("/auth/impersonate/{user_id}")
                    .wrap(actix_governor::Governor::new(&auth_gov_conf))
                    .route(web::post().to(auth::impersonate)),
            )
            .service(
                web::resource("/sites")
                    .route(web::get().to(sites::list_sites))
                    .route(web::post().to(sites::create_site)),
            )
            .service(
                web::resource("/sites/{id}")
                    .route(web::get().to(sites::get_site))
                    .route(web::put().to(sites::update_site))
                    .route(web::delete().to(sites::delete_site)),
            )
            .service(
                web::resource("/orders")
                    .route(web::get().to(orders::list_orders))
                    .route(web::post().to(orders::create_order)),
            )
            .service(web::resource("/orders/my-tasks").route(web::get().to(orders::my_tasks)))
            .service(
                web::resource("/orders/{id}")
                    .route(web::get().to(orders::get_order))
                    .route(web::put().to(orders::update_order))
                    .route(web::delete().to(orders::delete_order)),
            )
            .service(
                web::resource("/orders/{id}/complete")
                    .route(web::post().to(orders::complete_order)),
            )
            .service(web::resource("/orders/{id}/start").route(web::post().to(orders::start_order)))
            .service(
                web::resource("/tasks/{id}/start-for-worker")
                    .route(web::post().to(orders::start_task_for_worker)),
            )
            .service(
                web::resource("/tasks/{id}/stop-for-worker")
                    .route(web::post().to(orders::stop_task_for_worker)),
            )
            .service(
                web::resource("/users")
                    .route(web::get().to(users::list_users))
                    .route(web::post().to(users::create_user)),
            )
            .service(
                web::resource("/users/{id}")
                    .route(web::get().to(users::get_user))
                    .route(web::put().to(users::update_user))
                    .route(web::delete().to(users::delete_user)),
            )
            .service(
                web::resource("/tasks")
                    .route(web::get().to(tasks::list_tasks))
                    .route(web::post().to(tasks::create_task)),
            )
            .service(
                web::resource("/tasks/{id}")
                    .route(web::get().to(tasks::get_task))
                    .route(web::put().to(tasks::update_task))
                    .route(web::delete().to(tasks::delete_task)),
            )
            .service(
                web::scope("/system")
                    .route("/status", web::get().to(system::get_status))
                    .route("/setup", web::post().to(system::initial_setup))
                    .route("/tenant", web::delete().to(system::delete_tenant)),
            )
            .service(
                web::resource("/equipments")
                    .route(web::get().to(equipment::list_equipments))
                    .route(web::post().to(equipment::create_equipment)),
            )
            .service(
                web::resource("/equipments/{id}")
                    .route(web::get().to(equipment::get_equipment))
                    .route(web::put().to(equipment::update_equipment))
                    .route(web::delete().to(equipment::delete_equipment)),
            )
            // Inventory routes
            .service(
                web::resource("/inventory/items")
                    .route(web::get().to(inventory::list_inventory_items))
                    .route(web::post().to(inventory::create_inventory_item)),
            )
            .service(
                web::resource("/inventory/items/{id}")
                    .route(web::get().to(inventory::get_inventory_item))
                    .route(web::put().to(inventory::update_inventory_item))
                    .route(web::delete().to(inventory::delete_inventory_item)),
            )
            .service(
                web::resource("/inventory/balances")
                    .route(web::get().to(inventory::list_inventory_balances)),
            )
            .service(
                web::resource("/inventory/balances/below-minimum")
                    .route(web::get().to(inventory::list_below_minimum)),
            )
            .service(
                web::resource("/inventory/items/{id}/transactions")
                    .route(web::get().to(inventory::list_item_transactions)),
            )
            .service(
                web::resource("/inventory/transactions")
                    .route(web::post().to(inventory::create_transaction)),
            )
            .service(
                web::resource("/inventory/stock-in").route(web::post().to(inventory::stock_in)),
            )
            .service(
                web::resource("/inventory/stock-out").route(web::post().to(inventory::stock_out)),
            )
            .service(
                web::resource("/inventory/transfer")
                    .route(web::post().to(inventory::transfer_inventory)),
            )
            .service(
                web::resource("/inventory/adjust")
                    .route(web::post().to(inventory::adjust_inventory)),
            )
            .service(
                web::resource("/inventory/locations")
                    .route(web::get().to(inventory::list_inventory_locations))
                    .route(web::post().to(inventory::create_inventory_location)),
            )
            .configure(settings::configure)
            .configure(compliance::configure)
            .configure(customers::configure)
            .configure(specialized::configure)
            .configure(water::configure)
            .configure(workforce::configure)
            .configure(weather::configure)
            .configure(finance::configure)
            .configure(reporting::configure)
            .configure(nutrition::configure)
            .configure(harvest::configure)
            .configure(livestock::configure)
            .configure(iot::configure)
            .configure(agriculture::configure),
    );
}

async fn health() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}
