use super::StreamHandler;
use crate::service::connect::error::{ConnectServiceError, Result, ServerError};
use crate::service::connect::plugin::ListenerEventPlugin;
use crate::util::{CloseSignal, EventHandler};
use futures::{Future, Stream};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{self, net::TcpListener};

pub struct ClientListener {
  on_startup: EventHandler<SocketAddr>,
  on_error: EventHandler<ConnectServiceError>,
  socket: SocketAddr,
  close_signal: CloseSignal,
}

impl ClientListener {
  pub fn new(socket: SocketAddr, close_signal: CloseSignal) -> Self {
    ClientListener {
      on_startup: EventHandler::new(),
      on_error: EventHandler::new(),
      socket,
      close_signal,
    }
  }

  pub fn register_plugin(&self, plugin: impl ListenerEventPlugin) {
    let plugin = Arc::new(plugin);
    self
      .on_startup
      .subscribe_fn(closet!([plugin] move |event| plugin.on_startup(event)));
    self
      .on_error
      .subscribe_fn(closet!([plugin] move |event| plugin.on_error(event)));
  }

  /// Starts listening for incoming connections.
  pub fn listen(self, stream_handler: impl StreamHandler) -> Result<()> {
    let ClientListener {
      on_startup,
      on_error,
      socket,
      close_signal,
    } = self;

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
        tokio::spawn(stream_handler.handle(stream).map_err(|_| ()));
        Ok(())
      })
      // Listen for any cancellation events from the controller
      .select(close_signal);

    if !on_startup.dispatch(local_addr) {
      Err(ServerError::Abort)?;
    }

    tokio::run(server.then(move |result| {
      if let Err((error, _)) = result {
        on_error.dispatch(error);
      }
      Ok(())
    }));
    Ok(())
  }
}
