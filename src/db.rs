use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

// Startup-only failures get their own small enum, same pattern as AppError:
// an honest list of what can go wrong before the first request is served.
// main() expects on it — a server without a working database has nothing
// better to do than say why and exit.
#[derive(Debug, thiserror::Error)]
pub enum DbInitError {
    #[error("could not open the database")]
    Connect(#[from] sqlx::Error),
    #[error("migrations failed")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

// Opens (creating on first run) the SQLite file and brings its schema up to
// date. The `migrate!` macro embeds every file under migrations/ into the
// binary at compile time — the deployment artifact stays one file.
pub async fn pool(url: &str) -> Result<SqlitePool, DbInitError> {
    let options: SqliteConnectOptions = url.parse::<SqliteConnectOptions>()?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}
