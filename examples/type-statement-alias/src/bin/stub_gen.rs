use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let stub = type_statement_alias::stub_info()?;
    stub.generate()?;
    Ok(())
}
