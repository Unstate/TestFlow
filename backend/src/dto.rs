use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::models::{TaskStatus, TaskUrgency, UserRole};

// ── Auth ──

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Username is required"))]
    pub username: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
    pub user: UserResponse,
}

// ── User DTOs ──

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50, message = "Username must be 3-50 characters"))]
    pub username: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
    #[validate(length(min = 1, max = 100, message = "Full name is required"))]
    pub full_name: String,
    pub role: UserRole,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, max = 50, message = "Username must be 3-50 characters"))]
    pub username: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: Option<String>,
    #[validate(length(min = 1, max = 100, message = "Full name is required"))]
    pub full_name: Option<String>,
    pub role: Option<UserRole>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// ── Task DTOs ──

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateTaskRequest {
    #[validate(length(min = 1, max = 255, message = "Title is required"))]
    pub title: String,
    pub description: Option<String>,
    pub tester_id: Option<Uuid>,
    pub urgency: Option<TaskUrgency>,
    pub acceptance_criteria: Option<String>,
    pub evaluation_criteria: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateTaskRequest {
    #[validate(length(min = 1, max = 255, message = "Title must not be empty"))]
    pub title: Option<String>,
    pub description: Option<String>,
    pub tester_id: Option<Uuid>,
    pub status: Option<TaskStatus>,
    pub urgency: Option<TaskUrgency>,
    pub acceptance_criteria: Option<String>,
    pub evaluation_criteria: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskResponse {
    pub id: Uuid,
    pub task_number: i32,
    pub title: String,
    pub description: Option<String>,
    pub assigned_by: Uuid,
    pub assigned_by_name: Option<String>,
    pub tester_id: Option<Uuid>,
    pub tester_name: Option<String>,
    pub status: TaskStatus,
    pub urgency: TaskUrgency,
    pub created_at: String,
    pub closed_at: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub evaluation_criteria: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskListItem {
    pub id: Uuid,
    pub task_number: i32,
    pub title: String,
    pub status: TaskStatus,
    pub urgency: TaskUrgency,
}

// ── Statistics ──

#[derive(Debug, Serialize, ToSchema)]
pub struct EmployeeStats {
    pub user_id: Uuid,
    pub full_name: String,
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub in_progress_tasks: i64,
}

// ── Pagination ──

#[derive(Debug, Deserialize, ToSchema)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TaskFilterParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<TaskStatus>,
    pub urgency: Option<TaskUrgency>,
    pub tester_id: Option<Uuid>,
    pub assigned_by: Option<Uuid>,
}
