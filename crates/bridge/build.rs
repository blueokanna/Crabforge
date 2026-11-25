fn main() {
    let manifest = "../../gui/pubspec.yaml";
    println!("cargo:rerun-if-changed={manifest}");
    let yaml = std::fs::read_to_string(manifest).expect("read GUI package version");
    let version = yaml
        .lines()
        .find_map(|line| line.strip_prefix("version:"))
        .expect("missing Flutter version")
        .trim()
        .split('+')
        .next()
        .unwrap();
    assert_eq!(
        version,
        env!("CARGO_PKG_VERSION"),
        "Flutter and Rust versions must match"
    );
}
