use chashmap::CHashMap;
use crate::util::Dispatcher;
use failure::Fail;
use parking_lot::Mutex;
use std::{cell::RefCell, ops::DerefMut};
use std::{fmt, sync::Arc};

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

pub trait RealmListener: Send + Sync {
  fn on_register(&self, _realm: &RealmServer) {}
  fn on_deregister(&self, _realm: &RealmServer) {}
  fn on_update(&self, _realm: &RealmServer) {}
}

#[derive(Fail, Debug)]
pub enum RealmBrowserError {
  #[fail(display = "Non-unique realm ID")]
  DuplicateId,

  #[fail(display = "Inexistent realm ID")]
  InexistentId,
}

struct RealmBrowserInner {
  dispatcher: Dispatcher<RealmListener>,
}

#[derive(Clone)]
pub struct RealmBrowser {
  inner: Arc<Mutex<RealmBrowserInner>>,
  map: Arc<CHashMap<RealmServerId, RealmServer>>,
}

impl RealmBrowser {
  pub fn new() -> Self {
    RealmBrowser {
      inner: Arc::new(Mutex::new(RealmBrowserInner {
        dispatcher: Dispatcher::new(),
      })),
      map: Arc::new(CHashMap::new()),
    }
  }

  pub fn subscribe(&self, listener: &Arc<RealmListener>) {
    let inner = self.inner.lock();
    inner.dispatcher.subscribe(listener);
  }

  pub fn add(&self, realm: RealmServer) -> Result<(), RealmBrowserError> {
    if self.map.contains_key(&realm.id) {
      Err(RealmBrowserError::DuplicateId)?;
    }

    let inner = self.inner.lock();
    inner.dispatcher.dispatch(|l| l.on_register(&realm));
    self.map.insert_new(realm.id, realm);
    Ok(())
  }

  pub fn remove(&self, id: RealmServerId) -> Result<(), RealmBrowserError> {
    let inner = self.inner.lock();
    self
      .map
      .get(&id)
      .map(|realm| inner.dispatcher.dispatch(|l| l.on_deregister(&realm)))
      .ok_or(RealmBrowserError::InexistentId)?;
    self.map.remove(&id);
    Ok(())
  }

  pub fn update<F>(&self, id: RealmServerId, mutator: F) -> Result<(), RealmBrowserError>
  where
    F: FnOnce(&mut RealmServer),
  {
    let inner = self.inner.lock();
    self
      .map
      .get_mut(&id)
      .map(|mut realm| {
        mutator(&mut realm);
        inner.dispatcher.dispatch(|l| l.on_update(&realm));
      }).ok_or(RealmBrowserError::InexistentId)
  }

  pub fn for_each<F: FnMut(&RealmServer)>(&self, func: F) {
    let func = RefCell::new(func);
    self.map.retain(|_, realm| {
      func.borrow_mut().deref_mut()(&realm);
      true
    });
  }

  pub fn get<'a>(
    &'a self,
    id: RealmServerId,
  ) -> Result<impl std::ops::Deref<Target = RealmServer> + 'a, RealmBrowserError> {
    self.map.get(&id).ok_or(RealmBrowserError::InexistentId)
  }

  pub fn len(&self) -> usize {
    self.map.len()
  }
}
