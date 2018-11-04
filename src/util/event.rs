use parking_lot::Mutex;
use std::{ops::Deref, sync::Arc};

pub struct EventArgs<T> {
  stop_propagation: bool,
  prevent_default: bool,
  value: T,
}

impl<T> EventArgs<T> {
  fn new(value: T) -> Self {
    EventArgs {
      stop_propagation: false,
      prevent_default: false,
      value,
    }
  }

  pub fn is_propagation_stopped(&self) -> bool {
    self.stop_propagation
  }

  pub fn is_default_prevented(&self) -> bool {
    self.prevent_default
  }

  /*pub fn stop_propagation(&mut self) {
    self.stop_propagation = true;
  }*/

  pub fn prevent_default(&mut self) {
    self.prevent_default = true;
  }
}

impl<T> Deref for EventArgs<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.value
  }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum EventAction {
  //Remove,
  Keep,
}

impl From<()> for EventAction {
  fn from(_: ()) -> Self {
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

  pub fn dispatch(&self, data: T) -> bool {
    let mut event = EventArgs::new(data);
    self.listeners.lock().retain(|listener| {
      if !event.is_propagation_stopped() {
        listener.call(&mut event) == EventAction::Keep
      } else {
        true
      }
    });
    !event.is_default_prevented()
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
}
