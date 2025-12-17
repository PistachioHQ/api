use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);

    let api_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to find api root directory");

    let proto_dir = api_root.join("proto");

    // Vendored buf dependencies
    let deps_dir = api_root.join("gen/proto-deps");

    // Build client and server for the Pistachio API
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .bytes(".")
        .compile_protos(
            &[
                proto_dir.join("pistachio/v1/api.proto"),
                proto_dir.join("pistachio/admin/v1/admin_api.proto"),
            ],
            &[proto_dir.clone(), deps_dir],
        )?;

    Ok(())
}
