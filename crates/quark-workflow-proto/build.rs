/// Proto codegen for `workflow-api`.
///
/// Compiles the vendored `.proto` files under `protos/` into Rust types via
/// `prost-build` (messages) and `tonic-build` (gRPC service stubs).
///
/// Proto source layout (vendored, self-contained):
///   protos/                         ← include root (resolves all import paths)
///   protos/temporal/api/            → public SDK API protos (`temporal/api/...`)
///   protos/temporal/server/api/     → server-internal protos (`temporal/server/api/...`)
///   protos/google/                  → well-known google types
///   protos/google/api/              → google api annotations
///   protos/nexusannotations/       → nexus annotations
use std::io::Result;

fn main() -> Result<()> {
    let proto_root = "protos";

    // Collect all .proto files from the unified tree.
    let mut proto_files: Vec<String> = Vec::new();
    collect_protos(proto_root, &mut proto_files)?;

    // Single include root — matches all import path prefixes used in .proto files.
    let include_dirs = [proto_root];

    // Configure prost + tonic codegen.
    // Note: prost-build with `prost_types=true` (default) already maps all
    // google.protobuf well-known types to prost_types equivalents. We only
    // add mappings for types NOT covered by the defaults.
    let mut prost_config = prost_build::Config::new();
    prost_config
        // Map `temporal.api.X.v1` → `api::X::v1` etc.
        .bytes(["."])
        // FieldMask, Struct, Value, ListValue are NOT in the default extern paths.
        .extern_path(".google.protobuf.FieldMask", "::prost_types::FieldMask")
        .extern_path(".google.protobuf.Struct", "::prost_types::Struct")
        .extern_path(".google.protobuf.Value", "::prost_types::Value")
        .extern_path(".google.protobuf.ListValue", "::prost_types::ListValue");

    // Compile all protos. tonic-build handles files containing `service` defs;
    // pure-message files go through prost only (tonic ignores them gracefully).
    tonic_build::configure()
        .compile_protos_with_config(prost_config, &proto_files, &include_dirs)
        .expect("proto codegen failed");

    Ok(())
}

/// Recursively collect all `.proto` files under `dir`.
fn collect_protos(dir: &str, out: &mut Vec<String>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_protos(path.to_str().unwrap(), out)?;
        } else if path.extension().is_some_and(|ext| ext == "proto") {
            out.push(path.to_string_lossy().into_owned());
        }
    }
    Ok(())
}
