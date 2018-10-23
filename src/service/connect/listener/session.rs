use self::error::ClientSessionError;
use self::util::{packet_limiter, TcpStreamSocket};
use super::ConnectServiceConfig;
use crate::state::{ClientPool, RealmBrowser};
use futures::{future, Future, Sink, Stream};
use muonline_packet::{Packet, PacketEncodable, XOR_CIPHER};
use muonline_packet::{PacketCodec, PacketCodecState};
use muonline_protocol::connect::{self, server, Client};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::prelude::{FutureExt, StreamExt};
use tokio::{self, codec::Decoder};

mod error;
mod util;

/// Setups and spawns a new session for a client.
pub fn process(
  config: &Arc<impl ConnectServiceConfig>,
  realms: &RealmBrowser,
  clients: &ClientPool,
  stream: TcpStream,
) -> crate::Result<()> {
  let client = stream
    .peer_addr_v4()
    .and_then(closet!([clients] move |socket| {
    // Add the client to the pool
    clients.add(socket).map_err(ClientSessionError::ClientState)
  }));

  // Bootstraps the connection stream.
  let pipeline = future::lazy(closet!([config, realms] move || {
    let (writer, reader) = codec(config.max_packet_size())
      // Use a non C3/C4 encrypted TCP codec
      .framed(stream)
      // Split the stream value into two separate handles
      .split();

    reader
      // Prevent idle clients from reserving resources
      .timeout(config.max_idle_time())
      // Determine whether it's a timeout or stream error
      .map_err(ClientSessionError::from)
      // Limit the number of client requests allowed
      .and_then(packet_limiter(config.max_requests()))
      // Map each packet to a corresponding response
      .and_then(closet!([config, realms] move |packet| respond(&*config, &realms, &packet)))
      // Ignore any empty responses
      .filter_map(|packet| packet)
      // Forward the packets to the client
      .fold(writer, closet!([config] move |sink, packet| {
        sink
          .send(packet)
          .timeout(config.max_unresponsive_time())
          .map_err(ClientSessionError::from)
      }))
      .map(|_| ())
  }));

  // Wait for the session to finish
  let session = future::result(client).join(pipeline);

  // Spawn each client on an executor
  tokio::spawn(session.then(|result| {
    if let Err(error) = result {
      if !error.connection_reset_by_peer() {
        println!("Client session error: {}", error);
      }
    }
    Ok(())
  }));
  Ok(())
}

/// Constructs a server response for each client packet.
fn respond(
  config: &impl ConnectServiceConfig,
  realms: &RealmBrowser,
  packet: &Packet,
) -> Result<Option<Packet>, ClientSessionError> {
  match Client::from_packet(&packet).map_err(ClientSessionError::InvalidPacket)? {
    Client::ConnectServerRequest(request) => {
      if request.version == connect::VERSION {
        server::ConnectServerResult::success()
          .to_packet()
          .map_err(ClientSessionError::InvalidServerPacket)
          .map(Some)
      } else {
        Err(ClientSessionError::VersionMismatch {
          has: request.version,
          expected: connect::VERSION,
        })
      }
    }
    Client::RealmServerConnectRequest(server) => realms
      .get(server.id, |realm| {
        server::RealmServerConnect::new(realm.host.clone(), realm.port)
      }).map_err(ClientSessionError::RealmState)
      .and_then(|response| {
        response
          .to_packet()
          .map_err(ClientSessionError::InvalidServerPacket)
          .map(Some)
      }),
    Client::RealmServerListRequest => {
      let mut list = Vec::with_capacity(realms.len());
      realms.for_each(|realm| list.push((realm.id, realm.load_factor().into()).into()));
      server::RealmServerList(list)
        .to_packet()
        .map_err(ClientSessionError::InvalidServerPacket)
        .map(Some)
    }
    _ => {
      if config.ignore_unknown_packets() {
        Ok(None)
      } else {
        Err(ClientSessionError::UnknownPacket {
          // Preserve enough bytes to construct a footprint
          header: [packet.kind() as u8, packet.code()]
            .iter()
            .chain(packet.data().iter().take(2))
            .cloned()
            .collect::<Vec<_>>(),
        })
      }
    }
  }
}

/// Returns the codec used for a Connect Server.
fn codec(max_size: usize) -> PacketCodec {
  PacketCodec::new_with_max_size(
    PacketCodecState::new(),
    PacketCodecState::builder().cipher(&XOR_CIPHER).build(),
    max_size,
  )
}
