use parking_lot::Mutex;
use std::sync::Arc;

pub trait EventArgs<T> {
  fn is_propagation_stopped(&self) -> bool;

  fn is_default_prevented(&self) -> bool;

  fn stop_propagation(&mut self);

  fn prevent_default(&mut self);

  fn data(&self) -> &T;
}

struct EventArgsOwned<T> {
  stop_propagation: bool,
  prevent_default: bool,
  value: T,
}

impl<T> EventArgsOwned<T> {
  fn new(value: T) -> Self {
    EventArgsOwned {
      stop_propagation: false,
      prevent_default: false,
      value,
    }
  }
}

impl<T> EventArgs<T> for EventArgsOwned<T> {
  fn is_propagation_stopped(&self) -> bool {
    self.stop_propagation
  }

  fn is_default_prevented(&self) -> bool {
    self.prevent_default
  }

  fn stop_propagation(&mut self) {
    self.stop_propagation = true;
  }

  fn prevent_default(&mut self) {
    self.prevent_default = true;
  }

  fn data(&self) -> &T {
    &self.value
  }
}

struct EventArgsRef<'a, T: 'static> {
  stop_propagation: bool,
  prevent_default: bool,
  value: &'a T,
}

impl<'a, T> EventArgsRef<'a, T> {
  fn new(value: &'a T) -> Self {
    EventArgsRef {
      stop_propagation: false,
      prevent_default: false,
      value,
    }
  }
}

impl<'a, T> EventArgs<T> for EventArgsRef<'a, T> {
  fn is_propagation_stopped(&self) -> bool {
    self.stop_propagation
  }

  fn is_default_prevented(&self) -> bool {
    self.prevent_default
  }

  fn stop_propagation(&mut self) {
    self.stop_propagation = true;
  }

  fn prevent_default(&mut self) {
    self.prevent_default = true;
  }

  fn data(&self) -> &T {
    self.value
  }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum EventAction {
  //Remove,
  Keep,
}

impl From<()> for EventAction {
  fn from(_unit: ()) -> Self {
    EventAction::Keep
  }
}

pub trait EventListener<T>: Send + 'static {
  fn call(&self, event: &mut EventArgs<T>) -> EventAction;
}

impl<F, T, R> EventListener<T> for F
where
  F: Fn(&mut EventArgs<T>) -> R + Send + 'static,
  R: Into<EventAction>,
{
  fn call(&self, event: &mut EventArgs<T>) -> EventAction {
    self(event).into()
  }
}

pub struct EventHandler<T: 'static> {
  listeners: Arc<Mutex<Vec<Box<dyn EventListener<T>>>>>,
}

impl<T: 'static> Clone for EventHandler<T> {
  fn clone(&self) -> Self {
    EventHandler {
      listeners: self.listeners.clone(),
    }
  }
}

impl<T: 'static> EventHandler<T> {
  pub fn new() -> Self {
    EventHandler {
      listeners: Arc::new(Mutex::new(Vec::new())),
    }
  }

  pub fn dispatch_ref(&self, data: &T) -> bool {
    self.dispatch_impl(EventArgsRef::new(data))
  }

  pub fn dispatch(&self, data: T) -> bool {
    self.dispatch_impl(EventArgsOwned::new(data))
  }

  pub fn subscribe<E: EventListener<T>>(&self, listener: E) {
    self.listeners.lock().push(Box::new(listener));
  }

  pub fn subscribe_fn<F, R>(&self, closure: F)
  where
    F: Fn(&mut EventArgs<T>) -> R + Send + 'static,
    R: Into<EventAction>,
  {
    self.subscribe(closure);
  }

  fn dispatch_impl(&self, mut event: impl EventArgs<T>) -> bool {
    self.listeners.lock().retain(|listener| {
      if !event.is_propagation_stopped() {
        listener.call(&mut event) == EventAction::Keep
      } else {
        true
      }
    });
    !event.is_default_prevented()
  }
}
