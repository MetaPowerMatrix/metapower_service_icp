fn main() {
    let proto_file = "../protos/agent.proto";

    tonic_build::configure()
        .build_server(true)
        .out_dir("./src/service")
        .compile(&[proto_file], &["../protos"])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));


    let proto_file2 = "../protos/matrix.proto";

    tonic_build::configure()
        .build_server(true)
        .out_dir("./src/service")
        .compile(&[proto_file2], &["../protos"])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));


    let proto_file3 = "../protos/battery.proto";

    tonic_build::configure()
        .build_server(true)
        .out_dir("./src/service")
        .compile(&[proto_file3], &["../protos"])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));

    let proto_file4 = "../protos/llm.proto";

    tonic_build::configure()
        .build_server(true)
        .out_dir("./src/service")
        .compile(&[proto_file4], &["../protos"])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));

    println!("cargo:rerun-if-changed={}", proto_file);
    println!("cargo:rerun-if-changed={}", proto_file2);
    println!("cargo:rerun-if-changed={}", proto_file3);
}
