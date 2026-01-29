use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = underscore_items::stub_info()?;
    stub.generate()?;
    Ok(())
}
