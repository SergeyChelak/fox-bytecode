extern crate fox_bytecode;

mod probe;
pub use probe::*;

use std::rc::Rc;

use fox_bytecode::{EmptyNative, NativeFunctionsProvider, Shared, interpret, shared};

pub fn str_to_code_ref(input: &str) -> Rc<Vec<char>> {
    Rc::new(input.chars().collect())
}

pub fn interpret_using_probe(input: &str) -> Shared<Probe> {
    interpret_with(input, EmptyNative)
}

pub fn interpret_with(
    input: &str,
    native_fn_provider: impl NativeFunctionsProvider,
) -> Shared<Probe> {
    let code_ref = str_to_code_ref(input);
    let probe = Probe::default();
    // HACK: Due unknown reasons,  the compiler doesn't see call of this function in funcs.rs
    // I was managed to add this "fake" call to make it happy
    assert!(probe.stack_trace_text().is_none());
    let probe_shared = shared(probe);
    interpret(
        code_ref,
        probe_shared.clone(),
        probe_shared.clone(),
        native_fn_provider,
    );
    probe_shared
}
