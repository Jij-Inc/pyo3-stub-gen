use pyo3_stub_gen::type_info::PYFUNCTIONS;

fn main() {
    pyo3_stub_gen_testing::dbg();

    dbg!(PYFUNCTIONS.len());
    for info in PYFUNCTIONS {
        dbg!(info);
    }
}
