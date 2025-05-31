use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().filter_or("RUST_LOG", "info")).init();
    let stub = mixed_sub_import_type::stub_info()?;
    stub.generate()?;
    Ok(())
}
