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
pub enum FileTypeExtension {
    Units,          // w3u
    Items,          // w3t
    Destructables,  // w3b
    Doodads,       // w3d
    Abilities,      // w3a
    Buffs,         // w3h
    Upgrades,      // w3q
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