use axum::{
    http::StatusCode,
    response::{IntoResponse},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    UniqueViolation(String),
    NotFound(String),
    DatabaseError(sqlx::Error),
    Internal(String),
    ValidationError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::UniqueViolation(message) => (StatusCode::CONFLICT, message),
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            AppError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into()),
            AppError::ValidationError(message) => (StatusCode::BAD_REQUEST, message),
        };
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let Some(db_err) = err.as_database_error() {
            if db_err.is_unique_violation() {
                return AppError::UniqueViolation("Record already exists.".into());
            }
        }
        AppError::DatabaseError(err)
    }
}