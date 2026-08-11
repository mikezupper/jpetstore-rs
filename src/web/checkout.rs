use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use sqlx::SqlitePool;
use tower_sessions::Session;

use crate::db;
use crate::domain::cart::Cart;
use crate::domain::order::{Address, CardType, OrderDraft};
use crate::web::account::AuthUser;
use crate::web::{cart, AppError, AppResult};

pub(crate) const DRAFT_KEY: &str = "order_draft";

#[derive(Template)]
#[template(path = "checkout.html")]
struct CheckoutTemplate {
    bill: Address,
    error: Option<String>,
}

// Checkout is gated twice before any form logic runs: AuthUser proves
// sign-in, and an empty cart bounces to /cart — there is nothing to check
// out. Billing prefills from the account, like the original.
pub async fn form(
    AuthUser(username): AuthUser,
    State(pool): State<SqlitePool>,
    session: Session,
) -> AppResult<Response> {
    if cart::load(&session).await?.is_empty() {
        return Ok(Redirect::to("/cart").into_response());
    }
    let bill = db::account::address(&pool, &username).await?.ok_or(AppError::NotFound)?;
    Ok(Html(CheckoutTemplate { bill, error: None }.render()?).into_response())
}

#[derive(serde::Deserialize)]
pub struct CheckoutForm {
    ship_first_name: String,
    ship_last_name: String,
    ship_address: String,
    ship_city: String,
    ship_state: String,
    ship_zip: String,
    ship_country: String,
    bill_first_name: String,
    bill_last_name: String,
    bill_address: String,
    bill_city: String,
    bill_state: String,
    bill_zip: String,
    bill_country: String,
    card_type: String,
    card_number: String,
}

pub async fn submit(
    AuthUser(username): AuthUser,
    State(pool): State<SqlitePool>,
    session: Session,
    Form(form): Form<CheckoutForm>,
) -> AppResult<Response> {
    let ship = Address {
        first_name: form.ship_first_name,
        last_name: form.ship_last_name,
        address: form.ship_address,
        city: form.ship_city,
        state: form.ship_state,
        zip: form.ship_zip,
        country: form.ship_country,
    };
    let bill = Address {
        first_name: form.bill_first_name,
        last_name: form.bill_last_name,
        address: form.bill_address,
        city: form.bill_city,
        state: form.bill_state,
        zip: form.bill_zip,
        country: form.bill_country,
    };

    let card_type = CardType::try_from(form.card_type).ok();
    let valid = ship.is_complete()
        && bill.is_complete()
        && card_type.is_some()
        && !form.card_number.trim().is_empty();

    let Some(card_type) = card_type.filter(|_| valid) else {
        let prefill = db::account::address(&pool, &username).await?.ok_or(AppError::NotFound)?;
        let page = CheckoutTemplate {
            bill: prefill,
            error: Some("Every address field, the card type, and a card number are required.".into()),
        };
        return Ok(Html(page.render()?).into_response());
    };

    // form.card_number's scope ends here. It was read to prove the flow
    // works, it validated as present, and it is now gone: OrderDraft has no
    // field for it, the schema has no column for it, and this comment is
    // the only place it gets mentioned again.
    session.insert(DRAFT_KEY, OrderDraft { ship, bill, card_type }).await?;
    Ok(Redirect::to("/checkout/confirm").into_response())
}

#[derive(Template)]
#[template(path = "confirm.html")]
struct ConfirmTemplate {
    draft: OrderDraft,
    cart: Cart,
}

pub async fn confirm(AuthUser(_): AuthUser, session: Session) -> AppResult<Response> {
    let Some(draft) = session.get::<OrderDraft>(DRAFT_KEY).await? else {
        return Ok(Redirect::to("/checkout").into_response());
    };
    let cart = cart::load(&session).await?;
    if cart.is_empty() {
        return Ok(Redirect::to("/cart").into_response());
    }
    Ok(Html(ConfirmTemplate { draft, cart }.render()?).into_response())
}
