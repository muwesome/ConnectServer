use crate::state::{Client, ClientEvent, RealmEvent, RealmServer};
use crate::util::Listener;
use log::info;

pub struct EventObserver;

impl Listener<ClientEvent> for EventObserver {
  fn on_event(&mut self, event: &ClientEvent, client: &Client) {
    match event {
      ClientEvent::Connect => info!("Client connected: {}", &client),
      ClientEvent::Disconnect => info!("Client disconnected: {}", client),
    }
  }
}

impl Listener<RealmEvent> for EventObserver {
  fn on_event(&mut self, event: &RealmEvent, realm: &RealmServer) {
    match event {
      RealmEvent::Register => info!("Realm registered: {}", &realm),
      RealmEvent::Deregister => info!("Realm deregistered: {}", &realm),
      RealmEvent::Update => info!("Realm updated: {}", &realm),
    }
  }
}
