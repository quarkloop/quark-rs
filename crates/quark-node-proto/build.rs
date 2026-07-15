use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let proto_dir = manifest_dir.join("proto");
    println!("cargo:rerun-if-changed={}", proto_dir.display());
    let mut protos: Vec<PathBuf> = Vec::new();
    collect_protos(&proto_dir, &mut protos)?;
    if protos.is_empty() { panic!("no .proto files found in {}", proto_dir.display()); }
    let mut includes = vec![proto_dir.clone()];
    if let Ok(dir) = std::env::var("PROTOC_INCLUDE") {
        let p = PathBuf::from(dir);
        if p.exists() { includes.push(p); }
    }
    tonic_build::configure().build_server(true).build_client(true).compile_protos(&protos, &includes)?;
    Ok(())
}
fn collect_protos(dir: &std::path::Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() { collect_protos(&path, out)?; }
        else if path.extension().and_then(|e| e.to_str()) == Some("proto") { out.push(path); }
    }
    Ok(())
}
