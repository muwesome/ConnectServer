use super::proto;
use crate::state::{RealmBrowser, RealmServer};
use failure::{Context, Error, Fail, ResultExt};
use futures::{Future, Stream};
use grpcio::{ClientStreamingSink, RequestStream, RpcContext, RpcStatus, RpcStatusCode};
use try_from::TryFrom;

#[derive(Clone)]
pub struct RpcListener {
  realms: RealmBrowser,
}

impl RpcListener {
  pub fn new(realms: RealmBrowser) -> Self {
    RpcListener { realms }
  }
}

impl proto::ConnectService for RpcListener {
  fn register_realm(
    &self,
    ctx: RpcContext,
    stream: RequestStream<proto::RealmParams>,
    sink: ClientStreamingSink<proto::RealmResult>,
  ) {
    let realm_stream = stream
      // Apply context for any potential errors
      .map_err(|error| error.context("RPC receiving error").into())
      // Require the realm field to be specified
      .and_then(|input| {
        input
          .realm
          .ok_or_else(|| Context::new("Invalid input; realm must be specified").into())
      });

    let await_realm_definition = realm_stream.into_future()
      // Discard the stream in case of an error
      .map_err(|(error, _)| error)
      // Wait for the first input; the realm definition
      .and_then(|(input, stream)| {
        let input = input.ok_or_else(|| Context::new("Unexpected end of data"))?;
        let definition = opt_match!(input, proto::RealmParams_oneof_realm::definition(x) => x)
          .ok_or_else(|| Context::new("Invalid input; expected realm definition"))?;

        let realm = RealmServer::try_from(definition)
          .context("Invalid realm definition")?;
        Ok((realm, stream))
      });

    let realms = self.realms.clone();
    let process_realm_updates = await_realm_definition
      // Add the realm server to the browser
      .and_then(closet!([realms] move |(realm, stream)| {
        let realm_id = realm.id;
        realms.add(realm)?;
        println!("Registered realm");
        Ok((realm_id, stream))
      }))
      // Wait for any realm updates
      .and_then(move |(realm_id, stream)| {
        // Iterate over each status update
        stream.for_each(closet!([realms] move |input| {
          let status = opt_match!(input, proto::RealmParams_oneof_realm::status(x) => x)
            .ok_or_else(|| Context::new("Invalid input; expected realm status"))?;

          realms.update(realm_id, |realm| {
            println!("Updated realm");
            realm.clients = status.get_clients() as usize;
            realm.capacity = status.get_capacity() as usize;
          })
        })).then(move |result| {
          // TODO: Introduce 'tap/inspect_any' here?
          println!("Unregistered realm");
          realms.remove(realm_id)?;
          result
        })
      });

    let send_response = process_realm_updates
      // Notify the client of the outcome
      .then(|result| {
        match result {
          Ok(_) => sink.success(proto::RealmResult::new()),
          Err(error) => {
            // TODO: Identify and use proper RPC status code
            sink.fail(RpcStatus::new(RpcStatusCode::Unknown, Some(error.to_string())))
          }
        }.map_err(|error| Error::from(error.context("RPC transmission error")))
      });

    let session = send_response.map_err(|error| {
      println!("TODO: LOG {:?}", error);
    });

    // Dispatch the session
    ctx.spawn(session);
  }
}
