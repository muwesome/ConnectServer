use std::sync::{Arc, Mutex, Weak};

pub trait Event: Send + Sync {
  type Context;
}

pub trait Listener<T: Event> {
  fn on_event(&mut self, event: &T, context: &T::Context);
}

pub struct Dispatcher<T: Send + Sync> {
  listeners: Vec<Weak<Mutex<Listener<T> + Send + Sync + 'static>>>,
}

impl<T: Event> Dispatcher<T> {
  /// Creates an empty event dispatcher.
  pub fn new() -> Self {
    Dispatcher {
      listeners: Vec::new(),
    }
  }

  /// Adds a new listener.
  pub fn add_listener<L>(&mut self, listener: &Arc<Mutex<L>>)
  where
    L: Listener<T> + Send + Sync + 'static,
  {
    self.listeners.push(Arc::downgrade(
      &(Arc::clone(listener) as Arc<Mutex<Listener<T> + Send + Sync + 'static>>),
    ));
  }

  // Dispatches an event to all listeners.
  pub fn dispatch(&mut self, event: &T, context: &T::Context) {
    self.listeners.retain(|listener| {
      if let Some(listener) = listener.upgrade() {
        let mut listener = listener.lock().expect("TODO");
        listener.on_event(event, &context);
        true
      } else {
        false
      }
    });
  }
}
