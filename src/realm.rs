use evmap::{self, ReadHandle, ShallowCopy, WriteHandle};
use failure::Context;
use std::collections::hash_map::RandomState;
use std::sync::{Arc, Mutex, MutexGuard};
use Result;

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

/// Realm server collection reader.
type RealmReader = ReadHandle<RealmServerId, RealmServer, (), RandomState>;

/// Realm server collection writer.
type RealmWriter = WriteHandle<RealmServerId, RealmServer, (), RandomState>;

#[derive(Clone)]
pub struct RealmBrowser {
  writer: Arc<Mutex<RealmWriter>>,
  reader: RealmReader,
}

impl RealmBrowser {
  pub fn new() -> Self {
    let (reader, writer) = evmap::new();
    RealmBrowser {
      writer: Arc::new(Mutex::new(writer)),
      reader,
    }
  }

  pub fn insert(&self, realm: RealmServer) -> Result<()> {
    let mut realms = self.lock()?;

    if realms.contains_key(&realm.id) {
      Err(Context::new("Duplicated realm ID entries"))?;
    }

    realms.insert(realm.id, realm);
    realms.refresh();
    Ok(())
  }

  pub fn update<F: FnOnce(&mut RealmServer)>(&self, id: RealmServerId, mutator: F) -> Result<()> {
    // TODO: Use 'chashmap' instead to avoid cloning?
    let mut realms = self.lock()?;
    let mut realm = realms
      .get_and(&id, |realm| realm[0].clone())
      .ok_or_else(|| Context::new("Non existent realm ID specified"))?;

    mutator(&mut realm);

    realms.update(id, realm);
    realms.refresh();
    Ok(())
  }

  pub fn remove(&self, id: RealmServerId) -> Result<()> {
    let mut realms = self.lock()?;
    realms.empty(id);
    Ok(())
  }

  fn lock(&self) -> Result<MutexGuard<RealmWriter>> {
    Ok(
      self
        .writer
        .lock()
        .map_err(|_| Context::new("Realm writer deadlock"))?,
    )
  }
}
