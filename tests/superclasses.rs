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
