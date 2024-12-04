use crate::buffer::{HexBuffer, W3Buffer};
use crate::types::{
    JsonResult, Modification, ModificationType, ModificationValue, ObjectModificationTable,
    ObjectType, TableType, TranslationError, WarResult,
};
use bstr::BString;
use indexmap::IndexMap;

pub struct ObjectsTranslator;

impl ObjectsTranslator {
    fn type_to_int(mod_type: &ModificationType) -> i32 {
        match mod_type {
            ModificationType::Int => 0,
            ModificationType::Real => 1,
            ModificationType::Unreal => 2,
            ModificationType::String => 3,
        }
    }

    fn int_to_type(value: i32) -> Result<ModificationType, TranslationError> {
        match value {
            0 => Ok(ModificationType::Int),
            1 => Ok(ModificationType::Real),
            2 => Ok(ModificationType::Unreal),
            3 => Ok(ModificationType::String),
            invalid => Err(TranslationError::new(format!(
                "Invalid modification type value: {}",
                invalid
            ))),
        }
    }

    pub fn json_to_war(object_type: ObjectType, json: ObjectModificationTable) -> WarResult {
        let mut buffer = HexBuffer::new();

        // File version
        buffer.add_int(2);

        buffer.add_int(json.original.len() as i32);
        Self::generate_table_from_json(
            &mut buffer,
            TableType::Original,
            &json.original,
            &object_type,
        );

        buffer.add_int(json.custom.len() as i32);
        Self::generate_table_from_json(&mut buffer, TableType::Custom, &json.custom, &object_type);

        Ok(buffer.get_buffer())
    }

    fn generate_table_from_json(
        buffer: &mut HexBuffer,
        table_type: TableType,
        table_data: &IndexMap<BString, Vec<Modification>>,
        object_type: &ObjectType,
    ) {
        for (def_key, obj) in table_data {
            match table_type {
                TableType::Original => {
                    buffer.add_chars(def_key);
                    buffer.add_zero_padding(4);
                }
                TableType::Custom => {
                    buffer.add_chars(&BString::from(&def_key[5..9])); // original id
                    buffer.add_chars(&BString::from(&def_key[0..4])); // custom id
                }
            }

            buffer.add_int(obj.len() as i32);

            for mod_item in obj.iter() {
                buffer.add_chars(&mod_item.id);
                buffer.add_int(ObjectsTranslator::type_to_int(&mod_item.kind));

                if matches!(
                    object_type,
                    ObjectType::Doodads | ObjectType::Abilities | ObjectType::Upgrades
                ) {
                    buffer.add_int(if mod_item.level != 0 {
                        mod_item.level
                    } else {
                        mod_item.variation
                    });
                    buffer.add_int(mod_item.column);
                }

                match &mod_item.value {
                    ModificationValue::Int(v) => buffer.add_int(*v),
                    ModificationValue::Real(v) | ModificationValue::Unreal(v) => {
                        buffer.add_float(*v)
                    }
                    ModificationValue::String(v) => {
                        buffer.add_string(v);
                    }
                }

                buffer.add_zero_padding(4);
            }
        }
    }

    pub fn war_to_json(
        object_type: ObjectType,
        buffer: Vec<u8>,
    ) -> JsonResult<ObjectModificationTable> {
        let mut result = ObjectModificationTable {
            original: IndexMap::new(),
            custom: IndexMap::new(),
        };

        let mut reader = W3Buffer::new(buffer);
        let _file_version = reader.read_int();

        Self::read_modification_table(&mut reader, true, &mut result, &object_type)?;
        Self::read_modification_table(&mut reader, false, &mut result, &object_type)?;

        Ok(result)
    }

    fn read_modification_table(
        reader: &mut W3Buffer,
        is_original_table: bool,
        result: &mut ObjectModificationTable,
        object_type: &ObjectType,
    ) -> Result<(), TranslationError> {
        let num_table_modifications = reader.read_int();

        for _ in 0..num_table_modifications {
            let mut object_definition = Vec::new();

            let original_id = reader.read_chars(4);
            let custom_id = reader.read_chars(4);
            let modification_count = reader.read_int();

            for _ in 0..modification_count {
                let id = reader.read_chars(4);
                let kind = ObjectsTranslator::int_to_type(reader.read_int())?;

                let (level, column) = if matches!(
                    object_type,
                    ObjectType::Doodads | ObjectType::Abilities | ObjectType::Upgrades
                ) {
                    (reader.read_int(), reader.read_int())
                } else {
                    (0, 0)
                };

                let value = match kind {
                    ModificationType::Int => ModificationValue::Int(reader.read_int()),
                    ModificationType::Real | ModificationType::Unreal => {
                        ModificationValue::Real(reader.read_float())
                    }
                    ModificationType::String => ModificationValue::String(reader.read_string()),
                };

                if is_original_table {
                    reader.read_int();
                } else {
                    reader.read_chars(4);
                }

                let modification = Modification {
                    id,
                    kind,
                    value,
                    level,
                    column,
                    variation: 0,
                };

                object_definition.push(modification);
            }

            if is_original_table {
                result.original.insert(original_id, object_definition);
            } else {
                result
                    .custom
                    .insert(BString::from(format!("{}:{}", custom_id, original_id)), object_definition);
            }
        }

        Ok(())
    }
}
