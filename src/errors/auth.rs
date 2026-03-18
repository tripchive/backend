use axum::http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid email or password")]
    InvalidCredentials,

    #[error("Account already exists with this email")]
    AccountAlreadyExists,

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Missing authentication token")]
    MissingToken,

    #[error("CSRF token mismatch")]
    CsrfMismatch,
}

impl AuthError {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCredentials | Self::InvalidToken | Self::MissingToken => {
                StatusCode::UNAUTHORIZED
            }
            Self::AccountAlreadyExists => StatusCode::CONFLICT,
            Self::CsrfMismatch => StatusCode::BAD_REQUEST,
        }
    }
}
