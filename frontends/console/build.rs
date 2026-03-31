fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = std::path::Path::new(&manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("Failed to resolve workspace root");

    println!("cargo:rustc-env=WORKSPACE_ROOT={}", workspace_root.display());
    println!("cargo:rerun-if-changed=build.rs");
}
