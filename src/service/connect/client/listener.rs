use crate::service::ConnectServiceConfig;
use crate::{util::CloseSignal, Result};
use failure::{Context, Fail, ResultExt};
use futures::{Future, Stream};
use std::sync::Arc;
use tokio;
use tokio::net::{TcpListener, TcpStream};

/// Starts listening for incoming connections.
pub fn listen(
  config: &Arc<impl ConnectServiceConfig>,
  close_signal: CloseSignal,
  client_handler: impl FnMut(TcpStream) -> Result<()> + Send + 'static,
) -> Result<()> {
  let config = Arc::new(config);
  let close_signal =
    close_signal.map_err(|_| Context::new("Controller channel closed prematurely").into());

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
    .for_each(client_handler)
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
