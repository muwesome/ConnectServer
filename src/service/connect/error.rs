use crate::state::{ClientPoolError, RealmBrowserError};
use failure::Fail;
use muonline_protocol::connect::Version;
use std::io;
use tokio::timer::timeout;

#[derive(Fail, Debug)]
pub enum ClientError {
  #[fail(display = "Failed to resolve address")]
  CannotResolveAddress(#[fail(cause)] io::Error),

  #[fail(display = "Connection stream failed")]
  Connection(#[fail(cause)] io::Error),

  #[fail(display = "Invalid packet received")]
  InvalidPacket(#[fail(cause)] io::Error),

  #[fail(display = "Invalid IP version")]
  InvalidIpVersion,

  #[fail(display = "Maximum packet count exceeded")]
  MaxPacketsExceeded,

  #[fail(display = "Session timed out")]
  TimedOut,

  #[fail(display = "Unknown packet received; {:?}", header)]
  UnknownPacket { header: Vec<u8> },

  #[fail(
    display = "Version mismatch; was {:?}, expected {:?}",
    has,
    expected
  )]
  VersionMismatch { has: Version, expected: Version },
}

impl ClientError {
  pub fn connection_reset_by_peer(&self) -> bool {
    matches!(self, ClientError::Connection(error) if error.kind() == io::ErrorKind::ConnectionReset)
  }
}

#[derive(Fail, Debug)]
pub enum ServerError {
  #[fail(display = "Failed to resolve address")]
  CannotResolveAddress(#[fail(cause)] io::Error),

  #[fail(display = "Client state error")]
  ClientState(#[fail(cause)] ClientPoolError),

  #[fail(display = "Close signal aborted")]
  CloseSignalAborted,

  #[fail(display = "Connection stream failed")]
  Connection(#[fail(cause)] io::Error),

  #[fail(display = "Failed to bind to port")]
  Bind(#[fail(cause)] io::Error),

  #[fail(display = "Invalid packet constructed")]
  InvalidPacket(#[fail(cause)] io::Error),

  #[fail(display = "Realm state error")]
  RealmState(#[fail(cause)] RealmBrowserError),
}

#[derive(Fail, Debug)]
pub enum ConnectServiceError {
  #[fail(display = "Server error")]
  Server(ServerError),

  #[fail(display = "Client error")]
  Client(ClientError),
}

impl ConnectServiceError {
  pub fn from_client_timeout(error: timeout::Error<std::io::Error>) -> Self {
    error
      .into_inner()
      .map(ClientError::Connection)
      .unwrap_or_else(|| ClientError::TimedOut)
      .into()
  }
}

impl From<ServerError> for ConnectServiceError {
  fn from(error: ServerError) -> Self {
    ConnectServiceError::Server(error)
  }
}

impl From<ClientError> for ConnectServiceError {
  fn from(error: ClientError) -> Self {
    ConnectServiceError::Client(error)
  }
}

pub type Result<T> = std::result::Result<T, ConnectServiceError>;
