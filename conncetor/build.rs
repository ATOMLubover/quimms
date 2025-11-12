fn main() {
    const USER_PROTO_FILE: &str = "../proto/user_service.proto";
    const MESSAGE_PROTO_FILE: &str = "../proto/message_service.proto";
    const CHANNEL_PROTO_FILE: &str = "../proto/channel_service.proto";
    const DISPATCH_PROTO_FILE: &str = "../proto/dispatch_service.proto";

    tonic_prost_build::compile_protos(USER_PROTO_FILE)
        .unwrap_or_else(|err| panic!("Failed to compile protos: {}", err));
    tonic_prost_build::compile_protos(MESSAGE_PROTO_FILE)
        .unwrap_or_else(|err| panic!("Failed to compile protos: {}", err));
    tonic_prost_build::compile_protos(CHANNEL_PROTO_FILE)
        .unwrap_or_else(|err| panic!("Failed to compile protos: {}", err));
    tonic_prost_build::compile_protos(DISPATCH_PROTO_FILE)
        .unwrap_or_else(|err| panic!("Failed to compile protos: {}", err));

    println!("{}", format!("cargo:rerun-if-changed={}", USER_PROTO_FILE));
    println!(
        "{}",
        format!("cargo:rerun-if-changed={}", MESSAGE_PROTO_FILE)
    );
    println!(
        "{}",
        format!("cargo:rerun-if-changed={}", CHANNEL_PROTO_FILE)
    );
    println!(
        "{}",
        format!("cargo:rerun-if-changed={}", DISPATCH_PROTO_FILE)
    );
}
