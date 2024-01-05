fn main() {
    let stub = pyo3_stub_gen_testing::stub_info();
    stub.generate_single_stub_file(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
}
