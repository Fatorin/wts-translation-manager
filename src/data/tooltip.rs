use std::collections::HashMap;

#[derive(Default)]
pub struct TooltipData {
    pub current_id: String,
    pub skill_manager: SkillManager,
}

#[derive(Default)]
pub struct SkillManager {
    pub skills: HashMap<String, SkillData>,
    pub translation_skills: HashMap<String, SkillData>,
}

#[derive(Default, Debug, Clone)]
pub struct SkillData {
    pub id: String,
    pub researchtip: Vec<String>,
    pub researchubertip: Vec<String>,
    pub tip: Vec<String>,
    pub ubertip: Vec<String>,
}

impl SkillManager {
    pub fn get_data_mut(&mut self, id: &str) -> (Option<&mut SkillData>, Option<&mut SkillData>) {
        (
            self.skills.get_mut(id),
            self.translation_skills.get_mut(id)
        )
    }

    pub fn get_skill_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.skills.keys().cloned().collect();
        ids.sort();
        ids
    }
}
