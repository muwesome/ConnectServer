use mucs::{ConnectConfig, ConnectServer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};
use structopt::StructOpt;

fn main() {
  // Parse any CLI arguments
  let config = ConnectConfig::from_args();
  let server = ConnectServer::spawn(config).expect("Failed to spawn");

  let running = Arc::new(AtomicBool::new(true));
  let runningc = running.clone();

  ctrlc::set_handler(move || {
    println!("Shutting down server...");
    runningc.store(false, Ordering::SeqCst);
  }).expect("Error setting Ctrl-C handler");

  while server.is_active() && running.load(Ordering::SeqCst) {
    thread::sleep(Duration::from_millis(100));
  }

  server.stop().expect("Failed to stop server");
}
