use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::Display,
    io::{self, Write},
    mem::size_of,
};

use lazy_static::lazy_static;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

pub mod arith_macro;
pub mod inst;

/// A chunk of Rosa ByteCode.
#[derive(Debug)]
pub struct Chunk {
    data: Vec<u8>,
}

impl From<Vec<u8>> for Chunk {
    fn from(data: Vec<u8>) -> Self {
        Chunk { data }
    }
}

impl Chunk {
    pub fn get(&self, i: usize) -> Option<u8> {
        self.data.get(i).copied()
    }
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

lazy_static! {
    static ref WHITE_BOLD: ColorSpec = {
        let mut color = ColorSpec::new();
        color.set_fg(Some(Color::White));
        color.set_bold(true);
        color
    };
    static ref RED_BOLD: ColorSpec = {
        let mut color = ColorSpec::new();
        color.set_fg(Some(Color::Red));
        color.set_bold(true);
        color
    };
}

#[derive(Clone, Debug)]
pub enum RuntimeError {
    /// Stack over flow
    OverFlow,
    /// Stack under flow
    UnderFlow,
    /// Unknown instruction
    UnknownInst { inst: u8 },
    /// IP tried to get an instruction out of the boundaries of the Chunk
    ProgramOverFlow,
    /// failed to decode a dynamic integer.
    DynInt,
    /// unknown offset in the constant pool
    UnknownConst { offset: usize },
    /// arithmetic error, the message ('msg') explains what is the arithmetic
    /// error in question
    ArithmeticError { msg: &'static str },
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OverFlow => write!(f, "stack over flow"),
            Self::UnderFlow => write!(f, "stack under flow"),
            Self::UnknownInst { inst } => write!(f, "unknown instruction {inst:#04X?}"),
            Self::ProgramOverFlow => write!(f, "over run of the chunk bytecode"),
            Self::DynInt => write!(f, "failed to decode a dynamic integer"),
            Self::UnknownConst { offset } => {
                write!(f, "unknown offset ({offset:#010X?}) in the constant pool")
            }
            Self::ArithmeticError { msg } => {
                write!(f, "arithmetic error: {msg}")
            }
        }
    }
}

impl RuntimeError {
    pub fn format(&self, vm: &VirtualMachine, s: &mut StandardStream) -> io::Result<()> {
        s.set_color(&WHITE_BOLD)?;
        write!(s, "rosa: ")?;
        s.set_color(&RED_BOLD)?;
        write!(s, "runtime error: ")?;
        s.reset()?;
        writeln!(s, "{}", self)?;

        s.set_color(&WHITE_BOLD)?;
        writeln!(s, "STACK TRACE ({}):", vm.sp)?;

        let stacktrace = vm.stacktrace();
        if stacktrace.is_empty() {
            writeln!(s, "  ...")?;
        }
        s.reset()?;
        for (i, byte) in stacktrace.iter().enumerate() {
            writeln!(s, "  {i}: {:#04X?}", byte)?;
        }

        // TODO: format the call stack
        s.reset()?;
        s.flush()?;
        Ok(())
    }
}

/// The constant pool, it contains all constants that will be used by the
/// program.
#[derive(Debug, Clone)]
pub struct ConstantPool {
    // TODO: idk if a usize as the size is realy appropriate.. it maybe too big
    // for what we really need
    /// The layout of the constant pool. The key is the starting byte offset in
    /// the data buffer, the value is the lenght of the pointed data.
    layout: HashMap<usize, usize>,
    /// The buffer containing all the constant values in the program.
    data: Vec<u8>,
}

impl ConstantPool {
    pub const fn new(layout: HashMap<usize, usize>, data: Vec<u8>) -> ConstantPool {
        ConstantPool { layout, data }
    }

    #[inline]
    /// Get the constant at the offset. If it doesn't succeed this method
    /// returns `None`.
    pub fn get(&self, offset: usize) -> Option<&[u8]> {
        self.data.get(offset..offset + self.layout.get(&offset)?)
    }
}

impl Default for ConstantPool {
    fn default() -> Self {
        ConstantPool::new(HashMap::new(), Vec::new())
    }
}

pub trait FromBytes {
    fn from_bytes(bytes: &[u8]) -> Self;
}

pub trait IntoBytes {
    fn into_bytes(self, dst: &mut [u8]);
}

impl IntoBytes for bool {
    fn into_bytes(self, dst: &mut [u8]) {
        let bytes = Into::<u8>::into(self).to_be_bytes();
        dst.clone_from_slice(&bytes);
    }
}

macro_rules! bytes_impl {
    ($ty:ident) => {
        impl FromBytes for $ty {
            fn from_bytes(bytes: &[u8]) -> Self {
                let bytes: [u8; std::mem::size_of::<$ty>()] = bytes.try_into().unwrap();
                $ty::from_be_bytes(bytes)
            }
        }

        impl IntoBytes for $ty {
            fn into_bytes(self, dst: &mut [u8]) {
                let bytes = self.to_be_bytes();
                dst.clone_from_slice(&bytes);
            }
        }
    };

    ($( $type:ident ; )*) => (
        $(
            bytes_impl!{$type}
        )*
    )
}

bytes_impl! {
    u8;
    u16;
    u32;
    u64;
    u128;
    usize;
    i8;
    i16;
    i32;
    i64;
    i128;
    isize;
}

/// The stack virtual machine used to execute Rosa ByteCode.
#[derive(Debug)]
pub struct VirtualMachine {
    /// the bytecode executed by the VM
    program: Chunk,
    /// the instruction pointer, tells where we are in the chunk
    ip: usize,
    /// the stack used by the VM to do all the computation.
    stack: Vec<u8>,
    /// the stack pointer. it points the top of the stack. it starts from 0 so,
    /// when the stack pointer points to the first byte, it's equal to 1.
    sp: usize,
    /// if `None`, then keep running.
    /// but if `Some`, stop and the value is the exit code.
    exit: Option<u8>,
    pool: ConstantPool,
}

impl VirtualMachine {
    /// The default size of the stack, it is used to make the VM faster not
    /// waiting time to grow the stack.
    ///
    /// # Note
    /// This value is arbitrary and may change in the future, don't rely on
    /// it being a certain size.
    pub const DEFAULT_STACK_SIZE: usize = 2_usize.pow(16);

    /// Creates a new virtual machine with the given program. The stack has a
    /// default size of [`Self::DEFAULT_STACK_SIZE`].
    pub fn new(program: Chunk, pool: ConstantPool) -> VirtualMachine {
        VirtualMachine::with_stack_size(program, Self::DEFAULT_STACK_SIZE, pool)
    }

    /// Creates a new virtual machine with the given program and the initial
    /// stack size.
    pub fn with_stack_size(
        program: Chunk,
        stack_size: usize,
        pool: ConstantPool,
    ) -> VirtualMachine {
        VirtualMachine {
            program,
            ip: 0,
            stack: vec![0; stack_size],
            sp: 0,
            exit: None,
            pool,
        }
    }

    pub fn run(&mut self) -> Result<u8> {
        while self.exit.is_none() && !self.finished() {
            let inst = self.read_byte()?;
            match inst::INSTRUCTION_SET.get(&inst) {
                Some(inst) => {
                    inst.execute(self)?;
                }
                None => return Err(RuntimeError::UnknownInst { inst }),
            }
        }
        Ok(self.exit.unwrap())
    }

    /// Reads the byte pointed by `ip` and advance by one the `ip` pointer.
    pub fn read_byte(&mut self) -> Result<u8> {
        let byte = match self.program.get(self.ip) {
            Some(byte) => byte,
            None => return Err(RuntimeError::ProgramOverFlow),
        };
        self.ip += 1;
        Ok(byte)
    }

    pub fn stack_push_raw<'a>(&mut self, data: impl Into<Cow<'a, [u8]>>) {
        let data = data.into();
        let size = data.len();
        if self.stack.len() < self.sp + size {
            // maybe not optimal to double the size?
            self.extend_stack(self.sp);
        }
        let stack_bite = &mut self.stack[self.sp..self.sp + size];
        stack_bite.copy_from_slice(&data);
        self.sp += size;
    }

    pub fn stack_pop_raw(&mut self, amount: impl Into<usize>) -> Result<&[u8]> {
        let amount = amount.into();
        let frame = &self
            .stack
            .get(self.sp.checked_sub(amount).ok_or(RuntimeError::UnderFlow)?..self.sp);
        let poped = match frame {
            Some(data) => data,
            None => return Err(RuntimeError::UnderFlow),
        };
        self.sp -= amount;
        Ok(poped)
    }

    pub fn stack_pop_one(&mut self) -> Result<u8> {
        Ok(*self.stack_pop_raw(1usize)?.first().unwrap())
    }

    /// Extends the stack to contain `amount` more bytes of free space.
    pub fn extend_stack(&mut self, amount: usize) {
        self.stack.extend(vec![0; amount]);
    }

    #[must_use]
    pub fn stacktrace(&self) -> Box<[u8]> {
        // TODO: Make the stack size configurable.
        const TRACE_SIZE: usize = 32;
        let amount = TRACE_SIZE.min(self.sp);
        Box::from(self.stack.get(self.sp - amount..self.sp).unwrap())
    }

    #[must_use]
    pub fn finished(&mut self) -> bool {
        if self.ip >= self.program.data.len() {
            self.exit = Some(0);
            return true;
        }
        false
    }

    /// Read a dynamic integer from the chunk.
    pub fn read_dyn_int(&mut self) -> Result<u64> {
        let first = self.read_byte()?;
        let size = ones_before_zero(first);
        let number = DynamicInt::decode(
            self.program
                .data
                .get(self.ip - 1..self.ip + size as usize)
                .ok_or(RuntimeError::ProgramOverFlow)?,
        );
        self.ip += size as usize;
        match number {
            Some(num) => Ok(num),
            None => Err(RuntimeError::DynInt),
        }
    }

    pub fn stack_pop<T>(&mut self) -> Result<T>
    where
        T: FromBytes,
    {
        match self.stack_pop_raw(size_of::<T>()) {
            Ok(bytes) => Ok(T::from_bytes(bytes)),
            Err(err) => Err(err),
        }
    }

    pub fn stack_push<T: IntoBytes>(&mut self, value: T) {
        let size = size_of::<T>();
        if self.stack.len() < self.sp + size {
            // maybe not optimal to double the size?
            self.extend_stack(self.sp);
        }
        let here = &mut self.stack[self.sp..self.sp + size];
        value.into_bytes(here);
        self.sp += size;
    }
}

pub struct DynamicInt;

impl DynamicInt {
    // TODO: try to optimize the decode and encode functions.
    pub fn decode(buf: &[u8]) -> Option<u64> {
        let first = *buf.first()?;
        let ones = ones_before_zero(first);
        if ones == 0 {
            return Some(first.into());
        }
        let mask = (2_u8.pow(ones.into()) - 1) << (8 - ones);

        let mut result: u64 = 0;
        let first_part = first ^ mask;
        let pos = ones * 8;
        result |= (first_part as u64) << pos;

        for offset in 1..=ones {
            let rev: i16 = (ones as i16 - offset as i16).abs();
            let pos = rev * 8;
            result |= (buf[offset as usize] as u64) << pos;
        }
        Some(result)
    }

    pub fn encode(num: impl Into<u64>) -> Vec<u8> {
        let number: u64 = num.into();
        // STEPS:

        // 1. Compute how many bytes are needed depending on the size of the number
        let ones = ones_needed(number);

        // 2. Encode that size as ones in the first byte.
        let mut encoded_ones = 2u8.pow(ones.into()) - 1;
        if ones != 0 {
            encoded_ones <<= 8 - ones;
        }

        // 3. Encode all the remaining digits.
        let bits_to_encode = bits_dyn_int(ones);
        let first: u8 = encoded_ones | (number >> (bits_to_encode - bits_to_encode % 8)) as u8;
        if ones == 0 {
            return vec![first];
        }
        let mut result: Vec<u8> = vec![0; ones as usize + 1];
        result[0] = first;

        let num_slice = number.to_be_bytes();
        result[1..(ones + 1).into()].copy_from_slice(&num_slice[8 - ones as usize..8]);

        // 4. Return
        result
    }
}

/// Count the ones startings from the most significant bit, until it reaches a zero.
///
/// # Example
/// ```rust
/// let number = 0b1110_0101;
/// let ones = rosa::ones_before_zero(number);
/// assert_eq!(ones, 3);
/// ```
#[inline]
pub const fn ones_before_zero(byte: u8) -> u8 {
    let mut mask = 1 << 7;
    let mut ones = 0;

    while mask & byte != 0 {
        mask >>= 1;
        ones += 1;
    }

    ones
}

#[inline]
pub const fn bits_dyn_int(ones: u8) -> u8 {
    (8 - ones - 1) + 8 * ones
}

#[inline]
pub const fn size_dyn_int(ones: u8) -> u64 {
    2_u64.pow(bits_dyn_int(ones) as u32) - 1
}

pub fn ones_needed(number: u64) -> u8 {
    let mut ones: u8 = 0;
    let mut range = 0..=size_dyn_int(ones);
    while !range.contains(&number) {
        if ones > 7 {
            panic!("this number cannot fit into a dynamic integer");
        }
        ones += 1;
        range = *range.end()..=size_dyn_int(ones);
    }
    ones
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dyn_int_decode() {
        let dynint: &[u8] = &[0b1000_0001, 0b0000_1111];
        let decoded = DynamicInt::decode(dynint);
        assert_eq!(decoded, Some(0b0000_0001_0000_1111));
    }

    #[test]
    fn dyn_int_encode() {
        assert_eq!(DynamicInt::encode(127u16), vec![0b0111_1111]);
        assert_eq!(
            DynamicInt::encode(0b1010_1010_1010u16),
            vec![0b1000_1010, 0b1010_1010]
        );
    }

    #[test]
    fn dyn_int_size_correct() {
        assert_eq!(size_dyn_int(0), 127);
        assert_eq!(size_dyn_int(1), 16_383);
        assert_eq!(size_dyn_int(2), 2_097_151);
        assert_eq!(size_dyn_int(3), 268_435_455);
        assert_eq!(size_dyn_int(4), 34_359_738_367);
        assert_eq!(size_dyn_int(5), 4_398_046_511_103);
        assert_eq!(size_dyn_int(6), 562_949_953_421_311);
        assert_eq!(size_dyn_int(7), 72_057_594_037_927_935);
    }

    #[test]
    #[ignore = "It encodes and decodes all number starting from 0 up to 2,684,354. So too long."]
    fn dyn_int_symmetry() {
        for number in 0..size_dyn_int(3) / 100 {
            let encoded = DynamicInt::encode(number);
            let decoded = DynamicInt::decode(&encoded);
            assert_eq!(
                number,
                decoded.expect("failed to decode the dynamic integer")
            );
        }
    }
}
