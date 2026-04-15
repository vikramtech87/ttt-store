use std::sync::Arc;
use axum::Router;
use crate::app::{create_router, AppState};
use crate::db::init_pool;
use crate::db::users::UserRepo;

pub mod db;
pub mod models;
pub mod app;
pub mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let pool = init_pool().await?;

    let state = Arc::new(AppState {
        user_repo: UserRepo::new(pool),
    });

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("🚀 Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}