use crate::common::interpret_using_probe;
mod common;

#[test]
fn func_declaration_test() {
    let src = r#"
        fun areWeHavingItYet() {
          print "Yes we are!";
        }

        print areWeHavingItYet;
    "#;
    let probe = interpret_using_probe(src);
    let output = &["<fn areWeHavingItYet>"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}
