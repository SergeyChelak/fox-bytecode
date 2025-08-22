use crate::common::interpret_by_probe;

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
    let probe = interpret_by_probe(src);
    let output = &["1".to_string(), "2".to_string(), "3".to_string()];
    if let Some(err) = probe.borrow().vm_error() {
        panic!("Err: {err}");
    }
    probe.borrow().assert_output_match(output);
}
