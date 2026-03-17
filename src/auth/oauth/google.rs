use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl, basic::BasicClient,
};
use serde::Deserialize;

use super::OAuthClient;
use crate::config::GoogleConfig;
use crate::errors::{AppError, Result};

#[derive(Deserialize)]
pub struct GoogleUser {
    pub id: String,
    pub email: String,
}

pub fn build_client(config: &GoogleConfig) -> Result<OAuthClient> {
    Ok(BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_client_secret(ClientSecret::new(config.client_secret.clone()))
        .set_auth_uri(AuthUrl::new(
            "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        )?)
        .set_token_uri(TokenUrl::new(
            "https://oauth2.googleapis.com/token".to_string(),
        )?)
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.clone())?))
}

pub fn get_authorize_url(client: &OAuthClient) -> (String, CsrfToken) {
    let (url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    (url.to_string(), csrf_token)
}

pub async fn exchange_code(
    client: &OAuthClient,
    http_client: &reqwest::Client,
    code: String,
) -> Result<String> {
    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(http_client)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(token.access_token().secret().clone())
}

pub async fn fetch_user_info(
    http_client: &reqwest::Client,
    access_token: &str,
) -> Result<GoogleUser> {
    Ok(http_client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<GoogleUser>()
        .await?)
}
