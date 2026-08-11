mod error;
mod home;

pub use error::{AppError, AppResult};

use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new()
        .route("/", get(home::home))
        .fallback(not_found)
}

// Any path we don't route is a plain 404 — returned as a value, rendered
// by AppError's IntoResponse impl, never a panic.
async fn not_found() -> AppError {
    AppError::NotFound
}
