use jpetstore_rs::{db, web};

// Startup errors and request errors get different treatment on purpose:
// if the server can't bind its port there is nothing sensible to do but
// say so and exit, so `expect` is the honest tool here. Errors that happen
// while serving a request never crash the process — they become values
// that render as error pages (see web/error.rs).
// The scratch image contains exactly one executable, so the healthcheck
// probe is that executable in a second role: `jpetstore-rs healthcheck`
// opens a TCP connection to the serving process, asks /healthz, and turns
// the answer into an exit code. Twenty lines of std, zero new dependencies.
fn healthcheck_probe() -> ! {
    use std::io::{Read, Write};
    let addr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8081".to_string())
        .replace("0.0.0.0", "127.0.0.1");
    let healthy = std::net::TcpStream::connect(&addr)
        .ok()
        .and_then(|mut stream| {
            stream
                .write_all(b"GET /healthz HTTP/1.1\r\nHost: healthcheck\r\nConnection: close\r\n\r\n")
                .ok()?;
            let mut response = String::new();
            stream.read_to_string(&mut response).ok()?;
            response.starts_with("HTTP/1.1 200").then_some(())
        })
        .is_some();
    std::process::exit(if healthy { 0 } else { 1 });
}

#[tokio::main]
async fn main() {
    if std::env::args().nth(1).as_deref() == Some("healthcheck") {
        healthcheck_probe();
    }

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
