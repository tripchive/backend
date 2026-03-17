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
}
