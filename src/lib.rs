pub use self::builder::ServerBuilder;
use self::service::{ClientService, RpcService};

#[macro_use]
mod util;
mod builder;
mod service;
mod state;

// TODO: Use structured logging
// TODO: Add tons of logging
// TODO: Improve error reporting:
// - Improved messages
// - Customized types
// - RPC status codes
// TODO: Configurations
// - DisconnectOnUnknownPacket
// - Client IP & PORT
// - Client timeout
// - RPC IP & PORT
// - Max packet size
// - Max connections (global)
// - Max connections (per IP)
// - Max server list/ip requests?

/// Default result type used.
type Result<T> = ::std::result::Result<T, failure::Error>;

/// The server object.
pub struct ConnectServer {
  client_service: ClientService,
  rpc_service: RpcService,
}

impl ConnectServer {
  /// Spawns a new Connect Server using defaults.
  pub fn spawn() -> Result<Self> {
    Self::builder().spawn()
  }

  /// Returns a builder for the Connect Server.
  pub fn builder() -> ServerBuilder {
    ServerBuilder::default()
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
