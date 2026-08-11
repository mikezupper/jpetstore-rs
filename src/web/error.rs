use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

// The single error type every handler returns. Variants get added when a
// lesson introduces a new failure source (sqlx arrives in lesson 3), so the
// enum is always an honest list of everything that can actually go wrong.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("That page doesn't exist.")]
    NotFound,
    #[error("The page failed to render.")]
    Template(#[from] askama::Error),
    #[error("A database query failed.")]
    Database(#[from] sqlx::Error),
    #[error("The session store failed.")]
    Session(#[from] tower_sessions::session::Error),
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    status: u16,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Template(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Session(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let page = ErrorTemplate {
            status: status.as_u16(),
            message: self.to_string(),
        };
        match page.render() {
            Ok(html) => (status, Html(html)).into_response(),
            // If even the error page won't render, fall back to plain text
            // rather than recursing into ourselves.
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "error page failed to render").into_response(),
        }
    }
}
