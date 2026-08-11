mod account;
mod assets;
mod cart;
mod catalog;
mod error;

pub use error::{AppError, AppResult};

use axum::routing::{get, post};
use axum::Router;
use sqlx::SqlitePool;
use tower_sessions::{MemoryStore, SessionManagerLayer};

pub fn router(pool: SqlitePool) -> Router {
    // Sessions live in memory for now: a restart empties every cart, the
    // same amnesia the original's in-memory HSQLDB had. A durable store
    // arrives with accounts in lesson 8. with_secure(false) because dev is
    // plain http; the deployment story (and https) is the paid course.
    let sessions = SessionManagerLayer::new(MemoryStore::default()).with_secure(false);

    Router::new()
        .route("/", get(catalog::home))
        .route("/search", get(catalog::search))
        .route("/categories/{id}", get(catalog::category))
        .route("/products/{id}", get(catalog::product))
        .route("/items/{id}", get(catalog::item))
        .route("/images/{file}", get(assets::image))
        .route("/cart", get(cart::view))
        .route("/cart/items", post(cart::add))
        .route("/cart/update", post(cart::update))
        .route("/cart/remove", post(cart::remove))
        .route("/signin", get(account::signin_form).post(account::signin))
        .route("/register", get(account::register_form).post(account::register))
        .route("/signout", post(account::signout))
        .route("/account", get(account::account))
        .fallback(not_found)
        .layer(sessions)
        .with_state(pool)
}

// A raw path or form value becomes a typed id here, at the boundary, or the
// request is over. An id that can't parse names nothing: 404, same as an id
// that parses but isn't in the catalog.
pub(crate) fn parse_id<T: TryFrom<String>>(raw: String) -> Result<T, AppError> {
    T::try_from(raw).map_err(|_| AppError::NotFound)
}

// Any path we don't route is a plain 404 — returned as a value, rendered
// by AppError's IntoResponse impl, never a panic.
async fn not_found() -> AppError {
    AppError::NotFound
}
