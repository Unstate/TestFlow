use axum::{extract::State, Json};
use validator::Validate;

use crate::auth::create_token;
use crate::dto::{LoginRequest, LoginResponse, UserResponse};
use crate::errors::AppError;
use crate::models::User;
use crate::AppState;

/// Login and receive JWT token
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 400, description = "Validation error")
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    payload.validate()?;

    let user: User = sqlx::query_as(
        r#"SELECT id, username, email, password_hash, full_name,
                  role, is_active, created_at, updated_at
           FROM users WHERE username = $1"#,
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    if !user.is_active {
        return Err(AppError::Unauthorized("Account is deactivated".to_string()));
    }

    let parsed_hash = argon2::password_hash::PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::Internal("Password hash error".to_string()))?;

    use argon2::PasswordVerifier;
    argon2::Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Invalid username or password".to_string()))?;

    let token = create_token(
        user.id,
        &user.username,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_expiration_hours,
    )?;

    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer".to_string(),
        user: UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            full_name: user.full_name,
            role: user.role,
            is_active: user.is_active,
            created_at: user.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: user.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        },
    }))
}
