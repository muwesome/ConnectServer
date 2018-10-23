pub use self::config::ConnectServiceConfig;
use crate::state::{ClientPool, RealmBrowser};
use crate::{util::ThreadController, Result};
use std::sync::Arc;

mod client;
mod config;

/// A connect service instance.
pub struct ConnectService(ThreadController);

impl ConnectService {
  /// Spawns a new Connect Service instance.
  pub fn spawn(
    config: Arc<impl ConnectServiceConfig>,
    realms: RealmBrowser,
    clients: ClientPool,
  ) -> Self {
    let ctl = ThreadController::spawn(move |close_signal| {
      client::listen(
        &config,
        close_signal,
        client::handler(&config, &clients, &realms),
      )
    });

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
}
