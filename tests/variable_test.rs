use crate::common::interpret_using_probe;

mod common;

#[test]
fn global_variables() {
    let src = r"
        var a = 1;
        print a;
        a = 2;
        print a;
        var b;
        b = 3;
        print b;
    ";
    let probe = interpret_using_probe(src);
    let output = &["1", "2", "3"];
    if let Some(err) = probe.borrow().vm_error() {
        panic!("Err: {err}");
    }
    probe.borrow().assert_output_match(output);
}

#[test]
fn local_scopes() {
    let src = r"
        {
            var a = 1;
            {
                var b = 2;
                {
                    var c = 3;
                    {
                        var d = 4;
                    }
                    var e = 5;
                }
            }
            var f = 6;
            {
                var g = 7;
            }
        }
    ";
    let probe = interpret_using_probe(src);
    assert!(!probe.borrow().has_errors());
}

#[test]
fn local_scope_var_duplicate() {
    let src = r"
        {
            var a = 1;
            var a = 2;
        }
    ";
    let probe = interpret_using_probe(src);
    assert!(probe.borrow().has_errors());
}

#[test]
fn local_var_out_of_scope() {
    let src = r"
        {
            {
                var a = 1;
            }
            print a;
        }
    ";
    let probe = interpret_using_probe(src);
    assert!(probe.borrow().has_errors());
}

#[test]
fn local_var_own_value_in_init() {
    let src = r"
        {
            var a = a;
            print a;
        }
    ";
    let probe = interpret_using_probe(src);
    assert!(probe.borrow().has_errors());
}

#[test]
fn local_scope_cleanup() {
    let src = r"
        {
            var a = 1;
            {
                var a = 2;
                {
                    var a = 3;
                    print a;
                }
                print a;
            }
            print a;
        }
    ";
    let probe = interpret_using_probe(src);
    let output = &["3", "2", "1"];
    assert!(!probe.borrow().has_errors());
    probe.borrow().assert_output_match(output);
}

#[test]
fn update_local_variables() {
    let src = r"
        {
            var a = 1;
            print a;
            a = 2;
            print a;
            {
                var a = 3;
                print a;
                a = 4;
                print a;
            }
            print a;
        }
    ";
    let probe = interpret_using_probe(src);
    let output = &["1", "2", "3", "4", "2"];
    assert!(!probe.borrow().has_errors());
    probe.borrow().assert_output_match(output);
}
