#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse.rs");

    // t.compile_fail("tests/02-fail-struct.rs");
    // t.compile_fail("tests/03-fail-missing-value-attr.rs");
    // t.compile_fail("tests/04-fail-non-unit-variant.rs");
    // t.compile_fail("tests/.rs");
    // t.compile_fail("tests/.rs");
}

// #[test]
// fn expand_tests() {
//     macrotest::expand("tests/expand/*.rs");
// }
