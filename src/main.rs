use jpetstore_rs::{db, web};

// Startup errors and request errors get different treatment on purpose:
// if the server can't bind its port there is nothing sensible to do but
// say so and exit, so `expect` is the honest tool here. Errors that happen
// while serving a request never crash the process — they become values
// that render as error pages (see web/error.rs).
#[tokio::main]
async fn main() {
    // The two facts that change between a laptop and a server — where the
    // database lives, and what to bind — cross the boundary as environment
    // variables. The defaults are exactly the course's dev behavior, so
    // `cargo run` works the same as it has since lesson 1.
    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:jpetstore.db".to_string());
    let bind_addr =
        std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8081".to_string());

    let pool = db::pool(&db_url).await.expect("database init failed");
    let app = web::router(pool);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| panic!("could not bind {bind_addr}: {e}"));
    println!("jpetstore-rs listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.expect("server error");
}
