//! Request-level tests: the real router, the real migrations and seed data,
//! an in-memory database, and no network socket anywhere. `oneshot` pushes a
//! request through the whole tower stack — session layer included — and
//! hands back the response the browser would have seen.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use tower::ServiceExt;

async fn app() -> axum::Router {
    let pool = jpetstore_rs::db::pool("sqlite::memory:").await.expect("test db");
    jpetstore_rs::web::router(pool)
}

async fn text(response: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn the_home_page_lists_all_five_categories() {
    let response = app()
        .await
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let html = text(response).await;
    for category in ["Fish", "Dogs", "Cats", "Reptiles", "Birds"] {
        assert!(html.contains(category), "missing {category}");
    }
}

#[tokio::test]
async fn the_item_page_renders_the_price() {
    let response = app()
        .await
        .oneshot(Request::get("/items/EST-1").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(text(response).await.contains("$16.50"));
}

#[tokio::test]
async fn unknown_paths_get_the_styled_404() {
    let response = app()
        .await
        .oneshot(Request::get("/no/such/page").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(text(response).await.contains("Back to the store"));
}

#[tokio::test]
async fn the_embedded_images_serve_as_gifs() {
    let response = app()
        .await
        .oneshot(Request::get("/images/fish1.gif").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "image/gif");
}

#[tokio::test]
async fn add_to_cart_round_trips_through_the_session() {
    let app = app().await;

    let response = app
        .clone()
        .oneshot(
            Request::post("/cart/items")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from("item_id=EST-1"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    // The session cookie is the thread between requests — exactly what a
    // browser would carry back.
    let cookie = response.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    let response = app
        .oneshot(
            Request::get("/cart")
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let html = text(response).await;
    assert!(html.contains("EST-1"));
    assert!(html.contains("$16.50"));
}

#[tokio::test]
async fn gated_pages_redirect_anonymous_visitors() {
    let response = app()
        .await
        .oneshot(Request::get("/orders").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()[header::LOCATION], "/signin");
}
