use askama::Template;
use axum::extract::State;
use axum::response::{Html, Redirect};
use axum::Form;
use sqlx::SqlitePool;
use tower_sessions::Session;

use crate::db;
use crate::domain::cart::Cart;
use crate::domain::catalog::ItemId;
use crate::web::{parse_id, AppError, AppResult};

const CART_KEY: &str = "cart";

// The session stores the cart under one key; these two helpers are the
// only code that knows that. A missing cart and an empty cart are the
// same thing, which is why load() defaults instead of erroring.
pub(crate) async fn load(session: &Session) -> AppResult<Cart> {
    Ok(session.get::<Cart>(CART_KEY).await?.unwrap_or_default())
}

async fn save(session: &Session, cart: &Cart) -> AppResult<()> {
    Ok(session.insert(CART_KEY, cart).await?)
}

#[derive(Template)]
#[template(path = "cart.html")]
struct CartTemplate {
    cart: Cart,
}

pub async fn view(session: Session) -> AppResult<Html<String>> {
    let cart = load(&session).await?;
    Ok(Html(CartTemplate { cart }.render()?))
}

#[derive(serde::Deserialize)]
pub struct AddForm {
    item_id: String,
}

pub async fn add(
    State(pool): State<SqlitePool>,
    session: Session,
    Form(form): Form<AddForm>,
) -> AppResult<Redirect> {
    let id: ItemId = parse_id(form.item_id)?;
    let item = db::catalog::item(&pool, &id).await?.ok_or(AppError::NotFound)?;
    let product = db::catalog::product(&pool, &item.product_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let name = match &item.attribute {
        Some(attr) => format!("{attr} {}", product.name),
        None => product.name.clone(),
    };

    let mut cart = load(&session).await?;
    cart.add(item.id, name, item.list_price);
    save(&session, &cart).await?;

    // POST, then redirect, then GET — so a refresh of the cart page rerenders
    // instead of re-adding a fish.
    Ok(Redirect::to("/cart"))
}

#[derive(serde::Deserialize)]
pub struct UpdateForm {
    item_id: String,
    quantity: u32,
}

pub async fn update(session: Session, Form(form): Form<UpdateForm>) -> AppResult<Redirect> {
    let id: ItemId = parse_id(form.item_id)?;
    let mut cart = load(&session).await?;
    cart.set_quantity(&id, form.quantity);
    save(&session, &cart).await?;
    Ok(Redirect::to("/cart"))
}

#[derive(serde::Deserialize)]
pub struct RemoveForm {
    item_id: String,
}

pub async fn remove(session: Session, Form(form): Form<RemoveForm>) -> AppResult<Redirect> {
    let id: ItemId = parse_id(form.item_id)?;
    let mut cart = load(&session).await?;
    cart.remove(&id);
    save(&session, &cart).await?;
    Ok(Redirect::to("/cart"))
}
