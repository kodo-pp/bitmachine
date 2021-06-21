mod parser;
mod ast;
mod bindings;
mod bitstring;
mod bytecode;
mod callable;
mod pattern;
mod translator;
mod value;
mod compiled;
mod coded_function;

use crate::translator::Compile;

fn main() -> anyhow::Result<()> {
    const CODE: &'static str = "add a+?x b+1 ?y+01 _ = (@inc (add a b)+x)+(11)";
    //const CODE: &'static str = "foo = 101011+1+0";

    let program = parser::parse(CODE)?;
    dbg!(&program);
    
    let compiled_program = program.compile();
    dbg!(&compiled_program);

    Ok(())
}
