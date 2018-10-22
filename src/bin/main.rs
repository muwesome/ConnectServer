use failure::{Error, ResultExt};
use mucs::{ConnectConfig, ConnectServer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};
use structopt::StructOpt;

fn run() -> Result<(), Error> {
  let running = Arc::new(AtomicBool::new(true));
  let runningc = running.clone();

  ctrlc::set_handler(move || {
    println!("Shutting down server...");
    runningc.store(false, Ordering::SeqCst);
  }).context("Error setting interrupt handler")?;

  // Parse any CLI arguments
  let config = ConnectConfig::from_args();
  let server = ConnectServer::spawn(config).context("Error trying to spawn connect server")?;

  while server.is_active() && running.load(Ordering::SeqCst) {
    thread::sleep(Duration::from_millis(100));
  }

  server.stop().context("Error during shutdown")?;
  Ok(())
}

fn main() {
  if let Err(error) = run() {
    eprintln!("Runtime exit — {}", error);
    for cause in error.iter_causes() {
      eprintln!(" — {}", cause);
    }

    if std::env::var_os("RUST_BACKTRACE").is_some() {
      eprintln!("{}", error.backtrace());
    }

    std::process::exit(1);
  }
}
