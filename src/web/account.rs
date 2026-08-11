use askama::Template;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use sqlx::SqlitePool;
use tower_sessions::Session;

use crate::domain::account::NewAccount;
use crate::web::{AppError, AppResult};
use crate::{auth, db};

const USER_KEY: &str = "user";

/// A signed-in username, proven by existing. Handlers that take an AuthUser
/// argument are gated: axum runs this extractor first, and a visitor with
/// no session user never reaches the handler body — they're already on
/// their way to /signin.
pub struct AuthUser(pub String);

impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = Redirect;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Redirect> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| Redirect::to("/signin"))?;
        match session.get::<String>(USER_KEY).await {
            Ok(Some(username)) => Ok(AuthUser(username)),
            _ => Err(Redirect::to("/signin")),
        }
    }
}

// ---- sign in ----

#[derive(Template)]
#[template(path = "signin.html")]
struct SigninTemplate {
    error: Option<String>,
}

pub async fn signin_form() -> AppResult<Html<String>> {
    Ok(Html(SigninTemplate { error: None }.render()?))
}

#[derive(serde::Deserialize)]
pub struct SigninForm {
    username: String,
    password: String,
}

pub async fn signin(
    State(pool): State<SqlitePool>,
    session: Session,
    Form(form): Form<SigninForm>,
) -> AppResult<Response> {
    let stored = db::account::password_hash(&pool, &form.username).await?;

    // Unknown user and wrong password get the same message — the form
    // shouldn't confirm which usernames exist.
    let ok = stored.as_deref().is_some_and(|hash| auth::verify(hash, &form.password));
    if !ok {
        let page = SigninTemplate { error: Some("Wrong username or password.".into()) };
        return Ok(Html(page.render()?).into_response());
    }

    // A fresh session id on every privilege change: whatever id the browser
    // carried while anonymous is worthless after sign-in (session fixation).
    // The session *data* — the cart — survives the id swap.
    session.cycle_id().await?;
    session.insert(USER_KEY, &form.username).await?;
    Ok(Redirect::to("/account").into_response())
}

pub async fn signout(session: Session) -> AppResult<Redirect> {
    // Everything goes, cart included — same as the original's signoff
    // invalidating the whole HttpSession.
    session.flush().await?;
    Ok(Redirect::to("/"))
}

// ---- register ----

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    error: Option<String>,
}

pub async fn register_form() -> AppResult<Html<String>> {
    Ok(Html(RegisterTemplate { error: None }.render()?))
}

#[derive(serde::Deserialize)]
pub struct RegisterForm {
    username: String,
    password: String,
    email: String,
    first_name: String,
    last_name: String,
    address: String,
    city: String,
    state: String,
    zip: String,
    country: String,
    phone: String,
}

pub async fn register(
    State(pool): State<SqlitePool>,
    session: Session,
    Form(form): Form<RegisterForm>,
) -> AppResult<Response> {
    let username = form.username.trim().to_string();
    if username.is_empty() || username.len() > 80 || form.password.len() < 4 {
        let page = RegisterTemplate {
            error: Some("Username is required, and the password needs at least 4 characters.".into()),
        };
        return Ok(Html(page.render()?).into_response());
    }

    let account = NewAccount {
        username: username.clone(),
        email: form.email,
        first_name: form.first_name,
        last_name: form.last_name,
        address: form.address,
        city: form.city,
        state: form.state,
        zip: form.zip,
        country: form.country,
        phone: form.phone,
    };

    let stored_hash = auth::hash_password(&form.password)?;
    match db::account::create(&pool, &account, &stored_hash).await {
        Ok(()) => {
            session.cycle_id().await?;
            session.insert(USER_KEY, &username).await?;
            Ok(Redirect::to("/account").into_response())
        }
        Err(err) if db::account::is_unique_violation(&err) => {
            let page = RegisterTemplate { error: Some("That username is taken.".into()) };
            Ok(Html(page.render()?).into_response())
        }
        Err(err) => Err(AppError::from(err)),
    }
}

// ---- the gated page ----

#[derive(Template)]
#[template(path = "account.html")]
struct AccountTemplate {
    username: String,
}

pub async fn account(AuthUser(username): AuthUser) -> AppResult<Html<String>> {
    Ok(Html(AccountTemplate { username }.render()?))
}
