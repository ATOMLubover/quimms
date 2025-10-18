fn main() {
    const PROTO_FILE: &str = "../proto/dispatched.proto";

    

    tonic_prost_build::compile_protos(PROTO_FILE)
        .unwrap_or_else(|err| panic!("Failed to compile protos: {}", err));

    println!("{}", format!("cargo:rerun-if-changed={}", PROTO_FILE));
}
