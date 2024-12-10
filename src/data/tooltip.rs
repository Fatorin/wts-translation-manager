use crate::utils::common::*;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct TooltipData {
    pub current_id: String,
    pub skill_manager: SkillManager,
}

#[derive(Default)]
pub struct SkillManager {
    pub skills: BTreeMap<String, SkillData>,
    pub translation_skills: BTreeMap<String, SkillData>,
}

impl SkillManager {
    pub fn get_data_mut(&mut self, id: &str) -> (Option<&mut SkillData>, Option<&mut SkillData>) {
        (self.skills.get_mut(id), self.translation_skills.get_mut(id))
    }

    pub fn get_skill_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.skills.keys().cloned().collect();
        ids.sort();
        ids
    }
}

#[derive(Default, Debug, Clone)]
pub struct SkillData {
    pub id: String,
    pub researchtip: Vec<String>,
    pub researchubertip: Vec<String>,
    pub tip: Vec<String>,
    pub ubertip: Vec<String>,
    pub text_type_map: BTreeMap<FieldType, TextType>,
}

impl SkillData {
    pub fn insert_data(
        &mut self,
        text_type: TextType,
        field_type: FieldType,
        field_value: Vec<String>,
    ) {
        match field_type {
            FieldType::Researchtip => {
                self.researchtip = field_value;
            }
            FieldType::Researchubertip => {
                self.researchubertip = field_value;
            }
            FieldType::Tip => {
                self.tip = field_value;
            }
            FieldType::Ubertip => {
                self.ubertip = field_value;
            }
        }

        self.text_type_map.insert(field_type, text_type);
    }
}
