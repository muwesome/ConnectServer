use crate::state::{ClientManager, RealmBrowser};
use crate::{ClientService, ConnectServer, EventObserver, Result, RpcService};
use failure::ResultExt;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::{Arc, Mutex};

pub struct ServerBuilder {
  socket: SocketAddrV4,
  rpc_host: String,
  rpc_port: u16,
}

impl ServerBuilder {
  pub fn socket(mut self, socket: SocketAddrV4) -> Self {
    self.socket = socket;
    self
  }

  pub fn rpc<S: Into<String>>(mut self, host: S, port: u16) -> Self {
    self.rpc_host = host.into();
    self.rpc_port = port;
    self
  }

  pub fn spawn(self) -> Result<ConnectServer> {
    let realms = RealmBrowser::new();
    let clients = ClientManager::new();

    let observer = Arc::new(Mutex::new(EventObserver));
    realms.add_listener(&observer)?;
    clients.add_listener(&observer)?;

    let client_service = ClientService::spawn(self.socket, realms.clone(), clients)
      .context("Failed to spawn client service")?;
    let rpc_service = RpcService::spawn(self.rpc_host, self.rpc_port, realms)
      .context("Failed to spawn RPC service")?;

    Ok(ConnectServer {
      observer,
      rpc_service,
      client_service,
    })
  }
}

impl Default for ServerBuilder {
  fn default() -> Self {
    ServerBuilder {
      socket: SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0),
      rpc_host: "0.0.0.0".into(),
      rpc_port: 0,
    }
  }
}
