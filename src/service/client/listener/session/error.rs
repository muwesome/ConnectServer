use crate::state::{ClientError, RealmError};
use failure::Fail;
use muonline_protocol::connect::Version;
use std::io;
use tokio::timer::timeout;

#[derive(Fail, Debug)]
pub enum ClientSessionError {
  #[fail(display = "Failed to resolve client IP")]
  CannotResolveIp(#[fail(cause)] io::Error),

  #[fail(display = "Client state error")]
  ClientState(#[fail(cause)] ClientError),

  #[fail(display = "Client stream failed")]
  Connection(#[fail(cause)] io::Error),

  #[fail(display = "Invalid server packet constructed")]
  InvalidServerPacket(#[fail(cause)] io::Error),

  #[fail(display = "Invalid client packet received")]
  InvalidPacket(#[fail(cause)] io::Error),

  #[fail(display = "Invalid client IP version")]
  InvalidIpVersion,

  #[fail(display = "Maximum packet count exceeded")]
  MaxPacketsExceeded,

  #[fail(display = "Realm state error")]
  RealmState(#[fail(cause)] RealmError),

  #[fail(display = "Client timed out")]
  TimedOut,

  #[fail(display = "Client sent an unknown packet; {:?}", header)]
  UnknownPacket { header: Vec<u8> },

  #[fail(
    display = "Client version mismatch; was {:?}, expected {:?}",
    has,
    expected
  )]
  VersionMismatch { has: Version, expected: Version },
}

impl ClientSessionError {
  pub fn connection_reset_by_peer(&self) -> bool {
    matches!(self, ClientSessionError::Connection(error) if error.kind() == io::ErrorKind::ConnectionReset)
  }
}

impl From<timeout::Error<std::io::Error>> for ClientSessionError {
  fn from(error: timeout::Error<std::io::Error>) -> Self {
    error
      .into_inner()
      .map(ClientSessionError::Connection)
      .unwrap_or_else(|| ClientSessionError::TimedOut)
  }
}
