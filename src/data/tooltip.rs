use crate::ui::components;
use eframe::egui;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref EMPTY_SKILL: SkillData = SkillData::default();
}

#[derive(Default)]
pub struct TooltipData {
    pub current_id: String,
    pub skills: HashMap<String, SkillData>,
    pub translation_skills: HashMap<String, SkillData>,
}

#[derive(Default, Clone)]
pub struct SkillData {
    pub id: String,
    pub researchtip: Vec<String>,
    pub researchubertip: Vec<String>,
    pub tip: Vec<String>,
    pub ubertip: Vec<String>,
}

impl TooltipData {
    pub fn show_section(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        content: impl FnOnce(&mut egui::Ui),
    ) {
        components::show_section(ui, title, content);
    }

    pub fn show_text_list(&self, ui: &mut egui::Ui, items: &[String], is_interactive: bool) {
        components::show_text_list(ui, items, is_interactive);
    }

    pub fn get_current_skill(&self) -> &SkillData {
        self.skills.get(&self.current_id).unwrap_or(&EMPTY_SKILL)
    }

    pub fn try_get_current_translation_skill(&self) -> &SkillData {
        self.translation_skills
            .get(&self.current_id)
            .unwrap_or(&EMPTY_SKILL)
    }

    pub fn get_skill_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.skills.keys().cloned().collect();
        ids.sort();
        ids
    }
}
