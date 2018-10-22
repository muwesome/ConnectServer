use crate::observer::EventObserver;
use crate::service::{ConnectService, ConnectServiceConfig, RpcService};
use crate::state::{ClientPool, RealmBrowser};
use parking_lot::Mutex;
use std::sync::Arc;

pub use crate::config::ConnectConfig;

#[macro_use]
mod util;
mod config;
mod observer;
mod service;
mod state;

// TODO: Fix local packet dependencies
// TODO: Quit if the RPC service fails
// TODO: Check error with invalid RPC host
// TODO: Parse arguments from TOML as well
// TODO: Use structured logging

/// Default result type used.
type Result<T> = ::std::result::Result<T, failure::Error>;

/// The server object.
pub struct ConnectServer {
  #[allow(dead_code)]
  observer: Arc<Mutex<EventObserver>>,
  connect_service: ConnectService,
  rpc_service: RpcService,
}

impl ConnectServer {
  /// Spawns a new Connect Server using defaults.
  pub fn spawn(config: ConnectConfig) -> Result<Self> {
    let realms = RealmBrowser::new();
    let clients = ClientPool::new(config.max_connections(), config.max_connections_per_ip());

    let observer = Arc::new(Mutex::new(EventObserver));
    realms.add_listener(&observer);
    clients.add_listener(&observer);

    let connect_service = ConnectService::spawn(config.clone(), realms.clone(), clients);
    let rpc_service = RpcService::spawn(config, realms);

    Ok(ConnectServer {
      observer,
      rpc_service,
      connect_service,
    })
  }

  /// Returns whether the server is still active or not.
  pub fn is_active(&self) -> bool {
    self.connect_service.is_active()
  }

  /// Stops the server.
  pub fn stop(self) -> Result<()> {
    let result = self.connect_service.stop();
    self.rpc_service.stop()?;
    result
  }

  /// Will block, waiting for the server to finish.
  pub fn wait(self) -> Result<()> {
    let result = self.connect_service.wait();
    self.rpc_service.stop()?;
    result
  }
}
