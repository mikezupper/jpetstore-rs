use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use sqlx::SqlitePool;
use tower_sessions::Session;

use crate::db;
use crate::db::order::PlaceOrderError;
use crate::domain::cart::Cart;
use crate::domain::order::OrderDraft;
use crate::web::account::AuthUser;
use crate::web::{cart, checkout, AppError, AppResult};

pub async fn place(
    AuthUser(username): AuthUser,
    State(pool): State<SqlitePool>,
    session: Session,
) -> AppResult<Response> {
    // Both preconditions re-checked at the moment of truth — the confirm
    // page's checks were a different request, and sessions drift between
    // requests. A refresh after success hits the missing-draft arm and
    // bounces harmlessly: you can't place the same draft twice.
    let Some(draft) = session.get::<OrderDraft>(checkout::DRAFT_KEY).await? else {
        return Ok(Redirect::to("/checkout").into_response());
    };
    let cart_now = cart::load(&session).await?;
    if cart_now.is_empty() {
        return Ok(Redirect::to("/cart").into_response());
    }

    match db::order::place(&pool, &username, &draft, &cart_now).await {
        Ok(order_id) => {
            session.remove::<OrderDraft>(checkout::DRAFT_KEY).await?;
            cart::save(&session, &Cart::default()).await?;
            Ok(Redirect::to(&format!("/orders/placed/{order_id}")).into_response())
        }
        Err(PlaceOrderError::OutOfStock(item_id)) => {
            let page = OrderFailedTemplate {
                message: format!("Not enough stock for {item_id}. Adjust your cart and try again."),
            };
            Ok(Html(page.render()?).into_response())
        }
        Err(PlaceOrderError::Db(err)) => Err(AppError::from(err)),
    }
}

#[derive(Template)]
#[template(path = "order-failed.html")]
struct OrderFailedTemplate {
    message: String,
}

#[derive(Template)]
#[template(path = "order-placed.html")]
struct OrderPlacedTemplate {
    order_id: i64,
}

pub async fn placed(
    AuthUser(username): AuthUser,
    State(pool): State<SqlitePool>,
    Path(order_id): Path<i64>,
) -> AppResult<Html<String>> {
    // Order ids are sequential, so guessing them is trivial — the ownership
    // check is what stands between /orders/placed/7 and everyone's orders.
    // "Not yours" and "doesn't exist" get the same 404: a URL that names
    // nothing of yours names nothing.
    match db::order::owner(&pool, order_id).await? {
        Some(owner) if owner == username => {
            Ok(Html(OrderPlacedTemplate { order_id }.render()?))
        }
        _ => Err(AppError::NotFound),
    }
}
