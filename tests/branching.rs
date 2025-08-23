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

#[test]
fn logical_operators_test() {
    let src = r#"
        // short 'and' eval
        var a = 1 > 4 and 5 < 10;
        print a;
        // long 'and' eval
        var b = 5 > 10 and 1 > 4;
        print b;
        // long 'or' eval
        var c = 5 > 10 or 5 > 4;
        print c;
        // short 'or' eval
        var d = 4 > 2 or 8 < 11;
        print d;
        // precedence
        var e = 5 < 10 and 4 < 5 or 7 > 8 and 10 > 11;
        print e;
    "#;
    let probe = interpret_using_probe(src);
    let output = &["false", "false", "true", "true", "true"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}
