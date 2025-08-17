use chumsky::{prelude::*, text::whitespace};
use rust_decimal::Decimal;
use std::fmt::Write;

use crate::{
    model::Amount,
    parser::chumsky::commodity::{marshal_commodity, parse_commodity},
};

pub fn parse_amount<'a>() -> impl Parser<'a, &'a str, Amount<'a>, extra::Err<Rich<'a, char>>> {
    parse_decimal()
        .then_ignore(whitespace().at_least(1))
        .then(parse_commodity())
        .map(|(number, commodity)| Amount::new(number, commodity))
}

fn parse_decimal<'a>() -> impl Parser<'a, &'a str, Decimal, extra::Err<Rich<'a, char>>> {
    let sign = one_of("+-").or_not();
    let digits = one_of('0'..='9').repeated().at_least(1);
    let decimal_part = just('.')
        .then(one_of('0'..='9').repeated().at_least(1))
        .or_not();

    sign.then(digits)
        .then(decimal_part)
        .to_slice()
        .try_map(|slice: &'a str, span| {
            slice.parse::<Decimal>().map_err(|e| {
                chumsky::error::Rich::custom(span, format!("Invalid decimal number: {}", e))
            })
        })
}

pub fn marshal_amount(amount: &Amount, writer: &mut impl Write) -> std::fmt::Result {
    write!(writer, "{} ", amount.number())?;
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
    #[case("100.50 USD")]
    #[case("-50.25 EUR")]
    #[case("0 BTC")]
    #[case("1234567.89 AAPL")]
    #[case("0.00001 ETH")]
    #[case("1000000 JPY")]
    #[case("+500.75 CAD")]
    #[case("42 SHARES")]
    #[case("3.14159 A")]
    #[case("999.999999 A'B.C_D-E1")]
    fn valid_amount_template(#[case] input: &str) {}

    #[apply(valid_amount_template)]
    fn parse_valid_amount(#[case] input: &str) {
        let result = parse_amount().parse(input);
        assert!(result.has_output(), "Failed to parse amount: {}", input);
        let _parsed = result.into_result().unwrap();
    }

    #[apply(valid_amount_template)]
    fn marshal_and_parse_amount(#[case] input: &str) {
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
    fn parse_amount_basic() {
        let input = "100.50 USD";
        let result = parse_amount().parse(input);
        assert!(result.has_output());
        let amount = result.into_result().unwrap();

        assert_eq!(*amount.number(), dec!(100.50));
        assert_eq!(amount.commodity().as_ref(), "USD");
    }

    #[test]
    fn parse_amount_negative() {
        let input = "-3492.02 USD";
        let result = parse_amount().parse(input);
        assert!(result.has_output());
        let amount = result.into_result().unwrap();

        assert_eq!(*amount.number(), dec!(-3492.02));
        assert_eq!(amount.commodity().as_ref(), "USD");
    }

    #[test]
    fn parse_amount_positive_sign() {
        let input = "+250.00 EUR";
        let result = parse_amount().parse(input);
        assert!(result.has_output());
        let amount = result.into_result().unwrap();

        assert_eq!(*amount.number(), dec!(250.00));
        assert_eq!(amount.commodity().as_ref(), "EUR");
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
