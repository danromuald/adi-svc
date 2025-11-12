fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure tonic-build to generate Rust code from protobuf
    // The generated files will be placed in OUT_DIR by default
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &["proto/document_intelligence.proto"],
            &["proto"],
        )?;
    
    println!("cargo:rerun-if-changed=proto/document_intelligence.proto");
    
    Ok(())
}

