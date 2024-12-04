mod common;
use std::fs;
use std::path::PathBuf;
use wts_translation_manager::{translator::ObjectsTranslator, types::ObjectType};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::compare_files;

    fn get_test_file_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("test_files");
        path.push(filename);
        path
    }

    #[test]
    fn test_abilities_conversion() {
        let file_path = get_test_file_path("war3map.w3a");
        let source_bytes = fs::read(&file_path).unwrap_or_else(|e| {
            panic!(
                "Failed to read test file: {}, error: {}",
                file_path.display(),
                e
            )
        });

        let table = ObjectsTranslator::war_to_json(ObjectType::Abilities, source_bytes.clone())
            .expect("Failed to convert WAR to JSON");

        let output_bytes = ObjectsTranslator::json_to_war(ObjectType::Abilities, table)
            .expect("Failed to convert JSON to WAR");

        match compare_files(&source_bytes, &output_bytes) {
            Ok(_) => (),
            Err(e) => panic!("Files are different: {:?}", e),
        }
    }

    #[test]
    fn test_multiple_files() {
        // 定義要測試的檔案類型
        let test_cases = vec![
            ("test.w3a", ObjectType::Abilities),
            ("test.w3u", ObjectType::Units),
            ("test.w3t", ObjectType::Items),
            // 可以添加更多測試案例
        ];

        for (filename, object_type) in test_cases {
            let file_path = get_test_file_path(filename);

            // 跳過不存在的檔案
            if !file_path.exists() {
                println!("Skipping non-existent file: {}", file_path.display());
                continue;
            }

            let source_bytes = fs::read(&file_path).unwrap_or_else(|e| {
                panic!("Failed to read file: {}, error: {}", file_path.display(), e)
            });

            let table = ObjectsTranslator::war_to_json(object_type.clone(), source_bytes.clone())
                .unwrap_or_else(|e| {
                    panic!("Failed to convert WAR to JSON for {}: {}", filename, e)
                });

            let output_bytes =
                ObjectsTranslator::json_to_war(object_type, table).unwrap_or_else(|e| {
                    panic!("Failed to convert JSON to WAR for {}: {}", filename, e)
                });

            match compare_files(&source_bytes, &output_bytes) {
                Ok(_) => (),
                Err(e) => panic!("Files are different: {:?}", e),
            }
        }
    }
}
