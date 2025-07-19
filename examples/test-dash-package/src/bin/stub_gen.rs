use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = test_dash_package::stub_info()?;
    stub.generate()?;
    Ok(())
}