use std::fmt::Debug;

pub struct Memory {
    inner: Vec<u8>,
}

impl Memory {
    pub fn new(size_in_bytes: usize) -> Memory {
        let inner = vec![0; size_in_bytes];
        Memory { inner }
    }

    pub fn set_byte(&mut self, offset: usize, value: u8) {
        let buffer_len = self.inner.len();
        if offset >= buffer_len {
            panic!("set_byte: offset out of bound");
        }
        let slice = self.inner.as_mut_slice();
        slice[offset] = value;
    }

    pub fn get_byte(&mut self, offset: usize) -> u8 {
        let slice = self.inner.as_slice();
        slice[offset]
    }

    pub fn set_word(&mut self, offset: usize, value: u16) {
        let buffer_len = self.inner.len();

        if offset >= buffer_len - 1 {
            panic!(
                "Value won't fit at offset {}, because it will be out of bound {}",
                offset, buffer_len
            );
        }
        let be_bytes = value.to_be_bytes();
        let slice = self.inner.as_mut_slice();
        slice[offset] = be_bytes[0];
        slice[offset + 1] = be_bytes[1];
    }

    pub fn get_word(&self, offset: usize) -> u16 {
        let slice = self.inner.as_slice();
        u16::from_be_bytes([slice[offset], slice[offset + 1]])
    }
}

impl Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut regs = Vec::new();
        for register in &self.inner {
            regs.push(format!("{:#x}", register));
        }
        write!(f, "MEMORY: {:?}", regs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_word_size_operations() {
        use super::Memory;

        let offs = 8;
        let mut mem = Memory::new(10);
        mem.set_word(offs, 0x4243);

        let value = mem.get_word(offs);
        assert_eq!(value, 0x4243);

        let value = mem.get_byte(offs);
        assert_eq!(value, 0x42);

        let value = mem.get_byte(offs + 1);
        assert_eq!(value, 0x43);
    }

    #[test]
    fn test_byte_size_operations() {
        use super::Memory;

        let offs = 8;
        let mut mem = Memory::new(10);
        mem.set_byte(offs, 0x42);

        let value = mem.get_byte(offs);

        assert_eq!(value, 0x42);
    }

    #[test]
    fn to_index_test() {
        use super::to_index;

        assert_eq!(to_index(16), 2);
        assert_eq!(to_index(8), 1);
        assert_eq!(to_index(0), 0);
    }
}
