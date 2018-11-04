use failure::{Error, ResultExt};
use log::{error, info, LevelFilter};
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
    info!("Shutting down server...");
    runningc.store(false, Ordering::SeqCst);
  }).context("Error setting interrupt handler")?;

  // Parse any CLI arguments
  let Config { connect } = Config::from_args();
  let server = ConnectServer::spawn(connect).context("Error trying to spawn connect server")?;

  while server.is_active() && running.load(Ordering::SeqCst) {
    thread::sleep(Duration::from_millis(100));
  }

  server.stop().context("Error during execution")?;
  Ok(())
}

fn main() {
  match pretty_env_logger::formatted_builder() {
    Ok(mut builder) => builder.filter_module("mu", LevelFilter::Trace).init(),
    Err(error) => {
      eprintln!("Failed to configure logger; {}", error);
      std::process::exit(1);
    }
  }

  if let Err(error) = run() {
    error!("Runtime exit — {}", error);
    for cause in error.iter_causes() {
      error!("— {}", cause);
    }

    if std::env::var_os("RUST_BACKTRACE").is_some() {
      error!("{}", error.backtrace());
    }

    std::process::exit(1);
  }
}
