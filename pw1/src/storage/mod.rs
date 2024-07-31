pub mod json_dictionary_storage;
pub mod key_val_dictionary_storage;

pub use json_dictionary_storage::JsonDictionaryStorage;
pub use key_val_dictionary_storage::KeyValDictionaryStorage;

use anyhow::Result;
use std::path::Path;
use crate::dictionary::Dictionary;

pub trait DictionaryStorage {
    fn read(path: &Path) -> Result<Dictionary>;
    fn write(path: &Path, dictionary: &Dictionary) -> Result<()>;
}
