use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header};
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl, basic::BasicClient,
};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

use super::OAuthClient;
use crate::config::AppleConfig;
use crate::errors::Result;

#[derive(Deserialize)]
pub struct AppleUser {
    pub id: String,
    pub email: Option<String>,
}

#[derive(Serialize)]
struct AppleClientSecretClaims {
    iss: String,
    sub: String,
    aud: String,
    iat: usize,
    exp: usize,
}

#[derive(Deserialize)]
struct AppleIdTokenClaims {
    sub: String,
    email: Option<String>,
}

fn generate_client_secret(config: &AppleConfig) -> Result<String> {
    let now = Utc::now();

    let claims = AppleClientSecretClaims {
        iss: config.team_id.clone(),
        sub: config.client_id.clone(),
        aud: "https://appleid.apple.com".to_string(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::days(180)).timestamp() as usize,
    };

    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(config.key_id.clone());

    let key = EncodingKey::from_ec_pem(config.private_key.as_bytes())?;

    Ok(jsonwebtoken::encode(&header, &claims, &key)?)
}

pub fn build_client(config: &AppleConfig) -> Result<OAuthClient> {
    let client_secret = generate_client_secret(config)?;

    Ok(BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_auth_uri(AuthUrl::new(
            "https://appleid.apple.com/auth/authorize".to_string(),
        )?)
        .set_token_uri(TokenUrl::new(
            "https://appleid.apple.com/auth/token".to_string(),
        )?)
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.clone())?))
}

pub fn get_authorize_url(client: &OAuthClient) -> (String, CsrfToken) {
    let (url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("name".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_extra_param("response_mode", "form_post")
        .url();

    (url.to_string(), csrf_token)
}

#[derive(Deserialize)]
struct AppleTokenResponse {
    id_token: String,
}

pub async fn exchange_code(
    config: &AppleConfig,
    http_client: &reqwest::Client,
    code: String,
) -> Result<String> {
    let client_secret = generate_client_secret(config)?;

    let resp = http_client
        .post("https://appleid.apple.com/auth/token")
        .form(&[
            ("client_id", config.client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", config.redirect_url.as_str()),
        ])
        .send()
        .await?
        .json::<AppleTokenResponse>()
        .await?;

    Ok(resp.id_token)
}

#[derive(Deserialize)]
struct AppleJwk {
    kid: String,
    n: String,
    e: String,
}

#[derive(Deserialize)]
struct AppleJwks {
    keys: Vec<AppleJwk>,
}

pub async fn decode_id_token(
    id_token: &str,
    config: &AppleConfig,
    http_client: &reqwest::Client,
) -> Result<AppleUser> {
    let header = jsonwebtoken::decode_header(id_token)?;
    let kid = header
        .kid
        .ok_or_else(|| AppError::Internal("missing kid in id_token header".to_string()))?;

    let jwks = http_client
        .get("https://appleid.apple.com/auth/keys")
        .send()
        .await?
        .json::<AppleJwks>()
        .await?;

    let jwk = jwks
        .keys
        .iter()
        .find(|k| k.kid == kid)
        .ok_or_else(|| AppError::Internal("no matching key in Apple JWKS".to_string()))?;

    let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;

    let mut validation = jsonwebtoken::Validation::new(Algorithm::RS256);
    validation.set_audience(&[&config.client_id]);
    validation.set_issuer(&["https://appleid.apple.com"]);

    let token_data = jsonwebtoken::decode::<AppleIdTokenClaims>(id_token, &key, &validation)?;

    Ok(AppleUser {
        id: token_data.claims.sub,
        email: token_data.claims.email,
    })
}
