use semver::Version;

fn main() {
    // Add all pyo3's #[cfg] flags to the current compilation.
    pyo3_build_config::use_pyo3_cfgs();

    // Add `pyo3_0_25` flag if pyo3's version >= 0.25.0
    let metadata = cargo_metadata::MetadataCommand::new()
        .features(cargo_metadata::CargoOpt::AllFeatures)
        .exec()
        .expect("Failed to run cargo metadata");
    let pyo3_pkg = metadata
        .packages
        .iter()
        .find(|p| p.name == "pyo3")
        .expect("`pyo3` not found in dependencies");
    let pyo3_ver = Version::parse(&pyo3_pkg.version.to_string())
        .expect("Invalid semver for pyo3");
    if pyo3_ver >= Version::new(0, 25, 0) {
        println!("cargo::rustc-check-cfg=cfg(pyo3_0_25)");
    }
}
