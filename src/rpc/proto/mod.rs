use failure::{Error, ResultExt};
use try_from::TryFrom;
use {realm, Result};

pub use self::connectservice::*;
pub use self::connectservice_grpc::*;

mod connectservice;
mod connectservice_grpc;

impl TryFrom<RealmDefinition> for realm::RealmServer {
  type Err = Error;

  fn try_from(definition: RealmDefinition) -> Result<Self> {
    let status = definition.get_status();

    Ok(realm::RealmServer {
      id: realm::RealmServerId::try_from(definition.get_id()).context("Invalid id specified")?,
      host: definition.get_host().into(),
      port: u16::try_from(definition.get_port()).context("Invalid port specified")?,
      clients: status.get_clients() as usize,
      capacity: status.get_capacity() as usize,
    })
  }
}
