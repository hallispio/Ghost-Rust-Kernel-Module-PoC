use crate::i18n::translate;

pub struct TranslationTable;

impl TranslationTable {
    #[inline]
    pub fn lookup(&self, input: &str) -> Option<&'static str> {
        translate(input)
    }
}

pub static TRANSLATION_TABLE: TranslationTable = TranslationTable;