pub mod users;
pub mod auth;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::env;

pub async fn init_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;

    #[tokio::test]
    async fn test_db_connection_and_schema_access() {
        dotenv().ok();
        let pool = init_pool().await.expect("Failed to create pool");

        let row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch row");

        assert_eq!(row.0, 1);
    }
}
