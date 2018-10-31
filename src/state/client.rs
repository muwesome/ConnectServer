use chashmap::CHashMap;
use crate::util::{Dispatcher, IndexPool};
use failure::Fail;
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

#[derive(Clone)]
pub struct ClientPool {
  dispatcher: Dispatcher<ClientListener>,
  clients: Arc<CHashMap<ClientId, Client>>,
  pool: Arc<IndexPool>,
  capacity_per_ip: usize,
}

impl ClientPool {
  pub fn new(capacity: usize, capacity_per_ip: usize) -> Self {
    ClientPool {
      dispatcher: Dispatcher::new(),
      clients: Arc::new(CHashMap::new()),
      pool: Arc::new(IndexPool::with_capacity(capacity)),
      capacity_per_ip,
    }
  }

  pub fn subscribe(&self, listener: &Arc<ClientListener>) {
    self.dispatcher.subscribe(listener);
  }

  pub fn add(&self, socket: SocketAddrV4) -> Result<ClientHandle, ClientPoolError> {
    // TODO: Should this condition be checked externally?
    if self.num_of_clients_for_ip(*socket.ip()) > self.capacity_per_ip - 1 {
      Err(ClientPoolError::MaxCapacityForIp(*socket.ip()))?;
    }

    let client_id = self.pool.new_id().ok_or(ClientPoolError::MaxCapacity)?;
    let client = Client::new(client_id, socket);

    self.dispatcher.dispatch(|l| l.on_connect(&client));
    self.clients.insert_new(client_id, client);

    Ok(ClientHandle {
      pool: self.clone(),
      id: client_id,
    })
  }

  fn remove(&self, id: ClientId) -> Result<(), ClientPoolError> {
    self
      .clients
      .remove(&id)
      .map(|client| self.dispatcher.dispatch(|l| l.on_disconnect(&client)))
      .ok_or(ClientPoolError::InexistentId)?;
    self.pool.return_id(id);
    Ok(())
  }

  /// Returns the number of clients for an IP address.
  fn num_of_clients_for_ip(&self, ip: Ipv4Addr) -> usize {
    let ip_clients = Cell::new(0);
    self.clients.retain(|_, client| {
      if *client.socket.ip() == ip {
        ip_clients.set(ip_clients.get() + 1);
      }
      true
    });
    ip_clients.into_inner()
  }
}

pub struct ClientHandle {
  pool: ClientPool,
  id: ClientId,
}

impl Drop for ClientHandle {
  fn drop(&mut self) {
    self.pool.remove(self.id).expect("Invalid pool state");
  }
}
