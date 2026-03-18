use axum_extra::extract::CookieJar;
use cookie::{Cookie, SameSite, time::Duration};
use hmac::{Hmac, Mac};
use oauth2::CsrfToken;
use sha2::Sha256;

use crate::errors::{Result, auth::AuthError};

pub fn create_csrf_cookie(token: &CsrfToken, secret: &str) -> Cookie<'static> {
    let signature = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size")
        .chain_update(token.secret().as_bytes())
        .finalize()
        .into_bytes();

    let value = format!("{}.{}", token.secret(), hex::encode(signature));

    Cookie::build(("oauth_csrf", value))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(true)
        .max_age(Duration::minutes(15))
        .build()
}

pub fn verify_csrf_cookie(jar: &CookieJar, state_param: &str, secret: &str) -> Result<()> {
    jar.get("oauth_csrf")
        .map(Cookie::value)
        .and_then(|cookie_value| {
            let (token, sig) = cookie_value.rsplit_once('.')?;
            let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).ok()?;
            mac.update(token.as_bytes());
            mac.verify_slice(&hex::decode(sig).ok()?).ok()?;
            (token == state_param).then_some(())
        })
        .ok_or_else(|| AuthError::CsrfMismatch.into())
}
