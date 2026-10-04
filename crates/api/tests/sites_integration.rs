mod common;

use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::handlers::configure;
use agrocore_domain::entities::site::Site;
use agrocore_domain::entities::{CropType, SiteType};
use agrocore_domain::repositories::{MockSiteRepository, PaginatedResponse};
use chrono::Utc;
use std::future::ready;
use std::sync::Arc;
use uuid::Uuid;

#[actix_web::test]
#[allow(deprecated)]
async fn test_list_sites() {
    let mut site_repo = MockSiteRepository::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let site = Site {
        id: Uuid::new_v4(),
        tenant_id,
        business_id: None,
        label: "Parcel 1".into(),
        site_type: SiteType::Vineyard,
        crop_type: CropType::Grape,
        variety: Some("Tempranillo".into()),
        area: 5.0,
        gross_area: Some(5.5),
        plots: serde_json::json!([]),
        row_config: None,
        bbch_stage: None,
        planted_date: None,
        cleared_date: None,
        soil_type: None,
        slope: None,
        slope_facing: None,
        altitude: None,
        organic: Some(true),
        organic_eligible: Some(true),
        center: None,
        sigpac_data: None,
        lpis_country: None,
        lpis_data: None,
        regepac_id: None,
        boundary: None,
        properties: None,
        custom_fields: None,
        note1: None,
        note2: None,
        is_active: true,
        is_temporary: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: None,
        updated_by: None,
    };

    site_repo
        .expect_find_all_visible()
        .returning(move |_, _, _, _| {
            Box::pin(ready(Ok(PaginatedResponse {
                data: vec![site.clone()],
                total: 1,
                page: 0,
                per_page: 20,
                total_pages: 1,
            })))
        });

    let mock_db = crate::common::db_with(|db| db.site_repo = Some(Arc::new(site_repo)));

    let state = crate::common::state_with(mock_db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token =
        crate::common::signed_token(&user_id.to_string(), &tenant_id.to_string(), &["Viewer"]);
    let req = test::TestRequest::get()
        .uri("/api/v1/sites")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
