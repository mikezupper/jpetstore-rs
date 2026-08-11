mod error;
mod home;

pub use error::{AppError, AppResult};

use axum::{routing::get, Router};
use sqlx::SqlitePool;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(home::home))
        .fallback(not_found)
        .with_state(pool)
}

// Any path we don't route is a plain 404 — returned as a value, rendered
// by AppError's IntoResponse impl, never a panic.
async fn not_found() -> AppError {
    AppError::NotFound
}
