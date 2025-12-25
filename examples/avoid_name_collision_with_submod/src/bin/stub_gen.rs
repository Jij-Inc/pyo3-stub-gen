use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    env_logger::init();
    let stub = avoid_name_collision_with_submod::stub_info()?;
    stub.generate()?;
    Ok(())
}
