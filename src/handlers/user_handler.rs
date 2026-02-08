use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::auth::AuthUser;
use crate::dto::{CreateUserRequest, PaginationParams, UpdateUserRequest, UserResponse};
use crate::errors::AppError;
use crate::models::User;
use crate::AppState;

fn require_admin(auth: &AuthUser) -> Result<(), AppError> {
    if !auth.is_admin() {
        return Err(AppError::Forbidden(
            "Only administrators can manage users".to_string(),
        ));
    }
    Ok(())
}

fn user_to_response(u: User) -> UserResponse {
    UserResponse {
        id: u.id,
        username: u.username,
        email: u.email,
        full_name: u.full_name,
        role: u.role,
        is_active: u.is_active,
        created_at: u.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: u.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

/// Get all users (admin only)
#[utoipa::path(
    get,
    path = "/api/users",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("per_page" = Option<i64>, Query, description = "Items per page (default 20)")
    ),
    responses(
        (status = 200, description = "List of users", body = Vec<UserResponse>),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = [])),
    tag = "Users"
)]
pub async fn get_users(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    require_admin(&auth)?;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let users: Vec<User> = sqlx::query_as(
        "SELECT id, username, email, password_hash, full_name, role, is_active, created_at, updated_at
         FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let response: Vec<UserResponse> = users.into_iter().map(user_to_response).collect();
    Ok(Json(response))
}

/// Get user by ID (admin only)
#[utoipa::path(
    get,
    path = "/api/users/{id}",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = [])),
    tag = "Users"
)]
pub async fn get_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    require_admin(&auth)?;

    let user: User = sqlx::query_as(
        "SELECT id, username, email, password_hash, full_name, role, is_active, created_at, updated_at
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user_to_response(user)))
}

/// Create a new user (admin only)
#[utoipa::path(
    post,
    path = "/api/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = UserResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Username or email already exists"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = [])),
    tag = "Users"
)]
pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(axum::http::StatusCode, Json<UserResponse>), AppError> {
    require_admin(&auth)?;
    payload.validate()?;

    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM users WHERE username = $1 OR email = $2",
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict(
            "Username or email already exists".to_string(),
        ));
    }

    use argon2::PasswordHasher;
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let password_hash = argon2::Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hash error: {}", e)))?
        .to_string();

    let role_str = payload.role.to_string();
    let user: User = sqlx::query_as(
        "INSERT INTO users (username, email, password_hash, full_name, role)
         VALUES ($1, $2, $3, $4, $5::user_role)
         RETURNING id, username, email, password_hash, full_name, role, is_active, created_at, updated_at",
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.full_name)
    .bind(&role_str)
    .fetch_one(&state.db)
    .await?;

    Ok((axum::http::StatusCode::CREATED, Json(user_to_response(user))))
}

/// Update a user (admin only)
#[utoipa::path(
    put,
    path = "/api/users/{id}",
    params(("id" = Uuid, Path, description = "User ID")),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = UserResponse),
        (status = 400, description = "Validation error"),
        (status = 404, description = "User not found"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = [])),
    tag = "Users"
)]
pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    require_admin(&auth)?;
    payload.validate()?;

    let existing: User = sqlx::query_as(
        "SELECT id, username, email, password_hash, full_name, role, is_active, created_at, updated_at
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let new_username = payload.username.unwrap_or(existing.username);
    let new_email = payload.email.unwrap_or(existing.email);
    let new_full_name = payload.full_name.unwrap_or(existing.full_name);
    let new_role = payload.role.unwrap_or(existing.role);
    let new_is_active = payload.is_active.unwrap_or(existing.is_active);

    let new_password_hash = if let Some(new_password) = payload.password {
        use argon2::PasswordHasher;
        let salt = argon2::password_hash::SaltString::generate(
            &mut argon2::password_hash::rand_core::OsRng,
        );
        argon2::Argon2::default()
            .hash_password(new_password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Password hash error: {}", e)))?
            .to_string()
    } else {
        existing.password_hash
    };

    let role_str = new_role.to_string();
    let user: User = sqlx::query_as(
        "UPDATE users SET username = $1, email = $2, password_hash = $3,
                          full_name = $4, role = $5::user_role, is_active = $6,
                          updated_at = NOW()
         WHERE id = $7
         RETURNING id, username, email, password_hash, full_name, role, is_active, created_at, updated_at",
    )
    .bind(&new_username)
    .bind(&new_email)
    .bind(&new_password_hash)
    .bind(&new_full_name)
    .bind(&role_str)
    .bind(new_is_active)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(user_to_response(user)))
}

/// Delete a user (admin only)
#[utoipa::path(
    delete,
    path = "/api/users/{id}",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = [])),
    tag = "Users"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    require_admin(&auth)?;

    if id == auth.user_id {
        return Err(AppError::BadRequest(
            "Cannot delete your own account".to_string(),
        ));
    }

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// Get current user profile
#[utoipa::path(
    get,
    path = "/api/users/me",
    responses(
        (status = 200, description = "Current user profile", body = UserResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Users"
)]
pub async fn get_me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserResponse>, AppError> {
    let user: User = sqlx::query_as(
        "SELECT id, username, email, password_hash, full_name, role, is_active, created_at, updated_at
         FROM users WHERE id = $1",
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user_to_response(user)))
}
