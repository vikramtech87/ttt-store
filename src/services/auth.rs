use crate::db::auth::AuthRepo;
use crate::error::AppError;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub sub: uuid::Uuid,
    pub exp: usize,
    pub iat: usize,
}

impl Claims {
    pub fn new(sub: uuid::Uuid) -> Self {
        let now = Utc::now();
        let expire = now + Duration::hours(24);

        Self {
            sub,
            iat: now.timestamp() as usize,
            exp: expire.timestamp() as usize,
        }
    }
}

#[derive(Clone)]
pub struct AuthService {
    auth_repo: AuthRepo,
}

impl AuthService {
    pub fn new(auth_repo: AuthRepo) -> Self {
        Self { auth_repo }
    }

    pub async fn request_otp(&self, email: &str) -> Result<(), AppError> {
        let code = rand::thread_rng().gen_range(100_000..999_999).to_string();

        self.auth_repo.store_otp(email, &code).await?;

        info!("🔑 OTP generated for {}: {}", email, code);

        Ok(())
    }

    pub async fn verify_otp(&self, email: &str, code: &str) -> Result<(), AppError> {
        let is_valid = self.auth_repo.find_otp(email, code).await?;


        match is_valid {
            Some(true) => {
                let _ = self.auth_repo.delete_otp(email).await;
                info!("Successful login for {}", email);
                Ok(())
            }
            Some(false) => Err(AppError::ValidationError("OTP has expired".into())),
            None => Err(AppError::ValidationError("Invalid code or email".into())),
        }
    }

    pub fn generate_jwt(&self, user_id: uuid::Uuid) -> Result<String, AppError> {
        let claims = Claims::new(user_id);
        let secret = std::env::var("JWT_SECRET")
            .map_err(|_| AppError::Internal("JWT_SECRET not set".into()))?;

        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| AppError::Internal(format!("JWT encoding failed: {}", e.to_string())))
    }
}
