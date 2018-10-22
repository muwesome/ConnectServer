use crate::observer::EventObserver;
use crate::service::{ClientService, RpcService};
use crate::state::{ClientPool, RealmBrowser};
use parking_lot::Mutex;
use std::sync::Arc;

#[cfg(feature = "build-binary")]
use structopt::StructOpt;

#[macro_use]
mod util;
mod observer;
mod service;
mod state;

// TODO: Fix local packet dependencies
// TODO: Quit if the RPC service fails
// TODO: Check error with invalid RPC host
// TODO: Use structured logging
// TODO: Config separated or not?
// TODO: Configurations
// - max_connections_per_ip
// - max_packet_size

/// Default result type used.
type Result<T> = ::std::result::Result<T, failure::Error>;

#[cfg_attr(feature = "build-binary", derive(StructOpt))]
#[cfg_attr(
  feature = "build-binary",
  structopt(about = "Mu Online Connect Server")
)]
pub struct ConnectConfig {
  #[cfg_attr(feature = "build-binary", structopt(flatten))]
  pub client: service::ClientServiceConfig,
  #[cfg_attr(feature = "build-binary", structopt(flatten))]
  pub rpc: service::RpcServiceConfig,
}

/// The server object.
pub struct ConnectServer {
  #[allow(dead_code)]
  observer: Arc<Mutex<EventObserver>>,
  client_service: ClientService,
  rpc_service: RpcService,
}

impl ConnectServer {
  /// Spawns a new Connect Server using defaults.
  pub fn spawn(config: ConnectConfig) -> Result<Self> {
    // TODO: Exposing awareness of 'max_connections' here?
    let clients = ClientPool::new(config.client.max_connections);
    let realms = RealmBrowser::new();

    let observer = Arc::new(Mutex::new(EventObserver));
    realms.add_listener(&observer);
    clients.add_listener(&observer);

    let client_service = ClientService::spawn(config.client, realms.clone(), clients);
    let rpc_service = RpcService::spawn(config.rpc, realms);

    Ok(ConnectServer {
      observer,
      rpc_service,
      client_service,
    })
  }

  /// Returns whether the server is still active or not.
  pub fn is_active(&self) -> bool {
    self.client_service.is_active()
  }

  /// Stops the server.
  pub fn stop(self) -> Result<()> {
    let result = self.client_service.stop();
    self.rpc_service.stop()?;
    result
  }

  /// Will block, waiting for the server to finish.
  pub fn wait(self) -> Result<()> {
    let result = self.client_service.wait();
    self.rpc_service.stop()?;
    result
  }
}
