use crate::util::{CloseSignalFut, ThreadController};
use crate::{state::RealmBrowser, Result};
use failure::{Context, Error, ResultExt};
use futures::Future;
use grpcio::{Environment, ServerBuilder};
use std::sync::Arc;

mod listener;
mod proto;

pub struct RpcService(ThreadController);

impl RpcService {
  pub fn spawn<S: Into<String>>(host: S, port: u16, realms: RealmBrowser) -> Self {
    let host = host.into();
    let ctl = ThreadController::spawn(move |rx| Self::serve(host, port, realms, rx));
    RpcService(ctl)
  }

  pub fn stop(self) -> Result<()> {
    self.0.stop()
  }

  fn serve(host: String, port: u16, realms: RealmBrowser, close_rx: CloseSignalFut) -> Result<()> {
    let service = proto::create_realm_service(listener::RpcListener::new(realms, close_rx.clone()));

    let environment = Arc::new(Environment::new(1));
    let mut server = ServerBuilder::new(environment)
      .register_service(service)
      .bind(host, port)
      .build()
      .context("Failed to build service")?;

    server.start();
    for &(ref host, port) in server.bind_addrs() {
      println!("RPC listening on {}:{}", host, port);
    }

    close_rx
      .wait()
      .map_err(|_| Error::from(Context::new("Thread transmitter closed prematurely")))?;
    server
      .shutdown()
      .wait()
      .context("Error whilst shutting down service")
      .map_err(From::from)
  }
}
