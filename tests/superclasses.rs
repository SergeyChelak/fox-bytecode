use crate::common::interpret_using_probe;
mod common;

#[test]
fn class_inherit_itself_test() {
    let src = r#"
        class A : A {}
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("A class can't inherit from itself"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn class_invalid_superclasses_test() {
    let src = r#"
        var NotClass = "So not a class";
        class OhNo : NotClass {}
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("Superclass must be a class"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn super_outside_class_test() {
    let src = r#"
        super.a = 10;
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("Can't use 'super' outside of a class"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn super_on_root_class_test() {
    let src = r#"
        class A {
            do() {
                super.x = 10;
            }
        }
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("Can't use 'super' in a class with no superclass"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn call_super_methods_test() {
    let src = r#"
        class A {
          method() {
            print "A method";
          }
        }

        class B : A {
          method() {
            print "B method";
          }

          test() {
            super.method();
          }
        }

        class C : B {}

        C().test();
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["A method", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn get_super_opcode_test() {
    let src = r#"
        class Doughnut {
          cook() {
            print "Dunk in the fryer.";
            this.finish("sprinkles");
          }

          finish(ingredient) {
            print "Finish with " + ingredient;
          }
        }

        class Cruller : Doughnut {
          finish(ingredient) {
            // No sprinkles, always icing.
            super.finish("icing");
          }
        }

        var c = Cruller();
        c.cook();
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["Dunk in the fryer.", "Finish with icing", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}
