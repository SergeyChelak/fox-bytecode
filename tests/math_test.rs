use crate::common::interpret_by_probe;
mod common;

#[test]
fn precedence_test() {
    let src = r"
        print 2 + 2 * 2;
        print (2 + 2) * 2;
        print 2 * 2 / 2;
        print 2 + 2 / 2;
    ";
    let probe = interpret_by_probe(src);
    let output = &["6", "8", "2", "3"];
    assert!(!probe.borrow().has_errors(), "Error found");
    probe.borrow().assert_output_match(output);
}

#[test]
fn unary_precedence() {
    let src = r"
        print 2 + -2;
        print -2 * -2;
        print -(1 + 2);
    ";
    let probe = interpret_by_probe(src);
    let output = &["0", "4", "-3"];
    assert!(probe.borrow().vm_error().is_none());
    probe.borrow().assert_output_match(output);
}
