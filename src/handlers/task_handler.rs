use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::NaiveDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::auth::AuthUser;
use crate::dto::{
    CreateTaskRequest, EmployeeStats, TaskFilterParams, TaskListItem, TaskResponse,
    UpdateTaskRequest,
};
use crate::errors::AppError;
use crate::models::{Task, TaskStatus, TaskUrgency, UserRole};
use crate::AppState;

fn task_to_response(
    t: Task,
    assigned_by_name: Option<String>,
    tester_name: Option<String>,
) -> TaskResponse {
    TaskResponse {
        id: t.id,
        task_number: t.task_number,
        title: t.title,
        description: t.description,
        assigned_by: t.assigned_by,
        assigned_by_name,
        tester_id: t.tester_id,
        tester_name,
        status: t.status,
        urgency: t.urgency,
        created_at: t.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        closed_at: t
            .closed_at
            .map(|d: NaiveDateTime| d.format("%Y-%m-%d %H:%M:%S").to_string()),
        acceptance_criteria: t.acceptance_criteria,
        evaluation_criteria: t.evaluation_criteria,
        comment: t.comment,
    }
}

async fn fetch_user_name(db: &sqlx::PgPool, user_id: Uuid) -> Option<String> {
    sqlx::query_scalar::<_, String>("SELECT full_name FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
}

/// Get all tasks (with filtering)
#[utoipa::path(
    get,
    path = "/api/tasks",
    params(
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("status" = Option<TaskStatus>, Query, description = "Filter by status"),
        ("urgency" = Option<TaskUrgency>, Query, description = "Filter by urgency"),
        ("tester_id" = Option<Uuid>, Query, description = "Filter by tester"),
        ("assigned_by" = Option<Uuid>, Query, description = "Filter by assigner")
    ),
    responses(
        (status = 200, description = "List of tasks", body = Vec<TaskListItem>)
    ),
    security(("bearer_auth" = [])),
    tag = "Tasks"
)]
pub async fn get_tasks(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<TaskFilterParams>,
) -> Result<Json<Vec<TaskListItem>>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let status_str = params.status.map(|s| s.to_string());
    let urgency_str = params.urgency.map(|u| u.to_string());

    let tasks: Vec<Task> = sqlx::query_as(
        "SELECT id, task_number, title, description, assigned_by, tester_id,
                status, urgency, created_at, closed_at, acceptance_criteria,
                evaluation_criteria, comment
         FROM tasks
         WHERE ($1::text IS NULL OR status::text = $1)
           AND ($2::text IS NULL OR urgency::text = $2)
           AND ($3::uuid IS NULL OR tester_id = $3)
           AND ($4::uuid IS NULL OR assigned_by = $4)
         ORDER BY created_at DESC
         LIMIT $5 OFFSET $6",
    )
    .bind(&status_str)
    .bind(&urgency_str)
    .bind(params.tester_id)
    .bind(params.assigned_by)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let response: Vec<TaskListItem> = tasks
        .into_iter()
        .map(|t| TaskListItem {
            id: t.id,
            task_number: t.task_number,
            title: t.title,
            status: t.status,
            urgency: t.urgency,
        })
        .collect();

    Ok(Json(response))
}

/// Get task by ID
#[utoipa::path(
    get,
    path = "/api/tasks/{id}",
    params(("id" = Uuid, Path, description = "Task ID")),
    responses(
        (status = 200, description = "Task details", body = TaskResponse),
        (status = 404, description = "Task not found")
    ),
    security(("bearer_auth" = [])),
    tag = "Tasks"
)]
pub async fn get_task(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TaskResponse>, AppError> {
    let task: Task = sqlx::query_as(
        "SELECT id, task_number, title, description, assigned_by, tester_id,
                status, urgency, created_at, closed_at, acceptance_criteria,
                evaluation_criteria, comment
         FROM tasks WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    let assigned_by_name = fetch_user_name(&state.db, task.assigned_by).await;
    let tester_name = match task.tester_id {
        Some(tid) => fetch_user_name(&state.db, tid).await,
        None => None,
    };

    Ok(Json(task_to_response(task, assigned_by_name, tester_name)))
}

/// Create a new task (all roles except admin)
#[utoipa::path(
    post,
    path = "/api/tasks",
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task created", body = TaskResponse),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Admins cannot create tasks")
    ),
    security(("bearer_auth" = [])),
    tag = "Tasks"
)]
pub async fn create_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(axum::http::StatusCode, Json<TaskResponse>), AppError> {
    if auth.is_admin() {
        return Err(AppError::Forbidden(
            "Administrators cannot create tasks".to_string(),
        ));
    }

    payload.validate()?;

    let urgency_str = payload
        .urgency
        .as_ref()
        .map(|u| u.to_string())
        .unwrap_or_else(|| "medium".to_string());

    let task: Task = sqlx::query_as(
        "INSERT INTO tasks (title, description, assigned_by, tester_id, urgency,
                            acceptance_criteria, evaluation_criteria, comment)
         VALUES ($1, $2, $3, $4, $5::task_urgency, $6, $7, $8)
         RETURNING id, task_number, title, description, assigned_by, tester_id,
                   status, urgency, created_at, closed_at, acceptance_criteria,
                   evaluation_criteria, comment",
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(auth.user_id)
    .bind(payload.tester_id)
    .bind(&urgency_str)
    .bind(&payload.acceptance_criteria)
    .bind(&payload.evaluation_criteria)
    .bind(&payload.comment)
    .fetch_one(&state.db)
    .await?;

    let assigned_by_name = fetch_user_name(&state.db, task.assigned_by).await;
    let tester_name = match task.tester_id {
        Some(tid) => fetch_user_name(&state.db, tid).await,
        None => None,
    };

    Ok((
        axum::http::StatusCode::CREATED,
        Json(task_to_response(task, assigned_by_name, tester_name)),
    ))
}

/// Update a task (all roles except admin)
#[utoipa::path(
    put,
    path = "/api/tasks/{id}",
    params(("id" = Uuid, Path, description = "Task ID")),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, description = "Task updated", body = TaskResponse),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Task not found"),
        (status = 403, description = "Admins cannot edit tasks")
    ),
    security(("bearer_auth" = [])),
    tag = "Tasks"
)]
pub async fn update_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    if auth.is_admin() {
        return Err(AppError::Forbidden(
            "Administrators cannot edit tasks".to_string(),
        ));
    }

    payload.validate()?;

    let existing: Task = sqlx::query_as(
        "SELECT id, task_number, title, description, assigned_by, tester_id,
                status, urgency, created_at, closed_at, acceptance_criteria,
                evaluation_criteria, comment
         FROM tasks WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    let new_title = payload.title.unwrap_or(existing.title);
    let new_description = payload.description.or(existing.description);
    let new_tester_id = payload.tester_id.or(existing.tester_id);
    let new_status = payload.status.unwrap_or(existing.status);
    let new_urgency = payload.urgency.unwrap_or(existing.urgency);
    let new_acceptance = payload.acceptance_criteria.or(existing.acceptance_criteria);
    let new_evaluation = payload.evaluation_criteria.or(existing.evaluation_criteria);
    let new_comment = payload.comment.or(existing.comment);

    let closed_at = if new_status == TaskStatus::Closed || new_status == TaskStatus::Done {
        Some(chrono::Utc::now().naive_utc())
    } else {
        existing.closed_at
    };

    let status_str = new_status.to_string();
    let urgency_str = new_urgency.to_string();

    let task: Task = sqlx::query_as(
        "UPDATE tasks SET title = $1, description = $2, tester_id = $3,
                          status = $4::task_status, urgency = $5::task_urgency,
                          acceptance_criteria = $6, evaluation_criteria = $7,
                          comment = $8, closed_at = $9
         WHERE id = $10
         RETURNING id, task_number, title, description, assigned_by, tester_id,
                   status, urgency, created_at, closed_at, acceptance_criteria,
                   evaluation_criteria, comment",
    )
    .bind(&new_title)
    .bind(&new_description)
    .bind(new_tester_id)
    .bind(&status_str)
    .bind(&urgency_str)
    .bind(&new_acceptance)
    .bind(&new_evaluation)
    .bind(&new_comment)
    .bind(closed_at)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    let assigned_by_name = fetch_user_name(&state.db, task.assigned_by).await;
    let tester_name = match task.tester_id {
        Some(tid) => fetch_user_name(&state.db, tid).await,
        None => None,
    };

    Ok(Json(task_to_response(task, assigned_by_name, tester_name)))
}

/// Delete a task (manager or the person who created it)
#[utoipa::path(
    delete,
    path = "/api/tasks/{id}",
    params(("id" = Uuid, Path, description = "Task ID")),
    responses(
        (status = 204, description = "Task deleted"),
        (status = 404, description = "Task not found"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = [])),
    tag = "Tasks"
)]
pub async fn delete_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    if auth.is_admin() {
        return Err(AppError::Forbidden(
            "Administrators cannot manage tasks".to_string(),
        ));
    }

    let task: Task = sqlx::query_as(
        "SELECT id, task_number, title, description, assigned_by, tester_id,
                status, urgency, created_at, closed_at, acceptance_criteria,
                evaluation_criteria, comment
         FROM tasks WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    if task.assigned_by != auth.user_id && !auth.is_manager() {
        return Err(AppError::Forbidden(
            "Only the task creator or a manager can delete tasks".to_string(),
        ));
    }

    sqlx::query("DELETE FROM tasks WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// Get employee statistics (manager/admin only)
#[utoipa::path(
    get,
    path = "/api/statistics/employees",
    responses(
        (status = 200, description = "Employee statistics", body = Vec<EmployeeStats>),
        (status = 403, description = "Forbidden - managers only")
    ),
    security(("bearer_auth" = [])),
    tag = "Statistics"
)]
pub async fn get_employee_stats(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<EmployeeStats>>, AppError> {
    if auth.role != UserRole::Manager && auth.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "Only managers and admins can view statistics".to_string(),
        ));
    }

    let rows: Vec<(Uuid, String, Option<i64>, Option<i64>, Option<i64>)> = sqlx::query_as(
        "SELECT u.id, u.full_name,
                COUNT(t.id) as total_tasks,
                COUNT(t.id) FILTER (WHERE t.status::text IN ('done', 'closed')) as completed_tasks,
                COUNT(t.id) FILTER (WHERE t.status::text = 'in_progress') as in_progress_tasks
         FROM users u
         LEFT JOIN tasks t ON t.tester_id = u.id
         WHERE u.role::text != 'admin'
         GROUP BY u.id, u.full_name
         ORDER BY u.full_name",
    )
    .fetch_all(&state.db)
    .await?;

    let response: Vec<EmployeeStats> = rows
        .into_iter()
        .map(|(user_id, full_name, total, completed, in_progress)| EmployeeStats {
            user_id,
            full_name,
            total_tasks: total.unwrap_or(0),
            completed_tasks: completed.unwrap_or(0),
            in_progress_tasks: in_progress.unwrap_or(0),
        })
        .collect();

    Ok(Json(response))
}
