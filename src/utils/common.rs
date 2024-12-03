use crate::data::tooltip::SkillData;

pub const RESEARCHTIP: &str = "Researchtip";
pub const RESEARCHUBERTIP: &str = "Researchubertip";
pub const TIP: &str = "Tip";
pub const UBERTIP: &str = "Ubertip";

pub fn is_available_skill_data(data: &SkillData) -> bool {
    !data.id.is_empty()
        && [
        &data.researchtip,
        &data.researchubertip,
        &data.tip,
        &data.ubertip,
    ].iter().all(|desc| !desc.is_empty())
}
