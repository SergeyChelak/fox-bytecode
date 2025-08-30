// use fox_bytecode::compile;

// use crate::common::{interpret_using_probe, interpret_with, str_to_code_ref};
// mod common;

// #[test]
// fn emit_upvalues_test() {
//     let src = r#"
//         fun outer() {
//           var a = 1;
//           var b = 2;
//           fun middle() {
//             var c = 3;
//             var d = 4;
//             fun inner() {
//               print a + c + b + d;
//             }
//           }
//         }
//     "#;
//     let code_ref = str_to_code_ref(src);
//     let closure = compile(code_ref).expect("Compilation failed");

//     let mut offset = 0;
//     while let Ok(instr) = closure.func().chunk().fetch(&mut offset) {
//         println!("{instr:?}");
//     }
//     panic!();

//     let outer_fn = closure
//         .func()
//         .chunk()
//         .read_const(1)
//         .expect("outer function not found")
//         .as_function()
//         .expect("not a function at outer");

//     let middle_fn = outer_fn
//         .chunk()
//         .read_const(2)
//         .expect("middle function not found")
//         .as_function()
//         .expect("not a function at middle");

//     let inner_fn = middle_fn
//         .chunk()
//         .read_const(2)
//         .expect("middle function not found")
//         .as_function()
//         .expect("not a function at middle");

//     let mut offset = 0;
//     while let Ok(instr) = inner_fn.chunk().fetch(&mut offset) {
//         println!("{instr:?}");
//     }

//     // let mut idx = 0;
//     // while let Some(value) = middle_fn.chunk().read_const(idx) {
//     //     println!("Const @{idx} = {value} {value:?}");
//     //     idx += 1;
//     // }
//     panic!()
// }
