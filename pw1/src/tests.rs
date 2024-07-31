#[cfg(test)]
mod tests {
    use anyhow::Result;
    use crate::common::add_file_to_dict;

    #[test]
    fn case() -> Result<()> {
        let (dict, _stats) = add_file_to_dict("data/tests/case.txt")?.unwrap();
        assert_eq!(dict.unique_word_count(), 1);
        assert_eq!(dict.total_word_count(), 5);

        Ok(())
    }

    #[test]
    fn ukr() -> Result<()> {
        let (dict, _stats) = add_file_to_dict("data/tests/ukr.txt")?.unwrap();
        assert_eq!(dict.unique_word_count(), 5);
        assert_eq!(dict.total_word_count(), 8);

        Ok(())
    }

    #[test]
    fn ukr_case() -> Result<()> {
        let (dict, _stats) = add_file_to_dict("data/tests/ukr_case.txt")?.unwrap();
        assert_eq!(dict.unique_word_count(), 1);
        assert_eq!(dict.total_word_count(), 5);

        Ok(())
    }

    #[test]
    fn ukr_apostrophe() -> Result<()> {
        let (dict, _stats) = add_file_to_dict("data/tests/ukr_apostrophe.txt")?.unwrap();
        assert_eq!(dict.unique_word_count(), 4);
        assert_eq!(dict.total_word_count(), 4);

        Ok(())
    }

    #[test]
    fn line_count() -> Result<()> {
        let (_dict, stats) = add_file_to_dict("data/tests/line_count.txt")?.unwrap();
        assert_eq!(stats.lines, 10);

        Ok(())
    }

    #[test]
    fn empty() -> Result<()> {
        let result = add_file_to_dict("data/tests/empty.txt")?;
        assert!(matches!(result, None));

        Ok(())
    }

    #[test]
    fn word_count() -> Result<()> {
        let (dict, _stats) = add_file_to_dict("data/tests/word_count.txt")?.unwrap();
        assert_eq!(dict.unique_word_count(), 4);
        assert_eq!(dict.total_word_count(), 11);

        Ok(())
    }

    #[test]
    fn character_count() -> Result<()> {
        let (_dict, stats) = add_file_to_dict("data/tests/character_count.txt")?.unwrap();
        assert_eq!(stats.characters_read, 15);
        assert_eq!(stats.characters_ignored, 3);

        Ok(())
    }

    #[test]
    fn character_count_with_newlines() -> Result<()> {
        let (_dict, stats) = add_file_to_dict("data/tests/character_count_with_newlines.txt")?.unwrap();
        assert_eq!(stats.characters_read, 47);
        assert_eq!(stats.characters_ignored, 9);

        Ok(())
    }

    #[test]
    fn ukr_sentence() -> Result<()> {
        let (dict, _stats) = add_file_to_dict("data/tests/ukr_sentence.txt")?.unwrap();
        assert_eq!(dict.unique_word_count(), 39);
        assert_eq!(dict.total_word_count(), 43);

        Ok(())
    }

    #[test]
    fn special_symbols() -> Result<()> {
        let (dict, stats) = add_file_to_dict("data/tests/special_symbols.txt")?.unwrap();
        assert_eq!(dict.unique_word_count(), 0);
        assert_eq!(dict.total_word_count(), 0);
        assert_eq!(stats.characters_read, 30);
        assert_eq!(stats.characters_ignored, stats.characters_read);

        Ok(())
    }
}
