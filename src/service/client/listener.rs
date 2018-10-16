use crate::state::{ClientManager, RealmBrowser};
use crate::Result;
use failure::{Context, Fail, ResultExt};
use futures::{sync::oneshot, Future, Stream};
use muonline_packet::XOR_CIPHER;
use muonline_packet_codec::{self, PacketCodec};
use std::net::{Shutdown, SocketAddr, SocketAddrV4};
use tokio::net::{TcpListener, TcpStream};
use tokio::{self, codec::Decoder};

/// Starts serving the Connect Server
pub fn serve(
  socket: SocketAddrV4,
  realms: RealmBrowser,
  clients: ClientManager,
  close_rx: oneshot::Receiver<()>,
) -> Result<()> {
  let close_signal = close_rx.map_err(|error| {
    error
      .context("Controller channel closed prematurely")
      .into()
  });

  // Listen on the supplied TCP socket
  let server = TcpListener::bind(&socket.into())?
    // Wait for incoming connections
    .incoming()
    // Apply context for any errors
    .map_err(|error| error.context("Client stream error").into())
    // Process each new client connection
    .for_each(closet!([realms, clients] move |stream| {
      process_client(&realms, &clients, stream)
    }))
    // Listen for any cancellation events from the controller
    .select(close_signal);

  println!("Client listening on {}", socket);
  tokio::run(
    server
      .map(|(item, _)| item)
      .map_err(|(error, _)| println!("Connect Service: {}", error)),
  );
  Ok(())
}

/// Setups and spawns a new task for a client.
fn process_client(realms: &RealmBrowser, clients: &ClientManager, stream: TcpStream) -> Result<()> {
  // Try to add the client to the manager
  let id = match clients.add(ipv4socket(&stream)?) {
    Ok(id) => id,
    Err(error) => {
      let _ = stream.shutdown(Shutdown::Both);
      return Err(error);
    }
  };

  let (writer, reader) = codec()
    // Use a non C3/C4 encrypted TCP codec
    .framed(stream)
    // Split the stream value into two separate handles
    .split();

  // TODO: Connection reset by peer is expected
  let client = reader
    // Apply context for any errors
    .map_err(|error| error.context("Client sink error").into())
    // Each packet received maps to a response packet
    .and_then(closet!([realms] move |packet| super::io::process(&realms, packet)))
    // Send each response packet to the client
    .forward(writer)
    // Remove the client from the service state
    .then(closet!([clients] move |future| {
      // TODO: Use 'inspect/tap' here
      clients.remove(id)?;
      future
    }));

  // Spawn each client on an executor
  tokio::spawn(
    client
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
