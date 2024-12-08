use crate::data::tooltip::{SkillData, TooltipData};
use crate::utils::common::{
    get_field_name, is_available_skill_data, ParseType, RESEARCHTIP, RESEARCHUBERTIP, TIP, UBERTIP,
};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::iter::Peekable;

const SINGLE_LINE_REGEX: &str = r#"^[A-Za-z]+\s*=\s*"(.*)"$"#;
const SINGLE_LINE_ARRAY_REGEX: &str = r#"^"(.*)",$"#;
const SINGLE_LINE_ARRAY_EXT_REGEX: &str = r#"^\d+\s*=\s*"(.*)",$"#;
const MULTI_LINE_ARRAY_EXT_REGEX: &str = r#"^\d+\s*=\s*\[=\[$"#;
const MULTI_LINE_NEWLINE_SYMBOL: &str = "]=],";
const NEWLINE_SYMBOL: char = '\n';

pub fn parse_tooltip_files(tooltip_filename: &str, translation_filename: &str) -> TooltipData {
    let mut data = TooltipData::default();

    // Parse main tooltip file
    let tooltip_content =
        fs::read_to_string(tooltip_filename).expect("Failed to read tooltip file");
    data.skill_manager.skills = parse_content(&tooltip_content);

    // Parse translation file
    let translation_content =
        fs::read_to_string(translation_filename).expect("Failed to read translation file");
    data.skill_manager.translation_skills = parse_content(&translation_content);

    // Set first skill ID as current if any exists
    if let Some(first_id) = data.skill_manager.skills.keys().next() {
        data.current_id = first_id.clone();
    }

    data
}

pub fn parse_content(content: &str) -> HashMap<String, SkillData> {
    let mut entries = HashMap::new();
    let mut current_id = String::new();
    let mut current_data = SkillData::default();
    let multiple_pattern = Regex::new(r#"^.*\s*=\s*\{$"#).unwrap();

    let mut lines = content.lines().peekable();
    while let Some(line) = lines.next() {
        if is_valid_id(line) {
            if is_available_skill_data(&current_data) {
                entries.insert(current_id, current_data);
                current_data = SkillData::default();
            }

            current_id = line.to_string();
            current_data.id = current_id.clone();
        }

        if let Some(field_name) = get_field_name(line) {
            let parse_type = match multiple_pattern.is_match(line) {
                true => get_parse_type_from_multi(lines.next().unwrap_or("")),
                false => get_parse_type_from_single(line),
            };

            if let Some(parse_type) = parse_type {
                let field_value = get_field_value(parse_type, &mut lines);
                match field_name {
                    RESEARCHTIP => current_data.researchtip = field_value,
                    RESEARCHUBERTIP => current_data.researchubertip = field_value,
                    TIP => current_data.tip = field_value,
                    UBERTIP => current_data.ubertip = field_value,
                    _ => {}
                }
            };
        }
    }

    if is_available_skill_data(&current_data) {
        entries.insert(current_id, current_data);
    }

    entries
}

fn is_valid_id(input: &str) -> bool {
    let pattern = Regex::new(r#"^\[(?:[a-zA-Z0-9]{4}|"[a-zA-Z0-9]{3}@")]$"#).unwrap();
    pattern.is_match(input)
}

fn get_parse_type_from_single(input: &str) -> Option<ParseType> {
    // SingleLine
    let single_pattern = Regex::new(SINGLE_LINE_REGEX).unwrap();
    if single_pattern.is_match(input) {
        return Some(ParseType::SingleLine);
    }

    // MultiLine
    let multi_pattern = Regex::new(r#"^[A-Za-z]+\s*=\s*\[=\[$"#).unwrap();
    if multi_pattern.is_match(input) {
        return Some(ParseType::MultiLine);
    }

    None
}
fn get_parse_type_from_multi(input: &str) -> Option<ParseType> {
    // 多列單行
    let pattern = Regex::new(SINGLE_LINE_ARRAY_REGEX).unwrap();
    if pattern.is_match(input) {
        return Some(ParseType::SingleLineArray);
    }

    // 超多列單行
    let pattern = Regex::new(SINGLE_LINE_ARRAY_EXT_REGEX).unwrap();
    if pattern.is_match(input) {
        return Some(ParseType::SingleLineArrayExt);
    }

    // 多列多行
    let pattern = Regex::new(r#"^\[=\[$"#).unwrap();
    if pattern.is_match(input) {
        return Some(ParseType::MultiLineArray);
    }

    // 超多列多行
    let pattern = Regex::new(MULTI_LINE_ARRAY_EXT_REGEX).unwrap();
    if pattern.is_match(input) {
        return Some(ParseType::MultiLineArrayExt);
    }

    None
}

fn get_field_value<'a, I>(parse_type: ParseType, lines: &mut Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    match parse_type {
        ParseType::SingleLine => handle_single_line(lines),
        ParseType::MultiLine => handle_multi_line(lines),
        ParseType::SingleLineArray => handle_single_line_array(lines),
        ParseType::SingleLineArrayExt => handle_single_line_array_ext(lines),
        ParseType::MultiLineArray => handle_multi_line_array(lines),
        ParseType::MultiLineArrayExt => handle_multi_line_array_ext(lines),
    }
}

fn handle_single_line<'a, I>(lines: &mut Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut result = vec![];
    if let Some(str) = lines.peek() {
        if let Some(value) = extract_value_from_regex(SINGLE_LINE_REGEX, str) {
            result.push(value);
        }
    }

    result
}

fn handle_multi_line<'a, I>(lines: &mut Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut result = String::new();
    while let Some(line) = lines.next() {
        if line.ends_with("]=]") {
            result.push_str(line.strip_suffix("]=]").unwrap_or(line));
            break;
        }

        result.push_str(line);
        result.push(NEWLINE_SYMBOL);
    }
    vec![result]
}

fn handle_single_line_array<'a, I>(lines: &mut Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut result = vec![];
    while let Some(line) = lines.next() {
        if line == "}" {
            break;
        }

        if let Some(trim_str) = line.strip_prefix("\"").and_then(|s| s.strip_suffix("\",")) {
            result.push(trim_str.to_string());
        }
    }

    result
}

fn handle_single_line_array_ext<'a, I>(lines: &mut Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut result = vec![];
    while let Some(line) = lines.next() {
        if line == "}" {
            break;
        }

        if let Some(value) = extract_value_from_regex(SINGLE_LINE_ARRAY_EXT_REGEX, line) {
            result.push(value);
        }
    }

    result
}

fn handle_multi_line_array<'a, I>(lines: &mut Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut result = vec![];
    let mut current_block = String::new();
    while let Some(line) = lines.next() {
        match line {
            "}" => {
                result.push(current_block.clone());
                break;
            }
            "[=[" => {
                if !current_block.is_empty() {
                    result.push(current_block.clone());
                    current_block.clear();
                }
            }
            _ => handle_newline(line, &mut current_block, MULTI_LINE_NEWLINE_SYMBOL),
        }
    }

    result
}

fn handle_multi_line_array_ext<'a, I>(lines: &mut Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    let pattern = Regex::new(MULTI_LINE_ARRAY_EXT_REGEX).unwrap();
    let mut result = vec![];
    let mut current_block = String::new();
    while let Some(line) = lines.next() {
        if line == "}" {
            result.push(current_block.clone());
            break;
        }

        if pattern.is_match(line) {
            if !current_block.is_empty() {
                result.push(current_block.clone());
                current_block.clear();
            }
            continue;
        }

        handle_newline(line, &mut current_block, MULTI_LINE_NEWLINE_SYMBOL);
    }

    result
}

fn extract_value_from_regex(pattern: &str, text: &str) -> Option<String> {
    let re = Regex::new(pattern).unwrap();
    re.captures(text)
        .and_then(|captures| captures.get(1))
        .map(|m| m.as_str().to_string())
}

fn handle_newline(line: &str, current_block: &mut String, end_marker: &str) {
    if let Some(trim_str) = line.strip_suffix(end_marker) {
        current_block.push_str(trim_str);
    } else {
        current_block.push_str(line);
        current_block.push(NEWLINE_SYMBOL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_file() {}

    #[test]
    fn test_no_quotes_pattern() {
        // 有效的案例 - 不帶引號
        assert!(is_valid_id("[A123]"), "應該接受 [A123]");
        assert!(is_valid_id("[1234]"), "應該接受 [1234]");
        assert!(is_valid_id("[ABCD]"), "應該接受 [ABCD]");
        assert!(is_valid_id("[A1B2]"), "應該接受 [A1B2]");
        assert!(is_valid_id("[a123]"), "應該接受小寫字母 [a123]");
        assert!(is_valid_id("[1ABC]"), "應該接受數字開頭 [1ABC]");

        // 無效的案例 - 不帶引號
        assert!(!is_valid_id("[ABC]"), "不應接受少於4個字符");
        assert!(!is_valid_id("[ABCDE]"), "不應接受超過4個字符");
        assert!(!is_valid_id("[A@BC]"), "不應接受特殊符號");
        assert!(!is_valid_id("[ABC@]"), "不應接受特殊符號");
    }

    #[test]
    fn test_quoted_pattern() {
        // 有效的案例 - 帶引號
        assert!(is_valid_id("[\"ABC@\"]"), "應該接受 [\"ABC@\"]");
        assert!(is_valid_id("[\"123@\"]"), "應該接受 [\"123@\"]");
        assert!(is_valid_id("[\"A2B@\"]"), "應該接受 [\"A2B@\"]");
        assert!(is_valid_id("[\"aBC@\"]"), "應該接受小寫字母 [\"aBC@\"]");

        // 無效的案例 - 帶引號
        assert!(!is_valid_id("[\"AB@\"]"), "不應接受少於3個字符加@");
        assert!(!is_valid_id("[\"ABCD@\"]"), "不應接受超過3個字符加@");
        assert!(!is_valid_id("[\"ABC#\"]"), "不應接受@以外的特殊符號");
        assert!(!is_valid_id("[\"ABC\"]"), "不應接受沒有@的字符串");
    }

    #[test]
    fn test_invalid_formats() {
        // 格式錯誤的案例
        assert!(!is_valid_id("A123"), "不應接受沒有方括號的字符串");
        assert!(!is_valid_id("[A123"), "不應接受缺少結束方括號的字符串");
        assert!(!is_valid_id("A123]"), "不應接受缺少開始方括號的字符串");
        assert!(!is_valid_id("[\"A123]"), "不應接受引號不匹配的字符串");
        assert!(!is_valid_id("[A123\"]"), "不應接受引號不匹配的字符串");
        assert!(!is_valid_id(""), "不應接受空字符串");
    }

    #[test]
    fn test_get_parse_type_from_single() {
        assert_eq!(
            get_parse_type_from_single(r#"Tip = "TEST|cffffcc00(Q)|r ""#),
            Some(ParseType::SingleLine)
        );
        assert_eq!(
            get_parse_type_from_single(r#"123 = "TEST|cffffcc00(Q)|r ""#),
            None
        );
        assert_eq!(
            get_parse_type_from_single("Researchubertip = [=["),
            Some(ParseType::MultiLine)
        );
        assert_eq!(
            get_parse_type_from_single("Tip = [=["),
            Some(ParseType::MultiLine)
        );
        assert_eq!(get_parse_type_from_single("Tip = {"), None);
        assert_eq!(get_parse_type_from_single("123 = [=["), None);
    }

    #[test]
    fn test_get_parse_type_from_multi() {
        assert_eq!(
            get_parse_type_from_multi(r#""|c00ffff80TEST|cffffcc00(A)|r","#),
            Some(ParseType::SingleLineArray)
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
            Some(ParseType::SingleLineArrayExt)
        );
        assert_eq!(get_parse_type_from_multi(r#"9999 = "some text""#), None);

        assert_eq!(
            get_parse_type_from_multi(r#"1 = [=["#),
            Some(ParseType::MultiLineArrayExt)
        );
    }

    #[test]
    fn test_handle_single_line() {
        let content = r#"Tip = "마력 전달과 흡수|cffffcc00(Q)|r ""#;
        let mut lines = content.lines().peekable();
        let result = handle_single_line(&mut lines);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 1);
        assert_eq!(result.first().unwrap(), "마력 전달과 흡수|cffffcc00(Q)|r ");
    }

    #[test]
    fn test_handle_multi_line() {
        let content = r#"Researchubertip = [=[
|c00ff8080
 ※레벨당 능력
|c0000ff80직선상 적에게 엄청난 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 1|r - 정면 1600범위에 1000의 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 2|r - 정면 1600범위에 1350의 데미지를 입히는 붉은 회오리를 방출합니다.]=]"#;

        let expected_content = r#"|c00ff8080
 ※레벨당 능력
|c0000ff80직선상 적에게 엄청난 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 1|r - 정면 1600범위에 1000의 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 2|r - 정면 1600범위에 1350의 데미지를 입히는 붉은 회오리를 방출합니다."#;

        let mut lines = content.lines().peekable();
        lines.next();
        let result = handle_multi_line(&mut lines);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 1);
        assert_eq!(result.first().unwrap(), expected_content);
    }

    #[test]
    fn test_handle_single_line_array() {
        let content = r#"Tip = {
"|c00ffff80무공|cffffcc00(A)|r",
"|c00ffff80무공|cffffcc00(A)|r",
}"#;
        let mut lines = content.lines().peekable();
        let result = handle_single_line_array(&mut lines);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "|c00ffff80무공|cffffcc00(A)|r");
        assert_eq!(result[1], "|c00ffff80무공|cffffcc00(A)|r");
    }

    #[test]
    fn test_handle_single_line_array_ext() {
        let content = r#"Tip = {
1 = "|c00ffff80천지를 가르는 개벽의 별 [Enuma Elish]|r(|cffffcc00R|r) - |r[|cffffcc00레벨 1|r]",
2 = "|c00ffff80천지를 가르는 개벽의 별 [Enuma Elish]|r(|cffffcc00R|r) - |r[|cffffcc00레벨 2|r]",
3 = "|c00ffff80천지를 가르는 개벽의 별 [Enuma Elish]|r(|cffffcc00R|r) - |r[|cffffcc00레벨 3|r]",
}"#;
        let mut lines = content.lines().peekable();
        let result = handle_single_line_array_ext(&mut lines);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "|c00ffff80천지를 가르는 개벽의 별 [Enuma Elish]|r(|cffffcc00R|r) - |r[|cffffcc00레벨 1|r]");
        assert_eq!(result[1], "|c00ffff80천지를 가르는 개벽의 별 [Enuma Elish]|r(|cffffcc00R|r) - |r[|cffffcc00레벨 2|r]");
        assert_eq!(result[2], "|c00ffff80천지를 가르는 개벽의 별 [Enuma Elish]|r(|cffffcc00R|r) - |r[|cffffcc00레벨 3|r]");
    }

    #[test]
    fn test_handle_multi_line_array() {
        let content = r#"Ubertip = {
[=[
|c0080ffff속성 습득 포인트: 12

|c009E0ADD스테이터스|r
습득조건으로 모든 속성 선행 습득 필요
올스텟 3 증가

|c009E0ADD절대쿨다운|r : 120초(힘/공투자 중 더 높은쪽 적용, 투자 당 1초씩 감소)]=],
[=[
|c0080ffff속성 습득 포인트: 8

|c009E0ADD노템모드 적용|r

|c009E0ADD절대쿨다운|r : 120초(힘/공투자 중 더 높은쪽 적용, 투자 당 1초씩 감소)
|c00ff8080노템모드 패널티|r : 반사데미지50% 감소,대마력증가량 5%감소]=],
}"#;

        let result1_content = r#"|c0080ffff속성 습득 포인트: 12

|c009E0ADD스테이터스|r
습득조건으로 모든 속성 선행 습득 필요
올스텟 3 증가

|c009E0ADD절대쿨다운|r : 120초(힘/공투자 중 더 높은쪽 적용, 투자 당 1초씩 감소)"#;

        let result2_content = r#"|c0080ffff속성 습득 포인트: 8

|c009E0ADD노템모드 적용|r

|c009E0ADD절대쿨다운|r : 120초(힘/공투자 중 더 높은쪽 적용, 투자 당 1초씩 감소)
|c00ff8080노템모드 패널티|r : 반사데미지50% 감소,대마력증가량 5%감소"#;

        let mut lines = content.lines().peekable();
        lines.next();
        let result = handle_multi_line_array(&mut lines);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], result1_content);
        assert_eq!(result[1], result2_content);
    }

    #[test]
    fn test_handle_multi_line_array_ext() {
        let content = r#"Ubertip = {
1 = [=[
|c009E0ADD스킬종류|r : 대계보구
|c009E0ADD데미지|r : 1000
|c009E0ADD직선범위|r : 1600
|c009E0ADD발동시간|r : 3초
|c009E0ADD대상|r : 범위 내의 적 전체.
|c009E0ADD쿨다운|r : 45초

괴리검 에아에 의한 공간 절단. 압축되어서 서로 마찰하는 풍압의 단층은, 의사적인 시공단층이 되어 적대하는 모든 것을 분쇄합니다.]=],
2 = [=[
|c009E0ADD스킬종류|r : 대계보구
|c009E0ADD데미지|r : 1350
|c009E0ADD직선범위|r : 1600
|c009E0ADD발동시간|r : 3초
|c009E0ADD대상|r : 범위 내의 적 전체.
|c009E0ADD쿨다운|r : 45초

괴리검 에아에 의한 공간 절단. 압축되어서 서로 마찰하는 풍압의 단층은, 의사적인 시공단층이 되어 적대하는 모든 것을 분쇄합니다.]=],
}"#;
        let result1_content = r#"|c009E0ADD스킬종류|r : 대계보구
|c009E0ADD데미지|r : 1000
|c009E0ADD직선범위|r : 1600
|c009E0ADD발동시간|r : 3초
|c009E0ADD대상|r : 범위 내의 적 전체.
|c009E0ADD쿨다운|r : 45초

괴리검 에아에 의한 공간 절단. 압축되어서 서로 마찰하는 풍압의 단층은, 의사적인 시공단층이 되어 적대하는 모든 것을 분쇄합니다."#;

        let result2_content = r#"|c009E0ADD스킬종류|r : 대계보구
|c009E0ADD데미지|r : 1350
|c009E0ADD직선범위|r : 1600
|c009E0ADD발동시간|r : 3초
|c009E0ADD대상|r : 범위 내의 적 전체.
|c009E0ADD쿨다운|r : 45초

괴리검 에아에 의한 공간 절단. 압축되어서 서로 마찰하는 풍압의 단층은, 의사적인 시공단층이 되어 적대하는 모든 것을 분쇄합니다."#;

        let mut lines = content.lines().peekable();
        lines.next();
        let result = handle_multi_line_array_ext(&mut lines);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], result1_content);
        assert_eq!(result[1], result2_content);
    }
}
