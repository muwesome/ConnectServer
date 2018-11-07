use super::{ConnectServiceFuture, PacketCodecProvider};
use super::{PacketResponder, StreamHandler};
use boolinator::Boolinator;
use crate::service::connect::error::*;
use crate::service::connect::plugin::ClientEventPlugin;
use crate::util::EventHandler;
use futures::{future, Future, IntoFuture, Sink, Stream};
use std::net::SocketAddr;
use std::{sync::Arc, time::Duration};
use tap::{TapOps, TapResultOps};
use tokio::codec::Decoder;
use tokio::net::TcpStream;
use tokio::prelude::{FutureExt, StreamExt};

pub struct ClientStreamHandler<R: PacketResponder, P: PacketCodecProvider> {
  on_connect: EventHandler<SocketAddr>,
  on_disconnect: EventHandler<SocketAddr>,
  on_error: EventHandler<ConnectServiceError>,
  codec_provider: P,
  max_idle_time: Duration,
  max_requests: usize,
  max_unresponsive_time: Duration,
  responder: Arc<R>,
}

impl<R, P> ClientStreamHandler<R, P>
where
  R: PacketResponder,
  P: PacketCodecProvider,
{
  pub fn new(responder: R, codec_provider: P) -> Self {
    ClientStreamHandler {
      on_connect: EventHandler::new(),
      on_disconnect: EventHandler::new(),
      on_error: EventHandler::new(),
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

  pub fn register_plugin(&self, plugin: impl ClientEventPlugin) {
    let plugin = Arc::new(plugin);
    self
      .on_connect
      .subscribe_fn(closet!([plugin] move |event| plugin.on_connect(event)));
    self
      .on_disconnect
      .subscribe_fn(closet!([plugin] move |event| plugin.on_disconnect(event)));
    self
      .on_error
      .subscribe_fn(closet!([plugin] move |event| plugin.on_error(event)));
  }
}

impl<R, P> StreamHandler for ClientStreamHandler<R, P>
where
  R: PacketResponder,
  P: PacketCodecProvider,
{
  /// Bootstraps the connection stream.
  fn handle(&self, stream: TcpStream) -> ConnectServiceFuture<()> {
    let peer_addr = future::result(
      stream
        .peer_addr()
        .map_err(ClientError::CannotResolveAddress)
        .map_err(ConnectServiceError::from),
    );

    let (writer, reader) = self.codec_provider.create()
      // Use a non C3/C4 encrypted TCP codec
      .framed(stream)
      // Split the stream value into two separate handles
      .split();

    let responder = self.responder.clone();
    let max_unresponsive_time = self.max_unresponsive_time;

    let communicate = reader
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

    let on_connect = self.on_connect.clone();
    let on_disconnect = self.on_disconnect.clone();
    let on_error = self.on_error.clone();

    let session = peer_addr.and_then(move |socket| {
      on_connect
        .dispatch(socket)
        .ok_or(ServerError::ClientRejected.into())
        .into_future()
        .and_then(|_| communicate)
        .then(move |result| {
          result
            .tap_err(|error| on_error.dispatch_ref(error))
            .tap(|_| on_disconnect.dispatch(socket))
        })
    });

    Box::new(session)
  }
}

/// Returns a packet limiter.
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
