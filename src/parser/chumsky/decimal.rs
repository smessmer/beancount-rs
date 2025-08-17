use chumsky::prelude::*;
use rust_decimal::Decimal;
use std::fmt::Write;

pub fn parse_decimal<'a>() -> impl Parser<'a, &'a str, Decimal, extra::Err<Rich<'a, char>>> {
    let sign = one_of("+-").or_not();

    sign.then(parse_positive_decimal())
        .map(|(sign, mut number)| {
            if let Some(s) = sign {
                if s == '-' {
                    number.set_sign_negative(true);
                }
            }
            number
        })
}

pub fn parse_positive_decimal<'a>() -> impl Parser<'a, &'a str, Decimal, extra::Err<Rich<'a, char>>>
{
    let digits = one_of('0'..='9').repeated().at_least(1);
    let decimal_part = just('.').then(digits.clone()).or_not();

    digits
        .then(decimal_part)
        .to_slice()
        .try_map(|slice: &'a str, span| {
            slice.parse::<Decimal>().map_err(|e| {
                chumsky::error::Rich::custom(span, format!("Invalid decimal number: {}", e))
            })
        })
}

pub fn marshal_decimal(decimal: &Decimal, writer: &mut impl Write) -> std::fmt::Result {
    write!(writer, "{}", decimal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use rstest_reuse::*;
    use rust_decimal_macros::dec;

    #[template]
    #[rstest]
    #[case("123", dec!(123))]
    #[case("123.45", dec!(123.45))]
    #[case("-123.45", dec!(-123.45))]
    #[case("+123.45", dec!(123.45))]
    #[case("0", dec!(0))]
    #[case("0.001", dec!(0.001))]
    #[case("1000000", dec!(1000000))]
    #[case("0.000000001", dec!(0.000000001))]
    #[case("999.999999", dec!(999.999999))]
    #[case("+0", dec!(0))]
    #[case("-0", dec!(0))]
    #[case("42", dec!(42))]
    #[case("3.14159", dec!(3.14159))]
    #[case("1234567.89", dec!(1234567.89))]
    fn valid_decimal_template(#[case] input: &str, #[case] expected: Decimal) {}

    #[apply(valid_decimal_template)]
    fn parse_valid_decimal(#[case] input: &str, #[case] expected: Decimal) {
        let result = parse_decimal().parse(input);
        assert!(result.has_output(), "Failed to parse decimal: {}", input);
        let parsed = result.into_result().unwrap();
        assert_eq!(
            parsed, expected,
            "Parsed decimal doesn't match expected for input: {}",
            input
        );
    }

    #[apply(valid_decimal_template)]
    fn marshal_and_parse_decimal(#[case] input: &str, #[case] expected: Decimal) {
        // Parse the original
        let result = parse_decimal().parse(input);
        assert!(result.has_output(), "Failed to parse decimal: {}", input);
        let original = result.into_result().unwrap();

        // Marshal it
        let mut marshalled = String::new();
        let marshal_result = marshal_decimal(&original, &mut marshalled);
        assert!(
            marshal_result.is_ok(),
            "Failed to marshal decimal: {:?}",
            original
        );

        // Parse it back
        let reparse_result = parse_decimal().parse(&marshalled);
        assert!(
            reparse_result.has_output(),
            "Failed to reparse marshalled decimal: {}",
            marshalled
        );
        let reparsed = reparse_result.into_result().unwrap();

        // Should be equal
        assert_eq!(
            original, reparsed,
            "Marshalled and reparsed decimal doesn't match original"
        );
    }

    #[rstest]
    #[case("")] // Empty input
    #[case("abc")] // Invalid characters
    #[case("123.45.67")] // Multiple decimal points
    #[case("12.34.56")] // Multiple decimal points
    #[case("+")] // Sign only
    #[case("-")] // Sign only
    #[case(".123")] // No digits before decimal
    #[case("123.")] // No digits after decimal
    #[case("12..34")] // Double dots
    #[case("1.2.3.4")] // Multiple dots
    #[case("12abc34")] // Mixed valid/invalid
    fn parse_decimal_invalid(#[case] input: &str) {
        let result = parse_decimal().parse(input);
        assert!(
            !result.has_output(),
            "Should fail to parse decimal: {}",
            input
        );
    }

    #[test]
    fn parse_decimal_basic() {
        let result = parse_decimal().parse("123.45");
        assert!(result.has_output());
        let decimal = result.into_result().unwrap();
        assert_eq!(decimal, dec!(123.45));
    }

    #[test]
    fn parse_decimal_negative() {
        let result = parse_decimal().parse("-123.45");
        assert!(result.has_output());
        let decimal = result.into_result().unwrap();
        assert_eq!(decimal, dec!(-123.45));
    }

    #[test]
    fn parse_decimal_positive_sign() {
        let result = parse_decimal().parse("+123.45");
        assert!(result.has_output());
        let decimal = result.into_result().unwrap();
        assert_eq!(decimal, dec!(123.45));
    }

    #[test]
    fn parse_decimal_integer() {
        let result = parse_decimal().parse("1000");
        assert!(result.has_output());
        let decimal = result.into_result().unwrap();
        assert_eq!(decimal, dec!(1000));
    }

    #[test]
    fn parse_decimal_zero() {
        let result = parse_decimal().parse("0");
        assert!(result.has_output());
        let decimal = result.into_result().unwrap();
        assert_eq!(decimal, dec!(0));
    }

    #[test]
    fn parse_decimal_high_precision() {
        let result = parse_decimal().parse("0.000000001");
        assert!(result.has_output());
        let decimal = result.into_result().unwrap();
        assert_eq!(decimal, dec!(0.000000001));
    }

    #[test]
    fn marshal_decimal_basic() {
        let decimal = dec!(123.45);
        let mut output = String::new();
        let result = marshal_decimal(&decimal, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "123.45");
    }

    #[test]
    fn marshal_decimal_negative() {
        let decimal = dec!(-123.45);
        let mut output = String::new();
        let result = marshal_decimal(&decimal, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "-123.45");
    }

    #[test]
    fn marshal_decimal_zero() {
        let decimal = dec!(0);
        let mut output = String::new();
        let result = marshal_decimal(&decimal, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "0");
    }

    #[test]
    fn marshal_decimal_integer() {
        let decimal = dec!(1000);
        let mut output = String::new();
        let result = marshal_decimal(&decimal, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "1000");
    }

    #[test]
    fn marshal_decimal_high_precision() {
        let decimal = dec!(0.000000001);
        let mut output = String::new();
        let result = marshal_decimal(&decimal, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "0.000000001");
    }
}
