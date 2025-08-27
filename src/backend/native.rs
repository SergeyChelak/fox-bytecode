use crate::Value;

pub fn native_list(args: &[Value]) -> Value {
    args.iter().enumerate().for_each(|(idx, val)| {
        println!("{idx} : {val}");
    });
    Value::number(args.len() as f32)
}
