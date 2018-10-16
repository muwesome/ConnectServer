use crate::state::{ClientManager, RealmBrowser};
use crate::{ClientService, ConnectServer, Result, RpcService};
use failure::ResultExt;

#[derive(Default)]
pub struct ServerBuilder {}

impl ServerBuilder {
  pub fn spawn(self) -> Result<ConnectServer> {
    let realms = RealmBrowser::new();
    let clients = ClientManager::new();

    let socket = "0.0.0.0:2004".parse().expect("TODO:");
    let client_service = ClientService::spawn(socket, realms.clone(), clients)
      .context("Failed to spawn client service")?;
    let rpc_service =
      RpcService::spawn("0.0.0.0", 50051, realms).context("Failed to spawn RPC service")?;

    Ok(ConnectServer {
      rpc_service,
      client_service,
    })
  }
}
