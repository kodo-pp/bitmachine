mod ast;
mod bindings;
mod bitstring;
mod bytecode;
mod callable;
mod coded_function;
mod compiled;
mod native_function;
mod parser;
mod pattern;
mod translator;
mod value;
mod vm;

use crate::bindings::Bindings;
use crate::bitstring::BitString;
use crate::translator::Compile;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Usage: {argv0} <filename>")]
struct UsageError {
    argv0: String,
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    Ok(data)
}

fn main() -> anyhow::Result<()> {
    let filename = {
        let mut args = std::env::args();
        let argv0 = args.next().unwrap();
        args.next().ok_or(UsageError { argv0 })?
    };
    let code = read_file(Path::new(&filename))?;

    let program = parser::parse(&code)?;
    let compiled_program = program.compile();

    dbg!(&compiled_program);

    let mut vm = vm::VM::new(
        Bindings::new(
            vec![("test", BitString::empty().into())]
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v))
                .collect(),
        )
        .union_with(compiled_program.into())
        .union_with(native_function::make_bindings()),
    );

    vm.invoke_by_name("main", vec![])?;
    loop {
        match vm.step() {
            Err(vm::ExecError::TaskStackEmpty) => break,
            x => x?,
        }
    }

    Ok(())
}
