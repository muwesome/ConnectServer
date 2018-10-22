use super::ConnectServiceConfig;
use crate::state::{ClientPool, RealmBrowser};
use crate::{util::CloseSignal, Result};
use failure::{Context, Fail, ResultExt};
use futures::{Future, Stream};
use std::sync::Arc;
use tokio::{self, net::TcpListener};

mod session;

/// Starts listening for incoming connections.
pub fn serve(
  config: impl ConnectServiceConfig,
  realms: RealmBrowser,
  clients: ClientPool,
  close_rx: CloseSignal,
) -> Result<()> {
  let config = Arc::new(config);
  let close_signal =
    close_rx.map_err(|_| Context::new("Controller channel closed prematurely").into());

  // Listen on the supplied TCP socket
  let listener =
    TcpListener::bind(&config.socket().into()).context("Failed to bind connect service socket")?;
  let local_addr = listener
    .local_addr()
    .context("Failed to determine connect service socket")?;

  let server = listener
    // Wait for incoming connections
    .incoming()
    // Apply context for any errors
    .map_err(|error| error.context("Connect service stream error").into())
    // Process each new client connection
    .for_each(move |stream| session::process(&config, &realms, &clients, stream))
    // Listen for any cancellation events from the controller
    .select(close_signal);

  println!("Client listening on {}", local_addr);
  tokio::run(
    server
      .map(|(item, _)| item)
      .map_err(|(error, _)| println!("Connect Service: {}", error)),
  );
  Ok(())
}
