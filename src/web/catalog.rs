use askama::Template;
use axum::extract::{Path, State};
use axum::response::Html;
use sqlx::SqlitePool;

use crate::db;
use crate::domain::catalog::{Category, CategoryId, Item, ItemId, Product, ProductId};
use crate::web::{AppError, AppResult};

// A raw path segment becomes a typed id here, at the boundary, or the
// request is over. An id that can't even be parsed names nothing — that's
// a 404, the same as an id that parses but isn't in the catalog. Nobody
// past this line handles a raw string.
fn parse_id<T: TryFrom<String>>(raw: String) -> Result<T, AppError> {
    T::try_from(raw).map_err(|_| AppError::NotFound)
}

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
