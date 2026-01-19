use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    env_logger::init();
    let stub = same_name_submod::stub_info()?;
    stub.generate()?;
    Ok(())
}
