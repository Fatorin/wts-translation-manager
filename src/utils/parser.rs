use std::collections::HashMap;
use std::fs;
use crate::data::tooltip::{SkillData, TooltipData};
use crate::utils::parse_field_line;

pub fn parse_tooltip_files(tooltip_filename: &str, translation_filename: &str) -> TooltipData {
    let mut data = TooltipData::default();

    // Parse main tooltip file
    let tooltip_content = fs::read_to_string(tooltip_filename).expect("Failed to read tooltip file");
    data.skills = parse_content(&tooltip_content);

    // Parse translation file
    let translation_content = fs::read_to_string(translation_filename).expect("Failed to read translation file");
    data.translation_skills = parse_content(&translation_content);

    // Set first skill ID as current if any exists
    if let Some(first_id) = data.skills.keys().next() {
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

        if trimmed.is_empty() { continue; }

        // Parse section header [XXXX]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if !current_id.is_empty() {
                entries.insert(current_id.clone(), current_data);
                current_data = SkillData::default();
            }
            current_id = trimmed[1..trimmed.len()-1].to_string();
            current_data.id = current_id.clone();
            continue;
        }

        // Parse tooltip fields
        if let Some((field_name, value)) = parse_field_line(trimmed) {
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
                "Researchtip" => current_data.researchtip = field_value,
                "Researchubertip" => current_data.researchubertip = field_value,
                "Tip" => current_data.tip = field_value,
                "Ubertip" => current_data.ubertip = field_value,
                _ => {}
            }
        }
    }

    // Save the last entry
    if !current_id.is_empty() {
        entries.insert(current_id.clone(), current_data);
    }

    entries
}

fn parse_single_line_string(content: &str) -> String {
    content.trim_matches('"').trim_matches('\'').to_string()
}

fn parse_multi_line_string<'a, I>(lines: &mut std::iter::Peekable<I>) -> String
where I: Iterator<Item = &'a str> {
    let mut content = String::new();
    let mut first_line = true;

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if first_line {
            first_line = false;
            continue;
        }

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
where I: Iterator<Item = &'a str> {
    let mut result = Vec::new();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed == "}" {
            break;
        }
        if trimmed.starts_with('"') && trimmed.contains(',') {
            let content = trimmed
                .trim_start_matches('"')  // 移除開頭的引號
                .trim_end_matches("\",")  // 移除結尾的引號和逗號
                .to_string();
            result.push(content);
        }
    }

    result
}

fn parse_multi_line_string_array<'a, I>(lines: &mut std::iter::Peekable<I>) -> Vec<String>
where I: Iterator<Item = &'a str> {
    let mut result = Vec::new();
    let mut current_string = String::new();
    let mut in_multiline = false;

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if trimmed == "}" {
            break;
        }

        if trimmed.starts_with("[=[") {
            in_multiline = true;
            current_string.clear();
            continue;
        }

        if in_multiline {
            if trimmed.ends_with("]=],") || trimmed.ends_with("]=]") {
                current_string.push_str(trimmed.trim_end_matches("]=],").trim_end_matches("]=]"));
                result.push(current_string.trim().to_string());
                current_string.clear();
                in_multiline = false;
            } else {
                current_string.push_str(line);
                current_string.push('\n');
            }
        }
    }

    result
}