use bstr::{BString, ByteSlice};

pub struct HexBuffer {
    buffer: Vec<u8>,
}

impl HexBuffer {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn trim_to_alignment(&mut self) {
        // 計算有效的長度 (4 位元組對齊)
        let valid_length = (self.buffer.len() + 3) & !3; // 向上對齊到 4 的倍數

        // 如果目前的長度大於有效長度，則裁剪
        if self.buffer.len() > valid_length {
            self.buffer.truncate(valid_length);
        }
        // 如果長度小於有效長度，則補充到 4 位元組對齊
        else if self.buffer.len() < valid_length {
            let padding = valid_length - self.buffer.len();
            self.add_zero_padding(padding);
        }
    }

    pub fn add_string(&mut self, s: &BString) {
        self.buffer.extend_from_slice(s.as_bytes());
        self.add_null_terminator();
    }

    pub fn add_new_line(&mut self) {
        self.buffer.extend_from_slice(&[0x0D, 0x0A]);
    }

    pub fn add_char(&mut self, c: char) {
        self.buffer.push(c as u8);
    }

    pub fn add_chars(&mut self, chars: &BString) {
        self.buffer.extend(chars.as_bytes());
    }

    pub fn add_int(&mut self, value: i32) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub fn add_short(&mut self, value: i16) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub fn add_float(&mut self, value: f32) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub fn add_byte(&mut self, value: u8) {
        self.buffer.push(value);
    }

    pub fn add_zero_padding(&mut self, count: usize) {
        for _ in 0..count {
            self.add_byte(0);
        }
    }

    pub fn add_null_terminator(&mut self) {
        self.buffer.push(0x00);
    }

    pub fn get_buffer(&self) -> Vec<u8> {
        self.buffer.clone()
    }
}
