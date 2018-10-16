use crate::state::{Client, ClientEvent, RealmEvent, RealmServer};
use crate::util::Listener;

pub struct EventObserver;

impl Listener<ClientEvent> for EventObserver {
  fn on_event(&mut self, event: &ClientEvent, client: &Client) {
    match event {
      ClientEvent::Connect => println!("Client connected: {}", &client),
      ClientEvent::Disconnect => println!("Client disconnected: {}", client),
    }
  }
}

impl Listener<RealmEvent> for EventObserver {
  fn on_event(&mut self, event: &RealmEvent, realm: &RealmServer) {
    match event {
      RealmEvent::Register => println!("Realm registered: {}", &realm),
      RealmEvent::Deregister => println!("Realm deregistered: {}", &realm),
      RealmEvent::Update => println!("Realm updated: {}", &realm),
    }
  }
}
