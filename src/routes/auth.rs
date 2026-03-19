use axum::{
    Json, Router,
    extract::{Query, State},
    response::Redirect,
    routing::{get, post},
};
use axum_extra::extract::CookieJar;

use crate::{
    auth::{csrf, jwt, oauth, password},
    config::SharedState,
    dto::auth::{LoginRequest, OAuthCallbackParams, RegisterRequest, TokenResponse},
    errors::{Result, auth::AuthError},
    models::user::User,
};

fn create_token_response(user_id: i64, state: &SharedState) -> Result<Json<TokenResponse>> {
    let token = jwt::create_token(
        user_id,
        &state.config.jwt_secret,
        state.config.jwt_expiration_hours,
    )?;
    Ok(Json(TokenResponse {
        access_token: token,
    }))
}

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/google", get(google_redirect))
        .route("/google/callback", get(google_callback))
        .route("/github", get(github_redirect))
        .route("/github/callback", get(github_callback))
        .route("/apple", get(apple_redirect))
        .route("/apple/callback", post(apple_callback))
}

async fn register(
    State(state): State<SharedState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<TokenResponse>> {
    if User::find_by_email(&state.pool, &body.email)
        .await?
        .is_some()
    {
        return Err(AuthError::AccountAlreadyExists.into());
    }

    let hash = tokio::task::spawn_blocking(move || password::hash_password(&body.password)).await??;
    let user = User::create_email_user(&state.pool, &body.email, &hash).await?;
    create_token_response(user.id, &state)
}

async fn login(
    State(state): State<SharedState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<TokenResponse>> {
    let user = User::find_by_email(&state.pool, &body.email)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    if user.auth_provider != "email" {
        return Err(AuthError::InvalidCredentials.into());
    }

    let hash = user.password_hash.ok_or(AuthError::InvalidCredentials)?;
    if !password::verify_password(&body.password, &hash)? {
        return Err(AuthError::InvalidCredentials.into());
    }

    create_token_response(user.id, &state)
}

fn oauth_redirect(
    jar: CookieJar,
    url: &str,
    csrf_token: &oauth2::CsrfToken,
    secret: &str,
) -> (CookieJar, Redirect) {
    let cookie = csrf::create_csrf_cookie(csrf_token, secret);
    (jar.add(cookie), Redirect::temporary(url))
}

async fn google_redirect(
    jar: CookieJar,
    State(state): State<SharedState>,
) -> Result<(CookieJar, Redirect)> {
    let client = oauth::google::build_client(&state.config.google)?;
    let (url, csrf_token) = oauth::google::get_authorize_url(&client);

    Ok(oauth_redirect(
        jar,
        &url,
        &csrf_token,
        &state.config.jwt_secret,
    ))
}

async fn google_callback(
    jar: CookieJar,
    State(state): State<SharedState>,
    Query(params): Query<OAuthCallbackParams>,
) -> Result<Json<TokenResponse>> {
    csrf::verify_csrf_cookie(&jar, &params.state, &state.config.jwt_secret)?;

    let client = oauth::google::build_client(&state.config.google)?;
    let access_token =
        oauth::google::exchange_code(&client, &state.http_client, params.code).await?;
    let google_user = oauth::google::fetch_user_info(&state.http_client, &access_token).await?;

    let user = User::upsert_oauth_user(
        &state.pool,
        "google",
        &google_user.id,
        Some(&google_user.email),
    )
    .await?;
    create_token_response(user.id, &state)
}

async fn github_redirect(
    jar: CookieJar,
    State(state): State<SharedState>,
) -> Result<(CookieJar, Redirect)> {
    let client = oauth::github::build_client(&state.config.github)?;
    let (url, csrf_token) = oauth::github::get_authorize_url(&client);

    Ok(oauth_redirect(
        jar,
        &url,
        &csrf_token,
        &state.config.jwt_secret,
    ))
}

async fn github_callback(
    jar: CookieJar,
    State(state): State<SharedState>,
    Query(params): Query<OAuthCallbackParams>,
) -> Result<Json<TokenResponse>> {
    csrf::verify_csrf_cookie(&jar, &params.state, &state.config.jwt_secret)?;

    let client = oauth::github::build_client(&state.config.github)?;
    let access_token =
        oauth::github::exchange_code(&client, &state.http_client, params.code).await?;
    let github_user = oauth::github::fetch_user_info(&state.http_client, &access_token).await?;

    let user = User::upsert_oauth_user(
        &state.pool,
        "github",
        &github_user.id.to_string(),
        github_user.email.as_deref(),
    )
    .await?;
    create_token_response(user.id, &state)
}

async fn apple_redirect(
    jar: CookieJar,
    State(state): State<SharedState>,
) -> Result<(CookieJar, Redirect)> {
    let client = oauth::apple::build_client(&state.config.apple)?;
    let (url, csrf_token) = oauth::apple::get_authorize_url(&client);

    Ok(oauth_redirect(
        jar,
        &url,
        &csrf_token,
        &state.config.jwt_secret,
    ))
}

async fn apple_callback(
    jar: CookieJar,
    State(state): State<SharedState>,
    axum::Form(params): axum::Form<OAuthCallbackParams>,
) -> Result<Json<TokenResponse>> {
    csrf::verify_csrf_cookie(&jar, &params.state, &state.config.jwt_secret)?;

    let id_token =
        oauth::apple::exchange_code(&state.config.apple, &state.http_client, params.code).await?;
    let apple_user =
        oauth::apple::decode_id_token(&id_token, &state.config.apple, &state.http_client).await?;

    let user = User::upsert_oauth_user(
        &state.pool,
        "apple",
        &apple_user.id,
        apple_user.email.as_deref(),
    )
    .await?;
    create_token_response(user.id, &state)
}
