mod objects;
mod results;

pub use objects::{
    Modification, ModificationType, ModificationValue, ObjectModificationTable, ObjectType,
    TableType,
};
pub use results::{JsonResult, TranslationError, WarResult};
