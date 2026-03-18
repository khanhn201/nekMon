use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};
use tokio::time::{timeout, Duration};

use crate::models::server::Server;

#[derive(Clone)]
pub struct SSHClient {
    client: Client,
}

impl SSHClient {
    pub async fn new(server: &Server) -> Result<Self, async_ssh2_tokio::error::Error> {
        let connect_future = Client::connect(
            (server.address.clone(), server.port),
            &server.username,
            AuthMethod::with_key_file(server.key_file_path.clone(), Option::None),
            ServerCheckMethod::NoCheck,
        ); // TODO: connect interative (refer to visit or paraview), use russh and russh-sftp

        let client = timeout(Duration::from_secs(5), connect_future)
            .await
            .map_err(|_| {
                async_ssh2_tokio::error::Error::from(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "SSH connect timed out",
                ))
            })??;

        Ok(Self { client })
    }

    pub async fn ping(&self) -> Result<(), async_ssh2_tokio::error::Error> {
        let execute_future = self.client.execute("echo ping");
        let result = timeout(Duration::from_secs(5), execute_future)
            .await
            .map_err(|_| {
                async_ssh2_tokio::error::Error::from(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "SSH connect timed out",
                ))
            })??;
        assert_eq!(result.exit_status, 0);
        Ok(())
    }

    pub async fn execute(&self, cmd: &str) -> Result<String, async_ssh2_tokio::error::Error> {
        let result = self.client.execute(cmd).await?;
        Ok(result.stdout)
    }

    pub async fn download_file(
        &self,
        local_file: &str,
        remote_file: &str,
    ) -> Result<(), async_ssh2_tokio::error::Error> {
        self.client.download_file(remote_file, local_file).await?;
        Ok(())
    }
    pub async fn upload_file(
        &self,
        local_file: &str,
        remote_file: &str,
    ) -> Result<(), async_ssh2_tokio::error::Error> {
        self.client
            .upload_file(local_file, remote_file, Option::None, Option::None, false)
            .await?;
        Ok(())
    }
}
