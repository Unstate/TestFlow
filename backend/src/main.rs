mod auth;
mod config;
mod dto;
mod errors;
mod handlers;
mod models;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppConfig;
use crate::handlers::{auth_handler, task_handler, user_handler};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: AppConfig,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        auth_handler::login,
        user_handler::get_users,
        user_handler::get_user,
        user_handler::get_me,
        user_handler::create_user,
        user_handler::update_user,
        user_handler::delete_user,
        task_handler::get_tasks,
        task_handler::get_task,
        task_handler::create_task,
        task_handler::update_task,
        task_handler::delete_task,
        task_handler::get_employee_stats,
    ),
    components(schemas(
        dto::LoginRequest,
        dto::LoginResponse,
        dto::UserResponse,
        dto::CreateUserRequest,
        dto::UpdateUserRequest,
        dto::TaskResponse,
        dto::TaskListItem,
        dto::CreateTaskRequest,
        dto::UpdateTaskRequest,
        dto::EmployeeStats,
        models::UserRole,
        models::TaskStatus,
        models::TaskUrgency,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "Authentication", description = "Login and token management"),
        (name = "Users", description = "User CRUD (admin only)"),
        (name = "Tasks", description = "Task management"),
        (name = "Statistics", description = "Employee statistics (manager/admin)")
    ),
    info(
        title = "TestFlow API",
        version = "1.0.0",
        description = "TestFlow - Task and Testing Management System API"
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "testflow=debug,tower_http=debug".parse().unwrap()),
        )
        .init();

    let db = config::create_db_pool().await;
    let app_config = config::load_config();

    // Run migrations
    tracing::info!("Running database migrations...");
    run_migrations(&db).await;
    tracing::info!("Migrations completed.");

    // Seed default admin if no users exist
    seed_admin(&db).await;

    let state = AppState {
        db,
        config: app_config,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Auth
        .route("/api/auth/login", post(auth_handler::login))
        // Users
        .route(
            "/api/users",
            get(user_handler::get_users).post(user_handler::create_user),
        )
        .route("/api/users/me", get(user_handler::get_me))
        .route(
            "/api/users/{id}",
            get(user_handler::get_user)
                .put(user_handler::update_user)
                .delete(user_handler::delete_user),
        )
        // Tasks
        .route(
            "/api/tasks",
            get(task_handler::get_tasks).post(task_handler::create_task),
        )
        .route(
            "/api/tasks/{id}",
            get(task_handler::get_task)
                .put(task_handler::update_task)
                .delete(task_handler::delete_task),
        )
        // Statistics
        .route(
            "/api/statistics/employees",
            get(task_handler::get_employee_stats),
        )
        // Swagger UI
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("Server starting on http://{}", addr);
    tracing::info!("Swagger UI: http://localhost:3000/swagger-ui/");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn run_migrations(db: &PgPool) {
    // Split migration file into individual statements and execute them
    let migration_sql = include_str!("../migrations/001_init.sql");
    
    // Execute the entire migration as a simple query (not prepared statement)
    sqlx::raw_sql(migration_sql)
        .execute(db)
        .await
        .expect("Failed to run migrations");
}

async fn seed_admin(db: &PgPool) {
    let count: Option<i64> =
        sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(db)
            .await
            .unwrap_or(Some(0));

    if count.unwrap_or(0) == 0 {
        tracing::info!("No users found. Creating default admin...");

        use argon2::PasswordHasher;
        let salt = argon2::password_hash::SaltString::generate(
            &mut argon2::password_hash::rand_core::OsRng,
        );
        let password_hash = argon2::Argon2::default()
            .hash_password(b"admin123", &salt)
            .expect("Failed to hash password")
            .to_string();

        sqlx::query(
            "INSERT INTO users (username, email, password_hash, full_name, role)
             VALUES ($1, $2, $3, $4, $5::user_role)",
        )
        .bind("admin")
        .bind("admin@testflow.local")
        .bind(&password_hash)
        .bind("System Administrator")
        .bind("admin")
        .execute(db)
        .await
        .expect("Failed to create default admin");

        tracing::info!("Default admin created: username='admin', password='admin123'");
    }
}
