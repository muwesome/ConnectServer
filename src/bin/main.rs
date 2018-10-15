extern crate muwesome_connect_server as mucs;

fn main() {
  let server = mucs::ConnectServer::spawn().expect("Failed to spawn");
  server.wait().expect("Failed to wait");
  println!("WE ENDZ NOWZ BOYZ ;)");
}
