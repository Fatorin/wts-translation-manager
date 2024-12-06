use crate::translator::ObjectsTranslator;
use crate::types::{Modification, ObjectModificationTable, ObjectType};
use bstr::BString;
use indexmap::IndexMap;
use std::fs;

#[derive(Debug)]
pub struct TooltipData {
    pub current_id: String,
    pub skill_manager: SkillManager,
}

#[derive(Debug)]
pub struct SkillManager {
    pub source: IndexMap<BString, Vec<Modification>>,
    pub localized: IndexMap<BString, Vec<Modification>>,
    pub mapping: IndexMap<BString, BString>,
    pub original_count: i32,
}

impl SkillManager {
    pub fn get_first_skill_id(&self) -> Option<&BString> {
        if let Some(key) = self.source.keys().next() {
            Some(key)
        } else {
            None
        }
    }
}

impl TooltipData {
    pub fn new() -> Self {
        TooltipData {
            current_id: String::new(),
            skill_manager: SkillManager::new(),
        }
    }

    pub fn import(&mut self, file_path: &std::path::Path) -> Result<(), String> {
        if !file_path.exists() {
            return Err(String::from("檔案不存在"));
        }
        let source_bytes = fs::read(&file_path).map_err(|e| {
            format!(
                "Failed to read test file: {}, error: {}",
                file_path.display(),
                e
            )
        })?;

        if let Some(ext) = ObjectType::from_path(file_path) {
            let table = ObjectsTranslator::war_to_json(ext, source_bytes)
                .map_err(|e| format!("Failed to import war-to-json: {}", e))?;

            self.skill_manager.import(table.clone(), table.clone())?;
            let id = self.skill_manager.get_first_skill_id();
            if let Some(id) = id {
                self.current_id = id.to_string();
            } else {
                return Err(String::from("找不到第一個ID，請檢查資料是否異常"));
            }

            Ok(())
        } else {
            Err(String::from("不支援的副檔名"))
        }
    }
}

impl SkillManager {
    pub fn new() -> Self {
        SkillManager {
            source: IndexMap::new(),
            localized: IndexMap::new(),
            mapping: IndexMap::new(),
            original_count: 0,
        }
    }

    pub fn import(
        &mut self,
        table: ObjectModificationTable,
        localized: ObjectModificationTable,
    ) -> Result<(), String> {
        self.original_count = table.original.len() as i32;

        for (key, data) in table.original {
            self.source.insert(key, data);
        }

        for (key, data) in table.custom {
            if let Some((original_key, custom_key)) = get_ids_from_custom_obj(&key) {
                self.mapping
                    .insert(custom_key.clone(), original_key.to_owned());
                self.source.insert(custom_key.to_owned(), data);
            } else {
                return Err(format!("not found custom id: {}", key));
            }
        }

        Ok(())
    }

    pub fn get_data_mut(
        &mut self,
        id: &BString,
    ) -> (
        Option<&mut Vec<Modification>>,
        Option<&mut Vec<Modification>>,
    ) {
        (self.source.get_mut(id), self.localized.get_mut(id))
    }

    pub fn get_skill_ids(&self) -> Vec<BString> {
        let mut ids: Vec<BString> = self.source.keys().map(|k| k.to_owned()).collect();
        ids.sort();
        ids
    }

    pub fn is_empty(&self) -> bool {
        self.original_count == 0
    }
}

fn get_ids_from_custom_obj(source: &BString) -> Option<(BString, BString)> {
    if source.len() < 9 {
        return None;
    }

    let original_id = &BString::from(&source[5..9]);
    let custom_id = &BString::from(&source[0..4]);
    Some((original_id.to_owned(), custom_id.to_owned()))
}
