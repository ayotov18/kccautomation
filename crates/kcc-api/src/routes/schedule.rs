use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn schedule_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_schedule))
        .route("/{id}", get(get_schedule))
        .route("/{id}/activities", post(create_activity))
        .route("/activities/{id}", put(update_activity).delete(delete_activity))
        .route("/{id}/cpm", post(calculate_cpm))
}

// ── Models ──────────────────────────────────────────

#[derive(Serialize, sqlx::FromRow)]
struct Schedule {
    id: Uuid,
    project_id: Uuid,
    name: String,
    description: Option<String>,
    start_date: Option<chrono::NaiveDate>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
struct Activity {
    id: Uuid,
    schedule_id: Uuid,
    code: String,
    name: String,
    description: Option<String>,
    duration_days: f64,
    predecessors: Option<serde_json::Value>,
    planned_start: Option<chrono::NaiveDate>,
    planned_finish: Option<chrono::NaiveDate>,
    actual_start: Option<chrono::NaiveDate>,
    actual_finish: Option<chrono::NaiveDate>,
    percent_complete: f64,
    sort_order: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
struct ScheduleWithActivities {
    schedule: Schedule,
    activities: Vec<Activity>,
}

// ── DTOs ────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateScheduleRequest {
    project_id: Uuid,
    name: String,
    description: Option<String>,
    start_date: Option<chrono::NaiveDate>,
}

#[derive(Deserialize)]
struct CreateActivityRequest {
    code: String,
    name: String,
    description: Option<String>,
    duration_days: f64,
    predecessors: Option<serde_json::Value>,
    planned_start: Option<chrono::NaiveDate>,
    planned_finish: Option<chrono::NaiveDate>,
    sort_order: Option<i32>,
}

#[derive(Deserialize)]
struct UpdateActivityRequest {
    code: Option<String>,
    name: Option<String>,
    description: Option<String>,
    duration_days: Option<f64>,
    predecessors: Option<serde_json::Value>,
    planned_start: Option<chrono::NaiveDate>,
    planned_finish: Option<chrono::NaiveDate>,
    actual_start: Option<chrono::NaiveDate>,
    actual_finish: Option<chrono::NaiveDate>,
    percent_complete: Option<f64>,
    sort_order: Option<i32>,
}

// ── Handlers ────────────────────────────────────────

async fn create_schedule(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<CreateScheduleRequest>,
) -> Result<Json<Schedule>, ApiError> {
    verify_project_owner(&state, body.project_id, user_id).await?;

    let schedule = sqlx::query_as::<_, Schedule>(
        r#"INSERT INTO schedules (id, project_id, name, description, start_date, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, now(), now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(body.project_id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(body.start_date)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(schedule))
}

async fn get_schedule(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<ScheduleWithActivities>, ApiError> {
    let schedule = sqlx::query_as::<_, Schedule>(
        r#"SELECT s.* FROM schedules s
           JOIN projects p ON p.id = s.project_id
           WHERE s.id = $1 AND p.user_id = $2"#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Schedule not found".into()))?;

    let activities = sqlx::query_as::<_, Activity>(
        "SELECT * FROM schedule_activities WHERE schedule_id = $1 ORDER BY sort_order, code",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(ScheduleWithActivities { schedule, activities }))
}

async fn create_activity(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(schedule_id): Path<Uuid>,
    Json(body): Json<CreateActivityRequest>,
) -> Result<Json<Activity>, ApiError> {
    verify_schedule_owner(&state, schedule_id, user_id).await?;

    let sort_order = body.sort_order.unwrap_or(0);

    let activity = sqlx::query_as::<_, Activity>(
        r#"INSERT INTO schedule_activities
           (id, schedule_id, code, name, description, duration_days, predecessors,
            planned_start, planned_finish, actual_start, actual_finish, percent_complete,
            sort_order, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NULL, NULL, 0.0, $10, now(), now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(schedule_id)
    .bind(&body.code)
    .bind(&body.name)
    .bind(&body.description)
    .bind(body.duration_days)
    .bind(&body.predecessors)
    .bind(body.planned_start)
    .bind(body.planned_finish)
    .bind(sort_order)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(activity))
}

async fn update_activity(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(activity_id): Path<Uuid>,
    Json(body): Json<UpdateActivityRequest>,
) -> Result<Json<Activity>, ApiError> {
    let schedule_id = get_schedule_id_for_activity(&state, activity_id).await?;
    verify_schedule_owner(&state, schedule_id, user_id).await?;

    let current = sqlx::query_as::<_, Activity>(
        "SELECT * FROM schedule_activities WHERE id = $1",
    )
    .bind(activity_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Activity not found".into()))?;

    let code = body.code.unwrap_or(current.code);
    let name = body.name.unwrap_or(current.name);
    let description = body.description.or(current.description);
    let duration_days = body.duration_days.unwrap_or(current.duration_days);
    let predecessors = body.predecessors.or(current.predecessors);
    let planned_start = body.planned_start.or(current.planned_start);
    let planned_finish = body.planned_finish.or(current.planned_finish);
    let actual_start = body.actual_start.or(current.actual_start);
    let actual_finish = body.actual_finish.or(current.actual_finish);
    let percent_complete = body.percent_complete.unwrap_or(current.percent_complete);
    let sort_order = body.sort_order.unwrap_or(current.sort_order);

    let activity = sqlx::query_as::<_, Activity>(
        r#"UPDATE schedule_activities
           SET code = $1, name = $2, description = $3, duration_days = $4,
               predecessors = $5, planned_start = $6, planned_finish = $7,
               actual_start = $8, actual_finish = $9, percent_complete = $10,
               sort_order = $11, updated_at = now()
           WHERE id = $12
           RETURNING *"#,
    )
    .bind(&code)
    .bind(&name)
    .bind(&description)
    .bind(duration_days)
    .bind(&predecessors)
    .bind(planned_start)
    .bind(planned_finish)
    .bind(actual_start)
    .bind(actual_finish)
    .bind(percent_complete)
    .bind(sort_order)
    .bind(activity_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(activity))
}

async fn delete_activity(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let schedule_id = get_schedule_id_for_activity(&state, activity_id).await?;
    verify_schedule_owner(&state, schedule_id, user_id).await?;

    let result = sqlx::query("DELETE FROM schedule_activities WHERE id = $1")
        .bind(activity_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Activity not found".into()));
    }

    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn calculate_cpm(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(schedule_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    verify_schedule_owner(&state, schedule_id, user_id).await?;

    let activities = sqlx::query_as::<_, Activity>(
        "SELECT * FROM schedule_activities WHERE schedule_id = $1 ORDER BY sort_order, code",
    )
    .bind(schedule_id)
    .fetch_all(&state.db)
    .await?;

    // Convert DB activities to CPM input
    let cpm_activities: Vec<erp_core::cpm::CpmActivity> = activities
        .iter()
        .map(|a| {
            let predecessors = a
                .predecessors
                .as_ref()
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|p| {
                            let pred_code = p.get("code")?.as_str()?.to_string();
                            let rel = match p.get("type").and_then(|t| t.as_str()).unwrap_or("FS") {
                                "FF" => erp_core::cpm::RelationType::FF,
                                "SS" => erp_core::cpm::RelationType::SS,
                                "SF" => erp_core::cpm::RelationType::SF,
                                _ => erp_core::cpm::RelationType::FS,
                            };
                            let lag = p.get("lag").and_then(|l| l.as_f64()).unwrap_or(0.0);
                            Some((pred_code, rel, lag))
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            erp_core::cpm::CpmActivity {
                id: a.code.clone(),
                duration: a.duration_days,
                predecessors,
            }
        })
        .collect();

    let results = erp_core::cpm::calculate_cpm(&cpm_activities);

    let critical: Vec<&str> = results
        .iter()
        .filter(|(_, r)| r.is_critical)
        .map(|(id, _)| id.as_str())
        .collect();

    let project_duration = results
        .values()
        .map(|r| r.early_finish)
        .fold(0.0_f64, f64::max);

    Ok(Json(serde_json::json!({
        "schedule_id": schedule_id,
        "project_duration": project_duration,
        "critical_path": critical,
        "activities": results,
    })))
}

// ── Helpers ─────────────────────────────────────────

async fn verify_project_owner(state: &AppState, project_id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
    let exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM projects WHERE id = $1 AND user_id = $2",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    if exists.is_none() {
        return Err(ApiError::NotFound("Project not found".into()));
    }

    Ok(())
}

async fn verify_schedule_owner(state: &AppState, schedule_id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
    let exists: Option<(Uuid,)> = sqlx::query_as(
        r#"SELECT s.id FROM schedules s
           JOIN projects p ON p.id = s.project_id
           WHERE s.id = $1 AND p.user_id = $2"#,
    )
    .bind(schedule_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    if exists.is_none() {
        return Err(ApiError::NotFound("Schedule not found".into()));
    }

    Ok(())
}

async fn get_schedule_id_for_activity(state: &AppState, activity_id: Uuid) -> Result<Uuid, ApiError> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT schedule_id FROM schedule_activities WHERE id = $1",
    )
    .bind(activity_id)
    .fetch_optional(&state.db)
    .await?;

    row.map(|r| r.0)
        .ok_or_else(|| ApiError::NotFound("Activity not found".into()))
}
