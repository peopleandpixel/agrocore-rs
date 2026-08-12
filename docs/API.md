# AgroCore-RS API Documentation

> Version: 0.8.3  
> Base URL: `http://localhost:8080`  
> API Prefix: `/api/v1`

---

## 1. Authentication

All API endpoints require a Bearer token (JWT) in the `Authorization` header, except for the login and initial setup endpoints.

```
Authorization: Bearer <your-jwt-token>
```

### 1.1 Login

**Endpoint:** `POST /api/v1/auth/login`

Authenticate with email and password to receive a JWT access token and refresh token.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "your_password"
}
```

**Response (200):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "token_expires_in": 3600,
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "tenant_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "firstname": "John",
  "lastname": "Farmer",
  "roles": ["admin", "manager"]
}
```

| Field              | Type   | Description                          |
|--------------------|--------|--------------------------------------|
| token              | string | JWT access token (expires in 30 min) |
| refresh_token      | string | Refresh token (expires in 7 days)    |
| token_expires_in   | int    | Seconds until access token expires   |
| user_id            | UUID   | User identifier                      |
| tenant_id          | UUID   | Tenant identifier                    |
| firstname          | string | User's first name                    |
| lastname           | string | User's last name                     |
| roles              | array  | User roles (admin, manager, worker, viewer) |

**Errors:**
- 400: Invalid request (validation failed)
- 401: Invalid credentials

### 1.2 Refresh Token

**Endpoint:** `POST /api/v1/auth/refresh`

Exchange a refresh token for a new access token and a new refresh token.

**Request Body:**
```json
{
  "refresh_token": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
}
```

**Response (200):** Same structure as login response.

---

## 2. Pagination

All list endpoints support pagination via query parameters:

| Parameter  | Type   | Default | Description           |
|------------|--------|---------|-----------------------|
| page       | uint   | 1       | Page number (1-based) |
| per_page   | uint   | 50      | Items per page (max varies) |

### Paginated Response Format

All list endpoints return:
```json
{
  "data": [/* array of items */],
  "total": 100,
  "page": 1,
  "per_page": 50,
  "total_pages": 2
}
```

### Error Response Format

```json
{
  "error": "ValidationError",
  "message": "Detailed error message"
}
```

---

## 3. Health Check

**Endpoint:** `GET /api/v1/health`

No authentication required.

**Response (200):**
```json
{
  "status": "ok"
}
```

---

## 4. Sites & Parcels

Base path: `/api/v1/sites`

### 4.1 List Sites

**Endpoint:** `GET /api/v1/sites?page=1&per_page=50`

Lists all sites visible to the authenticated user. Workers see only assigned sites; managers/admins see all.

**Response (200):**
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "tenant_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
      "label": "North Field",
      "site_type": "Main",
      "crop_type": "Cereal",
      "variety": "Winter Wheat",
      "area": 12.5,
      "gross_area": 13.2,
      "bbch_stage": "71",
      "soil_type": "Clay",
      "slope": 2.5,
      "altitude": 150.0,
      "organic": true,
      "center": {"lng": 8.5, "lat": 47.3},
      "boundary": [{"lng": 8.5, "lat": 47.3}, {"lng": 8.6, "lat": 47.4}],
      "is_active": true,
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-02-20T14:22:00Z"
    }
  ],
  "total": 42,
  "page": 1,
  "per_page": 50,
  "total_pages": 1
}
```

**Roles:** admin, manager, worker, viewer

### 4.2 Get Site

**Endpoint:** `GET /api/v1/sites/{id}`

Returns a single site by UUID. Returns 404 if not found.

**Roles:** admin, manager, worker (assigned sites only), viewer (assigned sites only)

### 4.3 Create Site

**Endpoint:** `POST /api/v1/sites`

Creates a new site.

**Request Body:**
```json
{
  "label": "North Field",
  "site_type": "Main",
  "crop_type": "Cereal",
  "variety": "Winter Wheat",
  "area": 12.5,
  "center": {"lng": 8.5, "lat": 47.3},
  "boundary": [{"lng": 8.5, "lat": 47.3}, {"lng": 8.6, "lat": 47.4}]
}
```

**Response (201):** The created site object (same structure as 4.1 response)

**Roles:** admin, manager

### 4.4 Update Site

**Endpoint:** `PUT /api/v1/sites/{id}`

Updates an existing site.

**Request Body:** Same as create, but partial (all fields optional).

**Roles:** admin, manager

### 4.5 Delete Site

**Endpoint:** `DELETE /api/v1/sites/{id}`

Deletes a site. Returns `{"deleted": true}` on success.

**Roles:** admin, manager

### 4.6 Import Sites (JSON)

**Endpoint:** `POST /api/v1/sites/import`

Bulk import sites via JSON request body.

**Request Body:**
```json
{
  "sites": [
    {
      "label": "Field 1",
      "site_type": "Main",
      "crop_type": "Cereal",
      "area": 5.0,
      "boundary": [{"lng": 8.5, "lat": 47.3}]
    }
  ],
  "skip_duplicates": true,
  "update_existing": false,
  "validate_lpis": true,
  "source": "GeoJSON",
  "lpis_country": "ES"
}
```

**Response (200):**
```json
{
  "total": 10,
  "created": 8,
  "updated": 1,
  "skipped": 1,
  "errors": [],
  "warnings": [],
  "duplicate_ids": []
}
```

### 4.7 Import GeoJSON

**Endpoint:** `POST /api/v1/sites/import/geojson`

Import sites from a GeoJSON FeatureCollection.

**Request Body:**
```json
{
  "features": [
    {
      "type": "Feature",
      "geometry": {
        "type": "Polygon",
        "coordinates": [[[8.5, 47.3], [8.6, 47.4], ...]]
      },
      "properties": {
        "label": "Field A",
        "crop_type": "Cereal"
      }
    }
  ],
  "validate_lpis": true,
  "lpis_country": "ES"
}
```

### 4.8 Import Shapefile

**Endpoint:** `POST /api/v1/sites/import/shapefile`

Import sites from a base64-encoded Shapefile.

**Request Body:**
```json
{
  "file_base64": "<base64-encoded-shapefile>",
  "encoding": "UTF-8",
  "validate_lpis": true,
  "lpis_country": "ES"
}
```

**Roles for all import endpoints:** admin, manager

---

## 5. Orders & Tasks

Base path: `/api/v1/orders` and `/api/v1/tasks`

### 5.1 List Orders

**Endpoint:** `GET /api/v1/orders?page=1&per_page=50`

**Response (200):** Paginated list of OrderDto objects.

| Field              | Type    | Description                          |
|--------------------|---------|--------------------------------------|
| id                 | UUID    | Order identifier                     |
| label              | string  | Human-readable label                 |
| order_type         | enum    | Order type (see §12)                 |
| status             | enum    | Order status (Draft, Planned, InProgress, Completed, Cancelled, Paused) |
| site_ids           | array   | UUID array of affected sites         |
| assigned_worker_ids| array   | UUID array of assigned workers       |
| planned_date       | string  | ISO 8601 planned date                |
| deadline_date      | string  | ISO 8601 deadline                    |
| started_at         | string  | ISO 8601 start timestamp             |
| completed_at       | string  | ISO 8601 completion timestamp        |
| recurrence         | object  | RecurrenceRule (optional)            |
| execution_policy   | object  | TaskExecutionPolicy (optional)       |

**Roles:** admin, manager, worker, viewer

### 5.2 Get Order

**Endpoint:** `GET /api/v1/orders/{id}`

**Roles:** admin, manager, worker, viewer

### 5.3 Create Order

**Endpoint:** `POST /api/v1/orders`

**Request Body:**
```json
{
  "label": "Spring Plowing",
  "order_type": "Plowing",
  "site_ids": ["550e8400-e29b-41d4-a716-446655440000"],
  "planned_date": "2024-03-15T08:00:00Z",
  "deadline_date": "2024-03-20T17:00:00Z",
  "assigned_worker_ids": ["550e8400-e29b-41d4-a716-446655440001"],
  "recurrence": {
    "cadence": "Yearly",
    "interval": 1,
    "end_date": "2025-12-31T23:59:59Z"
  },
  "execution_policy": {
    "mode": "Manual",
    "auto_start": true,
    "auto_complete": true,
    "notify_on_completion": false
  }
}
```

**Response (201):** The created OrderDto object.

**Roles:** admin, manager

### 5.4 Update Order

**Endpoint:** `PUT /api/v1/orders/{id}`

**Roles:** admin, manager

### 5.5 Delete Order

**Endpoint:** `DELETE /api/v1/orders/{id}`

**Roles:** admin, manager

### 5.6 Start Order

**Endpoint:** `POST /api/v1/orders/{id}/start`

Transitions the order to "InProgress" status. Creates a worklog automatically on completion.

**Response (200):** The updated order with `status: "InProgress"`.

**Roles:** admin, manager, worker

### 5.7 Complete Order

**Endpoint:** `POST /api/v1/orders/{id}/complete`

Transitions the order to "Completed" status. Generates a worklog entry with duration calculation and triggers follow-up orders from workflow rules.

**Response (200):** The updated order with `status: "Completed"`.

**Roles:** admin, manager, worker

### 5.8 My Tasks

**Endpoint:** `GET /api/v1/orders/my-tasks`

Returns a list of orders assigned to the current user that are in progress.

**Response (200):** Array of `MyTask` objects:
```json
[
  {
    "order_id": "UUID",
    "label": "Spring Plowing",
    "order_type": "Plowing",
    "started_at": "ISO8601",
    "site_ids": ["UUID"]
  }
]
```

### 5.9 Start Task For Worker

**Endpoint:** `POST /api/v1/tasks/{id}/start-for-worker`

Sets the worker task status to "Started" for the current user. Creates a new worker-task-status entry if one doesn't exist.

**Response (200):** `{"status": "started"}` or `{"status": "created"}`

**Roles:** admin, manager, worker

### 5.10 Stop Task For Worker

**Endpoint:** `POST /api/v1/tasks/{id}/stop-for-worker`

Sets the worker task status to "Stopped".

**Roles:** admin, manager, worker

---

## 6. Users

Base path: `/api/v1/users`

### 6.1 List Users

**Endpoint:** `GET /api/v1/users?page=1&per_page=50`

**Response (200):** Paginated list of UserDto objects.

**Roles:** admin, manager

### 6.2 Get User

**Endpoint:** `GET /api/v1/users/{id}`

Workers can only view their own profile.

**Roles:** admin, manager, worker (own profile only)

### 6.3 Create User

**Endpoint:** `POST /api/v1/users`

**Request Body:**
```json
{
  "firstname": "John",
  "lastname": "Farmer",
  "email": "john@example.com",
  "password": "secure_password_123",
  "roles": ["worker"],
  "language": "en",
  "assigned_site_ids": ["UUID"]
}
```

**Roles:** admin only

### 6.4 Update User

**Endpoint:** `PUT /api/v1/users/{id}`

Partial update. Workers can only update their own profile (non-role fields).

**Request Body:**
```json
{
  "firstname": "John",
  "lastname": "Updated",
  "language": "de",
  "internal_cost_per_hour": 25.50
}
```

**Roles:** admin, manager, worker (own profile)

### 6.5 Delete User

**Endpoint:** `DELETE /api/v1/users/{id}`

**Roles:** admin only

---

## 7. Workforce

Base path: `/api/v1/workforce`

### 7.1 Workers

| Method   | Endpoint                          | Description                    | Auth Role |
|----------|-----------------------------------|--------------------------------|-----------|
| GET      | `/workforce/workers?page=&per_page=` | List workers               | admin, manager, viewer |
| GET      | `/workforce/workers/{id}`         | Get worker details             | admin, manager, worker (own) |
| POST     | `/workforce/workers`              | Create worker                  | admin, manager |
| PUT      | `/workforce/workers/{id}`         | Update worker                  | admin, manager |
| DELETE   | `/workforce/workers/{id}`         | Delete worker                  | admin, manager |

### 7.2 Work Logs

| Method   | Endpoint                          | Description                    | Auth Role |
|----------|-----------------------------------|--------------------------------|-----------|
| GET      | `/workforce/logs?page=&per_page=` | List work logs                 | admin, manager, viewer |
| GET      | `/workforce/logs/{id}`            | Get work log                   | admin, manager, worker (own) |
| POST     | `/workforce/logs`                 | Create work log                | admin, manager |
| PUT      | `/workforce/logs/{id}`            | Update work log                | admin, manager |
| DELETE   | `/workforce/logs/{id}`            | Delete work log                | admin, manager |

### 7.3 Worker Locations

| Method   | Endpoint                          | Description                    | Auth Role |
|----------|-----------------------------------|--------------------------------|-----------|
| GET      | `/workforce/locations`            | Get latest locations           | admin, manager, viewer |
| POST     | `/workforce/locations`            | Report location (GPS)          | admin, manager, worker |

Location reporting triggers spatial presence events and auto-start/stop for `AutoPresence` execution policy orders.

### 7.4 Worker Task Status

| Method   | Endpoint                                           | Description                     | Auth Role |
|----------|----------------------------------------------------|---------------------------------|-----------|
| GET      | `/workforce/tasks/{id}/status`                     | Get per-worker statuses         | admin, manager |
| POST     | `/workforce/tasks/{id}/status`                     | Create worker status            | admin, manager, worker |
| GET      | `/workforce/tasks/{id}/status/{worker_id}`         | Get specific worker status      | admin, manager |
| PUT      | `/workforce/tasks/{id}/status/{worker_id}`         | Update worker status            | admin, manager, worker (own) |
| GET      | `/workforce/tasks/{id}/status/aggregate`           | Get aggregated status           | admin, manager, worker |

---

## 8. Equipment

Base path: `/api/v1/equipments`

| Method   | Endpoint                    | Description              | Auth Role |
|----------|-----------------------------|--------------------------|-----------|
| GET      | `/equipments?page=&per_page=` | List equipment         | all       |
| GET      | `/equipments/{id}`         | Get equipment details    | all       |
| POST     | `/equipments`              | Create equipment         | admin, manager |
| PUT      | `/equipments/{id}`         | Update equipment         | admin, manager |
| DELETE   | `/equipments/{id}`         | Delete equipment         | admin, manager |

**EquipmentDto fields:**
- id, tenant_id, name, equipment_type, serial_number, model, manufacturer
- purchase_date, warranty_end, cost, is_active
- last_service_date, next_service_date, service_interval_hours
- assigned_site_id, assigned_worker_id, metadata (JSON)

---

## 9. Tasks

Base path: `/api/v1/tasks`

| Method   | Endpoint             | Description           | Auth Role |
|----------|----------------------|-----------------------|-----------|
| GET      | `/tasks?page=&per_page=` | List tasks          | admin, manager |
| GET      | `/tasks/{id}`        | Get task details      | admin, manager, worker (assigned) |
| POST     | `/tasks`             | Create task           | admin, manager, worker |
| PUT      | `/tasks/{id}`        | Update task           | admin, manager, worker (assigned) |
| DELETE   | `/tasks/{id}`        | Delete task           | admin, manager |

**TaskDataDto fields:**
- id, tenant_id, order_id, label, description, site_id
- assigned_worker_ids (array), type, priority, estimated_duration_minutes
- actual_duration_minutes, started_at, completed_at, status
- materials_used (JSON), gps_points (array), custom_fields (JSON)

---

## 10. Weather & Phenology

Base path: `/api/v1/weather`

### 10.1 Weather Stations

| Method   | Endpoint                    | Description              | Auth Role |
|----------|-----------------------------|--------------------------|-----------|
| GET      | `/weather/stations?page=&per_page=` | List stations   | all       |
| GET      | `/weather/stations/{id}`   | Get station              | all       |
| POST     | `/weather/stations`        | Create station           | admin, manager |
| PUT      | `/weather/stations/{id}`   | Update station           | admin, manager |
| DELETE   | `/weather/stations/{id}`   | Delete station           | admin, manager |

### 10.2 Weather Data

| Method   | Endpoint             | Description            | Auth Role |
|----------|----------------------|------------------------|-----------|
| GET      | `/weather/data?page=&per_page=` | List weather data | all       |
| GET      | `/weather/data/{id}`   | Get weather record     | all       |
| POST     | `/weather/data`        | Create weather data    | admin, manager |
| PUT      | `/weather/data/{id}`   | Update weather data    | admin, manager |
| DELETE   | `/weather/data/{id}`   | Delete weather data    | admin, manager |

### 10.3 Phenology Records

| Method   | Endpoint                 | Description        | Auth Role |
|----------|--------------------------|--------------------|-----------|
| GET      | `/weather/phenology?page=&per_page=` | List phenology | all       |
| GET      | `/weather/phenology/{id}` | Get phenology record | all     |
| POST     | `/weather/phenology`     | Create phenology   | admin, manager |
| PUT      | `/weather/phenology/{id}` | Update phenology  | admin, manager |
| DELETE   | `/weather/phenology/{id}` | Delete phenology  | admin, manager |

Phenology records track BBCH growth stages for sites.

---

## 11. Compliance

Base path: `/api/v1/compliance`

### 11.1 Checklists

| Method   | Endpoint                                      | Description               | Auth Role |
|----------|-----------------------------------------------|---------------------------|-----------|
| GET      | `/compliance/checklists?page=&per_page=`      | List checklists           | all       |
| GET      | `/compliance/checklists/{id}`                 | Get checklist             | all       |
| POST     | `/compliance/checklists`                      | Create checklist          | admin, manager |
| PUT      | `/compliance/checklists/{id}`                 | Update checklist          | admin, manager |
| DELETE   | `/compliance/checklists/{id}`                 | Delete checklist          | admin, manager |
| GET      | `/compliance/checklists/find-by-site/{site_id}` | Find by site           | all       |
| GET      | `/compliance/checklists/find-by-type/{type}` | Find by type (GAP, Organic, GlobalGAP, HACCP) | all |

### 11.2 Fertilizer Records

| Method   | Endpoint                              | Description              | Auth Role |
|----------|---------------------------------------|--------------------------|-----------|
| GET      | `/compliance/fertilizer?page=&per_page=` | List fertilizer records | all       |
| GET      | `/compliance/fertilizer/{id}`         | Get fertilizer record    | all       |
| PUT      | `/compliance/fertilizer/{id}`         | Update fertilizer record | admin, manager |
| DELETE   | `/compliance/fertilizer/{id}`         | Delete fertilizer record | admin, manager |

### 11.3 Plant Protection Records

| Method   | Endpoint                                      | Description              | Auth Role |
|----------|-----------------------------------------------|--------------------------|-----------|
| GET      | `/compliance/plant-protection?page=&per_page=` | List records            | all       |
| GET      | `/compliance/plant-protection/{id}`            | Get record              | all       |
| POST     | `/compliance/plant-protection`                 | Create record           | admin, manager |
| PUT      | `/compliance/plant-protection/{id}`            | Update record           | admin, manager |
| DELETE   | `/compliance/plant-protection/{id}`            | Delete record           | admin, manager |

### 11.4 Applicator Licenses

| Method   | Endpoint                                          | Description              | Auth Role |
|----------|---------------------------------------------------|--------------------------|-----------|
| GET      | `/compliance/applicator-licenses`                 | List licenses            | all       |
| GET      | `/compliance/applicator-licenses/{id}`            | Get license              | all       |
| POST     | `/compliance/applicator-licenses`                 | Create license           | admin, manager |
| PUT      | `/compliance/applicator-licenses/{id}`            | Update license           | admin, manager |
| DELETE   | `/compliance/applicator-licenses/{id}`            | Delete license           | admin, manager |

### 11.5 Audit Logs

| Method   | Endpoint                          | Description        | Auth Role |
|----------|-----------------------------------|--------------------|-----------|
| GET      | `/compliance/audit-logs?page=`    | List audit entries | admin, manager |

---

## 12. Specialized Crops

Base path: `/api/v1/specialized` and `/api/v1/predict`

### 12.1 List Specialized Sites

**Endpoint:** `GET /api/v1/specialized/sites?site_type=Main&crop_type=Wine&per_page=50`

Lists sites filtered by site_type and/or crop_type.

### 12.2 Vineyards

| Method   | Endpoint                              | Description      | Auth Role |
|----------|---------------------------------------|------------------|-----------|
| GET      | `/specialized/vineyards?page=&per_page=` | List vineyards | all       |
| GET      | `/specialized/vineyards/{id}`         | Get vineyard     | all       |
| POST     | `/specialized/vineyards`              | Create vineyard  | admin, manager |
| PUT      | `/specialized/vineyards/{id}`         | Update vineyard  | admin, manager |
| DELETE   | `/specialized/vineyards/{id}`         | Delete vineyard  | admin, manager |
| GET      | `/specialized/vineyards/site/{site_id}` | List vineyards by site | all |

### 12.3 Calculations

| Method | Endpoint                    | Description                    | Auth Role |
|--------|-----------------------------|--------------------------------|-----------|
| POST   | `/calculate/material`       | Material calculation (tank volumes, sprayer calibration) | all |
| POST   | `/calculate/water-rate`     | Water rate calculation (L/ha)  | all       |
| GET    | `/predict/harvest?site_id=UUID&target_bbch=89` | Harvest prediction | all |
| POST   | `/specialized/profitability` | Profitability analysis (yield x price vs costs) | all |

### 12.4 Olive Groves

| Method   | Endpoint                       | Description              | Auth Role |
|----------|--------------------------------|--------------------------|-----------|
| GET      | `/specialized/olive-groves?page=` | List olive groves     | all       |
| GET      | `/specialized/olive-groves/{id}`  | Get olive grove       | all       |
| POST     | `/specialized/olive-groves`       | Create olive grove    | admin, manager |
| PUT      | `/specialized/olive-groves/{id}`  | Update olive grove    | admin, manager |
| DELETE   | `/specialized/olive-groves/{id}`  | Delete olive grove    | admin, manager |

### 12.5 Olive Oil Records

| Method   | Endpoint                       | Description              | Auth Role |
|----------|--------------------------------|--------------------------|-----------|
| GET      | `/specialized/olive-oil-records?page=` | List records        | all       |
| GET      | `/specialized/olive-oil-records/{id}`  | Get record          | all       |
| POST     | `/specialized/olive-oil-records`       | Create record         | admin, manager |
| PUT      | `/specialized/olive-oil-records/{id}`  | Update record         | admin, manager |
| DELETE   | `/specialized/olive-oil-records/{id}`  | Delete record         | admin, manager |

**Note:** Olive endpoints are registered in the `specialized` module and follow the same CRUD pattern as vineyards.

---

## 13. Water Management

Base path: `/api/v1/water`

### 13.1 Water Sources

| Method   | Endpoint                    | Description               | Auth Role |
|----------|-----------------------------|---------------------------|-----------|
| GET      | `/water/sources?page=`      | List water sources        | all       |
| GET      | `/water/sources/{id}`       | Get water source          | all       |
| POST     | `/water/sources`            | Create water source       | admin, manager |
| PUT      | `/water/sources/{id}`       | Update water source       | admin, manager |
| DELETE   | `/water/sources/{id}`       | Delete water source       | admin, manager |

### 13.2 Water Usage

| Method   | Endpoint                  | Description            | Auth Role |
|----------|---------------------------|------------------------|-----------|
| GET      | `/water/usage?page=`      | List water usage       | all       |
| GET      | `/water/usage/{id}`       | Get usage record       | all       |
| POST     | `/water/usage`            | Create usage record    | admin, manager |
| PUT      | `/water/usage/{id}`       | Update usage record    | admin, manager |
| DELETE   | `/water/usage/{id}`       | Delete usage record    | admin, manager |

### 13.3 Water Quotas

| Method   | Endpoint                  | Description            | Auth Role |
|----------|---------------------------|------------------------|-----------|
| GET      | `/water/quotas?page=`     | List quotas            | all       |
| GET      | `/water/quotas/{id}`      | Get quota              | all       |
| POST     | `/water/quotas`           | Create quota           | admin, manager |
| PUT      | `/water/quotas/{id}`      | Update quota           | admin, manager |
| DELETE   | `/water/quotas/{id}`      | Delete quota           | admin, manager |

---

## 14. Harvest Logistics

Base path: `/api/v1/harvest`

### 14.1 Harvest Seasons

| Method   | Endpoint                    | Description            | Auth Role |
|----------|-----------------------------|------------------------|-----------|
| GET      | `/harvest/seasons?page=`    | List seasons           | all       |
| GET      | `/harvest/seasons/{id}`     | Get season             | all       |
| POST     | `/harvest/seasons`          | Create season          | admin, manager |
| PUT      | `/harvest/seasons/{id}`     | Update season          | admin, manager |
| DELETE   | `/harvest/seasons/{id}`     | Delete season          | admin, manager |

### 14.2 Harvest Lots

| Method   | Endpoint                  | Description          | Auth Role |
|----------|---------------------------|----------------------|-----------|
| GET      | `/harvest/lots?page=`     | List lots            | all       |
| GET      | `/harvest/lots/{id}`      | Get lot              | all       |
| POST     | `/harvest/lots`           | Create lot           | admin, manager |
| PUT      | `/harvest/lots/{id}`      | Update lot           | admin, manager |
| DELETE   | `/harvest/lots/{id}`      | Delete lot           | admin, manager |

### 14.3 Deliveries

| Method   | Endpoint                    | Description          | Auth Role |
|----------|-----------------------------|----------------------|-----------|
| GET      | `/harvest/deliveries?page=` | List deliveries      | all       |
| GET      | `/harvest/deliveries/{id}`  | Get delivery         | all       |
| POST     | `/harvest/deliveries`       | Create delivery      | admin, manager |
| PUT      | `/harvest/deliveries/{id}`  | Update delivery      | admin, manager |
| DELETE   | `/harvest/deliveries/{id}`  | Delete delivery      | admin, manager |

### 14.4 Cold Chain Logs

| Method   | Endpoint                    | Description          | Auth Role |
|----------|-----------------------------|----------------------|-----------|
| GET      | `/harvest/cold-chain?page=` | List cold chain logs | all       |
| GET      | `/harvest/cold-chain/{id}`  | Get cold chain log   | all       |
| POST     | `/harvest/cold-chain`       | Create cold chain log | admin, manager |
| PUT      | `/harvest/cold-chain/{id}`  | Update cold chain log | admin, manager |
| DELETE   | `/harvest/cold-chain/{id}`  | Delete cold chain log | admin, manager |

---

## 15. Livestock

Base path: `/api/v1/animals`

### 15.1 Animals

| Method   | Endpoint                    | Description          | Auth Role |
|----------|-----------------------------|----------------------|-----------|
| GET      | `/animals?page=`            | List animals         | all       |
| GET      | `/animals/{id}`             | Get animal           | all       |
| POST     | `/animals`                  | Create animal        | admin, manager |
| PUT      | `/animals/{id}`             | Update animal        | admin, manager |
| DELETE   | `/animals/{id}`             | Delete animal        | admin, manager |

### 15.2 Animal Treatments

**Endpoint:** `POST /animals/{id}/treatments`

Add a health treatment record to an animal.

**Request Body:**
```json
{
  "date": "2024-03-15T10:00:00Z",
  "treatment_type": "Vaccination",
  "medicine": "Bovilis",
  "dosage": "2ml",
  "notes": "Annual vaccine",
  "administered_by": "Dr. Smith"
}
```

### 15.3 Grazing Records

**Endpoint:** `POST /animals/{id}/grazing`

Add a grazing record for an animal.

**Request Body:**
```json
{
  "date": "2024-03-15T08:00:00Z",
  "pasture_id": "UUID",
  "duration_hours": 4.5,
  "notes": "Morning grazing in North Field"
}
```

---

## 16. Finance

Base path: `/api/v1/finance`

### 16.1 PAC Applications (EU Agricultural Fund)

| Method   | Endpoint                              | Description          | Auth Role |
|----------|---------------------------------------|----------------------|-----------|
| GET      | `/finance/pac-applications?page=`     | List PAC applications | all       |
| GET      | `/finance/pac-applications/{id}`      | Get PAC application  | all       |
| POST     | `/finance/pac-applications`           | Create PAC application | admin, manager |
| PUT      | `/finance/pac-applications/{id}`      | Update PAC application | admin, manager |
| DELETE   | `/finance/pac-applications/{id}`      | Delete PAC application | admin, manager |

### 16.2 Cost Centers

| Method   | Endpoint                              | Description        | Auth Role |
|----------|---------------------------------------|--------------------|-----------|
| GET      | `/finance/cost-centers?page=`         | List cost centers  | all       |
| GET      | `/finance/cost-centers/{id}`          | Get cost center    | all       |
| POST     | `/finance/cost-centers`               | Create cost center | admin, manager |
| PUT      | `/finance/cost-centers/{id}`          | Update cost center | admin, manager |
| DELETE   | `/finance/cost-centers/{id}`          | Delete cost center | admin, manager |

### 16.3 Financial Records

| Method   | Endpoint                                | Description          | Auth Role |
|----------|-----------------------------------------|----------------------|-----------|
| GET      | `/finance/financial-records?page=`      | List financial records | all       |
| GET      | `/finance/financial-records/{id}`       | Get financial record  | all       |
| POST     | `/finance/financial-records`            | Create financial record | admin, manager |
| PUT      | `/finance/financial-records/{id}`       | Update financial record | admin, manager |
| DELETE   | `/finance/financial-records/{id}`       | Delete financial record | admin, manager |

---

## 17. IoT Device Registry

Base path: `/api/v1/iot/devices`

### 17.1 List Devices

**Endpoint:** `GET /api/v1/iot/devices?tenant_id=UUID&site_id=UUID&status=online`

Lists all IoT devices for the tenant. Admins can query other tenants.

**Response (200):**
```json
{
  "devices": [
    {
      "device_id": "sensor-001",
      "device_type": "soil-sensor",
      "tenant_id": "UUID",
      "site_id": "UUID",
      "capabilities": [
        {
          "type": "Temperature",
          "unit": "°C",
          "min_value": -40.0,
          "max_value": 85.0,
          "precision": 1
        }
      ],
      "metadata": {},
      "status": "Online",
      "firmware_version": "1.2.3",
      "battery_level": 85.5,
      "signal_strength": -65,
      "error_message": null,
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-03-15T14:22:00Z",
      "last_seen": "2024-03-15T14:22:00Z"
    }
  ],
  "total": 1
}
```

### 17.2 Get Device

**Endpoint:** `GET /api/v1/iot/devices/{device_id}`

### 17.3 Create Device

**Endpoint:** `POST /api/v1/iot/devices`

**Request Body:**
```json
{
  "device_id": "sensor-001",
  "device_type": "soil-sensor",
  "tenant_id": "UUID",
  "site_id": "UUID",
  "capabilities": [
    {"type": "Temperature", "unit": "°C"}
  ],
  "metadata": {"manufacturer": "Acme", "model": "TS-100"}
}
```

**Response (201):** The created IoTDeviceResponse.

**Roles:** admin, manager

### 17.4 Update Device

**Endpoint:** `PUT /api/v1/iot/devices/{device_id}`

Partial update (all fields optional).

### 17.5 Delete Device

**Endpoint:** `DELETE /api/v1/iot/devices/{device_id}`

Returns 204 No Content on success.

**Roles:** admin only

### 17.6 Device Telemetry

**Endpoint:** `GET /api/v1/iot/devices/{device_id}/telemetry?limit=100&since=ISO8601`

Returns telemetry data for a device.

**Response (200):**
```json
{
  "device_id": "sensor-001",
  "timestamp": "2024-03-15T14:22:00Z",
  "measurements": [
    {
      "type": "Temperature",
      "value": 22.5,
      "unit": "°C"
    }
  ]
}
```

### 17.7 Send Command

**Endpoint:** `POST /api/v1/iot/devices/{device_id}/command`

Send a command to an online device.

**Request Body:**
```json
{
  "command_type": "calibrate",
  "payload": {"target_temp": 22.0}
}
```

**Response (200):**
```json
{
  "command_id": "UUID",
  "device_id": "sensor-001",
  "command_type": "calibrate",
  "status": "Sent",
  "response_payload": null,
  "requested_at": "2024-03-15T14:22:00Z",
  "completed_at": null
}
```

### 17.8 Home Assistant Discovery

**Endpoint:** `GET /api/v1/iot/devices/{device_id}/ha-discovery`

Returns Home Assistant auto-discovery configuration payloads for all capabilities of the device.

```

### 17.9 IoT Capability Types

| Type             | Unit     | Default Range          |
|------------------|----------|------------------------|
| Temperature      | °C       | -40.0 to 85.0          |
| Humidity         | %        | 0.0 to 100.0           |
| SoilMoisture     | %        | 0.0 to 100.0           |
| Light            | lux      | 0.0 to 200000.0        |
| Gps                | degrees  | -180.0 to 180.0        |
| BatteryLevel     | %        | 0.0 to 100.0           |
| SignalStrength   | dBm      | —                      |
| ActuatorControl  | —        | —                      |
| FirmwareUpdate   | —        | —                      |

---

## 18. SIGPAC (Spain) Parcel API

Base path: `/api/v1/sigpac`

### 18.1 List SIGPAC Parcels

**Endpoint:** `GET /api/v1/sigpac/parcels?province=41&municipality=123&parcel=456&per_page=50`

Query official Spanish SIGPAC parcel reference data by province, municipality, aggregate, zone, polygon, parcel, enclosure, or full sigpac_reference.

**Response (200):**
```json
{
  "data": [
    {
      "id": "UUID",
      "tenant_id": "UUID",
      "sigpac_reference": "02012345678901234567",
      "province": 2,
      "municipality": 123,
      "aggregate": 456,
      "zone": 789,
      "polygon": 012,
      "parcel": 345,
      "enclosure": 678,
      "usage_code": "AGR",
      "usage_description": "Agricultural land",
      "geometry": { "type": "Polygon", "coordinates": [...] },
      "area_hectares": 10.5,
      "official_area_ha": 10.4,
      "source_dataset": "SIGPAC",
      "source_year": 2024,
      "created_at": "2024-03-15T10:30:00Z",
      "updated_at": "2024-03-15T10:30:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "per_page": 50,
  "total_pages": 1
}
```

### 18.2 Get SIGPAC Parcel

**Endpoint:** `GET /api/v1/sigpac/parcels/{id}`

### 18.3 Search Near Point

**Endpoint:** `GET /api/v1/sigpac/parcels/search/near-point?lng=-3.7&lat=40.4&radius_m=1000`

Spatial query returning up to 100 parcels within a radius (default 1000m, max 10000m).

---

## 19. Reporting & Export

Base path: `/api/v1/reporting`

### 19.1 Export Orders to Excel

**Endpoint:** `GET /api/v1/reporting/export/orders/excel`

Downloads orders as an Excel (.xlsx) file. The export is processed asynchronously via NATS by the reporting-service worker.

### 19.2 Export Sites to GeoJSON

**Endpoint:** `GET /api/v1/reporting/export/sites/geojson`

Returns site boundaries as a GeoJSON FeatureCollection.

### 19.3 Export PAC SIP Report

**Endpoint:** `GET /api/v1/reporting/export/pac/sip`

Generates a PAC-SIP compliant Excel report for EU subsidy applications.

### 19.4 Export Veterinary Report

**Endpoint:** `GET /api/v1/reporting/export/veterinary`

Generates a veterinary report Excel file for livestock treatments.

---

## 20. Settings

Base path: `/api/v1/settings`

### 20.1 LPIS Provider Settings

**Endpoint:** `GET /api/v1/settings/lpis`

Loads LPIS provider configuration from `config/lpis-providers.toml`.

**Response (200):**
```json
{
  "providers": {
    "ES": {
      "base_url": "https://sigpac.example.com/wfs",
      "timeout_seconds": 30,
      "cache_ttl_seconds": 3600,
      "rate_limit_requests_per_second": 10,
      "rate_limit_burst_size": 20,
      "enabled": true
    },
    "NL": { ... }
  },
  "cache_backend": "memory",
  "cache_default_ttl_seconds": 3600,
  "cache_max_entries": 10000
}
```

**Endpoint:** `PUT /api/v1/settings/lpis`

Updates provider configuration. Requires `restart_required: true` in response — changes take effect on service restart.

**Endpoint:** `GET /api/v1/settings/lpis/providers`

Lists all available provider configurations with defaults for all 8 supported countries.

**Roles:** admin only

---

## 21. System

Base path: `/api/v1/system`

### 21.1 System Status

**Endpoint:** `GET /api/v1/system/status`

Returns system status, version, and uptime information.

### 21.2 Initial Setup

**Endpoint:** `POST /api/v1/system/setup`

Performs initial tenant and admin user setup. Only available when no tenants exist yet.

### 21.3 Delete Tenant

**Endpoint:** `DELETE /api/v1/system/tenant`

Deletes a tenant and all associated data. **Irreversible.** Use with extreme caution.

**Roles:** admin only (system-level)

---

## 22. Enums & Domain Types

### 22.1 SiteType

| Value      | Description              |
|------------|--------------------------|
| Main       | Primary production site  |
| Secondary  | Secondary/support site   |
| Storage    | Storage facility         |
| Processing | Processing facility      |

### 22.2 CropType

`Cereal`, `Vegetable`, `Fruit`, `Oil`, `Vine`, `Olive`, `Other`, `Fallow`, `Meadow`, `Hay`, `Industrial`

### 22.3 OrderType

`Plowing`, `Sowing`, `Fertilizing`, `Spraying`, `Weeding`, `Harvesting`, `Pruning`, `Irrigation`, `Scouting`, `Other`

### 22.4 OrderStatus

`Draft`, `Planned`, `InProgress`, `Paused`, `Completed`, `Cancelled`

### 22.5 BbchStage

`00` — No growth  
`10` — First leaf  
`20` — Tillering  
`30` — Stem elongation  
`50` — Heading  
`60` — Flowering  
`70` — Fruit development  
`80` — Fruit filling  
`89` — Before harvest  
`90` — Harvest

### 22.6 UserRole

`admin`, `manager`, `worker`, `viewer`

### 22.7 ChecklistType

`GAP`, `Organic`, `GlobalGAP`, `HACCP`

### 22.8 ComplianceStatus

`Pending`, `InProgress`, `Completed`, `Failed`

### 22.9 WaterSourceType

`Well`, `Surface`, `Rainwater`, `Municipal`, `Other`

### 22.10 IrrigationMethod

`Drip`, `Sprinkler`, `Flood`, `CenterPivot`, `Other`

### 22.11 AnimalSpecies

`Cattle`, `Sheep`, `Goat`, `Pig`, `Chicken`, `Other`

### 22.12 AnimalStatus

`Healthy`, `Sick`, `Recovering`, `Quarantined`, `Deceased`

### 22.13 PACStatus

`Draft`, `Submitted`, `UnderReview`, `Approved`, `Rejected`, `Paid`

### 22.14 FinancialRecordType

`Income`, `Expense`, `Transfer`

### 22.15 CostCenterType

`Direct`, `Indirect`, `Overhead`, `Project`

### 22.16 DeviceStatusDto

`Online`, `Offline`, `Maintenance`, `Updating`, `Error`

### 22.17 CommandStatus

`Sent`, `Acknowledged`, `Completed`, `Failed`, `Timeout`

### 22.18 TaskAutomationState

`Enabled`, `Disabled`, `Suspended`

### 22.19 RecurrenceCadence

`Daily`, `Weekly`, `BiWeekly`, `Monthly`, `Yearly`

### 22.20 TaskExecutionMode

`Manual`, `AutoPresence`, `Scheduled`

### 22.21 LpisCountry

`ES`, `PT`, `FR`, `IT`, `NL`, `DE`, `PL`, `AT`

### 22.22 CacheBackend

`Memory`, `Redis`

---

## 23. API Endpoints Summary

| Module           | Endpoints | Auth Required | Tags          |
|------------------|-----------|---------------|---------------|
| Health           | 1         | No            | system        |
| Auth             | 2         | No (login)    | auth          |
| System           | 3         | Yes           | system        |
| Sites            | 8         | Yes           | sites         |
| Orders           | 8         | Yes           | orders, tasks |
| Users            | 5         | Yes           | users         |
| Workforce        | 15        | Yes           | workforce     |
| Equipment        | 5         | Yes           | equipment     |
| Tasks            | 5         | Yes           | tasks         |
| Weather          | 10        | Yes           | weather       |
| Compliance       | 18        | Yes           | compliance    |
| Specialized      | 7+        | Yes           | specialized   |
| Water            | 15        | Yes           | water         |
| Harvest          | 19        | Yes           | harvest       |
| Livestock        | 5         | Yes           | livestock     |
| Finance          | 15        | Yes           | finance       |
| IoT              | 8         | Yes           | iot           |
| SIGPAC           | 3         | Yes           | sigpac        |
| Reporting        | 4         | Yes           | reporting     |
| Settings         | 3         | Yes           | settings      |
| **Total**        | **140+**  | **Most**      | 21 tags       |

---

## 24. OpenAPI / Swagger UI

- OpenAPI 3.0 spec: `GET /api-docs/openapi.json`
- Swagger UI: `GET /swagger-ui/`
- All endpoints are annotated with `#[utoipa::path(...)]` and registered in `ApiDoc`

---

## 25. Error Codes

| HTTP Status | Code              | Description                        |
|-------------|-------------------|------------------------------------|
| 200         | OK                | Success                           |
| 201         | Created           | Resource created                  |
| 204         | No Content        | Resource deleted (no body)        |
| 400         | BadRequest        | Validation failed                 |
| 401         | Unauthorized      | Missing or invalid JWT            |
| 403         | Forbidden         | Insufficient role permissions     |
| 404         | NotFound          | Resource not found                |
| 409         | Conflict          | Duplicate resource (e.g., device_id) |
| 429         | TooManyRequests   | Rate limit exceeded               |
| 500         | InternalError     | Internal server error             |
