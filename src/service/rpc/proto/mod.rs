use crate::{state, Result};
use failure::{Error, ResultExt};
use try_from::TryFrom;

pub use self::connectserver::*;
pub use self::connectserver_grpc::*;

mod connectserver;
mod connectserver_grpc;

impl TryFrom<RealmParams_RealmDefinition> for state::RealmServer {
  type Err = Error;

  fn try_from(definition: RealmParams_RealmDefinition) -> Result<Self> {
    let status = definition.get_status();

    // TODO: Validate more? (e.g clients <= capacity, host == ip/domain)
    Ok(state::RealmServer {
      id: state::RealmServerId::try_from(definition.get_id()).context("Invalid id specified")?,
      host: definition.get_host().into(),
      port: u16::try_from(definition.get_port()).context("Invalid port specified")?,
      clients: status.get_clients() as usize,
      capacity: status.get_capacity() as usize,
    })
  }
}
