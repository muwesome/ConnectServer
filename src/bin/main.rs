use std::net::{Ipv4Addr, SocketAddrV4};
use mucs::ConnectServer;
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

  server.wait().expect("Failed to wait");
  println!("WE ENDZ NOWZ BOYZ ;)");
}
