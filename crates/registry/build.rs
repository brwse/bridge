fn main() {
    tonic_build::compile_protos("./protobuf/registry/v1/service.proto").unwrap();
}
