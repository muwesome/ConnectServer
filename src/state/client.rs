use crate::util::{Dispatcher, Event, Listener};
use crate::Result;
use evmap::{self, ReadHandle, ShallowCopy, WriteHandle};
use failure::{Context, ResultExt};
use index_pool::IndexPool;
use std::collections::hash_map::RandomState;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{fmt, net::SocketAddrV4};

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
}

impl ClientPool {
  pub fn new(capacity: usize) -> Self {
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
    }
  }

  pub fn add_listener<L>(&self, listener: &Arc<Mutex<L>>) -> Result<()>
  where
    L: Listener<ClientEvent> + Send + Sync + 'static,
  {
    let mut inner = self.inner()?;
    inner.dispatcher.add_listener(listener);
    Ok(())
  }

  pub fn add(&self, socket: SocketAddrV4) -> Result<ClientId> {
    if self.reader.len() >= self.capacity {
      Err(Context::new("Client pool is full"))?;
    }

    let mut inner = self.inner()?;
    let client_id = inner.pool.new_id();
    let client = Client::new(client_id, socket);

    inner.dispatcher.dispatch(&ClientEvent::Connect, &client);
    inner.writer.insert(client_id, client);
    inner.writer.refresh();

    Ok(client_id)
  }

  pub fn remove(&self, id: ClientId) -> Result<()> {
    let mut inner = self.inner()?;

    inner.pool.return_id(id).context("Non existent client ID")?;
    self.reader.get_and(&id, |client| {
      let event = ClientEvent::Disconnect;
      inner.dispatcher.dispatch(&event, &client[0]);
    });

    inner.writer.empty(id);
    inner.writer.refresh();
    Ok(())
  }

  fn inner(&self) -> Result<MutexGuard<ClientPoolInner>> {
    Ok(
      self
        .inner
        .lock()
        .map_err(|_| Context::new("Client manager inner deadlock"))?,
    )
  }
}
