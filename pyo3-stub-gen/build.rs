use semver::Version;

fn main() {
    // Get cargo metadata
    let metadata = cargo_metadata::MetadataCommand::new()
        .features(cargo_metadata::CargoOpt::AllFeatures)
        .exec()
        .expect("Failed to run cargo metadata");

    // Find pyo3 package
    let pyo3_pkg = metadata
        .packages
        .iter()
        .find(|p| p.name == "pyo3")
        .expect("`pyo3` not found in dependencies");

    // Check pyo3 version for pyo3_0_25 flag
    let pyo3_ver = Version::parse(&pyo3_pkg.version.to_string()).expect("Invalid semver for pyo3");
    if pyo3_ver >= Version::new(0, 25, 0) {
        println!("cargo::rustc-check-cfg=cfg(pyo3_0_25)");
    }

    // Add check-cfg for Py_LIMITED_API
    println!("cargo::rustc-check-cfg=cfg(Py_LIMITED_API)");

    // Find pyo3 in resolve nodes to check activated features
    let pyo3_node = metadata
        .resolve
        .as_ref()
        .expect("No resolve section in metadata")
        .nodes
        .iter()
        .find(|node| node.id == pyo3_pkg.id)
        .expect("pyo3 not found in resolve nodes");

    // Check if abi3 feature is enabled
    let has_abi3 = pyo3_node.features.iter().any(|f| f.starts_with("abi3"));
    if has_abi3 {
        println!("cargo:rustc-cfg=Py_LIMITED_API");
    }
}
