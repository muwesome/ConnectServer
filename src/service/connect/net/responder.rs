use super::ClientPacketResponder;
use crate::service::connect::error::{ClientError, Result, ServerError};
use crate::state::RealmBrowser;
use muonline_packet::{Packet, PacketEncodable};
use muonline_protocol::connect::{self, server, Client};

pub struct PacketResponder {
  ignore_unknown_packets: bool,
  realms: RealmBrowser,
}

impl PacketResponder {
  pub fn new(realms: RealmBrowser) -> Self {
    PacketResponder {
      realms,
      ignore_unknown_packets: false,
    }
  }

  pub fn set_ignore_unknown_packets(&mut self, value: bool) {
    self.ignore_unknown_packets = value;
  }
}

impl ClientPacketResponder for PacketResponder {
  /// Constructs a response for a client packet.
  fn respond(&self, packet: &Packet) -> Result<Option<Packet>> {
    match Client::from_packet(&packet).map_err(ClientError::InvalidPacket)? {
      Client::ConnectServerRequest(request) => {
        if request.version == connect::VERSION {
          server::ConnectServerResult::success()
            .to_packet()
            .map_err(ServerError::InvalidPacket)
            .map(Some)
            .map_err(From::from)
        } else {
          Err(
            ClientError::VersionMismatch {
              has: request.version,
              expected: connect::VERSION,
            }.into(),
          )
        }
      }
      Client::RealmServerConnectRequest(server) => self
        .realms
        .get(server.id)
        .map(|realm| server::RealmServerConnect::new(realm.host.clone(), realm.port))
        .map_err(ServerError::RealmState)
        .and_then(|response| {
          response
            .to_packet()
            .map_err(ServerError::InvalidPacket)
            .map(Some)
        }).map_err(From::from),
      Client::RealmServerListRequest => {
        let mut list = Vec::with_capacity(self.realms.len());
        self
          .realms
          .for_each(|realm| list.push((realm.id, realm.load_factor().into()).into()));
        server::RealmServerList(list)
          .to_packet()
          .map_err(ServerError::InvalidPacket)
          .map(Some)
          .map_err(From::from)
      }
      _ => {
        if self.ignore_unknown_packets {
          Ok(None)
        } else {
          Err(
            ClientError::UnknownPacket {
              // Preserve enough bytes to construct a footprint
              header: [packet.kind() as u8, packet.code()]
                .iter()
                .chain(packet.data().iter().take(2))
                .cloned()
                .collect::<Vec<_>>(),
            }.into(),
          )
        }
      }
    }
  }
}
