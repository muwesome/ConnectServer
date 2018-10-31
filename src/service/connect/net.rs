use auto_impl::auto_impl;
use crate::service::connect::error::{ConnectServiceError, Result};
use futures::Future;
use muonline_packet::{Packet, PacketCodec, PacketCodecState, XOR_CIPHER};
use tokio::net::TcpStream;

pub use self::handler::ClientStreamHandler;
pub use self::listener::listen;
pub use self::responder::ClientPacketResponder;
pub use self::session::ClientSessionDecorator;

mod handler;
mod listener;
mod responder;
mod session;

/// Represents a boxed connect service future.
type ConnectServiceFuture<T> = Box<Future<Item = T, Error = ConnectServiceError> + Send + 'static>;

#[auto_impl(Fn)]
pub trait StreamHandler: Send + Sync + 'static {
  /// Handles a new client stream, returning the session as a future.
  fn handle(&self, stream: TcpStream) -> ConnectServiceFuture<()>;
}

#[auto_impl(Fn)]
pub trait PacketResponder: Send + Sync + 'static {
  /// Constructs a response for a client packet.
  fn respond(&self, packet: &Packet) -> Result<Option<Packet>>;
}

#[auto_impl(Fn)]
pub trait PacketCodecProvider: Send + Sync + 'static {
  /// Constructs a codec for a stream.
  fn create(&self) -> PacketCodec;
}

/// Returns the codec used for a Connect Server.
pub fn codec(max_size: usize) -> PacketCodec {
  PacketCodec::with_max_size(
    PacketCodecState::new(),
    PacketCodecState::builder().cipher(&XOR_CIPHER).build(),
    max_size,
  )
}
