use crate::common::interpret_using_probe;
mod common;

#[test]
fn class_declaration_test() {
    let src = r#"
        class Brioche {}
        print Brioche;
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["<class Brioche>", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}
