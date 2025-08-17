use chumsky::{prelude::*, text::whitespace};
use std::fmt::Write;

use crate::{
    model::{Amount, AmountWithTolerance},
    parser::chumsky::{
        commodity::{marshal_commodity, parse_commodity},
        decimal::{marshal_decimal, parse_decimal, parse_positive_decimal},
    },
};

/// Parser for amount with optional tolerance
/// Syntax: <number> [~ <tolerance>] <commodity>
pub fn parse_amount_with_tolerance<'a>()
-> impl Parser<'a, &'a str, AmountWithTolerance<'a>, extra::Err<Rich<'a, char>>> {
    let tolerance = just('~')
        .ignore_then(whitespace().at_least(1))
        .ignore_then(parse_positive_decimal());

    parse_decimal()
        .then_ignore(whitespace().at_least(1))
        .then(tolerance.then_ignore(whitespace().at_least(1)).or_not())
        .then(parse_commodity())
        .map(|((number, tolerance), commodity)| {
            AmountWithTolerance::new(Amount::new(number, commodity), tolerance)
        })
}

pub fn marshal_amount_with_tolerance(
    amount: &AmountWithTolerance,
    writer: &mut impl Write,
) -> std::fmt::Result {
    marshal_decimal(amount.number(), writer)?;

    if let Some(tolerance) = amount.tolerance() {
        write!(writer, " ~ ")?;
        marshal_decimal(tolerance, writer)?;
    }

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
    #[case("100.50 USD")]
    #[case("-50.25 EUR")]
    #[case("0 BTC")]
    #[case("319.020 ~ 0.002 RGAGX")]
    #[case("1000 ~ 1 JPY")]
    #[case("+500.75 CAD")]
    #[case("42 SHARES")]
    #[case("3.14159 ~ 0.00001 PI")]
    #[case("0.00001 ETH")]
    fn valid_amount_with_tolerance_template(#[case] input: &str) {}

    #[apply(valid_amount_with_tolerance_template)]
    fn parse_valid_amount_with_tolerance(#[case] input: &str) {
        let result = parse_amount_with_tolerance().parse(input);
        assert!(
            result.has_output(),
            "Failed to parse amount with tolerance: {}",
            input
        );
        let _parsed = result.into_result().unwrap();
    }

    #[apply(valid_amount_with_tolerance_template)]
    fn marshal_and_parse_amount_with_tolerance(#[case] input: &str) {
        // Parse the original
        let result = parse_amount_with_tolerance().parse(input);
        assert!(result.has_output());
        let original = result.into_result().unwrap();

        // Marshal it
        let mut marshalled = String::new();
        let marshal_result = marshal_amount_with_tolerance(&original, &mut marshalled);
        assert!(marshal_result.is_ok());

        // Parse it back
        let reparse_result = parse_amount_with_tolerance().parse(&marshalled);
        assert!(reparse_result.has_output());
        let reparsed = reparse_result.into_result().unwrap();

        // Should be equal
        assert_eq!(original, reparsed);
    }

    #[test]
    fn parse_amount_with_tolerance_basic() {
        let input = "100.50 USD";
        let result = parse_amount_with_tolerance().parse(input);
        assert!(result.has_output());
        let amount = result.into_result().unwrap();

        assert_eq!(*amount.number(), dec!(100.50));
        assert_eq!(amount.tolerance(), None);
        assert_eq!(amount.commodity().as_ref(), "USD");
    }

    #[test]
    fn parse_amount_with_tolerance_with_tolerance() {
        let input = "319.020 ~ 0.002 RGAGX";
        let result = parse_amount_with_tolerance().parse(input);
        assert!(result.has_output());
        let amount = result.into_result().unwrap();

        assert_eq!(*amount.number(), dec!(319.020));
        assert_eq!(amount.tolerance(), Some(&dec!(0.002)));
        assert_eq!(amount.commodity().as_ref(), "RGAGX");
    }

    #[test]
    fn parse_amount_with_tolerance_negative() {
        let input = "-3492.02 USD";
        let result = parse_amount_with_tolerance().parse(input);
        assert!(result.has_output());
        let amount = result.into_result().unwrap();

        assert_eq!(*amount.number(), dec!(-3492.02));
        assert_eq!(amount.tolerance(), None);
        assert_eq!(amount.commodity().as_ref(), "USD");
    }

    #[test]
    fn parse_amount_with_tolerance_positive_sign() {
        let input = "+250.00 EUR";
        let result = parse_amount_with_tolerance().parse(input);
        assert!(result.has_output());
        let amount = result.into_result().unwrap();

        assert_eq!(*amount.number(), dec!(250.00));
        assert_eq!(amount.tolerance(), None);
        assert_eq!(amount.commodity().as_ref(), "EUR");
    }

    #[test]
    fn parse_amount_with_tolerance_zero() {
        let input = "0 BTC";
        let result = parse_amount_with_tolerance().parse(input);
        assert!(result.has_output());
        let amount = result.into_result().unwrap();

        assert_eq!(*amount.number(), dec!(0));
        assert_eq!(amount.tolerance(), None);
        assert_eq!(amount.commodity().as_ref(), "BTC");
    }

    #[rstest]
    #[case("USD")] // Missing number
    #[case("100.50")] // Missing commodity
    #[case("100.50USD")] // Missing space
    #[case("abc USD")] // Invalid number
    #[case("100.50.25 USD")] // Invalid number format
    #[case("100.50 usd")] // Invalid commodity
    #[case("100.50 ~ USD")] // Missing tolerance value
    #[case("100.50 ~ abc USD")] // Invalid tolerance
    #[case("100.50 ~ -0.1 USD")] // Negative tolerance
    #[case("")] // Empty input
    fn parse_amount_with_tolerance_invalid(#[case] input: &str) {
        let result = parse_amount_with_tolerance().parse(input);
        assert!(!result.has_output(), "Should fail to parse: {}", input);
    }

    #[test]
    fn marshal_amount_with_tolerance_basic() {
        let commodity = commodity!(USD);
        let amount = AmountWithTolerance::without_tolerance(dec!(100.50), commodity);

        let mut output = String::new();
        let result = marshal_amount_with_tolerance(&amount, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "100.50 USD");
    }

    #[test]
    fn marshal_amount_with_tolerance_with_tolerance() {
        let commodity = commodity!(RGAGX);
        let amount = AmountWithTolerance::with_tolerance(dec!(319.020), dec!(0.002), commodity);

        let mut output = String::new();
        let result = marshal_amount_with_tolerance(&amount, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "319.020 ~ 0.002 RGAGX");
    }

    #[test]
    fn marshal_amount_with_tolerance_negative() {
        let commodity = commodity!(EUR);
        let amount = AmountWithTolerance::without_tolerance(dec!(-3492.02), commodity);

        let mut output = String::new();
        let result = marshal_amount_with_tolerance(&amount, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "-3492.02 EUR");
    }

    #[test]
    fn marshal_amount_with_tolerance_zero() {
        let commodity = commodity!(BTC);
        let amount = AmountWithTolerance::without_tolerance(dec!(0), commodity);

        let mut output = String::new();
        let result = marshal_amount_with_tolerance(&amount, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "0 BTC");
    }

    #[test]
    fn test_from_and_to_amount_conversion() {
        let commodity = commodity!(USD);
        let basic_amount = crate::model::Amount::new(dec!(123.45), commodity);

        // Convert to AmountWithTolerance
        let amount_with_tolerance = AmountWithTolerance::from_amount(basic_amount.clone());

        // Convert back to Amount
        let converted_back = amount_with_tolerance.amount();

        assert_eq!(basic_amount, *converted_back);
    }
}
