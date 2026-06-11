use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = PathBuf::from("../../../proto");

    let protos: Vec<PathBuf> = [
        "craftpath/v1/common.proto",
        "craftpath/v1/item.proto",
        "craftpath/v1/currency.proto",
        "craftpath/v1/presets.proto",
        "craftpath/v1/job.proto",
    ]
    .iter()
    .map(|p| proto_root.join(p))
    .collect();

    for proto in &protos {
        println!("cargo:rerun-if-changed={}", proto.display());
    }

    let descriptor_path = PathBuf::from(env::var("OUT_DIR")?).join("craftpath_descriptor.bin");

    let mut config = prost_build::Config::new();
    config.file_descriptor_set_path(&descriptor_path);
    config.compile_protos(&protos, &[proto_root])?;

    let descriptor_set = std::fs::read(&descriptor_path)?;
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)?
        .build(&[".craftpath"])?;

    Ok(())
}
