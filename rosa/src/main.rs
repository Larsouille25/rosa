use rosa::{
    inst::{ExitInst, NoOpInst},
    Chunk, VirtualMachine,
};

fn main() {
    println!("Hello, Rosa 🌹!\n");
    // TODO: use CLAP to handle arguments etc..

    let chunk = Chunk::from(vec![NoOpInst::OPCODE, ExitInst::OPCODE]);
    let mut vm = VirtualMachine::new(chunk);

    vm.run();
}
