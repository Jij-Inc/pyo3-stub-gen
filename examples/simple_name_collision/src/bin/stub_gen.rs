use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    env_logger::init();
    let stub = simple_name_collision::stub_info()?;
    stub.generate()?;
    Ok(())
}
