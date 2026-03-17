use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl, basic::BasicClient,
};
use serde::Deserialize;

use super::OAuthClient;
use crate::config::GitHubConfig;
use crate::errors::{AppError, Result};

#[derive(Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
}

pub fn build_client(config: &GitHubConfig) -> Result<OAuthClient> {
    Ok(BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_client_secret(ClientSecret::new(config.client_secret.clone()))
        .set_auth_uri(AuthUrl::new(
            "https://github.com/login/oauth/authorize".to_string(),
        )?)
        .set_token_uri(TokenUrl::new(
            "https://github.com/login/oauth/access_token".to_string(),
        )?)
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.clone())?))
}

pub fn get_authorize_url(client: &OAuthClient) -> (String, CsrfToken) {
    let (url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
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

    Ok(token.access_token().secret().to_string())
}

pub async fn fetch_user_info(
    http_client: &reqwest::Client,
    access_token: &str,
) -> Result<GitHubUser> {
    let mut user = http_client
        .get("https://api.github.com/user")
        .bearer_auth(access_token)
        .header("User-Agent", "tripchive")
        .send()
        .await?
        .json::<GitHubUser>()
        .await?;

    if user.email.is_none() {
        let emails: Vec<GitHubEmail> = http_client
            .get("https://api.github.com/user/emails")
            .bearer_auth(access_token)
            .header("User-Agent", "tripchive")
            .send()
            .await?
            .json()
            .await?;

        user.email = emails.into_iter().find(|e| e.primary).map(|e| e.email);
    }

    Ok(user)
}
