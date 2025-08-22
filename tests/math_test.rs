use crate::common::interpret_by_probe;
mod common;

#[test]
fn math_test() {
    let src = r"
        print 2 + 2 * 2;
    ";
    let probe = interpret_by_probe(src);
    let output = &["6".to_string(), "12".to_string()];
    assert!(probe.borrow().vm_error().is_none());
    probe.borrow().assert_output_match(output);
}
