use super::{StreamHandler, ConnectServiceFuture};
use crate::service::connect::error::{ClientError, Result, ServerError};
use crate::state::ClientPool;
use futures::{future, Future};
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio::net::TcpStream;

pub struct ClientSessionDecorator<T: StreamHandler> {
  clients: ClientPool,
  handler: Arc<T>,
}

impl<T: StreamHandler> ClientSessionDecorator<T> {
  pub fn new(clients: ClientPool, handler: T) -> Self {
    ClientSessionDecorator {
      handler: Arc::new(handler),
      clients,
    }
  }
}

impl<T: StreamHandler> StreamHandler for ClientSessionDecorator<T> {
  /// Bootstraps the connection stream.
  fn handle(&self, stream: TcpStream) -> ConnectServiceFuture<()> {
    let clients = self.clients.clone();
    let handler = self.handler.clone();

    let session = future::result(peer_addr_v4(&stream))
      // Add the client to the pool
      .and_then(closet!([clients] move |socket| {
        clients.add(socket).map_err(ServerError::ClientState).map_err(From::from)
      }))
      // Bootstrap the connection
      .and_then(closet!([handler] move |client| handler.handle(stream).map(|_| client)))
      .map(|_| ());
    Box::new(session)
  }
}

fn peer_addr_v4(stream: &TcpStream) -> Result<SocketAddrV4> {
  let socket = stream
    .peer_addr()
    .map_err(ClientError::CannotResolveAddress)?;
  matches_opt!(socket, SocketAddr::V4(addr) => addr)
    .ok_or(ClientError::InvalidIpVersion)
    .map_err(From::from)
}
