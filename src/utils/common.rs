use crate::data::tooltip::SkillData;
use regex::Regex;
use std::iter::Peekable;

pub const SOURCE_FILE_NAME: &str = "source.ini";
pub const EXPORT_FILE_NAME: &str = "source_new.ini";
pub const TRANSLATE_FILE_NAME: &str = "translation.ini";
pub const PARSE_ID_REGEX: &str = r#"^\[([a-zA-Z0-9]{4}|[a-zA-Z0-9]{3}@)\]$"#;
pub const EXPORT_ID_REGEX: &str = r#"^([a-zA-Z0-9]{4})|([a-zA-Z0-9]{3}@)$"#;
pub const SINGLE_LINE_REGEX: &str = r#"^[A-Za-z]+\s*=\s*"(.*)"$"#;
pub const SINGLE_LINE_ARRAY_REGEX: &str = r#"^"(.*)",$"#;
pub const SINGLE_LINE_ARRAY_EXT_REGEX: &str = r#"^\d+\s*=\s*"(.*)",$"#;
pub const MULTI_LINE_ARRAY_EXT_REGEX: &str = r#"^\d+\s*=\s*\[=\[$"#;
pub const MULTI_LINE_NEWLINE_SYMBOL: &str = "]=],";
pub const NEWLINE_SYMBOL: char = '\n';

pub trait StringExt {
    fn push_line(&mut self, line: &str);
}

impl StringExt for String {
    fn push_line(&mut self, line: &str) {
        self.push_str(line);
        self.push('\n');
    }
}

#[derive(Default, Debug, Hash, Eq, PartialEq, Clone, Ord, PartialOrd)]
pub enum FieldType {
    #[default]
    Researchtip,
    Researchubertip,
    Tip,
    Ubertip,
}

impl FieldType {
    const RESEARCHTIP: &'static str = "Researchtip";
    const RESEARCHUBERTIP: &'static str = "Researchubertip";
    const TIP: &'static str = "Tip";
    const UBERTIP: &'static str = "Ubertip";

    pub fn from_str(s: &str) -> Option<FieldType> {
        match s {
            Self::RESEARCHTIP => Some(FieldType::Researchtip),
            Self::RESEARCHUBERTIP => Some(FieldType::Researchubertip),
            Self::TIP => Some(FieldType::Tip),
            Self::UBERTIP => Some(FieldType::Ubertip),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            FieldType::Researchtip => Self::RESEARCHTIP,
            FieldType::Researchubertip => Self::RESEARCHUBERTIP,
            FieldType::Tip => Self::TIP,
            FieldType::Ubertip => Self::UBERTIP,
        }
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
pub enum TextType {
    #[default]
    SingleLine,
    MultiLine,
    SingleLineArray,
    SingleLineArrayExt,
    MultiLineArray,
    MultiLineArrayExt,
}

pub fn get_id(key: &str) -> Option<String> {
    let pattern = Regex::new(PARSE_ID_REGEX).unwrap();
    if let Some(caps) = pattern.captures(key) {
        if caps.get(1).is_some() {
            return Some(caps.get(1).unwrap().as_str().to_string());
        }
        if caps.get(2).is_some() {
            return Some(caps.get(2).unwrap().as_str().to_string());
        }
    }
    None
}

pub fn get_field_type(line: &str) -> Option<FieldType> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() != 2 {
        return None;
    }

    let field_name = parts[0].trim();
    FieldType::from_str(field_name)
}

pub fn is_available_skill_data(data: &SkillData) -> bool {
    !data.id.is_empty()
        && [
            &data.researchtip,
            &data.researchubertip,
            &data.tip,
            &data.ubertip,
        ]
        .iter()
        .any(|desc| !desc.is_empty())
}

pub fn get_text_type<'a, I>(input: &str, lines: &mut Peekable<I>) -> Option<TextType>
where
    I: Iterator<Item = &'a str>,
{
    let multiple_pattern = Regex::new(r#"^.*\s*=\s*\{$"#).unwrap();
    match multiple_pattern.is_match(input) {
        // 對當前 lines 使用peek可以拿到下一行的文字
        true => lines
            .peek()
            .and_then(|next_line| get_parse_type_from_multi(next_line)),
        false => get_parse_type_from_single(input),
    }
}

fn get_parse_type_from_single(input: &str) -> Option<TextType> {
    // SingleLine
    let single_pattern = Regex::new(SINGLE_LINE_REGEX).unwrap();
    if single_pattern.is_match(input) {
        return Some(TextType::SingleLine);
    }

    // MultiLine
    let multi_pattern = Regex::new(r#"^[A-Za-z]+\s*=\s*\[=\[$"#).unwrap();
    if multi_pattern.is_match(input) {
        return Some(TextType::MultiLine);
    }

    None
}

fn get_parse_type_from_multi(input: &str) -> Option<TextType> {
    // 多列單行
    let pattern = Regex::new(SINGLE_LINE_ARRAY_REGEX).unwrap();
    if pattern.is_match(input) {
        return Some(TextType::SingleLineArray);
    }

    // 超多列單行
    let pattern = Regex::new(SINGLE_LINE_ARRAY_EXT_REGEX).unwrap();
    if pattern.is_match(input) {
        return Some(TextType::SingleLineArrayExt);
    }

    // 多列多行
    let pattern = Regex::new(r#"^\[=\[$"#).unwrap();
    if pattern.is_match(input) {
        return Some(TextType::MultiLineArray);
    }

    // 超多列多行
    let pattern = Regex::new(MULTI_LINE_ARRAY_EXT_REGEX).unwrap();
    if pattern.is_match(input) {
        return Some(TextType::MultiLineArrayExt);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_quotes_pattern() {
        // 有效的案例 - 不帶引號
        assert_eq!(get_id("[A123]"), Some("A1234".to_string()));
        assert_eq!(get_id("[ABCD]"), Some("ABCD".to_string()));
        assert_eq!(get_id("[A1B2]"), Some("A1B2".to_string()));
        assert_eq!(get_id("[a123]"), Some("a123".to_string()));
        assert_eq!(get_id("[1ABC]"), Some("1ABC".to_string()));

        // 無效的案例 - 不帶引號
        assert_eq!(get_id("[ABC]"), None);
        assert_eq!(get_id("[ABCDE]"), None);
        assert_eq!(get_id("[A@BC]"), None);
        assert_eq!(get_id("[ABC@]"), None);
    }

    #[test]
    fn test_quoted_pattern() {
        // 有效的案例 - 帶引號
        assert_eq!(get_id("[\"ABC@\"]"), Some("ABC@".to_string()));
        assert_eq!(get_id("[\"123@\"]"), Some("123@".to_string()));
        assert_eq!(get_id("[\"A2B@\"]"), Some("A2B@".to_string()));
        assert_eq!(get_id("[\"aBC@\"]"), Some("aBC@".to_string()));

        // 無效的案例 - 帶引號
        assert_eq!(get_id("[\"AB@\"]"), None);
        assert_eq!(get_id("[\"ABCD@\"]"), None);
        assert_eq!(get_id("[\"ABC#\"]"), None);
        assert_eq!(get_id("[\"ABC\"]"), None);
    }

    #[test]
    fn test_invalid_formats() {
        // 格式錯誤的案例
        assert_eq!(get_id("A123"), None);
        assert_eq!(get_id("[A123"), None);
        assert_eq!(get_id("A123]"), None);
        assert_eq!(get_id("[\"A123]"), None);
        assert_eq!(get_id("[A123\"]"), None);
        assert_eq!(get_id(""), None);
    }

    #[test]
    fn test_get_parse_type_from_single() {
        assert_eq!(
            get_parse_type_from_single(r#"Tip = "TEST|cffffcc00(Q)|r ""#),
            Some(TextType::SingleLine)
        );
        assert_eq!(
            get_parse_type_from_single(r#"123 = "TEST|cffffcc00(Q)|r ""#),
            None
        );
        assert_eq!(
            get_parse_type_from_single("Researchubertip = [=["),
            Some(TextType::MultiLine)
        );
        assert_eq!(
            get_parse_type_from_single("Tip = [=["),
            Some(TextType::MultiLine)
        );
        assert_eq!(get_parse_type_from_single("Tip = {"), None);
        assert_eq!(get_parse_type_from_single("123 = [=["), None);
    }

    #[test]
    fn test_get_parse_type_from_multi() {
        assert_eq!(
            get_parse_type_from_multi(r#""|c00ffff80TEST|cffffcc00(A)|r","#),
            Some(TextType::SingleLineArray)
        );

        assert_eq!(
            get_parse_type_from_multi(r#""|c00ffff80TEST|cffffcc00(A)|r""#),
            None
        );

        assert_eq!(
            get_parse_type_from_multi(r#""|c00ffff80TEST|cffffcc00(A)|r"#),
            None
        );

        assert_eq!(
            get_parse_type_from_multi(r#"99 = "some text","#),
            Some(TextType::SingleLineArrayExt)
        );
        assert_eq!(get_parse_type_from_multi(r#"9999 = "some text""#), None);

        assert_eq!(
            get_parse_type_from_multi(r#"1 = [=["#),
            Some(TextType::MultiLineArrayExt)
        );
    }
}
