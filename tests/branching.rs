use crate::common::interpret_using_probe;
mod common;

#[test]
fn if_statement_test() {
    let src = r#"
    if (4 > 3) {
        print "Inside if stmt";
        if (4 < 3) {
            print "Unreachable";
        }
        var a = "End block";
        print a;
    }
    print "Done";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["Inside if stmt", "End block", "Done"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn if_else_statement_test() {
    let src = r#"
    if (4 > 3) {
        print "True condition passed";
    } else {
        print "Unreachable";
    }
    print "Jmp1";
    if (4 < 3) {
        print "Unreachable";
    } else {
        print "Else condition passed";
    }
    print "Jmp2";
    "#;
    let probe = interpret_using_probe(src);
    let output = &[
        "True condition passed",
        "Jmp1",
        "Else condition passed",
        "Jmp2",
    ];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}
