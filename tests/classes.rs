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

#[test]
fn class_instantiation_test() {
    let src = r#"
        class Brioche {}
        print Brioche();
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["<Brioche instance>", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn class_undefined_property_test() {
    let src = r#"
        class Brioche {}
        var obj = Brioche();
        print obj.prop;
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("Undefined property 'prop'"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn class_fields_get_set_test() {
    let src = r#"
        class Toast {}
        var toast = Toast();
        print toast.jam = "grape";
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["grape", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn class_non_class_get_test() {
    let src = r#"
        var obj = "not an instance";
        print obj.field;
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("Only instances have fields"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn class_non_class_set_test() {
    let src = r#"
        var obj = "not an instance";
        obj.field = 50;
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("Only instances have fields"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn calling_methods_test() {
    let src = r#"
        class Scone {
          topping(first, second) {
            print "scone with " + first + " and " + second;
          }
        }

        var scone = Scone();
        scone.topping("berries", "cream");
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["scone with berries and cream", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn class_this_nested_test() {
    let src = r#"
        class Nested {
          method() {
            fun function() {
              print this;
            }

            function();
          }
        }

        Nested().method();
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["<Nested instance>", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn misuse_this_at_top_level_test() {
    let src = r#"
        print this; // At top level.
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("Can't use 'this' outside of a class"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn misuse_this_in_function_test() {
    let src = r#"
        fun notMethod() {
          print this; // In a function.
        }
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("Can't use 'this' outside of a class"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn class_initializer_test() {
    let src = r#"
        class Class {
          init() {
            print "Initialized";
          }
        }

        var x = Class();
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["Initialized", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

// #[test]
fn _class_initializer_with_args_test() {
    let src = r#"
        class Class {
          init(a) {
            this.value = a;
            // return this;
          }
        }

        var x = Class("hello");
        print x.value;
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["hello", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn class_initializer_with_wrong_args_test() {
    let src = r#"
        class Class {
          init(a, b, c) {
            print a + b + c;
          }
        }

        Class("hello");
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("Expected 3 arguments but got 1"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn class_initializer_implicit_test() {
    let src = r#"
        class Class {
          init() {
            print "ready";
          }
        }

        Class();
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["ready", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn class_initializer_invalid_implicit_test() {
    let src = r#"
        class Class {
          init() {
            print "ready";
          }
        }

        Class(1, 2, 3);
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("Expected 0 arguments but got 3"),
        probe.borrow().top_error_message()
    );
}
