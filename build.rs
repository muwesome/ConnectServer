extern crate protoc_grpcio;

fn main() {
  let proto_root = "../RPC";
  println!("cargo:rerun-if-changed={}", proto_root);
  protoc_grpcio::compile_grpc_protos(
    &["connectserver.proto"],
    &[proto_root],
    "src/service/rpc/proto",
  ).expect("Failed to compile gRPC definitions!");
}
