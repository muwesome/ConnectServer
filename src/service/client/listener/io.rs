use super::ClientServiceConfig;
use crate::{state::RealmBrowser, Result};
use failure::{Context, ResultExt};
use muonline_packet::{Packet, PacketEncodable};
use muonline_protocol::connect::{self, server, Client};

pub fn process(
  config: &ClientServiceConfig,
  realms: &RealmBrowser,
  packet: &Packet,
) -> Result<Option<Packet>> {
  // TODO: Simplify error handling & conversion
  // TODO: Require 'ConnectServerRequest' before other packets?
  match Client::from_packet(&packet)? {
    Client::ConnectServerRequest(request) => {
      if request.version == connect::VERSION {
        server::ConnectServerResult::success()
          .to_packet()
          .context("Failed to construct server result packet")
          .map_err(From::from)
          .map(Some)
      } else {
        Err(Context::new("Incorrect API version").into())
      }
    }
    Client::RealmServerConnectRequest(server) => realms
      .get(server.id, |realm| {
        server::RealmServerConnect::new(realm.host.clone(), realm.port)
          .to_packet()
          .context("Failed to construct realm connect packet")
          .map_err(From::from)
          .map(Some)
      }).and_then(|result| result),
    Client::RealmServerListRequest => {
      let mut list = Vec::with_capacity(realms.len());
      realms.for_each(|realm| list.push((realm.id, realm.load_factor().into())));
      list
        .into_iter()
        .collect::<server::RealmServerList>()
        .to_packet()
        .context("Failed to construct realm list packet")
        .map_err(From::from)
        .map(Some)
    }
    _ => {
      if config.ignore_unknown_packets {
        Ok(None)
      } else {
        Err(Context::new("Unknown packet type").into())
      }
    }
  }
}