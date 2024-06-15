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
    pub fn get(&self, i: usize) -> u8 {
        self.try_get(i).unwrap()
    }

    pub fn try_get(&self, i: usize) -> Option<u8> {
        self.data.get(i).copied()
    }
}

// TODO: Make better error handling, with a custom RuntimError type etc..
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
    exit_code: Option<u8>,
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
    pub fn new(program: Chunk) -> VirtualMachine {
        VirtualMachine::with_stack_size(program, Self::DEFAULT_STACK_SIZE)
    }

    /// Creates a new virtual machine with the given program and the initial
    /// stack size.
    pub fn with_stack_size(program: Chunk, stack_size: usize) -> VirtualMachine {
        VirtualMachine {
            program,
            ip: 0,
            stack: vec![0; stack_size],
            sp: 0,
            exit_code: None,
        }
    }

    pub fn run(&mut self) -> u8 {
        while self.exit_code.is_none() {
            match inst::INSTRUCTION_SET.get(&self.read_byte()) {
                Some(inst) => {
                    dbg!(inst);
                    inst.execute(self);
                }
                None => todo!(),
            }
        }
        self.exit_code.unwrap()
    }

    /// Reads the byte pointed by `ip` and advance by one the `ip` pointer.
    pub fn read_byte(&mut self) -> u8 {
        let byte = self.program.get(self.ip);
        self.ip += 1;
        byte
    }

    pub fn stack_push(&mut self, data: &[u8]) {
        let size = data.len();
        if self.stack.len() < self.sp + size {
            // maybe not optimal to double the size?
            self.extend_stack(self.sp);
        }
        let stack_bite = &mut self.stack[self.sp..self.sp + size];
        stack_bite.copy_from_slice(data);
        self.sp += size;
    }

    pub fn stack_pop(&mut self, amount: impl Into<usize>) -> &[u8] {
        let amount = amount.into();
        let poped = &self.stack[self.sp - amount..self.sp];
        self.sp -= amount;
        poped
    }

    pub fn stack_pop_one(&mut self) -> u8 {
        *self.stack_pop(1usize).first().unwrap()
    }

    /// Extends the stack to contain `amount` more bytes of free space.
    pub fn extend_stack(&mut self, amount: usize) {
        self.stack.extend(vec![0; amount]);
    }
}

// TODO: Make it to hold, ref the data.
pub struct DynamicInt;

impl DynamicInt {
    pub fn decode(buf: &[u8]) -> Option<u64> {
        let first = *buf.first()?;
        let ones = ones_before_zero(first);
        if ones == 0 {
            return Some(first.into());
        }
        let mask = 2_u8.pow(ones.into()) - 1 << 8 - ones;

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
        // STEPS:
        // 1. Compute how many bytes are needed depending on the size of the number
        // 2. Encode that size as ones in the first byte.
        // 3. Encode all the remaining digits.
        // 4. Enjoy!
        todo!()
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
pub fn ones_before_zero(byte: u8) -> u8 {
    let mut mask = 1 << 7;
    let mut ones = 0;

    while mask & byte != 0 {
        mask >>= 1;
        ones += 1;
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
        let num = 127_u8;
        let encoded = DynamicInt::encode(num);
    }
}
