extern crate evmap;
extern crate failure;
extern crate futures;
extern crate grpcio;
extern crate protobuf;
extern crate try_from;

use builder::ServerBuilder;
use realm::RealmBrowser;
use rpc::ConnectService;

#[macro_use]
mod macros;
mod builder;
mod realm;
mod rpc;

/// Default result type used.
type Result<T> = ::std::result::Result<T, failure::Error>;

/// The server object.
pub struct ConnectServer {
  rpc_service: ConnectService,
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
    //let result = self.client_service.stop();
    self.rpc_service.stop()
    //result
  }

  /// Will block, waiting for the server to finish.
  pub fn wait(self) -> Result<()> {
    //let result = self.connect_service.wait();
    // TODO: Should just stop
    self.rpc_service.wait()
    //result
  }
}
