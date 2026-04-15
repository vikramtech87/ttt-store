use crate::db::users::UserRepo;
use axum::{Router, routing::{get}};
use std::sync::Arc;
use crate::handlers::users::{self, UserState};

#[derive(Clone)]
pub struct AppState {
    pub user_repo: UserRepo,
}

pub type SharedState = Arc<AppState>;

impl UserState for SharedState {
    fn user_repo(&self) -> &UserRepo {
        &self.user_repo
    }
}

pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .nest("/users", users::router())
        .with_state(state)
}

async fn health_check() -> &'static str {
    "Ok"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_pool;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    async fn get_configured_router() -> Router {
        let pool = init_pool().await.unwrap();
        let user_repo = UserRepo::new(pool);
        let state = Arc::new(AppState { user_repo });
        create_router(state)
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = get_configured_router().await;
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }


}
