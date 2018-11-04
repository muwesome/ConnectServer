use super::proto;
use crate::state::{RealmBrowser, RealmServer};
use crate::util::CloseSignal;
use futures::{Future, Stream};
use grpcio::{ClientStreamingSink, RequestStream, RpcContext, RpcStatus, RpcStatusCode};
use log::error;
use tap::TapResultOps;
use try_from::TryFrom;

/// Shorthand macro for creating an RPC error status.
macro_rules! rpcerr {
  ($e:ident, $($arg:tt)*) => {
    RpcStatus::new(RpcStatusCode::$e, Some(format!($($arg)*)))
  };
}

#[derive(Clone)]
pub struct RpcListener {
  close_rx: CloseSignal,
  realms: RealmBrowser,
}

impl RpcListener {
  pub fn new(realms: RealmBrowser, close_rx: CloseSignal) -> Self {
    RpcListener { realms, close_rx }
  }
}

impl proto::RealmService for RpcListener {
  fn register_realm(
    &self,
    ctx: RpcContext,
    stream: RequestStream<proto::RealmParams>,
    sink: ClientStreamingSink<proto::RealmResult>,
  ) {
    let realm_stream = stream
      // Apply context for any potential errors
      .map_err(|error| rpcerr!(Aborted, "Stream closed: {}", error))
      // Require the realm field to be specified
      .and_then(|input| input.kind.ok_or_else(|| {
        rpcerr!(InvalidArgument, "Kind not specified")
      }));

    let await_realm_definition = realm_stream.into_future()
      // Discard the stream in case of an error
      .map_err(|(error, _)| error)
      // Wait for the first input; the realm definition
      .and_then(|(input, stream)| {
        let input = input.ok_or_else(|| rpcerr!(Cancelled, "Missing input"))?;
        let definition = matches_opt!(input, proto::RealmParams_oneof_kind::definition(x) => x)
          .ok_or_else(|| rpcerr!(InvalidArgument, "Expected realm definition"))?;

        let realm = RealmServer::try_from(definition)
          .map_err(|error| rpcerr!(InvalidArgument, "Realm insert failed: {}", error))?;
        Ok((realm, stream))
      });

    let realms = self.realms.clone();
    let process_realm_updates = await_realm_definition
      // Add the realm server to the browser
      .and_then(closet!([realms] move |(realm, stream)| {
        let realm_id = realm.id;
        realms.add(realm).map_err(|_| rpcerr!(InvalidArgument, "Non-unique realm ID"))?;
        Ok((realm_id, stream))
      }))
      // Wait for any realm updates
      .and_then(move |(realm_id, stream)| {
        // Iterate over each status update
        stream.for_each(closet!([realms] move |input| {
          let status = matches_opt!(input, proto::RealmParams_oneof_kind::status(x) => x)
            .ok_or_else(|| rpcerr!(InvalidArgument, "Expected realm status"))?;
          realms.update(realm_id, |realm| {
            realm.clients = status.get_clients() as usize;
            realm.capacity = status.get_capacity() as usize;
          }).map_err(|error| rpcerr!(Internal, "Realm update failed: {}", error))
        })).then(move |result| result.and(
          realms
            .remove(realm_id)
            .map_err(|error| rpcerr!(Internal, "Realm removal failed: {}", error))
        ))
      }).then(|result| result.tap_err(|error| error!("RPC client: {:?}", error)));

    let close_signal = self
      .close_rx
      .clone()
      .then(|_| Err(rpcerr!(Unavailable, "Shutting down")));

    let send_response = process_realm_updates
      // Check for a potential close signal
      .select(close_signal)
      // Notify the client of the outcome
      .then(|result| match result {
        Ok(_) => sink.success(proto::RealmResult::new()),
        Err((error, _)) => sink.fail(error),
      });

    let session = send_response.map_err(|error| {
      if !matches!(error, grpcio::Error::RemoteStopped) {
        error!("RPC sink: {}", error)
      }
    });

    // Dispatch the session
    ctx.spawn(session);
  }
}
