use crate::data::tooltip::{SkillData, TooltipData};
use crate::utils::common::{is_available_skill_data, RESEARCHTIP, RESEARCHUBERTIP, TIP, UBERTIP};
use crate::utils::parse_field_line;
use std::collections::HashMap;
use std::fs;

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

fn parse_content(content: &str) -> HashMap<String, SkillData> {
    let mut entries = HashMap::new();
    let mut current_id = String::new();
    let mut current_data = SkillData::default();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Parse section header [XXXX]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if !current_id.is_empty() && is_available_skill_data(&current_data) {
                entries.insert(current_id.clone(), current_data);
                current_data = SkillData::default();
            }

            current_id = trimmed[1..trimmed.len() - 1].to_string();
            current_data.id = current_id.clone();
            continue;
        }

        // Parse tooltip fields
        if let Some((field_name, value)) = parse_field_line(trimmed) {
            if !is_available_field_name(field_name) {
                continue;
            }

            let field_value = if value.ends_with('{') {
                let next_line = lines.peek().map(|line| line.trim());
                if next_line.map_or(false, |line| line.starts_with("[=[")) {
                    parse_multi_line_string_array(&mut lines)
                } else {
                    parse_string_array(&mut lines)
                }
            } else if value.contains("[=[") {
                vec![parse_multi_line_string(&mut lines)]
            } else {
                vec![parse_single_line_string(value)]
            };

            match field_name {
                RESEARCHTIP => current_data.researchtip = field_value,
                RESEARCHUBERTIP => current_data.researchubertip = field_value,
                TIP => current_data.tip = field_value,
                UBERTIP => current_data.ubertip = field_value,
                _ => {}
            }
        }
    }

    if !current_id.is_empty() && is_available_skill_data(&current_data) {
        entries.insert(current_id.clone(), current_data);
    }

    entries
}

fn is_available_field_name(field_name: &str) -> bool {
    match field_name {
        RESEARCHTIP | RESEARCHUBERTIP | TIP | UBERTIP => true,
        _ => false,
    }
}

fn parse_single_line_string(content: &str) -> String {
    content.trim_matches('"').trim_matches('\'').to_string()
}

fn parse_multi_line_string<'a, I>(lines: &mut std::iter::Peekable<I>) -> String
where
    I: Iterator<Item = &'a str>,
{
    let mut content = String::new();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        // 檢查是否包含開始標記 [=[
        if trimmed.contains("[=[") {
            // 獲取 [=[ 後的內容
            if let Some(remaining) = trimmed.split("[=[").nth(1) {
                if !remaining.is_empty() {
                    content.push_str(remaining);
                    content.push('\n');
                }
            }
            continue;
        }

        // 檢查結束標記
        if trimmed.ends_with("]=]") {
            content.push_str(trimmed.trim_end_matches("]=]"));
            break;
        }

        content.push_str(line);
        content.push('\n');
    }

    content.trim().to_string()
}

fn parse_string_array<'a, I>(lines: &mut std::iter::Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut result = Vec::new();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed == "}" {
            break;
        }

        // 處理空字符串的情況
        if trimmed == "\"\","  || trimmed == "\"\"," {
            result.push(String::new());
            continue;
        }

        // 處理一般字符串
        if trimmed.starts_with('"') {
            let content = trimmed
                .trim_start_matches('"')
                .trim_end_matches("\",")
                .to_string();
            result.push(content);
        }
    }

    result
}

fn parse_multi_line_string_array<'a, I>(lines: &mut std::iter::Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut result = Vec::new();
    let mut current_string = String::new();
    let mut in_multiline = false;

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        // Handle end of table
        if trimmed == "}" {
            break;
        }

        // Start of a new multi-line string
        if trimmed.starts_with("[=[") {
            in_multiline = true;
            current_string.clear();
            // Capture any content after [=[ on the same line
            if let Some(content) = trimmed.strip_prefix("[=[") {
                if !content.is_empty() {
                    current_string.push_str(content);
                    current_string.push('\n');
                }
            }
            continue;
        }

        if in_multiline {
            // Check for end of multi-line string
            if trimmed.ends_with("]=],") || trimmed.ends_with("]=]") {
                let content = if trimmed.ends_with("]=],") {
                    trimmed.strip_suffix("]=],").unwrap()
                } else {
                    trimmed.strip_suffix("]=]").unwrap()
                };

                if !content.is_empty() {
                    current_string.push_str(content);
                }

                // Add the completed string to results
                result.push(current_string.clone());
                current_string.clear();
                in_multiline = false;
            } else {
                // Add the line with proper line ending
                current_string.push_str(line);
                current_string.push('\n');
            }
        }
    }

    result
}
