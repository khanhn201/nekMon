use leptos::server_fn::ServerFnError;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

pub async fn open_db() -> Result<SqlitePool, ServerFnError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:nekMon.db?mode=rwc") // TODO: configurable
        .await?;
    
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    Ok(pool)
}
