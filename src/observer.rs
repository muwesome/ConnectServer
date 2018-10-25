use crate::state::{Client, ClientListener, RealmListener, RealmServer};
use log::info;

pub struct EventObserver;

impl ClientListener for EventObserver {
  fn on_connect(&self, client: &Client) {
    info!("Client connected: {}", &client);
  }

  fn on_disconnect(&self, client: &Client) {
    info!("Client disconnected: {}", client);
  }
}

impl RealmListener for EventObserver {
  fn on_register(&self, realm: &RealmServer) {
    info!("Realm registered: {}", &realm);
  }

  fn on_deregister(&self, realm: &RealmServer) {
    info!("Realm deregistered: {}", &realm);
  }

  fn on_update(&self, realm: &RealmServer) {
    info!("Realm updated: {}", &realm);
  }
}
