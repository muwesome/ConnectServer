use failure::{Error, ResultExt};
use mucs::{ConnectConfig, ConnectServer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about = "Mu Online Connect Server")]
pub struct Config {
  #[structopt(flatten)]
  pub connect: ConnectConfig,
}

fn run() -> Result<(), Error> {
  let running = Arc::new(AtomicBool::new(true));
  let runningc = running.clone();

  ctrlc::set_handler(move || {
    println!("Shutting down server...");
    runningc.store(false, Ordering::SeqCst);
  }).context("Error setting interrupt handler")?;

  // Parse any CLI arguments
  let Config { connect } = Config::from_args();
  let server = ConnectServer::spawn(connect).context("Error trying to spawn connect server")?;

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
