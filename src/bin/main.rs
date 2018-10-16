use mucs::ConnectServer;

fn main() {
  let server = ConnectServer::spawn().expect("Failed to spawn");
  server.wait().expect("Failed to wait");
  println!("WE ENDZ NOWZ BOYZ ;)");
}
