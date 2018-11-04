use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

pub trait ConnectServiceConfig: Send + Sync + 'static {
  fn host(&self) -> IpAddr;

  fn port(&self) -> u16;

  fn max_idle_time(&self) -> Duration;

  fn max_unresponsive_time(&self) -> Duration;

  fn max_packet_size(&self) -> usize;

  fn max_requests(&self) -> usize;

  fn max_connections(&self) -> usize;

  fn max_connections_per_ip(&self) -> usize;

  fn ignore_unknown_packets(&self) -> bool;

  fn socket(&self) -> SocketAddr {
    SocketAddr::new(self.host(), self.port())
  }
}
