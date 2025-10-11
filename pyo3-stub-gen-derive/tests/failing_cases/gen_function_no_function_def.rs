use pyo3_stub_gen_derive::gen_function_from_python;

fn main() {
    pyo3_stub_gen::inventory::submit! {
        gen_function_from_python! {
            r#"
import typing

# Only imports, no function definition
            "#
        }
    }
}
