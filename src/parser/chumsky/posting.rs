use chumsky::{prelude::*, text::whitespace};
use std::fmt::Write;

use crate::{
    model::Posting,
    parser::chumsky::{
        account::{marshal_account, parse_account},
        flag::{marshal_flag, parse_flag},
        posting_amount::{marshal_posting_amount, parse_posting_amount},
    },
};

/// Parser for posting line
/// Syntax: <whitespace> [<flag>] <account> [<amount> [{<cost>}] [@ <price>]]
pub fn parse_posting<'a>() -> impl Parser<'a, &'a str, Posting<'a, 'a>, extra::Err<Rich<'a, char>>>
{
    whitespace()
        .at_least(1)
        .ignore_then(parse_flag().then_ignore(whitespace().at_least(1)).or_not())
        .then(parse_account())
        .then(
            whitespace()
                .at_least(1)
                .ignore_then(parse_posting_amount())
                .or_not(),
        )
        .map(|((flag, account), posting_amount)| {
            let mut posting = match posting_amount {
                Some(amount) => Posting::new(account, amount),
                None => Posting::new_without_amount(account),
            };

            if let Some(f) = flag {
                posting = posting.with_flag(f);
            }

            posting
        })
}

pub fn marshal_posting(posting: &Posting, writer: &mut impl Write) -> std::fmt::Result {
    write!(writer, "  ")?;

    // Write flag if present
    if let Some(flag) = posting.flag() {
        marshal_flag(flag, writer)?;
        write!(writer, " ")?;
    }

    marshal_account(posting.account().clone(), writer)?;

    if let Some(posting_amount) = posting.amount() {
        write!(writer, "  ")?;
        marshal_posting_amount(posting_amount, writer)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Amount, Flag, account, commodity};
    use rstest::rstest;
    use rstest_reuse::*;
    use rust_decimal_macros::dec;

    #[template]
    #[rstest]
    #[case("  Assets:Checking  100.50 USD")]
    #[case("    Liabilities:CreditCard  -37.45 USD")]
    #[case("  Expenses:Restaurant")]
    #[case("  Income:Salary  -5000.00 USD")]
    #[case("   Assets:Cash   0 USD")]
    #[case("\t\tAssets:Investment\t\t1000.00 EUR")]
    #[case("  Equity:Opening-Balances")]
    #[case("  * Assets:Checking  100.50 USD")]
    #[case("  ! Liabilities:CreditCard  -37.45 USD")]
    #[case("  Assets:Investment  10 STOCK {50.00 USD}")]
    #[case("  Assets:Investment  10 STOCK @ 55.00 USD")]
    #[case("  Assets:Investment  10 STOCK {50.00 USD} @ 55.00 USD")]
    #[case("  * Assets:Investment  10 STOCK { 50.00 USD } @ 55.00 USD")]
    fn valid_posting_template(#[case] input: &str) {}

    #[apply(valid_posting_template)]
    fn parse_valid_posting(#[case] input: &str) {
        let result = parse_posting().parse(input);
        assert!(result.has_output(), "Failed to parse posting: {}", input);
        let _parsed = result.into_result().unwrap();
    }

    #[apply(valid_posting_template)]
    fn marshal_and_parse_posting(#[case] input: &str) {
        // Parse the original
        let result = parse_posting().parse(input);
        assert!(result.has_output());
        let original = result.into_result().unwrap();

        // Marshal it
        let mut marshalled = String::new();
        let marshal_result = marshal_posting(&original, &mut marshalled);
        assert!(marshal_result.is_ok());

        // Parse it back
        let reparse_result = parse_posting().parse(&marshalled);
        assert!(reparse_result.has_output());
        let reparsed = reparse_result.into_result().unwrap();

        // Should be equal
        assert_eq!(original, reparsed);
    }

    #[test]
    fn parse_posting_with_amount() {
        let input = "  Assets:Checking  100.50 USD";
        let result = parse_posting().parse(input);
        assert!(result.has_output());
        let posting = result.into_result().unwrap();

        let components: Vec<&str> = posting.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Checking"]);
        assert!(posting.has_amount());
        assert_eq!(*posting.amount().unwrap().amount().number(), dec!(100.50));
        assert_eq!(
            posting.amount().unwrap().amount().commodity().as_ref(),
            "USD"
        );
    }

    #[test]
    fn parse_posting_without_amount() {
        let input = "  Expenses:Restaurant";
        let result = parse_posting().parse(input);
        assert!(result.has_output());
        let posting = result.into_result().unwrap();

        let components: Vec<&str> = posting.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Restaurant"]);
        assert!(!posting.has_amount());
        assert_eq!(posting.amount(), None);
    }

    #[test]
    fn parse_posting_negative_amount() {
        let input = "  Liabilities:CreditCard  -37.45 USD";
        let result = parse_posting().parse(input);
        assert!(result.has_output());
        let posting = result.into_result().unwrap();

        let components: Vec<&str> = posting.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["CreditCard"]);
        assert!(posting.has_amount());
        assert_eq!(*posting.amount().unwrap().amount().number(), dec!(-37.45));
    }

    #[test]
    fn parse_posting_zero_amount() {
        let input = "  Assets:Cash  0 USD";
        let result = parse_posting().parse(input);
        assert!(result.has_output());
        let posting = result.into_result().unwrap();

        assert!(posting.has_amount());
        assert_eq!(*posting.amount().unwrap().amount().number(), dec!(0));
    }

    #[test]
    fn parse_posting_tabs() {
        let input = "\t\tAssets:Investment\t\t1000.00 EUR";
        let result = parse_posting().parse(input);
        assert!(result.has_output());
        let posting = result.into_result().unwrap();

        let components: Vec<&str> = posting.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Investment"]);
        assert!(posting.has_amount());
        assert_eq!(*posting.amount().unwrap().amount().number(), dec!(1000.00));
    }

    #[rstest]
    #[case("Assets:Checking 100.50 USD")] // Missing leading whitespace
    #[case(" Assets:checking 100.50 USD")] // Invalid account
    #[case("  assets:checking 100.50 USD")] // Invalid account
    #[case("  Assets:Checking abc USD")] // Invalid amount
    #[case("  Assets:Checking 100.50 usd")] // Invalid commodity
    #[case("")] // Empty input
    #[case("  ")] // Only whitespace
    fn parse_posting_invalid(#[case] input: &str) {
        let result = parse_posting().parse(input);
        assert!(!result.has_output(), "Should fail to parse: {}", input);
    }

    #[test]
    fn marshal_posting_with_amount() {
        let account = account!(Assets:Checking);
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(100.50), commodity);
        let posting_amount = crate::model::PostingAmount::new(amount);
        let posting = Posting::new(account, posting_amount);

        let mut output = String::new();
        let result = marshal_posting(&posting, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "  Assets:Checking  100.50 USD");
    }

    #[test]
    fn marshal_posting_without_amount() {
        let account = account!(Expenses:Restaurant);
        let posting = Posting::new_without_amount(account);

        let mut output = String::new();
        let result = marshal_posting(&posting, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "  Expenses:Restaurant");
    }

    #[test]
    fn marshal_posting_negative_amount() {
        let account = account!(Liabilities:CreditCard);
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(-37.45), commodity);
        let posting_amount = crate::model::PostingAmount::new(amount);
        let posting = Posting::new(account, posting_amount);

        let mut output = String::new();
        let result = marshal_posting(&posting, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "  Liabilities:CreditCard  -37.45 USD");
    }

    #[test]
    fn marshal_posting_zero_amount() {
        let account = account!(Assets:Cash);
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(0), commodity);
        let posting_amount = crate::model::PostingAmount::new(amount);
        let posting = Posting::new(account, posting_amount);

        let mut output = String::new();
        let result = marshal_posting(&posting, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "  Assets:Cash  0 USD");
    }

    #[test]
    fn parse_posting_with_flag() {
        let input = "  * Assets:Checking  100.50 USD";
        let result = parse_posting().parse(input);
        assert!(result.has_output());
        let posting = result.into_result().unwrap();

        let components: Vec<&str> = posting.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Checking"]);
        assert!(posting.has_flag());
        assert_eq!(posting.flag(), Some(Flag::Complete));
        assert!(posting.has_amount());
    }

    #[test]
    fn parse_posting_with_cost() {
        let input = "  Assets:Investment  10 STOCK {50.00 USD}";
        let result = parse_posting().parse(input);
        assert!(result.has_output());
        let posting = result.into_result().unwrap();

        let components: Vec<&str> = posting.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Investment"]);
        assert!(posting.has_amount());
        let posting_amount = posting.amount().unwrap();
        assert_eq!(*posting_amount.amount().number(), dec!(10));
        assert_eq!(posting_amount.amount().commodity().as_ref(), "STOCK");
        assert!(posting_amount.has_cost());
        assert_eq!(*posting_amount.cost().unwrap().number(), dec!(50.00));
        assert_eq!(posting_amount.cost().unwrap().commodity().as_ref(), "USD");
    }

    #[test]
    fn parse_posting_with_price() {
        let input = "  Assets:Investment  10 STOCK @ 55.00 USD";
        let result = parse_posting().parse(input);
        assert!(result.has_output());
        let posting = result.into_result().unwrap();

        let components: Vec<&str> = posting.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Investment"]);
        assert!(posting.has_amount());
        let posting_amount = posting.amount().unwrap();
        assert_eq!(*posting_amount.amount().number(), dec!(10));
        assert_eq!(posting_amount.amount().commodity().as_ref(), "STOCK");
        assert!(posting_amount.has_price());
        assert_eq!(*posting_amount.price().unwrap().number(), dec!(55.00));
        assert_eq!(posting_amount.price().unwrap().commodity().as_ref(), "USD");
    }

    #[test]
    fn parse_posting_with_cost_and_price() {
        let input = "  Assets:Investment  10 STOCK {50.00 USD} @ 55.00 USD";
        let result = parse_posting().parse(input);
        assert!(result.has_output());
        let posting = result.into_result().unwrap();

        let components: Vec<&str> = posting.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Investment"]);
        assert!(posting.has_amount());
        let posting_amount = posting.amount().unwrap();
        assert_eq!(*posting_amount.amount().number(), dec!(10));
        assert_eq!(posting_amount.amount().commodity().as_ref(), "STOCK");
        assert!(posting_amount.has_cost());
        assert_eq!(*posting_amount.cost().unwrap().number(), dec!(50.00));
        assert_eq!(posting_amount.cost().unwrap().commodity().as_ref(), "USD");
        assert!(posting_amount.has_price());
        assert_eq!(*posting_amount.price().unwrap().number(), dec!(55.00));
        assert_eq!(posting_amount.price().unwrap().commodity().as_ref(), "USD");
    }

    #[test]
    fn marshal_posting_with_flag() {
        let account = account!(Assets:Checking);
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(100.50), commodity);
        let posting_amount = crate::model::PostingAmount::new(amount);
        let posting = Posting::new(account, posting_amount).with_flag(Flag::Complete);

        let mut output = String::new();
        let result = marshal_posting(&posting, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "  * Assets:Checking  100.50 USD");
    }

    #[test]
    fn marshal_posting_with_cost() {
        let account = account!(Assets:Investment);
        let stock = commodity!(STOCK);
        let usd = commodity!(USD);
        let amount = Amount::new(dec!(10), stock);
        let cost = Amount::new(dec!(50.00), usd);
        let posting_amount = crate::model::PostingAmount::new(amount).with_cost(cost);
        let posting = Posting::new(account, posting_amount);

        let mut output = String::new();
        let result = marshal_posting(&posting, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "  Assets:Investment  10 STOCK {50.00 USD}");
    }

    #[test]
    fn marshal_posting_with_price() {
        let account = account!(Assets:Investment);
        let stock = commodity!(STOCK);
        let usd = commodity!(USD);
        let amount = Amount::new(dec!(10), stock);
        let price = Amount::new(dec!(55.00), usd);
        let posting_amount = crate::model::PostingAmount::new(amount).with_price(price);
        let posting = Posting::new(account, posting_amount);

        let mut output = String::new();
        let result = marshal_posting(&posting, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "  Assets:Investment  10 STOCK @ 55.00 USD");
    }
}
