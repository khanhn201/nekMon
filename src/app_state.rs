use std::sync::Arc;

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use dashmap::DashMap;

use crate::models::server::Server;
use crate::ssh::SSHClient;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    pool: SqlitePool,
    servers: DashMap<i64, SSHClient>,
}

impl AppState {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite:nekMon.db?mode=rwc") // TODO: make configurable
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self {
            inner: Arc::new(AppStateInner {
                pool,
                servers: DashMap::new(),
            }),
        })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.inner.pool
    }
    pub fn servers(&self) -> &DashMap<i64, SSHClient> {
        &self.inner.servers
    }

    pub async fn get_ssh_client(&self, server: &Server) -> Option<SSHClient> {
        let servers = self.servers();

        if let Some(ssh_client_ref) = servers.get(&server.id) {
            if ssh_client_ref.ping().await.is_ok() {
                return Some(ssh_client_ref.clone());
            } else {
                servers.remove(&server.id);
            }
        }

        match SSHClient::new(server).await {
            Ok(client) => {
                // let client = Arc::new(client);
                if client.ping().await.is_ok() {
                    servers.insert(server.id, client.clone());
                    return Some(client);
                } else {
                    return None;
                }
            }
            Err(_) => None,
        }
    }
}
