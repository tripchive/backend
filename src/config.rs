use std::sync::Arc;

use sqlx::PgPool;

macro_rules! env {
    ($key:expr) => {
        std::env::var($key).expect(concat!($key, " must be set"))
    };
}

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
            database_url: env!("DATABASE_URL"),
            jwt_secret: env!("JWT_SECRET"),
            jwt_expiration_hours: env!("JWT_EXPIRATION_HOURS")
                .parse()
                .expect("JWT_EXPIRATION_HOURS must be a valid number"),
            google: GoogleConfig {
                client_id: env!("GOOGLE_CLIENT_ID"),
                client_secret: env!("GOOGLE_CLIENT_SECRET"),
                redirect_url: env!("GOOGLE_REDIRECT_URL"),
            },
            github: GitHubConfig {
                client_id: env!("GITHUB_CLIENT_ID"),
                client_secret: env!("GITHUB_CLIENT_SECRET"),
                redirect_url: env!("GITHUB_REDIRECT_URL"),
            },
            apple: AppleConfig {
                client_id: env!("APPLE_CLIENT_ID"),
                team_id: env!("APPLE_TEAM_ID"),
                key_id: env!("APPLE_KEY_ID"),
                private_key: env!("APPLE_PRIVATE_KEY"),
                redirect_url: env!("APPLE_REDIRECT_URL"),
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
