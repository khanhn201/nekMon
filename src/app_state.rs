use std::sync::Arc;

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use crate::ssh::SSHClient;

use dashmap::DashMap;


pub struct AppState {
    pub pool: SqlitePool,
    pub servers: DashMap<i64, Arc<SSHClient>>,
}

pub async fn init_app_state() -> Result<Arc<AppState>, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:nekMon.db?mode=rwc") // TODO: configurable
        .await?;
    
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(Arc::new(AppState {
        pool: pool,
        servers: DashMap::new(),
    }))
}
