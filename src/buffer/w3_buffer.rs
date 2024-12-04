use bstr::BString;

pub struct W3Buffer {
    offset: usize,
    buffer: Vec<u8>,
}

impl W3Buffer {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self { offset: 0, buffer }
    }

    pub fn read_int(&mut self) -> i32 {
        let value = i32::from_le_bytes(
            self.buffer[self.offset..self.offset + 4]
                .try_into()
                .unwrap(),
        );
        self.offset += 4;
        value
    }

    pub fn read_short(&mut self) -> i16 {
        let value = i16::from_le_bytes(
            self.buffer[self.offset..self.offset + 2]
                .try_into()
                .unwrap(),
        );
        self.offset += 2;
        value
    }

    pub fn read_float(&mut self) -> f32 {
        let value = f32::from_le_bytes(
            self.buffer[self.offset..self.offset + 4]
                .try_into()
                .unwrap(),
        );
        self.offset += 4;
        value
    }

    pub fn read_string(&mut self) -> BString {
        let start_offset = self.offset;

        while self.offset < self.buffer.len() && self.buffer[self.offset] != 0x00 {
            self.offset += 1;
        }

        let bytes = &self.buffer[start_offset..self.offset];
        self.offset += 1;
        BString::from(bytes)
    }

    pub fn read_chars(&mut self, len: usize) -> BString {
        let start_offset = self.offset;

        for _ in 0..len {
            if self.offset >= self.buffer.len() {
                break;
            }
            self.offset += 1;
        }

        let bytes = &self.buffer[start_offset..self.offset];
        BString::from(bytes)
    }

    pub fn read_byte(&mut self) -> u8 {
        if self.offset >= self.buffer.len() {
            panic!("Attempted to read past end of buffer");
        }
        let byte = self.buffer[self.offset];
        self.offset += 1;
        byte
    }

    pub fn is_exhausted(&self) -> bool {
        self.offset >= self.buffer.len()
    }
}
