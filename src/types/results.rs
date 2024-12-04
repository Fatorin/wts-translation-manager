use std::fmt;

#[derive(Debug, Clone)]
pub struct TranslationError {
    pub message: String,
}

impl fmt::Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Translation error: {}", self.message)
    }
}

impl std::error::Error for TranslationError {}

impl TranslationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub type WarResult = Result<Vec<u8>, TranslationError>;
pub type JsonResult<T> = Result<T, TranslationError>;