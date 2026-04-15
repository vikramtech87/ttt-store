use crate::models::{User, UserIdentity};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepo {
    pool: PgPool,
}

impl UserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, email: &str, full_name: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
                INSERT INTO store.users (email, full_name)
                VALUES ($1, $2)
                RETURNING id, email, full_name, created_at, updated_at
            "#,
        )
        .bind(email)
        .bind(full_name)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn add_user_identity(
        &self,
        user_id: Uuid,
        provider: &str,
        provider_id: &str,
    ) -> Result<UserIdentity, sqlx::Error> {
        sqlx::query_as::<_, UserIdentity>(
            r#"
                INSERT INTO store.user_identities
                    (user_id, provider, provider_id)
                VALUES ($1, $2, $3)
                RETURNING id, user_id, provider, provider_id, created_at
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .bind(provider_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_user_by_identity(
        &self,
        provider: &str,
        provider_id: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
                SELECT u.id, u.email, u.full_name, u.created_at, u.updated_at
                FROM store.users u
                JOIN store.user_identities ui on u.id = ui.user_id
                WHERE ui.provider_id = $1 AND ui.provider = $2
            "#,
        )
        .bind(provider_id)
        .bind(provider)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
                SELECT id, email, full_name, created_at, updated_at
                FROM store.users
                WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_pool;
    use dotenvy::dotenv;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_user_success() {
        dotenv().ok();
        let pool = init_pool().await.unwrap();
        let repo = UserRepo::new(pool);

        let email = format!("test-{}@example.com", Uuid::new_v4());
        let name = "Test User".to_string();

        let result = repo.create_user(&email, &name).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, email);
        assert_eq!(user.full_name, name);
    }

    #[tokio::test]
    async fn test_add_user_identity() {
        dotenv().ok();
        let pool = init_pool().await.unwrap();
        let repo = UserRepo::new(pool);

        // 1. Create user first
        let email = format!("auth-{}@exmaple.dev", Uuid::new_v4());
        let user = repo.create_user(&email, "Auth User").await.unwrap();

        // 2. Try to add google identity
        let provider_id = format!("google-{}", Uuid::new_v4());
        let identity = repo
            .add_user_identity(user.id, "google", &provider_id)
            .await;

        assert!(identity.is_ok());
        let iden = identity.unwrap();
        assert_eq!(iden.provider, "google");
        assert_eq!(iden.provider_id, provider_id);
        assert_eq!(iden.user_id, user.id);
    }

    #[tokio::test]
    async fn test_duplicate_identity_fails() {
        dotenv().ok();
        let pool = init_pool().await.unwrap();
        let repo = UserRepo::new(pool);

        // 1. Setup: Create a user and an identity
        let email = format!("dup-auth-{}@example.com", uuid::Uuid::new_v4());
        let user = repo.create_user(&email, "Dup User").await.unwrap();

        let provider = "google";
        let provider_id = "same-google-id";

        // First one should succeed. Should be uncommented only for one time
        // add_user_identity(&pool, user.id, provider, provider_id)
        //     .await
        //     .unwrap();

        // 2. The "Red" Action: Try to add the same identity again
        let result = repo.add_user_identity(user.id, provider, provider_id).await;

        // This should be an error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_user_with_identity() {
        dotenv().ok();
        let pool = init_pool().await.unwrap();
        let repo = UserRepo::new(pool);

        // 1. Create user
        let email = format!("find-{}@exmaple.dev", Uuid::new_v4());
        let user = repo.create_user(&email, "Find me").await.unwrap();

        let provider = "google";
        let provider_id = format!("google-{}", Uuid::new_v4());
        let _ = repo
            .add_user_identity(user.id, "google", &provider_id)
            .await
            .unwrap();

        // 2. Fetch user
        let found_user = repo
            .find_user_by_identity(provider, &provider_id)
            .await
            .unwrap();

        assert!(found_user.is_some());
        let u = found_user.unwrap();
        assert_eq!(u.id, user.id);
        assert_eq!(u.email, email);
    }

        #[tokio::test]
        async fn test_find_user_by_email_success() {
            dotenv().ok();
            let pool = init_pool().await.unwrap();
            let repo = UserRepo::new(pool);

            let email = format!("email-find-{}@example.com", uuid::Uuid::new_v4());
            let _ = repo.create_user(&email, "Email User").await.unwrap();

            let found = repo.find_user_by_email(&email).await.unwrap();

            assert!(found.is_some());
            assert_eq!(found.unwrap().email, email);
        }
}
