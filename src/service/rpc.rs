use crate::{state::RealmBrowser, util::ThreadController, Result};
use failure::ResultExt;
use futures::{sync::oneshot, Future};
use grpcio::{Environment, ServerBuilder};
use std::{sync::Arc, thread};

mod listener;
mod proto;

pub struct RpcService(ThreadController);

impl RpcService {
  pub fn spawn<S: Into<String>>(host: S, port: u16, realms: RealmBrowser) -> Result<Self> {
    let service = proto::create_connect_service(listener::RpcListener::new(realms));

    let environment = Arc::new(Environment::new(1));

    let mut server = ServerBuilder::new(environment)
      .register_service(service)
      .bind(host, port)
      .build()
      .context("Failed to build service")?;

    let (tx, rx) = oneshot::channel();
    let handle = thread::spawn(move || {
      server.start();
      for &(ref host, port) in server.bind_addrs() {
        println!("RPC listening on {}:{}", host, port);
      }

      rx.wait().context("Thread transmitter closed prematurely")?;
      server
        .shutdown()
        .wait()
        .context("Error whilst shutting down service")
        .map_err(From::from)
    });

    Ok(RpcService(ThreadController::new(tx, handle)))
  }

  pub fn stop(self) -> Result<()> {
    self.0.stop()
  }
}
