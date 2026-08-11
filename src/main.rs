use axum::{routing::get, Router};

// Lesson 1: prove the toolchain works end to end. The unwraps are temporary —
// lesson 2 is entirely about the error strategy that replaces them.
#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(home));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8081").await.unwrap();
    println!("jpetstore-rs listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn home() -> &'static str {
    "jpetstore-rs — the pet store you already know, one binary at a time"
}
