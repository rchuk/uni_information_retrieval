use std::path::Path;
use crate::dictionary::Dictionary;
use crate::storage::DictionaryStorage;

pub struct JsonDictionaryStorage;

impl DictionaryStorage for JsonDictionaryStorage {
    fn read(path: &Path) -> anyhow::Result<Dictionary> {
        let file = std::fs::File::open(path)?;

        Ok(serde_json::from_reader(file)?)
    }

    fn write(path: &Path, dictionary: &Dictionary) -> anyhow::Result<()> {
        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(file, dictionary)?;

        Ok(())
    }
}
