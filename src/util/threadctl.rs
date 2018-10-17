use crate::Result;
use failure::Context;
use futures::sync::oneshot;
use std::thread::{self, JoinHandle};

struct ThreadControllerInner {
  close_tx: oneshot::Sender<()>,
  thread: JoinHandle<Result<()>>,
}

/// Controller for managing a thread.
pub struct ThreadController(Option<ThreadControllerInner>);

impl ThreadController {
  /// Spawns a thread and returns its controller.
  pub fn spawn<F>(closure: F) -> Self
  where
    F: FnOnce(oneshot::Receiver<()>) -> Result<()> + Send + 'static,
  {
    let (close_tx, close_rx) = oneshot::channel();
    let thread = thread::spawn(move || closure(close_rx));
    ThreadController(Some(ThreadControllerInner { close_tx, thread }))
  }

  /// Returns whether the thread is still alive or not.
  pub fn is_alive(&self) -> bool {
    self
      .0
      .as_ref()
      .map_or(false, |inner| !inner.close_tx.is_canceled())
  }

  /// Waits for the thread to finish.
  pub fn wait(mut self) -> Result<()> {
    if let Some(inner) = self.0.take() {
      Self::join_thread(inner.thread)?;
    }
    Ok(())
  }

  /// Sends a stop signal and waits for the thread to exit.
  pub fn stop(mut self) -> Result<()> {
    self.stop_and_join_thread()
  }

  fn stop_and_join_thread(&mut self) -> Result<()> {
    if let Some(inner) = self.0.take() {
      inner
        .close_tx
        .send(())
        .map_err(|_| Context::new("Thread receiver closed prematurely"))?;
      Self::join_thread(inner.thread)?;
    }
    Ok(())
  }

  fn join_thread(thread: JoinHandle<Result<()>>) -> Result<()> {
    thread
      .join()
      // TODO: Log the 'Debug' result
      .map_err(|_| Context::new("Thread managed by controller panicked").into())
      .and_then(|result| result)
  }
}

impl Drop for ThreadController {
  fn drop(&mut self) {
    self.stop_and_join_thread().expect("TODO:");
  }
}
