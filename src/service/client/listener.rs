use super::ClientServiceConfig;
use crate::state::{ClientPool, RealmBrowser};
use crate::{util::CloseSignal, Result};
use failure::{Context, Fail, ResultExt};
use futures::{Future, Sink, Stream};
use muonline_packet::XOR_CIPHER;
use muonline_packet_codec::{self, PacketCodec};
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::{FutureExt, StreamExt};
use tokio::{self, codec::Decoder};

mod io;

/// Starts serving the Connect Server
pub fn serve(
  config: ClientServiceConfig,
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
    .for_each(move |stream| serve_client(config.clone(), &realms, &clients, stream))
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

/// Setups and spawns a new task for a client.
fn serve_client(
  config: Arc<ClientServiceConfig>,
  realms: &RealmBrowser,
  clients: &ClientPool,
  stream: TcpStream,
) -> Result<()> {
  // Try to add the client to the manager
  let id = clients.add(ipv4socket(&stream)?)?;
  let mut requests = 0;

  let (writer, reader) = codec()
    // Use a non C3/C4 encrypted TCP codec
    .framed(stream)
    // Split the stream value into two separate handles
    .split();

  // TODO: Connection reset by peer is expected
  let session = reader
    // Prevent idle clients from reserving resources
    .timeout(config.max_idle_time)
    // Determine whether it's a timeout or stream error
    .map_err(|timeout|
      timeout
        .into_inner()
        .map(|error| error.context("Client stream error").into())
        .unwrap_or_else(|| Context::new("Client timed out").into()))
    // Map each packet to a corresponding response
    .and_then(closet!([config, realms] move |packet| {
      requests += 1;
      if requests >= config.max_requests {
        return Err(Context::new("Client exceeded maximum request count"))?;
      }

      io::process(&config, &realms, &packet)
    }))
    // Ignore any empty responses
    .filter_map(|packet| packet)
    // Forward the packets to the client
    .fold(writer, closet!([config] move |sink, packet| {
      sink.send(packet).timeout(config.max_unresponsive_time)
    }))
    // Remove the client from the service state
    .then(closet!([clients] move |future| future.and(clients.remove(id))));

  // Spawn each client on an executor
  tokio::spawn(
    session
      .map(|_| ())
      .map_err(|error| println!("Connect Client: {:?}", error)),
  );
  Ok(())
}

/// Returns the codec used for a Connect Server.
fn codec() -> PacketCodec {
  PacketCodec::new(
    muonline_packet_codec::State::new(None, None),
    muonline_packet_codec::State::new(Some(&XOR_CIPHER), None),
  )
}

/// Returns the client's IPv4 socket.
fn ipv4socket(stream: &TcpStream) -> Result<SocketAddrV4> {
  match stream
    .peer_addr()
    .context("Failed to determine client address")?
  {
    SocketAddr::V4(socket) => Ok(socket),
    _ => Err(Context::new("Invalid client IP version").into()),
  }
}
