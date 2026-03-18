use serde::{Deserialize, Serialize};

/// ------------------------------
/// Struct: one-to-one with SQL table
/// ------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Server {
    #[serde(default)]
    pub id: i64,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub username: String,
    pub key_file_path: String,
    //  !TODO a run script, can be different for Nek5000 and NekRS?
    pub remote_directory: String, // Default prefix for each new run
                                  // pub run_script: String // TODO
}



/// ------------------------------
/// Server functions
/// ------------------------------
use leptos::prelude::*;

#[server]
pub async fn get_servers() -> Result<Vec<Server>, ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    let servers: Vec<Server> = sqlx::query_as("SELECT * FROM server")
        .fetch_all(pool)
        .await?;
    Ok(servers)
}

#[server]
pub async fn get_alive_status(server: Server) -> Result<bool, ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    Ok(app_state.get_ssh_client(&server).await.is_some())
}

#[server]
pub async fn create_server(server: Server) -> Result<(), ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    let _server = sqlx::query_as::<_, Server>(
        r#"INSERT INTO server (name, address, username, remote_directory, key_file_path, port)
         VALUES (?, ?, ?, ?, ?, ?)
         RETURNING *"#,
    )
    .bind(server.name)
    .bind(server.address)
    .bind(server.username)
    .bind(server.remote_directory)
    .bind(server.key_file_path)
    .bind(server.port)
    .fetch_one(pool)
    .await?;
    Ok(())
}

#[server]
pub async fn update_server(server: Server) -> Result<(), ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    let _server = sqlx::query_as::<_, Server>(
        r#"
        UPDATE server
        SET
            name = COALESCE(?, name),
            address = COALESCE(?, address),
            username = COALESCE(?, username),
            remote_directory = COALESCE(?, remote_directory),
            key_file_path = COALESCE(?, key_file_path),
            port = COALESCE(?, port)
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(server.name)
    .bind(server.address)
    .bind(server.username)
    .bind(server.remote_directory)
    .bind(server.key_file_path)
    .bind(server.port)
    .bind(server.id)
    .fetch_one(pool)
    .await?;
    Ok(())
}

#[server]
pub async fn delete_server(server_id: i64) -> Result<(), ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    sqlx::query("DELETE FROM server WHERE id = ?")
        .bind(server_id)
        .execute(pool)
        .await?;
    Ok(())
}


