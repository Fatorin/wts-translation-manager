use std::fs;
use std::path::PathBuf;
use wts_translation_manager::utils::parser::parse_content;

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_file_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(filename);
        path
    }

    #[test]
    fn test_parser() {
        let file_path = get_test_file_path("source.ini");
        let content = fs::read_to_string(file_path).expect("Failed to read file");

        let map = parse_content(&content);
        assert!(!map.is_empty());
        assert!(map.contains_key("A0O9"),"Not found A0O9");
        assert!(map.contains_key("A011"),"Not found A011");
        assert!(map.contains_key("Az03"),"Not found Az03");
        assert!(!map.contains_key("A037"),"Should not found A037");
        assert!(map.contains_key("A038"),"Not found A038");
    }
}
