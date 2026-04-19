use crate::error::AppError;
use crate::handlers::users::UserState;
use crate::services::auth::AuthService;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tracing::info;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct OtpRequestDto {
    #[validate(email(message = "email is invalid"))]
    pub email: String,
}

#[derive(Deserialize)]
pub struct VerifyOtpRequestDto {
    pub email: String,
    pub code: String,
}

#[derive(Deserialize, Validate)]
pub struct EmailAuthRequestDto {
    #[validate(email(message = "email is invalid"))]
    pub email: String,
}

#[derive(Serialize)]
pub struct AuthResponseDto {
    pub access_token: String,
    pub token_type: String,
}

pub trait AuthServiceProvier: UserState + Clone + Send + Sync + 'static {
    fn auth_service(&self) -> &AuthService;
}

pub fn router<S>() -> Router<S>
where
    S: AuthServiceProvier,
{
    Router::new()
        .route("/otp/verify", post(verify_otp_handler::<S>))
        .route("/email", post(email_auth_handler::<S>))
}

pub async fn verify_otp_handler<S>(
    State(state): State<S>,
    Json(payload): Json<VerifyOtpRequestDto>,
) -> Result<Json<AuthResponseDto>, AppError>
where
    S: AuthServiceProvier,
{
    let auth_service = state.auth_service();
    auth_service
        .verify_otp(&payload.email, &payload.code)
        .await?;

    let user_id = state
        .user_repo()
        .find_by_email(&payload.email)
        .await?
        .ok_or_else(|| AppError::Internal("User not found after token verification".to_string()))?;

    let token = auth_service.generate_jwt(user_id)?;

    Ok(Json(AuthResponseDto {
        access_token: token,
        token_type: "Bearer".to_string(),
    }))
}

pub async fn email_auth_handler<S>(
    State(state): State<S>,
    Json(payload): Json<EmailAuthRequestDto>,
) -> Result<StatusCode, AppError>
where
    S: AuthServiceProvier,
{
    payload.validate().map_err(|e| {
        info!("Validation failure: {}", e);
        AppError::ValidationError(e.to_string())
    })?;
    let auth_service = state.auth_service();
    let user_repo = state.user_repo();

    // Verify email whether exists in store.users
    let email_exists = user_repo.user_with_email_exists(&payload.email).await?;

    if !email_exists {
        return Ok(StatusCode::NOT_FOUND);
    }

    // Generate OTP
    auth_service.request_otp(&payload.email).await?;

    Ok(StatusCode::OK)
}
