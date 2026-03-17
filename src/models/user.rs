use serde::Serialize;
use sqlx::{FromRow, PgPool};

use crate::errors::Result;

#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: Option<String>,
    pub auth_provider: String,
    pub provider_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<Self>> {
        let user = sqlx::query_as::<_, Self>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn find_by_provider(
        pool: &PgPool,
        provider: &str,
        provider_id: &str,
    ) -> Result<Option<Self>> {
        let user = sqlx::query_as::<_, Self>(
            "SELECT * FROM users WHERE auth_provider = $1 AND provider_id = $2",
        )
        .bind(provider)
        .bind(provider_id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn create_email_user(
        pool: &PgPool,
        email: &str,
        password_hash: &str,
    ) -> Result<Self> {
        let user = sqlx::query_as::<_, Self>(
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING *",
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn upsert_oauth_user(
        pool: &PgPool,
        provider: &str,
        provider_id: &str,
        email: Option<&str>,
    ) -> Result<Self> {
        let user = sqlx::query_as::<_, Self>(
            "INSERT INTO users (email, auth_provider, provider_id) VALUES ($1, $2, $3) \
             ON CONFLICT (auth_provider, provider_id) DO UPDATE SET email = EXCLUDED.email \
             RETURNING *",
        )
        .bind(email)
        .bind(provider)
        .bind(provider_id)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }
}
