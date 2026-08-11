use askama::Template;
use axum::extract::{Path, Query, State};
use axum::response::Html;
use sqlx::SqlitePool;

use crate::db;
use crate::domain::catalog::{Category, CategoryId, Item, ItemId, Product, ProductId};
use crate::web::{parse_id, AppError, AppResult};

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    categories: Vec<Category>,
}

pub async fn home(State(pool): State<SqlitePool>) -> AppResult<Html<String>> {
    let categories = db::catalog::categories(&pool).await?;
    Ok(Html(HomeTemplate { categories }.render()?))
}

#[derive(Template)]
#[template(path = "category.html")]
struct CategoryTemplate {
    category: Category,
    products: Vec<Product>,
}

pub async fn category(
    State(pool): State<SqlitePool>,
    Path(raw): Path<String>,
) -> AppResult<Html<String>> {
    let id: CategoryId = parse_id(raw)?;
    let category = db::catalog::category(&pool, &id).await?.ok_or(AppError::NotFound)?;
    let products = db::catalog::products_in_category(&pool, &id).await?;
    Ok(Html(CategoryTemplate { category, products }.render()?))
}

#[derive(Template)]
#[template(path = "product.html")]
struct ProductTemplate {
    product: Product,
    items: Vec<Item>,
}

pub async fn product(
    State(pool): State<SqlitePool>,
    Path(raw): Path<String>,
) -> AppResult<Html<String>> {
    let id: ProductId = parse_id(raw)?;
    let product = db::catalog::product(&pool, &id).await?.ok_or(AppError::NotFound)?;
    let items = db::catalog::items_for_product(&pool, &id).await?;
    Ok(Html(ProductTemplate { product, items }.render()?))
}

// The query string, parsed into a struct by serde before the handler runs.
// #[serde(default)] makes a bare /search (no ?keyword=) mean "empty search"
// instead of a 400 — the form page itself is a valid page to land on.
#[derive(serde::Deserialize)]
pub struct SearchParams {
    #[serde(default)]
    keyword: String,
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    keyword: String,
    products: Vec<Product>,
}

pub async fn search(
    State(pool): State<SqlitePool>,
    Query(params): Query<SearchParams>,
) -> AppResult<Html<String>> {
    let keyword = params.keyword.trim();
    let products = if keyword.is_empty() {
        Vec::new()
    } else {
        db::catalog::search_products(&pool, keyword).await?
    };
    Ok(Html(SearchTemplate { keyword: keyword.to_string(), products }.render()?))
}

#[derive(Template)]
#[template(path = "item.html")]
struct ItemTemplate {
    item: Item,
    product: Product,
}

pub async fn item(
    State(pool): State<SqlitePool>,
    Path(raw): Path<String>,
) -> AppResult<Html<String>> {
    let id: ItemId = parse_id(raw)?;
    let item = db::catalog::item(&pool, &id).await?.ok_or(AppError::NotFound)?;
    let product = db::catalog::product(&pool, &item.product_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Html(ItemTemplate { item, product }.render()?))
}
