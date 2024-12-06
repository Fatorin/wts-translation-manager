use crate::translator::ObjectsTranslator;
use crate::types::{Modification, ObjectModificationTable, ObjectType};
use bstr::BString;
use indexmap::IndexMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct TooltipData {
    pub current_id: String,
    pub object_manager: ObjectManager,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct ObjectManager {
    source: IndexMap<BString, Vec<Modification>>,
    localized: IndexMap<BString, Vec<Modification>>,
    mapping: IndexMap<BString, BString>,
    pub object_type: ObjectType,
    original_count: i32,
}

impl ObjectManager {
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
            object_manager: ObjectManager::new(),
            path: PathBuf::default(),
        }
    }

    pub fn add_localized(&mut self, key: String) -> Result<(), String> {
        if self.object_manager.is_empty() {
            return Err(String::from("尚未讀取資料"));
        }

        if !self.object_manager.is_exist(&BString::from(key.as_str())) {
            return Err(format!("Object '{}' isn't exists", key));
        }

        Ok(self.object_manager.add_localized(&key)?)
    }

    pub fn import(&mut self, file_path: &std::path::Path) -> Result<(), String> {
        if !file_path.exists() {
            return Err(String::from("檔案不存在"));
        }

        self.path = file_path.to_path_buf();

        let source_bytes = fs::read(&file_path)
            .map_err(|e| format!("Failed to read file: {}, error: {}", file_path.display(), e))?;

        if let Some(ext) = ObjectType::from_path(file_path) {
            let table = ObjectsTranslator::war_to_json(&ext, source_bytes)
                .map_err(|e| format!("Failed to import war-to-json: {}", e))?;
            self.object_manager.import(table, None)?;
            self.object_manager.object_type = ext;
            if let Some(id) = self.object_manager.get_first_skill_id() {
                self.current_id = id.to_string();
            } else {
                return Err(String::from("找不到第一個ID，請檢查資料是否異常"));
            }

            Ok(())
        } else {
            Err(String::from("不支援的副檔名"))
        }
    }

    pub fn export(&self, is_localized: bool) -> Result<String, String> {
        if self.object_manager.is_empty() {
            return Err(String::from("尚未讀取資料"));
        }

        let table = self.object_manager.export(
            &self.object_manager.source,
            &self.object_manager.localized,
            is_localized,
        )?;

        let bytes = ObjectsTranslator::json_to_war(&self.object_manager.object_type, table)
            .map_err(|e| format!("Failed to import json-to-war: {}", e))?;

        let file_name = get_file_name(&self.path, &self.object_manager.object_type)?;

        fs::write(&file_name, bytes).map_err(|e| {
            format!(
                "Failed to write file: {}, error: {}",
                file_name.display(),
                e
            )
        })?;

        Ok(file_name.display().to_string())
    }
}

impl ObjectManager {
    pub fn new() -> Self {
        ObjectManager {
            source: IndexMap::new(),
            localized: IndexMap::new(),
            mapping: IndexMap::new(),
            object_type: ObjectType::Units,
            original_count: 0,
        }
    }

    fn import(
        &mut self,
        source: ObjectModificationTable,
        localized: Option<ObjectModificationTable>,
    ) -> Result<(), String> {
        self.original_count = source.original.len() as i32;

        for (key, data) in source.original {
            self.source.insert(key, data);
        }

        for (key, source_data) in source.custom {
            if let Some((original_key, custom_key)) = get_ids_from_custom_obj(&key) {
                self.mapping
                    .insert(custom_key.clone(), original_key.to_owned());
                self.source.insert(custom_key.to_owned(), source_data);
            } else {
                return Err(format!("not found custom id: {}", key));
            }
        }

        if let Some(localized_data) = localized {
            for (key, data) in localized_data.original {
                self.source.insert(key, data);
            }

            for (key, localized_data) in localized_data.custom {
                if let Some((_, custom_key)) = get_ids_from_custom_obj(&key) {
                    self.localized.insert(custom_key.to_owned(), localized_data);
                } else {
                    return Err(format!("not found custom id: {}", key));
                }
            }
        }

        Ok(())
    }

    fn export(
        &self,
        source: &IndexMap<BString, Vec<Modification>>,
        localized: &IndexMap<BString, Vec<Modification>>,
        is_localized: bool,
    ) -> Result<ObjectModificationTable, String> {
        let target_data = if is_localized { localized } else { source };

        if target_data.is_empty() {
            return Err(String::from("輸出資料為空，請確認是否有寫入資料。"));
        }

        let mut original = IndexMap::new();
        let mut custom = IndexMap::new();

        for (key, data) in target_data {
            let raw_key = if let Some(original_key) = self.mapping.get(key) {
                BString::from(format!("{}:{}", key, original_key))
            } else {
                key.clone()
            };

            let modifications = if !is_localized {
                localized.get(key).unwrap_or(data).to_owned()
            } else {
                data.to_owned()
            };

            if raw_key.contains(&b':') {
                custom.insert(raw_key, modifications);
            } else {
                original.insert(raw_key, modifications);
            }
        }

        Ok(ObjectModificationTable { original, custom })
    }

    fn add_localized(&mut self, id: &str) -> Result<(), String> {
        let skill_id = BString::from(id);
        let data = self.get_skill_from_id(&skill_id)?;
        self.localized.insert(skill_id, data);
        Ok(())
    }

    fn get_skill_from_id(&self, id: &BString) -> Result<Vec<Modification>, String> {
        if let Some(data) = self.source.get(id) {
            Ok(data.clone())
        } else {
            Err(format!("Skill '{}' not found", id))
        }
    }

    pub fn is_exist(&self, id: &BString) -> bool {
        self.source.contains_key(id)
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
        self.source.is_empty()
    }
}

fn get_file_name(path: &PathBuf, object_type: &ObjectType) -> Result<PathBuf, String> {
    let file_name = path
        .file_stem()
        .ok_or("無法解析檔案名稱")?
        .to_string_lossy();

    Ok(path.with_file_name(format!(
        "{}_new.{}",
        file_name,
        ObjectType::get_extension(object_type)
    )))
}

fn get_ids_from_custom_obj(source: &BString) -> Option<(BString, BString)> {
    if source.len() < 9 {
        return None;
    }

    let original_id = &BString::from(&source[5..9]);
    let custom_id = &BString::from(&source[0..4]);
    Some((original_id.to_owned(), custom_id.to_owned()))
}
