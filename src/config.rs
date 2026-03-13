use sqlx::PgPool;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        }
    }
}

pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}
