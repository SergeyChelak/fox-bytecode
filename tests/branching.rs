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

#[test]
fn while_loop_test() {
    let src = r#"
        var a = 0;
        while (a < 5) {
            print a;
            a = a + 1;
        }
        print "Done";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["0", "1", "2", "3", "4", "Done"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn for_loop_test() {
    let src = r#"
        // init; condition; modifier
        for (var i = 0; i < 5; i = i + 1) {
            print i;
        }
        // ; condition; modifier
        var x = 4;
        for (;x < 10; x = x + 2) {
            print x;
        }
        // ; condition;
        var y = 1;
        for (; y < 10; ) {
            print y;
            y = y + y;
        }
        print "Done";
    "#;
    let probe = interpret_using_probe(src);
    let output = &[
        "0", "1", "2", "3", "4", "4", "6", "8", "1", "2", "4", "8", "Done",
    ];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn break_inside_while_loop_test() {
    let src = r#"
        var counter = 0;
        while (true) {
          print "Loop iteration:" + counter;
          counter = counter + 1;
          if (counter == 3) {
            print "The counter is 3, breaking out of the loop.";
            break;
          }
        }
        print "Done";
    "#;
    let probe = interpret_using_probe(src);
    let output = &[
        "Loop iteration:0",
        "Loop iteration:1",
        "Loop iteration:2",
        "The counter is 3, breaking out of the loop.",
        "Done",
    ];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn break_inside_for_loop_test() {
    let src = r#"
        for (var i = 0; i < 10; i = i + 1) {
          if (i == 5) {
            print "Breaking the loop at i = "+ i;
            break;
          }
          print "Current number:"+ i;
        }
        print "Done";
    "#;
    let probe = interpret_using_probe(src);
    let output = &[
        "Current number:0",
        "Current number:1",
        "Current number:2",
        "Current number:3",
        "Current number:4",
        "Breaking the loop at i = 5",
        "Done",
    ];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn break_nested_for_loop_test() {
    let src = r#"
        for (var i = 0; i < 3; i = i + 1) {
          print "Outer loop iteration: " + i;
          for (var j = 0; j < 5; j = j + 1) {
            if (j == 2) {
              print "  Inner loop breaking at j = 2";
              break;
            }
            print "  * Inner loop iteration: " + j;
          }
        }
        print "Done";
    "#;
    let probe = interpret_using_probe(src);
    let output = &[
        "Outer loop iteration: 0",
        "  * Inner loop iteration: 0",
        "  * Inner loop iteration: 1",
        "  Inner loop breaking at j = 2",
        "Outer loop iteration: 1",
        "  * Inner loop iteration: 0",
        "  * Inner loop iteration: 1",
        "  Inner loop breaking at j = 2",
        "Outer loop iteration: 2",
        "  * Inner loop iteration: 0",
        "  * Inner loop iteration: 1",
        "  Inner loop breaking at j = 2",
        "Done",
    ];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn break_nested_while_loop_test() {
    let src = r#"
        var outerCount = 0;
        while (outerCount < 3) {
          var innerCount = 0;
          print "Outer loop iteration: " + outerCount;
          while (innerCount < 5) {
            if (innerCount == 3) {
              print "  Inner loop breaking at innerCount = 3";
              break; // This only breaks the inner 'while' loop
            }
            print "  Inner loop iteration: " +innerCount;
            innerCount = innerCount + 1;
          }
          outerCount = outerCount + 1;
        }
        print "Done";
    "#;
    let probe = interpret_using_probe(src);
    let output = &[
        "Outer loop iteration: 0",
        "  Inner loop iteration: 0",
        "  Inner loop iteration: 1",
        "  Inner loop iteration: 2",
        "  Inner loop breaking at innerCount = 3",
        "Outer loop iteration: 1",
        "  Inner loop iteration: 0",
        "  Inner loop iteration: 1",
        "  Inner loop iteration: 2",
        "  Inner loop breaking at innerCount = 3",
        "Outer loop iteration: 2",
        "  Inner loop iteration: 0",
        "  Inner loop iteration: 1",
        "  Inner loop iteration: 2",
        "  Inner loop breaking at innerCount = 3",
        "Done",
    ];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn break_outside_loop_test() {
    let src = r#"
        var x = 1;
        break;
        print x;
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("'break' statement allowed inside loops only"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn continue_outside_loop_test() {
    let src = r#"
        var x = 1;
        continue;
        print x;
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("'continue' statement allowed inside loops only"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn switch_test() {
    let src = r#"
        for (var i = 0; i < 10; i = i + 1) {
            switch (i) {
                case 0: {
                    var tmp = "Zero";
                    print tmp;
                }
                case 1:
                    print "***";
                    print "One";
                    print "***";
                case 2: print "Two";
                case 3: print "Three";
                case 4: print "Four";
                default: {
                    var formatted = "Value " + i;
                    print formatted;
                }
            }
        }
        var msg = "Done";
        print msg;
    "#;
    let probe = interpret_using_probe(src);
    let output = &[
        "Zero", "***", "One", "***", "Two", "Three", "Four", "Value 5", "Value 6", "Value 7",
        "Value 8", "Value 9", "Done",
    ];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}
