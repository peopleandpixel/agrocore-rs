use crate::AppState;
use crate::dto::PaginatedResponseDto;
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::order::TaskExecutionMode;
use agrocore_domain::entities::site::GeoPoint;
use agrocore_domain::entities::workforce::{
    CreateWorkerDto, CreateWorkerLocationDto, CreateWorkLogDto, ReportLocationDto, UpdateWorkerDto,
    UpdateWorkLogDto,
};
use agrocore_messaging::{Event, GlobalEvent, SpatialPolygonEventKind, SpatialPresenceEvent};
use agrocore_shared::{Pagination, SharedError};
use chrono::{Utc};
use std::collections::HashSet;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/workforce")
            .service(
                web::resource("/workers")
                    .route(web::get().to(list_workers))
                    .route(web::post().to(create_worker)),
            )
            .service(
                web::resource("/workers/{id}")
                    .route(web::get().to(get_worker))
                    .route(web::put().to(update_worker))
                    .route(web::delete().to(delete_worker)),
            )
            .service(
                web::resource("/logs")
                    .route(web::get().to(list_work_logs))
                    .route(web::post().to(create_work_log)),
            )
            .service(
                web::resource("/logs/{id}")
                    .route(web::get().to(get_work_log))
                    .route(web::put().to(update_work_log))
                    .route(web::delete().to(delete_work_log)),
            )
            .service(
                web::resource("/locations")
                    .route(web::get().to(get_latest_locations))
                    .route(web::post().to(report_location)),
            ),
    );
}

pub async fn list_workers(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .worker_repo()
        .find_all(auth.0.tenant_id, query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data,
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn create_worker(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateWorkerDto>,
) -> Result<HttpResponse, ApiError> {
    let worker = state
        .db
        .worker_repo()
        .create(auth.0.tenant_id, dto.into_inner(), auth.0.user_id)
        .await?;
    Ok(HttpResponse::Created().json(worker))
}

pub async fn get_worker(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let worker = state
        .db
        .worker_repo()
        .find_by_id(auth.0.tenant_id, *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Worker not found".into()))?;
    Ok(HttpResponse::Ok().json(worker))
}

pub async fn update_worker(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateWorkerDto>,
) -> Result<HttpResponse, ApiError> {
    let worker = state
        .db
        .worker_repo()
        .update(auth.0.tenant_id, *id, dto.into_inner(), auth.0.user_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Worker not found".into()))?;
    Ok(HttpResponse::Ok().json(worker))
}

pub async fn delete_worker(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state.db.worker_repo().delete(auth.0.tenant_id, *id).await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Worker not found".into()).into())
    }
}

pub async fn list_work_logs(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .work_log_repo()
        .find_all(auth.0.tenant_id, query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data,
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn create_work_log(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateWorkLogDto>,
) -> Result<HttpResponse, ApiError> {
    let log = state
        .db
        .work_log_repo()
        .create(auth.0.tenant_id, dto.into_inner(), auth.0.user_id)
        .await?;
    Ok(HttpResponse::Created().json(log))
}

pub async fn get_work_log(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let log = state
        .db
        .work_log_repo()
        .find_by_id(auth.0.tenant_id, *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Work log not found".into()))?;
    Ok(HttpResponse::Ok().json(log))
}

pub async fn update_work_log(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateWorkLogDto>,
) -> Result<HttpResponse, ApiError> {
    let log = state
        .db
        .work_log_repo()
        .update(auth.0.tenant_id, *id, dto.into_inner(), auth.0.user_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Work log not found".into()))?;
    Ok(HttpResponse::Ok().json(log))
}

pub async fn delete_work_log(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state.db.work_log_repo().delete(auth.0.tenant_id, *id).await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Work log not found".into()).into())
    }
}

pub async fn report_location(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<ReportLocationDto>,
) -> Result<HttpResponse, ApiError> {
    let location = dto.into_inner();
    let previous_location = state
        .db
        .worker_location_repo()
        .find_latest_by_worker(auth.0.tenant_id, auth.0.user_id)
        .await
        .ok()
        .flatten();

    // Falls der User ein Worker ist, nutzen wir seine ID. Ansonsten müsste die ID im DTO sein,
    // aber laut Anforderung "Arbeiter sollen permanent ihre positionen melden können"
    // gehen wir davon aus, dass der Request vom Arbeiter selbst kommt.
    let create_dto = CreateWorkerLocationDto {
        worker_id: auth.0.user_id,
        lat: location.lat,
        lng: location.lng,
        current_task_id: location.current_task_id,
        timestamp: Utc::now(),
    };
    let loc = state
        .db
        .worker_location_repo()
        .create(auth.0.tenant_id, create_dto)
        .await?;
    let current_point = GeoPoint {
        lng: loc.lng,
        lat: loc.lat,
    };
    let previous_point = previous_location.as_ref().map(|previous| GeoPoint {
        lng: previous.lng,
        lat: previous.lat,
    });

    let current_objects = state
        .db
        .spatial_object_repo()
        .find_containing_point(auth.0.tenant_id, current_point.clone(), None)
        .await
        .unwrap_or_default();
    let previous_objects = if let Some(previous_point) = previous_point {
        state
            .db
            .spatial_object_repo()
            .find_containing_point(auth.0.tenant_id, previous_point, None)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let current_ids: HashSet<Uuid> = current_objects.iter().map(|object| object.id).collect();
    let previous_ids: HashSet<Uuid> = previous_objects.iter().map(|object| object.id).collect();
    let current_site_ids: HashSet<Uuid> = current_objects
        .iter()
        .filter_map(|object| object.site_id)
        .collect();

    for object in current_objects
        .iter()
        .filter(|object| !previous_ids.contains(&object.id))
    {
        let event = Event::new(
            auth.0.user_id.to_string(),
            GlobalEvent::SpatialPolygonEntered(SpatialPresenceEvent {
                tenant_id: auth.0.tenant_id,
                worker_id: auth.0.user_id,
                spatial_object_id: object.id,
                spatial_object_type: object.object_type.clone(),
                spatial_object_label: object.label.clone(),
                site_id: object.site_id,
                parent_id: object.parent_id,
                location: current_point.clone(),
                observed_at: loc.timestamp,
                kind: SpatialPolygonEventKind::EnteredPolygon,
            }),
        );
        let _ = state.messaging.publish("events.spatial", &event).await;
    }

    for object in current_objects.iter() {
        let event = Event::new(
            auth.0.user_id.to_string(),
            GlobalEvent::SpatialPolygonIn(SpatialPresenceEvent {
                tenant_id: auth.0.tenant_id,
                worker_id: auth.0.user_id,
                spatial_object_id: object.id,
                spatial_object_type: object.object_type.clone(),
                spatial_object_label: object.label.clone(),
                site_id: object.site_id,
                parent_id: object.parent_id,
                location: current_point.clone(),
                observed_at: loc.timestamp,
                kind: SpatialPolygonEventKind::InPolygon,
            }),
        );
        let _ = state.messaging.publish("events.spatial", &event).await;
    }

    match state
        .db
        .order_repo()
        .find_assigned_to_worker(auth.0.tenant_id, auth.0.user_id)
        .await
    {
        Ok(mut orders) => {
            for mut order in orders.drain(..) {
                let Some(policy) = order.execution_policy.as_ref() else {
                    continue;
                };
                if policy.mode != TaskExecutionMode::AutoPresence {
                    continue;
                }

                let inside = order
                    .site_ids
                    .iter()
                    .any(|site_id| current_site_ids.contains(site_id));
                let duration_minutes = order.expected_work_duration_minutes();
                let action = order.observe_presence(inside, loc.timestamp);

                if action.is_none() {
                    if let Err(e) = state
                        .db
                        .order_repo()
                        .update(
                            auth.0.tenant_id,
                            order.id,
                            agrocore_domain::entities::order::UpdateOrderDto {
                                status: Some(order.status.clone()),
                                planned_date: order.planned_date,
                                deadline_date: order.deadline_date,
                                started_at: order.started_at,
                                completed_at: order.completed_at,
                                last_completed_at: order.last_completed_at,
                                recurrence: order.recurrence.clone(),
                                execution_policy: order.execution_policy.clone(),
                                automation_state: order.automation_state.clone(),
                                site_ids: Some(order.site_ids.clone()),
                                assigned_worker_ids: Some(order.assigned_worker_ids.clone()),
                                ..Default::default()
                            },
                            auth.0.user_id,
                        )
                        .await
                    {
                        tracing::warn!(
                            "Failed to persist auto-presence state for order {}: {}",
                            order.id,
                            e
                        );
                    }
                    continue;
                }

                if let Err(e) = state
                    .db
                    .order_repo()
                    .update(
                        auth.0.tenant_id,
                        order.id,
                        agrocore_domain::entities::order::UpdateOrderDto {
                            status: Some(order.status.clone()),
                            planned_date: order.planned_date,
                            deadline_date: order.deadline_date,
                            started_at: order.started_at,
                            completed_at: order.completed_at,
                            last_completed_at: order.last_completed_at,
                            recurrence: order.recurrence.clone(),
                            execution_policy: order.execution_policy.clone(),
                            automation_state: order.automation_state.clone(),
                            site_ids: Some(order.site_ids.clone()),
                            assigned_worker_ids: Some(order.assigned_worker_ids.clone()),
                            ..Default::default()
                        },
                        auth.0.user_id,
                    )
                    .await
                {
                    tracing::warn!(
                        "Failed to persist auto-presence transition for order {}: {}",
                        order.id,
                        e
                    );
                }

                let event = Event::new("api".into(), GlobalEvent::OrderUpdated(order.clone()));
                let _ = state.messaging.publish("events.orders", &event).await;

                if matches!(
                    action,
                    Some(agrocore_domain::entities::order::TaskAutomationAction::Completed)
                ) {
                    let worker_id = match state
                        .db
                        .worker_repo()
                        .find_by_user_id(auth.0.tenant_id, auth.0.user_id)
                        .await
                    {
                        Ok(Some(worker)) => worker.id,
                        Ok(None) => auth.0.user_id,
                        Err(e) => {
                            tracing::warn!(
                                "Failed to resolve worker for auto-complete worklog: {}",
                                e
                            );
                            auth.0.user_id
                        }
                    };
                    let worklog = agrocore_domain::entities::workforce::CreateWorkLogDto {
                        worker_id,
                        date: loc.timestamp,
                        hours_worked: duration_minutes.unwrap_or(0) as f64 / 60.0,
                        overtime_hours: 0.0,
                        rest_period_hours: 0.0,
                        task_description: order.label.clone(),
                        site_id: order.site_ids.first().copied(),
                        is_night_shift: false,
                        breaks_taken: 0,
                    };
                    if let Err(e) = state
                        .db
                        .work_log_repo()
                        .create(auth.0.tenant_id, worklog, auth.0.user_id)
                        .await
                    {
                        tracing::warn!(
                            "Failed to create auto-complete worklog for order {}: {}",
                            order.id,
                            e
                        );
                    }
                }
            }
        }
        Err(e) => tracing::warn!("Failed to load auto-presence orders: {}", e),
    }

    for object in previous_objects
        .iter()
        .filter(|object| !current_ids.contains(&object.id))
    {
        let event = Event::new(
            auth.0.user_id.to_string(),
            GlobalEvent::SpatialPolygonLeft(SpatialPresenceEvent {
                tenant_id: auth.0.tenant_id,
                worker_id: auth.0.user_id,
                spatial_object_id: object.id,
                spatial_object_type: object.object_type.clone(),
                spatial_object_label: object.label.clone(),
                site_id: object.site_id,
                parent_id: object.parent_id,
                location: current_point.clone(),
                observed_at: loc.timestamp,
                kind: SpatialPolygonEventKind::LeftPolygon,
            }),
        );
        let _ = state.messaging.publish("events.spatial", &event).await;
    }

    Ok(HttpResponse::Created().json(loc))
}

pub async fn get_latest_locations(
    state: web::Data<AppState>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let locations = state
        .db
        .worker_location_repo()
        .get_latest_locations(auth.0.tenant_id)
        .await?;
    Ok(HttpResponse::Ok().json(locations))
}
