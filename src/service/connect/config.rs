use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;

pub trait ConnectServiceConfig: Send + Sync + 'static {
  fn host(&self) -> Ipv4Addr;

  fn port(&self) -> u16;

  fn max_idle_time(&self) -> Duration;

  fn max_unresponsive_time(&self) -> Duration;

  fn max_packet_size(&self) -> usize;

  fn max_requests(&self) -> usize;

  fn max_connections(&self) -> usize;

  fn max_connections_per_ip(&self) -> usize;

  fn ignore_unknown_packets(&self) -> bool;

  fn socket(&self) -> SocketAddrV4 {
    SocketAddrV4::new(self.host(), self.port())
  }
}
