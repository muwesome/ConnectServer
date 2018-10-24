use std::sync::{Arc, Weak};

pub trait Listener {}

pub struct Dispatcher<T: Listener + ?Sized> {
  listeners: Vec<Weak<T>>,
}

impl<T: Listener + ?Sized> Dispatcher<T> {
  /// Creates an empty event dispatcher.
  pub fn new() -> Self {
    Dispatcher {
      listeners: Vec::new(),
    }
  }

  /// Adds a new listener.
  pub fn add_listener(&mut self, listener: &Arc<T>) {
    self.listeners.push(Arc::downgrade(listener));
  }

  // Dispatches an event to all listeners.
  pub fn dispatch<'a, F, R>(&'a mut self, f: F) -> impl Iterator<Item = R> + 'a
  where
    F: Fn(&T) -> R + 'a,
  {
    self
      .listeners
      .iter()
      .filter_map(|listener| listener.upgrade())
      .map(move |listener| f(&*listener))
  }
}
