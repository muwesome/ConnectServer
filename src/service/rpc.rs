pub use self::config::RpcServiceConfig;
use crate::util::{CloseSignal, ThreadController};
use crate::{state::RealmBrowser, Result};
use failure::Fail;
use futures::Future;
use grpcio::{Environment, ServerBuilder};
use log::info;
use std::sync::Arc;

mod config;
mod listener;
mod proto;

#[derive(Fail, Debug)]
enum RpcServiceError {
  #[fail(display = "Failed to build service: {}", _0)]
  BuildFailure(#[cause] grpcio::Error),

  #[fail(display = "Failed to shutdown service: {}", _0)]
  ShutdownFailure(#[cause] grpcio::Error),

  #[fail(display = "Close signal was aborted prematurely")]
  CloseSignalAborted,
}

/// An RPC service instance.
pub struct RpcService(ThreadController);

impl RpcService {
  /// Spawns a new RPC service instance.
  pub fn spawn(config: Arc<impl RpcServiceConfig>, realms: RealmBrowser) -> Self {
    grpcio::redirect_log();
    let ctl = ThreadController::spawn(move |rx| Self::serve(&config, realms, rx));
    RpcService(ctl)
  }

  /// Returns whether the service is still active or not.
  pub fn is_active(&self) -> bool {
    self.0.is_alive()
  }

  /// Stops the service.
  pub fn stop(self) -> Result<()> {
    self.0.stop()
  }

  fn serve(
    config: &Arc<impl RpcServiceConfig>,
    realms: RealmBrowser,
    close_rx: CloseSignal,
  ) -> Result<()> {
    let service = proto::create_realm_service(listener::RpcListener::new(realms, close_rx.clone()));

    let environment = Arc::new(Environment::new(1));
    let mut server = ServerBuilder::new(environment)
      .register_service(service)
      .bind(config.host(), config.port())
      .build()
      .map_err(RpcServiceError::BuildFailure)?;

    server.start();
    for &(ref host, port) in server.bind_addrs() {
      info!("RPC listening on {}:{}", host, port);
    }

    let close_result = close_rx
      .wait()
      .map_err(|_| RpcServiceError::CloseSignalAborted);
    let shutdown_result = server
      .shutdown()
      .wait()
      .map_err(RpcServiceError::ShutdownFailure);
    shutdown_result.and(close_result).map_err(From::from)
  }
}
