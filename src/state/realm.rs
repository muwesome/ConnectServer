use chashmap::CHashMap;
use failure::Fail;
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

#[derive(Fail, Debug)]
pub enum RealmBrowserError {
  #[fail(display = "Non-unique realm ID")]
  DuplicateId,

  #[fail(display = "Inexistent realm ID")]
  InexistentId,
}

#[derive(Clone)]
pub struct RealmBrowser {
  realms: Arc<CHashMap<RealmServerId, RealmServer>>,
}

impl RealmBrowser {
  pub fn new() -> Self {
    RealmBrowser {
      realms: Arc::new(CHashMap::new()),
    }
  }

  pub fn add(&self, realm: RealmServer) -> Result<(), RealmBrowserError> {
    if self.realms.contains_key(&realm.id) {
      Err(RealmBrowserError::DuplicateId)?;
    }

    self.realms.insert_new(realm.id, realm);
    Ok(())
  }

  pub fn remove(&self, id: RealmServerId) -> Result<(), RealmBrowserError> {
    self
      .realms
      .remove(&id)
      .ok_or(RealmBrowserError::InexistentId)?;
    Ok(())
  }

  pub fn update<F>(&self, id: RealmServerId, mutator: F) -> Result<(), RealmBrowserError>
  where
    F: FnOnce(&mut RealmServer),
  {
    self
      .realms
      .get_mut(&id)
      .map(|mut realm| mutator(&mut realm))
      .ok_or(RealmBrowserError::InexistentId)
  }

  pub fn for_each<F: FnMut(&RealmServer)>(&self, func: F) {
    let func = RefCell::new(func);
    self.realms.retain(|_, realm| {
      func.borrow_mut().deref_mut()(&realm);
      true
    });
  }

  pub fn get<'a>(
    &'a self,
    id: RealmServerId,
  ) -> Result<impl std::ops::Deref<Target = RealmServer> + 'a, RealmBrowserError> {
    self.realms.get(&id).ok_or(RealmBrowserError::InexistentId)
  }

  pub fn len(&self) -> usize {
    self.realms.len()
  }
}
