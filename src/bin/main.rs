use mucs::ConnectServer;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
#[structopt(name = "mu-cs", about = "Mu Online Season 2 Connect Server")]
pub struct Options {
  #[structopt(
    long = "rpc-host",
    value_name = "host",
    help = "Bind RPC to this host",
    default_value = "0.0.0.0"
  )]
  pub rpc_host: String,

  #[structopt(
    long = "rpc-port",
    value_name = "port",
    help = "Bind RPC to this port",
    default_value = "0"
  )]
  pub rpc_port: u16,

  #[structopt(
    short = "h",
    long = "host",
    help = "Bind to this IPv4 address",
    default_value = "0.0.0.0"
  )]
  pub host: Ipv4Addr,

  #[structopt(
    short = "p",
    long = "port",
    help = "Bind to this port",
    default_value = "2004"
  )]
  pub port: u16,
}

fn main() {
  // Parse any CLI arguments
  let options = Options::from_args();
  let server = ConnectServer::builder()
    .socket(SocketAddrV4::new(options.host, options.port))
    .rpc(options.rpc_host, options.rpc_port)
    .spawn()
    .expect("Failed to spawn");

  let running = Arc::new(AtomicBool::new(true));
  let runningc = running.clone();

  ctrlc::set_handler(move || {
    println!("Shutting down server...");
    runningc.store(false, Ordering::SeqCst);
  }).expect("Error setting Ctrl-C handler");

  while server.is_active() && running.load(Ordering::SeqCst) {
    thread::sleep(Duration::from_millis(100));
  }

  server.stop().expect("Failed to stop server");
}
