use super::StreamHandler;
use crate::service::connect::error::{ConnectServiceError, Result, ServerError};
use crate::util::CloseSignal;
use futures::{Future, Stream};
use log::{error, info};
use std::net::SocketAddrV4;
use tokio::{self, net::TcpListener};

/// Starts listening for incoming connections.
pub fn listen(
  socket: SocketAddrV4,
  stream_handler: impl StreamHandler,
  close_signal: CloseSignal,
) -> Result<()> {
  let close_signal =
    close_signal.map_err(|_| ConnectServiceError::from(ServerError::CloseSignalAborted));

  // Listen on the supplied TCP socket
  let listener = TcpListener::bind(&socket.into()).map_err(ServerError::Bind)?;
  let local_addr = listener
    .local_addr()
    .map_err(ServerError::CannotResolveAddress)?;

  let server = listener
    // Wait for incoming connections
    .incoming()
    // Apply context for any errors
    .map_err(|error| ServerError::Connection(error).into())
    // Process each new client connection
    .for_each(move |stream| {
      tokio::spawn(
        stream_handler
          .handle(stream)
          .map_err(|error| error!("Connect service (client): {}", error)));
      Ok(())
    })
    // Listen for any cancellation events from the controller
    .select(close_signal);

  info!("Connect service listening on {}", local_addr);
  tokio::run(
    server
      .map(|_| ())
      .map_err(|(error, _)| error!("Connect service (server): {}", error)),
  );
  Ok(())
}
