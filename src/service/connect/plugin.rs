use super::ConnectServiceError;
use chashmap::CHashMap;
use crate::util::EventArgs;
use failure::Fail;
use log::{error, info, warn};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A trait describing a listener event plugin.
pub trait ListenerEventPlugin: Send + Sync + 'static {
  fn on_startup(&self, _event: &mut EventArgs<SocketAddr>) {}
  fn on_error(&self, _event: &mut EventArgs<ConnectServiceError>) {}
}

/// A trait describing a client event plugin.
pub trait ClientEventPlugin: Send + Sync + 'static {
  fn on_connect(&self, _event: &mut EventArgs<SocketAddr>) {}
  fn on_disconnect(&self, _event: &mut EventArgs<SocketAddr>) {}
  fn on_error(&self, _event: &mut EventArgs<ConnectServiceError>) {}
}

/// Plugin logging any listener events.
pub struct ListenerEventLogger;

impl ListenerEventPlugin for ListenerEventLogger {
  fn on_startup(&self, event: &mut EventArgs<SocketAddr>) {
    info!("Connect service listening on {}", event.data());
  }

  fn on_error(&self, event: &mut EventArgs<ConnectServiceError>) {
    error!("Client listener — {}", event.data());
    for cause in (event.data() as &Fail).iter_causes() {
      error!("— {}", cause);
    }
  }
}

/// Plugin logging any client events.
pub struct ClientEventLogger;

impl ClientEventPlugin for ClientEventLogger {
  fn on_connect(&self, event: &mut EventArgs<SocketAddr>) {
    info!("Client connected: {}", event.data());
  }

  fn on_disconnect(&self, event: &mut EventArgs<SocketAddr>) {
    info!("Client disconnected: {}", event.data());
  }

  fn on_error(&self, event: &mut EventArgs<ConnectServiceError>) {
    let error = event.data();
    if !error.connection_reset_by_peer() && !error.reject_by_server() {
      warn!("Client session — {}", error);
      for cause in (error as &Fail).iter_causes() {
        warn!("— {}", cause);
      }
    }
  }
}

/// Plugin restricting maximum clients per IP.
pub struct CheckMaximumClientsPerIp {
  clients: CHashMap<SocketAddr, usize>,
  capacity_per_ip: usize,
}

impl CheckMaximumClientsPerIp {
  pub fn new(capacity_per_ip: usize) -> Self {
    CheckMaximumClientsPerIp {
      clients: CHashMap::new(),
      capacity_per_ip,
    }
  }
}

impl ClientEventPlugin for CheckMaximumClientsPerIp {
  fn on_connect(&self, event: &mut EventArgs<SocketAddr>) {
    let socket = *event.data();
    let mut is_capacity_reached_for_ip = false;

    self.clients.upsert(
      socket,
      || 1,
      |count| {
        is_capacity_reached_for_ip = *count == self.capacity_per_ip;
        *count += 1;
      },
    );

    if is_capacity_reached_for_ip {
      warn!(
        "Client refused from {}; maximum connections reached for IP",
        socket
      );
      event.prevent_default();
    }
  }

  fn on_disconnect(&self, event: &mut EventArgs<SocketAddr>) {
    *self
      .clients
      .get_mut(event.data())
      .expect("Invalid client state") -= 1;
  }
}

/// Plugin restricting maximum clients.
pub struct CheckMaximumClients {
  clients: AtomicUsize,
  capacity: usize,
}

impl CheckMaximumClients {
  pub fn new(capacity: usize) -> Self {
    CheckMaximumClients {
      clients: AtomicUsize::new(0),
      capacity,
    }
  }
}

impl ClientEventPlugin for CheckMaximumClients {
  fn on_connect(&self, event: &mut EventArgs<SocketAddr>) {
    if self.clients.fetch_add(1, Ordering::SeqCst) == self.capacity {
      warn!(
        "Client refused from {}; client capacity reached ({})",
        event.data(),
        self.capacity
      );
      event.prevent_default();
    }
  }

  fn on_disconnect(&self, _event: &mut EventArgs<SocketAddr>) {
    self.clients.fetch_sub(1, Ordering::SeqCst);
  }
}
