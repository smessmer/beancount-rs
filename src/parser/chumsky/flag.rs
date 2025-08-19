use chumsky::prelude::*;
use std::fmt::Write;

use crate::model::Flag;

const fn flag_char(flag: Flag) -> char {
    match flag {
        Flag::Complete => '*',
        Flag::Incomplete => '!',
    }
}

/// Parser for flag characters (without whitespace)
/// Syntax: * (complete) or ! (incomplete)
pub fn parse_flag<'a>() -> impl Parser<'a, &'a str, Flag, extra::Err<Rich<'a, char>>> {
    choice((
        just(flag_char(Flag::Complete)).to(Flag::Complete),
        just(flag_char(Flag::Incomplete)).to(Flag::Incomplete),
    ))
}

/// Marshal a flag to its string representation
pub fn marshal_flag(flag: Flag, writer: &mut impl Write) -> std::fmt::Result {
    write!(writer, "{}", flag_char(flag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn parse_flag_complete() {
        let result = parse_flag().parse("*");
        assert!(result.has_output());
        let flag = result.into_result().unwrap();
        assert_eq!(flag, Flag::Complete);
    }

    #[test]
    fn parse_flag_incomplete() {
        let result = parse_flag().parse("!");
        assert!(result.has_output());
        let flag = result.into_result().unwrap();
        assert_eq!(flag, Flag::Incomplete);
    }

    #[rstest]
    #[case("x")] // Invalid character
    #[case("")] // Empty string
    #[case("**")] // Multiple flags
    #[case("*!")] // Both flags
    #[case(" ")] // Just whitespace
    #[case("complete")] // Word instead of symbol
    fn parse_flag_invalid(#[case] input: &str) {
        let result = parse_flag().parse(input);
        assert!(!result.has_output(), "Should fail to parse: {}", input);
    }

    #[test]
    fn marshal_flag_complete() {
        let mut output = String::new();
        let result = marshal_flag(Flag::Complete, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "*");
    }

    #[test]
    fn marshal_flag_incomplete() {
        let mut output = String::new();
        let result = marshal_flag(Flag::Incomplete, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "!");
    }

    #[rstest]
    #[case(Flag::Complete, "*")]
    #[case(Flag::Incomplete, "!")]
    fn marshal_flag_roundtrip(#[case] original_flag: Flag, #[case] expected_text: &str) {
        // Marshal flag to string
        let mut output = String::new();
        let marshal_result = marshal_flag(original_flag, &mut output);
        assert!(marshal_result.is_ok());
        assert_eq!(output, expected_text);

        // Parse string back to flag
        let parse_result = parse_flag().parse(&output);
        assert!(parse_result.has_output());
        let parsed_flag = parse_result.into_result().unwrap();
        assert_eq!(parsed_flag, original_flag);
    }

    #[test]
    fn parse_flag_copy_semantics() {
        // Test that Flag implements Copy and can be used efficiently
        let result = parse_flag().parse("*");
        assert!(result.has_output());
        let flag1 = result.into_result().unwrap();
        let flag2 = flag1; // This should work because Flag implements Copy
        assert_eq!(flag1, flag2);
        assert_eq!(flag1, Flag::Complete);
        assert_eq!(flag2, Flag::Complete);
    }
}
