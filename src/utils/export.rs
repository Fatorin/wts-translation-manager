use crate::data::tooltip::TooltipData;
use crate::utils::common::{is_available_skill_data, RESEARCHTIP, RESEARCHUBERTIP, TIP, UBERTIP};
use crate::utils::parse_field_line;
use std::fs;

const OUTPUT_FILE_NAME: &str = "output.ini";

struct ContentProcessor<'a> {
    tooltip_data: &'a TooltipData,
    current_id: String,
    current_field: String,
    result: String,
}

impl<'a> ContentProcessor<'a> {
    pub fn process_field(
        &mut self,
        field_name: &str,
        value: &str,
        line: &str,
        lines: &mut std::iter::Peekable<std::str::Lines>,
    ) {
        if value.contains("[=[") {
            self.process_multiline_string(field_name, lines);
        } else if value.ends_with('{') {
            self.process_array(field_name, lines);
        } else {
            self.process_single_line(field_name, value, line);
        }
    }

    fn new(tooltip_data: &'a TooltipData) -> Self {
        Self {
            tooltip_data,
            current_id: String::new(),
            current_field: String::new(),
            result: String::new(),
        }
    }

    fn write_multiline_content(&mut self, field_name: &str, content: &str) {
        self.result.push_str(field_name);
        self.result.push_str(" = [=[\n");
        self.result.push_str(content);
        self.result.push_str("]=]\n");
    }

    fn process_multiline_string(
        &mut self,
        field_name: &str,
        lines: &mut std::iter::Peekable<std::str::Lines>,
    ) {
        let mut original_content = Vec::new();

        // 收集所有內容直到遇到 ]=]
        while let Some(line) = lines.next() {
            if line.trim().ends_with("]=]") {
                // 保存最後一行，但移除 ]=]
                let last_line = line.trim_end_matches("]=]");
                if !last_line.is_empty() {
                    original_content.push(last_line.to_string());
                }
                break;
            }
            original_content.push(line.to_string());
        }

        let content = if let Some(translation) =
            get_translation(self.tooltip_data, &self.current_id, field_name, 0)
        {
            if !translation.trim().is_empty() {
                translation
            } else {
                original_content.join("\n")
            }
        } else {
            original_content.join("\n")
        };

        self.write_multiline_content(field_name, &content);
    }

    fn process_array(
        &mut self,
        field_name: &str,
        lines: &mut std::iter::Peekable<std::str::Lines>,
    ) {
        self.result.push_str(field_name);
        self.result.push_str(" = {\n");

        // 檢查翻譯內容
        let translated_content = if let Some(translation) = self
            .tooltip_data
            .skill_manager
            .translation_skills
            .get(&self.current_id)
        {
            match field_name {
                RESEARCHTIP => &translation.researchtip,
                RESEARCHUBERTIP => &translation.researchubertip,
                TIP => &translation.tip,
                UBERTIP => &translation.ubertip,
                _ => &vec![],
            }
        } else {
            &vec![]
        };

        let mut line_count = 0;

        while let Some(next_line) = lines.peek() {
            let next_trimmed = next_line.trim();

            if next_trimmed == "}" {
                break;
            }
            lines.next(); // Consume the line

            if next_trimmed.starts_with("[=[") {
                // 處理多行字串
                let mut content = String::new();
                while let Some(line) = lines.next() {
                    if line.trim().ends_with("]=],") || line.trim().ends_with("]=]") {
                        break;
                    }
                    content.push_str(line);
                    content.push('\n');
                }
                if let Some(trans) = translated_content.get(line_count) {
                    self.result.push_str("[=[\n");
                    self.result.push_str(trans);
                    self.result.push_str("]=],\n");
                } else {
                    self.result.push_str("[=[\n");
                    self.result.push_str(&content);
                    self.result.push_str("]=],\n");
                }
                line_count += 1;
            } else if next_trimmed.starts_with('"') {
                // 處理單行字串
                if let Some(trans) = translated_content.get(line_count) {
                    self.result.push_str(&format!("\"{}\",\n", trans));
                } else {
                    self.result.push_str(&format!("{}\n", next_trimmed));
                }
                line_count += 1;
            }
        }

        lines.next(); // Consume the closing brace
        self.result.push_str("}\n");
    }

    fn process_single_line(&mut self, field_name: &str, value: &str, line: &str) {
        if let Some(translation) =
            get_translation(self.tooltip_data, &self.current_id, field_name, 0)
        {
            if !translation.trim().is_empty() {
                self.result.push_str(field_name);
                self.result.push_str(" = ");
                if value.starts_with('"') {
                    self.result.push_str(&format!("\"{}\"", translation));
                } else {
                    self.result.push_str(&translation);
                }
                self.result.push('\n');
            } else {
                self.result.push_str(line);
                self.result.push('\n');
            }
        } else {
            self.result.push_str(line);
            self.result.push('\n');
        }
    }
}

pub fn export_translated_content(
    tooltip_data: &TooltipData,
    original_file: &str,
) -> Result<(), std::io::Error> {
    let content = fs::read_to_string(original_file)?;
    let mut processor = ContentProcessor::new(tooltip_data);
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            processor.result.push('\n');
            continue;
        }

        // Handle section headers [XXXX]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            processor.current_id = trimmed[1..trimmed.len() - 1].to_string();
            processor.result.push_str(line);
            processor.result.push('\n');
            continue;
        }

        // Handle field lines
        if let Some((field_name, value)) = parse_field_line(trimmed) {
            match field_name {
                RESEARCHTIP | RESEARCHUBERTIP | TIP | UBERTIP => {
                    processor.current_field = field_name.to_string();
                    processor.process_field(field_name, value, line, &mut lines);
                }
                _ => {
                    processor.result.push_str(line);
                    processor.result.push('\n');
                }
            }
            continue;
        }

        processor.result.push_str(line);
        processor.result.push('\n');
    }

    fs::write(OUTPUT_FILE_NAME, processor.result)?;
    Ok(())
}

pub fn save_translation(tooltip_data: &TooltipData, filename: &str) -> Result<(), std::io::Error> {
    let mut content = String::new();

    for (id, skill) in &tooltip_data.skill_manager.translation_skills {
        if !is_available_skill_data(skill) {
            continue;
        }

        content.push_str(&format!("[{}]\n", id));
        content.push_str(&format_tip("Researchtip", &skill.researchtip));
        content.push_str(&format_tip("Researchubertip", &skill.researchubertip));
        content.push_str(&format_tip("Tip", &skill.tip));
        content.push_str(&format_tip("Ubertip", &skill.ubertip));

        content.push('\n');
    }

    fs::write(filename, content)?;
    Ok(())
}

fn format_tip(name: &str, tips: &[String]) -> String {
    if tips.is_empty() {
        return String::new();
    }

    let contains_multiline = tips.iter().any(|tip| tip.contains('\n'));

    if tips.len() == 1 {
        if contains_multiline {
            format!("{} = [=[\n{}]=]\n", name, tips[0])
        } else {
            format!("{} = \"{}\"\n", name, tips[0])
        }
    } else {
        let mut result = format!("{} = {{\n", name);
        for tip in tips {
            if contains_multiline {
                result.push_str("[=[\n");
                result.push_str(tip);
                result.push_str("]=],\n");
            } else {
                result.push_str(&format!("\"{}\",\n", tip));
            }
        }
        result.push_str("}\n");
        result
    }
}

// Helper functions remain unchanged
fn get_translation(
    tooltip_data: &TooltipData,
    id: &str,
    field: &str,
    index: usize,
) -> Option<String> {
    if let Some(translation) = tooltip_data.skill_manager.translation_skills.get(id) {
        let vec = match field {
            RESEARCHTIP => &translation.researchtip,
            RESEARCHUBERTIP => &translation.researchubertip,
            TIP => &translation.tip,
            UBERTIP => &translation.ubertip,
            _ => return None,
        };
        vec.get(index).cloned()
    } else {
        None
    }
}
