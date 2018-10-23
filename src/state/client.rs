use crate::util::{Dispatcher, Event, Listener};
use evmap::{self, ReadHandle, ShallowCopy, WriteHandle};
use failure::Fail;
use index_pool::IndexPool;
use parking_lot::Mutex;
use std::collections::hash_map::RandomState;
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

impl ShallowCopy for Client {
  unsafe fn shallow_copy(&mut self) -> Self {
    self.clone()
  }
}

pub enum ClientEvent {
  Connect,
  Disconnect,
}

impl Event for ClientEvent {
  type Context = Client;
}

#[derive(Fail, Debug)]
pub enum ClientError {
  #[fail(display = "Inexistent client ID")]
  InexistentId,

  #[fail(display = "Max client capacity")]
  MaxCapacity,

  #[fail(display = "Max client capacity for IP: {}", _0)]
  MaxCapacityForIp(Ipv4Addr),
}

/// Realm server collection reader.
type ClientReader = ReadHandle<ClientId, Client, (), RandomState>;

/// Client server collection writer.
type ClientWriter = WriteHandle<ClientId, Client, (), RandomState>;

struct ClientPoolInner {
  dispatcher: Dispatcher<ClientEvent>,
  writer: ClientWriter,
  pool: IndexPool,
}

#[derive(Clone)]
pub struct ClientPool {
  inner: Arc<Mutex<ClientPoolInner>>,
  reader: ClientReader,
  capacity: usize,
  capacity_per_ip: usize,
}

impl ClientPool {
  pub fn new(capacity: usize, capacity_per_ip: usize) -> Self {
    let (reader, writer) = evmap::new();
    let inner = Arc::new(Mutex::new(ClientPoolInner {
      pool: IndexPool::new(),
      dispatcher: Dispatcher::new(),
      writer,
    }));
    ClientPool {
      reader,
      inner,
      capacity,
      capacity_per_ip,
    }
  }

  pub fn add_listener<L>(&self, listener: &Arc<Mutex<L>>)
  where
    L: Listener<ClientEvent> + Send + Sync + 'static,
  {
    let mut inner = self.inner.lock();
    inner.dispatcher.add_listener(listener);
  }

  pub fn add(&self, socket: SocketAddrV4) -> Result<ClientHandle, ClientError> {
    // TODO: Should these conditions be checked externally?
    if self.reader.len() >= self.capacity {
      Err(ClientError::MaxCapacity)?;
    }

    if self.num_of_clients_for_ip(*socket.ip()) > self.capacity_per_ip - 1 {
      Err(ClientError::MaxCapacityForIp(*socket.ip()))?;
    }

    let mut inner = self.inner.lock();
    let client_id = inner.pool.new_id();
    let client = Client::new(client_id, socket);

    inner.dispatcher.dispatch(&ClientEvent::Connect, &client);
    inner.writer.insert(client_id, client);
    inner.writer.refresh();

    Ok(ClientHandle {
      inner: self.inner.clone(),
      reader: self.reader.clone(),
      id: client_id,
    })
  }

  /// Returns the number of clients for an IP address.
  fn num_of_clients_for_ip(&self, ip: Ipv4Addr) -> usize {
    let mut ip_clients = 0;
    self.reader.for_each(|_, client| {
      if *client[0].socket.ip() == ip {
        ip_clients += 1;
      }
    });
    ip_clients
  }
}

pub struct ClientHandle {
  inner: Arc<Mutex<ClientPoolInner>>,
  reader: ClientReader,
  id: ClientId,
}

impl Drop for ClientHandle {
  fn drop(&mut self) {
    let mut inner = self.inner.lock();

    inner.pool.return_id(self.id).expect("Invalid pool state");
    self
      .reader
      .get_and(&self.id, |client| {
        let event = ClientEvent::Disconnect;
        inner.dispatcher.dispatch(&event, &client[0]);
      }).expect("Invalid pool state");

    inner.writer.empty(self.id);
    inner.writer.refresh();
  }
}
