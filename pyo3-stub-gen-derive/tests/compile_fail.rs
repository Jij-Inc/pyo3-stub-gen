#[test]
fn failing_cases() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/failing_cases/*.rs");
}
