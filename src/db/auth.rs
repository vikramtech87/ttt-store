use crate::error::AppError;
use chrono::{Duration, Utc};
use sqlx::{FromRow, PgPool};

#[derive(Clone)]
pub struct AuthRepo {
    pool: PgPool,
}

#[derive(FromRow)]
struct OtpValid {
    is_valid: bool,
}

impl AuthRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_otp(&self, email: &str, code: &str) -> Result<(), AppError> {
        let expires = Utc::now() + Duration::minutes(10);

        sqlx::query!(
            r#"
                INSERT INTO store.otp_codes
                    (email, code, expires_at)
                VALUES
                    ($1, $2, $3)
            "#,
            email,
            code,
            expires,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_otp(&self, email: &str, code: &str) -> Result<Option<bool>, AppError> {
        let result = sqlx::query_as::<_, OtpValid>(
            r#"
                SELECT expires_at > NOW() as is_valid
                FROM store.otp_codes
                WHERE email = $1 AND code = $2
                ORDER BY expires_at DESC
                LIMIT 1
            "#,
        )
        .bind(email)
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|otp_valid| otp_valid.is_valid))
    }

    pub async fn delete_otp(&self, email: &str) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM store.otp_codes WHERE email = $1", email)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_pool;
    use dotenvy::dotenv;

    #[tokio::test]
    pub async fn test_store_otp() {
        dotenv().ok();
        let pool = init_pool().await.unwrap();
        let repo = AuthRepo::new(pool);

        let result = repo.store_otp("dr.vikramraj87@gmail.com", "123456").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    pub async fn test_find_otp() {
        dotenv().ok();
        let pool = init_pool().await.unwrap();
        let repo = AuthRepo::new(pool);

        let email = "dr.vikramraj87@gmail.com";
        let code = "211124";

        repo.store_otp(email, code)
            .await
            .expect("Storing otp failed");

        let result = repo.find_otp(email, code).await.unwrap().unwrap_or(false);

        assert!(result);
    }
}
