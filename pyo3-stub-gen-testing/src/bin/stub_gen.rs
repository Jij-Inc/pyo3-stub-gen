use pyo3_stub_gen::*;

fn main() {
    let modules = generate::gather().unwrap();
    dbg!(modules);
}
