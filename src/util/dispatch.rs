use parking_lot::{Mutex, MutexGuard};
use std::sync::{Arc, Weak};

struct DispatchIter<'a, T, F, R>
where
  T: 'a + ?Sized,
  F: Fn(&T) -> R,
{
  guard: MutexGuard<'a, Vec<Weak<T>>>,
  dispatch: F,
  index: usize,
}

impl<'a, T, F, R> Iterator for DispatchIter<'a, T, F, R>
where
  T: 'a + ?Sized,
  F: Fn(&T) -> R,
{
  type Item = R;

  fn next(&mut self) -> Option<Self::Item> {
    while let Some(subscriber) = self.guard.get(self.index) {
      if let Some(subscriber) = subscriber.upgrade() {
        self.index += 1;
        return Some((self.dispatch)(&*subscriber));
      } else {
        self.guard.remove(self.index);
      }
    }
    None
  }
}

#[derive(Clone)]
pub struct Dispatcher<T: ?Sized> {
  subscribers: Arc<Mutex<Vec<Weak<T>>>>,
}

impl<T: ?Sized> Dispatcher<T> {
  /// Creates an empty dispatcher.
  pub fn new() -> Self {
    Dispatcher {
      subscribers: Arc::new(Mutex::new(Vec::new())),
    }
  }

  /// Adds a new subscriber.
  pub fn subscribe(&self, subscriber: &Arc<T>) {
    self.subscribers.lock().push(Arc::downgrade(subscriber));
  }

  /// Dispatches an operation to all subscribers.
  pub fn dispatch<F: Fn(&T)>(&self, f: F) {
    self.dispatch_iter(f).for_each(drop);
  }

  // Returns a dispatch iterator over all subscribers.
  pub fn dispatch_iter<'a, F, R>(&'a self, f: F) -> impl Iterator<Item = R> + 'a
  where
    F: Fn(&T) -> R + 'a,
    R: 'a,
  {
    DispatchIter {
      guard: self.subscribers.lock(),
      dispatch: f,
      index: 0,
    }
  }
}
