use rosa::{
    inst::{ExitInst, NoOpInst},
    Chunk, VirtualMachine,
};
use termcolor::{ColorChoice, StandardStream};

fn main() {
    println!("Hello, Rosa 🌹!\n");
    // TODO: use CLAP to handle arguments etc..

    let mut s = StandardStream::stdout(ColorChoice::Auto);
    let chunk = Chunk::from(vec![NoOpInst::OPCODE, ExitInst::OPCODE]);
    let mut vm = VirtualMachine::new(chunk);

    match vm.run() {
        Ok(code) if code != 0 => println!("Exitted with code {code:?}"),
        Ok(_) => {}
        Err(err) => err.format(&vm, &mut s).unwrap(),
    }
}
