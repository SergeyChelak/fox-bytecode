use crate::common::interpret_using_probe;
mod common;

#[test]
fn closed_upvalues_test() {
    let src = r#"
        fun outer() {
          var x = "outside";
          fun inner() {
            print x;
          }

          return inner;
        }

        var closure = outer();
        closure();
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["outside", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn value_and_vars_test() {
    let src = r#"
        var globalSet;
        var globalGet;

        fun main() {
          var a = "initial";

          fun set() { a = "updated"; }
          fun get() { print a; }

          globalSet = set;
          globalGet = get;
        }

        main();
        globalGet();
        globalSet();
        globalGet();
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["initial", "updated", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn closure_test() {
    let src = r#"
        var x = "global";
        fun outer() {
          var x = "outer";
          fun inner() {
            print x;
          }
          inner();
        }
        outer();
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["outer", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn make_closure_test() {
    let src = r#"
        fun makeClosure() {
          var local = "local";
          fun closure() {
            print local;
          }
          return closure;
        }
        var closure = makeClosure();
        closure();
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["local", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn closure_mutate_shared_test() {
    let src = r#"
        fun outer() {
          var x = 1;
          x = 2;
          fun inner() {
            print x;
          }
          x = 3;
          inner();
        }
        outer();
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["3", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn closure_nested_upvalues_test() {
    let src = r#"
        fun outer() {
          var a = 1;
          var b = 2;
          fun middle() {
            var c = 3;
            var d = 4;
            fun inner() {
                print a + c + b + d;
            }
            return inner;
          }
          return middle;
        }

        var f1 = outer();
        var f2 = f1();
        f2();
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["10", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn closure_nesting_test() {
    let src = r#"
        fun outer() {
          var x = "value";
          fun middle() {
            fun inner() {
              print x;
            }

            print "create inner closure";
            return inner;
          }

          print "return from outer";
          return middle;
        }

        var mid = outer();
        var in = mid();
        in();
    "#;
    let probe = interpret_using_probe(src);
    let output = &["return from outer", "create inner closure", "value"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn closure_capture_loop_var_test() {
    let src = r#"
        var globalOne;
        var globalTwo;

        fun main() {
          for (var a = 1; a <= 2; a = a + 1) {
            fun closure() {
              print a;
            }
            if (globalOne == nil) {
              globalOne = closure;
            } else {
              globalTwo = closure;
            }
          }
        }

        main();
        globalOne();
        globalTwo();
    "#;
    let probe = interpret_using_probe(src);
    let output = &["3", "3"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}
