use super::ClientSessionError;
use std::net::{SocketAddr, SocketAddrV4};
use tokio::net::TcpStream;

impl ClientSessionError {
  pub fn connection_reset_by_peer(&self) -> bool {
    matches!(self, ClientSessionError::Connection(error) if error.kind() == io::ErrorKind::ConnectionReset)
  }
}
