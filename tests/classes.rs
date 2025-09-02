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
