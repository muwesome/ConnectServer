use crate::state::{ClientManager, RealmBrowser};
use crate::{util::ThreadController, Result};
use futures::sync::oneshot;
use std::{net::SocketAddrV4, thread};

mod io;
mod listener;

/// Wraps the underlying connect server thread.
pub struct ClientService(ThreadController);

impl ClientService {
  /// Spawns a new Connect Service instance.
  pub fn spawn(socket: SocketAddrV4, realms: RealmBrowser, clients: ClientManager) -> Result<Self> {
    let (tx, rx) = oneshot::channel();
    let handle = thread::spawn(move || listener::serve(socket, realms, clients, rx));
    Ok(ClientService(ThreadController::new(tx, handle)))
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
