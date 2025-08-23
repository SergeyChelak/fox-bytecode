use crate::common::interpret_using_probe;
mod common;

#[test]
fn precedence_test() {
    let src = r"
        print 2 + 2 * 2;
        print (2 + 2) * 2;
        print 2 * 2 / 2;
        print 2 + 2 / 2;
    ";
    let probe = interpret_using_probe(src);
    let output = &["6", "8", "2", "3"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn unary_precedence() {
    let src = r"
        print 2 + -2;
        print -2 * -2;
        print -(1 + 2);
    ";
    let probe = interpret_using_probe(src);
    let output = &["0", "4", "-3"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}
