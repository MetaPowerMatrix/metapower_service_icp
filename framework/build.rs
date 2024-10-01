fn main() {
    let proto_file4 = "../protos/llm.proto";

    tonic_build::configure()
        .build_server(true)
        .out_dir("./src/service")
        .compile(&[proto_file4], &["../protos"])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));

    println!("cargo:rerun-if-changed={}", proto_file4);
}
