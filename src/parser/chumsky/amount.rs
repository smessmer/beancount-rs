use chumsky::{prelude::*, text::whitespace};
use std::fmt::Write;

use crate::{
    model::Amount,
    parser::chumsky::{
        commodity::{marshal_commodity, parse_commodity},
        decimal::{marshal_decimal, parse_decimal},
    },
};

pub fn parse_amount<'a>() -> impl Parser<'a, &'a str, Amount<'a>, extra::Err<Rich<'a, char>>> {
    parse_decimal()
        .then_ignore(whitespace().at_least(1))
        .then(parse_commodity())
        .map(|(number, commodity)| Amount::new(number, commodity))
}

pub fn marshal_amount(amount: &Amount, writer: &mut impl Write) -> std::fmt::Result {
    marshal_decimal(amount.number(), writer)?;
    write!(writer, " ")?;
    marshal_commodity(amount.commodity(), writer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::commodity;
    use rstest::rstest;
    use rstest_reuse::*;
    use rust_decimal_macros::dec;

    #[template]
    #[rstest]
    #[case("100.50 USD", dec!(100.50), "USD")]
    #[case("-50.25 EUR", dec!(-50.25), "EUR")]
    #[case("0 BTC", dec!(0), "BTC")]
    #[case("1234567.89 AAPL", dec!(1234567.89), "AAPL")]
    #[case("0.00001 ETH", dec!(0.00001), "ETH")]
    #[case("1000000 JPY", dec!(1000000), "JPY")]
    #[case("+500.75 CAD", dec!(500.75), "CAD")]
    #[case("42 SHARES", dec!(42), "SHARES")]
    #[case("3.14159 A", dec!(3.14159), "A")]
    #[case("999.999999 A'B.C_D-E1", dec!(999.999999), "A'B.C_D-E1")]
    #[case("-3492.02 USD", dec!(-3492.02), "USD")]
    #[case("+250.00 EUR", dec!(250.00), "EUR")]
    fn valid_amount_template(#[case] input: &str, #[case] expected_number: rust_decimal::Decimal, #[case] expected_commodity: &str) {}

    #[apply(valid_amount_template)]
    fn parse_valid_amount(#[case] input: &str, #[case] expected_number: rust_decimal::Decimal, #[case] expected_commodity: &str) {
        let result = parse_amount().parse(input);
        assert!(result.has_output(), "Failed to parse amount: {}", input);
        let parsed = result.into_result().unwrap();
        assert_eq!(*parsed.number(), expected_number);
        assert_eq!(parsed.commodity().as_ref(), expected_commodity);
    }

    #[apply(valid_amount_template)]
    fn marshal_and_parse_amount(#[case] input: &str, #[case] _expected_number: rust_decimal::Decimal, #[case] _expected_commodity: &str) {
        // Parse the original
        let result = parse_amount().parse(input);
        assert!(result.has_output());
        let original = result.into_result().unwrap();

        // Marshal it
        let mut marshalled = String::new();
        let marshal_result = marshal_amount(&original, &mut marshalled);
        assert!(marshal_result.is_ok());

        // Parse it back
        let reparse_result = parse_amount().parse(&marshalled);
        assert!(reparse_result.has_output());
        let reparsed = reparse_result.into_result().unwrap();

        // Should be equal
        assert_eq!(original, reparsed);
    }



    #[test]
    fn parse_amount_integer() {
        let input = "1000 JPY";
        let result = parse_amount().parse(input);
        assert!(result.has_output());
        let amount = result.into_result().unwrap();

        assert_eq!(*amount.number(), dec!(1000));
        assert_eq!(amount.commodity().as_ref(), "JPY");
    }

    #[test]
    fn parse_amount_zero() {
        let input = "0 BTC";
        let result = parse_amount().parse(input);
        assert!(result.has_output());
        let amount = result.into_result().unwrap();

        assert_eq!(*amount.number(), dec!(0));
        assert_eq!(amount.commodity().as_ref(), "BTC");
    }

    #[test]
    fn parse_amount_multiple_spaces() {
        let input = "123.45    USD";
        let result = parse_amount().parse(input);
        assert!(result.has_output());
        let amount = result.into_result().unwrap();

        assert_eq!(*amount.number(), dec!(123.45));
        assert_eq!(amount.commodity().as_ref(), "USD");
    }

    #[test]
    fn parse_amount_tab_separator() {
        let input = "567.89\tEUR";
        let result = parse_amount().parse(input);
        assert!(result.has_output());
        let amount = result.into_result().unwrap();

        assert_eq!(*amount.number(), dec!(567.89));
        assert_eq!(amount.commodity().as_ref(), "EUR");
    }

    #[test]
    fn parse_amount_high_precision() {
        let input = "0.000000001 BTC";
        let result = parse_amount().parse(input);
        assert!(result.has_output());
        let amount = result.into_result().unwrap();

        assert_eq!(*amount.number(), dec!(0.000000001));
        assert_eq!(amount.commodity().as_ref(), "BTC");
    }

    #[rstest]
    #[case("USD")] // Missing number
    #[case("100.50")] // Missing commodity
    #[case("100.50USD")] // Missing space
    #[case("abc USD")] // Invalid number
    #[case("100.50.25 USD")] // Invalid number format
    #[case("100. USD")] // Invalid number format
    #[case(".100 USD")] // Invalid number format
    #[case("100.50 usd")] // Invalid commodity
    #[case("")] // Empty input
    fn parse_amount_invalid(#[case] input: &str) {
        let result = parse_amount().parse(input);
        assert!(!result.has_output(), "Should fail to parse: {}", input);
    }

    #[test]
    fn marshal_amount_basic() {
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(100.50), commodity);

        let mut output = String::new();
        let result = marshal_amount(&amount, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "100.50 USD");
    }

    #[test]
    fn marshal_amount_negative() {
        let commodity = commodity!(EUR);
        let amount = Amount::new(dec!(-3492.02), commodity);

        let mut output = String::new();
        let result = marshal_amount(&amount, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "-3492.02 EUR");
    }

    #[test]
    fn marshal_amount_zero() {
        let commodity = commodity!(BTC);
        let amount = Amount::new(dec!(0), commodity);

        let mut output = String::new();
        let result = marshal_amount(&amount, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "0 BTC");
    }

    #[test]
    fn parse_decimal_valid() {
        let cases = [
            ("123", dec!(123)),
            ("123.45", dec!(123.45)),
            ("-123.45", dec!(-123.45)),
            ("+123.45", dec!(123.45)),
            ("0", dec!(0)),
            ("0.001", dec!(0.001)),
        ];

        for (input, expected) in cases {
            let result = parse_decimal().parse(input);
            assert!(result.has_output(), "Failed to parse decimal: {}", input);
            let parsed = result.into_result().unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn parse_decimal_invalid() {
        let invalid_cases = ["", "abc", "123.45.67", "12.34.56", "+", "-", ".123"];

        for input in invalid_cases {
            let result = parse_decimal().parse(input);
            assert!(
                !result.has_output(),
                "Should fail to parse decimal: {}",
                input
            );
        }
    }
}
