use super::{plugin::RealmEventPlugin, proto};
use crate::state::{RealmServer, RealmServerId, RealmServerList};
use crate::util::{CloseSignal, EventHandler, StreamExt};
use futures::{Future, Stream};
use grpcio::{ClientStreamingSink, RequestStream, RpcContext, RpcStatus, RpcStatusCode};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use try_from::TryFrom;

/// Shorthand macro for creating an RPC status error.
macro_rules! rpcerr {
  ($e:ident, $($arg:tt)*) => {
    RpcStatus::new(RpcStatusCode::$e, Some(format!($($arg)*)))
  };
}

#[derive(Clone)]
pub struct RealmRpc {
  on_register: EventHandler<RealmServer>,
  on_deregister: EventHandler<RealmServer>,
  on_update: EventHandler<RealmServer>,
  on_error: EventHandler<grpcio::Error>,
  close_rx: CloseSignal,
  realms: RealmServerList,
}

impl RealmRpc {
  pub fn new(realms: RealmServerList, close_rx: CloseSignal) -> Self {
    RealmRpc {
      on_register: EventHandler::new(),
      on_deregister: EventHandler::new(),
      on_update: EventHandler::new(),
      on_error: EventHandler::new(),
      realms,
      close_rx,
    }
  }

  pub fn register_plugin(&self, plugin: impl RealmEventPlugin) {
    let plugin = Arc::new(plugin);
    self
      .on_register
      .subscribe_fn(closet!([plugin] move |event| plugin.on_register(event)));
    self
      .on_update
      .subscribe_fn(closet!([plugin] move |event| plugin.on_update(event)));
    self
      .on_deregister
      .subscribe_fn(closet!([plugin] move |event| plugin.on_deregister(event)));
    self
      .on_error
      .subscribe_fn(closet!([plugin] move |event| plugin.on_error(event)));
  }

  fn add_realm(
    &self,
    realm: proto::RealmParams_RealmDefinition,
  ) -> Result<RealmServerId, RpcStatus> {
    let realm = RealmServer::try_from(realm)
      .map_err(|error| rpcerr!(InvalidArgument, "Realm parsing failed: {}", error))?;
    let realm_id = realm.id;
    self
      .realms
      .add(realm)
      .map_err(|error| rpcerr!(InvalidArgument, "Realm registration failed: {}", error))?;
    self
      .on_register
      .dispatch_ref(&*self.realms.get(realm_id).expect("Invalid realm state"));
    Ok(realm_id)
  }

  fn update_realm(
    &self,
    id: RealmServerId,
    status: &proto::RealmParams_RealmStatus,
  ) -> Result<(), RpcStatus> {
    self
      .realms
      .get_mut(id)
      .map(|mut realm| {
        realm.clients = status.get_clients() as usize;
        realm.capacity = status.get_capacity() as usize;
        self.on_update.dispatch_ref(&*realm);
      }).map_err(|error| rpcerr!(InvalidArgument, "Realm update failed: {}", error))
  }

  fn remove_realm(&self, id: RealmServerId) -> Result<(), RpcStatus> {
    self
      .realms
      .remove(id)
      .map_err(|error| rpcerr!(Internal, "Realm deregister failed: {}", error))
      .map(|realm| self.on_deregister.dispatch_ref(&realm))?;
    Ok(())
  }
}

impl proto::RealmService for RealmRpc {
  fn register_realm(
    &self,
    ctx: RpcContext,
    stream: RequestStream<proto::RealmParams>,
    sink: ClientStreamingSink<proto::RealmResult>,
  ) {
    let stream = stream
      // Apply context for any potential errors
      .map_err(|error| rpcerr!(Aborted, "Stream closed: {}", error))
      // Require the realm field to be specified
      .and_then(|input| input.kind.ok_or_else(|| rpcerr!(InvalidArgument, "Kind not specified")));

    let this = self.clone();
    let realm_id = Arc::new(AtomicUsize::new(0));

    let wait_for_realm_register = stream
      // Require one item for registering
      .next_or_else(|| rpcerr!(Cancelled, "Missing input"))
      // Process the realm registration
      .and_then(closet!([this, realm_id] move |(input, stream)| {
        let definition = matches_opt!(input, proto::RealmParams_oneof_kind::definition(x) => x)
          .ok_or_else(|| rpcerr!(InvalidArgument, "Expected realm definition"))?;
        realm_id.store(this.add_realm(definition)? as usize, Ordering::Relaxed);
        Ok(stream)
      })).flatten_stream();

    let process_realm_updates = wait_for_realm_register
      // Update the internal state for each status update
      .for_each(closet!([this, realm_id] move |input| {
        let status = matches_opt!(input, proto::RealmParams_oneof_kind::status(x) => x)
          .ok_or_else(|| rpcerr!(InvalidArgument, "Expected realm status"))?;
        this.update_realm(realm_id.load(Ordering::Relaxed) as RealmServerId, &status)
      }))
      // Remove the realm after deregistering
      .then(closet!([this] move |result| {
        result.and(this.remove_realm(realm_id.load(Ordering::Relaxed) as RealmServerId))
      }));

    let session = process_realm_updates
      // Check for a potential close signal
      .select(self.close_rx.clone().then(|_| Err(rpcerr!(Unavailable, "Shutting down"))))
      // Notify the client of the outcome
      .then(|result| match result {
        Ok(_) => sink.success(proto::RealmResult::new()),
        Err((error, _)) => sink.fail(error),
      })
      .map_err(move |error| { this.on_error.dispatch(error); });

    // Dispatch the session
    ctx.spawn(session);
  }
}
