use crate::Result;
use failure::Context;
use futures::{future::Shared, sync::oneshot, Async, Future, Poll};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

#[derive(Clone)]
pub struct CloseSignal {
  receiver: Shared<oneshot::Receiver<()>>,
}

impl Future for CloseSignal {
  type Item = ();
  type Error = ();

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    match self.receiver.poll() {
      Ok(Async::Ready(_)) => Ok(Async::Ready(())),
      Ok(Async::NotReady) => Ok(Async::NotReady),
      Err(_) => Err(()),
    }
  }
}

struct ThreadControllerInner {
  is_alive: Arc<()>,
  sender: oneshot::Sender<()>,
  thread: JoinHandle<Result<()>>,
}

/// Controller for managing a thread.
pub struct ThreadController(Option<ThreadControllerInner>);

impl ThreadController {
  /// Spawns a thread and returns its controller.
  pub fn spawn<F>(closure: F) -> Self
  where
    F: FnOnce(CloseSignal) -> Result<()> + Send + 'static,
  {
    let (sender, receiver) = oneshot::channel();
    let is_alive = Arc::new(());
    let thread = thread::spawn(closet!([is_alive] move || {
      let _keep_alive = is_alive;
      let close_rx = CloseSignal { receiver: receiver.shared() };
      closure(close_rx)
    }));

    ThreadController(Some(ThreadControllerInner {
      is_alive,
      sender,
      thread,
    }))
  }

  /// Returns whether the thread is still alive or not.
  pub fn is_alive(&self) -> bool {
    self
      .0
      .as_ref()
      .map_or(false, |inner| Arc::strong_count(&inner.is_alive) > 1)
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
      let close_result = inner
        .sender
        .send(())
        .map_err(|_| Context::new("Thread receiver closed prematurely").into());
      Self::join_thread(inner.thread).and(close_result)?;
    }
    Ok(())
  }

  fn join_thread(thread: JoinHandle<Result<()>>) -> Result<()> {
    thread
      .join()
      .map_err(|_| Context::new("Thread managed by controller panicked").into())
      .and_then(|result| result)
  }
}

impl Drop for ThreadController {
  fn drop(&mut self) {
    if let Err(error) = self.stop_and_join_thread() {
      println!("Thread error during destructor: {}", error);
    }
  }
}
