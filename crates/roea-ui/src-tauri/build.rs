fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile protobuf for gRPC client
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&["../../../proto/roea.proto"], &["../../../proto"])?;

    tauri_build::build();
    Ok(())
}
