use crate::util::Dispatcher;
use evmap::{self, ReadHandle, ShallowCopy, WriteHandle};
use failure::Fail;
use parking_lot::Mutex;
use std::sync::Arc;
use std::{collections::hash_map::RandomState, fmt};

/// A realm server identifier.
pub type RealmServerId = u16;

/// Realm server information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RealmServer {
  pub id: RealmServerId,
  pub host: String,
  pub port: u16,
  pub clients: usize,
  pub capacity: usize,
}

impl RealmServer {
  pub fn load_factor(&self) -> f32 {
    self.clients as f32 / self.capacity as f32
  }
}

impl fmt::Display for RealmServer {
  fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
    write!(
      output,
      "{}:{} <{}> [{}/{}]",
      &self.host, self.port, self.id, self.clients, self.capacity
    )
  }
}

impl ShallowCopy for RealmServer {
  unsafe fn shallow_copy(&mut self) -> Self {
    RealmServer {
      id: self.id,
      host: self.host.shallow_copy(),
      port: self.port,
      clients: self.clients,
      capacity: self.capacity,
    }
  }
}

pub trait RealmListener: Send + Sync {
  fn on_register(&self, _realm: &RealmServer) {}
  fn on_deregister(&self, _realm: &RealmServer) {}
  fn on_update(&self, _realm: &RealmServer) {}
}

#[derive(Fail, Debug)]
pub enum RealmError {
  #[fail(display = "Non-unique realm ID")]
  DuplicateId,

  #[fail(display = "Inexistent realm ID")]
  InexistentId,
}

/// Realm server collection reader.
type RealmReader = ReadHandle<RealmServerId, RealmServer, (), RandomState>;

/// Realm server collection writer.
type RealmWriter = WriteHandle<RealmServerId, RealmServer, (), RandomState>;

struct RealmBrowserInner {
  dispatcher: Dispatcher<RealmListener>,
  writer: RealmWriter,
}

#[derive(Clone)]
pub struct RealmBrowser {
  inner: Arc<Mutex<RealmBrowserInner>>,
  reader: RealmReader,
}

impl RealmBrowser {
  pub fn new() -> Self {
    let (reader, writer) = evmap::new();
    RealmBrowser {
      reader,
      inner: Arc::new(Mutex::new(RealmBrowserInner {
        dispatcher: Dispatcher::new(),
        writer,
      })),
    }
  }

  pub fn subscribe(&self, listener: &Arc<RealmListener>) {
    let inner = self.inner.lock();
    inner.dispatcher.subscribe(listener);
  }

  pub fn add(&self, realm: RealmServer) -> Result<(), RealmError> {
    if self.reader.contains_key(&realm.id) {
      Err(RealmError::DuplicateId)?;
    }

    let mut inner = self.inner.lock();
    inner.dispatcher.dispatch(|l| l.on_register(&realm));
    inner.writer.insert(realm.id, realm);
    inner.writer.refresh();
    Ok(())
  }

  pub fn remove(&self, id: RealmServerId) -> Result<(), RealmError> {
    let mut inner = self.inner.lock();
    self
      .reader
      .get_and(&id, |realm| {
        inner.dispatcher.dispatch(|l| l.on_deregister(&realm[0]));
      }).ok_or(RealmError::InexistentId)?;
    inner.writer.empty(id);
    inner.writer.refresh();
    Ok(())
  }

  pub fn update<F>(&self, id: RealmServerId, mutator: F) -> Result<(), RealmError>
  where
    F: FnOnce(&mut RealmServer),
  {
    let mut inner = self.inner.lock();
    let mut realm = inner
      .writer
      .get_and(&id, |realm| realm[0].clone())
      .ok_or(RealmError::InexistentId)?;

    mutator(&mut realm);
    inner.dispatcher.dispatch(|l| l.on_update(&realm));

    inner.writer.update(id, realm);
    inner.writer.refresh();
    Ok(())
  }

  pub fn for_each<F: FnMut(&RealmServer)>(&self, mut func: F) {
    self.reader.for_each(|_, realm| func(&realm[0]));
  }

  pub fn get<R, F: FnOnce(&RealmServer) -> R>(
    &self,
    id: RealmServerId,
    func: F,
  ) -> Result<R, RealmError> {
    self
      .reader
      .get_and(&id, |realm| func(&realm[0]))
      .ok_or(RealmError::InexistentId)
  }

  pub fn len(&self) -> usize {
    self.reader.len()
  }
}
