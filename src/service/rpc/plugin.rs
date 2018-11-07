use crate::state::RealmServer;
use crate::util::EventArgs;
use log::{error, info};

/// A trait describing a realm event plugin.
pub trait RealmEventPlugin: Send + Sync + 'static {
  fn on_register(&self, _event: &mut EventArgs<RealmServer>) {}
  fn on_deregister(&self, _event: &mut EventArgs<RealmServer>) {}
  fn on_update(&self, _event: &mut EventArgs<RealmServer>) {}
  fn on_error(&self, _event: &mut EventArgs<grpcio::Error>) {}
}

/// Plugin logging any realm events.
pub struct RealmEventLogger;

impl RealmEventPlugin for RealmEventLogger {
  fn on_register(&self, event: &mut EventArgs<RealmServer>) {
    info!("Realm registered: {}", event.data());
  }

  fn on_deregister(&self, event: &mut EventArgs<RealmServer>) {
    info!("Realm deregistered: {}", event.data());
  }

  fn on_update(&self, event: &mut EventArgs<RealmServer>) {
    info!("Realm updated: {}", event.data());
  }

  fn on_error(&self, event: &mut EventArgs<grpcio::Error>) {
    error!("Realm RPC — {}", event.data());
  }
}
