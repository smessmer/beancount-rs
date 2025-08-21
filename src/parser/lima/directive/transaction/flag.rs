use crate::model::Flag;

impl From<&beancount_parser_lima::Flag> for Flag {
    fn from(flag: &beancount_parser_lima::Flag) -> Self {
        match flag {
            beancount_parser_lima::Flag::Asterisk => Flag::ASTERISK,
            beancount_parser_lima::Flag::Exclamation => Flag::EXCLAMATION,
            beancount_parser_lima::Flag::Ampersand => Flag::AMPERSAND,
            beancount_parser_lima::Flag::Hash => Flag::HASH,
            beancount_parser_lima::Flag::Question => Flag::QUESTION,
            beancount_parser_lima::Flag::Percent => Flag::PERCENT,
            beancount_parser_lima::Flag::Letter(letter) => Flag::new(letter.char()),
        }
    }
}
