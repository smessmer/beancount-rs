use chumsky::prelude::*;
use std::borrow::Cow;
use std::fmt::Write;

/// Parser for quoted strings with escape sequences
/// Syntax: "content" where \" is an escaped quote and \\ is an escaped backslash
/// Returns a borrowed string if no escapes are present, owned if escaping was needed
pub fn parse_quoted_string<'a>()
-> impl Parser<'a, &'a str, Cow<'a, str>, extra::Err<Rich<'a, char>>> {
    let escape_sequence = just('\\').ignore_then(one_of("\"\\")).ignored();
    let regular_char = none_of("\"\\").ignored();
    let string_content = regular_char.or(escape_sequence).repeated();

    just('"')
        .ignore_then(string_content.to_slice())
        .then_ignore(just('"'))
        .map(|content: &str| {
            if content.contains('\\') {
                let mut result = String::with_capacity(content.len());
                let mut chars = content.chars();
                while let Some(c) = chars.next() {
                    if c == '\\' {
                        if let Some(escaped) = chars.next() {
                            result.push(escaped);
                        }
                    } else {
                        result.push(c);
                    }
                }
                Cow::Owned(result)
            } else {
                Cow::Borrowed(content)
            }
        })
}

/// Marshal a string to its quoted representation with proper escaping
/// Quotes are escaped as \" and backslashes as \\
pub fn marshal_quoted_string(s: &str, writer: &mut impl Write) -> std::fmt::Result {
    writer.write_char('\"')?;
    for c in s.chars() {
        match c {
            '"' => writer.write_str("\\\"")?,
            '\\' => writer.write_str("\\\\")?,
            _ => writer.write_char(c)?,
        }
    }
    writer.write_char('\"')?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn parse_quoted_string_basic() {
        let result = parse_quoted_string().parse("\"Cafe Mogador\"");
        assert!(result.has_output());
        let string = result.into_result().unwrap();
        assert_eq!(string, "Cafe Mogador");
    }

    #[test]
    fn parse_quoted_string_empty() {
        let result = parse_quoted_string().parse("\"\"");
        assert!(result.has_output());
        let string = result.into_result().unwrap();
        assert_eq!(string, "");
    }

    #[test]
    fn parse_quoted_string_with_spaces() {
        let result = parse_quoted_string().parse("\"  Hello World  \"");
        assert!(result.has_output());
        let string = result.into_result().unwrap();
        assert_eq!(string, "  Hello World  ");
    }

    #[test]
    fn parse_quoted_string_with_special_chars() {
        let result = parse_quoted_string().parse("\"Item: $123.45 (tax included)\"");
        assert!(result.has_output());
        let string = result.into_result().unwrap();
        assert_eq!(string, "Item: $123.45 (tax included)");
    }

    #[rstest]
    #[case("\"unterminated")] // Missing closing quote
    #[case("unterminated\"")] // Missing opening quote
    #[case("no quotes")] // No quotes at all
    #[case("\"")] // Only one quote
    #[case("")] // Empty string
    #[case("\"incomplete escape\\")] // Incomplete escape at end
    #[case("\"invalid escape \\x\"")] // Invalid escape sequence
    #[case("\"invalid escape \\n\"")] // Invalid escape sequence
    fn parse_quoted_string_invalid(#[case] input: &str) {
        let result = parse_quoted_string().parse(input);
        assert!(!result.has_output(), "Should fail to parse: {}", input);
    }

    #[test]
    fn parse_quoted_string_with_escaped_quotes() {
        let result = parse_quoted_string().parse("\"She said \\\"Hello\\\"\"");
        assert!(result.has_output());
        let cow_string = result.into_result().unwrap();
        assert_eq!(cow_string, "She said \"Hello\"");
        // Should be owned (current implementation always returns owned)
        assert!(matches!(cow_string, Cow::Owned(_)));
    }

    #[test]
    fn parse_quoted_string_with_escaped_backslashes() {
        let result = parse_quoted_string().parse("\"Path: C:\\\\Users\\\\Name\"");
        assert!(result.has_output());
        let cow_string = result.into_result().unwrap();
        assert_eq!(cow_string, "Path: C:\\Users\\Name");
        // Should be owned (current implementation always returns owned)
        assert!(matches!(cow_string, Cow::Owned(_)));
    }

    #[test]
    fn parse_quoted_string_mixed_escapes() {
        let result = parse_quoted_string().parse("\"Quote: \\\"text\\\" and path: C:\\\\temp\"");
        assert!(result.has_output());
        let cow_string = result.into_result().unwrap();
        assert_eq!(cow_string, "Quote: \"text\" and path: C:\\temp");
        // Should be owned (current implementation always returns owned)
        assert!(matches!(cow_string, Cow::Owned(_)));
    }

    #[test]
    fn parse_quoted_string_only_escapes() {
        let result = parse_quoted_string().parse("\"\\\"\\\\\\\"\"");
        assert!(result.has_output());
        let cow_string = result.into_result().unwrap();
        assert_eq!(cow_string, "\"\\\"");
        // Should be owned (current implementation always returns owned)
        assert!(matches!(cow_string, Cow::Owned(_)));
    }

    #[test]
    fn marshal_quoted_string_basic() {
        let mut output = String::new();
        let result = marshal_quoted_string("Cafe Mogador", &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "\"Cafe Mogador\"");
    }

    #[test]
    fn marshal_quoted_string_empty() {
        let mut output = String::new();
        let result = marshal_quoted_string("", &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "\"\"");
    }

    #[test]
    fn marshal_quoted_string_with_spaces() {
        let mut output = String::new();
        let result = marshal_quoted_string("  Hello World  ", &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "\"  Hello World  \"");
    }

    #[test]
    fn marshal_quoted_string_with_special_chars() {
        let mut output = String::new();
        let result = marshal_quoted_string("Item: $123.45 (tax included)", &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "\"Item: $123.45 (tax included)\"");
    }

    #[test]
    fn marshal_quoted_string_with_quotes() {
        let mut output = String::new();
        let result = marshal_quoted_string("She said \"Hello\"", &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "\"She said \\\"Hello\\\"\"");
    }

    #[test]
    fn marshal_quoted_string_with_backslashes() {
        let mut output = String::new();
        let result = marshal_quoted_string("Path: C:\\Users\\Name", &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "\"Path: C:\\\\Users\\\\Name\"");
    }

    #[test]
    fn marshal_quoted_string_mixed_escapes() {
        let mut output = String::new();
        let result = marshal_quoted_string("Quote: \"text\" and path: C:\\temp", &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "\"Quote: \\\"text\\\" and path: C:\\\\temp\"");
    }

    #[test]
    fn marshal_quoted_string_only_escapes() {
        let mut output = String::new();
        let result = marshal_quoted_string("\"\\\"", &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "\"\\\"\\\\\\\"\"");
    }

    #[rstest]
    #[case("Simple text", "\"Simple text\"")]
    #[case("", "\"\"")]
    #[case("  Spaces  ", "\"  Spaces  \"")]
    #[case("Special chars: $123.45!", "\"Special chars: $123.45!\"")]
    #[case("Café Napoléon", "\"Café Napoléon\"")]
    #[case("She said \"Hello\"", "\"She said \\\"Hello\\\"\"")]
    #[case("Path: C:\\Users\\Name", "\"Path: C:\\\\Users\\\\Name\"")]
    #[case(
        "Quote: \"text\" and path: C:\\temp",
        "\"Quote: \\\"text\\\" and path: C:\\\\temp\""
    )]
    #[case("\"\\\"", "\"\\\"\\\\\\\"\"")]
    fn marshal_quoted_string_roundtrip(#[case] original_text: &str, #[case] expected_quoted: &str) {
        // Marshal string to quoted format
        let mut output = String::new();
        let marshal_result = marshal_quoted_string(original_text, &mut output);
        assert!(marshal_result.is_ok());
        assert_eq!(output, expected_quoted);

        // Parse quoted string back to original text
        let parse_result = parse_quoted_string().parse(&output);
        assert!(parse_result.has_output());
        let parsed_cow = parse_result.into_result().unwrap();
        assert_eq!(parsed_cow.as_ref(), original_text);
    }

    #[test]
    fn parse_quoted_string_multiple_in_sequence() {
        // Test parsing multiple quoted strings in sequence
        let multiple_strings = parse_quoted_string()
            .then_ignore(just(' ').repeated().at_least(1))
            .then(parse_quoted_string());

        let result = multiple_strings.parse("\"first\" \"second\"");
        assert!(result.has_output());
        let (first, second) = result.into_result().unwrap();
        assert_eq!(first, "first");
        assert_eq!(second, "second");
        // Both should be borrowed since no escapes
        assert!(matches!(first, Cow::Borrowed(_)));
        assert!(matches!(second, Cow::Borrowed(_)));
    }

    #[test]
    fn parse_quoted_string_in_context() {
        // Test parsing quoted string as part of a larger parsing context
        let flag_then_quoted = just('*')
            .then_ignore(just(' ').repeated().at_least(1))
            .then(parse_quoted_string());

        let result = flag_then_quoted.parse("* \"Transaction description\"");
        assert!(result.has_output());
        let (flag, description) = result.into_result().unwrap();
        assert_eq!(flag, '*');
        assert_eq!(description, "Transaction description");
        // Should be borrowed since no escapes
        assert!(matches!(description, Cow::Borrowed(_)));
    }

    #[test]
    fn parse_quoted_string_borrowed_vs_owned() {
        // Test simple strings (should be borrowed)
        let result1 = parse_quoted_string().parse("\"Simple text\"");
        assert!(result1.has_output());
        let cow1 = result1.into_result().unwrap();
        assert_eq!(cow1, "Simple text");
        assert!(matches!(cow1, Cow::Borrowed(_))); // Should be borrowed since no escapes

        // Test that strings with escapes return owned values
        let result2 = parse_quoted_string().parse("\"Text with \\\"quotes\\\"\"");
        assert!(result2.has_output());
        let cow2 = result2.into_result().unwrap();
        assert_eq!(cow2, "Text with \"quotes\"");
        assert!(matches!(cow2, Cow::Owned(_)));

        // Test empty string (should be borrowed)
        let result3 = parse_quoted_string().parse("\"\"");
        assert!(result3.has_output());
        let cow3 = result3.into_result().unwrap();
        assert_eq!(cow3, "");
        assert!(matches!(cow3, Cow::Borrowed(_))); // Should be borrowed since no escapes
    }
}
