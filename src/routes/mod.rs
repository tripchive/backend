use axum::Router;

use crate::config::SharedState;

pub mod auth;

pub fn router() -> Router<SharedState> {
    Router::new().nest("/auth", auth::router())
}
