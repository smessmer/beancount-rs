use chumsky::{prelude::*, text::whitespace};
use std::fmt::Write;

use crate::{
    model::directive::TransactionDescription,
    parser::chumsky::quoted_string::{marshal_quoted_string, parse_quoted_string},
};

/// Parser for transaction description (payee and narration)
/// Syntax: ["payee"] "narration" or just "narration"
pub fn parse_transaction_description<'a>()
-> impl Parser<'a, &'a str, TransactionDescription<'a>, extra::Err<Rich<'a, char>>> {
    parse_quoted_string()
        .then(
            whitespace()
                .at_least(1)
                .ignore_then(parse_quoted_string())
                .or_not(),
        )
        .map(|(first_str, second_str)| {
            if let Some(second_str) = second_str {
                TransactionDescription::new_with_payee(first_str, second_str)
            } else {
                TransactionDescription::new_without_payee(first_str)
            }
        })
}

/// Marshal a TransactionDescription to its string representation
pub fn marshal_transaction_description(
    description: &TransactionDescription<'_>,
    writer: &mut impl Write,
) -> std::fmt::Result {
    match description.payee() {
        Some(payee) => {
            marshal_quoted_string(payee, writer)?;
            write!(writer, " ")?;
            marshal_quoted_string(description.narration(), writer)
        }
        None => marshal_quoted_string(description.narration(), writer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn parse_transaction_description_narration_only() {
        let result = parse_transaction_description().parse("\"Direct deposit\"");
        assert!(result.has_output());
        let description = result.into_result().unwrap();

        assert_eq!(description.narration(), "Direct deposit");
        assert_eq!(description.payee(), None);
        assert!(!description.has_payee());
    }

    #[test]
    fn parse_transaction_description_with_payee() {
        let result =
            parse_transaction_description().parse("\"Cafe Mogador\" \"Lamb tagine with wine\"");
        assert!(result.has_output());
        let description = result.into_result().unwrap();

        assert_eq!(description.narration(), "Lamb tagine with wine");
        assert_eq!(description.payee(), Some("Cafe Mogador"));
        assert!(description.has_payee());
    }

    #[test]
    fn parse_transaction_description_empty_narration() {
        let result = parse_transaction_description().parse("\"\"");
        assert!(result.has_output());
        let description = result.into_result().unwrap();

        assert_eq!(description.narration(), "");
        assert_eq!(description.payee(), None);
    }

    #[test]
    fn parse_transaction_description_empty_payee_and_narration() {
        let result = parse_transaction_description().parse("\"\" \"\"");
        assert!(result.has_output());
        let description = result.into_result().unwrap();

        assert_eq!(description.narration(), "");
        assert_eq!(description.payee(), Some(""));
        assert!(description.has_payee());
    }

    #[test]
    fn parse_transaction_description_with_multiple_spaces() {
        let result = parse_transaction_description().parse("\"Payee\"   \"Narration\"");
        assert!(result.has_output());
        let description = result.into_result().unwrap();

        assert_eq!(description.narration(), "Narration");
        assert_eq!(description.payee(), Some("Payee"));
    }

    #[test]
    fn parse_transaction_description_with_tabs() {
        let result = parse_transaction_description().parse("\"Payee\"\t\t\"Narration\"");
        assert!(result.has_output());
        let description = result.into_result().unwrap();

        assert_eq!(description.narration(), "Narration");
        assert_eq!(description.payee(), Some("Payee"));
    }

    #[rstest]
    #[case("\"unterminated")] // Unterminated quote
    #[case("\"payee\" \"unterminated")] // Second unterminated quote
    #[case("\"payee\"")] // Missing space between payee and narration when two strings expected
    #[case("no quotes")] // No quotes
    #[case("")] // Empty string
    fn parse_transaction_description_invalid(#[case] input: &str) {
        let result = parse_transaction_description().parse(input);
        // Note: Some of these might actually succeed (like "\"payee\"" which would be valid narration-only)
        // The test case "\"payee\"" should actually succeed as narration-only
        if input == "\"payee\"" {
            assert!(
                result.has_output(),
                "Should succeed for narration-only: {}",
                input
            );
        } else {
            assert!(!result.has_output(), "Should fail to parse: {}", input);
        }
    }

    #[test]
    fn marshal_transaction_description_narration_only() {
        let description = TransactionDescription::new_without_payee("Direct deposit".to_string());
        let mut output = String::new();
        let result = marshal_transaction_description(&description, &mut output);

        assert!(result.is_ok());
        assert_eq!(output, "\"Direct deposit\"");
    }

    #[test]
    fn marshal_transaction_description_with_payee() {
        let description = TransactionDescription::new_with_payee(
            "Cafe Mogador".to_string(),
            "Lamb tagine with wine".to_string(),
        );
        let mut output = String::new();
        let result = marshal_transaction_description(&description, &mut output);

        assert!(result.is_ok());
        assert_eq!(output, "\"Cafe Mogador\" \"Lamb tagine with wine\"");
    }

    #[test]
    fn marshal_transaction_description_empty_narration() {
        let description = TransactionDescription::new_without_payee("".to_string());
        let mut output = String::new();
        let result = marshal_transaction_description(&description, &mut output);

        assert!(result.is_ok());
        assert_eq!(output, "\"\"");
    }

    #[test]
    fn marshal_transaction_description_empty_payee_and_narration() {
        let description = TransactionDescription::new_with_payee("".to_string(), "".to_string());
        let mut output = String::new();
        let result = marshal_transaction_description(&description, &mut output);

        assert!(result.is_ok());
        assert_eq!(output, "\"\" \"\"");
    }

    #[rstest]
    #[case("Direct deposit")]
    #[case("")]
    #[case("Transaction with special chars: $123.45!")]
    fn marshal_and_parse_narration_only_roundtrip(#[case] narration_text: &str) {
        let original_description =
            TransactionDescription::new_without_payee(narration_text.to_string());

        // Marshal to string
        let mut output = String::new();
        let marshal_result = marshal_transaction_description(&original_description, &mut output);
        assert!(marshal_result.is_ok());

        // Parse back from string
        let parse_result = parse_transaction_description().parse(&output);
        assert!(parse_result.has_output());
        let parsed_description = parse_result.into_result().unwrap();

        assert_eq!(parsed_description, original_description);
        assert_eq!(parsed_description.narration(), narration_text);
        assert_eq!(parsed_description.payee(), None);
    }

    #[rstest]
    #[case("Cafe Mogador", "Lamb tagine")]
    #[case("", "")]
    #[case("Store ABC", "Purchase of items")]
    #[case("Bank Transfer", "Monthly salary deposit")]
    fn marshal_and_parse_with_payee_roundtrip(
        #[case] payee_text: &str,
        #[case] narration_text: &str,
    ) {
        let original_description = TransactionDescription::new_with_payee(
            payee_text.to_string(),
            narration_text.to_string(),
        );

        // Marshal to string
        let mut output = String::new();
        let marshal_result = marshal_transaction_description(&original_description, &mut output);
        assert!(marshal_result.is_ok());

        // Parse back from string
        let parse_result = parse_transaction_description().parse(&output);
        assert!(parse_result.has_output());
        let parsed_description = parse_result.into_result().unwrap();

        assert_eq!(parsed_description, original_description);
        assert_eq!(parsed_description.narration(), narration_text);
        assert_eq!(parsed_description.payee(), Some(payee_text));
    }

    #[test]
    fn parse_transaction_description_in_context() {
        // Test parsing description after a flag, simulating transaction parsing
        let flag_then_description = just('*')
            .then_ignore(whitespace().at_least(1))
            .then(parse_transaction_description());

        let result = flag_then_description.parse("* \"Payee\" \"Description\"");
        assert!(result.has_output());
        let (flag, description) = result.into_result().unwrap();

        assert_eq!(flag, '*');
        assert_eq!(description.payee(), Some("Payee"));
        assert_eq!(description.narration(), "Description");
    }

    #[test]
    fn parse_transaction_description_priority() {
        // Test that parser correctly handles the choice between two strings vs one string
        // This should parse as narration-only (single string), not fail trying to find two strings
        let result1 = parse_transaction_description().parse("\"Just narration\"");
        assert!(result1.has_output());
        let desc1 = result1.into_result().unwrap();
        assert_eq!(desc1.narration(), "Just narration");
        assert_eq!(desc1.payee(), None);

        // This should parse as payee + narration (two strings)
        let result2 = parse_transaction_description().parse("\"Payee\" \"Narration\"");
        assert!(result2.has_output());
        let desc2 = result2.into_result().unwrap();
        assert_eq!(desc2.narration(), "Narration");
        assert_eq!(desc2.payee(), Some("Payee"));
    }
}
