use std::sync::Arc;

use sqlx::PgPool;

pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
}

pub struct GitHubConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
}

pub struct AppleConfig {
    pub client_id: String,
    pub team_id: String,
    pub key_id: String,
    pub private_key: String,
    pub redirect_url: String,
}

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub google: GoogleConfig,
    pub github: GitHubConfig,
    pub apple: AppleConfig,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            jwt_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_EXPIRATION_HOURS must be a valid number"),
            google: GoogleConfig {
                client_id: std::env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"),
                client_secret: std::env::var("GOOGLE_CLIENT_SECRET")
                    .expect("GOOGLE_CLIENT_SECRET must be set"),
                redirect_url: std::env::var("GOOGLE_REDIRECT_URL")
                    .expect("GOOGLE_REDIRECT_URL must be set"),
            },
            github: GitHubConfig {
                client_id: std::env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set"),
                client_secret: std::env::var("GITHUB_CLIENT_SECRET")
                    .expect("GITHUB_CLIENT_SECRET must be set"),
                redirect_url: std::env::var("GITHUB_REDIRECT_URL")
                    .expect("GITHUB_REDIRECT_URL must be set"),
            },
            apple: AppleConfig {
                client_id: std::env::var("APPLE_CLIENT_ID").expect("APPLE_CLIENT_ID must be set"),
                team_id: std::env::var("APPLE_TEAM_ID").expect("APPLE_TEAM_ID must be set"),
                key_id: std::env::var("APPLE_KEY_ID").expect("APPLE_KEY_ID must be set"),
                private_key: std::env::var("APPLE_PRIVATE_KEY")
                    .expect("APPLE_PRIVATE_KEY must be set"),
                redirect_url: std::env::var("APPLE_REDIRECT_URL")
                    .expect("APPLE_REDIRECT_URL must be set"),
            },
        }
    }
}

pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub http_client: reqwest::Client,
}

pub type SharedState = Arc<AppState>;
