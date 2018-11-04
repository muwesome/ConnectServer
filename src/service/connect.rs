pub use self::config::ConnectServiceConfig;
pub use self::error::ConnectServiceError;
use crate::util::{CloseSignal, ThreadController};
use crate::{state::RealmBrowser, Result};
use std::sync::Arc;

mod config;
mod error;
mod net;
mod plugin;

/// A connect service instance.
pub struct ConnectService(ThreadController);

impl ConnectService {
  /// Spawns a new Connect Service instance.
  pub fn spawn(config: Arc<impl ConnectServiceConfig>, realms: RealmBrowser) -> Self {
    let ctl = ThreadController::spawn(move |rx| Self::serve(&*config, realms, rx));
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
    let mut client_handler = net::ClientStreamHandler::new(responder, codec_provider);
    client_handler.set_max_idle_time(config.max_idle_time());
    client_handler.set_max_requests(config.max_requests());
    client_handler.set_max_unresponsive_time(config.max_unresponsive_time());
    client_handler.register_plugin(plugin::ClientEventLogger);
    client_handler.register_plugin(plugin::CheckMaximumClients::new(config.max_connections()));
    client_handler.register_plugin(plugin::CheckMaximumClientsPerIp::new(
      config.max_connections_per_ip(),
    ));

    // Listen for incoming client connections
    let listener = net::ClientListener::new(config.socket(), close_rx);
    listener.register_plugin(plugin::ListenerEventLogger);
    listener.listen(client_handler).map_err(From::from)
  }
}
