//! ByteCode of Rosa.

use std::{collections::HashMap, fmt::Debug};

use lazy_static::lazy_static;

use crate::{Result, RuntimeError, VirtualMachine};

/// An abstraction over what is an instruction of the Rosa VM.
///
/// # Note
/// Instruction needs to be sync because it is used inside a lazy_static that
/// require it to be thread safe.
pub trait Instruction: Sync + Debug {
    fn execute(&self, vm: &mut VirtualMachine) -> Result<()>;

    fn opcode(&self) -> u8;
}

/// The No-operation instruction, does nothing.
///
/// # Bytecode Layout
///
/// `NOOP`
///
/// Only the Op code.
///
/// # Stack
///
/// Does nothing.
#[derive(Debug)]
pub struct NoOpInst;

impl Instruction for NoOpInst {
    fn execute(&self, _: &mut VirtualMachine) -> Result<()> {
        Ok(())
    }

    fn opcode(&self) -> u8 {
        0
    }
}

/// The exit instruction, stops the VM with the code poped from the stack.
///
/// # Bytecode Layout
///
/// `EXIT`
///
/// Only the Op code.
///
/// # Stack
///
/// Pops a byte from the stack and exit with the code poped from the stack.
#[derive(Debug)]
pub struct ExitInst;

impl Instruction for ExitInst {
    fn execute(&self, vm: &mut VirtualMachine) -> Result<()> {
        vm.exit = Some(vm.stack_pop_one()?);
        Ok(())
    }

    fn opcode(&self) -> u8 {
        1
    }
}

/// The const instruction, loads a constant from the constant pool and push it
/// on the stack.
///
/// # Bytecode Layout
///
/// `CONST offset:dynint`
///
/// The opcode for the const instruction is followed by the offset in the pool
/// encoded as a dynamic integer.
///
/// # Stack
///
/// Push the constant on to the stack.
#[derive(Debug)]
pub struct ConstInst;

impl Instruction for ConstInst {
    fn execute(&self, vm: &mut VirtualMachine) -> Result<()> {
        let offset: usize = vm.read_dyn_int()? as usize;
        let data = vm
            .pool
            .get(offset)
            .ok_or(RuntimeError::UnknownConst { offset })?
            .to_owned();
        vm.stack_push(data);
        Ok(())
    }

    fn opcode(&self) -> u8 {
        2
    }
}

/// An help macro used to more easily build the [instruction set] of the VM.
///
/// [instruction set]: struct@crate::inst::INSTRUCTION_SET
#[macro_export]
macro_rules! inst_set {
    ($($inst:expr),*) => {
        HashMap::from([
            $( ($crate::inst::Instruction::opcode(&$inst), &$inst as &'static dyn Instruction), )*
        ])
    };
}

lazy_static! {
    /// The actual Instructions of the [Virtual Machine][crate::VirtualMachine].
    ///
    /// Using an HashMap with a key of type u8 is kinda dumb but idk what to
    /// use then??
    pub static ref INSTRUCTION_SET: HashMap<u8, &'static dyn Instruction> = inst_set!(NoOpInst, ExitInst, ConstInst);
}
