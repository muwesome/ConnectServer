pub use self::config::ConnectServiceConfig;
pub use self::error::ConnectServiceError;
use crate::state::{ClientPool, RealmBrowser};
use crate::util::{CloseSignal, ThreadController};
use crate::Result;
use std::sync::Arc;

mod config;
mod error;
mod net;

/// A connect service instance.
pub struct ConnectService(ThreadController);

impl ConnectService {
  /// Spawns a new Connect Service instance.
  pub fn spawn(
    config: Arc<impl ConnectServiceConfig>,
    clients: ClientPool,
    realms: RealmBrowser,
  ) -> Self {
    let ctl = ThreadController::spawn(move |rx| Self::serve(&*config, clients, realms, rx));
    ConnectService(ctl)
  }

  /// Returns whether the service is still active or not.
  pub fn is_active(&self) -> bool {
    self.0.is_alive()
  }

  /// Stops the service.
  pub fn stop(self) -> Result<()> {
    self.0.stop()
  }

  /// Will block, waiting for the service to finish.
  pub fn wait(self) -> Result<()> {
    self.0.wait()
  }

  fn serve(
    config: &impl ConnectServiceConfig,
    clients: ClientPool,
    realms: RealmBrowser,
    close_rx: CloseSignal,
  ) -> Result<()> {
    // Maps incoming packets to server responses
    let mut responder = net::ClientPacketResponder::new(realms);
    responder.set_ignore_unknown_packets(config.ignore_unknown_packets());

    // Factory for the packet codec
    let max_packet_size = config.max_packet_size();
    let codec_provider = move || net::codec(max_packet_size);

    // Manages each client's stream
    let mut handler = net::ClientStreamHandler::new(responder, codec_provider);
    handler.set_max_idle_time(config.max_idle_time());
    handler.set_max_requests(config.max_requests());
    handler.set_max_unresponsive_time(config.max_unresponsive_time());

    // Wraps each client's stream with a session
    let client_handler = net::ClientSessionDecorator::new(clients, handler);

    // Listen for incoming client connections
    net::listen(config.socket(), client_handler, close_rx).map_err(From::from)
  }
}
