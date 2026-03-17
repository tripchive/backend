use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;

use crate::config::SharedState;
use crate::errors::AppError;
use crate::errors::auth::AuthError;

use super::jwt;

pub struct AuthUser(pub i64);

impl FromRequestParts<SharedState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SharedState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AuthError::MissingToken)?;

        let claims = jwt::decode_token(bearer.token(), &state.config.jwt_secret)?;
        Ok(AuthUser(claims.sub))
    }
}
