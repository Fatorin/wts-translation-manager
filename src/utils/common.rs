use crate::data::tooltip::SkillData;

pub const RESEARCHTIP: &str = "Researchtip";
pub const RESEARCHUBERTIP: &str = "Researchubertip";
pub const TIP: &str = "Tip";
pub const UBERTIP: &str = "Ubertip";
const PREFIXES: [&str; 4] = [RESEARCHTIP, RESEARCHUBERTIP, TIP, UBERTIP];

#[derive(Debug, PartialEq)]
pub enum ParseType {
    SingleLine,
    MultiLine,
    SingleLineArray,
    SingleLineArrayExt,
    MultiLineArray,
    MultiLineArrayExt,
}

pub fn get_field_name(line: &str) -> Option<&str> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() != 2 {
        return None;
    }

    let field_name = parts[0].trim();
    if PREFIXES
        .iter()
        .any(|&prefix| field_name.starts_with(prefix))
    {
        Some(field_name)
    } else {
        None
    }
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
