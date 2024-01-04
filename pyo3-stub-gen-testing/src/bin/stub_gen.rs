use pyo3_stub_gen::*;

fn main() {
    dbg!(type_info::PYFUNCTIONS.len());
    for info in type_info::PYFUNCTIONS {
        dbg!(info);
    }
}
