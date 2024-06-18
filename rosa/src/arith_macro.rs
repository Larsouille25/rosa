//! This mod provide a macro to implement all arithemetic instructions of the VM.

#[doc(hidden)]
#[macro_export]
macro_rules! arith_inst {
    ($type:ty, $name:ident, $opcode:expr, $op:tt) => {
        // TODO: Add documentation here like the doc of the ConstInst.
        #[derive(Debug)]
        pub struct $name;

        impl $crate::inst::Instruction for $name {
            fn execute(&self, vm: &mut $crate::VirtualMachine) -> $crate::Result<()> {
                let b = vm.stack_pop::<$type>()?;
                let a = vm.stack_pop::<$type>()?;
                // TODO: provide better error handling of operations
                vm.stack_push(a $op b);
                Ok(())
            }

            fn opcode(&self) -> u8 {
                $opcode
            }
        }
    };
}

#[macro_export]
macro_rules! arith_impl {
    (
        RustType = $type:ty;

        MulInst = $mulinst:ident;
        MulInstOpcode = $mulinst_opcode:expr;

        DivInst = $divinst:ident;
        DivInstOpcode = $divinst_opcode:expr;

        RemInst = $reminst:ident;
        RemInstOpcode = $reminst_opcode:expr;

        AddInst = $addinst:ident;
        AddInstOpcode = $addinst_opcode:expr;

        SubInst = $subinst:ident;
        SubInstOpcode = $subinst_opcode:expr;

        ShrInst = $shrinst:ident;
        ShrInstOpcode = $shrinst_opcode:expr;

        ShlInst = $shlinst:ident;
        ShlInstOpcode = $shlinst_opcode:expr;

        CompLTInst = $compltinst:ident;
        CompLTInstOpcode = $compltinst_opcode:expr;

        CompGTInst = $compgtinst:ident;
        CompGTInstOpcode = $compgtinst_opcode:expr;

        CompLTEInst = $complteinst:ident;
        CompLTEInstOpcode = $complteinst_opcode:expr;

        CompGTEInst = $compgteinst:ident;
        CompGTEInstOpcode = $compgteinst_opcode:expr;

        CompEqInst = $compeqinst:ident;
        CompEqInstOpcode = $compeqinst_opcode:expr;

        CompNeInst = $compneinst:ident;
        CompNeInstOpcode = $compneinst_opcode:expr;
    ) => {
        $crate::arith_inst! { $type, $mulinst, $mulinst_opcode, * }
        $crate::arith_inst! { $type, $divinst, $divinst_opcode, / }
        $crate::arith_inst! { $type, $reminst, $reminst_opcode, % }

        $crate::arith_inst! { $type, $addinst, $addinst_opcode, + }
        $crate::arith_inst! { $type, $subinst, $subinst_opcode, - }

        $crate::arith_inst! { $type, $shrinst, $shrinst_opcode, >> }
        $crate::arith_inst! { $type, $shlinst, $shlinst_opcode, << }

        $crate::arith_inst! { $type, $compltinst, $compltinst_opcode, < }
        $crate::arith_inst! { $type, $compgtinst, $compgtinst_opcode, > }
        $crate::arith_inst! { $type, $complteinst, $complteinst_opcode, <= }
        $crate::arith_inst! { $type, $compgteinst, $compgteinst_opcode, >= }

        $crate::arith_inst! { $type, $compeqinst, $compeqinst_opcode, == }
        $crate::arith_inst! { $type, $compneinst, $compneinst_opcode, != }
    };
}
