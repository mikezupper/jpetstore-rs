mod db;
mod web;

// Startup errors and request errors get different treatment on purpose:
// if the server can't bind its port there is nothing sensible to do but
// say so and exit, so `expect` is the honest tool here. Errors that happen
// while serving a request never crash the process — they become values
// that render as error pages (see web/error.rs).
#[tokio::main]
async fn main() {
    let pool = db::pool("sqlite:jpetstore.db")
        .await
        .expect("database init failed");
    let app = web::router(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8081")
        .await
        .expect("could not bind 127.0.0.1:8081 — is another jpetstore-rs running?");
    println!("jpetstore-rs listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.expect("server error");
}
