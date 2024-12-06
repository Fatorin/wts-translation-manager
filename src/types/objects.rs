use bstr::BString;
use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TableType {
    Original,
    Custom,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModificationType {
    Int,
    Real,
    Unreal,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    Units,
    Items,
    Destructables,
    Doodads,
    Abilities,
    Buffs,
    Upgrades,
}

impl ObjectType {
    pub fn all_extensions() -> Vec<&'static str> {
        vec!["w3u", "w3t", "w3b", "w3d", "w3a", "w3h", "w3q"]
    }

    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "w3u" => Some(ObjectType::Units),
            "w3t" => Some(ObjectType::Items),
            "w3b" => Some(ObjectType::Destructables),
            "w3d" => Some(ObjectType::Doodads),
            "w3a" => Some(ObjectType::Abilities),
            "w3h" => Some(ObjectType::Buffs),
            "w3q" => Some(ObjectType::Upgrades),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ModificationValue {
    Int(i32),
    Real(f32),
    Unreal(f32),
    String(BString),
}

#[derive(Debug, Clone)]
pub struct Modification {
    pub id: BString,
    pub kind: ModificationType,
    pub value: ModificationValue,
    pub level: i32,
    pub column: i32,
    pub variation: i32,
}

#[derive(Debug, Clone)]
pub struct ObjectModificationTable {
    pub original: IndexMap<BString, Vec<Modification>>,
    pub custom: IndexMap<BString, Vec<Modification>>,
}
