pub use self::config::ConnectServiceConfig;
use crate::state::{ClientPool, RealmBrowser};
use crate::{util::ThreadController, Result};

mod config;
mod listener;

/// Wraps the underlying connect server thread.
pub struct ConnectService(ThreadController);

impl ConnectService {
  /// Spawns a new Connect Service instance.
  pub fn spawn(config: ConnectServiceConfig, realms: RealmBrowser, clients: ClientPool) -> Self {
    let ctl = ThreadController::spawn(move |rx| listener::serve(config, realms, clients, rx));
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
