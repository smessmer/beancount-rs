use chumsky::{prelude::*, text::whitespace};
use std::fmt::Write;

use crate::{
    model::directive::Posting,
    parser::chumsky::{
        account::{marshal_account, parse_account},
        directive::transaction::posting_amount::{marshal_posting_amount, parse_posting_amount},
        flag::{marshal_flag, parse_flag},
    },
};

/// Parser for posting line
/// Syntax: <whitespace> [<flag>] <account> [<amount> [{<cost>}] [@ <price>]]
pub fn parse_posting<'a>() -> impl Parser<'a, &'a str, Posting<'a>, extra::Err<Rich<'a, char>>> {
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
    use crate::model::{AccountType, Amount, Flag, account, commodity, directive::PostingAmount};
    use rstest::rstest;
    use rstest_reuse::*;
    use rust_decimal_macros::dec;

    #[template]
    #[rstest]
    #[case("  Assets:Checking  100.50 USD", None, AccountType::Assets, vec!["Checking"], Some((dec!(100.50), "USD", None, None)))]
    #[case("    Liabilities:CreditCard  -37.45 USD", None, AccountType::Liabilities, vec!["CreditCard"], Some((dec!(-37.45), "USD", None, None)))]
    #[case("  Expenses:Restaurant", None, AccountType::Expenses, vec!["Restaurant"], None)]
    #[case("  Income:Salary  -5000.00 USD", None, AccountType::Income, vec!["Salary"], Some((dec!(-5000.00), "USD", None, None)))]
    #[case("   Assets:Cash   0 USD", None, AccountType::Assets, vec!["Cash"], Some((dec!(0), "USD", None, None)))]
    #[case("\t\tAssets:Investment\t\t1000.00 EUR", None, AccountType::Assets, vec!["Investment"], Some((dec!(1000.00), "EUR", None, None)))]
    #[case("  Equity:Opening-Balances", None, AccountType::Equity, vec!["Opening-Balances"], None)]
    #[case("  * Assets:Checking  100.50 USD", Some(Flag::Complete), AccountType::Assets, vec!["Checking"], Some((dec!(100.50), "USD", None, None)))]
    #[case("  ! Liabilities:CreditCard  -37.45 USD", Some(Flag::Incomplete), AccountType::Liabilities, vec!["CreditCard"], Some((dec!(-37.45), "USD", None, None)))]
    #[case("  Assets:Investment  10 STOCK {50.00 USD}", None, AccountType::Assets, vec!["Investment"], Some((dec!(10), "STOCK", Some((dec!(50.00), "USD")), None)))]
    #[case("  Assets:Investment  10 STOCK @ 55.00 USD", None, AccountType::Assets, vec!["Investment"], Some((dec!(10), "STOCK", None, Some((dec!(55.00), "USD")))))]
    #[case("  Assets:Investment  10 STOCK {50.00 USD} @ 55.00 USD", None, AccountType::Assets, vec!["Investment"], Some((dec!(10), "STOCK", Some((dec!(50.00), "USD")), Some((dec!(55.00), "USD")))))]
    #[case("  * Assets:Investment  10 STOCK { 50.00 USD } @ 55.00 USD", Some(Flag::Complete), AccountType::Assets, vec!["Investment"], Some((dec!(10), "STOCK", Some((dec!(50.00), "USD")), Some((dec!(55.00), "USD")))))]
    fn valid_posting_template(
        #[case] input: &str,
        #[case] expected_flag: Option<Flag>,
        #[case] expected_account_type: AccountType,
        #[case] expected_account_components: Vec<&str>,
        #[case] expected_amount: Option<(
            rust_decimal::Decimal,
            &str,
            Option<(rust_decimal::Decimal, &str)>,
            Option<(rust_decimal::Decimal, &str)>,
        )>,
    ) {
    }

    #[apply(valid_posting_template)]
    fn parse_valid_posting(
        #[case] input: &str,
        #[case] expected_flag: Option<Flag>,
        #[case] expected_account_type: AccountType,
        #[case] expected_account_components: Vec<&str>,
        #[case] expected_amount: Option<(
            rust_decimal::Decimal,
            &str,
            Option<(rust_decimal::Decimal, &str)>,
            Option<(rust_decimal::Decimal, &str)>,
        )>,
    ) {
        let result = parse_posting().parse(input);
        assert!(result.has_output(), "Failed to parse posting: {}", input);
        let parsed = result.into_result().unwrap();

        // Validate flag
        assert_eq!(parsed.flag(), expected_flag);

        // Validate account
        assert_eq!(parsed.account().account_type(), expected_account_type);
        let components: Vec<&str> = parsed.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, expected_account_components);

        // Validate amount
        match expected_amount {
            Some((exp_number, exp_commodity, exp_cost, exp_price)) => {
                assert!(parsed.has_amount());
                let posting_amount = parsed.amount().unwrap();
                assert_eq!(*posting_amount.amount().number(), exp_number);
                assert_eq!(posting_amount.amount().commodity().as_ref(), exp_commodity);

                // Validate cost
                if let Some((cost_number, cost_commodity)) = exp_cost {
                    assert!(posting_amount.has_cost());
                    let cost = posting_amount.cost().unwrap();
                    assert_eq!(*cost.number(), cost_number);
                    assert_eq!(cost.commodity().as_ref(), cost_commodity);
                } else {
                    assert!(!posting_amount.has_cost());
                }

                // Validate price
                if let Some((price_number, price_commodity)) = exp_price {
                    assert!(posting_amount.has_price());
                    let price = posting_amount.price().unwrap();
                    assert_eq!(*price.number(), price_number);
                    assert_eq!(price.commodity().as_ref(), price_commodity);
                } else {
                    assert!(!posting_amount.has_price());
                }
            }
            None => {
                assert!(!parsed.has_amount());
            }
        }
    }

    #[apply(valid_posting_template)]
    fn marshal_and_parse_posting(
        #[case] input: &str,
        #[case] _expected_flag: Option<Flag>,
        #[case] _expected_account_type: AccountType,
        #[case] _expected_account_components: Vec<&str>,
        #[case] _expected_amount: Option<(
            rust_decimal::Decimal,
            &str,
            Option<(rust_decimal::Decimal, &str)>,
            Option<(rust_decimal::Decimal, &str)>,
        )>,
    ) {
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
        let posting_amount = PostingAmount::new(amount);
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
        let posting_amount = PostingAmount::new(amount);
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
        let posting_amount = PostingAmount::new(amount);
        let posting = Posting::new(account, posting_amount);

        let mut output = String::new();
        let result = marshal_posting(&posting, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "  Assets:Cash  0 USD");
    }

    #[test]
    fn marshal_posting_with_flag() {
        let account = account!(Assets:Checking);
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(100.50), commodity);
        let posting_amount = PostingAmount::new(amount);
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
        let posting_amount = PostingAmount::new(amount).with_cost(cost);
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
        let posting_amount = PostingAmount::new(amount).with_price(price);
        let posting = Posting::new(account, posting_amount);

        let mut output = String::new();
        let result = marshal_posting(&posting, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "  Assets:Investment  10 STOCK @ 55.00 USD");
    }
}
