use super::ClientSessionError;
use std::net::{SocketAddr, SocketAddrV4};
use tokio::net::TcpStream;

pub trait TcpStreamSocket {
  fn peer_addr_v4(&self) -> Result<SocketAddrV4, ClientSessionError>;
}

impl TcpStreamSocket for TcpStream {
  /// Returns the client's IPv4 socket.
  fn peer_addr_v4(&self) -> Result<SocketAddrV4, ClientSessionError> {
    let socket = self
      .peer_addr()
      .map_err(ClientSessionError::CannotResolveIp)?;
    match_opt!(socket, SocketAddr::V4(addr) => addr).ok_or(ClientSessionError::InvalidIpVersion)
  }
}

/// Returns a packet limiter closure.
pub fn packet_limiter<P>(limit: usize) -> impl FnMut(P) -> Result<P, ClientSessionError> {
  let mut counter = 0;
  move |packet| {
    counter += 1;
    if counter > limit {
      Err(ClientSessionError::MaxPacketsExceeded)
    } else {
      Ok(packet)
    }
  }
}
