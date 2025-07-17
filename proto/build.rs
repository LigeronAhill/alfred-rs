fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    tonic_build::configure()
        .file_descriptor_set_path(out.join("proto_descriptor.bin"))
        .compile_protos(&["users/v1/users.proto"], &["users"])?;
    Ok(())
}
