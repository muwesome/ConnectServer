use failure::ResultExt;
use {ConnectServer, ConnectService, RealmBrowser, Result};

#[derive(Default)]
pub struct ServerBuilder {}

impl ServerBuilder {
  pub fn spawn(self) -> Result<ConnectServer> {
    let realms = RealmBrowser::new();
    Ok(ConnectServer {
      rpc_service: ConnectService::spawn("127.0.0.1", 50051, realms)
        .context("Failed to spawn connect service")?,
    })
  }
}
