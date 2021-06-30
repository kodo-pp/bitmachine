use crate::bindings::Bindings;
use crate::bitstring::BitString;
use crate::callable::Callable;
use crate::value::Value;
use crate::vm::{BasicExecResult, ExecError};

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub func: fn(Vec<Value>) -> BasicExecResult<Value>,
    pub name: String,
}

pub fn make_bindings() -> Bindings {
    let vec: Vec<(_, fn(Vec<Value>) -> BasicExecResult<Value>)> =
        vec![("$", alloc_wrapper), ("?!", debug)];

    Bindings::new(
        vec.into_iter()
            .map(|(name_str, func)| {
                let name = String::from(name_str);
                (
                    name.clone(),
                    Value::Callable(Callable::Native(NativeFunction { func, name })),
                )
            })
            .collect(),
    )
}

fn alloc_wrapper(args: Vec<Value>) -> BasicExecResult<Value> {
    alloc(args.clone()).ok_or_else(|| ExecError::NoMatch {
        func_name: String::from("$"),
        args,
    })
}

fn alloc(mut args: Vec<Value>) -> Option<Value> {
    if args.len() != 1 {
        return None;
    }

    let bit_string = args.remove(0).into_bit_string()?;
    let num_bytes = bit_string.as_usize();

    // Safety: this code is unsafe. Good luck!
    let encoded_ptr: usize = unsafe {
        let ptr = libc::malloc(num_bytes);
        std::mem::transmute(ptr)
    };

    Some(BitString::from_u64(encoded_ptr as u64).into())
}

fn debug(args: Vec<Value>) -> BasicExecResult<Value> {
    println!("debug: {:?}", args);
    Ok(Value::BitString(BitString::empty()))
}
