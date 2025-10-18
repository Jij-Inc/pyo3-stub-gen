use pyo3_stub_gen_derive::gen_function_from_python;

fn main() {
    pyo3_stub_gen::inventory::submit! {
        gen_function_from_python! {
            r#"
            def process(x: pyo3_stub_gen.RustType["Invalid:::"]) -> int:
                """Function using invalid Rust type syntax"""
            "#
        }
    }
}
