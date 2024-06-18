use std::collections::HashMap;

use rosa::{
    inst::{ConstInst, ExitInst, Instruction, U8AddInst},
    Chunk, ConstantPool, VirtualMachine,
};
use termcolor::{ColorChoice, StandardStream};

fn main() {
    println!("Hello, Rosa 🌹!\n");
    // TODO: use CLAP to handle arguments etc..

    let mut s = StandardStream::stdout(ColorChoice::Auto);
    let chunk = Chunk::from(vec![
        ConstInst.opcode(),
        0,
        ConstInst.opcode(),
        1,
        U8AddInst.opcode(),
        ExitInst.opcode(),
    ]);
    let pool = ConstantPool::new(HashMap::from([(0, 1), (1, 1)]), vec![52, 49]);
    let mut vm = VirtualMachine::new(chunk, pool);

    match vm.run() {
        Ok(code) if code != 0 => println!("Exited with code {code:?}"),
        Ok(_) => {}
        Err(err) => err.format(&vm, &mut s).unwrap(),
    }
}
