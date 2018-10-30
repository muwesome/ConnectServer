use chashmap::CHashMap;
use crate::util::Dispatcher;
use failure::Fail;
use index_pool::IndexPool;
use parking_lot::Mutex;
use std::cell::Cell;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::{fmt, sync::Arc};

pub type ClientId = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Client {
  id: ClientId,
  socket: SocketAddrV4,
}

impl Client {
  pub fn new(id: ClientId, socket: SocketAddrV4) -> Self {
    Client { id, socket }
  }
}

impl fmt::Display for Client {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} <{}>", &self.socket, self.id)
  }
}

pub trait ClientListener: Send + Sync {
  fn on_connect(&self, _client: &Client) {}
  fn on_disconnect(&self, _client: &Client) {}
}

#[derive(Fail, Debug)]
pub enum ClientPoolError {
  #[fail(display = "Inexistent client ID")]
  InexistentId,

  #[fail(display = "Max client capacity")]
  MaxCapacity,

  #[fail(display = "Max client capacity for IP: {}", _0)]
  MaxCapacityForIp(Ipv4Addr),
}

struct ClientPoolInner {
  dispatcher: Dispatcher<ClientListener>,
  pool: IndexPool,
}

#[derive(Clone)]
pub struct ClientPool {
  inner: Arc<Mutex<ClientPoolInner>>,
  map: Arc<CHashMap<ClientId, Client>>,
  capacity: usize,
  capacity_per_ip: usize,
}

impl ClientPool {
  pub fn new(capacity: usize, capacity_per_ip: usize) -> Self {
    let inner = Arc::new(Mutex::new(ClientPoolInner {
      pool: IndexPool::new(),
      dispatcher: Dispatcher::new(),
    }));
    ClientPool {
      map: Arc::new(CHashMap::new()),
      inner,
      capacity,
      capacity_per_ip,
    }
  }

  pub fn subscribe(&self, listener: &Arc<ClientListener>) {
    let inner = self.inner.lock();
    inner.dispatcher.subscribe(listener);
  }

  pub fn add(&self, socket: SocketAddrV4) -> Result<ClientHandle, ClientPoolError> {
    // TODO: Should these conditions be checked externally?
    if self.map.len() >= self.capacity {
      Err(ClientPoolError::MaxCapacity)?;
    }

    if self.num_of_clients_for_ip(*socket.ip()) > self.capacity_per_ip - 1 {
      Err(ClientPoolError::MaxCapacityForIp(*socket.ip()))?;
    }

    let mut inner = self.inner.lock();
    let client_id = inner.pool.new_id();
    let client = Client::new(client_id, socket);

    inner.dispatcher.dispatch(|l| l.on_connect(&client));
    self.map.insert_new(client_id, client);

    Ok(ClientHandle {
      inner: self.inner.clone(),
      map: self.map.clone(),
      id: client_id,
    })
  }

  /// Returns the number of clients for an IP address.
  fn num_of_clients_for_ip(&self, ip: Ipv4Addr) -> usize {
    let ip_clients = Cell::new(0);
    self.map.retain(|_, client| {
      if *client.socket.ip() == ip {
        ip_clients.set(ip_clients.get() + 1);
      }
      true
    });
    ip_clients.into_inner()
  }
}

pub struct ClientHandle {
  inner: Arc<Mutex<ClientPoolInner>>,
  map: Arc<CHashMap<ClientId, Client>>,
  id: ClientId,
}

impl Drop for ClientHandle {
  fn drop(&mut self) {
    let mut inner = self.inner.lock();

    inner.pool.return_id(self.id).expect("Invalid pool state");
    self
      .map
      .get(&self.id)
      .map(|client| inner.dispatcher.dispatch(|l| l.on_disconnect(&client)))
      .expect("Invalid pool state");
    self.map.remove(&self.id);
  }
}
