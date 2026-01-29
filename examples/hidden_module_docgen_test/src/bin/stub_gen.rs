use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().filter_or("RUST_LOG", "info")).init();
    let stub = hidden_module_docgen_test::stub_info()?;
    stub.generate()?;
    Ok(())
}
