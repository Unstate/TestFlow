use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

// ── Roles ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type, ToSchema)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum UserRole {
    #[sqlx(rename = "admin")]
    #[serde(rename = "admin")]
    Admin,
    #[sqlx(rename = "manager")]
    #[serde(rename = "manager")]
    Manager,
    #[sqlx(rename = "tester")]
    #[serde(rename = "tester")]
    Tester,
    #[sqlx(rename = "developer")]
    #[serde(rename = "developer")]
    Developer,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Manager => write!(f, "manager"),
            UserRole::Tester => write!(f, "tester"),
            UserRole::Developer => write!(f, "developer"),
        }
    }
}

// ── User ──

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// ── Task urgency ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type, ToSchema)]
#[sqlx(type_name = "task_urgency", rename_all = "snake_case")]
pub enum TaskUrgency {
    #[sqlx(rename = "low")]
    #[serde(rename = "low")]
    Low,
    #[sqlx(rename = "medium")]
    #[serde(rename = "medium")]
    Medium,
    #[sqlx(rename = "high")]
    #[serde(rename = "high")]
    High,
    #[sqlx(rename = "critical")]
    #[serde(rename = "critical")]
    Critical,
}

impl std::fmt::Display for TaskUrgency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskUrgency::Low => write!(f, "low"),
            TaskUrgency::Medium => write!(f, "medium"),
            TaskUrgency::High => write!(f, "high"),
            TaskUrgency::Critical => write!(f, "critical"),
        }
    }
}

// ── Task status ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type, ToSchema)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
pub enum TaskStatus {
    #[sqlx(rename = "new")]
    #[serde(rename = "new")]
    New,
    #[sqlx(rename = "in_progress")]
    #[serde(rename = "in_progress")]
    InProgress,
    #[sqlx(rename = "testing")]
    #[serde(rename = "testing")]
    Testing,
    #[sqlx(rename = "done")]
    #[serde(rename = "done")]
    Done,
    #[sqlx(rename = "closed")]
    #[serde(rename = "closed")]
    Closed,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::New => write!(f, "new"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::Testing => write!(f, "testing"),
            TaskStatus::Done => write!(f, "done"),
            TaskStatus::Closed => write!(f, "closed"),
        }
    }
}

// ── Task ──

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Task {
    pub id: Uuid,
    pub task_number: i32,
    pub title: String,
    pub description: Option<String>,
    pub assigned_by: Uuid,
    pub tester_id: Option<Uuid>,
    pub status: TaskStatus,
    pub urgency: TaskUrgency,
    pub created_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>,
    pub acceptance_criteria: Option<String>,
    pub evaluation_criteria: Option<String>,
    pub comment: Option<String>,
}
