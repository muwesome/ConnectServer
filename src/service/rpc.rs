use crate::{state::RealmBrowser, util::ThreadController, Result};
use failure::ResultExt;
use futures::{sync::oneshot, Future, Sink};
use grpcio::{Environment, Server, ServerBuilder};
use multiqueue::BroadcastFutSender;
use std::sync::Arc;

mod listener;
mod proto;

pub struct RpcService(ThreadController);

impl RpcService {
  pub fn spawn<S: Into<String>>(host: S, port: u16, realms: RealmBrowser) -> Result<Self> {
    let (bctx, bcrx) = multiqueue::broadcast_fut_queue(1);
    let service = proto::create_realm_service(listener::RpcListener::new(realms, bcrx));

    let environment = Arc::new(Environment::new(1));
    let server = ServerBuilder::new(environment)
      .register_service(service)
      .bind(host, port)
      .build()
      .context("Failed to build service")?;

    let ctl = ThreadController::spawn(move |csrx| Self::serve(server, csrx, bctx));
    Ok(RpcService(ctl))
  }

  pub fn stop(self) -> Result<()> {
    self.0.stop()
  }

  fn serve(
    mut server: Server,
    close_signal: oneshot::Receiver<()>,
    close_broadcast: BroadcastFutSender<()>,
  ) -> Result<()> {
    server.start();
    for &(ref host, port) in server.bind_addrs() {
      println!("RPC listening on {}:{}", host, port);
    }

    close_signal
      .then(|_| close_broadcast.send(()))
      .wait()
      .context("Thread transmitter closed prematurely")?;
    server
      .shutdown()
      .wait()
      .context("Error whilst shutting down service")
      .map_err(From::from)
  }
}
