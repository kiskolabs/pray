fn main() {
    let path = std::path::Path::new("examples/simple-project/Prayfile");
    let text = pray_core::manifest::read_manifest_text(path).unwrap();
    let manifest = pray_core::manifest::parse_manifest(&text).unwrap();
    println!("{}", manifest.manifest_hash().unwrap());
    println!("{}", serde_json::to_string_pretty(&manifest.canonicalized()).unwrap());
}
