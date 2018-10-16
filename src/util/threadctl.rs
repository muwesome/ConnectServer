use crate::Result;
use failure::Context;
use futures::sync::oneshot;
use std::thread;

pub struct ThreadController(Option<(oneshot::Sender<()>, thread::JoinHandle<Result<()>>)>);

impl ThreadController {
  pub fn new(close_tx: oneshot::Sender<()>, thread: thread::JoinHandle<Result<()>>) -> Self {
    ThreadController(Some((close_tx, thread)))
  }

  pub fn wait(mut self) -> Result<()> {
    if let Some((_, thread)) = self.0.take() {
      Self::join_thread(thread)?;
    }
    Ok(())
  }

  pub fn stop(mut self) -> Result<()> {
    self.stop_impl()
  }

  fn stop_impl(&mut self) -> Result<()> {
    if let Some((close_tx, thread)) = self.0.take() {
      close_tx
        .send(())
        .map_err(|_| Context::new("Thread receiver closed prematurely"))?;
      Self::join_thread(thread)?;
    }
    Ok(())
  }

  fn join_thread(thread: thread::JoinHandle<Result<()>>) -> Result<()> {
    thread
      .join()
      // TODO: Log the 'Debug' result
      .map_err(|_| Context::new("Thread managed by controller panicked").into())
      .and_then(|result| result)
  }
}

impl Drop for ThreadController {
  fn drop(&mut self) {
    self.stop_impl().expect("TODO:");
  }
}
