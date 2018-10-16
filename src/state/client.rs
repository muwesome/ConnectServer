use crate::Result;
use evmap::{self, ReadHandle, ShallowCopy, WriteHandle};
use failure::{Context, ResultExt};
use index_pool::IndexPool;
use std::collections::hash_map::RandomState;
use std::net::SocketAddrV4;
use std::sync::{Arc, Mutex, MutexGuard};

pub type ClientId = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Client {
  socket: SocketAddrV4,
}

impl Client {
  pub fn new(socket: SocketAddrV4) -> Self {
    Client { socket }
  }
}

impl ShallowCopy for Client {
  unsafe fn shallow_copy(&mut self) -> Self {
    self.clone()
  }
}

/// Realm server collection reader.
type ClientReader = ReadHandle<ClientId, Client, (), RandomState>;

/// Client server collection writer.
type ClientWriter = WriteHandle<ClientId, Client, (), RandomState>;

struct ClientManagerInner {
  writer: ClientWriter,
  pool: IndexPool,
}

#[derive(Clone)]
pub struct ClientManager {
  inner: Arc<Mutex<ClientManagerInner>>,
  reader: ClientReader,
}

impl ClientManager {
  pub fn new() -> Self {
    let (reader, writer) = evmap::new();
    ClientManager {
      reader,
      inner: Arc::new(Mutex::new(ClientManagerInner {
        pool: IndexPool::new(),
        writer,
      })),
    }
  }

  pub fn add(&self, client: Client) -> Result<ClientId> {
    let mut inner = self.inner()?;
    let client_id = inner.pool.new_id();

    inner.writer.insert(client_id, client);
    inner.writer.refresh();
    Ok(client_id)
  }

  pub fn remove(&self, id: ClientId) -> Result<()> {
    let mut inner = self.inner()?;
    inner
      .pool
      .return_id(id)
      .context("Non existent client ID specified")?;
    inner.writer.empty(id);
    inner.writer.refresh();
    Ok(())
  }

  fn inner(&self) -> Result<MutexGuard<ClientManagerInner>> {
    Ok(
      self
        .inner
        .lock()
        .map_err(|_| Context::new("Client manager inner deadlock"))?,
    )
  }
}
