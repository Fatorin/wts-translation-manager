use std::fmt::{self, Display};
use std::fmt::Write;

#[derive(Debug)]
pub enum CompareError {
    SizeMismatch {
        source_size: usize,
        output_size: usize,
        extra_bytes: String,
    },
    ContentMismatch {
        position: usize,
        source_remaining: usize,
        output_remaining: usize,
    }
}

impl Display for CompareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompareError::SizeMismatch { source_size, output_size, extra_bytes } => {
                write!(f, "Size mismatch: source({}) vs output({}), Extra bytes: {}",
                       source_size, output_size, extra_bytes)
            },
            CompareError::ContentMismatch { position, source_remaining, output_remaining } => {
                write!(f, "Content mismatch at 0x{:08X}, Remaining: source({}) output({})",
                       position, source_remaining, output_remaining)
            }
        }
    }
}

pub fn compare_files(source: &[u8], output: &[u8]) -> Result<(), CompareError> {
    // 找出實際有效長度（去除結尾的 00）
    let effective_source_len = source.iter().rposition(|&x| x != 0).map_or(0, |i| i + 1);
    let effective_output_len = output.iter().rposition(|&x| x != 0).map_or(0, |i| i + 1);

    // 使用最長的有效長度來比較
    let compare_len = std::cmp::max(effective_source_len, effective_output_len);

    // 如果有效內容長度不同，才報錯
    if effective_source_len != effective_output_len {
        let mut extra_bytes = String::new();
        if effective_source_len > effective_output_len {
            let extra = &source[effective_output_len..effective_source_len];
            let display_len = std::cmp::min(16, extra.len());
            for i in 0..display_len {
                write!(extra_bytes, "{:02X}", extra[i]).unwrap();
                if i < display_len - 1 {
                    write!(extra_bytes, "-").unwrap();
                }
            }
        } else {
            let extra = &output[effective_source_len..effective_output_len];
            let display_len = std::cmp::min(16, extra.len());
            for i in 0..display_len {
                write!(extra_bytes, "{:02X}", extra[i]).unwrap();
                if i < display_len - 1 {
                    write!(extra_bytes, "-").unwrap();
                }
            }
        }

        return Err(CompareError::SizeMismatch {
            source_size: effective_source_len,
            output_size: effective_output_len,
            extra_bytes,
        });
    }

    // 檢查有效內容
    for i in 0..compare_len {
        if source[i] != output[i] {
            return Err(CompareError::ContentMismatch {
                position: i,
                source_remaining: effective_source_len - i,
                output_remaining: effective_output_len - i,
            });
        }
    }

    Ok(())
}