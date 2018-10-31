use crossbeam::stack::TreiberStack;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct IndexPool {
  max_value: usize,
  current_value: AtomicUsize,
  previous_values: TreiberStack<usize>,
}

impl IndexPool {
  pub fn with_capacity(capacity: usize) -> Self {
    IndexPool {
      max_value: capacity,
      current_value: AtomicUsize::new(0),
      previous_values: TreiberStack::new(),
    }
  }

  pub fn new_id(&self) -> Option<usize> {
    if let Some(id) = self.previous_values.try_pop() {
      return Some(id);
    }

    let id = self.current_value.fetch_add(1, Ordering::SeqCst);
    if id == self.max_value {
      self.current_value.fetch_sub(1, Ordering::SeqCst);
      None
    } else {
      Some(id)
    }
  }

  pub fn return_id(&self, id: usize) {
    self.previous_values.push(id);
  }
}
