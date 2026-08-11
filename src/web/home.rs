use askama::Template;
use axum::extract::State;
use axum::response::Html;
use sqlx::SqlitePool;

use crate::web::AppResult;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    categories: i64,
    products: i64,
}

// State<SqlitePool> is the DataSource injection point: axum hands every
// handler whatever was registered with .with_state(). The two counts are
// runtime-checked queries — lesson 4 upgrades to compile-time checking.
pub async fn home(State(pool): State<SqlitePool>) -> AppResult<Html<String>> {
    let categories: i64 = sqlx::query_scalar("SELECT count(*) FROM category")
        .fetch_one(&pool)
        .await?;
    let products: i64 = sqlx::query_scalar("SELECT count(*) FROM product")
        .fetch_one(&pool)
        .await?;

    Ok(Html(HomeTemplate { categories, products }.render()?))
}
