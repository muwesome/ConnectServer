use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;

#[cfg(feature = "build-binary")]
use structopt::StructOpt;

#[cfg_attr(feature = "build-binary", derive(StructOpt))]
pub struct ClientServiceConfig {
  #[cfg_attr(
    feature = "build-binary",
    structopt(
      short = "h",
      long = "host",
      help = "Bind to this IPv4 client address",
      default_value = "0.0.0.0"
    )
  )]
  pub host: Ipv4Addr,

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
}

impl ClientServiceConfig {
  pub fn socket(&self) -> SocketAddrV4 {
    SocketAddrV4::new(self.host, self.port)
  }
}
