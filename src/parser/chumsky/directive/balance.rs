use chumsky::{
    prelude::*,
    text::{keyword, whitespace},
};
use std::fmt::Write;

use crate::{
    model::DirectiveBalance,
    parser::chumsky::{
        account::{marshal_account, parse_account},
        amount_with_tolerance::{marshal_amount_with_tolerance, parse_amount_with_tolerance},
    },
};

const KEYWORD_BALANCE: &str = "balance";

/// Parser for balance directive (without date)
/// Syntax: "balance" <account> <number> [~ <tolerance>] <commodity>
pub fn parse_balance_directive<'a>()
-> impl Parser<'a, &'a str, DirectiveBalance<'a, 'a>, extra::Err<Rich<'a, char>>> {
    keyword(KEYWORD_BALANCE)
        .then_ignore(whitespace().at_least(1))
        .ignore_then(parse_account())
        .then_ignore(whitespace().at_least(1))
        .then(parse_amount_with_tolerance())
        .map(|(account, amount_with_tolerance)| {
            DirectiveBalance::new(account, amount_with_tolerance)
        })
}

/// Marshaller for balance directive (without date)
pub fn marshal_balance_directive(
    directive: &DirectiveBalance,
    writer: &mut impl Write,
) -> std::fmt::Result {
    write!(writer, "{KEYWORD_BALANCE} ")?;
    marshal_account(directive.account().clone(), writer)?;
    write!(writer, " ")?;
    marshal_amount_with_tolerance(directive.amount_with_tolerance(), writer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{account, commodity};
    use rstest::rstest;
    use rstest_reuse::*;
    use rust_decimal_macros::dec;

    #[template]
    #[rstest]
    #[case("balance Assets:Checking 1000.50 USD")]
    #[case("balance Liabilities:CreditCard -3492.02 USD")]
    #[case("balance Assets:Investment 319.020 ~ 0.002 RGAGX")]
    #[case("balance Assets:Cash 0 USD")]
    #[case("balance Assets:Crypto 1.5 BTC")]
    #[case("balance Expenses:Food 500.00 USD")]
    #[case("balance Income:Salary -5000.00 USD")]
    #[case("balance Assets:Savings +2000.75 EUR")]
    fn valid_balance_directive_template(#[case] input: &str) {}

    #[apply(valid_balance_directive_template)]
    fn parse_balance_directive_valid(#[case] input: &str) {
        let result = parse_balance_directive().parse(input);
        assert!(
            result.has_output(),
            "Failed to parse balance directive: {}",
            input
        );
        let _parsed = result.into_result().unwrap();
    }

    #[apply(valid_balance_directive_template)]
    fn marshal_and_parse_balance_directive(#[case] input: &str) {
        // First parse the original
        let result = parse_balance_directive().parse(input);
        assert!(result.has_output());
        let original = result.into_result().unwrap();

        // Marshal it
        let mut marshalled = String::new();
        let marshal_result = marshal_balance_directive(&original, &mut marshalled);
        assert!(marshal_result.is_ok());

        // Parse it back
        let reparse_result = parse_balance_directive().parse(&marshalled);
        assert!(reparse_result.has_output());
        let reparsed = reparse_result.into_result().unwrap();

        // Should be equal
        assert_eq!(original, reparsed);
    }

    #[test]
    fn parse_balance_directive_basic() {
        let input = "balance Assets:Checking 1000.50 USD";
        let result = parse_balance_directive().parse(input);
        assert!(result.has_output());
        let balance = result.into_result().unwrap();

        let components: Vec<&str> = balance.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Checking"]);
        assert_eq!(*balance.amount_with_tolerance().number(), dec!(1000.50));
        assert_eq!(balance.amount_with_tolerance().commodity().as_ref(), "USD");
        assert_eq!(balance.amount_with_tolerance().tolerance(), None);
    }

    #[test]
    fn parse_balance_directive_negative() {
        let input = "balance Liabilities:CreditCard -3492.02 USD";
        let result = parse_balance_directive().parse(input);
        assert!(result.has_output());
        let balance = result.into_result().unwrap();

        let components: Vec<&str> = balance.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["CreditCard"]);
        assert_eq!(*balance.amount_with_tolerance().number(), dec!(-3492.02));
        assert_eq!(balance.amount_with_tolerance().commodity().as_ref(), "USD");
        assert_eq!(balance.amount_with_tolerance().tolerance(), None);
    }

    #[test]
    fn parse_balance_directive_with_tolerance() {
        let input = "balance Assets:Investment 319.020 ~ 0.002 RGAGX";
        let result = parse_balance_directive().parse(input);
        assert!(result.has_output());
        let balance = result.into_result().unwrap();

        let components: Vec<&str> = balance.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Investment"]);
        assert_eq!(*balance.amount_with_tolerance().number(), dec!(319.020));
        assert_eq!(
            balance.amount_with_tolerance().commodity().as_ref(),
            "RGAGX"
        );
        assert_eq!(
            balance.amount_with_tolerance().tolerance(),
            Some(&dec!(0.002))
        );
    }

    #[test]
    fn parse_balance_directive_zero_balance() {
        let input = "balance Assets:Checking 0 USD";
        let result = parse_balance_directive().parse(input);
        assert!(result.has_output());
        let balance = result.into_result().unwrap();

        assert_eq!(*balance.amount_with_tolerance().number(), dec!(0));
        assert_eq!(balance.amount_with_tolerance().commodity().as_ref(), "USD");
        assert_eq!(balance.amount_with_tolerance().tolerance(), None);
    }

    #[test]
    fn parse_balance_directive_positive_sign() {
        let input = "balance Assets:Savings +2000.75 EUR";
        let result = parse_balance_directive().parse(input);
        assert!(result.has_output());
        let balance = result.into_result().unwrap();

        assert_eq!(*balance.amount_with_tolerance().number(), dec!(2000.75));
        assert_eq!(balance.amount_with_tolerance().commodity().as_ref(), "EUR");
        assert_eq!(balance.amount_with_tolerance().tolerance(), None);
    }

    #[rstest]
    #[case("balance")] // Missing account and amount
    #[case("balance Assets:Cash")] // Missing amount
    #[case("balance 1000.50 USD")] // Missing account
    #[case("balance Assets:Cash USD 1000.50")] // Wrong order
    #[case("balance assets:cash 1000.50 USD")] // Invalid account
    #[case("balance Assets:Cash abc USD")] // Invalid amount
    #[case("balance Assets:Cash 1000.50 usd")] // Invalid commodity
    #[case("balance Assets:Cash 1000.50 USD ~")] // Missing tolerance value
    #[case("balance Assets:Cash 1000.50 USD ~ abc")] // Invalid tolerance
    fn parse_balance_directive_invalid(#[case] input: &str) {
        let result = parse_balance_directive().parse(input);
        assert!(!result.has_output(), "Should fail to parse: {}", input);
    }

    #[test]
    fn marshal_balance_directive_basic() {
        let account = account!(Assets:Checking);
        let commodity = commodity!(USD);
        let amount_with_tolerance =
            crate::model::AmountWithTolerance::without_tolerance(dec!(1000.50), commodity);
        let balance = DirectiveBalance::new(account, amount_with_tolerance);

        let mut output = String::new();
        let result = marshal_balance_directive(&balance, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "balance Assets:Checking 1000.50 USD");
    }

    #[test]
    fn marshal_balance_directive_negative() {
        let account = account!(Liabilities:CreditCard);
        let commodity = commodity!(USD);
        let amount_with_tolerance =
            crate::model::AmountWithTolerance::without_tolerance(dec!(-3492.02), commodity);
        let balance = DirectiveBalance::new(account, amount_with_tolerance);

        let mut output = String::new();
        let result = marshal_balance_directive(&balance, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "balance Liabilities:CreditCard -3492.02 USD");
    }

    #[test]
    fn marshal_balance_directive_with_tolerance() {
        let account = account!(Assets:Investment);
        let commodity = commodity!(RGAGX);
        let amount_with_tolerance = crate::model::AmountWithTolerance::with_tolerance(
            dec!(319.020),
            dec!(0.002),
            commodity,
        );
        let balance = DirectiveBalance::new(account, amount_with_tolerance);

        let mut output = String::new();
        let result = marshal_balance_directive(&balance, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "balance Assets:Investment 319.020 ~ 0.002 RGAGX");
    }

    #[test]
    fn marshal_balance_directive_zero() {
        let account = account!(Assets:Cash);
        let commodity = commodity!(USD);
        let amount_with_tolerance =
            crate::model::AmountWithTolerance::without_tolerance(dec!(0), commodity);
        let balance = DirectiveBalance::new(account, amount_with_tolerance);

        let mut output = String::new();
        let result = marshal_balance_directive(&balance, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "balance Assets:Cash 0 USD");
    }
}
