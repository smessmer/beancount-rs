use chumsky::prelude::*;
use std::fmt::Write;

use crate::model::Flag;

/// Parser for flag characters (without whitespace)
/// Syntax: * (complete) or ! (incomplete)
pub fn parse_flag<'a>() -> impl Parser<'a, &'a str, Flag, extra::Err<Rich<'a, char>>> {
    any().filter(|c: &char| !c.is_whitespace()).map(Flag::new)
}

/// Marshal a flag to its string representation
pub fn marshal_flag(flag: Flag, writer: &mut impl Write) -> std::fmt::Result {
    write!(writer, "{}", flag.as_char())
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
        assert_eq!(flag, Flag::ASTERISK);
    }

    #[test]
    fn parse_flag_incomplete() {
        let result = parse_flag().parse("!");
        assert!(result.has_output());
        let flag = result.into_result().unwrap();
        assert_eq!(flag, Flag::EXCLAMATION);
    }

    #[rstest]
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
        let result = marshal_flag(Flag::ASTERISK, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "*");
    }

    #[test]
    fn marshal_flag_incomplete() {
        let mut output = String::new();
        let result = marshal_flag(Flag::EXCLAMATION, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "!");
    }

    #[rstest]
    #[case(Flag::ASTERISK, "*")]
    #[case(Flag::EXCLAMATION, "!")]
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
        assert_eq!(flag1, Flag::ASTERISK);
        assert_eq!(flag2, Flag::ASTERISK);
    }
}
