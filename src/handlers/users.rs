use crate::models::User;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use crate::db::users::UserRepo;

pub trait UserState: Clone + Send + Sync + 'static {
    fn user_repo(&self) -> &UserRepo;
}

#[derive(Deserialize)]
pub struct CreateUserRequestDto {
    pub email: String,
    pub full_name: String,
}

pub fn router<S>() -> Router<S>
where
    S: UserState,
{
    Router::new().route("/", post(create_user_handler::<S>))
}

async fn create_user_handler<S>(
    State(state): State<S>,
    Json(payload): Json<CreateUserRequestDto>,
) -> Result<(StatusCode, Json<User>), StatusCode>
where
    S: UserState,
{
    let user = state
        .user_repo()
        .create_user(&payload.email, &payload.full_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(user)))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{header, Method};
    use serde_json::json;
    use crate::db::init_pool;
    use axum::extract::Request;
    use tower::ServiceExt;
    use super::*;

    #[derive(Clone)]
    struct TestState { repo: UserRepo }

    impl UserState for TestState {
        fn user_repo(&self) -> &UserRepo {
            &self.repo
        }
    }

    async fn get_mock_router() -> Router {
        let pool = init_pool().await.unwrap();
        let state = TestState { repo: UserRepo::new(pool) };

        Router::new()
            .nest("/users", router::<TestState>())
            .with_state(state)
    }

    #[tokio::test]
    async fn test_create_user_handler() {
        let app = get_mock_router().await;

        let payload = json!({
            "email": format!("api-{}@example.com", uuid::Uuid::new_v4()),
            "full_name": "API user",
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/users")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}