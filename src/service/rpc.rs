use crate::{state::RealmBrowser, util::ThreadController, Result};
use failure::ResultExt;
use futures::{sync::oneshot, Future};
use grpcio::{Environment, Server, ServerBuilder};
use std::sync::Arc;

mod listener;
mod proto;

pub struct RpcService(ThreadController);

impl RpcService {
  pub fn spawn<S: Into<String>>(host: S, port: u16, realms: RealmBrowser) -> Result<Self> {
    let service = proto::create_realm_service(listener::RpcListener::new(realms));

    let environment = Arc::new(Environment::new(1));
    let server = ServerBuilder::new(environment)
      .register_service(service)
      .bind(host, port)
      .build()
      .context("Failed to build service")?;

    let ctl = ThreadController::spawn(move |rx| Self::serve(server, rx));
    Ok(RpcService(ctl))
  }

  pub fn stop(self) -> Result<()> {
    self.0.stop()
  }

  fn serve(mut server: Server, close_signal: oneshot::Receiver<()>) -> Result<()> {
    server.start();
    for &(ref host, port) in server.bind_addrs() {
      println!("RPC listening on {}:{}", host, port);
    }

    close_signal
      .wait()
      .context("Thread transmitter closed prematurely")?;
    server
      .shutdown()
      .wait()
      .context("Error whilst shutting down service")
      .map_err(From::from)
  }
}
