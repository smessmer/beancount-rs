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
-> impl Parser<'a, &'a str, DirectiveBalance<'a>, extra::Err<Rich<'a, char>>> {
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
    use crate::model::{AccountType, account, commodity};
    use rstest::rstest;
    use rstest_reuse::*;
    use rust_decimal_macros::dec;

    #[template]
    #[rstest]
    #[case("balance Assets:Checking 1000.50 USD", AccountType::Assets, vec!["Checking"], dec!(1000.50), "USD", None)]
    #[case("balance Liabilities:CreditCard -3492.02 USD", AccountType::Liabilities, vec!["CreditCard"], dec!(-3492.02), "USD", None)]
    #[case("balance Assets:Investment 319.020 ~ 0.002 RGAGX", AccountType::Assets, vec!["Investment"], dec!(319.020), "RGAGX", Some(dec!(0.002)))]
    #[case("balance Assets:Cash 0 USD", AccountType::Assets, vec!["Cash"], dec!(0), "USD", None)]
    #[case("balance Assets:Crypto 1.5 BTC", AccountType::Assets, vec!["Crypto"], dec!(1.5), "BTC", None)]
    #[case("balance Expenses:Food 500.00 USD", AccountType::Expenses, vec!["Food"], dec!(500.00), "USD", None)]
    #[case("balance Income:Salary -5000.00 USD", AccountType::Income, vec!["Salary"], dec!(-5000.00), "USD", None)]
    #[case("balance Assets:Savings +2000.75 EUR", AccountType::Assets, vec!["Savings"], dec!(2000.75), "EUR", None)]
    fn valid_balance_directive_template(
        #[case] input: &str,
        #[case] expected_account_type: AccountType,
        #[case] expected_account_components: Vec<&str>,
        #[case] expected_number: rust_decimal::Decimal,
        #[case] expected_commodity: &str,
        #[case] expected_tolerance: Option<rust_decimal::Decimal>,
    ) {
    }

    #[apply(valid_balance_directive_template)]
    fn parse_balance_directive_valid(
        #[case] input: &str,
        #[case] expected_account_type: AccountType,
        #[case] expected_account_components: Vec<&str>,
        #[case] expected_number: rust_decimal::Decimal,
        #[case] expected_commodity: &str,
        #[case] expected_tolerance: Option<rust_decimal::Decimal>,
    ) {
        let result = parse_balance_directive().parse(input);
        assert!(
            result.has_output(),
            "Failed to parse balance directive: {}",
            input
        );
        let parsed = result.into_result().unwrap();

        // Validate account
        assert_eq!(parsed.account().account_type(), expected_account_type);
        let components: Vec<&str> = parsed.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, expected_account_components);

        // Validate amount
        assert_eq!(*parsed.amount_with_tolerance().number(), expected_number);
        assert_eq!(
            parsed.amount_with_tolerance().commodity().as_ref(),
            expected_commodity
        );

        // Validate tolerance
        assert_eq!(
            parsed.amount_with_tolerance().tolerance().map(|t| *t),
            expected_tolerance
        );
    }

    #[apply(valid_balance_directive_template)]
    fn marshal_and_parse_balance_directive(
        #[case] input: &str,
        #[case] _expected_account_type: AccountType,
        #[case] _expected_account_components: Vec<&str>,
        #[case] _expected_number: rust_decimal::Decimal,
        #[case] _expected_commodity: &str,
        #[case] _expected_tolerance: Option<rust_decimal::Decimal>,
    ) {
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
