//! ByteCode of Rosa.

use std::{collections::HashMap, fmt::Debug};

use lazy_static::lazy_static;

use crate::{Result, VirtualMachine};

/// An abstraction over what is an instruction of the Rosa VM.
///
/// # Note
/// Instruction needs to be sync because it is used inside a lazy_static that
/// require it to be thread safe.
pub trait Instruction: Sync + Debug {
    fn execute(&self, vm: &mut VirtualMachine) -> Result<()>;
}

#[derive(Debug)]
pub struct NoOpInst;

impl NoOpInst {
    pub const OPCODE: u8 = 0;
}

impl Instruction for NoOpInst {
    fn execute(&self, _: &mut VirtualMachine) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct ExitInst;

impl ExitInst {
    pub const OPCODE: u8 = 1;
}

impl Instruction for ExitInst {
    fn execute(&self, vm: &mut VirtualMachine) -> Result<()> {
        vm.exit = Some(vm.stack_pop_one()?);
        Ok(())
    }
}

/// An help macro used to more easily build the [instruction set] of the VM.
///
/// [instruction set]: struct@crate::inst::INSTRUCTION_SET
#[macro_export]
macro_rules! inst_set {
    ($($inst:tt),*) => {
        HashMap::from([
            $( ($inst::OPCODE, &$inst as &'static dyn Instruction), )*
        ])
    };
}

lazy_static! {
    /// The actual Instructions of the [Virtual Machine][crate::VirtualMachine].
    ///
    /// Using an HashMap with a key of type u8 is kinda dumb but idk what to
    /// use then??
    pub static ref INSTRUCTION_SET: HashMap<u8, &'static dyn Instruction> = inst_set!(NoOpInst, ExitInst);
}
