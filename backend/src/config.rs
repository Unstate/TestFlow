use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
}

pub async fn create_db_pool() -> PgPool {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL")
}

pub fn load_config() -> AppConfig {
    AppConfig {
        jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        jwt_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .expect("JWT_EXPIRATION_HOURS must be a number"),
    }
}
