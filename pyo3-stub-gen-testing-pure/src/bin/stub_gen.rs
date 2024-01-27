use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = pyo3_stub_gen_testing_pure::stub_info()?;
    stub.generate_single_stub_file(env!("CARGO_MANIFEST_DIR"))?;
    Ok(())
}
