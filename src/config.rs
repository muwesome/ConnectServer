use crate::service::{ConnectServiceConfig, RpcServiceConfig};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

#[cfg(feature = "build-binary")]
use structopt::StructOpt;

#[derive(Clone)]
#[cfg_attr(feature = "build-binary", derive(StructOpt))]
#[cfg_attr(
  feature = "build-binary",
  structopt(about = "Mu Online Connect Server")
)]
pub struct ConnectConfig {
  #[cfg_attr(
    feature = "build-binary",
    structopt(
      short = "h",
      long = "host",
      help = "Bind to this IP client address",
      default_value = "0.0.0.0"
    )
  )]
  pub host: IpAddr,

  #[cfg_attr(
    feature = "build-binary",
    structopt(
      short = "p",
      long = "port",
      help = "Bind to this client listener port",
      default_value = "2004"
    )
  )]
  pub port: u16,

  #[cfg_attr(
    feature = "build-binary",
    structopt(
      long = "max-idle-time",
      help = "Maximum idle time until a client is disconnected",
      default_value = "100s",
      parse(try_from_str = "humantime::parse_duration")
    )
  )]
  pub max_idle_time: Duration,

  #[cfg_attr(
    feature = "build-binary",
    structopt(
      long = "max-unresponsive-time",
      help = "Maximum unresponsive time until a client is dropped",
      default_value = "60s",
      parse(try_from_str = "humantime::parse_duration")
    )
  )]
  pub max_unresponsive_time: Duration,

  #[cfg_attr(
    feature = "build-binary",
    structopt(
      long = "max-packet-size",
      help = "Maximum packet size allowed from a client",
      default_value = "6"
    )
  )]
  pub max_packet_size: usize,

  #[cfg_attr(
    feature = "build-binary",
    structopt(
      long = "max-requests",
      help = "Maximum requests allowed from a client",
      default_value = "20"
    )
  )]
  pub max_requests: usize,

  #[cfg_attr(
    feature = "build-binary",
    structopt(
      long = "max-connections",
      help = "Maximum connections the server should handle",
      default_value = "1000"
    )
  )]
  pub max_connections: usize,

  #[cfg_attr(
    feature = "build-binary",
    structopt(
      long = "max-connections-per-ip",
      help = "Maximum connections per IP",
      default_value = "1"
    )
  )]
  pub max_connections_per_ip: usize,

  #[cfg_attr(
    feature = "build-binary",
    structopt(
      long = "ignore-unknown-packets",
      help = "Ignore unknown packets from a client instead of disconnecting",
    )
  )]
  pub ignore_unknown_packets: bool,

  #[cfg_attr(
    feature = "build-binary",
    structopt(
      long = "rpc-host",
      help = "Bind to this RPC domain",
      default_value = "0.0.0.0"
    )
  )]
  pub rpc_host: String,

  #[cfg_attr(
    feature = "build-binary",
    structopt(
      long = "rpc-port",
      help = "Bind to this RPC listener port",
      default_value = "0"
    )
  )]
  pub rpc_port: u16,
}

impl ConnectConfig {
  pub fn socket(&self) -> SocketAddr {
    SocketAddr::new(self.host, self.port)
  }
}

impl ConnectServiceConfig for ConnectConfig {
  fn host(&self) -> IpAddr {
    self.host
  }

  fn port(&self) -> u16 {
    self.port
  }

  fn max_idle_time(&self) -> Duration {
    self.max_idle_time
  }

  fn max_unresponsive_time(&self) -> Duration {
    self.max_unresponsive_time
  }

  fn max_packet_size(&self) -> usize {
    self.max_packet_size
  }

  fn max_requests(&self) -> usize {
    self.max_requests
  }

  fn max_connections(&self) -> usize {
    self.max_connections
  }

  fn max_connections_per_ip(&self) -> usize {
    self.max_connections_per_ip
  }

  fn ignore_unknown_packets(&self) -> bool {
    self.ignore_unknown_packets
  }
}

impl RpcServiceConfig for ConnectConfig {
  fn host(&self) -> &str {
    &self.rpc_host
  }

  fn port(&self) -> u16 {
    self.rpc_port
  }
}
