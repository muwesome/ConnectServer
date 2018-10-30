use super::{ClientPacketResponder, ClientStreamHandler};
use super::{ConnectServiceFuture, PacketCodecProvider};
use crate::service::connect::error::{ClientError, ConnectServiceError, Result};
use futures::{Future, Sink, Stream};
use std::sync::Arc;
use std::time::Duration;
use tokio::codec::Decoder;
use tokio::net::TcpStream;
use tokio::prelude::{FutureExt, StreamExt};

pub struct StreamHandler<R: ClientPacketResponder, P: PacketCodecProvider> {
  codec_provider: P,
  max_idle_time: Duration,
  max_requests: usize,
  max_unresponsive_time: Duration,
  responder: Arc<R>,
}

impl<R, P> StreamHandler<R, P>
where
  R: ClientPacketResponder,
  P: PacketCodecProvider,
{
  pub fn new(responder: R, codec_provider: P) -> Self {
    StreamHandler {
      codec_provider,
      max_idle_time: Duration::from_secs(100),
      max_requests: 20,
      max_unresponsive_time: Duration::from_secs(60),
      responder: Arc::new(responder),
    }
  }

  pub fn set_max_idle_time(&mut self, value: Duration) {
    self.max_idle_time = value;
  }

  pub fn set_max_requests(&mut self, value: usize) {
    self.max_requests = value;
  }

  pub fn set_max_unresponsive_time(&mut self, value: Duration) {
    self.max_unresponsive_time = value;
  }
}

impl<R, P> ClientStreamHandler for StreamHandler<R, P>
where
  R: ClientPacketResponder,
  P: PacketCodecProvider,
{
  /// Bootstraps the connection stream.
  fn handle(&self, stream: TcpStream) -> ConnectServiceFuture<()> {
    let (writer, reader) = self.codec_provider.create()
      // Use a non C3/C4 encrypted TCP codec
      .framed(stream)
      // Split the stream value into two separate handles
      .split();

    let responder = self.responder.clone();
    let max_unresponsive_time = self.max_unresponsive_time;

    let session = reader
      // Prevent idle clients from reserving resources
      .timeout(self.max_idle_time)
      // Determine whether it's a timeout or stream error
      .map_err(ConnectServiceError::from_client_timeout)
      // Limit the number of client requests allowed
      .and_then(packet_limiter(self.max_requests))
      // Map each packet to a corresponding response
      .and_then(move |packet| responder.respond(&packet))
      // Ignore any empty responses
      .filter_map(|packet| packet)
      // Forward the packets to the client
      .fold(writer, move |sink, packet| {
        sink
          .send(packet)
          .timeout(max_unresponsive_time)
          .map_err(ConnectServiceError::from_client_timeout)
      })
      .map(|_| ());
    Box::new(session)
  }
}

/// Returns a packet limiter closure.
fn packet_limiter<P>(limit: usize) -> impl FnMut(P) -> Result<P> {
  let mut counter = 0;
  move |packet| {
    counter += 1;
    if counter > limit {
      Err(ClientError::MaxPacketsExceeded.into())
    } else {
      Ok(packet)
    }
  }
}
