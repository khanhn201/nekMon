use async_ssh2_tokio::client::{Client, AuthMethod, ServerCheckMethod};

use crate::model::Server;


pub struct SSHClient {
    client: Client,
}

impl SSHClient {
    // pub async fn new(server: &Server) -> Result<Self, Box<dyn std::error::Error>> {
    //     let client = Client::connect(
    //         (server.address.clone(), 22),
    //         "khanhn2",
    //         AuthMethod::with_key_file("/home/nekoconn/.ssh/id_ed25519", Option::None),
    //         ServerCheckMethod::NoCheck,
    //     )
    //     .await?;
    //
    //     let result = client.execute("tail /srv/scratch/khanhn2/orr-sommerfeld/fsos/logfile").await?;
    //     print!("{0}", result.stdout);
    //
    //     Ok(Self { client })
    // }
    pub async fn new() -> Result<Self, async_ssh2_tokio::error::Error> {
        let client = Client::connect(
            ("enzo.cs.illinois.edu", 22),
            "khanhn2",
            AuthMethod::with_key_file("/home/nekoconn/.ssh/id_ed25519", Option::None),
            ServerCheckMethod::NoCheck,
        )
        .await?;

        let result = client.execute("tail /srv/scratch/khanhn2/orr-sommerfeld/fsos/logfile").await?;
        print!("{0}", result.stdout);

        Ok(Self { client })
    }

    pub async fn execute(&self, cmd: &str) -> Result<String, async_ssh2_tokio::error::Error> {
        let result = self.client.execute(cmd).await?;
        Ok(result.stdout)
    }

    pub async fn download_file(&self, file: &str) -> Result<(), async_ssh2_tokio::error::Error> {
        self.client.download_file(file, file).await?;
        Ok(())
    }
}
