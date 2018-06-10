extern crate tower_grpc_build;

fn main () {

  // build a simple service
  tower_grpc_build::Config::new()
    .enable_server(true)
    .build(&["proto/simple_service.proto"], &["proto"])
    .unwrap_or_else(|e| panic!("protobuf compilation failed {}", e));

}
