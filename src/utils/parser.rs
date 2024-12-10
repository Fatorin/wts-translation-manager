use crate::data::tooltip::{SkillData, TooltipData};
use crate::utils::common::*;
use eframe::egui::TextBuffer;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::iter::Peekable;

pub fn parse_tooltip_files() -> TooltipData {
    let mut data = TooltipData::default();

    // Parse source file
    let source_content = fs::read_to_string(SOURCE_FILE_NAME).expect("Failed to read source file");
    data.skill_manager.skills = parse_content(&source_content);

    // Parse translation file
    let translation_content =
        fs::read_to_string(TRANSLATE_FILE_NAME).expect("Failed to read translation file");
    data.skill_manager.translation_skills = parse_content(&translation_content);

    // Set first skill ID as current if any exists
    if let Some(first_id) = data.skill_manager.skills.keys().next() {
        data.current_id = first_id.clone();
    }

    data
}

pub fn parse_content(content: &str) -> BTreeMap<String, SkillData> {
    let mut entries = BTreeMap::new();
    let mut current_id = String::new();
    let mut current_data = SkillData::default();

    let mut lines = content.lines().peekable();
    while let Some(line) = lines.next() {
        if let Some(id) = get_id(line) {
            if is_available_skill_data(&current_data) {
                entries.insert(current_id, current_data);
                current_data = SkillData::default();
            }

            current_id = id;
            current_data.id = current_id.clone();
        }

        if let Some(field_type) = get_field_type(line) {
            if let Some(text_type) = get_text_type(line, &mut lines) {
                let field_value = match text_type {
                    TextType::SingleLine => handle_single_line(line),
                    TextType::MultiLine => handle_multi_line(&mut lines),
                    TextType::SingleLineArray => handle_single_line_array(&mut lines),
                    TextType::SingleLineArrayExt => handle_single_line_array_ext(&mut lines),
                    TextType::MultiLineArray => handle_multi_line_array(&mut lines),
                    TextType::MultiLineArrayExt => handle_multi_line_array_ext(&mut lines),
                };
                current_data.insert_data(text_type, field_type, field_value);
            };
        }
    }

    if is_available_skill_data(&current_data) {
        entries.insert(current_id, current_data);
    }

    entries
}

fn get_id(key: &str) -> Option<String> {
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

fn handle_single_line(str: &str) -> Vec<String> {
    let mut result = String::new();
    if let Some(v) = extract_value_from_regex(SINGLE_LINE_REGEX, str) {
        result.push_str(v.as_str());
    }
    vec![result]
}

fn handle_multi_line<'a, I>(lines: &mut Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut result = String::new();
    while let Some(line) = lines.next() {
        // if line.ends_with("[=[") {
        //     continue;
        // }

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
    lines
        .take_while(|line| line.as_str() != "}")
        .filter_map(|line| extract_value_from_regex(SINGLE_LINE_ARRAY_REGEX, &line))
        .collect()
}

fn handle_single_line_array_ext<'a, I>(lines: &mut Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    lines
        .take_while(|line| line.as_str() != "}")
        .filter_map(|line| extract_value_from_regex(SINGLE_LINE_ARRAY_EXT_REGEX, &line))
        .collect()
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
    fn test_handle_single_line() {
        let content = r#"Tip = "마력 전달과 흡수|cffffcc00(Q)|r ""#;
        let result = handle_single_line(&content);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 1);
        assert_eq!(result.first().unwrap(), "마력 전달과 흡수|cffffcc00(Q)|r ");
    }

    #[test]
    fn test_handle_multi_line() {
        let content = r#"|c00ff8080
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
