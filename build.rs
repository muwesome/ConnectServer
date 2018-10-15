extern crate protoc_grpcio;

fn main() {
  let proto_root = "../RPC";
  println!("cargo:rerun-if-changed={}", proto_root);
  protoc_grpcio::compile_grpc_protos(&["connectservice.proto"], &[proto_root], "src/rpc/proto")
    .expect("Failed to compile gRPC definitions!");
}
