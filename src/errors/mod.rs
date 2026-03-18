use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

pub mod auth;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Auth(#[from] auth::AuthError),

    #[error("Internal server error: {0}")]
    Internal(String),
}

macro_rules! impl_internal_from {
    ($($t:ty),+ $(,)?) => {
        $(
            impl From<$t> for AppError {
                fn from(err: $t) -> Self {
                    Self::Internal(err.to_string())
                }
            }
        )+
    };
}

impl_internal_from!(
    sqlx::Error,
    argon2::password_hash::Error,
    jsonwebtoken::errors::Error,
    oauth2::url::ParseError,
    reqwest::Error,
);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Auth(err) => (err.status_code(), err.to_string()),
            Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal server error occurred".to_string(),
            ),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
