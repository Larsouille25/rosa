use std::collections::HashMap;

use rosa::{
    inst::{ConstInst, ExitInst, Instruction},
    Chunk, ConstantPool, VirtualMachine,
};
use termcolor::{ColorChoice, StandardStream};

fn main() {
    println!("Hello, Rosa 🌹!\n");
    // TODO: use CLAP to handle arguments etc..

    let mut s = StandardStream::stdout(ColorChoice::Auto);
    let chunk = Chunk::from(vec![ConstInst.opcode(), 0, ExitInst.opcode()]);
    let pool = ConstantPool::new(HashMap::from([(0, 1)]), vec![101]);
    let mut vm = VirtualMachine::new(chunk, pool);

    match vm.run() {
        Ok(code) if code != 0 => println!("Exited with code {code:?}"),
        Ok(_) => {}
        Err(err) => err.format(&vm, &mut s).unwrap(),
    }
}
