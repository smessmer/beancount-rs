#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Flag {
    flag: char,
}

impl Flag {
    pub const fn new(flag: char) -> Self {
        Flag { flag }
    }

    pub const fn as_char(&self) -> char {
        self.flag
    }

    pub const ASTERISK: Self = Flag::new('*');
    pub const EXCLAMATION: Self = Flag::new('!');
    pub const AMPERSAND: Self = Flag::new('&');
    pub const HASH: Self = Flag::new('#');
    pub const QUESTION: Self = Flag::new('?');
    pub const PERCENT: Self = Flag::new('%');
}
