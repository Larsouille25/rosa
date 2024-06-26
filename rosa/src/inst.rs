//! ByteCode of Rosa.

use std::{collections::HashMap, fmt::Debug};

use lazy_static::lazy_static;

use crate::{arith_impl, Result, RuntimeError, VirtualMachine};

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
        vm.exit = Some(vm.stack_pop()?);
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
        vm.stack_push_raw(data);
        Ok(())
    }

    fn opcode(&self) -> u8 {
        2
    }
}

arith_impl! {
    RustType = u8;

    MulInst = U8MulInst;
    MulInstOpcode = 3;

    DivInst = U8DivInst;
    DivInstOpcode = 4;

    RemInst = U8RemInst;
    RemInstOpcode = 5;

    AddInst = U8AddInst;
    AddInstOpcode = 6;

    SubInst = U8SubInst;
    SubInstOpcode = 7;

    ShrInst = U8ShrInst;
    ShrInstOpcode = 8;

    ShlInst = U8ShlInst;
    ShlInstOpcode = 9;

    CompLTInst = U8CompLTInst;
    CompLTInstOpcode = 10;

    CompGTInst = U8CompGTInst;
    CompGTInstOpcode = 11;

    CompLTEInst = U8CompLTEInst;
    CompLTEInstOpcode = 12;

    CompGTEInst = U8CompGTEInst;
    CompGTEInstOpcode = 13;

    CompEqInst = U8CompEqInst;
    CompEqInstOpcode = 14;

    CompNeInst = U8CompNeInst;
    CompNeInstOpcode = 15;
}

arith_impl! {
    RustType = u8;

    MulInst = U16MulInst;
    MulInstOpcode = 16;

    DivInst = U16DivInst;
    DivInstOpcode = 17;

    RemInst = U16RemInst;
    RemInstOpcode = 18;

    AddInst = U16AddInst;
    AddInstOpcode = 19;

    SubInst = U16SubInst;
    SubInstOpcode = 20;

    ShrInst = U16ShrInst;
    ShrInstOpcode = 21;

    ShlInst = U16ShlInst;
    ShlInstOpcode = 22;

    CompLTInst = U16CompLTInst;
    CompLTInstOpcode = 23;

    CompGTInst = U16CompGTInst;
    CompGTInstOpcode = 24;

    CompLTEInst = U16CompLTEInst;
    CompLTEInstOpcode = 25;

    CompGTEInst = U16CompGTEInst;
    CompGTEInstOpcode = 26;

    CompEqInst = U16CompEqInst;
    CompEqInstOpcode = 27;

    CompNeInst = U16CompNeInst;
    CompNeInstOpcode = 28;
}

/// An help macro used to more easily build the [instruction set] of the VM.
///
/// [instruction set]: struct@crate::inst::INSTRUCTION_SET
#[macro_export]
macro_rules! inst_set {
    ($($inst:expr),* $(,)?) => {
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
    pub static ref INSTRUCTION_SET: HashMap<u8, &'static dyn Instruction> = inst_set!(
        NoOpInst,
        ExitInst,
        ConstInst,
        // u8
        U8MulInst,
        U8DivInst,
        U8RemInst,
        U8AddInst,
        U8SubInst,
        U8ShrInst,
        U8ShlInst,
        U8CompLTInst,
        U8CompGTInst,
        U8CompLTEInst,
        U8CompGTEInst,
        U8CompEqInst,
        U8CompNeInst,
        // u16
        U16MulInst,
        U16DivInst,
        U16RemInst,
        U16AddInst,
        U16SubInst,
        U16ShrInst,
        U16ShlInst,
        U16CompLTInst,
        U16CompGTInst,
        U16CompLTEInst,
        U16CompGTEInst,
        U16CompEqInst,
        U16CompNeInst,
    );
}
