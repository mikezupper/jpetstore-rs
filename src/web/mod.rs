mod assets;
mod catalog;
mod error;

pub use error::{AppError, AppResult};

use axum::{routing::get, Router};
use sqlx::SqlitePool;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(catalog::home))
        .route("/search", get(catalog::search))
        .route("/categories/{id}", get(catalog::category))
        .route("/products/{id}", get(catalog::product))
        .route("/items/{id}", get(catalog::item))
        .route("/images/{file}", get(assets::image))
        .fallback(not_found)
        .with_state(pool)
}

// Any path we don't route is a plain 404 — returned as a value, rendered
// by AppError's IntoResponse impl, never a panic.
async fn not_found() -> AppError {
    AppError::NotFound
}
