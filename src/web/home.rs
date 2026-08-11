use askama::Template;
use axum::response::Html;

use crate::web::AppResult;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate;

// The `?` propagates a render failure into AppError::Template via the
// #[from] conversion — this is the whole error-handling pattern the port
// uses, visible in one line.
pub async fn home() -> AppResult<Html<String>> {
    Ok(Html(HomeTemplate.render()?))
}
